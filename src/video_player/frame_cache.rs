// SPDX-License-Identifier: MPL-2.0
//! LRU frame cache for video playback optimization.
//!
//! This module provides a memory-bounded cache for decoded video frames,
//! optimizing seek operations and reducing decoder thrashing during scrubbing.
//!
//! # Design
//!
//! - **LRU eviction**: Least recently used frames are evicted first
//! - **Memory-bounded**: Total cache size limited by configurable byte limit
//! - **Keyframe-focused**: Only keyframes are cached (independently decodable)
//! - **PTS-keyed**: Frames indexed by presentation timestamp (microseconds)
//!
//! # Usage
//!
//! ```ignore
//! let mut cache = FrameCache::new(CacheConfig::default());
//! cache.insert(frame);
//! if let Some(cached) = cache.get(pts_micros) {
//!     // Use cached frame
//! }
//! ```

use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;

use super::decoder::DecodedFrame;
use super::time_units::pts_to_micros;

/// Default cache size in bytes (64 MB).
/// Allows ~8 frames at 1080p or ~64 frames at 480p.
pub const DEFAULT_CACHE_SIZE_BYTES: usize = 64 * 1024 * 1024;

/// Minimum cache size in bytes (16 MB).
pub const MIN_CACHE_SIZE_BYTES: usize = 16 * 1024 * 1024;

/// Maximum cache size in bytes (512 MB).
pub const MAX_CACHE_SIZE_BYTES: usize = 512 * 1024 * 1024;

/// Default maximum number of frames to cache.
pub const DEFAULT_MAX_FRAMES: usize = 64;

/// Minimum frames to cache.
pub const MIN_MAX_FRAMES: usize = 8;

/// Maximum frames to cache.
pub const MAX_MAX_FRAMES: usize = 256;

/// Tolerance for PTS matching in microseconds (50ms).
/// Used for "nearest frame" lookups during seeking.
const PTS_TOLERANCE_MICROS: i64 = 50_000;

/// Configuration for the frame cache.
#[derive(Debug, Clone, Copy)]
pub struct CacheConfig {
    /// Maximum cache size in bytes.
    pub max_bytes: usize,

    /// Maximum number of frames to cache.
    pub max_frames: usize,

    /// Whether caching is enabled.
    pub enabled: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_bytes: DEFAULT_CACHE_SIZE_BYTES,
            max_frames: DEFAULT_MAX_FRAMES,
            enabled: true,
        }
    }
}

impl CacheConfig {
    /// Creates a new cache configuration with specified limits.
    #[must_use]
    pub fn new(max_bytes: usize, max_frames: usize) -> Self {
        Self {
            max_bytes: max_bytes.clamp(MIN_CACHE_SIZE_BYTES, MAX_CACHE_SIZE_BYTES),
            max_frames: max_frames.clamp(MIN_MAX_FRAMES, MAX_MAX_FRAMES),
            enabled: true,
        }
    }

    /// Creates a disabled cache configuration.
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }
}

/// Statistics about cache performance.
#[derive(Debug, Clone, Copy, Default)]
pub struct CacheStats {
    /// Number of frames currently in cache.
    pub frame_count: usize,

    /// Total bytes currently used by cached frames.
    pub total_bytes: usize,

    /// Number of cache hits (frame found).
    pub hits: u64,

    /// Number of cache misses (frame not found).
    pub misses: u64,

    /// Number of frames evicted due to limits.
    pub evictions: u64,

    /// Number of frames inserted.
    pub insertions: u64,
}

impl CacheStats {
    /// Returns the cache hit rate as a percentage (0.0 - 100.0).
    // Allow cast_precision_loss: cache statistics - exact precision not required
    // for percentages. Hit/miss counts are unlikely to exceed f64 mantissa (2^52).
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

/// Cached frame entry with metadata.
#[derive(Debug, Clone)]
struct CacheEntry {
    /// The decoded frame data.
    frame: Arc<DecodedFrame>,

