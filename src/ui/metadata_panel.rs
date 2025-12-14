// SPDX-License-Identifier: MPL-2.0
//! Metadata panel component displaying technical information about media files.
//!
//! This module renders a sidebar panel showing EXIF data for images (camera settings,
//! GPS coordinates, etc.) and codec/format information for videos.

use crate::i18n::fluent::I18n;
use crate::media::metadata::{
    format_bitrate, format_file_size, format_gps_coordinates, ExtendedVideoMetadata, ImageMetadata,
    MediaMetadata,
};
use crate::ui::design_tokens::{radius, sizing, spacing, typography};
use crate::ui::icons;
use crate::ui::styles;
use iced::{
    alignment::Vertical,
    widget::{container, rule, scrollable, Column, Container, Row, Text},
    Border, Element, Length, Theme,
};

/// Width of the metadata panel in pixels.
pub const PANEL_WIDTH: f32 = 290.0;

/// Contextual data needed to render the metadata panel.
pub struct ViewContext<'a> {
    pub i18n: &'a I18n,
    pub metadata: Option<&'a MediaMetadata>,
}

/// Messages emitted by the metadata panel.
#[derive(Debug, Clone)]
pub enum Message {
    Close,
}

/// Events propagated to the parent application.
#[derive(Debug, Clone)]
pub enum Event {
    None,
    Close,
}

/// Process a metadata panel message and return the corresponding event.
pub fn update(message: Message) -> Event {
    match message {
        Message::Close => Event::Close,
    }
}

/// Render the metadata panel.
pub fn view<'a>(ctx: ViewContext<'a>) -> Element<'a, Message> {
    let title = Text::new(ctx.i18n.tr("metadata-panel-title")).size(typography::TITLE_SM);

    let content = if let Some(metadata) = ctx.metadata {
        match metadata {
            MediaMetadata::Image(image_meta) => build_image_metadata(&ctx, image_meta),
            MediaMetadata::Video(video_meta) => build_video_metadata(&ctx, video_meta),
        }
    } else {
        Column::new()
            .push(Text::new(ctx.i18n.tr("metadata-value-unknown")).size(typography::BODY))
            .into()
    };

    let panel_content = Column::new()
        .width(Length::Fill)
        .spacing(spacing::MD)
        .padding(spacing::MD)
        .push(title)
        .push(rule::horizontal(1))
        .push(content);

    let scrollable_content = scrollable(panel_content).width(Length::Fixed(PANEL_WIDTH));

    Container::new(scrollable_content)
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

/// Build metadata display for images.
fn build_image_metadata<'a>(ctx: &ViewContext<'a>, meta: &ImageMetadata) -> Element<'a, Message> {
    let mut sections = Column::new().spacing(spacing::MD);

    // File section
    let file_section = build_file_section_image(ctx, meta);
    sections = sections.push(file_section);

    // Camera section (if available)
    if meta.camera_make.is_some() || meta.camera_model.is_some() || meta.date_taken.is_some() {
        let camera_section = build_camera_section(ctx, meta);
        sections = sections.push(camera_section);
    }

    // Exposure section (if available)
    if meta.exposure_time.is_some()
        || meta.aperture.is_some()
        || meta.iso.is_some()
        || meta.flash.is_some()
        || meta.focal_length.is_some()
    {
        let exposure_section = build_exposure_section(ctx, meta);
        sections = sections.push(exposure_section);
    }

    // GPS section (if available)
    if meta.gps_latitude.is_some() && meta.gps_longitude.is_some() {
        let gps_section = build_gps_section(ctx, meta);
        sections = sections.push(gps_section);
    }

    sections.into()
}

/// Build metadata display for videos.
fn build_video_metadata<'a>(
    ctx: &ViewContext<'a>,
    meta: &ExtendedVideoMetadata,
) -> Element<'a, Message> {
    let mut sections = Column::new().spacing(spacing::MD);

    // File section
    let file_section = build_file_section_video(ctx, meta);
    sections = sections.push(file_section);

    // Video section
    let video_section = build_video_codec_section(ctx, meta);
    sections = sections.push(video_section);

    // Audio section (if available)
    if meta.has_audio {
        let audio_section = build_audio_section(ctx, meta);
        sections = sections.push(audio_section);
    }

    sections.into()
}

