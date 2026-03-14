//! Quality analysis engine — data immune system.
//!
//! Invention 8: Data Immune System. Continuous quality monitoring,
//! anomaly detection, and quarantine.

use crate::types::*;
use super::store::DataStore;

/// Engine for analyzing data quality.
pub struct QualityEngine;

impl QualityEngine {
    /// Score quality for a schema node.
    pub fn score(store: &DataStore, node: &str) -> QualityScore {
        let records = store.records_for_node(node);
        if records.is_empty() {
            return QualityScore::compute(node, 0.0, 0.0, 0.0, 0.0, 0.0);
        }
        let total = records.len() as f64;

        // Completeness: ratio of non-null field values
        let mut total_fields = 0u64;
        let mut non_null_fields = 0u64;
        for r in &records {
            for val in r.fields.values() {
                total_fields += 1;
                if !val.is_null() { non_null_fields += 1; }
            }
        }
        let completeness = if total_fields > 0 { non_null_fields as f64 / total_fields as f64 } else { 1.0 };

        // Uniqueness: ratio of unique record checksums
        let mut checksums: Vec<&str> = records.iter().map(|r| r.checksum.as_str()).collect();
        checksums.sort();
        checksums.dedup();
        let uniqueness = checksums.len() as f64 / total;

        // Consistency: all records have the same field count (structural consistency)
        let field_counts: Vec<usize> = records.iter().map(|r| r.field_count()).collect();
        let mode_count = field_counts.iter().max().copied().unwrap_or(0);
        let consistent = field_counts.iter().filter(|&&c| c == mode_count).count();
        let consistency = consistent as f64 / total;

        // Freshness: based on most recent update timestamp
        let now = now_micros();
        let newest = records.iter().map(|r| r.updated_at).max().unwrap_or(0);
        let age_hours = (now.saturating_sub(newest)) as f64 / (3_600_000_000.0);
        let freshness = (1.0 - age_hours / 720.0).max(0.0).min(1.0); // Degrades over 30 days

        // Accuracy: placeholder (would need validation rules)
        let accuracy = 0.85;

        QualityScore::compute(node, completeness, uniqueness, consistency, freshness, accuracy)
    }

    /// Detect anomalies in a schema node.
    pub fn detect_anomalies(store: &DataStore, node: &str) -> Vec<AnomalyRecord> {
        let records = store.records_for_node(node);
        let mut anomalies = Vec::new();
        if records.is_empty() { return anomalies; }

        // Collect numeric values per field for statistical analysis
        let mut numeric_fields: std::collections::HashMap<String, Vec<f64>> = std::collections::HashMap::new();
        let mut null_counts: std::collections::HashMap<String, u64> = std::collections::HashMap::new();

        for r in &records {
            for (key, val) in &r.fields {
                if val.is_null() {
                    *null_counts.entry(key.clone()).or_default() += 1;
                } else if let Some(n) = val.as_f64() {
                    numeric_fields.entry(key.clone()).or_default().push(n);
                }
            }
        }

        let total = records.len() as f64;

        // Null spike detection: field with >50% nulls
        for (field, count) in &null_counts {
            let ratio = *count as f64 / total;
            if ratio > 0.5 {
                anomalies.push(AnomalyRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    anomaly_type: AnomalyType::NullSpike,
                    severity: ratio,
                    target: format!("{}.{}", node, field),
                    description: format!("{:.0}% of values are null", ratio * 100.0),
                    values: vec![format!("{}/{}", count, records.len())],
                    detected_at: now_micros(),
                    reviewed: false,
                });
            }
        }

        // Statistical outlier detection (z-score > 3)
        for (field, values) in &numeric_fields {
            if values.len() < 5 { continue; }
            let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;
            let variance: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
            let stddev = variance.sqrt();
            if stddev < f64::EPSILON { continue; }

            for val in values {
                let z = (val - mean).abs() / stddev;
                if z > 3.0 {
                    anomalies.push(AnomalyRecord {
                        id: uuid::Uuid::new_v4().to_string(),
                        anomaly_type: AnomalyType::Outlier,
                        severity: (z / 5.0).min(1.0),
                        target: format!("{}.{}", node, field),
                        description: format!("Outlier: {} (z-score: {:.1}, mean: {:.1})", val, z, mean),
                        values: vec![format!("{}", val)],
                        detected_at: now_micros(),
                        reviewed: false,
                    });
                    break; // One anomaly per field is enough
                }
            }
        }

        anomalies
    }

    /// Full health report for all data in the store.
    pub fn health_report(store: &DataStore) -> Vec<QualityScore> {
        let mut scores = Vec::new();
        // Score each unique schema node
        let mut nodes: Vec<String> = store.active_records().iter()
            .map(|r| r.schema_node.clone())
            .collect();
        nodes.sort();
        nodes.dedup();
        for node in nodes {
            scores.push(Self::score(store, &node));
        }
        scores
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn populated_store() -> DataStore {
        let mut store = DataStore::new();
        for i in 0..10 {
            let mut f = HashMap::new();
            f.insert("value".into(), serde_json::json!(i * 10));
            f.insert("name".into(), serde_json::json!(format!("item_{}", i)));
            store.add_record(DataRecord::new("s", "items", f));
        }
        store
    }

    #[test]
    fn test_quality_score() {
        let store = populated_store();
        let score = QualityEngine::score(&store, "items");
        assert!(score.score > 50);
        assert!(score.completeness > 0.9);
    }

    #[test]
    fn test_anomaly_detection() {
        let mut store = DataStore::new();
        // Add normal values
        for i in 0..20 {
            let mut f = HashMap::new();
            f.insert("val".into(), serde_json::json!(10.0 + i as f64 * 0.5));
            store.add_record(DataRecord::new("s", "data", f));
        }
        // Add outlier
        let mut f = HashMap::new();
        f.insert("val".into(), serde_json::json!(1000.0));
        store.add_record(DataRecord::new("s", "data", f));

        let anomalies = QualityEngine::detect_anomalies(&store, "data");
        assert!(!anomalies.is_empty());
        assert!(anomalies.iter().any(|a| a.anomaly_type == AnomalyType::Outlier));
    }

    #[test]
    fn test_null_spike_detection() {
        let mut store = DataStore::new();
        for i in 0..10 {
            let mut f = HashMap::new();
            f.insert("a".into(), if i < 7 { serde_json::Value::Null } else { serde_json::json!(i) });
            store.add_record(DataRecord::new("s", "data", f));
        }
        let anomalies = QualityEngine::detect_anomalies(&store, "data");
        assert!(anomalies.iter().any(|a| a.anomaly_type == AnomalyType::NullSpike));
    }

    #[test]
    fn test_health_report() {
        let store = populated_store();
        let report = QualityEngine::health_report(&store);
        assert_eq!(report.len(), 1); // One node: "items"
    }

    #[test]
    fn test_empty_store() {
        let store = DataStore::new();
        let score = QualityEngine::score(&store, "nothing");
        assert_eq!(score.score, 0);
    }
}
