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

pub const ZOOM_INPUT_INVALID_KEY: &str = "viewer-zoom-input-error-invalid";
pub const ZOOM_STEP_INVALID_KEY: &str = "viewer-zoom-step-error-invalid";
pub const ZOOM_STEP_RANGE_KEY: &str = "viewer-zoom-step-error-range";

/// Manages all zoom-related state for the image viewer
#[derive(Debug, Clone)]
pub struct ZoomState {
    /// Current zoom percentage (may be auto-calculated if fit_to_window is true)
    pub zoom_percent: f32,

    /// Last user-set zoom level (restored when disabling fit-to-window)
    pub manual_zoom_percent: f32,

    /// Whether fit-to-window mode is enabled
    pub fit_to_window: bool,

    /// Zoom step percentage for zoom in/out operations
    pub zoom_step_percent: f32,

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
            zoom_step_percent: DEFAULT_ZOOM_STEP_PERCENT,
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
        let clamped = clamp_zoom(percent);
        self.manual_zoom_percent = clamped;
        self.update_zoom_display(clamped);
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
        let current = clamp_zoom(self.zoom_percent);
        self.manual_zoom_percent = current;
        self.update_zoom_display(current);
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
        let new_zoom = self.zoom_percent + self.zoom_step_percent;
        self.apply_manual_zoom(new_zoom);
    }

    /// Applies zoom out by one step
    pub fn zoom_out(&mut self) {
        let new_zoom = self.zoom_percent - self.zoom_step_percent;
        self.apply_manual_zoom(new_zoom);
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
    pub fn zoom_input_value(&self) -> &str {
        &self.zoom_input
    }
}

/// Clamps zoom percentage to valid range
pub fn clamp_zoom(percent: f32) -> f32 {
    percent.clamp(MIN_ZOOM_PERCENT, MAX_ZOOM_PERCENT)
}

/// Formats a number for display (removes unnecessary decimal places)
pub fn format_number(value: f32) -> String {
    if value.fract().abs() < f32::EPSILON {
        format!("{}", value as i32)
    } else {
        format!("{:.1}", value)
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
            zoom_step_percent: 10.0,
            zoom_percent: 100.0,
            ..ZoomState::default()
        };

        state.zoom_in();
        assert_eq!(state.zoom_percent, 110.0);

        state.zoom_out();
        assert_eq!(state.zoom_percent, 100.0);
    }
}
