use arrow::array::{Float64Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use meti_profil_core::dataframe::DataFrame;
use meti_profil_core::report::html::HtmlRenderer;
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
fn test_html_render() {
    let df = sample_df();
    let report = Report::build(&df, "HTML Test", Some("test.csv".into()));
    let html = HtmlRenderer::render(&report);

    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<title>HTML Test</title>"));
    assert!(html.contains("window.METI_PROFIL ="));
    assert!(html.contains("data-chart=\"hist\""));
    assert!(html.contains("data-chart=\"cat\""));
    assert!(html.contains("data-chart=\"missing\""));
    // Embedded CSS and JS.
    assert!(html.contains(".mp-wrap"));
    assert!(html.contains("METI_PROFIL"));
    assert!(html.trim_end().ends_with("</html>"));
}

#[test]
fn test_html_escapes_and_script_safety() {
    let schema = Arc::new(Schema::new(vec![Field::new("a", DataType::Utf8, false)]));
    let batch = RecordBatch::try_new(
        schema,
        vec![Arc::new(StringArray::from(vec![
            "</script><b>x",
            "</script><b>x",
            "safe",
        ]))],
    )
    .unwrap();
    let df = DataFrame::from_record_batch(batch);
    let report = Report::build(&df, "Inj & <test>", None);
    let html = HtmlRenderer::render(&report);

    // Title is HTML-escaped.
    assert!(html.contains("Inj &amp; &lt;test&gt;"));
    // No raw closing script tag from the data leaks into the document.
    assert!(!html.contains("</script><b>x"));
}
