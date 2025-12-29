// SPDX-License-Identifier: MPL-2.0
//! Crop tool panel for the editor sidebar.

use crate::ui::design_tokens::{spacing, typography};
use crate::ui::image_editor::state::{CropRatio, CropState};
use crate::ui::styles;
use crate::ui::styles::button as button_styles;
use iced::widget::{button, container, text, Column, Row};
use iced::{Element, Length};

use super::super::ViewContext;
use crate::ui::image_editor::{Message, SidebarMessage};

pub fn panel<'a>(crop: &'a CropState, ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let title = text(ctx.i18n.tr("image-editor-crop-section-title")).size(typography::BODY);
    let ratio_label = text(ctx.i18n.tr("image-editor-crop-ratio-label")).size(typography::BODY_SM);

    let ratios_row1 = Row::new()
        .spacing(spacing::XXS)
        .push(ratio_button(
            crop,
            ctx.i18n.tr("image-editor-crop-ratio-free"),
            CropRatio::Free,
        ))
        .push(ratio_button(
            crop,
            ctx.i18n.tr("image-editor-crop-ratio-square"),
            CropRatio::Square,
        ));

    let ratios_row2 = Row::new()
        .spacing(spacing::XXS)
        .push(ratio_button(
            crop,
            ctx.i18n.tr("image-editor-crop-ratio-landscape"),
            CropRatio::Landscape,
        ))
        .push(ratio_button(
            crop,
            ctx.i18n.tr("image-editor-crop-ratio-portrait"),
            CropRatio::Portrait,
        ));

    let ratios_row3 = Row::new()
        .spacing(spacing::XXS)
        .push(ratio_button(
            crop,
            ctx.i18n.tr("image-editor-crop-ratio-photo"),
            CropRatio::Photo,
        ))
        .push(ratio_button(
            crop,
            ctx.i18n.tr("image-editor-crop-ratio-photo-portrait"),
            CropRatio::PhotoPortrait,
        ));

    let crop_info = text(format!("{}×{} px", crop.width, crop.height)).size(typography::CAPTION);

    let apply_btn = {
        let btn = button(text(ctx.i18n.tr("image-editor-crop-apply")).size(typography::BODY))
            .padding(spacing::XS)
            .width(Length::Fill);
        if crop.overlay.visible {
            btn.on_press(SidebarMessage::ApplyCrop.into())
        } else {
            btn.style(button_styles::disabled())
        }
    };

    container(
        Column::new()
            .spacing(spacing::XS)
            .push(title)
            .push(ratio_label)
            .push(ratios_row1)
            .push(ratios_row2)
            .push(ratios_row3)
            .push(crop_info)
            .push(apply_btn),
    )
    .padding(spacing::SM)
    .width(Length::Fill)
    .style(styles::editor::settings_panel)
    .into()
}

fn ratio_button(crop: &CropState, label: String, ratio: CropRatio) -> Element<'_, Message> {
    let is_selected = crop.ratio == ratio;
    button(text(label).size(typography::CAPTION))
        .on_press(SidebarMessage::SetCropRatio(ratio).into())
        .padding([spacing::XXS, spacing::XS])
        .width(Length::Fill)
        .style(if is_selected {
            button_styles::selected
        } else {
            button_styles::unselected
        })
        .into()
}
