// SPDX-License-Identifier: MPL-2.0
//! EXIF metadata writing for image files.
//!
//! This module provides functionality to write EXIF metadata to image files
//! using the `little_exif` crate. It supports JPEG, PNG, WebP, TIFF, and HEIF formats.

use crate::error::{Error, Result};
use little_exif::exif_tag::ExifTag;
use little_exif::metadata::Metadata;
use little_exif::rational::uR64;
use std::path::Path;

/// Editable metadata fields for EXIF writing.
///
/// All fields are strings to simplify UI binding. Validation and conversion
/// to EXIF types happens during the write operation.
#[derive(Debug, Clone, Default)]
pub struct EditableMetadata {
    // Camera info
    pub camera_make: String,
    pub camera_model: String,

    // Date info
    pub date_taken: String,

    // Exposure info
    pub exposure_time: String,
    pub aperture: String,
    pub iso: String,
    pub flash: String,

    // Lens info
    pub focal_length: String,
    pub focal_length_35mm: String,

    // GPS info
    pub gps_latitude: String,
    pub gps_longitude: String,
}

impl EditableMetadata {
    /// Creates an EditableMetadata from an ImageMetadata reference.
    pub fn from_image_metadata(meta: &super::metadata::ImageMetadata) -> Self {
        Self {
            camera_make: meta.camera_make.clone().unwrap_or_default(),
            camera_model: meta.camera_model.clone().unwrap_or_default(),
            date_taken: meta.date_taken.clone().unwrap_or_default(),
            exposure_time: meta.exposure_time.clone().unwrap_or_default(),
            aperture: meta.aperture.clone().unwrap_or_default(),
            iso: meta.iso.clone().unwrap_or_default(),
            flash: meta.flash.clone().unwrap_or_default(),
            focal_length: meta.focal_length.clone().unwrap_or_default(),
            focal_length_35mm: meta.focal_length_35mm.clone().unwrap_or_default(),
            gps_latitude: meta
                .gps_latitude
                .map(|v| format!("{:.6}", v))
                .unwrap_or_default(),
            gps_longitude: meta
                .gps_longitude
                .map(|v| format!("{:.6}", v))
                .unwrap_or_default(),
        }
    }

    /// Returns true if any field has a non-empty value.
    pub fn has_any_data(&self) -> bool {
        !self.camera_make.is_empty()
            || !self.camera_model.is_empty()
            || !self.date_taken.is_empty()
            || !self.exposure_time.is_empty()
            || !self.aperture.is_empty()
            || !self.iso.is_empty()
            || !self.flash.is_empty()
            || !self.focal_length.is_empty()
            || !self.focal_length_35mm.is_empty()
            || !self.gps_latitude.is_empty()
            || !self.gps_longitude.is_empty()
    }
}

