// SPDX-License-Identifier: MPL-2.0
//! Audio normalization for consistent loudness between media files.
//!
//! This module implements LUFS (Loudness Units Full Scale) analysis and
//! gain calculation for normalizing audio to a target loudness level.
//!
//! # LUFS Standard
//!
//! LUFS is the standard for loudness measurement (ITU-R BS.1770).
//! Common targets:
//! - Streaming platforms: -14 LUFS (Spotify, YouTube)
//! - Broadcast: -23 LUFS (EBU R128)
//! - This application: -16 LUFS (balanced default)
//!
//! # Usage
//!
//! ```ignore
//! let analyzer = LufsAnalyzer::new(-16.0);
//! let measured_lufs = analyzer.analyze_file("video.mp4")?;
//! let gain = analyzer.calculate_gain(measured_lufs);
//! ```

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, RwLock};

use crate::error::{Error, Result};

/// Default target LUFS level for normalization.
/// -16 LUFS is a balanced target between streaming (-14) and broadcast (-23).
pub const DEFAULT_TARGET_LUFS: f64 = -16.0;

/// Minimum LUFS value we consider valid (silence threshold).
const MIN_VALID_LUFS: f64 = -70.0;

/// Maximum gain to apply (to avoid distortion).
const MAX_GAIN_DB: f64 = 12.0;

/// Cache for LUFS measurements to avoid re-analyzing the same file.
#[derive(Debug, Default)]
pub struct LufsCache {
    /// Map from file path to measured LUFS value.
    cache: RwLock<HashMap<String, f64>>,
}

impl LufsCache {
    /// Creates a new empty LUFS cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets a cached LUFS value for a file path.
    pub fn get(&self, path: &str) -> Option<f64> {
        self.cache.read().ok()?.get(path).copied()
    }

    /// Stores a LUFS value for a file path.
    pub fn insert(&self, path: String, lufs: f64) {
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(path, lufs);
        }
    }

    /// Clears all cached values.
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }

    /// Returns the number of cached entries.
    pub fn len(&self) -> usize {
        self.cache.read().map(|c| c.len()).unwrap_or(0)
    }

    /// Returns true if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Thread-safe shared LUFS cache.
pub type SharedLufsCache = Arc<LufsCache>;

/// Creates a new shared LUFS cache.
pub fn create_lufs_cache() -> SharedLufsCache {
    Arc::new(LufsCache::new())
}

/// LUFS (Loudness Units Full Scale) analyzer for audio normalization.
///
/// Uses FFmpeg's loudnorm filter to measure integrated loudness.
#[derive(Debug, Clone)]
pub struct LufsAnalyzer {
    /// Target LUFS level for normalization.
    target_lufs: f64,
}

impl Default for LufsAnalyzer {
    fn default() -> Self {
        Self::new(DEFAULT_TARGET_LUFS)
    }
}

impl LufsAnalyzer {
    /// Creates a new LUFS analyzer with the specified target level.
    pub fn new(target_lufs: f64) -> Self {
        Self { target_lufs }
    }

    /// Returns the target LUFS level.
    pub fn target_lufs(&self) -> f64 {
        self.target_lufs
    }

    /// Sets a new target LUFS level.
    pub fn set_target_lufs(&mut self, target: f64) {
        self.target_lufs = target;
    }

    /// Analyzes a media file and returns its integrated LUFS value.
    ///
    /// Uses FFmpeg with the loudnorm filter in measurement mode.
    /// Only analyzes the first 180 seconds (3 minutes) of audio to balance
    /// accuracy with processing time.
    pub fn analyze_file<P: AsRef<Path>>(&self, path: P) -> Result<f64> {
        let path_str = path.as_ref().to_string_lossy();

        // Use FFmpeg with loudnorm filter in measurement mode
        // Analyze first 3 minutes for better accuracy while keeping processing reasonable
        // The filter outputs JSON statistics to stderr
        let output = Command::new("ffmpeg")
            .args([
                "-t",
                "180", // Limit to first 3 minutes
                "-i",
                &path_str,
                "-af",
                "loudnorm=I=-16:TP=-1.5:LRA=11:print_format=json",
                "-f",
                "null",
                "-",
            ])
            .output()
            .map_err(|e| Error::Io(format!("Failed to run FFmpeg: {}", e)))?;

        // FFmpeg outputs to stderr
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Parse the JSON output from loudnorm filter
        self.parse_loudnorm_output(&stderr)
    }

