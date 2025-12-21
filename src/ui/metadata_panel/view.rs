// SPDX-License-Identifier: MPL-2.0
//! View rendering for the metadata panel.

use super::{Message, MetadataEditorState, MetadataField, PanelContext};
use crate::i18n::fluent::I18n;
use crate::media::metadata::{
    format_bitrate, format_file_size, format_gps_coordinates, ExtendedVideoMetadata, ImageMetadata,
    MediaMetadata,
};
use crate::ui::action_icons;
use crate::ui::design_tokens::{palette, radius, sizing, spacing, typography};
use crate::ui::icons;
use crate::ui::styles::button as button_styles;
use iced::widget::image::{Handle, Image};
use crate::ui::styles::tooltip as styled_tooltip;
use iced::widget::{
    button, container, pick_list, rule, scrollable, text, text_input, Column, Row, Text,
};
use iced::{alignment::Vertical, Border, Element, Length, Padding, Theme};

/// Width of the metadata panel in pixels.
pub const PANEL_WIDTH: f32 = 290.0;

/// Contextual data needed to render the metadata panel (legacy, for backward compatibility).
pub struct ViewContext<'a> {
    pub i18n: &'a I18n,
    pub metadata: Option<&'a MediaMetadata>,
    pub is_dark_theme: bool,
}

/// Render the metadata panel.
pub fn panel<'a>(ctx: PanelContext<'a>) -> Element<'a, Message> {
    let is_editing = ctx.editor_state.is_some();
    let title = Text::new(ctx.i18n.tr("metadata-panel-title")).size(typography::TITLE_SM);

    // Header buttons
    let has_unsaved_changes = ctx
        .editor_state
        .map(|editor| editor.has_changes())
        .unwrap_or(false);
    let header_buttons = build_header_buttons(&ctx, is_editing, has_unsaved_changes);

    // Header row with title and buttons
    let header = Row::new()
        .width(Length::Fill)
        .align_y(Vertical::Center)
        .push(title)
        .push(iced::widget::Space::new().width(Length::Fill))
        .push(header_buttons);

    // Content depends on edit mode
    let content = if let Some(metadata) = ctx.metadata {
        if is_editing {
            if let MediaMetadata::Image(image_meta) = metadata {
                build_edit_content(&ctx, image_meta)
            } else {
                // Should not happen - videos can't be in edit mode
                build_view_content(&ctx, metadata)
            }
        } else {
            build_view_content(&ctx, metadata)
        }
    } else {
        Column::new()
            .push(Text::new(ctx.i18n.tr("metadata-value-unknown")).size(typography::BODY))
            .into()
    };

    let mut panel_content = Column::new()
        .width(Length::Fill)
        .spacing(spacing::MD)
        .padding(spacing::MD)
        .push(header)
        .push(rule::horizontal(1))
        .push(content);

    // Add footer with save buttons when editing
    if is_editing {
        panel_content = panel_content.push(build_edit_footer(&ctx));
    }

    let scrollable_content = scrollable(panel_content).width(Length::Fixed(PANEL_WIDTH));

    container(scrollable_content)
        .width(Length::Fixed(PANEL_WIDTH))
        .height(Length::Fill)
        .style(|theme: &Theme| container::Style {
            background: Some(theme.extended_palette().background.weak.color.into()),
            border: Border {
                radius: radius::MD.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

/// Build header buttons (Edit, Close).
fn build_header_buttons<'a>(
    ctx: &PanelContext<'a>,
    is_editing: bool,
    has_unsaved_changes: bool,
) -> Row<'a, Message> {
    let mut buttons = Row::new().spacing(spacing::XS).align_y(Vertical::Center);

    // Edit button (only for images, not in edit mode)
    if !is_editing && ctx.is_image {
        let edit_tooltip = ctx.i18n.tr("metadata-edit-button");
        let edit_btn = button(action_icons::sized(
            action_icons::navigation::edit(ctx.is_dark_theme),
            sizing::ICON_SM,
        ))
        .on_press(Message::EnterEditMode)
        .padding(spacing::XXS);

        let edit_button = styled_tooltip::styled(
            edit_btn,
            edit_tooltip,
            iced::widget::tooltip::Position::Bottom,
        );
        buttons = buttons.push(edit_button);
    } else if !is_editing && !ctx.is_image && ctx.metadata.is_some() {
        // Disabled edit button for videos with tooltip
        let edit_btn = button(action_icons::sized(
            action_icons::navigation::edit(ctx.is_dark_theme),
            sizing::ICON_SM,
        ))
        .padding(spacing::XXS)
        .style(button_styles::disabled());

        let edit_button = styled_tooltip::styled(
            edit_btn,
            ctx.i18n.tr("metadata-edit-disabled-video"),
            iced::widget::tooltip::Position::Bottom,
        );
        buttons = buttons.push(edit_button);
    }

    // Close button - disabled when there are unsaved changes
    let close_btn = button(action_icons::sized(
        action_icons::navigation::collapse_right_panel(ctx.is_dark_theme),
        sizing::ICON_SM,
    ))
    .padding(spacing::XXS);

    let (close_btn, close_tooltip) = if has_unsaved_changes {
        // Disabled: cannot close with unsaved changes
        (
            close_btn.style(button_styles::disabled()),
            ctx.i18n.tr("metadata-panel-close-disabled"),
        )
    } else {
        // Enabled: can close normally
        (
            close_btn.on_press(Message::Close),
            ctx.i18n.tr("metadata-panel-close"),
        )
    };

    let close_button = styled_tooltip::styled(
        close_btn,
        close_tooltip,
        iced::widget::tooltip::Position::Bottom,
    );

    buttons.push(close_button)
}

/// Build view mode content.
fn build_view_content<'a>(
    ctx: &PanelContext<'a>,
    metadata: &MediaMetadata,
) -> Element<'a, Message> {
    match metadata {
        MediaMetadata::Image(image_meta) => build_image_metadata_view(ctx.i18n, image_meta),
        MediaMetadata::Video(video_meta) => build_video_metadata_view(ctx.i18n, video_meta),
    }
}

