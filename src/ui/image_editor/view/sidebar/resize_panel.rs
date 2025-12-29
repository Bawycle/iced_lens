// SPDX-License-Identifier: MPL-2.0
//! Resize tool panel for the editor sidebar.
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]

use crate::app::config::{MAX_RESIZE_SCALE_PERCENT, MIN_RESIZE_SCALE_PERCENT};
use crate::media::upscale::UpscaleModelStatus;
use crate::media::ImageData;
use crate::ui::design_tokens::{spacing, typography};
use crate::ui::styles;
use iced::widget::{button, checkbox, container, image, slider, text, text_input, Column, Row};
use iced::{Element, Length};

use super::super::ViewContext;
use crate::ui::image_editor::state::ResizeState;
use crate::ui::image_editor::{Message, SidebarMessage};

/// Maximum size for the thumbnail preview in the sidebar.
const THUMBNAIL_MAX_SIZE: f32 = 150.0;

// Allow too_many_lines: declarative UI composition for resize panel.
// Linear widget construction without complex logic; extraction would
// add indirection without reducing complexity.
#[allow(clippy::too_many_lines)]
pub fn panel<'a>(
    resize: &'a ResizeState,
    thumbnail: Option<&'a ImageData>,
    upscale_model_status: &'a UpscaleModelStatus,
    enable_upscale: bool,
    ctx: &ViewContext<'a>,
) -> Element<'a, Message> {
    let scale_section = Column::new()
        .spacing(spacing::XXS)
        .push(text(ctx.i18n.tr("image-editor-resize-section-title")).size(typography::BODY))
        .push(text(ctx.i18n.tr("image-editor-resize-scale-label")).size(typography::BODY_SM))
        .push(
            slider(
                MIN_RESIZE_SCALE_PERCENT..=MAX_RESIZE_SCALE_PERCENT,
                resize.scale.value(),
                |percent| Message::Sidebar(SidebarMessage::ScaleChanged(percent)),
            )
            .step(1.0),
        )
        .push(text(format!("{:.0}%", resize.scale.value())).size(typography::BODY_SM));

    // Presets: reduction presets on first row, enlargement presets on second row
    // Both rows have 4 buttons for uniform width
    let reduction_presets = Row::new()
        .spacing(spacing::XS)
        .push(preset_button(25.0))
        .push(preset_button(50.0))
        .push(preset_button(75.0))
        .push(preset_button(100.0));

    let enlargement_presets = Row::new()
        .spacing(spacing::XS)
        .push(preset_button(150.0))
        .push(preset_button(200.0))
        .push(preset_button(300.0))
        .push(preset_button(400.0));

    let presets_section = Column::new()
        .spacing(spacing::XXS)
        .push(text(ctx.i18n.tr("image-editor-resize-presets-label")).size(typography::BODY_SM))
        .push(reduction_presets)
        .push(enlargement_presets);

    let width_placeholder = ctx.i18n.tr("image-editor-resize-width-label");
    let width_label = text(width_placeholder.clone()).size(typography::BODY_SM);
    let width_input = text_input(width_placeholder.as_str(), &resize.width_input)
        .on_input(|value| Message::Sidebar(SidebarMessage::WidthInputChanged(value)))
        .on_submit(Message::Sidebar(SidebarMessage::WidthInputSubmitted))
        .padding(spacing::XXS)
        .size(typography::BODY)
        .width(Length::Fill);

    let height_placeholder = ctx.i18n.tr("image-editor-resize-height-label");
    let height_label = text(height_placeholder.clone()).size(typography::BODY_SM);
    let height_input = text_input(height_placeholder.as_str(), &resize.height_input)
        .on_input(|value| Message::Sidebar(SidebarMessage::HeightInputChanged(value)))
        .on_submit(Message::Sidebar(SidebarMessage::HeightInputSubmitted))
        .padding(spacing::XXS)
        .size(typography::BODY)
        .width(Length::Fill);

    let dimensions_row = Row::new()
        .spacing(spacing::XS)
        .push(
            Column::new()
                .spacing(spacing::XXS)
                .width(Length::Fill)
                .push(width_label)
                .push(width_input),
        )
        .push(
            Column::new()
                .spacing(spacing::XXS)
                .width(Length::Fill)
                .push(height_label)
                .push(height_input),
        );

    let lock_checkbox = checkbox(resize.lock_aspect)
        .label(ctx.i18n.tr("image-editor-resize-lock-aspect"))
        .on_toggle(|_| Message::Sidebar(SidebarMessage::ToggleLockAspect));

    // Build content with controls first, preview at the bottom
    // This prevents layout shift when user types in input fields
    let mut content = Column::new()
        .spacing(spacing::SM)
        .push(scale_section)
        .push(presets_section)
        .push(text(ctx.i18n.tr("image-editor-resize-dimensions-label")).size(typography::BODY_SM))
        .push(dimensions_row)
        .push(lock_checkbox);

    // Show AI upscale checkbox when the feature is enabled globally.
    // Disable (not hide) when conditions aren't met to prevent layout shift.
    // Use tooltip to explain why it's disabled (no layout shift).
    if enable_upscale {
        let is_enlargement = resize.scale.value() > 100.0;
        let model_ready = matches!(upscale_model_status, UpscaleModelStatus::Ready);
        let can_use_ai_upscale = is_enlargement && model_ready;

        let ai_upscale_checkbox = checkbox(resize.use_ai_upscale && can_use_ai_upscale)
            .label(ctx.i18n.tr("image-editor-resize-ai-upscale"));

        // Enable the checkbox only if enlarging AND model is ready
        let ai_upscale_checkbox = if can_use_ai_upscale {
            ai_upscale_checkbox.on_toggle(|_| Message::Sidebar(SidebarMessage::ToggleAiUpscale))
        } else {
            ai_upscale_checkbox
        };

        // Determine tooltip text when disabled
        let tooltip_text: Option<String> = if is_enlargement {
            match upscale_model_status {
                UpscaleModelStatus::NotDownloaded => {
                    Some(ctx.i18n.tr("image-editor-resize-ai-model-not-downloaded"))
                }
                UpscaleModelStatus::Downloading { progress } => {
                    // Progress is 0.0-1.0, so *100 is 0-100 which fits in u32
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let percent = (*progress * 100.0) as u32;
                    Some(format!(
                        "{} ({}%)",
                        ctx.i18n.tr("image-editor-resize-ai-model-downloading"),
                        percent
                    ))
                }
                UpscaleModelStatus::Validating => {
                    Some(ctx.i18n.tr("image-editor-resize-ai-model-validating"))
                }
                UpscaleModelStatus::Error(msg) => Some(format!(
                    "{}: {}",
                    ctx.i18n.tr("image-editor-resize-ai-model-error"),
                    msg
                )),
                UpscaleModelStatus::Ready => None,
            }
        } else {
            Some(ctx.i18n.tr("image-editor-resize-ai-enlargement-only"))
        };

        // Wrap in tooltip when disabled, otherwise just show the checkbox
        if let Some(hint) = tooltip_text {
            content = content.push(styles::tooltip::styled(
                ai_upscale_checkbox,
                hint,
                iced::widget::tooltip::Position::Top,
            ));
        } else {
            content = content.push(ai_upscale_checkbox);
        }
    }

    let apply_btn = {
        let btn = button(text(ctx.i18n.tr("image-editor-resize-apply")).size(typography::BODY_LG))
            .padding(spacing::SM)
            .width(Length::Fill);

        // Only enable the button if there are pending changes to apply
        if resize.has_pending_changes() {
            btn.on_press(SidebarMessage::ApplyResize.into())
        } else {
            btn
        }
    };

    content = content.push(apply_btn);

    // Preview section at the bottom - only shown when there are changes
    // Being at the bottom means it won't shift the controls above when it appears
    // Note: thumbnail is a small preview for performance; display target dimensions from resize state
    if let Some(img) = thumbnail {
        let (display_width, display_height) = calculate_thumbnail_size(img.width, img.height);

        let preview_image = image(img.handle.clone())
            .width(Length::Fixed(display_width))
            .height(Length::Fixed(display_height));

        // Show target dimensions from resize state, not thumbnail dimensions
        let target_width = resize.width;
        let target_height = resize.height;

        let preview_section = Column::new()
            .spacing(spacing::XXS)
            .align_x(iced::Alignment::Center)
            .push(text(ctx.i18n.tr("image-editor-resize-preview-label")).size(typography::BODY_SM))
            .push(
                container(preview_image)
                    .width(Length::Fill)
                    .center_x(Length::Fill),
            )
            .push(
                text(format!("{target_width}×{target_height} px"))
                    .size(typography::BODY_SM)
                    .center(),
            );

        content = content.push(preview_section);
    }

    container(content)
        .padding(spacing::SM)
        .width(Length::Fill)
        .style(styles::editor::settings_panel)
        .into()
}

/// Creates a preset button for a given scale percentage.
/// Uses `Length::Fill` to ensure uniform width within each row.
fn preset_button(percent: f32) -> iced::widget::Button<'static, Message> {
    let label = format!("{percent:.0}%");
    button(text(label).center())
        .on_press(SidebarMessage::ApplyResizePreset(percent).into())
        .padding([spacing::XXS, spacing::XS])
        .width(Length::Fill)
}

/// Calculate thumbnail display size while preserving aspect ratio.
fn calculate_thumbnail_size(width: u32, height: u32) -> (f32, f32) {
    // Image dimensions < 2^24 are exactly representable in f32
    #[allow(clippy::cast_precision_loss)]
    let w = width as f32;
    #[allow(clippy::cast_precision_loss)]
    let h = height as f32;

    if w <= THUMBNAIL_MAX_SIZE && h <= THUMBNAIL_MAX_SIZE {
        (w, h)
    } else if w > h {
        let scale = THUMBNAIL_MAX_SIZE / w;
        (THUMBNAIL_MAX_SIZE, h * scale)
    } else {
        let scale = THUMBNAIL_MAX_SIZE / h;
        (w * scale, THUMBNAIL_MAX_SIZE)
    }
}
