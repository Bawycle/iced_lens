// SPDX-License-Identifier: MPL-2.0
//! Light adjustment tool panel for brightness and contrast controls.

use crate::ui::design_tokens::{spacing, typography};
use crate::ui::styles;
use crate::ui::styles::button as button_styles;
use iced::widget::{button, container, slider, text, Column, Row};
use iced::{Element, Length};

use super::super::ViewContext;
use crate::ui::image_editor::state::AdjustmentState;
use crate::ui::image_editor::{Message, SidebarMessage};

/// Format adjustment value with sign and padding for consistent width.
fn format_value(value: i32) -> String {
    format!("{:+4}", value)
}

pub fn panel<'a>(adjustment: &'a AdjustmentState, ctx: &ViewContext<'a>) -> Element<'a, Message> {
    // Brightness section - vertical layout: label, slider, value
    let brightness_section = Column::new()
        .spacing(spacing::XXS)
        .push(text(ctx.i18n.tr("image-editor-light-brightness-label")).size(typography::BODY_SM))
        .push(
            slider(-100..=100, adjustment.brightness, |value| {
                Message::Sidebar(SidebarMessage::BrightnessChanged(value))
            })
            .step(1),
        )
        .push(text(format_value(adjustment.brightness)).size(typography::BODY_SM));

    // Contrast section - vertical layout: label, slider, value
    let contrast_section = Column::new()
        .spacing(spacing::XXS)
        .push(text(ctx.i18n.tr("image-editor-light-contrast-label")).size(typography::BODY_SM))
        .push(
            slider(-100..=100, adjustment.contrast, |value| {
                Message::Sidebar(SidebarMessage::ContrastChanged(value))
            })
            .step(1),
        )
        .push(text(format_value(adjustment.contrast)).size(typography::BODY_SM));

    // Action buttons row
    let reset_btn = button(text(ctx.i18n.tr("image-editor-light-reset")).size(typography::BODY))
        .padding(spacing::SM)
        .width(Length::Fill);
    let reset_btn = if adjustment.has_changes() {
        reset_btn.on_press(SidebarMessage::ResetAdjustments.into())
    } else {
        reset_btn.style(button_styles::disabled())
    };

    let apply_btn = button(text(ctx.i18n.tr("image-editor-light-apply")).size(typography::BODY_LG))
        .padding(spacing::SM)
        .width(Length::Fill);
    let apply_btn = if adjustment.has_changes() {
        apply_btn.on_press(SidebarMessage::ApplyAdjustments.into())
    } else {
        apply_btn.style(button_styles::disabled())
    };

    let buttons_row = Row::new()
        .spacing(spacing::XS)
        .push(reset_btn)
        .push(apply_btn);

    container(
        Column::new()
            .spacing(spacing::SM)
            .push(text(ctx.i18n.tr("image-editor-light-section-title")).size(typography::BODY))
            .push(brightness_section)
            .push(contrast_section)
            .push(buttons_row),
    )
    .padding(spacing::SM)
    .width(Length::Fill)
    .style(styles::editor::settings_panel)
    .into()
}
