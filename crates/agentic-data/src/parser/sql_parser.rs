//! SQL dump parser — extracts schema and records from SQL statements.

use std::collections::HashMap;
use crate::types::*;
use super::ParseResult;

/// Parse SQL dump (CREATE TABLE + INSERT INTO statements).
pub fn parse(data: &str, source_name: &str) -> AdatResult<ParseResult> {
    let mut schema = UniversalSchema::new(source_name);
    let mut records = Vec::new();
    let mut current_table = String::new();
    let mut warnings = Vec::new();

    for line in data.lines() {
        let trimmed = line.trim();
        let upper = trimmed.to_uppercase();

        // CREATE TABLE
        if upper.starts_with("CREATE TABLE") {
            if let Some(table_name) = extract_table_name(trimmed) {
                current_table = table_name.clone();
                let columns = extract_columns(data, &table_name);
                let fields: Vec<SchemaField> = columns.iter().map(|(name, sql_type)| {
                    SchemaField::inferred(name, sql_type_to_field(sql_type), 0.95)
                }).collect();
                schema.nodes.push(SchemaNode {
                    name: table_name, source: source_name.into(),
                    fields, record_count: None,
                });
            }
        }

        // INSERT INTO
        if upper.starts_with("INSERT INTO") {
            if let Some((table, values)) = extract_insert(trimmed) {
                let node = schema.find_node(&table);
                let field_names: Vec<String> = node.map(|n| n.fields.iter().map(|f| f.name.clone()).collect())
                    .unwrap_or_else(|| (0..values.len()).map(|i| format!("col{}", i)).collect());

                let mut fields = HashMap::new();
                for (i, val) in values.iter().enumerate() {
                    let key = field_names.get(i).cloned().unwrap_or_else(|| format!("col{}", i));
                    fields.insert(key, parse_sql_value(val));
                }
                records.push(DataRecord::new(source_name, &table, fields));
            }
        }
    }

    // Update record counts
    for node in &mut schema.nodes {
        let count = records.iter().filter(|r| r.schema_node == node.name).count();
        node.record_count = Some(count as u64);
    }

    Ok(ParseResult { schema, records, errors: 0, warnings })
}

fn extract_table_name(line: &str) -> Option<String> {
    let upper = line.to_uppercase();
    let pos = upper.find("CREATE TABLE")? + "CREATE TABLE".len();
    let rest = line[pos..].trim().trim_start_matches("IF NOT EXISTS").trim();
    let name = rest.split(|c: char| c.is_whitespace() || c == '(')
        .next()?
        .trim_matches(|c| c == '`' || c == '"' || c == '[' || c == ']');
    if name.is_empty() { None } else { Some(name.to_string()) }
}

fn extract_columns(data: &str, table_name: &str) -> Vec<(String, String)> {
    let mut cols = Vec::new();
    let upper = data.to_uppercase();
    let needle = format!("CREATE TABLE");
    let mut in_table = false;

    for line in data.lines() {
        let trimmed = line.trim();
        if trimmed.to_uppercase().contains(&needle) && trimmed.contains(table_name) {
            in_table = true;
            continue;
        }
        if in_table {
            if trimmed.starts_with(')') { break; }
            let trimmed = trimmed.trim_start_matches('(').trim().trim_end_matches(',');
            if trimmed.is_empty() { continue; }
            let upper_trimmed = trimmed.to_uppercase();
            if upper_trimmed.starts_with("PRIMARY") || upper_trimmed.starts_with("FOREIGN")
                || upper_trimmed.starts_with("UNIQUE") || upper_trimmed.starts_with("INDEX")
                || upper_trimmed.starts_with("CONSTRAINT") { continue; }
            let parts: Vec<&str> = trimmed.splitn(3, char::is_whitespace).collect();
            if parts.len() >= 2 {
                let name = parts[0].trim_matches(|c| c == '`' || c == '"');
                let sql_type = parts[1].trim_matches(|c| c == '(' || c == ')');
                cols.push((name.to_string(), sql_type.to_uppercase()));
            }
        }
    }
    cols
}

fn sql_type_to_field(sql_type: &str) -> FieldType {
    let upper = sql_type.to_uppercase();
    if upper.contains("INT") { return FieldType::Integer; }
    if upper.contains("FLOAT") || upper.contains("REAL") || upper.contains("DOUBLE") || upper.contains("DECIMAL") { return FieldType::Float; }
    if upper.contains("BOOL") { return FieldType::Boolean; }
    if upper.contains("DATE") || upper.contains("TIME") { return FieldType::DateTime; }
    if upper.contains("BLOB") || upper.contains("BINARY") { return FieldType::Binary; }
    if upper.contains("UUID") { return FieldType::Uuid; }
    FieldType::Text
}

fn extract_insert(line: &str) -> Option<(String, Vec<String>)> {
    let upper = line.to_uppercase();
    let into_pos = upper.find("INTO")? + 4;
    let rest = line[into_pos..].trim();
    let table_end = rest.find(|c: char| c.is_whitespace() || c == '(')?;
    let table = rest[..table_end].trim_matches(|c| c == '`' || c == '"').to_string();
    let values_pos = upper.find("VALUES")? + 6;
    let values_str = line[values_pos..].trim().trim_start_matches('(').trim_end_matches(|c| c == ')' || c == ';');
    let values: Vec<String> = values_str.split(',')
        .map(|v| v.trim().trim_matches('\'').trim_matches('"').to_string())
        .collect();
    Some((table, values))
}

fn parse_sql_value(val: &str) -> serde_json::Value {
    if val == "NULL" || val == "null" { return serde_json::Value::Null; }
    if let Ok(n) = val.parse::<i64>() { return serde_json::json!(n); }
    if let Ok(f) = val.parse::<f64>() { return serde_json::json!(f); }
    serde_json::json!(val)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sql() {
        let sql = "CREATE TABLE users (\n  id INTEGER,\n  name TEXT\n);\nINSERT INTO users VALUES (1, 'Alice');\nINSERT INTO users VALUES (2, 'Bob');\n";
        let result = parse(sql, "test").unwrap();
        assert_eq!(result.records.len(), 2);
        assert_eq!(result.schema.nodes.len(), 1);
        assert_eq!(result.schema.nodes[0].name, "users");
    }

    #[test]
    fn test_extract_table_name() {
        assert_eq!(extract_table_name("CREATE TABLE users ("), Some("users".into()));
        assert_eq!(extract_table_name("CREATE TABLE IF NOT EXISTS `orders` ("), Some("orders".into()));
    }

    #[test]
    fn test_sql_type_mapping() {
        assert_eq!(sql_type_to_field("INTEGER"), FieldType::Integer);
        assert_eq!(sql_type_to_field("TEXT"), FieldType::Text);
        assert_eq!(sql_type_to_field("REAL"), FieldType::Float);
        assert_eq!(sql_type_to_field("BOOLEAN"), FieldType::Boolean);
    }
}
