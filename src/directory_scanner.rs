// SPDX-License-Identifier: MPL-2.0
//! Directory scanner module for finding and sorting image files.
//!
//! This module scans a directory for supported image formats, filters them,
//! and sorts them according to the configured sort order.

use crate::config::SortOrder;
use crate::error::Result;
use crate::image_handler::SUPPORTED_EXTENSIONS;
use std::path::{Path, PathBuf};

/// Represents a list of image files in a directory with navigation capabilities.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageList {
    images: Vec<PathBuf>,
    current_index: Option<usize>,
}

impl ImageList {
    /// Creates a new empty ImageList.
    pub fn new() -> Self {
        Self {
            images: Vec::new(),
            current_index: None,
        }
    }

    /// Scans a directory for supported image files and sorts them.
    /// If the current file doesn't exist anymore, the scan still succeeds but
    /// current_index will be None.
    pub fn scan_directory(current_file: &Path, sort_order: SortOrder) -> Result<Self> {
        let parent = current_file
            .parent()
            .ok_or_else(|| crate::error::Error::Io("No parent directory".into()))?;

        let mut images = Vec::new();

        for entry in std::fs::read_dir(parent)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && is_supported_image(&path) {
                images.push(path);
            }
        }

        sort_images(&mut images, sort_order)?;

        // Find current file in the list (may be None if file was deleted)
        let current_index = images.iter().position(|p| p == current_file);

        Ok(Self {
            images,
            current_index,
        })
    }

    /// Returns the current image path.
    pub fn current(&self) -> Option<&Path> {
        self.current_index
            .and_then(|idx| self.images.get(idx))
            .map(|p| p.as_path())
    }

    /// Returns the next image path, wrapping around to the start.
    pub fn next(&self) -> Option<&Path> {
        if self.images.is_empty() {
            return None;
        }

        let next_index = match self.current_index {
            Some(idx) => (idx + 1) % self.images.len(),
            None => 0,
        };

        self.images.get(next_index).map(|p| p.as_path())
    }

    /// Returns the previous image path, wrapping around to the end.
    pub fn previous(&self) -> Option<&Path> {
        if self.images.is_empty() {
            return None;
        }

        let prev_index = match self.current_index {
            Some(idx) => {
                if idx == 0 {
                    self.images.len() - 1
                } else {
                    idx - 1
                }
            }
            None => self.images.len() - 1,
        };

        self.images.get(prev_index).map(|p| p.as_path())
    }

    /// Checks if we're at the first image (used for boundary indication).
    pub fn is_at_first(&self) -> bool {
        matches!(self.current_index, Some(0))
    }

    /// Checks if we're at the last image (used for boundary indication).
    pub fn is_at_last(&self) -> bool {
        if self.images.is_empty() {
            return false;
        }
        matches!(self.current_index, Some(idx) if idx == self.images.len() - 1)
    }

    /// Returns the total number of images in the list.
    pub fn len(&self) -> usize {
        self.images.len()
    }

    /// Checks if the image list is empty.
    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }

    /// Updates the current index to the given path if it exists in the list.
    pub fn set_current(&mut self, path: &Path) {
        self.current_index = self.images.iter().position(|p| p == path);
    }

    /// Returns the current index if set.
    pub fn current_index(&self) -> Option<usize> {
        self.current_index
    }

    /// Returns the path at the specified index.
    pub fn get(&self, index: usize) -> Option<&Path> {
        self.images.get(index).map(|p| p.as_path())
    }

    /// Sets the current index directly.
    pub fn set_current_index(&mut self, index: usize) {
        if index < self.images.len() {
            self.current_index = Some(index);
        }
    }
}

impl Default for ImageList {
    fn default() -> Self {
        Self::new()
    }
}

