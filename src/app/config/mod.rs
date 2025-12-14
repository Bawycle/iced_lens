// SPDX-License-Identifier: MPL-2.0
//! This module handles the application's configuration, including loading and saving
//! user preferences to a `settings.toml` file.
//!
//! # Configuration Sections
//!
//! The configuration is organized into logical sections:
//! - `[general]` - Language and theme mode
//! - `[display]` - Viewer display settings (zoom, background, sorting)
//! - `[video]` - Video playback settings (volume, caching, seek step)
//! - `[fullscreen]` - Fullscreen overlay settings
//!
//! # Path Resolution
//!
//! The config file location can be customized for testing or portable deployments:
//! 1. Use `load_from_path()`/`save_to_path()` with explicit path
//! 2. Set `ICED_LENS_CONFIG_DIR` environment variable
//! 3. Falls back to platform-specific config directory
//!
//! # Migration
//!
//! Old flat config files (pre-0.3.0) are automatically migrated to the new
//! sectioned format when loaded. The next save will write the new format.
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
//! config.general.language = Some("fr".to_string());
//!
//! // Save the modified configuration
//! config::save(&config).expect("Failed to save config");
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

// =============================================================================
// Enums (shared between sections)
// =============================================================================

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

// =============================================================================
// Section Structs
// =============================================================================

/// General application settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeneralConfig {
    /// UI language code (e.g., "en-US", "fr").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Application theme mode (light, dark, or system).
    #[serde(
        default = "default_theme_mode",
        deserialize_with = "deserialize_theme_mode"
    )]
    pub theme_mode: ThemeMode,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            language: None,
            theme_mode: default_theme_mode(),
        }
    }
}

/// Display and viewer settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DisplayConfig {
    /// Whether to fit images to window by default.
    #[serde(
        default = "default_fit_to_window",
        skip_serializing_if = "Option::is_none"
    )]
    pub fit_to_window: Option<bool>,

    /// Zoom step percentage for zoom in/out.
    #[serde(default = "default_zoom_step", skip_serializing_if = "Option::is_none")]
    pub zoom_step: Option<f32>,

    /// Viewer background theme.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_theme: Option<BackgroundTheme>,

    /// Media file sorting order in directory.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<SortOrder>,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            fit_to_window: Some(true),
            zoom_step: Some(DEFAULT_ZOOM_STEP_PERCENT),
            background_theme: Some(BackgroundTheme::default()),
            sort_order: Some(SortOrder::default()),
        }
    }
}

/// Video playback settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VideoConfig {
    /// Auto-play videos when loaded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub autoplay: Option<bool>,

    /// Playback volume (0.0 to 1.0).
    #[serde(default = "default_volume", skip_serializing_if = "Option::is_none")]
    pub volume: Option<f32>,

    /// Whether audio is muted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub muted: Option<bool>,

    /// Whether playback should loop.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loop_enabled: Option<bool>,

    /// Normalize audio volume across different media files.
    #[serde(
        default = "default_audio_normalization",
        skip_serializing_if = "Option::is_none"
    )]
    pub audio_normalization: Option<bool>,

    /// Frame cache size in megabytes for seek performance.
    #[serde(
        default = "default_frame_cache_mb",
        skip_serializing_if = "Option::is_none"
    )]
    pub frame_cache_mb: Option<u32>,

    /// Frame history size in megabytes for backward stepping.
    #[serde(
        default = "default_frame_history_mb",
        skip_serializing_if = "Option::is_none"
    )]
    pub frame_history_mb: Option<u32>,

    /// Keyboard seek step in seconds (arrow keys).
    #[serde(
        default = "default_keyboard_seek_step_secs",
        skip_serializing_if = "Option::is_none"
    )]
    pub keyboard_seek_step_secs: Option<f64>,
}

impl Default for VideoConfig {
    fn default() -> Self {
        Self {
            autoplay: Some(false),
            volume: Some(DEFAULT_VOLUME),
            muted: Some(false),
            loop_enabled: Some(false),
            audio_normalization: default_audio_normalization(),
            frame_cache_mb: default_frame_cache_mb(),
            frame_history_mb: default_frame_history_mb(),
            keyboard_seek_step_secs: default_keyboard_seek_step_secs(),
        }
    }
}

