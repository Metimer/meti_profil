use crate::dataframe::DataFrame;
use crate::error::{Error, Result};
use std::path::Path;

pub fn read_parquet(_path: impl AsRef<Path>) -> Result<DataFrame> {
    Err(Error::UnsupportedSource("parquet not yet implemented".into()))
}
