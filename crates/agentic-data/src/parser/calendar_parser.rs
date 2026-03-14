//! iCalendar (.ics) parser — extracts events, todos, and calendar data.

use std::collections::HashMap;
use crate::types::*;
use super::ParseResult;

/// Parse iCalendar data.
pub fn parse(data: &str, source_name: &str) -> AdatResult<ParseResult> {
    let mut records = Vec::new();
    let mut current_fields: HashMap<String, serde_json::Value> = HashMap::new();
    let mut in_event = false;

    for line in data.lines() {
        let trimmed = line.trim();
        match trimmed {
            "BEGIN:VEVENT" | "BEGIN:VTODO" => {
                in_event = true;
                current_fields.clear();
                current_fields.insert("type".into(), serde_json::json!(
                    if trimmed.contains("VEVENT") { "event" } else { "todo" }
                ));
            }
            "END:VEVENT" | "END:VTODO" => {
                if in_event && !current_fields.is_empty() {
                    records.push(DataRecord::new(source_name, "calendar_entry", current_fields.clone()));
                }
                in_event = false;
            }
            _ if in_event => {
                if let Some(colon) = trimmed.find(':') {
                    let key = trimmed[..colon].split(';').next().unwrap_or("").to_lowercase();
                    let val = &trimmed[colon + 1..];
                    if !key.is_empty() {
                        current_fields.insert(key, serde_json::json!(val));
                    }
                }
            }
            _ => {}
        }
    }

    let fields = vec![
        SchemaField::inferred("type", FieldType::Text, 1.0),
        SchemaField::inferred("summary", FieldType::Text, 0.95),
        SchemaField::inferred("dtstart", FieldType::DateTime, 0.90),
        SchemaField::inferred("dtend", FieldType::DateTime, 0.90),
        SchemaField::inferred("location", FieldType::Text, 0.70),
        SchemaField::inferred("description", FieldType::Text, 0.70),
    ];

    let mut schema = UniversalSchema::new(source_name);
    schema.nodes.push(SchemaNode {
        name: "calendar_entry".into(), source: source_name.into(),
        fields, record_count: Some(records.len() as u64),
    });

    Ok(ParseResult { schema, records, errors: 0, warnings: Vec::new() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ics() {
        let ics = "BEGIN:VCALENDAR\nBEGIN:VEVENT\nSUMMARY:Meeting\nDTSTART:20260313T100000Z\nDTEND:20260313T110000Z\nLOCATION:Room A\nEND:VEVENT\nBEGIN:VEVENT\nSUMMARY:Lunch\nDTSTART:20260313T120000Z\nEND:VEVENT\nEND:VCALENDAR\n";
        let result = parse(ics, "test").unwrap();
        assert_eq!(result.records.len(), 2);
        assert_eq!(result.records[0].get_str("summary"), Some("Meeting"));
        assert_eq!(result.records[0].get_str("location"), Some("Room A"));
    }

    #[test]
    fn test_parse_todo() {
        let ics = "BEGIN:VCALENDAR\nBEGIN:VTODO\nSUMMARY:Buy groceries\nEND:VTODO\nEND:VCALENDAR\n";
        let result = parse(ics, "test").unwrap();
        assert_eq!(result.records.len(), 1);
        assert_eq!(result.records[0].get_str("type"), Some("todo"));
    }
}
