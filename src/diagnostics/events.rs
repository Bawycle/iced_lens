// SPDX-License-Identifier: MPL-2.0
//! Diagnostic event types for activity tracking.
//!
//! This module defines the various types of events that can be captured
//! during application usage for diagnostic purposes.

use std::path::Path;
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

/// Media dimensions (width and height in pixels).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dimensions {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
}

impl Dimensions {
    /// Creates a new `Dimensions` struct.
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
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

/// Type of filter change for diagnostic tracking.
///
/// This enum captures the specific type of filter modification that occurred,
/// using String serialization to avoid coupling diagnostics to the filter module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "change_type", rename_all = "snake_case")]
pub enum FilterChangeType {
    /// Media type filter changed (e.g., "all" → "imagesonly")
    MediaType {
        /// Previous filter value (e.g., "all", "imagesonly", "videosonly")
        from: String,
        /// New filter value
        to: String,
    },
    /// Date range filter was enabled
    DateRangeEnabled,
    /// Date range filter was disabled
    DateRangeDisabled,
    /// Date filter field changed (e.g., "modified" or "created")
    DateFieldChanged {
        /// The field that was selected
        field: String,
    },
    /// Date bound was set (start or end date)
    DateBoundSet {
        /// Which bound was set ("start" or "end")
        target: String,
    },
    /// Date bound was cleared (start or end date)
    DateBoundCleared {
        /// Which bound was cleared ("start" or "end")
        target: String,
    },
}

/// Storage type for media files.
///
/// Used to identify whether a media file is located on local storage,
/// network storage, or an unknown/ambiguous location.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StorageType {
    /// Local storage (home directories, user folders)
    Local,
    /// Network storage (UNC paths, NFS/SMB mounts)
    Network,
    /// Unknown or ambiguous storage type (default)
    #[default]
    Unknown,
}

impl StorageType {
    /// Detects the storage type from a file path using simple heuristics.
    ///
    /// Detection rules:
    /// - **Network**: UNC paths (`\\server\share`)
    /// - **Local**: Paths under `/home/`, `/Users/`, or `C:\Users\`
    /// - **Unknown**: Default for ambiguous cases
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use iced_lens::diagnostics::StorageType;
    ///
    /// // Local paths
    /// assert_eq!(StorageType::detect(Path::new("/home/user/photo.jpg")), StorageType::Local);
    ///
    /// // Unknown paths (ambiguous)
    /// assert_eq!(StorageType::detect(Path::new("/tmp/file.jpg")), StorageType::Unknown);
    /// ```
    #[must_use]
    pub fn detect(path: &Path) -> Self {
        let path_str = path.to_string_lossy();

        // Network detection (high confidence): UNC paths (Windows-style, e.g., \\server\share)
        // Note: This check runs on all platforms as some tools may use UNC paths cross-platform
        if path_str.starts_with("\\\\") {
            return Self::Network;
        }

        // Network detection (high confidence): GVFS mounts (Linux)
        // Paths like /run/user/<uid>/gvfs/smb-share:server=... or /run/user/<uid>/gvfs/nfs:...
        #[cfg(unix)]
        {
            if path_str.contains("/gvfs/smb-share:") || path_str.contains("/gvfs/nfs:") {
                return Self::Network;
            }
        }

        // Local detection (high confidence): User directories
        #[cfg(unix)]
        {
            if path_str.starts_with("/home/") || path_str.starts_with("/Users/") {
                return Self::Local;
            }
        }

        #[cfg(windows)]
        {
            // Check for C:\Users\, D:\Users\, etc.
            if path_str.len() >= 3 {
                let chars: Vec<char> = path_str.chars().take(10).collect();
                if chars.len() >= 3
                    && chars[1] == ':'
                    && (chars[2] == '\\' || chars[2] == '/')
                    && path_str.to_lowercase().contains("\\users\\")
                {
                    return Self::Local;
                }
            }
        }

        // Default to Unknown for ambiguous paths
        Self::Unknown
    }
}

// =============================================================================
// Media Metadata
// =============================================================================

/// Metadata extracted from a media file path for diagnostic purposes.
///
/// This struct collects enriched metadata about media files including
/// the file extension, storage type, and an anonymized path hash.
#[derive(Debug, Clone, Default)]
pub struct MediaMetadata {
    /// File extension (e.g., "jpg", "mp4", "heic") - lowercase, without dot
    pub extension: Option<String>,
    /// Storage type (local, network, or unknown)
    pub storage_type: StorageType,
    /// 8-char blake3 hash of the path for correlation
    pub path_hash: Option<String>,
}