/// Build edit mode content for images with progressive disclosure.
fn build_edit_content<'a>(ctx: &PanelContext<'a>, _meta: &ImageMetadata) -> Element<'a, Message> {
    let editor = ctx
        .editor_state
        .expect("Editor state required for edit mode");

    let mut sections = Column::new().spacing(spacing::MD);

    // Dublin Core / XMP section first (user-facing metadata)
    if let Some(dc_section) = build_dublin_core_section_edit(ctx.i18n, editor) {
        sections = sections.push(dc_section);
    }

    // Camera section
    if let Some(camera_section) = build_camera_section_edit(ctx.i18n, editor) {
        sections = sections.push(camera_section);
    }

    // Exposure section
    if let Some(exposure_section) = build_exposure_section_edit(ctx.i18n, editor) {
        sections = sections.push(exposure_section);
    }

    // GPS section
    if let Some(gps_section) = build_gps_section_edit(ctx.i18n, editor) {
        sections = sections.push(gps_section);
    }

    // Show message if no fields are visible
    if editor.visible_fields.is_empty() {
        sections = sections.push(
            text(ctx.i18n.tr("metadata-no-fields-message"))
                .size(typography::BODY)
                .color(palette::GRAY_400),
        );
    }

    // Add field picker (only if there are available fields)
    let available = editor.available_fields();
    if !available.is_empty() {
        sections = sections.push(build_add_field_picker(ctx.i18n, &available));
    }

    sections.into()
}

/// Build footer with save buttons for edit mode.
fn build_edit_footer<'a>(ctx: &PanelContext<'a>) -> Column<'a, Message> {
    let editor = ctx.editor_state.expect("Editor state required for footer");
    let has_changes = editor.has_changes();
    let has_errors = editor.errors.has_errors();

    let mut footer = Column::new().spacing(spacing::XS);

    // Warning text about modifying original file
    let warning_text = text(ctx.i18n.tr("metadata-save-warning"))
        .size(typography::CAPTION)
        .color(palette::WARNING_500);
    footer = footer.push(warning_text);

    // Button row
    let mut button_row = Row::new().spacing(spacing::XS);

    // Cancel button
    let cancel_btn = button(text(ctx.i18n.tr("metadata-cancel-button")).size(typography::BODY))
        .on_press(Message::ExitEditMode)
        .padding(spacing::SM)
        .width(Length::FillPortion(1));
    button_row = button_row.push(cancel_btn);

    // Save button (enabled only if changes and no errors)
    let save_btn = button(text(ctx.i18n.tr("metadata-save-button")).size(typography::BODY))
        .padding(spacing::SM)
        .width(Length::FillPortion(1));
    let save_btn = if has_changes && !has_errors && ctx.current_path.is_some() {
        save_btn.on_press(Message::Save)
    } else {
        save_btn.style(button_styles::disabled())
    };
    button_row = button_row.push(save_btn);

    footer = footer.push(button_row);

    // Save As button (always enabled when there are changes)
    let save_as_btn = button(text(ctx.i18n.tr("metadata-save-as-button")).size(typography::BODY))
        .padding(spacing::SM)
        .width(Length::Fill);
    let save_as_btn = if has_changes && !has_errors {
        save_as_btn.on_press(Message::SaveAs)
    } else {
        save_as_btn.style(button_styles::disabled())
    };
    footer = footer.push(save_as_btn);

    footer
}

