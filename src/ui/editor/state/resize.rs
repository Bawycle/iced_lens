// SPDX-License-Identifier: MPL-2.0
//! Resize tool state and helpers.

use crate::image_handler::{transform, ImageData};
use crate::ui::editor::{State, Transformation};

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

impl State {
    pub(crate) fn set_resize_percent(&mut self, percent: f32) {
        let clamped = percent.clamp(10.0, 200.0);
        self.resize_state.scale_percent = clamped;
        let width = (self.base_width() * clamped / 100.0).round().max(1.0) as u32;
        let height = (self.base_height() * clamped / 100.0).round().max(1.0) as u32;

        if self.resize_state.lock_aspect {
            self.set_width_preserving_aspect(width);
        } else {
            self.resize_state.width = width;
            self.resize_state.height = height;
            self.resize_state.width_input = width.to_string();
            self.resize_state.height_input = height.to_string();
        }

        self.update_resize_preview();
    }

    pub(crate) fn handle_width_input_change(&mut self, value: String) {
        self.resize_state.width_input = value.clone();
        if let Some(width) = parse_dimension_input(&value) {
            if self.resize_state.lock_aspect {
                self.set_width_preserving_aspect(width);
            } else {
                let width = width.max(1);
                self.resize_state.width = width;
                self.resize_state.width_input = width.to_string();
            }
            self.update_scale_percent_from_width();
        }
    }

    pub(crate) fn handle_height_input_change(&mut self, value: String) {
        self.resize_state.height_input = value.clone();
        if let Some(height) = parse_dimension_input(&value) {
            if self.resize_state.lock_aspect {
                self.set_height_preserving_aspect(height);
                self.update_scale_percent_from_width();
            } else {
                let height = height.max(1);
                self.resize_state.height = height;
                self.resize_state.height_input = height.to_string();
            }
            self.update_resize_preview();
        }
    }

    pub(crate) fn toggle_resize_lock(&mut self) {
        self.resize_state.lock_aspect = !self.resize_state.lock_aspect;
        if self.resize_state.lock_aspect {
            let width = self.resize_state.width;
            self.set_width_preserving_aspect(width);
        }
        self.update_resize_preview();
    }

    pub(crate) fn set_width_preserving_aspect(&mut self, width: u32) {
        let width = width.max(1);
        let aspect = self.resize_state.original_aspect.max(f32::EPSILON);
        let height = (width as f32 / aspect).round().max(1.0) as u32;
        self.resize_state.width = width;
        self.resize_state.height = height;
        self.resize_state.width_input = width.to_string();
        self.resize_state.height_input = height.to_string();
    }

    pub(crate) fn set_height_preserving_aspect(&mut self, height: u32) {
        let height = height.max(1);
        let aspect = self.resize_state.original_aspect.max(f32::EPSILON);
        let width = (height as f32 * aspect).round().max(1.0) as u32;
        self.resize_state.height = height;
        self.resize_state.width = width.max(1);
        self.resize_state.width_input = self.resize_state.width.to_string();
        self.resize_state.height_input = height.to_string();
    }

    pub(crate) fn update_scale_percent_from_width(&mut self) {
        let base_width = self.base_width();
        if base_width <= 0.0 {
            return;
        }
        let percent = (self.resize_state.width as f32 / base_width) * 100.0;
        let clamped = percent.clamp(10.0, 200.0);
        if (clamped - percent).abs() > f32::EPSILON {
            self.set_resize_percent(clamped);
        } else {
            self.resize_state.scale_percent = clamped;
            self.update_resize_preview();
        }
    }

    pub(crate) fn apply_resize_dimensions(&mut self) {
        let target_width = self.resize_state.width.max(1);
        let target_height = self.resize_state.height.max(1);
        if target_width == self.current_image.width && target_height == self.current_image.height {
            return;
        }

        self.apply_dynamic_transformation(
            Transformation::Resize {
                width: target_width,
                height: target_height,
            },
            move |image| transform::resize(image, target_width, target_height),
        );

        self.resize_state
            .overlay
            .set_original_dimensions(self.current_image.width, self.current_image.height);
    }

    pub(crate) fn update_resize_preview(&mut self) {
        // Don't generate preview when overlay is visible - the overlay will show the preview
        if self.resize_state.overlay.visible {
            self.preview_image = None;
            return;
        }

        let target_width = self.resize_state.width.max(1);
        let target_height = self.resize_state.height.max(1);
        if target_width == self.current_image.width && target_height == self.current_image.height {
            self.preview_image = None;
            return;
        }

        let preview_dynamic = transform::resize(&self.working_image, target_width, target_height);
        match transform::dynamic_to_image_data(&preview_dynamic) {
            Ok(image_data) => {
                self.preview_image = Some(image_data);
            }
            Err(err) => {
                eprintln!("Failed to build resize preview: {err:?}");
                self.preview_image = None;
            }
        }
    }
}

fn parse_dimension_input(value: &str) -> Option<u32> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    match trimmed.parse::<u32>() {
        Ok(result) if result > 0 => Some(result),
        _ => None,
    }
}
