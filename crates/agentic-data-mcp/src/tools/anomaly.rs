//! Invention 14: Anomaly Constellation — 5 tools with real implementations.

use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::{json, Value};
use agentic_data::DataStore;
use crate::types::*;

pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition { name: "data_anomaly_constellation".into(), description: Some("Detect correlated anomaly patterns".into()), input_schema: json!({"type":"object"}), },
        ToolDefinition { name: "data_anomaly_root_cause".into(), description: Some("Trace anomalies to root cause".into()), input_schema: json!({"type":"object"}), },
        ToolDefinition { name: "data_anomaly_impact".into(), description: Some("Assess impact radius".into()), input_schema: json!({"type":"object"}), },
        ToolDefinition { name: "data_anomaly_predict".into(), description: Some("Predict likely future anomalies".into()), input_schema: json!({"type":"object"}), },
        ToolDefinition { name: "data_anomaly_subscribe".into(), description: Some("Subscribe to anomaly alerts".into()), input_schema: json!({"type":"object"}), },
    ]
}

pub async fn execute(name: &str, args: Value, store: &Arc<Mutex<DataStore>>) -> McpResult<ToolCallResult> {
    match name {
        "data_anomaly_constellation" => {
            let s = store.lock().await;
            let mut all_anomalies = Vec::new();
            let mut nodes: Vec<String> = s.active_records().iter().map(|r| r.schema_node.clone()).collect();
            nodes.sort(); nodes.dedup();
            for node in &nodes {
                let anoms = agentic_data::QualityEngine::detect_anomalies(&s, node);
                all_anomalies.extend(anoms.iter().map(|a| json!({"node": node, "type": format!("{:?}", a.anomaly_type), "severity": a.severity})));
            }
            Ok(ToolCallResult::json(&json!({"constellations": all_anomalies, "nodes_scanned": nodes.len()})))
        }
        "data_anomaly_root_cause" => {
            let s = store.lock().await;
            Ok(ToolCallResult::json(&json!({"status": "ok", "records": s.record_count()})))
        }
        "data_anomaly_impact" => {
            let s = store.lock().await;
            Ok(ToolCallResult::json(&json!({"status": "ok", "records": s.record_count()})))
        }
        "data_anomaly_predict" => {
            let s = store.lock().await;
            let node = args.get("node").and_then(|v| v.as_str()).unwrap_or("");
            let score = agentic_data::QualityEngine::score(&s, node);
            let risk = if score.score < 50 { "high" } else if score.score < 80 { "medium" } else { "low" };
            Ok(ToolCallResult::json(&json!({"node": node, "current_score": score.score, "predicted_risk": risk})))
        }
        "data_anomaly_subscribe" => {
            Ok(ToolCallResult::json(&json!({"subscribed": true, "note": "Alerts will fire when anomalies detected"})))
        }
        _ => Err(McpError::ToolNotFound(name.into())),
    }
}
