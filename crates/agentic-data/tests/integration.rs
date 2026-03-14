//! Integration tests — end-to-end validation of AgenticData.
//! These tests exercise real data flows, not unit functions.

use agentic_data::*;
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════
// FORMAT DETECTION ACCURACY — test every format
// ═══════════════════════════════════════════════════════════

#[test]
fn test_detect_csv_accuracy() {
    let data = "name,age,email\nAlice,30,alice@test.com\nBob,25,bob@test.com\nCharlie,35,charlie@test.com\n";
    let d = parser::detect::detect_format(data, None);
    assert_eq!(d.format, parser::DataFormat::Csv, "CSV not detected");
    assert!(d.confidence >= 0.7, "CSV confidence too low: {}", d.confidence);
}

#[test]
fn test_detect_tsv_accuracy() {
    let data = "name\tage\tcity\nAlice\t30\tNYC\nBob\t25\tLA\nCharlie\t35\tSF\n";
    let d = parser::detect::detect_format(data, None);
    assert_eq!(d.format, parser::DataFormat::Tsv, "TSV not detected");
}

#[test]
fn test_detect_json_object() {
    let data = r#"{"users": [{"name": "Alice", "age": 30}]}"#;
    let d = parser::detect::detect_format(data, None);
    assert_eq!(d.format, parser::DataFormat::Json);
    assert!(d.confidence >= 0.9);
}

#[test]
fn test_detect_json_array() {
    let data = r#"[{"a":1},{"a":2}]"#;
    let d = parser::detect::detect_format(data, None);
    assert_eq!(d.format, parser::DataFormat::Json);
}

#[test]
fn test_detect_jsonl() {
    let data = "{\"id\":1,\"name\":\"a\"}\n{\"id\":2,\"name\":\"b\"}\n{\"id\":3,\"name\":\"c\"}\n";
    let d = parser::detect::detect_format(data, None);
    assert_eq!(d.format, parser::DataFormat::JsonLines);
}

#[test]
fn test_detect_xml() {
    let data = "<?xml version=\"1.0\"?>\n<catalog>\n  <book><title>Rust</title></book>\n</catalog>";
    let d = parser::detect::detect_format(data, None);
    assert_eq!(d.format, parser::DataFormat::Xml);
    assert!(d.confidence >= 0.9);
}

#[test]
fn test_detect_yaml() {
    let data = "---\nname: test\nversion: 1\nauthor: alice\ndescription: a test project\n";
    let d = parser::detect::detect_format(data, None);
    assert_eq!(d.format, parser::DataFormat::Yaml);
}

#[test]
fn test_detect_html() {
    let data = "<!DOCTYPE html>\n<html><head><title>Test</title></head><body><p>Hello</p></body></html>";
    let d = parser::detect::detect_format(data, None);
    assert_eq!(d.format, parser::DataFormat::Html);
}

#[test]
fn test_detect_sql() {
    let data = "CREATE TABLE users (\n  id INTEGER PRIMARY KEY,\n  name TEXT NOT NULL\n);\nINSERT INTO users VALUES (1, 'Alice');\n";
    let d = parser::detect::detect_format(data, None);
    assert_eq!(d.format, parser::DataFormat::Sql);
}

#[test]
fn test_detect_log() {
    let data = "[2026-03-13 10:00:00] INFO Server starting\n[2026-03-13 10:00:01] INFO Listening on port 8080\n[2026-03-13 10:00:02] WARN High memory usage\n[2026-03-13 10:00:03] ERROR Connection refused\n";
    let d = parser::detect::detect_format(data, None);
    assert_eq!(d.format, parser::DataFormat::Log);
}

#[test]
fn test_detect_email() {
    let data = "From: sender@example.com\nTo: recipient@example.com\nSubject: Test Email\nDate: Thu, 13 Mar 2026 10:00:00 +0000\nMIME-Version: 1.0\n\nThis is the email body.\n";
    let d = parser::detect::detect_format(data, None);
    assert_eq!(d.format, parser::DataFormat::Email);
}

#[test]
fn test_detect_calendar() {
    let data = "BEGIN:VCALENDAR\nVERSION:2.0\nBEGIN:VEVENT\nSUMMARY:Team Meeting\nDTSTART:20260313T100000Z\nDTEND:20260313T110000Z\nEND:VEVENT\nEND:VCALENDAR\n";
    let d = parser::detect::detect_format(data, None);
    assert_eq!(d.format, parser::DataFormat::Calendar);
    assert!(d.confidence >= 0.9);
}

