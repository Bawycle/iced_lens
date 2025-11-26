// SPDX-License-Identifier: MPL-2.0
//! Image navigation module for managing image lists and navigation state.
//!
//! This module provides a shared `ImageNavigator` that can be used by both
//! the viewer and editor components to maintain a single source of truth
//! for image list and current image path.

use crate::config::SortOrder;
use crate::directory_scanner::ImageList;
use crate::error::Result;
use std::path::{Path, PathBuf};

/// Manages navigation through a list of images in a directory.
///
/// This component encapsulates both the image list and the current image path,
/// providing a single source of truth for image navigation shared between
/// viewer and editor components.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageNavigator {
    /// List of images in the current directory
    image_list: ImageList,
    /// Path to the currently selected image
    current_image_path: Option<PathBuf>,
}

impl ImageNavigator {
    /// Creates a new empty ImageNavigator.
    pub fn new() -> Self {
        Self {
            image_list: ImageList::new(),
            current_image_path: None,
        }
    }

    /// Scans the directory containing the given image file and updates the image list.
    ///
    /// Returns an error if the directory cannot be read or the path has no parent directory.
    pub fn scan_directory(&mut self, current_file: &Path, sort_order: SortOrder) -> Result<()> {
        self.image_list = ImageList::scan_directory(current_file, sort_order)?;
        self.current_image_path = Some(current_file.to_path_buf());
        Ok(())
    }

    /// Returns the path to the current image, if set.
    pub fn current_image_path(&self) -> Option<&Path> {
        self.current_image_path.as_deref()
    }

    /// Sets the current image path and updates the image list index.
    pub fn set_current_image_path(&mut self, path: PathBuf) {
        self.image_list.set_current(&path);
        self.current_image_path = Some(path);
    }

    /// Navigates to the next image and returns its path.
    ///
    /// Returns `None` if there are no images in the list.
    /// Wraps around to the first image when at the last image.
    pub fn navigate_next(&mut self) -> Option<PathBuf> {
        let next_path = self.image_list.next()?.to_path_buf();
        self.current_image_path = Some(next_path.clone());
        self.image_list.set_current(&next_path);
        Some(next_path)
    }

    /// Navigates to the previous image and returns its path.
    ///
    /// Returns `None` if there are no images in the list.
    /// Wraps around to the last image when at the first image.
    pub fn navigate_previous(&mut self) -> Option<PathBuf> {
        let prev_path = self.image_list.previous()?.to_path_buf();
        self.current_image_path = Some(prev_path.clone());
        self.image_list.set_current(&prev_path);
        Some(prev_path)
    }

    /// Checks if there is a next image available.
    pub fn has_next(&self) -> bool {
        self.image_list.next().is_some()
    }

    /// Checks if there is a previous image available.
    pub fn has_previous(&self) -> bool {
        self.image_list.previous().is_some()
    }

    /// Checks if the current image is the first in the list.
    pub fn is_at_first(&self) -> bool {
        self.image_list.is_at_first()
    }

    /// Checks if the current image is the last in the list.
    pub fn is_at_last(&self) -> bool {
        self.image_list.is_at_last()
    }

    /// Returns the total number of images in the list.
    pub fn len(&self) -> usize {
        self.image_list.len()
    }

    /// Checks if the image list is empty.
    pub fn is_empty(&self) -> bool {
        self.image_list.is_empty()
    }

    /// Returns the current index in the image list, if set.
    pub fn current_index(&self) -> Option<usize> {
        self.image_list.current_index()
    }
}

impl Default for ImageNavigator {
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
        let nav = ImageNavigator::new();
        assert!(nav.is_empty());
        assert_eq!(nav.len(), 0);
        assert_eq!(nav.current_image_path(), None);
    }

    #[test]
    fn scan_directory_finds_images() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let _img2 = create_test_image(temp_dir.path(), "b.png");
        let _img3 = create_test_image(temp_dir.path(), "c.gif");

        let mut nav = ImageNavigator::new();
        nav.scan_directory(&img1, SortOrder::Alphabetical)
            .expect("scan failed");

        assert_eq!(nav.len(), 3);
        assert_eq!(nav.current_image_path(), Some(img1.as_path()));
    }

    #[test]
    fn navigate_next_advances_to_next_image() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let img2 = create_test_image(temp_dir.path(), "b.png");

        let mut nav = ImageNavigator::new();
        nav.scan_directory(&img1, SortOrder::Alphabetical)
            .expect("scan failed");

        let next = nav.navigate_next();
        assert_eq!(next.as_deref(), Some(img2.as_path()));
        assert_eq!(nav.current_image_path(), Some(img2.as_path()));
    }

    #[test]
    fn navigate_previous_goes_to_previous_image() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let img2 = create_test_image(temp_dir.path(), "b.png");

        let mut nav = ImageNavigator::new();
        nav.scan_directory(&img2, SortOrder::Alphabetical)
            .expect("scan failed");

        let prev = nav.navigate_previous();
        assert_eq!(prev.as_deref(), Some(img1.as_path()));
        assert_eq!(nav.current_image_path(), Some(img1.as_path()));
    }

    #[test]
    fn navigate_next_wraps_around() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let img2 = create_test_image(temp_dir.path(), "b.png");

        let mut nav = ImageNavigator::new();
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

        let mut nav = ImageNavigator::new();
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

        let mut nav = ImageNavigator::new();
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

        let mut nav = ImageNavigator::new();
        nav.scan_directory(&img1, SortOrder::Alphabetical)
            .expect("scan failed");

        assert!(nav.is_at_first());
        assert!(!nav.is_at_last());

        nav.set_current_image_path(img2);
        assert!(!nav.is_at_first());
        assert!(nav.is_at_last());
    }

    #[test]
    fn set_current_image_path_updates_state() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let img2 = create_test_image(temp_dir.path(), "b.png");

        let mut nav = ImageNavigator::new();
        nav.scan_directory(&img1, SortOrder::Alphabetical)
            .expect("scan failed");

        nav.set_current_image_path(img2.clone());
        assert_eq!(nav.current_image_path(), Some(img2.as_path()));
        assert_eq!(nav.current_index(), Some(1));
    }

    #[test]
    fn empty_navigator_returns_none_on_navigation() {
        let mut nav = ImageNavigator::new();
        assert_eq!(nav.navigate_next(), None);
        assert_eq!(nav.navigate_previous(), None);
        assert!(!nav.has_next());
        assert!(!nav.has_previous());
    }
}
