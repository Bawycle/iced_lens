// SPDX-License-Identifier: MPL-2.0
//! Custom Iced widgets for specialized UI needs.
//!
//! These widgets extend Iced's built-in widget library with application-specific
//! functionality that isn't available out of the box.
//!
//! # Widgets
//!
//! - [`AnimatedSpinner`] - Loading indicator with smooth rotation animation
//! - [`VideoCanvas`] - GPU-accelerated video frame rendering with scaling
//! - [`wheel_blocking_scrollable`] - Scrollable that captures mouse wheel events
//!   to prevent them from propagating (useful for zoom controls)

pub mod animated_spinner;
pub mod video_canvas;
pub mod wheel_blocking_scrollable;

pub use animated_spinner::AnimatedSpinner;
pub use video_canvas::VideoCanvas;