impl MediaMetadata {
    /// Creates media metadata from a file path.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path to extract metadata from
    /// * `anonymizer` - The path anonymizer for generating path hashes
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use std::path::Path;
    /// use iced_lens::diagnostics::{MediaMetadata, PathAnonymizer};
    ///
    /// let anonymizer = PathAnonymizer::new();
    /// let metadata = MediaMetadata::from_path(Path::new("/home/user/photo.jpg"), &anonymizer);
    ///
    /// assert_eq!(metadata.extension, Some("jpg".to_string()));
    /// assert_eq!(metadata.storage_type, StorageType::Local);
    /// assert!(metadata.path_hash.is_some());
    /// ```
    #[must_use]
    pub fn from_path(path: &Path, anonymizer: &super::PathAnonymizer) -> Self {
        // Extract extension (lowercase, no dot)
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_lowercase);

        // Detect storage type
        let storage_type = StorageType::detect(path);

        // Generate path hash
        let path_hash = Some(anonymizer.hash_path(path));

        Self {
            extension,
            storage_type,
            path_hash,
        }
    }

    /// Creates an empty metadata struct with default values.
    ///
    /// Useful when path information is not available.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }
}

// =============================================================================
// Navigation Context
// =============================================================================

/// Context in which navigation occurred.
///
/// Distinguishes between navigation in Viewer mode (respects filters)
/// and Editor mode (ignores filters, images only).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NavigationContext {
    /// Navigation in Viewer mode (respects active filters)
    Viewer,
    /// Navigation in Editor mode (ignores filters, images only)
    Editor,
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
    NavigateNext {
        /// Context in which navigation occurred (Viewer or Editor)
        context: NavigationContext,
        /// Whether a filter is currently active
        filter_active: bool,
        /// Position in filtered list (None if no filter active)
        #[serde(skip_serializing_if = "Option::is_none")]
        position_in_filtered: Option<usize>,
        /// Position in total (unfiltered) list
        position_in_total: usize,
    },

    /// Navigate to the previous media file.
    NavigatePrevious {
        /// Context in which navigation occurred (Viewer or Editor)
        context: NavigationContext,
        /// Whether a filter is currently active
        filter_active: bool,
        /// Position in filtered list (None if no filter active)
        #[serde(skip_serializing_if = "Option::is_none")]
        position_in_filtered: Option<usize>,
        /// Position in total (unfiltered) list
        position_in_total: usize,
    },

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

    /// Open diagnostics screen.
    OpenDiagnostics,

    /// Enter image editor.
    EnterEditor,

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
    ApplyCrop {
        /// Crop region X offset in pixels
        x: u32,
        /// Crop region Y offset in pixels
        y: u32,
        /// Crop region width in pixels
        width: u32,
        /// Crop region height in pixels
        height: u32,
    },

    /// Apply resize operation.
    ApplyResize {
        /// Scale percentage (100 = original size)
        scale_percent: f32,
        /// New width in pixels
        new_width: u32,
        /// New height in pixels
        new_height: u32,
    },

    /// Apply AI deblur.
    ApplyDeblur,

    /// Apply AI upscale.
    ApplyUpscale {
        /// Scale factor (e.g., 4 for 4x upscale)
        scale_factor: u32,
    },

    /// Undo last edit.
    Undo {
        /// Type of operation that was undone
        #[serde(skip_serializing_if = "Option::is_none")]
        operation_type: Option<String>,
    },

    /// Redo last undone edit.
    Redo {
        /// Type of operation that was redone
        #[serde(skip_serializing_if = "Option::is_none")]
        operation_type: Option<String>,
    },

    /// Save edited image.
    SaveImage {
        /// Export format (e.g., "png", "jpg", "webp")
        format: String,
    },

    /// Return to viewer from editor.
    ReturnToViewer {
        /// Whether there were unsaved changes when returning
        had_unsaved_changes: bool,
    },
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
        /// Exact file size in bytes
        file_size_bytes: u64,
        /// Media dimensions (width × height) when available
        #[serde(skip_serializing_if = "Option::is_none")]
        dimensions: Option<Dimensions>,
        /// File extension (e.g., "jpg", "mp4", "heic")
        #[serde(skip_serializing_if = "Option::is_none")]
        extension: Option<String>,
        /// Storage type (local, network, or unknown)
        #[serde(default)]
        storage_type: StorageType,
        /// 8-char blake3 hash of the path for correlation
        #[serde(skip_serializing_if = "Option::is_none")]
        path_hash: Option<String>,
    },

    /// Media has been successfully loaded.
    MediaLoaded {
        /// Type of media loaded
        media_type: MediaType,
        /// Exact file size in bytes
        file_size_bytes: u64,
        /// Media dimensions (width × height) when available
        #[serde(skip_serializing_if = "Option::is_none")]
        dimensions: Option<Dimensions>,
        /// File extension (e.g., "jpg", "mp4", "heic")
        #[serde(skip_serializing_if = "Option::is_none")]
        extension: Option<String>,
        /// Storage type (local, network, or unknown)
        #[serde(default)]
        storage_type: StorageType,
        /// 8-char blake3 hash of the path for correlation
        #[serde(skip_serializing_if = "Option::is_none")]
        path_hash: Option<String>,
    },

    /// Media loading has failed.
    MediaFailed {
        /// Type of media that failed to load
        media_type: MediaType,
        /// Sanitized reason for failure (no paths)
        reason: String,
        /// File extension (e.g., "jpg", "mp4", "heic")
        #[serde(skip_serializing_if = "Option::is_none")]
        extension: Option<String>,
        /// Storage type (local, network, or unknown)
        #[serde(default)]
        storage_type: StorageType,
        /// 8-char blake3 hash of the path for correlation
        #[serde(skip_serializing_if = "Option::is_none")]
        path_hash: Option<String>,
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

    // -------------------------------------------------------------------------
    // Navigation Filter Events
    // -------------------------------------------------------------------------
    /// Navigation filter has been changed.
    FilterChanged {
        /// Type of filter change that occurred
        filter_type: FilterChangeType,
        /// Whether filter was active before the change
        previous_active: bool,
        /// Whether filter is active after the change
        new_active: bool,
        /// Number of items matching current filter
        filtered_count: usize,
        /// Total number of items (before filtering)
        total_count: usize,
    },

    /// All navigation filters have been cleared/reset.
    FilterCleared {
        /// Whether a media type filter was active before reset
        had_media_type_filter: bool,
        /// Whether a date range filter was active before reset
        had_date_filter: bool,
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
        /// Exact file size in bytes
        file_size_bytes: u64,
        /// Image dimensions (width × height)
        #[serde(skip_serializing_if = "Option::is_none")]
        dimensions: Option<Dimensions>,
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
        /// Exact file size in bytes
        file_size_bytes: u64,
        /// Image dimensions (width × height)
        #[serde(skip_serializing_if = "Option::is_none")]
        dimensions: Option<Dimensions>,
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
        /// Exact file size in bytes
        file_size_bytes: u64,
        /// Image dimensions (width × height)
        #[serde(skip_serializing_if = "Option::is_none")]
        dimensions: Option<Dimensions>,
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
                action: UserAction::NavigateNext {
                    context: NavigationContext::Viewer,
                    filter_active: false,
                    position_in_filtered: None,
                    position_in_total: 0,
                },
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
    // Dimensions Tests
    // =========================================================================

    #[test]
    fn dimensions_new_creates_struct() {
        let dims = Dimensions::new(1920, 1080);
        assert_eq!(dims.width, 1920);
        assert_eq!(dims.height, 1080);
    }

    #[test]
    fn dimensions_serializes_correctly() {
        let dims = Dimensions::new(1920, 1080);
        let json = serde_json::to_string(&dims).expect("serialization should succeed");

        assert!(json.contains("\"width\":1920"));
        assert!(json.contains("\"height\":1080"));
    }

    #[test]
    fn dimensions_deserializes_correctly() {
        let json = r#"{"width":3840,"height":2160}"#;
        let dims: Dimensions = serde_json::from_str(json).expect("deserialization should succeed");

        assert_eq!(dims.width, 3840);
        assert_eq!(dims.height, 2160);
    }

    // =========================================================================
    // StorageType Tests
    // =========================================================================

    #[test]
    fn storage_type_detects_home_as_local() {
        use std::path::Path;

        let path = Path::new("/home/user/photos/image.jpg");
        assert_eq!(StorageType::detect(path), StorageType::Local);

        let path = Path::new("/home/alice/Documents/file.png");
        assert_eq!(StorageType::detect(path), StorageType::Local);
    }

    #[test]
    fn storage_type_detects_users_as_local() {
        use std::path::Path;

        // macOS-style path
        let path = Path::new("/Users/bob/Pictures/photo.jpg");
        assert_eq!(StorageType::detect(path), StorageType::Local);
    }

    #[test]
    fn storage_type_detects_unc_as_network() {
        use std::path::Path;

        let path = Path::new("\\\\server\\share\\folder\\file.jpg");
        assert_eq!(StorageType::detect(path), StorageType::Network);

        let path = Path::new("\\\\nas\\photos\\2024\\vacation.jpg");
        assert_eq!(StorageType::detect(path), StorageType::Network);
    }

    #[test]
    #[cfg(unix)]
    fn storage_type_detects_gvfs_smb_as_network() {
        use std::path::Path;

        // GVFS SMB mount (Linux Mint, Ubuntu, etc.)
        let path = Path::new("/run/user/1000/gvfs/smb-share:server=nas,share=photos/vacation.jpg");
        assert_eq!(StorageType::detect(path), StorageType::Network);

        // Nested subdirectory in SMB share
        let path =
            Path::new("/run/user/1000/gvfs/smb-share:server=fileserver,share=data/docs/file.pdf");
        assert_eq!(StorageType::detect(path), StorageType::Network);
    }

    #[test]
    #[cfg(unix)]
    fn storage_type_detects_gvfs_nfs_as_network() {
        use std::path::Path;

        // GVFS NFS mount
        let path = Path::new("/run/user/1000/gvfs/nfs:host=server,path=/exports/photos/img.jpg");
        assert_eq!(StorageType::detect(path), StorageType::Network);
    }

    #[test]
    fn storage_type_defaults_to_unknown() {
        use std::path::Path;

        // Ambiguous paths should default to Unknown
        let path = Path::new("/tmp/file.jpg");
        assert_eq!(StorageType::detect(path), StorageType::Unknown);

        let path = Path::new("/var/data/file.png");
        assert_eq!(StorageType::detect(path), StorageType::Unknown);

        let path = Path::new("/opt/media/video.mp4");
        assert_eq!(StorageType::detect(path), StorageType::Unknown);

        // Empty path
        let path = Path::new("");
        assert_eq!(StorageType::detect(path), StorageType::Unknown);
    }

    #[test]
    fn storage_type_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&StorageType::Local).unwrap(),
            "\"local\""
        );
        assert_eq!(
            serde_json::to_string(&StorageType::Network).unwrap(),
            "\"network\""
        );
        assert_eq!(
            serde_json::to_string(&StorageType::Unknown).unwrap(),
            "\"unknown\""
        );
    }

    #[test]
    fn storage_type_deserializes_from_json() {
        let local: StorageType = serde_json::from_str("\"local\"").unwrap();
        assert_eq!(local, StorageType::Local);

        let network: StorageType = serde_json::from_str("\"network\"").unwrap();
        assert_eq!(network, StorageType::Network);

        let unknown: StorageType = serde_json::from_str("\"unknown\"").unwrap();
        assert_eq!(unknown, StorageType::Unknown);
    }

    #[test]
    fn storage_type_default_is_unknown() {
        assert_eq!(StorageType::default(), StorageType::Unknown);
    }

    // =========================================================================
    // MediaMetadata Tests
    // =========================================================================

    #[test]
    fn media_metadata_extracts_extension() {
        use crate::diagnostics::PathAnonymizer;
        use std::path::Path;

        let anonymizer = PathAnonymizer::with_seed(42);

        let metadata = MediaMetadata::from_path(Path::new("/home/user/photo.jpg"), &anonymizer);
        assert_eq!(metadata.extension, Some("jpg".to_string()));

        let metadata = MediaMetadata::from_path(Path::new("/home/user/video.MP4"), &anonymizer);
        assert_eq!(metadata.extension, Some("mp4".to_string())); // Lowercase

        let metadata = MediaMetadata::from_path(Path::new("/home/user/image.HEIC"), &anonymizer);
        assert_eq!(metadata.extension, Some("heic".to_string())); // Lowercase
    }

    #[test]
    fn media_metadata_handles_no_extension() {
        use crate::diagnostics::PathAnonymizer;
        use std::path::Path;

        let anonymizer = PathAnonymizer::with_seed(42);

        let metadata = MediaMetadata::from_path(Path::new("/home/user/Makefile"), &anonymizer);
        assert_eq!(metadata.extension, None);

        let metadata = MediaMetadata::from_path(Path::new(""), &anonymizer);
        assert_eq!(metadata.extension, None);
    }

    #[test]
    fn media_metadata_generates_path_hash() {
        use crate::diagnostics::PathAnonymizer;
        use std::path::Path;

        let anonymizer = PathAnonymizer::with_seed(42);

        let metadata = MediaMetadata::from_path(Path::new("/home/user/photo.jpg"), &anonymizer);

        let hash = metadata.path_hash.expect("should have path hash");
        assert_eq!(hash.len(), 8);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn media_metadata_detects_storage_type() {
        use crate::diagnostics::PathAnonymizer;
        use std::path::Path;

        let anonymizer = PathAnonymizer::with_seed(42);

        let metadata = MediaMetadata::from_path(Path::new("/home/user/photo.jpg"), &anonymizer);
        assert_eq!(metadata.storage_type, StorageType::Local);

        let metadata =
            MediaMetadata::from_path(Path::new("\\\\server\\share\\photo.jpg"), &anonymizer);
        assert_eq!(metadata.storage_type, StorageType::Network);

        let metadata = MediaMetadata::from_path(Path::new("/tmp/photo.jpg"), &anonymizer);
        assert_eq!(metadata.storage_type, StorageType::Unknown);
    }

    #[test]
    fn media_metadata_empty_returns_defaults() {
        let metadata = MediaMetadata::empty();

        assert_eq!(metadata.extension, None);
        assert_eq!(metadata.storage_type, StorageType::Unknown);
        assert_eq!(metadata.path_hash, None);
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
            file_size_bytes: 5_242_880, // 5 MB
            dimensions: Some(Dimensions::new(1920, 1080)),
            extension: Some("jpg".to_string()),
            storage_type: StorageType::Local,
            path_hash: Some("abc12345".to_string()),
        };
        let json = serde_json::to_string(&state).expect("serialization should succeed");

        assert!(json.contains("\"state\":\"media_loaded\""));
        assert!(json.contains("\"media_type\":\"image\""));
        assert!(json.contains("\"file_size_bytes\":5242880"));
        assert!(json.contains("\"dimensions\""));
        assert!(json.contains("\"width\":1920"));
        assert!(json.contains("\"height\":1080"));
        assert!(json.contains("\"extension\":\"jpg\""));
        assert!(json.contains("\"storage_type\":\"local\""));
        assert!(json.contains("\"path_hash\":\"abc12345\""));
    }

    #[test]
    fn app_state_event_media_loaded_omits_none_fields() {
        let state = AppStateEvent::MediaLoaded {
            media_type: MediaType::Image,
            file_size_bytes: 1_048_576,
            dimensions: None,
            extension: None,
            storage_type: StorageType::Unknown,
            path_hash: None,
        };
        let json = serde_json::to_string(&state).expect("serialization should succeed");

        assert!(json.contains("\"state\":\"media_loaded\""));
        // None fields should be omitted
        assert!(!json.contains("\"extension\""));
        assert!(!json.contains("\"path_hash\""));
        assert!(!json.contains("\"dimensions\""));
        // storage_type with default (unknown) should still be serialized
        assert!(json.contains("\"storage_type\":\"unknown\""));
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
            file_size_bytes: 52_428_800, // 50 MB
            dimensions: Some(Dimensions::new(4000, 3000)),
            success: true,
        };
        let json = serde_json::to_string(&op).expect("serialization should succeed");

        assert!(json.contains("\"operation\":\"ai_deblur_process\""));
        assert!(json.contains("\"duration_ms\":1500"));
        assert!(json.contains("\"file_size_bytes\":52428800"));
        assert!(json.contains("\"dimensions\""));
        assert!(json.contains("\"success\":true"));
    }

    #[test]
    fn app_operation_ai_upscale_serializes() {
        let op = AppOperation::AIUpscaleProcess {
            duration_ms: 2500,
            scale_factor: 4.0,
            file_size_bytes: 10_485_760, // 10 MB
            dimensions: Some(Dimensions::new(2000, 1500)),
            success: false,
        };
        let json = serde_json::to_string(&op).expect("serialization should succeed");

        assert!(json.contains("\"operation\":\"ai_upscale_process\""));
        assert!(json.contains("\"scale_factor\":4.0"));
        assert!(json.contains("\"file_size_bytes\":10485760"));
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
        let json = r#"{"type":"operation","operation":"resize_image","duration_ms":200,"file_size_bytes":1048576,"dimensions":{"width":800,"height":600}}"#;
        let kind: DiagnosticEventKind =
            serde_json::from_str(json).expect("deserialization should succeed");

        match kind {
            DiagnosticEventKind::Operation { operation } => match operation {
                AppOperation::ResizeImage {
                    duration_ms,
                    file_size_bytes,
                    dimensions,
                } => {
                    assert_eq!(duration_ms, 200);
                    assert_eq!(file_size_bytes, 1_048_576);
                    assert_eq!(dimensions, Some(Dimensions::new(800, 600)));
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

    // =========================================================================
    // FilterChangeType Tests
    // =========================================================================

    #[test]
    fn filter_change_type_media_type_serializes() {
        let change = FilterChangeType::MediaType {
            from: "all".to_string(),
            to: "imagesonly".to_string(),
        };
        let json = serde_json::to_string(&change).expect("serialization should succeed");

        assert!(json.contains("\"change_type\":\"media_type\""));
        assert!(json.contains("\"from\":\"all\""));
        assert!(json.contains("\"to\":\"imagesonly\""));
    }

    #[test]
    fn filter_change_type_date_range_enabled_serializes() {
        let change = FilterChangeType::DateRangeEnabled;
        let json = serde_json::to_string(&change).expect("serialization should succeed");

        assert!(json.contains("\"change_type\":\"date_range_enabled\""));
    }

    #[test]
    fn filter_change_type_date_range_disabled_serializes() {
        let change = FilterChangeType::DateRangeDisabled;
        let json = serde_json::to_string(&change).expect("serialization should succeed");

        assert!(json.contains("\"change_type\":\"date_range_disabled\""));
    }

    #[test]
    fn filter_change_type_date_field_changed_serializes() {
        let change = FilterChangeType::DateFieldChanged {
            field: "modified".to_string(),
        };
        let json = serde_json::to_string(&change).expect("serialization should succeed");

        assert!(json.contains("\"change_type\":\"date_field_changed\""));
        assert!(json.contains("\"field\":\"modified\""));
    }

    #[test]
    fn filter_change_type_date_bound_set_serializes() {
        let change = FilterChangeType::DateBoundSet {
            target: "start".to_string(),
        };
        let json = serde_json::to_string(&change).expect("serialization should succeed");

        assert!(json.contains("\"change_type\":\"date_bound_set\""));
        assert!(json.contains("\"target\":\"start\""));
    }

    #[test]
    fn filter_change_type_date_bound_cleared_serializes() {
        let change = FilterChangeType::DateBoundCleared {
            target: "end".to_string(),
        };
        let json = serde_json::to_string(&change).expect("serialization should succeed");

        assert!(json.contains("\"change_type\":\"date_bound_cleared\""));
        assert!(json.contains("\"target\":\"end\""));
    }

    #[test]
    fn filter_change_type_deserializes() {
        let json = r#"{"change_type":"media_type","from":"videosonly","to":"all"}"#;
        let change: FilterChangeType =
            serde_json::from_str(json).expect("deserialization should succeed");

        match change {
            FilterChangeType::MediaType { from, to } => {
                assert_eq!(from, "videosonly");
                assert_eq!(to, "all");
            }
            _ => panic!("expected MediaType variant"),
        }
    }

    // =========================================================================
    // FilterChanged AppStateEvent Tests
    // =========================================================================

    #[test]
    fn filter_changed_event_serializes() {
        let event = AppStateEvent::FilterChanged {
            filter_type: FilterChangeType::MediaType {
                from: "all".to_string(),
                to: "imagesonly".to_string(),
            },
            previous_active: false,
            new_active: true,
            filtered_count: 42,
            total_count: 100,
        };
        let json = serde_json::to_string(&event).expect("serialization should succeed");

        assert!(json.contains("\"state\":\"filter_changed\""));
        assert!(json.contains("\"change_type\":\"media_type\""));
        assert!(json.contains("\"previous_active\":false"));
        assert!(json.contains("\"new_active\":true"));
        assert!(json.contains("\"filtered_count\":42"));
        assert!(json.contains("\"total_count\":100"));
    }

    #[test]
    fn filter_changed_with_date_range_serializes() {
        let event = AppStateEvent::FilterChanged {
            filter_type: FilterChangeType::DateRangeEnabled,
            previous_active: false,
            new_active: true,
            filtered_count: 50,
            total_count: 200,
        };
        let json = serde_json::to_string(&event).expect("serialization should succeed");

        assert!(json.contains("\"state\":\"filter_changed\""));
        assert!(json.contains("\"change_type\":\"date_range_enabled\""));
    }

    #[test]
    fn filter_changed_event_deserializes() {
        let json = r#"{"state":"filter_changed","filter_type":{"change_type":"date_bound_set","target":"start"},"previous_active":true,"new_active":true,"filtered_count":10,"total_count":50}"#;
        let event: AppStateEvent =
            serde_json::from_str(json).expect("deserialization should succeed");

        match event {
            AppStateEvent::FilterChanged {
                filter_type,
                previous_active,
                new_active,
                filtered_count,
                total_count,
            } => {
                assert!(matches!(filter_type, FilterChangeType::DateBoundSet { .. }));
                assert!(previous_active);
                assert!(new_active);
                assert_eq!(filtered_count, 10);
                assert_eq!(total_count, 50);
            }
            _ => panic!("expected FilterChanged variant"),
        }
    }

    // =========================================================================
    // FilterCleared AppStateEvent Tests
    // =========================================================================

    #[test]
    fn filter_cleared_event_serializes() {
        let event = AppStateEvent::FilterCleared {
            had_media_type_filter: true,
            had_date_filter: false,
        };
        let json = serde_json::to_string(&event).expect("serialization should succeed");

        assert!(json.contains("\"state\":\"filter_cleared\""));
        assert!(json.contains("\"had_media_type_filter\":true"));
        assert!(json.contains("\"had_date_filter\":false"));
    }

    #[test]
    fn filter_cleared_with_both_filters_serializes() {
        let event = AppStateEvent::FilterCleared {
            had_media_type_filter: true,
            had_date_filter: true,
        };
        let json = serde_json::to_string(&event).expect("serialization should succeed");

        assert!(json.contains("\"had_media_type_filter\":true"));
        assert!(json.contains("\"had_date_filter\":true"));
    }

    #[test]
    fn filter_cleared_event_deserializes() {
        let json =
            r#"{"state":"filter_cleared","had_media_type_filter":false,"had_date_filter":true}"#;
        let event: AppStateEvent =
            serde_json::from_str(json).expect("deserialization should succeed");

        match event {
            AppStateEvent::FilterCleared {
                had_media_type_filter,
                had_date_filter,
            } => {
                assert!(!had_media_type_filter);
                assert!(had_date_filter);
            }
            _ => panic!("expected FilterCleared variant"),
        }
    }

    // =========================================================================
    // Editor UserAction Tests (Story 4.3a)
    // =========================================================================

    #[test]
    fn apply_crop_action_serializes() {
        let action = UserAction::ApplyCrop {
            x: 10,
            y: 20,
            width: 800,
            height: 600,
        };
        let json = serde_json::to_string(&action).expect("serialization should succeed");

        assert!(json.contains("\"action\":\"apply_crop\""));
        assert!(json.contains("\"x\":10"));
        assert!(json.contains("\"y\":20"));
        assert!(json.contains("\"width\":800"));
        assert!(json.contains("\"height\":600"));
    }

    #[test]
    fn apply_crop_action_deserializes() {
        let json = r#"{"action":"apply_crop","x":50,"y":100,"width":1920,"height":1080}"#;
        let action: UserAction =
            serde_json::from_str(json).expect("deserialization should succeed");

        match action {
            UserAction::ApplyCrop {
                x,
                y,
                width,
                height,
            } => {
                assert_eq!(x, 50);
                assert_eq!(y, 100);
                assert_eq!(width, 1920);
                assert_eq!(height, 1080);
            }
            _ => panic!("expected ApplyCrop variant"),
        }
    }

    #[test]
    fn apply_resize_action_serializes() {
        let action = UserAction::ApplyResize {
            scale_percent: 150.0,
            new_width: 1920,
            new_height: 1080,
        };
        let json = serde_json::to_string(&action).expect("serialization should succeed");

        assert!(json.contains("\"action\":\"apply_resize\""));
        assert!(json.contains("\"scale_percent\":150.0"));
        assert!(json.contains("\"new_width\":1920"));
        assert!(json.contains("\"new_height\":1080"));
    }

    #[test]
    fn apply_resize_action_deserializes() {
        let json =
            r#"{"action":"apply_resize","scale_percent":75.5,"new_width":800,"new_height":600}"#;
        let action: UserAction =
            serde_json::from_str(json).expect("deserialization should succeed");

        match action {
            UserAction::ApplyResize {
                scale_percent,
                new_width,
                new_height,
            } => {
                assert_relative_eq!(scale_percent, 75.5);
                assert_eq!(new_width, 800);
                assert_eq!(new_height, 600);
            }
            _ => panic!("expected ApplyResize variant"),
        }
    }

    #[test]
    fn apply_deblur_action_serializes() {
        let action = UserAction::ApplyDeblur;
        let json = serde_json::to_string(&action).expect("serialization should succeed");

        assert!(json.contains("\"action\":\"apply_deblur\""));
    }

    #[test]
    fn apply_upscale_action_serializes() {
        let action = UserAction::ApplyUpscale { scale_factor: 4 };
        let json = serde_json::to_string(&action).expect("serialization should succeed");

        assert!(json.contains("\"action\":\"apply_upscale\""));
        assert!(json.contains("\"scale_factor\":4"));
    }

    #[test]
    fn apply_upscale_action_deserializes() {
        let json = r#"{"action":"apply_upscale","scale_factor":2}"#;
        let action: UserAction =
            serde_json::from_str(json).expect("deserialization should succeed");

        match action {
            UserAction::ApplyUpscale { scale_factor } => {
                assert_eq!(scale_factor, 2);
            }
            _ => panic!("expected ApplyUpscale variant"),
        }
    }

    #[test]
    fn save_image_action_serializes() {
        let action = UserAction::SaveImage {
            format: "png".to_string(),
        };
        let json = serde_json::to_string(&action).expect("serialization should succeed");

        assert!(json.contains("\"action\":\"save_image\""));
        assert!(json.contains("\"format\":\"png\""));
    }

    #[test]
    fn save_image_action_deserializes() {
        let json = r#"{"action":"save_image","format":"webp"}"#;
        let action: UserAction =
            serde_json::from_str(json).expect("deserialization should succeed");

        match action {
            UserAction::SaveImage { format } => {
                assert_eq!(format, "webp");
            }
            _ => panic!("expected SaveImage variant"),
        }
    }

    #[test]
    fn undo_action_with_operation_type_serializes() {
        let action = UserAction::Undo {
            operation_type: Some("crop".to_string()),
        };
        let json = serde_json::to_string(&action).expect("serialization should succeed");

        assert!(json.contains("\"action\":\"undo\""));
        assert!(json.contains("\"operation_type\":\"crop\""));
    }

    #[test]
    fn undo_action_without_operation_type_omits_field() {
        let action = UserAction::Undo {
            operation_type: None,
        };
        let json = serde_json::to_string(&action).expect("serialization should succeed");

        assert!(json.contains("\"action\":\"undo\""));
        assert!(!json.contains("operation_type"));
    }

    #[test]
    fn redo_action_with_operation_type_serializes() {
        let action = UserAction::Redo {
            operation_type: Some("resize".to_string()),
        };
        let json = serde_json::to_string(&action).expect("serialization should succeed");

        assert!(json.contains("\"action\":\"redo\""));
        assert!(json.contains("\"operation_type\":\"resize\""));
    }

    #[test]
    fn redo_action_deserializes() {
        let json = r#"{"action":"redo","operation_type":"deblur"}"#;
        let action: UserAction =
            serde_json::from_str(json).expect("deserialization should succeed");

        match action {
            UserAction::Redo { operation_type } => {
                assert_eq!(operation_type, Some("deblur".to_string()));
            }
            _ => panic!("expected Redo variant"),
        }
    }

    #[test]
    fn return_to_viewer_action_serializes() {
        let action = UserAction::ReturnToViewer {
            had_unsaved_changes: true,
        };
        let json = serde_json::to_string(&action).expect("serialization should succeed");

        assert!(json.contains("\"action\":\"return_to_viewer\""));
        assert!(json.contains("\"had_unsaved_changes\":true"));
    }

    #[test]
    fn return_to_viewer_action_deserializes() {
        let json = r#"{"action":"return_to_viewer","had_unsaved_changes":false}"#;
        let action: UserAction =
            serde_json::from_str(json).expect("deserialization should succeed");

        match action {
            UserAction::ReturnToViewer {
                had_unsaved_changes,
            } => {
                assert!(!had_unsaved_changes);
            }
            _ => panic!("expected ReturnToViewer variant"),
        }
    }
}
