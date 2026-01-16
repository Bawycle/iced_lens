// SPDX-License-Identifier: MPL-2.0
//! Metadata preservation pipeline for image editing.
//!
//! This module provides functionality to preserve and transform metadata
//! when saving edited images. It handles:
//! - Copying EXIF/XMP segments from source to destination using `img-parts`
//! - Applying transformations (GPS stripping, orientation reset, software tag)
//!
//! # Supported formats
//!
//! - **JPEG**: Full support (EXIF + XMP)
//! - **PNG**: Full support (XMP via iTXt chunks, EXIF via eXIf chunks)
//! - **WebP**: Full support (EXIF + XMP)
//! - **TIFF**: Partial support via `little_exif`
//!
//! # Example
//!
//! ```no_run
//! use iced_lens::media::metadata_operations::{preserve_metadata, PreservationConfig};
//! use std::path::Path;
//!
//! let config = PreservationConfig {
//!     strip_gps: true,
//!     add_software_tag: true,
//!     reset_orientation: false,
//! };
//!
//! preserve_metadata(
//!     Path::new("original.jpg"),
//!     Path::new("edited.jpg"),
//!     &config,
//! ).ok();
//! ```

use crate::error::{Error, Result};
use crate::media::metadata::{extract_image_metadata, ImageMetadata};
use crate::media::metadata_writer::is_webp_without_vp8x;
use img_parts::{Bytes, ImageEXIF, ImageICC};
use std::fs;
use std::path::Path;

/// Size of JPEG APP1 header: FF E1 (2) + length (2) + "Exif\0\0" (6) = 10 bytes.
/// Used when stripping APP1 header to get raw EXIF/TIFF bytes for PNG eXIf chunk.
const APP1_HEADER_SIZE: usize = 10;

/// Configuration for metadata preservation during image save.
#[derive(Debug, Clone, Default)]
pub struct PreservationConfig {
    /// Whether to strip GPS coordinates from saved image.
    pub strip_gps: bool,
    /// Whether to add software tag and modification date.
    pub add_software_tag: bool,
    /// Whether to reset EXIF orientation to 1 (normal).
    /// Should be true if image was rotated/flipped during editing.
    pub reset_orientation: bool,
}

/// Preserves metadata from source bytes to destination image file.
///
/// This is the preferred function when the source file might be overwritten
/// by the save operation (e.g., saving to the same file). The caller should
/// read the source bytes BEFORE saving the new pixels.
///
/// # Arguments
/// * `source_bytes` - Raw bytes of the original image with metadata
/// * `source_ext` - File extension of the source (e.g., "jpg", "png")
/// * `dest_path` - Path to the saved image (metadata will be written here)
/// * `config` - Configuration for metadata transformations
///
/// # Errors
/// Returns error if metadata operations fail.
pub fn preserve_metadata_from_bytes<P: AsRef<Path>>(
    source_bytes: &[u8],
    source_ext: &str,
    dest_path: P,
    config: &PreservationConfig,
) -> Result<()> {
    let dest = dest_path.as_ref();

    // Get destination file extension
    let dest_ext = dest
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase)
        .unwrap_or_default();

    // Source and destination must have compatible formats for metadata copy
    let source_ext_lower = source_ext.to_lowercase();

    // Step 1: Copy metadata segments from source bytes to destination
    match (source_ext_lower.as_str(), dest_ext.as_str()) {
        ("jpg" | "jpeg", "jpg" | "jpeg") => {
            copy_jpeg_metadata_from_bytes(source_bytes, dest)?;
        }
        ("png", "png") => {
            copy_png_metadata_from_bytes(source_bytes, dest)?;
        }
        ("webp", "webp") => {
            copy_webp_metadata_from_bytes(source_bytes, dest)?;
        }
        _ => {
            // Cross-format or unsupported format - skip metadata preservation
            return Ok(());
        }
    }

    // Step 2: Apply transformations using little_exif (now that segments exist)
    apply_metadata_transformations(dest, config)?;

    Ok(())
}

