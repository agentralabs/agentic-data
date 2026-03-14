//! Edge case tests — malformed inputs, boundary conditions, adversarial data.

use agentic_data::*;
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════
// EMPTY / MINIMAL INPUT EDGE CASES
// ═══════════════════════════════════════════════════════════

#[test]
fn test_empty_string_detection() {
    let d = parser::detect::detect_format("", None);
    assert_eq!(d.format, parser::DataFormat::Unknown);
    assert_eq!(d.confidence, 0.0);
}

#[test]
fn test_whitespace_only_detection() {
    let d = parser::detect::detect_format("   \n\n  \t  \n", None);
    assert_eq!(d.format, parser::DataFormat::Unknown);
}

#[test]
fn test_single_char_detection() {
    let d = parser::detect::detect_format("x", None);
    // Should not crash, may detect as unknown or markdown
    assert!(d.confidence <= 1.0);
}

#[test]
fn test_csv_header_only() {
    let result = parser::parse_auto("name,age,city\n", "test");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().records.len(), 0);
}

#[test]
fn test_csv_single_column() {
    // Single column CSV has no delimiters — must specify format explicitly
    let result = parser::parse_as("value\n1\n2\n3\n", "test", parser::DataFormat::Csv);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().records.len(), 3);
}

#[test]
fn test_json_empty_array() {
    let result = parser::parse_auto("[]", "test");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().records.len(), 0);
}

#[test]
fn test_json_empty_object() {
    let result = parser::parse_auto("{}", "test");
    assert!(result.is_ok());
}

