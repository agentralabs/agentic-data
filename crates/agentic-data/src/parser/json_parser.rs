//! JSON / JSON Lines parser — handles objects, arrays, and NDJSON.

use std::collections::HashMap;
use crate::types::*;
use super::ParseResult;

/// Parse JSON data (object or array of objects).
pub fn parse(data: &str, source_name: &str) -> AdatResult<ParseResult> {
    let value: serde_json::Value = serde_json::from_str(data)
        .map_err(|e| AdatError::Parse(format!("Invalid JSON: {}", e)))?;

    match value {
        serde_json::Value::Array(arr) => parse_array(&arr, source_name),
        serde_json::Value::Object(_) => parse_array(&[value], source_name),
        _ => Err(AdatError::Parse("JSON must be an object or array".into())),
    }
}

/// Parse JSON Lines (one JSON object per line).
pub fn parse_lines(data: &str, source_name: &str) -> AdatResult<ParseResult> {
    let mut objects = Vec::new();
    let mut warnings = Vec::new();
    for (i, line) in data.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }
        match serde_json::from_str::<serde_json::Value>(trimmed) {
            Ok(v) => objects.push(v),
            Err(e) => warnings.push(format!("Line {}: {}", i + 1, e)),
        }
    }
    let mut result = parse_array(&objects, source_name)?;
    result.warnings.extend(warnings);
    Ok(result)
}

/// Parse an array of JSON values into records + schema.
fn parse_array(values: &[serde_json::Value], source_name: &str) -> AdatResult<ParseResult> {
    let mut records = Vec::new();
    let mut all_keys: Vec<String> = Vec::new();
    let mut type_samples: HashMap<String, Vec<String>> = HashMap::new();
    let mut errors = 0;

    for value in values {
        if let serde_json::Value::Object(map) = value {
            let mut fields = HashMap::new();
            for (key, val) in map {
                if !all_keys.contains(key) {
                    all_keys.push(key.clone());
                }
                fields.insert(key.clone(), val.clone());
                let samples = type_samples.entry(key.clone()).or_default();
                if samples.len() < 5 {
                    samples.push(format!("{}", val));
                }
            }
            records.push(DataRecord::new(source_name, source_name, fields));
        } else {
            errors += 1;
        }
    }

    // Build schema from observed keys
    let fields: Vec<SchemaField> = all_keys.iter().map(|key| {
        let samples = type_samples.get(key).cloned().unwrap_or_default();
        let field_type = infer_json_type(values, key);
        let mut f = SchemaField::inferred(key, field_type, 0.90);
        f.sample_values = samples;
        f
    }).collect();

    let mut schema = UniversalSchema::new(source_name);
    schema.nodes.push(SchemaNode {
        name: source_name.to_string(),
        source: source_name.to_string(),
        fields,
        record_count: Some(records.len() as u64),
    });

    Ok(ParseResult { schema, records, errors, warnings: Vec::new() })
}

/// Infer the type of a JSON field across multiple objects.
fn infer_json_type(values: &[serde_json::Value], key: &str) -> FieldType {
    let mut types = Vec::new();
    for v in values {
        if let Some(field) = v.get(key) {
            match field {
                serde_json::Value::Null => {}
                serde_json::Value::Bool(_) => types.push("bool"),
                serde_json::Value::Number(n) => {
                    if n.is_i64() { types.push("int"); } else { types.push("float"); }
                }
                serde_json::Value::String(_) => types.push("string"),
                serde_json::Value::Array(_) => types.push("array"),
                serde_json::Value::Object(_) => types.push("object"),
            }
        }
    }
    if types.is_empty() { return FieldType::Null; }
    let most_common = types.iter().max_by_key(|t| types.iter().filter(|x| x == t).count()).unwrap();
    match *most_common {
        "bool" => FieldType::Boolean,
        "int" => FieldType::Integer,
        "float" => FieldType::Float,
        "array" => FieldType::Array(Box::new(FieldType::Unknown)),
        "object" => FieldType::Object(Box::new(UniversalSchema::new("nested"))),
        _ => FieldType::Text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_object() {
        let result = parse(r#"{"name":"Alice","age":30}"#, "test").unwrap();
        assert_eq!(result.records.len(), 1);
        assert_eq!(result.schema.nodes[0].fields.len(), 2);
    }

    #[test]
    fn test_parse_array() {
        let result = parse(r#"[{"a":1},{"a":2,"b":"x"}]"#, "test").unwrap();
        assert_eq!(result.records.len(), 2);
        // Schema should have both "a" and "b"
        assert_eq!(result.schema.nodes[0].fields.len(), 2);
    }

    #[test]
    fn test_parse_jsonl() {
        let data = "{\"a\":1}\n{\"a\":2}\n{\"a\":3}\n";
        let result = parse_lines(data, "test").unwrap();
        assert_eq!(result.records.len(), 3);
    }

    #[test]
    fn test_type_inference() {
        let result = parse(r#"[{"n":1,"s":"hi","b":true,"f":1.5}]"#, "test").unwrap();
        let fields = &result.schema.nodes[0].fields;
        let find = |name: &str| fields.iter().find(|f| f.name == name).unwrap();
        assert_eq!(find("n").field_type, FieldType::Integer);
        assert_eq!(find("s").field_type, FieldType::Text);
        assert_eq!(find("b").field_type, FieldType::Boolean);
        assert_eq!(find("f").field_type, FieldType::Float);
    }

    #[test]
    fn test_invalid_json() {
        assert!(parse("not json", "test").is_err());
    }
}
