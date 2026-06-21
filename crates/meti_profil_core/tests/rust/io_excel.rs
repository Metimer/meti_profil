use arrow::datatypes::DataType;
use meti_profil_core::io::read_file;

#[test]
fn test_read_excel() {
    let df = read_file("tests/fixtures/small.xlsx").unwrap();
    assert_eq!(df.row_count(), 5);
    assert_eq!(df.column_count(), 3);

    let column_names: Vec<&str> = df.columns().iter().map(|c| c.name.as_str()).collect();
    assert_eq!(column_names, vec!["age", "name", "score"]);

    let age = df.column("age").unwrap();
    assert!(matches!(age.data_type, DataType::Int64 | DataType::Float64));
    assert!(!age.is_empty());
}
