// SPDX-License-Identifier: MPL-2.0
//! Derived viewer state helpers used to keep [`App`] lean.

use crate::image_handler::ImageData;
use crate::ui::state::viewport::ViewportState;
use crate::ui::state::zoom::{clamp_zoom, DEFAULT_ZOOM_PERCENT};
use iced::{Padding, Point, Rectangle, Size};

/// Extra spacing reserved for the scrollbars area when the image overflows.
pub const SCROLLBAR_GUTTER: f32 = 16.0;

/// Provides higher-level geometry information for the viewer pane.
pub struct ViewerState<'a> {
    image: Option<&'a ImageData>,
    viewport: &'a ViewportState,
    zoom_percent: f32,
    cursor_position: Option<Point>,
}

impl<'a> ViewerState<'a> {
    /// Creates a new derived state helper.
    pub fn new(
        image: Option<&'a ImageData>,
        viewport: &'a ViewportState,
        zoom_percent: f32,
        cursor_position: Option<Point>,
    ) -> Self {
        Self {
            image,
            viewport,
            zoom_percent,
            cursor_position,
        }
    }

    /// Computes the zoom percentage required to fit the current image inside the viewport.
    pub fn compute_fit_zoom_percent(&self) -> Option<f32> {
        let image = self.image?;
        let viewport = self.viewport.bounds?;

        if image.width == 0 || image.height == 0 {
            return Some(DEFAULT_ZOOM_PERCENT);
        }

        if viewport.width <= 0.0 || viewport.height <= 0.0 {
            return None;
        }

        let image_width = image.width as f32;
        let image_height = image.height as f32;

        let scale_x = viewport.width / image_width;
        let scale_y = viewport.height / image_height;

        let scale = scale_x.min(scale_y);

        if !scale.is_finite() || scale <= 0.0 {
            return Some(DEFAULT_ZOOM_PERCENT);
        }

        Some(clamp_zoom(scale * 100.0))
    }

    /// Returns the scaled image dimensions for the current zoom level.
    pub fn scaled_image_size(&self) -> Option<Size> {
        let image = self.image?;
        let scale = (self.zoom_percent / 100.0).max(0.01);
        let width = (image.width as f32 * scale).max(1.0);
        let height = (image.height as f32 * scale).max(1.0);
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

    /// Returns the padding needed to center the image inside the viewport.
    pub fn image_padding(&self) -> Padding {
        match (self.viewport.bounds, self.scaled_image_size()) {
            (Some(viewport), Some(size)) => Self::compute_padding(viewport, size),
            _ => Padding::default(),
        }
    }

    /// Returns the scroll position as a percentage when scrolling is possible.
    pub fn scroll_position_percentage(&self) -> Option<(f32, f32)> {
        let size = self.scaled_image_size()?;
        self.viewport
            .scroll_position_percentage(size.width, size.height)
    }

    /// Returns the image bounds relative to the window, factoring in scroll and padding.
    pub fn image_bounds_in_window(&self) -> Option<Rectangle> {
        let viewport = self.viewport.bounds?;
        let size = self.scaled_image_size()?;
        let padding = Self::compute_padding(viewport, size);

        let content_origin_x = viewport.x - self.viewport.offset.x;
        let content_origin_y = viewport.y - self.viewport.offset.y;

        let left = content_origin_x + padding.left;
        let top = content_origin_y + padding.top;

        Some(Rectangle::new(Point::new(left, top), size))
    }

    /// Indicates whether the cursor is currently positioned over the image.
    pub fn is_cursor_over_image(&self) -> bool {
        let cursor = match self.cursor_position {
            Some(position) => position,
            None => return false,
        };

        let viewport = match self.viewport.bounds {
            Some(bounds) => bounds,
            None => return false,
        };

        let size = match self.scaled_image_size() {
            Some(dimensions) => dimensions,
            None => return false,
        };

        let image_bounds = match self.image_bounds_in_window() {
            Some(bounds) => bounds,
            None => return false,
        };

        let viewport_rect = Rectangle::new(
            Point::new(viewport.x, viewport.y),
            Size::new(viewport.width, viewport.height),
        );

        if !viewport_rect.contains(cursor) {
            return false;
        }

        let mut hitbox = match intersect_rectangles(image_bounds, viewport_rect) {
            Some(intersection) => intersection,
            None => return false,
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
    use iced::widget::image::Handle;
    use iced::widget::scrollable::AbsoluteOffset;

    fn sample_image() -> ImageData {
        let pixels = vec![255_u8; 4];
        ImageData {
            handle: Handle::from_rgba(1, 1, pixels),
            width: 1,
            height: 1,
        }
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
        let image = sample_image();
        let viewport = viewport_with_bounds();
        let state = ViewerState::new(Some(&image), &viewport, 200.0, None);

        let size = state.scaled_image_size().expect("size");
        assert_eq!(size.width, 2.0);
        assert_eq!(size.height, 2.0);
    }

    #[test]
    fn compute_fit_zoom_percent_without_viewport_returns_none() {
        let image = sample_image();
        let viewport = ViewportState::default();
        let state = ViewerState::new(Some(&image), &viewport, 100.0, None);

        assert!(state.compute_fit_zoom_percent().is_none());
    }

    #[test]
    fn cursor_outside_viewport_is_not_over_image() {
        let image = sample_image();
        let mut viewport = viewport_with_bounds();
        viewport.offset = AbsoluteOffset { x: 0.0, y: 0.0 };
        let state = ViewerState::new(
            Some(&image),
            &viewport,
            100.0,
            Some(Point::new(500.0, 500.0)),
        );

        assert!(!state.is_cursor_over_image());
    }
}
