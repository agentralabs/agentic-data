//! Spatial index — R-tree-style bounding box queries for geospatial data.
//!
//! Invention 15: Geospatial Consciousness (indexing layer).
//! Simple grid-based spatial index (production would use an R-tree).

use crate::types::*;

/// Grid-based spatial index for geographic queries.
#[derive(Debug, Default)]
pub struct SpatialIndex {
    /// Grid cell (lat_bucket, lng_bucket) → list of (record_id, point).
    cells: std::collections::HashMap<(i32, i32), Vec<(String, GeoPoint)>>,
    /// Total indexed points.
    count: usize,
}

const GRID_SIZE: f64 = 1.0; // 1 degree per cell (~111km at equator)

impl SpatialIndex {
    pub fn new() -> Self { Self::default() }

    /// Index a record with a geographic point.
    pub fn add(&mut self, record_id: &str, point: GeoPoint) {
        let cell = to_cell(&point);
        self.cells.entry(cell).or_default().push((record_id.to_string(), point));
        self.count += 1;
    }

    /// Find all records within a bounding box.
    pub fn within_bounds(&self, bounds: &GeoBounds) -> Vec<(&str, GeoPoint)> {
        let min_cell = to_cell(&GeoPoint::new(bounds.south, bounds.west));
        let max_cell = to_cell(&GeoPoint::new(bounds.north, bounds.east));
        let mut results = Vec::new();

        for lat in min_cell.0..=max_cell.0 {
            for lng in min_cell.1..=max_cell.1 {
                if let Some(entries) = self.cells.get(&(lat, lng)) {
                    for (id, pt) in entries {
                        if pt.within(bounds) {
                            results.push((id.as_str(), *pt));
                        }
                    }
                }
            }
        }
        results
    }

    /// Find records within distance_meters of a point.
    pub fn within_radius(&self, center: &GeoPoint, radius_meters: f64) -> Vec<(&str, GeoPoint, f64)> {
        // Search cells within the radius
        let deg_radius = radius_meters / 111_000.0; // Approximate
        let bounds = GeoBounds::new(
            center.lat - deg_radius, center.lng - deg_radius,
            center.lat + deg_radius, center.lng + deg_radius,
        );
        self.within_bounds(&bounds).into_iter()
            .map(|(id, pt)| {
                let dist = center.distance_meters(&pt);
                (id, pt, dist)
            })
            .filter(|(_, _, dist)| *dist <= radius_meters)
            .collect()
    }

    /// Find the N nearest records to a point.
    pub fn nearest(&self, center: &GeoPoint, n: usize) -> Vec<(&str, GeoPoint, f64)> {
        // Search expanding radius until we have enough
        let mut radius = 10_000.0; // Start with 10km
        loop {
            let mut results = self.within_radius(center, radius);
            if results.len() >= n || radius > 1_000_000.0 {
                results.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
                results.truncate(n);
                return results;
            }
            radius *= 2.0;
        }
    }

    /// Total indexed points.
    pub fn len(&self) -> usize { self.count }
    pub fn is_empty(&self) -> bool { self.count == 0 }

    /// Number of grid cells used.
    pub fn cell_count(&self) -> usize { self.cells.len() }
}

fn to_cell(point: &GeoPoint) -> (i32, i32) {
    ((point.lat / GRID_SIZE).floor() as i32, (point.lng / GRID_SIZE).floor() as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_bounds_query() {
        let mut idx = SpatialIndex::new();
        idx.add("nyc", GeoPoint::new(40.7128, -74.0060));
        idx.add("la", GeoPoint::new(34.0522, -118.2437));
        idx.add("chi", GeoPoint::new(41.8781, -87.6298));

        // Query east coast
        let bounds = GeoBounds::new(35.0, -80.0, 45.0, -70.0);
        let results = idx.within_bounds(&bounds);
        assert_eq!(results.len(), 1); // Only NYC
        assert_eq!(results[0].0, "nyc");
    }

    #[test]
    fn test_radius_query() {
        let mut idx = SpatialIndex::new();
        idx.add("a", GeoPoint::new(40.7128, -74.0060)); // NYC
        idx.add("b", GeoPoint::new(40.7580, -73.9855)); // Midtown (5km away)
        idx.add("c", GeoPoint::new(34.0522, -118.2437)); // LA (far)

        let results = idx.within_radius(&GeoPoint::new(40.7128, -74.0060), 10_000.0);
        assert_eq!(results.len(), 2); // NYC + Midtown, not LA
    }

    #[test]
    fn test_nearest() {
        let mut idx = SpatialIndex::new();
        idx.add("close", GeoPoint::new(40.713, -74.006));
        idx.add("medium", GeoPoint::new(40.750, -73.990));
        idx.add("far", GeoPoint::new(41.000, -73.800));

        let nearest = idx.nearest(&GeoPoint::new(40.712, -74.005), 2);
        assert_eq!(nearest.len(), 2);
        assert_eq!(nearest[0].0, "close"); // Closest first
    }

    #[test]
    fn test_empty_index() {
        let idx = SpatialIndex::new();
        assert!(idx.is_empty());
        assert!(idx.within_bounds(&GeoBounds::new(0.0, 0.0, 1.0, 1.0)).is_empty());
    }

    #[test]
    fn test_cell_count() {
        let mut idx = SpatialIndex::new();
        idx.add("a", GeoPoint::new(40.7, -74.0));
        idx.add("b", GeoPoint::new(40.8, -74.0)); // Same cell
        idx.add("c", GeoPoint::new(34.0, -118.0)); // Different cell
        assert_eq!(idx.len(), 3);
        assert_eq!(idx.cell_count(), 2);
    }
}
