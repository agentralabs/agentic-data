//! AgenticData CLI — command-line interface for data comprehension.

use std::path::PathBuf;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "adat", version, about = "AgenticData — universal data comprehension")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Ingest data from a file
    Ingest {
        /// File path to ingest
        path: PathBuf,
        /// Force a specific format (auto-detect if omitted)
        #[arg(short, long)]
        format: Option<String>,
    },
    /// Detect the format of a file
    Detect {
        /// File path to detect
        path: PathBuf,
    },
    /// Query records from the store
    Query {
        /// Schema node to query
        node: String,
        /// Filter expression (field=value)
        #[arg(short, long)]
        filter: Option<String>,
        /// Max results
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    /// Show quality score for a dataset
    Quality {
        /// Schema node to score
        node: String,
    },
    /// Detect PII in a file
    Pii {
        /// File path to scan
        path: PathBuf,
    },
    /// Show data store summary
    Status,
    /// List all supported formats
    Formats,
    /// Show schema for ingested data
    Schema {
        /// Schema name or ID
        name: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    let mut store = agentic_data::DataStore::new();

    match cli.command {
        Commands::Ingest { path, format } => {
            let data = match std::fs::read_to_string(&path) {
                Ok(d) => d,
                Err(e) => { eprintln!("Error reading {}: {}", path.display(), e); return; }
            };
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("input");
            let mut engine = agentic_data::IngestEngine::new(&mut store);
            let result = if let Some(fmt) = format {
                let detection = agentic_data::parser::detect::detect_format(&data, Some(&fmt));
                engine.ingest_as(&data, name, detection.format)
            } else {
                engine.ingest_string(&data, name)
            };
            match result {
                Ok(r) => {
                    println!("Ingested: {} records, {} errors", r.records_added, r.parse_errors);
                    println!("Schema ID: {}", r.schema_id);
                    println!("Source ID: {}", r.source_id);
                    if !r.warnings.is_empty() {
                        println!("Warnings:");
                        for w in &r.warnings { println!("  {}", w); }
                    }
                }
                Err(e) => eprintln!("Ingest failed: {}", e),
            }
        }
        Commands::Detect { path } => {
            let data = match std::fs::read_to_string(&path) {
                Ok(d) => d,
                Err(e) => { eprintln!("Error: {}", e); return; }
            };
            let ext = path.extension().and_then(|e| e.to_str());
            let detection = agentic_data::parser::detect::detect_format(&data, ext);
            println!("Format:     {}", detection.format.name());
            println!("Confidence: {:.0}%", detection.confidence * 100.0);
            println!("Details:    {}", detection.details);
        }
        Commands::Query { node, filter, limit } => {
            let engine = agentic_data::QueryEngine::new(&store);
            let filters = if let Some(ref expr) = filter {
                parse_filter_expr(expr)
            } else {
                vec![]
            };
            let result = engine.query(&node, &filters, Some(limit), 0);
            println!("{} results (scanned {})", result.total_matched, result.total_scanned);
            for rec in &result.records {
                println!("  {} {:?}", rec.id, rec.fields);
            }
        }
        Commands::Quality { node } => {
            let score = agentic_data::QualityEngine::score(&store, &node);
            println!("Quality Score: {}/100", score.score);
            println!("  Completeness: {:.0}%", score.completeness * 100.0);
            println!("  Uniqueness:   {:.0}%", score.uniqueness * 100.0);
            println!("  Consistency:  {:.0}%", score.consistency * 100.0);
            println!("  Freshness:    {:.0}%", score.freshness * 100.0);
            println!("  Accuracy:     {:.0}%", score.accuracy * 100.0);
        }
        Commands::Pii { path } => {
            let data = match std::fs::read_to_string(&path) {
                Ok(d) => d,
                Err(e) => { eprintln!("Error: {}", e); return; }
            };
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("input");
            let mut engine = agentic_data::IngestEngine::new(&mut store);
            if let Ok(result) = engine.ingest_string(&data, name) {
                let records = store.active_records();
                let mut total_pii = 0;
                for rec in records {
                    let detections = agentic_data::RedactionEngine::detect(rec);
                    for d in &detections {
                        println!("  {:?} in field '{}' (confidence: {:.0}%)", d.pii_type, d.field, d.confidence * 100.0);
                        total_pii += 1;
                    }
                }
                println!("{} PII instances found in {} records", total_pii, result.records_added);
            }
        }
        Commands::Status => {
            println!("{}", store.summary());
        }
        Commands::Formats => {
            println!("Supported formats:");
            for fmt in agentic_data::parser::supported_formats() {
                let exts = fmt.extensions().join(", ");
                println!("  {:15} .{}", fmt.name(), exts);
            }
        }
        Commands::Schema { name } => {
            if let Some(ref n) = name {
                if let Some(schema) = store.find_schema_by_name(n) {
                    println!("Schema: {} (v{})", schema.name, schema.version);
                    for node in &schema.nodes {
                        println!("  {} ({} fields, {} records)", node.name, node.fields.len(), node.record_count.unwrap_or(0));
                        for f in &node.fields {
                            println!("    {:20} {:?} (confidence: {:.0}%)", f.name, f.field_type, f.confidence * 100.0);
                        }
                    }
                } else {
                    println!("Schema '{}' not found", n);
                }
            } else {
                let schemas = store.all_schemas();
                if schemas.is_empty() { println!("No schemas loaded. Use 'adat ingest <file>' first."); }
                for s in schemas {
                    println!("  {} — {} nodes, {} fields", s.name, s.nodes.len(), s.total_fields());
                }
            }
        }
    }
}

fn parse_filter_expr(expr: &str) -> Vec<agentic_data::engine::QueryFilter> {
    // Simple "field=value" parser
    if let Some(eq) = expr.find('=') {
        let field = expr[..eq].trim().to_string();
        let value = expr[eq + 1..].trim();
        vec![agentic_data::engine::QueryFilter {
            field,
            op: agentic_data::engine::FilterOp::Eq,
            value: serde_json::json!(value),
        }]
    } else {
        vec![]
    }
}
