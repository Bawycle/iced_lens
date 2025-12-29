// SPDX-License-Identifier: MPL-2.0
//! Scrollable canvas widget for editor.

use crate::ui::image_editor::Message;
use iced::Element;

use super::centered_scrollable;

/// Scrollable ID for the editor canvas (used for `snap_to` operations).
pub const EDITOR_CANVAS_SCROLLABLE_ID: &str = "image-editor-canvas-scrollable";

/// Creates a scrollable canvas for the editor.
///
/// Wheel events are blocked to allow zoom via mouse wheel.
pub fn scrollable_canvas(
    image_content: Element<'_, Message>,
    image_width: f32,
    image_height: f32,
) -> Element<'_, Message> {
    centered_scrollable::centered_scrollable(
        image_content,
        image_width,
        image_height,
        EDITOR_CANVAS_SCROLLABLE_ID,
    )
}