/// Build file info section for images.
fn build_file_section_image<'a>(
    ctx: &ViewContext<'a>,
    meta: &ImageMetadata,
) -> Element<'a, Message> {
    let mut rows = Column::new().spacing(spacing::XS);

    // Dimensions
    if meta.width.is_some() || meta.height.is_some() {
        let dims = format!(
            "{} x {} px",
            meta.width
                .map_or_else(|| "?".to_string(), |v| v.to_string()),
            meta.height
                .map_or_else(|| "?".to_string(), |v| v.to_string())
        );
        rows = rows.push(build_metadata_row(
            ctx.i18n.tr("metadata-label-dimensions"),
            dims,
        ));
    }

    // File size
    if let Some(size) = meta.file_size {
        rows = rows.push(build_metadata_row(
            ctx.i18n.tr("metadata-label-file-size"),
            format_file_size(size),
        ));
    }

    // Format
    if let Some(ref format) = meta.format {
        rows = rows.push(build_metadata_row(
            ctx.i18n.tr("metadata-label-format"),
            format.clone(),
        ));
    }

    build_section(
        icons::image(),
        ctx.i18n.tr("metadata-section-file"),
        rows.into(),
    )
}

/// Build file info section for videos.
fn build_file_section_video<'a>(
    ctx: &ViewContext<'a>,
    meta: &ExtendedVideoMetadata,
) -> Element<'a, Message> {
    let mut rows = Column::new().spacing(spacing::XS);

    // Dimensions
    rows = rows.push(build_metadata_row(
        ctx.i18n.tr("metadata-label-dimensions"),
        format!("{} x {} px", meta.width, meta.height),
    ));

    // File size
    if let Some(size) = meta.file_size {
        rows = rows.push(build_metadata_row(
            ctx.i18n.tr("metadata-label-file-size"),
            format_file_size(size),
        ));
    }

    // Duration
    rows = rows.push(build_metadata_row(
        ctx.i18n.tr("metadata-label-duration"),
        format_duration(meta.duration_secs),
    ));

    // FPS
    if meta.fps > 0.0 {
        rows = rows.push(build_metadata_row(
            ctx.i18n.tr("metadata-label-fps"),
            format!("{:.2} fps", meta.fps),
        ));
    }

    // Container format
    if let Some(ref format) = meta.container_format {
        rows = rows.push(build_metadata_row(
            ctx.i18n.tr("metadata-label-format"),
            format.to_uppercase(),
        ));
    }

    build_section(
        icons::video_camera(),
        ctx.i18n.tr("metadata-section-file"),
        rows.into(),
    )
}

/// Build camera info section.
fn build_camera_section<'a>(ctx: &ViewContext<'a>, meta: &ImageMetadata) -> Element<'a, Message> {
    let mut rows = Column::new().spacing(spacing::XS);

    // Camera make and model
    if meta.camera_make.is_some() || meta.camera_model.is_some() {
        let camera = match (&meta.camera_make, &meta.camera_model) {
            (Some(make), Some(model)) => format!("{} {}", make, model),
            (Some(make), None) => make.clone(),
            (None, Some(model)) => model.clone(),
            _ => ctx.i18n.tr("metadata-value-unknown"),
        };
        rows = rows.push(build_metadata_row(
            ctx.i18n.tr("metadata-label-camera"),
            camera,
        ));
    }

    // Date taken
    if let Some(ref date) = meta.date_taken {
        rows = rows.push(build_metadata_row(
            ctx.i18n.tr("metadata-label-date-taken"),
            date.clone(),
        ));
    }

    build_section(
        icons::camera(),
        ctx.i18n.tr("metadata-section-camera"),
        rows.into(),
    )
}

/// Build exposure settings section.
fn build_exposure_section<'a>(ctx: &ViewContext<'a>, meta: &ImageMetadata) -> Element<'a, Message> {
    let mut rows = Column::new().spacing(spacing::XS);

    // Exposure time
    if let Some(ref exposure) = meta.exposure_time {
        rows = rows.push(build_metadata_row(
            ctx.i18n.tr("metadata-label-exposure"),
            exposure.clone(),
        ));
    }

    // Aperture
    if let Some(ref aperture) = meta.aperture {
        rows = rows.push(build_metadata_row(
            ctx.i18n.tr("metadata-label-aperture"),
            aperture.clone(),
        ));
    }

    // ISO
    if let Some(ref iso) = meta.iso {
        rows = rows.push(build_metadata_row(
            ctx.i18n.tr("metadata-label-iso"),
            iso.clone(),
        ));
    }

    // Focal length
    if let Some(ref focal) = meta.focal_length {
        let focal_str = if let Some(ref focal_35) = meta.focal_length_35mm {
            format!("{} ({})", focal, focal_35)
        } else {
            focal.clone()
        };
        rows = rows.push(build_metadata_row(
            ctx.i18n.tr("metadata-label-focal-length"),
            focal_str,
        ));
    }

    build_section(
        icons::cog(),
        ctx.i18n.tr("metadata-section-exposure"),
        rows.into(),
    )
}

