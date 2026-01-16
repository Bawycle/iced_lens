// SPDX-License-Identifier: MPL-2.0
//! Save/discard helpers for the editor session.
//!
//! Image dimension conversions between u32 and f32 for display/calculations.
//! Precision loss is acceptable for typical image sizes.
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use super::{CropDragState, CropRatio};
use crate::error::{Error, Result};
use crate::media::image_transform;
use crate::media::metadata_operations::{preserve_metadata_from_bytes, PreservationConfig};
use crate::ui::image_editor::{ImageSource, State};
use std::fs;

/// Result of a save operation, indicating success with optional warnings.
#[derive(Debug, Default)]
pub struct SaveResult {
    /// If metadata preservation failed, contains the i18n key for the warning message.
    pub metadata_warning: Option<&'static str>,
}

impl State {
    /// Save the edited image to a file, preserving the original format and metadata.
    ///
    /// This function:
    /// 1. Saves the pixel data using the `image` crate
    /// 2. Preserves EXIF/XMP metadata from the source image (if editing a file)
    /// 3. Applies metadata transformations based on user options:
    ///    - Strips GPS data if requested
    ///    - Resets orientation tag if image was rotated
    ///    - Adds software tag and modification date if requested
    ///
    /// # Returns
    ///
    /// Returns `Ok(SaveResult)` on success. The `SaveResult` may contain a
    /// `metadata_warning` if metadata preservation failed (the image is still
    /// saved successfully in this case).
    ///
    /// # Errors
    ///
    /// Returns an error if the image format is unsupported or the file
    /// cannot be written.
    pub fn save_image(&mut self, path: &std::path::Path) -> Result<SaveResult> {
        use image_rs::ImageFormat;

        // Detect format from file extension (case-insensitive)
        let ext_lower = path
            .extension()
            .and_then(|s| s.to_str())
            .map(str::to_lowercase);

        // Note: png is listed explicitly for clarity even though it matches the default
        #[allow(clippy::match_same_arms)]
        let format = match ext_lower.as_deref() {
            Some("jpg" | "jpeg") => ImageFormat::Jpeg,
            Some("png") => ImageFormat::Png,
            Some("gif") => ImageFormat::Gif,
            Some("bmp") => ImageFormat::Bmp,
            Some("ico") => ImageFormat::Ico,
            Some("tiff" | "tif") => ImageFormat::Tiff,
            Some("webp") => ImageFormat::WebP,
            _ => ImageFormat::Png, // Default fallback
        };

        // Read source bytes BEFORE saving (in case we're overwriting the same file)
        // This must happen before save_with_format() which would overwrite the file
        let source_data = if let ImageSource::File(source_path) = &self.image_source {
            let source_ext = source_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_string();
            fs::read(source_path).ok().map(|bytes| (bytes, source_ext))
        } else {
            None
        };

        // Save the working image (pixels only)
        self.working_image
            .save_with_format(path, format)
            .map_err(|err| Error::Io(format!("Failed to save image: {err}")))?;

        // Preserve metadata if editing a file (not captured frame)
        let mut result = SaveResult::default();

        if let Some((source_bytes, source_ext)) = source_data {
            // Update orientation_changed flag from transformation history
            self.metadata_options
                .update_from_transformations(&self.transformation_history);

            let config = PreservationConfig {
                strip_gps: self.metadata_options.strip_gps,
                add_software_tag: self.metadata_options.add_software_tag,
                reset_orientation: self.metadata_options.orientation_changed(),
            };

            // Preserve metadata from the bytes we read earlier (best-effort)
            if let Err(e) = preserve_metadata_from_bytes(&source_bytes, &source_ext, path, &config)
            {
                eprintln!("[WARN] Failed to preserve metadata: {e}. Image saved without metadata.");
                // Set warning so caller can notify user
                result.metadata_warning = Some("editor-metadata-write-failed");
            }
        }

        // Clear transformation history after successful save
        self.transformation_history.clear();
        self.history_index = 0;

        Ok(result)
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
        self.crop.x = (self.current_image.width - crop_width) / 2;
        self.crop.y = (self.current_image.height - crop_height) / 2;
        self.crop.width = crop_width;
        self.crop.height = crop_height;
        self.crop.ratio = CropRatio::None;
        self.crop.overlay.visible = false;
        self.crop.overlay.drag_state = CropDragState::None;
        self.crop_modified = false;

        // Hide resize overlay to avoid stale rectangles after cancel
        self.resize.overlay.visible = false;
        self.resize
            .overlay
            .set_original_dimensions(self.current_image.width, self.current_image.height);

        // Clear transformation history
        self.transformation_history.clear();
        self.history_index = 0;

        // Clear preview but keep tool panel open
        self.preview_image = None;
    }
}
