// SPDX-License-Identifier: MPL-2.0
//! This module handles the application's configuration, including loading and saving
//! user preferences to a `settings.toml` file.
//!
//! # Path Resolution
//!
//! The config file location can be customized for testing or portable deployments:
//! 1. Use `load_from_path()`/`save_to_path()` with explicit path
//! 2. Set `ICED_LENS_CONFIG_DIR` environment variable
//! 3. Falls back to platform-specific config directory
//!
//! # Examples
//!
//! ```no_run
//! use iced_lens::config::{self, Config};
//! use std::path::PathBuf;
//!
//! // Load existing configuration (returns tuple with optional warning)
//! let (mut config, _warning) = config::load();
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

pub mod defaults;

// Re-export all default constants for backward compatibility
pub use defaults::*;

use crate::app::paths;
use crate::error::{Error, Result};
use crate::ui::theming::ThemeMode;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const CONFIG_FILE: &str = "settings.toml";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum BackgroundTheme {
    Light,
    #[default]
    Dark,
    Checkerboard,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SortOrder {
    #[default]
    Alphabetical,
    ModifiedDate,
    CreatedDate,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub language: Option<String>,
    #[serde(default)]
    pub fit_to_window: Option<bool>,
    #[serde(default)]
    pub zoom_step: Option<f32>,
    #[serde(default)]
    pub background_theme: Option<BackgroundTheme>,
    #[serde(default)]
    pub sort_order: Option<SortOrder>,
    #[serde(default)]
    pub overlay_timeout_secs: Option<u32>,
    #[serde(
        default = "default_theme_mode",
        deserialize_with = "deserialize_theme_mode"
    )]
    pub theme_mode: ThemeMode,
    /// Whether videos should auto-play when loaded.
    /// Defaults to false (no auto-play).
    #[serde(default)]
    pub video_autoplay: Option<bool>,
    /// Video playback volume (0.0 to 1.0).
    /// Defaults to 0.8 (80%).
    #[serde(default)]
    pub video_volume: Option<f32>,
    /// Whether video audio is muted.
    /// Defaults to false.
    #[serde(default)]
    pub video_muted: Option<bool>,
    /// Whether video playback should loop.
    /// Defaults to false.
    #[serde(default)]
    pub video_loop: Option<bool>,
    /// Whether to normalize audio volume across different media files.
    /// Uses loudness normalization to prevent sudden volume changes when navigating.
    /// Defaults to true.
    #[serde(default = "default_audio_normalization")]
    pub audio_normalization: Option<bool>,
    /// Video frame cache size in megabytes.
    /// Higher values improve seek performance but use more memory.
    /// Defaults to 64 MB.
    #[serde(default = "default_frame_cache_mb")]
    pub frame_cache_mb: Option<u32>,
    /// Frame history size in megabytes for frame-by-frame backward stepping.
    /// Higher values allow stepping back further but use more memory.
    /// Only used during frame stepping mode, not during normal playback.
    /// Defaults to 128 MB.
    #[serde(default = "default_frame_history_mb")]
    pub frame_history_mb: Option<u32>,
    /// Keyboard seek step in seconds (arrow keys during video playback).
    /// Defaults to 2.0 seconds.
    #[serde(default = "default_keyboard_seek_step_secs")]
    pub keyboard_seek_step_secs: Option<f64>,
}

fn default_frame_cache_mb() -> Option<u32> {
    Some(DEFAULT_FRAME_CACHE_MB)
}

fn default_frame_history_mb() -> Option<u32> {
    Some(DEFAULT_FRAME_HISTORY_MB)
}

fn default_keyboard_seek_step_secs() -> Option<f64> {
    Some(DEFAULT_KEYBOARD_SEEK_STEP_SECS)
}

fn default_audio_normalization() -> Option<bool> {
    Some(true) // Enabled by default - normalizes audio volume between different media files
}

fn default_theme_mode() -> ThemeMode {
    ThemeMode::System
}

