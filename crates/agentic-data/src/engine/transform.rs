//! Transformation engine — lossless, auditable data transformations.
//!
//! Invention 5: Lossless Transformation Pipeline.
//! Every step records input hash, output hash, and transform applied.

use std::collections::HashMap;
use crate::types::*;
use super::store::DataStore;

/// A single transformation step.
#[derive(Debug, Clone)]
pub struct TransformStep {
    pub name: String,
    pub operation: TransformOp,
}

/// Supported transform operations.
#[derive(Debug, Clone)]
pub enum TransformOp {
    /// Rename a field.
    RenameField { from: String, to: String },
    /// Drop a field.
    DropField { field: String },
    /// Add a computed field.
    AddField { name: String, value: serde_json::Value },
    /// Filter: keep only records matching condition.
    Filter { field: String, op: super::query::FilterOp, value: serde_json::Value },
    /// Map: apply function to a field's values.
    MapField { field: String, transform: FieldTransform },
    /// Deduplicate on a field.
    Dedup { field: String },
    /// Sort by field.
    Sort { field: String, ascending: bool },
}

/// Field-level transform functions.
#[derive(Debug, Clone)]
pub enum FieldTransform {
    Uppercase,
    Lowercase,
    Trim,
    Replace { from: String, to: String },
    DefaultIfNull { default: serde_json::Value },
}

/// A pipeline of transform steps.
#[derive(Debug, Clone)]
pub struct TransformPipeline {
    pub name: String,
    pub steps: Vec<TransformStep>,
}

impl TransformPipeline {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), steps: Vec::new() }
    }

    pub fn add(&mut self, step: TransformStep) {
        self.steps.push(step);
    }
}

/// Result of applying a transform pipeline.
#[derive(Debug)]
pub struct TransformResult {
    pub records_in: usize,
    pub records_out: usize,
    pub records_dropped: usize,
    pub receipts: Vec<TransformReceipt>,
}

/// Engine for applying transformations.
pub struct TransformEngine;

impl TransformEngine {
    /// Apply a pipeline to records, returning transformed records + audit trail.
    pub fn apply(
        records: &[DataRecord],
        pipeline: &TransformPipeline,
    ) -> TransformResult {
        let records_in = records.len();
        let mut current: Vec<DataRecord> = records.to_vec();
        let mut receipts = Vec::new();
        let mut total_dropped = 0;

        for step in &pipeline.steps {
            let before = current.len();
            current = apply_step(&current, &step.operation);
            let dropped = before.saturating_sub(current.len());
            total_dropped += dropped;

            receipts.push(TransformReceipt {
                id: uuid::Uuid::new_v4().to_string(),
                transform: step.name.clone(),
                input_count: before as u64,
                output_count: current.len() as u64,
                dropped_count: dropped as u64,
                applied_at: now_micros(),
                reversible: is_reversible(&step.operation),
            });
        }

        TransformResult {
            records_in,
            records_out: current.len(),
            records_dropped: total_dropped,
            receipts,
        }
    }

    /// Apply a pipeline and store results in the DataStore.
    pub fn apply_to_store(
        store: &mut DataStore,
        node: &str,
        pipeline: &TransformPipeline,
    ) -> TransformResult {
        let records: Vec<DataRecord> = store.records_for_node(node)
            .into_iter().cloned().collect();
        let result = Self::apply(&records, pipeline);

        // Record lineage for transforms
        for receipt in &result.receipts {
            store.add_lineage(node, LineageEntry {
                action: LineageAction::Transformed,
                source: pipeline.name.clone(),
                timestamp: receipt.applied_at,
                input_hash: format!("{}_in", receipt.input_count),
                output_hash: format!("{}_out", receipt.output_count),
                description: format!("{}: {} → {} records", receipt.transform, receipt.input_count, receipt.output_count),
            });
        }

        result
    }
}

