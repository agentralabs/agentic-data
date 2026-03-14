//! In-memory data store — holds schemas, sources, and records.
//!
//! Central state that all engine components operate on.

use std::collections::HashMap;
use crate::types::*;

/// In-memory store for all AgenticData state.
#[derive(Debug, Default)]
pub struct DataStore {
    schemas: HashMap<String, UniversalSchema>,
    sources: HashMap<String, DataSource>,
    records: Vec<DataRecord>,
    lineage: HashMap<String, LineageChain>,
}

impl DataStore {
    pub fn new() -> Self { Self::default() }

    // ── Schema operations ──

    pub fn add_schema(&mut self, schema: UniversalSchema) {
        self.schemas.insert(schema.id.clone(), schema);
    }

    pub fn get_schema(&self, id: &str) -> Option<&UniversalSchema> {
        self.schemas.get(id)
    }

    pub fn find_schema_by_name(&self, name: &str) -> Option<&UniversalSchema> {
        self.schemas.values().find(|s| s.name == name)
    }

    pub fn schema_count(&self) -> usize { self.schemas.len() }
    pub fn all_schemas(&self) -> Vec<&UniversalSchema> { self.schemas.values().collect() }

    // ── Source operations ──

    pub fn add_source(&mut self, source: DataSource) {
        self.sources.insert(source.id.clone(), source);
    }

    pub fn get_source(&self, id: &str) -> Option<&DataSource> {
        self.sources.get(id)
    }

    pub fn source_count(&self) -> usize { self.sources.len() }
    pub fn all_sources(&self) -> Vec<&DataSource> { self.sources.values().collect() }

    // ── Record operations ──

    pub fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }

    pub fn add_records(&mut self, records: Vec<DataRecord>) {
        self.records.extend(records);
    }

    pub fn get_record(&self, id: &str) -> Option<&DataRecord> {
        self.records.iter().find(|r| r.id == id)
    }

    pub fn records_for_source(&self, source_id: &str) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.source_id == source_id).collect()
    }

    pub fn records_for_node(&self, node: &str) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.schema_node == node).collect()
    }

    pub fn record_count(&self) -> usize { self.records.len() }

    pub fn active_records(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.is_active()).collect()
    }

    /// Remove records matching a predicate. Returns count removed.
    pub fn remove_records<F: Fn(&DataRecord) -> bool>(&mut self, pred: F) -> usize {
        let before = self.records.len();
        self.records.retain(|r| !pred(r));
        before - self.records.len()
    }

    // ── Lineage operations ──

    pub fn add_lineage(&mut self, target: &str, entry: LineageEntry) {
        self.lineage.entry(target.to_string())
            .or_insert_with(|| LineageChain::new(target))
            .add(entry);
    }

    pub fn get_lineage(&self, target: &str) -> Option<&LineageChain> {
        self.lineage.get(target)
    }

    // ── Bulk operations ──

    /// Summary string for diagnostics.
    pub fn summary(&self) -> String {
        format!("{} schemas, {} sources, {} records, {} lineage chains",
            self.schemas.len(), self.sources.len(), self.records.len(), self.lineage.len())
    }

    /// Clear all data.
    pub fn clear(&mut self) {
        self.schemas.clear();
        self.sources.clear();
        self.records.clear();
        self.lineage.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_store_basics() {
        let mut store = DataStore::new();
        assert_eq!(store.record_count(), 0);

        let schema = UniversalSchema::new("test");
        let schema_id = schema.id.clone();
        store.add_schema(schema);
        assert_eq!(store.schema_count(), 1);
        assert!(store.get_schema(&schema_id).is_some());

        let source = DataSource::file("csv", "/tmp/test.csv");
        let source_id = source.id.clone();
        store.add_source(source);
        assert_eq!(store.source_count(), 1);

        let mut fields = HashMap::new();
        fields.insert("x".into(), serde_json::json!(1));
        store.add_record(DataRecord::new(&source_id, "test", fields));
        assert_eq!(store.record_count(), 1);
        assert_eq!(store.records_for_source(&source_id).len(), 1);
    }

    #[test]
    fn test_store_lineage() {
        let mut store = DataStore::new();
        store.add_lineage("users.email", LineageEntry {
            action: LineageAction::Ingested,
            source: "csv".into(), timestamp: 1000,
            input_hash: "a".into(), output_hash: "b".into(),
            description: "Imported".into(),
        });
        let chain = store.get_lineage("users.email").unwrap();
        assert_eq!(chain.depth(), 1);
    }

    #[test]
    fn test_store_clear() {
        let mut store = DataStore::new();
        store.add_schema(UniversalSchema::new("s"));
        store.clear();
        assert_eq!(store.schema_count(), 0);
    }

    #[test]
    fn test_summary() {
        let store = DataStore::new();
        assert!(store.summary().contains("0 schemas"));
    }
}
