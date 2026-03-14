//! MCP protocol types for AgenticData.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// MCP tool definition.
#[derive(Debug, Clone, Serialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// Result of a tool execution.
#[derive(Debug, Clone, Serialize)]
pub struct ToolCallResult {
    pub content: Vec<ToolContent>,
    #[serde(rename = "isError", skip_serializing_if = "std::ops::Not::not")]
    pub is_error: bool,
}

/// Content block in a tool result.
#[derive(Debug, Clone, Serialize)]
pub struct ToolContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

impl ToolCallResult {
    /// Create a successful text result.
    pub fn text(content: &str) -> Self {
        Self {
            content: vec![ToolContent { content_type: "text".into(), text: content.to_string() }],
            is_error: false,
        }
    }

    /// Create a successful JSON result.
    pub fn json(value: &Value) -> Self {
        Self::text(&serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string()))
    }

    /// Create an error result.
    pub fn error(message: &str) -> Self {
        Self {
            content: vec![ToolContent { content_type: "text".into(), text: message.to_string() }],
            is_error: true,
        }
    }
}

/// MCP error type.
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("Invalid params: {0}")]
    InvalidParams(String),
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    #[error("Execution error: {0}")]
    ExecutionError(String),
}

pub type McpResult<T> = Result<T, McpError>;