// =============================================================================
// Edit Mode Sections
// =============================================================================

fn build_camera_section_edit<'a>(
    i18n: &'a I18n,
    editor: &MetadataEditorState,
) -> Option<Element<'a, Message>> {
    let mut rows = Column::new().spacing(spacing::XS);
    let mut has_fields = false;

    // Camera Make
    if editor.is_field_visible(&MetadataField::CameraMake) {
        rows = rows.push(build_edit_field_with_remove(
            i18n.tr("metadata-label-make"),
            &editor.edited.camera_make,
            MetadataField::CameraMake,
            None,
            None,
        ));
        has_fields = true;
    }

    // Camera Model
    if editor.is_field_visible(&MetadataField::CameraModel) {
        rows = rows.push(build_edit_field_with_remove(
            i18n.tr("metadata-label-model"),
            &editor.edited.camera_model,
            MetadataField::CameraModel,
            None,
            None,
        ));
        has_fields = true;
    }

    // Date Taken (uses smart date picker component)
    if editor.is_field_visible(&MetadataField::DateTaken) {
        rows = rows.push(build_date_field_with_remove(
            i18n,
            i18n.tr("metadata-label-date-taken"),
            &editor.edited.date_taken,
            editor.errors.date_taken.as_ref(),
        ));
        has_fields = true;
    }

    if has_fields {
        Some(build_section(
            icons::camera(),
            i18n.tr("metadata-section-camera"),
            rows.into(),
        ))
    } else {
        None
    }
}

fn build_exposure_section_edit<'a>(
    i18n: &'a I18n,
    editor: &MetadataEditorState,
) -> Option<Element<'a, Message>> {
    let mut rows = Column::new().spacing(spacing::XS);
    let mut has_fields = false;

    // Exposure Time
    if editor.is_field_visible(&MetadataField::ExposureTime) {
        rows = rows.push(build_edit_field_with_remove(
            i18n.tr("metadata-label-exposure"),
            &editor.edited.exposure_time,
            MetadataField::ExposureTime,
            Some("1/250".to_string()),
            editor.errors.exposure_time.as_ref(),
        ));
        has_fields = true;
    }

    // Aperture
    if editor.is_field_visible(&MetadataField::Aperture) {
        rows = rows.push(build_edit_field_with_remove(
            i18n.tr("metadata-label-aperture"),
            &editor.edited.aperture,
            MetadataField::Aperture,
            Some("f/2.8".to_string()),
            editor.errors.aperture.as_ref(),
        ));
        has_fields = true;
    }

    // ISO
    if editor.is_field_visible(&MetadataField::Iso) {
        rows = rows.push(build_edit_field_with_remove(
            i18n.tr("metadata-label-iso"),
            &editor.edited.iso,
            MetadataField::Iso,
            Some("100".to_string()),
            editor.errors.iso.as_ref(),
        ));
        has_fields = true;
    }

    // Focal Length
    if editor.is_field_visible(&MetadataField::FocalLength) {
        rows = rows.push(build_edit_field_with_remove(
            i18n.tr("metadata-label-focal-length"),
            &editor.edited.focal_length,
            MetadataField::FocalLength,
            Some("50 mm".to_string()),
            editor.errors.focal_length.as_ref(),
        ));
        has_fields = true;
    }

    if has_fields {
        Some(build_section(
            icons::cog(),
            i18n.tr("metadata-section-exposure"),
            rows.into(),
        ))
    } else {
        None
    }
}

fn build_gps_section_edit<'a>(
    i18n: &'a I18n,
    editor: &MetadataEditorState,
) -> Option<Element<'a, Message>> {
    // GPS fields are treated as a pair - show both or none
    if !editor.is_field_visible(&MetadataField::GpsLatitude)
        && !editor.is_field_visible(&MetadataField::GpsLongitude)
    {
        return None;
    }

    let mut rows = Column::new().spacing(spacing::XS);

    // Latitude (with remove button that removes both GPS fields)
    rows = rows.push(build_edit_field_with_remove(
        i18n.tr("metadata-label-latitude"),
        &editor.edited.gps_latitude,
        MetadataField::GpsLatitude,
        Some("48.8566".to_string()),
        editor.errors.gps_latitude.as_ref(),
    ));

    // Longitude (no remove button, removing latitude removes both)
    rows = rows.push(build_edit_field(
        i18n.tr("metadata-label-longitude"),
        &editor.edited.gps_longitude,
        MetadataField::GpsLongitude,
        Some("2.3522".to_string()),
        editor.errors.gps_longitude.as_ref(),
    ));

    Some(build_section(
        icons::globe(),
        i18n.tr("metadata-section-gps"),
        rows.into(),
    ))
}

