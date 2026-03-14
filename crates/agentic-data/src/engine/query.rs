//! Query engine — filter, sort, aggregate, and search records.
//!
//! Invention 13: Query Prophecy support — natural language queries
//! are translated to structured filters externally; this engine executes them.

use crate::types::*;
use super::store::DataStore;

/// Result of a query operation.
#[derive(Debug)]
pub struct QueryResult {
    pub records: Vec<DataRecord>,
    pub total_scanned: usize,
    pub total_matched: usize,
}

/// A filter condition for querying records.
#[derive(Debug, Clone)]
pub struct QueryFilter {
    pub field: String,
    pub op: FilterOp,
    pub value: serde_json::Value,
}

/// Filter operations.
#[derive(Debug, Clone, PartialEq)]
pub enum FilterOp {
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
    Contains,
    StartsWith,
    IsNull,
    IsNotNull,
}

/// Query engine operates on a DataStore.
pub struct QueryEngine<'a> {
    store: &'a DataStore,
}

impl<'a> QueryEngine<'a> {
    pub fn new(store: &'a DataStore) -> Self {
        Self { store }
    }

    /// Query records from a specific schema node with filters.
    pub fn query(
        &self,
        node: &str,
        filters: &[QueryFilter],
        limit: Option<usize>,
        offset: usize,
    ) -> QueryResult {
        let all = self.store.records_for_node(node);
        let total_scanned = all.len();

        let matched: Vec<DataRecord> = all.into_iter()
            .filter(|r| r.is_active() && filters.iter().all(|f| apply_filter(r, f)))
            .skip(offset)
            .take(limit.unwrap_or(usize::MAX))
            .cloned()
            .collect();

        let total_matched = matched.len();
        QueryResult { records: matched, total_scanned, total_matched }
    }

    /// Query all active records with filters (no node restriction).
    pub fn query_all(
        &self,
        filters: &[QueryFilter],
        limit: Option<usize>,
    ) -> QueryResult {
        let all = self.store.active_records();
        let total_scanned = all.len();

        let matched: Vec<DataRecord> = all.into_iter()
            .filter(|r| filters.iter().all(|f| apply_filter(r, f)))
            .take(limit.unwrap_or(usize::MAX))
            .cloned()
            .collect();

        let total_matched = matched.len();
        QueryResult { records: matched, total_scanned, total_matched }
    }

    /// Full-text search across all text fields.
    pub fn search(&self, query: &str, limit: Option<usize>) -> QueryResult {
        let query_lower = query.to_lowercase();
        let all = self.store.active_records();
        let total_scanned = all.len();

        let matched: Vec<DataRecord> = all.into_iter()
            .filter(|r| {
                r.fields.values().any(|v| {
                    if let Some(s) = v.as_str() {
                        s.to_lowercase().contains(&query_lower)
                    } else {
                        false
                    }
                })
            })
            .take(limit.unwrap_or(100))
            .cloned()
            .collect();

        let total_matched = matched.len();
        QueryResult { records: matched, total_scanned, total_matched }
    }

    /// Count records matching filters.
    pub fn count(&self, node: &str, filters: &[QueryFilter]) -> usize {
        self.store.records_for_node(node).into_iter()
            .filter(|r| r.is_active() && filters.iter().all(|f| apply_filter(r, f)))
            .count()
    }

    /// Get distinct values for a field.
    pub fn distinct(&self, node: &str, field: &str) -> Vec<serde_json::Value> {
        let mut seen = Vec::new();
        for r in self.store.records_for_node(node) {
            if let Some(val) = r.get(field) {
                if !seen.contains(val) {
                    seen.push(val.clone());
                }
            }
        }
        seen
    }
}

/// Apply a single filter to a record (public for transform engine).
pub fn apply_filter_pub(record: &DataRecord, filter: &QueryFilter) -> bool {
    apply_filter(record, filter)
}

