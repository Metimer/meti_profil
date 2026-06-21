pub mod csv;
pub mod excel;
pub mod parquet;

use crate::dataframe::DataFrame;
use crate::error::Result;
use std::path::Path;

pub fn read_file(path: impl AsRef<Path>) -> Result<DataFrame> {
    let path = path.as_ref();
    match path.extension().and_then(|e| e.to_str()) {
        Some("csv") => csv::read_csv(path),
        Some("parquet") => parquet::read_parquet(path),
        Some("xlsx") | Some("xls") => excel::read_excel(path),
        _ => Err(crate::error::Error::UnsupportedSource(
            path.display().to_string(),
        )),
    }
}
