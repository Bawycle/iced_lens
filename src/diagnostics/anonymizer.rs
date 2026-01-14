// SPDX-License-Identifier: MPL-2.0
//! Anonymization for diagnostic reports.
//!
//! This module provides anonymization utilities to protect user privacy in
//! diagnostic reports while preserving diagnostic value:
//!
//! - [`PathAnonymizer`]: Hashes file paths while preserving directory structure
//!   and file extensions.
//! - [`IdentityAnonymizer`]: Hashes IP addresses, domain names, and usernames
//!   found in strings.
//!
//! # Example
//!
//! ```
//! use std::path::Path;
//! use iced_lens::diagnostics::{PathAnonymizer, IdentityAnonymizer};
//!
//! // Path anonymization
//! let path_anon = PathAnonymizer::new();
//! let path = Path::new("/home/user/photos/vacation.jpg");
//! let anonymized = path_anon.anonymize_path(path);
//! // Result: something like "a1b2c3d4/e5f6g7h8/i9j0k1l2/m3n4o5p6.jpg"
//!
//! // Identity anonymization
//! let id_anon = IdentityAnonymizer::new();
//! let message = "Error from 192.168.1.1 at example.com";
//! let anonymized = id_anon.anonymize_string(message);
//! // Result: "Error from <ip:a1b2c3d4> at <domain:e5f6g7h8>.com"
//! ```
//!
//! # Privacy
//!
//! - Each anonymizer instance has its own session salt
//! - Same input + same instance = same hash (consistent within session)
//! - Different instances produce different hashes (no cross-session correlation)
//! - Hashes are one-way and cannot be reversed to original values

use std::net::Ipv4Addr;
use std::path::{Component, Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;

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

// =============================================================================
// Identity Anonymization
// =============================================================================

/// IPv4 address pattern: matches addresses like 192.168.1.1
static IPV4_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").expect("IPv4 regex should compile")
});

/// IPv6 address pattern: matches various IPv6 formats
/// Note: No word boundaries since : is not a word character - IPv6 patterns are distinctive enough
static IPV6_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"(?i)",
        r"(?:[0-9a-f]{1,4}:){7}[0-9a-f]{1,4}|",    // Full form
        r"(?:[0-9a-f]{1,4}:){1,7}:|",              // Trailing ::
        r"(?:[0-9a-f]{1,4}:){1,6}:[0-9a-f]{1,4}|", // :: in middle
        r"(?:[0-9a-f]{1,4}:){1,5}(?::[0-9a-f]{1,4}){1,2}|",
        r"(?:[0-9a-f]{1,4}:){1,4}(?::[0-9a-f]{1,4}){1,3}|",
        r"(?:[0-9a-f]{1,4}:){1,3}(?::[0-9a-f]{1,4}){1,4}|",
        r"(?:[0-9a-f]{1,4}:){1,2}(?::[0-9a-f]{1,4}){1,5}|",
        r"[0-9a-f]{1,4}:(?::[0-9a-f]{1,4}){1,6}|",
        r":(?::[0-9a-f]{1,4}){1,7}|",               // Leading ::
        r"::(?:[fF]{4}:)?(?:\d{1,3}\.){3}\d{1,3}|", // IPv4-mapped
        r"::1|::"                                   // Loopback, unspecified
    ))
    .expect("IPv6 regex should compile")
});

/// Domain name pattern: matches domains like example.com, sub.domain.org
static DOMAIN_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:[a-zA-Z0-9](?:[a-zA-Z0-9-]*[a-zA-Z0-9])?\.)+[a-zA-Z]{2,}\b")
        .expect("Domain regex should compile")
});

/// Known TLDs to recognize as domain names (not file extensions)
const KNOWN_TLDS: &[&str] = &[
    // Generic TLDs
    "com", "org", "net", "edu", "gov", "mil", "int", // Popular TLDs
    "io", "dev", "app", "cloud", "ai", "tech", "co", "info", "biz", "me",
    // Country codes (selection of common ones)
    "uk", "de", "fr", "jp", "cn", "au", "ca", "nl", "ru", "br", "in", "it", "es",
];

