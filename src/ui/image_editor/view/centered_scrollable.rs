// SPDX-License-Identifier: MPL-2.0
//! Scrollable widget for editor canvas.
//!
//! Provides a scrollable container with wheel events blocked for zoom handling.
//! Centering is handled via responsive padding in canvas.rs.

use crate::ui::image_editor::Message;
use crate::ui::widgets::wheel_blocking_scrollable::wheel_blocking_scrollable;
use iced::widget::scrollable::{Direction, Scrollbar, Viewport};
use iced::widget::{Id, Scrollable};
use iced::{Element, Length};

/// Creates a scrollable widget with wheel events blocked for zoom.
///
/// Wheel events are blocked to allow zoom via subscription.
pub fn centered_scrollable<'a>(
    content: Element<'a, Message>,
    _content_width: f32,
    _content_height: f32,
    scrollable_id: &'static str,
) -> Element<'a, Message> {
    let scrollable = Scrollable::new(content)
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
        });

    // Block wheel events to allow zoom via subscription (like viewer does)
    wheel_blocking_scrollable(scrollable).into()
}
