// SPDX-License-Identifier: MPL-2.0
//! Diagnostics module for collecting and exporting anonymized activity reports.
//!
//! This module provides infrastructure for capturing diagnostic events during
//! application usage, storing them in a memory-bounded circular buffer, and
//! exporting them as anonymized JSON reports for performance analysis.
//!
//! # Architecture
//!
//! - [`CircularBuffer`]: Generic ring buffer with configurable capacity
//! - [`DiagnosticEvent`]: Enum representing different types of diagnostic events
//! - [`BufferCapacity`]: Newtype for validated buffer capacity bounds
//!
//! # Privacy
//!
//! All exported data is anonymized before export. File paths, IP addresses,
//! and usernames are hashed to protect user privacy while preserving
//! diagnostic value.

mod anonymizer;
mod buffer;
mod collector;
mod events;
mod export;
mod report;
mod resource_collector;
mod sanitizer;

use std::time::Instant;

/// Current status of diagnostic data collection.
#[derive(Debug, Clone)]
pub enum CollectionStatus {
    /// Resource collection is disabled (`ResourceCollector` not running).
    Disabled,
    /// Resource collection is active.
    Enabled {
        /// When collection started (monotonic).
        started_at: Instant,
    },
    /// Resource collection encountered an error.
    Error {
        /// Error description.
        message: String,
    },
}

pub use anonymizer::{AnonymizationPipeline, IdentityAnonymizer, PathAnonymizer};
pub use buffer::{BufferCapacity, CircularBuffer};
pub use collector::{DiagnosticsCollector, DiagnosticsHandle};
// CollectionStatus is defined at module level and exported directly
pub use events::{
    AIModel, AppOperation, AppStateEvent, DiagnosticEvent, DiagnosticEventKind, EditorTool,
    ErrorEvent, MediaMetadata, MediaType, SizeCategory, StorageType, UserAction, WarningEvent,
};
pub use export::{ExportError, MAX_CLIPBOARD_SIZE_BYTES};
pub use report::{
    DiagnosticReport, DiskType, ReportMetadata, ReportSummary, ResourceStats, SerializableEvent,
    SystemInfo,
};
pub use resource_collector::{ResourceCollector, ResourceMetrics, SamplingInterval};
pub use sanitizer::{sanitize_message, ErrorType, WarningType};
