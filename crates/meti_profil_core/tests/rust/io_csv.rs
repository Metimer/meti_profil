use meti_profil_core::io::read_file;

#[test]
fn test_io_csv() {
    let df = read_file("tests/fixtures/small.csv").unwrap();
    assert_eq!(df.row_count(), 5);
    assert_eq!(df.column_count(), 3);
    assert!(df.column("age").is_ok());
    assert!(df.column("name").is_ok());
    assert!(df.column("score").is_ok());
}
