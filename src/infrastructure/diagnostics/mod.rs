// SPDX-License-Identifier: MPL-2.0
//! Diagnostics infrastructure re-exports.
//!
//! This module re-exports diagnostics types from the existing `diagnostics` module
//! as part of the infrastructure layer organization.
//!
//! # Design Notes
//!
//! The diagnostics system collects application events, errors, and resource metrics
//! for debugging and support purposes. The key types are:
//!
//! - [`DiagnosticsCollector`]: Central collector that stores events in a circular buffer
//! - [`DiagnosticsHandle`]: Thread-safe handle for sending events to the collector
//! - [`DiagnosticsExporter`]: Trait for export functionality (file, clipboard)
//!
//! # Example
//!
//! ```ignore
//! use iced_lens::infrastructure::diagnostics::{DiagnosticsCollector, DiagnosticsHandle};
//!
//! let collector = DiagnosticsCollector::default();
//! let handle = collector.handle();
//!
//! // Log events from anywhere
//! handle.log_action(UserAction::NavigateNext { ... });
//! ```

use std::path::{Path, PathBuf};

// Re-export collector and handle
pub use crate::diagnostics::{DiagnosticsCollector, DiagnosticsHandle};

// Re-export export error and constants
pub use crate::diagnostics::{ExportError, MAX_CLIPBOARD_SIZE_BYTES};

// Re-export event types
pub use crate::diagnostics::{
    AppOperation, AppStateEvent, BufferCapacity, CollectionStatus, DiagnosticEvent,
    DiagnosticEventKind, DiagnosticReport, Dimensions, ErrorEvent, ErrorType, MediaMetadata,
    MediaType, NavigationContext, ReportMetadata, ResourceMetrics, SerializableEvent, StorageType,
    SystemInfo, UserAction, WarningEvent, WarningType,
};

/// Trait for diagnostic report export operations.
///
/// This trait defines a clean interface for exporting diagnostic reports.
/// The [`DiagnosticsCollector`] implements this trait.
pub trait DiagnosticsExporter {
    /// Exports the diagnostic report to a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    fn export_to_file(&self, path: &Path) -> Result<PathBuf, ExportError>;

    /// Exports the diagnostic report to the system clipboard.
    ///
    /// # Errors
    ///
    /// Returns an error if clipboard access fails or content is too large.
    fn export_to_clipboard(&self) -> Result<(), ExportError>;

    /// Opens a file dialog and exports to the selected location.
    ///
    /// # Errors
    ///
    /// Returns an error if the user cancels or the file cannot be written.
    fn export_with_dialog(&self) -> Result<PathBuf, ExportError>;
}

impl DiagnosticsExporter for DiagnosticsCollector {
    fn export_to_file(&self, path: &Path) -> Result<PathBuf, ExportError> {
        DiagnosticsCollector::export_to_file(self, path)
    }

    fn export_to_clipboard(&self) -> Result<(), ExportError> {
        DiagnosticsCollector::export_to_clipboard(self)
    }

    fn export_with_dialog(&self) -> Result<PathBuf, ExportError> {
        DiagnosticsCollector::export_with_dialog(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collector_can_be_created() {
        let collector = DiagnosticsCollector::default();
        assert!(collector.is_empty());
    }

    #[test]
    fn handle_can_be_obtained() {
        let collector = DiagnosticsCollector::default();
        let _handle = collector.handle();
        // Handle is created successfully
    }

    #[test]
    fn exporter_trait_is_implemented() {
        let collector = DiagnosticsCollector::default();
        // Verify the trait is implemented by using it through trait object
        let _exporter: &dyn DiagnosticsExporter = &collector;
    }
}
