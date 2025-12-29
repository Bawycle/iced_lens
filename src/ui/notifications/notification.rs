// SPDX-License-Identifier: MPL-2.0
//! Core notification data structures.
//!
//! This module defines the `Notification` struct and `Severity` enum
//! used throughout the notification system.

use crate::ui::design_tokens::palette;
use iced::Color;
use std::time::{Duration, Instant};

/// Unique identifier for a notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NotificationId(u64);

impl NotificationId {
    /// Creates a new unique notification ID.
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for NotificationId {
    fn default() -> Self {
        Self::new()
    }
}

/// Severity level determines display duration and visual styling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Severity {
    /// Operation completed successfully (green, 3s duration).
    #[default]
    Success,
    /// Informational message (blue, 3s duration).
    Info,
    /// Warning that doesn't block operation (orange, 5s duration).
    Warning,
    /// Error requiring attention (red, manual dismiss).
    Error,
}

impl Severity {
    /// Returns the primary color for this severity level.
    #[must_use] 
    pub fn color(&self) -> Color {
        match self {
            Severity::Success => palette::SUCCESS_500,
            Severity::Info => palette::INFO_500,
            Severity::Warning => palette::WARNING_500,
            Severity::Error => palette::ERROR_500,
        }
    }

    /// Returns the auto-dismiss duration for this severity.
    /// Returns `None` for errors (manual dismiss required).
    #[must_use] 
    pub fn auto_dismiss_duration(&self) -> Option<Duration> {
        match self {
            Severity::Success | Severity::Info => Some(Duration::from_secs(3)),
            Severity::Warning => Some(Duration::from_secs(5)),
            Severity::Error => None, // Manual dismiss required
        }
    }
}

/// A notification to be displayed to the user.
#[derive(Debug, Clone)]
pub struct Notification {
    /// Unique identifier for this notification.
    id: NotificationId,
    /// Severity level (determines color and auto-dismiss behavior).
    severity: Severity,
    /// The i18n key for the notification message.
    message_key: String,
    /// Optional arguments for message interpolation.
    message_args: Vec<(String, String)>,
    /// When this notification was created.
    created_at: Instant,
    /// Custom auto-dismiss duration (overrides severity default).
    custom_dismiss_duration: Option<Duration>,
}

impl Notification {
    /// Creates a new notification with the given severity and message key.
    ///
    /// The `message_key` should be a valid i18n key that will be resolved
    /// at render time.
    pub fn new(severity: Severity, message_key: impl Into<String>) -> Self {
        Self {
            id: NotificationId::new(),
            severity,
            message_key: message_key.into(),
            message_args: Vec::new(),
            created_at: Instant::now(),
            custom_dismiss_duration: None,
        }
    }

    /// Creates a success notification.
    pub fn success(message_key: impl Into<String>) -> Self {
        Self::new(Severity::Success, message_key)
    }

    /// Creates an info notification.
    pub fn info(message_key: impl Into<String>) -> Self {
        Self::new(Severity::Info, message_key)
    }

    /// Creates a warning notification.
    pub fn warning(message_key: impl Into<String>) -> Self {
        Self::new(Severity::Warning, message_key)
    }

    /// Creates an error notification.
    pub fn error(message_key: impl Into<String>) -> Self {
        Self::new(Severity::Error, message_key)
    }

    /// Adds an argument for message interpolation.
    ///
    /// Arguments are passed to the i18n system when resolving the message.
    #[must_use]
    pub fn with_arg(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.message_args.push((key.into(), value.into()));
        self
    }

    /// Sets a custom auto-dismiss duration, overriding the severity default.
    ///
    /// Useful for notifications that need more time to read (e.g., long file lists).
    #[must_use] 
    pub fn auto_dismiss(mut self, duration: Duration) -> Self {
        self.custom_dismiss_duration = Some(duration);
        self
    }

    /// Returns the notification's unique ID.
    #[must_use] 
    pub fn id(&self) -> NotificationId {
        self.id
    }

    /// Returns the severity level.
    #[must_use] 
    pub fn severity(&self) -> Severity {
        self.severity
    }

    /// Returns the i18n message key.
    #[must_use] 
    pub fn message_key(&self) -> &str {
        &self.message_key
    }

    /// Returns the message arguments for interpolation.
    #[must_use] 
    pub fn message_args(&self) -> &[(String, String)] {
        &self.message_args
    }

    /// Returns when this notification was created.
    #[must_use] 
    pub fn created_at(&self) -> Instant {
        self.created_at
    }

    /// Returns the age of this notification.
    #[must_use] 
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Returns whether this notification should auto-dismiss.
    #[must_use] 
    pub fn should_auto_dismiss(&self) -> bool {
        // Custom duration takes precedence over severity default
        let duration = self
            .custom_dismiss_duration
            .or_else(|| self.severity.auto_dismiss_duration());

        if let Some(d) = duration {
            self.age() >= d
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_ids_are_unique() {
        let n1 = Notification::success("test");
        let n2 = Notification::success("test");
        assert_ne!(n1.id(), n2.id());
    }

    #[test]
    fn severity_colors_are_distinct() {
        let success = Severity::Success.color();
        let info = Severity::Info.color();
        let warning = Severity::Warning.color();
        let error = Severity::Error.color();

        assert_ne!(success, info);
        assert_ne!(success, warning);
        assert_ne!(success, error);
        assert_ne!(info, warning);
        assert_ne!(info, error);
        assert_ne!(warning, error);
    }

    #[test]
    fn error_severity_has_no_auto_dismiss() {
        assert!(Severity::Error.auto_dismiss_duration().is_none());
    }

    #[test]
    fn success_and_info_have_same_duration() {
        assert_eq!(
            Severity::Success.auto_dismiss_duration(),
            Severity::Info.auto_dismiss_duration()
        );
    }

    #[test]
    fn warning_duration_is_longer_than_success() {
        let success_duration = Severity::Success.auto_dismiss_duration().unwrap();
        let warning_duration = Severity::Warning.auto_dismiss_duration().unwrap();
        assert!(warning_duration > success_duration);
    }

    #[test]
    fn notification_builder_pattern_works() {
        let notification = Notification::error("test-error")
            .with_arg("filename", "test.png")
            .with_arg("size", "1024");

        assert_eq!(notification.severity(), Severity::Error);
        assert_eq!(notification.message_key(), "test-error");
        assert_eq!(notification.message_args().len(), 2);
    }

    #[test]
    fn notification_constructors_set_correct_severity() {
        assert_eq!(Notification::success("").severity(), Severity::Success);
        assert_eq!(Notification::info("").severity(), Severity::Info);
        assert_eq!(Notification::warning("").severity(), Severity::Warning);
        assert_eq!(Notification::error("").severity(), Severity::Error);
    }
}
