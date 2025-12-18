// SPDX-License-Identifier: MPL-2.0
//! Application state persistence using CBOR format.
//!
//! This module handles transient application state that should persist across sessions
//! but is not user-configurable (unlike preferences in `settings.toml`).
//!
//! State is stored in CBOR (Concise Binary Object Representation) format for:
//! - Compact binary storage
//! - Fast serialization/deserialization
//! - Clear separation from user-editable TOML preferences
//!
//! # Path Resolution
//!
//! The state file location can be customized for testing or portable deployments:
//! 1. Use `load_from()`/`save_to()` with explicit path override
//! 2. Set `ICED_LENS_DATA_DIR` environment variable
//! 3. Falls back to platform-specific data directory

use super::paths;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

/// State file name within the app data directory.
const STATE_FILE: &str = "state.cbor";

/// Application state that persists across sessions.
///
/// This struct contains transient state that improves UX but is not
/// user-configurable. It is stored separately from user preferences.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AppState {
    /// Last directory used for Save As operations.
    /// Used as the initial directory when opening file save dialogs.
    #[serde(default)]
    pub last_save_directory: Option<PathBuf>,

    /// Last directory used for Open File operations.
    /// Used as the initial directory when opening file open dialogs.
    #[serde(default)]
    pub last_open_directory: Option<PathBuf>,

    /// Whether AI deblurring is enabled.
    /// This is application-managed state, not a user preference.
    /// The value depends on whether the model has been successfully downloaded and validated.
    #[serde(default)]
    pub enable_deblur: bool,
}

impl AppState {
    /// Loads application state from the default location.
    ///
    /// Returns a tuple of (state, optional_warning). If loading fails, returns
    /// default state with a warning message explaining what went wrong.
    /// The warning can be displayed to the user via notifications.
    ///
    /// # Path Resolution
    ///
    /// Uses the standard path resolution (see [`paths::get_app_data_dir`]):
    /// 1. `ICED_LENS_DATA_DIR` environment variable (if set)
    /// 2. Platform-specific data directory
    pub fn load() -> (Self, Option<String>) {
        Self::load_from(None)
    }

