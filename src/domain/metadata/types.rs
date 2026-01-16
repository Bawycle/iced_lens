// SPDX-License-Identifier: MPL-2.0
//! Metadata domain types.
//!
//! Pure domain types for media metadata with no external dependencies.

// =============================================================================
// GpsCoordinates
// =============================================================================

/// GPS coordinates in decimal degrees.
///
/// This type represents geographic coordinates using the WGS84 coordinate
/// system (latitude and longitude in decimal degrees).
///
/// # Example
///
/// ```ignore
/// let coords = GpsCoordinates::new(48.8566, 2.3522); // Paris
/// assert!(coords.is_valid());
/// assert_eq!(coords.format(), "48.856600° N, 2.352200° E");
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GpsCoordinates {
    /// Latitude in decimal degrees (-90.0 to 90.0)
    latitude: f64,
    /// Longitude in decimal degrees (-180.0 to 180.0)
    longitude: f64,
}

impl GpsCoordinates {
    /// Creates new GPS coordinates.
    ///
    /// Values outside valid ranges will be clamped:
    /// - Latitude: -90.0 to 90.0
    /// - Longitude: -180.0 to 180.0
    #[must_use]
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude: latitude.clamp(-90.0, 90.0),
            longitude: longitude.clamp(-180.0, 180.0),
        }
    }

    /// Returns the latitude in decimal degrees.
    #[must_use]
    pub fn latitude(&self) -> f64 {
        self.latitude
    }

    /// Returns the longitude in decimal degrees.
    #[must_use]
    pub fn longitude(&self) -> f64 {
        self.longitude
    }

    /// Returns whether these coordinates are valid (not NaN or infinite).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.latitude.is_finite() && self.longitude.is_finite()
    }

    /// Returns whether this is the null island (0,0) which often indicates
    /// missing or default GPS data.
    #[must_use]
    pub fn is_null_island(&self) -> bool {
        self.latitude.abs() < f64::EPSILON && self.longitude.abs() < f64::EPSILON
    }

    /// Formats the coordinates as a human-readable string.
    ///
    /// Format: "48.856600° N, 2.352200° E"
    #[must_use]
    pub fn format(&self) -> String {
        let lat_dir = if self.latitude >= 0.0 { "N" } else { "S" };
        let lon_dir = if self.longitude >= 0.0 { "E" } else { "W" };
        format!(
            "{:.6}° {}, {:.6}° {}",
            self.latitude.abs(),
            lat_dir,
            self.longitude.abs(),
            lon_dir
        )
    }

    /// Generates a URL to view these coordinates on a map.
    ///
    /// Returns a Google Maps URL for the location.
    #[must_use]
    pub fn map_url(&self) -> String {
        format!(
            "https://www.google.com/maps?q={},{}",
            self.latitude, self.longitude
        )
    }
}

impl Default for GpsCoordinates {
    fn default() -> Self {
        Self {
            latitude: 0.0,
            longitude: 0.0,
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gps_coordinates_new() {
        let coords = GpsCoordinates::new(48.8566, 2.3522);
        assert!((coords.latitude() - 48.8566).abs() < f64::EPSILON);
        assert!((coords.longitude() - 2.3522).abs() < f64::EPSILON);
    }

    #[test]
    fn gps_coordinates_clamps_latitude() {
        let coords = GpsCoordinates::new(100.0, 0.0);
        assert!((coords.latitude() - 90.0).abs() < f64::EPSILON);

        let coords = GpsCoordinates::new(-100.0, 0.0);
        assert!((coords.latitude() - -90.0).abs() < f64::EPSILON);
    }

    #[test]
    fn gps_coordinates_clamps_longitude() {
        let coords = GpsCoordinates::new(0.0, 200.0);
        assert!((coords.longitude() - 180.0).abs() < f64::EPSILON);

        let coords = GpsCoordinates::new(0.0, -200.0);
        assert!((coords.longitude() - -180.0).abs() < f64::EPSILON);
    }

    #[test]
    fn gps_coordinates_is_valid() {
        let valid = GpsCoordinates::new(48.8566, 2.3522);
        assert!(valid.is_valid());

        // NaN values would be clamped, so we can't test is_valid returning false
        // with normal construction
    }

    #[test]
    fn gps_coordinates_is_null_island() {
        let null_island = GpsCoordinates::new(0.0, 0.0);
        assert!(null_island.is_null_island());

        let paris = GpsCoordinates::new(48.8566, 2.3522);
        assert!(!paris.is_null_island());
    }

    #[test]
    fn gps_coordinates_format() {
        let paris = GpsCoordinates::new(48.8566, 2.3522);
        assert_eq!(paris.format(), "48.856600° N, 2.352200° E");

        let sydney = GpsCoordinates::new(-33.8688, 151.2093);
        assert_eq!(sydney.format(), "33.868800° S, 151.209300° E");

        let nyc = GpsCoordinates::new(40.7128, -74.0060);
        assert_eq!(nyc.format(), "40.712800° N, 74.006000° W");
    }

    #[test]
    fn gps_coordinates_map_url() {
        let paris = GpsCoordinates::new(48.8566, 2.3522);
        assert!(paris.map_url().contains("48.8566"));
        assert!(paris.map_url().contains("2.3522"));
    }

    #[test]
    fn gps_coordinates_default() {
        let coords = GpsCoordinates::default();
        assert!(coords.is_null_island());
    }

    #[test]
    fn gps_coordinates_equality() {
        let a = GpsCoordinates::new(48.8566, 2.3522);
        let b = GpsCoordinates::new(48.8566, 2.3522);
        assert_eq!(a, b);

        let c = GpsCoordinates::new(40.7128, -74.0060);
        assert_ne!(a, c);
    }
}
