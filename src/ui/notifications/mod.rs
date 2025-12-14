// SPDX-License-Identifier: MPL-2.0
//! Toast notification system for user feedback.
//!
//! This module provides a non-intrusive notification system following
//! toast/snackbar UX patterns. Notifications appear temporarily to inform
//! users about actions (save success, errors, etc.) without blocking interaction.
//!
//! # Components
//!
//! - [`notification`] - Core `Notification` struct with severity levels
//! - [`manager`] - `NotificationManager` for queuing and lifecycle management
//! - [`toast`] - Toast widget component for rendering notifications
//!
//! # Usage
//!
//! ```ignore
//! use crate::ui::notifications::{Notification, NotificationManager, Severity};
//!
//! // Create a manager
//! let mut manager = NotificationManager::new();
//!
//! // Push a notification
//! manager.push(Notification::new(Severity::Success, "Image saved successfully"));
//!
//! // In your view function, render toasts
//! let toast_overlay = manager.view().map(Message::Notification);
//! ```
//!
//! # Design Considerations
//!
//! - Toast duration: ~3s for success/info, ~5s for warnings, manual dismiss for errors
//! - Max visible toasts: 3 (others are queued)
//! - Position: bottom-right corner
//! - Accessibility: sufficient contrast, screen reader support

mod manager;
mod notification;
mod toast;

pub use manager::{Manager, Message as NotificationMessage};
pub use notification::{Notification, Severity};
pub use toast::Toast;
