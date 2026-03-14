//! Index modules — fast lookups for schema, temporal, quality, spatial, lineage.

pub mod schema_index;
pub mod temporal_index;
pub mod quality_index;
pub mod spatial_index;
pub mod lineage_index;

pub use schema_index::SchemaIndex;
pub use temporal_index::TemporalIndex;
pub use quality_index::QualityIndex;
pub use spatial_index::SpatialIndex;
pub use lineage_index::LineageIndex;
