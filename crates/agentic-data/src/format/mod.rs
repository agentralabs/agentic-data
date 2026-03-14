//! .adat file format — binary read/write for AgenticData files.

pub mod writer;
pub mod reader;

pub use writer::AdatWriter;
pub use reader::AdatReader;