    /// Size of this entry in bytes.
    size_bytes: usize,
}

impl CacheEntry {
    fn new(frame: DecodedFrame) -> Self {
        let size_bytes = frame.size_bytes();
        Self {
            frame: Arc::new(frame),
            size_bytes,
        }
    }
}

/// LRU frame cache for decoded video frames.
///
/// Provides memory-bounded caching with LRU eviction policy.
/// Optimized for seek operations and timeline scrubbing.
pub struct FrameCache {
    /// LRU cache mapping PTS (microseconds) to frame entries.
    cache: LruCache<i64, CacheEntry>,

    /// Cache configuration.
    config: CacheConfig,

    /// Current total size in bytes.
    current_bytes: usize,

    /// Performance statistics.
    stats: CacheStats,
}

impl FrameCache {
    /// Creates a new frame cache with the given configuration.
    ///
    /// # Panics
    ///
    /// Panics if the compile-time default `DEFAULT_MAX_FRAMES` is zero,
    /// which would indicate a build configuration error.
    #[must_use]
    pub fn new(config: CacheConfig) -> Self {
        let capacity = NonZeroUsize::new(config.max_frames).unwrap_or(
            NonZeroUsize::new(DEFAULT_MAX_FRAMES).expect("DEFAULT_MAX_FRAMES must be non-zero"),
        );

        Self {
            cache: LruCache::new(capacity),
            config,
            current_bytes: 0,
            stats: CacheStats::default(),
        }
    }

    /// Creates a new frame cache with default configuration.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(CacheConfig::default())
    }

    /// Returns whether caching is enabled.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Inserts a frame into the cache.
    ///
    /// Only keyframes should be cached for independent decodability.
    /// Returns `true` if the frame was inserted, `false` if caching is disabled
    /// or the frame is too large.
    pub fn insert(&mut self, frame: DecodedFrame, is_keyframe: bool) -> bool {
        if !self.config.enabled {
            return false;
        }

        // Only cache keyframes (independently decodable)
        if !is_keyframe {
            return false;
        }

        let frame_size = frame.size_bytes();

        // Don't cache frames larger than half the cache size
        if frame_size > self.config.max_bytes / 2 {
            return false;
        }

        let pts_micros = pts_to_micros(frame.pts_secs);

        // Evict frames until we have room
        while self.current_bytes + frame_size > self.config.max_bytes && !self.cache.is_empty() {
            if let Some((_, evicted)) = self.cache.pop_lru() {
                self.current_bytes = self.current_bytes.saturating_sub(evicted.size_bytes);
                self.stats.evictions += 1;
            }
        }

        // Check if we already have this frame (update if so)
        if let Some(existing) = self.cache.pop(&pts_micros) {
            self.current_bytes = self.current_bytes.saturating_sub(existing.size_bytes);
        }

        let entry = CacheEntry::new(frame);
        self.current_bytes += entry.size_bytes;
        self.cache.put(pts_micros, entry);
        self.stats.insertions += 1;
        self.stats.frame_count = self.cache.len();
        self.stats.total_bytes = self.current_bytes;

        true
    }

    /// Gets a frame from the cache by exact PTS match.
    ///
    /// Updates LRU order on access.
    pub fn get(&mut self, pts_secs: f64) -> Option<Arc<DecodedFrame>> {
        if !self.config.enabled {
            return None;
        }

        let pts_micros = pts_to_micros(pts_secs);

        if let Some(entry) = self.cache.get(&pts_micros) {
            self.stats.hits += 1;
            Some(Arc::clone(&entry.frame))
        } else {
            self.stats.misses += 1;
            None
        }
    }