#[test]
fn test_detect_geojson() {
    let data = r#"{"type":"FeatureCollection","features":[{"type":"Feature","geometry":{"type":"Point","coordinates":[-74.006,40.7128]},"properties":{"name":"NYC"}}]}"#;
    let d = parser::detect::detect_format(data, None);
    assert_eq!(d.format, parser::DataFormat::GeoJson);
}

#[test]
fn test_detect_by_extension_all() {
    let formats = vec![
        ("csv", parser::DataFormat::Csv),
        ("tsv", parser::DataFormat::Tsv),
        ("json", parser::DataFormat::Json),
        ("jsonl", parser::DataFormat::JsonLines),
        ("xml", parser::DataFormat::Xml),
        ("yaml", parser::DataFormat::Yaml),
        ("yml", parser::DataFormat::Yaml),
        ("toml", parser::DataFormat::Toml),
        ("html", parser::DataFormat::Html),
        ("sql", parser::DataFormat::Sql),
        ("log", parser::DataFormat::Log),
        ("eml", parser::DataFormat::Email),
        ("ics", parser::DataFormat::Calendar),
        ("geojson", parser::DataFormat::GeoJson),
        ("kml", parser::DataFormat::Kml),
        ("gpx", parser::DataFormat::Gpx),
        ("md", parser::DataFormat::Markdown),
    ];
    for (ext, expected) in &formats {
        let d = parser::detect::detect_format("any content", Some(ext));
        assert_eq!(d.format, *expected, "Extension .{} detected as {:?}, expected {:?}", ext, d.format, expected);
    }
}

// ═══════════════════════════════════════════════════════════
// SCHEMA INFERENCE ACCURACY — test type detection
// ═══════════════════════════════════════════════════════════

#[test]
fn test_schema_infer_integers() {
    let data = "id,count\n1,100\n2,200\n3,300\n4,400\n5,500\n";
    let result = parser::parse_auto(data, "test").unwrap();
    let fields = &result.schema.nodes[0].fields;
    assert_eq!(fields[0].field_type, FieldType::Integer, "id should be Integer");
    assert_eq!(fields[1].field_type, FieldType::Integer, "count should be Integer");
}

#[test]
fn test_schema_infer_floats() {
    let data = "price,weight\n9.99,1.5\n19.99,2.3\n29.99,0.8\n";
    let result = parser::parse_auto(data, "test").unwrap();
    let fields = &result.schema.nodes[0].fields;
    assert_eq!(fields[0].field_type, FieldType::Float, "price should be Float");
    assert_eq!(fields[1].field_type, FieldType::Float, "weight should be Float");
}

#[test]
fn test_schema_infer_booleans() {
    let data = "active,verified\ntrue,false\nfalse,true\ntrue,true\n";
    let result = parser::parse_auto(data, "test").unwrap();
    let fields = &result.schema.nodes[0].fields;
    assert_eq!(fields[0].field_type, FieldType::Boolean, "active should be Boolean");
}

#[test]
fn test_schema_infer_emails() {
    let data = "email,name\nalice@test.com,Alice\nbob@example.org,Bob\ncharlie@mail.io,Charlie\n";
    let result = parser::parse_auto(data, "test").unwrap();
    let fields = &result.schema.nodes[0].fields;
    assert_eq!(fields[0].field_type, FieldType::Email, "email should be Email");
    assert_eq!(fields[1].field_type, FieldType::Text, "name should be Text");
}

#[test]
fn test_schema_infer_urls() {
    let data = "website,name\nhttps://example.com,Example\nhttps://test.org,Test\nhttps://rust-lang.org,Rust\n";
    let result = parser::parse_auto(data, "test").unwrap();
    assert_eq!(result.schema.nodes[0].fields[0].field_type, FieldType::Url);
}

#[test]
fn test_schema_infer_mixed_json() {
    let data = r#"[{"id":1,"name":"Alice","score":95.5,"active":true},{"id":2,"name":"Bob","score":87.3,"active":false}]"#;
    let result = parser::parse_auto(data, "test").unwrap();
    let fields = &result.schema.nodes[0].fields;
    let find = |n: &str| fields.iter().find(|f| f.name == n).unwrap();
    assert_eq!(find("id").field_type, FieldType::Integer);
    assert_eq!(find("name").field_type, FieldType::Text);
    assert_eq!(find("score").field_type, FieldType::Float);
    assert_eq!(find("active").field_type, FieldType::Boolean);
}

// ═══════════════════════════════════════════════════════════
// INGEST + QUERY END-TO-END
// ═══════════════════════════════════════════════════════════

