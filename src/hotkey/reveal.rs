use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager,
    hotkey::{Code, HotKey, Modifiers},
};

pub struct RevealHotkey {
    manager: GlobalHotKeyManager,
    hotkey: HotKey,
    pub is_active: bool,
}

impl RevealHotkey {
    pub fn new() -> Option<Self> {
        let manager = GlobalHotKeyManager::new().ok()?;
        let hotkey = HotKey::new(
            Some(Modifiers::CONTROL | Modifiers::SHIFT),
            Code::KeyF,
        );
        manager.register(hotkey).ok()?;
        log::info!("Registered global hotkey: Ctrl+Shift+F");

        Some(Self {
            manager,
            hotkey,
            is_active: false,
        })
    }

    /// Check for pending hotkey events. Returns true if hotkey was pressed.
    pub fn poll(&self) -> bool {
        if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.id() == self.hotkey.id() {
                return true;
            }
        }
        false
    }
}

impl Drop for RevealHotkey {
    fn drop(&mut self) {
        let _ = self.manager.unregister(self.hotkey);
    }
}
