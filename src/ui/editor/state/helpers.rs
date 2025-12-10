// SPDX-License-Identifier: MPL-2.0
//! Small helper methods that keep the editor facade lean.

use crate::media::image_transform;
use crate::ui::editor::{EditorTool, State, Transformation};
use image_rs::DynamicImage;

impl State {
    pub(crate) fn apply_dynamic_transformation<F>(
        &mut self,
        transformation: Transformation,
        operation: F,
    ) where
        F: Fn(&DynamicImage) -> DynamicImage,
    {
        let updated = operation(&self.working_image);
        match image_transform::dynamic_to_image_data(&updated) {
            Ok(image_data) => {
                self.working_image = updated;
                self.current_image = image_data;
                self.sync_resize_state_dimensions();
                self.preview_image = None;

                // Reset crop state after rotation to match new dimensions
                if matches!(
                    transformation,
                    Transformation::RotateLeft | Transformation::RotateRight
                ) {
                    self.sync_crop_state_dimensions();
                }

                self.record_transformation(transformation);
            }
            Err(err) => {
                eprintln!("Failed to apply transformation: {err:?}");
            }
        }
    }

    pub(crate) fn sync_resize_state_dimensions(&mut self) {
        self.resize_state.sync_from_image(&self.current_image);
    }

    pub(crate) fn sync_crop_state_dimensions(&mut self) {
        // Reset crop rectangle to cover entire image after rotation
        self.crop_state.x = 0;
        self.crop_state.y = 0;
        self.crop_state.width = self.current_image.width;
        self.crop_state.height = self.current_image.height;

        // Update crop base dimensions if crop tool is active
        if matches!(self.active_tool, Some(EditorTool::Crop)) {
            self.crop_base_width = self.current_image.width;
            self.crop_base_height = self.current_image.height;
            if let Some(ref base_image) = self.crop_base_image {
                // If base image dimensions don't match, update the base image
                if base_image.width() != self.current_image.width
                    || base_image.height() != self.current_image.height
                {
                    self.crop_base_image = Some(self.working_image.clone());
                }
            }
        }
    }

    pub(crate) fn base_width(&self) -> f32 {
        self.current_image.width.max(1) as f32
    }

    pub(crate) fn base_height(&self) -> f32 {
        self.current_image.height.max(1) as f32
    }

    pub(crate) fn commit_active_tool_changes(&mut self) {
        if matches!(self.active_tool, Some(EditorTool::Crop))
            && self.crop_modified
            && self.crop_state.overlay.visible
        {
            self.finalize_crop_overlay();
        }
    }
}
