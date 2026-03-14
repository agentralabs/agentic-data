# AgenticData — Agent Integration Playbook

## Claude Desktop Integration

```json
{
  "mcpServers": {
    "agentic-data": {
      "command": "agentic-data-mcp",
      "args": []
    }
  }
}
```

After restart, ask Claude: *"Detect the format of this file and tell me about its structure"*

## Multi-Sister Setup

AgenticData works alongside other sisters:

```json
{
  "mcpServers": {
    "agentic-memory": { "command": "agentic-memory-mcp", "args": ["--memory-dir", "~/.agentic/memory"] },
    "agentic-data": { "command": "agentic-data-mcp", "args": [] },
    "agentic-codebase": { "command": "agentic-codebase-mcp", "args": [] }
  }
}
```

## Tool Categories for Agents

| Agent task | Tools to use | Category |
|-----------|-------------|----------|
| "Understand this data" | `data_schema_infer`, `data_soul_extract` | Comprehension |
| "Clean this dataset" | `data_quality_score`, `data_quality_heal` | Quality |
| "Find sensitive data" | `data_redact_detect`, `data_redact_apply` | Security |
| "Convert between formats" | `data_format_detect`, `data_bridge_convert` | Transformation |
| "Query across sources" | `data_federate_register`, `data_cross_query` | Federation |
| "Track data lineage" | `data_dna_trace`, `data_dna_impact` | Provenance |
| "Monitor quality" | `data_quality_timeline`, `data_anomaly_subscribe` | Monitoring |

## Standalone Usage

AgenticData requires NO other Agentra components. It works as a standalone MCP server with any MCP-compatible client.
