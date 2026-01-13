// SPDX-License-Identifier: MPL-2.0
//! Diagnostic event types for activity tracking.
//!
//! This module defines the various types of events that can be captured
//! during application usage for diagnostic purposes.

use std::time::Instant;

use serde::{Deserialize, Serialize};

use super::sanitizer::{ErrorType, WarningType};
use super::ResourceMetrics;

// =============================================================================
// Supporting Enums
// =============================================================================

/// Type of media being processed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaType {
    /// Static image (JPEG, PNG, WebP, etc.)
    Image,
    /// Video file (MP4, `WebM`, etc.)
    Video,
    /// Unknown or unrecognized media type
    Unknown,
}

/// Size category for privacy-preserving file size reporting.
///
/// Uses broad categories instead of exact sizes to protect user privacy
/// while still providing useful diagnostic information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SizeCategory {
    /// Less than 1 MB (thumbnails, small images)
    Small,
    /// 1 MB to 10 MB (standard photos)
    Medium,
    /// 10 MB to 100 MB (RAW images, short videos)
    Large,
    /// More than 100 MB (long videos, 4K content)
    VeryLarge,
}

impl SizeCategory {
    /// Size threshold constants in bytes
    const ONE_MB: u64 = 1_048_576;
    const TEN_MB: u64 = 10_485_760;
    const HUNDRED_MB: u64 = 104_857_600;

    /// Determines the size category from a byte count.
    ///
    /// # Examples
    ///
    /// ```
    /// use iced_lens::diagnostics::SizeCategory;
    ///
    /// assert_eq!(SizeCategory::from_bytes(500_000), SizeCategory::Small);
    /// assert_eq!(SizeCategory::from_bytes(5_000_000), SizeCategory::Medium);
    /// assert_eq!(SizeCategory::from_bytes(50_000_000), SizeCategory::Large);
    /// assert_eq!(SizeCategory::from_bytes(500_000_000), SizeCategory::VeryLarge);
    /// ```
    #[must_use]
    pub fn from_bytes(bytes: u64) -> Self {
        if bytes < Self::ONE_MB {
            Self::Small
        } else if bytes < Self::TEN_MB {
            Self::Medium
        } else if bytes < Self::HUNDRED_MB {
            Self::Large
        } else {
            Self::VeryLarge
        }
    }
}

/// Editor tool being used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditorTool {
    /// Crop tool for selecting regions
    Crop,
    /// Resize tool for scaling images
    Resize,
    /// Adjustment tool for brightness/contrast/etc.
    Adjust,
    /// AI deblur tool
    Deblur,
    /// AI upscale tool
    Upscale,
}

/// AI model type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AIModel {
    /// `NAFNet` deblur model
    Deblur,
    /// Real-ESRGAN upscale model
    Upscale,
}

// =============================================================================
// User Actions
// =============================================================================

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

// =============================================================================
// Application State Events
// =============================================================================

