// SPDX-License-Identifier: MPL-2.0
//! Export functionality for diagnostic reports.
//!
//! This module provides file export capabilities with full anonymization
//! of user-identifiable information before writing to disk.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use chrono::Local;

use super::anonymizer::AnonymizationPipeline;
use super::events::{AppOperation, AppStateEvent, DiagnosticEventKind, ErrorEvent, WarningEvent};
use super::report::SerializableEvent;

// =============================================================================
// Export Error
// =============================================================================

/// Maximum clipboard content size in bytes (10 MB).
///
/// Clipboard operations with very large content can cause performance issues
/// or fail on some platforms. This limit provides a reasonable safety margin.
pub const MAX_CLIPBOARD_SIZE_BYTES: usize = 10 * 1024 * 1024;

/// Errors that can occur during diagnostic report export.
#[derive(Debug)]
pub enum ExportError {
    /// I/O error during file operations.
    Io(io::Error),
    /// JSON serialization error.
    Serialization(serde_json::Error),
    /// User cancelled the file dialog.
    Cancelled,
    /// Clipboard access error.
    Clipboard(String),
    /// Content exceeds maximum size for clipboard export.
    ContentTooLarge {
        /// Actual size in bytes.
        size: usize,
        /// Maximum allowed size in bytes.
        max_size: usize,
    },
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::Serialization(err) => write!(f, "serialization error: {err}"),
            Self::Cancelled => write!(f, "export cancelled"),
            Self::Clipboard(msg) => write!(f, "clipboard error: {msg}"),
            #[allow(clippy::cast_precision_loss)] // Precision loss acceptable for display
            Self::ContentTooLarge { size, max_size } => {
                let size_mb = *size as f64 / (1024.0 * 1024.0);
                let max_mb = *max_size as f64 / (1024.0 * 1024.0);
                write!(
                    f,
                    "content too large for clipboard: {size_mb:.1} MB exceeds {max_mb:.1} MB limit"
                )
            }
        }
    }
}

impl std::error::Error for ExportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Serialization(err) => Some(err),
            Self::Cancelled | Self::Clipboard(_) | Self::ContentTooLarge { .. } => None,
        }
    }
}

impl From<io::Error> for ExportError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<serde_json::Error> for ExportError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err)
    }
}

// =============================================================================
// Filename Generation
// =============================================================================

/// Generates a default filename for diagnostic reports.
///
/// Format: `iced_lens_diagnostics_YYYYMMDD_HHMMSS.json`
///
/// Uses local time for user-friendly filenames.
#[must_use]
pub fn generate_default_filename() -> String {
    let now = Local::now();
    format!("iced_lens_diagnostics_{}.json", now.format("%Y%m%d_%H%M%S"))
}

// =============================================================================
// Event Anonymization
// =============================================================================

/// Anonymizes a single event using the provided pipeline.
///
/// Processes string fields in events that may contain PII:
/// - `UserAction::details`
/// - `WarningEvent::message`
/// - `ErrorEvent::message`
/// - `AppStateEvent` string fields (e.g., `MediaFailed.reason`, `VideoError.message`)
#[must_use]
pub fn anonymize_event(
    event: &SerializableEvent,
    pipeline: &AnonymizationPipeline,
) -> SerializableEvent {
    let anonymized_kind = match &event.kind {
        DiagnosticEventKind::UserAction { action, details } => DiagnosticEventKind::UserAction {
            action: action.clone(),
            details: details.as_ref().map(|d| pipeline.anonymize_string(d)),
        },
        DiagnosticEventKind::Warning { event: w } => DiagnosticEventKind::Warning {
            event: WarningEvent {
                message: pipeline.anonymize_string(&w.message),
                warning_type: w.warning_type,
                source_module: w.source_module.clone(),
            },
        },
        DiagnosticEventKind::Error { event: e } => DiagnosticEventKind::Error {
            event: ErrorEvent {
                message: pipeline.anonymize_string(&e.message),
                error_type: e.error_type,
                error_code: e.error_code.clone(),
                source_module: e.source_module.clone(),
            },
        },
        DiagnosticEventKind::AppState { state } => DiagnosticEventKind::AppState {
            state: anonymize_state_event(state, pipeline),
        },
        DiagnosticEventKind::Operation { operation } => DiagnosticEventKind::Operation {
            operation: anonymize_operation(operation, pipeline),
        },
        // ResourceSnapshot doesn't contain user strings
        DiagnosticEventKind::ResourceSnapshot { metrics } => {
            DiagnosticEventKind::ResourceSnapshot {
                metrics: metrics.clone(),
            }
        }
    };

    SerializableEvent {
        timestamp_ms: event.timestamp_ms,
        kind: anonymized_kind,
    }
}

