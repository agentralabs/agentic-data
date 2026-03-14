//! Quality index — tracks quality scores and anomalies over time.
//!
//! Invention 8: Data Immune System (indexing layer).

use std::collections::HashMap;
use crate::types::*;

/// Index for quality scores and anomaly history.
#[derive(Debug, Default)]
pub struct QualityIndex {
    /// Target → list of quality scores over time.
    scores: HashMap<String, Vec<QualityScore>>,
    /// Target → list of anomalies.
    anomalies: HashMap<String, Vec<AnomalyRecord>>,
    /// Record IDs that are quarantined.
    quarantined: Vec<String>,
}

impl QualityIndex {
    pub fn new() -> Self { Self::default() }

    /// Record a quality score snapshot.
    pub fn add_score(&mut self, score: QualityScore) {
        self.scores.entry(score.target.clone()).or_default().push(score);
    }

    /// Record an anomaly.
    pub fn add_anomaly(&mut self, anomaly: AnomalyRecord) {
        self.anomalies.entry(anomaly.target.clone()).or_default().push(anomaly);
    }

    /// Quarantine a record by ID.
    pub fn quarantine(&mut self, record_id: &str) {
        if !self.quarantined.contains(&record_id.to_string()) {
            self.quarantined.push(record_id.to_string());
        }
    }

    /// Check if a record is quarantined.
    pub fn is_quarantined(&self, record_id: &str) -> bool {
        self.quarantined.contains(&record_id.to_string())
    }

    /// Get quality timeline for a target.
    pub fn timeline(&self, target: &str) -> Vec<&QualityScore> {
        self.scores.get(target).map(|v| v.iter().collect()).unwrap_or_default()
    }

    /// Get latest quality score for a target.
    pub fn latest_score(&self, target: &str) -> Option<&QualityScore> {
        self.scores.get(target)?.last()
    }

    /// Get anomalies for a target.
    pub fn anomalies_for(&self, target: &str) -> Vec<&AnomalyRecord> {
        self.anomalies.get(target).map(|v| v.iter().collect()).unwrap_or_default()
    }

    /// Get all unreviewed anomalies.
    pub fn unreviewed_anomalies(&self) -> Vec<&AnomalyRecord> {
        self.anomalies.values()
            .flat_map(|v| v.iter())
            .filter(|a| !a.reviewed)
            .collect()
    }

    /// Get quality trend for a target (improving, stable, degrading).
    pub fn trend(&self, target: &str) -> Trend {
        let scores = match self.scores.get(target) {
            Some(s) if s.len() >= 2 => s,
            _ => return Trend::Unknown,
        };
        let recent = scores.last().unwrap().score as i16;
        let previous = scores[scores.len() - 2].score as i16;
        let diff = recent - previous;
        if diff > 5 { Trend::Improving }
        else if diff < -5 { Trend::Degrading }
        else { Trend::Stable }
    }

    /// Total anomaly count.
    pub fn anomaly_count(&self) -> usize {
        self.anomalies.values().map(|v| v.len()).sum()
    }

    /// Total quarantined count.
    pub fn quarantined_count(&self) -> usize { self.quarantined.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_timeline() {
        let mut idx = QualityIndex::new();
        idx.add_score(QualityScore::compute("users", 0.9, 0.8, 0.9, 1.0, 0.85));
        idx.add_score(QualityScore::compute("users", 0.95, 0.85, 0.92, 1.0, 0.90));
        assert_eq!(idx.timeline("users").len(), 2);
        assert!(idx.latest_score("users").is_some());
    }

    #[test]
    fn test_trend_improving() {
        let mut idx = QualityIndex::new();
        idx.add_score(QualityScore::compute("x", 0.5, 0.5, 0.5, 0.5, 0.5)); // 50
        idx.add_score(QualityScore::compute("x", 0.9, 0.9, 0.9, 0.9, 0.9)); // 90
        assert_eq!(idx.trend("x"), Trend::Improving);
    }

    #[test]
    fn test_quarantine() {
        let mut idx = QualityIndex::new();
        idx.quarantine("rec-1");
        assert!(idx.is_quarantined("rec-1"));
        assert!(!idx.is_quarantined("rec-2"));
        assert_eq!(idx.quarantined_count(), 1);
    }

    #[test]
    fn test_anomaly_tracking() {
        let mut idx = QualityIndex::new();
        idx.add_anomaly(AnomalyRecord {
            id: "a1".into(), anomaly_type: AnomalyType::Outlier,
            severity: 0.8, target: "data.val".into(),
            description: "Outlier".into(), values: vec!["1000".into()],
            detected_at: 0, reviewed: false,
        });
        assert_eq!(idx.anomaly_count(), 1);
        assert_eq!(idx.unreviewed_anomalies().len(), 1);
    }
}
