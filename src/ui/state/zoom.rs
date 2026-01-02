// SPDX-License-Identifier: MPL-2.0
//! Zoom state management
//!
//! This module handles all zoom-related state and logic, including:
//! - Manual zoom percentage
//! - Fit-to-window mode
//! - Zoom step configuration
//! - Zoom input validation and error handling

// Re-export zoom constants from centralized config for backward compatibility
pub use crate::config::{
    DEFAULT_ZOOM_PERCENT, DEFAULT_ZOOM_STEP_PERCENT, MAX_ZOOM_PERCENT, MAX_ZOOM_STEP_PERCENT,
    MIN_ZOOM_PERCENT, MIN_ZOOM_STEP_PERCENT,
};

/// Zoom percentage, guaranteed to be within valid range (10%–800%).
///
/// This type ensures that zoom values are always valid, eliminating
/// the need for manual clamping at usage sites.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZoomPercent(f32);

impl ZoomPercent {
    /// Creates a new zoom percentage, clamping the value to the valid range.
    #[must_use]
    pub fn new(percent: f32) -> Self {
        Self(percent.clamp(MIN_ZOOM_PERCENT, MAX_ZOOM_PERCENT))
    }

    /// Returns the raw percentage value.
    #[must_use]
    pub fn value(self) -> f32 {
        self.0
    }

    /// Returns the zoom as a multiplier (e.g., 100% → 1.0).
    #[must_use]
    pub fn as_factor(self) -> f32 {
        self.0 / 100.0
    }

    /// Returns whether the zoom is at the minimum value.
    #[must_use]
    pub fn is_min(self) -> bool {
        self.0 <= MIN_ZOOM_PERCENT
    }

    /// Returns whether the zoom is at the maximum value.
    #[must_use]
    pub fn is_max(self) -> bool {
        self.0 >= MAX_ZOOM_PERCENT
    }

    /// Increases zoom by the given step.
    #[must_use]
    pub fn zoom_in(self, step: f32) -> Self {
        Self::new(self.0 + step)
    }

    /// Decreases zoom by the given step.
    #[must_use]
    pub fn zoom_out(self, step: f32) -> Self {
        Self::new(self.0 - step)
    }
}

impl Default for ZoomPercent {
    fn default() -> Self {
        Self(DEFAULT_ZOOM_PERCENT)
    }
}

/// Zoom step percentage, guaranteed to be within valid range (1%–200%).
///
/// This type ensures that zoom step values are always valid, eliminating
/// the need for manual clamping at usage sites.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZoomStep(f32);

impl ZoomStep {
    /// Creates a new zoom step, clamping the value to the valid range.
    #[must_use]
    pub fn new(percent: f32) -> Self {
        Self(percent.clamp(MIN_ZOOM_STEP_PERCENT, MAX_ZOOM_STEP_PERCENT))
    }

    /// Returns the raw percentage value.
    #[must_use]
    pub fn value(self) -> f32 {
        self.0
    }

    /// Returns whether the step is at the minimum value.
    #[must_use]
    pub fn is_min(self) -> bool {
        self.0 <= MIN_ZOOM_STEP_PERCENT
    }

    /// Returns whether the step is at the maximum value.
    #[must_use]
    pub fn is_max(self) -> bool {
        self.0 >= MAX_ZOOM_STEP_PERCENT
    }
}

impl Default for ZoomStep {
    fn default() -> Self {
        Self(DEFAULT_ZOOM_STEP_PERCENT)
    }
}

pub const ZOOM_INPUT_INVALID_KEY: &str = "viewer-zoom-input-error-invalid";
pub const ZOOM_STEP_INVALID_KEY: &str = "viewer-zoom-step-error-invalid";
pub const ZOOM_STEP_RANGE_KEY: &str = "viewer-zoom-step-error-range";

/// Manages all zoom-related state for the image viewer
#[derive(Debug, Clone)]
pub struct ZoomState {
    /// Current zoom percentage (may be auto-calculated if `fit_to_window` is true)
    pub zoom_percent: f32,

    /// Last user-set zoom level (restored when disabling fit-to-window)
    pub manual_zoom_percent: f32,

    /// Whether fit-to-window mode is enabled
    pub fit_to_window: bool,

    /// Zoom step for zoom in/out operations (guaranteed valid by type).
    pub zoom_step: ZoomStep,

    /// Current zoom input string (for the text field)
    pub zoom_input: String,

    /// Whether the zoom input has been modified but not submitted
    pub zoom_input_dirty: bool,

    /// Error key for zoom input validation
    pub zoom_input_error_key: Option<&'static str>,
}

impl Default for ZoomState {
    fn default() -> Self {
        Self {
            zoom_percent: DEFAULT_ZOOM_PERCENT,
            manual_zoom_percent: DEFAULT_ZOOM_PERCENT,
            fit_to_window: true,
            zoom_step: ZoomStep::default(),
            zoom_input: format_number(DEFAULT_ZOOM_PERCENT),
            zoom_input_dirty: false,
            zoom_input_error_key: None,
        }
    }
}

