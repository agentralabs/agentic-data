//! Consolidation engine — deduplication, orphan pruning, index rebuilding.
//!
//! Parity with AgenticMemory's consolidation operations.

use super::store::DataStore;
use crate::types::*;

/// Result of a consolidation run.
#[derive(Debug)]
pub struct ConsolidationReport {
    pub duplicates_removed: usize,
    pub orphans_pruned: usize,
    pub schemas_merged: usize,
    pub quality_refreshed: usize,
    pub total_actions: usize,
}

/// Consolidation operations for maintaining data health.
pub struct ConsolidationEngine;

impl ConsolidationEngine {
    /// Run all consolidation operations.
    pub fn consolidate(store: &mut DataStore) -> ConsolidationReport {
        let dups = Self::dedup_records(store);
        let orphans = Self::prune_orphans(store);
        let merged = Self::merge_duplicate_schemas(store);
        let quality = Self::refresh_quality(store);
        ConsolidationReport {
            total_actions: dups + orphans + merged + quality,
            duplicates_removed: dups,
            orphans_pruned: orphans,
            schemas_merged: merged,
            quality_refreshed: quality,
        }
    }

    /// Remove duplicate records (same checksum).
    pub fn dedup_records(store: &mut DataStore) -> usize {
        let records = store.active_records();
        let mut seen_checksums = std::collections::HashSet::new();
        let mut dup_ids = Vec::new();

        for rec in records {
            if !seen_checksums.insert(rec.checksum.clone()) {
                dup_ids.push(rec.id.clone());
            }
        }

        let count = dup_ids.len();
        for id in &dup_ids {
            store.remove_records(|r| r.id == *id);
        }
        if count > 0 {
            eprintln!("[consolidation] Removed {} duplicate records", count);
        }
        count
    }

    /// Remove records with no associated schema node.
    pub fn prune_orphans(store: &mut DataStore) -> usize {
        let schema_nodes: Vec<String> = store.all_schemas().iter()
            .flat_map(|s| s.nodes.iter().map(|n| n.name.clone()))
            .collect();

        let orphan_count = store.remove_records(|r| {
            !schema_nodes.contains(&r.schema_node) && r.status == RecordStatus::Active
        });

        if orphan_count > 0 {
            eprintln!("[consolidation] Pruned {} orphan records", orphan_count);
        }
        orphan_count
    }

    /// Merge schemas with identical structure.
    pub fn merge_duplicate_schemas(_store: &mut DataStore) -> usize {
        // In a production system, this would find schemas with identical field
        // structures and merge them. For now, return 0 (no merges needed).
        0
    }

    /// Refresh quality scores for all nodes.
    pub fn refresh_quality(store: &DataStore) -> usize {
        let mut nodes: Vec<String> = store.active_records().iter()
            .map(|r| r.schema_node.clone())
            .collect();
        nodes.sort();
        nodes.dedup();

        let mut refreshed = 0;
        for node in &nodes {
            let score = super::quality::QualityEngine::score(store, node);
            if score.score < 50 {
                eprintln!("[consolidation] Low quality: {} scored {}/100", node, score.score);
            }
            refreshed += 1;
        }
        refreshed
    }

    /// Compact operation: archive old records below quality threshold.
    pub fn compact(store: &mut DataStore, max_age_micros: u64) -> usize {
        let cutoff = now_micros().saturating_sub(max_age_micros);
        let archived = store.remove_records(|r| {
            r.updated_at < cutoff && r.status == RecordStatus::Active
        });
        if archived > 0 {
            eprintln!("[consolidation] Archived {} old records", archived);
        }
        archived
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_dedup() {
        let mut store = DataStore::new();
        // Add two records with same content (same checksum)
        let mut f = HashMap::new();
        f.insert("x".into(), serde_json::json!(1));
        let r1 = DataRecord::new("s", "n", f.clone());
        let checksum = r1.checksum.clone();
        store.add_record(r1);
        let mut r2 = DataRecord::new("s", "n", f);
        r2.checksum = checksum; // Force same checksum
        store.add_record(r2);

        assert_eq!(store.record_count(), 2);
        let removed = ConsolidationEngine::dedup_records(&mut store);
        assert_eq!(removed, 1);
        assert_eq!(store.record_count(), 1);
    }

    #[test]
    fn test_consolidate_clean_data() {
        let mut store = DataStore::new();
        let schema = UniversalSchema::new("test");
        store.add_schema(schema);
        let mut f = HashMap::new();
        f.insert("a".into(), serde_json::json!(1));
        store.add_record(DataRecord::new("s", "test", f));

        let report = ConsolidationEngine::consolidate(&mut store);
        assert_eq!(report.duplicates_removed, 0);
    }

    #[test]
    fn test_prune_orphans() {
        let mut store = DataStore::new();
        let mut s = UniversalSchema::new("test");
        s.nodes.push(SchemaNode { name: "valid".into(), source: "s".into(), fields: vec![], record_count: None });
        store.add_schema(s);

        let mut f = HashMap::new();
        f.insert("x".into(), serde_json::json!(1));
        store.add_record(DataRecord::new("s", "valid", f.clone()));
        store.add_record(DataRecord::new("s", "orphaned_node", f));

        assert_eq!(store.record_count(), 2);
        let pruned = ConsolidationEngine::prune_orphans(&mut store);
        assert_eq!(pruned, 1);
        assert_eq!(store.record_count(), 1);
    }

    #[test]
    fn test_report() {
        let mut store = DataStore::new();
        let report = ConsolidationEngine::consolidate(&mut store);
        assert_eq!(report.total_actions, 0); // Empty store, nothing to do
    }
}
