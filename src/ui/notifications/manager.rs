// SPDX-License-Identifier: MPL-2.0
//! Notification lifecycle management.
//!
//! The `Manager` handles queuing, display timing, and dismissal of notifications.
//! It limits the number of visible toasts and manages auto-dismiss timers.

use super::notification::{Notification, NotificationId, Severity};
use crate::diagnostics::{DiagnosticsHandle, ErrorEvent, ErrorType, WarningEvent, WarningType};
use std::collections::VecDeque;

/// Maximum number of notifications visible at once.
const MAX_VISIBLE: usize = 3;

/// Messages for notification state changes.
#[derive(Debug, Clone)]
pub enum Message {
    /// Dismiss a specific notification by ID.
    Dismiss(NotificationId),
    /// Tick for checking auto-dismiss timers.
    Tick,
}

/// Manages the notification queue and visible notifications.
#[derive(Debug, Default)]
pub struct Manager {
    /// Currently visible notifications (newest first).
    visible: VecDeque<Notification>,
    /// Queued notifications waiting to be displayed.
    queue: VecDeque<Notification>,
    /// Optional diagnostics handle for logging warnings/errors.
    diagnostics: Option<DiagnosticsHandle>,
}

impl Manager {
    /// Creates a new empty notification manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the diagnostics handle for logging warnings and errors.
    pub fn set_diagnostics(&mut self, handle: DiagnosticsHandle) {
        self.diagnostics = Some(handle);
    }

    /// Pushes a new notification to be displayed.
    ///
    /// If fewer than `MAX_VISIBLE` notifications are showing, it's displayed
    /// immediately. Otherwise, it's added to the queue and shown when space
    /// becomes available.
    ///
    /// Warnings and errors are automatically logged to the diagnostics system.
    /// All notification sites should use `with_warning_type()` or `with_error_type()`
    /// to set an explicit diagnostic type. If not set, `Other` is used as fallback.
    pub fn push(&mut self, notification: Notification) {
        // Log warnings and errors to diagnostics
        if let Some(handle) = &self.diagnostics {
            match notification.severity() {
                Severity::Warning => {
                    // All notification sites should use .with_warning_type() explicitly
                    let warning_type = notification.warning_type().unwrap_or(WarningType::Other);
                    handle.log_warning(WarningEvent::new(warning_type, notification.message_key()));
                }
                Severity::Error => {
                    // All notification sites should use .with_error_type() explicitly
                    let error_type = notification.error_type().unwrap_or(ErrorType::Other);
                    handle.log_error(ErrorEvent::new(error_type, notification.message_key()));
                }
                Severity::Success | Severity::Info => {
                    // Success and Info notifications are not logged as diagnostic events
                }
            }
        }

        if self.visible.len() < MAX_VISIBLE {
            self.visible.push_front(notification);
        } else {
            self.queue.push_back(notification);
        }
    }

    /// Dismisses a notification by its ID.
    ///
    /// Returns `true` if the notification was found and removed.
    pub fn dismiss(&mut self, id: NotificationId) -> bool {
        // Try to remove from visible
        if let Some(pos) = self.visible.iter().position(|n| n.id() == id) {
            self.visible.remove(pos);
            self.promote_from_queue();
            return true;
        }

        // Try to remove from queue
        if let Some(pos) = self.queue.iter().position(|n| n.id() == id) {
            self.queue.remove(pos);
            return true;
        }

        false
    }

    /// Processes a tick event, dismissing any notifications that have expired.
    ///
    /// Should be called periodically (e.g., every 100-500ms) to handle auto-dismiss.
    pub fn tick(&mut self) {
        // Collect IDs of notifications to dismiss
        let to_dismiss: Vec<NotificationId> = self
            .visible
            .iter()
            .filter(|n| n.should_auto_dismiss())
            .map(super::notification::Notification::id)
            .collect();

        // Dismiss them
        for id in to_dismiss {
            self.dismiss(id);
        }
    }

    /// Handles a notification message.
    pub fn handle_message(&mut self, message: &Message) {
        match message {
            Message::Dismiss(id) => {
                self.dismiss(*id);
            }
            Message::Tick => {
                self.tick();
            }
        }
    }

    /// Returns the currently visible notifications.
    pub fn visible(&self) -> impl Iterator<Item = &Notification> {
        self.visible.iter()
    }

    /// Returns the number of visible notifications.
    #[must_use]
    pub fn visible_count(&self) -> usize {
        self.visible.len()
    }

    /// Returns the number of queued notifications.
    #[must_use]
    pub fn queued_count(&self) -> usize {
        self.queue.len()
    }

    /// Returns whether there are any notifications (visible or queued).
    #[must_use]
    pub fn has_notifications(&self) -> bool {
        !self.visible.is_empty() || !self.queue.is_empty()
    }

    /// Clears all notifications (visible and queued).
    pub fn clear(&mut self) {
        self.visible.clear();
        self.queue.clear();
    }

