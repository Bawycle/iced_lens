// SPDX-License-Identifier: MPL-2.0
//! Image prefetch cache for faster navigation.
//!
//! This module provides background preloading of adjacent images in the media list,
//! reducing perceived latency when navigating between images.
//!
//! # Design
//!
//! - **LRU eviction**: Least recently used images are evicted first
//! - **Memory-bounded**: Total cache size limited by configurable byte limit
//! - **Path-keyed**: Images indexed by their file path
//! - **Async loading**: Prefetching runs in background without blocking UI
//!
//! # Usage
//!
//! ```ignore
//! let mut cache = ImagePrefetchCache::new(config);
//!
//! // Check if image is already cached
//! if let Some(image_data) = cache.get(&path) {
//!     // Use cached image
//! }
//!
//! // Prefetch adjacent images in background
//! cache.prefetch_paths(paths);
//! ```

use crate::error::Result;
use crate::media::ImageData;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Default prefetch cache size in bytes (32 MB).
/// Allows ~4 full HD images (8 MB each) or ~16 smaller images.
pub const DEFAULT_PREFETCH_CACHE_BYTES: usize = 32 * 1024 * 1024;

/// Minimum prefetch cache size in bytes (8 MB).
pub const MIN_PREFETCH_CACHE_BYTES: usize = 8 * 1024 * 1024;

/// Maximum prefetch cache size in bytes (128 MB).
pub const MAX_PREFETCH_CACHE_BYTES: usize = 128 * 1024 * 1024;

/// Default maximum number of images to cache.
pub const DEFAULT_MAX_IMAGES: usize = 16;

/// Minimum images to cache.
pub const MIN_MAX_IMAGES: usize = 4;

/// Maximum images to cache.
pub const MAX_MAX_IMAGES: usize = 32;

/// Default number of images to prefetch in each direction.
pub const DEFAULT_PREFETCH_COUNT: usize = 2;

/// Configuration for the prefetch cache.
#[derive(Debug, Clone, Copy)]
pub struct PrefetchConfig {
    /// Maximum cache size in bytes.
    pub max_bytes: usize,

    /// Maximum number of images to cache.
    pub max_images: usize,

    /// Number of images to prefetch in each direction (next/previous).
    pub prefetch_count: usize,

    /// Whether prefetching is enabled.
    pub enabled: bool,
}

impl Default for PrefetchConfig {
    fn default() -> Self {
        Self {
            max_bytes: DEFAULT_PREFETCH_CACHE_BYTES,
            max_images: DEFAULT_MAX_IMAGES,
            prefetch_count: DEFAULT_PREFETCH_COUNT,
            enabled: true,
        }
    }
}

impl PrefetchConfig {
    /// Creates a new prefetch configuration with specified limits.
    #[must_use]
    pub fn new(max_bytes: usize, max_images: usize, prefetch_count: usize) -> Self {
        Self {
            max_bytes: max_bytes.clamp(MIN_PREFETCH_CACHE_BYTES, MAX_PREFETCH_CACHE_BYTES),
            max_images: max_images.clamp(MIN_MAX_IMAGES, MAX_MAX_IMAGES),
            prefetch_count,
            enabled: true,
        }
    }

    /// Creates a disabled prefetch configuration.
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }
}

/// Cached image entry with metadata.
#[derive(Debug, Clone)]
struct CacheEntry {
    /// The loaded image data.
    image: Arc<ImageData>,

    /// Size of this entry in bytes (width * height * 4 for RGBA).
    size_bytes: usize,
}

impl CacheEntry {
    fn new(image: ImageData) -> Self {
        // Calculate size: width * height * 4 bytes per pixel (RGBA)
        let size_bytes = (image.width as usize) * (image.height as usize) * 4;
        Self {
            image: Arc::new(image),
            size_bytes,
        }
    }
}

/// Statistics about prefetch cache performance.
#[derive(Debug, Clone, Copy, Default)]
pub struct PrefetchStats {
    /// Number of images currently in cache.
    pub image_count: usize,

