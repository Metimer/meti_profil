use arrow::array::Float64Array;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use meti_profil_core::analysis::numeric::NumericAnalysis;
use meti_profil_core::dataframe::DataFrame;
use std::sync::Arc;

#[test]
fn test_numeric_analysis() {
    let schema = Arc::new(Schema::new(vec![Field::new(
        "score",
        DataType::Float64,
        true,
    )]));
    let batch = RecordBatch::try_new(
        schema,
        vec![Arc::new(Float64Array::from(vec![
            Some(1.0),
            Some(2.0),
            Some(3.0),
            Some(4.0),
            None,
        ]))],
    )
    .unwrap();
    let df = DataFrame::from_record_batch(batch);
    let analysis = NumericAnalysis::analyze(&df);
    assert_eq!(analysis.columns.len(), 1);
    let stats = &analysis.columns[0].1;
    assert_eq!(stats.count, 5);
    assert_eq!(stats.missing, 1);
    assert!((stats.mean.unwrap() - 2.5).abs() < 1e-10);
    assert!((stats.min.unwrap() - 1.0).abs() < 1e-10);
    assert!((stats.max.unwrap() - 4.0).abs() < 1e-10);
    assert!((stats.median.unwrap() - 2.5).abs() < 1e-10);
}
