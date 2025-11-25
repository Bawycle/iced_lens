// SPDX-License-Identifier: MPL-2.0
//! Resize tool state and helpers.

use crate::image_handler::ImageData;

#[derive(Debug, Clone, PartialEq)]
pub struct ResizeState {
    /// Scale percentage (10-200%)
    pub scale_percent: f32,
    /// Target width in pixels
    pub width: u32,
    /// Target height in pixels
    pub height: u32,
    /// Whether aspect ratio is locked
    pub lock_aspect: bool,
    /// Original aspect ratio
    pub original_aspect: f32,
    /// Width input field value
    pub width_input: String,
    /// Height input field value
    pub height_input: String,
    /// Visual overlay showing original size markers
    pub overlay: ResizeOverlay,
}

impl ResizeState {
    pub fn from_image(image: &ImageData) -> Self {
        let width = image.width;
        let height = image.height;
        let original_aspect = if height == 0 {
            1.0
        } else {
            width as f32 / height.max(1) as f32
        };

        Self {
            scale_percent: 100.0,
            width,
            height,
            lock_aspect: true,
            original_aspect,
            width_input: width.to_string(),
            height_input: height.to_string(),
            overlay: ResizeOverlay {
                visible: false,
                original_width: width,
                original_height: height,
            },
        }
    }

    /// Syncs derived fields with the provided image dimensions.
    pub fn sync_from_image(&mut self, image: &ImageData) {
        self.width = image.width;
        self.height = image.height;
        self.width_input = image.width.to_string();
        self.height_input = image.height.to_string();
        self.scale_percent = 100.0;
        self.original_aspect = if image.height == 0 {
            1.0
        } else {
            image.width as f32 / image.height.max(1) as f32
        };
    }
}

/// Visual overlay for resize tool showing original dimensions
#[derive(Debug, Clone, PartialEq)]
pub struct ResizeOverlay {
    /// Whether the overlay is currently visible
    pub visible: bool,
    /// Original image dimensions for reference
    pub original_width: u32,
    pub original_height: u32,
}

impl ResizeOverlay {
    pub fn set_original_dimensions(&mut self, width: u32, height: u32) {
        self.original_width = width;
        self.original_height = height;
    }
}
