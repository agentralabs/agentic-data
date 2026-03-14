//! Media metadata parser — extracts properties from image, audio, video files.
//!
//! Invention 7: Media Alchemy. Media as structured data.

use std::collections::HashMap;
use crate::types::*;
use super::ParseResult;

/// Parse media metadata from a file path or description string.
/// In production, this would use actual media libraries (ffprobe, exiftool).
/// For now, extracts metadata from file naming conventions and common patterns.
pub fn parse(data: &str, source_name: &str) -> AdatResult<ParseResult> {
    let mut fields = HashMap::new();

    // Detect media type from content AND source name
    let media_type = {
        let from_data = detect_media_type(data);
        if from_data == "unknown" { detect_media_type(source_name) } else { from_data }
    };
    fields.insert("media_type".into(), serde_json::json!(media_type));
    fields.insert("source".into(), serde_json::json!(source_name));

    // Extract any key-value metadata present in the data
    for line in data.lines() {
        let trimmed = line.trim();
        if let Some(colon) = trimmed.find(':') {
            let key = trimmed[..colon].trim().to_lowercase().replace(' ', "_");
            let val = trimmed[colon + 1..].trim();
            if !key.is_empty() && !val.is_empty() && key.len() < 50 {
                fields.insert(key, serde_json::json!(val));
            }
        }
    }

    let schema_fields = vec![
        SchemaField::inferred("media_type", FieldType::Text, 0.90),
        SchemaField::inferred("source", FieldType::Text, 1.0),
        SchemaField::inferred("width", FieldType::Integer, 0.50),
        SchemaField::inferred("height", FieldType::Integer, 0.50),
        SchemaField::inferred("duration", FieldType::Duration, 0.50),
        SchemaField::inferred("codec", FieldType::Text, 0.50),
    ];

    let mut schema = UniversalSchema::new(source_name);
    schema.nodes.push(SchemaNode {
        name: "media".into(), source: source_name.into(),
        fields: schema_fields, record_count: Some(1),
    });

    Ok(ParseResult {
        schema,
        records: vec![DataRecord::new(source_name, "media", fields)],
        errors: 0, warnings: Vec::new(),
    })
}

fn detect_media_type(data: &str) -> &'static str {
    let lower = data.to_lowercase();
    if lower.contains(".jpg") || lower.contains(".jpeg") || lower.contains(".png")
        || lower.contains(".gif") || lower.contains(".webp") || lower.contains(".svg") {
        "image"
    } else if lower.contains(".mp4") || lower.contains(".avi") || lower.contains(".mkv")
        || lower.contains(".mov") || lower.contains(".webm") {
        "video"
    } else if lower.contains(".mp3") || lower.contains(".wav") || lower.contains(".flac")
        || lower.contains(".ogg") || lower.contains(".aac") {
        "audio"
    } else {
        "unknown"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_image() {
        assert_eq!(detect_media_type("photo.jpg"), "image");
        assert_eq!(detect_media_type("icon.png"), "image");
    }

    #[test]
    fn test_detect_video() {
        assert_eq!(detect_media_type("clip.mp4"), "video");
    }

    #[test]
    fn test_detect_audio() {
        assert_eq!(detect_media_type("song.mp3"), "audio");
    }

    #[test]
    fn test_parse_media_metadata() {
        let data = "Width: 1920\nHeight: 1080\nCodec: H.264\n";
        let result = parse(data, "test.mp4").unwrap();
        assert_eq!(result.records.len(), 1);
        assert_eq!(result.records[0].get_str("media_type"), Some("video"));
    }
}
