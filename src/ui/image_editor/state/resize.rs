// SPDX-License-Identifier: MPL-2.0
//! Resize tool state and helpers.

use crate::media::{image_transform, ImageData};
use crate::ui::image_editor::{State, Transformation};

/// Minimum resize scale percentage.
const MIN_RESIZE_SCALE: f32 = 10.0;
/// Maximum resize scale percentage.
const MAX_RESIZE_SCALE: f32 = 200.0;
/// Default resize scale percentage.
const DEFAULT_RESIZE_SCALE: f32 = 100.0;

/// Resize scale percentage, guaranteed to be within valid range (10%–200%).
///
/// This type ensures that resize scale values are always valid, eliminating
/// the need for manual clamping at usage sites.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResizeScale(f32);

impl ResizeScale {
    /// Creates a new resize scale, clamping the value to the valid range.
    pub fn new(percent: f32) -> Self {
        Self(percent.clamp(MIN_RESIZE_SCALE, MAX_RESIZE_SCALE))
    }

    /// Returns the raw percentage value.
    pub fn value(self) -> f32 {
        self.0
    }

    /// Returns the scale as a multiplier (e.g., 100% → 1.0).
    pub fn as_factor(self) -> f32 {
        self.0 / 100.0
    }

    /// Returns whether the scale is at the minimum value.
    pub fn is_min(self) -> bool {
        self.0 <= MIN_RESIZE_SCALE
    }

    /// Returns whether the scale is at the maximum value.
    pub fn is_max(self) -> bool {
        self.0 >= MAX_RESIZE_SCALE
    }

    /// Returns whether the scale represents 100% (no resize).
    pub fn is_original(self) -> bool {
        (self.0 - DEFAULT_RESIZE_SCALE).abs() < f32::EPSILON
    }
}

impl Default for ResizeScale {
    fn default() -> Self {
        Self(DEFAULT_RESIZE_SCALE)
    }
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
        }
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
        self.resize_state.overlay.visible = false;
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

    fn set_resize_percent(&mut self, percent: f32) {
        let scale = ResizeScale::new(percent);
        self.resize_state.scale = scale;
        let width = (self.base_width() * scale.as_factor()).round().max(1.0) as u32;
        let height = (self.base_height() * scale.as_factor()).round().max(1.0) as u32;

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

    fn handle_width_input_change(&mut self, value: String) {
        self.resize_state.width_input = value.clone();
        if let Some(width) = parse_dimension_input(&value) {
            if self.resize_state.lock_aspect {
                self.set_width_preserving_aspect(width);
                self.update_scale_percent_from_width();
            } else {
                // When aspect is unlocked, allow any value without clamping
                let width = width.max(1);
                self.resize_state.width = width;
                self.resize_state.width_input = width.to_string();
                self.update_resize_preview();
            }
        }
    }

    fn handle_height_input_change(&mut self, value: String) {
        self.resize_state.height_input = value.clone();
        if let Some(height) = parse_dimension_input(&value) {
            if self.resize_state.lock_aspect {
                self.set_height_preserving_aspect(height);
                self.update_scale_percent_from_width();
            } else {
                // When aspect is unlocked, allow any value without clamping
                let height = height.max(1);
                self.resize_state.height = height;
                self.resize_state.height_input = height.to_string();
                self.update_resize_preview();
            }
        }
    }

    fn toggle_resize_lock(&mut self) {
        self.resize_state.lock_aspect = !self.resize_state.lock_aspect;
        if self.resize_state.lock_aspect {
            let width = self.resize_state.width;
            self.set_width_preserving_aspect(width);
        }
        self.update_resize_preview();
    }

    fn set_width_preserving_aspect(&mut self, width: u32) {
        let width = width.max(1);
        let aspect = self.resize_state.original_aspect.max(f32::EPSILON);
        let height = (width as f32 / aspect).round().max(1.0) as u32;
        self.resize_state.width = width;
        self.resize_state.height = height;
        self.resize_state.width_input = width.to_string();
        self.resize_state.height_input = height.to_string();
    }

    fn set_height_preserving_aspect(&mut self, height: u32) {
        let height = height.max(1);
        let aspect = self.resize_state.original_aspect.max(f32::EPSILON);
        let width = (height as f32 * aspect).round().max(1.0) as u32;
        self.resize_state.height = height;
        self.resize_state.width = width.max(1);
        self.resize_state.width_input = self.resize_state.width.to_string();
        self.resize_state.height_input = height.to_string();
    }

    fn update_scale_percent_from_width(&mut self) {
        let base_width = self.base_width();
        if base_width <= 0.0 {
            return;
        }
        let percent = (self.resize_state.width as f32 / base_width) * 100.0;
        let scale = ResizeScale::new(percent);
        // If clamping changed the value, recalculate dimensions
        if (scale.value() - percent).abs() > f32::EPSILON {
            self.set_resize_percent(scale.value());
        } else {
            self.resize_state.scale = scale;
            self.update_resize_preview();
        }
    }

    fn apply_resize_dimensions(&mut self) {
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
            move |image| image_transform::resize(image, target_width, target_height),
        );

        self.resize_state
            .overlay
            .set_original_dimensions(self.current_image.width, self.current_image.height);
    }

    fn update_resize_preview(&mut self) {
        let target_width = self.resize_state.width.max(1);
        let target_height = self.resize_state.height.max(1);
        if target_width == self.current_image.width && target_height == self.current_image.height {
            self.preview_image = None;
            return;
        }

        let preview_dynamic =
            image_transform::resize(&self.working_image, target_width, target_height);
        if let Ok(image_data) = image_transform::dynamic_to_image_data(&preview_dynamic) {
            self.preview_image = Some(image_data);
        } else {
            self.preview_image = None;
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