/// Anonymizes string fields in `AppStateEvent` variants.
fn anonymize_state_event(state: &AppStateEvent, pipeline: &AnonymizationPipeline) -> AppStateEvent {
    match state {
        AppStateEvent::MediaFailed { media_type, reason } => AppStateEvent::MediaFailed {
            media_type: *media_type,
            reason: pipeline.anonymize_string(reason),
        },
        AppStateEvent::VideoError { message } => AppStateEvent::VideoError {
            message: pipeline.anonymize_string(message),
        },
        AppStateEvent::ModelDownloadFailed { model, reason } => {
            AppStateEvent::ModelDownloadFailed {
                model: *model,
                reason: pipeline.anonymize_string(reason),
            }
        }
        // All other variants don't contain user strings
        other => other.clone(),
    }
}

/// Anonymizes string fields in `AppOperation` variants.
fn anonymize_operation(operation: &AppOperation, pipeline: &AnonymizationPipeline) -> AppOperation {
    match operation {
        AppOperation::ApplyFilter {
            duration_ms,
            filter_type,
        } => AppOperation::ApplyFilter {
            duration_ms: *duration_ms,
            filter_type: pipeline.anonymize_string(filter_type),
        },
        // All other variants don't contain user strings
        other => other.clone(),
    }
}

// =============================================================================
// Atomic File Write
// =============================================================================

/// Writes content to a file atomically.
///
/// Uses a temporary file with `.tmp` extension, then renames to the final path.
/// This prevents partial writes from corrupting the target file.
///
/// # Errors
///
/// Returns an error if writing or renaming fails.
pub fn write_atomic(path: &Path, content: &str) -> io::Result<()> {
    let temp_path = path.with_extension("json.tmp");

    // Write to temp file
    fs::write(&temp_path, content)?;

    // Atomic rename
    if let Err(e) = fs::rename(&temp_path, path) {
        // Clean up temp file on failure
        let _ = fs::remove_file(&temp_path);
        return Err(e);
    }

    Ok(())
}