/// Fullscreen overlay settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FullscreenConfig {
    /// Auto-hide timeout for fullscreen controls (seconds).
    #[serde(
        default = "default_overlay_timeout_secs",
        skip_serializing_if = "Option::is_none"
    )]
    pub overlay_timeout_secs: Option<u32>,
}

impl Default for FullscreenConfig {
    fn default() -> Self {
        Self {
            overlay_timeout_secs: Some(DEFAULT_OVERLAY_TIMEOUT_SECS),
        }
    }
}

// =============================================================================
// Main Config Struct (Sectioned)
// =============================================================================

/// Application configuration with logical sections.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Config {
    /// General application settings.
    #[serde(default)]
    pub general: GeneralConfig,

    /// Display and viewer settings.
    #[serde(default)]
    pub display: DisplayConfig,

    /// Video playback settings.
    #[serde(default)]
    pub video: VideoConfig,

    /// Fullscreen overlay settings.
    #[serde(default)]
    pub fullscreen: FullscreenConfig,
}

// =============================================================================
// Legacy Config (for migration from flat format)
// =============================================================================

/// Legacy flat configuration format (pre-0.3.0).
/// Used for automatic migration of old config files.
#[derive(Debug, Deserialize)]
struct LegacyConfig {
    language: Option<String>,
    #[serde(default)]
    fit_to_window: Option<bool>,
    #[serde(default)]
    zoom_step: Option<f32>,
    #[serde(default)]
    background_theme: Option<BackgroundTheme>,
    #[serde(default)]
    sort_order: Option<SortOrder>,
    #[serde(default)]
    overlay_timeout_secs: Option<u32>,
    #[serde(
        default = "default_theme_mode",
        deserialize_with = "deserialize_theme_mode"
    )]
    theme_mode: ThemeMode,
    #[serde(default)]
    video_autoplay: Option<bool>,
    #[serde(default)]
    video_volume: Option<f32>,
    #[serde(default)]
    video_muted: Option<bool>,
    #[serde(default)]
    video_loop: Option<bool>,
    #[serde(default = "default_audio_normalization")]
    audio_normalization: Option<bool>,
    #[serde(default = "default_frame_cache_mb")]
    frame_cache_mb: Option<u32>,
    #[serde(default = "default_frame_history_mb")]
    frame_history_mb: Option<u32>,
    #[serde(default = "default_keyboard_seek_step_secs")]
    keyboard_seek_step_secs: Option<f64>,
}

impl From<LegacyConfig> for Config {
    fn from(legacy: LegacyConfig) -> Self {
        Config {
            general: GeneralConfig {
                language: legacy.language,
                theme_mode: legacy.theme_mode,
            },
            display: DisplayConfig {
                fit_to_window: legacy.fit_to_window,
                zoom_step: legacy.zoom_step,
                background_theme: legacy.background_theme,
                sort_order: legacy.sort_order,
            },
            video: VideoConfig {
                autoplay: legacy.video_autoplay,
                volume: legacy.video_volume,
                muted: legacy.video_muted,
                loop_enabled: legacy.video_loop,
                audio_normalization: legacy.audio_normalization,
                frame_cache_mb: legacy.frame_cache_mb,
                frame_history_mb: legacy.frame_history_mb,
                keyboard_seek_step_secs: legacy.keyboard_seek_step_secs,
            },
            fullscreen: FullscreenConfig {
                overlay_timeout_secs: legacy.overlay_timeout_secs,
            },
        }
    }
}

// =============================================================================
// Default Value Functions
// =============================================================================

fn default_theme_mode() -> ThemeMode {
    ThemeMode::System
}

fn default_fit_to_window() -> Option<bool> {
    Some(true)
}

fn default_zoom_step() -> Option<f32> {
    Some(DEFAULT_ZOOM_STEP_PERCENT)
}

fn default_volume() -> Option<f32> {
    Some(DEFAULT_VOLUME)
}

