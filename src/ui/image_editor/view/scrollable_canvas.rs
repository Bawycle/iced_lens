// SPDX-License-Identifier: MPL-2.0
//! Scrollable canvas widget for editor.

use crate::ui::image_editor::Message;
use iced::Element;

use super::centered_scrollable;

const EDITOR_CANVAS_SCROLLABLE_ID: &str = "image-editor-canvas-scrollable";

/// Creates a scrollable canvas for the editor.
///
/// Wheel events are blocked to allow zoom via mouse wheel.
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