    /// Parses the loudnorm filter's JSON output to extract integrated LUFS.
    fn parse_loudnorm_output(&self, output: &str) -> Result<f64> {
        // Find the JSON block in the output
        // loudnorm outputs: {"input_i" : "-23.5", "input_tp" : "-1.0", ...}
        let json_start = output
            .find("{")
            .ok_or_else(|| Error::Io("No JSON output from loudnorm filter".to_string()))?;

        let json_end = output[json_start..]
            .find("}")
            .ok_or_else(|| Error::Io("Malformed JSON from loudnorm filter".to_string()))?;

        let json_str = &output[json_start..json_start + json_end + 1];

        // Parse input_i (integrated loudness)
        // Format: "input_i" : "-23.5"
        let lufs = self
            .extract_json_value(json_str, "input_i")
            .ok_or_else(|| Error::Io("Could not find input_i in loudnorm output".to_string()))?;

        // Validate the LUFS value
        if lufs < MIN_VALID_LUFS {
            return Err(Error::Io(format!(
                "Measured LUFS {} is below silence threshold",
                lufs
            )));
        }

        Ok(lufs)
    }

    /// Extracts a numeric value from a simple JSON object.
    fn extract_json_value(&self, json: &str, key: &str) -> Option<f64> {
        // Look for "key" : "value" or "key" : value
        let pattern = format!("\"{}\"", key);
        let key_pos = json.find(&pattern)?;

        // Find the colon after the key
        let after_key = &json[key_pos + pattern.len()..];
        let colon_pos = after_key.find(':')?;

        // Find the value (skip whitespace and optional quotes)
        let value_start = &after_key[colon_pos + 1..];
        let value_str = value_start.trim();

        // Handle quoted or unquoted values
        if let Some(inner) = value_str.strip_prefix('"') {
            // Quoted value: find closing quote
            let end = inner.find('"')?;
            inner[..end].parse().ok()
        } else {
            // Unquoted value: parse until comma, }, or whitespace
            let end = value_str
                .find(|c: char| c == ',' || c == '}' || c.is_whitespace())
                .unwrap_or(value_str.len());
            value_str[..end].parse().ok()
        }
    }

    /// Calculates the gain in dB needed to reach the target LUFS.
    ///
    /// # Arguments
    /// * `measured_lufs` - The measured integrated LUFS of the audio
    ///
    /// # Returns
    /// The gain in dB to apply, clamped to a safe range.
    pub fn calculate_gain(&self, measured_lufs: f64) -> f64 {
        let gain_db = self.target_lufs - measured_lufs;

        // Clamp to prevent excessive amplification (distortion)
        // Allow negative gain (attenuation) without limit
        gain_db.min(MAX_GAIN_DB)
    }

    /// Converts gain in dB to a linear multiplier.
    ///
    /// The formula is: linear = 10^(dB/20)
    pub fn db_to_linear(gain_db: f64) -> f64 {
        10.0_f64.powf(gain_db / 20.0)
    }

    /// Converts a linear multiplier to gain in dB.
    ///
    /// The formula is: dB = 20 * log10(linear)
    pub fn linear_to_db(linear: f64) -> f64 {
        if linear <= 0.0 {
            f64::NEG_INFINITY
        } else {
            20.0 * linear.log10()
        }
    }
}

/// Normalization settings for the application.
#[derive(Debug, Clone)]
pub struct NormalizationSettings {
    /// Whether normalization is enabled.
    pub enabled: bool,

    /// Target LUFS level.
    pub target_lufs: f64,
}

impl Default for NormalizationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            target_lufs: DEFAULT_TARGET_LUFS,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lufs_cache_stores_and_retrieves() {
        let cache = LufsCache::new();

        cache.insert("/path/to/video.mp4".to_string(), -18.5);

