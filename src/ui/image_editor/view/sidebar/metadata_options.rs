// SPDX-License-Identifier: MPL-2.0
//! Metadata preservation options panel for the image editor sidebar.
//!
//! This panel is displayed in the footer section above the save buttons,
//! allowing users to control how metadata is handled when saving.

use crate::ui::design_tokens::{spacing, typography};
use crate::ui::image_editor::state::MetadataPreservationOptions;
use crate::ui::image_editor::{Message, SidebarMessage, ViewContext};
use crate::ui::styles;
use iced::widget::{checkbox, container, text, Column};
use iced::{Element, Length};

/// Renders the metadata options section.
///
/// Only displayed when editing a file (not captured frames).
/// The "Strip GPS" checkbox is only shown if the image has GPS data.
pub fn section<'a>(
    options: &MetadataPreservationOptions,
    has_gps: bool,
    ctx: &ViewContext<'a>,
) -> Element<'a, Message> {
    let mut column = Column::new().spacing(spacing::XXS);

    // Section title
    column = column
        .push(text(ctx.i18n.tr("image-editor-metadata-options-title")).size(typography::BODY));

    // "Add software/modification date" checkbox (checked by default)
    let software_checkbox = checkbox(options.add_software_tag)
        .label(ctx.i18n.tr("image-editor-metadata-add-software"))
        .on_toggle(|_| SidebarMessage::ToggleAddSoftwareTag.into())
        .text_size(typography::BODY_SM);

    column = column.push(software_checkbox);

    // "Strip GPS data" checkbox (only shown if image has GPS)
    if has_gps {
        let gps_checkbox = checkbox(options.strip_gps)
            .label(ctx.i18n.tr("image-editor-metadata-strip-gps"))
            .on_toggle(|_| SidebarMessage::ToggleStripGps.into())
            .text_size(typography::BODY_SM);

        column = column.push(gps_checkbox);
    }

    container(column)
        .padding(spacing::SM)
        .width(Length::Fill)
        .style(styles::container::panel)
        .into()
}
