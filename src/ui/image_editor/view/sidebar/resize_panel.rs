// SPDX-License-Identifier: MPL-2.0
//! Resize tool panel for the editor sidebar.

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

pub fn panel<'a>(
    resize: &'a ResizeState,
    thumbnail: Option<&'a ImageData>,
    ctx: &ViewContext<'a>,
) -> Element<'a, Message> {
    let scale_section = Column::new()
        .spacing(spacing::XXS)
        .push(text(ctx.i18n.tr("image-editor-resize-section-title")).size(typography::BODY))
        .push(text(ctx.i18n.tr("image-editor-resize-scale-label")).size(typography::BODY_SM))
        .push(
            slider(10.0..=200.0, resize.scale.value(), |percent| {
                Message::Sidebar(SidebarMessage::ScaleChanged(percent))
            })
            .step(1.0),
        )
        .push(text(format!("{:.0}%", resize.scale.value())).size(typography::BODY_SM));

    let mut presets = Row::new().spacing(spacing::XS);
    for preset in [50.0, 75.0, 150.0, 200.0] {
        let label = format!("{preset:.0}%");
        presets = presets.push(
            button(text(label))
                .on_press(SidebarMessage::ApplyResizePreset(preset).into())
                .padding([spacing::XXS, spacing::XS]),
        );
    }

    let presets_section = Column::new()
        .spacing(spacing::XXS)
        .push(text(ctx.i18n.tr("image-editor-resize-presets-label")).size(typography::BODY_SM))
        .push(presets);

    let width_placeholder = ctx.i18n.tr("image-editor-resize-width-label");
    let width_label = text(width_placeholder.clone()).size(typography::BODY_SM);
    let width_input = text_input(width_placeholder.as_str(), &resize.width_input)
        .on_input(|value| Message::Sidebar(SidebarMessage::WidthInputChanged(value)))
        .padding(spacing::XXS)
        .size(typography::BODY)
        .width(Length::Fill);

    let height_placeholder = ctx.i18n.tr("image-editor-resize-height-label");
    let height_label = text(height_placeholder.clone()).size(typography::BODY_SM);
    let height_input = text_input(height_placeholder.as_str(), &resize.height_input)
        .on_input(|value| Message::Sidebar(SidebarMessage::HeightInputChanged(value)))
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

    let apply_btn =
        button(text(ctx.i18n.tr("image-editor-resize-apply")).size(typography::BODY_LG))
            .padding(spacing::SM)
            .width(Length::Fill)
            .on_press(SidebarMessage::ApplyResize.into());

    // Build content with controls first, preview at the bottom
    // This prevents layout shift when user types in input fields
    let mut content = Column::new()
        .spacing(spacing::SM)
        .push(scale_section)
        .push(presets_section)
        .push(text(ctx.i18n.tr("image-editor-resize-dimensions-label")).size(typography::BODY_SM))
        .push(dimensions_row)
        .push(lock_checkbox)
        .push(apply_btn);

    // Preview section at the bottom - only shown when there are changes
    // Being at the bottom means it won't shift the controls above when it appears
    if let Some(img) = thumbnail {
        let (display_width, display_height) = calculate_thumbnail_size(img.width, img.height);

        let preview_image = image(img.handle.clone())
            .width(Length::Fixed(display_width))
            .height(Length::Fixed(display_height));

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
                text(format!("{}×{} px", img.width, img.height))
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

/// Calculate thumbnail display size while preserving aspect ratio.
fn calculate_thumbnail_size(width: u32, height: u32) -> (f32, f32) {
    let w = width as f32;
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