/// Writes EXIF metadata to an image file.
///
/// This function reads existing metadata from the file, updates it with the
/// provided values, and writes it back. Empty strings are skipped (existing
/// values are preserved).
///
/// # Arguments
/// * `path` - Path to the image file
/// * `metadata` - The metadata to write
///
/// # Errors
/// Returns an error if:
/// - The file format is not supported
/// - The file cannot be read or written
/// - Metadata conversion fails
pub fn write_exif<P: AsRef<Path>>(path: P, metadata: &EditableMetadata) -> Result<()> {
    let path = path.as_ref();

    // Load existing metadata to preserve unmodified tags
    let mut exif_metadata = Metadata::new_from_path(path).map_err(|e| {
        Error::Io(format!(
            "Failed to read existing metadata from '{}': {:?}",
            path.display(),
            e
        ))
    })?;

    // Set camera info
    if !metadata.camera_make.is_empty() {
        exif_metadata.set_tag(ExifTag::Make(metadata.camera_make.clone()));
    }
    if !metadata.camera_model.is_empty() {
        exif_metadata.set_tag(ExifTag::Model(metadata.camera_model.clone()));
    }

    // Set date info
    if !metadata.date_taken.is_empty() {
        exif_metadata.set_tag(ExifTag::DateTimeOriginal(metadata.date_taken.clone()));
    }

    // Set exposure info
    if !metadata.exposure_time.is_empty() {
        if let Some((num, den)) = parse_exposure_time(&metadata.exposure_time) {
            let rational = uR64 {
                nominator: num,
                denominator: den,
            };
            exif_metadata.set_tag(ExifTag::ExposureTime(vec![rational]));
        }
    }
    if !metadata.aperture.is_empty() {
        if let Some((num, den)) = parse_aperture(&metadata.aperture) {
            let rational = uR64 {
                nominator: num,
                denominator: den,
            };
            exif_metadata.set_tag(ExifTag::FNumber(vec![rational]));
        }
    }
    if !metadata.iso.is_empty() {
        if let Ok(iso_value) = metadata.iso.trim().parse::<u16>() {
            exif_metadata.set_tag(ExifTag::ISO(vec![iso_value]));
        }
    }

    // Set lens info
    if !metadata.focal_length.is_empty() {
        if let Some((num, den)) = parse_focal_length(&metadata.focal_length) {
            let rational = uR64 {
                nominator: num,
                denominator: den,
            };
            exif_metadata.set_tag(ExifTag::FocalLength(vec![rational]));
        }
    }
    if !metadata.focal_length_35mm.is_empty() {
        if let Ok(fl) = metadata
            .focal_length_35mm
            .trim()
            .trim_end_matches(" mm")
            .trim_end_matches("mm")
            .parse::<u16>()
        {
            exif_metadata.set_tag(ExifTag::FocalLengthIn35mmFormat(vec![fl]));
        }
    }

    // Set GPS info
    if !metadata.gps_latitude.is_empty() && !metadata.gps_longitude.is_empty() {
        if let (Ok(lat), Ok(lon)) = (
            metadata.gps_latitude.trim().parse::<f64>(),
            metadata.gps_longitude.trim().parse::<f64>(),
        ) {
            set_gps_coordinates(&mut exif_metadata, lat, lon);
        }
    }

    // Write metadata back to file
    exif_metadata.write_to_file(path).map_err(|e| {
        Error::Io(format!(
            "Failed to write metadata to '{}': {:?}",
            path.display(),
            e
        ))
    })?;

    Ok(())
}

/// Parses exposure time string (e.g., "1/250" or "1/250 sec") to EXIF rational.
fn parse_exposure_time(value: &str) -> Option<(u32, u32)> {
    let cleaned = value
        .trim()
        .trim_end_matches(" sec")
        .trim_end_matches("s")
        .trim();

    if cleaned.contains('/') {
        let parts: Vec<&str> = cleaned.split('/').collect();
        if parts.len() == 2 {
            let num = parts[0].trim().parse::<u32>().ok()?;
            let den = parts[1].trim().parse::<u32>().ok()?;
            return Some((num, den));
        }
    } else {
        // Decimal format (e.g., "0.004")
        let decimal = cleaned.parse::<f64>().ok()?;
        if decimal > 0.0 {
            // Convert to fraction with reasonable precision
            let denominator = (1.0 / decimal).round() as u32;
            return Some((1, denominator));
        }
    }
    None
}

/// Parses aperture string (e.g., "f/2.8" or "2.8") to EXIF rational.
fn parse_aperture(value: &str) -> Option<(u32, u32)> {
    let cleaned = value
        .trim()
        .trim_start_matches("f/")
        .trim_start_matches("F/")
        .trim();

    let f_number = cleaned.parse::<f64>().ok()?;
    if f_number > 0.0 {
        // Store as rational with 10x precision (e.g., 2.8 -> 28/10)
        let numerator = (f_number * 10.0).round() as u32;
        Some((numerator, 10))
    } else {
        None
    }
}

/// Parses focal length string (e.g., "50 mm" or "50") to EXIF rational.
fn parse_focal_length(value: &str) -> Option<(u32, u32)> {
    let cleaned = value
        .trim()
        .trim_end_matches(" mm")
        .trim_end_matches("mm")
        .trim();

    let focal = cleaned.parse::<f64>().ok()?;
    if focal > 0.0 {
        // Store as rational with 10x precision for decimal values
        let numerator = (focal * 10.0).round() as u32;
        Some((numerator, 10))
    } else {
        None
    }
}

/// Sets GPS coordinates in EXIF metadata.
///
/// Converts decimal degrees to EXIF GPS format (degrees, minutes, seconds as rationals)
/// and sets both coordinate values and reference directions.
fn set_gps_coordinates(metadata: &mut Metadata, latitude: f64, longitude: f64) {
    // Latitude
    let lat_ref = if latitude >= 0.0 { "N" } else { "S" };
    let lat_dms = decimal_to_dms(latitude.abs());
    metadata.set_tag(ExifTag::GPSLatitudeRef(lat_ref.to_string()));
    metadata.set_tag(ExifTag::GPSLatitude(lat_dms));

    // Longitude
    let lon_ref = if longitude >= 0.0 { "E" } else { "W" };
    let lon_dms = decimal_to_dms(longitude.abs());
    metadata.set_tag(ExifTag::GPSLongitudeRef(lon_ref.to_string()));
    metadata.set_tag(ExifTag::GPSLongitude(lon_dms));
}

