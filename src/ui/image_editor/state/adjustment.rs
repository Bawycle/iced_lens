// SPDX-License-Identifier: MPL-2.0
//! Adjustment tool state and helpers for brightness/contrast.

use crate::media::image_transform;
use crate::ui::image_editor::{State, Transformation};

/// Brightness and contrast adjustment state.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AdjustmentState {
    /// Brightness level (-100 to +100, 0 = unchanged)
    pub brightness: i32,
    /// Contrast level (-100 to +100, 0 = unchanged)
    pub contrast: i32,
}

impl AdjustmentState {
    /// Returns true if any adjustment has been made (non-zero values).
    pub fn has_changes(&self) -> bool {
        self.brightness != 0 || self.contrast != 0
    }

    /// Reset adjustments to default values.
    pub fn reset(&mut self) {
        self.brightness = 0;
        self.contrast = 0;
    }
}

impl State {
    /// Handle brightness slider change with live preview.
    pub(crate) fn sidebar_brightness_changed(&mut self, value: i32) {
        self.adjustment_state.brightness = value.clamp(-100, 100);
        self.update_adjustment_preview();
    }

    /// Handle contrast slider change with live preview.
    pub(crate) fn sidebar_contrast_changed(&mut self, value: i32) {
        self.adjustment_state.contrast = value.clamp(-100, 100);
        self.update_adjustment_preview();
    }

    /// Apply current adjustments to the image history.
    pub(crate) fn sidebar_apply_adjustments(&mut self) {
        let brightness = self.adjustment_state.brightness;
        let contrast = self.adjustment_state.contrast;

        // Only apply if there are actual changes
        if brightness == 0 && contrast == 0 {
            return;
        }

        // Apply brightness first if non-zero
        if brightness != 0 {
            self.apply_dynamic_transformation(
                Transformation::AdjustBrightness { value: brightness },
                move |image| image_transform::adjust_brightness(image, brightness),
            );
        }

        // Apply contrast if non-zero
        if contrast != 0 {
            self.apply_dynamic_transformation(
                Transformation::AdjustContrast { value: contrast },
                move |image| image_transform::adjust_contrast(image, contrast),
            );
        }

        // Reset sliders after applying
        self.adjustment_state.reset();
        self.preview_image = None;
    }

    /// Reset adjustments and clear preview.
    pub(crate) fn sidebar_reset_adjustments(&mut self) {
        self.adjustment_state.reset();
        self.preview_image = None;
    }

    /// Update the preview image with current adjustment values.
    fn update_adjustment_preview(&mut self) {
        let brightness = self.adjustment_state.brightness;
        let contrast = self.adjustment_state.contrast;

        // No adjustments = no preview needed
        if brightness == 0 && contrast == 0 {
            self.preview_image = None;
            return;
        }

        // Apply adjustments to working image for preview
        let mut preview = self.working_image.clone();

        if brightness != 0 {
            preview = image_transform::adjust_brightness(&preview, brightness);
        }

        if contrast != 0 {
            preview = image_transform::adjust_contrast(&preview, contrast);
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
        self.adjustment_state.reset();
        self.preview_image = None;
    }

    /// Teardown adjustment tool when deselected.
    pub(crate) fn teardown_adjustment_tool(&mut self) {
        // Clear any pending preview
        self.adjustment_state.reset();
        self.preview_image = None;
    }

    /// Commit pending adjustment changes (called when switching tools).
    pub(crate) fn commit_adjustment_changes(&mut self) {
        if self.adjustment_state.has_changes() {
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
        assert_eq!(state.brightness, 0);
        assert_eq!(state.contrast, 0);
    }

    #[test]
    fn adjustment_state_detects_changes() {
        let mut state = AdjustmentState::default();
        assert!(!state.has_changes());

        state.brightness = 10;
        assert!(state.has_changes());

        state.brightness = 0;
        state.contrast = -20;
        assert!(state.has_changes());
    }

    #[test]
    fn adjustment_state_reset_clears_values() {
        let mut state = AdjustmentState {
            brightness: 50,
            contrast: -30,
        };
        assert!(state.has_changes());

        state.reset();
        assert!(!state.has_changes());
        assert_eq!(state.brightness, 0);
        assert_eq!(state.contrast, 0);
    }
}
