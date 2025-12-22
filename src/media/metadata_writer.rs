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

/// Editable metadata fields for EXIF and XMP writing.
///
/// All fields are strings to simplify UI binding. Validation and conversion
/// to EXIF types happens during the write operation.
#[derive(Debug, Clone, Default)]
pub struct EditableMetadata {
    // Camera info (EXIF)
    pub camera_make: String,
    pub camera_model: String,

    // Date info (EXIF)
    pub date_taken: String,

    // Exposure info (EXIF)
    pub exposure_time: String,
    pub aperture: String,
    pub iso: String,
    pub flash: String,

    // Lens info (EXIF)
    pub focal_length: String,
    pub focal_length_35mm: String,

    // GPS info (EXIF)
    pub gps_latitude: String,
    pub gps_longitude: String,

    // Dublin Core / XMP metadata
    /// dc:title - Title of the work
    pub dc_title: String,
    /// dc:creator - Creator/author
    pub dc_creator: String,
    /// dc:description - Description
    pub dc_description: String,
    /// dc:subject - Keywords/tags (comma-separated)
    pub dc_subject: String,
    /// dc:rights - Copyright/license
    pub dc_rights: String,
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
                .map(|v| format!("{v:.6}"))
                .unwrap_or_default(),
            gps_longitude: meta
                .gps_longitude
                .map(|v| format!("{v:.6}"))
                .unwrap_or_default(),
            dc_title: meta.dc_title.clone().unwrap_or_default(),
            dc_creator: meta.dc_creator.clone().unwrap_or_default(),
            dc_description: meta.dc_description.clone().unwrap_or_default(),
            dc_subject: meta
                .dc_subject
                .as_ref()
                .map(|v| v.join(", "))
                .unwrap_or_default(),
            dc_rights: meta.dc_rights.clone().unwrap_or_default(),
        }
    }

    /// Returns true if any EXIF field has a non-empty value.
    pub fn has_any_exif_data(&self) -> bool {
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

    /// Returns true if any Dublin Core / XMP field has a non-empty value.
    pub fn has_any_xmp_data(&self) -> bool {
        !self.dc_title.is_empty()
            || !self.dc_creator.is_empty()
            || !self.dc_description.is_empty()
            || !self.dc_subject.is_empty()
            || !self.dc_rights.is_empty()
    }

    /// Returns true if any field has a non-empty value.
    pub fn has_any_data(&self) -> bool {
        self.has_any_exif_data() || self.has_any_xmp_data()
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

    // Try to load existing metadata to preserve unmodified tags.
    // If loading fails (e.g., no EXIF data), we track this to avoid a crash
    // in little_exif when writing to files without existing EXIF.
    // See: https://github.com/TechnikTobi/little_exif/issues/XX
    let (mut exif_metadata, has_existing_exif) = match Metadata::new_from_path(path) {
        Ok(m) => (m, true),
        Err(e) => {
            // Only warn if user is trying to write EXIF fields
            if metadata.has_any_exif_data() {
                eprintln!(
                    "[WARN] Could not read existing EXIF from '{}': {:?}. EXIF write will be skipped.",
                    path.display(),
                    e
                );
            }
            (Metadata::new(), false)
        }
    };

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

    // Write EXIF metadata back to file only if file had existing EXIF.
    // Workaround: little_exif panics when writing to files without EXIF data.
    if has_existing_exif {
        exif_metadata.write_to_file(path).map_err(|e| {
            Error::Io(format!(
                "Failed to write EXIF metadata to '{}': {:?}",
                path.display(),
                e
            ))
        })?;
    }

    // Write XMP metadata (JPEG only for now) - always attempt this
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        if ext.eq_ignore_ascii_case("jpg") || ext.eq_ignore_ascii_case("jpeg") {
            write_xmp_to_jpeg(path, metadata)?;
        }
    }

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

/// XMP namespace marker for JPEG APP1 segments.
const XMP_MARKER: &[u8] = b"http://ns.adobe.com/xap/1.0/\0";

/// Writes XMP Dublin Core metadata to a JPEG file.
///
/// This function creates an XMP packet with Dublin Core metadata and embeds it
/// in the JPEG file's APP1 segment. If an XMP segment already exists, it is replaced.
fn write_xmp_to_jpeg<P: AsRef<Path>>(path: P, metadata: &EditableMetadata) -> Result<()> {
    let path = path.as_ref();

    // Skip if no XMP data to write
    if !metadata.has_any_xmp_data() {
        return Ok(());
    }

    // Generate XMP packet
    let xmp_data = generate_xmp_packet(metadata);

    // Read the entire file
    let file_data = std::fs::read(path)
        .map_err(|e| Error::Io(format!("Failed to read file '{}': {}", path.display(), e)))?;

    // Verify JPEG magic number
    if file_data.len() < 2 || file_data[0] != 0xFF || file_data[1] != 0xD8 {
        return Err(Error::Io("Not a valid JPEG file".to_string()));
    }

    // Find existing XMP segment or insertion point
    let (xmp_start, xmp_end, insert_pos) = find_xmp_segment_or_insertion_point(&file_data)?;

    // Build new XMP segment: APP1 marker + length + XMP marker + data
    let xmp_segment = build_xmp_segment(&xmp_data)?;

    // Create new file data
    let new_file_data = if let (Some(start), Some(end)) = (xmp_start, xmp_end) {
        // Replace existing XMP segment
        let mut new_data = Vec::with_capacity(file_data.len() - (end - start) + xmp_segment.len());
        new_data.extend_from_slice(&file_data[..start]);
        new_data.extend_from_slice(&xmp_segment);
        new_data.extend_from_slice(&file_data[end..]);
        new_data
    } else {
        // Insert new XMP segment at insertion point
        let mut new_data = Vec::with_capacity(file_data.len() + xmp_segment.len());
        new_data.extend_from_slice(&file_data[..insert_pos]);
        new_data.extend_from_slice(&xmp_segment);
        new_data.extend_from_slice(&file_data[insert_pos..]);
        new_data
    };

    // Write back to file
    std::fs::write(path, new_file_data).map_err(|e| {
        Error::Io(format!(
            "Failed to write XMP to '{}': {}",
            path.display(),
            e
        ))
    })?;

    Ok(())
}

/// Generates an XMP packet with Dublin Core metadata.
fn generate_xmp_packet(metadata: &EditableMetadata) -> Vec<u8> {
    use xmp_writer::XmpWriter;

    let mut writer = XmpWriter::new();

    // Set Dublin Core fields
    if !metadata.dc_title.is_empty() {
        writer.title([(None, metadata.dc_title.as_str())]);
    }

    if !metadata.dc_creator.is_empty() {
        writer.creator([metadata.dc_creator.as_str()]);
    }

    if !metadata.dc_description.is_empty() {
        writer.description([(None, metadata.dc_description.as_str())]);
    }

    if !metadata.dc_subject.is_empty() {
        // Parse comma-separated keywords
        let keywords: Vec<&str> = metadata
            .dc_subject
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        if !keywords.is_empty() {
            writer.subject(keywords);
        }
    }

    if !metadata.dc_rights.is_empty() {
        writer.rights([(None, metadata.dc_rights.as_str())]);
    }

    writer.finish(None).into_bytes()
}

/// Finds the XMP segment boundaries or the insertion point after SOI.
///
/// Returns (xmp_start, xmp_end, insertion_point):
/// - If XMP exists: (Some(start), Some(end), _) where start..end is the segment range
/// - If no XMP: (None, None, insertion_point) where insertion_point is after the first APP segment
fn find_xmp_segment_or_insertion_point(
    data: &[u8],
) -> Result<(Option<usize>, Option<usize>, usize)> {
    let mut pos = 2; // Skip SOI (0xFF 0xD8)
    let mut first_app_end = 2; // Default to after SOI

    while pos + 4 < data.len() {
        // Check for marker
        if data[pos] != 0xFF {
            return Err(Error::Io("Invalid JPEG structure".to_string()));
        }

        let marker_type = data[pos + 1];

        match marker_type {
            0xD9 => break, // EOI - end of image
            0xD8 => {
                pos += 2;
                continue; // Skip embedded SOI
            }
            0x00 => {
                pos += 2;
                continue; // Stuffed byte
            }
            _ if (0xD0..=0xD7).contains(&marker_type) => {
                // RST markers have no length
                pos += 2;
                continue;
            }
            0xDA => break, // SOS - start of scan data, stop searching
            _ => {
                // Marker with length
                if pos + 4 > data.len() {
                    break;
                }

                let segment_len = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) as usize;
                let segment_end = pos + 2 + segment_len;

                if segment_end > data.len() {
                    break;
                }

                // Check if this is an APP1 segment with XMP marker
                if marker_type == 0xE1 {
                    let data_start = pos + 4;
                    if data_start + XMP_MARKER.len() <= segment_end
                        && &data[data_start..data_start + XMP_MARKER.len()] == XMP_MARKER
                    {
                        // Found XMP segment
                        return Ok((Some(pos), Some(segment_end), pos));
                    }
                }

                // Track end of first APP segment for insertion point
                if (0xE0..=0xEF).contains(&marker_type) && first_app_end == 2 {
                    first_app_end = segment_end;
                }

                pos = segment_end;
            }
        }
    }

    // No XMP found, return insertion point
    Ok((None, None, first_app_end))
}

/// Builds an XMP APP1 segment for JPEG.
fn build_xmp_segment(xmp_data: &[u8]) -> Result<Vec<u8>> {
    // APP1 structure: FF E1 + length (2 bytes) + XMP marker + XMP data
    // Length includes the 2 length bytes + marker + data
    let total_len = 2 + XMP_MARKER.len() + xmp_data.len();

    if total_len > 0xFFFF {
        return Err(Error::Io(
            "XMP data too large for JPEG APP1 segment".to_string(),
        ));
    }

    let mut segment = Vec::with_capacity(2 + total_len);
    segment.push(0xFF);
    segment.push(0xE1);
    segment.extend_from_slice(&(total_len as u16).to_be_bytes());
    segment.extend_from_slice(XMP_MARKER);
    segment.extend_from_slice(xmp_data);

    Ok(segment)
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
        .is_some_and(|ext| supported_extensions().contains(&ext.to_lowercase().as_str()))
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
        assert!((240..=260).contains(&den)); // Allow some rounding tolerance
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