/// Common file extensions that look like TLDs but should be skipped
const FILE_EXTENSIONS: &[&str] = &[
    "rs", "js", "ts", "py", "go", "md", "txt", "log", "json", "xml", "html", "css", "jpg", "jpeg",
    "png", "gif", "bmp", "webp", "svg", "ico", "mp3", "mp4", "wav", "avi", "mkv", "webm", "mov",
    "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "zip", "tar", "gz", "rar", "7z", "exe",
    "dll", "so", "dylib", "toml", "yaml", "yml", "ini", "cfg", "conf",
];

/// Anonymizes IP addresses, domain names, and usernames in strings.
///
/// Uses blake3 for fast, secure hashing with a session-unique salt.
/// The system username is detected at construction time for detection.
#[derive(Debug, Clone)]
pub struct IdentityAnonymizer {
    /// Session-unique salt for hashing (32 bytes)
    salt: [u8; 32],
    /// Pre-computed username hash for replacement
    username_replacement: Option<String>,
    /// Compiled regex for username detection (compiled once at construction)
    username_pattern: Option<Regex>,
}

impl IdentityAnonymizer {
    /// Creates a new `IdentityAnonymizer` with a cryptographically random session salt.
    ///
    /// The system username is automatically detected using environment variables
    /// (`USER` on Unix, `USERNAME` on Windows).
    ///
    /// # Panics
    ///
    /// Panics if the operating system fails to provide random bytes.
    /// This is extremely rare and typically indicates a critical system failure.
    #[must_use]
    pub fn new() -> Self {
        let mut salt = [0u8; 32];
        getrandom::fill(&mut salt).expect("Failed to generate random salt");
        let username = Self::get_system_username();
        Self::build(salt, username.as_deref())
    }

    /// Creates a new `IdentityAnonymizer` with a deterministic seed.
    ///
    /// This is useful for testing where reproducible hashes are needed.
    /// The seed is expanded to a 32-byte salt using blake3.
    ///
    /// The system username is still detected automatically.
    #[must_use]
    pub fn with_seed(seed: u64) -> Self {
        let salt = Self::salt_from_seed(seed);
        let username = Self::get_system_username();
        Self::build(salt, username.as_deref())
    }

    /// Creates a new `IdentityAnonymizer` with a deterministic seed and custom username.
    ///
    /// This is useful for testing where both reproducible hashes and a known
    /// username are needed.
    #[must_use]
    pub fn with_seed_and_username(seed: u64, username: Option<&str>) -> Self {
        let salt = Self::salt_from_seed(seed);
        Self::build(salt, username)
    }

