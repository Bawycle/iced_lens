// SPDX-License-Identifier: MPL-2.0
//! Diagnostic event types for activity tracking.
//!
//! This module defines the various types of events that can be captured
//! during application usage for diagnostic purposes.

use std::time::Instant;

use serde::{Deserialize, Serialize};

use super::ResourceMetrics;

/// User-initiated actions that can be captured for diagnostics.
///
/// These actions represent meaningful user interactions that help
/// understand what the user was doing when issues occurred.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum UserAction {
    // ==========================================================================
    // Navigation Actions
    // ==========================================================================
    /// Navigate to the next media file.
    NavigateNext,

    /// Navigate to the previous media file.
    NavigatePrevious,

    /// Load a media file (via file dialog, drag-drop, or CLI).
    LoadMedia {
        /// Optional context (e.g., `file_dialog`, `drag_drop`, `cli`).
        #[serde(skip_serializing_if = "Option::is_none")]
        source: Option<String>,
    },

    /// Delete the current media file.
    DeleteMedia,

    // ==========================================================================
    // Playback Actions (Video)
    // ==========================================================================
    /// Toggle play/pause state.
    TogglePlayback,

    /// Seek to a specific position in the video.
    SeekVideo {
        /// Target position in seconds.
        position_secs: f64,
    },

    /// Step forward one frame (when paused).
    StepForward,

    /// Step backward one frame (when paused).
    StepBackward,

    /// Change playback speed.
    SetPlaybackSpeed {
        /// New playback speed multiplier.
        speed: f64,
    },

    /// Toggle loop mode.
    ToggleLoop,

    // ==========================================================================
    // Audio Actions
    // ==========================================================================
    /// Change volume level.
    SetVolume {
        /// Volume level (0.0 to 1.5).
        volume: f32,
    },

    /// Toggle mute state.
    ToggleMute,

    // ==========================================================================
    // View Actions
    // ==========================================================================
    /// Zoom in on the current media.
    ZoomIn,

    /// Zoom out on the current media.
    ZoomOut,

    /// Reset zoom to default (100%).
    ResetZoom,

    /// Toggle fit-to-window mode.
    ToggleFitToWindow,

    /// Toggle fullscreen mode.
    ToggleFullscreen,

    /// Exit fullscreen mode.
    ExitFullscreen,

    /// Rotate media clockwise (90°).
    RotateClockwise,

    /// Rotate media counter-clockwise (90°).
    RotateCounterClockwise,

    // ==========================================================================
    // Screen Navigation
    // ==========================================================================
    /// Open settings screen.
    OpenSettings,

    /// Open help screen.
    OpenHelp,

    /// Open about screen.
    OpenAbout,

    /// Enter image editor.
    EnterEditor,

    /// Return to viewer from another screen.
    ReturnToViewer,

    // ==========================================================================
    // Capture/Export Actions
    // ==========================================================================
    /// Capture current video frame.
    CaptureFrame,

    /// Export/save file.
    ExportFile,

    // ==========================================================================
    // Editor Actions
    // ==========================================================================
    /// Apply crop operation.
    ApplyCrop,

    /// Apply resize operation.
    ApplyResize,

    /// Apply AI deblur.
    ApplyDeblur,

    /// Apply AI upscale.
    ApplyUpscale,

    /// Undo last edit.
    Undo,

    /// Redo last undone edit.
    Redo,

    /// Save edited image.
    SaveImage,
}

/// A diagnostic event with timestamp.
///
/// Each event captures a specific type of activity or state change
/// in the application, along with when it occurred.
///
/// # Variants
///
/// - `ResourceSnapshot`: System resource metrics (CPU, RAM, disk)
/// - `UserAction`: User-initiated actions (navigation, playback controls)
/// - `AppState`: Application state changes (media loaded, screen changed)
/// - `Warning`: Non-critical issues that may affect behavior
/// - `Error`: Critical issues that caused operation failure
#[derive(Debug, Clone)]
pub struct DiagnosticEvent {
    /// When the event occurred (monotonic clock for duration calculations)
    pub timestamp: Instant,
    /// The type and data of the event
    pub kind: DiagnosticEventKind,
}

impl DiagnosticEvent {
    /// Creates a new diagnostic event with the current timestamp.
    #[must_use]
    pub fn new(kind: DiagnosticEventKind) -> Self {
        Self {
            timestamp: Instant::now(),
            kind,
        }
    }

    /// Creates a new diagnostic event with a specific timestamp.
    #[must_use]
    pub fn with_timestamp(kind: DiagnosticEventKind, timestamp: Instant) -> Self {
        Self { timestamp, kind }
    }
}

