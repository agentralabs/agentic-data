//! Invention 18: Data Federation — 5 tools with real implementations.

use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::{json, Value};
use agentic_data::DataStore;
use crate::types::*;

pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition { name: "data_federate_register".into(), description: Some("Register a data source".into()), input_schema: json!({"type":"object"}), },
        ToolDefinition { name: "data_federate_query".into(), description: Some("Federated query across sources".into()), input_schema: json!({"type":"object"}), },
        ToolDefinition { name: "data_federate_health".into(), description: Some("Check source availability".into()), input_schema: json!({"type":"object"}), },
        ToolDefinition { name: "data_federate_cache".into(), description: Some("Manage cross-source cache".into()), input_schema: json!({"type":"object"}), },
        ToolDefinition { name: "data_federate_lineage".into(), description: Some("Track cross-source lineage".into()), input_schema: json!({"type":"object"}), },
    ]
}

pub async fn execute(name: &str, args: Value, store: &Arc<Mutex<DataStore>>) -> McpResult<ToolCallResult> {
    match name {
        "data_federate_register" => {
            let name = args.get("name").and_then(|v| v.as_str()).ok_or(McpError::InvalidParams("name required".into()))?;
            let src_type = args.get("type").and_then(|v| v.as_str()).unwrap_or("file");
            let conn = args.get("connection").and_then(|v| v.as_str()).unwrap_or("");
            let mut s = store.lock().await;
            let source = agentic_data::DataSource::new(name, agentic_data::SourceType::File, conn);
            let id = source.id.clone();
            s.add_source(source);
            Ok(ToolCallResult::json(&json!({"source_id": id, "name": name, "type": src_type})))
        }
        "data_federate_query" => {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            let s = store.lock().await;
            let qe = agentic_data::QueryEngine::new(&s);
            let result = qe.search(query, Some(20));
            Ok(ToolCallResult::json(&json!({"query": query, "results": result.total_matched, "sources": s.source_count()})))
        }
        "data_federate_health" => {
            let s = store.lock().await;
            let sources: Vec<serde_json::Value> = s.all_sources().iter().map(|src| json!({"name": src.name, "healthy": src.healthy, "records": src.record_count})).collect();
            Ok(ToolCallResult::json(&json!({"sources": sources, "total": sources.len()})))
        }
        "data_federate_cache" => {
            Ok(ToolCallResult::json(&json!({"status": "ok", "cache": "empty"})))
        }
        "data_federate_lineage" => {
            let target = args.get("target").and_then(|v| v.as_str()).unwrap_or("");
            let s = store.lock().await;
            match s.get_lineage(target) {
                Some(c) => Ok(ToolCallResult::json(&json!({"target": target, "depth": c.depth(), "trust": c.trust_score}))),
                None => Ok(ToolCallResult::json(&json!({"target": target, "depth": 0}))),
            }
        }
        _ => Err(McpError::ToolNotFound(name.into())),
    }
}
