// SPDX-License-Identifier: MPL-2.0
//! Metadata preservation options for image editor save operations.
//!
//! This module provides configuration for how metadata should be handled
//! when saving edited images:
//! - Preserve original EXIF/XMP metadata
//! - Strip GPS coordinates for privacy
//! - Add software tag and modification date
//! - Reset orientation after rotation

use super::super::component::Transformation;

/// Options for metadata preservation when saving edited images.
#[derive(Debug, Clone)]
pub struct MetadataPreservationOptions {
    /// Whether to strip GPS coordinates from saved image.
    pub strip_gps: bool,
    /// Whether to add software tag and modification date.
    pub add_software_tag: bool,
    /// Whether orientation was changed (rotations/flips applied).
    /// When true, EXIF orientation tag will be reset to 1 (normal).
    orientation_changed: bool,
}

impl Default for MetadataPreservationOptions {
    fn default() -> Self {
        Self {
            strip_gps: false,
            add_software_tag: true, // Checked by default as per user request
            orientation_changed: false,
        }
    }
}

impl MetadataPreservationOptions {
    /// Updates the `orientation_changed` flag based on transformation history.
    ///
    /// Should be called whenever transformations are applied to track
    /// whether the orientation tag needs to be reset on save.
    pub fn update_from_transformations(&mut self, transformations: &[Transformation]) {
        self.orientation_changed = transformations.iter().any(|t| {
            matches!(
                t,
                Transformation::RotateLeft
                    | Transformation::RotateRight
                    | Transformation::FlipHorizontal
                    | Transformation::FlipVertical
            )
        });
    }

    /// Returns true if orientation transformations were applied.
    #[must_use]
    pub fn orientation_changed(&self) -> bool {
        self.orientation_changed
    }

    /// Sets the orientation changed flag directly.
    pub fn set_orientation_changed(&mut self, changed: bool) {
        self.orientation_changed = changed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::Rectangle;

    #[test]
    fn test_default_options() {
        let opts = MetadataPreservationOptions::default();
        assert!(!opts.strip_gps);
        assert!(opts.add_software_tag);
        assert!(!opts.orientation_changed());
    }

    #[test]
    fn test_orientation_detection_rotate_left() {
        let mut opts = MetadataPreservationOptions::default();
        let transformations = vec![Transformation::RotateLeft];
        opts.update_from_transformations(&transformations);
        assert!(opts.orientation_changed());
    }

    #[test]
    fn test_orientation_detection_rotate_right() {
        let mut opts = MetadataPreservationOptions::default();
        let transformations = vec![Transformation::RotateRight];
        opts.update_from_transformations(&transformations);
        assert!(opts.orientation_changed());
    }

    #[test]
    fn test_orientation_detection_flip_horizontal() {
        let mut opts = MetadataPreservationOptions::default();
        let transformations = vec![Transformation::FlipHorizontal];
        opts.update_from_transformations(&transformations);
        assert!(opts.orientation_changed());
    }

    #[test]
    fn test_orientation_detection_flip_vertical() {
        let mut opts = MetadataPreservationOptions::default();
        let transformations = vec![Transformation::FlipVertical];
        opts.update_from_transformations(&transformations);
        assert!(opts.orientation_changed());
    }

    #[test]
    fn test_orientation_not_changed_for_crop() {
        let mut opts = MetadataPreservationOptions::default();
        let transformations = vec![Transformation::Crop {
            rect: Rectangle::new(iced::Point::ORIGIN, iced::Size::new(100.0, 100.0)),
        }];
        opts.update_from_transformations(&transformations);
        assert!(!opts.orientation_changed());
    }

    #[test]
    fn test_orientation_not_changed_for_resize() {
        let mut opts = MetadataPreservationOptions::default();
        let transformations = vec![Transformation::Resize {
            width: 800,
            height: 600,
        }];
        opts.update_from_transformations(&transformations);
        assert!(!opts.orientation_changed());
    }

    #[test]
    fn test_orientation_detection_mixed_transformations() {
        let mut opts = MetadataPreservationOptions::default();
        let transformations = vec![
            Transformation::Resize {
                width: 800,
                height: 600,
            },
            Transformation::Crop {
                rect: Rectangle::new(iced::Point::ORIGIN, iced::Size::new(100.0, 100.0)),
            },
            Transformation::RotateLeft,
        ];
        opts.update_from_transformations(&transformations);
        assert!(opts.orientation_changed());
    }

    #[test]
    fn test_orientation_empty_transformations() {
        let mut opts = MetadataPreservationOptions::default();
        opts.update_from_transformations(&[]);
        assert!(!opts.orientation_changed());
    }

    #[test]
    fn test_set_orientation_changed() {
        let mut opts = MetadataPreservationOptions::default();
        assert!(!opts.orientation_changed());
        opts.set_orientation_changed(true);
        assert!(opts.orientation_changed());
        opts.set_orientation_changed(false);
        assert!(!opts.orientation_changed());
    }
}
