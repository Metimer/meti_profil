use crate::analysis::categorical::CategoricalAnalysis;
use crate::analysis::correlation::CorrelationAnalysis;
use crate::analysis::duplicate::DuplicateAnalysis;
use crate::analysis::missing::MissingAnalysis;
use crate::analysis::numeric::NumericAnalysis;
use crate::analysis::schema::{DetectedType, SchemaAnalysis};
use crate::dataframe::DataFrame;
use serde::Serialize;

/// Aggregated profiling results for a single dataset.
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub title: String,
    pub source: Option<String>,
    pub version: String,
    pub schema: SchemaAnalysis,
    pub numeric: NumericAnalysis,
    pub categorical: CategoricalAnalysis,
    pub missing: MissingAnalysis,
    pub duplicates: DuplicateAnalysis,
    pub correlations: CorrelationAnalysis,
}

impl Report {
    /// Run every analysis module over `df` and assemble a complete report.
    pub fn build(df: &DataFrame, title: impl Into<String>, source: Option<String>) -> Self {
        Self {
            title: title.into(),
            source,
            version: env!("CARGO_PKG_VERSION").to_string(),
            schema: SchemaAnalysis::analyze(df),
            numeric: NumericAnalysis::analyze(df),
            categorical: CategoricalAnalysis::analyze(df),
            missing: MissingAnalysis::analyze(df),
            duplicates: DuplicateAnalysis::analyze(df),
            correlations: CorrelationAnalysis::analyze(df),
        }
    }

    /// Names of columns whose detected type is numeric, in schema order.
    pub fn numeric_column_names(&self) -> Vec<&str> {
        self.schema
            .columns
            .iter()
            .filter(|c| c.detected_type == DetectedType::Numeric)
            .map(|c| c.name.as_str())
            .collect()
    }

    /// Names of columns whose detected type is categorical or boolean, in schema order.
    pub fn categorical_column_names(&self) -> Vec<&str> {
        self.schema
            .columns
            .iter()
            .filter(|c| {
                matches!(
                    c.detected_type,
                    DetectedType::Categorical | DetectedType::Boolean
                )
            })
            .map(|c| c.name.as_str())
            .collect()
    }
}