fn default_audio_normalization() -> Option<bool> {
    Some(true)
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

fn default_overlay_timeout_secs() -> Option<u32> {
    Some(DEFAULT_OVERLAY_TIMEOUT_SECS)
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

// =============================================================================
// Config Path Resolution
// =============================================================================

/// Returns the config file path with an optional override.
fn get_config_path_with_override(base_dir: Option<PathBuf>) -> Option<PathBuf> {
    paths::get_app_config_dir_with_override(base_dir).map(|mut path| {
        path.push(CONFIG_FILE);
        path
    })
}

// =============================================================================
// Load Functions
// =============================================================================

/// Loads the configuration from the default path.
///
/// Returns a tuple of (config, optional_warning). If loading fails, returns
/// default config with a warning message explaining what went wrong.
pub fn load() -> (Config, Option<String>) {
    load_with_override(None)
}

/// Loads the configuration from a custom directory.
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

/// Loads configuration from a specific path.
///
/// Automatically migrates legacy flat format to new sectioned format.
pub fn load_from_path(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path)?;

    // Try parsing as new sectioned format first
    if let Ok(config) = toml::from_str::<Config>(&content) {
        // Check if this looks like a valid sectioned config
        // (has at least one section table)
        if content.contains("[general]")
            || content.contains("[display]")
            || content.contains("[video]")
            || content.contains("[fullscreen]")
        {
            return Ok(config);
        }
    }

    // Try parsing as legacy flat format
    if let Ok(legacy) = toml::from_str::<LegacyConfig>(&content) {
        return Ok(Config::from(legacy));
    }

    // If neither works, try new format again and let errors propagate
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

// =============================================================================
// Save Functions
// =============================================================================

/// Saves the configuration to the default path.
pub fn save(config: &Config) -> Result<()> {
    save_with_override(config, None)
}

/// Saves the configuration to a custom directory.
pub fn save_with_override(config: &Config, base_dir: Option<PathBuf>) -> Result<()> {
    if let Some(path) = get_config_path_with_override(base_dir) {
        return save_to_path(config, &path);
    }
    Ok(())
}

/// Saves configuration to a specific path.
pub fn save_to_path(config: &Config, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(config).map_err(Error::from)?;
    fs::write(path, content)?;
    Ok(())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use tempfile::tempdir;

    #[test]
    fn save_and_load_round_trip_preserves_settings() {
        let config = Config {
            general: GeneralConfig {
                language: Some("fr".to_string()),
                theme_mode: ThemeMode::Light,
            },
            display: DisplayConfig {
                fit_to_window: Some(false),
                zoom_step: Some(5.0),
                background_theme: Some(BackgroundTheme::Light),
                sort_order: Some(SortOrder::Alphabetical),
            },
            video: VideoConfig {
                autoplay: Some(false),
                volume: Some(DEFAULT_VOLUME),
                muted: Some(false),
                loop_enabled: Some(false),
                audio_normalization: Some(true),
                frame_cache_mb: Some(DEFAULT_FRAME_CACHE_MB),
                frame_history_mb: Some(DEFAULT_FRAME_HISTORY_MB),
                keyboard_seek_step_secs: Some(DEFAULT_KEYBOARD_SEEK_STEP_SECS),
            },
            fullscreen: FullscreenConfig {
                overlay_timeout_secs: Some(DEFAULT_OVERLAY_TIMEOUT_SECS),
            },
        };
        let temp_dir = tempdir().expect("failed to create temp dir");
        let config_path = temp_dir.path().join("nested").join("settings.toml");

        save_to_path(&config, &config_path).expect("failed to save config");
        let loaded = load_from_path(&config_path).expect("failed to load config");

        assert_eq!(loaded.general.language, config.general.language);
        assert_eq!(loaded.display.fit_to_window, config.display.fit_to_window);
        assert_eq!(loaded.display.zoom_step, config.display.zoom_step);
        assert_eq!(loaded.general.theme_mode, config.general.theme_mode);
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
            general: GeneralConfig {
                language: Some("en-US".to_string()),
                theme_mode: ThemeMode::System,
            },
            display: DisplayConfig {
                fit_to_window: Some(false),
                zoom_step: Some(7.5),
                background_theme: Some(BackgroundTheme::Checkerboard),
                sort_order: Some(SortOrder::CreatedDate),
            },
            video: VideoConfig {
                autoplay: Some(true),
                volume: Some(0.5),
                muted: Some(true),
                loop_enabled: Some(true),
                audio_normalization: Some(false),
                frame_cache_mb: Some(128),
                frame_history_mb: Some(DEFAULT_FRAME_HISTORY_MB),
                keyboard_seek_step_secs: Some(DEFAULT_KEYBOARD_SEEK_STEP_SECS),
            },
            fullscreen: FullscreenConfig {
                overlay_timeout_secs: Some(DEFAULT_OVERLAY_TIMEOUT_SECS),
            },
        };

        save_to_path(&config, &config_path).expect("save should create directories");
        assert!(config_path.exists());
    }

    #[test]
    fn default_config_has_expected_values() {
        let config = Config::default();
        assert_eq!(config.display.fit_to_window, Some(true));
        assert_eq!(config.display.zoom_step, Some(DEFAULT_ZOOM_STEP_PERCENT));
        assert_eq!(
            config.display.background_theme,
            Some(BackgroundTheme::default())
        );
        assert_eq!(config.display.sort_order, Some(SortOrder::default()));
        assert_eq!(config.general.theme_mode, ThemeMode::System);
        assert_eq!(config.video.autoplay, Some(false));
        assert_eq!(config.video.volume, Some(DEFAULT_VOLUME));
        assert_eq!(config.video.muted, Some(false));
        assert_eq!(config.video.audio_normalization, Some(true));
        assert_eq!(config.video.frame_cache_mb, Some(DEFAULT_FRAME_CACHE_MB));
    }

    #[test]
    fn save_and_load_preserves_sort_order() {
        let config = Config {
            display: DisplayConfig {
                sort_order: Some(SortOrder::ModifiedDate),
                ..DisplayConfig::default()
            },
            ..Config::default()
        };
        let temp_dir = tempdir().expect("failed to create temp dir");
        let config_path = temp_dir.path().join("settings.toml");

        save_to_path(&config, &config_path).expect("failed to save config");
        let loaded = load_from_path(&config_path).expect("failed to load config");

        assert_eq!(loaded.display.sort_order, Some(SortOrder::ModifiedDate));
    }

    #[test]
    fn sort_order_default_is_alphabetical() {
        assert_eq!(SortOrder::default(), SortOrder::Alphabetical);
    }

    #[test]
    fn default_config_sets_overlay_timeout() {
        let config = Config::default();
        assert_eq!(
            config.fullscreen.overlay_timeout_secs,
            Some(DEFAULT_OVERLAY_TIMEOUT_SECS)
        );
        assert_eq!(DEFAULT_OVERLAY_TIMEOUT_SECS, 3);
    }

    #[test]
    fn save_and_load_preserves_overlay_timeout() {
        let config = Config {
            fullscreen: FullscreenConfig {
                overlay_timeout_secs: Some(5),
            },
            ..Config::default()
        };
        let temp_dir = tempdir().expect("failed to create temp dir");
        let config_path = temp_dir.path().join("settings.toml");

        save_to_path(&config, &config_path).expect("failed to save config");
        let loaded = load_from_path(&config_path).expect("failed to load config");

        assert_eq!(loaded.fullscreen.overlay_timeout_secs, Some(5));
    }

    #[test]
    fn overlay_timeout_bounds_are_reasonable() {
        assert_eq!(MIN_OVERLAY_TIMEOUT_SECS, 1);
        assert_eq!(MAX_OVERLAY_TIMEOUT_SECS, 30);
    }

    #[test]
    fn save_and_load_preserves_audio_settings() {
        let config = Config {
            video: VideoConfig {
                autoplay: Some(true),
                volume: Some(0.65),
                muted: Some(true),
                loop_enabled: Some(true),
                audio_normalization: Some(false),
                ..VideoConfig::default()
            },
            ..Config::default()
        };
        let temp_dir = tempdir().expect("failed to create temp dir");
        let config_path = temp_dir.path().join("settings.toml");

        save_to_path(&config, &config_path).expect("failed to save config");
        let loaded = load_from_path(&config_path).expect("failed to load config");

        assert_eq!(loaded.video.volume, Some(0.65));
        assert_eq!(loaded.video.muted, Some(true));
        assert_eq!(loaded.video.loop_enabled, Some(true));
        assert_eq!(loaded.video.audio_normalization, Some(false));
    }

    #[test]
    fn audio_normalization_defaults_to_true() {
        let config = Config::default();
        assert_eq!(config.video.audio_normalization, Some(true));
    }

    #[test]
    fn volume_constants_are_valid() {
        assert_eq!(DEFAULT_VOLUME, 0.8);
        assert_eq!(MIN_VOLUME, 0.0);
        assert_eq!(MAX_VOLUME, 1.0);
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
            general: GeneralConfig {
                language: Some("de".to_string()),
                theme_mode: ThemeMode::Dark,
            },
            display: DisplayConfig {
                fit_to_window: Some(false),
                zoom_step: Some(15.0),
                background_theme: Some(BackgroundTheme::Light),
                sort_order: Some(SortOrder::CreatedDate),
            },
            video: VideoConfig {
                autoplay: Some(true),
                volume: Some(0.5),
                muted: Some(true),
                loop_enabled: Some(true),
                audio_normalization: Some(false),
                frame_cache_mb: Some(256),
                frame_history_mb: Some(64),
                keyboard_seek_step_secs: Some(5.0),
            },
            fullscreen: FullscreenConfig {
                overlay_timeout_secs: Some(7),
            },
        };

        save_with_override(&config, Some(base_dir.clone())).expect("save should succeed");

        let expected_path = base_dir.join("settings.toml");
        assert!(expected_path.exists(), "config file should exist");

        let (loaded, warning) = load_with_override(Some(base_dir));
        assert!(warning.is_none(), "load should succeed without warning");
        assert_eq!(loaded.general.language, Some("de".to_string()));
        assert_eq!(loaded.display.zoom_step, Some(15.0));
        assert_eq!(loaded.general.theme_mode, ThemeMode::Dark);
    }

    #[test]
    fn load_with_override_from_empty_directory_returns_default() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let base_dir = temp_dir.path().to_path_buf();

        let (config, warning) = load_with_override(Some(base_dir));
        assert!(warning.is_none(), "should not warn for missing file");
        assert_eq!(config.general.language, Config::default().general.language);
    }

    #[test]
    fn load_with_override_from_corrupted_file_returns_default_with_warning() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let base_dir = temp_dir.path().to_path_buf();

        let config_path = base_dir.join("settings.toml");
        fs::write(&config_path, "not = valid = toml").expect("write file");

        let (config, warning) = load_with_override(Some(base_dir));
        assert!(warning.is_some(), "should warn about parse error");
        assert_eq!(
            warning.unwrap(),
            "notification-config-load-error".to_string()
        );
        assert_eq!(config.general.language, Config::default().general.language);
    }

    #[test]
    fn multiple_isolated_config_tests_dont_interfere() {
        let temp_dir_a = tempdir().expect("create temp dir A");
        let config_a = Config {
            general: GeneralConfig {
                language: Some("fr".to_string()),
                ..GeneralConfig::default()
            },
            ..Config::default()
        };
        save_with_override(&config_a, Some(temp_dir_a.path().to_path_buf()))
            .expect("save A should succeed");

        let temp_dir_b = tempdir().expect("create temp dir B");
        let config_b = Config {
            general: GeneralConfig {
                language: Some("es".to_string()),
                ..GeneralConfig::default()
            },
            ..Config::default()
        };
        save_with_override(&config_b, Some(temp_dir_b.path().to_path_buf()))
            .expect("save B should succeed");

        let (loaded_a, _) = load_with_override(Some(temp_dir_a.path().to_path_buf()));
        let (loaded_b, _) = load_with_override(Some(temp_dir_b.path().to_path_buf()));

        assert_eq!(loaded_a.general.language, Some("fr".to_string()));
        assert_eq!(loaded_b.general.language, Some("es".to_string()));
    }

    #[test]
    fn save_with_override_creates_parent_directories() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let nested_dir = temp_dir.path().join("nested").join("deeply");

        let config = Config::default();

        save_with_override(&config, Some(nested_dir.clone())).expect("save should succeed");
        assert!(nested_dir.join("settings.toml").exists());
    }

    // =========================================================================
    // Migration Tests
    // =========================================================================

    #[test]
    fn migrate_legacy_flat_config() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let config_path = temp_dir.path().join("settings.toml");

        // Write legacy flat format
        let legacy_content = r#"
