// SPDX-License-Identifier: MPL-2.0
//! Resize tool state and helpers.
//!
//! Uses f32 for scale factors and u32 for pixel dimensions.
//! Precision loss in conversions is acceptable for typical image sizes.
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use crate::media::{image_transform, ImageData, ResizeScale};
use crate::ui::image_editor::{State, Transformation};

/// Tracks which dimension input field has uncommitted changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DirtyField {
    #[default]
    None,
    Width,
    Height,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResizeState {
    /// Scale percentage (10-200%), guaranteed to be valid by the type.
    pub scale: ResizeScale,
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
    /// Whether AI upscale processing is in progress
    pub is_upscale_processing: bool,
    /// Whether to use AI upscaling for enlargements (scale > 100%).
    /// This is a per-operation setting, not persisted between sessions.
    pub use_ai_upscale: bool,
    /// Tracks which input field has uncommitted changes (dirty flag pattern).
    /// Used to commit pending edits before other actions.
    pub dirty_field: DirtyField,
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
            scale: ResizeScale::default(),
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
            is_upscale_processing: false,
            // Default to true; will be effective only if model is ready
            use_ai_upscale: true,
            dirty_field: DirtyField::None,
        }
    }

    /// Returns true if the target dimensions differ from the current working image.
    /// The overlay stores the current image dimensions (updated after each resize).
    /// Used to determine whether the "Apply" button should be enabled.
    #[must_use]
    pub fn has_pending_changes(&self) -> bool {
        self.width != self.overlay.original_width || self.height != self.overlay.original_height
    }

    /// Syncs derived fields with the provided image dimensions.
    pub fn sync_from_image(&mut self, image: &ImageData) {
        self.width = image.width;
        self.height = image.height;
        self.width_input = image.width.to_string();
        self.height_input = image.height.to_string();
        self.scale = ResizeScale::default();
        self.original_aspect = if image.height == 0 {
            1.0
        } else {
            image.width as f32 / image.height.max(1) as f32
        };
        // Sync overlay to match current image dimensions so has_pending_changes()
        // correctly detects when user changes the target dimensions.
        self.overlay
            .set_original_dimensions(image.width, image.height);
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
    pub(crate) fn hide_resize_overlay(&mut self) {
        self.resize.overlay.visible = false;
    }

    pub(crate) fn sidebar_scale_changed(&mut self, percent: f32) {
        self.set_resize_percent(percent);
    }

    pub(crate) fn sidebar_width_input_changed(&mut self, value: String) {
        self.handle_width_input_change(value);
    }

    pub(crate) fn sidebar_height_input_changed(&mut self, value: String) {
        self.handle_height_input_change(value);
    }

    pub(crate) fn sidebar_toggle_lock(&mut self) {
        self.toggle_resize_lock();
    }

    pub(crate) fn sidebar_apply_resize(&mut self) {
        self.apply_resize_dimensions();
    }

    /// Returns the target dimensions for the pending resize operation.
    pub fn pending_resize_dimensions(&self) -> (u32, u32) {
        (self.resize.width.max(1), self.resize.height.max(1))
    }

    /// Returns true if the pending resize operation is an enlargement (scale > 100%).
    pub fn is_resize_enlargement(&self) -> bool {
        let (target_width, target_height) = self.pending_resize_dimensions();
        target_width > self.current_image.width || target_height > self.current_image.height
    }

    /// Returns true if the pending resize dimensions differ from current image.
    pub fn has_pending_resize(&self) -> bool {
        let (target_width, target_height) = self.pending_resize_dimensions();
        target_width != self.current_image.width || target_height != self.current_image.height
    }

    /// Applies the result of an AI upscale resize operation.
    /// This is called when the async AI upscaling task completes.
    pub fn apply_upscale_resize_result(&mut self, result: image_rs::DynamicImage) {
        // Clear processing state
        self.resize.is_upscale_processing = false;

        // Record the transformation for undo/redo with cached AI result
        self.record_transformation(Transformation::UpscaleResize {
            result: Box::new(result.clone()),
        });

        // Update the working image
        self.working_image = result;

        // Update the display image
        if let Ok(image_data) = image_transform::dynamic_to_image_data(&self.working_image) {
            self.current_image = image_data;
            self.sync_resize_state_dimensions();
        }

        // Clear any preview
        self.preview_image = None;

        // Update overlay with new dimensions
        self.resize
            .overlay
            .set_original_dimensions(self.current_image.width, self.current_image.height);
    }

    /// Clears the upscale processing state (called on error or fallback).
    pub fn clear_upscale_processing(&mut self) {
        self.resize.is_upscale_processing = false;
    }

    fn set_resize_percent(&mut self, percent: f32) {
        let scale = ResizeScale::new(percent);
        self.resize.scale = scale;
        let width = (self.base_width() * scale.as_factor()).round().max(1.0) as u32;
        let height = (self.base_height() * scale.as_factor()).round().max(1.0) as u32;

        if self.resize.lock_aspect {
            self.set_width_preserving_aspect(width);
        } else {
            self.resize.width = width;
            self.resize.height = height;
            self.resize.width_input = width.to_string();
            self.resize.height_input = height.to_string();
        }

        self.update_resize_preview();
    }

    fn handle_width_input_change(&mut self, value: String) {
        // Store the raw input value and mark as dirty
        // Calculation happens on submit, blur (via commit_dirty), or other actions
        self.resize.width_input = value;
        self.resize.dirty_field = DirtyField::Width;
    }

    fn handle_height_input_change(&mut self, value: String) {
        // Store the raw input value and mark as dirty
        // Calculation happens on submit, blur (via commit_dirty), or other actions
        self.resize.height_input = value;
        self.resize.dirty_field = DirtyField::Height;
    }

    /// Commits any pending (dirty) input field changes.
    /// Call this before any action that depends on dimension values.
    pub(crate) fn commit_dirty_resize_input(&mut self) {
        match self.resize.dirty_field {
            DirtyField::Width => self.commit_width_input(),
            DirtyField::Height => self.commit_height_input(),
            DirtyField::None => {}
        }
    }

    fn commit_width_input(&mut self) {
        self.resize.dirty_field = DirtyField::None;

        if let Some(width) = parse_dimension_input(&self.resize.width_input) {
            let width = width.max(1);
            self.resize.width = width;
            self.resize.width_input = width.to_string();

            if self.resize.lock_aspect {
                let aspect = self.resize.original_aspect.max(f32::EPSILON);
                let height = (width as f32 / aspect).round().max(1.0) as u32;
                self.resize.height = height;
                self.resize.height_input = height.to_string();
            }
            self.update_scale_percent_from_width();
            self.update_resize_preview();
        } else {
            // Invalid input: restore from current width value
            self.resize.width_input = self.resize.width.to_string();
        }
    }

    fn commit_height_input(&mut self) {
        self.resize.dirty_field = DirtyField::None;

        if let Some(height) = parse_dimension_input(&self.resize.height_input) {
            let height = height.max(1);
            self.resize.height = height;
            self.resize.height_input = height.to_string();

            if self.resize.lock_aspect {
                let aspect = self.resize.original_aspect.max(f32::EPSILON);
                let width = (height as f32 * aspect).round().max(1.0) as u32;
                self.resize.width = width;
                self.resize.width_input = width.to_string();
            }
            self.update_scale_percent_from_width();
            self.update_resize_preview();
        } else {
            // Invalid input: restore from current height value
            self.resize.height_input = self.resize.height.to_string();
        }
    }

    pub(crate) fn sidebar_width_input_submitted(&mut self) {
        self.commit_width_input();
    }

    pub(crate) fn sidebar_height_input_submitted(&mut self) {
        self.commit_height_input();
    }

    fn toggle_resize_lock(&mut self) {
        self.resize.lock_aspect = !self.resize.lock_aspect;
        if self.resize.lock_aspect {
            let width = self.resize.width;
            self.set_width_preserving_aspect(width);
        }
        self.update_resize_preview();
    }

    fn set_width_preserving_aspect(&mut self, width: u32) {
        let width = width.max(1);
        let aspect = self.resize.original_aspect.max(f32::EPSILON);
        let height = (width as f32 / aspect).round().max(1.0) as u32;
        self.resize.width = width;
        self.resize.height = height;
        self.resize.width_input = width.to_string();
        self.resize.height_input = height.to_string();
    }

    fn update_scale_percent_from_width(&mut self) {
        let base_width = self.base_width();
        if base_width <= 0.0 {
            return;
        }
        let percent = (self.resize.width as f32 / base_width) * 100.0;
        let scale = ResizeScale::new(percent);
        // If clamping changed the value, recalculate dimensions
        if (scale.value() - percent).abs() > f32::EPSILON {
            self.set_resize_percent(scale.value());
        } else {
            self.resize.scale = scale;
            self.update_resize_preview();
        }
    }

    fn apply_resize_dimensions(&mut self) {
        // Note: commit_dirty_resize_input is called before this method in routing
        let target_width = self.resize.width.max(1);
        let target_height = self.resize.height.max(1);
        if target_width == self.current_image.width && target_height == self.current_image.height {
            return;
        }

        self.apply_dynamic_transformation(
            Transformation::Resize {
                width: target_width,
                height: target_height,
            },
            move |image| image_transform::resize(image, target_width, target_height),
        );

        self.resize
            .overlay
            .set_original_dimensions(self.current_image.width, self.current_image.height);
    }

    fn update_resize_preview(&mut self) {
        let target_width = self.resize.width.max(1);
        let target_height = self.resize.height.max(1);
        if target_width == self.current_image.width && target_height == self.current_image.height {
            self.preview_image = None;
            return;
        }

        // Generate a small thumbnail for sidebar preview instead of full-size image.
        // This dramatically improves performance for large images.
        // The thumbnail preserves the target aspect ratio.
        let (thumb_width, thumb_height) =
            calculate_preview_thumbnail_size(target_width, target_height);

        let preview_dynamic =
            image_transform::resize(&self.working_image, thumb_width, thumb_height);
        if let Ok(image_data) = image_transform::dynamic_to_image_data(&preview_dynamic) {
            self.preview_image = Some(image_data);
        } else {
            self.preview_image = None;
        }
    }
}

/// Maximum size for the resize preview thumbnail.
/// Kept small for performance during slider interaction.
const PREVIEW_THUMBNAIL_MAX_SIZE: u32 = 300;

/// Calculate thumbnail dimensions for the resize preview.
/// Scales down to fit within max size while preserving aspect ratio.
fn calculate_preview_thumbnail_size(target_width: u32, target_height: u32) -> (u32, u32) {
    let max_dim = target_width.max(target_height);
    if max_dim <= PREVIEW_THUMBNAIL_MAX_SIZE {
        // Already small enough
        (target_width, target_height)
    } else {
        // Scale down to fit within max size
        let scale = PREVIEW_THUMBNAIL_MAX_SIZE as f32 / max_dim as f32;
        let thumb_width = (target_width as f32 * scale).round().max(1.0) as u32;
        let thumb_height = (target_height as f32 * scale).round().max(1.0) as u32;
        (thumb_width, thumb_height)
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
