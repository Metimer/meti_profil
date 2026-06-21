// The `#[pymethods]` macro in PyO3 0.22 generates result conversions that newer
// clippy flags as `useless_conversion`; the lint targets generated code we don't
// control, so silence it crate-wide.
#![allow(clippy::useless_conversion)]

use meti_profil_core::dataframe::DataFrame;
use meti_profil_core::io::read_file;
use meti_profil_core::report::html::HtmlRenderer;
use meti_profil_core::report::markdown::MarkdownRenderer;
use meti_profil_core::report::model::Report;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

/// Profiling report exposed to Python.
///
/// The constructor accepts a path to a CSV, Parquet or Excel file. The Python
/// facade in `meti_profil/__init__.py` converts pandas/polars DataFrames to a
/// temporary Parquet file before handing the path to this class.
#[pyclass(name = "ProfileReport")]
pub struct PyProfileReport {
    report: Report,
}

#[pymethods]
impl PyProfileReport {
    #[new]
    #[pyo3(signature = (path, *, title="Dataset Profile", source=None, minimal=false, explorative=true))]
    fn new(
        path: &str,
        title: &str,
        source: Option<String>,
        minimal: bool,
        explorative: bool,
    ) -> PyResult<Self> {
        // `minimal` / `explorative` are reserved for tuning which analyses run;
        // the MVP always runs the full set.
        let _ = (minimal, explorative);
        let df: DataFrame = read_file(path).map_err(|e| PyValueError::new_err(e.to_string()))?;
        let source = source.or_else(|| Some(path.to_string()));
        let report = Report::build(&df, title, source);
        Ok(Self { report })
    }

    fn to_file(&self, path: &str) -> PyResult<()> {
        let md = MarkdownRenderer::render(&self.report);
        std::fs::write(path, md)?;
        Ok(())
    }

    fn to_markdown(&self) -> String {
        MarkdownRenderer::render(&self.report)
    }

    /// Write the self-contained interactive HTML report to `path`.
    fn to_html_file(&self, path: &str) -> PyResult<()> {
        let html = HtmlRenderer::render(&self.report);
        std::fs::write(path, html)?;
        Ok(())
    }

    /// Return the self-contained interactive HTML report as a string.
    fn to_html(&self) -> String {
        HtmlRenderer::render(&self.report)
    }

    fn get_summary<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new_bound(py);
        dict.set_item("title", &self.report.title)?;
        dict.set_item("rows", self.report.schema.row_count)?;
        dict.set_item("columns", self.report.schema.column_count)?;
        dict.set_item("missing_cells", self.report.missing.missing_cells)?;
        dict.set_item("missing_cells_pct", self.report.missing.missing_pct)?;
        dict.set_item("duplicate_rows", self.report.duplicates.duplicate_rows)?;
        dict.set_item("duplicate_rows_pct", self.report.duplicates.duplicate_pct)?;
        Ok(dict)
    }

    fn get_column_info<'py>(
        &self,
        py: Python<'py>,
        name: &str,
    ) -> PyResult<Option<Bound<'py, PyDict>>> {
        let Some(col) = self.report.schema.columns.iter().find(|c| c.name == name) else {
            return Ok(None);
        };
        let dict = PyDict::new_bound(py);
        dict.set_item("name", &col.name)?;
        dict.set_item("detected_type", format!("{:?}", col.detected_type))?;
        dict.set_item("arrow_type", &col.arrow_type)?;
        dict.set_item("unique_count", col.unique_count)?;
        dict.set_item("missing_count", col.null_count)?;
        dict.set_item("cardinality_ratio", col.cardinality_ratio)?;
        dict.set_item("is_constant", col.is_constant)?;
        Ok(Some(dict))
    }
}

#[pymodule]
fn _meti_profil(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyProfileReport>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
