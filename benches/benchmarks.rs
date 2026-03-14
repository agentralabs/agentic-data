//! Benchmarks for AgenticData core operations.
//! Run with: cargo bench

use std::collections::HashMap;
use std::time::Instant;

fn main() {
    println!("AgenticData Benchmarks");
    println!("======================\n");

    bench_format_detection();
    bench_csv_parse();
    bench_json_parse();
    bench_ingest();
    bench_query();
    bench_quality();
    bench_anomaly();
    bench_pii();
    bench_encrypt();
    bench_spatial();
    bench_adat_write();
    bench_adat_read();

    println!("\nAll benchmarks complete.");
}

fn bench_format_detection() {
    let samples = vec![
        ("CSV", "name,age\nAlice,30\nBob,25\n"),
        ("JSON", r#"{"key":"value","num":42}"#),
        ("XML", "<?xml version=\"1.0\"?><root><item/></root>"),
        ("SQL", "CREATE TABLE t (id INT);"),
        ("Email", "From: a@b.com\nSubject: Hi\n\nBody"),
    ];
    let start = Instant::now();
    let iterations = 10_000;
    for _ in 0..iterations {
        for (_, data) in &samples {
            let _ = agentic_data::parser::detect::detect_format(data, None);
        }
    }
    let elapsed = start.elapsed();
    let per_op = elapsed / (iterations * samples.len() as u32);
    println!("Format detection:  {:?}/op ({} ops)", per_op, iterations * samples.len() as u32);
}

fn bench_csv_parse() {
    let mut data = String::from("id,name,value\n");
    for i in 0..1000 { data.push_str(&format!("{},item_{},{}\n", i, i, i*3)); }
    let start = Instant::now();
    for _ in 0..100 {
        let _ = agentic_data::parser::csv_parser::parse(&data, "bench", b',');
    }
    let elapsed = start.elapsed();
    println!("CSV parse 1K rows: {:?}/op (100 iterations)", elapsed / 100);
}

fn bench_json_parse() {
    let items: Vec<String> = (0..1000)
        .map(|i| format!(r#"{{"id":{},"name":"item_{}"}}"#, i, i))
        .collect();
    let data = format!("[{}]", items.join(","));
    let start = Instant::now();
    for _ in 0..100 {
        let _ = agentic_data::parser::json_parser::parse(&data, "bench");
    }
    let elapsed = start.elapsed();
    println!("JSON parse 1K objs: {:?}/op (100 iterations)", elapsed / 100);
}

fn bench_ingest() {
    let mut data = String::from("a,b,c\n");
    for i in 0..5000 { data.push_str(&format!("{},{},{}\n", i, i*2, i*3)); }
    let start = Instant::now();
    let mut store = agentic_data::DataStore::new();
    let mut engine = agentic_data::IngestEngine::new(&mut store);
    let _ = engine.ingest_string(&data, "bench");
    let elapsed = start.elapsed();
    println!("Ingest 5K records:  {:?}", elapsed);
}

fn bench_query() {
    let mut store = agentic_data::DataStore::new();
    for i in 0..10_000 {
        let mut f = HashMap::new();
        f.insert("id".into(), serde_json::json!(i));
        f.insert("cat".into(), serde_json::json!(format!("c{}", i % 100)));
        store.add_record(agentic_data::DataRecord::new("s", "bench", f));
    }
    let qe = agentic_data::QueryEngine::new(&store);
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = qe.query("bench", &[
            agentic_data::engine::QueryFilter { field: "cat".into(), op: agentic_data::engine::FilterOp::Eq, value: serde_json::json!("c42") }
        ], None, 0);
    }
    let elapsed = start.elapsed();
    println!("Query filter 10K:   {:?}/op (1000 iterations)", elapsed / 1000);
}

fn bench_quality() {
    let mut store = agentic_data::DataStore::new();
    for i in 0..5000 {
        let mut f = HashMap::new();
        f.insert("v".into(), serde_json::json!(i));
        store.add_record(agentic_data::DataRecord::new("s", "q", f));
    }
    let start = Instant::now();
    for _ in 0..100 {
        let _ = agentic_data::QualityEngine::score(&store, "q");
    }
    let elapsed = start.elapsed();
    println!("Quality score 5K:   {:?}/op (100 iterations)", elapsed / 100);
}

fn bench_anomaly() {
    let mut store = agentic_data::DataStore::new();
    for i in 0..5000 {
        let mut f = HashMap::new();
        f.insert("v".into(), serde_json::json!(50.0 + (i as f64 % 10.0)));
        store.add_record(agentic_data::DataRecord::new("s", "a", f));
    }
    let start = Instant::now();
    let _ = agentic_data::QualityEngine::detect_anomalies(&store, "a");
    let elapsed = start.elapsed();
    println!("Anomaly detect 5K:  {:?}", elapsed);
}

fn bench_pii() {
    let mut records = Vec::new();
    for i in 0..1000 {
        let mut f = HashMap::new();
        f.insert("email".into(), serde_json::json!(format!("u{}@co.com", i)));
        records.push(agentic_data::DataRecord::new("s", "p", f));
    }
    let start = Instant::now();
    let mut total = 0;
    for r in &records { total += agentic_data::RedactionEngine::detect(r).len(); }
    let elapsed = start.elapsed();
    println!("PII scan 1K:        {:?} ({} detections)", elapsed, total);
}

fn bench_encrypt() {
    let enc = agentic_data::FieldEncryptor::new("bench-key");
    let start = Instant::now();
    for i in 0..10_000 {
        let e = enc.encrypt_field("f", &format!("data-{}", i));
        let _ = enc.decrypt_field("f", &e);
    }
    let elapsed = start.elapsed();
    println!("Encrypt+decrypt:    {:?}/op (10K cycles)", elapsed / 10_000);
}

fn bench_spatial() {
    let mut idx = agentic_data::index::SpatialIndex::new();
    for i in 0..5000 {
        idx.add(&format!("p{}", i), agentic_data::GeoPoint::new(
            30.0 + (i as f64 % 20.0), -120.0 + (i as f64 % 50.0)
        ));
    }
    let start = Instant::now();
    for _ in 0..100 {
        let _ = idx.within_radius(&agentic_data::GeoPoint::new(40.0, -100.0), 500_000.0);
    }
    let elapsed = start.elapsed();
    println!("Spatial query 5K:   {:?}/op (100 iterations)", elapsed / 100);
}

fn bench_adat_write() {
    let mut buf = Vec::new();
    let mut writer = agentic_data::AdatWriter::new(&mut buf);
    writer.add_schema(agentic_data::UniversalSchema::new("bench"));
    for i in 0..5000 {
        let mut f = HashMap::new();
        f.insert("id".into(), serde_json::json!(i));
        f.insert("data".into(), serde_json::json!(format!("content_{}", i)));
        writer.add_record(agentic_data::DataRecord::new("s", "bench", f));
    }
    let start = Instant::now();
    writer.finish().unwrap();
    let elapsed = start.elapsed();
    println!(".adat write 5K:     {:?} ({} bytes)", elapsed, buf.len());
}

fn bench_adat_read() {
    let mut buf = Vec::new();
    let mut writer = agentic_data::AdatWriter::new(&mut buf);
    writer.add_schema(agentic_data::UniversalSchema::new("bench"));
    for i in 0..5000 {
        let mut f = HashMap::new();
        f.insert("id".into(), serde_json::json!(i));
        writer.add_record(agentic_data::DataRecord::new("s", "bench", f));
    }
    writer.finish().unwrap();

    let start = Instant::now();
    let cursor = std::io::Cursor::new(&buf);
    let mut reader = agentic_data::AdatReader::open(cursor).unwrap();
    let _ = reader.read_records().unwrap();
    let elapsed = start.elapsed();
    println!(".adat read 5K:      {:?}", elapsed);
}