/// Returns the default directory for saving diagnostic reports.
///
/// Uses the user's Documents folder if available, otherwise falls back
/// to the current directory.
#[must_use]
pub fn default_export_directory() -> PathBuf {
    dirs::document_dir().unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    use approx::assert_relative_eq;

    use crate::diagnostics::{ErrorType, ResourceMetrics, UserAction, WarningType};

    // =========================================================================
    // ExportError Tests
    // =========================================================================

    #[test]
    fn export_error_io_displays_correctly() {
        let err = ExportError::Io(io::Error::new(io::ErrorKind::NotFound, "file not found"));
        let display = format!("{err}");
        assert!(display.contains("I/O error"));
        assert!(display.contains("file not found"));
    }

    #[test]
    fn export_error_cancelled_displays_correctly() {
        let err = ExportError::Cancelled;
        let display = format!("{err}");
        assert_eq!(display, "export cancelled");
    }

    #[test]
    fn export_error_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let export_err: ExportError = io_err.into();
        assert!(matches!(export_err, ExportError::Io(_)));
    }

    #[test]
    fn export_error_clipboard_displays_correctly() {
        let err = ExportError::Clipboard("clipboard unavailable".to_string());
        let display = format!("{err}");
        assert!(display.contains("clipboard error"));
        assert!(display.contains("clipboard unavailable"));
    }

    #[test]
    fn export_error_content_too_large_displays_correctly() {
        let err = ExportError::ContentTooLarge {
            size: 15 * 1024 * 1024,     // 15 MB
            max_size: 10 * 1024 * 1024, // 10 MB
        };
        let display = format!("{err}");
        assert!(display.contains("content too large"));
        assert!(display.contains("15.0 MB"));
        assert!(display.contains("10.0 MB"));
    }

    #[test]
    fn max_clipboard_size_is_reasonable() {
        // Verify the constant is set to 10 MB
        assert_eq!(MAX_CLIPBOARD_SIZE_BYTES, 10 * 1024 * 1024);
    }

    // =========================================================================
    // Filename Generation Tests
    // =========================================================================

    #[test]
    fn generate_default_filename_has_correct_format() {
        let filename = generate_default_filename();

        assert!(filename.starts_with("iced_lens_diagnostics_"));
        assert!(Path::new(&filename)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json")));

        // Extract timestamp part
        let timestamp = &filename[22..filename.len() - 5]; // Remove prefix and .json
        assert_eq!(timestamp.len(), 15); // YYYYMMDD_HHMMSS

        // Verify it contains underscore separator
        assert!(timestamp.contains('_'));
    }

    // =========================================================================
    // Event Anonymization Tests
    // =========================================================================

    #[test]
    fn anonymize_event_user_action_with_details() {
        let pipeline = AnonymizationPipeline::with_seed(1);
        let start = Instant::now();

        let event = SerializableEvent::new(
            start,
            start,
            DiagnosticEventKind::UserAction {
                action: UserAction::NavigateNext,
                details: Some("Error from 192.168.1.1".to_string()),
            },
        );

        let anonymized = anonymize_event(&event, &pipeline);

        match &anonymized.kind {
            DiagnosticEventKind::UserAction { action, details } => {
                assert!(matches!(action, UserAction::NavigateNext));
                let details = details.as_ref().expect("should have details");
                assert!(details.contains("<ip:"));
                assert!(!details.contains("192.168.1.1"));
            }
            _ => panic!("expected UserAction"),
        }
    }

    #[test]
    fn anonymize_event_warning_message() {
        let pipeline = AnonymizationPipeline::with_seed(1);
        let start = Instant::now();

        let event = SerializableEvent::new(
            start,
            start,
            DiagnosticEventKind::Warning {
                event: WarningEvent {
                    warning_type: WarningType::NetworkError,
                    message: "Failed to connect to api.example.com".to_string(),
                    source_module: None,
                },
            },
        );

        let anonymized = anonymize_event(&event, &pipeline);

        match &anonymized.kind {
            DiagnosticEventKind::Warning { event } => {
                assert!(event.message.contains("<domain:"));
                assert!(!event.message.contains("example.com"));
                assert_eq!(event.warning_type, WarningType::NetworkError);
            }
            _ => panic!("expected Warning"),
        }
    }

    #[test]
    fn anonymize_event_error_message() {
        let pipeline = AnonymizationPipeline::with_seed(1);
        let start = Instant::now();

        let event = SerializableEvent::new(
            start,
            start,
            DiagnosticEventKind::Error {
                event: ErrorEvent {
                    error_type: ErrorType::IoError,
                    error_code: Some("E001".to_string()),
                    message: "Connection refused by 10.0.0.1".to_string(),
                    source_module: Some("network".to_string()),
                },
            },
        );

        let anonymized = anonymize_event(&event, &pipeline);

        match &anonymized.kind {
            DiagnosticEventKind::Error { event } => {
                assert!(event.message.contains("<ip:"));
                assert!(!event.message.contains("10.0.0.1"));
                assert_eq!(event.error_type, ErrorType::IoError);
                assert_eq!(event.error_code.as_deref(), Some("E001"));
                assert_eq!(event.source_module.as_deref(), Some("network"));
            }
            _ => panic!("expected Error"),
        }
    }

    #[test]
    fn anonymize_event_preserves_resource_snapshot() {
        let pipeline = AnonymizationPipeline::with_seed(1);
        let start = Instant::now();

        let metrics = ResourceMetrics::new(50.0, 2_000_000_000, 8_000_000_000, 100, 200);
        let event = SerializableEvent::new(
            start,
            start,
            DiagnosticEventKind::ResourceSnapshot {
                metrics: metrics.clone(),
            },
        );

        let anonymized = anonymize_event(&event, &pipeline);

        match &anonymized.kind {
            DiagnosticEventKind::ResourceSnapshot { metrics: m } => {
                assert_relative_eq!(m.cpu_percent, metrics.cpu_percent, epsilon = 0.01);
                assert_eq!(m.ram_used_bytes, metrics.ram_used_bytes);
            }
            _ => panic!("expected ResourceSnapshot"),
        }
    }

    #[test]
    fn anonymize_event_preserves_timestamp() {
        let pipeline = AnonymizationPipeline::with_seed(1);
        let start = Instant::now();

        let event = SerializableEvent::new(
            start,
            start,
            DiagnosticEventKind::UserAction {
                action: UserAction::ZoomIn,
                details: None,
            },
        );

        let anonymized = anonymize_event(&event, &pipeline);

        assert_eq!(anonymized.timestamp_ms, event.timestamp_ms);
    }

    // =========================================================================
    // Atomic Write Tests
    // =========================================================================

    #[test]
    fn atomic_write_creates_file() {
        let temp_dir = tempfile::tempdir().expect("should create temp dir");
        let path = temp_dir.path().join("test_report.json");

        write_atomic(&path, r#"{"test": true}"#).expect("write should succeed");

        assert!(path.exists());
        let content = fs::read_to_string(&path).expect("should read file");
        assert_eq!(content, r#"{"test": true}"#);
    }

    #[test]
    fn atomic_write_no_temp_file_on_success() {
        let temp_dir = tempfile::tempdir().expect("should create temp dir");
        let path = temp_dir.path().join("test_report.json");
        let temp_path = path.with_extension("json.tmp");

        write_atomic(&path, r#"{"test": true}"#).expect("write should succeed");

        assert!(path.exists());
        assert!(!temp_path.exists(), "temp file should be removed");
    }

    #[test]
    fn default_export_directory_returns_valid_path() {
        let dir = default_export_directory();
        // Should return some path (may be empty in restricted environments)
        // Just verify it doesn't panic
        let _ = dir.to_string_lossy();
    }
}
