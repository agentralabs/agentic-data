# Contributing to AgenticData

## Getting Started

```bash
git clone https://github.com/agentralabs/agentic-data.git
cd agentic-data
cargo test -j 1
```

## Development Guidelines

- **400-line limit** per .rs file. Split by responsibility.
- **All files under 400 lines.** No exceptions.
- Run `cargo clippy --all -- -D warnings` before submitting.
- Run `cargo fmt --check` before submitting.
- Run `bash scripts/check-canonical-sister.sh` before submitting.

## Adding a New Parser

1. Create `crates/agentic-data/src/parser/<format>_parser.rs`
2. Implement `pub fn parse(data: &str, source_name: &str) -> AdatResult<ParseResult>`
3. Add the format to `DataFormat` enum in `parser/mod.rs`
4. Add detection logic in `parser/detect.rs`
5. Wire into `parse_as()` in `parser/mod.rs`
6. Add tests (valid input, malformed input, edge cases)

## Adding a New MCP Tool

1. Add tool definition in the appropriate invention module under `crates/agentic-data-mcp/src/tools/`
2. Add match arm in the module's `execute()` function
3. Tool descriptions must be verb-first imperative, no trailing periods
4. Tool errors use `isError: true` in the result
5. Unknown tools return JSON-RPC error code `-32803` (TOOL_NOT_FOUND)

## Pull Request Process

1. Fork the repo and create a feature branch
2. Write tests for new functionality
3. Ensure all tests pass: `cargo test --all`
4. Ensure guardrails pass: `bash scripts/check-canonical-sister.sh`
5. Submit PR with clear description of changes
