//! AgenticData — universal data comprehension engine.
//!
//! Understands structure, infers meaning, tracks lineage, detects anomalies,
//! and transforms between any format. One engine for ALL data.

pub mod types;
pub mod format;
pub mod parser;
pub mod engine;
pub mod index;
pub mod crypto;

// Re-export commonly used types at the crate root
pub use types::{
    AdatError, AdatResult,
    UniversalSchema, SchemaNode, SchemaField, SchemaEdge, FieldType,
    DataRecord, RecordId, RecordStatus,
    DataSource, SourceId, SourceType,
    LineageChain, LineageEntry, LineageAction, TransformReceipt,
    QualityScore, AnomalyRecord, AnomalyType, HealthMetric,
    GeoPoint, GeoBounds, SpatialRef,
    FileHeader, HEADER_SIZE,
    ADAT_MAGIC, FORMAT_VERSION, MAX_CONTENT_SIZE,
};
pub use format::{AdatWriter, AdatReader};
pub use engine::{DataStore, IngestEngine, QueryEngine, QueryResult, TransformEngine, QualityEngine};
pub use index::{SchemaIndex, TemporalIndex, QualityIndex, SpatialIndex, LineageIndex};
pub use crypto::{FieldEncryptor, KeyManager, RedactionEngine, PiiDetection, RedactionPolicy};
