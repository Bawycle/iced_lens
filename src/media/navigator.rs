// SPDX-License-Identifier: MPL-2.0
//! Media navigation module for managing media lists and navigation state.
//!
//! This module provides a shared `MediaNavigator` that can be used by both
//! the viewer and editor components to maintain a single source of truth
//! for media list and current media path.

use crate::config::SortOrder;
use crate::directory_scanner::ImageList;
use crate::error::Result;
use crate::media::{detect_media_type, MediaType};
use std::path::{Path, PathBuf};

/// Navigation state information for UI rendering.
///
/// This struct contains all the information needed by the viewer to render
/// navigation controls without needing direct access to the media list.
/// It acts as a snapshot of the current navigation state.
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
}

/// Manages navigation through a list of media files in a directory.
///
/// This component encapsulates both the media list and the current media path,
/// providing a single source of truth for media navigation shared between
/// viewer and editor components.
#[derive(Debug, Clone, PartialEq)]
pub struct MediaNavigator {
    /// List of media files in the current directory
    media_list: ImageList,
    /// Path to the currently selected media
    current_media_path: Option<PathBuf>,
}

impl MediaNavigator {
    /// Creates a new empty MediaNavigator.
    pub fn new() -> Self {
        Self {
            media_list: ImageList::new(),
            current_media_path: None,
        }
    }

    /// Scans the directory containing the given media file and updates the media list.
    ///
    /// Returns an error if the directory cannot be read or the path has no parent directory.
    pub fn scan_directory(&mut self, current_file: &Path, sort_order: SortOrder) -> Result<()> {
        self.media_list = ImageList::scan_directory(current_file, sort_order)?;
        self.current_media_path = Some(current_file.to_path_buf());
        Ok(())
    }

    /// Returns the path to the current media, if set.
    pub fn current_media_path(&self) -> Option<&Path> {
        self.current_media_path.as_deref()
    }

    /// Sets the current media path and updates the media list index.
    pub fn set_current_media_path(&mut self, path: PathBuf) {
        self.media_list.set_current(&path);
        self.current_media_path = Some(path);
    }

    /// Navigates to the next media and returns its path.
    ///
    /// Returns `None` if there are no media in the list.
    /// Wraps around to the first media when at the last media.
    pub fn navigate_next(&mut self) -> Option<PathBuf> {
        let next_path = self.media_list.next()?.to_path_buf();
        self.current_media_path = Some(next_path.clone());
        self.media_list.set_current(&next_path);
        Some(next_path)
    }

    /// Navigates to the previous media and returns its path.
    ///
    /// Returns `None` if there are no media in the list.
    /// Wraps around to the last media when at the first media.
    pub fn navigate_previous(&mut self) -> Option<PathBuf> {
        let prev_path = self.media_list.previous()?.to_path_buf();
        self.current_media_path = Some(prev_path.clone());
        self.media_list.set_current(&prev_path);
        Some(prev_path)
    }

    /// Navigates to the next image, skipping videos.
    ///
    /// Returns `None` if there are no images in the list (only videos).
    /// Wraps around to the first image when at the last.
    pub fn navigate_next_image(&mut self) -> Option<PathBuf> {
        let start_path = self.current_media_path.clone();
        let total = self.len();

        // Try up to `total` times to find an image (avoid infinite loop)
        for _ in 0..total {
            if let Some(next_path) = self.navigate_next() {
                if matches!(detect_media_type(&next_path), Some(MediaType::Image)) {
                    return Some(next_path);
                }
                // If we've wrapped back to start, no images found
                if Some(&next_path) == start_path.as_ref() {
                    return None;
                }
            } else {
                return None;
            }
        }
        None
    }

    /// Navigates to the previous image, skipping videos.
    ///
    /// Returns `None` if there are no images in the list (only videos).
    /// Wraps around to the last image when at the first.
    pub fn navigate_previous_image(&mut self) -> Option<PathBuf> {
        let start_path = self.current_media_path.clone();
        let total = self.len();

        // Try up to `total` times to find an image (avoid infinite loop)
        for _ in 0..total {
            if let Some(prev_path) = self.navigate_previous() {
                if matches!(detect_media_type(&prev_path), Some(MediaType::Image)) {
                    return Some(prev_path);
                }
                // If we've wrapped back to start, no images found
                if Some(&prev_path) == start_path.as_ref() {
                    return None;
                }
            } else {
                return None;
            }
        }
        None
    }

    /// Checks if there is a next media available.
    pub fn has_next(&self) -> bool {
        self.media_list.next().is_some()
    }

    /// Checks if there is a previous media available.
    pub fn has_previous(&self) -> bool {
        self.media_list.previous().is_some()
    }

    /// Checks if the current media is the first in the list.
    pub fn is_at_first(&self) -> bool {
        self.media_list.is_at_first()
    }

    /// Checks if the current media is the last in the list.
    pub fn is_at_last(&self) -> bool {
        self.media_list.is_at_last()
    }

    /// Returns the total number of media in the list.
    pub fn len(&self) -> usize {
        self.media_list.len()
    }

    /// Checks if the media list is empty.
    pub fn is_empty(&self) -> bool {
        self.media_list.is_empty()
    }

    /// Returns the current index in the media list, if set.
    pub fn current_index(&self) -> Option<usize> {
        self.media_list.current_index()
    }

    /// Returns a snapshot of the current navigation state for UI rendering.
    ///
    /// This method provides all the information needed by the viewer to render
    /// navigation controls without needing direct access to the media list.
    pub fn navigation_info(&self) -> NavigationInfo {
        NavigationInfo {
            has_next: self.has_next(),
            has_previous: self.has_previous(),
            at_first: self.is_at_first(),
            at_last: self.is_at_last(),
            current_index: self.current_index(),
            total_count: self.len(),
        }
    }
}

