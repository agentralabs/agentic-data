//! PII detection and redaction — context-aware sensitive data handling.
//!
//! Invention 17: Data Redaction Intelligence.

use crate::types::*;

/// Result of PII detection scan.
#[derive(Debug, Clone)]
pub struct PiiDetection {
    pub field: String,
    pub pii_type: PiiType,
    pub confidence: f64,
    pub value_preview: String,
}

/// Types of PII detected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PiiType {
    Email,
    Phone,
    Ssn,
    CreditCard,
    IpAddress,
    Name,
    Address,
    DateOfBirth,
    Custom(String),
}

/// Redaction policy configuration.
#[derive(Debug, Clone)]
pub struct RedactionPolicy {
    pub name: String,
    /// PII types to redact.
    pub types: Vec<PiiType>,
    /// Redaction method.
    pub method: RedactionMethod,
}

/// How to redact detected PII.
#[derive(Debug, Clone)]
pub enum RedactionMethod {
    /// Replace with [REDACTED].
    Mask,
    /// Replace with type-appropriate placeholder.
    Placeholder,
    /// Replace with a hash of the original.
    Hash,
    /// Remove the field entirely.
    Remove,
}

impl Default for RedactionPolicy {
    fn default() -> Self {
        Self {
            name: "default".into(),
            types: vec![PiiType::Email, PiiType::Phone, PiiType::Ssn, PiiType::CreditCard],
            method: RedactionMethod::Mask,
        }
    }
}

/// Engine for PII detection and redaction.
pub struct RedactionEngine;

impl RedactionEngine {
    /// Scan a record for PII.
    pub fn detect(record: &DataRecord) -> Vec<PiiDetection> {
        let mut detections = Vec::new();
        for (field, value) in &record.fields {
            if let Some(s) = value.as_str() {
                detections.extend(detect_pii_in_string(field, s));
            }
        }
        detections
    }

    /// Apply a redaction policy to a record. Returns modified record.
    pub fn redact(record: &DataRecord, policy: &RedactionPolicy) -> DataRecord {
        let detections = Self::detect(record);
        let mut fields = record.fields.clone();

        for detection in &detections {
            if policy.types.contains(&detection.pii_type) {
                if let Some(val) = fields.get(&detection.field) {
                    if let Some(s) = val.as_str() {
                        let redacted = apply_redaction(s, &detection.pii_type, &policy.method);
                        fields.insert(detection.field.clone(), serde_json::json!(redacted));
                    }
                }
            }
        }

        let mut result = record.clone();
        result.fields = fields;
        result.status = if !detections.is_empty() { RecordStatus::Redacted } else { record.status };
        result
    }

    /// Scan a batch of records and return all detections.
    pub fn scan_batch(records: &[DataRecord]) -> Vec<(String, PiiDetection)> {
        records.iter()
            .flat_map(|r| Self::detect(r).into_iter().map(|d| (r.id.clone(), d)))
            .collect()
    }
}

/// Detect PII patterns in a string value.
fn detect_pii_in_string(field: &str, value: &str) -> Vec<PiiDetection> {
    let mut detections = Vec::new();
    let preview = if value.len() > 20 { format!("{}...", &value[..20]) } else { value.to_string() };

    // Email: contains @ and .
    if value.contains('@') && value.contains('.') && value.len() > 5 {
        let parts: Vec<&str> = value.split('@').collect();
        if parts.len() == 2 && parts[1].contains('.') {
            detections.push(PiiDetection {
                field: field.into(), pii_type: PiiType::Email,
                confidence: 0.95, value_preview: preview.clone(),
            });
        }
    }

    // Phone: digits with optional dashes/spaces, 10+ digits
    let digit_count = value.chars().filter(|c| c.is_ascii_digit()).count();
    if digit_count >= 10 && digit_count <= 15 && value.len() < 20 {
        detections.push(PiiDetection {
            field: field.into(), pii_type: PiiType::Phone,
            confidence: 0.70, value_preview: preview.clone(),
        });
    }

    // SSN: NNN-NN-NNNN
    if value.len() == 11 && value.chars().enumerate().all(|(i, c)| {
        if i == 3 || i == 6 { c == '-' } else { c.is_ascii_digit() }
    }) {
        detections.push(PiiDetection {
            field: field.into(), pii_type: PiiType::Ssn,
            confidence: 0.90, value_preview: preview.clone(),
        });
    }

    // Credit card: 13-19 digits (possibly with spaces/dashes)
    let cleaned: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
    if cleaned.len() >= 13 && cleaned.len() <= 19 && value.len() < 25 {
        if luhn_check(&cleaned) {
            detections.push(PiiDetection {
                field: field.into(), pii_type: PiiType::CreditCard,
                confidence: 0.85, value_preview: preview.clone(),
            });
        }
    }

    // IP address: N.N.N.N
    let octets: Vec<&str> = value.split('.').collect();
    if octets.len() == 4 && octets.iter().all(|o| o.parse::<u8>().is_ok()) {
        detections.push(PiiDetection {
            field: field.into(), pii_type: PiiType::IpAddress,
            confidence: 0.90, value_preview: preview,
        });
    }

    detections
}

