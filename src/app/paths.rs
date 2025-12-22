// SPDX-License-Identifier: MPL-2.0
//! Centralized path management for application directories.
//!
//! This module provides a single source of truth for application data paths,
//! ensuring consistent directory usage across all components.
//!
//! # Path Resolution Order
//!
//! Paths are resolved in the following priority order:
//! 1. **Explicit override** - parameter to `_with_override()` functions (for tests)
//! 2. **CLI arguments** (`--data-dir`, `--config-dir`) - set via [`init_cli_overrides`]
//! 3. **Environment variables** (`ICED_LENS_DATA_DIR`, `ICED_LENS_CONFIG_DIR`)
//! 4. **Platform default** - via `dirs` crate
//!
//! The explicit override has highest priority because it's the most specific -
//! when code explicitly passes a path, it should always be respected.
//!
//! # Usage
//!
//! CLI overrides should be initialized once at startup:
//! ```ignore
//! paths::init_cli_overrides(flags.data_dir, flags.config_dir);
//! ```
//!
//! After initialization, all path functions will respect the CLI overrides
//! (unless an explicit override is passed).

use std::path::PathBuf;
use std::sync::OnceLock;

/// Application name used for directory naming.
const APP_NAME: &str = "IcedLens";

/// Environment variable to override the data directory.
pub const ENV_DATA_DIR: &str = "ICED_LENS_DATA_DIR";

/// Environment variable to override the config directory.
pub const ENV_CONFIG_DIR: &str = "ICED_LENS_CONFIG_DIR";

/// Global CLI override for data directory (set once at startup).
static CLI_DATA_DIR: OnceLock<Option<PathBuf>> = OnceLock::new();

/// Global CLI override for config directory (set once at startup).
static CLI_CONFIG_DIR: OnceLock<Option<PathBuf>> = OnceLock::new();

/// Initializes CLI overrides for data and config directories.
///
/// This should be called once at application startup, before any path
/// resolution functions are called. The CLI overrides take highest priority.
///
/// # Arguments
///
/// * `data_dir` - Optional data directory from `--data-dir` CLI argument
/// * `config_dir` - Optional config directory from `--config-dir` CLI argument
///
/// # Panics
///
/// Panics if called more than once (OnceLock can only be set once).
pub fn init_cli_overrides(data_dir: Option<String>, config_dir: Option<String>) {
    CLI_DATA_DIR
        .set(data_dir.map(PathBuf::from))
        .expect("CLI data dir override already initialized");
    CLI_CONFIG_DIR
        .set(config_dir.map(PathBuf::from))
        .expect("CLI config dir override already initialized");
}

/// Returns the CLI override for data directory, if set.
fn get_cli_data_dir() -> Option<PathBuf> {
    CLI_DATA_DIR.get().and_then(Clone::clone)
}

/// Returns the CLI override for config directory, if set.
fn get_cli_config_dir() -> Option<PathBuf> {
    CLI_CONFIG_DIR.get().and_then(Clone::clone)
}

/// Returns the application data directory path.
///
/// This directory is used for storing application state (not user preferences).
/// User preferences are stored separately in the config directory via `config::load/save`.
///
/// # Resolution Order
///
/// 1. CLI argument `--data-dir` (if set via [`init_cli_overrides`])
/// 2. `ICED_LENS_DATA_DIR` environment variable (if set and non-empty)
/// 3. Platform-specific data directory:
///    - Linux: `~/.local/share/IcedLens/`
///    - macOS: `~/Library/Application Support/IcedLens/`
///    - Windows: `C:\Users\<User>\AppData\Roaming\IcedLens\`
///
/// Returns `None` if the data directory cannot be determined (rare edge case).
pub fn get_app_data_dir() -> Option<PathBuf> {
    get_app_data_dir_with_override(None)
}

/// Returns the application data directory path with an optional override.
///
/// # Resolution Order
///
/// 1. `override_path` parameter (if `Some`) - most specific, for tests
/// 2. CLI argument `--data-dir` (if set via [`init_cli_overrides`])
/// 3. `ICED_LENS_DATA_DIR` environment variable (if set and non-empty)
/// 4. Platform-specific data directory (with app name appended)
///
/// # Arguments
///
/// * `override_path` - Optional path to use instead of default. Takes highest priority.
pub fn get_app_data_dir_with_override(override_path: Option<PathBuf>) -> Option<PathBuf> {
    // Priority 1: Explicit override (for tests)
    if let Some(path) = override_path {
        return Some(path);
    }

    // Priority 2: CLI argument
    if let Some(path) = get_cli_data_dir() {
        return Some(path);
    }

    // Priority 3: Environment variable
    if let Ok(env_path) = std::env::var(ENV_DATA_DIR) {
        if !env_path.is_empty() {
            return Some(PathBuf::from(env_path));
        }
    }

    // Priority 4: Platform default with app name
    dirs::data_dir().map(|mut path| {
        path.push(APP_NAME);
        path
    })
}