/// Build GPS section.
fn build_gps_section<'a>(ctx: &ViewContext<'a>, meta: &ImageMetadata) -> Element<'a, Message> {
    let mut rows = Column::new().spacing(spacing::XS);

    if let (Some(lat), Some(lon)) = (meta.gps_latitude, meta.gps_longitude) {
        rows = rows.push(build_metadata_row(
            ctx.i18n.tr("metadata-label-gps"),
            format_gps_coordinates(lat, lon),
        ));
    }

    build_section(
        icons::globe(),
        ctx.i18n.tr("metadata-section-gps"),
        rows.into(),
    )
}

/// Build video codec section.
fn build_video_codec_section<'a>(
    ctx: &ViewContext<'a>,
    meta: &ExtendedVideoMetadata,
) -> Element<'a, Message> {
    let mut rows = Column::new().spacing(spacing::XS);

    // Video codec
    if let Some(ref codec) = meta.video_codec {
        rows = rows.push(build_metadata_row(
            ctx.i18n.tr("metadata-label-codec"),
            codec.to_uppercase(),
        ));
    }

    // Video bitrate
    if let Some(bitrate) = meta.video_bitrate {
        rows = rows.push(build_metadata_row(
            ctx.i18n.tr("metadata-label-bitrate"),
            format_bitrate(bitrate),
        ));
    }

    build_section(
        icons::video_camera(),
        ctx.i18n.tr("metadata-section-video"),
        rows.into(),
    )
}

/// Build audio section.
fn build_audio_section<'a>(
    ctx: &ViewContext<'a>,
    meta: &ExtendedVideoMetadata,
) -> Element<'a, Message> {
    let mut rows = Column::new().spacing(spacing::XS);

    // Audio codec
    if let Some(ref codec) = meta.audio_codec {
        rows = rows.push(build_metadata_row(
            ctx.i18n.tr("metadata-label-codec"),
            codec.to_uppercase(),
        ));
    }

    // Audio bitrate
    if let Some(bitrate) = meta.audio_bitrate {
        rows = rows.push(build_metadata_row(
            ctx.i18n.tr("metadata-label-bitrate"),
            format_bitrate(bitrate),
        ));
    }

    build_section(
        icons::volume(),
        ctx.i18n.tr("metadata-section-audio"),
        rows.into(),
    )
}

/// Build a single metadata row with label and value.
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

/// Build a section with icon, title, and content.
fn build_section<'a>(
    icon: iced::widget::Svg<'a>,
    title: String,
    content: Element<'a, Message>,
) -> Element<'a, Message> {
    let icon_sized = icons::sized(icon, sizing::ICON_SM).style(styles::tinted_svg);

    let header = Row::new()
        .spacing(spacing::XS)
        .align_y(Vertical::Center)
        .push(icon_sized)
        .push(Text::new(title).size(typography::BODY_LG));

    let inner = Column::new()
        .spacing(spacing::XS)
        .push(header)
        .push(content);

    inner.into()
}

/// Format duration in HH:MM:SS or MM:SS format.
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
    use crate::i18n::fluent::I18n;

    #[test]
    fn metadata_panel_view_renders_without_metadata() {
        let i18n = I18n::default();
        let ctx = ViewContext {
            i18n: &i18n,
            metadata: None,
        };
        let _element = view(ctx);
    }

    #[test]
    fn metadata_panel_view_renders_with_image_metadata() {
        let i18n = I18n::default();
        let image_meta = ImageMetadata {
            width: Some(1920),
            height: Some(1080),
            file_size: Some(1024 * 1024),
            format: Some("JPEG".to_string()),
            camera_make: Some("Canon".to_string()),
            camera_model: Some("EOS 5D".to_string()),
            ..Default::default()
        };
        let media_meta = MediaMetadata::Image(image_meta);
        let ctx = ViewContext {
            i18n: &i18n,
            metadata: Some(&media_meta),
        };
        let _element = view(ctx);
    }

    #[test]
    fn metadata_panel_view_renders_with_video_metadata() {
        let i18n = I18n::default();
        let video_meta = ExtendedVideoMetadata {
            width: 1920,
            height: 1080,
            duration_secs: 125.5,
            fps: 30.0,
            has_audio: true,
            video_codec: Some("h264".to_string()),
            audio_codec: Some("aac".to_string()),
            ..Default::default()
        };
        let media_meta = MediaMetadata::Video(video_meta);
        let ctx = ViewContext {
            i18n: &i18n,
            metadata: Some(&media_meta),
        };
        let _element = view(ctx);
    }

    #[test]
    fn format_duration_formats_correctly() {
        assert_eq!(format_duration(0.0), "00:00");
        assert_eq!(format_duration(65.0), "01:05");
        assert_eq!(format_duration(3665.0), "01:01:05");
    }

    #[test]
    fn close_message_emits_event() {
        let event = update(Message::Close);
        assert!(matches!(event, Event::Close));
    }
}
