//! Data quality types — health scores, anomalies, validation.
//!
//! Invention 8: Data Immune System.

use serde::{Deserialize, Serialize};

/// Quality score for a dataset, table, or column (0–100).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityScore {
    /// Overall score (0–100).
    pub score: u8,
    /// Completeness: ratio of non-null values.
    pub completeness: f64,
    /// Uniqueness: ratio of distinct values.
    pub uniqueness: f64,
    /// Consistency: ratio of values matching expected patterns.
    pub consistency: f64,
    /// Freshness: how recent the data is (0.0 = stale, 1.0 = fresh).
    pub freshness: f64,
    /// Accuracy: estimated correctness (based on validation rules).
    pub accuracy: f64,
    /// When this score was computed.
    pub computed_at: u64,
    /// Target (schema node name, field name, or "overall").
    pub target: String,
}

impl QualityScore {
    /// Compute overall score from dimensions.
    pub fn compute(target: &str, completeness: f64, uniqueness: f64, consistency: f64, freshness: f64, accuracy: f64) -> Self {
        let score = ((completeness + uniqueness + consistency + freshness + accuracy) / 5.0 * 100.0) as u8;
        Self {
            score, completeness, uniqueness, consistency, freshness, accuracy,
            computed_at: super::now_micros(), target: target.to_string(),
        }
    }

    /// Check if quality is below threshold.
    pub fn is_degraded(&self, threshold: u8) -> bool {
        self.score < threshold
    }
}

/// A detected anomaly in the data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyRecord {
    /// Unique anomaly ID.
    pub id: String,
    /// What type of anomaly.
    pub anomaly_type: AnomalyType,
    /// Severity (0.0 = minor, 1.0 = critical).
    pub severity: f64,
    /// Which field/column/record.
    pub target: String,
    /// Human-readable description.
    pub description: String,
    /// The anomalous value(s).
    pub values: Vec<String>,
    /// When detected.
    pub detected_at: u64,
    /// Whether this anomaly has been reviewed.
    pub reviewed: bool,
}

/// Types of anomalies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnomalyType {
    /// Statistical outlier (z-score > 3).
    Outlier,
    /// Sudden spike in null values.
    NullSpike,
    /// Value outside expected range.
    OutOfRange,
    /// Pattern break (expected format violated).
    PatternBreak,
    /// Duplicate where uniqueness expected.
    Duplicate,
    /// Type mismatch (string in numeric column).
    TypeMismatch,
    /// Sudden volume change.
    VolumeAnomaly,
}

/// A health metric tracked over time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetric {
    /// Metric name.
    pub name: String,
    /// Current value.
    pub value: f64,
    /// Previous value (for trend detection).
    pub previous: Option<f64>,
    /// Trend direction.
    pub trend: Trend,
    /// When recorded.
    pub recorded_at: u64,
}

/// Trend direction for a metric.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Trend {
    Improving,
    Stable,
    Degrading,
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_score() {
        let q = QualityScore::compute("users", 0.95, 0.80, 0.90, 1.0, 0.85);
        assert!(q.score >= 85);
        assert!(!q.is_degraded(80));
        assert!(q.is_degraded(95));
    }

    #[test]
    fn test_perfect_quality() {
        let q = QualityScore::compute("test", 1.0, 1.0, 1.0, 1.0, 1.0);
        assert_eq!(q.score, 100);
    }
}
