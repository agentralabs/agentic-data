//! Data records — the fundamental unit of data in AgenticData.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique record identifier.
pub type RecordId = String;

/// Status of a record in the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordStatus {
    /// Normal active record.
    Active,
    /// Quarantined by quality checks.
    Quarantined,
    /// Soft-deleted / archived.
    Archived,
    /// Redacted (PII removed).
    Redacted,
}

/// A single data record with metadata and lineage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    /// Unique identifier.
    pub id: RecordId,
    /// Source this record came from.
    pub source_id: String,
    /// Schema node this record belongs to.
    pub schema_node: String,
    /// Field values (field name → JSON value).
    pub fields: HashMap<String, serde_json::Value>,
    /// Record status.
    pub status: RecordStatus,
    /// Version number (increments on update).
    pub version: u32,
    /// When this version was created (epoch micros).
    pub created_at: u64,
    /// When this version was last modified (epoch micros).
    pub updated_at: u64,
    /// Content checksum (BLAKE3).
    pub checksum: String,
    /// Compressed content size in bytes.
    pub compressed_size: u32,
    /// Uncompressed content size in bytes.
    pub raw_size: u32,
}

impl DataRecord {
    /// Create a new record with auto-generated ID and timestamps.
    pub fn new(source_id: &str, schema_node: &str, fields: HashMap<String, serde_json::Value>) -> Self {
        let now = super::now_micros();
        let id = uuid::Uuid::new_v4().to_string();
        let content = serde_json::to_vec(&fields).unwrap_or_default();
        let checksum = blake3::hash(&content).to_hex().to_string();
        let raw_size = content.len() as u32;

        Self {
            id,
            source_id: source_id.to_string(),
            schema_node: schema_node.to_string(),
            fields,
            status: RecordStatus::Active,
            version: 1,
            created_at: now,
            updated_at: now,
            checksum,
            compressed_size: 0,
            raw_size,
        }
    }

    /// Get a field value by name.
    pub fn get(&self, field: &str) -> Option<&serde_json::Value> {
        self.fields.get(field)
    }

    /// Get a field as string.
    pub fn get_str(&self, field: &str) -> Option<&str> {
        self.fields.get(field)?.as_str()
    }

    /// Get a field as i64.
    pub fn get_i64(&self, field: &str) -> Option<i64> {
        self.fields.get(field)?.as_i64()
    }

    /// Get a field as f64.
    pub fn get_f64(&self, field: &str) -> Option<f64> {
        self.fields.get(field)?.as_f64()
    }

    /// Check if this record is active.
    pub fn is_active(&self) -> bool {
        self.status == RecordStatus::Active
    }

    /// Field count.
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_record() {
        let mut fields = HashMap::new();
        fields.insert("name".into(), serde_json::json!("Alice"));
        fields.insert("age".into(), serde_json::json!(30));

        let rec = DataRecord::new("src-1", "users", fields);
        assert!(rec.is_active());
        assert_eq!(rec.field_count(), 2);
        assert_eq!(rec.get_str("name"), Some("Alice"));
        assert_eq!(rec.get_i64("age"), Some(30));
        assert!(!rec.checksum.is_empty());
        assert_eq!(rec.version, 1);
    }

    #[test]
    fn test_record_serialization() {
        let mut fields = HashMap::new();
        fields.insert("x".into(), serde_json::json!(42));
        let rec = DataRecord::new("s", "n", fields);
        let json = serde_json::to_string(&rec).unwrap();
        let back: DataRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_i64("x"), Some(42));
    }
}