#[test]
fn test_json_nested_deeply() {
    let data = r#"[{"a":{"b":{"c":{"d":"deep"}}}}]"#;
    let result = parser::parse_auto(data, "test");
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════
// MALFORMED INPUT EDGE CASES
// ═══════════════════════════════════════════════════════════

#[test]
fn test_csv_inconsistent_columns() {
    let data = "a,b,c\n1,2,3\n4,5\n6,7,8,9\n";
    let result = parser::csv_parser::parse(data, "test", b',').unwrap();
    assert!(result.errors > 0, "Should report errors for inconsistent columns");
    assert!(result.records.len() < 3, "Should not parse all rows");
}

#[test]
fn test_json_invalid() {
    let result = parser::json_parser::parse("{broken json", "test");
    assert!(result.is_err());
}

#[test]
fn test_json_non_object_array() {
    let result = parser::json_parser::parse("[1, 2, 3]", "test");
    // Array of primitives, not objects
    assert!(result.is_ok());
    // Records should be 0 since elements are not objects
    assert_eq!(result.unwrap().errors, 3);
}

#[test]
fn test_csv_with_quotes_and_commas() {
    let data = "name,description\nAlice,\"Has a cat, dog\"\nBob,\"No pets\"\n";
    // Our simple parser splits on commas, so quoted fields with commas will break
    // This is a known limitation — production would use the csv crate
    let result = parser::csv_parser::parse(data, "test", b',');
    assert!(result.is_ok()); // Should not crash
}

#[test]
fn test_unicode_content() {
    let data = "name,city\nSakura,東京\nMüller,München\nO'Brien,Zürich\n";
    let result = parser::parse_auto(data, "test");
    assert!(result.is_ok());
    let records = result.unwrap().records;
    assert_eq!(records.len(), 3);
}

// ═══════════════════════════════════════════════════════════
// LARGE DATA EDGE CASES
// ═══════════════════════════════════════════════════════════

#[test]
fn test_large_csv_1000_rows() {
    let mut data = String::from("id,value,label\n");
    for i in 0..1000 {
        data.push_str(&format!("{},{},{}\n", i, i * 10, format!("item_{}", i)));
    }
    let result = parser::parse_auto(&data, "large").unwrap();
    assert_eq!(result.records.len(), 1000);
    assert_eq!(result.errors, 0);
}

#[test]
fn test_large_json_array() {
    let items: Vec<String> = (0..500)
        .map(|i| format!(r#"{{"id":{},"name":"item_{}","value":{}}}"#, i, i, i * 3))
        .collect();
    let data = format!("[{}]", items.join(","));
    let result = parser::parse_auto(&data, "large").unwrap();
    assert_eq!(result.records.len(), 500);
}

#[test]
fn test_quality_score_large_dataset() {
    let mut store = DataStore::new();
    for i in 0..500 {
        let mut f = HashMap::new();
        f.insert("id".into(), serde_json::json!(i));
        f.insert("value".into(), serde_json::json!(i as f64 * 1.1));
        f.insert("name".into(), serde_json::json!(format!("item_{}", i)));
        store.add_record(DataRecord::new("s", "big", f));
    }
    let score = QualityEngine::score(&store, "big");
    assert!(score.score >= 80, "Large clean dataset should score high: {}", score.score);
    assert!(score.completeness >= 0.99);
}

// ═══════════════════════════════════════════════════════════
// SPATIAL EDGE CASES
// ═══════════════════════════════════════════════════════════

#[test]
fn test_geo_same_point_distance() {
    let p = GeoPoint::new(40.0, -74.0);
    assert_eq!(p.distance_meters(&p), 0.0);
}

#[test]
fn test_geo_antipodal_distance() {
    let a = GeoPoint::new(0.0, 0.0);
    let b = GeoPoint::new(0.0, 180.0);
    let dist = a.distance_meters(&b);
    // Half earth circumference: ~20,015 km
    assert!(dist > 19_000_000.0 && dist < 21_000_000.0);
}

#[test]
fn test_geo_poles() {
    let north = GeoPoint::new(90.0, 0.0);
    let south = GeoPoint::new(-90.0, 0.0);
    let dist = north.distance_meters(&south);
    // Pole to pole: ~20,004 km
    assert!(dist > 19_000_000.0 && dist < 21_000_000.0);
}

#[test]
fn test_geo_bounds_crossing_dateline() {
    let bounds = GeoBounds::new(-10.0, 170.0, 10.0, -170.0);
    // This is a degenerate case — our simple implementation doesn't handle dateline
    // Just verify it doesn't crash
    let p = GeoPoint::new(0.0, 175.0);
    let _ = p.within(&bounds);
}

// ═══════════════════════════════════════════════════════════
// PII EDGE CASES
// ═══════════════════════════════════════════════════════════

#[test]
fn test_pii_partial_email() {
    let mut f = HashMap::new();
    f.insert("data".into(), serde_json::json!("not@email")); // No TLD
    let rec = DataRecord::new("s", "n", f);
    let detections = RedactionEngine::detect(&rec);
    // Should NOT detect as email (no dot after @)
    assert!(!detections.iter().any(|d| d.pii_type == crypto::redaction::PiiType::Email),
        "Should not detect 'not@email' as email");
}

#[test]
fn test_pii_credit_card_luhn() {
    let mut f = HashMap::new();
    f.insert("card".into(), serde_json::json!("4111111111111111")); // Valid Visa test number
    let rec = DataRecord::new("s", "n", f);
    let detections = RedactionEngine::detect(&rec);
    assert!(detections.iter().any(|d| d.pii_type == crypto::redaction::PiiType::CreditCard),
        "Should detect valid Luhn credit card");
}

#[test]
fn test_pii_invalid_credit_card() {
    let mut f = HashMap::new();
    f.insert("data".into(), serde_json::json!("1234567890123456")); // Fails Luhn
    let rec = DataRecord::new("s", "n", f);
    let detections = RedactionEngine::detect(&rec);
    assert!(!detections.iter().any(|d| d.pii_type == crypto::redaction::PiiType::CreditCard),
        "Should not detect invalid Luhn number as credit card");
}

#[test]
fn test_pii_number_not_phone() {
    let mut f = HashMap::new();
    f.insert("amount".into(), serde_json::json!("$1,234,567.89")); // Currency, not phone
    let rec = DataRecord::new("s", "n", f);
    let detections = RedactionEngine::detect(&rec);
    assert!(!detections.iter().any(|d| d.pii_type == crypto::redaction::PiiType::Phone),
        "Currency amounts should not be detected as phone numbers");
}

// ═══════════════════════════════════════════════════════════
// ENCRYPTION EDGE CASES
// ═══════════════════════════════════════════════════════════

#[test]
fn test_encrypt_empty_string() {
    let enc = FieldEncryptor::new("key");
    let encrypted = enc.encrypt_field("f", "");
    let decrypted = enc.decrypt_field("f", &encrypted).unwrap();
    assert_eq!(decrypted, "");
}

#[test]
fn test_encrypt_unicode() {
    let enc = FieldEncryptor::new("key");
    let original = "こんにちは世界";
    let encrypted = enc.encrypt_field("f", original);
    let decrypted = enc.decrypt_field("f", &encrypted).unwrap();
    assert_eq!(decrypted, original);
}

#[test]
fn test_decrypt_non_encrypted() {
    let enc = FieldEncryptor::new("key");
    let result = enc.decrypt_field("f", "plain text");
    assert!(result.is_err(), "Should fail on non-encrypted value");
}

// ═══════════════════════════════════════════════════════════
// FILE FORMAT EDGE CASES
// ═══════════════════════════════════════════════════════════

#[test]
fn test_adat_empty_file_roundtrip() {
    let mut buf = Vec::new();
    let writer = AdatWriter::new(&mut buf);
    writer.finish().unwrap();

    let cursor = std::io::Cursor::new(buf);
    let mut reader = AdatReader::open(cursor).unwrap();
    assert_eq!(reader.header().record_count, 0);
    assert_eq!(reader.header().schema_count, 0);
}

#[test]
fn test_adat_invalid_magic() {
    let buf = vec![0xFF; 64]; // Invalid magic bytes
    let cursor = std::io::Cursor::new(buf);
    let result = AdatReader::open(cursor);
    assert!(result.is_err());
}

#[test]
fn test_adat_truncated_file() {
    let buf = vec![0x41, 0x44, 0x41, 0x54]; // Just magic, no rest
    let cursor = std::io::Cursor::new(buf);
    let result = AdatReader::open(cursor);
    assert!(result.is_err(), "Truncated file should fail");
}

// ═══════════════════════════════════════════════════════════
// STORE EDGE CASES
// ═══════════════════════════════════════════════════════════

#[test]
fn test_query_empty_store() {
    let store = DataStore::new();
    let qe = QueryEngine::new(&store);
    let result = qe.query("nonexistent", &[], None, 0);
    assert_eq!(result.total_matched, 0);
    assert_eq!(result.total_scanned, 0);
}

#[test]
fn test_quality_empty_node() {
    let store = DataStore::new();
    let score = QualityEngine::score(&store, "empty");
    assert_eq!(score.score, 0);
}

#[test]
fn test_search_no_matches() {
    let mut store = DataStore::new();
    let mut f = HashMap::new();
    f.insert("name".into(), serde_json::json!("Alice"));
    store.add_record(DataRecord::new("s", "n", f));
    let qe = QueryEngine::new(&store);
    let result = qe.search("zzzznonexistentzzzz", None);
    assert_eq!(result.total_matched, 0);
}
