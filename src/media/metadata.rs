// SPDX-License-Identifier: MPL-2.0
//! Media metadata extraction for images (EXIF) and videos (FFmpeg).
//!
//! This module provides unified metadata extraction for both images and videos,
//! extracting technical information like dimensions, camera settings, GPS coordinates,
//! and video codec details.

use crate::error::{Error, Result};
use crate::media::xmp;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;

/// Image metadata extracted from EXIF and XMP data.
#[derive(Debug, Clone, Default)]
pub struct ImageMetadata {
    // File info
    /// Image width in pixels
    pub width: Option<u32>,
    /// Image height in pixels
    pub height: Option<u32>,
    /// File size in bytes
    pub file_size: Option<u64>,
    /// Image format (e.g., "JPEG", "PNG")
    pub format: Option<String>,

    // Camera info (EXIF)
    /// Camera manufacturer (e.g., "Canon", "Nikon")
    pub camera_make: Option<String>,
    /// Camera model (e.g., "EOS 5D Mark IV")
    pub camera_model: Option<String>,

    // Date info (EXIF)
    /// Date and time the photo was taken
    pub date_taken: Option<String>,

    // Exposure info (EXIF)
    /// Exposure time (e.g., "1/250 sec")
    pub exposure_time: Option<String>,
    /// Aperture f-number (e.g., "f/2.8")
    pub aperture: Option<String>,
    /// ISO sensitivity (e.g., "100")
    pub iso: Option<String>,
    /// Flash status (e.g., "Flash fired")
    pub flash: Option<String>,

    // Lens info (EXIF)
    /// Focal length in mm (e.g., "50 mm")
    pub focal_length: Option<String>,
    /// Focal length equivalent to 35mm film
    pub focal_length_35mm: Option<String>,

    // GPS info (EXIF)
    /// Latitude in decimal degrees (e.g., 48.8566)
    pub gps_latitude: Option<f64>,
    /// Longitude in decimal degrees (e.g., 2.3522)
    pub gps_longitude: Option<f64>,

    // Dublin Core / XMP metadata
    /// dc:title - Title of the work
    pub dc_title: Option<String>,
    /// dc:creator - Creator/author of the work
    pub dc_creator: Option<String>,
    /// dc:description - Description of the work
    pub dc_description: Option<String>,
    /// dc:subject - Keywords/tags (comma-separated when displayed)
    pub dc_subject: Option<Vec<String>>,
    /// dc:rights - Copyright or license information
    pub dc_rights: Option<String>,
}

/// Extended video metadata with codec and format information.
#[derive(Debug, Clone, Default)]
pub struct ExtendedVideoMetadata {
    // Basic info (from existing VideoMetadata)
    /// Video width in pixels
    pub width: u32,
    /// Video height in pixels
    pub height: u32,
    /// Duration in seconds
    pub duration_secs: f64,
    /// Frames per second
    pub fps: f64,
    /// Whether the video has an audio track
    pub has_audio: bool,

    // Extended info
    /// Video codec name (e.g., "H.264", "VP9")
    pub video_codec: Option<String>,
    /// Audio codec name (e.g., "AAC", "MP3")
    pub audio_codec: Option<String>,
    /// Container format (e.g., "MP4", "MKV")
    pub container_format: Option<String>,
    /// Video bitrate in bits per second
    pub video_bitrate: Option<u64>,
    /// Audio bitrate in bits per second
    pub audio_bitrate: Option<u64>,
    /// File size in bytes
    pub file_size: Option<u64>,
}

/// Unified metadata enum for both images and videos.
#[derive(Debug, Clone)]
pub enum MediaMetadata {
    /// Image metadata (boxed to reduce enum size variance)
    Image(Box<ImageMetadata>),
    Video(ExtendedVideoMetadata),
}

impl MediaMetadata {
    /// Returns file size if available.
    pub fn file_size(&self) -> Option<u64> {
        match self {
            MediaMetadata::Image(m) => m.file_size,
            MediaMetadata::Video(m) => m.file_size,
        }
    }

    /// Returns dimensions (width, height).
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            MediaMetadata::Image(m) => (m.width.unwrap_or(0), m.height.unwrap_or(0)),
            MediaMetadata::Video(m) => (m.width, m.height),
        }
    }
}

