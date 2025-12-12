// SPDX-License-Identifier: MPL-2.0
//! Time unit conversion utilities for video playback.
//!
//! Provides conversion functions between seconds and microseconds for:
//! - Frame cache indexing (i64 for PTS keys)
//! - Internal timing calculations
//!
//! # Constants
//!
//! - `MICROS_PER_SECOND`: 1,000,000 (f64 for calculations)

/// Microseconds per second as f64 for calculations.
pub const MICROS_PER_SECOND: f64 = 1_000_000.0;

/// Converts seconds to microseconds (f64 for slider precision).
///
/// # Examples
///
/// ```
/// use iced_lens::video_player::time_units::secs_to_micros;
///
/// assert_eq!(secs_to_micros(1.0), 1_000_000.0);
/// assert_eq!(secs_to_micros(0.5), 500_000.0);
/// ```
#[inline]
pub fn secs_to_micros(secs: f64) -> f64 {
    secs * MICROS_PER_SECOND
}

/// Converts microseconds to seconds (f64).
///
/// # Examples
///
/// ```
/// use iced_lens::video_player::time_units::micros_to_secs;
///
/// assert_eq!(micros_to_secs(1_000_000.0), 1.0);
/// assert_eq!(micros_to_secs(500_000.0), 0.5);
/// ```
#[inline]
pub fn micros_to_secs(micros: f64) -> f64 {
    micros / MICROS_PER_SECOND
}

/// Converts PTS seconds to microseconds (i64 for cache indexing).
///
/// Used by frame cache for precise PTS-keyed lookups.
///
/// # Examples
///
/// ```
/// use iced_lens::video_player::time_units::pts_to_micros;
///
/// assert_eq!(pts_to_micros(1.0), 1_000_000);
/// assert_eq!(pts_to_micros(0.5), 500_000);
/// ```
#[inline]
pub fn pts_to_micros(pts_secs: f64) -> i64 {
    (pts_secs * MICROS_PER_SECOND) as i64
}

/// Converts microseconds to PTS seconds (f64).
///
/// Used by frame cache for converting stored keys back to presentation time.
///
/// # Examples
///
/// ```
/// use iced_lens::video_player::time_units::micros_to_pts;
///
/// assert_eq!(micros_to_pts(1_000_000), 1.0);
/// assert_eq!(micros_to_pts(500_000), 0.5);
/// ```
#[inline]
pub fn micros_to_pts(micros: i64) -> f64 {
    micros as f64 / MICROS_PER_SECOND
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secs_to_micros_converts_correctly() {
        assert_eq!(secs_to_micros(1.0), 1_000_000.0);
        assert_eq!(secs_to_micros(0.5), 500_000.0);
        assert_eq!(secs_to_micros(0.0), 0.0);
        assert_eq!(secs_to_micros(2.5), 2_500_000.0);
    }

    #[test]
    fn micros_to_secs_converts_correctly() {
        assert_eq!(micros_to_secs(1_000_000.0), 1.0);
        assert_eq!(micros_to_secs(500_000.0), 0.5);
        assert_eq!(micros_to_secs(0.0), 0.0);
        assert_eq!(micros_to_secs(2_500_000.0), 2.5);
    }

    #[test]
    fn f64_round_trip_preserves_value() {
        let original = 123.456789;
        let result = micros_to_secs(secs_to_micros(original));
        assert!((original - result).abs() < 1e-10);
    }

    #[test]
    fn pts_to_micros_converts_correctly() {
        assert_eq!(pts_to_micros(1.0), 1_000_000);
        assert_eq!(pts_to_micros(0.5), 500_000);
        assert_eq!(pts_to_micros(0.0), 0);
        assert_eq!(pts_to_micros(1.234567), 1_234_567);
    }

    #[test]
    fn micros_to_pts_converts_correctly() {
        assert_eq!(micros_to_pts(1_000_000), 1.0);
        assert_eq!(micros_to_pts(500_000), 0.5);
        assert_eq!(micros_to_pts(0), 0.0);
    }

    #[test]
    fn i64_round_trip_preserves_value_within_microsecond() {
        let pts_secs = 1.234567;
        let micros = pts_to_micros(pts_secs);
        let back = micros_to_pts(micros);
        // Should be accurate to microsecond precision
        assert!((pts_secs - back).abs() < 0.000001);
    }

    #[test]
    fn micros_per_second_constant() {
        assert_eq!(MICROS_PER_SECOND, 1_000_000.0);
    }

    #[test]
    fn handles_large_durations() {
        // 24 hours in seconds
        let day_secs = 24.0 * 60.0 * 60.0;
        let day_micros = secs_to_micros(day_secs);
        assert_eq!(day_micros, 86_400_000_000.0);
        assert_eq!(micros_to_secs(day_micros), day_secs);
    }

    #[test]
    fn handles_sub_millisecond_precision() {
        // 0.0001 seconds = 100 microseconds
        let sub_ms = 0.0001;
        let micros = secs_to_micros(sub_ms);
        assert_eq!(micros, 100.0);
        assert_eq!(micros_to_secs(micros), sub_ms);
    }
}
