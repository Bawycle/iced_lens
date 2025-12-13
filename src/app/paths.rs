// SPDX-License-Identifier: MPL-2.0
//! Centralized path management for application directories.
//!
//! This module provides a single source of truth for application data paths,
//! ensuring consistent directory usage across all components.

use std::path::PathBuf;

/// Application name used for directory naming.
const APP_NAME: &str = "IcedLens";

/// Returns the application data directory path.
///
/// This directory is used for storing application state (not user preferences).
/// User preferences are stored separately in the config directory via `config::load/save`.
///
/// The path follows platform conventions:
/// - Linux: `~/.local/share/IcedLens/`
/// - macOS: `~/Library/Application Support/IcedLens/`
/// - Windows: `C:\Users\<User>\AppData\Roaming\IcedLens\`
///
/// Returns `None` if the data directory cannot be determined (rare edge case).
pub fn get_app_data_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|mut path| {
        path.push(APP_NAME);
        path
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_data_dir_contains_app_name() {
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
        if let Some(path) = get_app_data_dir() {
            assert!(path.is_absolute(), "App data dir should be absolute path");
        }
    }
}
