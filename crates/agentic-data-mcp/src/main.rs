//! AgenticData MCP Server — JSON-RPC over stdio.

mod types;
mod tools;

use std::io::{self, BufRead, Write};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use tools::ToolRegistry;

#[derive(Deserialize)]
struct JsonRpcRequest {
    jsonrpc: Option<String>,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

impl JsonRpcResponse {
    fn success(id: Option<Value>, result: Value) -> Self {
        Self { jsonrpc: "2.0".into(), id, result: Some(result), error: None }
    }
    fn error(id: Option<Value>, code: i32, msg: String) -> Self {
        Self { jsonrpc: "2.0".into(), id, result: None,
            error: Some(json!({"code": code, "message": msg})) }
    }
}

#[tokio::main]
async fn main() {
    let store = Arc::new(Mutex::new(agentic_data::DataStore::new()));
    let registry = Arc::new(ToolRegistry::new(store.clone()));
    eprintln!("AgenticData MCP Server v{} — {} tools", env!("CARGO_PKG_VERSION"), registry.tool_count());

    let stdin = io::stdin();
    let stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() { continue; }

        let response = handle_message(&line, &registry, &store).await;
        let json = serde_json::to_string(&response).unwrap_or_default();

        let mut out = stdout.lock();
        let _ = writeln!(out, "{}", json);
        let _ = out.flush();
    }
}

async fn handle_message(
    line: &str,
    registry: &Arc<ToolRegistry>,
    store: &Arc<Mutex<agentic_data::DataStore>>,
) -> JsonRpcResponse {
    let request: JsonRpcRequest = match serde_json::from_str(line) {
        Ok(r) => r,
        Err(e) => return JsonRpcResponse::error(None, -32700, format!("Parse error: {}", e)),
    };

    let id = request.id.clone();

    match request.method.as_str() {
        "initialize" => JsonRpcResponse::success(id, json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": { "listChanged": false } },
            "serverInfo": { "name": "agentic-data", "version": env!("CARGO_PKG_VERSION") }
        })),

        "tools/list" => {
            let tools: Vec<Value> = registry.list_tools().iter().map(|t| {
                json!({
                    "name": t.name,
                    "description": t.description,
                    "inputSchema": t.input_schema
                })
            }).collect();
            JsonRpcResponse::success(id, json!({ "tools": tools }))
        }

        "tools/call" => {
            let params = request.params.unwrap_or(json!({}));
            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));

            match registry.call_tool(tool_name, args).await {
                Ok(result) => JsonRpcResponse::success(id, json!({
                    "content": result.content,
                    "isError": result.is_error
                })),
                Err(e) => JsonRpcResponse::error(id, -32803, format!("{}", e)),
            }
        }

        "notifications/initialized" => JsonRpcResponse::success(id, json!({})),

        _ => JsonRpcResponse::error(id, -32601, format!("Method not found: {}", request.method)),
    }
}
