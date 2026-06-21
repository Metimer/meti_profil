use arrow::array::Int64Array;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use meti_profil_core::analysis::correlation::CorrelationAnalysis;
use meti_profil_core::dataframe::DataFrame;
use std::sync::Arc;

#[test]
fn test_correlation_analysis() {
    let schema = Arc::new(Schema::new(vec![
        Field::new("a", DataType::Int64, false),
        Field::new("b", DataType::Int64, false),
    ]));
    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int64Array::from(vec![1, 2, 3, 4, 5])),
            Arc::new(Int64Array::from(vec![2, 4, 6, 8, 10])),
        ],
    )
    .unwrap();
    let df = DataFrame::from_record_batch(batch);
    let analysis = CorrelationAnalysis::analyze(&df);
    assert_eq!(analysis.pairs.len(), 1);
    assert!((analysis.pairs[0].pearson - 1.0).abs() < 1e-10);
}
