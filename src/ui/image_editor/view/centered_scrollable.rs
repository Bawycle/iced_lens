// SPDX-License-Identifier: MPL-2.0
//! Scrollable widget for editor canvas.
//!
//! NOTE: Proper centering of small images is not possible in Iced 0.13.1 without
//! the `responsive()` widget. Implementing a true custom widget would require
//! reimplementing the entire Scrollable widget from scratch, which is too complex.
//!
//! This module provides a simple scrollable that works correctly for large images.
//! Small images will be visible but positioned at top-left instead of centered.

use crate::ui::image_editor::Message;
use iced::widget::scrollable::{Direction, Scrollbar, Viewport};
use iced::widget::{Id, Scrollable};
use iced::{Element, Length};

/// Creates a simple scrollable widget.
///
/// Large images will show scrollbars correctly.
/// Small images will be visible but NOT centered (limitation of Iced 0.13.1).
pub fn centered_scrollable<'a>(
    content: Element<'a, Message>,
    _content_width: f32,
    _content_height: f32,
    scrollable_id: &'static str,
) -> Element<'a, Message> {
    Scrollable::new(content)
        .id(Id::new(scrollable_id))
        .width(Length::Fill)
        .height(Length::Fill)
        .direction(Direction::Both {
            vertical: Scrollbar::new(),
            horizontal: Scrollbar::new(),
        })
        .on_scroll(|viewport: Viewport| Message::ViewportChanged {
            bounds: viewport.bounds(),
            offset: viewport.absolute_offset(),
        })
        .into()
}