impl Default for MediaNavigator {
    fn default() -> Self {
        Self::new()
    }
}

// Backward compatibility alias
pub type ImageNavigator = MediaNavigator;

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
    fn navigate_next_advances_to_next_media() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let img2 = create_test_image(temp_dir.path(), "b.png");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img1, SortOrder::Alphabetical)
            .expect("scan failed");

        let next = nav.navigate_next();
        assert_eq!(next.as_deref(), Some(img2.as_path()));
        assert_eq!(nav.current_media_path(), Some(img2.as_path()));
    }

    #[test]
    fn navigate_previous_goes_to_previous_media() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let img2 = create_test_image(temp_dir.path(), "b.png");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img2, SortOrder::Alphabetical)
            .expect("scan failed");

        let prev = nav.navigate_previous();
        assert_eq!(prev.as_deref(), Some(img1.as_path()));
        assert_eq!(nav.current_media_path(), Some(img1.as_path()));
    }

    #[test]
    fn navigate_next_wraps_around() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let img2 = create_test_image(temp_dir.path(), "b.png");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img2, SortOrder::Alphabetical)
            .expect("scan failed");

        let next = nav.navigate_next();
        assert_eq!(next.as_deref(), Some(img1.as_path())); // wraps to first
    }

    #[test]
    fn navigate_previous_wraps_around() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let img2 = create_test_image(temp_dir.path(), "b.png");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img1, SortOrder::Alphabetical)
            .expect("scan failed");

        let prev = nav.navigate_previous();
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
    fn empty_navigator_returns_none_on_navigation() {
        let mut nav = MediaNavigator::new();
        assert_eq!(nav.navigate_next(), None);
        assert_eq!(nav.navigate_previous(), None);
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
    fn navigate_next_image_skips_videos() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let _vid1 = create_test_video(temp_dir.path(), "b.mp4");
        let _vid2 = create_test_video(temp_dir.path(), "c.mp4");
        let img2 = create_test_image(temp_dir.path(), "d.png");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img1, SortOrder::Alphabetical)
            .expect("scan failed");

        // Should skip b.mp4 and c.mp4, go directly to d.png
        let next = nav.navigate_next_image();
        assert_eq!(next.as_deref(), Some(img2.as_path()));
        assert_eq!(nav.current_media_path(), Some(img2.as_path()));
    }

    #[test]
    fn navigate_previous_image_skips_videos() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let _vid1 = create_test_video(temp_dir.path(), "b.mp4");
        let _vid2 = create_test_video(temp_dir.path(), "c.mp4");
        let img2 = create_test_image(temp_dir.path(), "d.png");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img2, SortOrder::Alphabetical)
            .expect("scan failed");

        // Should skip c.mp4 and b.mp4, go directly to a.jpg
        let prev = nav.navigate_previous_image();
        assert_eq!(prev.as_deref(), Some(img1.as_path()));
        assert_eq!(nav.current_media_path(), Some(img1.as_path()));
    }

    #[test]
    fn navigate_next_image_wraps_around_skipping_videos() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let _vid1 = create_test_video(temp_dir.path(), "b.mp4");
        let img2 = create_test_image(temp_dir.path(), "c.png");
        let _vid2 = create_test_video(temp_dir.path(), "d.mp4");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img2, SortOrder::Alphabetical)
            .expect("scan failed");

        // From c.png, should skip d.mp4, wrap to a.jpg (skipping b.mp4)
        let next = nav.navigate_next_image();
        assert_eq!(next.as_deref(), Some(img1.as_path()));
    }

    #[test]
    fn navigate_previous_image_wraps_around_skipping_videos() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let _vid1 = create_test_video(temp_dir.path(), "a.mp4");
        let img1 = create_test_image(temp_dir.path(), "b.jpg");
        let _vid2 = create_test_video(temp_dir.path(), "c.mp4");
        let img2 = create_test_image(temp_dir.path(), "d.png");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img1, SortOrder::Alphabetical)
            .expect("scan failed");

        // From b.jpg, should skip a.mp4, wrap to d.png (skipping c.mp4)
        let prev = nav.navigate_previous_image();
        assert_eq!(prev.as_deref(), Some(img2.as_path()));
    }

    #[test]
    fn navigate_next_image_returns_same_if_only_one_image() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let _vid1 = create_test_video(temp_dir.path(), "a.mp4");
        let img1 = create_test_image(temp_dir.path(), "b.jpg");
        let _vid2 = create_test_video(temp_dir.path(), "c.mp4");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img1, SortOrder::Alphabetical)
            .expect("scan failed");

        // Only one image, should wrap back to itself
        let next = nav.navigate_next_image();
        assert_eq!(next.as_deref(), Some(img1.as_path()));
    }

    #[test]
    fn navigate_previous_image_returns_same_if_only_one_image() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let _vid1 = create_test_video(temp_dir.path(), "a.mp4");
        let img1 = create_test_image(temp_dir.path(), "b.jpg");
        let _vid2 = create_test_video(temp_dir.path(), "c.mp4");

        let mut nav = MediaNavigator::new();
        nav.scan_directory(&img1, SortOrder::Alphabetical)
            .expect("scan failed");

        // Only one image, should wrap back to itself
        let prev = nav.navigate_previous_image();
        assert_eq!(prev.as_deref(), Some(img1.as_path()));
    }

    #[test]
    fn navigate_next_image_returns_none_for_empty_navigator() {
        let mut nav = MediaNavigator::new();
        assert_eq!(nav.navigate_next_image(), None);
    }

    #[test]
    fn navigate_previous_image_returns_none_for_empty_navigator() {
        let mut nav = MediaNavigator::new();
        assert_eq!(nav.navigate_previous_image(), None);
    }
}
