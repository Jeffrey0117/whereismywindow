use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::UpdateWindow;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;

/// Magenta color key — pixels with this exact RGB become fully transparent.
pub const COLOR_KEY: COLORREF = COLORREF(0x00FF00FF); // RGB(255, 0, 255)

/// Create a transparent, click-through, topmost overlay window.
///
/// WS_EX_LAYERED | WS_EX_TRANSPARENT guarantees mouse/keyboard pass-through.
/// Caller must set layered attributes via set_colorkey() or set_alpha().
pub fn create_overlay_window(class_name: &str, width: i32, height: i32) -> Option<HWND> {
    unsafe {
        let hinstance = GetModuleHandleW(None).ok()?;
        let class_wide: Vec<u16> = class_name.encode_utf16().chain(std::iter::once(0)).collect();

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(overlay_wnd_proc),
            hInstance: hinstance.into(),
            lpszClassName: PCWSTR(class_wide.as_ptr()),
            hbrBackground: windows::Win32::Graphics::Gdi::HBRUSH(std::ptr::null_mut()),
            ..Default::default()
        };

        RegisterClassExW(&wc);

        let ex_style = WS_EX_LAYERED
            | WS_EX_TRANSPARENT
            | WS_EX_TOPMOST
            | WS_EX_NOACTIVATE
            | WS_EX_TOOLWINDOW;

        let hwnd = CreateWindowExW(
            ex_style,
            PCWSTR(class_wide.as_ptr()),
            PCWSTR::null(),
            WS_POPUP,
            0,
            0,
            width,
            height,
            None,
            None,
            Some(hinstance.into()),
            None,
        )
        .ok()?;

        Some(hwnd)
    }
}

/// Set color-key transparency: pixels matching COLOR_KEY become invisible.
/// Used by border overlay and indicator badges.
pub fn set_colorkey(hwnd: HWND) {
    unsafe {
        let _ = SetLayeredWindowAttributes(hwnd, COLOR_KEY, 0, LWA_COLORKEY);
    }
}

/// Set uniform alpha transparency for the entire window.
/// Used by flash overlay.
pub fn set_alpha(hwnd: HWND, alpha: u8) {
    unsafe {
        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), alpha, LWA_ALPHA);
    }
}

/// Reposition and resize an overlay window without activating it.
pub fn reposition_overlay(hwnd: HWND, rect: &RECT) {
    unsafe {
        let _ = SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            rect.left,
            rect.top,
            rect.right - rect.left,
            rect.bottom - rect.top,
            SWP_NOACTIVATE | SWP_SHOWWINDOW,
        );
        let _ = UpdateWindow(hwnd);
    }
}

pub fn hide_overlay(hwnd: HWND) {
    unsafe {
        let _ = ShowWindow(hwnd, SW_HIDE);
    }
}

pub fn show_overlay(hwnd: HWND) {
    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
    }
}

unsafe extern "system" fn overlay_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_DESTROY => LRESULT(0),
        WM_NCHITTEST => LRESULT(-1), // HTTRANSPARENT
        WM_ERASEBKGND => LRESULT(1), // Skip GDI background erase
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
