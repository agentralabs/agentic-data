//! All data types for the AgenticData library.

pub mod error;
pub mod schema;
pub mod record;
pub mod source;
pub mod lineage;
pub mod quality;
pub mod spatial;
pub mod header;

pub use error::{AdatError, AdatResult};
pub use schema::{SchemaNode, SchemaEdge, SchemaField, FieldType, FieldStats, UniversalSchema};
pub use record::{DataRecord, RecordId, RecordStatus};
pub use source::{DataSource, SourceId, SourceType};
pub use lineage::{LineageEntry, LineageChain, LineageAction, TransformReceipt};
pub use quality::{QualityScore, AnomalyRecord, AnomalyType, HealthMetric, Trend};
pub use spatial::{GeoPoint, GeoBounds, SpatialRef};
pub use header::{FileHeader, HEADER_SIZE};

/// Magic bytes at the start of every .adat file.
pub const ADAT_MAGIC: [u8; 4] = [0x41, 0x44, 0x41, 0x54]; // "ADAT"

/// Current format version.
pub const FORMAT_VERSION: u32 = 1;

/// Maximum content size per record (before compression): 1MB.
pub const MAX_CONTENT_SIZE: usize = 1_048_576;

/// Maximum fields per schema.
pub const MAX_FIELDS_PER_SCHEMA: u16 = 4096;

/// Returns the current time as Unix epoch microseconds.
pub fn now_micros() -> u64 {
    chrono::Utc::now().timestamp_micros() as u64
}
