// SPDX-License-Identifier: MPL-2.0
//! Diagnostics collector for aggregating and storing diagnostic events.
//!
//! This module provides the central collector that receives events from
//! various parts of the application and stores them in a circular buffer.

use std::path::{Path, PathBuf};
use std::time::Instant;

use chrono::{DateTime, Utc};
use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};

use super::anonymizer::AnonymizationPipeline;
use super::export::{
    anonymize_event, default_export_directory, generate_default_filename, write_atomic, ExportError,
};
use super::{
    sanitize_message, AppOperation, AppStateEvent, BufferCapacity, CircularBuffer, DiagnosticEvent,
    DiagnosticEventKind, DiagnosticReport, ErrorEvent, ErrorType, ReportMetadata, ResourceMetrics,
    SerializableEvent, SystemInfo, UserAction, WarningEvent, WarningType,
};

/// Handle for sending diagnostic events to the collector.
///
/// This handle is cheap to clone and can be shared across threads.
/// Events are sent via a bounded channel to avoid blocking the UI thread.
#[derive(Clone, Debug)]
pub struct DiagnosticsHandle {
    event_tx: Sender<DiagnosticEvent>,
}

impl DiagnosticsHandle {
    /// Logs a user action event.
    ///
    /// This method is non-blocking and will drop the event if the
    /// internal channel is full (backpressure protection).
    pub fn log_action(&self, action: UserAction) {
        self.log_action_with_details(action, None);
    }

    /// Logs a user action event with optional details.
    ///
    /// This method is non-blocking and will drop the event if the
    /// internal channel is full (backpressure protection).
    pub fn log_action_with_details(&self, action: UserAction, details: Option<String>) {
        let event = DiagnosticEvent::new(DiagnosticEventKind::UserAction { action, details });
        // Non-blocking send - drop if channel is full
        let _ = self.event_tx.try_send(event);
    }

    /// Logs a resource metrics snapshot.
    ///
    /// This method is non-blocking.
    pub fn log_resource_snapshot(&self, metrics: ResourceMetrics) {
        let event = DiagnosticEvent::new(DiagnosticEventKind::ResourceSnapshot { metrics });
        let _ = self.event_tx.try_send(event);
    }

    /// Logs a warning event with full details.
    ///
    /// The message is automatically sanitized to remove file paths.
    /// This method is non-blocking.
    pub fn log_warning(&self, warning_event: WarningEvent) {
        let sanitized_event = WarningEvent {
            message: sanitize_message(&warning_event.message),
            ..warning_event
        };
        let event = DiagnosticEvent::new(DiagnosticEventKind::Warning {
            event: sanitized_event,
        });
        let _ = self.event_tx.try_send(event);
    }

    /// Logs a simple warning message (backward-compatible convenience method).
    ///
    /// The message is automatically sanitized to remove file paths.
    /// Uses `WarningType::Other` as the category.
    /// This method is non-blocking.
    pub fn log_warning_simple(&self, message: impl Into<String>) {
        let warning = WarningEvent::new(WarningType::Other, sanitize_message(&message.into()));
        let event = DiagnosticEvent::new(DiagnosticEventKind::Warning { event: warning });
        let _ = self.event_tx.try_send(event);
    }

    /// Logs an error event with full details.
    ///
    /// The message is automatically sanitized to remove file paths.
    /// This method is non-blocking.
    pub fn log_error(&self, error_event: ErrorEvent) {
        let sanitized_event = ErrorEvent {
            message: sanitize_message(&error_event.message),
            ..error_event
        };
        let event = DiagnosticEvent::new(DiagnosticEventKind::Error {
            event: sanitized_event,
        });
        let _ = self.event_tx.try_send(event);
    }

    /// Logs a simple error message (backward-compatible convenience method).
    ///
    /// The message is automatically sanitized to remove file paths.
    /// Uses `ErrorType::Other` as the category.
    /// This method is non-blocking.
    pub fn log_error_simple(&self, message: impl Into<String>) {
        let error = ErrorEvent::new(ErrorType::Other, sanitize_message(&message.into()));
        let event = DiagnosticEvent::new(DiagnosticEventKind::Error { event: error });
        let _ = self.event_tx.try_send(event);
    }

    /// Logs an application state change event.
    ///
    /// This method is non-blocking and will drop the event if the
    /// internal channel is full (backpressure protection).
    pub fn log_state(&self, state: AppStateEvent) {
        let event = DiagnosticEvent::new(DiagnosticEventKind::AppState { state });
        let _ = self.event_tx.try_send(event);
    }

    /// Logs an application operation event with performance metrics.
    ///
    /// This method is non-blocking and will drop the event if the
    /// internal channel is full (backpressure protection).
    pub fn log_operation(&self, operation: AppOperation) {
        let event = DiagnosticEvent::new(DiagnosticEventKind::Operation { operation });
        let _ = self.event_tx.try_send(event);
    }

    /// Attempts to send an event, returning an error if the channel is full.
    ///
    /// Use this when you need to know if the event was actually sent.
    ///
    /// # Errors
    ///
    /// Returns `TrySendError::Full` if the internal channel buffer is full,
    /// or `TrySendError::Disconnected` if the collector has been dropped.
    pub fn try_log_action(&self, action: UserAction) -> Result<(), TrySendError<DiagnosticEvent>> {
        let event = DiagnosticEvent::new(DiagnosticEventKind::UserAction {
            action,
            details: None,
        });
        self.event_tx.try_send(event)
    }
}

/// Central collector for diagnostic events.
///
/// The collector receives events through a channel and stores them in a
/// memory-bounded circular buffer. Old events are automatically evicted
/// when the buffer reaches capacity.
pub struct DiagnosticsCollector {
    /// Circular buffer storing diagnostic events.
    buffer: CircularBuffer<DiagnosticEvent>,
    /// Receiver for incoming events.
    event_rx: Receiver<DiagnosticEvent>,
    /// Sender stored to create handles.
    event_tx: Sender<DiagnosticEvent>,
    /// When collection started (monotonic clock for duration calculations).
    collection_started_at: Instant,
    /// When collection started (wall clock for report metadata).
    collection_started_at_utc: DateTime<Utc>,
}