/// Application state change events.
///
/// These events capture significant state transitions in the application,
/// including media lifecycle, video playback, editor sessions, and AI operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum AppStateEvent {
    // -------------------------------------------------------------------------
    // Media Lifecycle
    // -------------------------------------------------------------------------
    /// Media loading has started.
    MediaLoadingStarted {
        /// Type of media being loaded
        media_type: MediaType,
        /// Size category of the file
        size_category: SizeCategory,
    },

    /// Media has been successfully loaded.
    MediaLoaded {
        /// Type of media loaded
        media_type: MediaType,
        /// Size category of the file
        size_category: SizeCategory,
    },

    /// Media loading has failed.
    MediaFailed {
        /// Type of media that failed to load
        media_type: MediaType,
        /// Sanitized reason for failure (no paths)
        reason: String,
    },

    // -------------------------------------------------------------------------
    // Video Playback (Must Have)
    // -------------------------------------------------------------------------
    /// Video playback has started.
    VideoPlaying {
        /// Current position in seconds
        position_secs: f64,
    },

    /// Video playback has been paused.
    VideoPaused {
        /// Current position in seconds
        position_secs: f64,
    },

    /// Video seek operation initiated.
    VideoSeeking {
        /// Target position in seconds
        target_secs: f64,
    },

    /// Video is buffering.
    VideoBuffering {
        /// Current position in seconds
        position_secs: f64,
    },

    /// Video playback error occurred.
    VideoError {
        /// Sanitized error message (no paths)
        message: String,
    },

    /// Video reached end of stream.
    VideoAtEndOfStream,

    // -------------------------------------------------------------------------
    // Video Playback (Should Have)
    // -------------------------------------------------------------------------
    /// Video loop mode toggled.
    VideoLoopToggled {
        /// Whether loop is now enabled
        enabled: bool,
    },

    /// Video playback speed changed.
    VideoSpeedChanged {
        /// New playback speed multiplier
        speed: f64,
    },

    // -------------------------------------------------------------------------
    // Editor (Must Have)
    // -------------------------------------------------------------------------
    /// Editor screen opened.
    EditorOpened {
        /// Initial tool selected, if any
        #[serde(skip_serializing_if = "Option::is_none")]
        tool: Option<EditorTool>,
    },

    /// Editor screen closed.
    EditorClosed {
        /// Whether there were unsaved changes
        had_unsaved_changes: bool,
    },

    /// AI deblur operation started.
    EditorDeblurStarted,

    /// AI deblur operation progress update.
    EditorDeblurProgress {
        /// Progress percentage (0.0 to 100.0)
        percent: f32,
    },

    /// AI deblur operation completed successfully.
    EditorDeblurCompleted,

    /// AI deblur operation was cancelled.
    EditorDeblurCancelled,

    // -------------------------------------------------------------------------
    // Editor (Should Have)
    // -------------------------------------------------------------------------
    /// Editor unsaved changes state changed.
    EditorUnsavedChanges {
        /// Whether the editor now has unsaved changes
        has_changes: bool,
    },

    // -------------------------------------------------------------------------
    // AI Model Lifecycle (Should Have)
    // -------------------------------------------------------------------------
    /// AI model download started.
    ModelDownloadStarted {
        /// The model being downloaded
        model: AIModel,
    },

    /// AI model download completed successfully.
    ModelDownloadCompleted {
        /// The model that was downloaded
        model: AIModel,
    },

    /// AI model download failed.
    ModelDownloadFailed {
        /// The model that failed to download
        model: AIModel,
        /// Sanitized reason for failure (generic, no paths)
        reason: String,
    },
}

// =============================================================================
// Application Operations
// =============================================================================

/// Application operations with duration tracking.
///
/// These events capture internal operations that may affect performance,
/// including frame processing, AI operations, and video seeking.
/// All operations include duration in milliseconds for performance analysis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum AppOperation {
    // -------------------------------------------------------------------------
    // Original Operations
    // -------------------------------------------------------------------------
    /// Video frame decode operation.
    DecodeFrame {
        /// Duration in milliseconds
        duration_ms: u64,
    },

    /// Image resize operation.
    ResizeImage {
        /// Duration in milliseconds
        duration_ms: u64,
        /// Size category of the image
        size_category: SizeCategory,
    },

    /// Filter application operation.
    ApplyFilter {
        /// Duration in milliseconds
        duration_ms: u64,
        /// Type of filter applied
        filter_type: String,
    },

    /// Video frame export operation.
    ExportFrame {
        /// Duration in milliseconds
        duration_ms: u64,
    },

    /// Metadata loading operation.
    LoadMetadata {
        /// Duration in milliseconds
        duration_ms: u64,
    },

    // -------------------------------------------------------------------------
    // AI Operations (Must Have)
    // -------------------------------------------------------------------------
    /// AI deblur processing operation.
    #[serde(rename = "ai_deblur_process")]
    AIDeblurProcess {
        /// Duration in milliseconds
        duration_ms: u64,
        /// Size category of the image
        size_category: SizeCategory,
        /// Whether the operation succeeded
        success: bool,
    },

    /// AI upscale processing operation.
    #[serde(rename = "ai_upscale_process")]
    AIUpscaleProcess {
        /// Duration in milliseconds
        duration_ms: u64,
        /// Scale factor applied (e.g., 2.0 for 2x upscale)
        scale_factor: f32,
        /// Size category of the original image
        size_category: SizeCategory,
        /// Whether the operation succeeded
        success: bool,
    },

    // -------------------------------------------------------------------------
    // Video Operations (Should Have)
    // -------------------------------------------------------------------------
    /// Video seek operation.
    VideoSeek {
        /// Duration in milliseconds
        duration_ms: u64,
        /// Distance seeked in seconds (absolute value)
        seek_distance_secs: f64,
    },
}

