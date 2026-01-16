// SPDX-License-Identifier: MPL-2.0
//! Media filtering types for the domain layer.
//!
//! This module contains pure filter types without I/O operations.
//! The `matches()` methods that require filesystem access are implemented
//! in the infrastructure/media layer.
//!
//! # Available Filters
//!
//! - [`MediaTypeFilter`]: Filter by media type (images, videos, or all)
//! - [`DateRangeFilter`]: Filter by creation or modification date range
//! - [`MediaFilter`]: Combined filter with AND logic

use std::time::SystemTime;

// =============================================================================
// Media Type Filter
// =============================================================================

/// Filter by media type.
///
/// This filter determines which types of media files are included during navigation.
/// Note that animated GIF and WebP files are classified as videos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
    /// Returns `true` if this filter matches the given media type.
    ///
    /// This is a pure domain check without I/O.
    #[must_use]
    pub fn matches_type(&self, media_type: super::MediaType) -> bool {
        match self {
            Self::All => true,
            Self::ImagesOnly => matches!(media_type, super::MediaType::Image),
            Self::VideosOnly => matches!(media_type, super::MediaType::Video),
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DateRangeFilter {
    /// Which date field to use for filtering.
    pub field: DateFilterField,
    /// Start of the date range (inclusive). `None` means no lower bound.
    pub start: Option<SystemTime>,
    /// End of the date range (inclusive). `None` means no upper bound.
    pub end: Option<SystemTime>,
}

impl DateRangeFilter {
    /// Returns `true` if the given timestamp matches this date range filter.
    ///
    /// This is a pure domain check - the timestamp should be obtained
    /// from filesystem metadata by the infrastructure layer.
    #[must_use]
    pub fn matches_time(&self, file_time: SystemTime) -> bool {
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
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MediaFilter {
    /// Filter by media type.
    pub media_type: MediaTypeFilter,
    /// Filter by date range. `None` means no date filtering.
    pub date_range: Option<DateRangeFilter>,
}

impl MediaFilter {
    /// Creates a new filter with no active criteria (matches all files).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if any filter is active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.media_type.is_active()
            || self
                .date_range
                .as_ref()
                .is_some_and(DateRangeFilter::is_active)
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
            .is_some_and(DateRangeFilter::is_active)
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
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // -------------------------------------------------------------------------
    // MediaTypeFilter tests
    // -------------------------------------------------------------------------

    #[test]
    fn media_type_filter_all_matches_everything() {
        use super::super::MediaType;

        let filter = MediaTypeFilter::All;
        assert!(filter.matches_type(MediaType::Image));
        assert!(filter.matches_type(MediaType::Video));
        assert!(!filter.is_active());
    }

    #[test]
    fn media_type_filter_images_only() {
        use super::super::MediaType;

        let filter = MediaTypeFilter::ImagesOnly;
        assert!(filter.matches_type(MediaType::Image));
        assert!(!filter.matches_type(MediaType::Video));
        assert!(filter.is_active());
    }

    #[test]
    fn media_type_filter_videos_only() {
        use super::super::MediaType;

        let filter = MediaTypeFilter::VideosOnly;
        assert!(!filter.matches_type(MediaType::Image));
        assert!(filter.matches_type(MediaType::Video));
        assert!(filter.is_active());
    }

    // -------------------------------------------------------------------------
    // DateRangeFilter tests
    // -------------------------------------------------------------------------

    #[test]
    fn date_range_filter_no_bounds_matches_all() {
        let filter = DateRangeFilter::default();
        let now = SystemTime::now();
        assert!(filter.matches_time(now));
        assert!(!filter.is_active());
    }

    #[test]
    fn date_range_filter_with_start_bound() {
        let now = SystemTime::now();
        let past = SystemTime::UNIX_EPOCH;

        let filter = DateRangeFilter {
            field: DateFilterField::Modified,
            start: Some(past),
            end: None,
        };

        assert!(filter.matches_time(now));
        assert!(filter.is_active());

        // Time before start should not match
        let very_past = SystemTime::UNIX_EPOCH;
        let filter_future_start = DateRangeFilter {
            field: DateFilterField::Modified,
            start: Some(now + Duration::from_secs(86400)),
            end: None,
        };
        assert!(!filter_future_start.matches_time(very_past));
    }

    #[test]
    fn date_range_filter_with_end_bound() {
        let now = SystemTime::now();
        let future = now + Duration::from_secs(86400);

        let filter = DateRangeFilter {
            field: DateFilterField::Modified,
            start: None,
            end: Some(future),
        };

        assert!(filter.matches_time(now));

        // Time after end should not match
        let far_future = now + Duration::from_secs(86400 * 2);
        assert!(!filter.matches_time(far_future));
    }

    #[test]
    fn date_range_filter_with_both_bounds() {
        let start = SystemTime::UNIX_EPOCH + Duration::from_secs(1000);
        let end = SystemTime::UNIX_EPOCH + Duration::from_secs(2000);
        let middle = SystemTime::UNIX_EPOCH + Duration::from_secs(1500);

        let filter = DateRangeFilter {
            field: DateFilterField::Modified,
            start: Some(start),
            end: Some(end),
        };

        assert!(filter.matches_time(middle));
        assert!(filter.matches_time(start)); // Inclusive
        assert!(filter.matches_time(end)); // Inclusive
        assert!(!filter.matches_time(SystemTime::UNIX_EPOCH));
        assert!(!filter.matches_time(SystemTime::UNIX_EPOCH + Duration::from_secs(3000)));
    }

    // -------------------------------------------------------------------------
    // MediaFilter (composite) tests
    // -------------------------------------------------------------------------

    #[test]
    fn media_filter_default_is_inactive() {
        let filter = MediaFilter::default();
        assert!(!filter.is_active());
        assert_eq!(filter.active_count(), 0);
    }

    #[test]
    fn media_filter_with_media_type_only() {
        let filter = MediaFilter {
            media_type: MediaTypeFilter::ImagesOnly,
            date_range: None,
        };
        assert!(filter.is_active());
        assert_eq!(filter.active_count(), 1);
    }

    #[test]
    fn media_filter_with_both_filters() {
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
}
