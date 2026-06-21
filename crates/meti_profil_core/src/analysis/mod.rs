pub mod categorical;
pub mod correlation;
pub mod duplicate;
pub mod missing;
pub mod numeric;
pub mod schema;

pub use categorical::{CategoricalAnalysis, CategoricalStats, Frequency};
pub use correlation::{CorrelationAnalysis, CorrelationPair};
pub use duplicate::DuplicateAnalysis;
pub use missing::{ColumnMissing, MissingAnalysis};
pub use numeric::{NumericAnalysis, NumericStats};
pub use schema::{ColumnSchema, DetectedType, SchemaAnalysis};
