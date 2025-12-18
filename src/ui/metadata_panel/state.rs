// SPDX-License-Identifier: MPL-2.0
//! State management for metadata editing.

use super::MetadataField;
use crate::media::metadata::ImageMetadata;
use crate::media::metadata_writer::EditableMetadata;
use std::collections::HashSet;

/// Validation errors for metadata fields.
#[derive(Debug, Clone, Default)]
pub struct ValidationErrors {
    pub date_taken: Option<String>,
    pub exposure_time: Option<String>,
    pub aperture: Option<String>,
    pub iso: Option<String>,
    pub focal_length: Option<String>,
    pub focal_length_35mm: Option<String>,
    pub gps_latitude: Option<String>,
    pub gps_longitude: Option<String>,
}

impl ValidationErrors {
    /// Returns true if there are any validation errors.
    pub fn has_errors(&self) -> bool {
        self.date_taken.is_some()
            || self.exposure_time.is_some()
            || self.aperture.is_some()
            || self.iso.is_some()
            || self.focal_length.is_some()
            || self.focal_length_35mm.is_some()
            || self.gps_latitude.is_some()
            || self.gps_longitude.is_some()
    }
}

/// State for metadata editing mode.
#[derive(Debug, Clone)]
pub struct MetadataEditorState {
    /// Current edited values.
    pub edited: EditableMetadata,
    /// Original values (for change detection).
    original: EditableMetadata,
    /// Validation errors.
    pub errors: ValidationErrors,
    /// Fields currently visible in the editor (progressive disclosure).
    pub visible_fields: HashSet<MetadataField>,
}

impl MetadataEditorState {
    /// Creates a new editor state from image metadata.
    pub fn from_image_metadata(meta: &ImageMetadata) -> Self {
        let editable = EditableMetadata::from_image_metadata(meta);
        let visible = Self::visible_fields_from_data(&editable);
        Self {
            edited: editable.clone(),
            original: editable,
            errors: ValidationErrors::default(),
            visible_fields: visible,
        }
    }

    /// Creates an empty editor state (for images without EXIF data).
    pub fn new_empty() -> Self {
        Self {
            edited: EditableMetadata::default(),
            original: EditableMetadata::default(),
            errors: ValidationErrors::default(),
            visible_fields: HashSet::new(),
        }
    }

    /// Determines which fields should be visible based on non-empty values.
    fn visible_fields_from_data(data: &EditableMetadata) -> HashSet<MetadataField> {
        let mut visible = HashSet::new();

        if !data.camera_make.is_empty() {
            visible.insert(MetadataField::CameraMake);
        }
        if !data.camera_model.is_empty() {
            visible.insert(MetadataField::CameraModel);
        }
        if !data.date_taken.is_empty() {
            visible.insert(MetadataField::DateTaken);
        }
        if !data.exposure_time.is_empty() {
            visible.insert(MetadataField::ExposureTime);
        }
        if !data.aperture.is_empty() {
            visible.insert(MetadataField::Aperture);
        }
        if !data.iso.is_empty() {
            visible.insert(MetadataField::Iso);
        }
        if !data.focal_length.is_empty() {
            visible.insert(MetadataField::FocalLength);
        }
        // GPS fields are treated as a pair
        if !data.gps_latitude.is_empty() || !data.gps_longitude.is_empty() {
            visible.insert(MetadataField::GpsLatitude);
            visible.insert(MetadataField::GpsLongitude);
        }

        visible
    }

    /// Shows a field in the editor.
    /// For GPS fields, both latitude and longitude are shown together.
    pub fn show_field(&mut self, field: MetadataField) {
        self.visible_fields.insert(field);
        // GPS fields are treated as a pair
        if let Some(pair) = field.gps_pair() {
            self.visible_fields.insert(pair);
        }
    }

    /// Removes/hides a field from the editor and clears its value.
    /// For GPS fields, both latitude and longitude are removed together.
    pub fn remove_field(&mut self, field: MetadataField) {
        self.visible_fields.remove(&field);
        self.clear_field_value(&field);

        // GPS fields are treated as a pair
        if let Some(pair) = field.gps_pair() {
            self.visible_fields.remove(&pair);
            self.clear_field_value(&pair);
        }
    }

