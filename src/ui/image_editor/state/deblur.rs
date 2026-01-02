// SPDX-License-Identifier: MPL-2.0
//! Deblur tool state for AI-powered image deblurring.

use crate::media::image_transform;
use crate::ui::image_editor::{State, Transformation};
use image_rs::DynamicImage;

/// State for the deblur tool.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DeblurState {
    /// Whether the deblur tool is currently processing.
    pub is_processing: bool,
    /// Progress of the current operation (0.0 to 1.0).
    pub progress: f32,
    /// Whether a cancel was requested.
    pub cancel_requested: bool,
    /// Spinner rotation angle in radians (for animated loading indicator).
    pub spinner_rotation: f32,
}

impl DeblurState {
    /// Returns true if the deblur is currently in progress.
    #[must_use]
    pub fn is_busy(&self) -> bool {
        self.is_processing
    }

    /// Start a deblur operation.
    pub fn start_processing(&mut self) {
        self.is_processing = true;
        self.progress = 0.0;
        self.cancel_requested = false;
    }

    /// Update the processing progress.
    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
    }

    /// Request cancellation of the current operation.
    pub fn request_cancel(&mut self) {
        if self.is_processing {
            self.cancel_requested = true;
        }
    }

    /// Update spinner rotation for animation.
    /// Should be called at ~60 FPS for smooth animation.
    pub fn tick_spinner(&mut self) {
        // 180 degrees per second = π radians per second
        // At 60 FPS, each tick adds π/60 radians
        const ROTATION_SPEED: f32 = std::f32::consts::PI / 60.0;
        self.spinner_rotation =
            (self.spinner_rotation + ROTATION_SPEED) % (2.0 * std::f32::consts::PI);
    }

    /// Finish the deblur operation (success or failure).
    pub fn finish_processing(&mut self) {
        self.is_processing = false;
        self.progress = 0.0;
        self.cancel_requested = false;
    }

    /// Reset to default state.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

impl State {
    /// Prepare deblur tool when selected.
    pub(crate) fn prepare_deblur_tool(&mut self) {
        // Reset to defaults when opening the tool
        self.deblur.reset();
    }

    /// Teardown deblur tool when deselected.
    pub(crate) fn teardown_deblur_tool(&mut self) {
        // Cancel any in-progress operation
        if self.deblur.is_processing {
            self.deblur.request_cancel();
        }
        self.deblur.reset();
    }

    /// Apply deblur to the current image.
    ///
    /// Note: The actual deblur operation runs asynchronously.
    /// This method just initiates the operation.
    pub(crate) fn sidebar_apply_deblur(&mut self) {
        // Mark as processing - the actual inference will be handled
        // by the parent application which has access to the DeblurManager
        self.deblur.start_processing();
    }

    /// Cancel the ongoing deblur operation.
    pub(crate) fn sidebar_cancel_deblur(&mut self) {
        self.deblur.request_cancel();
    }

    /// Apply the deblur result to the editor state.
    ///
    /// This records the transformation in history (with the cached result for undo/redo),
    /// updates the working image, and finishes the processing state.
    pub fn apply_deblur_result(&mut self, deblurred_image: DynamicImage) {
        // Record the transformation with the result cached for undo/redo
        self.record_transformation(Transformation::Deblur {
            result: Box::new(deblurred_image.clone()),
        });

        // Update the working image
        self.working_image = deblurred_image;

        // Update the display image
        if let Ok(image_data) = image_transform::dynamic_to_image_data(&self.working_image) {
            self.current_image = image_data;
            self.sync_resize_state_dimensions();
        }

        // Clear any preview and finish processing
        self.preview_image = None;
        self.deblur.finish_processing();
    }

    /// Mark the deblur operation as failed and reset state.
    pub fn deblur_failed(&mut self) {
        self.deblur.finish_processing();
    }

    /// Returns true if a deblur transformation has already been applied.
    ///
    /// Multiple deblur applications are not recommended as they can introduce
    /// artifacts and degrade image quality.
    pub fn has_deblur_applied(&self) -> bool {
        self.transformation_history
            .iter()
            .take(self.history_index)
            .any(|t| matches!(t, Transformation::Deblur { .. }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deblur_state_default_not_busy() {
        let state = DeblurState::default();
        assert!(!state.is_busy());
        assert!(!state.is_processing);
        assert_eq!(state.progress, 0.0);
        assert!(!state.cancel_requested);
    }

    #[test]
    fn deblur_state_start_processing() {
        let mut state = DeblurState::default();
        state.start_processing();

        assert!(state.is_busy());
        assert!(state.is_processing);
        assert_eq!(state.progress, 0.0);
        assert!(!state.cancel_requested);
    }

    #[test]
    fn deblur_state_progress_clamped() {
        let mut state = DeblurState::default();
        state.start_processing();

        state.set_progress(0.5);
        assert_eq!(state.progress, 0.5);

        state.set_progress(1.5);
        assert_eq!(state.progress, 1.0);

        state.set_progress(-0.5);
        assert_eq!(state.progress, 0.0);
    }

    #[test]
    fn deblur_state_cancel() {
        let mut state = DeblurState::default();

        // Cannot cancel when not processing
        state.request_cancel();
        assert!(!state.cancel_requested);

        // Can cancel when processing
        state.start_processing();
        state.request_cancel();
        assert!(state.cancel_requested);
    }

    #[test]
    fn deblur_state_finish_processing() {
        let mut state = DeblurState::default();
        state.start_processing();
        state.set_progress(0.75);
        state.request_cancel();

        state.finish_processing();

        assert!(!state.is_busy());
        assert_eq!(state.progress, 0.0);
        assert!(!state.cancel_requested);
    }
}
