// SPDX-License-Identifier: MPL-2.0
//! AI deblur tool panel.
#![allow(clippy::cast_precision_loss)]

use crate::media::deblur::ModelStatus;
use crate::ui::design_tokens::{spacing, typography};
use crate::ui::image_editor::state::DeblurState;
use crate::ui::image_editor::{Message, SidebarMessage};
use crate::ui::styles;
use crate::ui::styles::button as button_styles;
use crate::ui::theme;
use iced::widget::{button, container, progress_bar, text, Button, Column, Text};
use iced::{Color, Element, Length, Theme};

use super::super::ViewContext;

/// Creates the disabled apply button used when deblur action is unavailable.
fn disabled_apply_button<'a>(label: String) -> Button<'a, Message> {
    button(text(label).size(typography::BODY_LG))
        .padding(spacing::SM)
        .width(Length::Fill)
        .style(button_styles::disabled())
}

/// Creates a styled status text with the given color.
fn status_text<'a>(message: String, color: Color) -> Text<'a> {
    text(message)
        .size(typography::BODY_SM)
        .style(move |_: &Theme| iced::widget::text::Style { color: Some(color) })
}

/// Render the deblur tool panel.
///
/// Shows:
/// - Lossless export recommendation warning
/// - Apply button (or progress during processing)
/// - Status-specific messages (downloading, validating, error, already applied)
pub fn panel<'a>(
    deblur: &'a DeblurState,
    model_status: &'a ModelStatus,
    has_deblur_applied: bool,
    ctx: &ViewContext<'a>,
) -> Element<'a, Message> {
    let apply_label = ctx.i18n.tr("image-editor-deblur-apply");
    let mut content = Column::new().spacing(spacing::SM);

    // Lossless export warning
    let muted_color = theme::muted_text_color_for_theme(ctx.is_dark_theme);
    content = content.push(status_text(
        ctx.i18n.tr("image-editor-deblur-lossless-warning"),
        muted_color,
    ));

    if deblur.is_processing {
        content = content.push(status_text(
            ctx.i18n.tr("image-editor-deblur-processing"),
            muted_color,
        ));
        content = content.push(disabled_apply_button(apply_label));
    } else if has_deblur_applied {
        content = content.push(status_text(
            ctx.i18n.tr("image-editor-deblur-already-applied"),
            theme::success_text_color(),
        ));
        content = content.push(disabled_apply_button(apply_label));
    } else {
        content = build_model_status_ui(content, model_status, &apply_label, ctx);
    }

    container(content)
        .padding(spacing::SM)
        .width(Length::Fill)
        .style(styles::editor::settings_panel)
        .into()
}

/// Builds the UI elements based on model status when deblur is available.
fn build_model_status_ui<'a>(
    mut content: Column<'a, Message>,
    model_status: &ModelStatus,
    apply_label: &str,
    ctx: &ViewContext<'_>,
) -> Column<'a, Message> {
    let muted_color = theme::muted_text_color_for_theme(ctx.is_dark_theme);
    match model_status {
        ModelStatus::Ready => {
            let apply_btn = button(text(apply_label.to_string()).size(typography::BODY_LG))
                .padding(spacing::SM)
                .width(Length::Fill)
                .on_press(SidebarMessage::ApplyDeblur.into());
            content.push(apply_btn)
        }
        ModelStatus::NeedsValidation | ModelStatus::Validating => {
            // NeedsValidation transitions to Validating when entering the editor
            content = content.push(status_text(
                ctx.i18n.tr("image-editor-deblur-validating"),
                muted_color,
            ));
            content.push(disabled_apply_button(apply_label.to_string()))
        }
        ModelStatus::Downloading { progress } => {
            content = content.push(progress_bar(0.0..=1.0, *progress));

            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let percent = (*progress * 100.0) as u32;
            content = content.push(
                text(ctx.i18n.tr_with_args(
                    "image-editor-deblur-downloading",
                    &[("progress", format!("{percent}").as_str())],
                ))
                .size(typography::BODY_SM),
            );
            content.push(disabled_apply_button(apply_label.to_string()))
        }
        ModelStatus::NotDownloaded => {
            content = content.push(status_text(
                ctx.i18n.tr("image-editor-deblur-model-not-ready"),
                theme::error_text_color(),
            ));
            content.push(disabled_apply_button(apply_label.to_string()))
        }
        ModelStatus::Error(error_msg) => {
            content = content.push(status_text(
                ctx.i18n.tr_with_args(
                    "image-editor-deblur-error",
                    &[("error", error_msg.as_str())],
                ),
                theme::error_text_color(),
            ));
            content.push(disabled_apply_button(apply_label.to_string()))
        }
    }
}
