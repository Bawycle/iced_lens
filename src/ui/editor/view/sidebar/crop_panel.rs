// SPDX-License-Identifier: MPL-2.0
//! Crop tool panel for the editor sidebar.

use crate::ui::editor::state::CropRatio;
use crate::ui::theme;
use iced::widget::{button, container, text, Column, Row};
use iced::{Element, Length};

use super::super::{Message, State, ViewContext};

pub fn panel<'a>(state: &'a State, ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let title = text(ctx.i18n.tr("editor-crop-section-title")).size(14);
    let ratio_label = text(ctx.i18n.tr("editor-crop-ratio-label")).size(13);

    let ratios_row1 = Row::new()
        .spacing(4)
        .push(ratio_button(
            state,
            ctx.i18n.tr("editor-crop-ratio-free"),
            CropRatio::Free,
        ))
        .push(ratio_button(
            state,
            ctx.i18n.tr("editor-crop-ratio-square"),
            CropRatio::Square,
        ));

    let ratios_row2 = Row::new()
        .spacing(4)
        .push(ratio_button(
            state,
            ctx.i18n.tr("editor-crop-ratio-landscape"),
            CropRatio::Landscape,
        ))
        .push(ratio_button(
            state,
            ctx.i18n.tr("editor-crop-ratio-portrait"),
            CropRatio::Portrait,
        ));

    let ratios_row3 = Row::new()
        .spacing(4)
        .push(ratio_button(
            state,
            ctx.i18n.tr("editor-crop-ratio-photo"),
            CropRatio::Photo,
        ))
        .push(ratio_button(
            state,
            ctx.i18n.tr("editor-crop-ratio-photo-portrait"),
            CropRatio::PhotoPortrait,
        ));

    let crop_info = text(format!(
        "{}×{} px",
        state.crop_state.width, state.crop_state.height
    ))
    .size(12);

    let apply_btn = {
        let btn = button(text(ctx.i18n.tr("editor-crop-apply")).size(14))
            .padding(8)
            .width(Length::Fill)
            .style(iced::widget::button::primary);
        if state.crop_state.overlay.visible {
            btn.on_press(Message::ApplyCrop)
        } else {
            btn
        }
    };

    container(
        Column::new()
            .spacing(8)
            .push(title)
            .push(ratio_label)
            .push(ratios_row1)
            .push(ratios_row2)
            .push(ratios_row3)
            .push(crop_info)
            .push(apply_btn),
    )
    .padding(12)
    .width(Length::Fill)
    .style(theme::settings_panel_style)
    .into()
}

fn ratio_button<'a>(state: &'a State, label: String, ratio: CropRatio) -> Element<'a, Message> {
    let selected = state.crop_state.ratio == ratio;
    button(text(label).size(11))
        .on_press(Message::SetCropRatio(ratio))
        .padding([4, 6])
        .width(Length::Fill)
        .style(if selected {
            iced::widget::button::primary
        } else {
            iced::widget::button::secondary
        })
        .into()
}
