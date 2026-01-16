// SPDX-License-Identifier: MPL-2.0
//! Metadata reading and writing port definitions.
//!
//! This module defines traits for reading and writing media metadata (EXIF, XMP).
//! Infrastructure adapters implement these traits using libraries like `little_exif`
//! and `img-parts`.
//!
//! # Supported Metadata Types
//!
//! - **EXIF**: Camera information, date taken, GPS coordinates, orientation
//! - **XMP**: Dublin Core fields (title, creator, description, keywords, copyright)
//! - **ICC Profile**: Color profile data (opaque bytes)

use crate::domain::metadata::GpsCoordinates;
use std::fmt;
use std::path::Path;

// =============================================================================
// MetadataError
// =============================================================================

/// Errors that can occur during metadata operations.
#[derive(Debug, Clone)]
pub enum MetadataError {
    /// Failed to read metadata from file.
    ReadFailed(String),

    /// Failed to write metadata to file.
    WriteFailed(String),

    /// The file format doesn't support metadata.
    UnsupportedFormat,

    /// The metadata in the file is corrupted.
    CorruptedMetadata(String),

    /// A specific metadata field was not found.
    FieldNotFound(String),

    /// The file could not be accessed.
    IoError(String),
}

impl fmt::Display for MetadataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetadataError::ReadFailed(msg) => write!(f, "Failed to read metadata: {msg}"),
            MetadataError::WriteFailed(msg) => write!(f, "Failed to write metadata: {msg}"),
            MetadataError::UnsupportedFormat => write!(f, "Format does not support metadata"),
            MetadataError::CorruptedMetadata(msg) => write!(f, "Corrupted metadata: {msg}"),
            MetadataError::FieldNotFound(field) => write!(f, "Metadata field not found: {field}"),
            MetadataError::IoError(msg) => write!(f, "I/O error: {msg}"),
        }
    }
}

impl std::error::Error for MetadataError {}

// =============================================================================
// ExifData
// =============================================================================

/// EXIF metadata extracted from an image file.
///
/// This is a pure value object containing common EXIF fields.
/// All fields are optional as not all images have complete metadata.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExifData {
    /// Camera manufacturer (e.g., "Canon", "Nikon").
    pub camera_make: Option<String>,

    /// Camera model (e.g., "EOS 5D Mark IV").
    pub camera_model: Option<String>,

    /// Date and time when the photo was taken (ISO 8601 format).
    pub date_taken: Option<String>,

    /// Exposure time as a string (e.g., "1/125").
    pub exposure_time: Option<String>,

    /// F-number (aperture).
    pub f_number: Option<f32>,

    /// ISO sensitivity.
    pub iso: Option<u32>,

    /// Focal length in millimeters.
    pub focal_length: Option<f32>,

    /// GPS coordinates.
    pub gps: Option<GpsCoordinates>,

    /// Image orientation (EXIF orientation tag, 1-8).
    pub orientation: Option<u16>,

    /// Software used to create/edit the image.
    pub software: Option<String>,

    /// Date and time when the file was last modified.
    pub modify_date: Option<String>,

    /// Image width in pixels.
    pub width: Option<u32>,

    /// Image height in pixels.
    pub height: Option<u32>,
}

impl ExifData {
    /// Creates an empty `ExifData`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if this metadata has GPS coordinates.
    #[must_use]
    pub fn has_gps(&self) -> bool {
        self.gps.is_some()
    }

    /// Returns `true` if all fields are empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.camera_make.is_none()
            && self.camera_model.is_none()
            && self.date_taken.is_none()
            && self.exposure_time.is_none()
            && self.f_number.is_none()
            && self.iso.is_none()
            && self.focal_length.is_none()
            && self.gps.is_none()
            && self.orientation.is_none()
            && self.software.is_none()
            && self.modify_date.is_none()
            && self.width.is_none()
            && self.height.is_none()
    }
}

// =============================================================================
// XmpData
// =============================================================================

/// XMP (Dublin Core) metadata.
///
/// This is a pure value object containing common Dublin Core fields.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct XmpData {
    /// Title of the work.
    pub title: Option<String>,

    /// Creator/author name.
    pub creator: Option<String>,

    /// Description of the content.
    pub description: Option<String>,

    /// Keywords/tags.
    pub keywords: Vec<String>,

    /// Copyright notice.
    pub copyright: Option<String>,

    /// Rating (1-5 stars, where 0 means unrated).
    pub rating: Option<u8>,
}

impl XmpData {
    /// Creates an empty `XmpData`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if all fields are empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.creator.is_none()
            && self.description.is_none()
            && self.keywords.is_empty()
            && self.copyright.is_none()
            && self.rating.is_none()
    }
}

// =============================================================================
// MediaMetadata
// =============================================================================

/// Complete metadata for a media file.
///
/// Combines EXIF, XMP, and ICC profile data.
#[derive(Debug, Clone, Default)]
pub struct MediaMetadata {
    /// EXIF metadata.
    pub exif: Option<ExifData>,

    /// XMP metadata.
    pub xmp: Option<XmpData>,

    /// ICC color profile (raw bytes).
    pub icc_profile: Option<Vec<u8>>,
}

impl MediaMetadata {
    /// Creates empty metadata.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if no metadata is present.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.exif.as_ref().is_none_or(ExifData::is_empty)
            && self.xmp.as_ref().is_none_or(XmpData::is_empty)
            && self.icc_profile.is_none()
    }

    /// Returns `true` if this metadata has GPS coordinates.
    #[must_use]
    pub fn has_gps(&self) -> bool {
        self.exif.as_ref().is_some_and(ExifData::has_gps)
    }
}