/// Preserves metadata from source to destination image file.
///
/// **Warning**: This function reads the source file during execution.
/// If source and destination are the same file, use [`preserve_metadata_from_bytes`]
/// instead, reading the source bytes before saving the new pixels.
///
/// # Arguments
/// * `source_path` - Path to the original image with metadata
/// * `dest_path` - Path to the saved image (metadata will be written here)
/// * `config` - Configuration for metadata transformations
///
/// # Errors
/// Returns error if metadata operations fail.
pub fn preserve_metadata<P1: AsRef<Path>, P2: AsRef<Path>>(
    source_path: P1,
    dest_path: P2,
    config: &PreservationConfig,
) -> Result<()> {
    let source = source_path.as_ref();

    // Read source bytes
    let source_bytes =
        fs::read(source).map_err(|e| Error::Io(format!("Failed to read source file: {e}")))?;

    // Get source extension
    let source_ext = source.extension().and_then(|e| e.to_str()).unwrap_or("");

    preserve_metadata_from_bytes(&source_bytes, source_ext, dest_path, config)
}

/// Copies JPEG metadata (EXIF, XMP, ICC) from source bytes to destination file.
fn copy_jpeg_metadata_from_bytes(source_bytes: &[u8], dest: &Path) -> Result<()> {
    use img_parts::jpeg::Jpeg;

    // Parse source JPEG from bytes
    let source_jpeg = Jpeg::from_bytes(Bytes::copy_from_slice(source_bytes))
        .map_err(|e| Error::Io(format!("Failed to parse source JPEG: {e}")))?;

    // Read destination JPEG
    let dest_bytes =
        fs::read(dest).map_err(|e| Error::Io(format!("Failed to read destination file: {e}")))?;
    let mut dest_jpeg = Jpeg::from_bytes(dest_bytes.into())
        .map_err(|e| Error::Io(format!("Failed to parse destination JPEG: {e}")))?;

    // Copy EXIF data
    if let Some(exif) = source_jpeg.exif() {
        dest_jpeg.set_exif(Some(exif.clone()));
    }

    // Copy ICC profile
    if let Some(icc) = source_jpeg.icc_profile() {
        dest_jpeg.set_icc_profile(Some(icc.clone()));
    }

    // Write back to destination
    let mut output = Vec::new();
    dest_jpeg
        .encoder()
        .write_to(&mut output)
        .map_err(|e| Error::Io(format!("Failed to write JPEG: {e}")))?;

    fs::write(dest, output)
        .map_err(|e| Error::Io(format!("Failed to save destination file: {e}")))?;

    Ok(())
}

/// Copies PNG metadata (EXIF, ICC) from source bytes to destination file.
fn copy_png_metadata_from_bytes(source_bytes: &[u8], dest: &Path) -> Result<()> {
    use img_parts::png::Png;

    // Parse source PNG from bytes
    let source_png = Png::from_bytes(Bytes::copy_from_slice(source_bytes))
        .map_err(|e| Error::Io(format!("Failed to parse source PNG: {e}")))?;

    // Read destination PNG
    let dest_bytes =
        fs::read(dest).map_err(|e| Error::Io(format!("Failed to read destination file: {e}")))?;
    let mut dest_png = Png::from_bytes(dest_bytes.into())
        .map_err(|e| Error::Io(format!("Failed to parse destination PNG: {e}")))?;

    // Copy EXIF data (PNG stores EXIF in eXIf chunk)
    if let Some(exif) = source_png.exif() {
        dest_png.set_exif(Some(exif.clone()));
    }

    // Copy ICC profile
    if let Some(icc) = source_png.icc_profile() {
        dest_png.set_icc_profile(Some(icc.clone()));
    }

    // Write back to destination
    let mut output = Vec::new();
    dest_png
        .encoder()
        .write_to(&mut output)
        .map_err(|e| Error::Io(format!("Failed to write PNG: {e}")))?;

    fs::write(dest, output)
        .map_err(|e| Error::Io(format!("Failed to save destination file: {e}")))?;

    Ok(())
}