/// Extract metadata from an image file.
///
/// Reads EXIF data from JPEG, PNG, WebP, TIFF, and HEIF files.
/// Returns default metadata if EXIF data is not available.
pub fn extract_image_metadata<P: AsRef<Path>>(path: P) -> Result<ImageMetadata> {
    let path = path.as_ref();
    let mut metadata = ImageMetadata::default();

    // Get file size
    if let Ok(fs_metadata) = fs::metadata(path) {
        metadata.file_size = Some(fs_metadata.len());
    }

    // Detect format from extension
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        metadata.format = Some(ext.to_uppercase());
    }

    // Try to read EXIF data
    let file = File::open(path).map_err(|e| Error::Io(e.to_string()))?;
    let mut reader = BufReader::new(file);

    let exif_reader = exif::Reader::new();
    if let Ok(exif) = exif_reader.read_from_container(&mut reader) {
        // Image dimensions
        if let Some(field) = exif.get_field(exif::Tag::PixelXDimension, exif::In::PRIMARY) {
            metadata.width = field.value.get_uint(0);
        } else if let Some(field) = exif.get_field(exif::Tag::ImageWidth, exif::In::PRIMARY) {
            metadata.width = field.value.get_uint(0);
        }

        if let Some(field) = exif.get_field(exif::Tag::PixelYDimension, exif::In::PRIMARY) {
            metadata.height = field.value.get_uint(0);
        } else if let Some(field) = exif.get_field(exif::Tag::ImageLength, exif::In::PRIMARY) {
            metadata.height = field.value.get_uint(0);
        }

        // Camera info
        if let Some(field) = exif.get_field(exif::Tag::Make, exif::In::PRIMARY) {
            metadata.camera_make = Some(
                field
                    .display_value()
                    .to_string()
                    .trim_matches('"')
                    .to_string(),
            );
        }

        if let Some(field) = exif.get_field(exif::Tag::Model, exif::In::PRIMARY) {
            metadata.camera_model = Some(
                field
                    .display_value()
                    .to_string()
                    .trim_matches('"')
                    .to_string(),
            );
        }

        // Date taken
        if let Some(field) = exif.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY) {
            metadata.date_taken = Some(
                field
                    .display_value()
                    .to_string()
                    .trim_matches('"')
                    .to_string(),
            );
        } else if let Some(field) = exif.get_field(exif::Tag::DateTime, exif::In::PRIMARY) {
            metadata.date_taken = Some(
                field
                    .display_value()
                    .to_string()
                    .trim_matches('"')
                    .to_string(),
            );
        }

        // Exposure settings
        if let Some(field) = exif.get_field(exif::Tag::ExposureTime, exif::In::PRIMARY) {
            metadata.exposure_time = Some(format!("{} sec", field.display_value()));
        }

        if let Some(field) = exif.get_field(exif::Tag::FNumber, exif::In::PRIMARY) {
            metadata.aperture = Some(field.display_value().to_string());
        }

        if let Some(field) = exif.get_field(exif::Tag::PhotographicSensitivity, exif::In::PRIMARY) {
            metadata.iso = Some(format!("ISO {}", field.display_value()));
        }

        if let Some(field) = exif.get_field(exif::Tag::Flash, exif::In::PRIMARY) {
            metadata.flash = Some(field.display_value().to_string());
        }

        // Lens info
        if let Some(field) = exif.get_field(exif::Tag::FocalLength, exif::In::PRIMARY) {
            metadata.focal_length = Some(field.display_value().to_string());
        }

        if let Some(field) = exif.get_field(exif::Tag::FocalLengthIn35mmFilm, exif::In::PRIMARY) {
            metadata.focal_length_35mm = Some(format!("{} mm", field.display_value()));
        }

        // GPS coordinates
        extract_gps_coordinates(&exif, &mut metadata);
    }

    // Try to extract XMP Dublin Core metadata (JPEG only for now)
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        if ext.eq_ignore_ascii_case("jpg") || ext.eq_ignore_ascii_case("jpeg") {
            if let Some(dc) = xmp::extract_xmp_from_jpeg(path) {
                metadata.dc_title = dc.title;
                metadata.dc_creator = dc.creator;
                metadata.dc_description = dc.description;
                metadata.dc_subject = dc.subject;
                metadata.dc_rights = dc.rights;
            }
        }
    }

    Ok(metadata)
}

/// Extract GPS coordinates from EXIF data.
fn extract_gps_coordinates(exif: &exif::Exif, metadata: &mut ImageMetadata) {
    // Get latitude
    if let (Some(lat_field), Some(lat_ref_field)) = (
        exif.get_field(exif::Tag::GPSLatitude, exif::In::PRIMARY),
        exif.get_field(exif::Tag::GPSLatitudeRef, exif::In::PRIMARY),
    ) {
        if let Some(lat) = parse_gps_coordinate(&lat_field.value) {
            let lat_ref = lat_ref_field.display_value().to_string();
            metadata.gps_latitude = Some(if lat_ref.contains('S') { -lat } else { lat });
        }
    }

    // Get longitude
    if let (Some(lon_field), Some(lon_ref_field)) = (
        exif.get_field(exif::Tag::GPSLongitude, exif::In::PRIMARY),
        exif.get_field(exif::Tag::GPSLongitudeRef, exif::In::PRIMARY),
    ) {
        if let Some(lon) = parse_gps_coordinate(&lon_field.value) {
            let lon_ref = lon_ref_field.display_value().to_string();
            metadata.gps_longitude = Some(if lon_ref.contains('W') { -lon } else { lon });
        }
    }
}