        assert_eq!(cache.get("/path/to/video.mp4"), Some(-18.5));
        assert_eq!(cache.get("/path/to/other.mp4"), None);
    }

    #[test]
    fn lufs_cache_clear_removes_all() {
        let cache = LufsCache::new();

        cache.insert("file1.mp4".to_string(), -20.0);
        cache.insert("file2.mp4".to_string(), -15.0);

        assert_eq!(cache.len(), 2);

        cache.clear();

        assert!(cache.is_empty());
        assert_eq!(cache.get("file1.mp4"), None);
    }

    #[test]
    fn lufs_analyzer_default_target() {
        let analyzer = LufsAnalyzer::default();
        assert!((analyzer.target_lufs() - DEFAULT_TARGET_LUFS).abs() < 0.001);
    }

    #[test]
    fn lufs_analyzer_custom_target() {
        let analyzer = LufsAnalyzer::new(-14.0);
        assert!((analyzer.target_lufs() - (-14.0)).abs() < 0.001);
    }

    #[test]
    fn calculate_gain_quiet_audio() {
        let analyzer = LufsAnalyzer::new(-16.0);

        // Audio at -23 LUFS needs +7 dB gain to reach -16 LUFS
        let gain = analyzer.calculate_gain(-23.0);
        assert!((gain - 7.0).abs() < 0.001);
    }

    #[test]
    fn calculate_gain_loud_audio() {
        let analyzer = LufsAnalyzer::new(-16.0);

        // Audio at -12 LUFS needs -4 dB (attenuation) to reach -16 LUFS
        let gain = analyzer.calculate_gain(-12.0);
        assert!((gain - (-4.0)).abs() < 0.001);
    }

    #[test]
    fn calculate_gain_clamped_to_max() {
        let analyzer = LufsAnalyzer::new(-16.0);

        // Audio at -40 LUFS would need +24 dB, but should be clamped
        let gain = analyzer.calculate_gain(-40.0);
        assert!((gain - MAX_GAIN_DB).abs() < 0.001);
    }

    #[test]
    fn db_to_linear_conversions() {
        // 0 dB = 1.0 linear
        assert!((LufsAnalyzer::db_to_linear(0.0) - 1.0).abs() < 0.001);

        // +6 dB ≈ 2.0 linear (doubled)
        assert!((LufsAnalyzer::db_to_linear(6.0) - 2.0).abs() < 0.01);

        // -6 dB ≈ 0.5 linear (halved)
        assert!((LufsAnalyzer::db_to_linear(-6.0) - 0.5).abs() < 0.01);

        // +20 dB = 10.0 linear
        assert!((LufsAnalyzer::db_to_linear(20.0) - 10.0).abs() < 0.01);
    }

    #[test]
    fn linear_to_db_conversions() {
        // 1.0 linear = 0 dB
        assert!((LufsAnalyzer::linear_to_db(1.0) - 0.0).abs() < 0.001);

        // 2.0 linear ≈ +6 dB
        assert!((LufsAnalyzer::linear_to_db(2.0) - 6.0).abs() < 0.1);

        // 0.5 linear ≈ -6 dB
        assert!((LufsAnalyzer::linear_to_db(0.5) - (-6.0)).abs() < 0.1);

        // 10.0 linear = +20 dB
        assert!((LufsAnalyzer::linear_to_db(10.0) - 20.0).abs() < 0.01);
    }

    #[test]
    fn linear_to_db_zero_returns_neg_infinity() {
        assert!(LufsAnalyzer::linear_to_db(0.0).is_infinite());
        assert!(LufsAnalyzer::linear_to_db(0.0).is_sign_negative());
    }

    #[test]
    fn db_linear_round_trip() {
        // Converting back and forth should give same value
        let original = -8.5;
        let linear = LufsAnalyzer::db_to_linear(original);
        let back = LufsAnalyzer::linear_to_db(linear);
        assert!((original - back).abs() < 0.001);
    }

    #[test]
    fn normalization_settings_default() {
        let settings = NormalizationSettings::default();
        assert!(settings.enabled);
        assert!((settings.target_lufs - DEFAULT_TARGET_LUFS).abs() < 0.001);
    }

    #[test]
    fn shared_cache_can_be_cloned() {
        let cache = create_lufs_cache();
        let cache2 = Arc::clone(&cache);

        cache.insert("file.mp4".to_string(), -20.0);

        assert_eq!(cache2.get("file.mp4"), Some(-20.0));
    }

    #[test]
    fn extract_json_value_quoted() {
        let analyzer = LufsAnalyzer::default();
        let json = r#"{"input_i" : "-23.5", "input_tp" : "-1.0"}"#;

        let value = analyzer.extract_json_value(json, "input_i");
        assert_eq!(value, Some(-23.5));
    }

    #[test]
    fn extract_json_value_unquoted() {
        let analyzer = LufsAnalyzer::default();
        let json = r#"{"input_i": -23.5, "input_tp": -1.0}"#;

        let value = analyzer.extract_json_value(json, "input_i");
        assert_eq!(value, Some(-23.5));
    }

    #[test]
    fn extract_json_value_missing_key() {
        let analyzer = LufsAnalyzer::default();
        let json = r#"{"other_key" : "-23.5"}"#;

        let value = analyzer.extract_json_value(json, "input_i");
        assert_eq!(value, None);
    }
}