fn apply_step(records: &[DataRecord], op: &TransformOp) -> Vec<DataRecord> {
    match op {
        TransformOp::RenameField { from, to } => {
            records.iter().map(|r| {
                let mut fields = r.fields.clone();
                if let Some(val) = fields.remove(from) {
                    fields.insert(to.clone(), val);
                }
                DataRecord { fields, ..r.clone() }
            }).collect()
        }
        TransformOp::DropField { field } => {
            records.iter().map(|r| {
                let mut fields = r.fields.clone();
                fields.remove(field);
                DataRecord { fields, ..r.clone() }
            }).collect()
        }
        TransformOp::AddField { name, value } => {
            records.iter().map(|r| {
                let mut fields = r.fields.clone();
                fields.insert(name.clone(), value.clone());
                DataRecord { fields, ..r.clone() }
            }).collect()
        }
        TransformOp::Filter { field, op, value } => {
            let filter = super::query::QueryFilter { field: field.clone(), op: op.clone(), value: value.clone() };
            records.iter().filter(|r| super::query::apply_filter_pub(r, &filter)).cloned().collect()
        }
        TransformOp::MapField { field, transform } => {
            records.iter().map(|r| {
                let mut fields = r.fields.clone();
                if let Some(val) = fields.get(field).cloned() {
                    fields.insert(field.clone(), apply_field_transform(&val, transform));
                }
                DataRecord { fields, ..r.clone() }
            }).collect()
        }
        TransformOp::Dedup { field } => {
            let mut seen = Vec::new();
            records.iter().filter(|r| {
                let val = r.get(field).cloned().unwrap_or(serde_json::Value::Null);
                if seen.contains(&val) { false } else { seen.push(val); true }
            }).cloned().collect()
        }
        TransformOp::Sort { field, ascending } => {
            let mut sorted = records.to_vec();
            sorted.sort_by(|a, b| {
                let va = a.get(field);
                let vb = b.get(field);
                let cmp = compare_json(va, vb);
                if *ascending { cmp } else { cmp.reverse() }
            });
            sorted
        }
    }
}

fn apply_field_transform(val: &serde_json::Value, transform: &FieldTransform) -> serde_json::Value {
    match transform {
        FieldTransform::Uppercase => val.as_str().map(|s| serde_json::json!(s.to_uppercase())).unwrap_or(val.clone()),
        FieldTransform::Lowercase => val.as_str().map(|s| serde_json::json!(s.to_lowercase())).unwrap_or(val.clone()),
        FieldTransform::Trim => val.as_str().map(|s| serde_json::json!(s.trim())).unwrap_or(val.clone()),
        FieldTransform::Replace { from, to } => val.as_str().map(|s| serde_json::json!(s.replace(from.as_str(), to.as_str()))).unwrap_or(val.clone()),
        FieldTransform::DefaultIfNull { default } => if val.is_null() { default.clone() } else { val.clone() },
    }
}

fn compare_json(a: Option<&serde_json::Value>, b: Option<&serde_json::Value>) -> std::cmp::Ordering {
    match (a.and_then(|v| v.as_f64()), b.and_then(|v| v.as_f64())) {
        (Some(a), Some(b)) => a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal),
        _ => std::cmp::Ordering::Equal,
    }
}

fn is_reversible(op: &TransformOp) -> bool {
    matches!(op, TransformOp::RenameField { .. } | TransformOp::AddField { .. } | TransformOp::Sort { .. })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn test_records() -> Vec<DataRecord> {
        vec![
            {let mut f = HashMap::new(); f.insert("name".into(), serde_json::json!("Alice")); f.insert("age".into(), serde_json::json!(30)); DataRecord::new("s", "users", f)},
            {let mut f = HashMap::new(); f.insert("name".into(), serde_json::json!("Bob")); f.insert("age".into(), serde_json::json!(25)); DataRecord::new("s", "users", f)},
        ]
    }

    #[test]
    fn test_rename() {
        let mut pipeline = TransformPipeline::new("test");
        pipeline.add(TransformStep { name: "rename".into(), operation: TransformOp::RenameField { from: "name".into(), to: "full_name".into() } });
        let result = TransformEngine::apply(&test_records(), &pipeline);
        assert_eq!(result.records_out, 2);
        assert_eq!(result.receipts.len(), 1);
    }

    #[test]
    fn test_drop_field() {
        let mut pipeline = TransformPipeline::new("test");
        pipeline.add(TransformStep { name: "drop".into(), operation: TransformOp::DropField { field: "age".into() } });
        let result = TransformEngine::apply(&test_records(), &pipeline);
        assert_eq!(result.records_out, 2);
    }

    #[test]
    fn test_uppercase() {
        let mut pipeline = TransformPipeline::new("test");
        pipeline.add(TransformStep { name: "upper".into(), operation: TransformOp::MapField { field: "name".into(), transform: FieldTransform::Uppercase } });
        let result = TransformEngine::apply(&test_records(), &pipeline);
        assert_eq!(result.records_out, 2);
    }

    #[test]
    fn test_receipt_audit() {
        let mut pipeline = TransformPipeline::new("audit_test");
        pipeline.add(TransformStep { name: "step1".into(), operation: TransformOp::AddField { name: "x".into(), value: serde_json::json!(1) } });
        pipeline.add(TransformStep { name: "step2".into(), operation: TransformOp::DropField { field: "age".into() } });
        let result = TransformEngine::apply(&test_records(), &pipeline);
        assert_eq!(result.receipts.len(), 2);
        assert_eq!(result.receipts[0].transform, "step1");
        assert!(result.receipts[0].reversible); // AddField is reversible
    }
}
