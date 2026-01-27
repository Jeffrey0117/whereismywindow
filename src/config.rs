/// Application configuration (colors, hotkey, toggles).
/// All values are compile-time defaults; runtime toggling via tray menu.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BorderStyle {
    Solid,
    Glow,
}

impl BorderStyle {
    pub fn next(self) -> Self {
        match self {
            Self::Solid => Self::Glow,
            Self::Glow => Self::Solid,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Solid => "Solid",
            Self::Glow => "Glow",
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Config {
    pub border_enabled: bool,
    pub flash_enabled: bool,
    pub indicator_enabled: bool,
    pub border_color: BorderColor,
    pub border_thickness: f32,
    pub border_style: BorderStyle,
    pub flash_duration_ms: u32,
    pub flash_opacity: f32,
    pub reveal_hotkey_enabled: bool,
    pub poll_interval_ms: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct BorderColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl BorderColor {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            border_enabled: true,
            flash_enabled: false,
            indicator_enabled: true,
            border_color: BorderColor::new(0.0, 0.47, 0.84, 0.9), // Blue
            border_thickness: 4.0,
            border_style: BorderStyle::Solid,
            flash_duration_ms: 150,
            flash_opacity: 0.25,
            reveal_hotkey_enabled: true,
            poll_interval_ms: 16, // ~60fps
        }
    }
}
