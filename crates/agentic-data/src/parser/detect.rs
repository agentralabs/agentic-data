//! Format auto-detection — identifies data format from content and extension.
//!
//! Uses magic bytes, header patterns, and statistical analysis.

use super::{DataFormat, FormatDetection};

/// Detect the format of data from its content and optional file extension.
pub fn detect_format(data: &str, extension: Option<&str>) -> FormatDetection {
    // 1. Extension-based detection (high confidence if matches content)
    if let Some(ext) = extension {
        if let Some(fmt) = detect_by_extension(ext) {
            return FormatDetection { format: fmt, confidence: 0.9, details: format!("Extension .{}", ext) };
        }
    }

    // 2. Content-based detection
    let trimmed = data.trim();
    if trimmed.is_empty() {
        return FormatDetection { format: DataFormat::Unknown, confidence: 0.0, details: "Empty input".into() };
    }

    // JSON Lines: multiple lines each starting with { (check before single JSON)
    let line_count = trimmed.lines().count();
    if line_count > 1 && trimmed.lines().take(5).all(|l| l.trim().starts_with('{')) {
        return FormatDetection { format: DataFormat::JsonLines, confidence: 0.90, details: "Multiple JSON objects per line".into() };
    }

    // GeoJSON detection (before generic JSON — it's JSON with specific structure)
    if trimmed.contains("\"type\"") && (trimmed.contains("\"FeatureCollection\"") || trimmed.contains("\"Feature\"")) {
        return FormatDetection { format: DataFormat::GeoJson, confidence: 0.92, details: "GeoJSON Feature".into() };
    }

    // JSON detection (single object or array)
    if (trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']')) {
        return FormatDetection { format: DataFormat::Json, confidence: 0.95, details: "JSON object/array".into() };
    }

    // XML/HTML detection
    if trimmed.starts_with("<?xml") || trimmed.starts_with("<xml") {
        // Check for specific XML subtypes
        if trimmed.contains("<kml") { return FormatDetection { format: DataFormat::Kml, confidence: 0.95, details: "KML root element".into() }; }
        if trimmed.contains("<gpx") { return FormatDetection { format: DataFormat::Gpx, confidence: 0.95, details: "GPX root element".into() }; }
        return FormatDetection { format: DataFormat::Xml, confidence: 0.95, details: "XML declaration".into() };
    }
    if trimmed.starts_with("<!DOCTYPE html") || trimmed.starts_with("<html") {
        return FormatDetection { format: DataFormat::Html, confidence: 0.95, details: "HTML document".into() };
    }
    if trimmed.starts_with('<') && trimmed.ends_with('>') {
        return FormatDetection { format: DataFormat::Xml, confidence: 0.70, details: "XML-like tags".into() };
    }

    // Email detection (before YAML — email headers have ": " patterns)
    if trimmed.starts_with("From:") || trimmed.starts_with("Subject:") || trimmed.starts_with("MIME-Version:") {
        return FormatDetection { format: DataFormat::Email, confidence: 0.90, details: "Email header".into() };
    }

    // Calendar detection
    if trimmed.starts_with("BEGIN:VCALENDAR") || trimmed.starts_with("BEGIN:VEVENT") {
        return FormatDetection { format: DataFormat::Calendar, confidence: 0.95, details: "iCalendar format".into() };
    }

    // YAML detection
    if trimmed.starts_with("---") || (trimmed.contains(": ") && !trimmed.contains(',') && trimmed.lines().count() > 2) {
        return FormatDetection { format: DataFormat::Yaml, confidence: 0.75, details: "YAML frontmatter or key-value".into() };
    }

    // TOML detection
    if trimmed.starts_with('[') && !trimmed.contains("] ") && trimmed.contains('=') {
        return FormatDetection { format: DataFormat::Toml, confidence: 0.70, details: "TOML section header".into() };
    }

    // SQL detection
    let upper = trimmed.to_uppercase();
    if upper.starts_with("CREATE TABLE") || upper.starts_with("INSERT INTO")
        || upper.starts_with("SELECT ") || upper.starts_with("DROP TABLE") {
        return FormatDetection { format: DataFormat::Sql, confidence: 0.90, details: "SQL statement".into() };
    }

    // Log detection (timestamped lines)
    let log_patterns = ["[20", "2024-", "2025-", "2026-", "INFO ", "WARN ", "ERROR ", "DEBUG "];
    let first_lines: Vec<&str> = trimmed.lines().take(5).collect();
    let log_matches = first_lines.iter().filter(|l| log_patterns.iter().any(|p| l.starts_with(p))).count();
    if log_matches >= 3 {
        return FormatDetection { format: DataFormat::Log, confidence: 0.85, details: "Timestamped log lines".into() };
    }

    // GeoJSON detection
    if trimmed.contains("\"type\"") && (trimmed.contains("\"FeatureCollection\"") || trimmed.contains("\"Feature\"")) {
        return FormatDetection { format: DataFormat::GeoJson, confidence: 0.90, details: "GeoJSON Feature".into() };
    }

    // Markdown detection
    if trimmed.starts_with('#') || trimmed.contains("\n## ") || trimmed.contains("\n- ") {
        return FormatDetection { format: DataFormat::Markdown, confidence: 0.60, details: "Markdown structure".into() };
    }

    // CSV/TSV detection (last resort — delimiter analysis)
    let comma_count: usize = first_lines.iter().map(|l| l.matches(',').count()).sum();
    let tab_count: usize = first_lines.iter().map(|l| l.matches('\t').count()).sum();
    if tab_count > comma_count && tab_count >= first_lines.len() {
        return FormatDetection { format: DataFormat::Tsv, confidence: 0.75, details: format!("Tab-delimited ({} tabs in {} lines)", tab_count, first_lines.len()) };
    }
    if comma_count >= first_lines.len() {
        return FormatDetection { format: DataFormat::Csv, confidence: 0.75, details: format!("Comma-delimited ({} commas in {} lines)", comma_count, first_lines.len()) };
    }

    FormatDetection { format: DataFormat::Unknown, confidence: 0.0, details: "No format detected".into() }
}