/// Luhn algorithm for credit card validation.
fn luhn_check(digits: &str) -> bool {
    let mut sum = 0u32;
    let mut double = false;
    for c in digits.chars().rev() {
        let d = c.to_digit(10).unwrap_or(0);
        let val = if double { let x = d * 2; if x > 9 { x - 9 } else { x } } else { d };
        sum += val;
        double = !double;
    }
    sum % 10 == 0
}

/// Apply a specific redaction method.
fn apply_redaction(value: &str, pii_type: &PiiType, method: &RedactionMethod) -> String {
    match method {
        RedactionMethod::Mask => "[REDACTED]".into(),
        RedactionMethod::Placeholder => match pii_type {
            PiiType::Email => "redacted@example.com".into(),
            PiiType::Phone => "000-000-0000".into(),
            PiiType::Ssn => "XXX-XX-XXXX".into(),
            PiiType::CreditCard => "XXXX-XXXX-XXXX-XXXX".into(),
            PiiType::IpAddress => "0.0.0.0".into(),
            _ => "[REDACTED]".into(),
        },
        RedactionMethod::Hash => {
            let hash = blake3::hash(value.as_bytes());
            format!("HASH:{}", &hash.to_hex()[..16])
        }
        RedactionMethod::Remove => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_detect_email() {
        let mut f = HashMap::new();
        f.insert("email".into(), serde_json::json!("alice@example.com"));
        let rec = DataRecord::new("s", "n", f);
        let detections = RedactionEngine::detect(&rec);
        assert!(detections.iter().any(|d| d.pii_type == PiiType::Email));
    }

    #[test]
    fn test_detect_ssn() {
        let mut f = HashMap::new();
        f.insert("ssn".into(), serde_json::json!("123-45-6789"));
        let rec = DataRecord::new("s", "n", f);
        let detections = RedactionEngine::detect(&rec);
        assert!(detections.iter().any(|d| d.pii_type == PiiType::Ssn));
    }

    #[test]
    fn test_detect_ip() {
        let mut f = HashMap::new();
        f.insert("ip".into(), serde_json::json!("192.168.1.1"));
        let rec = DataRecord::new("s", "n", f);
        let detections = RedactionEngine::detect(&rec);
        assert!(detections.iter().any(|d| d.pii_type == PiiType::IpAddress));
    }

    #[test]
    fn test_redact_mask() {
        let mut f = HashMap::new();
        f.insert("email".into(), serde_json::json!("alice@example.com"));
        f.insert("name".into(), serde_json::json!("Alice"));
        let rec = DataRecord::new("s", "n", f);
        let policy = RedactionPolicy::default();
        let redacted = RedactionEngine::redact(&rec, &policy);
        assert_eq!(redacted.get_str("email"), Some("[REDACTED]"));
        assert_eq!(redacted.get_str("name"), Some("Alice")); // Not PII
    }

    #[test]
    fn test_redact_placeholder() {
        let mut f = HashMap::new();
        f.insert("ssn".into(), serde_json::json!("123-45-6789"));
        let rec = DataRecord::new("s", "n", f);
        let policy = RedactionPolicy {
            types: vec![PiiType::Ssn],
            method: RedactionMethod::Placeholder,
            ..Default::default()
        };
        let redacted = RedactionEngine::redact(&rec, &policy);
        assert_eq!(redacted.get_str("ssn"), Some("XXX-XX-XXXX"));
    }

    #[test]
    fn test_luhn() {
        assert!(luhn_check("4111111111111111")); // Valid Visa test
        assert!(!luhn_check("1234567890123456"));
    }

    #[test]
    fn test_no_false_positives() {
        let mut f = HashMap::new();
        f.insert("note".into(), serde_json::json!("Just a normal note"));
        let rec = DataRecord::new("s", "n", f);
        let detections = RedactionEngine::detect(&rec);
        assert!(detections.is_empty());
    }
}