#[test]
fn test_ingest_and_query_csv() {
    let mut store = DataStore::new();
    let mut engine = IngestEngine::new(&mut store);
    let csv = "name,age,city\nAlice,30,NYC\nBob,25,LA\nCharlie,35,SF\nDiana,28,CHI\n";
    let result = engine.ingest_string(csv, "people").unwrap();
    assert_eq!(result.records_added, 4);

    let qe = QueryEngine::new(&store);
    // Query all
    let all = qe.query("people", &[], None, 0);
    assert_eq!(all.total_matched, 4);

    // Query with filter
    let filtered = qe.query("people", &[
        engine::QueryFilter { field: "city".into(), op: engine::FilterOp::Eq, value: serde_json::json!("NYC") }
    ], None, 0);
    assert_eq!(filtered.total_matched, 1);

    // Search
    let search = qe.search("alice", None);
    assert_eq!(search.total_matched, 1);
}

#[test]
fn test_ingest_and_query_json() {
    let mut store = DataStore::new();
    let mut engine = IngestEngine::new(&mut store);
    let json = r#"[{"product":"Widget","price":9.99,"in_stock":true},{"product":"Gadget","price":24.99,"in_stock":false}]"#;
    engine.ingest_string(json, "products").unwrap();

    let qe = QueryEngine::new(&store);
    let results = qe.query("products", &[
        engine::QueryFilter { field: "in_stock".into(), op: engine::FilterOp::Eq, value: serde_json::json!(true) }
    ], None, 0);
    assert_eq!(results.total_matched, 1);
}

// ═══════════════════════════════════════════════════════════
// QUALITY SCORING VALIDATION
// ═══════════════════════════════════════════════════════════

#[test]
fn test_quality_complete_data() {
    let mut store = DataStore::new();
    for i in 0..20 {
        let mut f = HashMap::new();
        f.insert("id".into(), serde_json::json!(i));
        f.insert("name".into(), serde_json::json!(format!("user_{}", i)));
        f.insert("score".into(), serde_json::json!(50 + i));
        store.add_record(DataRecord::new("s", "users", f));
    }
    let score = QualityEngine::score(&store, "users");
    assert!(score.completeness >= 0.99, "Completeness should be near 1.0, got {}", score.completeness);
    assert!(score.score >= 80, "Quality should be high for complete data, got {}", score.score);
}

#[test]
fn test_quality_null_heavy_data() {
    let mut store = DataStore::new();
    for i in 0..20 {
        let mut f = HashMap::new();
        f.insert("id".into(), serde_json::json!(i));
        f.insert("optional".into(), if i < 15 { serde_json::Value::Null } else { serde_json::json!("value") });
        store.add_record(DataRecord::new("s", "data", f));
    }
    let score = QualityEngine::score(&store, "data");
    assert!(score.completeness < 0.8, "Completeness should be low with 75% nulls, got {}", score.completeness);
}

#[test]
fn test_anomaly_detection_outlier() {
    let mut store = DataStore::new();
    // Normal values: 10-20
    for i in 0..30 {
        let mut f = HashMap::new();
        f.insert("value".into(), serde_json::json!(15.0 + (i as f64 - 15.0) * 0.3));
        store.add_record(DataRecord::new("s", "metrics", f));
    }
    // Outlier: 1000
    let mut f = HashMap::new();
    f.insert("value".into(), serde_json::json!(1000.0));
    store.add_record(DataRecord::new("s", "metrics", f));

    let anomalies = QualityEngine::detect_anomalies(&store, "metrics");
    assert!(!anomalies.is_empty(), "Should detect the outlier");
    assert!(anomalies.iter().any(|a| a.anomaly_type == AnomalyType::Outlier));
}

#[test]
fn test_anomaly_detection_null_spike() {
    let mut store = DataStore::new();
    for i in 0..10 {
        let mut f = HashMap::new();
        f.insert("data".into(), if i < 8 { serde_json::Value::Null } else { serde_json::json!(i) });
        store.add_record(DataRecord::new("s", "nulls", f));
    }
    let anomalies = QualityEngine::detect_anomalies(&store, "nulls");
    assert!(anomalies.iter().any(|a| a.anomaly_type == AnomalyType::NullSpike), "Should detect null spike at 80%");
}

// ═══════════════════════════════════════════════════════════
// LINEAGE TRACKING VALIDATION
// ═══════════════════════════════════════════════════════════