    /// Loads application state from a custom directory.
    ///
    /// # Arguments
    ///
    /// * `base_dir` - Optional base directory. If `None`, uses default path resolution.
    ///
    /// # Path Resolution
    ///
    /// 1. `base_dir` parameter (if `Some`)
    /// 2. `ICED_LENS_DATA_DIR` environment variable (if set)
    /// 3. Platform-specific data directory
    pub fn load_from(base_dir: Option<PathBuf>) -> (Self, Option<String>) {
        let Some(path) = Self::state_file_path_with_override(base_dir) else {
            return (Self::default(), None);
        };

        if !path.exists() {
            return (Self::default(), None);
        }

        match fs::File::open(&path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                match ciborium::from_reader(reader) {
                    Ok(state) => (state, None),
                    Err(_) => (
                        Self::default(),
                        Some("notification-state-parse-error".to_string()),
                    ),
                }
            }
            Err(_) => (
                Self::default(),
                Some("notification-state-read-error".to_string()),
            ),
        }
    }

    /// Saves application state to the default location.
    ///
    /// Creates the parent directory if it doesn't exist.
    /// Returns an optional warning message if save failed.
    ///
    /// # Path Resolution
    ///
    /// Uses the standard path resolution (see [`paths::get_app_data_dir`]):
    /// 1. `ICED_LENS_DATA_DIR` environment variable (if set)
    /// 2. Platform-specific data directory
    pub fn save(&self) -> Option<String> {
        self.save_to(None)
    }

    /// Saves application state to a custom directory.
    ///
    /// # Arguments
    ///
    /// * `base_dir` - Optional base directory. If `None`, uses default path resolution.
    ///
    /// # Path Resolution
    ///
    /// 1. `base_dir` parameter (if `Some`)
    /// 2. `ICED_LENS_DATA_DIR` environment variable (if set)
    /// 3. Platform-specific data directory
    pub fn save_to(&self, base_dir: Option<PathBuf>) -> Option<String> {
        let Some(path) = Self::state_file_path_with_override(base_dir) else {
            return Some("notification-state-path-error".to_string());
        };

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            if fs::create_dir_all(parent).is_err() {
                return Some("notification-state-dir-error".to_string());
            }
        }

        match fs::File::create(&path) {
            Ok(file) => {
                let writer = BufWriter::new(file);
                if ciborium::into_writer(self, writer).is_err() {
                    return Some("notification-state-write-error".to_string());
                }
                None
            }
            Err(_) => Some("notification-state-create-error".to_string()),
        }
    }

    /// Returns the full path to the state file with optional override.
    fn state_file_path_with_override(base_dir: Option<PathBuf>) -> Option<PathBuf> {
        paths::get_app_data_dir_with_override(base_dir).map(|mut path| {
            path.push(STATE_FILE);
            path
        })
    }

    /// Sets the last save directory from a file path.
    ///
    /// Extracts the parent directory from the given path. If the path has no
    /// parent (e.g., root path), the directory is not updated.
    pub fn set_last_save_directory_from_file(&mut self, file_path: &std::path::Path) {
        if let Some(parent) = file_path.parent() {
            self.last_save_directory = Some(parent.to_path_buf());
        }
    }

    /// Sets the last open directory from a file path.
    ///
    /// Extracts the parent directory from the given path. If the path has no
    /// parent (e.g., root path), the directory is not updated.
    pub fn set_last_open_directory_from_file(&mut self, file_path: &std::path::Path) {
        if let Some(parent) = file_path.parent() {
            self.last_open_directory = Some(parent.to_path_buf());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn default_state_has_no_last_directory() {
        let state = AppState::default();
        assert!(state.last_save_directory.is_none());
    }

    #[test]
    fn set_last_save_directory_extracts_parent() {
        let mut state = AppState::default();
        state
            .set_last_save_directory_from_file(std::path::Path::new("/home/user/images/photo.png"));
        assert_eq!(
            state.last_save_directory,
            Some(PathBuf::from("/home/user/images"))
        );
    }

    #[test]
    fn set_last_save_directory_ignores_root() {
        let mut state = AppState::default();
        state.set_last_save_directory_from_file(std::path::Path::new("/"));
        // Root has no parent, so directory should remain None
        assert!(state.last_save_directory.is_none());
    }

    #[test]
    fn set_last_open_directory_extracts_parent() {
        let mut state = AppState::default();
        state
            .set_last_open_directory_from_file(std::path::Path::new("/home/user/photos/image.jpg"));
        assert_eq!(
            state.last_open_directory,
            Some(PathBuf::from("/home/user/photos"))
        );
    }

    #[test]
    fn set_last_open_directory_ignores_root() {
        let mut state = AppState::default();
        state.set_last_open_directory_from_file(std::path::Path::new("/"));
        // Root has no parent, so directory should remain None
        assert!(state.last_open_directory.is_none());
    }

    #[test]
    fn cbor_round_trip_preserves_state() {
        let temp_dir = tempdir().expect("create temp dir");
        let state_path = temp_dir.path().join("test_state.cbor");

        // Create state with data
        let original = AppState {
            last_save_directory: Some(PathBuf::from("/home/user/documents")),
            last_open_directory: Some(PathBuf::from("/home/user/pictures")),
            enable_deblur: false,
        };

        // Write to CBOR
        {
            let file = fs::File::create(&state_path).expect("create file");
            let writer = BufWriter::new(file);
            ciborium::into_writer(&original, writer).expect("write cbor");
        }

        // Read back
        let loaded: AppState = {
            let file = fs::File::open(&state_path).expect("open file");
            let reader = BufReader::new(file);
            ciborium::from_reader(reader).expect("read cbor")
        };

        assert_eq!(original.last_save_directory, loaded.last_save_directory);
        assert_eq!(original.last_open_directory, loaded.last_open_directory);
    }

    #[test]
    fn load_does_not_panic() {
        // AppState::load() should never panic, even if the file exists
        // or doesn't exist. It should always return a valid AppState.
        // Note: We can't assert field values because the real state file
        // may exist on the developer's machine.
        let _state = AppState::load();
        // If we reach here without panicking, the test passes
    }

    #[test]
    fn save_to_and_load_from_custom_directory() {
        let temp_dir = tempdir().expect("create temp dir");
        let base_dir = temp_dir.path().to_path_buf();

        // Create state with data
        let original = AppState {
            last_save_directory: Some(PathBuf::from("/test/save/directory")),
            last_open_directory: Some(PathBuf::from("/test/open/directory")),
            enable_deblur: true,
        };

        // Save to custom directory
        let save_result = original.save_to(Some(base_dir.clone()));
        assert!(save_result.is_none(), "save should succeed");

        // Verify file was created
        let expected_path = base_dir.join(STATE_FILE);
        assert!(expected_path.exists(), "state file should exist");

        // Load from same custom directory
        let (loaded, warning) = AppState::load_from(Some(base_dir));
        assert!(warning.is_none(), "load should succeed without warning");
        assert_eq!(original, loaded);
    }

    #[test]
    fn load_from_empty_directory_returns_default() {
        let temp_dir = tempdir().expect("create temp dir");
        let base_dir = temp_dir.path().to_path_buf();

        // Load from empty directory (no state file exists)
        let (state, warning) = AppState::load_from(Some(base_dir));
        assert!(warning.is_none(), "should not warn for missing file");
        assert_eq!(state, AppState::default());
    }

    #[test]
    fn load_from_corrupted_file_returns_default_with_warning() {
        let temp_dir = tempdir().expect("create temp dir");
        let base_dir = temp_dir.path().to_path_buf();

        // Create corrupted state file
        let state_path = base_dir.join(STATE_FILE);
        fs::write(&state_path, "not valid cbor data").expect("write file");

        // Load should return default with warning
        let (state, warning) = AppState::load_from(Some(base_dir));
        assert!(warning.is_some(), "should warn about parse error");
        assert_eq!(
            warning.unwrap(),
            "notification-state-parse-error".to_string()
        );
        assert_eq!(state, AppState::default());
    }

    #[test]
    fn multiple_isolated_tests_dont_interfere() {
        // Test 1: Save state A
        let temp_dir_a = tempdir().expect("create temp dir A");
        let state_a = AppState {
            last_save_directory: Some(PathBuf::from("/path/a")),
            last_open_directory: None,
            enable_deblur: false,
        };
        state_a.save_to(Some(temp_dir_a.path().to_path_buf()));

        // Test 2: Save state B (different directory)
        let temp_dir_b = tempdir().expect("create temp dir B");
        let state_b = AppState {
            last_save_directory: Some(PathBuf::from("/path/b")),
            last_open_directory: None,
            enable_deblur: true,
        };
        state_b.save_to(Some(temp_dir_b.path().to_path_buf()));

        // Verify they are independent
        let (loaded_a, _) = AppState::load_from(Some(temp_dir_a.path().to_path_buf()));
        let (loaded_b, _) = AppState::load_from(Some(temp_dir_b.path().to_path_buf()));

        assert_eq!(loaded_a.last_save_directory, Some(PathBuf::from("/path/a")));
        assert_eq!(loaded_b.last_save_directory, Some(PathBuf::from("/path/b")));
    }

    #[test]
    fn save_creates_parent_directories() {
        let temp_dir = tempdir().expect("create temp dir");
        let nested_dir = temp_dir.path().join("nested").join("deeply");

        let state = AppState {
            last_save_directory: Some(PathBuf::from("/test")),
            last_open_directory: None,
            enable_deblur: false,
        };

        // Save should create nested directories
        let result = state.save_to(Some(nested_dir.clone()));
        assert!(result.is_none(), "save should succeed");
        assert!(nested_dir.join(STATE_FILE).exists());
    }
}
