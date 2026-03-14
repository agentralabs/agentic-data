//! HTML and Markdown parser — extracts structure from documents.

use std::collections::HashMap;
use crate::types::*;
use super::ParseResult;

/// Parse HTML into structured records (tables, headings, content).
pub fn parse(data: &str, source_name: &str) -> AdatResult<ParseResult> {
    let mut records = Vec::new();
    let mut headings = Vec::new();
    let mut tables = Vec::new();
    let mut current_text = String::new();

    for line in data.lines() {
        let trimmed = line.trim();
        // Extract headings
        if let Some(heading) = extract_html_heading(trimmed) {
            if !current_text.is_empty() {
                let mut f = HashMap::new();
                f.insert("type".into(), serde_json::json!("text"));
                f.insert("content".into(), serde_json::json!(current_text.trim()));
                records.push(DataRecord::new(source_name, "content", f));
                current_text.clear();
            }
            headings.push(heading.clone());
            let mut f = HashMap::new();
            f.insert("type".into(), serde_json::json!("heading"));
            f.insert("content".into(), serde_json::json!(heading));
            records.push(DataRecord::new(source_name, "heading", f));
        }
        // Extract table rows
        else if trimmed.starts_with("<tr") {
            if let Some(cells) = extract_table_row(trimmed) {
                tables.push(cells.clone());
                let mut f = HashMap::new();
                f.insert("type".into(), serde_json::json!("table_row"));
                f.insert("cells".into(), serde_json::json!(cells));
                records.push(DataRecord::new(source_name, "table", f));
            }
        } else {
            let text = strip_html_tags(trimmed);
            if !text.is_empty() {
                current_text.push_str(&text);
                current_text.push(' ');
            }
        }
    }

    if !current_text.is_empty() {
        let mut f = HashMap::new();
        f.insert("type".into(), serde_json::json!("text"));
        f.insert("content".into(), serde_json::json!(current_text.trim()));
        records.push(DataRecord::new(source_name, "content", f));
    }

    let fields = vec![
        SchemaField::inferred("type", FieldType::Text, 1.0),
        SchemaField::inferred("content", FieldType::Text, 1.0),
    ];
    let mut schema = UniversalSchema::new(source_name);
    schema.nodes.push(SchemaNode {
        name: source_name.into(), source: source_name.into(),
        fields, record_count: Some(records.len() as u64),
    });

    Ok(ParseResult { schema, records, errors: 0, warnings: Vec::new() })
}

/// Parse Markdown into structured records.
pub fn parse_markdown(data: &str, source_name: &str) -> AdatResult<ParseResult> {
    let mut records = Vec::new();
    let mut current_section = String::new();
    let mut current_content = String::new();

    for line in data.lines() {
        if line.starts_with('#') {
            // Flush previous section
            if !current_section.is_empty() || !current_content.is_empty() {
                let mut f = HashMap::new();
                f.insert("heading".into(), serde_json::json!(current_section));
                f.insert("content".into(), serde_json::json!(current_content.trim()));
                records.push(DataRecord::new(source_name, "section", f));
            }
            current_section = line.trim_start_matches('#').trim().to_string();
            current_content.clear();
        } else {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }
    if !current_section.is_empty() || !current_content.is_empty() {
        let mut f = HashMap::new();
        f.insert("heading".into(), serde_json::json!(current_section));
        f.insert("content".into(), serde_json::json!(current_content.trim()));
        records.push(DataRecord::new(source_name, "section", f));
    }

    let fields = vec![
        SchemaField::inferred("heading", FieldType::Text, 1.0),
        SchemaField::inferred("content", FieldType::Text, 1.0),
    ];
    let mut schema = UniversalSchema::new(source_name);
    schema.nodes.push(SchemaNode {
        name: source_name.into(), source: source_name.into(),
        fields, record_count: Some(records.len() as u64),
    });

    Ok(ParseResult { schema, records, errors: 0, warnings: Vec::new() })
}

fn extract_html_heading(line: &str) -> Option<String> {
    for tag in &["h1", "h2", "h3", "h4", "h5", "h6"] {
        let open = format!("<{}", tag);
        let close = format!("</{}>", tag);
        if line.contains(&open) && line.contains(&close) {
            let start = line.find('>').map(|i| i + 1)?;
            let end = line.find(&close)?;
            return Some(line[start..end].to_string());
        }
    }
    None
}

fn extract_table_row(line: &str) -> Option<Vec<String>> {
    let mut cells = Vec::new();
    let mut rest = line;
    while let Some(start) = rest.find("<td") {
        let content_start = rest[start..].find('>').map(|i| start + i + 1)?;
        let end = rest[content_start..].find("</td>").map(|i| content_start + i)?;
        cells.push(rest[content_start..end].to_string());
        rest = &rest[end + 5..];
    }
    if cells.is_empty() { None } else { Some(cells) }
}

fn strip_html_tags(text: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for c in text.chars() {
        if c == '<' { in_tag = true; }
        else if c == '>' { in_tag = false; }
        else if !in_tag { result.push(c); }
    }
    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_html_headings() {
        let html = "<html>\n<body>\n<h1>Title</h1>\n<p>Content here</p>\n</body>\n</html>";
        let result = parse(html, "test").unwrap();
        assert!(result.records.iter().any(|r| r.get_str("content") == Some("Title")));
    }

    #[test]
    fn test_parse_markdown() {
        let md = "# Hello\nWorld\n## Sub\nContent\n";
        let result = parse_markdown(md, "test").unwrap();
        assert_eq!(result.records.len(), 2);
    }

    #[test]
    fn test_strip_tags() {
        assert_eq!(strip_html_tags("<b>bold</b>"), "bold");
        assert_eq!(strip_html_tags("no tags"), "no tags");
    }
}
