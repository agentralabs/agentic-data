//! Parser registry — extensible format detection and parsing.
//!
//! Invention 2: Format Omniscience. Give AgenticData ANY file and it identifies
//! the format, selects the right parser, and produces Universal Schema records.

pub mod detect;
pub mod csv_parser;
pub mod json_parser;
pub mod xml_parser;
pub mod yaml_parser;
pub mod html_parser;
pub mod sql_parser;
pub mod log_parser;
pub mod email_parser;
pub mod calendar_parser;
pub mod geo_parser;
pub mod media_parser;

use crate::types::{AdatResult, AdatError, UniversalSchema, DataRecord};

/// Detected format with confidence.
#[derive(Debug, Clone)]
pub struct FormatDetection {
    pub format: DataFormat,
    pub confidence: f64,
    pub details: String,
}

/// All supported data formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataFormat {
    Csv,
    Tsv,
    Json,
    JsonLines,
    Xml,
    Yaml,
    Toml,
    Html,
    Sql,
    Log,
    Email,
    Calendar,
    GeoJson,
    Kml,
    Gpx,
    Markdown,
    Unknown,
}

impl DataFormat {
    /// Human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Csv => "CSV",
            Self::Tsv => "TSV",
            Self::Json => "JSON",
            Self::JsonLines => "JSON Lines",
            Self::Xml => "XML",
            Self::Yaml => "YAML",
            Self::Toml => "TOML",
            Self::Html => "HTML",
            Self::Sql => "SQL",
            Self::Log => "Log",
            Self::Email => "Email (EML)",
            Self::Calendar => "Calendar (ICS)",
            Self::GeoJson => "GeoJSON",
            Self::Kml => "KML",
            Self::Gpx => "GPX",
            Self::Markdown => "Markdown",
            Self::Unknown => "Unknown",
        }
    }

    /// Common file extensions for this format.
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Self::Csv => &["csv"],
            Self::Tsv => &["tsv", "tab"],
            Self::Json => &["json"],
            Self::JsonLines => &["jsonl", "ndjson"],
            Self::Xml => &["xml"],
            Self::Yaml => &["yaml", "yml"],
            Self::Toml => &["toml"],
            Self::Html => &["html", "htm"],
            Self::Sql => &["sql"],
            Self::Log => &["log"],
            Self::Email => &["eml"],
            Self::Calendar => &["ics", "ical"],
            Self::GeoJson => &["geojson"],
            Self::Kml => &["kml"],
            Self::Gpx => &["gpx"],
            Self::Markdown => &["md", "markdown"],
            Self::Unknown => &[],
        }
    }
}

/// Result of parsing a data source.
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// Inferred schema.
    pub schema: UniversalSchema,
    /// Parsed records.
    pub records: Vec<DataRecord>,
    /// Number of records that failed to parse.
    pub errors: usize,
    /// Warnings during parsing.
    pub warnings: Vec<String>,
}

/// List all supported formats.
pub fn supported_formats() -> Vec<DataFormat> {
    vec![
        DataFormat::Csv, DataFormat::Tsv, DataFormat::Json, DataFormat::JsonLines,
        DataFormat::Xml, DataFormat::Yaml, DataFormat::Toml, DataFormat::Html,
        DataFormat::Sql, DataFormat::Log, DataFormat::Email, DataFormat::Calendar,
        DataFormat::GeoJson, DataFormat::Kml, DataFormat::Gpx, DataFormat::Markdown,
    ]
}

/// Parse data from a string, auto-detecting format.
pub fn parse_auto(data: &str, source_name: &str) -> AdatResult<ParseResult> {
    let detection = detect::detect_format(data, None);
    parse_as(data, source_name, detection.format)
}

/// Parse data as a specific format.
pub fn parse_as(data: &str, source_name: &str, format: DataFormat) -> AdatResult<ParseResult> {
    match format {
        DataFormat::Csv => csv_parser::parse(data, source_name, b','),
        DataFormat::Tsv => csv_parser::parse(data, source_name, b'\t'),
        DataFormat::Json => json_parser::parse(data, source_name),
        DataFormat::JsonLines => json_parser::parse_lines(data, source_name),
        DataFormat::Xml => xml_parser::parse(data, source_name),
        DataFormat::Yaml => yaml_parser::parse_yaml(data, source_name),
        DataFormat::Toml => yaml_parser::parse_toml(data, source_name),
        DataFormat::Html => html_parser::parse(data, source_name),
        DataFormat::Sql => sql_parser::parse(data, source_name),
        DataFormat::Log => log_parser::parse(data, source_name),
        DataFormat::Email => email_parser::parse(data, source_name),
        DataFormat::Calendar => calendar_parser::parse(data, source_name),
        DataFormat::GeoJson => geo_parser::parse_geojson(data, source_name),
        DataFormat::Kml => geo_parser::parse_kml(data, source_name),
        DataFormat::Gpx => geo_parser::parse_gpx(data, source_name),
        DataFormat::Markdown => {
            // Markdown parsed as plain text with structure
            html_parser::parse_markdown(data, source_name)
        }
        DataFormat::Unknown => Err(AdatError::FormatDetection(
            "Cannot parse unknown format".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_formats_count() {
        assert!(supported_formats().len() >= 16);
    }

    #[test]
    fn test_format_names() {
        assert_eq!(DataFormat::Csv.name(), "CSV");
        assert_eq!(DataFormat::GeoJson.name(), "GeoJSON");
    }

    #[test]
    fn test_format_extensions() {
        assert!(DataFormat::Json.extensions().contains(&"json"));
        assert!(DataFormat::Yaml.extensions().contains(&"yml"));
    }
}
