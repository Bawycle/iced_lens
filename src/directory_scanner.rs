// SPDX-License-Identifier: MPL-2.0
//! Directory scanner module for finding and sorting media files.
//!
//! This module scans a directory for supported media formats (images and videos),
//! filters them, and sorts them according to the configured sort order.

use crate::config::SortOrder;
use crate::error::Result;
use crate::media;
use std::path::{Path, PathBuf};

/// Represents a list of media files (images and videos) in a directory with navigation capabilities.
#[derive(Debug, Clone, PartialEq)]
pub struct MediaList {
    media_files: Vec<PathBuf>,
    current_index: Option<usize>,
}

impl MediaList {
    /// Creates a new empty MediaList.
    pub fn new() -> Self {
        Self {
            media_files: Vec::new(),
            current_index: None,
        }
    }

    /// Scans a directory for supported media files and sorts them.
    /// If the current file doesn't exist anymore, the scan still succeeds but
    /// current_index will be None.
    pub fn scan_directory(current_file: &Path, sort_order: SortOrder) -> Result<Self> {
        let parent = current_file
            .parent()
            .ok_or_else(|| crate::error::Error::Io("No parent directory".into()))?;

        let mut media_files = Vec::new();

        for entry in std::fs::read_dir(parent)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && is_supported_media(&path) {
                media_files.push(path);
            }
        }

        sort_media_files(&mut media_files, sort_order)?;

        // Find current file in the list (may be None if file was deleted)
        let current_index = media_files.iter().position(|p| p == current_file);

        Ok(Self {
            media_files,
            current_index,
        })
    }

    /// Scans a directory directly for supported media files and sorts them.
    /// Sets current_index to 0 (first file) if any media files are found.
    ///
    /// Returns an error if the directory cannot be read.
    pub fn scan_directory_direct(directory: &Path, sort_order: SortOrder) -> Result<Self> {
        let mut media_files = Vec::new();

        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && is_supported_media(&path) {
                media_files.push(path);
            }
        }

        sort_media_files(&mut media_files, sort_order)?;

        // Set current_index to first file if any exist
        let current_index = if media_files.is_empty() {
            None
        } else {
            Some(0)
        };

        Ok(Self {
            media_files,
            current_index,
        })
    }

    /// Returns the first media file in the list, if any.
    pub fn first(&self) -> Option<&Path> {
        self.media_files.first().map(|p| p.as_path())
    }

    /// Returns the current media path.
    pub fn current(&self) -> Option<&Path> {
        self.current_index
            .and_then(|idx| self.media_files.get(idx))
            .map(|p| p.as_path())
    }

    /// Returns the next media path, wrapping around to the start.
    pub fn next(&self) -> Option<&Path> {
        self.peek_nth_next(0)
    }

    /// Returns the previous media path, wrapping around to the end.
    pub fn previous(&self) -> Option<&Path> {
        self.peek_nth_previous(0)
    }

    /// Returns the n-th next media path from current position, wrapping around.
    /// `skip_count = 0` returns immediate next, `skip_count = 1` skips one file, etc.
    pub fn peek_nth_next(&self, skip_count: usize) -> Option<&Path> {
        if self.media_files.is_empty() {
            return None;
        }

        let offset = skip_count + 1;
        let next_index = match self.current_index {
            Some(idx) => (idx + offset) % self.media_files.len(),
            None => offset.saturating_sub(1) % self.media_files.len(),
        };

        self.media_files.get(next_index).map(|p| p.as_path())
    }

    /// Returns the n-th previous media path from current position, wrapping around.
    /// `skip_count = 0` returns immediate previous, `skip_count = 1` skips one file, etc.
    pub fn peek_nth_previous(&self, skip_count: usize) -> Option<&Path> {
        if self.media_files.is_empty() {
            return None;
        }

        let offset = skip_count + 1;
        let len = self.media_files.len();
        let prev_index = match self.current_index {
            Some(idx) => {
                // Handle wrap-around with modular arithmetic
                (idx + len - (offset % len)) % len
            }
            None => len.saturating_sub(offset % len) % len,
        };

        self.media_files.get(prev_index).map(|p| p.as_path())
    }

    /// Checks if we're at the first media (used for boundary indication).
    pub fn is_at_first(&self) -> bool {
        matches!(self.current_index, Some(0))
    }

    /// Checks if we're at the last media (used for boundary indication).
    pub fn is_at_last(&self) -> bool {
        if self.media_files.is_empty() {
            return false;
        }
        matches!(self.current_index, Some(idx) if idx == self.media_files.len() - 1)
    }

    /// Returns the total number of media files in the list.
    pub fn len(&self) -> usize {
        self.media_files.len()
    }

    /// Checks if the media list is empty.
    pub fn is_empty(&self) -> bool {
        self.media_files.is_empty()
    }

    /// Updates the current index to the given path if it exists in the list.
    pub fn set_current(&mut self, path: &Path) {
        self.current_index = self.media_files.iter().position(|p| p == path);
    }

    /// Returns the current index if set.
    pub fn current_index(&self) -> Option<usize> {
        self.current_index
    }

    /// Returns the path at the specified index.
    pub fn get(&self, index: usize) -> Option<&Path> {
        self.media_files.get(index).map(|p| p.as_path())
    }

    /// Sets the current index directly.
    pub fn set_current_index(&mut self, index: usize) {
        if index < self.media_files.len() {
            self.current_index = Some(index);
        }
    }
}

