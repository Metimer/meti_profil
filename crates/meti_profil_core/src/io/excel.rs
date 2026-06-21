use crate::dataframe::DataFrame;
use crate::error::{Error, Result};
use std::path::Path;

pub fn read_excel(_path: impl AsRef<Path>) -> Result<DataFrame> {
    Err(Error::UnsupportedSource("excel not yet implemented".into()))
}
