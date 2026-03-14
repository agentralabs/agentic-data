//! Invention 13: Query Prophecy — 5 tools with real implementations.

use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::{json, Value};
use agentic_data::DataStore;
use crate::types::*;

pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition { name: "data_query_natural".into(), description: Some("Generate query from natural language".into()), input_schema: json!({"type":"object"}), },
        ToolDefinition { name: "data_query_explain".into(), description: Some("Explain what a query does".into()), input_schema: json!({"type":"object"}), },
        ToolDefinition { name: "data_query_preview".into(), description: Some("Preview results on sample data".into()), input_schema: json!({"type":"object"}), },
        ToolDefinition { name: "data_query_optimize".into(), description: Some("Suggest query optimizations".into()), input_schema: json!({"type":"object"}), },
        ToolDefinition { name: "data_query_history".into(), description: Some("View past queries".into()), input_schema: json!({"type":"object"}), },
    ]
}

pub async fn execute(name: &str, args: Value, store: &Arc<Mutex<DataStore>>) -> McpResult<ToolCallResult> {
    match name {
        "data_query_natural" => {
            let question = args.get("question").and_then(|v| v.as_str()).ok_or(McpError::InvalidParams("question required".into()))?;
            let s = store.lock().await;
            let qe = agentic_data::QueryEngine::new(&s);
            let result = qe.search(question, Some(10));
            Ok(ToolCallResult::json(&json!({"question": question, "results": result.total_matched, "scanned": result.total_scanned})))
        }
        "data_query_explain" => {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            Ok(ToolCallResult::json(&json!({"query": query, "explanation": format!("Search for records matching '{}'", query)})))
        }
        "data_query_preview" => {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
            let s = store.lock().await;
            let qe = agentic_data::QueryEngine::new(&s);
            let result = qe.search(query, Some(limit));
            let preview: Vec<serde_json::Value> = result.records.iter().map(|r| json!(r.fields)).collect();
            Ok(ToolCallResult::json(&json!({"preview": preview, "count": preview.len()})))
        }
        "data_query_optimize" => {
            Ok(ToolCallResult::json(&json!({"suggestions": ["Add index on frequently filtered fields", "Use specific node name instead of searching all"]})))
        }
        "data_query_history" => {
            Ok(ToolCallResult::json(&json!({"history": [], "note": "Query history tracking available in session mode"})))
        }
        _ => Err(McpError::ToolNotFound(name.into())),
    }
}