// =============================================================================
// Warning and Error Events
// =============================================================================

/// A warning event with categorization and context.
///
/// Warning events capture non-critical issues that may affect behavior
/// but don't prevent the application from functioning.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WarningEvent {
    /// Category of the warning
    pub warning_type: WarningType,
    /// Sanitized warning message (paths removed)
    pub message: String,
    /// Source module that generated the warning
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_module: Option<String>,
}

impl WarningEvent {
    /// Creates a new warning event.
    ///
    /// # Arguments
    ///
    /// * `warning_type` - Category of the warning
    /// * `message` - Warning message (will be stored as-is, sanitize before calling)
    #[must_use]
    pub fn new(warning_type: WarningType, message: impl Into<String>) -> Self {
        Self {
            warning_type,
            message: message.into(),
            source_module: None,
        }
    }

    /// Creates a new warning event with source module information.
    #[must_use]
    pub fn with_source(
        warning_type: WarningType,
        message: impl Into<String>,
        source_module: impl Into<String>,
    ) -> Self {
        Self {
            warning_type,
            message: message.into(),
            source_module: Some(source_module.into()),
        }
    }
}

/// An error event with categorization, optional error code, and context.
///
/// Error events capture critical issues that caused operation failure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorEvent {
    /// Category of the error
    pub error_type: ErrorType,
    /// Optional error code for specific error identification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    /// Sanitized error message (paths removed)
    pub message: String,
    /// Source module that generated the error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_module: Option<String>,
}

impl ErrorEvent {
    /// Creates a new error event.
    ///
    /// # Arguments
    ///
    /// * `error_type` - Category of the error
    /// * `message` - Error message (will be stored as-is, sanitize before calling)
    #[must_use]
    pub fn new(error_type: ErrorType, message: impl Into<String>) -> Self {
        Self {
            error_type,
            error_code: None,
            message: message.into(),
            source_module: None,
        }
    }

