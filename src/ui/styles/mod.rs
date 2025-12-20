// SPDX-License-Identifier: MPL-2.0
//! Centralized style definitions for UI components.
//!
//! This module provides consistent styling across the application, separating
//! visual presentation from component logic.
//!
//! # Submodules
//!
//! - [`button`] - Button styles (primary, overlay, icon buttons)
//! - [`container`] - Container backgrounds, borders, and shadows
//! - [`editor`] - Image editor specific styles (crop overlay, resize handles)
//! - [`overlay`] - Fullscreen overlay and floating UI styles

pub mod button;
pub mod container;
pub mod editor;
pub mod overlay;

// Re-exports for convenience
pub use button::primary as button_primary;

use iced::widget::svg;
use iced::Theme;

/// Style for SVG icons that tints them with the theme's primary text color.
/// Useful for section header icons that should match the surrounding text.
pub fn tinted_svg(theme: &Theme, _status: svg::Status) -> svg::Style {
    svg::Style {
        color: Some(theme.extended_palette().primary.strong.color),
    }
}
