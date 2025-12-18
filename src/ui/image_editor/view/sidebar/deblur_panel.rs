// SPDX-License-Identifier: MPL-2.0
//! AI deblur tool panel.

use crate::media::deblur::ModelStatus;
use crate::ui::design_tokens::{spacing, typography};
use crate::ui::image_editor::state::DeblurState;
use crate::ui::image_editor::{Message, SidebarMessage};
use crate::ui::styles;
use crate::ui::styles::button as button_styles;
use crate::ui::theme;
use iced::widget::{button, container, progress_bar, text, Column};
use iced::{Element, Length, Theme};

use super::super::ViewContext;

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
    let mut content = Column::new().spacing(spacing::SM);

    // Lossless export warning
    let warning_text = text(ctx.i18n.tr("image-editor-deblur-lossless-warning"))
        .size(typography::BODY_SM)
        .style(move |_: &Theme| iced::widget::text::Style {
            color: Some(theme::muted_text_color()),
        });
    content = content.push(warning_text);

    if deblur.is_processing {
        // Processing state: show processing message (no progress bar since ONNX inference is atomic)
        let processing_text = text(ctx.i18n.tr("image-editor-deblur-processing"))
            .size(typography::BODY_SM)
            .style(move |_: &Theme| iced::widget::text::Style {
                color: Some(theme::muted_text_color()),
            });
        content = content.push(processing_text);

        // Show disabled apply button to indicate operation in progress
        let apply_btn =
            button(text(ctx.i18n.tr("image-editor-deblur-apply")).size(typography::BODY_LG))
                .padding(spacing::SM)
                .width(Length::Fill)
                .style(button_styles::disabled());
        content = content.push(apply_btn);
    } else if has_deblur_applied {
        // Deblur already applied: show message and disabled button
        let already_applied_text = text(ctx.i18n.tr("image-editor-deblur-already-applied"))
            .size(typography::BODY_SM)
            .style(move |_: &Theme| iced::widget::text::Style {
                color: Some(theme::success_text_color()),
            });
        content = content.push(already_applied_text);

        let apply_btn =
            button(text(ctx.i18n.tr("image-editor-deblur-apply")).size(typography::BODY_LG))
                .padding(spacing::SM)
                .width(Length::Fill)
                .style(button_styles::disabled());
        content = content.push(apply_btn);
    } else {
        // Show status-specific UI based on model status
        match model_status {
            ModelStatus::Ready => {
                // Model ready: show apply button
                let apply_btn = button(
                    text(ctx.i18n.tr("image-editor-deblur-apply")).size(typography::BODY_LG),
                )
                .padding(spacing::SM)
                .width(Length::Fill)
                .on_press(SidebarMessage::ApplyDeblur.into());
                content = content.push(apply_btn);
            }
            ModelStatus::Validating => {
                // Model is being validated at startup
                let validating_text = text(ctx.i18n.tr("image-editor-deblur-validating"))
                    .size(typography::BODY_SM)
                    .style(move |_: &Theme| iced::widget::text::Style {
                        color: Some(theme::muted_text_color()),
                    });
                content = content.push(validating_text);

                let apply_btn = button(
                    text(ctx.i18n.tr("image-editor-deblur-apply")).size(typography::BODY_LG),
                )
                .padding(spacing::SM)
                .width(Length::Fill)
                .style(button_styles::disabled());
                content = content.push(apply_btn);
            }
            ModelStatus::Downloading { progress } => {
                // Model is downloading
                let download_progress = progress_bar(0.0..=1.0, *progress);
                content = content.push(download_progress);

                let progress_text = text(ctx.i18n.tr_with_args(
                    "image-editor-deblur-downloading",
                    &[(
                        "progress",
                        format!("{}", (*progress * 100.0) as u32).as_str(),
                    )],
                ))
                .size(typography::BODY_SM);
                content = content.push(progress_text);

                let apply_btn = button(
                    text(ctx.i18n.tr("image-editor-deblur-apply")).size(typography::BODY_LG),
                )
                .padding(spacing::SM)
                .width(Length::Fill)
                .style(button_styles::disabled());
                content = content.push(apply_btn);
            }
            ModelStatus::NotDownloaded => {
                // Model not downloaded: prompt user to enable in settings
                let not_ready_text = text(ctx.i18n.tr("image-editor-deblur-model-not-ready"))
                    .size(typography::BODY_SM)
                    .style(move |_: &Theme| iced::widget::text::Style {
                        color: Some(theme::error_text_color()),
                    });
                content = content.push(not_ready_text);

                let apply_btn = button(
                    text(ctx.i18n.tr("image-editor-deblur-apply")).size(typography::BODY_LG),
                )
                .padding(spacing::SM)
                .width(Length::Fill)
                .style(button_styles::disabled());
                content = content.push(apply_btn);
            }
            ModelStatus::Error(error_msg) => {
                // Error state: show error message
                let error_text = text(ctx.i18n.tr_with_args(
                    "image-editor-deblur-error",
                    &[("error", error_msg.as_str())],
                ))
                .size(typography::BODY_SM)
                .style(move |_: &Theme| iced::widget::text::Style {
                    color: Some(theme::error_text_color()),
                });
                content = content.push(error_text);

                let apply_btn = button(
                    text(ctx.i18n.tr("image-editor-deblur-apply")).size(typography::BODY_LG),
                )
                .padding(spacing::SM)
                .width(Length::Fill)
                .style(button_styles::disabled());
                content = content.push(apply_btn);
            }
        }
    }

    container(content)
        .padding(spacing::SM)
        .width(Length::Fill)
        .style(styles::editor::settings_panel)
        .into()
}
