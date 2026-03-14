//! .adat file reader — reads binary AgenticData files.

use std::io::{self, Read, Seek, SeekFrom};
use crate::types::{
    AdatResult, AdatError, FileHeader, DataRecord, DataSource, UniversalSchema, HEADER_SIZE,
};

/// Reader for .adat files.
pub struct AdatReader<R: Read + Seek> {
    inner: R,
    header: FileHeader,
}

impl<R: Read + Seek> AdatReader<R> {
    /// Open an .adat file for reading.
    pub fn open(mut reader: R) -> AdatResult<Self> {
        let mut header_buf = [0u8; HEADER_SIZE];
        reader.read_exact(&mut header_buf)
            .map_err(AdatError::Io)?;
        let header = FileHeader::from_bytes(&header_buf)?;

        Ok(Self { inner: reader, header })
    }

    /// Get the file header.
    pub fn header(&self) -> &FileHeader {
        &self.header
    }

    /// Read all schemas from the file.
    pub fn read_schemas(&mut self) -> AdatResult<Vec<UniversalSchema>> {
        self.read_section(self.header.schema_offset, self.header.source_offset)
    }

    /// Read all sources from the file.
    pub fn read_sources(&mut self) -> AdatResult<Vec<DataSource>> {
        self.read_section(self.header.source_offset, self.header.record_offset)
    }

    /// Read all records from the file.
    pub fn read_records(&mut self) -> AdatResult<Vec<DataRecord>> {
        self.read_section(self.header.record_offset, self.header.content_offset)
    }

    /// Read and decompress a section between two offsets.
    fn read_section<T: serde::de::DeserializeOwned>(
        &mut self, start: u64, end: u64,
    ) -> AdatResult<Vec<T>> {
        if start == 0 || end == 0 || end <= start {
            return Ok(Vec::new());
        }
        let size = (end - start) as usize;
        self.inner.seek(SeekFrom::Start(start)).map_err(AdatError::Io)?;

        let mut compressed = vec![0u8; size];
        self.inner.read_exact(&mut compressed).map_err(AdatError::Io)?;

        let decompressed = lz4_flex::decompress_size_prepended(&compressed)
            .map_err(|e| AdatError::Compression(e.to_string()))?;

        serde_json::from_slice(&decompressed)
            .map_err(|e| AdatError::Serialization(e.to_string()))
    }

    /// Summary of the file contents.
    pub fn summary(&self) -> String {
        format!(
            "AgenticData v{}: {} schemas, {} sources, {} records",
            self.header.version,
            self.header.schema_count,
            self.header.source_count,
            self.header.record_count,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::AdatWriter;
    use std::collections::HashMap;
    use std::io::Cursor;

    #[test]
    fn test_roundtrip() {
        // Write
        let mut buf = Vec::new();
        let mut writer = AdatWriter::new(&mut buf);
        writer.add_schema(UniversalSchema::new("test"));
        writer.add_source(crate::types::DataSource::file("f", "/tmp/x.csv"));
        let mut fields = HashMap::new();
        fields.insert("val".into(), serde_json::json!(42));
        writer.add_record(crate::types::DataRecord::new("s", "n", fields));
        writer.finish().unwrap();

        // Read
        let cursor = Cursor::new(buf);
        let mut reader = AdatReader::open(cursor).unwrap();
        assert_eq!(reader.header().schema_count, 1);
        assert_eq!(reader.header().source_count, 1);
        assert_eq!(reader.header().record_count, 1);

        let schemas = reader.read_schemas().unwrap();
        assert_eq!(schemas.len(), 1);
        assert_eq!(schemas[0].name, "test");

        let sources = reader.read_sources().unwrap();
        assert_eq!(sources.len(), 1);

        let records = reader.read_records().unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].get_i64("val"), Some(42));
    }

    #[test]
    fn test_empty_file() {
        let mut buf = Vec::new();
        let writer = AdatWriter::new(&mut buf);
        writer.finish().unwrap();

        let cursor = Cursor::new(buf);
        let reader = AdatReader::open(cursor).unwrap();
        assert_eq!(reader.header().record_count, 0);
        assert!(reader.summary().contains("0 records"));
    }

    #[test]
    fn test_invalid_file() {
        let buf = vec![0u8; 64]; // Not ADAT magic
        let cursor = Cursor::new(buf);
        let result = AdatReader::open(cursor);
        assert!(result.is_err());
    }
}