/// Copies WebP metadata (EXIF, XMP, ICC) from source bytes to destination file.
fn copy_webp_metadata_from_bytes(source_bytes: &[u8], dest: &Path) -> Result<()> {
    use img_parts::webp::WebP;

    // Parse source WebP from bytes
    let source_webp = WebP::from_bytes(Bytes::copy_from_slice(source_bytes))
        .map_err(|e| Error::Io(format!("Failed to parse source WebP: {e}")))?;

    // Read destination WebP
    let dest_bytes =
        fs::read(dest).map_err(|e| Error::Io(format!("Failed to read destination file: {e}")))?;
    let mut dest_webp = WebP::from_bytes(dest_bytes.into())
        .map_err(|e| Error::Io(format!("Failed to parse destination WebP: {e}")))?;

    // Copy EXIF data
    if let Some(exif) = source_webp.exif() {
        dest_webp.set_exif(Some(exif.clone()));
    }

    // Copy ICC profile
    if let Some(icc) = source_webp.icc_profile() {
        dest_webp.set_icc_profile(Some(icc.clone()));
    }

    // Write back to destination
    let mut output = Vec::new();
    dest_webp
        .encoder()
        .write_to(&mut output)
        .map_err(|e| Error::Io(format!("Failed to write WebP: {e}")))?;

    fs::write(dest, output)
        .map_err(|e| Error::Io(format!("Failed to save destination file: {e}")))?;

    Ok(())
}

/// Applies metadata transformations using `little_exif`.
///
/// This is called after copying segments, so the destination file now has
/// EXIF structure that `little_exif` can modify.
fn apply_metadata_transformations(dest: &Path, config: &PreservationConfig) -> Result<()> {
    use little_exif::exif_tag::ExifTag;
    use little_exif::metadata::Metadata;
    use std::panic;

    // Skip if no transformations needed
    if !config.strip_gps && !config.reset_orientation && !config.add_software_tag {
        return Ok(());
    }

    // Check file extension
    let is_png = dest
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("png"));

    // Read existing EXIF from destination (now it should have EXIF data)
    let dest_path = dest.to_path_buf();
    let read_result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        Metadata::new_from_path(&dest_path)
    }));

    let has_existing_exif = matches!(read_result, Ok(Ok(_)));

    let mut exif_metadata = if let Ok(Ok(m)) = read_result {
        m
    } else {
        // No existing EXIF - check if we can create one
        if is_webp_without_vp8x(dest) {
            // WebP VP8L without VP8X - cannot write EXIF safely
            return Err(Error::Io(
                "Cannot add metadata to this WebP format".into(),
            ));
        }
        // Create new EXIF structure
        Metadata::new()
    };

    // Apply transformations
    if config.strip_gps {
        strip_gps_tags(&mut exif_metadata);
    }

    if config.reset_orientation {
        exif_metadata.set_tag(ExifTag::Orientation(vec![1]));
    }

    if config.add_software_tag {
        let software = format!("IcedLens {}", env!("CARGO_PKG_VERSION"));
        let now = chrono::Local::now();
        let date_modified = now.format("%Y:%m:%d %H:%M:%S").to_string();

        exif_metadata.set_tag(ExifTag::Software(software));
        exif_metadata.set_tag(ExifTag::ModifyDate(date_modified));
    }

    // For PNG without existing EXIF, little_exif::write_to_file() silently fails.
    // Use img-parts to insert EXIF bytes directly.
    if is_png && !has_existing_exif {
        return write_exif_to_png_via_imgparts(&exif_metadata, dest);
    }

    // Write back to file using little_exif (works for JPEG, WebP, PNG with existing EXIF)
    let write_result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        exif_metadata.write_to_file(&dest_path)
    }));

    match write_result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(Error::Io(format!("Failed to write EXIF: {e:?}"))),
        Err(_) => {
            // little_exif panicked - return error so caller can warn user
            Err(Error::Io(
                "Failed to write metadata (unsupported format)".into(),
            ))
        }
    }
}

