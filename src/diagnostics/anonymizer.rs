// SPDX-License-Identifier: MPL-2.0
//! Path anonymization for diagnostic reports.
//!
//! This module provides [`PathAnonymizer`] which hashes file paths to protect
//! user privacy while preserving the directory structure depth and file extensions
//! for diagnostic analysis.
//!
//! # Example
//!
//! ```
//! use std::path::Path;
//! use iced_lens::diagnostics::PathAnonymizer;
//!
//! let anonymizer = PathAnonymizer::new();
//! let path = Path::new("/home/user/photos/vacation.jpg");
//! let anonymized = anonymizer.anonymize_path(path);
//!
//! // Result: something like "a1b2c3d4/e5f6g7h8/i9j0k1l2/m3n4o5p6.jpg"
//! // - Directory structure preserved (4 segments)
//! // - File extension preserved (.jpg)
//! // - All names hashed (one-way, cannot reverse)
//! ```
//!
//! # Privacy
//!
//! - Each `PathAnonymizer` instance has its own session salt
//! - Same path + same instance = same hash (consistent within session)
//! - Different instances produce different hashes (no cross-session correlation)
//! - Hashes are one-way and cannot be reversed to original paths

use std::path::{Component, Path, PathBuf};

/// Anonymizes file paths by hashing each segment while preserving structure.
///
/// Uses blake3 for fast, secure hashing with a session-unique salt.
/// The salt is generated using cryptographically secure randomness
/// or can be seeded deterministically for testing.
#[derive(Debug, Clone)]
pub struct PathAnonymizer {
    /// Session-unique salt for hashing (32 bytes)
    salt: [u8; 32],
}

impl PathAnonymizer {
    /// Creates a new `PathAnonymizer` with a cryptographically random session salt.
    ///
    /// Uses `getrandom` for secure entropy from the operating system.
    /// This ensures the salt cannot be predicted or reproduced.
    ///
    /// # Panics
    ///
    /// Panics if the operating system fails to provide random bytes.
    /// This is extremely rare and typically indicates a critical system failure.
    #[must_use]
    pub fn new() -> Self {
        let mut salt = [0u8; 32];
        // Use OS-provided cryptographic randomness
        getrandom::fill(&mut salt).expect("Failed to generate random salt");
        Self { salt }
    }

    /// Creates a new `PathAnonymizer` with a deterministic seed.
    ///
    /// This is useful for testing where reproducible hashes are needed.
    /// The seed is expanded to a 32-byte salt using blake3.
    #[must_use]
    pub fn with_seed(seed: u64) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&seed.to_le_bytes());
        hasher.update(b"iced_lens_path_anonymizer_seed");

        let hash = hasher.finalize();
        let mut salt = [0u8; 32];
        salt.copy_from_slice(hash.as_bytes());

        Self { salt }
    }

    /// Anonymizes a file path by hashing each segment.
    ///
    /// # Behavior
    ///
    /// - Each directory name is replaced with an 8-character hash
    /// - The filename stem is hashed, but the extension is preserved
    /// - Directory structure depth is maintained
    /// - Empty paths return empty `PathBuf`
    /// - Root components (`/`, `C:\`) are omitted from output
    ///
    /// # Examples
    ///
    /// - `/home/user/photo.jpg` → `a1b2c3d4/e5f6g7h8/i9j0k1l2.jpg`
    /// - `/home/user/Makefile` → `a1b2c3d4/e5f6g7h8/i9j0k1l2`
    /// - `.bashrc` → `a1b2c3d4`
    #[must_use]
    pub fn anonymize_path(&self, path: &Path) -> PathBuf {
        let mut result = PathBuf::new();

        for component in path.components() {
            match component {
                // Skip root and navigation components - they don't contain user data
                Component::Prefix(_)
                | Component::RootDir
                | Component::CurDir
                | Component::ParentDir => {}
                // Hash normal path segments
                Component::Normal(segment) => {
                    let segment_str = segment.to_string_lossy();
                    if segment_str.is_empty() {
                        continue;
                    }

                    // Handle hidden files (starting with dot) specially
                    // std::path treats ".bashrc" as having extension "bashrc", which is wrong
                    // A hidden file has no real extension if there's no second dot
                    let is_hidden = segment_str.starts_with('.');
                    let has_real_extension = if is_hidden {
                        // For hidden files, check if there's a dot after the first character
                        segment_str[1..].contains('.')
                    } else {
                        segment_str.contains('.')
                    };

                    if is_hidden && !has_real_extension {
                        // Hidden file with no extension (e.g., ".bashrc", ".gitignore")
                        // Hash the entire name including the dot
                        result.push(self.hash_segment(&segment_str));
                        continue;
                    }

                    // Check for extension using std::path
                    let segment_path = Path::new(segment);
                    let extension = segment_path.extension();
                    let stem = segment_path.file_stem();

                    if let Some(stem) = stem {
                        let stem_str = stem.to_string_lossy();

                        // For hidden files with extension (e.g., ".config.json"),
                        // std::path gives stem=".config", ext="json" which is correct
                        let hashed_stem = self.hash_segment(&stem_str);

                        if let Some(ext) = extension {
                            // Has extension: hash stem, preserve extension
                            let mut filename = hashed_stem;
                            filename.push('.');
                            filename.push_str(&ext.to_string_lossy());
                            result.push(filename);
                        } else {
                            // No extension: just hash the whole name
                            result.push(hashed_stem);
                        }
                    }
                }
            }
        }

        result
    }

    /// Hashes a single path segment using blake3 with the session salt.
    ///
    /// Returns the first 8 characters of the hex-encoded hash for readability.
    fn hash_segment(&self, segment: &str) -> String {
        let mut hasher = blake3::Hasher::new_keyed(&self.salt);
        hasher.update(segment.as_bytes());
        let hash = hasher.finalize();

        // Take first 8 hex characters (4 bytes = 32 bits)
        // This provides ~4 billion unique values, sufficient for diagnostics
        hash.to_hex()[..8].to_string()
    }
}

