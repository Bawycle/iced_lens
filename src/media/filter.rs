// SPDX-License-Identifier: MPL-2.0
//! Media filtering for navigation.
//!
//! This module provides filter types for filtering media files during navigation.
//! Filters are combined with AND logic - all active filters must match for a file
//! to be included in navigation.
//!
//! # Available Filters
//!
//! - [`MediaTypeFilter`]: Filter by media type (images, videos, or all)
//! - [`DateRangeFilter`]: Filter by creation or modification date range
//!
//! # Example
//!
//! ```
//! use iced_lens::media::filter::{MediaFilter, MediaTypeFilter, DateRangeFilter, DateFilterField};
//! use std::time::SystemTime;
//!
//! let filter = MediaFilter {
//!     media_type: MediaTypeFilter::ImagesOnly,
//!     date_range: Some(DateRangeFilter {
//!         field: DateFilterField::Modified,
//!         start: Some(SystemTime::UNIX_EPOCH),
//!         end: None,
//!     }),
//! };
//!
//! assert!(filter.is_active());
//! ```

use crate::media::{detect_media_type, MediaType};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::SystemTime;

// =============================================================================
// Media Type Filter
// =============================================================================

/// Filter by media type.
///
/// This filter determines which types of media files are included during navigation.
/// Note that animated GIF and WebP files are classified as videos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum MediaTypeFilter {
    /// Show all media (images and videos).
    #[default]
    All,
    /// Show only images (excluding animated GIF/WebP which are videos).
    ImagesOnly,
    /// Show only videos (including animated GIF/WebP).
    VideosOnly,
}

impl MediaTypeFilter {
    /// Returns `true` if the file matches this filter.
    ///
    /// Uses extension-based detection via [`detect_media_type`].
    #[must_use]
    pub fn matches(&self, path: &Path) -> bool {
        match self {
            Self::All => true,
            Self::ImagesOnly => matches!(detect_media_type(path), Some(MediaType::Image)),
            Self::VideosOnly => matches!(detect_media_type(path), Some(MediaType::Video)),
        }
    }

    /// Returns `true` if this filter is active (not `All`).
    #[must_use]
    pub fn is_active(&self) -> bool {
        !matches!(self, Self::All)
    }
}

// =============================================================================
// Date Range Filter
// =============================================================================

/// Which date field to use for filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum DateFilterField {
    /// Filter by file modification date.
    #[default]
    Modified,
    /// Filter by file creation date.
    Created,
}

/// Filter by date range.
///
/// Filters files based on their creation or modification date.
/// Both `start` and `end` bounds are inclusive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DateRangeFilter {
    /// Which date field to use for filtering.
    pub field: DateFilterField,
    /// Start of the date range (inclusive). `None` means no lower bound.
    #[serde(
        serialize_with = "serialize_system_time_option",
        deserialize_with = "deserialize_system_time_option"
    )]
    pub start: Option<SystemTime>,
    /// End of the date range (inclusive). `None` means no upper bound.
    #[serde(
        serialize_with = "serialize_system_time_option",
        deserialize_with = "deserialize_system_time_option"
    )]
    pub end: Option<SystemTime>,
}

impl Default for DateRangeFilter {
    fn default() -> Self {
        Self {
            field: DateFilterField::default(),
            start: None,
            end: None,
        }
    }
}

impl DateRangeFilter {
    /// Returns `true` if the file matches this date range filter.
    ///
    /// Reads file metadata to get the appropriate timestamp.
    /// Returns `false` if metadata cannot be read or the date is not available.
    #[must_use]
    pub fn matches(&self, path: &Path) -> bool {
        let metadata = match std::fs::metadata(path) {
            Ok(m) => m,
            Err(_) => return false,
        };

        let file_time = match self.field {
            DateFilterField::Modified => metadata.modified().ok(),
            DateFilterField::Created => metadata.created().ok(),
        };

        let file_time = match file_time {
            Some(t) => t,
            None => return false,
        };

        // Check lower bound
        if let Some(start) = self.start {
            if file_time < start {
                return false;
            }
        }

        // Check upper bound
        if let Some(end) = self.end {
            if file_time > end {
                return false;
            }
        }

        true
    }

    /// Returns `true` if this filter has any active bounds.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.start.is_some() || self.end.is_some()
    }
}

// =============================================================================
// Composite Media Filter
// =============================================================================

/// Combined media filter with AND logic.
///
/// All active filters must match for a file to be included.
/// When no filters are active, all files match.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MediaFilter {
    /// Filter by media type.
    #[serde(default)]
    pub media_type: MediaTypeFilter,
    /// Filter by date range. `None` means no date filtering.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date_range: Option<DateRangeFilter>,
}

