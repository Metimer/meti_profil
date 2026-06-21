pub mod categorical;
pub mod correlation;
pub mod duplicate;
pub mod missing;
pub mod numeric;
pub mod schema;

pub use schema::{ColumnSchema, DetectedType, SchemaAnalysis};