/// Checks if a file has a supported image extension.
fn is_supported_image(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Sorts a list of image paths according to the specified sort order.
fn sort_images(images: &mut [PathBuf], sort_order: SortOrder) -> Result<()> {
    match sort_order {
        SortOrder::Alphabetical => {
            images.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
        }
        SortOrder::ModifiedDate => {
            images.sort_by(|a, b| {
                let a_time = a
                    .metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                let b_time = b
                    .metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                a_time.cmp(&b_time)
            });
        }
        SortOrder::CreatedDate => {
            images.sort_by(|a, b| {
                let a_time = a
                    .metadata()
                    .and_then(|m| m.created())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                let b_time = b
                    .metadata()
                    .and_then(|m| m.created())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                a_time.cmp(&b_time)
            });
        }
    }
    Ok(())
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
    fn is_supported_image_recognizes_valid_extensions() {
        assert!(is_supported_image(Path::new("test.jpg")));
        assert!(is_supported_image(Path::new("test.JPEG")));
        assert!(is_supported_image(Path::new("test.png")));
        assert!(is_supported_image(Path::new("test.gif")));
        assert!(is_supported_image(Path::new("test.svg")));
        assert!(is_supported_image(Path::new("test.webp")));
        assert!(is_supported_image(Path::new("test.bmp")));
        assert!(is_supported_image(Path::new("test.ico")));
        assert!(is_supported_image(Path::new("test.tiff")));
        assert!(is_supported_image(Path::new("test.tif")));
    }

    #[test]
    fn is_supported_image_rejects_invalid_extensions() {
        assert!(!is_supported_image(Path::new("test.txt")));
        assert!(!is_supported_image(Path::new("test.pdf")));
        assert!(!is_supported_image(Path::new("test.doc")));
        assert!(!is_supported_image(Path::new("test")));
    }

    #[test]
    fn scan_directory_finds_all_images() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let _img2 = create_test_image(temp_dir.path(), "b.png");
        let _img3 = create_test_image(temp_dir.path(), "c.gif");
        create_test_image(temp_dir.path(), "not_image.txt");

        let list = ImageList::scan_directory(&img1, SortOrder::Alphabetical)
            .expect("failed to scan directory");

        assert_eq!(list.len(), 3);
        assert!(list.current().is_some());
    }

    #[test]
    fn scan_directory_sorts_alphabetically() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img_c = create_test_image(temp_dir.path(), "c.jpg");
        let img_a = create_test_image(temp_dir.path(), "a.jpg");
        let img_b = create_test_image(temp_dir.path(), "b.jpg");

        let list = ImageList::scan_directory(&img_a, SortOrder::Alphabetical)
            .expect("failed to scan directory");

        assert_eq!(list.images[0], img_a);
        assert_eq!(list.images[1], img_b);
        assert_eq!(list.images[2], img_c);
    }

    #[test]
    fn next_wraps_around_to_first() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let _img2 = create_test_image(temp_dir.path(), "b.jpg");
        let img3 = create_test_image(temp_dir.path(), "c.jpg");

        let list = ImageList::scan_directory(&img3, SortOrder::Alphabetical)
            .expect("failed to scan directory");

        assert_eq!(list.current(), Some(img3.as_path()));
        assert_eq!(list.next(), Some(img1.as_path()));
    }

    #[test]
    fn previous_wraps_around_to_last() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let _img2 = create_test_image(temp_dir.path(), "b.jpg");
        let img3 = create_test_image(temp_dir.path(), "c.jpg");

        let list = ImageList::scan_directory(&img1, SortOrder::Alphabetical)
            .expect("failed to scan directory");

        assert_eq!(list.current(), Some(img1.as_path()));
        assert_eq!(list.previous(), Some(img3.as_path()));
    }

    #[test]
    fn is_at_first_and_is_at_last_detect_boundaries() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let img2 = create_test_image(temp_dir.path(), "b.jpg");
        let img3 = create_test_image(temp_dir.path(), "c.jpg");

        let list_first = ImageList::scan_directory(&img1, SortOrder::Alphabetical)
            .expect("failed to scan directory");
        assert!(list_first.is_at_first());
        assert!(!list_first.is_at_last());

        let list_last = ImageList::scan_directory(&img3, SortOrder::Alphabetical)
            .expect("failed to scan directory");
        assert!(!list_last.is_at_first());
        assert!(list_last.is_at_last());

        let list_middle = ImageList::scan_directory(&img2, SortOrder::Alphabetical)
            .expect("failed to scan directory");
        assert!(!list_middle.is_at_first());
        assert!(!list_middle.is_at_last());
    }

    #[test]
    fn empty_list_navigation_returns_none() {
        let list = ImageList::new();
        assert!(list.current().is_none());
        assert!(list.next().is_none());
        assert!(list.previous().is_none());
        assert!(!list.is_at_first());
        assert!(!list.is_at_last());
    }

    #[test]
    fn single_image_navigation_returns_same_image() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "only.jpg");

        let list = ImageList::scan_directory(&img1, SortOrder::Alphabetical)
            .expect("failed to scan directory");

        assert_eq!(list.current(), Some(img1.as_path()));
        assert_eq!(list.next(), Some(img1.as_path()));
        assert_eq!(list.previous(), Some(img1.as_path()));
        assert!(list.is_at_first());
        assert!(list.is_at_last());
    }
}
