use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::DwmExtendFrameIntoClientArea;
use windows::Win32::Graphics::Gdi::UpdateWindow;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::MARGINS;
use windows::Win32::UI::WindowsAndMessaging::*;

/// Create a transparent, click-through, topmost overlay window.
/// Uses WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW.
/// DwmExtendFrameIntoClientArea enables per-pixel alpha via Direct2D premultiplied rendering.
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

        // Activate the layered window — set fully opaque at the window level.
        // Per-pixel alpha is then handled by DWM + Direct2D premultiplied rendering.
        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA);

        // Extend frame into client area for per-pixel alpha compositing
        let margins = MARGINS {
            cxLeftWidth: -1,
            cxRightWidth: -1,
            cyTopHeight: -1,
            cyBottomHeight: -1,
        };
        let _ = DwmExtendFrameIntoClientArea(hwnd, &margins);

        Some(hwnd)
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

/// Hide the overlay window.
pub fn hide_overlay(hwnd: HWND) {
    unsafe {
        let _ = ShowWindow(hwnd, SW_HIDE);
    }
}

/// Show the overlay window without activating.
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
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