/// Apply a single filter to a record.
fn apply_filter(record: &DataRecord, filter: &QueryFilter) -> bool {
    let val = record.get(&filter.field);
    match filter.op {
        FilterOp::IsNull => val.is_none() || val == Some(&serde_json::Value::Null),
        FilterOp::IsNotNull => val.is_some() && val != Some(&serde_json::Value::Null),
        FilterOp::Eq => val == Some(&filter.value),
        FilterOp::Ne => val != Some(&filter.value),
        FilterOp::Contains => {
            val.and_then(|v| v.as_str())
                .map(|s| s.to_lowercase().contains(&filter.value.as_str().unwrap_or("").to_lowercase()))
                .unwrap_or(false)
        }
        FilterOp::StartsWith => {
            val.and_then(|v| v.as_str())
                .map(|s| s.starts_with(filter.value.as_str().unwrap_or("")))
                .unwrap_or(false)
        }
        FilterOp::Gt => compare_values(val, &filter.value) == Some(std::cmp::Ordering::Greater),
        FilterOp::Gte => matches!(compare_values(val, &filter.value), Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)),
        FilterOp::Lt => compare_values(val, &filter.value) == Some(std::cmp::Ordering::Less),
        FilterOp::Lte => matches!(compare_values(val, &filter.value), Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal)),
    }
}

fn compare_values(a: Option<&serde_json::Value>, b: &serde_json::Value) -> Option<std::cmp::Ordering> {
    let a = a?;
    if let (Some(a_n), Some(b_n)) = (a.as_f64(), b.as_f64()) {
        a_n.partial_cmp(&b_n)
    } else if let (Some(a_s), Some(b_s)) = (a.as_str(), b.as_str()) {
        Some(a_s.cmp(b_s))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn test_store() -> DataStore {
        let mut store = DataStore::new();
        for (name, age) in [("Alice", 30), ("Bob", 25), ("Charlie", 35)] {
            let mut f = HashMap::new();
            f.insert("name".into(), serde_json::json!(name));
            f.insert("age".into(), serde_json::json!(age));
            store.add_record(DataRecord::new("s", "users", f));
        }
        store
    }

    #[test]
    fn test_query_all() {
        let store = test_store();
        let engine = QueryEngine::new(&store);
        let result = engine.query("users", &[], None, 0);
        assert_eq!(result.total_matched, 3);
    }

    #[test]
    fn test_query_eq() {
        let store = test_store();
        let engine = QueryEngine::new(&store);
        let result = engine.query("users", &[
            QueryFilter { field: "name".into(), op: FilterOp::Eq, value: serde_json::json!("Alice") },
        ], None, 0);
        assert_eq!(result.total_matched, 1);
    }

    #[test]
    fn test_query_gt() {
        let store = test_store();
        let engine = QueryEngine::new(&store);
        let result = engine.query("users", &[
            QueryFilter { field: "age".into(), op: FilterOp::Gt, value: serde_json::json!(28) },
        ], None, 0);
        assert_eq!(result.total_matched, 2); // Alice(30), Charlie(35)
    }

    #[test]
    fn test_search() {
        let store = test_store();
        let engine = QueryEngine::new(&store);
        let result = engine.search("bob", None);
        assert_eq!(result.total_matched, 1);
    }

    #[test]
    fn test_count() {
        let store = test_store();
        let engine = QueryEngine::new(&store);
        assert_eq!(engine.count("users", &[]), 3);
    }

    #[test]
    fn test_distinct() {
        let store = test_store();
        let engine = QueryEngine::new(&store);
        let names = engine.distinct("users", "name");
        assert_eq!(names.len(), 3);
    }

    #[test]
    fn test_limit_offset() {
        let store = test_store();
        let engine = QueryEngine::new(&store);
        let result = engine.query("users", &[], Some(2), 0);
        assert_eq!(result.total_matched, 2);
        let result2 = engine.query("users", &[], Some(2), 2);
        assert_eq!(result2.total_matched, 1);
    }

    #[test]
    fn test_contains_filter() {
        let store = test_store();
        let engine = QueryEngine::new(&store);
        let result = engine.query("users", &[
            QueryFilter { field: "name".into(), op: FilterOp::Contains, value: serde_json::json!("li") },
        ], None, 0);
        assert_eq!(result.total_matched, 2); // Alice, Charlie
    }
}