fn build_dublin_core_section_edit<'a>(
    i18n: &'a I18n,
    editor: &MetadataEditorState,
) -> Option<Element<'a, Message>> {
    let mut rows = Column::new().spacing(spacing::XS);
    let mut has_fields = false;

    // Title
    if editor.is_field_visible(&MetadataField::DcTitle) {
        rows = rows.push(build_edit_field_with_remove(
            i18n.tr("metadata-label-dc-title"),
            &editor.edited.dc_title,
            MetadataField::DcTitle,
            Some("Photo Title".to_string()),
            None,
        ));
        has_fields = true;
    }

    // Creator
    if editor.is_field_visible(&MetadataField::DcCreator) {
        rows = rows.push(build_edit_field_with_remove(
            i18n.tr("metadata-label-dc-creator"),
            &editor.edited.dc_creator,
            MetadataField::DcCreator,
            Some("John Doe".to_string()),
            None,
        ));
        has_fields = true;
    }

    // Description
    if editor.is_field_visible(&MetadataField::DcDescription) {
        rows = rows.push(build_edit_field_with_remove(
            i18n.tr("metadata-label-dc-description"),
            &editor.edited.dc_description,
            MetadataField::DcDescription,
            Some("A beautiful sunset".to_string()),
            None,
        ));
        has_fields = true;
    }

    // Subject (Keywords)
    if editor.is_field_visible(&MetadataField::DcSubject) {
        rows = rows.push(build_edit_field_with_remove(
            i18n.tr("metadata-label-dc-subject"),
            &editor.edited.dc_subject,
            MetadataField::DcSubject,
            Some("sunset, nature, landscape".to_string()),
            None,
        ));
        has_fields = true;
    }

    // Rights (Copyright)
    if editor.is_field_visible(&MetadataField::DcRights) {
        rows = rows.push(build_edit_field_with_remove(
            i18n.tr("metadata-label-dc-rights"),
            &editor.edited.dc_rights,
            MetadataField::DcRights,
            Some("(c) 2024 John Doe".to_string()),
            None,
        ));
        has_fields = true;
    }

    if has_fields {
        Some(build_section(
            icons::info(),
            i18n.tr("metadata-section-dublin-core"),
            rows.into(),
        ))
    } else {
        None
    }
}

/// Build an editable field with label, input, and optional error.
fn build_edit_field<'a>(
    label: String,
    value: &str,
    field: MetadataField,
    placeholder: Option<String>,
    error: Option<&String>,
) -> Element<'a, Message> {
    let mut col = Column::new().spacing(spacing::XXS);

    // Label
    col = col.push(text(format!("{}:", label)).size(typography::BODY_SM));

    // Input
    let placeholder_str = placeholder.unwrap_or_default();
    let input = text_input(&placeholder_str, value)
        .on_input(move |v| Message::FieldChanged(field, v))
        .padding(spacing::XS)
        .size(typography::BODY);
    col = col.push(input);

    // Error message if present
    if let Some(err) = error {
        col = col.push(
            text(err.clone())
                .size(typography::CAPTION)
                .color(palette::ERROR_500),
        );
    }

    col.into()
}

/// Build an editable field with a remove button.
fn build_edit_field_with_remove<'a>(
    label: String,
    value: &str,
    field: MetadataField,
    placeholder: Option<String>,
    error: Option<&String>,
) -> Element<'a, Message> {
    let mut col = Column::new().spacing(spacing::XXS);

    // Label row with remove button
    let label_row = Row::new()
        .spacing(spacing::XS)
        .align_y(Vertical::Center)
        .push(text(format!("{}:", label)).size(typography::BODY_SM))
        .push(iced::widget::Space::new().width(Length::Fill))
        .push(
            button(icons::sized(icons::cross(), sizing::ICON_SM))
                .on_press(Message::RemoveField(field))
                .padding(spacing::XXS),
        );
    col = col.push(label_row);

    // Input
    let placeholder_str = placeholder.unwrap_or_default();
    let input = text_input(&placeholder_str, value)
        .on_input(move |v| Message::FieldChanged(field, v))
        .padding(spacing::XS)
        .size(typography::BODY);
    col = col.push(input);

    // Error message if present
    if let Some(err) = error {
        col = col.push(
            text(err.clone())
                .size(typography::CAPTION)
                .color(palette::ERROR_500),
        );
    }

    col.into()
}