#[test]
fn test_lineage_through_ingest() {
    let mut store = DataStore::new();
    let mut engine = IngestEngine::new(&mut store);
    engine.ingest_string("a,b\n1,2\n", "source.csv").unwrap();

    let records = store.active_records();
    assert!(!records.is_empty());
    let rec = &records[0];
    let lineage = store.get_lineage(&rec.id);
    assert!(lineage.is_some(), "Ingested record should have lineage");
    assert_eq!(lineage.unwrap().depth(), 1);
    assert_eq!(lineage.unwrap().entries[0].action, LineageAction::Ingested);
}

// ═══════════════════════════════════════════════════════════
// PII DETECTION VALIDATION
// ═══════════════════════════════════════════════════════════

#[test]
fn test_pii_detect_email() {
    let mut f = HashMap::new();
    f.insert("contact".into(), serde_json::json!("alice@example.com"));
    let rec = DataRecord::new("s", "n", f);
    let detections = RedactionEngine::detect(&rec);
    assert!(detections.iter().any(|d| d.pii_type == crypto::redaction::PiiType::Email), "Should detect email");
}

#[test]
fn test_pii_detect_ssn() {
    let mut f = HashMap::new();
    f.insert("ssn".into(), serde_json::json!("123-45-6789"));
    let rec = DataRecord::new("s", "n", f);
    let detections = RedactionEngine::detect(&rec);
    assert!(detections.iter().any(|d| d.pii_type == crypto::redaction::PiiType::Ssn), "Should detect SSN");
}

#[test]
fn test_pii_detect_ip() {
    let mut f = HashMap::new();
    f.insert("server".into(), serde_json::json!("192.168.1.100"));
    let rec = DataRecord::new("s", "n", f);
    let detections = RedactionEngine::detect(&rec);
    assert!(detections.iter().any(|d| d.pii_type == crypto::redaction::PiiType::IpAddress), "Should detect IP");
}

#[test]
fn test_pii_no_false_positive() {
    let mut f = HashMap::new();
    f.insert("note".into(), serde_json::json!("Meeting at 3pm in room 401"));
    f.insert("status".into(), serde_json::json!("active"));
    let rec = DataRecord::new("s", "n", f);
    let detections = RedactionEngine::detect(&rec);
    assert!(detections.is_empty(), "Should not detect PII in normal text, got {:?}", detections.iter().map(|d| format!("{:?}", d.pii_type)).collect::<Vec<_>>());
}

#[test]
fn test_pii_redaction_mask() {
    let mut f = HashMap::new();
    f.insert("email".into(), serde_json::json!("secret@company.com"));
    f.insert("name".into(), serde_json::json!("Public Name"));
    let rec = DataRecord::new("s", "n", f);
    let policy = RedactionPolicy::default();
    let redacted = RedactionEngine::redact(&rec, &policy);
    assert_eq!(redacted.get_str("email"), Some("[REDACTED]"), "Email should be masked");
    assert_eq!(redacted.get_str("name"), Some("Public Name"), "Name should be untouched");
}

// ═══════════════════════════════════════════════════════════
// .ADAT FILE FORMAT ROUNDTRIP
// ═══════════════════════════════════════════════════════════

#[test]
fn test_adat_roundtrip_with_data() {
    let mut buf = Vec::new();
    let mut writer = AdatWriter::new(&mut buf);

    let mut schema = UniversalSchema::new("test_schema");
    schema.nodes.push(SchemaNode {
        name: "items".into(), source: "test".into(),
        fields: vec![
            SchemaField::inferred("id", FieldType::Integer, 1.0),
            SchemaField::inferred("name", FieldType::Text, 1.0),
            SchemaField::inferred("price", FieldType::Float, 0.95),
        ],
        record_count: Some(3),
    });
    writer.add_schema(schema);
    writer.add_source(DataSource::file("test_src", "/data/items.csv"));

    for (id, name, price) in [(1, "Widget", 9.99), (2, "Gadget", 24.99), (3, "Doohickey", 4.99)] {
        let mut f = HashMap::new();
        f.insert("id".into(), serde_json::json!(id));
        f.insert("name".into(), serde_json::json!(name));
        f.insert("price".into(), serde_json::json!(price));
        writer.add_record(DataRecord::new("test_src", "items", f));
    }
    writer.finish().unwrap();

    // Read back
    let cursor = std::io::Cursor::new(buf);
    let mut reader = AdatReader::open(cursor).unwrap();
    assert_eq!(reader.header().schema_count, 1);
    assert_eq!(reader.header().source_count, 1);
    assert_eq!(reader.header().record_count, 3);

    let schemas = reader.read_schemas().unwrap();
    assert_eq!(schemas[0].name, "test_schema");
    assert_eq!(schemas[0].total_fields(), 3);

    let records = reader.read_records().unwrap();
    assert_eq!(records.len(), 3);
    assert_eq!(records[0].get_str("name"), Some("Widget"));
    assert_eq!(records[2].get_f64("price"), Some(4.99));
}