/// Converts decimal degrees to DMS (degrees, minutes, seconds) as EXIF rationals.
fn decimal_to_dms(decimal: f64) -> Vec<uR64> {
    let degrees = decimal.floor();
    let minutes_decimal = (decimal - degrees) * 60.0;
    let minutes = minutes_decimal.floor();
    let seconds = (minutes_decimal - minutes) * 60.0;

    vec![
        uR64 {
            nominator: degrees as u32,
            denominator: 1,
        },
        uR64 {
            nominator: minutes as u32,
            denominator: 1,
        },
        uR64 {
            nominator: (seconds * 100.0) as u32,
            denominator: 100,
        },
    ]
}

/// Returns the list of file extensions that support EXIF writing.
pub fn supported_extensions() -> &'static [&'static str] {
    &[
        "jpg", "jpeg", "png", "webp", "tiff", "tif", "heic", "heif", "jxl", "avif",
    ]
}

/// Checks if a file format supports EXIF writing.
pub fn is_format_supported<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref()
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| supported_extensions().contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_exposure_time_fraction() {
        assert_eq!(parse_exposure_time("1/250"), Some((1, 250)));
        assert_eq!(parse_exposure_time("1/125 sec"), Some((1, 125)));
        assert_eq!(parse_exposure_time("1/1000s"), Some((1, 1000)));
    }

    #[test]
    fn test_parse_exposure_time_decimal() {
        // 0.004 seconds ≈ 1/250
        let result = parse_exposure_time("0.004");
        assert!(result.is_some());
        let (num, den) = result.unwrap();
        assert_eq!(num, 1);
        assert!(den >= 240 && den <= 260); // Allow some rounding tolerance
    }

    #[test]
    fn test_parse_aperture() {
        assert_eq!(parse_aperture("f/2.8"), Some((28, 10)));
        assert_eq!(parse_aperture("F/4"), Some((40, 10)));
        assert_eq!(parse_aperture("5.6"), Some((56, 10)));
    }

    #[test]
    fn test_parse_focal_length() {
        assert_eq!(parse_focal_length("50 mm"), Some((500, 10)));
        assert_eq!(parse_focal_length("85mm"), Some((850, 10)));
        assert_eq!(parse_focal_length("24"), Some((240, 10)));
    }

    #[test]
    fn test_decimal_to_dms() {
        // Paris coordinates: 48.8566, 2.3522
        let dms = decimal_to_dms(48.8566);
        assert_eq!(dms[0].nominator, 48); // 48 degrees
        assert_eq!(dms[0].denominator, 1);
        assert_eq!(dms[1].nominator, 51); // 51 minutes
        assert_eq!(dms[1].denominator, 1);
        // Seconds: approximately 23.76
    }

    #[test]
    fn test_is_format_supported() {
        assert!(is_format_supported("photo.jpg"));
        assert!(is_format_supported("image.PNG"));
        assert!(is_format_supported("picture.webp"));
        assert!(!is_format_supported("video.mp4"));
        assert!(!is_format_supported("document.pdf"));
    }

    #[test]
    fn test_editable_metadata_from_image_metadata() {
        let image_meta = super::super::metadata::ImageMetadata {
            camera_make: Some("Canon".to_string()),
            camera_model: Some("EOS 5D".to_string()),
            gps_latitude: Some(48.8566),
            gps_longitude: Some(2.3522),
            ..Default::default()
        };

        let editable = EditableMetadata::from_image_metadata(&image_meta);
        assert_eq!(editable.camera_make, "Canon");
        assert_eq!(editable.camera_model, "EOS 5D");
        assert!(editable.gps_latitude.starts_with("48.8566"));
        assert!(editable.gps_longitude.starts_with("2.3522"));
    }

    #[test]
    fn test_editable_metadata_has_any_data() {
        let empty = EditableMetadata::default();
        assert!(!empty.has_any_data());

        let with_data = EditableMetadata {
            camera_make: "Nikon".to_string(),
            ..Default::default()
        };
        assert!(with_data.has_any_data());
    }
}
