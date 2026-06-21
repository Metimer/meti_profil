use meti_profil_core::io::read_file;

#[test]
fn test_read_excel() {
    let df = read_file("tests/fixtures/small.xlsx").unwrap();
    assert_eq!(df.row_count(), 5);
    assert_eq!(df.column_count(), 3);
}
