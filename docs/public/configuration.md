# AgenticData — Configuration

## Environment Variables

All AgenticData-specific variables use the `ADAT_` prefix. Cross-sister variables use `AGENTRA_`.

| Variable | Default | Values | Effect |
|----------|---------|--------|--------|
| `ADAT_VERSION` | `latest` | semver | Pin install to specific version |
| `ADAT_STORAGE_DIR` | `~/.agentic/data` | path | Directory for .adat files |
| `ADAT_MAX_RECORD_SIZE` | `1048576` | bytes | Max content per record (1MB default) |
| `ADAT_QUALITY_THRESHOLD` | `50` | 0-100 | Quality score below which warnings are emitted |
| `ADAT_PII_POLICY` | `mask` | `mask`, `hash`, `placeholder`, `remove` | Default PII redaction method |
| `ADAT_LOG_LEVEL` | `info` | `error`, `warn`, `info`, `debug`, `trace` | Logging verbosity |
| `ADAT_ENCRYPTION_KEY` | — | string | Master key for field-level encryption |
| `ADAT_MCP_TOOL_SURFACE` | `full` | `full`, `compact` | MCP tool visibility mode |
| `AGENTRA_INSTALL_PROFILE` | `desktop` | `desktop`, `terminal`, `server` | Install profile |
| `AGENTIC_TOKEN` | — | string | Auth token for server profile |

## MCP Server Configuration

The MCP server reads configuration from:
1. Environment variables (highest priority)
2. `~/.agentic/data/config.toml` (user config)
3. Built-in defaults (lowest priority)

## Install Profiles

| Profile | CLI | MCP Server | Auth |
|---------|-----|-----------|------|
| `desktop` | Yes | Yes (Claude Desktop config) | No |
| `terminal` | Yes | Yes (manual config) | No |
| `server` | Yes | Yes (daemon mode) | `AGENTIC_TOKEN` required |
