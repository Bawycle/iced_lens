// SPDX-License-Identifier: MPL-2.0
//! Media navigation module for managing media lists and navigation state.
//!
//! This module provides a shared `MediaNavigator` that can be used by both
//! the viewer and editor components to maintain a single source of truth
//! for media list and current media path.

use crate::config::SortOrder;
use crate::directory_scanner::MediaList;
use crate::error::Result;
use crate::media::filter::MediaFilter;
use crate::media::{detect_media_type, MediaType};
use std::path::{Path, PathBuf};

/// Navigation state information for UI rendering.
///
/// This struct contains all the information needed by the viewer to render
/// navigation controls without needing direct access to the media list.
/// It acts as a snapshot of the current navigation state.
// Allow excessive bools: read-only UI snapshot with orthogonal capability flags.
// has_next/has_previous differ from at_first/at_last due to wrap-around logic.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, Default)]
pub struct NavigationInfo {
    /// Whether there is a next media to navigate to.
    pub has_next: bool,
    /// Whether there is a previous media to navigate to.
    pub has_previous: bool,
    /// Whether the current media is the first in the list.
    pub at_first: bool,
    /// Whether the current media is the last in the list.
    pub at_last: bool,
    /// Current position in the list (0-indexed), if set.
    pub current_index: Option<usize>,
    /// Total number of media items in the list.
    pub total_count: usize,
    /// Number of media items matching the current filter.
    /// Same as `total_count` when no filter is active.
    pub filtered_count: usize,
    /// Whether a filter is currently active.
    pub filter_active: bool,
}

/// Manages navigation through a list of media files in a directory.
///
/// This component encapsulates both the media list and the current media path,
/// providing a single source of truth for media navigation shared between
/// viewer and editor components.
///
/// # Filtering
///
/// The navigator supports media filtering via [`MediaFilter`]. When a filter is active,
/// the `peek_*_filtered` methods will only return paths that match the filter criteria.
/// The editor uses `peek_*_image` methods which ignore user filters entirely.
#[derive(Debug, Clone, PartialEq)]
pub struct MediaNavigator {
    /// List of media files in the current directory
    media_list: MediaList,
    /// Path to the currently selected media
    current_media_path: Option<PathBuf>,
    /// Current filter criteria for navigation
    filter: MediaFilter,
}

