use arrow::array::Int32Array;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use meti_profil_core::analysis::duplicate::DuplicateAnalysis;
use meti_profil_core::dataframe::DataFrame;
use std::sync::Arc;

#[test]
fn test_duplicate_analysis() {
    let schema = Arc::new(Schema::new(vec![Field::new("a", DataType::Int32, false)]));
    let batch = RecordBatch::try_new(
        schema,
        vec![Arc::new(Int32Array::from(vec![1, 2, 1, 3, 2]))],
    )
    .unwrap();
    let df = DataFrame::from_record_batch(batch);
    let analysis = DuplicateAnalysis::analyze(&df);
    assert_eq!(analysis.total_rows, 5);
    assert_eq!(analysis.duplicate_rows, 2);
    assert!((analysis.duplicate_pct - 40.0).abs() < 1e-10);
}
