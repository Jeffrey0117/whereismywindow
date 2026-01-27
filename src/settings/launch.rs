use std::process::{Child, Command};

use eframe::egui;

use crate::settings::data::{SettingsData, SettingsMessage};
use crate::settings::ui::SettingsApp;

/// Spawn the settings window as a subprocess.
/// The subprocess runs the same exe with `--settings` and reads/writes config via TOML.
/// Returns a Child handle, or None on error.
pub fn open_settings() -> Option<Child> {
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            log::error!("Failed to get current exe path: {}", e);
            return None;
        }
    };

    match Command::new(exe).arg("--settings").spawn() {
        Ok(child) => {
            log::info!("Settings subprocess spawned (pid={})", child.id());
            Some(child)
        }
        Err(e) => {
            log::error!("Failed to spawn settings subprocess: {}", e);
            None
        }
    }
}

/// Check if the settings subprocess has exited.
/// Returns Some(true) if applied, Some(false) if cancelled, None if still running.
pub fn poll_child(child: &mut Child) -> Option<bool> {
    match child.try_wait() {
        Ok(Some(status)) => {
            let applied = status.success();
            log::info!("Settings subprocess exited (applied={})", applied);
            Some(applied)
        }
        Ok(None) => None,
        Err(e) => {
            log::error!("Failed to poll settings subprocess: {}", e);
            Some(false)
        }
    }
}

/// Run settings UI as the main process (called with `--settings` flag).
/// Returns exit code: 0 = applied, 1 = cancelled/closed.
pub fn run_settings_main() -> i32 {
    let config = crate::settings::persistence::load_config();
    let data = SettingsData::from_config(&config);
    let (tx, rx) = std::sync::mpsc::channel();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("WhereIsMyWindow Settings")
            .with_inner_size([420.0, 587.0])
            .with_resizable(true)
            .with_min_inner_size([420.0, 400.0]),
        ..Default::default()
    };

    let result = eframe::run_native(
        "WhereIsMyWindow Settings",
        options,
        Box::new(move |_cc| Ok(Box::new(SettingsApp::new(data, tx)))),
    );

    if let Err(e) = result {
        log::error!("Settings eframe error: {}", e);
        return 1;
    }

    // Process messages sent before window closed
    let mut applied = false;
    while let Ok(msg) = rx.try_recv() {
        match msg {
            SettingsMessage::Apply(data) => {
                let new_config = data.to_config();
                if config.auto_start != new_config.auto_start {
                    crate::settings::autostart::set_auto_start(new_config.auto_start);
                }
                crate::settings::persistence::save_config(&new_config);
                applied = true;
            }
            SettingsMessage::Closed => {}
        }
    }

    if applied { 0 } else { 1 }
}