impl MediaNavigator {
    /// Creates a new empty `MediaNavigator`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            media_list: MediaList::new(),
            current_media_path: None,
            filter: MediaFilter::default(),
        }
    }

    /// Sets the media list directly (used for async scan results).
    pub fn set_media_list(&mut self, media_list: MediaList) {
        self.media_list = media_list;
    }

    /// Scans the directory containing the given media file and updates the media list.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be read or the path has no parent directory.
    pub fn scan_directory(&mut self, current_file: &Path, sort_order: SortOrder) -> Result<()> {
        self.media_list = MediaList::scan_directory(current_file, sort_order)?;
        self.current_media_path = Some(current_file.to_path_buf());
        Ok(())
    }

    /// Scans a directory directly for media files and selects the first one.
    ///
    /// Returns `Ok(Some(path))` with the first media file path if any media is found,
    /// or `Ok(None)` if the directory contains no supported media files.
    ///
    /// If a filter is active, returns the first media that matches the filter.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be read.
    pub fn scan_from_directory(
        &mut self,
        directory: &Path,
        sort_order: SortOrder,
    ) -> Result<Option<PathBuf>> {
        self.media_list = MediaList::scan_directory_direct(directory, sort_order)?;

        // Find the first media matching the active filter (or first overall if no filter)
        let first_matching = if self.filter.is_active() {
            let total = self.media_list.len();
            (0..total)
                .filter_map(|i| self.media_list.get(i))
                .find(|path| self.filter.matches(path))
                .map(std::path::Path::to_path_buf)
        } else {
            self.media_list.first().map(std::path::Path::to_path_buf)
        };

        if let Some(path) = first_matching {
            self.media_list.set_current(&path);
            self.current_media_path = Some(path.clone());
            Ok(Some(path))
        } else {
            self.current_media_path = None;
            Ok(None)
        }
    }

    /// Returns the path to the current media, if set.
    #[must_use]
    pub fn current_media_path(&self) -> Option<&Path> {
        self.current_media_path.as_deref()
    }

    /// Sets the current media path and updates the media list index.
    pub fn set_current_media_path(&mut self, path: PathBuf) {
        self.media_list.set_current(&path);
        self.current_media_path = Some(path);
    }

    /// Returns the next media path WITHOUT updating current position.
    ///
    /// Use this for pessimistic navigation where position is confirmed after load.
    /// Returns `None` if there are no media in the list.
    /// Wraps around to the first media when at the last media.
    #[must_use]
    pub fn peek_next(&self) -> Option<PathBuf> {
        self.media_list.next().map(std::path::Path::to_path_buf)
    }

    /// Returns the previous media path WITHOUT updating current position.
    ///
    /// Use this for pessimistic navigation where position is confirmed after load.
    /// Returns `None` if there are no media in the list.
    /// Wraps around to the last media when at the first media.
    #[must_use]
    pub fn peek_previous(&self) -> Option<PathBuf> {
        self.media_list.previous().map(std::path::Path::to_path_buf)
    }

    /// Returns the n-th next media path WITHOUT updating current position.
    ///
    /// `skip_count = 0` returns immediate next, `skip_count = 1` skips one file, etc.
    /// Returns `None` if there are no media in the list.
    /// Wraps around when reaching the end.
    #[must_use]
    pub fn peek_nth_next(&self, skip_count: usize) -> Option<PathBuf> {
        self.media_list
            .peek_nth_next(skip_count)
            .map(std::path::Path::to_path_buf)
    }

    /// Returns the n-th previous media path WITHOUT updating current position.
    ///
    /// `skip_count = 0` returns immediate previous, `skip_count = 1` skips one file, etc.
    /// Returns `None` if there are no media in the list.
    /// Wraps around when reaching the start.
    #[must_use]
    pub fn peek_nth_previous(&self, skip_count: usize) -> Option<PathBuf> {
        self.media_list
            .peek_nth_previous(skip_count)
            .map(std::path::Path::to_path_buf)
    }

    /// Returns the next image path (skipping videos) WITHOUT updating position.
    ///
    /// Returns `None` if there are no images in the list (only videos).
    /// Wraps around to the first image when at the last.
    #[must_use]
    pub fn peek_next_image(&self) -> Option<PathBuf> {
        self.peek_nth_next_image(0)
    }

    /// Returns the previous image path (skipping videos) WITHOUT updating position.
    ///
    /// Returns `None` if there are no images in the list (only videos).
    /// Wraps around to the last image when at the first.
    #[must_use]
    pub fn peek_previous_image(&self) -> Option<PathBuf> {
        self.peek_nth_previous_image(0)
    }

    /// Returns the n-th next image path (skipping videos) WITHOUT updating position.
    ///
    /// `skip_count = 0` returns immediate next image, `skip_count = 1` skips one image, etc.
    /// Videos are always skipped and don't count toward `skip_count`.
    /// Returns `None` if there are no images in the list.
    /// Wraps around when reaching the end.
    #[must_use]
    pub fn peek_nth_next_image(&self, skip_count: usize) -> Option<PathBuf> {
        let current_index = self.media_list.current_index()?;
        let total = self.len();

        if total == 0 {
            return None;
        }

        let mut images_found = 0;
        // Try up to `total` times to find enough images (avoid infinite loop)
        for offset in 1..=total {
            let candidate_index = (current_index + offset) % total;
            if let Some(path) = self.media_list.get(candidate_index) {
                if matches!(detect_media_type(path), Some(MediaType::Image)) {
                    if images_found == skip_count {
                        return Some(path.to_path_buf());
                    }
                    images_found += 1;
                }
            }
        }
        None
    }

    /// Returns the n-th previous image path (skipping videos) WITHOUT updating position.
    ///
    /// `skip_count = 0` returns immediate previous image, `skip_count = 1` skips one image, etc.
    /// Videos are always skipped and don't count toward `skip_count`.
    /// Returns `None` if there are no images in the list.
    /// Wraps around when reaching the start.
    #[must_use]
    pub fn peek_nth_previous_image(&self, skip_count: usize) -> Option<PathBuf> {
        let current_index = self.media_list.current_index()?;
        let total = self.len();

        if total == 0 {
            return None;
        }

        let mut images_found = 0;
        // Try up to `total` times to find enough images (avoid infinite loop)
        for offset in 1..=total {
            let candidate_index = if offset > current_index {
                total - (offset - current_index)
            } else {
                current_index - offset
            };
            if let Some(path) = self.media_list.get(candidate_index) {
                if matches!(detect_media_type(path), Some(MediaType::Image)) {
                    if images_found == skip_count {
                        return Some(path.to_path_buf());
                    }
                    images_found += 1;
                }
            }
        }
        None
    }

    /// Confirms navigation to a path after successful load.
    ///
    /// Updates `current_media_path` and the internal index.
    /// This should be called after the media at the peeked path has been
    /// successfully loaded.
    pub fn confirm_navigation(&mut self, path: &Path) {
        self.media_list.set_current(path);
        self.current_media_path = Some(path.to_path_buf());
    }

    /// Checks if there is a next media available.
    #[must_use]
    pub fn has_next(&self) -> bool {
        self.media_list.next().is_some()
    }

    /// Checks if there is a previous media available.
    #[must_use]
    pub fn has_previous(&self) -> bool {
        self.media_list.previous().is_some()
    }

    /// Checks if the current media is the first in the list.
    #[must_use]
    pub fn is_at_first(&self) -> bool {
        self.media_list.is_at_first()
    }

    /// Checks if the current media is the last in the list.
    #[must_use]
    pub fn is_at_last(&self) -> bool {
        self.media_list.is_at_last()
    }

    /// Returns the total number of media in the list.
    #[must_use]
    pub fn len(&self) -> usize {
        self.media_list.len()
    }

    /// Checks if the media list is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.media_list.is_empty()
    }

    /// Returns the current index in the media list, if set.
    #[must_use]
    pub fn current_index(&self) -> Option<usize> {
        self.media_list.current_index()
    }

    /// Returns a snapshot of the current navigation state for UI rendering.
    ///
    /// This method provides all the information needed by the viewer to render
    /// navigation controls without needing direct access to the media list.
    #[must_use]
    pub fn navigation_info(&self) -> NavigationInfo {
        NavigationInfo {
            has_next: self.has_next(),
            has_previous: self.has_previous(),
            at_first: self.is_at_first(),
            at_last: self.is_at_last(),
            current_index: self.current_index(),
            total_count: self.len(),
            filtered_count: self.filtered_count(),
            filter_active: self.filter.is_active(),
        }
    }

    // =========================================================================
    // Filter Methods
    // =========================================================================

    /// Returns a reference to the current filter.
    #[must_use]
    pub fn filter(&self) -> &MediaFilter {
        &self.filter
    }

    /// Sets the filter criteria for navigation.
    ///
    /// After setting a filter, use `peek_*_filtered` methods to navigate
    /// only through media that match the filter criteria.
    pub fn set_filter(&mut self, filter: MediaFilter) {
        self.filter = filter;
    }

    /// Clears all filter criteria (resets to match all media).
    pub fn clear_filter(&mut self) {
        self.filter.clear();
    }

    /// Returns the number of media files matching the current filter.
    ///
    /// Returns total count when no filter is active.
    #[must_use]
    pub fn filtered_count(&self) -> usize {
        if !self.filter.is_active() {
            return self.len();
        }

        let total = self.len();
        (0..total)
            .filter_map(|i| self.media_list.get(i))
            .filter(|path| self.filter.matches(path))
            .count()
    }

    /// Returns the next media path matching the filter WITHOUT updating position.
    ///
    /// Use this for filtered navigation in the viewer.
    /// Returns `None` if no media matches the filter.
    /// Wraps around when reaching the end.
    #[must_use]
    pub fn peek_next_filtered(&self) -> Option<PathBuf> {
        self.peek_nth_next_filtered(0)
    }

    /// Returns the previous media path matching the filter WITHOUT updating position.
    ///
    /// Use this for filtered navigation in the viewer.
    /// Returns `None` if no media matches the filter.
    /// Wraps around when reaching the start.
    #[must_use]
    pub fn peek_previous_filtered(&self) -> Option<PathBuf> {
        self.peek_nth_previous_filtered(0)
    }

    /// Returns the n-th next media path matching the filter WITHOUT updating position.
    ///
    /// `skip_count = 0` returns immediate next match, `skip_count = 1` skips one match, etc.
    /// Non-matching media are skipped and don't count toward `skip_count`.
    /// Returns `None` if no media matches the filter.
    /// Wraps around when reaching the end.
    #[must_use]
    pub fn peek_nth_next_filtered(&self, skip_count: usize) -> Option<PathBuf> {
        // If no filter is active, use the unfiltered navigation
        if !self.filter.is_active() {
            return self.peek_nth_next(skip_count);
        }

        let current_index = self.media_list.current_index()?;
        let total = self.len();

        if total == 0 {
            return None;
        }

        let mut matches_found = 0;
        // Try up to `total` times to find enough matches (avoid infinite loop)
        for offset in 1..=total {
            let candidate_index = (current_index + offset) % total;
            if let Some(path) = self.media_list.get(candidate_index) {
                if self.filter.matches(path) {
                    if matches_found == skip_count {
                        return Some(path.to_path_buf());
                    }
                    matches_found += 1;
                }
            }
        }
        None
    }

    /// Returns the n-th previous media path matching the filter WITHOUT updating position.
    ///
    /// `skip_count = 0` returns immediate previous match, `skip_count = 1` skips one match, etc.
    /// Non-matching media are skipped and don't count toward `skip_count`.
    /// Returns `None` if no media matches the filter.
    /// Wraps around when reaching the start.
    #[must_use]
    pub fn peek_nth_previous_filtered(&self, skip_count: usize) -> Option<PathBuf> {
        // If no filter is active, use the unfiltered navigation
        if !self.filter.is_active() {
            return self.peek_nth_previous(skip_count);
        }

        let current_index = self.media_list.current_index()?;
        let total = self.len();

        if total == 0 {
            return None;
        }

        let mut matches_found = 0;
        // Try up to `total` times to find enough matches (avoid infinite loop)
        for offset in 1..=total {
            let candidate_index = if offset > current_index {
                total - (offset - current_index)
            } else {
                current_index - offset
            };
            if let Some(path) = self.media_list.get(candidate_index) {
                if self.filter.matches(path) {
                    if matches_found == skip_count {
                        return Some(path.to_path_buf());
                    }
                    matches_found += 1;
                }
            }
        }
        None
    }

    /// Checks if the current media matches the active filter.
    ///
    /// Returns `true` if no filter is active or if the current media matches.
    /// Returns `false` if filtered out or if there's no current media.
    #[must_use]
    pub fn current_matches_filter(&self) -> bool {
        match &self.current_media_path {
            Some(path) => self.filter.matches(path),
            None => false,
        }
    }
}