/// Returns the application config directory path.
///
/// This directory is used for storing user preferences (settings.toml).
///
/// # Resolution Order
///
/// 1. CLI argument `--config-dir` (if set via [`init_cli_overrides`])
/// 2. `ICED_LENS_CONFIG_DIR` environment variable (if set and non-empty)
/// 3. Platform-specific config directory:
///    - Linux: `~/.config/IcedLens/`
///    - macOS: `~/Library/Application Support/IcedLens/`
///    - Windows: `C:\Users\<User>\AppData\Roaming\IcedLens\`
///
/// Returns `None` if the config directory cannot be determined (rare edge case).
pub fn get_app_config_dir() -> Option<PathBuf> {
    get_app_config_dir_with_override(None)
}

/// Returns the application config directory path with an optional override.
///
/// # Resolution Order
///
/// 1. `override_path` parameter (if `Some`) - most specific, for tests
/// 2. CLI argument `--config-dir` (if set via [`init_cli_overrides`])
/// 3. `ICED_LENS_CONFIG_DIR` environment variable (if set and non-empty)
/// 4. Platform-specific config directory (with app name appended)
///
/// # Arguments
///
/// * `override_path` - Optional path to use instead of default. Takes highest priority.
pub fn get_app_config_dir_with_override(override_path: Option<PathBuf>) -> Option<PathBuf> {
    // Priority 1: Explicit override (for tests)
    if let Some(path) = override_path {
        return Some(path);
    }

    // Priority 2: CLI argument
    if let Some(path) = get_cli_config_dir() {
        return Some(path);
    }

    // Priority 3: Environment variable
    if let Ok(env_path) = std::env::var(ENV_CONFIG_DIR) {
        if !env_path.is_empty() {
            return Some(PathBuf::from(env_path));
        }
    }

    // Priority 4: Platform default with app name
    dirs::config_dir().map(|mut path| {
        path.push(APP_NAME);
        path
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Mutex to prevent parallel tests from interfering with each other's env vars
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn app_data_dir_contains_app_name() {
        let _lock = ENV_MUTEX.lock().unwrap();
        // Clear env var to test default behavior
        std::env::remove_var(ENV_DATA_DIR);

        if let Some(path) = get_app_data_dir() {
            assert!(
                path.to_string_lossy().contains(APP_NAME),
                "App data dir should contain app name"
            );
        }
        // If dirs::data_dir() returns None (rare), the test passes silently
    }

    #[test]
    fn app_data_dir_is_absolute() {
        let _lock = ENV_MUTEX.lock().unwrap();
        std::env::remove_var(ENV_DATA_DIR);

        if let Some(path) = get_app_data_dir() {
            assert!(path.is_absolute(), "App data dir should be absolute path");
        }
    }

    #[test]
    fn app_config_dir_contains_app_name() {
        let _lock = ENV_MUTEX.lock().unwrap();
        std::env::remove_var(ENV_CONFIG_DIR);

        if let Some(path) = get_app_config_dir() {
            assert!(
                path.to_string_lossy().contains(APP_NAME),
                "App config dir should contain app name"
            );
        }
    }

    #[test]
    fn app_config_dir_is_absolute() {
        let _lock = ENV_MUTEX.lock().unwrap();
        std::env::remove_var(ENV_CONFIG_DIR);

        if let Some(path) = get_app_config_dir() {
            assert!(path.is_absolute(), "App config dir should be absolute path");
        }
    }

    #[test]
    fn override_path_takes_precedence_for_data_dir() {
        let override_path = PathBuf::from("/custom/data/path");
        let result = get_app_data_dir_with_override(Some(override_path.clone()));
        assert_eq!(result, Some(override_path));
    }

    #[test]
    fn override_path_takes_precedence_for_config_dir() {
        let override_path = PathBuf::from("/custom/config/path");
        let result = get_app_config_dir_with_override(Some(override_path.clone()));
        assert_eq!(result, Some(override_path));
    }

    #[test]
    fn env_var_overrides_default_data_dir() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let test_path = "/test/data/dir";
        std::env::set_var(ENV_DATA_DIR, test_path);

        let result = get_app_data_dir();
        assert_eq!(result, Some(PathBuf::from(test_path)));

        // Cleanup
        std::env::remove_var(ENV_DATA_DIR);
    }

    #[test]
    fn env_var_overrides_default_config_dir() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let test_path = "/test/config/dir";
        std::env::set_var(ENV_CONFIG_DIR, test_path);

        let result = get_app_config_dir();
        assert_eq!(result, Some(PathBuf::from(test_path)));

        // Cleanup
        std::env::remove_var(ENV_CONFIG_DIR);
    }

    #[test]
    fn empty_env_var_uses_default() {
        let _lock = ENV_MUTEX.lock().unwrap();
        std::env::set_var(ENV_DATA_DIR, "");

        let result = get_app_data_dir();
        // Should fall back to platform default which contains app name
        if let Some(path) = result {
            assert!(path.to_string_lossy().contains(APP_NAME));
        }

        std::env::remove_var(ENV_DATA_DIR);
    }

    #[test]
    fn override_path_takes_precedence_over_env_var() {
        let _lock = ENV_MUTEX.lock().unwrap();
        std::env::set_var(ENV_DATA_DIR, "/env/path");

        let override_path = PathBuf::from("/override/path");
        let result = get_app_data_dir_with_override(Some(override_path.clone()));

        assert_eq!(result, Some(override_path));

        std::env::remove_var(ENV_DATA_DIR);
    }
}