    /// Clears the value of a specific field.
    fn clear_field_value(&mut self, field: &MetadataField) {
        match field {
            MetadataField::CameraMake => self.edited.camera_make.clear(),
            MetadataField::CameraModel => self.edited.camera_model.clear(),
            MetadataField::DateTaken => {
                self.edited.date_taken.clear();
                self.errors.date_taken = None;
            }
            MetadataField::ExposureTime => {
                self.edited.exposure_time.clear();
                self.errors.exposure_time = None;
            }
            MetadataField::Aperture => {
                self.edited.aperture.clear();
                self.errors.aperture = None;
            }
            MetadataField::Iso => {
                self.edited.iso.clear();
                self.errors.iso = None;
            }
            MetadataField::Flash => self.edited.flash.clear(),
            MetadataField::FocalLength => {
                self.edited.focal_length.clear();
                self.errors.focal_length = None;
            }
            MetadataField::FocalLength35mm => {
                self.edited.focal_length_35mm.clear();
                self.errors.focal_length_35mm = None;
            }
            MetadataField::GpsLatitude => {
                self.edited.gps_latitude.clear();
                self.errors.gps_latitude = None;
            }
            MetadataField::GpsLongitude => {
                self.edited.gps_longitude.clear();
                self.errors.gps_longitude = None;
            }
        }
    }

    /// Returns fields that are not currently visible (available for adding).
    pub fn available_fields(&self) -> Vec<MetadataField> {
        MetadataField::all()
            .iter()
            .filter(|f| !self.visible_fields.contains(f))
            // For GPS, only show one entry (latitude) in the picker
            .filter(|f| **f != MetadataField::GpsLongitude || !self.visible_fields.contains(&MetadataField::GpsLatitude))
            .copied()
            .collect()
    }

    /// Returns true if a field is currently visible.
    pub fn is_field_visible(&self, field: &MetadataField) -> bool {
        self.visible_fields.contains(field)
    }

    /// Returns true if any field has been modified from the original.
    pub fn has_changes(&self) -> bool {
        self.edited.camera_make != self.original.camera_make
            || self.edited.camera_model != self.original.camera_model
            || self.edited.date_taken != self.original.date_taken
            || self.edited.exposure_time != self.original.exposure_time
            || self.edited.aperture != self.original.aperture
            || self.edited.iso != self.original.iso
            || self.edited.flash != self.original.flash
            || self.edited.focal_length != self.original.focal_length
            || self.edited.focal_length_35mm != self.original.focal_length_35mm
            || self.edited.gps_latitude != self.original.gps_latitude
            || self.edited.gps_longitude != self.original.gps_longitude
    }

    /// Resets all fields to their original values.
    pub fn reset(&mut self) {
        self.edited = self.original.clone();
        self.errors = ValidationErrors::default();
        self.visible_fields = Self::visible_fields_from_data(&self.original);
    }

    /// Sets a field value and validates it.
    pub fn set_field(&mut self, field: &MetadataField, value: String) {
        match field {
            MetadataField::CameraMake => self.edited.camera_make = value,
            MetadataField::CameraModel => self.edited.camera_model = value,
            MetadataField::DateTaken => {
                self.edited.date_taken = value.clone();
                self.errors.date_taken = validate_date(&value);
            }
            MetadataField::ExposureTime => {
                self.edited.exposure_time = value.clone();
                self.errors.exposure_time = validate_exposure_time(&value);
            }
            MetadataField::Aperture => {
                self.edited.aperture = value.clone();
                self.errors.aperture = validate_aperture(&value);
            }
            MetadataField::Iso => {
                self.edited.iso = value.clone();
                self.errors.iso = validate_iso(&value);
            }
            MetadataField::Flash => self.edited.flash = value,
            MetadataField::FocalLength => {
                self.edited.focal_length = value.clone();
                self.errors.focal_length = validate_focal_length(&value);
            }
            MetadataField::FocalLength35mm => {
                self.edited.focal_length_35mm = value.clone();
                self.errors.focal_length_35mm = validate_focal_length(&value);
            }
            MetadataField::GpsLatitude => {
                self.edited.gps_latitude = value.clone();
                self.errors.gps_latitude = validate_latitude(&value);
            }
            MetadataField::GpsLongitude => {
                self.edited.gps_longitude = value.clone();
                self.errors.gps_longitude = validate_longitude(&value);
            }
        }
    }