fn deserialize_theme_mode<'de, D>(deserializer: D) -> std::result::Result<ThemeMode, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    let raw = String::deserialize(deserializer)?;
    match raw.to_lowercase().as_str() {
        "light" => Ok(ThemeMode::Light),
        "dark" => Ok(ThemeMode::Dark),
        "system" => Ok(ThemeMode::System),
        other => Err(D::Error::custom(format!("invalid theme_mode: {}", other))),
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: None,
            fit_to_window: Some(true),
            zoom_step: Some(DEFAULT_ZOOM_STEP_PERCENT),
            background_theme: Some(BackgroundTheme::default()),
            sort_order: Some(SortOrder::default()),
            overlay_timeout_secs: Some(DEFAULT_OVERLAY_TIMEOUT_SECS),
            theme_mode: default_theme_mode(),
            video_autoplay: Some(false),
            video_volume: Some(DEFAULT_VOLUME),
            video_muted: Some(false),
            video_loop: Some(false),
            audio_normalization: default_audio_normalization(),
            frame_cache_mb: default_frame_cache_mb(),
            frame_history_mb: default_frame_history_mb(),
            keyboard_seek_step_secs: default_keyboard_seek_step_secs(),
        }
    }
}

/// Returns the config file path with an optional override.
///
/// # Arguments
///
/// * `base_dir` - Optional base directory. If `None`, uses default path resolution.
///
/// # Path Resolution
///
/// 1. `base_dir` parameter (if `Some`)
/// 2. `ICED_LENS_CONFIG_DIR` environment variable (if set)
/// 3. Platform-specific config directory
fn get_config_path_with_override(base_dir: Option<PathBuf>) -> Option<PathBuf> {
    paths::get_app_config_dir_with_override(base_dir).map(|mut path| {
        path.push(CONFIG_FILE);
        path
    })
}

/// Loads the configuration from the default path.
///
/// Returns a tuple of (config, optional_warning). If loading fails, returns
/// default config with a warning message explaining what went wrong.
///
/// # Path Resolution
///
/// Uses the standard path resolution (see [`paths::get_app_config_dir`]):
/// 1. `ICED_LENS_CONFIG_DIR` environment variable (if set)
/// 2. Platform-specific config directory
pub fn load() -> (Config, Option<String>) {
    load_with_override(None)
}

/// Loads the configuration from a custom directory.
///
/// # Arguments
///
/// * `base_dir` - Optional base directory. If `None`, uses default path resolution.
///
/// # Path Resolution
///
/// 1. `base_dir` parameter (if `Some`)
/// 2. `ICED_LENS_CONFIG_DIR` environment variable (if set)
/// 3. Platform-specific config directory
pub fn load_with_override(base_dir: Option<PathBuf>) -> (Config, Option<String>) {
    if let Some(path) = get_config_path_with_override(base_dir) {
        if path.exists() {
            match load_from_path(&path) {
                Ok(config) => return (config, None),
                Err(_) => {
                    return (
                        Config::default(),
                        Some("notification-config-load-error".to_string()),
                    );
                }
            }
        }
    }
    (Config::default(), None)
}

/// Saves the configuration to the default path.
///
/// # Path Resolution
///
/// Uses the standard path resolution (see [`paths::get_app_config_dir`]):
/// 1. `ICED_LENS_CONFIG_DIR` environment variable (if set)
/// 2. Platform-specific config directory
pub fn save(config: &Config) -> Result<()> {
    save_with_override(config, None)
}

/// Saves the configuration to a custom directory.
///
/// # Arguments
///
/// * `config` - The configuration to save
/// * `base_dir` - Optional base directory. If `None`, uses default path resolution.
///
/// # Path Resolution
///
/// 1. `base_dir` parameter (if `Some`)
/// 2. `ICED_LENS_CONFIG_DIR` environment variable (if set)
/// 3. Platform-specific config directory
pub fn save_with_override(config: &Config, base_dir: Option<PathBuf>) -> Result<()> {
    if let Some(path) = get_config_path_with_override(base_dir) {
        return save_to_path(config, &path);
    }
    Ok(())
}