impl MediaFilter {
    /// Creates a new filter with no active criteria (matches all files).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if the file matches all active filters.
    ///
    /// Checks are ordered from cheapest to most expensive:
    /// 1. Media type (extension check, no I/O)
    /// 2. Date range (filesystem metadata read)
    #[must_use]
    pub fn matches(&self, path: &Path) -> bool {
        // Media type filter (cheapest - extension check only)
        if !self.media_type.matches(path) {
            return false;
        }

        // Date range filter (requires metadata read)
        if let Some(ref date_filter) = self.date_range {
            if date_filter.is_active() && !date_filter.matches(path) {
                return false;
            }
        }

        true
    }

    /// Returns `true` if any filter is active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.media_type.is_active()
            || self
                .date_range
                .as_ref()
                .map_or(false, DateRangeFilter::is_active)
    }

    /// Returns the number of active filter criteria.
    #[must_use]
    pub fn active_count(&self) -> usize {
        let mut count = 0;
        if self.media_type.is_active() {
            count += 1;
        }
        if self
            .date_range
            .as_ref()
            .map_or(false, DateRangeFilter::is_active)
        {
            count += 1;
        }
        count
    }

    /// Resets all filters to their default (inactive) state.
    pub fn clear(&mut self) {
        self.media_type = MediaTypeFilter::default();
        self.date_range = None;
    }
}

// =============================================================================
// SystemTime Serialization Helpers
// =============================================================================

/// Serialize `Option<SystemTime>` as Unix timestamp in seconds.
fn serialize_system_time_option<S>(
    time: &Option<SystemTime>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match time {
        Some(t) => {
            let duration = t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
            serializer.serialize_some(&duration.as_secs())
        }
        None => serializer.serialize_none(),
    }
}

