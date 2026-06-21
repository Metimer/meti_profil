use crate::dataframe::DataFrame;
use crate::error::Result;
use arrow::csv::reader::{Format, ReaderBuilder};
use std::fs::File;
use std::io::Seek;
use std::path::Path;
use std::sync::Arc;

pub fn read_csv(path: impl AsRef<Path>) -> Result<DataFrame> {
    let mut file = File::open(path)?;
    let format = Format::default().with_header(true);
    let (schema, _) = format.infer_schema(&mut file, None)?;
    file.rewind()?;
    let builder = ReaderBuilder::new(Arc::new(schema)).with_header(true);
    let reader = builder.build(file)?;
    let batches: Vec<_> = reader.collect::<std::result::Result<Vec<_>, _>>()?;
    DataFrame::from_record_batches(batches)
}