    /// Total bytes currently used by cached images.
    pub total_bytes: usize,

    /// Number of cache hits (image found).
    pub hits: u64,

    /// Number of cache misses (image not found).
    pub misses: u64,

    /// Number of images evicted due to limits.
    pub evictions: u64,

    /// Number of images inserted.
    pub insertions: u64,
}

impl PrefetchStats {
    /// Returns the cache hit rate as a percentage (0.0 - 100.0).
    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }
}

/// LRU cache for prefetched images.
///
/// Provides memory-bounded caching with LRU eviction policy.
/// Optimized for navigation between adjacent images.
pub struct ImagePrefetchCache {
    /// LRU cache mapping file paths to image entries.
    cache: LruCache<PathBuf, CacheEntry>,

    /// Cache configuration.
    config: PrefetchConfig,

    /// Current total size in bytes.
    current_bytes: usize,

    /// Performance statistics.
    stats: PrefetchStats,
}

impl ImagePrefetchCache {
    /// Creates a new prefetch cache with the given configuration.
    ///
    /// # Panics
    ///
    /// Panics if `DEFAULT_MAX_IMAGES` is zero, which would indicate a build configuration error.
    #[must_use]
    pub fn new(config: PrefetchConfig) -> Self {
        let capacity = NonZeroUsize::new(config.max_images).unwrap_or(
            NonZeroUsize::new(DEFAULT_MAX_IMAGES).expect("DEFAULT_MAX_IMAGES must be non-zero"),
        );

        Self {
            cache: LruCache::new(capacity),
            config,
            current_bytes: 0,
            stats: PrefetchStats::default(),
        }
    }

    /// Creates a new prefetch cache with default configuration.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(PrefetchConfig::default())
    }

    /// Returns whether prefetching is enabled.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Returns the number of images to prefetch in each direction.
    #[must_use]
    pub fn prefetch_count(&self) -> usize {
        self.config.prefetch_count
    }

    /// Inserts an image into the cache.
    ///
    /// Returns `true` if the image was inserted, `false` if caching is disabled
    /// or the image is too large.
    pub fn insert(&mut self, path: PathBuf, image: ImageData) -> bool {
        if !self.config.enabled {
            return false;
        }

        let entry = CacheEntry::new(image);
        let image_size = entry.size_bytes;

        // Don't cache images larger than half the cache size
        if image_size > self.config.max_bytes / 2 {
            return false;
        }

        // Evict images until we have room
        while self.current_bytes + image_size > self.config.max_bytes && !self.cache.is_empty() {
            if let Some((_, evicted)) = self.cache.pop_lru() {
                self.current_bytes = self.current_bytes.saturating_sub(evicted.size_bytes);
                self.stats.evictions += 1;
            }
        }

        // Check if we already have this path (update if so)
        if let Some(existing) = self.cache.pop(&path) {
            self.current_bytes = self.current_bytes.saturating_sub(existing.size_bytes);
        }

        self.current_bytes += entry.size_bytes;
        self.cache.put(path, entry);
        self.stats.insertions += 1;
        self.stats.image_count = self.cache.len();
        self.stats.total_bytes = self.current_bytes;

        true
    }

    /// Gets an image from the cache by path.
    ///
    /// Updates LRU order on access.
    /// Returns a clone of the `ImageData` (the handle is reference-counted internally).
    pub fn get(&mut self, path: &Path) -> Option<ImageData> {
        if !self.config.enabled {
            return None;
        }

        if let Some(entry) = self.cache.get(path) {
            self.stats.hits += 1;
            // Clone the Arc's inner ImageData - this is cheap due to Arc in ImageData
            Some((*entry.image).clone())
        } else {
            self.stats.misses += 1;
            None
        }
    }

    /// Checks if an image is cached for the given path without updating LRU order.
    #[must_use]
    pub fn contains(&self, path: &Path) -> bool {
        if !self.config.enabled {
            return false;
        }
        self.cache.contains(path)
    }

    /// Returns paths that need to be prefetched (not already in cache).
    ///
    /// Given a list of paths to prefetch, returns only those not already cached.
    #[must_use]
    pub fn paths_to_prefetch(&self, paths: &[PathBuf]) -> Vec<PathBuf> {
        if !self.config.enabled {
            return Vec::new();
        }

        paths
            .iter()
            .filter(|p| !self.cache.contains(p.as_path()))
            .cloned()
            .collect()
    }

    /// Clears all cached images.
    pub fn clear(&mut self) {
        self.cache.clear();
        self.current_bytes = 0;
        self.stats.image_count = 0;
        self.stats.total_bytes = 0;
    }

    /// Returns the current cache statistics.
    #[must_use]
    pub fn stats(&self) -> PrefetchStats {
        self.stats
    }

    /// Returns the current number of cached images.
    #[must_use]
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Returns whether the cache is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Returns the current memory usage in bytes.
    #[must_use]
    pub fn memory_usage(&self) -> usize {
        self.current_bytes
    }

    /// Returns the cache configuration.
    #[must_use]
    pub fn config(&self) -> &PrefetchConfig {
        &self.config
    }
}