impl ZoomState {
    /// Updates the zoom display to show the given percentage
    pub fn update_zoom_display(&mut self, percent: f32) {
        self.zoom_percent = percent;
        self.zoom_input = format_number(percent);
    }

    /// Applies a manual zoom percentage and disables fit-to-window
    pub fn apply_manual_zoom(&mut self, percent: f32) {
        let zoom = ZoomPercent::new(percent);
        self.manual_zoom_percent = zoom.value();
        self.update_zoom_display(zoom.value());
        self.zoom_input_dirty = false;
        self.zoom_input_error_key = None;
        self.fit_to_window = false;
    }

    /// Enables fit-to-window mode
    pub fn enable_fit_to_window(&mut self) {
        self.fit_to_window = true;
        self.zoom_input_dirty = false;
        self.zoom_input_error_key = None;
    }

    /// Disables fit-to-window mode, preserving current zoom
    pub fn disable_fit_to_window(&mut self) {
        self.fit_to_window = false;
        let current = ZoomPercent::new(self.zoom_percent);
        self.manual_zoom_percent = current.value();
        self.update_zoom_display(current.value());
        self.zoom_input_dirty = false;
        self.zoom_input_error_key = None;
    }

    /// Resets zoom to default values
    pub fn reset_zoom(&mut self) {
        self.zoom_percent = DEFAULT_ZOOM_PERCENT;
        self.manual_zoom_percent = DEFAULT_ZOOM_PERCENT;
        self.zoom_input = format_number(DEFAULT_ZOOM_PERCENT);
        self.zoom_input_dirty = false;
        self.zoom_input_error_key = None;
        self.fit_to_window = false;
    }

    /// Applies zoom in by one step
    pub fn zoom_in(&mut self) {
        let new_zoom = ZoomPercent::new(self.zoom_percent).zoom_in(self.zoom_step.value());
        self.apply_manual_zoom(new_zoom.value());
    }

    /// Applies zoom out by one step
    pub fn zoom_out(&mut self) {
        let new_zoom = ZoomPercent::new(self.zoom_percent).zoom_out(self.zoom_step.value());
        self.apply_manual_zoom(new_zoom.value());
    }

    /// Handles zoom input change
    pub fn on_zoom_input_changed(&mut self, input: String) {
        self.zoom_input = input;
        self.zoom_input_dirty = true;
        self.zoom_input_error_key = None;
    }

    /// Handles zoom input submission
    pub fn on_zoom_input_submitted(&mut self) -> bool {
        self.zoom_input_dirty = false;

        if let Ok(value) = self.zoom_input.trim().parse::<f32>() {
            self.apply_manual_zoom(value);
            true
        } else {
            self.zoom_input_error_key = Some(ZOOM_INPUT_INVALID_KEY);
            false
        }
    }

    /// Gets the zoom input value
    #[must_use]
    pub fn zoom_input_value(&self) -> &str {
        &self.zoom_input
    }
}

/// Clamps zoom percentage to valid range.
///
/// This is a convenience function that uses `ZoomPercent::new()` internally.
/// Prefer using `ZoomPercent` directly for type-safe zoom handling.
#[must_use]
pub fn clamp_zoom(percent: f32) -> f32 {
    ZoomPercent::new(percent).value()
}

/// Formats a number for display (removes unnecessary decimal places)
#[must_use]
pub fn format_number(value: f32) -> String {
    if value.fract().abs() < f32::EPSILON {
        // Value has no fractional part, so it represents an integer exactly
        #[allow(clippy::cast_possible_truncation)]
        let int_value = value as i32;
        format!("{int_value}")
    } else {
        format!("{value:.1}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_zoom_state_is_consistent() {
        let state = ZoomState::default();
        assert!(state.fit_to_window);
        assert_eq!(state.zoom_percent, DEFAULT_ZOOM_PERCENT);
        assert_eq!(state.manual_zoom_percent, DEFAULT_ZOOM_PERCENT);
        assert!(!state.zoom_input_dirty);
        assert!(state.zoom_input_error_key.is_none());
    }

    #[test]
    fn apply_manual_zoom_clamps_and_disables_fit() {
        let mut state = ZoomState {
            fit_to_window: true,
            ..ZoomState::default()
        };

        state.apply_manual_zoom(9999.0);

        assert_eq!(state.zoom_percent, MAX_ZOOM_PERCENT);
        assert!(!state.fit_to_window);
        assert!(!state.zoom_input_dirty);
    }

    #[test]
    fn zoom_in_out_work_correctly() {
        let mut state = ZoomState {
            zoom_step: ZoomStep::new(10.0),
            zoom_percent: 100.0,
            ..ZoomState::default()
        };

        state.zoom_in();
        assert_eq!(state.zoom_percent, 110.0);

        state.zoom_out();
        assert_eq!(state.zoom_percent, 100.0);
    }
}
