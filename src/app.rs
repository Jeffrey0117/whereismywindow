use windows::Win32::Foundation::RECT;

use crate::config::Config;
use crate::monitor::enumeration::MonitorInfo;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FocusState {
    pub hwnd: isize,
    pub title: String,
    pub exe_name: String,
    pub window_rect: RECT,
    pub monitor_index: usize,
    pub monitor_name: String,
    pub monitor_rect: RECT,
}

/// Top-level application state managed by the message loop.
pub struct App {
    pub config: Config,
    pub focus: Option<FocusState>,
    pub prev_monitor_index: Option<usize>,
    pub monitors: Vec<MonitorInfo>,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            focus: None,
            prev_monitor_index: None,
            monitors: Vec::new(),
        }
    }

    /// Replace focus state with a new snapshot. Returns whether the monitor changed.
    pub fn update_focus(&mut self, new_focus: FocusState) -> bool {
        let monitor_changed = match self.prev_monitor_index {
            Some(prev) => prev != new_focus.monitor_index,
            None => false,
        };
        self.prev_monitor_index = Some(new_focus.monitor_index);
        self.focus = Some(new_focus);
        monitor_changed
    }

    #[allow(dead_code)]
    pub fn refresh_monitors(&mut self) {
        self.monitors = crate::monitor::enumeration::enumerate_monitors();
    }
}
