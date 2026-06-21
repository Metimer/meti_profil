use arrow::array::{Float64Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use meti_profil_core::dataframe::DataFrame;
use meti_profil_core::report::markdown::MarkdownRenderer;
use meti_profil_core::report::model::Report;
use std::sync::Arc;

fn sample_df() -> DataFrame {
    let schema = Arc::new(Schema::new(vec![
        Field::new("age", DataType::Int64, true),
        Field::new("name", DataType::Utf8, false),
        Field::new("score", DataType::Float64, true),
    ]));
    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int64Array::from(vec![
                Some(25),
                Some(30),
                None,
                Some(42),
                Some(25),
            ])),
            Arc::new(StringArray::from(vec![
                "alice", "bob", "alice", "david", "alice",
            ])),
            Arc::new(Float64Array::from(vec![
                Some(88.5),
                Some(92.0),
                Some(75.5),
                None,
                Some(88.5),
            ])),
        ],
    )
    .unwrap();
    DataFrame::from_record_batch(batch)
}

#[test]
fn test_markdown_render() {
    let df = sample_df();
    let report = Report::build(&df, "Test Report", Some("test.csv".into()));
    let md = MarkdownRenderer::render(&report);

    assert!(md.starts_with("---"));
    assert!(md.contains("title: Test Report"));
    assert!(md.contains("meti_profil_version:"));
    assert!(md.contains("# Test Report"));
    assert!(md.contains("## Overview"));
    assert!(md.contains("## Schema"));
    assert!(md.contains("## Numeric Columns"));
    assert!(md.contains("## Categorical Columns"));
    assert!(md.contains("## Missing Values"));
    assert!(md.contains("## Duplicate Rows"));
    assert!(md.contains("rows: 5"));
}