/// Parse GPS coordinate from EXIF rational values (degrees, minutes, seconds).
fn parse_gps_coordinate(value: &exif::Value) -> Option<f64> {
    match value {
        exif::Value::Rational(rationals) if rationals.len() >= 3 => {
            let degrees = rationals[0].to_f64();
            let minutes = rationals[1].to_f64();
            let seconds = rationals[2].to_f64();
            Some(degrees + minutes / 60.0 + seconds / 3600.0)
        }
        _ => None,
    }
}

/// Extract extended metadata from a video file using FFmpeg.
///
/// Extends the basic VideoMetadata with codec names, container format, and bitrates.
pub fn extract_extended_video_metadata<P: AsRef<Path>>(path: P) -> Result<ExtendedVideoMetadata> {
    use crate::media::video::init_ffmpeg;

    let path = path.as_ref();
    let mut metadata = ExtendedVideoMetadata::default();

    // Get file size
    if let Ok(fs_metadata) = fs::metadata(path) {
        metadata.file_size = Some(fs_metadata.len());
    }

    // Initialize FFmpeg
    init_ffmpeg()?;

    // Open video file
    let ictx = ffmpeg_next::format::input(path)
        .map_err(|e| Error::Io(format!("Failed to open video file: {e}")))?;

    // Get container format
    metadata.container_format = Some(ictx.format().name().to_string());

    // Find video stream
    if let Some(video_stream) = ictx.streams().best(ffmpeg_next::media::Type::Video) {
        // Create decoder context to get dimensions
        if let Ok(context_decoder) =
            ffmpeg_next::codec::context::Context::from_parameters(video_stream.parameters())
        {
            if let Ok(decoder) = context_decoder.decoder().video() {
                metadata.width = decoder.width();
                metadata.height = decoder.height();

                // Get video codec name
                metadata.video_codec = Some(
                    decoder
                        .codec()
                        .map_or_else(|| "Unknown".to_string(), |c| c.name().to_string()),
                );
            }
        }

        // Extract duration
        let duration_secs = if video_stream.duration() > 0 {
            let time_base = video_stream.time_base();
            video_stream.duration() as f64 * f64::from(time_base.numerator())
                / f64::from(time_base.denominator())
        } else if ictx.duration() > 0 {
            ictx.duration() as f64 / f64::from(ffmpeg_next::ffi::AV_TIME_BASE)
        } else {
            0.0
        };
        metadata.duration_secs = duration_secs;

        // Extract FPS
        let frame_rate = video_stream.avg_frame_rate();
        if frame_rate.denominator() != 0 {
            metadata.fps = f64::from(frame_rate.numerator()) / f64::from(frame_rate.denominator());
        }

        // Get video bitrate from stream or container
        let params = video_stream.parameters();
        let bit_rate = unsafe { (*params.as_ptr()).bit_rate };
        if bit_rate > 0 {
            metadata.video_bitrate = Some(bit_rate as u64);
        }
    }

    // Find audio stream
    if let Some(audio_stream) = ictx.streams().best(ffmpeg_next::media::Type::Audio) {
        metadata.has_audio = true;

        // Get audio codec name
        if let Ok(context_decoder) =
            ffmpeg_next::codec::context::Context::from_parameters(audio_stream.parameters())
        {
            if let Ok(decoder) = context_decoder.decoder().audio() {
                metadata.audio_codec = Some(
                    decoder
                        .codec()
                        .map_or_else(|| "Unknown".to_string(), |c| c.name().to_string()),
                );
            }
        }

        // Get audio bitrate
        let params = audio_stream.parameters();
        let bit_rate = unsafe { (*params.as_ptr()).bit_rate };
        if bit_rate > 0 {
            metadata.audio_bitrate = Some(bit_rate as u64);
        }
    }

    Ok(metadata)
}

/// Format file size in human-readable format.
pub fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