/// Default channel capacity for event buffering.
/// This allows some buffering without excessive memory usage.
const DEFAULT_CHANNEL_CAPACITY: usize = 100;

impl DiagnosticsCollector {
    /// Creates a new diagnostics collector with the specified buffer capacity.
    #[must_use]
    pub fn new(capacity: BufferCapacity) -> Self {
        let (event_tx, event_rx) = bounded(DEFAULT_CHANNEL_CAPACITY);

        Self {
            buffer: CircularBuffer::new(capacity),
            event_rx,
            event_tx,
            collection_started_at: Instant::now(),
            collection_started_at_utc: Utc::now(),
        }
    }

    /// Creates a handle for sending events to this collector.
    ///
    /// Handles are cheap to clone and can be distributed to different
    /// parts of the application.
    #[must_use]
    pub fn handle(&self) -> DiagnosticsHandle {
        DiagnosticsHandle {
            event_tx: self.event_tx.clone(),
        }
    }

    /// Processes all pending events from the channel.
    ///
    /// Call this periodically (e.g., on each UI tick) to drain the
    /// event channel and store events in the buffer.
    pub fn process_pending(&mut self) {
        while let Ok(event) = self.event_rx.try_recv() {
            self.buffer.push(event);
        }
    }

    /// Logs an action directly to the buffer (bypassing the channel).
    ///
    /// Use this for synchronous logging when you have direct access
    /// to the collector (e.g., in the main update loop).
    pub fn log_action(&mut self, action: UserAction) {
        self.log_action_with_details(action, None);
    }

    /// Logs an action with details directly to the buffer.
    pub fn log_action_with_details(&mut self, action: UserAction, details: Option<String>) {
        let event = DiagnosticEvent::new(DiagnosticEventKind::UserAction { action, details });
        self.buffer.push(event);
    }

    /// Logs a state change directly to the buffer (bypassing the channel).
    ///
    /// Use this for synchronous logging when you have direct access
    /// to the collector (e.g., in the main update loop).
    pub fn log_state(&mut self, state: AppStateEvent) {
        let event = DiagnosticEvent::new(DiagnosticEventKind::AppState { state });
        self.buffer.push(event);
    }

    /// Logs an operation directly to the buffer (bypassing the channel).
    ///
    /// Use this for synchronous logging when you have direct access
    /// to the collector (e.g., in the main update loop).
    pub fn log_operation(&mut self, operation: AppOperation) {
        let event = DiagnosticEvent::new(DiagnosticEventKind::Operation { operation });
        self.buffer.push(event);
    }

    /// Returns the number of events currently stored.
    #[must_use]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns true if no events are stored.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Returns an iterator over all stored events (oldest first).
    pub fn iter(&self) -> impl Iterator<Item = &DiagnosticEvent> {
        self.buffer.iter()
    }

    /// Clears all stored events.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Returns the buffer capacity.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Returns the current collection status.
    ///
    /// Note: Until Story 3.3 integrates `ResourceCollector`, this always
    /// returns `Disabled` for resource collection. Event collection is
    /// always active.
    #[must_use]
    pub fn get_status(&self) -> super::CollectionStatus {
        // Placeholder until ResourceCollector is integrated in Story 3.3
        // For now, report as Disabled since no resource metrics are being collected
        super::CollectionStatus::Disabled
    }

    /// Returns how long the collector has been running.
    #[must_use]
    pub fn get_collection_duration(&self) -> std::time::Duration {
        self.collection_started_at.elapsed()
    }

    /// Exports all collected events as a JSON diagnostic report.
    ///
    /// The report includes:
    /// - Metadata (report ID, timestamps, version, event count)
    /// - System information (OS, CPU cores, RAM)
    /// - All events with relative timestamps
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization fails.
    pub fn export_json(&self) -> serde_json::Result<String> {
        let report = self.build_report();
        serde_json::to_string_pretty(&report)
    }

    /// Builds a diagnostic report from the current buffer contents.
    #[allow(clippy::cast_possible_truncation)] // Duration in ms fits comfortably in u64
    fn build_report(&self) -> DiagnosticReport {
        let collection_duration_ms = self.collection_started_at.elapsed().as_millis() as u64;

        let events: Vec<SerializableEvent> = self
            .buffer
            .iter()
            .map(|event| {
                SerializableEvent::new(
                    event.timestamp,
                    self.collection_started_at,
                    event.kind.clone(),
                )
            })
            .collect();

        let metadata = ReportMetadata::new(
            self.collection_started_at_utc,
            collection_duration_ms,
            events.len(),
        );

        let system_info = SystemInfo::collect();

        DiagnosticReport::new(metadata, system_info, events)
    }

    /// Builds an anonymized diagnostic report.
    ///
    /// All string fields that may contain PII (IP addresses, domains, usernames)
    /// are processed through the anonymization pipeline before inclusion.
    #[allow(clippy::cast_possible_truncation)] // Duration in ms fits comfortably in u64
    fn build_anonymized_report(&self) -> DiagnosticReport {
        let pipeline = AnonymizationPipeline::new();
        let collection_duration_ms = self.collection_started_at.elapsed().as_millis() as u64;

        let events: Vec<SerializableEvent> = self
            .buffer
            .iter()
            .map(|event| {
                let serializable = SerializableEvent::new(
                    event.timestamp,
                    self.collection_started_at,
                    event.kind.clone(),
                );
                anonymize_event(&serializable, &pipeline)
            })
            .collect();

        let metadata = ReportMetadata::new(
            self.collection_started_at_utc,
            collection_duration_ms,
            events.len(),
        );

        let system_info = SystemInfo::collect();

        DiagnosticReport::new(metadata, system_info, events)
    }

