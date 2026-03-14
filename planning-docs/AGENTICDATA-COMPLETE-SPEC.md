# AGENTICDATA — COMPLETE IMPLEMENTATION PLAN

> **Status:** Implementation Specification
> **Version:** 1.0
> **Date:** March 2026
> **Benchmark:** AgenticMemory (24 inventions, 130+ MCP tools, 65K lines)
> **Goal:** Build the sister that UNDERSTANDS all information — databases, documents, media, geospatial, logs, and every data format that exists or will exist.

---

## WHAT AGENTICDATA IS

AgenticMemory answers: "What do I remember?"
AgenticData answers: "What does this DATA mean?"

```
╔═══════════════════════════════════════════════════════════════════════════╗
║                                                                           ║
║  AGENTICDATA IS A UNIVERSAL DATA UNDERSTANDING ENGINE                     ║
║                                                                           ║
║  It doesn't just PARSE data — it UNDERSTANDS structure, infers meaning,  ║
║  tracks lineage, detects anomalies, and transforms between any format.   ║
║                                                                           ║
║  Current state of the art:                                                ║
║  - pandas: reads CSV but doesn't understand what the columns MEAN        ║
║  - SQL tools: query databases but can't infer missing schemas            ║
║  - PDF parsers: extract text but lose table structure                    ║
║  - ETL tools: transform data but don't track WHERE it came from         ║
║                                                                           ║
║  AgenticData: one engine that does ALL of it with understanding.         ║
║                                                                           ║
║  This is NOT a data processing framework.                                 ║
║  This is a DATA COMPREHENSION system.                                     ║
║                                                                           ║
╚═══════════════════════════════════════════════════════════════════════════╝
```

---

## THE 24 INVENTIONS

### COMPREHENSION INVENTIONS (Understanding what data IS)

#### INVENTION 1: UNIVERSAL SCHEMA TELEPATHY

**The Problem:** Every data source has a different way of describing its structure. Databases have DDL. CSVs have headers (maybe). JSONs have implicit nesting. PDFs have visual layout. There's no universal way to understand "what is this data?"

**The Solution:** A Universal Schema that can represent ANY data structure — relational tables, nested documents, key-value pairs, graphs, time series, geospatial, even unstructured text with inferred structure. One schema language that captures them all.

**Key Concepts:**
- SchemaNode: column, field, array, map, nested object, geospatial, temporal
- SchemaEdge: belongs_to, references, derived_from, contains
- SchemaInference: given raw data, infer the schema automatically
- SchemaAlignment: map one schema onto another (CSV columns → database table)
- ConfidenceAnnotation: "I'm 95% sure column 3 is a date, 60% sure it's USD"

**MCP Tools:**
```
data_schema_infer       Infer schema from raw data samples
data_schema_align       Map one schema onto another format
data_schema_validate    Validate data against a schema
data_schema_evolve      Track schema changes over time
data_schema_merge       Merge two schemas intelligently
```

---

#### INVENTION 2: FORMAT OMNISCIENCE

**The Problem:** There are hundreds of data formats. CSV, TSV, JSON, JSONL, XML, YAML, TOML, Parquet, Avro, Protocol Buffers, MessagePack, Excel, PDF, Word, PowerPoint, HTML, Markdown, SQLite, PostgreSQL dump, MySQL dump, GeoJSON, KML, GPX, .eml, .ics, .vcf, log formats (syslog, Apache, Nginx, JSON lines), and more. Every tool handles a few. None handles all.

**The Solution:** A format registry with auto-detection. Give AgenticData ANY file and it identifies the format, selects the right parser, and produces a Universal Schema representation. New formats are added as parser plugins — the core engine doesn't change.

**Key Concepts:**
- MagicBytes detection: first N bytes identify binary formats
- HeaderPattern detection: first line/lines identify text formats
- ContentSampling: statistical analysis for ambiguous formats
- ParserRegistry: extensible map of format → parser
- FormatConfidence: "92% CSV, 8% TSV" based on delimiter frequency

**MCP Tools:**
```
data_format_detect      Detect format of unknown file/data
data_format_list        List all supported formats
data_format_convert     Convert between any two supported formats
data_format_register    Register a new format parser (plugin)
data_format_stats       Statistics about format detection accuracy
```

---

#### INVENTION 3: DEEP DOCUMENT COMPREHENSION

**The Problem:** PDF "parsers" extract text. They lose tables, headers, footnotes, cross-references, page structure. Word "parsers" extract text. They lose styles, tracked changes, comments, embedded objects. The STRUCTURE is the meaning.

**The Solution:** Parse documents into a Document Object Model (DOM) that preserves ALL structure — paragraphs, tables, lists, headers, footers, images, cross-references, styles, metadata. Then query the DOM, not the raw text.

**Key Concepts:**
- DocumentDOM: hierarchical representation of any document
- TableExtractor: extract tables with headers, spanning cells, nested tables
- CrossReferenceResolver: "Section 3.2 references Table 5" → linked
- StyleSemantic: "bold + larger font = heading" → structural inference
- EmbeddedObjectExtractor: images, charts, diagrams → extracted + described

