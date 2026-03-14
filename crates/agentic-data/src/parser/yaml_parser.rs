//! YAML and TOML parser — structured configuration and data formats.

use std::collections::HashMap;
use crate::types::*;
use super::ParseResult;

/// Parse YAML data.
pub fn parse_yaml(data: &str, source_name: &str) -> AdatResult<ParseResult> {
    // Parse YAML as JSON value (serde_json handles the conversion)
    let clean = data.trim().trim_start_matches("---").trim();
    // Simple YAML: key-value pairs → single record
    let mut fields = HashMap::new();
    let mut all_keys = Vec::new();

    for line in clean.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') { continue; }
        if let Some(colon_pos) = trimmed.find(':') {
            let key = trimmed[..colon_pos].trim().to_string();
            let val = trimmed[colon_pos + 1..].trim().to_string();
            if !key.is_empty() && !key.starts_with('-') {
                all_keys.push(key.clone());
                fields.insert(key, parse_yaml_value(&val));
            }
        }
    }

    let schema_fields: Vec<SchemaField> = all_keys.iter().map(|k| {
        SchemaField::inferred(k, FieldType::Text, 0.75)
    }).collect();

    let mut schema = UniversalSchema::new(source_name);
    schema.nodes.push(SchemaNode {
        name: source_name.to_string(),
        source: source_name.to_string(),
        fields: schema_fields,
        record_count: Some(1),
    });

    let records = if fields.is_empty() { vec![] } else {
        vec![DataRecord::new(source_name, source_name, fields)]
    };

    Ok(ParseResult { schema, records, errors: 0, warnings: Vec::new() })
}

/// Parse TOML data.
pub fn parse_toml(data: &str, source_name: &str) -> AdatResult<ParseResult> {
    let mut fields = HashMap::new();
    let mut all_keys = Vec::new();
    let mut current_section = String::new();

    for line in data.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') { continue; }
        // Section header
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_section = trimmed[1..trimmed.len() - 1].to_string();
            continue;
        }
        // Key-value pair
        if let Some(eq_pos) = trimmed.find('=') {
            let raw_key = trimmed[..eq_pos].trim();
            let val = trimmed[eq_pos + 1..].trim().trim_matches('"');
            let key = if current_section.is_empty() {
                raw_key.to_string()
            } else {
                format!("{}.{}", current_section, raw_key)
            };
            all_keys.push(key.clone());
            fields.insert(key, parse_yaml_value(val));
        }
    }

    let schema_fields: Vec<SchemaField> = all_keys.iter().map(|k| {
        SchemaField::inferred(k, FieldType::Text, 0.80)
    }).collect();

    let mut schema = UniversalSchema::new(source_name);
    schema.nodes.push(SchemaNode {
        name: source_name.to_string(),
        source: source_name.to_string(),
        fields: schema_fields,
        record_count: Some(1),
    });

    let records = if fields.is_empty() { vec![] } else {
        vec![DataRecord::new(source_name, source_name, fields)]
    };

    Ok(ParseResult { schema, records, errors: 0, warnings: Vec::new() })
}

fn parse_yaml_value(val: &str) -> serde_json::Value {
    if val.is_empty() || val == "~" || val == "null" { return serde_json::Value::Null; }
    if let Ok(n) = val.parse::<i64>() { return serde_json::json!(n); }
    if let Ok(f) = val.parse::<f64>() { return serde_json::json!(f); }
    if val == "true" || val == "false" { return serde_json::json!(val == "true"); }
    serde_json::json!(val.trim_matches('"').trim_matches('\''))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_yaml() {
        let data = "name: test\nversion: 1\nenabled: true\n";
        let result = parse_yaml(data, "test").unwrap();
        assert_eq!(result.records.len(), 1);
        assert_eq!(result.records[0].get_str("name"), Some("test"));
    }

    #[test]
    fn test_parse_toml() {
        let data = "[package]\nname = \"my-app\"\nversion = \"0.1.0\"\n";
        let result = parse_toml(data, "test").unwrap();
        assert_eq!(result.records.len(), 1);
    }

    #[test]
    fn test_yaml_with_frontmatter() {
        let data = "---\ntitle: My Doc\nauthor: Alice\n---\n";
        let result = parse_yaml(data, "test").unwrap();
        assert_eq!(result.records.len(), 1);
    }
}