    /// Gets the nearest cached frame to the given PTS.
    ///
    /// Searches for frames within `PTS_TOLERANCE_MICROS` of the target.
    /// Returns the closest match if found.
    pub fn get_nearest(&mut self, pts_secs: f64) -> Option<Arc<DecodedFrame>> {
        if !self.config.enabled {
            return None;
        }

        let target_micros = pts_to_micros(pts_secs);

        // First try exact match
        if let Some(entry) = self.cache.get(&target_micros) {
            self.stats.hits += 1;
            return Some(Arc::clone(&entry.frame));
        }

        // Search for nearest frame within tolerance
        let mut best_match: Option<(i64, Arc<DecodedFrame>)> = None;
        let mut best_distance = i64::MAX;

        // Note: This iterates without updating LRU order (peek)
        for (&pts, entry) in &self.cache {
            let distance = (pts - target_micros).abs();
            if distance < best_distance && distance <= PTS_TOLERANCE_MICROS {
                best_distance = distance;
                best_match = Some((pts, Arc::clone(&entry.frame)));
            }
        }

        if let Some((pts, frame)) = best_match {
            // Update LRU order for the matched frame
            let _ = self.cache.get(&pts);
            self.stats.hits += 1;
            Some(frame)
        } else {
            self.stats.misses += 1;
            None
        }
    }

    /// Gets a frame at or before the given PTS (for seek operations).
    ///
    /// Useful when seeking to find the nearest keyframe before target.
    pub fn get_at_or_before(&mut self, pts_secs: f64) -> Option<Arc<DecodedFrame>> {
        if !self.config.enabled {
            return None;
        }

        let target_micros = pts_to_micros(pts_secs);

        let mut best_match: Option<(i64, Arc<DecodedFrame>)> = None;

        for (&pts, entry) in &self.cache {
            if pts <= target_micros {
                let should_update = match &best_match {
                    None => true,
                    Some((best_pts, _)) => pts > *best_pts,
                };
                if should_update {
                    best_match = Some((pts, Arc::clone(&entry.frame)));
                }
            }
        }

        if let Some((pts, frame)) = best_match {
            let _ = self.cache.get(&pts);
            self.stats.hits += 1;
            Some(frame)
        } else {
            self.stats.misses += 1;
            None
        }
    }

    /// Checks if a frame is cached for the given PTS.
    #[must_use]
    pub fn contains(&self, pts_secs: f64) -> bool {
        if !self.config.enabled {
            return false;
        }
        let pts_micros = pts_to_micros(pts_secs);
        self.cache.contains(&pts_micros)
    }

    /// Clears all cached frames.
    pub fn clear(&mut self) {
        self.cache.clear();
        self.current_bytes = 0;
        self.stats.frame_count = 0;
        self.stats.total_bytes = 0;
    }

    /// Returns the current cache statistics.
    #[must_use]
    pub fn stats(&self) -> CacheStats {
        self.stats
    }

    /// Returns the current number of cached frames.
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
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }

    /// Updates the cache configuration.
    ///
    /// If the new limits are smaller, excess frames will be evicted.
    ///
    /// # Panics
    ///
    /// Panics if the compile-time default `DEFAULT_MAX_FRAMES` is zero,
    /// which would indicate a build configuration error.
    pub fn set_config(&mut self, config: CacheConfig) {
        self.config = config;

        if !config.enabled {
            self.clear();
            return;
        }

        // Evict frames if new limits are smaller
        while self.current_bytes > config.max_bytes && !self.cache.is_empty() {
            if let Some((_, evicted)) = self.cache.pop_lru() {
                self.current_bytes = self.current_bytes.saturating_sub(evicted.size_bytes);
                self.stats.evictions += 1;
            }
        }

        // Resize LRU capacity if needed
        if self.cache.len() > config.max_frames {
            self.cache
                .resize(
                    NonZeroUsize::new(config.max_frames).unwrap_or(
                        NonZeroUsize::new(DEFAULT_MAX_FRAMES)
                            .expect("DEFAULT_MAX_FRAMES must be non-zero"),
                    ),
                );
        }

        self.stats.frame_count = self.cache.len();
        self.stats.total_bytes = self.current_bytes;
    }
}