/// Build a smart date/time field with single input and intelligent parsing.
///
/// Accepts multiple date formats and converts to EXIF format (YYYY:MM:DD HH:MM:SS).
/// Includes a "Now" button for quick current date/time input.
fn build_date_field_with_remove<'a>(
    i18n: &'a I18n,
    label: String,
    value: &str,
    error: Option<&String>,
) -> Element<'a, Message> {
    let mut col = Column::new().spacing(spacing::XXS);

    // Label row with remove button
    let label_row = Row::new()
        .spacing(spacing::XS)
        .align_y(Vertical::Center)
        .push(text(format!("{}:", label)).size(typography::BODY_SM))
        .push(iced::widget::Space::new().width(Length::Fill))
        .push(
            button(icons::sized(icons::cross(), sizing::ICON_SM))
                .on_press(Message::RemoveField(MetadataField::DateTaken))
                .padding(spacing::XXS),
        );
    col = col.push(label_row);

    // Format the display value for better readability
    let display_value = format_date_for_display(value);

    // Input row with "Now" button
    let input_row = Row::new()
        .spacing(spacing::XS)
        .align_y(Vertical::Center)
        .push(
            text_input(&i18n.tr("metadata-date-placeholder"), &display_value)
                .on_input(|v| {
                    // Parse and convert to EXIF format
                    let exif_date = parse_date_input(&v);
                    Message::FieldChanged(MetadataField::DateTaken, exif_date)
                })
                .padding(spacing::XS)
                .size(typography::BODY)
                .width(Length::Fill),
        )
        .push(
            button(text(i18n.tr("metadata-date-now")).size(typography::BODY_SM))
                .on_press(Message::FieldChanged(
                    MetadataField::DateTaken,
                    get_current_datetime_exif(),
                ))
                .padding(spacing::XS),
        );
    col = col.push(input_row);

    // Help text
    col = col.push(
        text(i18n.tr("metadata-date-help"))
            .size(typography::CAPTION)
            .color(palette::GRAY_400),
    );

    // Error message if present
    if let Some(err) = error {
        col = col.push(
            text(err.clone())
                .size(typography::CAPTION)
                .color(palette::ERROR_500),
        );
    }

    col.into()
}

/// Format a date string for display (more readable format).
fn format_date_for_display(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }

    // Try to parse EXIF format and display in a more readable way
    if let Some(dt) = parse_exif_datetime(value) {
        return dt.format("%Y-%m-%d %H:%M:%S").to_string();
    }

    // Return as-is if can't parse
    value.to_string()
}

/// Parse EXIF datetime format (YYYY:MM:DD HH:MM:SS).
fn parse_exif_datetime(value: &str) -> Option<chrono::NaiveDateTime> {
    use chrono::NaiveDateTime;

    // EXIF format: "YYYY:MM:DD HH:MM:SS"
    NaiveDateTime::parse_from_str(value, "%Y:%m:%d %H:%M:%S").ok()
}

/// Parse various date input formats and convert to EXIF format.
fn parse_date_input(input: &str) -> String {
    use chrono::{NaiveDate, NaiveDateTime};

    let input = input.trim();
    if input.is_empty() {
        return String::new();
    }

    // List of formats to try (with time)
    let datetime_formats = [
        "%Y-%m-%d %H:%M:%S", // ISO: 2024-03-15 14:30:00
        "%Y-%m-%d %H:%M",    // ISO without seconds: 2024-03-15 14:30
        "%Y:%m:%d %H:%M:%S", // EXIF: 2024:03:15 14:30:00
        "%d/%m/%Y %H:%M:%S", // European: 15/03/2024 14:30:00
        "%d/%m/%Y %H:%M",    // European without seconds: 15/03/2024 14:30
        "%d-%m-%Y %H:%M:%S", // European with dashes: 15-03-2024 14:30:00
        "%d-%m-%Y %H:%M",    // European with dashes: 15-03-2024 14:30
        "%Y/%m/%d %H:%M:%S", // Alternative: 2024/03/15 14:30:00
        "%Y/%m/%d %H:%M",    // Alternative: 2024/03/15 14:30
    ];

    // Try datetime formats first
    for fmt in &datetime_formats {
        if let Ok(dt) = NaiveDateTime::parse_from_str(input, fmt) {
            return dt.format("%Y:%m:%d %H:%M:%S").to_string();
        }
    }

    // List of date-only formats to try
    let date_formats = [
        "%Y-%m-%d", // ISO: 2024-03-15
        "%Y:%m:%d", // EXIF date only: 2024:03:15
        "%d/%m/%Y", // European: 15/03/2024
        "%d-%m-%Y", // European with dashes: 15-03-2024
        "%Y/%m/%d", // Alternative: 2024/03/15
    ];

    // Try date-only formats (add midnight time)
    for fmt in &date_formats {
        if let Ok(d) = NaiveDate::parse_from_str(input, fmt) {
            let dt = d.and_hms_opt(0, 0, 0).unwrap();
            return dt.format("%Y:%m:%d %H:%M:%S").to_string();
        }
    }

    // If nothing matches, return input as-is (will fail validation if invalid)
    input.to_string()
}

