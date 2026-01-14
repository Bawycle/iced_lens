// SPDX-License-Identifier: MPL-2.0
//! Diagnostics screen module for viewing and exporting diagnostic data.
//!
//! This module provides a UI for accessing diagnostic controls, viewing
//! collection status, and exporting diagnostic reports.

use crate::i18n::fluent::I18n;
use crate::ui::design_tokens::{spacing, typography};
use iced::{
    alignment::Horizontal,
    widget::{button, scrollable, text, Column, Text},
    Element, Length,
};

/// Contextual data needed to render the diagnostics screen.
pub struct ViewContext<'a> {
    pub i18n: &'a I18n,
}

/// Messages emitted by the diagnostics screen.
#[derive(Debug, Clone)]
pub enum Message {
    BackToViewer,
}

/// Events propagated to the parent application.
#[derive(Debug, Clone)]
pub enum Event {
    None,
    BackToViewer,
}

/// Process a diagnostics screen message and return the corresponding event.
#[must_use]
pub fn update(message: &Message) -> Event {
    match message {
        Message::BackToViewer => Event::BackToViewer,
    }
}

/// Render the diagnostics screen.
#[must_use]
#[allow(clippy::needless_pass_by_value)] // ViewContext is small and consumed
pub fn view(ctx: ViewContext<'_>) -> Element<'_, Message> {
    let back_button = button(
        text(format!("← {}", ctx.i18n.tr("diagnostics-back-button"))).size(typography::BODY),
    )
    .on_press(Message::BackToViewer);

    let title = Text::new(ctx.i18n.tr("diagnostics-title")).size(typography::TITLE_LG);

    let placeholder = Text::new("Diagnostics controls will appear here.").size(typography::BODY);

    let content = Column::new()
        .width(Length::Fill)
        .spacing(spacing::LG)
        .align_x(Horizontal::Left)
        .padding(spacing::MD)
        .push(back_button)
        .push(title)
        .push(placeholder);

    scrollable(content).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::fluent::I18n;

    #[test]
    fn diagnostics_view_renders() {
        let i18n = I18n::default();
        let ctx = ViewContext { i18n: &i18n };
        let _element = view(ctx);
    }

    #[test]
    fn back_to_viewer_emits_event() {
        let event = update(&Message::BackToViewer);
        assert!(matches!(event, Event::BackToViewer));
    }
}
