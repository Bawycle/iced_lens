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

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub language: Option<String>,
}

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
