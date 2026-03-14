//! Data lineage / DNA — tracks provenance of every piece of data.
//!
//! Invention 10: Data DNA. Every record carries its complete history.

use serde::{Deserialize, Serialize};

/// A single entry in a lineage chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageEntry {
    /// What happened.
    pub action: LineageAction,
    /// Source of this action.
    pub source: String,
    /// When this happened (epoch micros).
    pub timestamp: u64,
    /// Input hash before this action.
    pub input_hash: String,
    /// Output hash after this action.
    pub output_hash: String,
    /// Human-readable description.
    pub description: String,
}

/// Types of lineage actions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineageAction {
    /// Data was ingested from an external source.
    Ingested,
    /// A transformation was applied.
    Transformed,
    /// Data was merged from multiple sources.
    Merged,
    /// A field was derived/computed.
    Derived,
    /// Data was corrected/updated.
    Corrected,
    /// Data was redacted (PII removed).
    Redacted,
    /// Data was migrated between formats.
    Migrated,
}

/// Complete lineage chain for a record or field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageChain {
    /// The record or field this chain belongs to.
    pub target: String,
    /// Ordered list of lineage entries (oldest first).
    pub entries: Vec<LineageEntry>,
    /// Trust score based on lineage depth (0.0–1.0).
    pub trust_score: f64,
}

impl LineageChain {
    /// Create a new empty chain.
    pub fn new(target: &str) -> Self {
        Self { target: target.to_string(), entries: Vec::new(), trust_score: 1.0 }
    }

    /// Add an entry and recalculate trust.
    pub fn add(&mut self, entry: LineageEntry) {
        self.entries.push(entry);
        self.trust_score = self.calculate_trust();
    }

    /// Trust decays with each transformation step.
    fn calculate_trust(&self) -> f64 {
        let transforms = self.entries.iter()
            .filter(|e| e.action == LineageAction::Transformed)
            .count();
        (1.0 - transforms as f64 * 0.05).max(0.1)
    }

    /// Depth of the lineage chain.
    pub fn depth(&self) -> usize {
        self.entries.len()
    }
}

/// A receipt for a single transformation step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformReceipt {
    /// Unique receipt ID.
    pub id: String,
    /// Transform name/type.
    pub transform: String,
    /// Input record count.
    pub input_count: u64,
    /// Output record count.
    pub output_count: u64,
    /// Records dropped (with reasons).
    pub dropped_count: u64,
    /// When the transform was applied.
    pub applied_at: u64,
    /// Whether this transform is reversible.
    pub reversible: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lineage_chain() {
        let mut chain = LineageChain::new("users.email");
        assert_eq!(chain.depth(), 0);
        assert_eq!(chain.trust_score, 1.0);

        chain.add(LineageEntry {
            action: LineageAction::Ingested,
            source: "csv".into(),
            timestamp: 1000,
            input_hash: "a".into(),
            output_hash: "b".into(),
            description: "Imported from CSV".into(),
        });
        assert_eq!(chain.depth(), 1);
        assert_eq!(chain.trust_score, 1.0); // Ingestion doesn't reduce trust

        chain.add(LineageEntry {
            action: LineageAction::Transformed,
            source: "normalize".into(),
            timestamp: 2000,
            input_hash: "b".into(),
            output_hash: "c".into(),
            description: "Lowercased email".into(),
        });
        assert_eq!(chain.depth(), 2);
        assert!(chain.trust_score < 1.0); // Transform reduces trust
    }
}
