//! Invention 3: Deep Document Comprehension — 6 tools with real implementations.

use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::{json, Value};
use agentic_data::DataStore;
use crate::types::*;

pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition { name: "data_doc_parse".into(), description: Some("Parse any document into structured DOM".into()), input_schema: json!({"type":"object"}), },
        ToolDefinition { name: "data_doc_tables".into(), description: Some("Extract all tables from document".into()), input_schema: json!({"type":"object"}), },
        ToolDefinition { name: "data_doc_headers".into(), description: Some("Extract document heading hierarchy".into()), input_schema: json!({"type":"object"}), },
        ToolDefinition { name: "data_doc_search".into(), description: Some("Search within document by content".into()), input_schema: json!({"type":"object"}), },
        ToolDefinition { name: "data_doc_compare".into(), description: Some("Compare two document versions".into()), input_schema: json!({"type":"object"}), },
        ToolDefinition { name: "data_doc_metadata".into(), description: Some("Extract all document metadata".into()), input_schema: json!({"type":"object"}), },
    ]
}

pub async fn execute(name: &str, args: Value, store: &Arc<Mutex<DataStore>>) -> McpResult<ToolCallResult> {
    match name {
        "data_doc_parse" => {
            let data = args.get("data").and_then(|v| v.as_str()).ok_or(McpError::InvalidParams("data required".into()))?;
            let mut s = store.lock().await;
            let mut eng = agentic_data::IngestEngine::new(&mut s);
            match eng.ingest_string(data, "document") {
                Ok(r) => Ok(ToolCallResult::json(&json!({"records": r.records_added, "schema_id": r.schema_id, "errors": r.parse_errors}))),
                Err(e) => Ok(ToolCallResult::error(&e.to_string())),
            }
        }
        "data_doc_tables" => {
            let data = args.get("data").and_then(|v| v.as_str()).ok_or(McpError::InvalidParams("data required".into()))?;
            let result = agentic_data::parser::parse_auto(data, "doc_tables");
            match result {
                Ok(r) => {
                    let tables: Vec<serde_json::Value> = r.records.iter()
                        .filter(|rec| rec.get_str("type") == Some("table_row"))
                        .map(|rec| json!(rec.fields)).collect();
                    Ok(ToolCallResult::json(&json!({"tables": tables, "count": tables.len()})))
                }
                Err(e) => Ok(ToolCallResult::error(&e.to_string())),
            }
        }
        "data_doc_headers" => {
            let data = args.get("data").and_then(|v| v.as_str()).ok_or(McpError::InvalidParams("data required".into()))?;
            let result = agentic_data::parser::parse_auto(data, "doc_headers");
            match result {
                Ok(r) => {
                    let headings: Vec<String> = r.records.iter()
                        .filter(|rec| rec.get_str("type") == Some("heading") || rec.get_str("heading").is_some())
                        .filter_map(|rec| rec.get_str("content").or(rec.get_str("heading")).map(|s| s.to_string())).collect();
                    Ok(ToolCallResult::json(&json!({"headings": headings, "count": headings.len()})))
                }
                Err(e) => Ok(ToolCallResult::error(&e.to_string())),
            }
        }
        "data_doc_search" => {
            let data = args.get("data").and_then(|v| v.as_str()).ok_or(McpError::InvalidParams("data required".into()))?;
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            let lower_q = query.to_lowercase();
            let matches: Vec<String> = data.lines().filter(|l| l.to_lowercase().contains(&lower_q)).map(|l| l.to_string()).take(20).collect();
            Ok(ToolCallResult::json(&json!({"query": query, "matches": matches, "count": matches.len()})))
        }
        "data_doc_compare" => {
            let a = args.get("doc_a").and_then(|v| v.as_str()).unwrap_or("");
            let b = args.get("doc_b").and_then(|v| v.as_str()).unwrap_or("");
            let a_lines: Vec<&str> = a.lines().collect();
            let b_lines: Vec<&str> = b.lines().collect();
            let added = b_lines.iter().filter(|l| !a_lines.contains(l)).count();
            let removed = a_lines.iter().filter(|l| !b_lines.contains(l)).count();
            Ok(ToolCallResult::json(&json!({"lines_a": a_lines.len(), "lines_b": b_lines.len(), "added": added, "removed": removed})))
        }
        "data_doc_metadata" => {
            let data = args.get("data").and_then(|v| v.as_str()).ok_or(McpError::InvalidParams("data required".into()))?;
            Ok(ToolCallResult::json(&json!({"length": data.len(), "lines": data.lines().count(), "words": data.split_whitespace().count()})))
        }
        _ => Err(McpError::ToolNotFound(name.into())),
    }
}
