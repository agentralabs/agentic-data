# Changelog

All notable changes to AgenticData will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] — 2026-03-13

### Added
- Core library with Universal Schema types, record management, and lineage tracking
- .adat binary file format (LZ4 compressed, BLAKE3 checksums, memory-mappable)
- 12 format parsers: CSV, TSV, JSON, JSONL, XML, YAML, TOML, HTML, SQL, Log, Email, Calendar
- 3 geospatial parsers: GeoJSON, KML, GPX
- Media metadata parser
- Auto-detection for 16 formats
- Core engine: DataStore, IngestEngine, QueryEngine, TransformEngine, QualityEngine
- 5 index types: SchemaIndex, TemporalIndex, QualityIndex, SpatialIndex, LineageIndex
- Crypto: field-level encryption, key management, PII detection + redaction
- MCP server with 122 tools across 24 inventions
- CLI with 8 commands (ingest, detect, query, quality, pii, status, formats, schema)
- FFI with 5 C-compatible functions
- Full Canonical Sister Kit compliance (18 doc pages, installer, CI guardrails)
- 144 tests across all crates
