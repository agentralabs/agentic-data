//! Tool registry — central dispatch for all 131 MCP tools.

use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::Value;

use crate::types::{McpError, McpResult, ToolCallResult, ToolDefinition};
use agentic_data::DataStore;

/// Central registry of all AgenticData MCP tools.
pub struct ToolRegistry {
    store: Arc<Mutex<DataStore>>,
}

impl ToolRegistry {
    pub fn new(store: Arc<Mutex<DataStore>>) -> Self {
        Self { store }
    }

    /// List all tool definitions.
    pub fn list_tools(&self) -> Vec<ToolDefinition> {
        let mut tools = Vec::new();
        tools.extend(super::schema::definitions());
        tools.extend(super::format::definitions());
        tools.extend(super::document::definitions());
        tools.extend(super::soul::definitions());
        tools.extend(super::transform::definitions());
        tools.extend(super::bridge::definitions());
        tools.extend(super::media::definitions());
        tools.extend(super::quality::definitions());
        tools.extend(super::temporal::definitions());
        tools.extend(super::dna::definitions());
        tools.extend(super::evolve::definitions());
        tools.extend(super::cross::definitions());
        tools.extend(super::query::definitions());
        tools.extend(super::anomaly::definitions());
        tools.extend(super::geo::definitions());
        tools.extend(super::vault::definitions());
        tools.extend(super::redact::definitions());
        tools.extend(super::federate::definitions());
        tools.extend(super::version::definitions());
        tools.extend(super::predict::definitions());
        tools.extend(super::dream::definitions());
        tools.extend(super::synthetic::definitions());
        tools.extend(super::metabolism::definitions());
        tools.extend(super::collective::definitions());
        tools
    }

    /// Dispatch a tool call to the correct handler.
    pub async fn call_tool(&self, name: &str, args: Value) -> McpResult<ToolCallResult> {
        let store = self.store.clone();
        match name {
            // Invention 1: Schema
            n if n.starts_with("data_schema_") => super::schema::execute(n, args, &store).await,
            // Invention 2: Format
            n if n.starts_with("data_format_") => super::format::execute(n, args, &store).await,
            // Invention 3: Document
            n if n.starts_with("data_doc_") => super::document::execute(n, args, &store).await,
            // Invention 4: Soul
            n if n.starts_with("data_soul_") => super::soul::execute(n, args, &store).await,
            // Invention 5: Transform
            n if n.starts_with("data_transform_") => super::transform::execute(n, args, &store).await,
            // Invention 6: Bridge
            n if n.starts_with("data_bridge_") => super::bridge::execute(n, args, &store).await,
            // Invention 7: Media
            n if n.starts_with("data_media_") => super::media::execute(n, args, &store).await,
            // Invention 8: Quality
            n if n.starts_with("data_quality_") => super::quality::execute(n, args, &store).await,
            // Invention 9: Temporal
            n if n.starts_with("data_temporal_") => super::temporal::execute(n, args, &store).await,
            // Invention 10: DNA
            n if n.starts_with("data_dna_") => super::dna::execute(n, args, &store).await,
            // Invention 11: Evolution
            n if n.starts_with("data_evolve_") => super::evolve::execute(n, args, &store).await,
            // Invention 12: Cross-dataset
            n if n.starts_with("data_cross_") => super::cross::execute(n, args, &store).await,
            // Invention 13: Query
            n if n.starts_with("data_query_") => super::query::execute(n, args, &store).await,
            // Invention 14: Anomaly
            n if n.starts_with("data_anomaly_") => super::anomaly::execute(n, args, &store).await,
            // Invention 15: Geo
            n if n.starts_with("data_geo_") => super::geo::execute(n, args, &store).await,
            // Invention 16: Vault
            n if n.starts_with("data_vault_") => super::vault::execute(n, args, &store).await,
            // Invention 17: Redact
            n if n.starts_with("data_redact_") => super::redact::execute(n, args, &store).await,
            // Invention 18: Federate
            n if n.starts_with("data_federate_") => super::federate::execute(n, args, &store).await,
            // Invention 19: Version
            n if n.starts_with("data_version_") => super::version::execute(n, args, &store).await,
            // Invention 20: Predict
            n if n.starts_with("data_predict_") => super::predict::execute(n, args, &store).await,
            // Invention 21: Dream
            n if n.starts_with("data_dream_") => super::dream::execute(n, args, &store).await,
            // Invention 22: Synthetic
            n if n.starts_with("data_synthetic_") => super::synthetic::execute(n, args, &store).await,
            // Invention 23: Metabolism
            n if n.starts_with("data_metabolism_") => super::metabolism::execute(n, args, &store).await,
            // Invention 24: Collective
            n if n.starts_with("data_collective_") => super::collective::execute(n, args, &store).await,
            _ => Err(McpError::ToolNotFound(name.to_string())),
        }
    }

    /// Total tool count.
    pub fn tool_count(&self) -> usize {
        self.list_tools().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_tool_count() {
        let store = Arc::new(Mutex::new(DataStore::new()));
        let registry = ToolRegistry::new(store);
        assert!(registry.tool_count() >= 120, "Expected 120+ tools, got {}", registry.tool_count());
    }

    #[tokio::test]
    async fn test_unknown_tool() {
        let store = Arc::new(Mutex::new(DataStore::new()));
        let registry = ToolRegistry::new(store);
        let result = registry.call_tool("nonexistent", serde_json::json!({})).await;
        assert!(result.is_err());
    }
}
