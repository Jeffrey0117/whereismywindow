use crate::config::{BorderColor, BorderStyle, Config};

/// Plain-data mirror of Config, used for egui editing and channel transport.
#[derive(Debug, Clone)]
pub struct SettingsData {
    pub border_enabled: bool,
    pub flash_enabled: bool,
    pub indicator_enabled: bool,
    pub border_color: [f32; 3],
    pub border_thickness: f32,
    pub border_style: BorderStyle,
    pub flash_duration_ms: u32,
    pub flash_opacity: f32,
    pub reveal_hotkey_enabled: bool,
    pub auto_start: bool,
    pub poll_interval_ms: u32,
}

pub enum SettingsMessage {
    Apply(SettingsData),
    Closed,
}

impl SettingsData {
    pub fn from_config(config: &Config) -> Self {
        Self {
            border_enabled: config.border_enabled,
            flash_enabled: config.flash_enabled,
            indicator_enabled: config.indicator_enabled,
            border_color: [config.border_color.r, config.border_color.g, config.border_color.b],
            border_thickness: config.border_thickness,
            border_style: config.border_style,
            flash_duration_ms: config.flash_duration_ms,
            flash_opacity: config.flash_opacity,
            reveal_hotkey_enabled: config.reveal_hotkey_enabled,
            auto_start: config.auto_start,
            poll_interval_ms: config.poll_interval_ms,
        }
    }

    pub fn to_config(&self) -> Config {
        Config {
            border_enabled: self.border_enabled,
            flash_enabled: self.flash_enabled,
            indicator_enabled: self.indicator_enabled,
            border_color: BorderColor::new(
                self.border_color[0],
                self.border_color[1],
                self.border_color[2],
                0.9,
            ),
            border_thickness: self.border_thickness,
            border_style: self.border_style,
            flash_duration_ms: self.flash_duration_ms,
            flash_opacity: self.flash_opacity,
            reveal_hotkey_enabled: self.reveal_hotkey_enabled,
            poll_interval_ms: self.poll_interval_ms,
            auto_start: self.auto_start,
        }
    }
}
