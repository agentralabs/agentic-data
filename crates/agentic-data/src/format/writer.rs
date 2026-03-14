//! .adat file writer — creates binary AgenticData files.

use std::io::{self, Write};
use crate::types::{FileHeader, DataRecord, DataSource, UniversalSchema, HEADER_SIZE};

/// Writer for .adat files.
pub struct AdatWriter<W: Write> {
    inner: W,
    header: FileHeader,
    schemas: Vec<UniversalSchema>,
    sources: Vec<DataSource>,
    records: Vec<DataRecord>,
}

impl<W: Write> AdatWriter<W> {
    /// Create a new writer.
    pub fn new(writer: W) -> Self {
        Self {
            inner: writer,
            header: FileHeader::new(),
            schemas: Vec::new(),
            sources: Vec::new(),
            records: Vec::new(),
        }
    }

    /// Add a schema to the file.
    pub fn add_schema(&mut self, schema: UniversalSchema) {
        self.schemas.push(schema);
    }

    /// Add a data source.
    pub fn add_source(&mut self, source: DataSource) {
        self.sources.push(source);
    }

    /// Add a record.
    pub fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }

    /// Write the complete .adat file.
    pub fn finish(mut self) -> io::Result<()> {
        // Serialize sections to JSON (will be replaced with binary in production)
        let schema_bytes = serde_json::to_vec(&self.schemas)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let source_bytes = serde_json::to_vec(&self.sources)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let record_bytes = serde_json::to_vec(&self.records)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Compress content with LZ4
        let schema_compressed = lz4_flex::compress_prepend_size(&schema_bytes);
        let source_compressed = lz4_flex::compress_prepend_size(&source_bytes);
        let record_compressed = lz4_flex::compress_prepend_size(&record_bytes);

        // Calculate offsets
        let schema_offset = HEADER_SIZE as u64;
        let source_offset = schema_offset + schema_compressed.len() as u64;
        let record_offset = source_offset + source_compressed.len() as u64;

        // Build header
        self.header.schema_count = self.schemas.len() as u32;
        self.header.source_count = self.sources.len() as u32;
        self.header.record_count = self.records.len() as u64;
        self.header.schema_offset = schema_offset;
        self.header.source_offset = source_offset;
        self.header.record_offset = record_offset;
        self.header.content_offset = record_offset + record_compressed.len() as u64;

        // Write header
        self.inner.write_all(&self.header.to_bytes())?;

        // Write sections
        self.inner.write_all(&schema_compressed)?;
        self.inner.write_all(&source_compressed)?;
        self.inner.write_all(&record_compressed)?;

        self.inner.flush()
    }

    /// Get counts for progress reporting.
    pub fn counts(&self) -> (usize, usize, usize) {
        (self.schemas.len(), self.sources.len(), self.records.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use std::collections::HashMap;

    #[test]
    fn test_write_empty() {
        let mut buf = Vec::new();
        let writer = AdatWriter::new(&mut buf);
        writer.finish().unwrap();
        assert!(buf.len() >= HEADER_SIZE);
        // Verify magic bytes
        assert_eq!(&buf[0..4], &ADAT_MAGIC);
    }

    #[test]
    fn test_write_with_data() {
        let mut buf = Vec::new();
        let mut writer = AdatWriter::new(&mut buf);

        writer.add_schema(UniversalSchema::new("test_schema"));
        writer.add_source(DataSource::file("csv", "/tmp/test.csv"));

        let mut fields = HashMap::new();
        fields.insert("name".into(), serde_json::json!("Alice"));
        writer.add_record(DataRecord::new("src-1", "users", fields));

        let (s, src, r) = writer.counts();
        assert_eq!(s, 1);
        assert_eq!(src, 1);
        assert_eq!(r, 1);

        writer.finish().unwrap();
        assert!(buf.len() > HEADER_SIZE);
    }
}
