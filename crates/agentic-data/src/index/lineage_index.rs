//! Lineage index — fast provenance lookups and impact analysis.
//!
//! Invention 10: Data DNA (indexing layer).

use std::collections::HashMap;
use crate::types::*;

/// Index for lineage chain lookups and dependency tracking.
#[derive(Debug, Default)]
pub struct LineageIndex {
    /// Target → lineage chain.
    chains: HashMap<String, LineageChain>,
    /// Source → list of targets derived from it (forward dependencies).
    forward_deps: HashMap<String, Vec<String>>,
    /// Target → list of sources it depends on (backward dependencies).
    backward_deps: HashMap<String, Vec<String>>,
}

impl LineageIndex {
    pub fn new() -> Self { Self::default() }

    /// Index a lineage chain.
    pub fn add_chain(&mut self, chain: &LineageChain) {
        self.chains.insert(chain.target.clone(), chain.clone());
        for entry in &chain.entries {
            self.forward_deps.entry(entry.source.clone())
                .or_default()
                .push(chain.target.clone());
            self.backward_deps.entry(chain.target.clone())
                .or_default()
                .push(entry.source.clone());
        }
    }

    /// Add a single lineage entry for a target.
    pub fn add_entry(&mut self, target: &str, entry: LineageEntry) {
        self.forward_deps.entry(entry.source.clone())
            .or_default()
            .push(target.to_string());
        self.backward_deps.entry(target.to_string())
            .or_default()
            .push(entry.source.clone());
        self.chains.entry(target.to_string())
            .or_insert_with(|| LineageChain::new(target))
            .add(entry);
    }

    /// Get the full lineage chain for a target.
    pub fn trace(&self, target: &str) -> Option<&LineageChain> {
        self.chains.get(target)
    }

    /// Get trust score for a target.
    pub fn trust_score(&self, target: &str) -> f64 {
        self.chains.get(target).map(|c| c.trust_score).unwrap_or(0.0)
    }

    /// Impact analysis: if source changes, what targets are affected?
    pub fn impact_of(&self, source: &str) -> Vec<&str> {
        self.forward_deps.get(source)
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Source tracing: where did this target's data come from?
    pub fn sources_of(&self, target: &str) -> Vec<&str> {
        self.backward_deps.get(target)
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Find all targets with low trust score.
    pub fn low_trust_targets(&self, threshold: f64) -> Vec<(&str, f64)> {
        self.chains.iter()
            .filter(|(_, chain)| chain.trust_score < threshold)
            .map(|(target, chain)| (target.as_str(), chain.trust_score))
            .collect()
    }

    /// Total indexed chains.
    pub fn chain_count(&self) -> usize { self.chains.len() }

    /// Generate a DNA fingerprint for a target.
    pub fn fingerprint(&self, target: &str) -> Option<String> {
        let chain = self.chains.get(target)?;
        let data = format!("{:?}", chain.entries);
        Some(blake3::hash(data.as_bytes()).to_hex()[..16].to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_trace() {
        let mut idx = LineageIndex::new();
        idx.add_entry("users.email", LineageEntry {
            action: LineageAction::Ingested, source: "csv_import".into(),
            timestamp: 1000, input_hash: "a".into(), output_hash: "b".into(),
            description: "Imported".into(),
        });
        let chain = idx.trace("users.email").unwrap();
        assert_eq!(chain.depth(), 1);
    }

    #[test]
    fn test_impact_analysis() {
        let mut idx = LineageIndex::new();
        idx.add_entry("orders.total", LineageEntry {
            action: LineageAction::Derived, source: "orders.quantity".into(),
            timestamp: 1000, input_hash: "".into(), output_hash: "".into(),
            description: "Computed".into(),
        });
        idx.add_entry("reports.revenue", LineageEntry {
            action: LineageAction::Derived, source: "orders.total".into(),
            timestamp: 2000, input_hash: "".into(), output_hash: "".into(),
            description: "Aggregated".into(),
        });
        // If orders.quantity changes, orders.total is affected
        let impact = idx.impact_of("orders.quantity");
        assert!(impact.contains(&"orders.total"));
    }

    #[test]
    fn test_sources_of() {
        let mut idx = LineageIndex::new();
        idx.add_entry("target", LineageEntry {
            action: LineageAction::Merged, source: "source_a".into(),
            timestamp: 1000, input_hash: "".into(), output_hash: "".into(),
            description: "Merged".into(),
        });
        idx.add_entry("target", LineageEntry {
            action: LineageAction::Merged, source: "source_b".into(),
            timestamp: 2000, input_hash: "".into(), output_hash: "".into(),
            description: "Merged".into(),
        });
        let sources = idx.sources_of("target");
        assert_eq!(sources.len(), 2);
    }

    #[test]
    fn test_trust_score() {
        let mut idx = LineageIndex::new();
        idx.add_entry("x", LineageEntry {
            action: LineageAction::Transformed, source: "a".into(),
            timestamp: 1000, input_hash: "".into(), output_hash: "".into(),
            description: "".into(),
        });
        assert!(idx.trust_score("x") < 1.0);
        assert_eq!(idx.trust_score("missing"), 0.0);
    }

    #[test]
    fn test_fingerprint() {
        let mut idx = LineageIndex::new();
        idx.add_entry("x", LineageEntry {
            action: LineageAction::Ingested, source: "csv".into(),
            timestamp: 1000, input_hash: "a".into(), output_hash: "b".into(),
            description: "Import".into(),
        });
        let fp = idx.fingerprint("x");
        assert!(fp.is_some());
        assert_eq!(fp.unwrap().len(), 16);
    }
}
