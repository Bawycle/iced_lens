// SPDX-License-Identifier: MPL-2.0
//! Top toolbar for the editor surface.

use crate::ui::styles;
use iced::widget::{button, container, Row, Text};
use iced::{Alignment, Element, Length};

use super::super::{Message, State, ToolbarMessage, ViewContext};

pub struct ToolbarModel {
    pub has_unsaved_changes: bool,
}

impl ToolbarModel {
    pub fn from_state(state: &State) -> Self {
        Self {
            has_unsaved_changes: state.has_unsaved_changes(),
        }
    }
}

pub fn view<'a>(model: &ToolbarModel, ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let has_changes = model.has_unsaved_changes;
    let back_label = format!("← {}", ctx.i18n.tr("editor-back-to-viewer"));

    let back_btn = button(Text::new(back_label).size(14)).padding([8, 12]);
    let back_btn = if has_changes {
        back_btn
    } else {
        back_btn.on_press(Message::Toolbar(ToolbarMessage::BackToViewer))
    };

    container(
        Row::new()
            .push(back_btn)
            .align_y(Alignment::Center)
            .padding(8),
    )
    .width(Length::Fill)
    .style(styles::editor::toolbar)
    .into()
}