impl std::fmt::Debug for ImagePrefetchCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImagePrefetchCache")
            .field("enabled", &self.config.enabled)
            .field("image_count", &self.cache.len())
            .field("memory_usage", &self.current_bytes)
            .field("max_bytes", &self.config.max_bytes)
            .field("max_images", &self.config.max_images)
            .field("prefetch_count", &self.config.prefetch_count)
            .field("stats", &self.stats)
            .finish()
    }
}

/// Loads an image for prefetching.
///
/// This is the async function called by the prefetch task.
/// Returns the path and loaded image data, or an error.
pub async fn load_image_for_prefetch(path: PathBuf) -> (PathBuf, Result<ImageData>) {
    let path_clone = path.clone();
    let result = tokio::task::spawn_blocking(move || crate::media::load_image(&path_clone))
        .await
        .unwrap_or_else(|e| {
            Err(crate::error::Error::Io(format!(
                "Prefetch task failed: {e}"
            )))
        });

    (path, result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image(width: u32, height: u32) -> ImageData {
        let pixels = vec![0u8; (width * height * 4) as usize];
        ImageData::from_rgba(width, height, pixels)
    }

    #[test]
    fn new_cache_is_empty() {
        let cache = ImagePrefetchCache::with_defaults();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.memory_usage(), 0);
    }

    #[test]
    fn insert_and_get_image() {
        let mut cache = ImagePrefetchCache::with_defaults();
        let path = PathBuf::from("/test/image.jpg");
        let image = create_test_image(100, 100);

        assert!(cache.insert(path.clone(), image));
        assert_eq!(cache.len(), 1);

        let retrieved = cache.get(&path);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().width, 100);
    }

    #[test]
    fn disabled_cache_returns_none() {
        let mut cache = ImagePrefetchCache::new(PrefetchConfig::disabled());
        let path = PathBuf::from("/test/image.jpg");
        let image = create_test_image(100, 100);

        assert!(!cache.insert(path.clone(), image));
        assert!(cache.get(&path).is_none());
    }

    #[test]
    fn lru_eviction_on_byte_limit() {
        let config = PrefetchConfig {
            max_bytes: 100_000, // Enough for ~2.5 images at 50x50 (10,000 bytes each)
            max_images: 100,
            prefetch_count: 2,
            enabled: true,
        };
        let mut cache = ImagePrefetchCache::new(config);

        // Insert images that exceed the byte limit (each image is 50*50*4 = 10,000 bytes)
        // With max_bytes 100,000, we can fit ~10 images, but inserting 15 should evict some
        for i in 0..15 {
            let path = PathBuf::from(format!("/test/image{i}.jpg"));
            let image = create_test_image(50, 50); // 10,000 bytes each
            cache.insert(path, image);
        }

        // Should have evicted some images
        assert!(cache.memory_usage() <= 100_000);
        assert!(cache.stats().evictions > 0);
    }

    #[test]
    fn contains_checks_without_updating_lru() {
        let mut cache = ImagePrefetchCache::with_defaults();
        let path = PathBuf::from("/test/image.jpg");
        let image = create_test_image(100, 100);

        cache.insert(path.clone(), image);

        // Contains should not update LRU order
        assert!(cache.contains(&path));
        assert!(!cache.contains(Path::new("/nonexistent")));
    }

    #[test]
    fn paths_to_prefetch_filters_cached() {
        let mut cache = ImagePrefetchCache::with_defaults();

        // Insert one image
        let cached_path = PathBuf::from("/test/cached.jpg");
        cache.insert(cached_path.clone(), create_test_image(100, 100));

        // Check which paths need prefetching
        let paths = vec![
            cached_path.clone(),
            PathBuf::from("/test/not_cached1.jpg"),
            PathBuf::from("/test/not_cached2.jpg"),
        ];

        let to_prefetch = cache.paths_to_prefetch(&paths);
        assert_eq!(to_prefetch.len(), 2);
        assert!(!to_prefetch.contains(&cached_path));
    }

    #[test]
    fn clear_removes_all_images() {
        let mut cache = ImagePrefetchCache::with_defaults();

        for i in 0..5 {
            let path = PathBuf::from(format!("/test/image{i}.jpg"));
            cache.insert(path, create_test_image(50, 50));
        }

        assert_eq!(cache.len(), 5);
        cache.clear();
        assert!(cache.is_empty());
        assert_eq!(cache.memory_usage(), 0);
    }

    #[test]
    fn stats_track_hits_and_misses() {
        let mut cache = ImagePrefetchCache::with_defaults();
        let path = PathBuf::from("/test/image.jpg");
        cache.insert(path.clone(), create_test_image(100, 100));

        // Hit
        let _ = cache.get(&path);
        assert_eq!(cache.stats().hits, 1);
        assert_eq!(cache.stats().misses, 0);

        // Miss
        let _ = cache.get(Path::new("/nonexistent"));
        assert_eq!(cache.stats().hits, 1);
        assert_eq!(cache.stats().misses, 1);

        // Hit rate should be 50%
        assert!((cache.stats().hit_rate() - 50.0).abs() < 0.01);
    }

    #[test]
    fn large_image_not_cached() {
        let config = PrefetchConfig {
            max_bytes: MIN_PREFETCH_CACHE_BYTES,
            max_images: 100,
            prefetch_count: 2,
            enabled: true,
        };
        let mut cache = ImagePrefetchCache::new(config);

        // Image larger than half the cache size
        let large_image = create_test_image(2000, 2000); // 16 MB
        let path = PathBuf::from("/test/large.jpg");
        assert!(!cache.insert(path, large_image));
        assert!(cache.is_empty());
    }

    #[test]
    fn duplicate_path_updates_image() {
        let mut cache = ImagePrefetchCache::with_defaults();
        let path = PathBuf::from("/test/image.jpg");

        let image1 = create_test_image(100, 100);
        let image2 = create_test_image(200, 200);

        cache.insert(path.clone(), image1);
        let initial_size = cache.memory_usage();

        cache.insert(path.clone(), image2);
        assert_eq!(cache.len(), 1); // Still one image
        assert!(cache.memory_usage() > initial_size); // Updated size

        let retrieved = cache.get(&path).unwrap();
        assert_eq!(retrieved.width, 200);
    }

    #[test]
    fn config_clamps_values() {
        let config = PrefetchConfig::new(0, 0, 2);
        assert_eq!(config.max_bytes, MIN_PREFETCH_CACHE_BYTES);
        assert_eq!(config.max_images, MIN_MAX_IMAGES);

        let config = PrefetchConfig::new(usize::MAX, usize::MAX, 2);
        assert_eq!(config.max_bytes, MAX_PREFETCH_CACHE_BYTES);
        assert_eq!(config.max_images, MAX_MAX_IMAGES);
    }
}
