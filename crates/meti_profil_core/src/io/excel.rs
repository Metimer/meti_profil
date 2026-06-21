use crate::dataframe::DataFrame;
use crate::error::{Error, Result};
use arrow::array::{ArrayRef, Float64Array, Int64Array, StringArray};
use arrow::datatypes::{Field, Schema};
use calamine::{open_workbook, Data, Reader, Xlsx, XlsxError};
use std::path::Path;
use std::sync::Arc;

pub fn read_excel(path: impl AsRef<Path>) -> Result<DataFrame> {
    let path = path.as_ref();
    let mut workbook: Xlsx<_> = open_workbook(path).map_err(|e: XlsxError| Error::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        e.to_string(),
    )))?;
    let sheet_name = workbook.sheet_names().first().cloned().ok_or_else(|| {
        Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "no sheet found",
        ))
    })?;
    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e: XlsxError| Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        )))?;

    let mut rows = range.rows();
    let header_row = rows.next().ok_or_else(|| {
        Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "empty excel sheet",
        ))
    })?;
    let headers: Vec<String> = header_row
        .iter()
        .map(|c| c.to_string().trim().to_string())
        .collect();

    let mut columns: Vec<Vec<Data>> = vec![Vec::new(); headers.len()];
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if let Some(col) = columns.get_mut(i) {
                col.push(cell.clone());
            }
        }
    }

    let arrays: Vec<ArrayRef> = columns
        .iter()
        .map(|cells| infer_and_build_array(cells))
        .collect();

    let fields: Vec<Field> = headers
        .iter()
        .zip(&arrays)
        .map(|(name, arr)| Field::new(name, arr.data_type().clone(), true))
        .collect();

    let schema = Arc::new(Schema::new(fields));
    let batch = arrow::record_batch::RecordBatch::try_new(schema, arrays)?;
    Ok(DataFrame::from_record_batch(batch))
}

fn infer_and_build_array(cells: &[Data]) -> ArrayRef {
    let all_int = cells.iter().all(|c| matches!(c, Data::Int(_) | Data::Empty));
    let all_float = cells.iter().all(|c| {
        matches!(c, Data::Float(_) | Data::Int(_) | Data::Empty)
    });

    if all_int {
        Arc::new(Int64Array::from(
            cells
                .iter()
                .map(|c| match c {
                    Data::Int(v) => Some(*v),
                    _ => None,
                })
                .collect::<Vec<_>>(),
        )) as ArrayRef
    } else if all_float {
        Arc::new(Float64Array::from(
            cells
                .iter()
                .map(|c| match c {
                    Data::Int(v) => Some(*v as f64),
                    Data::Float(v) => Some(*v),
                    _ => None,
                })
                .collect::<Vec<_>>(),
        )) as ArrayRef
    } else {
        Arc::new(StringArray::from(
            cells
                .iter()
                .map(|c| {
                    if matches!(c, Data::Empty) {
                        None
                    } else {
                        Some(c.to_string())
                    }
                })
                .collect::<Vec<_>>(),
        )) as ArrayRef
    }
}
