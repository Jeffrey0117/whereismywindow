#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod focus;
mod hotkey;
mod monitor;
mod overlay;
mod tray;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
use windows::Win32::UI::WindowsAndMessaging::*;

use app::{App, FocusState};
use config::Config;
use focus::tracker::{self, WM_FOCUS_CHANGED, WM_LOCATION_CHANGED};
use focus::window_info;
use monitor::{enumeration, geometry};
use overlay::border::BorderOverlay;
use overlay::flash::FlashOverlay;
use overlay::indicator::MonitorIndicators;
use tray::icon::{
    self as tray_icon_mod, SystemTray, MENU_BORDER_STYLE, MENU_QUIT, MENU_TOGGLE_BORDER,
    MENU_TOGGLE_FLASH, MENU_TOGGLE_INDICATOR,
};

const TIMER_POLL: usize = 1;
const TIMER_FLASH_HIDE: usize = 2;
const TIMER_HOTKEY_CHECK: usize = 3;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();
    log::info!("whereismywindow starting");

    let config = Config::default();

    // Enumerate monitors
    let monitors = enumeration::enumerate_monitors();
    for (i, m) in monitors.iter().enumerate() {
        log::info!("{}", enumeration::format_monitor(m, i));
    }

    let mut app = App::new(config.clone());
    app.monitors = monitors;

    // Create overlays
    let mut border_overlay = BorderOverlay::new(config.border_color, config.border_thickness, config.border_style);
    let flash_overlay = FlashOverlay::new(config.flash_opacity);

    // Create monitor indicators (bottom-left corner badges)
    // Use work_rect (excludes taskbar) so badges aren't hidden behind the taskbar
    let monitor_rects: Vec<_> = app.monitors.iter().map(|m| m.work_rect).collect();
    let mut indicators = MonitorIndicators::new(&monitor_rects);

    if border_overlay.is_none() {
        log::warn!("Failed to create border overlay");
    }
    if flash_overlay.is_none() {
        log::warn!("Failed to create flash overlay");
    }
    if indicators.is_none() {
        log::warn!("Failed to create monitor indicators");
    } else {
        log::info!("Monitor indicators created for {} monitors", monitor_rects.len());
    }

    // Create system tray
    let tray = SystemTray::new();
    if tray.is_none() {
        log::warn!("Failed to create system tray icon");
    }

    // Create hotkey handler
    let hotkey_handler = hotkey::reveal::RevealHotkey::new();
    if hotkey_handler.is_none() {
        log::warn!("Failed to register global hotkey");
    }

    // Create message-only window for receiving events
    let msg_hwnd = create_msg_window();
    if msg_hwnd.0.is_null() {
        log::error!("Failed to create message window");
        return;
    }
    tracker::set_msg_hwnd(msg_hwnd);

    // Install event hooks
    let (focus_hook, location_hook) = tracker::install_hooks();
    log::info!("Event hooks installed");

    // Set up a poll timer for position tracking (~60fps)
    unsafe {
        SetTimer(Some(msg_hwnd), TIMER_POLL, config.poll_interval_ms, None);
        SetTimer(Some(msg_hwnd), TIMER_HOTKEY_CHECK, 50, None);
    }

    // Do an initial focus check
    update_focus_state(&mut app, &mut border_overlay, &flash_overlay, &mut indicators);

    // Message loop
    log::info!("Entering message loop");
    let mut msg = MSG::default();
    loop {
        unsafe {
            let ret = GetMessageW(&mut msg, None, 0, 0);
            if ret.0 <= 0 {
                break;
            }

            match msg.message {
                WM_FOCUS_CHANGED => {
                    update_focus_state(
                        &mut app,
                        &mut border_overlay,
                        &flash_overlay,
                        &mut indicators,
                    );
                }
                WM_LOCATION_CHANGED => {
                    if app.config.border_enabled {
                        if let Some(ref focus) = app.focus {
                            let fg = GetForegroundWindow();
                            if fg.0 as isize == focus.hwnd {
                                if let Some(new_rect) = window_info::get_extended_frame_bounds(
                                    HWND(focus.hwnd as *mut _),
                                ) {
                                    let clamped = clamp_to_monitor(&new_rect, &focus.monitor_rect);
                                    if let Some(ref mut bo) = border_overlay {
                                        bo.update(&clamped);
                                    }
                                }
                            }
                        }
                    }
                }
                WM_TIMER => {
                    let timer_id = msg.wParam.0;
                    match timer_id {
                        TIMER_POLL => {
                            if app.config.border_enabled {
                                if let Some(ref focus) = app.focus {
                                    let fg = GetForegroundWindow();
                                    if fg.0 as isize == focus.hwnd {
                                        if let Some(new_rect) = window_info::get_extended_frame_bounds(
                                            HWND(focus.hwnd as *mut _),
                                        ) {
                                            let clamped = clamp_to_monitor(&new_rect, &focus.monitor_rect);
                                            if clamped != focus.window_rect {
                                                if let Some(ref mut bo) = border_overlay {
                                                    bo.update(&clamped);
                                                }
                                            }
                                        }
                                    } else {
                                        // Foreground changed away from tracked window —
                                        // hide border until next WM_FOCUS_CHANGED updates it
                                        if let Some(ref bo) = border_overlay {
                                            bo.hide();
                                        }
                                    }
                                }
                            }
                        }
                        TIMER_FLASH_HIDE => {
                            if let Some(ref fo) = flash_overlay {
                                fo.hide();
                            }
                            KillTimer(Some(msg_hwnd), TIMER_FLASH_HIDE).ok();
                        }
                        TIMER_HOTKEY_CHECK => {
                            if let Some(ref _hk) = hotkey_handler {
                                if _hk.poll() {
                                    log::info!("Hotkey reveal triggered");
                                    show_reveal_info(&app);
                                }
                            }

                            let ctrl_down = GetAsyncKeyState(0x11) < 0;
                            let shift_down = GetAsyncKeyState(0x10) < 0;
                            let f_down = GetAsyncKeyState(0x46) < 0;
                            if let Some(ref _hk) = hotkey_handler {
                                if _hk.is_active && !(ctrl_down && shift_down && f_down) {
                                    log::info!("Hotkey reveal released");
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }

            // Check tray menu events
            if let Some(id) = tray_icon_mod::poll_menu_event() {
                match id.as_str() {
                    MENU_TOGGLE_BORDER => {
                        app.config.border_enabled = !app.config.border_enabled;
                        log::info!(
                            "Border: {}",
                            if app.config.border_enabled { "ON" } else { "OFF" }
                        );
                        if let Some(ref t) = tray {
                            t.update_border_text(app.config.border_enabled);
                        }
                        if !app.config.border_enabled {
                            if let Some(ref bo) = border_overlay {
                                bo.hide();
                            }
                        } else {
                            update_focus_state(
                                &mut app,
                                &mut border_overlay,
                                &flash_overlay,
                                &mut indicators,
                            );
                        }
                    }
                    MENU_TOGGLE_FLASH => {
                        app.config.flash_enabled = !app.config.flash_enabled;
                        log::info!(
                            "Flash: {}",
                            if app.config.flash_enabled { "ON" } else { "OFF" }
                        );
                        if let Some(ref t) = tray {
                            t.update_flash_text(app.config.flash_enabled);
                        }
                    }
                    MENU_TOGGLE_INDICATOR => {
                        app.config.indicator_enabled = !app.config.indicator_enabled;
                        log::info!(
                            "Indicator: {}",
                            if app.config.indicator_enabled { "ON" } else { "OFF" }
                        );
                        if let Some(ref t) = tray {
                            t.update_indicator_text(app.config.indicator_enabled);
                        }
                        if let Some(ref ind) = indicators {
                            if app.config.indicator_enabled {
                                ind.show_all();
                            } else {
                                ind.hide_all();
                            }
                        }
                    }
                    MENU_BORDER_STYLE => {
                        let new_style = app.config.border_style.next();
                        app.config.border_style = new_style;
                        log::info!("Border style: {}", new_style.label());
                        if let Some(ref t) = tray {
                            t.update_border_style_text(new_style.label());
                        }
                        if let Some(ref mut bo) = border_overlay {
                            bo.set_style(new_style);
                            // Re-apply to current focus
                            if app.config.border_enabled {
                                if let Some(ref focus) = app.focus {
                                    let clamped = clamp_to_monitor(
                                        &focus.window_rect,
                                        &focus.monitor_rect,
                                    );
                                    bo.move_to(&clamped);
                                }
                            }
                        }
                    }
                    MENU_QUIT => {
                        log::info!("Quit requested");
                        PostQuitMessage(0);
                    }
                    _ => {}
                }
            }

            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    // Cleanup
    tracker::unhook(focus_hook, location_hook);
    unsafe {
        KillTimer(Some(msg_hwnd), TIMER_POLL).ok();
        KillTimer(Some(msg_hwnd), TIMER_HOTKEY_CHECK).ok();
        let _ = DestroyWindow(msg_hwnd);
    }

    log::info!("whereismywindow exiting");
}

/// Query current foreground window and update app state + overlays.
fn update_focus_state(
    app: &mut App,
    border_overlay: &mut Option<BorderOverlay>,
    flash_overlay: &Option<FlashOverlay>,
    indicators: &mut Option<MonitorIndicators>,
) {
    let Some(snapshot) = window_info::get_foreground_window_info() else {
        // Focus went to desktop, taskbar, minimized window, etc.
        // Hide the border so it doesn't linger on a stale position.
        if let Some(ref bo) = border_overlay {
            bo.hide();
        }
        app.focus = None;
        return;
    };

    // Skip our own overlay windows
    if let Some(ref bo) = border_overlay {
        if snapshot.hwnd == bo.hwnd.0 as isize {
            return;
        }
    }
    if let Some(ref fo) = flash_overlay {
        if snapshot.hwnd == fo.hwnd.0 as isize {
            return;
        }
    }
    if let Some(ref ind) = indicators {
        if ind.hwnd_list().contains(&snapshot.hwnd) {
            return;
        }
    }

    // Find which monitor
    let monitor_rects: Vec<_> = app.monitors.iter().map(|m| m.full_rect).collect();
    let monitor_index = if monitor_rects.is_empty() {
        0
    } else {
        geometry::best_monitor_index(&snapshot.rect, &monitor_rects)
    };

    let monitor_name = app
        .monitors
        .get(monitor_index)
        .map(|m| m.name.clone())
        .unwrap_or_default();

    let monitor_rect = app
        .monitors
        .get(monitor_index)
        .map(|m| m.full_rect)
        .unwrap_or_default();

    let focus_state = FocusState {
        hwnd: snapshot.hwnd,
        title: snapshot.title.clone(),
        exe_name: snapshot.exe_name.clone(),
        window_rect: snapshot.rect,
        monitor_index,
        monitor_name: monitor_name.clone(),
        monitor_rect,
    };

    log::info!(
        "Focus: \"{}\" ({}) on Monitor {} ({})",
        snapshot.title,
        snapshot.exe_name,
        monitor_index + 1,
        monitor_name.trim_end_matches('\0'),
    );

    let monitor_changed = app.update_focus(focus_state);

    // Update border overlay — use move_to on focus change to hide→move→show
    if app.config.border_enabled {
        let clamped = clamp_to_monitor(&snapshot.rect, &monitor_rect);
        if let Some(ref mut bo) = border_overlay {
            bo.move_to(&clamped);
        }
    }

    // Update monitor indicators
    if app.config.indicator_enabled {
        if let Some(ref mut ind) = indicators {
            ind.set_active(monitor_index);
        }
    }

    // Flash on monitor change
    if monitor_changed && app.config.flash_enabled {
        if let Some(ref fo) = flash_overlay {
            fo.flash(&monitor_rect);
            unsafe {
                let msg_hwnd = HWND(tracker::msg_hwnd_value() as *mut _);
                SetTimer(
                    Some(msg_hwnd),
                    TIMER_FLASH_HIDE,
                    app.config.flash_duration_ms,
                    None,
                );
            }
        }
    }
}

fn show_reveal_info(app: &App) {
    for (i, m) in app.monitors.iter().enumerate() {
        let focused = app
            .focus
            .as_ref()
            .map(|f| f.monitor_index == i)
            .unwrap_or(false);
        let marker = if focused { " [FOCUSED]" } else { "" };
        log::info!(
            "  Monitor {}: {}{}",
            i + 1,
            enumeration::format_monitor(m, i),
            marker,
        );
    }
    if let Some(ref focus) = app.focus {
        log::info!(
            "  Window: \"{}\" ({})",
            focus.title,
            focus.exe_name,
        );
    }
}

/// Clamp a window rect so it doesn't extend beyond its monitor.
/// Prevents the border overlay from leaking onto adjacent monitors
/// (maximized windows have a few px overscan beyond the screen edge).
fn clamp_to_monitor(rect: &RECT, monitor: &RECT) -> RECT {
    RECT {
        left: rect.left.max(monitor.left),
        top: rect.top.max(monitor.top),
        right: rect.right.min(monitor.right),
        bottom: rect.bottom.min(monitor.bottom),
    }
}

fn create_msg_window() -> HWND {
    unsafe {
        let hinstance = GetModuleHandleW(None).unwrap_or_default();
        let class_name: Vec<u16> = "WhereIsMyWindowMsg\0".encode_utf16().collect();

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(msg_wnd_proc),
            hInstance: hinstance.into(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };
        RegisterClassExW(&wc);

        match CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR::null(),
            WINDOW_STYLE::default(),
            0, 0, 0, 0,
            Some(HWND_MESSAGE),
            None,
            Some(hinstance.into()),
            None,
        ) {
            Ok(hwnd) => hwnd,
            Err(_) => HWND(std::ptr::null_mut()),
        }
    }
}

unsafe extern "system" fn msg_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    DefWindowProcW(hwnd, msg, wparam, lparam)
}
