//! Core engine — ingest, query, transform, quality, graph, session, consolidation.

pub mod ingest;
pub mod query;
pub mod transform;
pub mod quality;
pub mod store;
pub mod graph;
pub mod session;
pub mod consolidation;

pub use ingest::IngestEngine;
pub use query::{QueryEngine, QueryResult, QueryFilter, FilterOp};
pub use transform::{TransformEngine, TransformStep, TransformPipeline};
pub use quality::QualityEngine;
pub use store::DataStore;
pub use graph::{GraphEngine, TraversalDirection, TraversalResult, ImpactResult};
pub use session::{SessionManager, DataSession, OpType};
pub use consolidation::{ConsolidationEngine, ConsolidationReport};
