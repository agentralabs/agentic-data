//! Geospatial types — coordinates, regions, spatial references.
//!
//! Invention 15: Geospatial Consciousness.

use serde::{Deserialize, Serialize};

/// A geographic point (WGS84).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GeoPoint {
    pub lat: f64,
    pub lng: f64,
}

impl GeoPoint {
    pub fn new(lat: f64, lng: f64) -> Self {
        Self { lat, lng }
    }

    /// Haversine distance in meters between two points.
    pub fn distance_meters(&self, other: &GeoPoint) -> f64 {
        let r = 6_371_000.0; // Earth radius in meters
        let d_lat = (other.lat - self.lat).to_radians();
        let d_lng = (other.lng - self.lng).to_radians();
        let a = (d_lat / 2.0).sin().powi(2)
            + self.lat.to_radians().cos() * other.lat.to_radians().cos()
            * (d_lng / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().asin();
        r * c
    }

    /// Check if this point is within a bounding box.
    pub fn within(&self, bounds: &GeoBounds) -> bool {
        self.lat >= bounds.south && self.lat <= bounds.north
            && self.lng >= bounds.west && self.lng <= bounds.east
    }
}

/// A geographic bounding box.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GeoBounds {
    pub north: f64,
    pub south: f64,
    pub east: f64,
    pub west: f64,
}

impl GeoBounds {
    pub fn new(south: f64, west: f64, north: f64, east: f64) -> Self {
        Self { north, south, east, west }
    }

    /// Check if a point is inside these bounds.
    pub fn contains(&self, point: &GeoPoint) -> bool {
        point.within(self)
    }

    /// Center point of the bounding box.
    pub fn center(&self) -> GeoPoint {
        GeoPoint::new((self.north + self.south) / 2.0, (self.east + self.west) / 2.0)
    }
}

/// Spatial reference system identifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialRef {
    /// EPSG code (4326 = WGS84).
    pub epsg: u32,
    /// Human-readable name.
    pub name: String,
}

impl Default for SpatialRef {
    fn default() -> Self {
        Self { epsg: 4326, name: "WGS 84".into() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance() {
        let nyc = GeoPoint::new(40.7128, -74.0060);
        let la = GeoPoint::new(34.0522, -118.2437);
        let dist = nyc.distance_meters(&la);
        // NYC to LA is roughly 3,944 km
        assert!(dist > 3_900_000.0 && dist < 4_000_000.0);
    }

    #[test]
    fn test_within_bounds() {
        let point = GeoPoint::new(40.7, -74.0);
        let nyc_area = GeoBounds::new(40.0, -75.0, 41.0, -73.0);
        assert!(point.within(&nyc_area));
        assert!(nyc_area.contains(&point));
    }

    #[test]
    fn test_outside_bounds() {
        let point = GeoPoint::new(34.0, -118.0); // LA
        let nyc_area = GeoBounds::new(40.0, -75.0, 41.0, -73.0);
        assert!(!point.within(&nyc_area));
    }

    #[test]
    fn test_center() {
        let bounds = GeoBounds::new(40.0, -75.0, 42.0, -73.0);
        let c = bounds.center();
        assert!((c.lat - 41.0).abs() < 0.001);
        assert!((c.lng - (-74.0)).abs() < 0.001);
    }
}
