//! This module handles the application's configuration, including loading and saving
//! user preferences to a `settings.toml` file.
//!
//! # Examples
//!
//! ```no_run
//! use iced_lens::config::{self, Config};
//! use std::path::PathBuf;
//!
//! // Load existing configuration
//! let mut config = config::load().unwrap_or_default();
//!
//! // Modify a setting
//! config.language = Some("fr".to_string());
//!
//! // Save the modified configuration
//! config::save(&config).expect("Failed to save config");
//!
//! // To load/save from a specific path (e.g., for testing)
//! let temp_dir = PathBuf::from("./temp_config_dir");
//! std::fs::create_dir_all(&temp_dir).unwrap();
//! let temp_file = temp_dir.join("test_settings.toml");
//! config::save_to_path(&config, &temp_file).expect("Failed to save to path");
//! let loaded_config = config::load_from_path(&temp_file).expect("Failed to load from path");
//! assert_eq!(loaded_config.language, Some("fr".to_string()));
//! std::fs::remove_dir_all(&temp_dir).unwrap();
//! ```

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf}; // Added PathBuf back

const CONFIG_FILE: &str = "settings.toml";
const APP_NAME: &str = "IcedLens";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub language: Option<String>,
    #[serde(default)]
    pub fit_to_window: Option<bool>,
    #[serde(default)]
    pub zoom_step: Option<f32>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: None,
            fit_to_window: Some(true),
            zoom_step: Some(DEFAULT_ZOOM_STEP_PERCENT),
        }
    }
}

pub const DEFAULT_ZOOM_STEP_PERCENT: f32 = 10.0;

fn get_default_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|mut path| {
        path.push(APP_NAME);
        path.push(CONFIG_FILE);
        path
    })
}

pub fn load() -> Result<Config> {
    if let Some(path) = get_default_config_path() {
        if path.exists() {
            return load_from_path(&path);
        }
    }
    Ok(Config::default())
}

pub fn save(config: &Config) -> Result<()> {
    if let Some(path) = get_default_config_path() {
        return save_to_path(config, &path);
    }
    Ok(())
}

pub fn load_from_path(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path)?;
    Ok(toml::from_str(&content).unwrap_or_default())
}

pub fn save_to_path(config: &Config, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(config).unwrap();
    fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn save_and_load_round_trip_preserves_language() {
        let config = Config {
            language: Some("fr".to_string()),
            fit_to_window: Some(false),
            zoom_step: Some(5.0),
        };
        let temp_dir = tempdir().expect("failed to create temp dir");
        let config_path = temp_dir.path().join("nested").join("settings.toml");

        save_to_path(&config, &config_path).expect("failed to save config");
        let loaded = load_from_path(&config_path).expect("failed to load config");

        assert_eq!(loaded.language, config.language);
        assert_eq!(loaded.fit_to_window, config.fit_to_window);
        assert_eq!(loaded.zoom_step, config.zoom_step);
    }

    #[test]
    fn load_from_path_returns_default_on_invalid_toml() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let config_path = temp_dir.path().join("settings.toml");
        fs::write(&config_path, "not = valid = toml").expect("failed to write invalid toml");

        let loaded = load_from_path(&config_path).expect("load should not error");
        assert!(loaded.language.is_none());
    }

    #[test]
    fn save_to_path_creates_parent_directories() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let nested_dir = temp_dir.path().join("deep").join("path");
        let config_path = nested_dir.join("settings.toml");
        let config = Config {
            language: Some("en-US".to_string()),
            fit_to_window: Some(false),
            zoom_step: Some(7.5),
        };

        save_to_path(&config, &config_path).expect("save should create directories");
        assert!(config_path.exists());
    }

    #[test]
    fn default_config_sets_fit_and_zoom_step() {
        let config = Config::default();
        assert_eq!(config.fit_to_window, Some(true));
        assert_eq!(config.zoom_step, Some(DEFAULT_ZOOM_STEP_PERCENT));
    }
}