/// Deserialize `Option<SystemTime>` from Unix timestamp in seconds.
fn deserialize_system_time_option<'de, D>(deserializer: D) -> Result<Option<SystemTime>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt: Option<u64> = Option::deserialize(deserializer)?;
    Ok(opt.map(|secs| SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(secs)))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::time::Duration;
    use tempfile::tempdir;

    fn create_test_file(dir: &Path, name: &str) -> std::path::PathBuf {
        let path = dir.join(name);
        let mut file = File::create(&path).expect("create test file");
        file.write_all(b"test content").expect("write test file");
        path
    }

    // -------------------------------------------------------------------------
    // MediaTypeFilter tests
    // -------------------------------------------------------------------------

    #[test]
    fn media_type_filter_all_matches_everything() {
        let filter = MediaTypeFilter::All;
        assert!(filter.matches(Path::new("test.jpg")));
        assert!(filter.matches(Path::new("test.mp4")));
        assert!(filter.matches(Path::new("test.png")));
        assert!(!filter.is_active());
    }

    #[test]
    fn media_type_filter_images_only() {
        let filter = MediaTypeFilter::ImagesOnly;
        assert!(filter.matches(Path::new("test.jpg")));
        assert!(filter.matches(Path::new("test.png")));
        assert!(filter.matches(Path::new("test.bmp")));
        assert!(!filter.matches(Path::new("test.mp4")));
        assert!(!filter.matches(Path::new("test.avi")));
        assert!(filter.is_active());
    }

    #[test]
    fn media_type_filter_videos_only() {
        let filter = MediaTypeFilter::VideosOnly;
        assert!(!filter.matches(Path::new("test.jpg")));
        assert!(!filter.matches(Path::new("test.png")));
        assert!(filter.matches(Path::new("test.mp4")));
        assert!(filter.matches(Path::new("test.avi")));
        assert!(filter.matches(Path::new("test.mkv")));
        assert!(filter.is_active());
    }

    // -------------------------------------------------------------------------
    // DateRangeFilter tests
    // -------------------------------------------------------------------------

    #[test]
    fn date_range_filter_no_bounds_matches_all() {
        let temp_dir = tempdir().expect("create temp dir");
        let file = create_test_file(temp_dir.path(), "test.jpg");

        let filter = DateRangeFilter::default();
        assert!(filter.matches(&file));
        assert!(!filter.is_active());
    }

    #[test]
    fn date_range_filter_with_start_bound() {
        let temp_dir = tempdir().expect("create temp dir");
        let file = create_test_file(temp_dir.path(), "test.jpg");

        // File was just created, so it should be after UNIX_EPOCH
        let filter = DateRangeFilter {
            field: DateFilterField::Modified,
            start: Some(SystemTime::UNIX_EPOCH),
            end: None,
        };
        assert!(filter.matches(&file));
        assert!(filter.is_active());

        // File should not match if start is in the future
        let future = SystemTime::now() + Duration::from_secs(86400);
        let filter_future = DateRangeFilter {
            field: DateFilterField::Modified,
            start: Some(future),
            end: None,
        };
        assert!(!filter_future.matches(&file));
    }

    #[test]
    fn date_range_filter_with_end_bound() {
        let temp_dir = tempdir().expect("create temp dir");
        let file = create_test_file(temp_dir.path(), "test.jpg");

        // File was just created, so it should be before future
        let future = SystemTime::now() + Duration::from_secs(86400);
        let filter = DateRangeFilter {
            field: DateFilterField::Modified,
            start: None,
            end: Some(future),
        };
        assert!(filter.matches(&file));

        // File should not match if end is in the past
        let past = SystemTime::UNIX_EPOCH + Duration::from_secs(1);
        let filter_past = DateRangeFilter {
            field: DateFilterField::Modified,
            start: None,
            end: Some(past),
        };
        assert!(!filter_past.matches(&file));
    }

    #[test]
    fn date_range_filter_nonexistent_file() {
        let filter = DateRangeFilter {
            field: DateFilterField::Modified,
            start: Some(SystemTime::UNIX_EPOCH),
            end: None,
        };
        assert!(!filter.matches(Path::new("/nonexistent/path/file.jpg")));
    }

    // -------------------------------------------------------------------------
    // MediaFilter (composite) tests
    // -------------------------------------------------------------------------

    #[test]
    fn media_filter_default_matches_all() {
        let filter = MediaFilter::default();
        assert!(!filter.is_active());
        assert_eq!(filter.active_count(), 0);
        assert!(filter.matches(Path::new("test.jpg")));
        assert!(filter.matches(Path::new("test.mp4")));
    }

    #[test]
    fn media_filter_with_media_type_only() {
        let filter = MediaFilter {
            media_type: MediaTypeFilter::ImagesOnly,
            date_range: None,
        };
        assert!(filter.is_active());
        assert_eq!(filter.active_count(), 1);
        assert!(filter.matches(Path::new("test.jpg")));
        assert!(!filter.matches(Path::new("test.mp4")));
    }

    #[test]
    fn media_filter_combined_and_logic() {
        let temp_dir = tempdir().expect("create temp dir");
        let image = create_test_file(temp_dir.path(), "test.jpg");
        let video = create_test_file(temp_dir.path(), "test.mp4");

        let filter = MediaFilter {
            media_type: MediaTypeFilter::ImagesOnly,
            date_range: Some(DateRangeFilter {
                field: DateFilterField::Modified,
                start: Some(SystemTime::UNIX_EPOCH),
                end: None,
            }),
        };

        assert!(filter.is_active());
        assert_eq!(filter.active_count(), 2);
        assert!(filter.matches(&image)); // Image + recent
        assert!(!filter.matches(&video)); // Video filtered out by media type
    }

    #[test]
    fn media_filter_clear() {
        let mut filter = MediaFilter {
            media_type: MediaTypeFilter::ImagesOnly,
            date_range: Some(DateRangeFilter {
                field: DateFilterField::Modified,
                start: Some(SystemTime::UNIX_EPOCH),
                end: None,
            }),
        };

        assert!(filter.is_active());
        filter.clear();
        assert!(!filter.is_active());
        assert_eq!(filter.active_count(), 0);
    }

    // -------------------------------------------------------------------------
    // Serialization tests
    // -------------------------------------------------------------------------

    #[test]
    fn media_filter_serialization_round_trip() {
        let filter = MediaFilter {
            media_type: MediaTypeFilter::VideosOnly,
            date_range: Some(DateRangeFilter {
                field: DateFilterField::Created,
                start: Some(SystemTime::UNIX_EPOCH + Duration::from_secs(1000)),
                end: Some(SystemTime::UNIX_EPOCH + Duration::from_secs(2000)),
            }),
        };

        let serialized = toml::to_string(&filter).expect("serialize");
        let deserialized: MediaFilter = toml::from_str(&serialized).expect("deserialize");

        assert_eq!(filter, deserialized);
    }

    #[test]
    fn media_filter_default_serialization() {
        let filter = MediaFilter::default();
        let serialized = toml::to_string(&filter).expect("serialize");

        // Default values should produce minimal output
        assert!(!serialized.contains("date_range"));
    }
}
