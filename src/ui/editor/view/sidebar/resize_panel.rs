// SPDX-License-Identifier: MPL-2.0
//! Resize tool panel for the editor sidebar.

use crate::ui::theme;
use iced::widget::{button, checkbox, container, slider, text, text_input, Column, Row};
use iced::{Element, Length};

use super::super::{Message, ViewContext};
use crate::ui::editor::state::ResizeState;

pub fn panel<'a>(resize: &'a ResizeState, ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let scale_section = Column::new()
        .spacing(6)
        .push(text(ctx.i18n.tr("editor-resize-section-title")).size(14))
        .push(text(ctx.i18n.tr("editor-resize-scale-label")).size(13))
        .push(slider(10.0..=200.0, resize.scale_percent, Message::ScaleChanged).step(1.0))
        .push(text(format!("{:.0}%", resize.scale_percent)).size(13));

    let mut presets = Row::new().spacing(8);
    for preset in [50.0, 75.0, 150.0, 200.0] {
        let label = format!("{preset:.0}%");
        presets = presets.push(
            button(text(label))
                .on_press(Message::ApplyResizePreset(preset))
                .padding([6, 8])
                .style(iced::widget::button::secondary),
        );
    }

    let presets_section = Column::new()
        .spacing(6)
        .push(text(ctx.i18n.tr("editor-resize-presets-label")).size(13))
        .push(presets);

    let width_placeholder = ctx.i18n.tr("editor-resize-width-label");
    let width_label = text(width_placeholder.clone()).size(13);
    let width_input = text_input(width_placeholder.as_str(), &resize.width_input)
        .on_input(Message::WidthInputChanged)
        .padding(6)
        .size(14)
        .width(Length::Fill);

    let height_placeholder = ctx.i18n.tr("editor-resize-height-label");
    let height_label = text(height_placeholder.clone()).size(13);
    let height_input = text_input(height_placeholder.as_str(), &resize.height_input)
        .on_input(Message::HeightInputChanged)
        .padding(6)
        .size(14)
        .width(Length::Fill);

    let dimensions_row = Row::new()
        .spacing(8)
        .push(
            Column::new()
                .spacing(4)
                .width(Length::Fill)
                .push(width_label)
                .push(width_input),
        )
        .push(
            Column::new()
                .spacing(4)
                .width(Length::Fill)
                .push(height_label)
                .push(height_input),
        );

    let lock_checkbox = checkbox(ctx.i18n.tr("editor-resize-lock-aspect"), resize.lock_aspect)
        .on_toggle(|_| Message::ToggleLockAspect);

    let apply_btn = button(text(ctx.i18n.tr("editor-resize-apply")).size(16))
        .padding(10)
        .width(Length::Fill)
        .style(iced::widget::button::primary)
        .on_press(Message::ApplyResize);

    container(
        Column::new()
            .spacing(12)
            .push(scale_section)
            .push(presets_section)
            .push(text(ctx.i18n.tr("editor-resize-dimensions-label")).size(13))
            .push(dimensions_row)
            .push(lock_checkbox)
            .push(apply_btn),
    )
    .padding(12)
    .width(Length::Fill)
    .style(theme::settings_panel_style)
    .into()
}
