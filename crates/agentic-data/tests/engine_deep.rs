//! Deep engine tests — graph, session, consolidation, transform pipelines.

use agentic_data::*;
use agentic_data::types::SchemaEdgeType;
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════
// GRAPH ENGINE
// ═══════════════════════════════════════════════════════════

#[test]
fn test_graph_traversal_depth_limit() {
    let mut s = UniversalSchema::new("chain");
    s.nodes.push(SchemaNode { name: "a".into(), source: "s".into(), fields: vec![], record_count: None });
    s.nodes.push(SchemaNode { name: "b".into(), source: "s".into(), fields: vec![], record_count: None });
    s.nodes.push(SchemaNode { name: "c".into(), source: "s".into(), fields: vec![], record_count: None });
    s.edges.push(SchemaEdge { from: "a.id".into(), to: "b.id".into(), edge_type: SchemaEdgeType::References, confidence: 0.9 });
    s.edges.push(SchemaEdge { from: "b.id".into(), to: "c.id".into(), edge_type: SchemaEdgeType::References, confidence: 0.9 });

    let r0 = GraphEngine::traverse_schema(&s, "a", TraversalDirection::Forward, 0);
    assert_eq!(r0.visited.len(), 1); // Only start node at depth 0

    let r1 = GraphEngine::traverse_schema(&s, "a", TraversalDirection::Forward, 1);
    assert!(r1.visited.contains(&"b".to_string()));

    let r2 = GraphEngine::traverse_schema(&s, "a", TraversalDirection::Forward, 5);
    assert!(r2.visited.contains(&"c".to_string()));
}

#[test]
fn test_graph_centrality_hub() {
    let mut s = UniversalSchema::new("hub");
    for name in &["hub", "spoke1", "spoke2", "spoke3"] {
        s.nodes.push(SchemaNode { name: name.to_string(), source: "s".into(), fields: vec![], record_count: None });
    }
    for spoke in &["spoke1", "spoke2", "spoke3"] {
        s.edges.push(SchemaEdge { from: format!("{}.id", spoke), to: "hub.id".into(), edge_type: SchemaEdgeType::References, confidence: 0.9 });
    }
    let centrality = GraphEngine::node_centrality(&s);
    let hub_score = centrality.get("hub").copied().unwrap_or(0.0);
    let spoke_score = centrality.get("spoke1").copied().unwrap_or(0.0);
    assert!(hub_score > spoke_score, "Hub should have higher centrality than spokes");
}

#[test]
fn test_graph_relationship_discovery_fk() {
    let mut store = DataStore::new();
    let mut s = UniversalSchema::new("fk_test");
    s.nodes.push(SchemaNode { name: "users".into(), source: "db".into(),
        fields: vec![SchemaField::inferred("id", FieldType::Integer, 1.0)], record_count: None });
    s.nodes.push(SchemaNode { name: "orders".into(), source: "db".into(),
        fields: vec![SchemaField::inferred("id", FieldType::Integer, 1.0), SchemaField::inferred("user_id", FieldType::Integer, 1.0)], record_count: None });
    store.add_schema(s);

    let candidates = GraphEngine::discover_relationships(&store);
    assert!(!candidates.is_empty());
    assert!(candidates.iter().any(|c| c.from_field == "user_id" && c.to_field == "id" && c.confidence >= 0.8));
}

#[test]
fn test_graph_disconnected_components() {
    let mut s = UniversalSchema::new("disjoint");
    s.nodes.push(SchemaNode { name: "island_a".into(), source: "s".into(), fields: vec![], record_count: None });
    s.nodes.push(SchemaNode { name: "island_b".into(), source: "s".into(), fields: vec![], record_count: None });
    // No edges between them
    let components = GraphEngine::connected_components(&s);
    assert_eq!(components.len(), 2, "Two disconnected nodes should form 2 components");
}

// ═══════════════════════════════════════════════════════════
// SESSION MANAGER
// ═══════════════════════════════════════════════════════════

