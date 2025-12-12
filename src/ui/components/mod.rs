// SPDX-License-Identifier: MPL-2.0
//! Reusable UI components shared across multiple screens.
//!
//! These components encapsulate common UI patterns that appear in different
//! parts of the application, promoting consistency and reducing duplication.
//!
//! # Components
//!
//! - [`checkerboard`] - Transparency checkerboard background pattern for
//!   displaying images with alpha channels
//! - [`error_display`] - Consistent error presentation with severity levels,
//!   expandable technical details, and i18n support

pub mod checkerboard;
pub mod error_display;