language = "fr"
fit_to_window = true
zoom_step = 15.0
background_theme = "light"
sort_order = "modified-date"
overlay_timeout_secs = 5
theme_mode = "dark"
video_autoplay = true
video_volume = 0.7
video_muted = true
video_loop = true
audio_normalization = false
frame_cache_mb = 128
frame_history_mb = 256
keyboard_seek_step_secs = 5.0
"#;
        fs::write(&config_path, legacy_content).expect("write legacy config");

        // Load should migrate to new format
        let loaded = load_from_path(&config_path).expect("should load legacy config");

        // Verify migration
        assert_eq!(loaded.general.language, Some("fr".to_string()));
        assert_eq!(loaded.general.theme_mode, ThemeMode::Dark);
        assert_eq!(loaded.display.fit_to_window, Some(true));
        assert_eq!(loaded.display.zoom_step, Some(15.0));
        assert_eq!(
            loaded.display.background_theme,
            Some(BackgroundTheme::Light)
        );
        assert_eq!(loaded.display.sort_order, Some(SortOrder::ModifiedDate));
        assert_eq!(loaded.video.autoplay, Some(true));
        assert_eq!(loaded.video.volume, Some(0.7));
        assert_eq!(loaded.video.muted, Some(true));
        assert_eq!(loaded.video.loop_enabled, Some(true));
        assert_eq!(loaded.video.audio_normalization, Some(false));
        assert_eq!(loaded.video.frame_cache_mb, Some(128));
        assert_eq!(loaded.video.frame_history_mb, Some(256));
        assert_eq!(loaded.video.keyboard_seek_step_secs, Some(5.0));
        assert_eq!(loaded.fullscreen.overlay_timeout_secs, Some(5));
    }

    #[test]
    fn new_sectioned_format_loads_correctly() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let config_path = temp_dir.path().join("settings.toml");

        // Write new sectioned format
        let sectioned_content = r#"