/// Get current date/time in EXIF format.
fn get_current_datetime_exif() -> String {
    use chrono::Local;
    Local::now().format("%Y:%m:%d %H:%M:%S").to_string()
}

/// Wrapper for MetadataField to implement Display for pick_list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FieldOption {
    field: MetadataField,
    label: &'static str,
}

impl std::fmt::Display for FieldOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}

/// Build the "Add field" picker dropdown.
fn build_add_field_picker<'a>(i18n: &'a I18n, available: &[MetadataField]) -> Element<'a, Message> {
    // Create options with display labels
    // Order is determined by MetadataField::all() (Dublin Core first, then EXIF)
    let options: Vec<FieldOption> = available
        .iter()
        .map(|&field| FieldOption {
            field,
            label: match field {
                // Dublin Core / XMP fields (user-facing metadata)
                MetadataField::DcTitle => "Title",
                MetadataField::DcCreator => "Creator",
                MetadataField::DcDescription => "Description",
                MetadataField::DcSubject => "Keywords",
                MetadataField::DcRights => "Copyright",
                // EXIF fields (technical metadata)
                MetadataField::CameraMake => "Camera make",
                MetadataField::CameraModel => "Camera model",
                MetadataField::DateTaken => "Date taken",
                MetadataField::ExposureTime => "Exposure time",
                MetadataField::Aperture => "Aperture",
                MetadataField::Iso => "ISO",
                MetadataField::Flash => "Flash",
                MetadataField::FocalLength => "Focal length",
                MetadataField::FocalLength35mm => "Focal length (35mm)",
                MetadataField::GpsLatitude | MetadataField::GpsLongitude => "GPS coordinates",
            },
        })
        .collect();

    let picker = pick_list(options.clone(), None::<FieldOption>, |selected| {
        Message::ShowField(selected.field)
    })
    .placeholder(i18n.tr("metadata-add-field"))
    .width(Length::Fill)
    .padding(spacing::XS);

    container(picker)
        .width(Length::Fill)
        .padding(Padding::ZERO.top(spacing::SM))
        .into()
}

// =============================================================================
// View Mode Rendering (Read-Only)
// =============================================================================

fn build_image_metadata_view<'a>(i18n: &'a I18n, meta: &ImageMetadata) -> Element<'a, Message> {
    let mut sections = Column::new().spacing(spacing::MD);

    // File section (always first - basic file info)
    let file_section = build_file_section_image(i18n, meta);
    sections = sections.push(file_section);

    // Dublin Core / XMP section (user-facing metadata, shown second)
    if meta.dc_title.is_some()
        || meta.dc_creator.is_some()
        || meta.dc_description.is_some()
        || meta.dc_subject.is_some()
        || meta.dc_rights.is_some()
    {
        let dc_section = build_dublin_core_section_view(i18n, meta);
        sections = sections.push(dc_section);
    }

    // Camera section (if available)
    if meta.camera_make.is_some() || meta.camera_model.is_some() || meta.date_taken.is_some() {
        let camera_section = build_camera_section_view(i18n, meta);
        sections = sections.push(camera_section);
    }

    // Exposure section (if available)
    if meta.exposure_time.is_some()
        || meta.aperture.is_some()
        || meta.iso.is_some()
        || meta.flash.is_some()
        || meta.focal_length.is_some()
    {
        let exposure_section = build_exposure_section_view(i18n, meta);
        sections = sections.push(exposure_section);
    }

    // GPS section (if available)
    if meta.gps_latitude.is_some() && meta.gps_longitude.is_some() {
        let gps_section = build_gps_section_view(i18n, meta);
        sections = sections.push(gps_section);
    }

    sections.into()
}

fn build_video_metadata_view<'a>(
    i18n: &'a I18n,
    meta: &ExtendedVideoMetadata,
) -> Element<'a, Message> {
    let mut sections = Column::new().spacing(spacing::MD);

    // File section
    let file_section = build_file_section_video(i18n, meta);
    sections = sections.push(file_section);

    // Video section
    let video_section = build_video_codec_section(i18n, meta);
    sections = sections.push(video_section);

    // Audio section (if available)
    if meta.has_audio {
        let audio_section = build_audio_section(i18n, meta);
        sections = sections.push(audio_section);
    }

    sections.into()
}

