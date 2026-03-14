//! Stress tests — performance and scale validation.
//! Modeled after AgenticMemory's benchmark patterns.

use agentic_data::*;
use std::collections::HashMap;
use std::time::Instant;

// ═══════════════════════════════════════════════════════════
// INGEST THROUGHPUT
// ═══════════════════════════════════════════════════════════

#[test]
fn stress_ingest_10k_csv_records() {
    let mut data = String::from("id,name,value,category,score\n");
    for i in 0..10_000 {
        data.push_str(&format!("{},item_{},{},cat_{},{:.2}\n", i, i, i * 7, i % 10, i as f64 * 0.01));
    }

    let mut store = DataStore::new();
    let mut engine = IngestEngine::new(&mut store);
    let start = Instant::now();
    let result = engine.ingest_string(&data, "stress_csv").unwrap();
    let elapsed = start.elapsed();

    assert_eq!(result.records_added, 10_000);
    assert_eq!(result.parse_errors, 0);
    eprintln!("[stress] Ingested 10K CSV records in {:?}", elapsed);
    assert!(elapsed.as_secs() < 5, "10K ingest should complete in <5s, took {:?}", elapsed);
}

#[test]
fn stress_ingest_5k_json_records() {
    let items: Vec<String> = (0..5_000)
        .map(|i| format!(r#"{{"id":{},"name":"item_{}","price":{:.2},"active":{}}}"#,
            i, i, i as f64 * 1.5, if i % 2 == 0 { "true" } else { "false" }))
        .collect();
    let data = format!("[{}]", items.join(","));

    let mut store = DataStore::new();
    let mut engine = IngestEngine::new(&mut store);
    let start = Instant::now();
    let result = engine.ingest_string(&data, "stress_json").unwrap();
    let elapsed = start.elapsed();

    assert_eq!(result.records_added, 5_000);
    eprintln!("[stress] Ingested 5K JSON records in {:?}", elapsed);
    assert!(elapsed.as_secs() < 5, "5K JSON ingest should complete in <5s");
}

// ═══════════════════════════════════════════════════════════
// QUERY THROUGHPUT
// ═══════════════════════════════════════════════════════════

#[test]
fn stress_query_filter_10k() {
    let mut store = DataStore::new();
    for i in 0..10_000 {
        let mut f = HashMap::new();
        f.insert("id".into(), serde_json::json!(i));
        f.insert("category".into(), serde_json::json!(format!("cat_{}", i % 50)));
        f.insert("value".into(), serde_json::json!(i as f64 * 0.5));
        store.add_record(DataRecord::new("s", "items", f));
    }

    let qe = QueryEngine::new(&store);
    let start = Instant::now();
    let result = qe.query("items", &[
        engine::QueryFilter {
            field: "category".into(),
            op: engine::FilterOp::Eq,
            value: serde_json::json!("cat_7"),
        }
    ], None, 0);
    let elapsed = start.elapsed();

    assert_eq!(result.total_matched, 200); // 10000/50 = 200
    assert_eq!(result.total_scanned, 10_000);
    eprintln!("[stress] Filtered 10K records in {:?} ({} matches)", elapsed, result.total_matched);
    assert!(elapsed.as_millis() < 500, "Filter should complete in <500ms");
}

#[test]
fn stress_search_10k() {
    let mut store = DataStore::new();
    for i in 0..10_000 {
        let mut f = HashMap::new();
        f.insert("name".into(), serde_json::json!(format!("product_{}", i)));
        f.insert("desc".into(), serde_json::json!(if i == 4242 { "the special unicorn item" } else { "regular item" }));
        store.add_record(DataRecord::new("s", "products", f));
    }

    let qe = QueryEngine::new(&store);
    let start = Instant::now();
    let result = qe.search("unicorn", None);
    let elapsed = start.elapsed();

    assert_eq!(result.total_matched, 1);
    eprintln!("[stress] Full-text search 10K records in {:?}", elapsed);
    assert!(elapsed.as_millis() < 500, "Search should complete in <500ms");
}

// ═══════════════════════════════════════════════════════════
// QUALITY SCORING THROUGHPUT
// ═══════════════════════════════════════════════════════════

#[test]
fn stress_quality_5k_records() {
    let mut store = DataStore::new();
    for i in 0..5_000 {
        let mut f = HashMap::new();
        f.insert("id".into(), serde_json::json!(i));
        f.insert("value".into(), serde_json::json!(i as f64 * 0.1));
        f.insert("optional".into(), if i % 5 == 0 { serde_json::Value::Null } else { serde_json::json!("data") });
        store.add_record(DataRecord::new("s", "quality_test", f));
    }

    let start = Instant::now();
    let score = QualityEngine::score(&store, "quality_test");
    let elapsed = start.elapsed();

    eprintln!("[stress] Quality score for 5K records: {}/100 in {:?}", score.score, elapsed);
    assert!(score.score > 50);
    assert!(elapsed.as_millis() < 500, "Quality scoring should complete in <500ms");
}

