use crate::dataframe::column::Column;
use crate::error::{Error, Result};
use arrow::record_batch::RecordBatch;

#[derive(Clone, Debug)]
pub struct DataFrame {
    columns: Vec<Column>,
    row_count: usize,
}

impl DataFrame {
    pub fn from_record_batch(batch: RecordBatch) -> Self {
        let row_count = batch.num_rows();
        let columns: Vec<Column> = batch
            .schema()
            .fields()
            .iter()
            .zip(batch.columns())
            .map(|(field, array)| Column::new(field.name(), array.clone()))
            .collect();
        Self { columns, row_count }
    }

    pub fn from_record_batches(batches: Vec<RecordBatch>) -> Result<Self> {
        if batches.is_empty() {
            return Err(Error::Arrow("empty record batch list".into()));
        }
        let merged = arrow::compute::concat_batches(&batches[0].schema(), &batches)?;
        Ok(Self::from_record_batch(merged))
    }

    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    pub fn column(&self, name: &str) -> Result<&Column> {
        self.columns
            .iter()
            .find(|c| c.name == name)
            .ok_or_else(|| Error::ColumnNotFound(name.into()))
    }

    pub fn row_count(&self) -> usize {
        self.row_count
    }

    pub fn column_count(&self) -> usize {
        self.columns.len()
    }
}
