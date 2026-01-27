use windows::Win32::Foundation::{LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW,
};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MonitorInfo {
    pub handle: isize,
    pub name: String,
    pub work_rect: RECT,
    pub full_rect: RECT,
    pub is_primary: bool,
}

/// Enumerate all connected monitors.
pub fn enumerate_monitors() -> Vec<MonitorInfo> {
    let mut monitors: Vec<MonitorInfo> = Vec::new();
    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(enum_callback),
            LPARAM(&mut monitors as *mut Vec<MonitorInfo> as isize),
        );
    }
    // Sort by left edge so index 0 = leftmost
    monitors.sort_by_key(|m| m.full_rect.left);
    monitors
}

unsafe extern "system" fn enum_callback(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _lprect: *mut RECT,
    lparam: LPARAM,
) -> windows::core::BOOL {
    let monitors = &mut *(lparam.0 as *mut Vec<MonitorInfo>);

    let mut info = MONITORINFOEXW::default();
    info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

    if GetMonitorInfoW(hmonitor, &mut info as *mut _ as *mut _).as_bool() {
        let name = String::from_utf16_lossy(
            &info.szDevice[..info.szDevice.iter().position(|&c| c == 0).unwrap_or(info.szDevice.len())],
        );
        let is_primary = (info.monitorInfo.dwFlags & 1) != 0; // MONITORINFOF_PRIMARY

        monitors.push(MonitorInfo {
            handle: hmonitor.0 as isize,
            name,
            work_rect: info.monitorInfo.rcWork,
            full_rect: info.monitorInfo.rcMonitor,
            is_primary,
        });
    }

    windows::core::BOOL(1) // TRUE - continue enumeration
}

/// Format monitor info for display/logging.
pub fn format_monitor(info: &MonitorInfo, index: usize) -> String {
    let r = &info.full_rect;
    let w = r.right - r.left;
    let h = r.bottom - r.top;
    format!(
        "Monitor {} ({}) {}x{} @ ({},{})",
        index + 1,
        info.name.trim_end_matches('\0'),
        w,
        h,
        r.left,
        r.top,
    )
}
