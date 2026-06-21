use arrow::array::StringArray;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use meti_profil_core::analysis::categorical::CategoricalAnalysis;
use meti_profil_core::dataframe::DataFrame;
use std::sync::Arc;

#[test]
fn test_categorical_analysis() {
    let schema = Arc::new(Schema::new(vec![Field::new("name", DataType::Utf8, false)]));
    let batch = RecordBatch::try_new(
        schema,
        vec![Arc::new(StringArray::from(vec![
            "alice", "bob", "alice", "claire", "alice",
        ]))],
    )
    .unwrap();
    let df = DataFrame::from_record_batch(batch);
    let analysis = CategoricalAnalysis::analyze(&df);
    assert_eq!(analysis.columns.len(), 1);
    let stats = &analysis.columns[0].1;
    assert_eq!(stats.unique_count, 3);
    assert_eq!(stats.top_values[0].value, "alice");
    assert_eq!(stats.top_values[0].count, 3);
}