#[test]
fn test_session_operation_tracking() {
    let mut mgr = SessionManager::new();
    let sid = mgr.start("analyst");
    mgr.record_op(&sid, engine::OpType::Ingest, "sales.csv", "Loaded 1000 rows");
    mgr.record_op(&sid, engine::OpType::Query, "sales.csv", "Filtered by region");
    mgr.record_op(&sid, engine::OpType::QualityCheck, "sales.csv", "Score: 92");

    let session = mgr.get_active(&sid).unwrap();
    assert_eq!(session.operations.len(), 3);
    assert_eq!(session.records_created, 1);
    assert_eq!(session.queries_executed, 1);
    assert_eq!(session.sources_touched.len(), 1);
}

#[test]
fn test_session_multiple_users() {
    let mut mgr = SessionManager::new();
    let s1 = mgr.start("alice");
    let s2 = mgr.start("bob");
    assert_eq!(mgr.active_count(), 2);
    mgr.end(&s1);
    assert_eq!(mgr.active_count(), 1);
    assert_eq!(mgr.completed_count(), 1);
    mgr.end(&s2);
    assert_eq!(mgr.active_count(), 0);
    assert_eq!(mgr.completed_count(), 2);
}

#[test]
fn test_session_resume_empty() {
    let mgr = SessionManager::new();
    assert!(mgr.resume_context("nobody").is_none());
}

// ═══════════════════════════════════════════════════════════
// CONSOLIDATION
// ═══════════════════════════════════════════════════════════

#[test]
fn test_consolidation_no_dups() {
    let mut store = DataStore::new();
    let mut s = UniversalSchema::new("test");
    s.nodes.push(SchemaNode { name: "t".into(), source: "s".into(), fields: vec![], record_count: None });
    store.add_schema(s);
    for i in 0..5 {
        let mut f = HashMap::new();
        f.insert("id".into(), serde_json::json!(i));
        store.add_record(DataRecord::new("s", "t", f));
    }
    let report = ConsolidationEngine::consolidate(&mut store);
    assert_eq!(report.duplicates_removed, 0);
    assert_eq!(store.record_count(), 5);
}

#[test]
fn test_consolidation_full_report() {
    let mut store = DataStore::new();
    let mut s = UniversalSchema::new("valid");
    s.nodes.push(SchemaNode { name: "data".into(), source: "s".into(), fields: vec![], record_count: None });
    store.add_schema(s);

    for i in 0..10 {
        let mut f = HashMap::new();
        f.insert("v".into(), serde_json::json!(i));
        store.add_record(DataRecord::new("s", "data", f));
    }
    let report = ConsolidationEngine::consolidate(&mut store);
    assert!(report.quality_refreshed > 0);
}

// ═══════════════════════════════════════════════════════════
// TRANSFORM PIPELINE DEPTH
// ═══════════════════════════════════════════════════════════

#[test]
fn test_transform_dedup() {
    let mut records = Vec::new();
    for _ in 0..3 {
        let mut f = HashMap::new();
        f.insert("name".into(), serde_json::json!("Alice"));
        records.push(DataRecord::new("s", "n", f));
    }
    let mut f = HashMap::new();
    f.insert("name".into(), serde_json::json!("Bob"));
    records.push(DataRecord::new("s", "n", f));

    let mut pipeline = engine::TransformPipeline::new("dedup_test");
    pipeline.add(engine::TransformStep {
        name: "dedup".into(),
        operation: engine::transform::TransformOp::Dedup { field: "name".into() },
    });
    let result = TransformEngine::apply(&records, &pipeline);
    assert_eq!(result.records_out, 2); // Alice + Bob
    assert_eq!(result.records_dropped, 2); // 2 duplicate Alices
}

#[test]
fn test_transform_sort() {
    let mut records = Vec::new();
    for val in [30, 10, 20] {
        let mut f = HashMap::new();
        f.insert("score".into(), serde_json::json!(val));
        records.push(DataRecord::new("s", "n", f));
    }
    let mut pipeline = engine::TransformPipeline::new("sort_test");
    pipeline.add(engine::TransformStep {
        name: "sort_asc".into(),
        operation: engine::transform::TransformOp::Sort { field: "score".into(), ascending: true },
    });
    let result = TransformEngine::apply(&records, &pipeline);
    assert_eq!(result.records_out, 3);
}