    /// Creates a new error event with an error code.
    #[must_use]
    pub fn with_code(
        error_type: ErrorType,
        error_code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            error_type,
            error_code: Some(error_code.into()),
            message: message.into(),
            source_module: None,
        }
    }

    /// Creates a new error event with source module information.
    #[must_use]
    pub fn with_source(
        error_type: ErrorType,
        message: impl Into<String>,
        source_module: impl Into<String>,
    ) -> Self {
        Self {
            error_type,
            error_code: None,
            message: message.into(),
            source_module: Some(source_module.into()),
        }
    }

    /// Creates a new error event with both error code and source module.
    #[must_use]
    pub fn full(
        error_type: ErrorType,
        error_code: impl Into<String>,
        message: impl Into<String>,
        source_module: impl Into<String>,
    ) -> Self {
        Self {
            error_type,
            error_code: Some(error_code.into()),
            message: message.into(),
            source_module: Some(source_module.into()),
        }
    }
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// Captures significant state transitions in the application.
    AppState {
        /// The state change event
        #[serde(flatten)]
        state: AppStateEvent,
    },

    /// Internal operation with performance metrics.
    /// Captures operations that may affect performance.
    Operation {
        /// The operation event
        #[serde(flatten)]
        operation: AppOperation,
    },

    /// Non-critical warning with categorization and context.
    Warning {
        /// The warning event details
        #[serde(flatten)]
        event: WarningEvent,
    },

    /// Critical error with categorization and context.
    Error {
        /// The error event details
        #[serde(flatten)]
        event: ErrorEvent,
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
        let state = DiagnosticEventKind::AppState {
            state: AppStateEvent::VideoPlaying { position_secs: 0.0 },
        };
        let operation = DiagnosticEventKind::Operation {
            operation: AppOperation::DecodeFrame { duration_ms: 16 },
        };
        let warning = DiagnosticEventKind::Warning {
            event: WarningEvent::new(WarningType::Other, "test warning"),
        };
        let error = DiagnosticEventKind::Error {
            event: ErrorEvent::new(ErrorType::Other, "test error"),
        };

        assert!(matches!(
            resource,
            DiagnosticEventKind::ResourceSnapshot { .. }
        ));
        assert!(matches!(action, DiagnosticEventKind::UserAction { .. }));
        assert!(matches!(state, DiagnosticEventKind::AppState { .. }));
        assert!(matches!(operation, DiagnosticEventKind::Operation { .. }));
        assert!(matches!(warning, DiagnosticEventKind::Warning { .. }));
        assert!(matches!(error, DiagnosticEventKind::Error { .. }));
    }

    #[test]
    fn diagnostic_event_kind_serializes_to_json() {
        let warning = DiagnosticEventKind::Warning {
            event: WarningEvent::new(WarningType::FileNotFound, "test warning"),
        };

        let json = serde_json::to_string(&warning).expect("serialization should succeed");
        assert!(json.contains("\"type\":\"warning\""));
        assert!(json.contains("\"warning_type\":\"file_not_found\""));
        assert!(json.contains("\"message\":\"test warning\""));
    }

    #[test]
    fn diagnostic_event_kind_deserializes_from_json() {
        let json = r#"{"type":"error","error_type":"io_error","message":"test error"}"#;
        let event: DiagnosticEventKind =
            serde_json::from_str(json).expect("deserialization should succeed");

        match event {
            DiagnosticEventKind::Error { event } => {
                assert_eq!(event.message, "test error");
                assert_eq!(event.error_type, ErrorType::IoError);
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

    // =========================================================================
    // SizeCategory Tests
    // =========================================================================

    #[test]
    fn size_category_from_bytes_small() {
        // 0 bytes - Small
        assert_eq!(SizeCategory::from_bytes(0), SizeCategory::Small);
        // Just under 1MB - Small
        assert_eq!(SizeCategory::from_bytes(1_048_575), SizeCategory::Small);
    }

    #[test]
    fn size_category_from_bytes_medium() {
        // Exactly 1MB - Medium
        assert_eq!(SizeCategory::from_bytes(1_048_576), SizeCategory::Medium);
        // Just under 10MB - Medium
        assert_eq!(SizeCategory::from_bytes(10_485_759), SizeCategory::Medium);
    }

    #[test]
    fn size_category_from_bytes_large() {
        // Exactly 10MB - Large
        assert_eq!(SizeCategory::from_bytes(10_485_760), SizeCategory::Large);
        // Just under 100MB - Large
        assert_eq!(SizeCategory::from_bytes(104_857_599), SizeCategory::Large);
    }

    #[test]
    fn size_category_from_bytes_very_large() {
        // Exactly 100MB - VeryLarge
        assert_eq!(
            SizeCategory::from_bytes(104_857_600),
            SizeCategory::VeryLarge
        );
        // 1GB - VeryLarge
        assert_eq!(
            SizeCategory::from_bytes(1_073_741_824),
            SizeCategory::VeryLarge
        );
    }

    #[test]
    fn size_category_serializes_correctly() {
        assert_eq!(
            serde_json::to_string(&SizeCategory::Small).unwrap(),
            "\"small\""
        );
        assert_eq!(
            serde_json::to_string(&SizeCategory::VeryLarge).unwrap(),
            "\"very_large\""
        );
    }

    // =========================================================================
    // AppStateEvent Tests
    // =========================================================================

    #[test]
    fn app_state_event_video_playing_serializes() {
        let state = AppStateEvent::VideoPlaying {
            position_secs: 42.5,
        };
        let json = serde_json::to_string(&state).expect("serialization should succeed");

        assert!(json.contains("\"state\":\"video_playing\""));
        assert!(json.contains("\"position_secs\":42.5"));
    }

    #[test]
    fn app_state_event_media_loaded_serializes() {
        let state = AppStateEvent::MediaLoaded {
            media_type: MediaType::Image,
            size_category: SizeCategory::Medium,
        };
        let json = serde_json::to_string(&state).expect("serialization should succeed");

        assert!(json.contains("\"state\":\"media_loaded\""));
        assert!(json.contains("\"media_type\":\"image\""));
        assert!(json.contains("\"size_category\":\"medium\""));
    }

    #[test]
    fn app_state_event_editor_opened_serializes() {
        let state = AppStateEvent::EditorOpened {
            tool: Some(EditorTool::Deblur),
        };
        let json = serde_json::to_string(&state).expect("serialization should succeed");

        assert!(json.contains("\"state\":\"editor_opened\""));
        assert!(json.contains("\"tool\":\"deblur\""));
    }

    #[test]
    fn app_state_event_editor_opened_no_tool_serializes() {
        let state = AppStateEvent::EditorOpened { tool: None };
        let json = serde_json::to_string(&state).expect("serialization should succeed");

        assert!(json.contains("\"state\":\"editor_opened\""));
        // tool should be omitted when None
        assert!(!json.contains("\"tool\""));
    }

    #[test]
    fn app_state_event_model_download_serializes() {
        let state = AppStateEvent::ModelDownloadStarted {
            model: AIModel::Upscale,
        };
        let json = serde_json::to_string(&state).expect("serialization should succeed");

        assert!(json.contains("\"state\":\"model_download_started\""));
        assert!(json.contains("\"model\":\"upscale\""));
    }

    #[test]
    fn app_state_event_deserializes() {
        let json = r#"{"state":"video_paused","position_secs":10.0}"#;
        let state: AppStateEvent =
            serde_json::from_str(json).expect("deserialization should succeed");

        match state {
            AppStateEvent::VideoPaused { position_secs } => {
                assert_relative_eq!(position_secs, 10.0, epsilon = 0.01);
            }
            _ => panic!("expected VideoPaused variant"),
        }
    }

    // =========================================================================
    // AppOperation Tests
    // =========================================================================

    #[test]
    fn app_operation_decode_frame_serializes() {
        let op = AppOperation::DecodeFrame { duration_ms: 16 };
        let json = serde_json::to_string(&op).expect("serialization should succeed");

        assert!(json.contains("\"operation\":\"decode_frame\""));
        assert!(json.contains("\"duration_ms\":16"));
    }

    #[test]
    fn app_operation_ai_deblur_serializes() {
        let op = AppOperation::AIDeblurProcess {
            duration_ms: 1500,
            size_category: SizeCategory::Large,
            success: true,
        };
        let json = serde_json::to_string(&op).expect("serialization should succeed");

        assert!(json.contains("\"operation\":\"ai_deblur_process\""));
        assert!(json.contains("\"duration_ms\":1500"));
        assert!(json.contains("\"size_category\":\"large\""));
        assert!(json.contains("\"success\":true"));
    }

    #[test]
    fn app_operation_ai_upscale_serializes() {
        let op = AppOperation::AIUpscaleProcess {
            duration_ms: 2500,
            scale_factor: 4.0,
            size_category: SizeCategory::Medium,
            success: false,
        };
        let json = serde_json::to_string(&op).expect("serialization should succeed");

        assert!(json.contains("\"operation\":\"ai_upscale_process\""));
        assert!(json.contains("\"scale_factor\":4.0"));
        assert!(json.contains("\"success\":false"));
    }

    #[test]
    fn app_operation_video_seek_serializes() {
        let op = AppOperation::VideoSeek {
            duration_ms: 50,
            seek_distance_secs: 30.5,
        };
        let json = serde_json::to_string(&op).expect("serialization should succeed");

        assert!(json.contains("\"operation\":\"video_seek\""));
        assert!(json.contains("\"seek_distance_secs\":30.5"));
    }

    #[test]
    fn app_operation_deserializes() {
        let json = r#"{"operation":"export_frame","duration_ms":100}"#;
        let op: AppOperation = serde_json::from_str(json).expect("deserialization should succeed");

        match op {
            AppOperation::ExportFrame { duration_ms } => {
                assert_eq!(duration_ms, 100);
            }
            _ => panic!("expected ExportFrame variant"),
        }
    }

    // =========================================================================
    // DiagnosticEventKind with AppState/Operation Tests
    // =========================================================================

    #[test]
    fn diagnostic_event_kind_app_state_serializes() {
        let kind = DiagnosticEventKind::AppState {
            state: AppStateEvent::EditorDeblurCompleted,
        };
        let json = serde_json::to_string(&kind).expect("serialization should succeed");

        assert!(json.contains("\"type\":\"app_state\""));
        assert!(json.contains("\"state\":\"editor_deblur_completed\""));
    }

    #[test]
    fn diagnostic_event_kind_operation_serializes() {
        let kind = DiagnosticEventKind::Operation {
            operation: AppOperation::LoadMetadata { duration_ms: 25 },
        };
        let json = serde_json::to_string(&kind).expect("serialization should succeed");

        assert!(json.contains("\"type\":\"operation\""));
        assert!(json.contains("\"operation\":\"load_metadata\""));
        assert!(json.contains("\"duration_ms\":25"));
    }

    #[test]
    fn diagnostic_event_kind_app_state_deserializes() {
        let json = r#"{"type":"app_state","state":"video_at_end_of_stream"}"#;
        let kind: DiagnosticEventKind =
            serde_json::from_str(json).expect("deserialization should succeed");

        match kind {
            DiagnosticEventKind::AppState { state } => {
                assert!(matches!(state, AppStateEvent::VideoAtEndOfStream));
            }
            _ => panic!("expected AppState variant"),
        }
    }

    #[test]
    fn diagnostic_event_kind_operation_deserializes() {
        let json = r#"{"type":"operation","operation":"resize_image","duration_ms":200,"size_category":"small"}"#;
        let kind: DiagnosticEventKind =
            serde_json::from_str(json).expect("deserialization should succeed");

        match kind {
            DiagnosticEventKind::Operation { operation } => match operation {
                AppOperation::ResizeImage {
                    duration_ms,
                    size_category,
                } => {
                    assert_eq!(duration_ms, 200);
                    assert_eq!(size_category, SizeCategory::Small);
                }
                _ => panic!("expected ResizeImage operation"),
            },
            _ => panic!("expected Operation variant"),
        }
    }

    // =========================================================================
    // WarningEvent Tests
    // =========================================================================

    #[test]
    fn warning_event_new_creates_event() {
        let event = WarningEvent::new(WarningType::FileNotFound, "File not found");
        assert_eq!(event.warning_type, WarningType::FileNotFound);
        assert_eq!(event.message, "File not found");
        assert!(event.source_module.is_none());
    }

    #[test]
    fn warning_event_with_source_creates_event() {
        let event = WarningEvent::with_source(
            WarningType::PermissionDenied,
            "Access denied",
            "media_loader",
        );
        assert_eq!(event.warning_type, WarningType::PermissionDenied);
        assert_eq!(event.message, "Access denied");
        assert_eq!(event.source_module.as_deref(), Some("media_loader"));
    }

    #[test]
    fn warning_event_serializes_correctly() {
        let event = WarningEvent::new(WarningType::NetworkError, "Connection failed");
        let json = serde_json::to_string(&event).expect("serialization should succeed");

        assert!(json.contains("\"warning_type\":\"network_error\""));
        assert!(json.contains("\"message\":\"Connection failed\""));
        // source_module should be omitted when None
        assert!(!json.contains("source_module"));
    }

    #[test]
    fn warning_event_with_source_serializes_correctly() {
        let event = WarningEvent::with_source(
            WarningType::UnsupportedFormat,
            "Unknown codec",
            "video_player",
        );
        let json = serde_json::to_string(&event).expect("serialization should succeed");

        assert!(json.contains("\"warning_type\":\"unsupported_format\""));
        assert!(json.contains("\"source_module\":\"video_player\""));
    }

    #[test]
    fn warning_event_deserializes_correctly() {
        let json = r#"{"warning_type":"configuration_issue","message":"Invalid setting"}"#;
        let event: WarningEvent =
            serde_json::from_str(json).expect("deserialization should succeed");

        assert_eq!(event.warning_type, WarningType::ConfigurationIssue);
        assert_eq!(event.message, "Invalid setting");
        assert!(event.source_module.is_none());
    }

    // =========================================================================
    // ErrorEvent Tests
    // =========================================================================

    #[test]
    fn error_event_new_creates_event() {
        let event = ErrorEvent::new(ErrorType::IoError, "Read failed");
        assert_eq!(event.error_type, ErrorType::IoError);
        assert_eq!(event.message, "Read failed");
        assert!(event.error_code.is_none());
        assert!(event.source_module.is_none());
    }

    #[test]
    fn error_event_with_code_creates_event() {
        let event = ErrorEvent::with_code(ErrorType::DecodeError, "E001", "Invalid header");
        assert_eq!(event.error_type, ErrorType::DecodeError);
        assert_eq!(event.error_code.as_deref(), Some("E001"));
        assert_eq!(event.message, "Invalid header");
        assert!(event.source_module.is_none());
    }

    #[test]
    fn error_event_with_source_creates_event() {
        let event = ErrorEvent::with_source(ErrorType::ExportError, "Write failed", "exporter");
        assert_eq!(event.error_type, ErrorType::ExportError);
        assert_eq!(event.source_module.as_deref(), Some("exporter"));
        assert!(event.error_code.is_none());
    }

    #[test]
    fn error_event_full_creates_event() {
        let event = ErrorEvent::full(
            ErrorType::AIModelError,
            "MODEL_LOAD_001",
            "Failed to load model",
            "ai_engine",
        );
        assert_eq!(event.error_type, ErrorType::AIModelError);
        assert_eq!(event.error_code.as_deref(), Some("MODEL_LOAD_001"));
        assert_eq!(event.message, "Failed to load model");
        assert_eq!(event.source_module.as_deref(), Some("ai_engine"));
    }

    #[test]
    fn error_event_serializes_correctly() {
        let event = ErrorEvent::new(ErrorType::InternalError, "Unexpected state");
        let json = serde_json::to_string(&event).expect("serialization should succeed");

        assert!(json.contains("\"error_type\":\"internal_error\""));
        assert!(json.contains("\"message\":\"Unexpected state\""));
        // error_code and source_module should be omitted when None
        assert!(!json.contains("error_code"));
        assert!(!json.contains("source_module"));
    }

    #[test]
    fn error_event_with_code_serializes_correctly() {
        let event = ErrorEvent::with_code(ErrorType::IoError, "ENOENT", "File not found");
        let json = serde_json::to_string(&event).expect("serialization should succeed");

        assert!(json.contains("\"error_code\":\"ENOENT\""));
    }

    #[test]
    fn error_event_deserializes_correctly() {
        let json =
            r#"{"error_type":"decode_error","error_code":"DEC_001","message":"Invalid frame"}"#;
        let event: ErrorEvent = serde_json::from_str(json).expect("deserialization should succeed");

        assert_eq!(event.error_type, ErrorType::DecodeError);
        assert_eq!(event.error_code.as_deref(), Some("DEC_001"));
        assert_eq!(event.message, "Invalid frame");
    }
}