**MCP Tools:**
```
data_doc_parse          Parse any document into DOM
data_doc_tables         Extract all tables from document
data_doc_headers        Extract document structure (headings hierarchy)
data_doc_search         Search within document by content or structure
data_doc_compare        Compare two document versions (structural diff)
data_doc_metadata       Extract all metadata (author, dates, properties)
```

---

#### INVENTION 4: DATA SOUL EXTRACTION

**The Problem:** Enterprise systems have millions of rows across hundreds of tables. Nobody understands what the data MEANS anymore. The original developers are gone. The documentation is wrong. The column named "flag7" could mean anything.

**The Solution:** Analyze the data itself to extract its "soul" — what each field represents, how fields relate, what the business rules are, what the data patterns imply. Like reverse-engineering a codebase, but for data.

**Key Concepts:**
- FieldSemantics: "flag7 is always 0 or 1, correlates with is_active → it's a boolean status flag"
- RelationshipDiscovery: "order.customer_id matches customer.id in 99.7% of rows → foreign key"
- BusinessRuleExtraction: "price is always quantity × unit_price → derived column"
- DataDictionary: auto-generated human-readable explanation of every field
- TemporalPattern: "this table gets 1000 inserts/day on weekdays, 50 on weekends → business hours data"

**MCP Tools:**
```
data_soul_extract       Analyze dataset and extract meaning
data_soul_dictionary    Generate data dictionary for a schema
data_soul_rules         Discover business rules from data patterns
data_soul_relationships Discover inter-table relationships
data_soul_report        Generate full data understanding report
```

---

### TRANSFORMATION INVENTIONS (Changing data from one form to another)

#### INVENTION 5: LOSSLESS TRANSFORMATION PIPELINE

**The Problem:** ETL tools transform data but lose metadata. Which rows were dropped? Why? What was the original value before the transformation? When something goes wrong downstream, you can't trace back to the source.

**The Solution:** Every transformation is recorded. Every row tracks its lineage. Every dropped record is accounted for. The transformation pipeline is auditable, reversible, and reproducible.

**Key Concepts:**
- TransformationReceipt: every step records input hash, output hash, transform applied
- RowLineage: every output row knows its source row(s) and the path through the pipeline
- DropLog: every dropped/filtered record is logged with the reason
- ReversibleTransform: transformations that can be undone (round-trip guarantee)
- PipelineCheckpoint: save pipeline state at any step for debugging

**MCP Tools:**
```
data_transform_apply    Apply a transformation pipeline
data_transform_create   Create a new transformation step
data_transform_chain    Chain multiple transforms into a pipeline
data_transform_lineage  Trace any output row back to its source
data_transform_reverse  Reverse a transformation pipeline
data_transform_audit    Get full audit trail of all transformations
```

---

#### INVENTION 6: CROSS-FORMAT BRIDGE

**The Problem:** "Export this database table as a PDF report with charts." "Import this Excel file into PostgreSQL." These require not just format conversion but SEMANTIC bridging — understanding that column A in Excel maps to field B in the database, and that a chart should be generated from the revenue column.

**The Solution:** Semantic bridges between any two formats. Not just byte-level conversion but meaning-preserving transformation. A database row becomes a PDF table row with correct formatting. An Excel chart becomes a SQL query that produces the same visualization.