impl Default for MediaList {
    fn default() -> Self {
        Self::new()
    }
}

/// Checks if a file has a supported media extension (images or videos).
fn is_supported_media(path: &Path) -> bool {
    media::detect_media_type(path).is_some()
}

/// Sorts a list of media file paths according to the specified sort order.
fn sort_media_files(media_files: &mut [PathBuf], sort_order: SortOrder) -> Result<()> {
    match sort_order {
        SortOrder::Alphabetical => {
            media_files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
        }
        SortOrder::ModifiedDate => {
            media_files.sort_by(|a, b| {
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
            media_files.sort_by(|a, b| {
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

    fn create_test_video(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        let mut file = fs::File::create(&path).expect("failed to create test file");
        file.write_all(b"fake video data")
            .expect("failed to write test file");
        path
    }

    #[test]
    fn scan_directory_finds_all_images() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let _img2 = create_test_image(temp_dir.path(), "b.png");
        let _img3 = create_test_image(temp_dir.path(), "c.gif");
        create_test_image(temp_dir.path(), "not_image.txt");

        let list = MediaList::scan_directory(&img1, SortOrder::Alphabetical)
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

        let list = MediaList::scan_directory(&img_a, SortOrder::Alphabetical)
            .expect("failed to scan directory");

        assert_eq!(list.media_files[0], img_a);
        assert_eq!(list.media_files[1], img_b);
        assert_eq!(list.media_files[2], img_c);
    }

    #[test]
    fn next_wraps_around_to_first() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let _img2 = create_test_image(temp_dir.path(), "b.jpg");
        let img3 = create_test_image(temp_dir.path(), "c.jpg");

        let list = MediaList::scan_directory(&img3, SortOrder::Alphabetical)
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

        let list = MediaList::scan_directory(&img1, SortOrder::Alphabetical)
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

        let list_first = MediaList::scan_directory(&img1, SortOrder::Alphabetical)
            .expect("failed to scan directory");
        assert!(list_first.is_at_first());
        assert!(!list_first.is_at_last());

        let list_last = MediaList::scan_directory(&img3, SortOrder::Alphabetical)
            .expect("failed to scan directory");
        assert!(!list_last.is_at_first());
        assert!(list_last.is_at_last());

        let list_middle = MediaList::scan_directory(&img2, SortOrder::Alphabetical)
            .expect("failed to scan directory");
        assert!(!list_middle.is_at_first());
        assert!(!list_middle.is_at_last());
    }

    #[test]
    fn empty_list_navigation_returns_none() {
        let list = MediaList::new();
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

        let list = MediaList::scan_directory(&img1, SortOrder::Alphabetical)
            .expect("failed to scan directory");

        assert_eq!(list.current(), Some(img1.as_path()));
        assert_eq!(list.next(), Some(img1.as_path()));
        assert_eq!(list.previous(), Some(img1.as_path()));
        assert!(list.is_at_first());
        assert!(list.is_at_last());
    }

    // TDD tests for mixed image/video navigation support
    #[test]
    fn is_supported_media_recognizes_video_extensions() {
        assert!(is_supported_media(Path::new("test.mp4")));
        assert!(is_supported_media(Path::new("test.MP4")));
        assert!(is_supported_media(Path::new("test.avi")));
        assert!(is_supported_media(Path::new("test.mov")));
        assert!(is_supported_media(Path::new("test.mkv")));
        assert!(is_supported_media(Path::new("test.webm")));
        assert!(is_supported_media(Path::new("test.m4v")));
    }

    #[test]
    fn is_supported_media_recognizes_image_extensions() {
        assert!(is_supported_media(Path::new("test.jpg")));
        assert!(is_supported_media(Path::new("test.png")));
        assert!(is_supported_media(Path::new("test.gif")));
    }

    #[test]
    fn is_supported_media_rejects_unsupported_formats() {
        assert!(!is_supported_media(Path::new("test.txt")));
        assert!(!is_supported_media(Path::new("test.pdf")));
        assert!(!is_supported_media(Path::new("test.doc")));
    }

    #[test]
    fn scan_directory_finds_both_images_and_videos() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let _vid1 = create_test_video(temp_dir.path(), "b.mp4");
        let _img2 = create_test_image(temp_dir.path(), "c.png");
        let _vid2 = create_test_video(temp_dir.path(), "d.avi");
        create_test_image(temp_dir.path(), "not_media.txt");

        let list = MediaList::scan_directory(&img1, SortOrder::Alphabetical)
            .expect("failed to scan directory");

        // Should find 4 media files (2 images + 2 videos)
        assert_eq!(list.len(), 4, "Should find both images and videos");
    }

    #[test]
    fn navigation_works_across_images_and_videos() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img1 = create_test_image(temp_dir.path(), "a.jpg");
        let vid1 = create_test_video(temp_dir.path(), "b.mp4");
        let img2 = create_test_image(temp_dir.path(), "c.png");

        let mut list = MediaList::scan_directory(&img1, SortOrder::Alphabetical)
            .expect("failed to scan directory");

        // Start at first image
        assert_eq!(list.current(), Some(img1.as_path()));

        // Next should be video
        assert_eq!(list.next(), Some(vid1.as_path()));

        // Set current to video and check next
        list.set_current(&vid1);
        assert_eq!(list.current(), Some(vid1.as_path()));
        assert_eq!(list.next(), Some(img2.as_path()));

        // Previous from video should go back to image
        assert_eq!(list.previous(), Some(img1.as_path()));
    }

    // Tests for scan_directory_direct
    #[test]
    fn scan_directory_direct_finds_all_media() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img_a = create_test_image(temp_dir.path(), "a.jpg");
        let _img_b = create_test_image(temp_dir.path(), "b.png");
        let _vid = create_test_video(temp_dir.path(), "c.mp4");
        create_test_image(temp_dir.path(), "not_media.txt");

        let list = MediaList::scan_directory_direct(temp_dir.path(), SortOrder::Alphabetical)
            .expect("failed to scan directory");

        assert_eq!(list.len(), 3);
        assert_eq!(list.current_index(), Some(0));
        assert_eq!(list.first(), Some(img_a.as_path()));
    }

    #[test]
    fn scan_directory_direct_sorts_alphabetically() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let _img_c = create_test_image(temp_dir.path(), "c.jpg");
        let img_a = create_test_image(temp_dir.path(), "a.jpg");
        let _img_b = create_test_image(temp_dir.path(), "b.jpg");

        let list = MediaList::scan_directory_direct(temp_dir.path(), SortOrder::Alphabetical)
            .expect("failed to scan directory");

        assert_eq!(list.first(), Some(img_a.as_path()));
        assert_eq!(list.current(), Some(img_a.as_path()));
    }

    #[test]
    fn scan_directory_direct_returns_empty_for_no_media() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        create_test_image(temp_dir.path(), "readme.txt");
        create_test_image(temp_dir.path(), "document.pdf");

        let list = MediaList::scan_directory_direct(temp_dir.path(), SortOrder::Alphabetical)
            .expect("failed to scan directory");

        assert!(list.is_empty());
        assert_eq!(list.current_index(), None);
        assert_eq!(list.first(), None);
    }

    #[test]
    fn scan_directory_direct_handles_empty_directory() {
        let temp_dir = tempdir().expect("failed to create temp dir");

        let list = MediaList::scan_directory_direct(temp_dir.path(), SortOrder::Alphabetical)
            .expect("failed to scan directory");

        assert!(list.is_empty());
        assert_eq!(list.first(), None);
    }

    #[test]
    fn first_returns_first_media_file() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let img_a = create_test_image(temp_dir.path(), "a.jpg");
        let _img_b = create_test_image(temp_dir.path(), "b.jpg");

        let list = MediaList::scan_directory_direct(temp_dir.path(), SortOrder::Alphabetical)
            .expect("failed to scan directory");

        assert_eq!(list.first(), Some(img_a.as_path()));
    }
}