/// The type and associated data for a diagnostic event.
///
/// This enum categorizes events and holds the specific data for each type.
/// Placeholder variants will be expanded in subsequent stories.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DiagnosticEventKind {
    /// System resource metrics snapshot.
    /// Contains CPU, RAM, and disk I/O measurements.
    ResourceSnapshot {
        /// The collected resource metrics
        metrics: ResourceMetrics,
    },

    /// User-initiated action.
    /// Captures what the user was doing for diagnostic correlation.
    UserAction {
        /// The specific action performed.
        action: UserAction,
        /// Optional additional details (e.g., filename, error context).
        #[serde(skip_serializing_if = "Option::is_none")]
        details: Option<String>,
    },

    /// Application state change.
    /// Placeholder: Will be expanded in Story 1.4 with state transitions.
    AppState {
        /// Placeholder field for future state data
        #[serde(skip_serializing_if = "Option::is_none")]
        placeholder: Option<String>,
    },

    /// Non-critical warning.
    /// Placeholder: Will be expanded in Story 1.5 with warning details.
    Warning {
        /// Brief description of the warning
        message: String,
    },

    /// Critical error.
    /// Placeholder: Will be expanded in Story 1.5 with error details.
    Error {
        /// Brief description of the error
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    fn sample_metrics() -> ResourceMetrics {
        ResourceMetrics::new(50.0, 4_000_000_000, 8_000_000_000, 1000, 2000)
    }

    #[test]
    fn diagnostic_event_new_creates_with_current_timestamp() {
        let before = Instant::now();
        let event = DiagnosticEvent::new(DiagnosticEventKind::ResourceSnapshot {
            metrics: sample_metrics(),
        });
        let after = Instant::now();

        assert!(event.timestamp >= before);
        assert!(event.timestamp <= after);
    }

    #[test]
    fn diagnostic_event_with_timestamp_uses_provided_timestamp() {
        let timestamp = Instant::now();
        let event = DiagnosticEvent::with_timestamp(
            DiagnosticEventKind::UserAction {
                action: UserAction::NavigateNext,
                details: None,
            },
            timestamp,
        );

        assert_eq!(event.timestamp, timestamp);
    }

    #[test]
    fn diagnostic_event_kind_variants_exist() {
        // Verify all variants can be constructed and pattern-matched
        let resource = DiagnosticEventKind::ResourceSnapshot {
            metrics: sample_metrics(),
        };
        let action = DiagnosticEventKind::UserAction {
            action: UserAction::TogglePlayback,
            details: None,
        };
        let state = DiagnosticEventKind::AppState { placeholder: None };
        let warning = DiagnosticEventKind::Warning {
            message: "test warning".to_string(),
        };
        let error = DiagnosticEventKind::Error {
            message: "test error".to_string(),
        };

        assert!(matches!(
            resource,
            DiagnosticEventKind::ResourceSnapshot { .. }
        ));
        assert!(matches!(action, DiagnosticEventKind::UserAction { .. }));
        assert!(matches!(state, DiagnosticEventKind::AppState { .. }));
        assert!(matches!(warning, DiagnosticEventKind::Warning { .. }));
        assert!(matches!(error, DiagnosticEventKind::Error { .. }));
    }

    #[test]
    fn diagnostic_event_kind_serializes_to_json() {
        let warning = DiagnosticEventKind::Warning {
            message: "test warning".to_string(),
        };

        let json = serde_json::to_string(&warning).expect("serialization should succeed");
        assert!(json.contains("\"type\":\"warning\""));
        assert!(json.contains("\"message\":\"test warning\""));
    }

    #[test]
    fn diagnostic_event_kind_deserializes_from_json() {
        let json = r#"{"type":"error","message":"test error"}"#;
        let event: DiagnosticEventKind =
            serde_json::from_str(json).expect("deserialization should succeed");

        match event {
            DiagnosticEventKind::Error { message } => {
                assert_eq!(message, "test error");
            }
            _ => panic!("expected Error variant"),
        }
    }

    #[test]
    fn resource_snapshot_serializes_with_metrics() {
        let resource = DiagnosticEventKind::ResourceSnapshot {
            metrics: sample_metrics(),
        };
        let json = serde_json::to_string(&resource).expect("serialization should succeed");

        assert!(json.contains("\"type\":\"resource_snapshot\""));
        assert!(json.contains("\"cpu_percent\":50.0"));
        assert!(json.contains("\"ram_used_bytes\":4000000000"));
    }

    #[test]
    fn resource_snapshot_deserializes_from_json() {
        let json = r#"{"type":"resource_snapshot","metrics":{"cpu_percent":25.0,"ram_used_bytes":2000000000,"ram_total_bytes":8000000000,"disk_read_bytes":100,"disk_write_bytes":200}}"#;
        let event: DiagnosticEventKind =
            serde_json::from_str(json).expect("deserialization should succeed");

        match event {
            DiagnosticEventKind::ResourceSnapshot { metrics } => {
                assert_relative_eq!(metrics.cpu_percent, 25.0, epsilon = 0.01);
                assert_eq!(metrics.ram_used_bytes, 2_000_000_000);
            }
            _ => panic!("expected ResourceSnapshot variant"),
        }
    }
}