**Key Concepts:**
- SemanticMapping: "Excel column 'Revenue ($M)' → SQL column 'revenue_millions' (DECIMAL)"
- FormatCapability: what each format can represent (PDF has pages, CSV doesn't)
- LossReport: when conversion loses information, report exactly what was lost
- BidirectionalBridge: convert A→B→A and verify round-trip fidelity
- TemplateEngine: format-specific templates for beautiful output (PDF reports, Excel dashboards)

**MCP Tools:**
```
data_bridge_convert     Convert with semantic mapping between formats
data_bridge_map         Define a semantic mapping between two schemas
data_bridge_preview     Preview conversion before executing
data_bridge_loss        Report what information would be lost in conversion
data_bridge_template    Use a template for formatted output
```

---

#### INVENTION 7: MEDIA ALCHEMY

**The Problem:** Video is data. Audio is data. Images are data. But they're treated as opaque blobs. "Resize this image" requires ImageMagick. "Trim this audio" requires ffmpeg. "Extract frames from video" requires a media pipeline. None of these integrate with data understanding.

**The Solution:** Media as structured data. An image has dimensions, color profile, EXIF data, detected objects, text (OCR). A video has frames, audio tracks, subtitles, scene boundaries. Audio has waveform, frequency spectrum, speech-to-text. Treat media like any other structured data — queryable, transformable, analyzable.

**Key Concepts:**
- MediaSchema: structured representation of media properties
- MediaTransform: resize, crop, rotate, trim, merge, overlay, filter
- MediaExtract: frames from video, text from image (OCR), speech from audio
- MediaAnalyze: object detection, scene classification, sentiment from audio
- MediaPipeline: chain transforms like data transforms (with lineage)

**MCP Tools:**
```
data_media_analyze      Analyze media file (dimensions, duration, properties)
data_media_transform    Apply transformation (resize, crop, trim, convert)
data_media_extract      Extract data from media (OCR, speech-to-text, frames)
data_media_pipeline     Chain media transformations
data_media_metadata     Read/write media metadata (EXIF, ID3, etc.)
```

---

### QUALITY INVENTIONS (Ensuring data is correct and trustworthy)

#### INVENTION 8: DATA IMMUNE SYSTEM

**The Problem:** Bad data silently corrupts everything downstream. A single wrong decimal point in a financial dataset can cascade through reports, models, and decisions. There's no immune system that detects and quarantines bad data before it spreads.

**The Solution:** Continuous data quality monitoring. Every dataset has health metrics. Anomalies are detected and quarantined. Quality degrades gradually (not silently) with alerts at each threshold.

**Key Concepts:**
- HealthScore: 0-100 quality score per dataset/column
- AnomalyDetector: statistical outliers, pattern breaks, null spikes
- QuarantineZone: suspicious records isolated, not deleted
- QualityTimeline: health score over time (is quality improving or degrading?)
- ValidationRules: user-defined constraints (age > 0, email contains @)

**MCP Tools:**
```
data_quality_score      Score dataset quality (0-100)
data_quality_anomaly    Detect anomalies in dataset
data_quality_quarantine Quarantine suspicious records
data_quality_timeline   Quality score over time
data_quality_rules      Define/check validation rules
data_quality_heal       Suggest fixes for quality issues
```

---

#### INVENTION 9: TEMPORAL DATA ARCHAEOLOGY

**The Problem:** "What did the data look like last Tuesday?" Most systems overwrite in place. The history is lost. When something goes wrong, you can't go back to see what changed and when.

**The Solution:** Every version of every record is preserved. Query any point in time. Diff any two versions. Understand WHEN data changed, WHAT changed, and (combined with Data Soul) WHY it changed.

**Key Concepts:**
- VersionedRecord: every update creates a new version, old version preserved
- TemporalQuery: "SELECT * FROM orders AS OF '2026-03-01'" 
- VersionDiff: "what changed between version 3 and version 7?"
- ChangeStream: subscribe to changes in real-time
- ArchaeologyDig: reconstruct data state at any historical point

**MCP Tools:**
```
data_temporal_snapshot  Get data state at any point in time
data_temporal_diff      Diff two versions of a dataset
data_temporal_history   List all versions of a record
data_temporal_stream    Subscribe to data changes
data_temporal_restore   Restore data to a previous version
```

---

#### INVENTION 10: DATA DNA

**The Problem:** Where did this data come from? Which system generated it? Who touched it? What transformations were applied? Current lineage tools track table-level flows but not record-level or field-level lineage.

**The Solution:** Every piece of data carries its DNA — complete provenance from birth to current state. Field-level lineage, not just table-level. Every transformation, every merge, every derivation tracked.

**Key Concepts:**
- FieldLineage: "orders.total = orders.quantity × products.price, computed 2026-03-01"
- SourceAttribution: "this record came from api.example.com at 14:32:05 UTC"
- TransformChain: complete history of every transformation applied
- TrustScore: lineage depth affects trust (direct source = high, 5th-hand = low)
- DNAFingerprint: unique hash that identifies a piece of data across systems

**MCP Tools:**
```
data_dna_trace          Trace complete lineage of any data
data_dna_fingerprint    Generate unique fingerprint for data
data_dna_trust          Score trustworthiness based on lineage
data_dna_source         Identify original source of data
data_dna_impact         If this source changes, what's affected?
```

---

### INTELLIGENCE INVENTIONS (Reasoning about data)

#### INVENTION 11: PREDICTIVE SCHEMA EVOLUTION

**The Problem:** Schemas change. New columns appear. Types change. Relationships shift. Each change breaks downstream consumers. There's no way to predict or prepare for schema changes.

**The Solution:** Analyze schema history and data patterns to predict upcoming schema changes. "Column X has been growing by one enum value per quarter → expect a new value soon." "Table Y is approaching 2B rows → expect partition or archival."

**Key Concepts:**
- SchemaHistory: complete history of every schema change
- EvolutionPattern: recurring patterns in how schemas change
- PredictedChange: "80% confidence that a new column will be added for GDPR by Q4"
- MigrationPrep: auto-generate migration scripts for predicted changes
- BreakageAlert: "when this change happens, these 5 consumers will break"

**MCP Tools:**
```
data_evolve_predict     Predict upcoming schema changes
data_evolve_history     View schema change history
data_evolve_impact      Assess impact of a proposed schema change
data_evolve_migrate     Generate migration scripts
data_evolve_simulate    Simulate a schema change against real data
```

---

#### INVENTION 12: CROSS-DATASET REASONING

**The Problem:** Related data lives in different databases, files, and systems. "How does our customer data relate to our product data relate to our support tickets?" requires a human to manually join across sources.

**The Solution:** Automatically discover and reason about relationships ACROSS datasets. Join customer.csv with orders.db with support_tickets.json. Find correlations nobody asked about. Build a unified data graph from disparate sources.

**Key Concepts:**
- CrossSourceJoin: join datasets from different formats/locations
- RelationshipDiscovery: auto-detect how datasets relate
- CorrelationMining: "customers who buy X also file Y type of support tickets"
- UnifiedGraph: build a queryable graph across all connected datasets
- FederatedQuery: query across multiple data sources in one statement

**MCP Tools:**
```
data_cross_discover     Discover relationships across datasets
data_cross_join         Join data from multiple sources
data_cross_correlate    Find correlations across datasets
data_cross_graph        Build unified data graph
data_cross_query        Federated query across sources
```

---

#### INVENTION 13: QUERY PROPHECY

**The Problem:** "I need to know X" but the user doesn't know what query to write. SQL is hard. XPath is arcane. jq is a puzzle. Natural language to structured query is the holy grail but current tools hallucinate SQL.

**The Solution:** Generate queries from natural language WITH verification. The user says "which customers bought more than $1000 last month?" and gets a SQL query, a preview of results, a confidence score, AND an explanation of what the query does in plain English — before it executes.

**Key Concepts:**
- NaturalLanguageQuery: "show me all overdue invoices" → SQL
- QueryPreview: execute on sample data first, show preview
- QueryExplanation: "This query selects from invoices where due_date < today and status != 'paid'"
- QueryConfidence: "95% confident this is what you meant"
- QueryHistory: learn from past queries to improve future generation

**MCP Tools:**
```
data_query_natural      Generate query from natural language
data_query_explain      Explain what a query does in plain English
data_query_preview      Preview query results on sample data
data_query_optimize     Suggest optimizations for a query
data_query_history      Learn from past queries
```

---

#### INVENTION 14: ANOMALY CONSTELLATION

**The Problem:** Single anomalies are easy to detect (outlier). Correlated anomalies across multiple fields/datasets reveal systemic issues but are invisible to point anomaly detectors.

**The Solution:** Detect PATTERNS of anomalies across datasets. "Revenue dropped AND customer churn spiked AND support tickets increased" is a constellation that means something different than any individual anomaly.

**Key Concepts:**
- ConstellationDetector: find correlated anomaly groups
- CausalChain: "A caused B which caused C" anomaly paths
- ImpactRadius: how many downstream metrics are affected
- AlertPriority: constellations are higher priority than individual anomalies
- RootCause: trace the constellation back to the originating event

**MCP Tools:**
```
data_anomaly_constellation  Detect correlated anomaly patterns
data_anomaly_root_cause     Trace anomalies to root cause
data_anomaly_impact         Assess impact radius of anomaly
data_anomaly_predict        Predict likely future anomalies
data_anomaly_subscribe      Subscribe to anomaly alerts
```

---

### SPATIAL INVENTIONS (Understanding location and geometry)

#### INVENTION 15: GEOSPATIAL CONSCIOUSNESS

**The Problem:** Location is data's most common hidden dimension. Addresses are strings. Coordinates are numbers. Neither tells you "this is a hospital" or "these two addresses are in the same neighborhood." Spatial reasoning requires specialized GIS tools.

**The Solution:** Native geospatial understanding. Parse addresses, coordinates, regions. Calculate distances, containment, proximity. Cluster by location. Enrich with context (what's nearby, what region, what timezone).

**Key Concepts:**
- LocationParser: "123 Main St, NYC" → lat/lng → region → timezone
- SpatialQuery: "find all records within 5km of this point"
- RegionContainment: "is this address in Manhattan?"
- ProximityCluster: group records by spatial proximity
- SpatialEnrichment: add timezone, region, country, nearby landmarks

**MCP Tools:**
```
data_geo_parse          Parse location from text/coordinates
data_geo_distance       Calculate distance between locations
data_geo_contains       Check if point is within region
data_geo_cluster        Cluster records by location
data_geo_enrich         Add spatial context to records
data_geo_query          Spatial query (within, intersects, nearest)
```

---

### SECURITY INVENTIONS (Protecting data)

#### INVENTION 16: DATA VAULT

**The Problem:** Encryption tools encrypt files. They don't encrypt columns. They don't encrypt specific fields. They don't support querying encrypted data. You either encrypt everything (can't query) or nothing (not secure).

**Key Concepts:**
- FieldLevelEncryption: encrypt specific columns/fields, leave others queryable
- EncryptedQuery: search encrypted fields without decrypting all data
- KeyRotation: rotate encryption keys without re-encrypting all data
- AccessAudit: who accessed what decrypted data and when
- ZeroKnowledgeProof: prove a record meets criteria without revealing the record

**MCP Tools:**
```
data_vault_encrypt      Encrypt specific fields/columns
data_vault_decrypt      Decrypt with access logging
data_vault_query        Query encrypted data
data_vault_rotate       Rotate encryption keys
data_vault_audit        Audit decryption access
```

---

#### INVENTION 17: DATA REDACTION INTELLIGENCE

**The Problem:** GDPR, HIPAA, PCI — regulations require redacting sensitive data. Current tools use regex for SSN/email patterns. They miss context-dependent PII ("John lives on Oak Street" — the name + address is PII, neither alone is).

**The Solution:** Context-aware PII detection and redaction. Understand that "Dr. Smith's patient" is medical PII even though "Dr. Smith" alone isn't. Detect PII by context, not just pattern.

**Key Concepts:**
- ContextualPII: detect PII based on surrounding context
- RedactionPolicy: configurable rules per regulation (GDPR, HIPAA, PCI)
- SelectiveRedaction: redact PII but preserve data utility (anonymize, pseudonymize)
- RedactionAudit: log what was redacted, when, under what policy
- SyntheticReplacement: replace real PII with realistic synthetic data

**MCP Tools:**
```
data_redact_detect      Detect PII in dataset
data_redact_apply       Apply redaction policy
data_redact_policy      Configure redaction rules
data_redact_synthetic   Replace PII with synthetic data
data_redact_audit       Audit redaction history
```

---

### COLLABORATION INVENTIONS (Data across boundaries)

#### INVENTION 18: DATA FEDERATION

**The Problem:** Data lives in silos. Marketing has their database. Engineering has theirs. Finance has Excel files. Querying across silos requires expensive data warehouses and months of integration.

**The Solution:** Federated queries across ANY data source without moving data. Query marketing.db AND engineering.api AND finance.xlsx in one statement. The data stays where it is. Only the query results move.

**Key Concepts:**
- FederatedSource: register any data source (database, API, file, stream)
- CrossSourceQuery: "SELECT m.campaign, e.deploys, f.revenue FROM marketing m JOIN engineering e JOIN finance f"
- QueryPushdown: push filters to the source (don't fetch everything)
- CacheLayer: cache frequently queried cross-source joins
- SourceHealthCheck: monitor source availability and freshness

**MCP Tools:**
```
data_federate_register  Register a data source
data_federate_query     Federated query across sources
data_federate_health    Check source availability
data_federate_cache     Manage cross-source cache
data_federate_lineage   Track cross-source data lineage
```

---

#### INVENTION 19: DATA VERSIONING (Git for Data)

**The Problem:** Code has Git. Data has nothing. "Who changed this row?" "What did the table look like last month?" "Can we roll back that data migration?" These questions have no good answers.

**The Solution:** Git semantics for data. Branch, commit, diff, merge, revert — but for datasets. Every change is a commit. Every commit has a message. Branches allow parallel experiments. Merges reconcile changes.

**Key Concepts:**
- DataCommit: snapshot of data state with message and author
- DataBranch: parallel data experiments (A/B testing, migration testing)
- DataDiff: "these 47 rows changed, these 3 columns were added"
- DataMerge: reconcile two branches (conflict resolution for data)
- DataRevert: undo any commit or range of commits

**MCP Tools:**
```
data_version_commit     Commit current data state
data_version_branch     Create/switch data branches
data_version_diff       Diff between commits/branches
data_version_merge      Merge data branches
data_version_revert     Revert to previous commit
data_version_log        View commit history
```

---

### PROPHETIC INVENTIONS (Seeing data's future)

#### INVENTION 20: PREDICTIVE DATA QUALITY

**The Problem:** Data quality degrades slowly. By the time someone notices, months of bad data have accumulated. There's no early warning system.

**The Solution:** Predict quality degradation before it happens. "Null rate in column X has been increasing 0.5% per week → will hit 10% threshold in 6 weeks." "Data freshness is decreasing → source may be failing."

**MCP Tools:**
```
data_predict_quality    Predict future data quality
data_predict_volume     Predict data volume trends
data_predict_schema     Predict schema evolution
data_predict_cost       Predict storage/compute cost trends
```

---

#### INVENTION 21: DATA DREAM STATE

**The Problem:** Data sits idle most of the time. Nobody's querying it. Nobody's checking quality. Opportunities and problems go unnoticed until someone asks.

**The Solution:** Like Memory's dream state — when idle, analyze data for patterns, anomalies, relationships, and insights. Surface discoveries proactively.

**MCP Tools:**
```
data_dream_start        Start idle analysis
data_dream_insights     Get discovered insights
data_dream_patterns     Get discovered patterns
data_dream_health       Get proactive health report
```

---

#### INVENTION 22: SYNTHETIC DATA GENESIS

**The Problem:** Testing requires realistic data. Production data has PII. Generating fake data that preserves statistical properties and relationships is hard.

**The Solution:** Generate statistically faithful synthetic data. Same distributions. Same correlations. Same edge cases. Zero real PII.

**MCP Tools:**
```
data_synthetic_generate  Generate synthetic data from schema
data_synthetic_validate  Verify synthetic matches real distributions
data_synthetic_edge      Generate edge case data for testing
data_synthetic_scale     Generate data at any scale (1K to 1B rows)
```

---

#### INVENTION 23: DATA METABOLISM

**The Problem:** Datasets grow forever. Old data takes up space, slows queries, and is rarely accessed. There's no intelligent lifecycle management.

**The Solution:** Like Memory's metabolism — automatically tier, compress, archive, and eventually tombstone data based on access patterns, age, and value.

**MCP Tools:**
```
data_metabolism_status   Get data lifecycle status
data_metabolism_tier     Move data between hot/warm/cold tiers
data_metabolism_compress Compress old data
data_metabolism_archive  Archive to long-term storage
data_metabolism_policy   Configure lifecycle policies
```

---

#### INVENTION 24: DATA COLLECTIVE INTELLIGENCE

**The Problem:** Every organization solves the same data problems independently. "How do I clean phone numbers?" "What's the best way to parse addresses?" There's no shared learning.

**The Solution:** A collective where AgenticData instances share learned patterns — schema mappings, transformation recipes, quality rules — with attribution and trust scoring. Opt-in, privacy-preserving.

**MCP Tools:**
```
data_collective_share    Share a learned pattern
data_collective_search   Search for community patterns
data_collective_apply    Apply a community pattern
data_collective_rate     Rate a community pattern
data_collective_private  Ensure no private data leaks into collective
```

---

## THE .adat FILE FORMAT

```
┌─────────────────────────────────────────┐
│  HEADER                                  │
│  Magic: b"ADAT", version, counts         │
│  Schema count, Source count, Record count │
├─────────────────────────────────────────┤
│  SCHEMA REGISTRY                         │
│  Universal Schema definitions            │
│  One entry per known schema              │
├─────────────────────────────────────────┤
│  SOURCE TABLE                            │
│  Registered data sources                 │
│  Connection strings (encrypted)          │
├─────────────────────────────────────────┤
│  RECORD TABLE                            │
│  Fixed-size records with content offsets  │
│  O(1) access by ID                       │
├─────────────────────────────────────────┤
│  CONTENT BLOCK                           │
│  Variable-length compressed content      │
│  LZ4 compressed                          │
├─────────────────────────────────────────┤
│  LINEAGE GRAPH                           │
│  Field-level provenance chains           │
│  Transform receipts                      │
├─────────────────────────────────────────┤
│  QUALITY INDEX                           │
│  Health scores per source/column         │
│  Anomaly history                         │
├─────────────────────────────────────────┤
│  TEMPORAL INDEX                          │
│  Version history                         │
│  Point-in-time query support             │
├─────────────────────────────────────────┤
│  SPATIAL INDEX                           │
│  R-tree for geospatial queries           │
│  Coordinate lookups                      │
├─────────────────────────────────────────┤
│  ENCRYPTION BLOCK                        │
│  Key metadata (not keys themselves)      │
│  Field-level encryption pointers         │
└─────────────────────────────────────────┘
```

Binary format. Memory-mappable. Little-endian. BLAKE3 checksums.
Same design principles as .amem — one file, self-contained, portable.

---

## CRATE STRUCTURE

```
agentic-data/
├── Cargo.toml                   # workspace
├── LICENSE                      # MIT
├── README.md                    # Canonical layout
├── CONTRIBUTING.md
├── SECURITY.md
├── CHANGELOG.md
├── Makefile
├── crates/
│   ├── agentic-data/            # Core library (the engine)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs           # Public API
│   │       ├── types/
│   │       │   ├── mod.rs
│   │       │   ├── schema.rs    # Universal Schema types
│   │       │   ├── record.rs    # Record types
│   │       │   ├── source.rs    # Data source types
│   │       │   ├── lineage.rs   # Lineage/DNA types
│   │       │   ├── quality.rs   # Quality metric types
│   │       │   ├── spatial.rs   # Geospatial types
│   │       │   └── error.rs     # Error types
│   │       ├── format/
│   │       │   ├── mod.rs
│   │       │   ├── writer.rs    # Write .adat files
│   │       │   ├── reader.rs    # Read .adat files
│   │       │   ├── mmap.rs      # Memory-mapped access
│   │       │   └── compression.rs
│   │       ├── parser/
│   │       │   ├── mod.rs       # Parser registry
│   │       │   ├── csv.rs       # CSV/TSV parser
│   │       │   ├── json.rs      # JSON/JSONL parser
│   │       │   ├── xml.rs       # XML parser
│   │       │   ├── yaml.rs      # YAML/TOML parser
│   │       │   ├── excel.rs     # XLSX parser
│   │       │   ├── pdf.rs       # PDF parser (structural)
│   │       │   ├── docx.rs      # Word parser
│   │       │   ├── html.rs      # HTML parser
│   │       │   ├── sql.rs       # SQL dump parser
│   │       │   ├── log.rs       # Log format parser
│   │       │   ├── email.rs     # .eml parser
│   │       │   ├── calendar.rs  # .ics parser
│   │       │   ├── geo.rs       # GeoJSON/KML/GPX parser
│   │       │   ├── media.rs     # Media metadata parser
│   │       │   └── detect.rs    # Auto-detection
│   │       ├── engine/
│   │       │   ├── mod.rs
│   │       │   ├── ingest.rs    # Data ingestion pipeline
│   │       │   ├── query.rs     # Query executor
│   │       │   ├── transform.rs # Transformation engine
│   │       │   ├── quality.rs   # Quality analysis engine
│   │       │   └── federation.rs # Federated query engine
│   │       ├── index/
│   │       │   ├── mod.rs
│   │       │   ├── schema_index.rs    # Index by schema
│   │       │   ├── temporal_index.rs  # Index by time
│   │       │   ├── quality_index.rs   # Index by quality score
│   │       │   ├── spatial_index.rs   # R-tree for geo
│   │       │   └── lineage_index.rs   # Index by provenance
│   │       ├── crypto/
│   │       │   ├── mod.rs
│   │       │   ├── field_encrypt.rs   # Field-level encryption
│   │       │   ├── key_mgmt.rs        # Key management
│   │       │   └── redaction.rs       # PII detection + redaction
│   │       └── media/
│   │           ├── mod.rs
│   │           ├── image.rs     # Image operations
│   │           ├── audio.rs     # Audio operations
│   │           └── video.rs     # Video operations
│   │
│   ├── agentic-data-mcp/       # MCP server
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs          # MCP entry point
│   │       └── tools/
│   │           ├── mod.rs
│   │           ├── registry.rs  # Tool dispatch
│   │           ├── schema.rs    # Schema tools
│   │           ├── format.rs    # Format tools
│   │           ├── document.rs  # Document tools
│   │           ├── soul.rs      # Data soul tools
│   │           ├── transform.rs # Transform tools
│   │           ├── bridge.rs    # Cross-format tools
│   │           ├── media.rs     # Media tools
│   │           ├── quality.rs   # Quality tools
│   │           ├── temporal.rs  # Temporal tools
│   │           ├── dna.rs       # Lineage/DNA tools
│   │           ├── predict.rs   # Predictive tools
│   │           ├── cross.rs     # Cross-dataset tools
│   │           ├── query.rs     # Natural language query
│   │           ├── geo.rs       # Geospatial tools
│   │           ├── vault.rs     # Encryption tools
│   │           ├── redact.rs    # Redaction tools
│   │           ├── federate.rs  # Federation tools
│   │           ├── version.rs   # Version control tools
│   │           ├── dream.rs     # Dream state tools
│   │           ├── synthetic.rs # Synthetic data tools
│   │           ├── metabolism.rs # Lifecycle tools
│   │           └── collective.rs # Collective intelligence
│   │
│   ├── agentic-data-cli/       # CLI binary
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   │
│   └── agentic-data-ffi/       # FFI bindings
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
│
├── docs/
│   ├── ecosystem/
│   │   └── CANONICAL_SISTER_KIT.md    # Byte-identical copy
│   └── public/
│       ├── overview.md
│       ├── quickstart.md
│       ├── concepts.md
│       ├── installation.md
│       ├── guide.md
│       ├── api-reference.md
│       ├── benchmarks.md
│       ├── faq.md
│       ├── file-format.md
│       ├── integration-guide.md
│       ├── experience-with-vs-without.md
│       ├── command-surface.md
│       ├── SCENARIOS-AGENTIC-DATA.md
│       ├── architecture.md
│       ├── cli-reference.md
│       ├── configuration.md
│       ├── ffi-reference.md
│       ├── mcp-tools.md
│       ├── mcp-resources.md
│       ├── mcp-prompts.md
│       └── troubleshooting.md
│
├── assets/
│   ├── github-hero-pane.svg
│   ├── github-hero-pane-agentra.svg
│   ├── github-terminal-pane.svg
│   └── github-terminal-pane-agentra.svg
│
├── paper/
│   └── paper-i-data-comprehension/
│       ├── paper.tex
│       └── references.bib
│
├── installer/
│   └── adat_installer/
│
├── scripts/
│   ├── install.sh
│   ├── check-install-commands.sh
│   └── check-canonical-sister.sh
│
├── .github/workflows/
│   ├── ci.yml
│   ├── install-command-guardrails.yml
│   └── canonical-sister-guardrails.yml
│
├── tests/
│   ├── phase1_types.rs
│   ├── phase2_parsers.rs
│   ├── phase3_engine.rs
│   ├── phase4_indexes.rs
│   ├── phase5_crypto.rs
│   ├── phase6_mcp.rs
│   └── phase7_integration.rs
│
├── benches/
│   └── benchmarks.rs
│
└── examples/
    ├── basic_parse.rs
    ├── database_migration.rs
    ├── pdf_table_extract.rs
    └── federated_query.rs
```

---

## DEPENDENCIES

```toml
[dependencies]
# Core
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1"
uuid = { version = "1", features = ["v4", "serde"] }

# File format
lz4_flex = "0.11"
blake3 = "1"
memmap2 = "0.9"

# Parsers
csv = "1"
quick-xml = "0.36"
calamine = "0.26"           # Excel
pdf-extract = "0.7"         # PDF
zip = "2"                   # DOCX (it's a zip)
scraper = "0.20"            # HTML

# Database
rusqlite = { version = "0.32", features = ["bundled"] }

# Crypto
ring = "0.17"               # ChaCha20-Poly1305, Argon2

# Geospatial
geo = "0.28"                # Geo types + algorithms
geojson = "0.24"

# MCP server
tokio = { version = "1", features = ["full"] }

# CLI
clap = { version = "4", features = ["derive", "env"] }

# Media (optional feature)
# ffmpeg is called via Command::new, not linked
```

---

## BUILD ORDER

```
PHASE 1: Foundation (types + file format)
  1. SPEC-PROJECT-STRUCTURE — create directory
  2. SPEC-DATA-STRUCTURES — types/schema.rs, types/record.rs, types/error.rs
  3. SPEC-FILE-FORMAT — format/writer.rs, format/reader.rs
  4. Tests: phase1_types.rs

PHASE 2: Parsers
  5. parser/detect.rs — auto-detection
  6. parser/csv.rs, parser/json.rs, parser/xml.rs, parser/yaml.rs
  7. parser/excel.rs, parser/pdf.rs, parser/docx.rs, parser/html.rs
  8. parser/sql.rs, parser/log.rs, parser/email.rs, parser/calendar.rs
  9. parser/geo.rs, parser/media.rs
  10. Tests: phase2_parsers.rs

PHASE 3: Core Engine
  11. engine/ingest.rs — data ingestion pipeline
  12. engine/query.rs — query executor
  13. engine/transform.rs — transformation engine
  14. engine/quality.rs — quality analysis
  15. Tests: phase3_engine.rs

PHASE 4: Indexes
  16. index/schema_index.rs, index/temporal_index.rs
  17. index/quality_index.rs, index/spatial_index.rs
  18. index/lineage_index.rs
  19. Tests: phase4_indexes.rs

PHASE 5: Crypto + Security
  20. crypto/field_encrypt.rs
  21. crypto/key_mgmt.rs
  22. crypto/redaction.rs
  23. Tests: phase5_crypto.rs

PHASE 6: MCP Server (all tool handlers)
  24. tools/registry.rs — dispatch
  25. All 24 tool module files
  26. Tests: phase6_mcp.rs

PHASE 7: CLI + FFI + Integration
  27. CLI commands
  28. FFI bindings
  29. Integration tests
  30. Benchmarks
  31. Research paper

PHASE 8: Compliance
  32. All docs/public/ files
  33. Installer (3 profiles)
  34. CI workflows
  35. SVG assets
  36. README canonical layout
  37. scripts/check-canonical-sister.sh — must pass
```

---

## MCP TOOL SUMMARY (80 tools across 24 inventions)

```
Schema:      5 tools (infer, align, validate, evolve, merge)
Format:      5 tools (detect, list, convert, register, stats)
Document:    6 tools (parse, tables, headers, search, compare, metadata)
Soul:        5 tools (extract, dictionary, rules, relationships, report)
Transform:   6 tools (apply, create, chain, lineage, reverse, audit)
Bridge:      5 tools (convert, map, preview, loss, template)
Media:       5 tools (analyze, transform, extract, pipeline, metadata)
Quality:     6 tools (score, anomaly, quarantine, timeline, rules, heal)
Temporal:    5 tools (snapshot, diff, history, stream, restore)
DNA:         5 tools (trace, fingerprint, trust, source, impact)
Predict:     4 tools (quality, volume, schema, cost)
Cross:       5 tools (discover, join, correlate, graph, query)
Query:       5 tools (natural, explain, preview, optimize, history)
Anomaly:     5 tools (constellation, root_cause, impact, predict, subscribe)
Geo:         6 tools (parse, distance, contains, cluster, enrich, query)
Vault:       5 tools (encrypt, decrypt, query, rotate, audit)
Redact:      5 tools (detect, apply, policy, synthetic, audit)
Federate:    5 tools (register, query, health, cache, lineage)
Version:     6 tools (commit, branch, diff, merge, revert, log)
Dream:       4 tools (start, insights, patterns, health)
Synthetic:   4 tools (generate, validate, edge, scale)
Metabolism:  5 tools (status, tier, compress, archive, policy)
Collective:  5 tools (share, search, apply, rate, private)
Evolution:   5 tools (predict, history, impact, migrate, simulate)

TOTAL: ~131 MCP tools
```

---

## TEST TARGETS

```
Phase 1: 30+ tests (types, file format read/write)
Phase 2: 50+ tests (each parser: valid input, malformed input, edge cases)
Phase 3: 40+ tests (ingestion, query, transform, quality)
Phase 4: 30+ tests (each index type)
Phase 5: 20+ tests (encryption, redaction)
Phase 6: 50+ tests (MCP protocol compliance, each tool)
Phase 7: 30+ tests (CLI, FFI, end-to-end)

TOTAL: 250+ tests minimum (Memory has 291+)
```

---

## SUCCESS CRITERIA

Same as Memory's standard:

1. All tests pass with zero failures
2. cargo clippy — zero warnings
3. cargo fmt --check — passes
4. All files under 400 lines
5. check-canonical-sister.sh — passes
6. MCP protocol compliance (two-tier error handling, -32803)
7. Tool descriptions verb-first, no trailing periods
8. 3 install profiles work (desktop, terminal, server)
9. Research paper compiles to PDF
10. Benchmarks produce real data
11. .adat file format is portable (read on any platform)
12. CANONICAL_SISTER_KIT.md byte-identical to Memory's
13. README follows canonical layout
14. All 8 standard reference doc pages present
