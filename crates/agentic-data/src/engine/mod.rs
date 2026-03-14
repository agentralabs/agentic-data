//! Core engine — ingest, query, transform, and quality analysis.

pub mod ingest;
pub mod query;
pub mod transform;
pub mod quality;
pub mod store;

pub use ingest::IngestEngine;
pub use query::{QueryEngine, QueryResult, QueryFilter, FilterOp};
pub use transform::{TransformEngine, TransformStep, TransformPipeline};
pub use quality::QualityEngine;
pub use store::DataStore;
