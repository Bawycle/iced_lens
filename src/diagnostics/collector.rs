// SPDX-License-Identifier: MPL-2.0
//! Diagnostics collector for aggregating and storing diagnostic events.
//!
//! This module provides the central collector that receives events from
//! various parts of the application and stores them in a circular buffer.

use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};

use super::{
    AppOperation, AppStateEvent, BufferCapacity, CircularBuffer, DiagnosticEvent,
    DiagnosticEventKind, ResourceMetrics, UserAction,
};

/// Handle for sending diagnostic events to the collector.
///
/// This handle is cheap to clone and can be shared across threads.
/// Events are sent via a bounded channel to avoid blocking the UI thread.
#[derive(Clone)]
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

    /// Logs a warning event.
    ///
    /// This method is non-blocking.
    pub fn log_warning(&self, message: impl Into<String>) {
        let event = DiagnosticEvent::new(DiagnosticEventKind::Warning {
            message: message.into(),
        });
        let _ = self.event_tx.try_send(event);
    }

    /// Logs an error event.
    ///
    /// This method is non-blocking.
    pub fn log_error(&self, message: impl Into<String>) {
        let event = DiagnosticEvent::new(DiagnosticEventKind::Error {
            message: message.into(),
        });
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

        handle.log_warning("test warning message");

        collector.process_pending();

        assert_eq!(collector.len(), 1);

        let event = collector.iter().next().unwrap();
        match &event.kind {
            DiagnosticEventKind::Warning { message } => {
                assert_eq!(message, "test warning message");
            }
            _ => panic!("expected Warning event"),
        }
    }

    #[test]
    fn handle_log_error_sends_to_collector() {
        let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
        let handle = collector.handle();

        handle.log_error("test error message");

        collector.process_pending();

        assert_eq!(collector.len(), 1);

        let event = collector.iter().next().unwrap();
        match &event.kind {
            DiagnosticEventKind::Error { message } => {
                assert_eq!(message, "test error message");
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
}
