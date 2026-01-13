// SPDX-License-Identifier: MPL-2.0
//! Derived viewer state helpers used to keep `App` lean.

use crate::media::MediaData;
use crate::ui::state::rotation::RotationAngle;
use crate::ui::state::viewport::ViewportState;
use crate::ui::state::zoom::{clamp_zoom, DEFAULT_ZOOM_PERCENT};
use iced::{Padding, Point, Rectangle, Size};

/// Extra spacing reserved for the scrollbars area when the image overflows.
pub const SCROLLBAR_GUTTER: f32 = 16.0;

/// Provides higher-level geometry information for the viewer pane.
pub struct ViewerState<'a> {
    media: Option<&'a MediaData>,
    viewport: &'a ViewportState,
    zoom_percent: f32,
    cursor_position: Option<Point>,
}

impl<'a> ViewerState<'a> {
    /// Creates a new derived state helper.
    #[must_use]
    pub fn new(
        media: Option<&'a MediaData>,
        viewport: &'a ViewportState,
        zoom_percent: f32,
        cursor_position: Option<Point>,
    ) -> Self {
        Self {
            media,
            viewport,
            zoom_percent,
            cursor_position,
        }
    }

    /// Computes the zoom percentage required to fit the current media inside the viewport.
    #[must_use]
    // Allow cast_precision_loss: image dimensions are typically < 16M pixels;
    // f32 is exact up to 2^24 (~16.7M), sufficient for any reasonable image.
    #[allow(clippy::cast_precision_loss)]
    pub fn compute_fit_zoom_percent(&self) -> Option<f32> {
        let media = self.media?;
        let viewport = self.viewport.bounds?;

        if media.width() == 0 || media.height() == 0 {
            return Some(DEFAULT_ZOOM_PERCENT);
        }

        if viewport.width <= 0.0 || viewport.height <= 0.0 {
            return None;
        }

        let media_width = media.width() as f32;
        let media_height = media.height() as f32;

        let scale_x = viewport.width / media_width;
        let scale_y = viewport.height / media_height;

        let scale = scale_x.min(scale_y);

        if !scale.is_finite() || scale <= 0.0 {
            return Some(DEFAULT_ZOOM_PERCENT);
        }

        Some(clamp_zoom(scale * 100.0))
    }

    /// Returns the scaled media dimensions for the current zoom level.
    #[allow(clippy::cast_precision_loss)] // u32 to f32 for dimensions: f32 is exact up to 16M
    #[must_use]
    pub fn scaled_media_size(&self) -> Option<Size> {
        let media = self.media?;
        let scale = (self.zoom_percent / 100.0).max(0.01);
        let width = (media.width() as f32 * scale).max(1.0);
        let height = (media.height() as f32 * scale).max(1.0);
        Some(Size::new(width, height))
    }

    /// Returns the scaled media dimensions accounting for rotation.
    ///
    /// When rotated 90° or 270°, width and height are swapped.
    #[allow(clippy::cast_precision_loss)] // u32 to f32 for dimensions: f32 is exact up to 16M
    #[must_use]
    pub fn scaled_media_size_rotated(&self, rotation: RotationAngle) -> Option<Size> {
        let media = self.media?;
        let scale = (self.zoom_percent / 100.0).max(0.01);

        // Get effective dimensions based on rotation
        let (effective_width, effective_height) = if rotation.swaps_dimensions() {
            (media.height(), media.width())
        } else {
            (media.width(), media.height())
        };

        let width = (effective_width as f32 * scale).max(1.0);
        let height = (effective_height as f32 * scale).max(1.0);
        Some(Size::new(width, height))
    }

