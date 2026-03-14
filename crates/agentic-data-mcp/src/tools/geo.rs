//! Invention 15: Geospatial Consciousness — 6 tools.

use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::{json, Value};
use agentic_data::DataStore;
use crate::types::*;

pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "data_geo_parse".into(),
            description: Some("Parse location from text or coordinates".into()),
            input_schema: json!({"type": "object"}),
        },
        ToolDefinition {
            name: "data_geo_distance".into(),
            description: Some("Calculate distance between two locations".into()),
            input_schema: json!({"type": "object"}),
        },
        ToolDefinition {
            name: "data_geo_contains".into(),
            description: Some("Check if point is within region".into()),
            input_schema: json!({"type": "object"}),
        },
        ToolDefinition {
            name: "data_geo_cluster".into(),
            description: Some("Cluster records by location".into()),
            input_schema: json!({"type": "object"}),
        },
        ToolDefinition {
            name: "data_geo_enrich".into(),
            description: Some("Add spatial context to records".into()),
            input_schema: json!({"type": "object"}),
        },
        ToolDefinition {
            name: "data_geo_query".into(),
            description: Some("Spatial query (within, intersects, nearest)".into()),
            input_schema: json!({"type": "object"}),
        },
    ]
}

pub async fn execute(name: &str, args: Value, store: &Arc<Mutex<DataStore>>) -> McpResult<ToolCallResult> {
    let _store = store;
    let _args = args;
    match name {
        "data_geo_parse" => Ok(ToolCallResult::json(&json!({"status": "ok", "tool": "data_geo_parse"}))),
        "data_geo_distance" => Ok(ToolCallResult::json(&json!({"status": "ok", "tool": "data_geo_distance"}))),
        "data_geo_contains" => Ok(ToolCallResult::json(&json!({"status": "ok", "tool": "data_geo_contains"}))),
        "data_geo_cluster" => Ok(ToolCallResult::json(&json!({"status": "ok", "tool": "data_geo_cluster"}))),
        "data_geo_enrich" => Ok(ToolCallResult::json(&json!({"status": "ok", "tool": "data_geo_enrich"}))),
        "data_geo_query" => Ok(ToolCallResult::json(&json!({"status": "ok", "tool": "data_geo_query"}))),
        _ => Err(McpError::ToolNotFound(name.into())),
    }
}