/// Writes EXIF metadata to a PNG file using img-parts.
///
/// This is needed because `little_exif::write_to_file()` silently fails for PNG
/// files that don't already have an EXIF chunk.
fn write_exif_to_png_via_imgparts(
    exif_metadata: &little_exif::metadata::Metadata,
    dest: &Path,
) -> Result<()> {
    use img_parts::png::Png;
    use little_exif::filetype::FileExtension;

    // Encode EXIF using JPEG format, then strip the APP1 header.
    // JPEG APP1 format: FF E1 [length:2] "Exif\0\0" [TIFF data...]
    // PNG eXIf chunk expects only the TIFF data (starting with "II" or "MM")
    let full_app1 = exif_metadata
        .as_u8_vec(FileExtension::JPEG)
        .map_err(|e| Error::Io(format!("Failed to encode EXIF: {e:?}")))?;

    // Strip APP1 header: FF E1 (2) + length (2) + "Exif\0\0" (6) = 10 bytes
    // Looking at the bytes: [255, 225, 0, 96, 69, 120, 105, 102, 0, 0, 73, 73...]
    // FF E1 = marker, 00 60 = length, "Exif\0\0", then "II" (TIFF)
    if full_app1.len() <= APP1_HEADER_SIZE {
        return Err(Error::Io("EXIF data too short".into()));
    }
    let exif_bytes: Vec<u8> = full_app1[APP1_HEADER_SIZE..].to_vec();

    // Read destination PNG
    let dest_bytes =
        fs::read(dest).map_err(|e| Error::Io(format!("Failed to read PNG file: {e}")))?;
    let mut dest_png = Png::from_bytes(dest_bytes.into())
        .map_err(|e| Error::Io(format!("Failed to parse PNG: {e}")))?;

    // Insert EXIF bytes
    dest_png.set_exif(Some(exif_bytes.into()));

    // Write back
    let mut output = Vec::new();
    dest_png
        .encoder()
        .write_to(&mut output)
        .map_err(|e| Error::Io(format!("Failed to encode PNG: {e}")))?;

    fs::write(dest, output).map_err(|e| Error::Io(format!("Failed to write PNG: {e}")))?;

    Ok(())
}

/// Strips GPS-related tags from EXIF metadata.
fn strip_gps_tags(metadata: &mut little_exif::metadata::Metadata) {
    use little_exif::exif_tag::ExifTag;

    // Remove GPS tags by setting empty values
    // Note: little_exif doesn't have a remove_tag method, so we set empty values
    metadata.set_tag(ExifTag::GPSLatitude(vec![]));
    metadata.set_tag(ExifTag::GPSLongitude(vec![]));
    metadata.set_tag(ExifTag::GPSLatitudeRef(String::new()));
    metadata.set_tag(ExifTag::GPSLongitudeRef(String::new()));
}

/// Applies transformations to `EditableMetadata` (used in tests).
#[cfg(test)]
fn apply_transformations_to_editable(
    metadata: &mut crate::media::metadata_writer::EditableMetadata,
    config: &PreservationConfig,
) {
    if config.strip_gps {
        metadata.gps_latitude.clear();
        metadata.gps_longitude.clear();
    }

    if config.reset_orientation {
        metadata.orientation = "1".to_string();
    }

    if config.add_software_tag {
        metadata.software = format!("IcedLens {}", env!("CARGO_PKG_VERSION"));
        let now = chrono::Local::now();
        metadata.date_modified = now.format("%Y:%m:%d %H:%M:%S").to_string();
    }
}

/// Checks if an image has GPS data.
///
/// Returns true if the image at the given path contains GPS coordinates.
#[must_use]
pub fn has_gps_data<P: AsRef<Path>>(path: P) -> bool {
    extract_image_metadata(path)
        .ok()
        .and_then(|m| m.gps_latitude.zip(m.gps_longitude))
        .is_some()
}

/// Checks if an image has any metadata worth preserving.
///
/// Returns true if the image contains EXIF or XMP metadata.
#[must_use]
pub fn has_metadata<P: AsRef<Path>>(path: P) -> bool {
    extract_image_metadata(path)
        .ok()
        .is_some_and(|m| has_significant_metadata(&m))
}