    /// Derives a 32-byte salt from a u64 seed using blake3.
    fn salt_from_seed(seed: u64) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&seed.to_le_bytes());
        hasher.update(b"iced_lens_identity_anonymizer_seed");

        let hash = hasher.finalize();
        let mut salt = [0u8; 32];
        salt.copy_from_slice(hash.as_bytes());
        salt
    }

    /// Builds the anonymizer with pre-computed username pattern and replacement.
    fn build(salt: [u8; 32], username: Option<&str>) -> Self {
        let (username_replacement, username_pattern) = match username {
            Some(name) if !name.is_empty() => {
                // Pre-compute the hash for the username
                let hash = Self::hash_with_salt(&salt, name);
                let replacement = format!("<user:{hash}>");

                // Pre-compile the regex pattern
                let escaped = regex::escape(name);
                let pattern = Regex::new(&format!(r"(?i)\b{escaped}\b")).ok();

                (Some(replacement), pattern)
            }
            _ => (None, None),
        };

        Self {
            salt,
            username_replacement,
            username_pattern,
        }
    }

    /// Gets the system username from environment variables.
    fn get_system_username() -> Option<String> {
        std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .ok()
    }

    /// Hashes a value with a given salt (used during construction).
    fn hash_with_salt(salt: &[u8; 32], value: &str) -> String {
        let mut hasher = blake3::Hasher::new_keyed(salt);
        hasher.update(value.as_bytes());
        let hash = hasher.finalize();
        hash.to_hex()[..8].to_string()
    }

    /// Anonymizes all PII (IP addresses, domains, usernames) in a string.
    ///
    /// # Order of operations
    ///
    /// 1. Username replacement (to avoid partial matches)
    /// 2. IPv4 address replacement
    /// 3. IPv6 address replacement
    /// 4. Domain name replacement
    ///
    /// # Examples
    ///
    /// - `192.168.1.1` → `<ip:a1b2c3d4>`
    /// - `::1` → `<ip:e5f6g7h8>`
    /// - `example.com` → `<domain:i9j0k1l2>.com`
    /// - `{username}` → `<user:m3n4o5p6>`
    #[must_use]
    pub fn anonymize_string(&self, input: &str) -> String {
        let mut result = input.to_string();

        // 1. Replace username first (using pre-compiled regex and replacement)
        if let (Some(pattern), Some(replacement)) =
            (&self.username_pattern, &self.username_replacement)
        {
            result = pattern.replace_all(&result, replacement).into_owned();
        }

        // 2. Replace IPv4 addresses
        result = self.anonymize_ipv4(&result);

        // 3. Replace IPv6 addresses
        result = self.anonymize_ipv6(&result);

        // 4. Replace domain names
        result = self.anonymize_domains(&result);

        result
    }

    /// Anonymizes IPv4 addresses in a string.
    ///
    /// Uses regex to find candidates, then validates with `std::net::Ipv4Addr`
    /// to avoid false positives like `999.999.999.999`.
    fn anonymize_ipv4(&self, input: &str) -> String {
        IPV4_PATTERN
            .replace_all(input, |caps: &regex::Captures| {
                let ip = &caps[0];
                // Validate that it's actually a valid IPv4 address
                if ip.parse::<Ipv4Addr>().is_ok() {
                    let hash = self.hash_value(ip);
                    format!("<ip:{hash}>")
                } else {
                    // Not a valid IP, keep original
                    ip.to_string()
                }
            })
            .into_owned()
    }

    /// Anonymizes IPv6 addresses in a string.
    fn anonymize_ipv6(&self, input: &str) -> String {
        IPV6_PATTERN
            .replace_all(input, |caps: &regex::Captures| {
                let ip = &caps[0];
                let hash = self.hash_value(ip);
                format!("<ip:{hash}>")
            })
            .into_owned()
    }

    /// Anonymizes domain names in a string, preserving the TLD.
    fn anonymize_domains(&self, input: &str) -> String {
        DOMAIN_PATTERN
            .replace_all(input, |caps: &regex::Captures| {
                let domain = &caps[0];

                // Extract TLD (last segment after final dot)
                let tld = domain.rsplit('.').next().unwrap_or("");
                let tld_lower = tld.to_lowercase();

                // Skip if it looks like a file extension (not a real TLD)
                if FILE_EXTENSIONS.contains(&tld_lower.as_str()) {
                    return domain.to_string();
                }

                // Skip if TLD is not recognized (likely a filename)
                if !KNOWN_TLDS.contains(&tld_lower.as_str()) && tld.len() != 2 {
                    // Allow 2-letter TLDs as country codes even if not in list
                    return domain.to_string();
                }

                // Hash the domain part (everything before TLD)
                let domain_part = &domain[..domain.len() - tld.len() - 1];
                let hash = self.hash_value(domain_part);
                format!("<domain:{hash}>.{tld}")
            })
            .into_owned()
    }

    /// Hashes a value using blake3 with the session salt.
    ///
    /// Returns the first 8 characters of the hex-encoded hash for readability.
    fn hash_value(&self, value: &str) -> String {
        let mut hasher = blake3::Hasher::new_keyed(&self.salt);
        hasher.update(value.as_bytes());
        let hash = hasher.finalize();

        hash.to_hex()[..8].to_string()
    }
}

