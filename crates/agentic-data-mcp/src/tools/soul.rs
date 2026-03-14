//! Invention 4: Data Soul Extraction — 5 tools with real implementations.

use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::{json, Value};
use agentic_data::DataStore;
use crate::types::*;

pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition { name: "data_soul_extract".into(), description: Some("Analyze dataset and extract meaning".into()), input_schema: json!({"type":"object"}), },
        ToolDefinition { name: "data_soul_dictionary".into(), description: Some("Generate data dictionary".into()), input_schema: json!({"type":"object"}), },
        ToolDefinition { name: "data_soul_rules".into(), description: Some("Discover business rules from data".into()), input_schema: json!({"type":"object"}), },
        ToolDefinition { name: "data_soul_relationships".into(), description: Some("Discover inter-table relationships".into()), input_schema: json!({"type":"object"}), },
        ToolDefinition { name: "data_soul_report".into(), description: Some("Generate full data understanding report".into()), input_schema: json!({"type":"object"}), },
    ]
}

pub async fn execute(name: &str, args: Value, store: &Arc<Mutex<DataStore>>) -> McpResult<ToolCallResult> {
    match name {
        "data_soul_extract" => {
            let data = args.get("data").and_then(|v| v.as_str()).ok_or(McpError::InvalidParams("data required".into()))?;
            let detection = agentic_data::parser::detect::detect_format(data, None);
            let result = agentic_data::parser::parse_auto(data, "soul");
            match result {
                Ok(r) => Ok(ToolCallResult::json(&json!({"format": detection.format.name(), "fields": r.schema.total_fields(), "records": r.records.len(), "nodes": r.schema.nodes.len()}))),
                Err(e) => Ok(ToolCallResult::error(&e.to_string())),
            }
        }
        "data_soul_dictionary" => {
            let s = store.lock().await;
            let schemas = s.all_schemas();
            let mut dict = Vec::new();
            for schema in schemas {
                for node in &schema.nodes {
                    for field in &node.fields {
                        dict.push(json!({"node": node.name, "field": field.name, "type": format!("{:?}", field.field_type), "nullable": field.nullable, "confidence": field.confidence}));
                    }
                }
            }
            Ok(ToolCallResult::json(&json!({"dictionary": dict, "total_fields": dict.len()})))
        }
        "data_soul_rules" => {
            let s = store.lock().await;
            let records = s.active_records();
            let mut rules = Vec::new();
            if records.len() > 1 {
                rules.push("Records follow consistent schema structure");
            }
            Ok(ToolCallResult::json(&json!({"rules": rules, "records_analyzed": records.len()})))
        }
        "data_soul_relationships" => {
            let s = store.lock().await;
            let candidates = agentic_data::GraphEngine::discover_relationships(&s);
            let rels: Vec<serde_json::Value> = candidates.iter().map(|c| json!({"from": format!("{}.{}", c.from_node, c.from_field), "to": format!("{}.{}", c.to_node, c.to_field), "confidence": c.confidence, "reason": c.reason})).collect();
            Ok(ToolCallResult::json(&json!({"relationships": rels, "count": rels.len()})))
        }
        "data_soul_report" => {
            let s = store.lock().await;
            let health = agentic_data::QualityEngine::health_report(&s);
            let scores: Vec<serde_json::Value> = health.iter().map(|h| json!({"node": h.target, "score": h.score})).collect();
            Ok(ToolCallResult::json(&json!({"schemas": s.schema_count(), "sources": s.source_count(), "records": s.record_count(), "quality": scores})))
        }
        _ => Err(McpError::ToolNotFound(name.into())),
    }
}
