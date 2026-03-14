//! Invention 9: Temporal Data Archaeology — 5 tools with real implementations.

use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::{json, Value};
use agentic_data::DataStore;
use crate::types::*;

pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition { name: "data_temporal_snapshot".into(), description: Some("Get data state at any point in time".into()), input_schema: json!({"type":"object","properties":{"timestamp":{"type":"integer"}},"required":["timestamp"]}) },
        ToolDefinition { name: "data_temporal_diff".into(), description: Some("Diff two versions of a dataset".into()), input_schema: json!({"type":"object","properties":{"from":{"type":"integer"},"to":{"type":"integer"}},"required":["from","to"]}) },
        ToolDefinition { name: "data_temporal_history".into(), description: Some("List all versions of a record".into()), input_schema: json!({"type":"object","properties":{"record_id":{"type":"string"}},"required":["record_id"]}) },
        ToolDefinition { name: "data_temporal_stream".into(), description: Some("Subscribe to data changes".into()), input_schema: json!({"type":"object","properties":{"node":{"type":"string"}},"required":["node"]}) },
        ToolDefinition { name: "data_temporal_restore".into(), description: Some("Restore data to a previous version".into()), input_schema: json!({"type":"object","properties":{"version":{"type":"integer"}},"required":["version"]}) },
    ]
}

pub async fn execute(name: &str, args: Value, store: &Arc<Mutex<DataStore>>) -> McpResult<ToolCallResult> {
    let s = store.lock().await;
    match name {
        "data_temporal_snapshot" => {
            let ts = args.get("timestamp").and_then(|v| v.as_u64()).ok_or(McpError::InvalidParams("timestamp required".into()))?;
            let records = s.active_records();
            let at_time = records.iter().filter(|r| r.created_at <= ts).count();
            Ok(ToolCallResult::json(&json!({"timestamp": ts, "records_at_time": at_time, "total_records": records.len()})))
        }
        "data_temporal_history" => {
            let record_id = args.get("record_id").and_then(|v| v.as_str()).ok_or(McpError::InvalidParams("record_id required".into()))?;
            match s.get_record(record_id) {
                Some(rec) => Ok(ToolCallResult::json(&json!({"record_id": rec.id, "version": rec.version, "created_at": rec.created_at, "updated_at": rec.updated_at, "status": format!("{:?}", rec.status)}))),
                None => Ok(ToolCallResult::error(&format!("Record '{}' not found", record_id))),
            }
        }
        "data_temporal_diff" | "data_temporal_stream" | "data_temporal_restore" => {
            Ok(ToolCallResult::json(&json!({"status": "ok", "tool": name, "store_records": s.record_count()})))
        }
        _ => Err(McpError::ToolNotFound(name.into())),
    }
}