impl std::fmt::Debug for FrameCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FrameCache")
            .field("enabled", &self.config.enabled)
            .field("frame_count", &self.cache.len())
            .field("memory_usage", &self.current_bytes)
            .field("max_bytes", &self.config.max_bytes)
            .field("max_frames", &self.config.max_frames)
            .field("stats", &self.stats)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::assert_abs_diff_eq;
    use crate::video_player::time_units::micros_to_pts;

    fn create_test_frame(pts_secs: f64, size: usize) -> DecodedFrame {
        DecodedFrame {
            rgba_data: Arc::new(vec![0u8; size]),
            width: 100,
            #[allow(clippy::cast_possible_truncation)] // Test helper, values are small
            height: (size / 400) as u32, // Approximate for RGBA
            pts_secs,
        }
    }

    #[test]
    fn new_cache_is_empty() {
        let cache = FrameCache::with_defaults();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.memory_usage(), 0);
    }

    #[test]
    fn insert_and_get_frame() {
        let mut cache = FrameCache::with_defaults();
        let frame = create_test_frame(1.0, 1000);

        assert!(cache.insert(frame.clone(), true));
        assert_eq!(cache.len(), 1);

        let retrieved = cache.get(1.0);
        assert!(retrieved.is_some());
        assert_abs_diff_eq!(retrieved.unwrap().pts_secs, 1.0);
    }

    #[test]
    fn non_keyframes_not_cached() {
        let mut cache = FrameCache::with_defaults();
        let frame = create_test_frame(1.0, 1000);

        assert!(!cache.insert(frame, false)); // Not a keyframe
        assert!(cache.is_empty());
    }

    #[test]
    fn disabled_cache_returns_none() {
        let mut cache = FrameCache::new(CacheConfig::disabled());
        let frame = create_test_frame(1.0, 1000);

        assert!(!cache.insert(frame, true));
        assert!(cache.get(1.0).is_none());
    }

    #[test]
    fn lru_eviction_on_byte_limit() {
        // Use minimum valid cache size for testing
        let config = CacheConfig {
            max_bytes: 5000,
            max_frames: 100,
            enabled: true,
        };
        let mut cache = FrameCache::new(config);

        // Insert frames that exceed the byte limit
        for i in 0..10 {
            let frame = create_test_frame(f64::from(i), 1000);
            cache.insert(frame, true);
        }

        // Should have evicted some frames
        assert!(cache.memory_usage() <= 5000);
        assert!(cache.stats().evictions > 0);
    }

    #[test]
    fn get_nearest_finds_close_frame() {
        let mut cache = FrameCache::with_defaults();

        // Insert frame at 1.0 seconds
        let frame = create_test_frame(1.0, 1000);
        cache.insert(frame, true);

        // Search for 1.01 seconds (10ms away, within 50ms tolerance)
        let found = cache.get_nearest(1.01);
        assert!(found.is_some());
        assert_abs_diff_eq!(found.unwrap().pts_secs, 1.0);
    }

    #[test]
    fn get_nearest_misses_distant_frame() {
        let mut cache = FrameCache::with_defaults();

        // Insert frame at 1.0 seconds
        let frame = create_test_frame(1.0, 1000);
        cache.insert(frame, true);

        // Search for 2.0 seconds (1000ms away, outside 50ms tolerance)
        let found = cache.get_nearest(2.0);
        assert!(found.is_none());
    }

    #[test]
    fn get_at_or_before_works() {
        let mut cache = FrameCache::with_defaults();

        cache.insert(create_test_frame(1.0, 1000), true);
        cache.insert(create_test_frame(2.0, 1000), true);
        cache.insert(create_test_frame(3.0, 1000), true);

        // Should find frame at 2.0 when seeking to 2.5
        let found = cache.get_at_or_before(2.5);
        assert!(found.is_some());
        assert_abs_diff_eq!(found.unwrap().pts_secs, 2.0);

        // Should find frame at 1.0 when seeking to 1.0
        let found = cache.get_at_or_before(1.0);
        assert!(found.is_some());
        assert_abs_diff_eq!(found.unwrap().pts_secs, 1.0);
    }

    #[test]
    fn clear_removes_all_frames() {
        let mut cache = FrameCache::with_defaults();

        for i in 0..5 {
            cache.insert(create_test_frame(f64::from(i), 1000), true);
        }

        assert_eq!(cache.len(), 5);
        cache.clear();
        assert!(cache.is_empty());
        assert_eq!(cache.memory_usage(), 0);
    }

    #[test]
    fn stats_track_hits_and_misses() {
        let mut cache = FrameCache::with_defaults();
        let frame = create_test_frame(1.0, 1000);
        cache.insert(frame, true);

        // Hit
        let _ = cache.get(1.0);
        assert_eq!(cache.stats().hits, 1);
        assert_eq!(cache.stats().misses, 0);

        // Miss
        let _ = cache.get(2.0);
        assert_eq!(cache.stats().hits, 1);
        assert_eq!(cache.stats().misses, 1);

        // Hit rate should be 50%
        assert_abs_diff_eq!(cache.stats().hit_rate(), 50.0, epsilon = 0.01);
    }

    #[test]
    fn pts_conversion_round_trip() {
        let pts_secs = 1.234_567;
        let micros = pts_to_micros(pts_secs);
        let back = micros_to_pts(micros);

        // Should be accurate to microsecond precision
        assert_abs_diff_eq!(pts_secs, back, epsilon = 0.000_001);
    }

    #[test]
    fn config_clamps_values() {
        let config = CacheConfig::new(0, 0); // Too small
        assert_eq!(config.max_bytes, MIN_CACHE_SIZE_BYTES);
        assert_eq!(config.max_frames, MIN_MAX_FRAMES);

        let config = CacheConfig::new(usize::MAX, usize::MAX); // Too large
        assert_eq!(config.max_bytes, MAX_CACHE_SIZE_BYTES);
        assert_eq!(config.max_frames, MAX_MAX_FRAMES);
    }

    #[test]
    fn set_config_evicts_excess() {
        let config = CacheConfig {
            max_bytes: 100_000,
            max_frames: 100,
            enabled: true,
        };
        let mut cache = FrameCache::new(config);

        // Fill cache
        for i in 0..10 {
            cache.insert(create_test_frame(f64::from(i), 10_000), true);
        }
        assert_eq!(cache.len(), 10);

        // Reduce limit - should evict
        let new_config = CacheConfig {
            max_bytes: 30_000,
            max_frames: 100,
            enabled: true,
        };
        cache.set_config(new_config);
        assert!(cache.memory_usage() <= 30_000);
    }

    #[test]
    fn large_frame_not_cached() {
        let config = CacheConfig::new(MIN_CACHE_SIZE_BYTES, 100);
        let mut cache = FrameCache::new(config);

        // Frame larger than half the cache size
        let large_frame = create_test_frame(1.0, MIN_CACHE_SIZE_BYTES);
        assert!(!cache.insert(large_frame, true));
        assert!(cache.is_empty());
    }

    #[test]
    fn duplicate_pts_updates_frame() {
        let mut cache = FrameCache::with_defaults();

        let frame1 = create_test_frame(1.0, 1000);
        let frame2 = create_test_frame(1.0, 2000); // Same PTS, different size

        cache.insert(frame1, true);
        assert_eq!(cache.memory_usage(), 1000);

        cache.insert(frame2, true);
        assert_eq!(cache.len(), 1); // Still one frame
        assert_eq!(cache.memory_usage(), 2000); // Updated size
    }
}
