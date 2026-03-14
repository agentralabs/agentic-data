//! Geospatial format parsers — GeoJSON, KML, GPX.
//!
//! Invention 15: Geospatial Consciousness.

use std::collections::HashMap;
use crate::types::*;
use super::ParseResult;

/// Parse GeoJSON data.
pub fn parse_geojson(data: &str, source_name: &str) -> AdatResult<ParseResult> {
    let value: serde_json::Value = serde_json::from_str(data)
        .map_err(|e| AdatError::Parse(format!("Invalid GeoJSON: {}", e)))?;

    let features = if let Some(arr) = value.get("features").and_then(|f| f.as_array()) {
        arr.clone()
    } else if value.get("type").and_then(|t| t.as_str()) == Some("Feature") {
        vec![value]
    } else {
        return Err(AdatError::Parse("Expected FeatureCollection or Feature".into()));
    };

    let mut records = Vec::new();
    for feature in &features {
        let mut fields = HashMap::new();
        // Extract geometry
        if let Some(geom) = feature.get("geometry") {
            if let Some(coords) = geom.get("coordinates") {
                fields.insert("geometry_type".into(), geom.get("type").cloned().unwrap_or(serde_json::Value::Null));
                fields.insert("coordinates".into(), coords.clone());
                // Extract lat/lng for points
                if let Some(arr) = coords.as_array() {
                    if arr.len() >= 2 {
                        fields.insert("lng".into(), arr[0].clone());
                        fields.insert("lat".into(), arr[1].clone());
                    }
                }
            }
        }
        // Extract properties
        if let Some(props) = feature.get("properties").and_then(|p| p.as_object()) {
            for (k, v) in props {
                fields.insert(k.clone(), v.clone());
            }
        }
        records.push(DataRecord::new(source_name, "feature", fields));
    }

    let fields = vec![
        SchemaField::inferred("geometry_type", FieldType::Text, 0.95),
        SchemaField::inferred("coordinates", FieldType::GeoPoint, 0.90),
        SchemaField::inferred("lat", FieldType::Float, 0.90),
        SchemaField::inferred("lng", FieldType::Float, 0.90),
    ];

    let mut schema = UniversalSchema::new(source_name);
    schema.nodes.push(SchemaNode {
        name: "feature".into(), source: source_name.into(),
        fields, record_count: Some(records.len() as u64),
    });

    Ok(ParseResult { schema, records, errors: 0, warnings: Vec::new() })
}

/// Parse KML data (simplified).
pub fn parse_kml(data: &str, source_name: &str) -> AdatResult<ParseResult> {
    let mut records = Vec::new();
    let mut in_placemark = false;
    let mut fields: HashMap<String, serde_json::Value> = HashMap::new();

    for line in data.lines() {
        let trimmed = line.trim();
        if trimmed.contains("<Placemark") { in_placemark = true; fields.clear(); }
        if trimmed.contains("</Placemark>") && in_placemark {
            if !fields.is_empty() {
                records.push(DataRecord::new(source_name, "placemark", fields.clone()));
            }
            in_placemark = false;
        }
        if in_placemark {
            if let Some(val) = extract_kml_tag(trimmed, "name") {
                fields.insert("name".into(), serde_json::json!(val));
            }
            if let Some(val) = extract_kml_tag(trimmed, "description") {
                fields.insert("description".into(), serde_json::json!(val));
            }
            if let Some(val) = extract_kml_tag(trimmed, "coordinates") {
                fields.insert("coordinates".into(), serde_json::json!(val));
                if let Some((lng, lat)) = parse_kml_coords(&val) {
                    fields.insert("lat".into(), serde_json::json!(lat));
                    fields.insert("lng".into(), serde_json::json!(lng));
                }
            }
        }
    }

    let fields_schema = vec![
        SchemaField::inferred("name", FieldType::Text, 0.90),
        SchemaField::inferred("coordinates", FieldType::GeoPoint, 0.85),
        SchemaField::inferred("lat", FieldType::Float, 0.85),
        SchemaField::inferred("lng", FieldType::Float, 0.85),
    ];

    let mut schema = UniversalSchema::new(source_name);
    schema.nodes.push(SchemaNode {
        name: "placemark".into(), source: source_name.into(),
        fields: fields_schema, record_count: Some(records.len() as u64),
    });

    Ok(ParseResult { schema, records, errors: 0, warnings: Vec::new() })
}

/// Parse GPX data (simplified).
pub fn parse_gpx(data: &str, source_name: &str) -> AdatResult<ParseResult> {
    let mut records = Vec::new();

    for line in data.lines() {
        let trimmed = line.trim();
        // Track points: <trkpt lat="..." lon="...">
        if trimmed.contains("<trkpt") || trimmed.contains("<wpt") {
            let mut fields = HashMap::new();
            if let Some(lat) = extract_xml_attr(trimmed, "lat") {
                if let Ok(v) = lat.parse::<f64>() { fields.insert("lat".into(), serde_json::json!(v)); }
            }
            if let Some(lon) = extract_xml_attr(trimmed, "lon") {
                if let Ok(v) = lon.parse::<f64>() { fields.insert("lng".into(), serde_json::json!(v)); }
            }
            let point_type = if trimmed.contains("<wpt") { "waypoint" } else { "trackpoint" };
            fields.insert("type".into(), serde_json::json!(point_type));
            if !fields.is_empty() {
                records.push(DataRecord::new(source_name, "point", fields));
            }
        }
    }

    let fields = vec![
        SchemaField::inferred("lat", FieldType::Float, 0.95),
        SchemaField::inferred("lng", FieldType::Float, 0.95),
        SchemaField::inferred("type", FieldType::Text, 1.0),
    ];

    let mut schema = UniversalSchema::new(source_name);
    schema.nodes.push(SchemaNode {
        name: "point".into(), source: source_name.into(),
        fields, record_count: Some(records.len() as u64),
    });

    Ok(ParseResult { schema, records, errors: 0, warnings: Vec::new() })
}

fn extract_kml_tag(line: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = line.find(&open).map(|i| i + open.len())?;
    let end = line.find(&close)?;
    Some(line[start..end].to_string())
}

fn parse_kml_coords(coords: &str) -> Option<(f64, f64)> {
    let parts: Vec<&str> = coords.trim().split(',').collect();
    if parts.len() >= 2 {
        Some((parts[0].trim().parse().ok()?, parts[1].trim().parse().ok()?))
    } else { None }
}

fn extract_xml_attr(line: &str, attr: &str) -> Option<String> {
    let needle = format!("{}=\"", attr);
    let start = line.find(&needle).map(|i| i + needle.len())?;
    let end = line[start..].find('"').map(|i| start + i)?;
    Some(line[start..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_geojson() {
        let data = r#"{"type":"FeatureCollection","features":[{"type":"Feature","geometry":{"type":"Point","coordinates":[-74.006,40.7128]},"properties":{"name":"NYC"}}]}"#;
        let result = parse_geojson(data, "test").unwrap();
        assert_eq!(result.records.len(), 1);
        assert_eq!(result.records[0].get_str("name"), Some("NYC"));
    }

    #[test]
    fn test_parse_gpx() {
        let data = r#"<?xml version="1.0"?><gpx><trk><trkseg><trkpt lat="40.7128" lon="-74.006"></trkpt></trkseg></trk></gpx>"#;
        let result = parse_gpx(data, "test").unwrap();
        assert_eq!(result.records.len(), 1);
    }

    #[test]
    fn test_kml_coords() {
        assert_eq!(parse_kml_coords("-74.006,40.7128,0"), Some((-74.006, 40.7128)));
    }
}