impl Default for MediaNavigator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::tempdir;

    fn create_test_image(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        let mut file = fs::File::create(&path).expect("failed to create test file");
        file.write_all(b"fake image data")
            .expect("failed to write test file");
        path
    }

    #[test]
    fn new_navigator_is_empty() {
        let nav = MediaNavigator::new();
        assert!(nav.is_empty());
        assert_eq!(nav.len(), 0);
        assert_eq!(nav.current_media_path(), None);
    }

    #[test]
    fn scan_directory_finds_media() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let _img2 = create_test_image(temp_dir.path(), "b.png");
        let _img3 = create_test_image(temp_dir.path(), "c.gif");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img1, SortOrder::Alphabetical)
            .expect("scan failed");

        assert_eq!(nav.len(), 3);
        assert_eq!(nav.current_media_path(), Some(img1.as_path()));
    }

    #[test]
    fn peek_next_returns_next_without_changing_state() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let img2 = create_test_image(temp_dir.path(), "b.png");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img1, SortOrder::Alphabetical)
            .expect("scan failed");

        // Peek should return next without changing current position
        let next = nav.peek_next();
        assert_eq!(next.as_deref(), Some(img2.as_path()));
        // Current should still be img1 (pessimistic update)
        assert_eq!(nav.current_media_path(), Some(img1.as_path()));
    }

    #[test]
    fn peek_previous_returns_previous_without_changing_state() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let img2 = create_test_image(temp_dir.path(), "b.png");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img2, SortOrder::Alphabetical)
            .expect("scan failed");

        // Peek should return previous without changing current position
        let prev = nav.peek_previous();
        assert_eq!(prev.as_deref(), Some(img1.as_path()));
        // Current should still be img2 (pessimistic update)
        assert_eq!(nav.current_media_path(), Some(img2.as_path()));
    }

    #[test]
    fn confirm_navigation_updates_state() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let img2 = create_test_image(temp_dir.path(), "b.png");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img1, SortOrder::Alphabetical)
            .expect("scan failed");

        // Peek and then confirm
        let next = nav.peek_next().unwrap();
        nav.confirm_navigation(&next);

        // Now state should be updated
        assert_eq!(nav.current_media_path(), Some(img2.as_path()));
        assert_eq!(nav.current_index(), Some(1));
    }

    #[test]
    fn peek_next_wraps_around() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let img2 = create_test_image(temp_dir.path(), "b.png");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img2, SortOrder::Alphabetical)
            .expect("scan failed");

        let next = nav.peek_next();
        assert_eq!(next.as_deref(), Some(img1.as_path())); // wraps to first
    }

    #[test]
    fn peek_previous_wraps_around() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let img2 = create_test_image(temp_dir.path(), "b.png");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img1, SortOrder::Alphabetical)
            .expect("scan failed");

        let prev = nav.peek_previous();
        assert_eq!(prev.as_deref(), Some(img2.as_path())); // wraps to last
    }

    #[test]
    fn has_next_and_has_previous_work_correctly() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let _img2 = create_test_image(temp_dir.path(), "b.png");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img1, SortOrder::Alphabetical)
            .expect("scan failed");

        assert!(nav.has_next());
        assert!(nav.has_previous()); // wraps around
    }

    #[test]
    fn is_at_first_and_is_at_last_detect_boundaries() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let img2 = create_test_image(temp_dir.path(), "b.png");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img1, SortOrder::Alphabetical)
            .expect("scan failed");

        assert!(nav.is_at_first());
        assert!(!nav.is_at_last());

        nav.set_current_media_path(img2);
        assert!(!nav.is_at_first());
        assert!(nav.is_at_last());
    }

    #[test]
    fn set_current_media_path_updates_state() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let img2 = create_test_image(temp_dir.path(), "b.png");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img1, SortOrder::Alphabetical)
            .expect("scan failed");

        nav.set_current_media_path(img2.clone());
        assert_eq!(nav.current_media_path(), Some(img2.as_path()));
        assert_eq!(nav.current_index(), Some(1));
    }

    #[test]
    fn empty_navigator_returns_none_on_peek() {
        let nav = MediaNavigator::new();
        assert_eq!(nav.peek_next(), None);
        assert_eq!(nav.peek_previous(), None);
        assert!(!nav.has_next());
        assert!(!nav.has_previous());
    }

    fn create_test_video(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        let mut file = fs::File::create(&path).expect("failed to create test file");
        file.write_all(b"fake video data")
            .expect("failed to write test file");
        path
    }

    #[test]
    fn peek_next_image_skips_videos() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let _vid1 = create_test_video(temp_dir.path(), "b.mp4");
        let _vid2 = create_test_video(temp_dir.path(), "c.mp4");
        let img2 = create_test_image(temp_dir.path(), "d.png");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img1, SortOrder::Alphabetical)
            .expect("scan failed");

        // Should skip b.mp4 and c.mp4, return d.png WITHOUT changing state
        let next = nav.peek_next_image();
        assert_eq!(next.as_deref(), Some(img2.as_path()));
        // State should not change (pessimistic update)
        assert_eq!(nav.current_media_path(), Some(img1.as_path()));
    }

    #[test]
    fn peek_previous_image_skips_videos() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let _vid1 = create_test_video(temp_dir.path(), "b.mp4");
        let _vid2 = create_test_video(temp_dir.path(), "c.mp4");
        let img2 = create_test_image(temp_dir.path(), "d.png");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img2, SortOrder::Alphabetical)
            .expect("scan failed");

        // Should skip c.mp4 and b.mp4, return a.jpg WITHOUT changing state
        let prev = nav.peek_previous_image();
        assert_eq!(prev.as_deref(), Some(img1.as_path()));
        // State should not change (pessimistic update)
        assert_eq!(nav.current_media_path(), Some(img2.as_path()));
    }

    #[test]
    fn peek_next_image_wraps_around_skipping_videos() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let _vid1 = create_test_video(temp_dir.path(), "b.mp4");
        let img2 = create_test_image(temp_dir.path(), "c.png");
        let _vid2 = create_test_video(temp_dir.path(), "d.mp4");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img2, SortOrder::Alphabetical)
            .expect("scan failed");

        // From c.png, should skip d.mp4, wrap to a.jpg (skipping b.mp4)
        let next = nav.peek_next_image();
        assert_eq!(next.as_deref(), Some(img1.as_path()));
    }

    #[test]
    fn peek_previous_image_wraps_around_skipping_videos() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let _vid1 = create_test_video(temp_dir.path(), "a.mp4");
        let img1 = create_test_image(temp_dir.path(), "b.jpg");
        let _vid2 = create_test_video(temp_dir.path(), "c.mp4");
        let img2 = create_test_image(temp_dir.path(), "d.png");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img1, SortOrder::Alphabetical)
            .expect("scan failed");

        // From b.jpg, should skip a.mp4, wrap to d.png (skipping c.mp4)
        let prev = nav.peek_previous_image();
        assert_eq!(prev.as_deref(), Some(img2.as_path()));
    }

    #[test]
    fn peek_next_image_returns_same_if_only_one_image() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let _vid1 = create_test_video(temp_dir.path(), "a.mp4");
        let img1 = create_test_image(temp_dir.path(), "b.jpg");
        let _vid2 = create_test_video(temp_dir.path(), "c.mp4");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img1, SortOrder::Alphabetical)
            .expect("scan failed");

        // Only one image, should wrap back to itself
        let next = nav.peek_next_image();
        assert_eq!(next.as_deref(), Some(img1.as_path()));
    }

    #[test]
    fn peek_previous_image_returns_same_if_only_one_image() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let _vid1 = create_test_video(temp_dir.path(), "a.mp4");
        let img1 = create_test_image(temp_dir.path(), "b.jpg");
        let _vid2 = create_test_video(temp_dir.path(), "c.mp4");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img1, SortOrder::Alphabetical)
            .expect("scan failed");

        // Only one image, should wrap back to itself
        let prev = nav.peek_previous_image();
        assert_eq!(prev.as_deref(), Some(img1.as_path()));
    }

    #[test]
    fn peek_next_image_returns_none_for_empty_navigator() {
        let nav = MediaNavigator::new();
        assert_eq!(nav.peek_next_image(), None);
    }

    #[test]
    fn peek_previous_image_returns_none_for_empty_navigator() {
        let nav = MediaNavigator::new();
        assert_eq!(nav.peek_previous_image(), None);
    }

    // Tests for scan_from_directory
    #[test]
    fn scan_from_directory_finds_first_media() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img_a = create_test_image(temp_dir.path(), "a.jpg");
        let _img_b = create_test_image(temp_dir.path(), "b.png");
        let _vid = create_test_video(temp_dir.path(), "c.mp4");

        let mut nav = MediaNavigator::new();
        let result = nav
            .scan_from_directory(temp_dir.path(), SortOrder::Alphabetical)
            .expect("scan failed");

        assert_eq!(result, Some(img_a.clone()));
        assert_eq!(nav.current_media_path(), Some(img_a.as_path()));
        assert_eq!(nav.len(), 3);
    }

    #[test]
    fn scan_from_directory_returns_none_for_empty_directory() {
        let temp_dir = tempdir().expect("failed to create temp dir");

        let mut nav = MediaNavigator::new();
        let result = nav
            .scan_from_directory(temp_dir.path(), SortOrder::Alphabetical)
            .expect("scan failed");

        assert_eq!(result, None);
        assert_eq!(nav.current_media_path(), None);
        assert!(nav.is_empty());
    }

    #[test]
    fn scan_from_directory_returns_none_for_no_media_files() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        // Create non-media files
        let txt_path = temp_dir.path().join("readme.txt");
        fs::File::create(&txt_path).expect("failed to create txt file");

        let mut nav = MediaNavigator::new();
        let result = nav
            .scan_from_directory(temp_dir.path(), SortOrder::Alphabetical)
            .expect("scan failed");

        assert_eq!(result, None);
        assert_eq!(nav.current_media_path(), None);
    }

    #[test]
    fn scan_from_directory_enables_navigation() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img_a = create_test_image(temp_dir.path(), "a.jpg");
        let img_b = create_test_image(temp_dir.path(), "b.png");
        let img_c = create_test_image(temp_dir.path(), "c.gif");

        let mut nav = MediaNavigator::new();
        nav.scan_from_directory(temp_dir.path(), SortOrder::Alphabetical)
            .expect("scan failed");

        // Should start at first media
        assert_eq!(nav.current_media_path(), Some(img_a.as_path()));

        // Peek next and confirm
        let next = nav.peek_next().unwrap();
        assert_eq!(next.as_path(), img_b.as_path());
        nav.confirm_navigation(&next);
        assert_eq!(nav.current_media_path(), Some(img_b.as_path()));

        // Peek next again and confirm
        let next = nav.peek_next().unwrap();
        assert_eq!(next.as_path(), img_c.as_path());
        nav.confirm_navigation(&next);
        assert_eq!(nav.current_media_path(), Some(img_c.as_path()));
    }

    // -------------------------------------------------------------------------
    // Filter tests
    // -------------------------------------------------------------------------

    #[test]
    fn filter_default_is_inactive() {
        let nav = MediaNavigator::new();
        assert!(!nav.filter().is_active());
    }

    #[test]
    fn set_filter_and_clear_filter() {
        use crate::media::filter::MediaTypeFilter;

        let mut nav = MediaNavigator::new();
        let filter = MediaFilter {
            media_type: MediaTypeFilter::ImagesOnly,
            date_range: None,
        };

        nav.set_filter(filter);
        assert!(nav.filter().is_active());

        nav.clear_filter();
        assert!(!nav.filter().is_active());
    }

    #[test]
    fn filtered_count_with_no_filter() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let _vid1 = create_test_video(temp_dir.path(), "b.mp4");
        let _img2 = create_test_image(temp_dir.path(), "c.png");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img1, SortOrder::Alphabetical)
            .expect("scan failed");

        // No filter active, filtered_count should equal total count
        assert_eq!(nav.filtered_count(), 3);
        assert_eq!(nav.filtered_count(), nav.len());
    }

    #[test]
    fn filtered_count_with_images_only_filter() {
        use crate::media::filter::MediaTypeFilter;

        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let _vid1 = create_test_video(temp_dir.path(), "b.mp4");
        let _img2 = create_test_image(temp_dir.path(), "c.png");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img1, SortOrder::Alphabetical)
            .expect("scan failed");

        nav.set_filter(MediaFilter {
            media_type: MediaTypeFilter::ImagesOnly,
            date_range: None,
        });

        assert_eq!(nav.filtered_count(), 2); // Only images
    }

    #[test]
    fn peek_next_filtered_skips_non_matching() {
        use crate::media::filter::MediaTypeFilter;

        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let _vid1 = create_test_video(temp_dir.path(), "b.mp4");
        let _vid2 = create_test_video(temp_dir.path(), "c.mp4");
        let img2 = create_test_image(temp_dir.path(), "d.png");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img1, SortOrder::Alphabetical)
            .expect("scan failed");

        nav.set_filter(MediaFilter {
            media_type: MediaTypeFilter::ImagesOnly,
            date_range: None,
        });

        // Should skip b.mp4 and c.mp4, return d.png
        let next = nav.peek_next_filtered();
        assert_eq!(next.as_deref(), Some(img2.as_path()));
        // State should not change (pessimistic update)
        assert_eq!(nav.current_media_path(), Some(img1.as_path()));
    }

    #[test]
    fn peek_previous_filtered_skips_non_matching() {
        use crate::media::filter::MediaTypeFilter;

        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let _vid1 = create_test_video(temp_dir.path(), "b.mp4");
        let _vid2 = create_test_video(temp_dir.path(), "c.mp4");
        let img2 = create_test_image(temp_dir.path(), "d.png");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img2, SortOrder::Alphabetical)
            .expect("scan failed");

        nav.set_filter(MediaFilter {
            media_type: MediaTypeFilter::ImagesOnly,
            date_range: None,
        });

        // Should skip c.mp4 and b.mp4, return a.jpg
        let prev = nav.peek_previous_filtered();
        assert_eq!(prev.as_deref(), Some(img1.as_path()));
        // State should not change (pessimistic update)
        assert_eq!(nav.current_media_path(), Some(img2.as_path()));
    }

    #[test]
    fn peek_next_filtered_without_filter_uses_unfiltered() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let vid1 = create_test_video(temp_dir.path(), "b.mp4");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img1, SortOrder::Alphabetical)
            .expect("scan failed");

        // No filter set, should behave like peek_next
        let next = nav.peek_next_filtered();
        assert_eq!(next.as_deref(), Some(vid1.as_path()));
    }

    #[test]
    fn peek_filtered_returns_none_when_no_matches() {
        use crate::media::filter::MediaTypeFilter;

        let temp_dir = tempdir().expect("failed to create temp dir");
        let vid1 = create_test_video(temp_dir.path(), "a.mp4");
        let _vid2 = create_test_video(temp_dir.path(), "b.mp4");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&vid1, SortOrder::Alphabetical)
            .expect("scan failed");

        nav.set_filter(MediaFilter {
            media_type: MediaTypeFilter::ImagesOnly,
            date_range: None,
        });

        // No images in list, should return None
        assert_eq!(nav.peek_next_filtered(), None);
        assert_eq!(nav.peek_previous_filtered(), None);
    }

    #[test]
    fn current_matches_filter_with_no_filter() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let vid1 = create_test_video(temp_dir.path(), "a.mp4");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&vid1, SortOrder::Alphabetical)
            .expect("scan failed");

        // No filter, any media should match
        assert!(nav.current_matches_filter());
    }

    #[test]
    fn current_matches_filter_with_active_filter() {
        use crate::media::filter::MediaTypeFilter;

        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let vid1 = create_test_video(temp_dir.path(), "b.mp4");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img1, SortOrder::Alphabetical)
            .expect("scan failed");

        nav.set_filter(MediaFilter {
            media_type: MediaTypeFilter::ImagesOnly,
            date_range: None,
        });

        // Current is image, should match
        assert!(nav.current_matches_filter());

        // Change to video
        nav.set_current_media_path(vid1);
        // Current is video, should not match images-only filter
        assert!(!nav.current_matches_filter());
    }

    #[test]
    fn navigation_info_includes_filter_data() {
        use crate::media::filter::MediaTypeFilter;

        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let _vid1 = create_test_video(temp_dir.path(), "b.mp4");
        let _img2 = create_test_image(temp_dir.path(), "c.png");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img1, SortOrder::Alphabetical)
            .expect("scan failed");

        // No filter
        let info = nav.navigation_info();
        assert!(!info.filter_active);
        assert_eq!(info.filtered_count, 3);
        assert_eq!(info.total_count, 3);

        // With filter
        nav.set_filter(MediaFilter {
            media_type: MediaTypeFilter::ImagesOnly,
            date_range: None,
        });

        let info = nav.navigation_info();
        assert!(info.filter_active);
        assert_eq!(info.filtered_count, 2);
        assert_eq!(info.total_count, 3);
    }

    #[test]
    fn scan_from_directory_respects_active_filter() {
        use crate::media::filter::MediaTypeFilter;

        let temp_dir = tempdir().expect("failed to create temp dir");
        // Create files in alphabetical order: video first, then images
        let _vid1 = create_test_video(temp_dir.path(), "a.mp4");
        let _vid2 = create_test_video(temp_dir.path(), "b.mp4");
        let img1 = create_test_image(temp_dir.path(), "c.jpg");
        let _img2 = create_test_image(temp_dir.path(), "d.png");

        let mut nav = MediaNavigator::new();

        // Set filter BEFORE scanning
        nav.set_filter(MediaFilter {
            media_type: MediaTypeFilter::ImagesOnly,
            date_range: None,
        });

        let result = nav
            .scan_from_directory(temp_dir.path(), SortOrder::Alphabetical)
            .expect("scan failed");

        // Should return first IMAGE (c.jpg), not first file (a.mp4)
        assert_eq!(result, Some(img1.clone()));
        assert_eq!(nav.current_media_path(), Some(img1.as_path()));
        // Total should still be 4
        assert_eq!(nav.len(), 4);
        // Filtered count should be 2
        assert_eq!(nav.filtered_count(), 2);
    }

    #[test]
    fn scan_from_directory_with_filter_returns_none_when_no_matches() {
        use crate::media::filter::MediaTypeFilter;

        let temp_dir = tempdir().expect("failed to create temp dir");
        let _vid1 = create_test_video(temp_dir.path(), "a.mp4");
        let _vid2 = create_test_video(temp_dir.path(), "b.mp4");

        let mut nav = MediaNavigator::new();

        // Set images-only filter
        nav.set_filter(MediaFilter {
            media_type: MediaTypeFilter::ImagesOnly,
            date_range: None,
        });

        let result = nav
            .scan_from_directory(temp_dir.path(), SortOrder::Alphabetical)
            .expect("scan failed");

        // No images in directory, should return None
        assert_eq!(result, None);
        assert_eq!(nav.current_media_path(), None);
        // Directory still has 2 videos
        assert_eq!(nav.len(), 2);
        assert_eq!(nav.filtered_count(), 0);
    }
}