    /// Validates all fields and returns true if all are valid.
    pub fn validate_all(&mut self) -> bool {
        self.errors.date_taken = validate_date(&self.edited.date_taken);
        self.errors.exposure_time = validate_exposure_time(&self.edited.exposure_time);
        self.errors.aperture = validate_aperture(&self.edited.aperture);
        self.errors.iso = validate_iso(&self.edited.iso);
        self.errors.focal_length = validate_focal_length(&self.edited.focal_length);
        self.errors.focal_length_35mm = validate_focal_length(&self.edited.focal_length_35mm);
        self.errors.gps_latitude = validate_latitude(&self.edited.gps_latitude);
        self.errors.gps_longitude = validate_longitude(&self.edited.gps_longitude);

        !self.errors.has_errors()
    }

    /// Returns a reference to the edited metadata for writing.
    pub fn editable_metadata(&self) -> &EditableMetadata {
        &self.edited
    }
}

// =============================================================================
// Validation Functions
// =============================================================================

/// Validates date format (YYYY:MM:DD HH:MM:SS).
fn validate_date(value: &str) -> Option<String> {
    if value.is_empty() {
        return None; // Empty is valid (will clear the field)
    }

    // Basic format check: YYYY:MM:DD HH:MM:SS
    let parts: Vec<&str> = value.split(' ').collect();
    if parts.len() != 2 {
        return Some("Format: YYYY:MM:DD HH:MM:SS".to_string());
    }

    let date_parts: Vec<&str> = parts[0].split(':').collect();
    let time_parts: Vec<&str> = parts[1].split(':').collect();

    if date_parts.len() != 3 || time_parts.len() != 3 {
        return Some("Format: YYYY:MM:DD HH:MM:SS".to_string());
    }

    // Validate each component is numeric
    for part in date_parts.iter().chain(time_parts.iter()) {
        if part.parse::<u32>().is_err() {
            return Some("Invalid date/time values".to_string());
        }
    }

    None
}

/// Validates exposure time (e.g., "1/250" or "0.004").
fn validate_exposure_time(value: &str) -> Option<String> {
    if value.is_empty() {
        return None;
    }

    let cleaned = value
        .trim()
        .trim_end_matches(" sec")
        .trim_end_matches("s")
        .trim();

    if cleaned.contains('/') {
        let parts: Vec<&str> = cleaned.split('/').collect();
        if parts.len() == 2
            && parts[0].trim().parse::<u32>().is_ok()
            && parts[1].trim().parse::<u32>().is_ok()
        {
            return None;
        }
    } else if cleaned.parse::<f64>().is_ok() {
        return None;
    }

    Some("Format: 1/250 or 0.004".to_string())
}

/// Validates aperture (e.g., "f/2.8" or "2.8").
fn validate_aperture(value: &str) -> Option<String> {
    if value.is_empty() {
        return None;
    }

    let cleaned = value
        .trim()
        .trim_start_matches("f/")
        .trim_start_matches("F/")
        .trim();

    if cleaned.parse::<f64>().is_ok() {
        return None;
    }

    Some("Format: f/2.8 or 2.8".to_string())
}

/// Validates ISO (positive integer).
fn validate_iso(value: &str) -> Option<String> {
    if value.is_empty() {
        return None;
    }

    match value.trim().parse::<u32>() {
        Ok(v) if v > 0 => None,
        _ => Some("Must be a positive integer".to_string()),
    }
}