fn build_file_section_image<'a>(i18n: &'a I18n, meta: &ImageMetadata) -> Element<'a, Message> {
    let mut rows = Column::new().spacing(spacing::XS);

    if meta.width.is_some() || meta.height.is_some() {
        let dims = format!(
            "{} x {} px",
            meta.width
                .map_or_else(|| "?".to_string(), |v| v.to_string()),
            meta.height
                .map_or_else(|| "?".to_string(), |v| v.to_string())
        );
        rows = rows.push(build_metadata_row(
            i18n.tr("metadata-label-dimensions"),
            dims,
        ));
    }

    if let Some(size) = meta.file_size {
        rows = rows.push(build_metadata_row(
            i18n.tr("metadata-label-file-size"),
            format_file_size(size),
        ));
    }

    if let Some(ref format) = meta.format {
        rows = rows.push(build_metadata_row(
            i18n.tr("metadata-label-format"),
            format.clone(),
        ));
    }

    build_section(
        icons::image(),
        i18n.tr("metadata-section-file"),
        rows.into(),
    )
}

fn build_file_section_video<'a>(
    i18n: &'a I18n,
    meta: &ExtendedVideoMetadata,
) -> Element<'a, Message> {
    let mut rows = Column::new().spacing(spacing::XS);

    rows = rows.push(build_metadata_row(
        i18n.tr("metadata-label-dimensions"),
        format!("{} x {} px", meta.width, meta.height),
    ));

    if let Some(size) = meta.file_size {
        rows = rows.push(build_metadata_row(
            i18n.tr("metadata-label-file-size"),
            format_file_size(size),
        ));
    }

    rows = rows.push(build_metadata_row(
        i18n.tr("metadata-label-duration"),
        format_duration(meta.duration_secs),
    ));

    if meta.fps > 0.0 {
        rows = rows.push(build_metadata_row(
            i18n.tr("metadata-label-fps"),
            format!("{:.2} fps", meta.fps),
        ));
    }

    if let Some(ref format) = meta.container_format {
        rows = rows.push(build_metadata_row(
            i18n.tr("metadata-label-format"),
            format.to_uppercase(),
        ));
    }

    build_section(
        icons::video_camera(),
        i18n.tr("metadata-section-file"),
        rows.into(),
    )
}

fn build_camera_section_view<'a>(i18n: &'a I18n, meta: &ImageMetadata) -> Element<'a, Message> {
    let mut rows = Column::new().spacing(spacing::XS);

    if meta.camera_make.is_some() || meta.camera_model.is_some() {
        let camera = match (&meta.camera_make, &meta.camera_model) {
            (Some(make), Some(model)) => format!("{} {}", make, model),
            (Some(make), None) => make.clone(),
            (None, Some(model)) => model.clone(),
            _ => i18n.tr("metadata-value-unknown"),
        };
        rows = rows.push(build_metadata_row(i18n.tr("metadata-label-camera"), camera));
    }

    if let Some(ref date) = meta.date_taken {
        rows = rows.push(build_metadata_row(
            i18n.tr("metadata-label-date-taken"),
            date.clone(),
        ));
    }

    build_section(
        icons::camera(),
        i18n.tr("metadata-section-camera"),
        rows.into(),
    )
}

fn build_exposure_section_view<'a>(i18n: &'a I18n, meta: &ImageMetadata) -> Element<'a, Message> {
    let mut rows = Column::new().spacing(spacing::XS);

    if let Some(ref exposure) = meta.exposure_time {
        rows = rows.push(build_metadata_row(
            i18n.tr("metadata-label-exposure"),
            exposure.clone(),
        ));
    }

    if let Some(ref aperture) = meta.aperture {
        rows = rows.push(build_metadata_row(
            i18n.tr("metadata-label-aperture"),
            aperture.clone(),
        ));
    }

    if let Some(ref iso) = meta.iso {
        rows = rows.push(build_metadata_row(
            i18n.tr("metadata-label-iso"),
            iso.clone(),
        ));
    }

    if let Some(ref focal) = meta.focal_length {
        let focal_str = if let Some(ref focal_35) = meta.focal_length_35mm {
            format!("{} ({})", focal, focal_35)
        } else {
            focal.clone()
        };
        rows = rows.push(build_metadata_row(
            i18n.tr("metadata-label-focal-length"),
            focal_str,
        ));
    }

    build_section(
        icons::cog(),
        i18n.tr("metadata-section-exposure"),
        rows.into(),
    )
}