    fn compute_padding(viewport: Rectangle, size: Size) -> Padding {
        let horizontal = ((viewport.width - size.width) / 2.0).max(0.0);
        let vertical = ((viewport.height - size.height) / 2.0).max(0.0);

        Padding {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Returns the padding needed to center the media inside the viewport.
    #[must_use]
    pub fn media_padding(&self) -> Padding {
        match (self.viewport.bounds, self.scaled_media_size()) {
            (Some(viewport), Some(size)) => Self::compute_padding(viewport, size),
            _ => Padding::default(),
        }
    }

    /// Returns the scroll position as a percentage when scrolling is possible.
    #[must_use]
    pub fn scroll_position_percentage(&self) -> Option<(f32, f32)> {
        let size = self.scaled_media_size()?;
        self.viewport
            .scroll_position_percentage(size.width, size.height)
    }

    /// Returns the media bounds relative to the window, factoring in scroll and padding.
    #[must_use]
    pub fn media_bounds_in_window(&self) -> Option<Rectangle> {
        let viewport = self.viewport.bounds?;
        let size = self.scaled_media_size()?;
        let padding = Self::compute_padding(viewport, size);

        let content_origin_x = viewport.x - self.viewport.offset.x;
        let content_origin_y = viewport.y - self.viewport.offset.y;

        let left = content_origin_x + padding.left;
        let top = content_origin_y + padding.top;

        Some(Rectangle::new(Point::new(left, top), size))
    }

    /// Indicates whether the cursor is currently positioned over the media.
    #[must_use]
    pub fn is_cursor_over_media(&self) -> bool {
        let Some(cursor) = self.cursor_position else {
            return false;
        };

        let Some(viewport) = self.viewport.bounds else {
            return false;
        };

        let Some(size) = self.scaled_media_size() else {
            return false;
        };

        let Some(media_bounds) = self.media_bounds_in_window() else {
            return false;
        };

        let viewport_rect = Rectangle::new(
            Point::new(viewport.x, viewport.y),
            Size::new(viewport.width, viewport.height),
        );

        if !viewport_rect.contains(cursor) {
            return false;
        }

        let Some(mut hitbox) = intersect_rectangles(media_bounds, viewport_rect) else {
            return false;
        };

        if size.height > viewport.height {
            if hitbox.width <= SCROLLBAR_GUTTER {
                return false;
            }

            hitbox.width -= SCROLLBAR_GUTTER;
        }

        if size.width > viewport.width {
            if hitbox.height <= SCROLLBAR_GUTTER {
                return false;
            }

            hitbox.height -= SCROLLBAR_GUTTER;
        }

        hitbox.contains(cursor)
    }
}

fn intersect_rectangles(a: Rectangle, b: Rectangle) -> Option<Rectangle> {
    let left = a.x.max(b.x);
    let top = a.y.max(b.y);
    let right = (a.x + a.width).min(b.x + b.width);
    let bottom = (a.y + a.height).min(b.y + b.height);

    if right <= left || bottom <= top {
        None
    } else {
        Some(Rectangle::new(
            Point::new(left, top),
            Size::new(right - left, bottom - top),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::{ImageData, MediaData};
    use iced::widget::scrollable::AbsoluteOffset;

    fn sample_media() -> MediaData {
        let pixels = vec![255_u8; 4];
        let image_data = ImageData::from_rgba(1, 1, pixels);
        MediaData::Image(image_data)
    }

    fn viewport_with_bounds() -> ViewportState {
        ViewportState {
            bounds: Some(Rectangle::new(
                Point::new(0.0, 0.0),
                Size::new(400.0, 300.0),
            )),
            ..ViewportState::default()
        }
    }

    #[test]
    fn scaled_size_respects_zoom_percent() {
        use crate::test_utils::assert_abs_diff_eq;
        let media = sample_media();
        let viewport = viewport_with_bounds();
        let state = ViewerState::new(Some(&media), &viewport, 200.0, None);

        let size = state.scaled_media_size().expect("size");
        assert_abs_diff_eq!(size.width, 2.0);
        assert_abs_diff_eq!(size.height, 2.0);
    }

    #[test]
    fn compute_fit_zoom_percent_without_viewport_returns_none() {
        let media = sample_media();
        let viewport = ViewportState::default();
        let state = ViewerState::new(Some(&media), &viewport, 100.0, None);

        assert!(state.compute_fit_zoom_percent().is_none());
    }

    #[test]
    fn cursor_outside_viewport_is_not_over_media() {
        let media = sample_media();
        let mut viewport = viewport_with_bounds();
        viewport.offset = AbsoluteOffset { x: 0.0, y: 0.0 };
        let state = ViewerState::new(
            Some(&media),
            &viewport,
            100.0,
            Some(Point::new(500.0, 500.0)),
        );

        assert!(!state.is_cursor_over_media());
    }
}
