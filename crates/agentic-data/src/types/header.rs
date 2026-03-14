//! .adat file header — binary format header for AgenticData files.
//!
//! Same design principles as .amem: one file, self-contained, portable.

use super::{AdatError, AdatResult, ADAT_MAGIC, FORMAT_VERSION};

/// Fixed header size in bytes.
pub const HEADER_SIZE: usize = 64;

/// Binary file header for .adat files.
#[derive(Debug, Clone)]
pub struct FileHeader {
    /// Magic bytes (must be ADAT_MAGIC).
    pub magic: [u8; 4],
    /// Format version.
    pub version: u32,
    /// Number of schemas in this file.
    pub schema_count: u32,
    /// Number of sources registered.
    pub source_count: u32,
    /// Number of records.
    pub record_count: u64,
    /// Offset to schema registry section.
    pub schema_offset: u64,
    /// Offset to source table section.
    pub source_offset: u64,
    /// Offset to record table section.
    pub record_offset: u64,
    /// Offset to content block.
    pub content_offset: u64,
}

impl FileHeader {
    /// Create a new header with default offsets.
    pub fn new() -> Self {
        Self {
            magic: ADAT_MAGIC,
            version: FORMAT_VERSION,
            schema_count: 0,
            source_count: 0,
            record_count: 0,
            schema_offset: HEADER_SIZE as u64,
            source_offset: 0,
            record_offset: 0,
            content_offset: 0,
        }
    }

    /// Serialize header to bytes (little-endian).
    pub fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut buf = [0u8; HEADER_SIZE];
        buf[0..4].copy_from_slice(&self.magic);
        buf[4..8].copy_from_slice(&self.version.to_le_bytes());
        buf[8..12].copy_from_slice(&self.schema_count.to_le_bytes());
        buf[12..16].copy_from_slice(&self.source_count.to_le_bytes());
        buf[16..24].copy_from_slice(&self.record_count.to_le_bytes());
        buf[24..32].copy_from_slice(&self.schema_offset.to_le_bytes());
        buf[32..40].copy_from_slice(&self.source_offset.to_le_bytes());
        buf[40..48].copy_from_slice(&self.record_offset.to_le_bytes());
        buf[48..56].copy_from_slice(&self.content_offset.to_le_bytes());
        // bytes 56..64 reserved
        buf
    }

    /// Deserialize header from bytes.
    pub fn from_bytes(buf: &[u8; HEADER_SIZE]) -> AdatResult<Self> {
        let mut magic = [0u8; 4];
        magic.copy_from_slice(&buf[0..4]);
        if magic != ADAT_MAGIC {
            return Err(AdatError::InvalidMagic);
        }
        let version = u32::from_le_bytes(buf[4..8].try_into().unwrap());
        if version > FORMAT_VERSION {
            return Err(AdatError::UnsupportedVersion(version));
        }
        Ok(Self {
            magic,
            version,
            schema_count: u32::from_le_bytes(buf[8..12].try_into().unwrap()),
            source_count: u32::from_le_bytes(buf[12..16].try_into().unwrap()),
            record_count: u64::from_le_bytes(buf[16..24].try_into().unwrap()),
            schema_offset: u64::from_le_bytes(buf[24..32].try_into().unwrap()),
            source_offset: u64::from_le_bytes(buf[32..40].try_into().unwrap()),
            record_offset: u64::from_le_bytes(buf[40..48].try_into().unwrap()),
            content_offset: u64::from_le_bytes(buf[48..56].try_into().unwrap()),
        })
    }
}

impl Default for FileHeader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_roundtrip() {
        let mut h = FileHeader::new();
        h.schema_count = 3;
        h.source_count = 2;
        h.record_count = 1000;
        h.content_offset = 8192;

        let bytes = h.to_bytes();
        let back = FileHeader::from_bytes(&bytes).unwrap();

        assert_eq!(back.magic, ADAT_MAGIC);
        assert_eq!(back.version, FORMAT_VERSION);
        assert_eq!(back.schema_count, 3);
        assert_eq!(back.source_count, 2);
        assert_eq!(back.record_count, 1000);
        assert_eq!(back.content_offset, 8192);
    }

    #[test]
    fn test_invalid_magic() {
        let mut buf = [0u8; HEADER_SIZE];
        buf[0..4].copy_from_slice(b"NOPE");
        let result = FileHeader::from_bytes(&buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_header_size() {
        assert_eq!(HEADER_SIZE, 64);
        let h = FileHeader::new();
        assert_eq!(h.to_bytes().len(), HEADER_SIZE);
    }
}
