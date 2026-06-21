use arrow::array::{Int32Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use meti_profil_core::dataframe::DataFrame;
use std::sync::Arc;

#[test]
fn test_from_record_batch() {
    let schema = Arc::new(Schema::new(vec![
        Field::new("age", DataType::Int32, false),
        Field::new("name", DataType::Utf8, false),
    ]));
    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int32Array::from(vec![1, 2, 3])),
            Arc::new(StringArray::from(vec!["a", "b", "c"])),
        ],
    )
    .unwrap();
    let df = DataFrame::from_record_batch(batch);
    assert_eq!(df.row_count(), 3);
    assert_eq!(df.column_count(), 2);
    assert_eq!(df.column("age").unwrap().data_type, DataType::Int32);
}
