use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
    IsWindowVisible,
};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
    PROCESS_NAME_FORMAT,
};
use windows::Win32::Foundation::CloseHandle;

/// Snapshot of a window's properties at a point in time.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WindowSnapshot {
    pub hwnd: isize,
    pub title: String,
    pub exe_name: String,
    pub rect: RECT,
    pub is_visible: bool,
}

/// Get the current foreground window info, or None if no valid window.
pub fn get_foreground_window_info() -> Option<WindowSnapshot> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }

        if !IsWindowVisible(hwnd).as_bool() {
            return None;
        }

        let title = get_window_title(hwnd);
        let exe_name = get_exe_name(hwnd);

        // Skip shell UI windows (tray popup, taskbar, start menu, etc.)
        // These are explorer.exe windows with no title — not real app windows.
        if title.is_empty() && exe_name.eq_ignore_ascii_case("explorer.exe") {
            return None;
        }

        let rect = get_extended_frame_bounds(hwnd)?;

        Some(WindowSnapshot {
            hwnd: hwnd.0 as isize,
            title,
            exe_name,
            rect,
            is_visible: true,
        })
    }
}

/// Get the extended frame bounds (excludes invisible Win10/11 borders).
pub fn get_extended_frame_bounds(hwnd: HWND) -> Option<RECT> {
    unsafe {
        let mut rect = RECT::default();
        let hr = DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut rect as *mut RECT as *mut _,
            std::mem::size_of::<RECT>() as u32,
        );
        if hr.is_ok() {
            Some(rect)
        } else {
            None
        }
    }
}

fn get_window_title(hwnd: HWND) -> String {
    unsafe {
        let len = GetWindowTextLengthW(hwnd);
        if len == 0 {
            return String::new();
        }
        let mut buf = vec![0u16; (len + 1) as usize];
        let copied = GetWindowTextW(hwnd, &mut buf);
        String::from_utf16_lossy(&buf[..copied as usize])
    }
}

fn get_exe_name(hwnd: HWND) -> String {
    unsafe {
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return String::new();
        }

        let Ok(process) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
            return String::new();
        };

        let mut buf = vec![0u16; 260];
        let mut size = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_FORMAT(0),
            windows::core::PWSTR(buf.as_mut_ptr()),
            &mut size,
        );
        let _ = CloseHandle(process);

        if ok.is_err() {
            return String::new();
        }

        let path = String::from_utf16_lossy(&buf[..size as usize]);
        path.rsplit('\\')
            .next()
            .unwrap_or(&path)
            .to_string()
    }
}
