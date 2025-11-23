// SPDX-License-Identifier: MPL-2.0
//! UI state management modules
//!
//! This module contains all the UI state logic separated from the main App struct,
//! following the principle of separation of concerns.

pub mod zoom;
pub mod viewport;
pub mod drag;

// Re-export commonly used types for convenience
pub use zoom::ZoomState;
pub use viewport::ViewportState;
pub use drag::DragState;
