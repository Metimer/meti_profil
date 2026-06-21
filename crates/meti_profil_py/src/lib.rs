use pyo3::prelude::*;

#[pymodule]
fn _meti_profil(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let meti_profil_py = PyModule::new_bound(m.py(), "meti_profil_py")?;
    meti_profil_py.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_submodule(&meti_profil_py)?;
    Ok(())
}
