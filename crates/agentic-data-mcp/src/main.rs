//! AgenticData MCP Server — entry point.
//! 131 MCP tools across 24 inventions.

mod types;
mod tools;

use std::sync::Arc;
use tokio::sync::Mutex;

fn main() {
    let store = Arc::new(Mutex::new(agentic_data::DataStore::new()));
    let registry = tools::ToolRegistry::new(store);
    let tool_count = registry.list_tools().len();
    eprintln!("AgenticData MCP Server v{}", env!("CARGO_PKG_VERSION"));
    eprintln!("{} tools registered across 24 inventions", tool_count);

    // MCP stdio transport loop would go here (Phase 7)
    // For now, just verify all tools are registered
    for tool in registry.list_tools() {
        eprintln!("  {} — {}", tool.name, tool.description.as_deref().unwrap_or(""));
    }
}