    /// Exports an anonymized diagnostic report to a file.
    ///
    /// The report is anonymized (IPs, domains, usernames hashed) before writing.
    /// The file is written atomically to prevent corruption.
    ///
    /// # Arguments
    ///
    /// * `path` - The path where the report should be saved
    ///
    /// # Returns
    ///
    /// Returns `Ok(PathBuf)` with the actual path written, or an error.
    ///
    /// # Errors
    ///
    /// Returns `ExportError::Io` if file operations fail.
    /// Returns `ExportError::Serialization` if JSON serialization fails.
    pub fn export_to_file(&self, path: impl AsRef<Path>) -> Result<PathBuf, ExportError> {
        let path = path.as_ref();
        let report = self.build_anonymized_report();
        let json = serde_json::to_string_pretty(&report)?;

        write_atomic(path, &json)?;

        Ok(path.to_path_buf())
    }

    /// Exports an anonymized diagnostic report using a native file dialog.
    ///
    /// Opens a save file dialog with a default filename and the user's
    /// Documents folder as the initial directory. The report is anonymized
    /// before saving.
    ///
    /// # Returns
    ///
    /// Returns `Ok(PathBuf)` with the path where the file was saved.
    ///
    /// # Errors
    ///
    /// Returns `ExportError::Cancelled` if the user cancels the dialog.
    /// Returns `ExportError::Io` if file operations fail.
    /// Returns `ExportError::Serialization` if JSON serialization fails.
    pub fn export_with_dialog(&self) -> Result<PathBuf, ExportError> {
        let default_dir = default_export_directory();
        let default_name = generate_default_filename();

        let path = rfd::FileDialog::new()
            .set_directory(&default_dir)
            .set_file_name(&default_name)
            .add_filter("JSON", &["json"])
            .save_file()
            .ok_or(ExportError::Cancelled)?;

        self.export_to_file(&path)
    }

    /// Exports an anonymized diagnostic report to the system clipboard.
    ///
    /// The report is anonymized (IPs, domains, usernames hashed) and formatted
    /// as pretty JSON before copying to the clipboard.
    ///
    /// # Errors
    ///
    /// Returns `ExportError::ContentTooLarge` if the JSON exceeds 10 MB.
    /// Returns `ExportError::Clipboard` if clipboard access fails.
    /// This can happen on headless systems or if permissions are denied.
    /// Returns `ExportError::Serialization` if JSON serialization fails.
    pub fn export_to_clipboard(&self) -> Result<(), ExportError> {
        use super::export::MAX_CLIPBOARD_SIZE_BYTES;

        let report = self.build_anonymized_report();
        let json = serde_json::to_string_pretty(&report)?;

        // Check content size before attempting clipboard operation
        if json.len() > MAX_CLIPBOARD_SIZE_BYTES {
            return Err(ExportError::ContentTooLarge {
                size: json.len(),
                max_size: MAX_CLIPBOARD_SIZE_BYTES,
            });
        }

        let mut clipboard =
            arboard::Clipboard::new().map_err(|e| ExportError::Clipboard(e.to_string()))?;

        clipboard
            .set_text(&json)
            .map_err(|e| ExportError::Clipboard(e.to_string()))?;

        Ok(())
    }
}

impl Default for DiagnosticsCollector {
    fn default() -> Self {
        Self::new(BufferCapacity::default())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use approx::assert_relative_eq;

    use super::*;
    use crate::diagnostics::{MediaType, SizeCategory};

    #[test]
    fn collector_new_creates_empty_buffer() {
        let collector = DiagnosticsCollector::new(BufferCapacity::default());

        assert!(collector.is_empty());
        assert_eq!(collector.len(), 0);
    }

    #[test]
    fn collector_log_action_stores_event() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());

        collector.log_action(UserAction::NavigateNext);

