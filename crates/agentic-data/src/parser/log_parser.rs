//! Log format parser — handles timestamped log entries.

use std::collections::HashMap;
use crate::types::*;
use super::ParseResult;

/// Parse log data into structured records.
pub fn parse(data: &str, source_name: &str) -> AdatResult<ParseResult> {
    let mut records = Vec::new();
    let mut errors = 0;

    for line in data.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }

        let entry = parse_log_line(trimmed);
        let mut fields = HashMap::new();
        fields.insert("timestamp".into(), serde_json::json!(entry.timestamp));
        fields.insert("level".into(), serde_json::json!(entry.level));
        fields.insert("message".into(), serde_json::json!(entry.message));
        if let Some(ref src) = entry.source {
            fields.insert("source".into(), serde_json::json!(src));
        }
        fields.insert("raw".into(), serde_json::json!(trimmed));
        records.push(DataRecord::new(source_name, "log_entry", fields));
    }

    let fields = vec![
        SchemaField::inferred("timestamp", FieldType::DateTime, 0.80),
        SchemaField::inferred("level", FieldType::Text, 0.90),
        SchemaField::inferred("message", FieldType::Text, 1.0),
        SchemaField::inferred("source", FieldType::Text, 0.60),
        SchemaField::inferred("raw", FieldType::Text, 1.0),
    ];

    let mut schema = UniversalSchema::new(source_name);
    schema.nodes.push(SchemaNode {
        name: "log_entry".into(), source: source_name.into(),
        fields, record_count: Some(records.len() as u64),
    });

    Ok(ParseResult { schema, records, errors, warnings: Vec::new() })
}

struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    source: Option<String>,
}

fn parse_log_line(line: &str) -> LogEntry {
    // Try common log formats
    // Format: [2026-03-13 10:00:00] LEVEL message
    if line.starts_with('[') {
        if let Some(bracket_end) = line.find(']') {
            let ts = &line[1..bracket_end];
            let rest = line[bracket_end + 1..].trim();
            let (level, msg) = extract_level_message(rest);
            return LogEntry { timestamp: ts.to_string(), level, message: msg, source: None };
        }
    }

    // Format: 2026-03-13T10:00:00Z LEVEL message
    if line.len() > 19 && (line.as_bytes()[4] == b'-' || line.as_bytes()[4] == b'/') {
        let ts_end = line.find(|c: char| c == ' ' || c == '\t').unwrap_or(19).max(19);
        let ts = &line[..ts_end];
        let rest = line[ts_end..].trim();
        let (level, msg) = extract_level_message(rest);
        return LogEntry { timestamp: ts.to_string(), level, message: msg, source: None };
    }

    // Format: LEVEL message (no timestamp)
    let (level, msg) = extract_level_message(line);
    LogEntry { timestamp: String::new(), level, message: msg, source: None }
}

fn extract_level_message(text: &str) -> (String, String) {
    let levels = ["ERROR", "WARN", "WARNING", "INFO", "DEBUG", "TRACE", "FATAL", "CRITICAL"];
    let upper = text.to_uppercase();
    for level in &levels {
        if upper.starts_with(level) {
            let msg = text[level.len()..].trim().trim_start_matches(':').trim().to_string();
            return (level.to_string(), msg);
        }
    }
    ("UNKNOWN".into(), text.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bracketed_log() {
        let data = "[2026-03-13 10:00] INFO Starting\n[2026-03-13 10:01] ERROR Failed\n";
        let result = parse(data, "test").unwrap();
        assert_eq!(result.records.len(), 2);
        assert_eq!(result.records[0].get_str("level"), Some("INFO"));
        assert_eq!(result.records[1].get_str("level"), Some("ERROR"));
    }

    #[test]
    fn test_parse_iso_timestamp() {
        let data = "2026-03-13T10:00:00Z INFO Started\n";
        let result = parse(data, "test").unwrap();
        assert_eq!(result.records.len(), 1);
    }

    #[test]
    fn test_extract_level() {
        let (level, msg) = extract_level_message("ERROR connection refused");
        assert_eq!(level, "ERROR");
        assert_eq!(msg, "connection refused");
    }
}
