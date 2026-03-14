//! Email (.eml) parser — extracts headers, body, and attachments.

use std::collections::HashMap;
use crate::types::*;
use super::ParseResult;

/// Parse an email message.
pub fn parse(data: &str, source_name: &str) -> AdatResult<ParseResult> {
    let mut headers = HashMap::new();
    let mut body = String::new();
    let mut in_body = false;
    let mut current_header = String::new();
    let mut current_value = String::new();

    for line in data.lines() {
        if in_body {
            body.push_str(line);
            body.push('\n');
            continue;
        }
        // Empty line separates headers from body
        if line.trim().is_empty() && !headers.is_empty() {
            if !current_header.is_empty() {
                headers.insert(current_header.to_lowercase(), current_value.trim().to_string());
            }
            in_body = true;
            continue;
        }
        // Continuation line (starts with whitespace)
        if line.starts_with(' ') || line.starts_with('\t') {
            current_value.push(' ');
            current_value.push_str(line.trim());
            continue;
        }
        // New header
        if !current_header.is_empty() {
            headers.insert(current_header.to_lowercase(), current_value.trim().to_string());
        }
        if let Some(colon) = line.find(':') {
            current_header = line[..colon].to_string();
            current_value = line[colon + 1..].trim().to_string();
        }
    }
    if !current_header.is_empty() && !in_body {
        headers.insert(current_header.to_lowercase(), current_value.trim().to_string());
    }

    let mut fields = HashMap::new();
    for (key, val) in &headers {
        fields.insert(key.clone(), serde_json::json!(val));
    }
    fields.insert("body".into(), serde_json::json!(body.trim()));

    let schema_fields = vec![
        SchemaField::inferred("from", FieldType::Email, 0.95),
        SchemaField::inferred("to", FieldType::Email, 0.95),
        SchemaField::inferred("subject", FieldType::Text, 0.95),
        SchemaField::inferred("date", FieldType::DateTime, 0.90),
        SchemaField::inferred("body", FieldType::Text, 1.0),
    ];

    let mut schema = UniversalSchema::new(source_name);
    schema.nodes.push(SchemaNode {
        name: "email".into(), source: source_name.into(),
        fields: schema_fields, record_count: Some(1),
    });

    Ok(ParseResult {
        schema,
        records: vec![DataRecord::new(source_name, "email", fields)],
        errors: 0, warnings: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_email() {
        let eml = "From: alice@example.com\nTo: bob@example.com\nSubject: Hello\nDate: Mon, 13 Mar 2026 10:00:00 +0000\n\nHi Bob,\nHow are you?\n";
        let result = parse(eml, "test").unwrap();
        assert_eq!(result.records.len(), 1);
        assert_eq!(result.records[0].get_str("subject"), Some("Hello"));
        assert_eq!(result.records[0].get_str("from"), Some("alice@example.com"));
    }

    #[test]
    fn test_multiline_header() {
        let eml = "From: alice@example.com\nSubject: A very long\n subject line\n\nBody\n";
        let result = parse(eml, "test").unwrap();
        assert!(result.records[0].get_str("subject").unwrap().contains("long"));
    }
}
