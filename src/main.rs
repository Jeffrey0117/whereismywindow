#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod focus;
mod hotkey;
mod monitor;
mod overlay;
mod tray;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
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
use tray::icon::{self as tray_icon_mod, SystemTray, MENU_QUIT, MENU_TOGGLE_BORDER, MENU_TOGGLE_FLASH};

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
    let mut border_overlay = BorderOverlay::new(config.border_color, config.border_thickness);
    let flash_overlay = FlashOverlay::new(config.flash_opacity);

    if border_overlay.is_none() {
        log::warn!("Failed to create border overlay");
    }
    if flash_overlay.is_none() {
        log::warn!("Failed to create flash overlay");
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
    update_focus_state(&mut app, &mut border_overlay, &flash_overlay);

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
                    update_focus_state(&mut app, &mut border_overlay, &flash_overlay);
                }
                WM_LOCATION_CHANGED => {
                    // Window moved/resized: update border position
                    if app.config.border_enabled {
                        if let Some(ref focus) = app.focus {
                            if let Some(new_rect) = window_info::get_extended_frame_bounds(
                                HWND(focus.hwnd as *mut _),
                            ) {
                                if let Some(ref mut bo) = border_overlay {
                                    bo.update(&new_rect);
                                }
                            }
                        }
                    }
                }
                WM_TIMER => {
                    let timer_id = msg.wParam.0;
                    match timer_id {
                        TIMER_POLL => {
                            // Poll for window movement (backup for EVENT_OBJECT_LOCATIONCHANGE)
                            if app.config.border_enabled {
                                if let Some(ref focus) = app.focus {
                                    if let Some(new_rect) = window_info::get_extended_frame_bounds(
                                        HWND(focus.hwnd as *mut _),
                                    ) {
                                        if new_rect != focus.window_rect {
                                            if let Some(ref mut bo) = border_overlay {
                                                bo.update(&new_rect);
                                            }
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
                            // Check for hotkey events via global-hotkey channel
                            if let Some(ref _hk) = hotkey_handler {
                                if _hk.poll() {
                                    // Ctrl+Shift+F was pressed - toggle reveal
                                    log::info!("Hotkey reveal triggered");
                                    // Show all monitor info + highlight
                                    show_reveal_info(&app);
                                }
                            }

                            // Also check async key state to detect release
                            let ctrl_down = GetAsyncKeyState(0x11) < 0; // VK_CONTROL
                            let shift_down = GetAsyncKeyState(0x10) < 0; // VK_SHIFT
                            let f_down = GetAsyncKeyState(0x46) < 0; // 'F'
                            if let Some(ref _hk) = hotkey_handler {
                                if _hk.is_active && !(ctrl_down && shift_down && f_down) {
                                    // Keys released
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
                        log::info!("Border: {}", if app.config.border_enabled { "ON" } else { "OFF" });
                        if let Some(ref t) = tray {
                            t.update_border_text(app.config.border_enabled);
                        }
                        if !app.config.border_enabled {
                            if let Some(ref bo) = border_overlay {
                                bo.hide();
                            }
                        } else {
                            update_focus_state(&mut app, &mut border_overlay, &flash_overlay);
                        }
                    }
                    MENU_TOGGLE_FLASH => {
                        app.config.flash_enabled = !app.config.flash_enabled;
                        log::info!("Flash: {}", if app.config.flash_enabled { "ON" } else { "OFF" });
                        if let Some(ref t) = tray {
                            t.update_flash_text(app.config.flash_enabled);
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
) {
    let Some(snapshot) = window_info::get_foreground_window_info() else {
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

    // Update border overlay
    if app.config.border_enabled {
        if let Some(ref mut bo) = border_overlay {
            bo.update(&snapshot.rect);
            bo.show();
        }
    }

    // Flash on monitor change
    if monitor_changed && app.config.flash_enabled {
        if let Some(ref fo) = flash_overlay {
            fo.flash(&monitor_rect);
            // Set timer to hide flash
            unsafe {
                let msg_hwnd = HWND(tracker::msg_hwnd_value() as *mut _);
                SetTimer(Some(msg_hwnd), TIMER_FLASH_HIDE, app.config.flash_duration_ms, None);
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
