use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;

use eframe::egui;

use crate::config::Config;
use crate::settings::data::{SettingsData, SettingsMessage};
use crate::settings::ui::SettingsApp;

static SETTINGS_OPEN: AtomicBool = AtomicBool::new(false);

/// Guard that resets SETTINGS_OPEN on drop (including panic unwind).
struct OpenGuard;

impl Drop for OpenGuard {
    fn drop(&mut self) {
        SETTINGS_OPEN.store(false, Ordering::Release);
    }
}

/// Spawn the settings window on a new thread.
/// Returns a Receiver for settings messages, or None if already open.
pub fn open_settings(config: &Config) -> Option<mpsc::Receiver<SettingsMessage>> {
    if SETTINGS_OPEN.swap(true, Ordering::AcqRel) {
        log::info!("Settings window already open");
        return None;
    }

    let data = SettingsData::from_config(config);
    let (tx, rx) = mpsc::channel();

    std::thread::Builder::new()
        .name("settings-ui".into())
        .spawn(move || {
            let _guard = OpenGuard;

            let options = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_title("WhereIsMyWindow Settings")
                    .with_inner_size([420.0, 560.0])
                    .with_resizable(false),
                ..Default::default()
            };

            let result = eframe::run_native(
                "WhereIsMyWindow Settings",
                options,
                Box::new(move |_cc| Ok(Box::new(SettingsApp::new(data, tx)))),
            );

            if let Err(e) = result {
                log::warn!("Settings window error: {}", e);
            }
        })
        .ok()?;

    Some(rx)
}