impl Default for IdentityAnonymizer {
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

    // =========================================================================
    // IdentityAnonymizer Construction Tests
    // =========================================================================

    #[test]
    fn identity_new_creates_instance_with_random_salt() {
        let anon1 = IdentityAnonymizer::new();
        let anon2 = IdentityAnonymizer::new();

        // Different instances should produce different hashes
        let input = "192.168.1.1";
        let hash1 = anon1.anonymize_string(input);
        let hash2 = anon2.anonymize_string(input);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn identity_with_seed_creates_deterministic_instance() {
        let anon1 = IdentityAnonymizer::with_seed_and_username(42, None);
        let anon2 = IdentityAnonymizer::with_seed_and_username(42, None);

        let input = "192.168.1.1";
        let hash1 = anon1.anonymize_string(input);
        let hash2 = anon2.anonymize_string(input);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn identity_different_seeds_produce_different_hashes() {
        let anon1 = IdentityAnonymizer::with_seed_and_username(42, None);
        let anon2 = IdentityAnonymizer::with_seed_and_username(43, None);

        let input = "192.168.1.1";
        let hash1 = anon1.anonymize_string(input);
        let hash2 = anon2.anonymize_string(input);

        assert_ne!(hash1, hash2);
    }

    // =========================================================================
    // IPv4 Detection Tests
    // =========================================================================

    #[test]
    fn ipv4_detection_simple() {
        let anon = IdentityAnonymizer::with_seed_and_username(1, None);
        let input = "Connect to 192.168.1.1";
        let result = anon.anonymize_string(input);

        assert!(result.starts_with("Connect to <ip:"));
        assert!(result.ends_with('>'));
        assert!(!result.contains("192.168.1.1"));
    }

    #[test]
    fn ipv4_detection_multiple() {
        let anon = IdentityAnonymizer::with_seed_and_username(1, None);
        let input = "From 10.0.0.1 to 10.0.0.2";
        let result = anon.anonymize_string(input);

        assert!(!result.contains("10.0.0.1"));
        assert!(!result.contains("10.0.0.2"));

        // Should have two different hashes
        let count = result.matches("<ip:").count();
        assert_eq!(count, 2);
    }

    #[test]
    fn ipv4_detection_edge_values() {
        let anon = IdentityAnonymizer::with_seed_and_username(1, None);

        // Maximum values
        let result = anon.anonymize_string("IP: 255.255.255.255");
        assert!(result.contains("<ip:"));

        // Minimum values
        let result = anon.anonymize_string("IP: 0.0.0.0");
        assert!(result.contains("<ip:"));
    }

    #[test]
    fn ipv4_skips_invalid_addresses() {
        let anon = IdentityAnonymizer::with_seed_and_username(1, None);

        // Invalid IP: octets > 255 should NOT be anonymized
        let result = anon.anonymize_string("Not an IP: 999.999.999.999");
        assert!(!result.contains("<ip:"));
        assert!(result.contains("999.999.999.999"));

        // Invalid IP: 256 is out of range
        let result = anon.anonymize_string("Bad IP: 256.1.1.1");
        assert!(!result.contains("<ip:"));
        assert!(result.contains("256.1.1.1"));
    }

    // =========================================================================
    // IPv6 Detection Tests
    // =========================================================================

    #[test]
    fn ipv6_detection_loopback() {
        let anon = IdentityAnonymizer::with_seed_and_username(1, None);
        let input = "Listening on ::1";
        let result = anon.anonymize_string(input);

        assert!(result.contains("<ip:"));
        assert!(!result.contains("::1"));
    }

    #[test]
    fn ipv6_detection_full_form() {
        let anon = IdentityAnonymizer::with_seed_and_username(1, None);
        let input = "Address: 2001:0db8:85a3:0000:0000:8a2e:0370:7334";
        let result = anon.anonymize_string(input);

        assert!(result.contains("<ip:"));
        assert!(!result.contains("2001:"));
    }

    #[test]
    fn ipv6_detection_compressed() {
        let anon = IdentityAnonymizer::with_seed_and_username(1, None);
        let input = "Connected to 2001:db8::1";
        let result = anon.anonymize_string(input);

        assert!(result.contains("<ip:"));
        assert!(!result.contains("2001:db8::1"));
    }

    #[test]
    fn ipv6_detection_link_local() {
        let anon = IdentityAnonymizer::with_seed_and_username(1, None);
        let input = "Link-local: fe80::1";
        let result = anon.anonymize_string(input);

        assert!(result.contains("<ip:"));
        assert!(!result.contains("fe80::1"));
    }

    // =========================================================================
    // Domain Detection Tests
    // =========================================================================

    #[test]
    fn domain_detection_simple() {
        let anon = IdentityAnonymizer::with_seed_and_username(1, None);
        let input = "Fetched from example.com";
        let result = anon.anonymize_string(input);

        assert!(result.contains("<domain:"));
        assert!(result.ends_with(">.com"));
        assert!(!result.contains("example"));
    }

    #[test]
    fn domain_detection_subdomain() {
        let anon = IdentityAnonymizer::with_seed_and_username(1, None);
        let input = "API at api.github.com";
        let result = anon.anonymize_string(input);

        assert!(result.contains("<domain:"));
        assert!(result.contains(">.com"));
        assert!(!result.contains("github"));
        assert!(!result.contains("api.github"));
    }

    #[test]
    fn domain_detection_preserves_tld() {
        let anon = IdentityAnonymizer::with_seed_and_username(1, None);

        // .org
        let result = anon.anonymize_string("Visit mozilla.org");
        assert!(result.contains(">.org"));

        // .io
        let result = anon.anonymize_string("Host on codeberg.io");
        assert!(result.contains(">.io"));

        // .dev
        let result = anon.anonymize_string("Check web.dev");
        assert!(result.contains(">.dev"));
    }

    #[test]
    fn domain_skips_file_extensions() {
        let anon = IdentityAnonymizer::with_seed_and_username(1, None);

        // File extensions should NOT be anonymized
        let result = anon.anonymize_string("Open image.jpg");
        assert!(!result.contains("<domain:"));
        assert_eq!(result, "Open image.jpg");

        let result = anon.anonymize_string("Edit config.json");
        assert!(!result.contains("<domain:"));
        assert_eq!(result, "Edit config.json");

        let result = anon.anonymize_string("Read main.rs");
        assert!(!result.contains("<domain:"));
        assert_eq!(result, "Read main.rs");
    }

    // =========================================================================
    // Username Detection Tests
    // =========================================================================

    #[test]
    fn username_detected() {
        let anon = IdentityAnonymizer::with_seed_and_username(1, Some("johndoe"));
        let input = "User: johndoe logged in";
        let result = anon.anonymize_string(input);

        assert!(result.contains("<user:"));
        assert!(!result.contains("johndoe"));
    }

    #[test]
    fn username_case_insensitive() {
        let anon = IdentityAnonymizer::with_seed_and_username(1, Some("JohnDoe"));
        let input = "User: johndoe or JOHNDOE";
        let result = anon.anonymize_string(input);

        // Both should be replaced
        assert!(!result.to_lowercase().contains("johndoe"));
        let count = result.matches("<user:").count();
        assert_eq!(count, 2);
    }

    #[test]
    fn username_respects_word_boundaries() {
        let anon = IdentityAnonymizer::with_seed_and_username(1, Some("john"));
        let input = "User john is not johnson";
        let result = anon.anonymize_string(input);

        // "john" replaced, but "johnson" should stay
        assert!(result.contains("<user:"));
        assert!(result.contains("johnson"));
    }

    #[test]
    fn no_username_does_nothing() {
        let anon = IdentityAnonymizer::with_seed_and_username(1, None);
        let input = "User: admin logged in";
        let result = anon.anonymize_string(input);

        assert!(!result.contains("<user:"));
        assert_eq!(result, "User: admin logged in");
    }

    // =========================================================================
    // Combined Anonymization Tests
    // =========================================================================

    #[test]
    fn combined_all_types() {
        let anon = IdentityAnonymizer::with_seed_and_username(1, Some("admin"));
        let input = "User admin connected from 192.168.1.1 to api.example.com";
        let result = anon.anonymize_string(input);

        assert!(result.contains("<user:"));
        assert!(result.contains("<ip:"));
        assert!(result.contains("<domain:"));
        assert!(!result.contains("admin"));
        assert!(!result.contains("192.168.1.1"));
        assert!(!result.contains("example"));
    }

    #[test]
    fn combined_no_matches() {
        let anon = IdentityAnonymizer::with_seed_and_username(1, None);
        let input = "Hello world";
        let result = anon.anonymize_string(input);

        assert_eq!(result, "Hello world");
    }

    // =========================================================================
    // Consistency Tests
    // =========================================================================

    #[test]
    fn identity_consistency_same_instance() {
        let anon = IdentityAnonymizer::with_seed_and_username(1, None);
        let input = "IP: 192.168.1.1 and example.com";

        let result1 = anon.anonymize_string(input);
        let result2 = anon.anonymize_string(input);

        assert_eq!(result1, result2);
    }

    #[test]
    fn identity_consistency_same_value_same_hash() {
        let anon = IdentityAnonymizer::with_seed_and_username(1, None);

        // Same IP in different contexts should produce same hash
        let result1 = anon.anonymize_string("From 192.168.1.1");
        let result2 = anon.anonymize_string("To 192.168.1.1");

        // Extract the hash parts
        let hash1 = result1
            .split("<ip:")
            .nth(1)
            .and_then(|s| s.split('>').next());
        let hash2 = result2
            .split("<ip:")
            .nth(1)
            .and_then(|s| s.split('>').next());

        assert_eq!(hash1, hash2);
    }

    // =========================================================================
    // Hash Format Tests (Identity)
    // =========================================================================

    #[test]
    fn identity_hash_is_8_hex_characters() {
        let anon = IdentityAnonymizer::with_seed_and_username(1, None);
        let result = anon.anonymize_string("IP: 192.168.1.1");

        // Extract hash from <ip:XXXXXXXX>
        let hash = result
            .split("<ip:")
            .nth(1)
            .and_then(|s| s.split('>').next())
            .unwrap();

        assert_eq!(hash.len(), 8);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn identity_hash_is_lowercase() {
        let anon = IdentityAnonymizer::with_seed_and_username(1, None);
        let result = anon.anonymize_string("IP: 192.168.1.1");

        // Extract hash
        let hash = result
            .split("<ip:")
            .nth(1)
            .and_then(|s| s.split('>').next())
            .unwrap();

        assert_eq!(hash, hash.to_lowercase());
    }

    // =========================================================================
    // Edge Case Tests
    // =========================================================================

    #[test]
    fn edge_case_empty_string() {
        let anon = IdentityAnonymizer::with_seed_and_username(1, None);
        let result = anon.anonymize_string("");
        assert_eq!(result, "");
    }

    #[test]
    fn edge_case_only_ip() {
        let anon = IdentityAnonymizer::with_seed_and_username(1, None);
        let result = anon.anonymize_string("192.168.1.1");

        assert!(result.starts_with("<ip:"));
        assert!(result.ends_with('>'));
    }

    #[test]
    fn edge_case_ip_in_url() {
        let anon = IdentityAnonymizer::with_seed_and_username(1, None);
        let result = anon.anonymize_string("http://192.168.1.1:8080/path");

        assert!(result.contains("<ip:"));
        assert!(!result.contains("192.168.1.1"));
    }

    #[test]
    fn edge_case_complex_domain() {
        let anon = IdentityAnonymizer::with_seed_and_username(1, None);
        let input = "https://sub.domain.example.com/path";
        let result = anon.anonymize_string(input);

        assert!(result.contains("<domain:"));
        assert!(result.contains(">.com"));
        assert!(result.contains("/path"));
    }
}
