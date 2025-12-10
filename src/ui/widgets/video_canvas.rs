// SPDX-License-Identifier: MPL-2.0
//! Widget for rendering video frames.
//!
//! This module provides a widget that efficiently renders
//! decoded video frames using Iced's Image widget.

use iced::widget::{container, image, Container};
use iced::{Color, ContentFit, Element, Length};
use std::sync::Arc;

/// Video frame widget.
///
/// Renders RGBA frame data using Iced's Image widget.
/// Creates a new image::Handle when the frame changes.
pub struct VideoCanvas<Message> {
    /// Current frame as image handle.
    frame_handle: Option<image::Handle>,

    /// Frame dimensions.
    width: u32,
    height: u32,

    /// Zoom scale factor (1.0 = 100%).
    scale: f32,

    _phantom: std::marker::PhantomData<Message>,
}

impl<Message> VideoCanvas<Message> {
    /// Creates a new video canvas.
    pub fn new() -> Self {
        Self {
            frame_handle: None,
            width: 0,
            height: 0,
            scale: 1.0,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Updates the displayed frame.
    ///
    /// Creates a new image::Handle from the RGBA data.
    pub fn set_frame(&mut self, rgba_data: Arc<Vec<u8>>, width: u32, height: u32) {
        // Create image handle from RGBA data
        // Try to take ownership of the Arc's data if we're the only reference,
        // otherwise clone (which is unavoidable when there are other references)
        let data = Arc::try_unwrap(rgba_data).unwrap_or_else(|arc| (*arc).clone());
        let handle = image::Handle::from_rgba(width, height, data);

        self.frame_handle = Some(handle);
        self.width = width;
        self.height = height;
    }

    /// Sets the zoom scale factor.
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }

    /// Clears the current frame and releases memory.
    pub fn clear(&mut self) {
        self.frame_handle = None;
        self.width = 0;
        self.height = 0;
    }

    /// Returns true if the canvas has a frame to display.
    pub fn has_frame(&self) -> bool {
        self.frame_handle.is_some()
    }

    /// Returns the current scaled width.
    pub fn scaled_width(&self) -> f32 {
        self.width as f32 * self.scale
    }

    /// Returns the current scaled height.
    pub fn scaled_height(&self) -> f32 {
        self.height as f32 * self.scale
    }

    /// Renders the video frame to an Iced element.
    pub fn view(&self) -> Element<'_, Message> {
        if let Some(ref handle) = self.frame_handle {
            // Render the frame using Image widget
            let img = image::Image::new(handle.clone())
                .content_fit(ContentFit::Contain)
                .width(Length::Fixed(self.scaled_width().max(1.0)))
                .height(Length::Fixed(self.scaled_height().max(1.0)));

            img.into()
        } else {
            // No frame loaded, show placeholder
            let placeholder: Container<'_, Message> = container(iced::widget::text(""))
                .width(Length::Fixed(100.0))
                .height(Length::Fixed(100.0))
                .style(|_theme: &iced::Theme| container::Style {
                    background: Some(Color::from_rgb(0.1, 0.1, 0.1).into()),
                    ..Default::default()
                });

            placeholder.into()
        }
    }
}

impl<Message> Default for VideoCanvas<Message> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_canvas_starts_empty() {
        let canvas: VideoCanvas<()> = VideoCanvas::new();
        assert!(canvas.frame_handle.is_none());
        assert_eq!(canvas.width, 0);
        assert_eq!(canvas.height, 0);
        assert_eq!(canvas.scale, 1.0);
    }

    #[test]
    fn set_frame_updates_dimensions() {
        let mut canvas: VideoCanvas<()> = VideoCanvas::new();
        let rgba_data = Arc::new(vec![0u8; 1920 * 1080 * 4]);

        canvas.set_frame(rgba_data, 1920, 1080);

        assert!(canvas.frame_handle.is_some());
        assert_eq!(canvas.width, 1920);
        assert_eq!(canvas.height, 1080);
    }

    #[test]
    fn set_scale_updates_scale() {
        let mut canvas: VideoCanvas<()> = VideoCanvas::new();
        let rgba_data = Arc::new(vec![0u8; 1920 * 1080 * 4]);
        canvas.set_frame(rgba_data, 1920, 1080);

        canvas.set_scale(0.5);
        assert_eq!(canvas.scale, 0.5);
    }

    #[test]
    fn scaled_dimensions_calculated_correctly() {
        let mut canvas: VideoCanvas<()> = VideoCanvas::new();
        let rgba_data = Arc::new(vec![0u8; 1920 * 1080 * 4]);
        canvas.set_frame(rgba_data, 1920, 1080);
        canvas.set_scale(0.5);

        assert_eq!(canvas.scaled_width(), 960.0);
        assert_eq!(canvas.scaled_height(), 540.0);
    }

    #[test]
    fn default_creates_empty_canvas() {
        let canvas: VideoCanvas<()> = VideoCanvas::default();
        assert!(canvas.frame_handle.is_none());
    }
}
