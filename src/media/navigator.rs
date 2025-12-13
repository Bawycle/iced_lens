// SPDX-License-Identifier: MPL-2.0
//! Media navigation module for managing media lists and navigation state.
//!
//! This module provides a shared `MediaNavigator` that can be used by both
//! the viewer and editor components to maintain a single source of truth
//! for media list and current media path.

use crate::config::SortOrder;
use crate::directory_scanner::ImageList;
use crate::error::Result;
use std::path::{Path, PathBuf};

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
}