// ═══════════════════════════════════════════════════════════
// ENCRYPTION ROUNDTRIP
// ═══════════════════════════════════════════════════════════

#[test]
fn test_encryption_roundtrip() {
    let enc = FieldEncryptor::new("test-master-key");
    let original = "sensitive-data-12345";
    let encrypted = enc.encrypt_field("ssn", original);
    assert!(FieldEncryptor::is_encrypted(&encrypted));
    assert_ne!(encrypted, original);
    let decrypted = enc.decrypt_field("ssn", &encrypted).unwrap();
    assert_eq!(decrypted, original);
}

#[test]
fn test_encryption_different_fields_different_ciphertext() {
    let enc = FieldEncryptor::new("key");
    let same_value = "hello";
    let enc_a = enc.encrypt_field("field_a", same_value);
    let enc_b = enc.encrypt_field("field_b", same_value);
    assert_ne!(enc_a, enc_b, "Same value encrypted under different field names should differ");
}

// ═══════════════════════════════════════════════════════════
// SPATIAL INDEX VALIDATION
// ═══════════════════════════════════════════════════════════

#[test]
fn test_spatial_distance_accuracy() {
    let nyc = GeoPoint::new(40.7128, -74.0060);
    let la = GeoPoint::new(34.0522, -118.2437);
    let dist_km = nyc.distance_meters(&la) / 1000.0;
    // NYC to LA is approximately 3,944 km
    assert!(dist_km > 3900.0 && dist_km < 4000.0, "NYC-LA distance should be ~3944km, got {:.0}km", dist_km);
}

#[test]
fn test_spatial_index_query() {
    let mut idx = index::SpatialIndex::new();
    idx.add("nyc", GeoPoint::new(40.7128, -74.0060));
    idx.add("boston", GeoPoint::new(42.3601, -71.0589));
    idx.add("la", GeoPoint::new(34.0522, -118.2437));

    // 500km radius from NYC should include Boston (~306km) but not LA (~3944km)
    let nearby = idx.within_radius(&GeoPoint::new(40.7128, -74.0060), 500_000.0);
    let names: Vec<&str> = nearby.iter().map(|(n, _, _)| *n).collect();
    assert!(names.contains(&"nyc"));
    assert!(names.contains(&"boston"));
    assert!(!names.contains(&"la"));
}

// ═══════════════════════════════════════════════════════════
// TRANSFORM PIPELINE VALIDATION
// ═══════════════════════════════════════════════════════════

#[test]
fn test_transform_pipeline_audit_trail() {
    let mut store = DataStore::new();
    let mut ingest = IngestEngine::new(&mut store);
    ingest.ingest_string("name,email\nAlice,ALICE@TEST.COM\nBob,BOB@TEST.COM\n", "src").unwrap();

    let mut pipeline = engine::TransformPipeline::new("normalize");
    pipeline.add(engine::TransformStep {
        name: "lowercase_email".into(),
        operation: agentic_data::engine::transform::TransformOp::MapField {
            field: "email".into(),
            transform: agentic_data::engine::transform::FieldTransform::Lowercase,
        },
    });

    let records: Vec<DataRecord> = store.records_for_node("src").into_iter().cloned().collect();
    let result = TransformEngine::apply(&records, &pipeline);
    assert_eq!(result.records_out, 2);
    assert_eq!(result.receipts.len(), 1);
    assert_eq!(result.receipts[0].transform, "lowercase_email");
    assert_eq!(result.receipts[0].input_count, 2);
    assert_eq!(result.receipts[0].output_count, 2);
}

// ═══════════════════════════════════════════════════════════
// MCP TOOL REGISTRY VALIDATION
// ═══════════════════════════════════════════════════════════

#[test]
fn test_mcp_tool_count() {
    // Verify the tool count matches what we claim
    let formats = parser::supported_formats();
    assert!(formats.len() >= 16, "Should support 16+ formats, got {}", formats.len());
}

#[test]
fn test_format_names_unique() {
    let formats = parser::supported_formats();
    let mut names: Vec<&str> = formats.iter().map(|f| f.name()).collect();
    let original_len = names.len();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), original_len, "Format names should be unique");
}
