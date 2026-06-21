use crate::dataframe::DataFrame;
use crate::error::{Error, Result};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use std::fs::File;
use std::path::Path;

pub fn read_parquet(path: impl AsRef<Path>) -> Result<DataFrame> {
    let file = File::open(path)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file).map_err(as_arrow_err)?;
    let reader = builder.build().map_err(as_arrow_err)?;
    let batches: Vec<_> = reader
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(as_arrow_err)?;
    DataFrame::from_record_batches(batches)
}

fn as_arrow_err<E: std::fmt::Display>(e: E) -> Error {
    Error::Arrow(e.to_string())
}
