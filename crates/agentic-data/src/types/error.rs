//! Error types for AgenticData.

/// All possible errors from AgenticData operations.
#[derive(Debug, thiserror::Error)]
pub enum AdatError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid magic bytes — not an .adat file")]
    InvalidMagic,

    #[error("Unsupported format version: {0}")]
    UnsupportedVersion(u32),

    #[error("Schema error: {0}")]
    Schema(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Format detection failed: {0}")]
    FormatDetection(String),

    #[error("Record not found: {0}")]
    RecordNotFound(String),

    #[error("Source not found: {0}")]
    SourceNotFound(String),

    #[error("Compression error: {0}")]
    Compression(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Transform error: {0}")]
    Transform(String),

    #[error("Lineage error: {0}")]
    Lineage(String),

    #[error("Content too large: {size} bytes (max {max})")]
    ContentTooLarge { size: usize, max: usize },

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("Spatial error: {0}")]
    Spatial(String),

    #[error("Redaction error: {0}")]
    Redaction(String),
}

/// Convenience Result type.
pub type AdatResult<T> = Result<T, AdatError>;

impl From<serde_json::Error> for AdatError {
    fn from(e: serde_json::Error) -> Self {
        AdatError::Serialization(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let e = AdatError::InvalidMagic;
        assert!(e.to_string().contains("magic"));
    }

    #[test]
    fn test_content_too_large() {
        let e = AdatError::ContentTooLarge { size: 2_000_000, max: 1_048_576 };
        assert!(e.to_string().contains("2000000"));
    }
}