#[test]
fn stress_anomaly_detection_5k() {
    let mut store = DataStore::new();
    for i in 0..5_000 {
        let mut f = HashMap::new();
        let value = if i == 3333 { 999_999.0 } else { 50.0 + (i as f64 % 20.0) };
        f.insert("metric".into(), serde_json::json!(value));
        store.add_record(DataRecord::new("s", "anomaly_test", f));
    }

    let start = Instant::now();
    let anomalies = QualityEngine::detect_anomalies(&store, "anomaly_test");
    let elapsed = start.elapsed();

    eprintln!("[stress] Anomaly detection for 5K records: {} anomalies in {:?}", anomalies.len(), elapsed);
    assert!(!anomalies.is_empty(), "Should detect the 999999 outlier");
    assert!(elapsed.as_millis() < 1000, "Anomaly detection should complete in <1s");
}

// ═══════════════════════════════════════════════════════════
// SPATIAL INDEX THROUGHPUT
// ═══════════════════════════════════════════════════════════

#[test]
fn stress_spatial_index_1k_points() {
    let mut idx = index::SpatialIndex::new();
    for i in 0..1_000 {
        let lat = 30.0 + (i as f64 % 20.0);
        let lng = -120.0 + (i as f64 % 50.0);
        idx.add(&format!("pt_{}", i), GeoPoint::new(lat, lng));
    }

    let start = Instant::now();
    let nearby = idx.within_radius(&GeoPoint::new(40.0, -100.0), 500_000.0);
    let elapsed = start.elapsed();

    eprintln!("[stress] Spatial query over 1K points: {} results in {:?}", nearby.len(), elapsed);
    assert!(elapsed.as_millis() < 200);
}

// ═══════════════════════════════════════════════════════════
// .ADAT FILE FORMAT THROUGHPUT
// ═══════════════════════════════════════════════════════════

#[test]
fn stress_adat_roundtrip_1k_records() {
    let mut buf = Vec::new();
    let mut writer = AdatWriter::new(&mut buf);

    writer.add_schema(UniversalSchema::new("stress_schema"));
    for i in 0..1_000 {
        let mut f = HashMap::new();
        f.insert("id".into(), serde_json::json!(i));
        f.insert("data".into(), serde_json::json!(format!("record_{}_with_some_content", i)));
        writer.add_record(DataRecord::new("s", "stress", f));
    }

    let start = Instant::now();
    writer.finish().unwrap();
    let write_elapsed = start.elapsed();

    let size = buf.len();
    let start = Instant::now();
    let cursor = std::io::Cursor::new(buf);
    let mut reader = AdatReader::open(cursor).unwrap();
    let records = reader.read_records().unwrap();
    let read_elapsed = start.elapsed();

    assert_eq!(records.len(), 1_000);
    eprintln!("[stress] .adat write: {:?}, read: {:?}, size: {} bytes ({:.1} bytes/record)",
        write_elapsed, read_elapsed, size, size as f64 / 1000.0);
    assert!(write_elapsed.as_secs() < 2);
    assert!(read_elapsed.as_secs() < 2);
}

// ═══════════════════════════════════════════════════════════
// PII SCANNING THROUGHPUT
// ═══════════════════════════════════════════════════════════

#[test]
fn stress_pii_scan_1k_records() {
    let mut store = DataStore::new();
    for i in 0..1_000 {
        let mut f = HashMap::new();
        f.insert("name".into(), serde_json::json!(format!("User {}", i)));
        f.insert("email".into(), serde_json::json!(format!("user{}@company.com", i)));
        f.insert("notes".into(), serde_json::json!("Normal business notes"));
        store.add_record(DataRecord::new("s", "pii_test", f));
    }

    let start = Instant::now();
    let mut total_pii = 0;
    for rec in store.active_records() {
        total_pii += RedactionEngine::detect(rec).len();
    }
    let elapsed = start.elapsed();

    eprintln!("[stress] PII scan 1K records: {} detections in {:?}", total_pii, elapsed);
    assert_eq!(total_pii, 1_000, "Should detect 1000 emails"); // Each record has an email
    assert!(elapsed.as_millis() < 500, "PII scan should complete in <500ms");
}

// ═══════════════════════════════════════════════════════════
// ENCRYPTION THROUGHPUT
// ═══════════════════════════════════════════════════════════

#[test]
fn stress_encrypt_decrypt_1k() {
    let enc = FieldEncryptor::new("stress-test-key");
    let start = Instant::now();
    for i in 0..1_000 {
        let value = format!("sensitive-data-{}-with-extra-content", i);
        let encrypted = enc.encrypt_field("field", &value);
        let decrypted = enc.decrypt_field("field", &encrypted).unwrap();
        assert_eq!(decrypted, value);
    }
    let elapsed = start.elapsed();
    eprintln!("[stress] 1K encrypt+decrypt cycles in {:?}", elapsed);
    assert!(elapsed.as_secs() < 2);
}