#[test]
fn test_transform_multi_step_pipeline() {
    let mut records = Vec::new();
    for (name, age) in [("alice", 30), ("bob", 25), ("alice", 30)] {
        let mut f = HashMap::new();
        f.insert("name".into(), serde_json::json!(name));
        f.insert("age".into(), serde_json::json!(age));
        records.push(DataRecord::new("s", "n", f));
    }
    let mut pipeline = engine::TransformPipeline::new("multi");
    pipeline.add(engine::TransformStep { name: "upper".into(),
        operation: engine::transform::TransformOp::MapField { field: "name".into(), transform: engine::transform::FieldTransform::Uppercase } });
    pipeline.add(engine::TransformStep { name: "dedup".into(),
        operation: engine::transform::TransformOp::Dedup { field: "name".into() } });
    pipeline.add(engine::TransformStep { name: "drop_age".into(),
        operation: engine::transform::TransformOp::DropField { field: "age".into() } });

    let result = TransformEngine::apply(&records, &pipeline);
    assert_eq!(result.receipts.len(), 3);
    assert_eq!(result.records_out, 2); // ALICE + BOB after dedup
}

// ═══════════════════════════════════════════════════════════
// INDEX DEPTH
// ═══════════════════════════════════════════════════════════

#[test]
fn test_lineage_index_chain() {
    let mut idx = index::LineageIndex::new();
    idx.add_entry("field_a", types::LineageEntry { action: LineageAction::Ingested, source: "csv".into(), timestamp: 100, input_hash: "".into(), output_hash: "h1".into(), description: "".into() });
    idx.add_entry("field_b", types::LineageEntry { action: LineageAction::Derived, source: "field_a".into(), timestamp: 200, input_hash: "".into(), output_hash: "h2".into(), description: "".into() });

    let impact = idx.impact_of("field_a");
    assert!(impact.contains(&"field_b"));

    let sources = idx.sources_of("field_b");
    assert!(sources.contains(&"field_a"));
}

#[test]
fn test_schema_index_search_case_insensitive() {
    let mut idx = index::SchemaIndex::new();
    let mut s = UniversalSchema::new("test");
    s.nodes.push(SchemaNode { name: "users".into(), source: "db".into(),
        fields: vec![SchemaField::inferred("Email_Address", FieldType::Email, 0.95)], record_count: None });
    idx.add(&s);
    let results = idx.search_fields("email");
    assert_eq!(results.len(), 1);
}

#[test]
fn test_quality_index_trend() {
    let mut idx = index::QualityIndex::new();
    idx.add_score(QualityScore::compute("data", 0.5, 0.5, 0.5, 0.5, 0.5));
    idx.add_score(QualityScore::compute("data", 0.9, 0.9, 0.9, 0.9, 0.9));
    assert_eq!(idx.trend("data"), types::Trend::Improving);
}

// ═══════════════════════════════════════════════════════════
// ENCRYPTION DEPTH
// ═══════════════════════════════════════════════════════════

#[test]
fn test_key_manager_full_lifecycle() {
    let mut km = KeyManager::new();
    let _k1 = km.create_key("prod", "strong-passphrase-123");
    assert!(km.is_active("prod"));
    let _k2 = km.rotate_key("prod", "even-stronger-456").unwrap();
    let keys = km.list_keys();
    assert_eq!(keys[0].2, 2); // Version 2 after rotation
    km.deactivate_key("prod").unwrap();
    assert!(!km.is_active("prod"));
    assert_eq!(km.audit_log().len(), 3); // create + rotate + deactivate
}

#[test]
fn test_encrypt_record_selective() {
    let enc = FieldEncryptor::new("test-key");
    let mut fields = HashMap::new();
    fields.insert("ssn".into(), serde_json::json!("123-45-6789"));
    fields.insert("name".into(), serde_json::json!("Alice"));
    fields.insert("city".into(), serde_json::json!("NYC"));
    let mut record = DataRecord::new("s", "n", fields);

    enc.encrypt_record(&mut record, &["ssn"]);
    assert!(FieldEncryptor::is_encrypted(record.get_str("ssn").unwrap()));
    assert_eq!(record.get_str("name"), Some("Alice")); // Not encrypted
    assert_eq!(record.get_str("city"), Some("NYC")); // Not encrypted

    enc.decrypt_record(&mut record, &["ssn"]).unwrap();
    assert_eq!(record.get_str("ssn"), Some("123-45-6789"));
}