pub fn load_from_path(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

pub fn save_to_path(config: &Config, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(config).map_err(Error::from)?;
    fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use tempfile::tempdir;

    #[test]
    fn save_and_load_round_trip_preserves_language() {
        let config = Config {
            language: Some("fr".to_string()),
            fit_to_window: Some(false),
            zoom_step: Some(5.0),
            background_theme: Some(BackgroundTheme::Light),
            sort_order: Some(SortOrder::Alphabetical),
            overlay_timeout_secs: Some(DEFAULT_OVERLAY_TIMEOUT_SECS),
            theme_mode: ThemeMode::Light,
            video_autoplay: Some(false),
            video_volume: Some(DEFAULT_VOLUME),
            video_muted: Some(false),
            video_loop: Some(false),
            audio_normalization: Some(true),
            frame_cache_mb: Some(DEFAULT_FRAME_CACHE_MB),
            frame_history_mb: Some(DEFAULT_FRAME_HISTORY_MB),
            keyboard_seek_step_secs: Some(DEFAULT_KEYBOARD_SEEK_STEP_SECS),
        };
        let temp_dir = tempdir().expect("failed to create temp dir");
        let config_path = temp_dir.path().join("nested").join("settings.toml");

        save_to_path(&config, &config_path).expect("failed to save config");
        let loaded = load_from_path(&config_path).expect("failed to load config");

        assert_eq!(loaded.language, config.language);
        assert_eq!(loaded.fit_to_window, config.fit_to_window);
        assert_eq!(loaded.zoom_step, config.zoom_step);
        assert_eq!(loaded.theme_mode, config.theme_mode);
    }

    #[test]
    fn load_from_path_invalid_toml_errors() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let config_path = temp_dir.path().join("settings.toml");
        fs::write(&config_path, "not = valid = toml").expect("failed to write invalid toml");

        match load_from_path(&config_path) {
            Err(Error::Config(message)) => assert!(message.contains("expected")),
            other => panic!("expected Config error, got {:?}", other),
        }
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
            background_theme: Some(BackgroundTheme::Checkerboard),
            sort_order: Some(SortOrder::CreatedDate),
            overlay_timeout_secs: Some(DEFAULT_OVERLAY_TIMEOUT_SECS),
            theme_mode: ThemeMode::System,
            video_autoplay: Some(true),
            video_volume: Some(0.5),
            video_muted: Some(true),
            video_loop: Some(true),
            audio_normalization: Some(false),
            frame_cache_mb: Some(128),
            frame_history_mb: Some(DEFAULT_FRAME_HISTORY_MB),
            keyboard_seek_step_secs: Some(DEFAULT_KEYBOARD_SEEK_STEP_SECS),
        };

        save_to_path(&config, &config_path).expect("save should create directories");
        assert!(config_path.exists());
    }

    #[test]
    fn default_config_sets_fit_and_zoom_step() {
        let config = Config::default();
        assert_eq!(config.fit_to_window, Some(true));
        assert_eq!(config.zoom_step, Some(DEFAULT_ZOOM_STEP_PERCENT));
        assert_eq!(config.background_theme, Some(BackgroundTheme::default()));
        assert_eq!(config.sort_order, Some(SortOrder::default()));
        assert_eq!(config.theme_mode, ThemeMode::System);
        assert_eq!(config.video_autoplay, Some(false));
        assert_eq!(config.video_volume, Some(DEFAULT_VOLUME));
        assert_eq!(config.video_muted, Some(false));
        assert_eq!(config.audio_normalization, Some(true));
        assert_eq!(config.frame_cache_mb, Some(DEFAULT_FRAME_CACHE_MB));
    }

    #[test]
    fn save_and_load_preserves_sort_order() {
        let config = Config {
            language: Some("en-US".to_string()),
            fit_to_window: Some(true),
            zoom_step: Some(10.0),
            background_theme: Some(BackgroundTheme::Dark),
            sort_order: Some(SortOrder::ModifiedDate),
            overlay_timeout_secs: Some(DEFAULT_OVERLAY_TIMEOUT_SECS),
            theme_mode: ThemeMode::System,
            video_autoplay: Some(false),
            video_volume: Some(DEFAULT_VOLUME),
            video_muted: Some(false),
            video_loop: Some(false),
            audio_normalization: Some(true),
            frame_cache_mb: Some(DEFAULT_FRAME_CACHE_MB),
            frame_history_mb: Some(DEFAULT_FRAME_HISTORY_MB),
            keyboard_seek_step_secs: Some(DEFAULT_KEYBOARD_SEEK_STEP_SECS),
        };
        let temp_dir = tempdir().expect("failed to create temp dir");
        let config_path = temp_dir.path().join("settings.toml");

        save_to_path(&config, &config_path).expect("failed to save config");
        let loaded = load_from_path(&config_path).expect("failed to load config");

        assert_eq!(loaded.sort_order, Some(SortOrder::ModifiedDate));
    }

    #[test]
    fn sort_order_default_is_alphabetical() {
        assert_eq!(SortOrder::default(), SortOrder::Alphabetical);
    }

    #[test]
    fn default_config_sets_overlay_timeout() {
        let config = Config::default();
        assert_eq!(
            config.overlay_timeout_secs,
            Some(DEFAULT_OVERLAY_TIMEOUT_SECS)
        );
        assert_eq!(DEFAULT_OVERLAY_TIMEOUT_SECS, 3);
    }

    #[test]
    fn save_and_load_preserves_overlay_timeout() {
        let config = Config {
            language: Some("en-US".to_string()),
            fit_to_window: Some(true),
            zoom_step: Some(10.0),
            background_theme: Some(BackgroundTheme::Dark),
            sort_order: Some(SortOrder::Alphabetical),
            overlay_timeout_secs: Some(5),
            theme_mode: ThemeMode::System,
            video_autoplay: Some(false),
            video_volume: Some(DEFAULT_VOLUME),
            video_muted: Some(false),
            video_loop: Some(false),
            audio_normalization: Some(true),
            frame_cache_mb: Some(DEFAULT_FRAME_CACHE_MB),
            frame_history_mb: Some(DEFAULT_FRAME_HISTORY_MB),
            keyboard_seek_step_secs: Some(DEFAULT_KEYBOARD_SEEK_STEP_SECS),
        };
        let temp_dir = tempdir().expect("failed to create temp dir");
        let config_path = temp_dir.path().join("settings.toml");

        save_to_path(&config, &config_path).expect("failed to save config");
        let loaded = load_from_path(&config_path).expect("failed to load config");

        assert_eq!(loaded.overlay_timeout_secs, Some(5));
    }

    #[test]
    fn overlay_timeout_bounds_are_reasonable() {
        // Constant validation is done at compile-time (see const _: () assertion above)
        // This test verifies the actual values match expected reasonable ranges
        assert_eq!(MIN_OVERLAY_TIMEOUT_SECS, 1);
        assert_eq!(MAX_OVERLAY_TIMEOUT_SECS, 30);
    }

    #[test]
    fn save_and_load_preserves_audio_settings() {
        let config = Config {
            language: None,
            fit_to_window: Some(true),
            zoom_step: Some(10.0),
            background_theme: Some(BackgroundTheme::Dark),
            sort_order: Some(SortOrder::Alphabetical),
            overlay_timeout_secs: Some(DEFAULT_OVERLAY_TIMEOUT_SECS),
            theme_mode: ThemeMode::System,
            video_autoplay: Some(true),
            video_volume: Some(0.65),
            video_muted: Some(true),
            video_loop: Some(true),
            audio_normalization: Some(false),
            frame_cache_mb: Some(DEFAULT_FRAME_CACHE_MB),
            frame_history_mb: Some(DEFAULT_FRAME_HISTORY_MB),
            keyboard_seek_step_secs: Some(DEFAULT_KEYBOARD_SEEK_STEP_SECS),
        };
        let temp_dir = tempdir().expect("failed to create temp dir");
        let config_path = temp_dir.path().join("settings.toml");

        save_to_path(&config, &config_path).expect("failed to save config");
        let loaded = load_from_path(&config_path).expect("failed to load config");

        assert_eq!(loaded.video_volume, Some(0.65));
        assert_eq!(loaded.video_muted, Some(true));
        assert_eq!(loaded.video_loop, Some(true));
        assert_eq!(loaded.audio_normalization, Some(false));
    }

    #[test]
    fn audio_normalization_defaults_to_true() {
        // When audio_normalization is not in the config file, it should default to true
        let config = Config::default();
        assert_eq!(config.audio_normalization, Some(true));
    }

    #[test]
    fn volume_constants_are_valid() {
        assert_eq!(DEFAULT_VOLUME, 0.8);
        assert_eq!(MIN_VOLUME, 0.0);
        assert_eq!(MAX_VOLUME, 1.0);
        // Use runtime bindings to avoid clippy assertions_on_constants warning
        let step = VOLUME_STEP;
        let default = DEFAULT_VOLUME;
        let min = MIN_VOLUME;
        let max = MAX_VOLUME;
        assert!(step > 0.0, "VOLUME_STEP must be positive");
        assert!(
            default >= min && default <= max,
            "DEFAULT_VOLUME must be within valid range"
        );
    }

    #[test]
    fn save_with_override_and_load_with_override_round_trip() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let base_dir = temp_dir.path().to_path_buf();

        let config = Config {
            language: Some("de".to_string()),
            fit_to_window: Some(false),
            zoom_step: Some(15.0),
            background_theme: Some(BackgroundTheme::Light),
            sort_order: Some(SortOrder::CreatedDate),
            overlay_timeout_secs: Some(7),
            theme_mode: ThemeMode::Dark,
            video_autoplay: Some(true),
            video_volume: Some(0.5),
            video_muted: Some(true),
            video_loop: Some(true),
            audio_normalization: Some(false),
            frame_cache_mb: Some(256),
            frame_history_mb: Some(64),
            keyboard_seek_step_secs: Some(5.0),
        };

        // Save to custom directory
        save_with_override(&config, Some(base_dir.clone())).expect("save should succeed");

        // Verify file was created in the expected location
        let expected_path = base_dir.join("settings.toml");
        assert!(expected_path.exists(), "config file should exist");

        // Load from same custom directory
        let (loaded, warning) = load_with_override(Some(base_dir));
        assert!(warning.is_none(), "load should succeed without warning");
        assert_eq!(loaded.language, Some("de".to_string()));
        assert_eq!(loaded.zoom_step, Some(15.0));
        assert_eq!(loaded.theme_mode, ThemeMode::Dark);
    }

    #[test]
    fn load_with_override_from_empty_directory_returns_default() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let base_dir = temp_dir.path().to_path_buf();

        // Load from empty directory (no config file exists)
        let (config, warning) = load_with_override(Some(base_dir));
        assert!(warning.is_none(), "should not warn for missing file");
        assert_eq!(config.language, Config::default().language);
    }

    #[test]
    fn load_with_override_from_corrupted_file_returns_default_with_warning() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let base_dir = temp_dir.path().to_path_buf();

        // Create corrupted config file
        let config_path = base_dir.join("settings.toml");
        fs::write(&config_path, "not = valid = toml").expect("write file");

        // Load should return default with warning
        let (config, warning) = load_with_override(Some(base_dir));
        assert!(warning.is_some(), "should warn about parse error");
        assert_eq!(
            warning.unwrap(),
            "notification-config-load-error".to_string()
        );
        assert_eq!(config.language, Config::default().language);
    }

    #[test]
    fn multiple_isolated_config_tests_dont_interfere() {
        // Test 1: Save config A
        let temp_dir_a = tempdir().expect("create temp dir A");
        let config_a = Config {
            language: Some("fr".to_string()),
            ..Config::default()
        };
        save_with_override(&config_a, Some(temp_dir_a.path().to_path_buf()))
            .expect("save A should succeed");

        // Test 2: Save config B (different directory)
        let temp_dir_b = tempdir().expect("create temp dir B");
        let config_b = Config {
            language: Some("es".to_string()),
            ..Config::default()
        };
        save_with_override(&config_b, Some(temp_dir_b.path().to_path_buf()))
            .expect("save B should succeed");

        // Verify they are independent
        let (loaded_a, _) = load_with_override(Some(temp_dir_a.path().to_path_buf()));
        let (loaded_b, _) = load_with_override(Some(temp_dir_b.path().to_path_buf()));

        assert_eq!(loaded_a.language, Some("fr".to_string()));
        assert_eq!(loaded_b.language, Some("es".to_string()));
    }

    #[test]
    fn save_with_override_creates_parent_directories() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let nested_dir = temp_dir.path().join("nested").join("deeply");

        let config = Config::default();

        // Save should create nested directories
        save_with_override(&config, Some(nested_dir.clone())).expect("save should succeed");
        assert!(nested_dir.join("settings.toml").exists());
    }
}