    /// Clears all load error notifications.
    ///
    /// This should be called when a media is successfully loaded, to avoid
    /// showing stale error notifications that are no longer relevant.
    pub fn clear_load_errors(&mut self) {
        // Remove from visible
        let visible_before = self.visible.len();
        self.visible
            .retain(|n| !n.message_key().starts_with("notification-load-error-"));

        // Remove from queue
        self.queue
            .retain(|n| !n.message_key().starts_with("notification-load-error-"));

        // Promote from queue if we removed visible notifications
        if self.visible.len() < visible_before {
            self.promote_from_queue();
        }
    }

    /// Promotes a notification from the queue to visible if there's space.
    fn promote_from_queue(&mut self) {
        while self.visible.len() < MAX_VISIBLE {
            if let Some(notification) = self.queue.pop_front() {
                self.visible.push_back(notification);
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_manager_is_empty() {
        let manager = Manager::new();
        assert_eq!(manager.visible_count(), 0);
        assert_eq!(manager.queued_count(), 0);
        assert!(!manager.has_notifications());
    }

    #[test]
    fn push_adds_to_visible_when_space_available() {
        let mut manager = Manager::new();
        manager.push(Notification::success("test"));

        assert_eq!(manager.visible_count(), 1);
        assert_eq!(manager.queued_count(), 0);
    }

    #[test]
    fn push_queues_when_visible_is_full() {
        let mut manager = Manager::new();

        // Fill visible
        for i in 0..MAX_VISIBLE {
            manager.push(Notification::success(format!("test-{i}")));
        }
        assert_eq!(manager.visible_count(), MAX_VISIBLE);
        assert_eq!(manager.queued_count(), 0);

        // Add one more
        manager.push(Notification::success("queued"));
        assert_eq!(manager.visible_count(), MAX_VISIBLE);
        assert_eq!(manager.queued_count(), 1);
    }

    #[test]
    fn dismiss_removes_from_visible() {
        let mut manager = Manager::new();
        let notification = Notification::success("test");
        let id = notification.id();

        manager.push(notification);
        assert_eq!(manager.visible_count(), 1);

        let removed = manager.dismiss(id);
        assert!(removed);
        assert_eq!(manager.visible_count(), 0);
    }

    #[test]
    fn dismiss_promotes_from_queue() {
        let mut manager = Manager::new();

        // Fill visible
        let mut first_id = None;
        for i in 0..MAX_VISIBLE {
            let n = Notification::success(format!("visible-{i}"));
            if i == 0 {
                first_id = Some(n.id());
            }
            manager.push(n);
        }

        // Add to queue
        manager.push(Notification::success("queued"));
        assert_eq!(manager.queued_count(), 1);

        // Dismiss first visible
        manager.dismiss(first_id.unwrap());

        // Queued should have been promoted
        assert_eq!(manager.visible_count(), MAX_VISIBLE);
        assert_eq!(manager.queued_count(), 0);
    }

    #[test]
    fn dismiss_nonexistent_returns_false() {
        let mut manager = Manager::new();
        let fake_id = Notification::success("temp").id();

        assert!(!manager.dismiss(fake_id));
    }

    #[test]
    fn clear_removes_all() {
        let mut manager = Manager::new();

        for i in 0..5 {
            manager.push(Notification::success(format!("test-{i}")));
        }

        manager.clear();
        assert_eq!(manager.visible_count(), 0);
        assert_eq!(manager.queued_count(), 0);
    }

    #[test]
    fn handle_message_dismiss() {
        let mut manager = Manager::new();
        let notification = Notification::success("test");
        let id = notification.id();
        manager.push(notification);

        manager.handle_message(&Message::Dismiss(id));
        assert_eq!(manager.visible_count(), 0);
    }

    #[test]
    fn error_notifications_do_not_auto_dismiss() {
        let mut manager = Manager::new();
        let notification = Notification::error("test-error");
        let id = notification.id();
        manager.push(notification);

        // Tick should not dismiss errors
        manager.tick();
        assert_eq!(manager.visible_count(), 1);

        // Manual dismiss should work
        manager.dismiss(id);
        assert_eq!(manager.visible_count(), 0);
    }

    #[test]
    fn clear_load_errors_removes_only_load_error_notifications() {
        let mut manager = Manager::new();

        // Add various notifications
        manager.push(Notification::error("notification-load-error-io"));
        manager.push(Notification::error("notification-load-error-timeout"));
        manager.push(Notification::success("notification-save-success"));
        manager.push(Notification::error("some-other-error"));

        assert_eq!(manager.visible_count(), 3);
        assert_eq!(manager.queued_count(), 1);

        // Clear load errors
        manager.clear_load_errors();

        // Only non-load-error notifications should remain
        assert_eq!(manager.visible_count(), 2);
        assert_eq!(manager.queued_count(), 0);

        // Verify the remaining notifications are not load errors
        for notification in manager.visible() {
            assert!(
                !notification
                    .message_key()
                    .starts_with("notification-load-error-"),
                "load error notification should have been removed"
            );
        }
    }
}
