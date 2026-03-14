//! XML parser — extracts structure from XML documents.

use std::collections::HashMap;
use crate::types::*;
use super::ParseResult;

/// Parse XML data into records based on repeating elements.
pub fn parse(data: &str, source_name: &str) -> AdatResult<ParseResult> {
    // Simple XML parser: extract elements as key-value records
    let mut records = Vec::new();
    let mut current_fields: HashMap<String, serde_json::Value> = HashMap::new();
    let mut all_keys: Vec<String> = Vec::new();
    let mut depth = 0u32;
    let mut current_tag = String::new();
    let mut in_content = false;
    let mut content_buf = String::new();
    let mut root_child_tag = String::new();

    for line in data.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("<?") { continue; }

        // Opening tag
        if trimmed.starts_with('<') && !trimmed.starts_with("</") && !trimmed.ends_with("/>") {
            let tag = extract_tag_name(trimmed);
            depth += 1;
            if depth == 2 && root_child_tag.is_empty() {
                root_child_tag = tag.clone();
            }
            if depth == 2 && !current_fields.is_empty() {
                records.push(DataRecord::new(source_name, source_name, current_fields.clone()));
                current_fields.clear();
            }
            current_tag = tag;
            in_content = true;
            content_buf.clear();
        }
        // Self-closing tag
        else if trimmed.ends_with("/>") {
            let tag = extract_tag_name(trimmed);
            if depth >= 2 && !tag.is_empty() {
                if !all_keys.contains(&tag) { all_keys.push(tag.clone()); }
                current_fields.insert(tag, serde_json::Value::Null);
            }
        }
        // Closing tag
        else if trimmed.starts_with("</") {
            if in_content && depth >= 3 && !current_tag.is_empty() {
                let val = content_buf.trim().to_string();
                if !all_keys.contains(&current_tag) { all_keys.push(current_tag.clone()); }
                current_fields.insert(current_tag.clone(), parse_xml_value(&val));
            }
            depth = depth.saturating_sub(1);
            in_content = false;
        }
        // Content between tags
        else if in_content {
            content_buf.push_str(trimmed);
        }
        // Inline content: <tag>value</tag>
        if let Some((tag, val)) = extract_inline_tag(trimmed) {
            if depth >= 2 {
                if !all_keys.contains(&tag) { all_keys.push(tag.clone()); }
                current_fields.insert(tag, parse_xml_value(&val));
            }
        }
    }
    // Last record
    if !current_fields.is_empty() {
        records.push(DataRecord::new(source_name, source_name, current_fields));
    }

    // Build schema
    let fields: Vec<SchemaField> = all_keys.iter().map(|k| {
        SchemaField::inferred(k, FieldType::Text, 0.70)
    }).collect();

    let mut schema = UniversalSchema::new(source_name);
    schema.nodes.push(SchemaNode {
        name: if root_child_tag.is_empty() { source_name.to_string() } else { root_child_tag },
        source: source_name.to_string(),
        fields,
        record_count: Some(records.len() as u64),
    });

    Ok(ParseResult { schema, records, errors: 0, warnings: Vec::new() })
}

fn extract_tag_name(tag_str: &str) -> String {
    let s = tag_str.trim_start_matches('<').trim_end_matches('>').trim_end_matches('/');
    s.split_whitespace().next().unwrap_or("").to_string()
}

fn extract_inline_tag(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if !trimmed.starts_with('<') || trimmed.starts_with("<?") || trimmed.starts_with("</") { return None; }
    let open_end = trimmed.find('>')?;
    let close_start = trimmed.rfind("</")?;
    if close_start <= open_end { return None; }
    let tag = extract_tag_name(&trimmed[..open_end + 1]);
    let val = trimmed[open_end + 1..close_start].to_string();
    if tag.is_empty() || val.is_empty() { return None; }
    Some((tag, val))
}

fn parse_xml_value(val: &str) -> serde_json::Value {
    if val.is_empty() { return serde_json::Value::Null; }
    if let Ok(n) = val.parse::<i64>() { return serde_json::json!(n); }
    if let Ok(f) = val.parse::<f64>() { return serde_json::json!(f); }
    if val == "true" || val == "false" { return serde_json::json!(val == "true"); }
    serde_json::json!(val)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_xml() {
        let xml = "<root>\n<item>\n<name>Alice</name>\n<age>30</age>\n</item>\n<item>\n<name>Bob</name>\n<age>25</age>\n</item>\n</root>";
        let result = parse(xml, "test").unwrap();
        assert!(result.records.len() >= 1); // At least one record extracted
        assert!(!result.schema.nodes.is_empty());
    }

    #[test]
    fn test_inline_tags() {
        let (tag, val) = extract_inline_tag("<name>Alice</name>").unwrap();
        assert_eq!(tag, "name");
        assert_eq!(val, "Alice");
    }

    #[test]
    fn test_empty_xml() {
        let result = parse("<root></root>", "test").unwrap();
        assert!(result.records.is_empty());
    }
}
