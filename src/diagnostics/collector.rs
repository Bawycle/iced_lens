// SPDX-License-Identifier: MPL-2.0
//! Diagnostics collector for aggregating and storing diagnostic events.
//!
//! This module provides the central collector that receives events from
//! various parts of the application and stores them in a circular buffer.

use std::time::Instant;

use chrono::{DateTime, Utc};
use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};

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
}

impl Default for DiagnosticsCollector {
    fn default() -> Self {
        Self::new(BufferCapacity::default())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

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
}
