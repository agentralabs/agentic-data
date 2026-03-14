//! Ingestion engine — loads data from any source into the store.
//!
//! The entry point for all data. Detects format, parses, infers schema,
//! records lineage, and stores everything.

use crate::parser;
use crate::types::*;
use super::store::DataStore;

/// Result of an ingestion operation.
#[derive(Debug)]
pub struct IngestResult {
    pub source_id: String,
    pub schema_id: String,
    pub records_added: usize,
    pub parse_errors: usize,
    pub warnings: Vec<String>,
}

/// Engine for ingesting data from any source.
pub struct IngestEngine<'a> {
    store: &'a mut DataStore,
}

impl<'a> IngestEngine<'a> {
    pub fn new(store: &'a mut DataStore) -> Self {
        Self { store }
    }

    /// Ingest raw string data with auto-detection.
    pub fn ingest_string(&mut self, data: &str, name: &str) -> AdatResult<IngestResult> {
        let detection = parser::detect::detect_format(data, None);
        self.ingest_as(data, name, detection.format)
    }

    /// Ingest raw string data as a specific format.
    pub fn ingest_as(
        &mut self, data: &str, name: &str, format: parser::DataFormat,
    ) -> AdatResult<IngestResult> {
        let parse_result = parser::parse_as(data, name, format)?;

        // Register source
        let source = DataSource::new(name, SourceType::Memory, name);
        let source_id = source.id.clone();
        self.store.add_source(source);

        // Store schema
        let schema_id = parse_result.schema.id.clone();
        self.store.add_schema(parse_result.schema);

        // Store records + lineage
        let records_added = parse_result.records.len();
        for record in &parse_result.records {
            self.store.add_lineage(&record.id, LineageEntry {
                action: LineageAction::Ingested,
                source: name.to_string(),
                timestamp: now_micros(),
                input_hash: String::new(),
                output_hash: record.checksum.clone(),
                description: format!("Ingested from {} as {:?}", name, format),
            });
        }
        self.store.add_records(parse_result.records);

        Ok(IngestResult {
            source_id,
            schema_id,
            records_added,
            parse_errors: parse_result.errors,
            warnings: parse_result.warnings,
        })
    }

    /// Ingest from a file path.
    pub fn ingest_file(&mut self, path: &str) -> AdatResult<IngestResult> {
        let data = std::fs::read_to_string(path)
            .map_err(|e| AdatError::Io(e))?;
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str());
        let detection = parser::detect::detect_format(&data, ext);
        let name = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path);
        self.ingest_as(&data, name, detection.format)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ingest_csv() {
        let mut store = DataStore::new();
        let mut engine = IngestEngine::new(&mut store);
        let result = engine.ingest_string(
            "name,age\nAlice,30\nBob,25\n", "test.csv",
        ).unwrap();
        assert_eq!(result.records_added, 2);
        assert_eq!(result.parse_errors, 0);
        assert_eq!(store.record_count(), 2);
        assert_eq!(store.schema_count(), 1);
        assert_eq!(store.source_count(), 1);
    }

    #[test]
    fn test_ingest_json() {
        let mut store = DataStore::new();
        let mut engine = IngestEngine::new(&mut store);
        let result = engine.ingest_string(
            r#"[{"x":1},{"x":2}]"#, "test.json",
        ).unwrap();
        assert_eq!(result.records_added, 2);
    }

    #[test]
    fn test_ingest_records_lineage() {
        let mut store = DataStore::new();
        let mut engine = IngestEngine::new(&mut store);
        engine.ingest_string("a,b\n1,2\n", "test").unwrap();
        // Every record should have lineage
        let rec = &store.active_records()[0];
        let lineage = store.get_lineage(&rec.id);
        assert!(lineage.is_some());
        assert_eq!(lineage.unwrap().depth(), 1);
    }

    #[test]
    fn test_ingest_auto_detect() {
        let mut store = DataStore::new();
        let mut engine = IngestEngine::new(&mut store);
        // Should auto-detect as JSON
        let result = engine.ingest_string(r#"{"key":"val"}"#, "data").unwrap();
        assert_eq!(result.records_added, 1);
    }
}
