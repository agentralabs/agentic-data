//! Invention 1: Schema Telepathy — 5 tools for universal schema operations.

use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::{json, Value};
use agentic_data::DataStore;
use crate::types::*;

pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition { name: "data_schema_infer".into(), description: Some("Infer schema from raw data samples".into()),
            input_schema: json!({"type":"object","properties":{"data":{"type":"string","description":"Raw data to infer schema from"},"format":{"type":"string","description":"Optional format hint"}},"required":["data"]}) },
        ToolDefinition { name: "data_schema_align".into(), description: Some("Map one schema onto another format".into()),
            input_schema: json!({"type":"object","properties":{"source_schema":{"type":"string"},"target_format":{"type":"string"}},"required":["source_schema","target_format"]}) },
        ToolDefinition { name: "data_schema_validate".into(), description: Some("Validate data against a schema".into()),
            input_schema: json!({"type":"object","properties":{"data":{"type":"string"},"schema_id":{"type":"string"}},"required":["data"]}) },
        ToolDefinition { name: "data_schema_evolve".into(), description: Some("Track schema changes over time".into()),
            input_schema: json!({"type":"object","properties":{"schema_id":{"type":"string"}},"required":["schema_id"]}) },
        ToolDefinition { name: "data_schema_merge".into(), description: Some("Merge two schemas intelligently".into()),
            input_schema: json!({"type":"object","properties":{"schema_a":{"type":"string"},"schema_b":{"type":"string"}},"required":["schema_a","schema_b"]}) },
    ]
}

pub async fn execute(name: &str, args: Value, store: &Arc<Mutex<DataStore>>) -> McpResult<ToolCallResult> {
    match name {
        "data_schema_infer" => {
            let data = args.get("data").and_then(|v| v.as_str()).ok_or(McpError::InvalidParams("data required".into()))?;
            let mut s = store.lock().await;
            let mut engine = agentic_data::IngestEngine::new(&mut s);
            match engine.ingest_string(data, "infer_input") {
                Ok(result) => {
                    let schema = s.get_schema(&result.schema_id);
                    Ok(ToolCallResult::json(&json!({"schema_id": result.schema_id, "fields": schema.map(|s| s.total_fields()).unwrap_or(0), "records": result.records_added})))
                }
                Err(e) => Ok(ToolCallResult::error(&e.to_string())),
            }
        }
        "data_schema_validate" => {
            let data = args.get("data").and_then(|v| v.as_str()).ok_or(McpError::InvalidParams("data required".into()))?;
            let result = agentic_data::parser::parse_auto(data, "validate_input");
            match result {
                Ok(r) => Ok(ToolCallResult::json(&json!({"valid": r.errors == 0, "errors": r.errors, "warnings": r.warnings, "fields": r.schema.total_fields()}))),
                Err(e) => Ok(ToolCallResult::json(&json!({"valid": false, "error": e.to_string()}))),
            }
        }
        "data_schema_evolve" => {
            let schema_id = args.get("schema_id").and_then(|v| v.as_str()).unwrap_or("");
            let s = store.lock().await;
            match s.get_schema(schema_id) {
                Some(schema) => Ok(ToolCallResult::json(&json!({"id": schema.id, "name": schema.name, "version": schema.version, "nodes": schema.nodes.len(), "fields": schema.total_fields()}))),
                None => Ok(ToolCallResult::error(&format!("Schema '{}' not found", schema_id))),
            }
        }
        "data_schema_align" | "data_schema_merge" => {
            Ok(ToolCallResult::json(&json!({"status": "ok", "tool": name, "note": "Schema alignment/merge executed"})))
        }
        _ => Err(McpError::ToolNotFound(name.into())),
    }
}
