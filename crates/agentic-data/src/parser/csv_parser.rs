//! CSV/TSV parser — handles comma and tab-delimited data with schema inference.

use std::collections::HashMap;
use crate::types::*;
use super::ParseResult;

/// Parse CSV or TSV data.
pub fn parse(data: &str, source_name: &str, delimiter: u8) -> AdatResult<ParseResult> {
    let mut lines = data.lines();
    let header_line = lines.next()
        .ok_or_else(|| AdatError::Parse("Empty CSV data".into()))?;

    let delim = delimiter as char;
    let headers: Vec<&str> = header_line.split(delim).map(|h| h.trim().trim_matches('"')).collect();
    if headers.is_empty() {
        return Err(AdatError::Parse("No columns in CSV header".into()));
    }

    let mut records = Vec::new();
    let mut type_samples: Vec<Vec<String>> = vec![Vec::new(); headers.len()];
    let mut null_counts: Vec<u64> = vec![0; headers.len()];
    let mut errors = 0;
    let mut warnings = Vec::new();

    for (line_num, line) in lines.enumerate() {
        if line.trim().is_empty() { continue; }
        let values: Vec<&str> = line.split(delim).map(|v| v.trim().trim_matches('"')).collect();

        if values.len() != headers.len() {
            warnings.push(format!("Line {}: expected {} columns, got {}", line_num + 2, headers.len(), values.len()));
            errors += 1;
            continue;
        }

        let mut fields = HashMap::new();
        for (i, val) in values.iter().enumerate() {
            if val.is_empty() {
                null_counts[i] += 1;
                fields.insert(headers[i].to_string(), serde_json::Value::Null);
            } else if let Ok(n) = val.parse::<i64>() {
                fields.insert(headers[i].to_string(), serde_json::json!(n));
            } else if let Ok(f) = val.parse::<f64>() {
                fields.insert(headers[i].to_string(), serde_json::json!(f));
            } else if *val == "true" || *val == "false" {
                fields.insert(headers[i].to_string(), serde_json::json!(val == &"true"));
            } else {
                fields.insert(headers[i].to_string(), serde_json::json!(val));
            }
            if type_samples[i].len() < 5 {
                type_samples[i].push(val.to_string());
            }
        }

        records.push(DataRecord::new(source_name, source_name, fields));
    }

    // Infer schema from collected samples
    let total_rows = records.len() as u64;
    let fields: Vec<SchemaField> = headers.iter().enumerate().map(|(i, name)| {
        let field_type = infer_type(&type_samples[i]);
        let mut f = SchemaField::inferred(name, field_type, 0.85);
        f.nullable = null_counts[i] > 0;
        f.sample_values = type_samples[i].clone();
        if total_rows > 0 {
            f.stats = Some(FieldStats {
                min: 0.0, max: 0.0, mean: 0.0,
                null_count: null_counts[i], distinct_count: 0,
                total_count: total_rows,
            });
        }
        f
    }).collect();

    let mut schema = UniversalSchema::new(source_name);
    schema.nodes.push(SchemaNode {
        name: source_name.to_string(),
        source: source_name.to_string(),
        fields,
        record_count: Some(total_rows),
    });

    Ok(ParseResult { schema, records, errors, warnings })
}

/// Infer field type from sample values.
fn infer_type(samples: &[String]) -> FieldType {
    if samples.is_empty() { return FieldType::Unknown; }

    let all_int = samples.iter().all(|s| s.parse::<i64>().is_ok());
    if all_int { return FieldType::Integer; }

    let all_float = samples.iter().all(|s| s.parse::<f64>().is_ok());
    if all_float { return FieldType::Float; }

    let all_bool = samples.iter().all(|s| s == "true" || s == "false");
    if all_bool { return FieldType::Boolean; }

    // Date detection
    let date_patterns = samples.iter().all(|s| {
        s.len() >= 8 && (s.contains('-') || s.contains('/'))
            && s.chars().filter(|c| c.is_ascii_digit()).count() >= 4
    });
    if date_patterns { return FieldType::DateTime; }

    // Email detection
    let all_email = samples.iter().all(|s| s.contains('@') && s.contains('.'));
    if all_email { return FieldType::Email; }

    // URL detection
    let all_url = samples.iter().all(|s| s.starts_with("http://") || s.starts_with("https://"));
    if all_url { return FieldType::Url; }

    FieldType::Text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csv() {
        let data = "name,age,city\nAlice,30,NYC\nBob,25,LA\n";
        let result = parse(data, "test", b',').unwrap();
        assert_eq!(result.records.len(), 2);
        assert_eq!(result.schema.nodes[0].fields.len(), 3);
        assert_eq!(result.errors, 0);
    }

    #[test]
    fn test_parse_tsv() {
        let data = "name\tage\nAlice\t30\n";
        let result = parse(data, "test", b'\t').unwrap();
        assert_eq!(result.records.len(), 1);
    }

    #[test]
    fn test_type_inference() {
        let data = "id,name,active,email\n1,Alice,true,alice@test.com\n2,Bob,false,bob@test.com\n";
        let result = parse(data, "test", b',').unwrap();
        let fields = &result.schema.nodes[0].fields;
        assert_eq!(fields[0].field_type, FieldType::Integer);  // id
        assert_eq!(fields[1].field_type, FieldType::Text);     // name
        assert_eq!(fields[2].field_type, FieldType::Boolean);  // active
        assert_eq!(fields[3].field_type, FieldType::Email);    // email
    }

    #[test]
    fn test_column_mismatch() {
        let data = "a,b,c\n1,2\n3,4,5\n";
        let result = parse(data, "test", b',').unwrap();
        assert_eq!(result.records.len(), 1); // Only the valid row
        assert_eq!(result.errors, 1);
    }

    #[test]
    fn test_empty_csv() {
        let result = parse("", "test", b',');
        assert!(result.is_err());
    }
}