fn build_gps_section_view<'a>(i18n: &'a I18n, meta: &ImageMetadata) -> Element<'a, Message> {
    let mut rows = Column::new().spacing(spacing::XS);

    if let (Some(lat), Some(lon)) = (meta.gps_latitude, meta.gps_longitude) {
        rows = rows.push(build_metadata_row(
            i18n.tr("metadata-label-gps"),
            format_gps_coordinates(lat, lon),
        ));
    }

    build_section(icons::globe(), i18n.tr("metadata-section-gps"), rows.into())
}

fn build_dublin_core_section_view<'a>(
    i18n: &'a I18n,
    meta: &ImageMetadata,
) -> Element<'a, Message> {
    let mut rows = Column::new().spacing(spacing::XS);

    if let Some(ref title) = meta.dc_title {
        rows = rows.push(build_metadata_row(
            i18n.tr("metadata-label-dc-title"),
            title.clone(),
        ));
    }

    if let Some(ref creator) = meta.dc_creator {
        rows = rows.push(build_metadata_row(
            i18n.tr("metadata-label-dc-creator"),
            creator.clone(),
        ));
    }

    if let Some(ref description) = meta.dc_description {
        rows = rows.push(build_metadata_row(
            i18n.tr("metadata-label-dc-description"),
            description.clone(),
        ));
    }

    if let Some(ref subject) = meta.dc_subject {
        if !subject.is_empty() {
            rows = rows.push(build_metadata_row(
                i18n.tr("metadata-label-dc-subject"),
                subject.join(", "),
            ));
        }
    }

    if let Some(ref rights) = meta.dc_rights {
        rows = rows.push(build_metadata_row(
            i18n.tr("metadata-label-dc-rights"),
            rights.clone(),
        ));
    }

    build_section(
        icons::info(),
        i18n.tr("metadata-section-dublin-core"),
        rows.into(),
    )
}

fn build_video_codec_section<'a>(
    i18n: &'a I18n,
    meta: &ExtendedVideoMetadata,
) -> Element<'a, Message> {
    let mut rows = Column::new().spacing(spacing::XS);

    if let Some(ref codec) = meta.video_codec {
        rows = rows.push(build_metadata_row(
            i18n.tr("metadata-label-codec"),
            codec.to_uppercase(),
        ));
    }

    if let Some(bitrate) = meta.video_bitrate {
        rows = rows.push(build_metadata_row(
            i18n.tr("metadata-label-bitrate"),
            format_bitrate(bitrate),
        ));
    }

    build_section(
        icons::video_camera(),
        i18n.tr("metadata-section-video"),
        rows.into(),
    )
}

fn build_audio_section<'a>(i18n: &'a I18n, meta: &ExtendedVideoMetadata) -> Element<'a, Message> {
    let mut rows = Column::new().spacing(spacing::XS);

    if let Some(ref codec) = meta.audio_codec {
        rows = rows.push(build_metadata_row(
            i18n.tr("metadata-label-codec"),
            codec.to_uppercase(),
        ));
    }

    if let Some(bitrate) = meta.audio_bitrate {
        rows = rows.push(build_metadata_row(
            i18n.tr("metadata-label-bitrate"),
            format_bitrate(bitrate),
        ));
    }

    build_section(
        icons::volume(),
        i18n.tr("metadata-section-audio"),
        rows.into(),
    )
}

// =============================================================================
// Helper Functions
// =============================================================================

fn build_metadata_row<'a>(label: String, value: String) -> Element<'a, Message> {
    Row::new()
        .spacing(spacing::SM)
        .push(
            Text::new(format!("{}:", label))
                .size(typography::BODY)
                .width(Length::FillPortion(2)),
        )
        .push(
            Text::new(value)
                .size(typography::BODY)
                .width(Length::FillPortion(3)),
        )
        .into()
}

fn build_section<'a>(
    icon: Image<Handle>,
    title: String,
    content: Element<'a, Message>,
) -> Element<'a, Message> {
    let icon_sized = icons::sized(icon, sizing::ICON_SM);

    let header = Row::new()
        .spacing(spacing::XS)
        .align_y(Vertical::Center)
        .push(icon_sized)
        .push(Text::new(title).size(typography::BODY_LG));

    Column::new()
        .spacing(spacing::XS)
        .push(header)
        .push(content)
        .into()
}

fn format_duration(duration_secs: f64) -> String {
    let total_secs = duration_secs as u64;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_duration_formats_correctly() {
        assert_eq!(format_duration(0.0), "00:00");
        assert_eq!(format_duration(65.0), "01:05");
        assert_eq!(format_duration(3665.0), "01:01:05");
    }
}