// =============================================================================
// MetadataField
// =============================================================================

/// Metadata fields that can be stripped or edited.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetadataField {
    /// GPS coordinates.
    Gps,

    /// Software tag.
    Software,

    /// Modification date.
    ModifyDate,

    /// All EXIF data.
    AllExif,

    /// All XMP data.
    AllXmp,

    /// ICC color profile.
    IccProfile,

    /// All metadata (EXIF + XMP + ICC).
    All,
}

impl MetadataField {
    /// Returns a human-readable name for this field.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            MetadataField::Gps => "GPS",
            MetadataField::Software => "Software",
            MetadataField::ModifyDate => "Modify Date",
            MetadataField::AllExif => "All EXIF",
            MetadataField::AllXmp => "All XMP",
            MetadataField::IccProfile => "ICC Profile",
            MetadataField::All => "All Metadata",
        }
    }
}

// =============================================================================
// MetadataReader Trait
// =============================================================================

/// Port for reading metadata from media files.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` for concurrent reading.
///
/// # Example
///
/// ```ignore
/// use iced_lens::application::port::metadata::MetadataReader;
/// use std::path::Path;
///
/// fn show_camera_info(reader: &impl MetadataReader, path: &Path) {
///     if let Ok(meta) = reader.read(path) {
///         if let Some(exif) = meta.exif {
///             if let Some(model) = exif.camera_model {
///                 println!("Camera: {model}");
///             }
///         }
///     }
/// }
/// ```
pub trait MetadataReader: Send + Sync {
    /// Reads all metadata from a file.
    ///
    /// # Errors
    ///
    /// Returns a [`MetadataError`] if reading fails.
    fn read(&self, path: &Path) -> Result<MediaMetadata, MetadataError>;

    /// Checks if a file has GPS coordinates without reading all metadata.
    ///
    /// This is faster than calling `read()` when only GPS presence is needed.
    fn has_gps(&self, path: &Path) -> bool;
}

// =============================================================================
// MetadataWriter Trait
// =============================================================================

/// Port for writing metadata to media files.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` for concurrent writing.
///
/// # Example
///
/// ```ignore
/// use iced_lens::application::port::metadata::{MetadataWriter, MetadataField};
/// use std::path::Path;
///
/// fn remove_location(writer: &impl MetadataWriter, path: &Path) {
///     if let Err(e) = writer.strip(path, &[MetadataField::Gps]) {
///         eprintln!("Failed to strip GPS: {e}");
///     }
/// }
/// ```
pub trait MetadataWriter: Send + Sync {
    /// Writes metadata to a file, preserving existing data.
    ///
    /// This merges the provided metadata with existing file metadata.
    /// Fields in the provided metadata override existing values.
    ///
    /// # Errors
    ///
    /// Returns a [`MetadataError`] if writing fails.
    fn write(&self, path: &Path, metadata: &MediaMetadata) -> Result<(), MetadataError>;

    /// Strips specific metadata fields from a file.
    ///
    /// # Errors
    ///
    /// Returns a [`MetadataError`] if stripping fails.
    fn strip(&self, path: &Path, fields: &[MetadataField]) -> Result<(), MetadataError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_error_display() {
        let err = MetadataError::ReadFailed("invalid format".to_string());
        assert!(format!("{err}").contains("invalid format"));

        let err = MetadataError::FieldNotFound("GPS".to_string());
        assert!(format!("{err}").contains("GPS"));

        let err = MetadataError::UnsupportedFormat;
        assert!(format!("{err}").contains("does not support"));
    }

    #[test]
    fn exif_data_empty() {
        let exif = ExifData::new();
        assert!(exif.is_empty());
        assert!(!exif.has_gps());

        let mut exif = ExifData::new();
        exif.camera_make = Some("Canon".to_string());
        assert!(!exif.is_empty());
    }

    #[test]
    fn exif_data_with_gps() {
        let mut exif = ExifData::new();
        exif.gps = Some(GpsCoordinates::new(48.8566, 2.3522));
        assert!(exif.has_gps());
    }

    #[test]
    fn xmp_data_empty() {
        let xmp = XmpData::new();
        assert!(xmp.is_empty());

        let mut xmp = XmpData::new();
        xmp.title = Some("Test".to_string());
        assert!(!xmp.is_empty());
    }

    #[test]
    fn xmp_data_keywords() {
        let mut xmp = XmpData::new();
        xmp.keywords = vec!["nature".to_string(), "landscape".to_string()];
        assert!(!xmp.is_empty());
    }

    #[test]
    fn media_metadata_empty() {
        let meta = MediaMetadata::new();
        assert!(meta.is_empty());
        assert!(!meta.has_gps());
    }

    #[test]
    fn media_metadata_with_exif_gps() {
        let mut exif = ExifData::new();
        exif.gps = Some(GpsCoordinates::new(48.8566, 2.3522));

        let mut meta = MediaMetadata::new();
        meta.exif = Some(exif);
        assert!(meta.has_gps());
        assert!(!meta.is_empty());
    }

    #[test]
    fn metadata_field_names() {
        assert_eq!(MetadataField::Gps.name(), "GPS");
        assert_eq!(MetadataField::AllExif.name(), "All EXIF");
        assert_eq!(MetadataField::All.name(), "All Metadata");
    }

    // Test that traits are object-safe
    fn _assert_reader_object_safe(_: &dyn MetadataReader) {}
    fn _assert_writer_object_safe(_: &dyn MetadataWriter) {}
}
