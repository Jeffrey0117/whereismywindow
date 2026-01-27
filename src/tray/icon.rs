use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIcon, TrayIconBuilder, Icon,
};

pub const MENU_TOGGLE_BORDER: &str = "toggle_border";
pub const MENU_TOGGLE_FLASH: &str = "toggle_flash";
pub const MENU_QUIT: &str = "quit";

#[allow(dead_code)]
pub struct SystemTray {
    _tray: TrayIcon,
    pub toggle_border_item: MenuItem,
    pub toggle_flash_item: MenuItem,
    pub quit_item: MenuItem,
}

impl SystemTray {
    pub fn new() -> Option<Self> {
        let icon = create_default_icon()?;

        let toggle_border_item = MenuItem::with_id(
            MENU_TOGGLE_BORDER,
            "Border: ON",
            true,
            None,
        );
        let toggle_flash_item = MenuItem::with_id(
            MENU_TOGGLE_FLASH,
            "Flash: ON",
            true,
            None,
        );
        let quit_item = MenuItem::with_id(
            MENU_QUIT,
            "Quit",
            true,
            None,
        );

        let menu = Menu::new();
        let _ = menu.append(&toggle_border_item);
        let _ = menu.append(&toggle_flash_item);
        let _ = menu.append(&PredefinedMenuItem::separator());
        let _ = menu.append(&quit_item);

        let tray = TrayIconBuilder::new()
            .with_icon(icon)
            .with_tooltip("Where Is My Window?")
            .with_menu(Box::new(menu))
            .build()
            .ok()?;

        Some(Self {
            _tray: tray,
            toggle_border_item,
            toggle_flash_item,
            quit_item,
        })
    }

    pub fn update_border_text(&self, enabled: bool) {
        let text = if enabled { "Border: ON" } else { "Border: OFF" };
        self.toggle_border_item.set_text(text);
    }

    pub fn update_flash_text(&self, enabled: bool) {
        let text = if enabled { "Flash: ON" } else { "Flash: OFF" };
        self.toggle_flash_item.set_text(text);
    }
}

/// Create a simple colored icon in memory (16x16 blue square).
fn create_default_icon() -> Option<Icon> {
    let size = 16u32;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for _y in 0..size {
        for _x in 0..size {
            // Blue icon with slight border
            rgba.push(0);    // R
            rgba.push(120);  // G
            rgba.push(215);  // B
            rgba.push(255);  // A
        }
    }
    Icon::from_rgba(rgba, size, size).ok()
}

/// Poll for tray menu events. Returns the menu item ID if one was clicked.
pub fn poll_menu_event() -> Option<String> {
    if let Ok(event) = MenuEvent::receiver().try_recv() {
        Some(event.id().0.to_string())
    } else {
        None
    }
}