/// Validates focal length (e.g., "50 mm" or "50").
fn validate_focal_length(value: &str) -> Option<String> {
    if value.is_empty() {
        return None;
    }

    let cleaned = value
        .trim()
        .trim_end_matches(" mm")
        .trim_end_matches("mm")
        .trim();

    match cleaned.parse::<f64>() {
        Ok(v) if v > 0.0 => None,
        _ => Some("Format: 50 mm or 50".to_string()),
    }
}

/// Validates latitude (-90 to 90).
fn validate_latitude(value: &str) -> Option<String> {
    if value.is_empty() {
        return None;
    }

    match value.trim().parse::<f64>() {
        Ok(v) if (-90.0..=90.0).contains(&v) => None,
        Ok(_) => Some("Must be between -90 and 90".to_string()),
        Err(_) => Some("Invalid number".to_string()),
    }
}

/// Validates longitude (-180 to 180).
fn validate_longitude(value: &str) -> Option<String> {
    if value.is_empty() {
        return None;
    }

    match value.trim().parse::<f64>() {
        Ok(v) if (-180.0..=180.0).contains(&v) => None,
        Ok(_) => Some("Must be between -180 and 180".to_string()),
        Err(_) => Some("Invalid number".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_changes_detects_modifications() {
        let meta = ImageMetadata {
            camera_make: Some("Canon".to_string()),
            ..Default::default()
        };
        let mut state = MetadataEditorState::from_image_metadata(&meta);

        assert!(!state.has_changes());

        state.edited.camera_make = "Nikon".to_string();
        assert!(state.has_changes());
    }

    #[test]
    fn test_reset_restores_original() {
        let meta = ImageMetadata {
            camera_make: Some("Canon".to_string()),
            ..Default::default()
        };
        let mut state = MetadataEditorState::from_image_metadata(&meta);

        state.edited.camera_make = "Nikon".to_string();
        assert!(state.has_changes());

        state.reset();
        assert!(!state.has_changes());
        assert_eq!(state.edited.camera_make, "Canon");
    }

    #[test]
    fn test_validate_date_valid() {
        assert!(validate_date("").is_none());
        assert!(validate_date("2024:03:15 14:30:00").is_none());
    }

    #[test]
    fn test_validate_date_invalid() {
        assert!(validate_date("2024-03-15").is_some());
        assert!(validate_date("2024:03:15").is_some());
        assert!(validate_date("invalid").is_some());
    }

    #[test]
    fn test_validate_exposure_time() {
        assert!(validate_exposure_time("").is_none());
        assert!(validate_exposure_time("1/250").is_none());
        assert!(validate_exposure_time("1/250 sec").is_none());
        assert!(validate_exposure_time("0.004").is_none());
        assert!(validate_exposure_time("invalid").is_some());
    }

    #[test]
    fn test_validate_aperture() {
        assert!(validate_aperture("").is_none());
        assert!(validate_aperture("f/2.8").is_none());
        assert!(validate_aperture("2.8").is_none());
        assert!(validate_aperture("invalid").is_some());
    }

    #[test]
    fn test_validate_iso() {
        assert!(validate_iso("").is_none());
        assert!(validate_iso("100").is_none());
        assert!(validate_iso("0").is_some());
        assert!(validate_iso("-100").is_some());
        assert!(validate_iso("abc").is_some());
    }

    #[test]
    fn test_validate_latitude() {
        assert!(validate_latitude("").is_none());
        assert!(validate_latitude("48.8566").is_none());
        assert!(validate_latitude("-90").is_none());
        assert!(validate_latitude("90").is_none());
        assert!(validate_latitude("91").is_some());
        assert!(validate_latitude("-91").is_some());
    }

    #[test]
    fn test_validate_longitude() {
        assert!(validate_longitude("").is_none());
        assert!(validate_longitude("2.3522").is_none());
        assert!(validate_longitude("-180").is_none());
        assert!(validate_longitude("180").is_none());
        assert!(validate_longitude("181").is_some());
        assert!(validate_longitude("-181").is_some());
    }

    #[test]
    fn test_set_field_updates_value_and_validates() {
        let mut state = MetadataEditorState::new_empty();

        state.set_field(&MetadataField::Iso, "100".to_string());
        assert_eq!(state.edited.iso, "100");
        assert!(state.errors.iso.is_none());

        state.set_field(&MetadataField::Iso, "invalid".to_string());
        assert_eq!(state.edited.iso, "invalid");
        assert!(state.errors.iso.is_some());
    }

    #[test]
    fn test_visible_fields_from_data() {
        let meta = ImageMetadata {
            camera_make: Some("Canon".to_string()),
            iso: Some("100".to_string()),
            ..Default::default()
        };
        let state = MetadataEditorState::from_image_metadata(&meta);

        assert!(state.is_field_visible(&MetadataField::CameraMake));
        assert!(state.is_field_visible(&MetadataField::Iso));
        assert!(!state.is_field_visible(&MetadataField::DateTaken));
        assert!(!state.is_field_visible(&MetadataField::GpsLatitude));
    }

    #[test]
    fn test_show_field() {
        let mut state = MetadataEditorState::new_empty();

        assert!(!state.is_field_visible(&MetadataField::Aperture));
        state.show_field(MetadataField::Aperture);
        assert!(state.is_field_visible(&MetadataField::Aperture));
    }

    #[test]
    fn test_show_gps_field_adds_pair() {
        let mut state = MetadataEditorState::new_empty();

        state.show_field(MetadataField::GpsLatitude);
        assert!(state.is_field_visible(&MetadataField::GpsLatitude));
        assert!(state.is_field_visible(&MetadataField::GpsLongitude));
    }

    #[test]
    fn test_remove_field_clears_value() {
        let meta = ImageMetadata {
            camera_make: Some("Canon".to_string()),
            ..Default::default()
        };
        let mut state = MetadataEditorState::from_image_metadata(&meta);

        assert!(state.is_field_visible(&MetadataField::CameraMake));
        assert_eq!(state.edited.camera_make, "Canon");

        state.remove_field(MetadataField::CameraMake);
        assert!(!state.is_field_visible(&MetadataField::CameraMake));
        assert!(state.edited.camera_make.is_empty());
    }

    #[test]
    fn test_remove_gps_field_removes_pair() {
        let meta = ImageMetadata {
            gps_latitude: Some(48.8566),
            gps_longitude: Some(2.3522),
            ..Default::default()
        };
        let mut state = MetadataEditorState::from_image_metadata(&meta);

        assert!(state.is_field_visible(&MetadataField::GpsLatitude));
        assert!(state.is_field_visible(&MetadataField::GpsLongitude));

        state.remove_field(MetadataField::GpsLatitude);
        assert!(!state.is_field_visible(&MetadataField::GpsLatitude));
        assert!(!state.is_field_visible(&MetadataField::GpsLongitude));
        assert!(state.edited.gps_latitude.is_empty());
        assert!(state.edited.gps_longitude.is_empty());
    }

    #[test]
    fn test_available_fields() {
        let meta = ImageMetadata {
            camera_make: Some("Canon".to_string()),
            ..Default::default()
        };
        let state = MetadataEditorState::from_image_metadata(&meta);

        let available = state.available_fields();
        assert!(!available.contains(&MetadataField::CameraMake));
        assert!(available.contains(&MetadataField::CameraModel));
        assert!(available.contains(&MetadataField::DateTaken));
    }

    #[test]
    fn test_reset_restores_visible_fields() {
        let meta = ImageMetadata {
            camera_make: Some("Canon".to_string()),
            ..Default::default()
        };
        let mut state = MetadataEditorState::from_image_metadata(&meta);

        // Add a new field
        state.show_field(MetadataField::Aperture);
        assert!(state.is_field_visible(&MetadataField::Aperture));

        // Reset should restore original visibility
        state.reset();
        assert!(!state.is_field_visible(&MetadataField::Aperture));
        assert!(state.is_field_visible(&MetadataField::CameraMake));
    }
}
