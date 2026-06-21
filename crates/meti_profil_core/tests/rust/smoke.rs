#[test]
fn workspace_compiles() {
    use meti_profil_core::Result;
    fn _returns_result() -> Result<()> {
        Ok(())
    }
    assert!(_returns_result().is_ok());
}
