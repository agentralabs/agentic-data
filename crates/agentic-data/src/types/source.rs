//! Data source types — registered origins of data.

use serde::{Deserialize, Serialize};

/// Unique source identifier.
pub type SourceId = String;

/// Type of data source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceType {
    /// Local file (CSV, JSON, Excel, etc).
    File,
    /// SQLite database.
    Sqlite,
    /// PostgreSQL database.
    Postgres,
    /// MySQL database.
    Mysql,
    /// REST API endpoint.
    RestApi,
    /// In-memory data.
    Memory,
    /// Another AgenticData instance (federation).
    Federation,
    /// Stream/real-time source.
    Stream,
    /// Unknown/custom source.
    Custom(String),
}

/// A registered data source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    /// Unique identifier.
    pub id: SourceId,
    /// Human-readable name.
    pub name: String,
    /// Source type.
    pub source_type: SourceType,
    /// Connection string or file path (encrypted at rest).
    pub connection: String,
    /// Associated schema ID.
    pub schema_id: Option<String>,
    /// When this source was registered.
    pub registered_at: u64,
    /// When data was last fetched from this source.
    pub last_fetched: Option<u64>,
    /// Whether this source is currently reachable.
    pub healthy: bool,
    /// Record count from this source.
    pub record_count: u64,
}

impl DataSource {
    /// Create a new source registration.
    pub fn new(name: &str, source_type: SourceType, connection: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            source_type,
            connection: connection.to_string(),
            schema_id: None,
            registered_at: super::now_micros(),
            last_fetched: None,
            healthy: true,
            record_count: 0,
        }
    }

    /// Create a file source.
    pub fn file(name: &str, path: &str) -> Self {
        Self::new(name, SourceType::File, path)
    }

    /// Create a SQLite source.
    pub fn sqlite(name: &str, path: &str) -> Self {
        Self::new(name, SourceType::Sqlite, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_source() {
        let s = DataSource::file("my_csv", "/data/test.csv");
        assert_eq!(s.source_type, SourceType::File);
        assert!(s.healthy);
        assert_eq!(s.record_count, 0);
    }

    #[test]
    fn test_source_serialization() {
        let s = DataSource::sqlite("db", "/tmp/test.db");
        let json = serde_json::to_string(&s).unwrap();
        let back: DataSource = serde_json::from_str(&json).unwrap();
        assert_eq!(back.source_type, SourceType::Sqlite);
    }
}