[general]
language = "de"
theme_mode = "light"

[display]
fit_to_window = false
zoom_step = 20.0
background_theme = "checkerboard"
sort_order = "created-date"

[video]
autoplay = true
volume = 0.9
muted = false
loop_enabled = true
audio_normalization = true
frame_cache_mb = 256
frame_history_mb = 512
keyboard_seek_step_secs = 10.0

[fullscreen]
overlay_timeout_secs = 10
"#;
        fs::write(&config_path, sectioned_content).expect("write sectioned config");

        let loaded = load_from_path(&config_path).expect("should load sectioned config");

        assert_eq!(loaded.general.language, Some("de".to_string()));
        assert_eq!(loaded.general.theme_mode, ThemeMode::Light);
        assert_eq!(loaded.display.fit_to_window, Some(false));
        assert_eq!(loaded.display.zoom_step, Some(20.0));
        assert_eq!(
            loaded.display.background_theme,
            Some(BackgroundTheme::Checkerboard)
        );
        assert_eq!(loaded.display.sort_order, Some(SortOrder::CreatedDate));
        assert_eq!(loaded.video.autoplay, Some(true));
        assert_eq!(loaded.video.volume, Some(0.9));
        assert_eq!(loaded.video.loop_enabled, Some(true));
        assert_eq!(loaded.fullscreen.overlay_timeout_secs, Some(10));
    }

    #[test]
    fn saved_config_uses_sectioned_format() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let config_path = temp_dir.path().join("settings.toml");

        let config = Config::default();
        save_to_path(&config, &config_path).expect("save config");

        let content = fs::read_to_string(&config_path).expect("read config");

        // Verify sectioned format is used
        assert!(
            content.contains("[general]"),
            "should have [general] section"
        );
        assert!(
            content.contains("[display]"),
            "should have [display] section"
        );
        assert!(content.contains("[video]"), "should have [video] section");
        assert!(
            content.contains("[fullscreen]"),
            "should have [fullscreen] section"
        );
    }

    #[test]
    fn legacy_config_is_upgraded_on_resave() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let config_path = temp_dir.path().join("settings.toml");

        // Write legacy format
        let legacy_content = r#"
language = "ja"
fit_to_window = false
zoom_step = 25.0
"#;
        fs::write(&config_path, legacy_content).expect("write legacy config");

        // Load (migrates to new format in memory)
        let loaded = load_from_path(&config_path).expect("load legacy config");
        assert_eq!(loaded.general.language, Some("ja".to_string()));

        // Save (writes new format)
        save_to_path(&loaded, &config_path).expect("save migrated config");

        // Verify new format is written
        let content = fs::read_to_string(&config_path).expect("read config");
        assert!(
            content.contains("[general]"),
            "should have [general] section"
        );
        assert!(
            content.contains("language = \"ja\""),
            "should have language in general section"
        );
    }
}
