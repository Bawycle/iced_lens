// SPDX-License-Identifier: MPL-2.0
//! Save/discard helpers for the editor session.

use super::{CropDragState, CropRatio};
use crate::error::{Error, Result};
use crate::media::image_transform;
use crate::ui::image_editor::{ImageSource, State};

impl State {
    /// Save the edited image to a file, preserving the original format.
    pub fn save_image(&mut self, path: &std::path::Path) -> Result<()> {
        use image_rs::ImageFormat;

        // Detect format from file extension
        let format = match path.extension().and_then(|s| s.to_str()) {
            Some("jpg" | "jpeg") => ImageFormat::Jpeg,
            Some("png") => ImageFormat::Png,
            Some("gif") => ImageFormat::Gif,
            Some("bmp") => ImageFormat::Bmp,
            Some("ico") => ImageFormat::Ico,
            Some("tiff" | "tif") => ImageFormat::Tiff,
            Some("webp") => ImageFormat::WebP,
            _ => ImageFormat::Png, // Default fallback
        };

        // Save the working image
        self.working_image
            .save_with_format(path, format)
            .map_err(|err| Error::Io(format!("Failed to save image: {err}")))?;

        // Clear transformation history after successful save
        self.transformation_history.clear();
        self.history_index = 0;

        Ok(())
    }

    /// Discard all changes and reset to original image state.
    /// For captured frames, this does nothing (no source to reload from).
    pub fn discard_changes(&mut self) {
        let image_path = match &self.image_source {
            ImageSource::File(path) => path.clone(),
            ImageSource::CapturedFrame { .. } => {
                // For captured frames, we can't reload from disk.
                // Just clear the transformation history.
                self.transformation_history.clear();
                self.history_index = 0;
                self.preview_image = None;
                return;
            }
        };

        // Reload the working image from disk
        let Ok(fresh_image) = image_rs::open(&image_path) else {
            return;
        };
        self.working_image = fresh_image;

        let Ok(image_data) = image_transform::dynamic_to_image_data(&self.working_image) else {
            return;
        };

        self.current_image = image_data.clone();
        self.sync_resize_state_dimensions();

        // Reset crop state
        let crop_width = (self.current_image.width as f32 * 0.75).round() as u32;
        let crop_height = (self.current_image.height as f32 * 0.75).round() as u32;
        self.crop_state.x = (self.current_image.width - crop_width) / 2;
        self.crop_state.y = (self.current_image.height - crop_height) / 2;
        self.crop_state.width = crop_width;
        self.crop_state.height = crop_height;
        self.crop_state.ratio = CropRatio::None;
        self.crop_state.overlay.visible = false;
        self.crop_state.overlay.drag_state = CropDragState::None;
        self.crop_modified = false;

        // Hide resize overlay to avoid stale rectangles after cancel
        self.resize_state.overlay.visible = false;
        self.resize_state
            .overlay
            .set_original_dimensions(self.current_image.width, self.current_image.height);

        // Clear transformation history
        self.transformation_history.clear();
        self.history_index = 0;

        // Clear preview but keep tool panel open
        self.preview_image = None;
    }
}