        assert_eq!(collector.len(), 1);
    }

    #[test]
    fn collector_log_action_with_details_stores_event() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());

        collector.log_action_with_details(
            UserAction::LoadMedia {
                source: Some("file_dialog".to_string()),
            },
            Some("test.jpg".to_string()),
        );

        assert_eq!(collector.len(), 1);

        let event = collector.iter().next().unwrap();
        match &event.kind {
            DiagnosticEventKind::UserAction { action, details } => {
                assert!(matches!(action, UserAction::LoadMedia { .. }));
                assert_eq!(details.as_deref(), Some("test.jpg"));
            }
            _ => panic!("expected UserAction event"),
        }
    }

    #[test]
    fn handle_log_action_sends_to_collector() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        handle.log_action(UserAction::TogglePlayback);

        // Event is in channel, not yet in buffer
        assert!(collector.is_empty());

        // Process pending events
        collector.process_pending();

        assert_eq!(collector.len(), 1);
    }

    #[test]
    fn handle_log_action_with_details_sends_to_collector() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        handle.log_action_with_details(
            UserAction::SeekVideo {
                position_secs: 42.5,
            },
            Some("user clicked timeline".to_string()),
        );

        collector.process_pending();

        assert_eq!(collector.len(), 1);

        let event = collector.iter().next().unwrap();
        match &event.kind {
            DiagnosticEventKind::UserAction { action, details } => {
                match action {
                    UserAction::SeekVideo { position_secs } => {
                        assert!((position_secs - 42.5).abs() < f64::EPSILON);
                    }
                    _ => panic!("expected SeekVideo action"),
                }
                assert_eq!(details.as_deref(), Some("user clicked timeline"));
            }
            _ => panic!("expected UserAction event"),
        }
    }

    #[test]
    fn handle_log_warning_sends_to_collector() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        handle.log_warning(WarningEvent::new(
            WarningType::FileNotFound,
            "test warning message",
        ));

        collector.process_pending();

        assert_eq!(collector.len(), 1);

        let event = collector.iter().next().unwrap();
        match &event.kind {
            DiagnosticEventKind::Warning { event } => {
                assert_eq!(event.message, "test warning message");
                assert_eq!(event.warning_type, WarningType::FileNotFound);
            }
            _ => panic!("expected Warning event"),
        }
    }

    #[test]
    fn handle_log_warning_simple_sends_to_collector() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        handle.log_warning_simple("simple warning");

        collector.process_pending();

        assert_eq!(collector.len(), 1);

        let event = collector.iter().next().unwrap();
        match &event.kind {
            DiagnosticEventKind::Warning { event } => {
                assert_eq!(event.message, "simple warning");
                assert_eq!(event.warning_type, WarningType::Other);
            }
            _ => panic!("expected Warning event"),
        }
    }

    #[test]
    fn handle_log_error_sends_to_collector() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        handle.log_error(ErrorEvent::new(ErrorType::IoError, "test error message"));

        collector.process_pending();

        assert_eq!(collector.len(), 1);

        let event = collector.iter().next().unwrap();
        match &event.kind {
            DiagnosticEventKind::Error { event } => {
                assert_eq!(event.message, "test error message");
                assert_eq!(event.error_type, ErrorType::IoError);
            }
            _ => panic!("expected Error event"),
        }
    }

    #[test]
    fn handle_log_error_simple_sends_to_collector() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        handle.log_error_simple("simple error");

        collector.process_pending();

        assert_eq!(collector.len(), 1);

        let event = collector.iter().next().unwrap();
        match &event.kind {
            DiagnosticEventKind::Error { event } => {
                assert_eq!(event.message, "simple error");
                assert_eq!(event.error_type, ErrorType::Other);
            }
            _ => panic!("expected Error event"),
        }
    }

    #[test]
    fn handle_is_clone() {
        let collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle1 = collector.handle();
        let handle2 = handle1.clone();

        // Both handles should work
        assert!(handle1.try_log_action(UserAction::ZoomIn).is_ok());
        assert!(handle2.try_log_action(UserAction::ZoomOut).is_ok());
    }

    #[test]
    fn collector_clear_removes_all_events() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());

        collector.log_action(UserAction::NavigateNext);
        collector.log_action(UserAction::NavigatePrevious);

        assert_eq!(collector.len(), 2);

        collector.clear();

        assert!(collector.is_empty());
    }

    #[test]
    fn collector_iter_returns_events_in_order() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());

        collector.log_action(UserAction::NavigateNext);
        std::thread::sleep(Duration::from_millis(1)); // Ensure different timestamps
        collector.log_action(UserAction::NavigatePrevious);

        let events: Vec<_> = collector.iter().collect();
        assert_eq!(events.len(), 2);

        // First event should have earlier timestamp
        assert!(events[0].timestamp <= events[1].timestamp);
    }

    #[test]
    fn collector_default_uses_default_capacity() {
        let collector = DiagnosticsCollector::default();

        assert_eq!(
            collector.capacity(),
            crate::config::DEFAULT_DIAGNOSTICS_BUFFER_CAPACITY
        );
    }

    #[test]
    fn user_action_serializes_to_json() {
        let action = UserAction::SeekVideo {
            position_secs: 10.5,
        };
        let json = serde_json::to_string(&action).expect("serialization should succeed");

        assert!(json.contains("\"action\":\"seek_video\""));
        assert!(json.contains("\"position_secs\":10.5"));
    }

    #[test]
    fn user_action_deserializes_from_json() {
        let json = r#"{"action":"navigate_next"}"#;
        let action: UserAction =
            serde_json::from_str(json).expect("deserialization should succeed");

        assert_eq!(action, UserAction::NavigateNext);
    }

    #[test]
    fn user_action_with_data_deserializes_from_json() {
        let json = r#"{"action":"set_volume","volume":0.75}"#;
        let action: UserAction =
            serde_json::from_str(json).expect("deserialization should succeed");

        match action {
            UserAction::SetVolume { volume } => {
                assert!((volume - 0.75).abs() < f32::EPSILON);
            }
            _ => panic!("expected SetVolume action"),
        }
    }

    #[test]
    fn diagnostic_event_kind_user_action_serializes() {
        let kind = DiagnosticEventKind::UserAction {
            action: UserAction::ToggleFullscreen,
            details: Some("keyboard shortcut".to_string()),
        };

        let json = serde_json::to_string(&kind).expect("serialization should succeed");

        assert!(json.contains("\"type\":\"user_action\""));
        assert!(json.contains("\"action\":\"toggle_fullscreen\""));
        assert!(json.contains("\"details\":\"keyboard shortcut\""));
    }

    #[test]
    fn collector_log_state_stores_event() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());

        collector.log_state(AppStateEvent::VideoPlaying {
            position_secs: 10.5,
        });

        assert_eq!(collector.len(), 1);

        let event = collector.iter().next().unwrap();
        match &event.kind {
            DiagnosticEventKind::AppState { state } => {
                assert!(matches!(state, AppStateEvent::VideoPlaying { .. }));
            }
            _ => panic!("expected AppState event"),
        }
    }

    #[test]
    fn collector_log_operation_stores_event() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());

        collector.log_operation(AppOperation::DecodeFrame { duration_ms: 16 });

        assert_eq!(collector.len(), 1);

        let event = collector.iter().next().unwrap();
        match &event.kind {
            DiagnosticEventKind::Operation { operation } => {
                assert!(matches!(operation, AppOperation::DecodeFrame { .. }));
            }
            _ => panic!("expected Operation event"),
        }
    }

    #[test]
    fn handle_log_state_sends_to_collector() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        handle.log_state(AppStateEvent::EditorOpened { tool: None });

        // Event is in channel, not yet in buffer
        assert!(collector.is_empty());

        // Process pending events
        collector.process_pending();

        assert_eq!(collector.len(), 1);

        let event = collector.iter().next().unwrap();
        match &event.kind {
            DiagnosticEventKind::AppState { state } => {
                assert!(matches!(state, AppStateEvent::EditorOpened { .. }));
            }
            _ => panic!("expected AppState event"),
        }
    }

    #[test]
    fn handle_log_operation_sends_to_collector() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        handle.log_operation(AppOperation::AIDeblurProcess {
            duration_ms: 1500,
            size_category: crate::diagnostics::SizeCategory::Medium,
            success: true,
        });

        // Event is in channel, not yet in buffer
        assert!(collector.is_empty());

        // Process pending events
        collector.process_pending();

        assert_eq!(collector.len(), 1);

        let event = collector.iter().next().unwrap();
        match &event.kind {
            DiagnosticEventKind::Operation { operation } => match operation {
                AppOperation::AIDeblurProcess {
                    duration_ms,
                    success,
                    ..
                } => {
                    assert_eq!(*duration_ms, 1500);
                    assert!(*success);
                }
                _ => panic!("expected AIDeblurProcess operation"),
            },
            _ => panic!("expected Operation event"),
        }
    }

    #[test]
    fn log_warning_sanitizes_paths() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        handle.log_warning(WarningEvent::new(
            WarningType::FileNotFound,
            "Cannot open /home/user/secret/file.txt",
        ));

        collector.process_pending();

        let event = collector.iter().next().unwrap();
        match &event.kind {
            DiagnosticEventKind::Warning { event } => {
                assert_eq!(event.message, "Cannot open <path>");
                assert!(!event.message.contains("/home/"));
            }
            _ => panic!("expected Warning event"),
        }
    }

    #[test]
    fn log_error_sanitizes_paths() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        handle.log_error(ErrorEvent::new(
            ErrorType::IoError,
            "Failed to read C:\\Users\\name\\private\\data.txt",
        ));

        collector.process_pending();

        let event = collector.iter().next().unwrap();
        match &event.kind {
            DiagnosticEventKind::Error { event } => {
                assert_eq!(event.message, "Failed to read <path>");
                assert!(!event.message.contains("C:\\"));
            }
            _ => panic!("expected Error event"),
        }
    }

    #[test]
    fn log_warning_simple_sanitizes_paths() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        handle.log_warning_simple("Error at /tmp/iced_lens/cache");

        collector.process_pending();

        let event = collector.iter().next().unwrap();
        match &event.kind {
            DiagnosticEventKind::Warning { event } => {
                assert!(!event.message.contains("/tmp/"));
            }
            _ => panic!("expected Warning event"),
        }
    }

    // =========================================================================
    // Integration Tests for JSON Export
    // =========================================================================

    #[test]
    fn export_json_full_pipeline() {
        use crate::diagnostics::ResourceMetrics;

        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        // Add sample events via handle (async path)
        handle.log_resource_snapshot(ResourceMetrics::new(
            25.5,
            2_000_000_000,
            8_000_000_000,
            500,
            1000,
        ));
        handle.log_action(UserAction::NavigateNext);
        handle.log_warning(WarningEvent::new(
            WarningType::UnsupportedFormat,
            "Test warning message",
        ));
        handle.log_error(ErrorEvent::new(
            ErrorType::DecodeError,
            "Test error message",
        ));

        // Process pending events
        collector.process_pending();

        // Also add some events directly (sync path)
        collector.log_state(AppStateEvent::MediaLoaded {
            media_type: crate::diagnostics::MediaType::Image,
            size_category: crate::diagnostics::SizeCategory::Small,
        });
        collector.log_operation(AppOperation::DecodeFrame { duration_ms: 8 });

        // Export to JSON
        let json = collector.export_json().expect("export should succeed");

        // Parse JSON to verify structure
        let parsed: serde_json::Value =
            serde_json::from_str(&json).expect("JSON should be parseable");

        // Verify metadata section
        let metadata = parsed.get("metadata").expect("should have metadata");
        assert!(metadata.get("report_id").is_some());
        assert!(metadata.get("generated_at").is_some());
        assert!(metadata.get("iced_lens_version").is_some());
        assert!(metadata.get("collection_started_at").is_some());
        assert!(metadata.get("collection_duration_ms").is_some());
        assert_eq!(metadata.get("event_count").unwrap().as_u64().unwrap(), 6);

        // Verify system_info section
        let system_info = parsed.get("system_info").expect("should have system_info");
        assert!(system_info.get("os").is_some());
        assert!(system_info.get("os_version").is_some());
        assert!(system_info.get("cpu_cores").is_some());
        assert!(system_info.get("ram_total_mb").is_some());

        // Verify events section
        let events = parsed
            .get("events")
            .expect("should have events")
            .as_array()
            .expect("events should be array");
        assert_eq!(events.len(), 6);

        // Verify each event has timestamp_ms and type
        for event in events {
            assert!(event.get("timestamp_ms").is_some());
            assert!(event.get("type").is_some());
        }

        // Verify event types in order
        assert_eq!(events[0].get("type").unwrap(), "resource_snapshot");
        assert_eq!(events[1].get("type").unwrap(), "user_action");
        assert_eq!(events[2].get("type").unwrap(), "warning");
        assert_eq!(events[3].get("type").unwrap(), "error");
        assert_eq!(events[4].get("type").unwrap(), "app_state");
        assert_eq!(events[5].get("type").unwrap(), "operation");
    }

    #[test]
    fn export_json_with_empty_buffer() {
        let collector = DiagnosticsCollector::new(BufferCapacity::default());

        let json = collector.export_json().expect("export should succeed");
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let events = parsed.get("events").unwrap().as_array().unwrap();
        assert!(events.is_empty());

        let event_count = parsed
            .get("metadata")
            .unwrap()
            .get("event_count")
            .unwrap()
            .as_u64()
            .unwrap();
        assert_eq!(event_count, 0);
    }

    #[test]
    fn export_json_timestamps_are_relative() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());

        // First event should have timestamp near 0
        collector.log_action(UserAction::NavigateNext);

        // Wait a bit
        std::thread::sleep(Duration::from_millis(50));

        // Second event should have timestamp around 50ms
        collector.log_action(UserAction::NavigatePrevious);

        let json = collector.export_json().expect("export should succeed");
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let events = parsed.get("events").unwrap().as_array().unwrap();

        let ts0 = events[0].get("timestamp_ms").unwrap().as_u64().unwrap();
        let ts1 = events[1].get("timestamp_ms").unwrap().as_u64().unwrap();

        // First timestamp should be very small (< 10ms since collector creation)
        assert!(ts0 < 10, "first timestamp should be near 0, got {ts0}");

        // Second timestamp should be at least 50ms after the first
        assert!(
            ts1 >= ts0 + 50,
            "second timestamp should be at least 50ms after first: ts0={ts0}, ts1={ts1}"
        );
    }

    #[test]
    fn instrumentation_overhead_is_minimal() {
        use std::time::Instant;

        let collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        // Measure time to log 1000 events via handle (channel-based, non-blocking)
        let start = Instant::now();
        for _ in 0..1000 {
            handle.log_action(UserAction::TogglePlayback);
        }
        let elapsed = start.elapsed();

        // Should complete 1000 logs in < 1ms (channel send is very fast)
        assert!(
            elapsed.as_micros() < 1000,
            "1000 log_action calls should complete in < 1ms, took {} µs",
            elapsed.as_micros()
        );
    }

    // ============================================================
    // Story 1.11: Instrumentation Integration Tests
    // ============================================================

    // --- Task 1: User Action Event Tests ---

    #[test]
    fn test_navigate_actions_have_correct_structure() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        handle.log_action(UserAction::NavigateNext);
        handle.log_action(UserAction::NavigatePrevious);

        collector.process_pending();

        assert_eq!(collector.len(), 2);

        let events: Vec<_> = collector.iter().collect();

        match &events[0].kind {
            DiagnosticEventKind::UserAction { action, details } => {
                assert!(matches!(action, UserAction::NavigateNext));
                assert!(details.is_none());
            }
            _ => panic!("expected UserAction event"),
        }

        match &events[1].kind {
            DiagnosticEventKind::UserAction { action, details } => {
                assert!(matches!(action, UserAction::NavigatePrevious));
                assert!(details.is_none());
            }
            _ => panic!("expected UserAction event"),
        }
    }

    #[test]
    fn test_load_media_captures_source() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        handle.log_action(UserAction::LoadMedia {
            source: Some("file_dialog".to_string()),
        });

        collector.process_pending();

        let event = collector.iter().next().unwrap();
        match &event.kind {
            DiagnosticEventKind::UserAction { action, .. } => match action {
                UserAction::LoadMedia { source } => {
                    assert_eq!(source.as_deref(), Some("file_dialog"));
                }
                _ => panic!("expected LoadMedia action"),
            },
            _ => panic!("expected UserAction event"),
        }
    }

    #[test]
    fn test_editor_actions_captured() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        handle.log_action(UserAction::EnterEditor);
        handle.log_action(UserAction::ApplyDeblur);
        handle.log_action(UserAction::SaveImage);
        handle.log_action(UserAction::ReturnToViewer);

        collector.process_pending();

        assert_eq!(collector.len(), 4);

        let events: Vec<_> = collector.iter().collect();

        assert!(matches!(
            &events[0].kind,
            DiagnosticEventKind::UserAction {
                action: UserAction::EnterEditor,
                ..
            }
        ));
        assert!(matches!(
            &events[1].kind,
            DiagnosticEventKind::UserAction {
                action: UserAction::ApplyDeblur,
                ..
            }
        ));
        assert!(matches!(
            &events[2].kind,
            DiagnosticEventKind::UserAction {
                action: UserAction::SaveImage,
                ..
            }
        ));
        assert!(matches!(
            &events[3].kind,
            DiagnosticEventKind::UserAction {
                action: UserAction::ReturnToViewer,
                ..
            }
        ));
    }

    // --- Task 2: State Transition Event Tests ---

    #[test]
    fn test_editor_opened_closed_lifecycle() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        handle.log_state(AppStateEvent::EditorOpened { tool: None });
        handle.log_state(AppStateEvent::EditorClosed {
            had_unsaved_changes: false,
        });

        collector.process_pending();

        assert_eq!(collector.len(), 2);

        let events: Vec<_> = collector.iter().collect();

        assert!(matches!(
            &events[0].kind,
            DiagnosticEventKind::AppState {
                state: AppStateEvent::EditorOpened { .. }
            }
        ));
        assert!(matches!(
            &events[1].kind,
            DiagnosticEventKind::AppState {
                state: AppStateEvent::EditorClosed { .. }
            }
        ));
    }

    #[test]
    fn test_video_state_events_captured() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        handle.log_state(AppStateEvent::VideoPlaying { position_secs: 0.0 });
        handle.log_state(AppStateEvent::VideoPaused {
            position_secs: 10.5,
        });
        handle.log_state(AppStateEvent::VideoSeeking { target_secs: 30.0 });
        handle.log_state(AppStateEvent::VideoBuffering {
            position_secs: 30.0,
        });
        handle.log_state(AppStateEvent::VideoAtEndOfStream);

        collector.process_pending();

        assert_eq!(collector.len(), 5);

        let events: Vec<_> = collector.iter().collect();

        // Verify VideoPlaying with position 0.0
        match &events[0].kind {
            DiagnosticEventKind::AppState {
                state: AppStateEvent::VideoPlaying { position_secs },
            } => assert_relative_eq!(*position_secs, 0.0),
            _ => panic!("expected VideoPlaying event"),
        }

        // Verify VideoPaused with position 10.5
        match &events[1].kind {
            DiagnosticEventKind::AppState {
                state: AppStateEvent::VideoPaused { position_secs },
            } => assert_relative_eq!(*position_secs, 10.5),
            _ => panic!("expected VideoPaused event"),
        }

        // Verify VideoSeeking with target 30.0
        match &events[2].kind {
            DiagnosticEventKind::AppState {
                state: AppStateEvent::VideoSeeking { target_secs },
            } => assert_relative_eq!(*target_secs, 30.0),
            _ => panic!("expected VideoSeeking event"),
        }

        // Verify VideoBuffering
        assert!(matches!(
            &events[3].kind,
            DiagnosticEventKind::AppState {
                state: AppStateEvent::VideoBuffering { .. }
            }
        ));

        // Verify VideoAtEndOfStream
        assert!(matches!(
            &events[4].kind,
            DiagnosticEventKind::AppState {
                state: AppStateEvent::VideoAtEndOfStream
            }
        ));
    }

    #[test]
    fn test_media_loading_lifecycle_events() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        handle.log_state(AppStateEvent::MediaLoadingStarted {
            media_type: MediaType::Image,
            size_category: SizeCategory::Medium,
        });
        handle.log_state(AppStateEvent::MediaLoaded {
            media_type: MediaType::Image,
            size_category: SizeCategory::Medium,
        });

        collector.process_pending();

        assert_eq!(collector.len(), 2);

        let events: Vec<_> = collector.iter().collect();

        match &events[0].kind {
            DiagnosticEventKind::AppState { state } => match state {
                AppStateEvent::MediaLoadingStarted {
                    media_type,
                    size_category,
                } => {
                    assert!(matches!(media_type, MediaType::Image));
                    assert!(matches!(size_category, SizeCategory::Medium));
                }
                _ => panic!("expected MediaLoadingStarted"),
            },
            _ => panic!("expected AppState event"),
        }

        match &events[1].kind {
            DiagnosticEventKind::AppState { state } => match state {
                AppStateEvent::MediaLoaded {
                    media_type,
                    size_category,
                } => {
                    assert!(matches!(media_type, MediaType::Image));
                    assert!(matches!(size_category, SizeCategory::Medium));
                }
                _ => panic!("expected MediaLoaded"),
            },
            _ => panic!("expected AppState event"),
        }
    }

    // --- Task 3: Operation Event Tests ---

    #[test]
    fn test_ai_deblur_operation_has_valid_duration() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        handle.log_operation(AppOperation::AIDeblurProcess {
            duration_ms: 1500,
            size_category: SizeCategory::Medium,
            success: true,
        });

        collector.process_pending();

        let event = collector.iter().next().unwrap();
        match &event.kind {
            DiagnosticEventKind::Operation { operation } => match operation {
                AppOperation::AIDeblurProcess {
                    duration_ms,
                    size_category,
                    success,
                } => {
                    assert!(*duration_ms > 0, "Duration should be positive");
                    assert!(*duration_ms < 300_000, "Duration should be < 5 minutes");
                    assert!(matches!(size_category, SizeCategory::Medium));
                    assert!(*success);
                }
                _ => panic!("expected AIDeblurProcess"),
            },
            _ => panic!("expected Operation event"),
        }
    }

    #[test]
    fn test_ai_upscale_operation_has_scale_factor() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        handle.log_operation(AppOperation::AIUpscaleProcess {
            duration_ms: 2500,
            scale_factor: 2.0,
            size_category: SizeCategory::Large,
            success: true,
        });

        collector.process_pending();

        let event = collector.iter().next().unwrap();
        match &event.kind {
            DiagnosticEventKind::Operation { operation } => match operation {
                AppOperation::AIUpscaleProcess {
                    duration_ms,
                    scale_factor,
                    size_category,
                    success,
                } => {
                    assert!(*duration_ms > 0);
                    assert!(*duration_ms < 300_000);
                    assert!(*scale_factor > 0.0);
                    assert!(*scale_factor <= 4.0, "Real-ESRGAN max is 4x");
                    assert!(matches!(size_category, SizeCategory::Large));
                    assert!(*success);
                }
                _ => panic!("expected AIUpscaleProcess"),
            },
            _ => panic!("expected Operation event"),
        }
    }

    #[test]
    fn test_video_seek_operation_has_distance() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        handle.log_operation(AppOperation::VideoSeek {
            duration_ms: 150,
            seek_distance_secs: 10.5,
        });

        collector.process_pending();

        let event = collector.iter().next().unwrap();
        match &event.kind {
            DiagnosticEventKind::Operation { operation } => match operation {
                AppOperation::VideoSeek {
                    duration_ms,
                    seek_distance_secs,
                } => {
                    assert!(*duration_ms > 0);
                    assert!(*duration_ms < 300_000);
                    assert!(*seek_distance_secs >= 0.0);
                }
                _ => panic!("expected VideoSeek"),
            },
            _ => panic!("expected Operation event"),
        }
    }

    // --- Task 4: Warning/Error Event Tests ---

    #[test]
    fn test_warning_event_has_correct_type() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        handle.log_warning(WarningEvent::new(
            WarningType::UnsupportedFormat,
            "Format not supported",
        ));

        collector.process_pending();

        let event = collector.iter().next().unwrap();
        match &event.kind {
            DiagnosticEventKind::Warning { event } => {
                assert_eq!(event.warning_type, WarningType::UnsupportedFormat);
                assert_eq!(event.message, "Format not supported");
            }
            _ => panic!("expected Warning event"),
        }
    }

    #[test]
    fn test_error_event_has_correct_type() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        handle.log_error(ErrorEvent::new(
            ErrorType::AIModelError,
            "Model inference failed",
        ));

        collector.process_pending();

        let event = collector.iter().next().unwrap();
        match &event.kind {
            DiagnosticEventKind::Error { event } => {
                assert_eq!(event.error_type, ErrorType::AIModelError);
                assert_eq!(event.message, "Model inference failed");
            }
            _ => panic!("expected Error event"),
        }
    }

    #[test]
    fn test_warning_error_messages_sanitized() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        // Test that messages don't contain file paths (sanitization check)
        let warning_msg = "IO error: permission denied";
        let error_msg = "Failed to load model: network timeout";

        handle.log_warning(WarningEvent::new(WarningType::Other, warning_msg));
        handle.log_error(ErrorEvent::new(ErrorType::IoError, error_msg));

        collector.process_pending();

        let events: Vec<_> = collector.iter().collect();

        // Verify messages are stored correctly (sanitization happens at logging site)
        match &events[0].kind {
            DiagnosticEventKind::Warning { event } => {
                assert!(
                    !event.message.contains('/'),
                    "Message should not contain paths"
                );
                assert!(
                    !event.message.contains('\\'),
                    "Message should not contain paths"
                );
            }
            _ => panic!("expected Warning event"),
        }

        match &events[1].kind {
            DiagnosticEventKind::Error { event } => {
                assert!(
                    !event.message.contains('/'),
                    "Message should not contain paths"
                );
                assert!(
                    !event.message.contains('\\'),
                    "Message should not contain paths"
                );
            }
            _ => panic!("expected Error event"),
        }
    }

    // ============================================================
    // Story 2.4: File Export Integration Tests
    // ============================================================

    #[test]
    fn export_to_file_creates_valid_json() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());

        // Add some events
        collector.log_action(UserAction::NavigateNext);
        collector.log_state(AppStateEvent::MediaLoaded {
            media_type: MediaType::Image,
            size_category: SizeCategory::Small,
        });

        // Export to temp file
        let temp_dir = tempfile::tempdir().expect("should create temp dir");
        let path = temp_dir.path().join("test_export.json");

        let result = collector.export_to_file(&path);
        assert!(result.is_ok());

        // Verify file exists and contains valid JSON
        let content = std::fs::read_to_string(&path).expect("should read file");
        let parsed: serde_json::Value =
            serde_json::from_str(&content).expect("should parse as JSON");

        // Verify structure
        assert!(parsed.get("metadata").is_some());
        assert!(parsed.get("system_info").is_some());
        assert!(parsed.get("events").is_some());
        assert!(parsed.get("summary").is_some());

        // Verify event count
        let events = parsed["events"].as_array().expect("events should be array");
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn export_to_file_anonymizes_ip_addresses() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        // Add event with IP address in message
        handle.log_warning(WarningEvent::new(
            WarningType::NetworkError,
            "Connection failed to 192.168.1.100",
        ));
        collector.process_pending();

        // Export to temp file
        let temp_dir = tempfile::tempdir().expect("should create temp dir");
        let path = temp_dir.path().join("test_anonymize.json");

        collector
            .export_to_file(&path)
            .expect("export should succeed");

        // Read and verify no raw IP addresses
        let content = std::fs::read_to_string(&path).expect("should read file");

        assert!(
            !content.contains("192.168.1.100"),
            "Raw IP should be anonymized"
        );
        assert!(content.contains("<ip:"), "Should contain anonymized IP");
    }

    #[test]
    fn export_to_file_anonymizes_domains() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        // Add event with domain in details
        handle.log_action_with_details(
            UserAction::NavigateNext,
            Some("Fetched from api.example.com".to_string()),
        );
        collector.process_pending();

        // Export to temp file
        let temp_dir = tempfile::tempdir().expect("should create temp dir");
        let path = temp_dir.path().join("test_domain.json");

        collector
            .export_to_file(&path)
            .expect("export should succeed");

        // Read and verify domain is anonymized
        let content = std::fs::read_to_string(&path).expect("should read file");

        assert!(
            !content.contains("example.com"),
            "Raw domain should be anonymized"
        );
        assert!(
            content.contains("<domain:"),
            "Should contain anonymized domain"
        );
    }

    #[test]
    fn export_to_file_returns_correct_path() {
        let collector = DiagnosticsCollector::new(BufferCapacity::default());

        let temp_dir = tempfile::tempdir().expect("should create temp dir");
        let path = temp_dir.path().join("specific_name.json");

        let result = collector
            .export_to_file(&path)
            .expect("export should succeed");

        assert_eq!(result, path);
    }

    #[test]
    fn export_to_file_with_empty_collector() {
        let collector = DiagnosticsCollector::new(BufferCapacity::default());

        let temp_dir = tempfile::tempdir().expect("should create temp dir");
        let path = temp_dir.path().join("empty_export.json");

        collector
            .export_to_file(&path)
            .expect("export should succeed");

        let content = std::fs::read_to_string(&path).expect("should read file");
        let parsed: serde_json::Value = serde_json::from_str(&content).expect("should parse");

        let events = parsed["events"].as_array().expect("events should be array");
        assert!(events.is_empty());
    }

    // ============================================================
    // Story 2.5: Clipboard Export Tests
    // ============================================================

    #[test]
    #[ignore = "Clipboard not available in CI/headless environments"]
    fn export_to_clipboard_works() {
        let collector = DiagnosticsCollector::new(BufferCapacity::default());
        // This may fail in headless CI environments
        let result = collector.export_to_clipboard();
        // Verify it succeeds when clipboard is available
        assert!(result.is_ok());
    }

    #[test]
    fn export_to_clipboard_checks_content_size() {
        // Verify that the size check constant is properly imported and used.
        // A full test would require generating >10MB of events which is impractical.
        // Instead, we verify the error type structure is correct.
        use crate::diagnostics::MAX_CLIPBOARD_SIZE_BYTES;

        let err = ExportError::ContentTooLarge {
            size: MAX_CLIPBOARD_SIZE_BYTES + 1,
            max_size: MAX_CLIPBOARD_SIZE_BYTES,
        };

        // Verify the error displays correctly
        let display = format!("{err}");
        assert!(display.contains("content too large"));
        assert!(display.contains("10.0 MB"));
    }

    #[test]
    fn export_to_clipboard_small_report_does_not_fail_size_check() {
        // A normal collector with few events should be well under 10 MB
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());

        // Add a few events
        collector.log_action(UserAction::NavigateNext);
        collector.log_action(UserAction::ZoomIn);

        // Build the report and verify it's well under the limit
        let json = collector.export_json().expect("should serialize");

        assert!(
            json.len() < crate::diagnostics::MAX_CLIPBOARD_SIZE_BYTES,
            "Normal report should be well under 10 MB limit"
        );
    }

    // ============================================================
    // Story 3.2: Collection Status Tests
    // ============================================================

    #[test]
    fn get_status_returns_disabled_by_default() {
        let collector = DiagnosticsCollector::new(BufferCapacity::default());
        // Until ResourceCollector is integrated in Story 3.3, status is always Disabled
        assert!(matches!(
            collector.get_status(),
            crate::diagnostics::CollectionStatus::Disabled
        ));
    }

    #[test]
    fn get_collection_duration_increases_over_time() {
        let collector = DiagnosticsCollector::new(BufferCapacity::default());
        std::thread::sleep(Duration::from_millis(100));
        let duration = collector.get_collection_duration();
        assert!(
            duration.as_millis() >= 100,
            "Duration should be at least 100ms, got {}ms",
            duration.as_millis()
        );
    }

    #[test]
    fn get_collection_duration_returns_elapsed_time() {
        let collector = DiagnosticsCollector::new(BufferCapacity::default());
        let duration1 = collector.get_collection_duration();

        // Duration should be small initially
        assert!(duration1.as_secs() < 1, "Initial duration should be < 1s");

        // After a short delay, duration should increase
        std::thread::sleep(Duration::from_millis(50));
        let duration2 = collector.get_collection_duration();
        assert!(duration2 > duration1, "Duration should increase over time");
    }
}
