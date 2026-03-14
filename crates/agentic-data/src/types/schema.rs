//! Universal Schema — represents ANY data structure from any source.
//!
//! Invention 1: Schema Telepathy. One schema language that captures relational tables,
//! nested documents, key-value pairs, graphs, time series, and geospatial data.

use serde::{Deserialize, Serialize};

/// A field type in the universal schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldType {
    /// Text/string data.
    Text,
    /// Integer (i64).
    Integer,
    /// Floating point (f64).
    Float,
    /// Boolean.
    Boolean,
    /// Date/time (ISO 8601).
    DateTime,
    /// Date only (YYYY-MM-DD).
    Date,
    /// Time only (HH:MM:SS).
    Time,
    /// Duration/interval.
    Duration,
    /// Binary/blob data.
    Binary,
    /// UUID.
    Uuid,
    /// JSON/nested object.
    Object(Box<UniversalSchema>),
    /// Array of a specific type.
    Array(Box<FieldType>),
    /// Map/dictionary (key type → value type).
    Map(Box<FieldType>, Box<FieldType>),
    /// Geospatial point (lat, lng).
    GeoPoint,
    /// Geospatial polygon/region.
    GeoRegion,
    /// Enum with known values.
    Enum(Vec<String>),
    /// Currency amount (value + currency code).
    Currency,
    /// Email address.
    Email,
    /// URL.
    Url,
    /// IP address.
    IpAddress,
    /// Unknown/unresolved type.
    Unknown,
    /// Null/missing.
    Null,
}

/// A single field in a schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaField {
    /// Field name (column name, JSON key, etc).
    pub name: String,
    /// Detected or declared type.
    pub field_type: FieldType,
    /// Whether this field can be null/missing.
    pub nullable: bool,
    /// Human-readable description (auto-generated or provided).
    pub description: Option<String>,
    /// Confidence in type inference (0.0–1.0). 1.0 = declared, <1.0 = inferred.
    pub confidence: f64,
    /// Sample values seen during inference (up to 5).
    pub sample_values: Vec<String>,
    /// Statistical summary (for numeric fields).
    pub stats: Option<FieldStats>,
}

/// Statistical summary of a numeric field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldStats {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub null_count: u64,
    pub distinct_count: u64,
    pub total_count: u64,
}

/// A relationship edge between schema nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaEdge {
    /// Source field (fully qualified: "table.column" or "$.path.field").
    pub from: String,
    /// Target field.
    pub to: String,
    /// Relationship type.
    pub edge_type: SchemaEdgeType,
    /// Confidence in this relationship (0.0–1.0).
    pub confidence: f64,
}

/// Types of relationships between schema fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SchemaEdgeType {
    /// Foreign key / reference.
    References,
    /// Parent-child containment.
    Contains,
    /// Derived/computed from.
    DerivedFrom,
    /// Belongs to (inverse of Contains).
    BelongsTo,
    /// Semantically similar fields.
    SimilarTo,
}

/// A node in the schema graph (table, document, collection).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaNode {
    /// Node name (table name, collection name, file name).
    pub name: String,
    /// Source identifier.
    pub source: String,
    /// Fields in this node.
    pub fields: Vec<SchemaField>,
    /// Number of records/rows (if known).
    pub record_count: Option<u64>,
}

/// A complete universal schema covering one or more data sources.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UniversalSchema {
    /// Schema identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Schema nodes (tables, collections, document types).
    pub nodes: Vec<SchemaNode>,
    /// Relationships between fields across nodes.
    pub edges: Vec<SchemaEdge>,
    /// When this schema was inferred/updated.
    pub inferred_at: u64,
    /// Schema version (increments on evolution).
    pub version: u32,
}

impl UniversalSchema {
    /// Create a new empty schema.
    pub fn new(name: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            nodes: Vec::new(),
            edges: Vec::new(),
            inferred_at: super::now_micros(),
            version: 1,
        }
    }

    /// Total field count across all nodes.
    pub fn total_fields(&self) -> usize {
        self.nodes.iter().map(|n| n.fields.len()).sum()
    }

    /// Find a node by name.
    pub fn find_node(&self, name: &str) -> Option<&SchemaNode> {
        self.nodes.iter().find(|n| n.name == name)
    }

    /// Find a field by fully-qualified name ("node.field").
    pub fn find_field(&self, qualified: &str) -> Option<&SchemaField> {
        let parts: Vec<&str> = qualified.splitn(2, '.').collect();
        if parts.len() != 2 { return None; }
        let node = self.find_node(parts[0])?;
        node.fields.iter().find(|f| f.name == parts[1])
    }
}

impl SchemaField {
    /// Create a simple field with inferred type.
    pub fn inferred(name: &str, field_type: FieldType, confidence: f64) -> Self {
        Self {
            name: name.to_string(),
            field_type,
            nullable: false,
            description: None,
            confidence,
            sample_values: Vec::new(),
            stats: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_schema() {
        let s = UniversalSchema::new("test");
        assert_eq!(s.name, "test");
        assert_eq!(s.version, 1);
        assert!(s.nodes.is_empty());
    }

    #[test]
    fn test_total_fields() {
        let mut s = UniversalSchema::new("test");
        s.nodes.push(SchemaNode {
            name: "users".into(),
            source: "db".into(),
            fields: vec![
                SchemaField::inferred("id", FieldType::Integer, 1.0),
                SchemaField::inferred("name", FieldType::Text, 1.0),
            ],
            record_count: Some(100),
        });
        assert_eq!(s.total_fields(), 2);
    }

    #[test]
    fn test_find_field() {
        let mut s = UniversalSchema::new("test");
        s.nodes.push(SchemaNode {
            name: "orders".into(),
            source: "db".into(),
            fields: vec![SchemaField::inferred("total", FieldType::Float, 0.95)],
            record_count: None,
        });
        assert!(s.find_field("orders.total").is_some());
        assert!(s.find_field("orders.missing").is_none());
        assert!(s.find_field("missing.total").is_none());
    }

    #[test]
    fn test_nested_type() {
        let inner = UniversalSchema::new("address");
        let field_type = FieldType::Object(Box::new(inner));
        let f = SchemaField::inferred("address", field_type, 0.8);
        assert_eq!(f.confidence, 0.8);
    }

    #[test]
    fn test_schema_serialization() {
        let s = UniversalSchema::new("test");
        let json = serde_json::to_string(&s).unwrap();
        let back: UniversalSchema = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "test");
    }
}
