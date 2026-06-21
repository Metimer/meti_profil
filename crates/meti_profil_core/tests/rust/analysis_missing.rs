use arrow::array::Int32Array;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use meti_profil_core::analysis::missing::MissingAnalysis;
use meti_profil_core::dataframe::DataFrame;
use std::sync::Arc;

#[test]
fn test_missing_analysis() {
    let schema = Arc::new(Schema::new(vec![Field::new("age", DataType::Int32, true)]));
    let batch = RecordBatch::try_new(
        schema,
        vec![Arc::new(Int32Array::from(vec![
            Some(1),
            None,
            Some(3),
            None,
        ]))],
    )
    .unwrap();
    let df = DataFrame::from_record_batch(batch);
    let analysis = MissingAnalysis::analyze(&df);
    assert_eq!(analysis.missing_cells, 2);
    assert_eq!(analysis.columns[0].missing_count, 2);
    assert!((analysis.missing_pct - 50.0).abs() < 1e-10);
}