/// Format bitrate in human-readable format.
pub fn format_bitrate(bits_per_sec: u64) -> String {
    const KBPS: u64 = 1000;
    const MBPS: u64 = KBPS * 1000;

    if bits_per_sec >= MBPS {
        format!("{:.2} Mbps", bits_per_sec as f64 / MBPS as f64)
    } else if bits_per_sec >= KBPS {
        format!("{:.0} kbps", bits_per_sec as f64 / KBPS as f64)
    } else {
        format!("{bits_per_sec} bps")
    }
}

/// Format GPS coordinates as decimal degrees string.
pub fn format_gps_coordinates(lat: f64, lon: f64) -> String {
    let lat_dir = if lat >= 0.0 { "N" } else { "S" };
    let lon_dir = if lon >= 0.0 { "E" } else { "W" };
    format!(
        "{:.6}° {}, {:.6}° {}",
        lat.abs(),
        lat_dir,
        lon.abs(),
        lon_dir
    )
}

/// Extract metadata from a media file, automatically detecting the media type.
///
/// Uses file extension to determine whether to extract image or video metadata.
/// Returns `None` if extraction fails or the file type is not supported.
pub fn extract_metadata<P: AsRef<Path>>(path: P) -> Option<MediaMetadata> {
    let path = path.as_ref();
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())?;

    // Check if it's a video file
    if matches!(
        ext.as_str(),
        "mp4" | "mkv" | "avi" | "mov" | "webm" | "m4v" | "wmv" | "flv"
    ) {
        extract_extended_video_metadata(path)
            .ok()
            .map(MediaMetadata::Video)
    } else if matches!(
        ext.as_str(),
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "tiff" | "tif" | "heic" | "heif" | "svg"
    ) {
        extract_image_metadata(path)
            .ok()
            .map(|m| MediaMetadata::Image(Box::new(m)))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_file_size_formats_correctly() {
        assert_eq!(format_file_size(500), "500 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1_048_576), "1.00 MB");
        assert_eq!(format_file_size(1_572_864), "1.50 MB");
        assert_eq!(format_file_size(1_073_741_824), "1.00 GB");
    }

    #[test]
    fn format_bitrate_formats_correctly() {
        assert_eq!(format_bitrate(500), "500 bps");
        assert_eq!(format_bitrate(1000), "1 kbps");
        assert_eq!(format_bitrate(128_000), "128 kbps");
        assert_eq!(format_bitrate(1_000_000), "1.00 Mbps");
        assert_eq!(format_bitrate(5_000_000), "5.00 Mbps");
    }

    #[test]
    fn format_gps_coordinates_formats_correctly() {
        assert_eq!(
            format_gps_coordinates(48.8566, 2.3522),
            "48.856600° N, 2.352200° E"
        );
        assert_eq!(
            format_gps_coordinates(-33.8688, 151.2093),
            "33.868800° S, 151.209300° E"
        );
        assert_eq!(
            format_gps_coordinates(40.7128, -74.0060),
            "40.712800° N, 74.006000° W"
        );
    }

    #[test]
    fn image_metadata_has_defaults() {
        let metadata = ImageMetadata::default();
        assert!(metadata.width.is_none());
        assert!(metadata.camera_make.is_none());
        assert!(metadata.gps_latitude.is_none());
    }

    #[test]
    fn extended_video_metadata_has_defaults() {
        let metadata = ExtendedVideoMetadata::default();
        assert_eq!(metadata.width, 0);
        assert_eq!(metadata.duration_secs, 0.0);
        assert!(metadata.video_codec.is_none());
    }

    #[test]
    fn media_metadata_dimensions() {
        let image_meta = ImageMetadata {
            width: Some(1920),
            height: Some(1080),
            ..Default::default()
        };
        let media = MediaMetadata::Image(Box::new(image_meta));
        assert_eq!(media.dimensions(), (1920, 1080));

        let video_meta = ExtendedVideoMetadata {
            width: 3840,
            height: 2160,
            ..Default::default()
        };
        let media = MediaMetadata::Video(video_meta);
        assert_eq!(media.dimensions(), (3840, 2160));
    }

    #[test]
    fn extract_image_metadata_handles_missing_file() {
        let result = extract_image_metadata("/nonexistent/path/image.jpg");
        assert!(result.is_err());
    }

    #[test]
    fn extract_image_metadata_works_for_file_without_exif() {
        // Create a temp file with no EXIF data
        use std::io::Write;
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let path = temp_dir.path().join("test.txt");
        let mut file = File::create(&path).expect("create file");
        writeln!(file, "not an image").expect("write");

        // Should succeed but with minimal metadata (just file size)
        let result = extract_image_metadata(&path);
        assert!(result.is_ok());
        let metadata = result.unwrap();
        assert!(metadata.file_size.is_some());
        assert!(metadata.camera_make.is_none());
    }
}
