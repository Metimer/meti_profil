use arrow::array::{Int32Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use meti_profil_core::analysis::schema::{DetectedType, SchemaAnalysis};
use meti_profil_core::dataframe::DataFrame;
use std::sync::Arc;

#[test]
fn test_schema_analysis() {
    let schema = Arc::new(Schema::new(vec![
        Field::new("age", DataType::Int32, true),
        Field::new("name", DataType::Utf8, false),
    ]));
    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int32Array::from(vec![
                Some(25),
                Some(30),
                None,
                Some(42),
                Some(25),
            ])),
            Arc::new(StringArray::from(vec![
                "alice", "bob", "claire", "david", "alice",
            ])),
        ],
    )
    .unwrap();
    let df = DataFrame::from_record_batch(batch);
    let analysis = SchemaAnalysis::analyze(&df);
    assert_eq!(analysis.row_count, 5);
    assert_eq!(analysis.columns.len(), 2);
    assert_eq!(analysis.columns[0].detected_type, DetectedType::Numeric);
    assert_eq!(analysis.columns[1].detected_type, DetectedType::Categorical);
}