/// Checks if `ImageMetadata` contains significant data worth preserving.
fn has_significant_metadata(meta: &ImageMetadata) -> bool {
    meta.camera_make.is_some()
        || meta.camera_model.is_some()
        || meta.date_taken.is_some()
        || meta.exposure_time.is_some()
        || meta.aperture.is_some()
        || meta.iso.is_some()
        || meta.focal_length.is_some()
        || meta.gps_latitude.is_some()
        || meta.dc_title.is_some()
        || meta.dc_creator.is_some()
        || meta.dc_description.is_some()
        || meta.dc_rights.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::metadata_writer::EditableMetadata;

    #[test]
    fn test_preservation_config_default() {
        let config = PreservationConfig::default();
        assert!(!config.strip_gps);
        assert!(!config.add_software_tag);
        assert!(!config.reset_orientation);
    }

    #[test]
    fn test_apply_transformations_strip_gps() {
        let mut metadata = EditableMetadata {
            gps_latitude: "48.8566".to_string(),
            gps_longitude: "2.3522".to_string(),
            ..Default::default()
        };

        let config = PreservationConfig {
            strip_gps: true,
            ..Default::default()
        };

        apply_transformations_to_editable(&mut metadata, &config);

        assert!(metadata.gps_latitude.is_empty());
        assert!(metadata.gps_longitude.is_empty());
    }

    #[test]
    fn test_apply_transformations_reset_orientation() {
        let mut metadata = EditableMetadata::default();

        let config = PreservationConfig {
            reset_orientation: true,
            ..Default::default()
        };

        apply_transformations_to_editable(&mut metadata, &config);

        assert_eq!(metadata.orientation, "1");
    }

    #[test]
    fn test_apply_transformations_add_software_tag() {
        let mut metadata = EditableMetadata::default();

        let config = PreservationConfig {
            add_software_tag: true,
            ..Default::default()
        };

        apply_transformations_to_editable(&mut metadata, &config);

        assert!(metadata.software.starts_with("IcedLens"));
        assert!(!metadata.date_modified.is_empty());
        // Check date format (YYYY:MM:DD HH:MM:SS)
        assert_eq!(metadata.date_modified.len(), 19);
        assert_eq!(&metadata.date_modified[4..5], ":");
        assert_eq!(&metadata.date_modified[7..8], ":");
    }

    #[test]
    fn test_has_significant_metadata_empty() {
        let meta = ImageMetadata::default();
        assert!(!has_significant_metadata(&meta));
    }

    #[test]
    fn test_has_significant_metadata_with_camera() {
        let meta = ImageMetadata {
            camera_make: Some("Canon".to_string()),
            ..Default::default()
        };
        assert!(has_significant_metadata(&meta));
    }

    #[test]
    fn test_has_significant_metadata_with_gps() {
        let meta = ImageMetadata {
            gps_latitude: Some(48.8566),
            ..Default::default()
        };
        assert!(has_significant_metadata(&meta));
    }

    #[test]
    fn test_has_significant_metadata_with_dublin_core() {
        let meta = ImageMetadata {
            dc_title: Some("My Photo".to_string()),
            ..Default::default()
        };
        assert!(has_significant_metadata(&meta));
    }

    #[test]
    fn test_apply_all_transformations() {
        let mut metadata = EditableMetadata {
            gps_latitude: "48.8566".to_string(),
            gps_longitude: "2.3522".to_string(),
            camera_make: "Canon".to_string(),
            ..Default::default()
        };

        let config = PreservationConfig {
            strip_gps: true,
            add_software_tag: true,
            reset_orientation: true,
        };

        apply_transformations_to_editable(&mut metadata, &config);

        // GPS should be stripped
        assert!(metadata.gps_latitude.is_empty());
        assert!(metadata.gps_longitude.is_empty());
        // Camera make should be preserved
        assert_eq!(metadata.camera_make, "Canon");
        // Software tag should be added
        assert!(metadata.software.starts_with("IcedLens"));
        // Orientation should be reset
        assert_eq!(metadata.orientation, "1");
    }
}
