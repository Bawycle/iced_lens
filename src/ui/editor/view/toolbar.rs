// SPDX-License-Identifier: MPL-2.0
//! Top toolbar for the editor surface.

use crate::ui::theme;
use iced::widget::{button, container, Row, Text};
use iced::{Alignment, Background, Border, Element, Length};

use super::super::{Message, State, ViewContext};

pub fn view<'a>(state: &'a State, ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let has_changes = state.has_unsaved_changes();
    let back_label = format!("← {}", ctx.i18n.tr("editor-back-to-viewer"));

    let back_btn = button(Text::new(back_label).size(14)).padding([8, 12]);
    let back_btn = if has_changes {
        back_btn
    } else {
        back_btn.on_press(Message::BackToViewer)
    };

    container(
        Row::new()
            .push(back_btn)
            .align_y(Alignment::Center)
            .padding(8),
    )
    .width(Length::Fill)
    .style(|_theme: &iced::Theme| iced::widget::container::Style {
        background: Some(Background::Color(theme::viewer_toolbar_background())),
        border: Border {
            width: 0.0,
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}
