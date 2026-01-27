use std::path::PathBuf;

use crate::config::Config;
use crate::settings::autostart;

fn config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("whereismywindow"))
}

fn config_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("config.toml"))
}

pub fn load_config() -> Config {
    let Some(path) = config_path() else {
        log::warn!("Could not determine config directory; using defaults");
        let mut cfg = Config::default();
        cfg.auto_start = autostart::is_auto_start();
        return cfg;
    };

    let mut cfg = match std::fs::read_to_string(&path) {
        Ok(contents) => match toml::from_str::<Config>(&contents) {
            Ok(c) => {
                log::info!("Config loaded from {}", path.display());
                c
            }
            Err(e) => {
                log::warn!("Failed to parse {}: {}; using defaults", path.display(), e);
                Config::default()
            }
        },
        Err(_) => {
            log::info!("No config file at {}; using defaults", path.display());
            Config::default()
        }
    };

    // Sync auto_start with actual registry state
    cfg.auto_start = autostart::is_auto_start();
    cfg
}

pub fn save_config(config: &Config) {
    let Some(dir) = config_dir() else {
        log::warn!("Could not determine config directory; config not saved");
        return;
    };

    if let Err(e) = std::fs::create_dir_all(&dir) {
        log::warn!("Failed to create config dir {}: {}", dir.display(), e);
        return;
    }

    let Some(path) = config_path() else { return };

    match toml::to_string_pretty(config) {
        Ok(contents) => {
            if let Err(e) = std::fs::write(&path, contents) {
                log::warn!("Failed to write config to {}: {}", path.display(), e);
            } else {
                log::info!("Config saved to {}", path.display());
            }
        }
        Err(e) => {
            log::warn!("Failed to serialize config: {}", e);
        }
    }
}
