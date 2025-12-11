// SPDX-License-Identifier: MPL-2.0
//! Scrollable canvas widget for editor using custom centered widget.

use crate::ui::image_editor::Message;
use iced::Element;

use super::centered_scrollable;

const EDITOR_CANVAS_SCROLLABLE_ID: &str = "editor-canvas-scrollable";

/// Creates a scrollable canvas for the editor with automatic centering.
///
/// Uses a custom widget that centers small images and adds scrollbars for large images.
pub fn scrollable_canvas<'a>(
    image_content: Element<'a, Message>,
    image_width: f32,
    image_height: f32,
) -> Element<'a, Message> {
    centered_scrollable::centered_scrollable(
        image_content,
        image_width,
        image_height,
        EDITOR_CANVAS_SCROLLABLE_ID,
    )
}
