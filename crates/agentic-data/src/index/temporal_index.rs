//! Temporal index — fast time-range queries and version history.
//!
//! Invention 9: Temporal Data Archaeology.

use std::collections::BTreeMap;
use crate::types::*;

/// Temporal index for time-based record lookups.
#[derive(Debug, Default)]
pub struct TemporalIndex {
    /// Timestamp → list of record IDs created/updated at that time.
    by_time: BTreeMap<u64, Vec<String>>,
    /// Record ID → list of (timestamp, version) pairs.
    versions: std::collections::HashMap<String, Vec<(u64, u32)>>,
}

impl TemporalIndex {
    pub fn new() -> Self { Self::default() }

    /// Index a record by its timestamps.
    pub fn add(&mut self, record: &DataRecord) {
        self.by_time.entry(record.created_at)
            .or_default()
            .push(record.id.clone());
        self.versions.entry(record.id.clone())
            .or_default()
            .push((record.updated_at, record.version));
    }

    /// Query records within a time range (inclusive).
    pub fn range(&self, from: u64, to: u64) -> Vec<&str> {
        self.by_time.range(from..=to)
            .flat_map(|(_, ids)| ids.iter().map(|s| s.as_str()))
            .collect()
    }

    /// Get version history for a record.
    pub fn history(&self, record_id: &str) -> Vec<(u64, u32)> {
        self.versions.get(record_id).cloned().unwrap_or_default()
    }

    /// Get the latest N record IDs.
    pub fn latest(&self, n: usize) -> Vec<&str> {
        self.by_time.iter().rev()
            .flat_map(|(_, ids)| ids.iter().map(|s| s.as_str()))
            .take(n)
            .collect()
    }

    /// Count of records in a time range.
    pub fn count_range(&self, from: u64, to: u64) -> usize {
        self.by_time.range(from..=to)
            .map(|(_, ids)| ids.len())
            .sum()
    }

    /// Total indexed entries.
    pub fn len(&self) -> usize {
        self.by_time.values().map(|v| v.len()).sum()
    }

    pub fn is_empty(&self) -> bool { self.len() == 0 }

    /// Earliest timestamp in the index.
    pub fn earliest(&self) -> Option<u64> {
        self.by_time.keys().next().copied()
    }

    /// Latest timestamp in the index.
    pub fn most_recent(&self) -> Option<u64> {
        self.by_time.keys().next_back().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn record_at(ts: u64) -> DataRecord {
        let mut r = DataRecord::new("s", "n", HashMap::new());
        r.created_at = ts;
        r.updated_at = ts;
        r
    }

    #[test]
    fn test_range_query() {
        let mut idx = TemporalIndex::new();
        idx.add(&record_at(100));
        idx.add(&record_at(200));
        idx.add(&record_at(300));
        assert_eq!(idx.range(100, 200).len(), 2);
        assert_eq!(idx.range(150, 250).len(), 1);
        assert_eq!(idx.range(400, 500).len(), 0);
    }

    #[test]
    fn test_latest() {
        let mut idx = TemporalIndex::new();
        idx.add(&record_at(100));
        idx.add(&record_at(200));
        idx.add(&record_at(300));
        let latest = idx.latest(2);
        assert_eq!(latest.len(), 2);
    }

    #[test]
    fn test_version_history() {
        let mut idx = TemporalIndex::new();
        let mut r = record_at(100);
        let id = r.id.clone();
        idx.add(&r);
        r.version = 2;
        r.updated_at = 200;
        idx.add(&r);
        let hist = idx.history(&id);
        assert_eq!(hist.len(), 2);
    }

    #[test]
    fn test_earliest_latest() {
        let mut idx = TemporalIndex::new();
        idx.add(&record_at(100));
        idx.add(&record_at(300));
        assert_eq!(idx.earliest(), Some(100));
        assert_eq!(idx.most_recent(), Some(300));
    }

    #[test]
    fn test_empty() {
        let idx = TemporalIndex::new();
        assert!(idx.is_empty());
        assert_eq!(idx.earliest(), None);
    }
}