impl Default for PathAnonymizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // PathAnonymizer Construction Tests
    // =========================================================================

    #[test]
    fn new_creates_instance_with_random_salt() {
        let anon1 = PathAnonymizer::new();
        let anon2 = PathAnonymizer::new();

        // Different instances should have different salts (with very high probability)
        // We can't directly compare salts, but hashing the same path should differ
        let path = Path::new("/test/path.txt");
        let hash1 = anon1.anonymize_path(path);
        let hash2 = anon2.anonymize_path(path);

        // Extremely unlikely to be equal with random salts
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn with_seed_creates_deterministic_instance() {
        let anon1 = PathAnonymizer::with_seed(42);
        let anon2 = PathAnonymizer::with_seed(42);

        let path = Path::new("/test/path.txt");
        let hash1 = anon1.anonymize_path(path);
        let hash2 = anon2.anonymize_path(path);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn different_seeds_produce_different_hashes() {
        let anon1 = PathAnonymizer::with_seed(42);
        let anon2 = PathAnonymizer::with_seed(43);

        let path = Path::new("/test/path.txt");
        let hash1 = anon1.anonymize_path(path);
        let hash2 = anon2.anonymize_path(path);

        assert_ne!(hash1, hash2);
    }

    // =========================================================================
    // Extension Preservation Tests
    // =========================================================================

    #[test]
    fn extension_preserved_jpg() {
        let anon = PathAnonymizer::with_seed(1);
        let path = Path::new("photo.jpg");
        let result = anon.anonymize_path(path);

        assert!(
            result.to_string_lossy().ends_with(".jpg"),
            "Expected .jpg extension, got: {result:?}"
        );
    }

    #[test]
    fn extension_preserved_png() {
        let anon = PathAnonymizer::with_seed(1);
        let path = Path::new("/home/user/image.png");
        let result = anon.anonymize_path(path);

        assert!(
            result.to_string_lossy().ends_with(".png"),
            "Expected .png extension, got: {result:?}"
        );
    }

    #[test]
    fn extension_preserved_multiple_dots() {
        let anon = PathAnonymizer::with_seed(1);
        let path = Path::new("archive.tar.gz");
        let result = anon.anonymize_path(path);

        // std::path considers .gz as extension, archive.tar as stem
        assert!(
            result.to_string_lossy().ends_with(".gz"),
            "Expected .gz extension, got: {result:?}"
        );
    }

    // =========================================================================
    // Directory Structure Tests
    // =========================================================================

    #[test]
    fn directory_segments_hashed() {
        let anon = PathAnonymizer::with_seed(1);
        let path = Path::new("/home/user/photos");
        let result = anon.anonymize_path(path);

        // Should have 3 segments: home, user, photos
        let segments: Vec<_> = result.components().collect();
        assert_eq!(segments.len(), 3, "Expected 3 segments, got: {result:?}");
    }

    #[test]
    fn full_path_anonymized() {
        let anon = PathAnonymizer::with_seed(1);
        let path = Path::new("/home/user/photos/vacation/beach.jpg");
        let result = anon.anonymize_path(path);

        // Should have 5 segments
        let segments: Vec<_> = result.components().collect();
        assert_eq!(segments.len(), 5, "Expected 5 segments, got: {result:?}");

        // Last segment should end with .jpg
        assert!(result.to_string_lossy().ends_with(".jpg"));

        // Each segment (except extension) should be 8 chars
        let result_str = result.to_string_lossy();
        for (i, segment) in result_str.split('/').enumerate() {
            if i < 4 {
                // Directory segments
                assert_eq!(
                    segment.len(),
                    8,
                    "Directory segment {i} should be 8 chars: {segment}"
                );
            } else {
                // Filename with extension: 8 chars + .jpg = 12
                assert_eq!(
                    segment.len(),
                    12,
                    "Filename should be 12 chars (8 + .jpg): {segment}"
                );
            }
        }
    }

    // =========================================================================
    // Consistency Tests
    // =========================================================================

    #[test]
    fn consistency_same_instance() {
        let anon = PathAnonymizer::with_seed(1);
        let path = Path::new("/home/user/file.txt");

        let result1 = anon.anonymize_path(path);
        let result2 = anon.anonymize_path(path);

        assert_eq!(result1, result2);
    }

    #[test]
    fn consistency_same_segment_same_hash() {
        let anon = PathAnonymizer::with_seed(1);

        // Same directory name in different paths should hash to same value
        let path1 = Path::new("/home/user/file.txt");
        let path2 = Path::new("/home/other/file.txt");

        let result1 = anon.anonymize_path(path1);
        let result2 = anon.anonymize_path(path2);

        // First segment (home) should be the same
        let seg1: Vec<_> = result1.components().collect();
        let seg2: Vec<_> = result2.components().collect();

        assert_eq!(
            seg1[0], seg2[0],
            "Same directory 'home' should hash equally"
        );
    }

    // =========================================================================
    // Edge Case Tests
    // =========================================================================

    #[test]
    fn edge_case_no_extension() {
        let anon = PathAnonymizer::with_seed(1);
        let path = Path::new("/home/user/Makefile");
        let result = anon.anonymize_path(path);

        // Should have 3 segments, last one should be 8 chars (no extension)
        let segments: Vec<_> = result.components().collect();
        assert_eq!(segments.len(), 3);

        let filename = result.file_name().unwrap().to_string_lossy();
        assert_eq!(filename.len(), 8, "No-extension file should be 8 chars");
        assert!(!filename.contains('.'));
    }

    #[test]
    fn edge_case_hidden_file_no_extension() {
        let anon = PathAnonymizer::with_seed(1);
        let path = Path::new("/home/user/.bashrc");
        let result = anon.anonymize_path(path);

        // .bashrc is a hidden file with no real extension
        // Should be hashed as a single segment (8 chars, no dot)
        let segments: Vec<_> = result.components().collect();
        assert_eq!(segments.len(), 3);

        let filename = result.file_name().unwrap().to_string_lossy();
        assert_eq!(filename.len(), 8, "Hidden file should be 8 chars hash");
        assert!(
            !filename.contains('.'),
            "Hidden file hash should have no dot"
        );
    }

    #[test]
    fn edge_case_hidden_file_with_extension() {
        let anon = PathAnonymizer::with_seed(1);
        let path = Path::new("/home/user/.config.json");
        let result = anon.anonymize_path(path);

        // .config.json has a real extension (.json)
        // Should preserve the extension
        let segments: Vec<_> = result.components().collect();
        assert_eq!(segments.len(), 3);

        assert!(
            result.to_string_lossy().ends_with(".json"),
            "Hidden file with extension should preserve it: {result:?}"
        );
    }

    #[test]
    fn edge_case_empty_path() {
        let anon = PathAnonymizer::with_seed(1);
        let path = Path::new("");
        let result = anon.anonymize_path(path);

        assert!(result.as_os_str().is_empty());
    }

    #[test]
    fn edge_case_root_only() {
        let anon = PathAnonymizer::with_seed(1);
        let path = Path::new("/");
        let result = anon.anonymize_path(path);

        // Root is skipped, so result should be empty
        assert!(result.as_os_str().is_empty());
    }

    #[test]
    fn edge_case_relative_path() {
        let anon = PathAnonymizer::with_seed(1);
        let path = Path::new("relative/path/file.txt");
        let result = anon.anonymize_path(path);

        let segments: Vec<_> = result.components().collect();
        assert_eq!(segments.len(), 3);
        assert!(result.to_string_lossy().ends_with(".txt"));
    }

    #[test]
    #[cfg(windows)]
    fn edge_case_windows_path() {
        let anon = PathAnonymizer::with_seed(1);
        let path = Path::new(r"C:\Users\john\Documents\file.txt");
        let result = anon.anonymize_path(path);

        // Should have 4 segments (Users, john, Documents, file.txt)
        // C:\ prefix is skipped
        let segments: Vec<_> = result.components().collect();
        assert_eq!(segments.len(), 4);
        assert!(result.to_string_lossy().ends_with(".txt"));
    }

    // =========================================================================
    // Hash Format Tests
    // =========================================================================

    #[test]
    fn hash_is_8_hex_characters() {
        let anon = PathAnonymizer::with_seed(1);
        let path = Path::new("/test");
        let result = anon.anonymize_path(path);

        let hash = result.to_string_lossy();
        assert_eq!(hash.len(), 8);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn hash_is_lowercase() {
        let anon = PathAnonymizer::with_seed(1);
        let path = Path::new("/TEST/PATH");
        let result = anon.anonymize_path(path);

        let result_str = result.to_string_lossy();
        assert_eq!(result_str, result_str.to_lowercase());
    }

    // =========================================================================
    // One-Way Hash Tests
    // =========================================================================

    #[test]
    fn different_paths_produce_different_hashes() {
        let anon = PathAnonymizer::with_seed(1);

        let path1 = Path::new("/home/user/file1.txt");
        let path2 = Path::new("/home/user/file2.txt");

        let result1 = anon.anonymize_path(path1);
        let result2 = anon.anonymize_path(path2);

        // Last segments should differ
        assert_ne!(result1.file_name(), result2.file_name());
    }
}