/// Detect format by file extension.
fn detect_by_extension(ext: &str) -> Option<DataFormat> {
    match ext.to_lowercase().as_str() {
        "csv" => Some(DataFormat::Csv),
        "tsv" | "tab" => Some(DataFormat::Tsv),
        "json" => Some(DataFormat::Json),
        "jsonl" | "ndjson" => Some(DataFormat::JsonLines),
        "xml" => Some(DataFormat::Xml),
        "yaml" | "yml" => Some(DataFormat::Yaml),
        "toml" => Some(DataFormat::Toml),
        "html" | "htm" => Some(DataFormat::Html),
        "sql" => Some(DataFormat::Sql),
        "log" => Some(DataFormat::Log),
        "eml" => Some(DataFormat::Email),
        "ics" | "ical" => Some(DataFormat::Calendar),
        "geojson" => Some(DataFormat::GeoJson),
        "kml" => Some(DataFormat::Kml),
        "gpx" => Some(DataFormat::Gpx),
        "md" | "markdown" => Some(DataFormat::Markdown),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_json() {
        let d = detect_format(r#"{"key": "value"}"#, None);
        assert_eq!(d.format, DataFormat::Json);
        assert!(d.confidence >= 0.9);
    }

    #[test]
    fn test_detect_csv() {
        let d = detect_format("name,age,city\nAlice,30,NYC\nBob,25,LA\n", None);
        assert_eq!(d.format, DataFormat::Csv);
    }

    #[test]
    fn test_detect_xml() {
        let d = detect_format("<?xml version=\"1.0\"?><root><item/></root>", None);
        assert_eq!(d.format, DataFormat::Xml);
    }

    #[test]
    fn test_detect_sql() {
        let d = detect_format("CREATE TABLE users (id INT, name TEXT);", None);
        assert_eq!(d.format, DataFormat::Sql);
    }

    #[test]
    fn test_detect_email() {
        let d = detect_format("From: alice@example.com\nTo: bob@example.com\nSubject: Hello\n\nBody", None);
        assert_eq!(d.format, DataFormat::Email);
    }

    #[test]
    fn test_detect_calendar() {
        let d = detect_format("BEGIN:VCALENDAR\nBEGIN:VEVENT\nSUMMARY:Meeting\nEND:VEVENT\nEND:VCALENDAR", None);
        assert_eq!(d.format, DataFormat::Calendar);
    }

    #[test]
    fn test_detect_by_extension() {
        let d = detect_format("anything", Some("csv"));
        assert_eq!(d.format, DataFormat::Csv);
        assert!(d.confidence >= 0.9);
    }

    #[test]
    fn test_detect_jsonl() {
        let d = detect_format("{\"a\":1}\n{\"a\":2}\n{\"a\":3}\n", None);
        assert_eq!(d.format, DataFormat::JsonLines);
    }

    #[test]
    fn test_detect_yaml() {
        let d = detect_format("---\nname: test\nversion: 1\nitems:\n  - one\n  - two\n", None);
        assert_eq!(d.format, DataFormat::Yaml);
    }

    #[test]
    fn test_detect_log() {
        let d = detect_format("[2026-03-13 10:00] INFO Starting\n[2026-03-13 10:01] ERROR Failed\n[2026-03-13 10:02] WARN Retry\n[2026-03-13 10:03] INFO Done\n", None);
        assert_eq!(d.format, DataFormat::Log);
    }

    #[test]
    fn test_detect_unknown() {
        let d = detect_format("", None);
        assert_eq!(d.format, DataFormat::Unknown);
    }
}
