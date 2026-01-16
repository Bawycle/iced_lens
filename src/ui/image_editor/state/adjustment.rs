// SPDX-License-Identifier: MPL-2.0
//! Adjustment tool state and helpers for brightness/contrast.

use crate::media::image_transform;
use crate::ui::image_editor::{State, Transformation};

// Re-export domain type
#[allow(unused_imports)] // Used by tests and may be used by external consumers
pub use crate::domain::editing::newtypes::adjustment_bounds;
pub use crate::domain::editing::AdjustmentPercent;

/// Brightness and contrast adjustment state.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AdjustmentState {
    /// Brightness level (guaranteed valid by type).
    pub brightness: AdjustmentPercent,
    /// Contrast level (guaranteed valid by type).
    pub contrast: AdjustmentPercent,
}

impl AdjustmentState {
    /// Returns true if any adjustment has been made (non-neutral values).
    #[must_use]
    pub fn has_changes(&self) -> bool {
        !self.brightness.is_neutral() || !self.contrast.is_neutral()
    }

    /// Reset adjustments to default values.
    pub fn reset(&mut self) {
        self.brightness = AdjustmentPercent::default();
        self.contrast = AdjustmentPercent::default();
    }
}

impl State {
    /// Handle brightness slider change with live preview.
    pub(crate) fn sidebar_brightness_changed(&mut self, value: i32) {
        self.adjustment.brightness = AdjustmentPercent::new(value);
        self.update_adjustment_preview();
    }

    /// Handle contrast slider change with live preview.
    pub(crate) fn sidebar_contrast_changed(&mut self, value: i32) {
        self.adjustment.contrast = AdjustmentPercent::new(value);
        self.update_adjustment_preview();
    }

    /// Apply current adjustments to the image history.
    pub(crate) fn sidebar_apply_adjustments(&mut self) {
        let brightness = self.adjustment.brightness;
        let contrast = self.adjustment.contrast;

        // Only apply if there are actual changes
        if brightness.is_neutral() && contrast.is_neutral() {
            return;
        }

        // Apply brightness first if non-neutral
        if !brightness.is_neutral() {
            let value = brightness.value();
            self.apply_dynamic_transformation(
                Transformation::AdjustBrightness { value },
                move |image| image_transform::adjust_brightness(image, value),
            );
        }

        // Apply contrast if non-neutral
        if !contrast.is_neutral() {
            let value = contrast.value();
            self.apply_dynamic_transformation(
                Transformation::AdjustContrast { value },
                move |image| image_transform::adjust_contrast(image, value),
            );
        }

        // Reset sliders after applying
        self.adjustment.reset();
        self.preview_image = None;
    }

    /// Reset adjustments and clear preview.
    pub(crate) fn sidebar_reset_adjustments(&mut self) {
        self.adjustment.reset();
        self.preview_image = None;
    }

    /// Update the preview image with current adjustment values.
    fn update_adjustment_preview(&mut self) {
        let brightness = self.adjustment.brightness;
        let contrast = self.adjustment.contrast;

        // No adjustments = no preview needed
        if brightness.is_neutral() && contrast.is_neutral() {
            self.preview_image = None;
            return;
        }

        // Apply adjustments to working image for preview
        let mut preview = self.working_image.clone();

        if !brightness.is_neutral() {
            preview = image_transform::adjust_brightness(&preview, brightness.value());
        }

        if !contrast.is_neutral() {
            preview = image_transform::adjust_contrast(&preview, contrast.value());
        }

        if let Ok(image_data) = image_transform::dynamic_to_image_data(&preview) {
            self.preview_image = Some(image_data);
        } else {
            self.preview_image = None;
        }
    }

    /// Prepare adjustment tool when selected.
    pub(crate) fn prepare_adjustment_tool(&mut self) {
        // Reset to defaults when opening the tool
        self.adjustment.reset();
        self.preview_image = None;
    }

    /// Teardown adjustment tool when deselected.
    pub(crate) fn teardown_adjustment_tool(&mut self) {
        // Clear any pending preview
        self.adjustment.reset();
        self.preview_image = None;
    }

    /// Commit pending adjustment changes (called when switching tools).
    pub(crate) fn commit_adjustment_changes(&mut self) {
        if self.adjustment.has_changes() {
            self.sidebar_apply_adjustments();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adjustment_state_default_has_no_changes() {
        let state = AdjustmentState::default();
        assert!(!state.has_changes());
        assert!(state.brightness.is_neutral());
        assert!(state.contrast.is_neutral());
    }

    #[test]
    fn adjustment_state_detects_changes() {
        let mut state = AdjustmentState::default();
        assert!(!state.has_changes());

        state.brightness = AdjustmentPercent::new(10);
        assert!(state.has_changes());

        state.brightness = AdjustmentPercent::default();
        state.contrast = AdjustmentPercent::new(-20);
        assert!(state.has_changes());
    }

    #[test]
    fn adjustment_state_reset_clears_values() {
        let mut state = AdjustmentState {
            brightness: AdjustmentPercent::new(50),
            contrast: AdjustmentPercent::new(-30),
        };
        assert!(state.has_changes());

        state.reset();
        assert!(!state.has_changes());
        assert!(state.brightness.is_neutral());
        assert!(state.contrast.is_neutral());
    }

    #[test]
    fn adjustment_percent_clamps_values() {
        assert_eq!(AdjustmentPercent::new(150).value(), 100);
        assert_eq!(AdjustmentPercent::new(-150).value(), -100);
        assert_eq!(AdjustmentPercent::new(50).value(), 50);
    }

    #[test]
    fn adjustment_percent_boundary_checks() {
        assert!(AdjustmentPercent::new(-100).is_min());
        assert!(AdjustmentPercent::new(100).is_max());
        assert!(AdjustmentPercent::new(0).is_neutral());
        assert!(!AdjustmentPercent::new(50).is_neutral());
    }
}
