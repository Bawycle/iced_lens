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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppState {
    /// Last directory used for Save As operations.
    /// Used as the initial directory when opening file save dialogs.
    #[serde(default)]
    pub last_save_directory: Option<PathBuf>,
}

impl AppState {
    /// Loads application state from disk.
    ///
    /// Returns a tuple of (state, optional_warning). If loading fails, returns
    /// default state with a warning message explaining what went wrong.
    /// The warning can be displayed to the user via notifications.
    pub fn load() -> (Self, Option<String>) {
        let Some(path) = Self::state_file_path() else {
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

    /// Saves application state to disk.
    ///
    /// Creates the parent directory if it doesn't exist.
    /// Returns an optional warning message if save failed.
    pub fn save(&self) -> Option<String> {
        let Some(path) = Self::state_file_path() else {
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

    /// Returns the full path to the state file.
    fn state_file_path() -> Option<PathBuf> {
        paths::get_app_data_dir().map(|mut path| {
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
    fn cbor_round_trip_preserves_state() {
        let temp_dir = tempdir().expect("create temp dir");
        let state_path = temp_dir.path().join("test_state.cbor");

        // Create state with data
        let original = AppState {
            last_save_directory: Some(PathBuf::from("/home/user/documents")),
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
}
