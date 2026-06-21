use crate::dataframe::{Column, DataFrame};
use arrow::array::{
    Array, Float16Array, Float32Array, Float64Array, Int16Array, Int32Array, Int64Array,
    Int8Array, StringArray, UInt16Array, UInt32Array, UInt64Array, UInt8Array,
};
use arrow::datatypes::DataType;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum DetectedType {
    Numeric,
    Categorical,
    Boolean,
    Datetime,
    Text,
    Constant,
    Unique,
}

#[derive(Debug, Clone, Serialize)]
pub struct ColumnSchema {
    pub name: String,
    pub detected_type: DetectedType,
    pub arrow_type: String,
    pub unique_count: usize,
    pub null_count: usize,
    pub cardinality_ratio: f64,
    pub is_constant: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SchemaAnalysis {
    pub row_count: usize,
    pub column_count: usize,
    pub columns: Vec<ColumnSchema>,
}

impl SchemaAnalysis {
    pub fn analyze(df: &DataFrame) -> Self {
        let columns: Vec<ColumnSchema> = df.columns().iter().map(analyze_column).collect();
        Self {
            row_count: df.row_count(),
            column_count: df.column_count(),
            columns,
        }
    }
}

fn analyze_column(col: &Column) -> ColumnSchema {
    let row_count = col.len();
    let null_count = col.array.null_count();
    let unique_count = count_unique(&col.array);
    let cardinality_ratio = if row_count == 0 {
        0.0
    } else {
        unique_count as f64 / row_count as f64
    };
    let non_null_count = row_count - null_count;
    let is_constant = non_null_count > 0 && unique_count == 1;
    let is_unique = non_null_count > 0 && unique_count == non_null_count;

    let detected_type = if is_constant {
        DetectedType::Constant
    } else if is_unique && non_null_count > 10 {
        DetectedType::Unique
    } else {
        match &col.data_type {
            DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64
            | DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64
            | DataType::Float16 | DataType::Float32 | DataType::Float64 => DetectedType::Numeric,
            DataType::Boolean => DetectedType::Boolean,
            DataType::Utf8 | DataType::LargeUtf8 => {
                if unique_count < non_null_count && non_null_count > 0 {
                    DetectedType::Categorical
                } else {
                    DetectedType::Text
                }
            }
            DataType::Date32 | DataType::Date64 | DataType::Timestamp(_, _) => {
                DetectedType::Datetime
            }
            _ => DetectedType::Text,
        }
    };

    ColumnSchema {
        name: col.name.clone(),
        detected_type,
        arrow_type: format!("{:?}", col.data_type).to_lowercase(),
        unique_count,
        null_count,
        cardinality_ratio,
        is_constant,
    }
}

fn count_unique(array: &arrow::array::ArrayRef) -> usize {
    if let Some(arr) = array.as_any().downcast_ref::<StringArray>() {
        let mut set: HashSet<String> = HashSet::new();
        for i in 0..arr.len() {
            if arr.is_null(i) {
                continue;
            }
            set.insert(arr.value(i).to_string());
        }
        return set.len();
    }

    macro_rules! count_unique_primitive {
        ($array:expr, $ty:ty) => {{
            let arr = $array;
            let mut set: HashSet<$ty> = HashSet::new();
            for i in 0..arr.len() {
                if arr.is_null(i) {
                    continue;
                }
                set.insert(arr.value(i));
            }
            set.len()
        }};
    }

    if let Some(arr) = array.as_any().downcast_ref::<Int8Array>() {
        return count_unique_primitive!(arr, i8);
    }
    if let Some(arr) = array.as_any().downcast_ref::<Int16Array>() {
        return count_unique_primitive!(arr, i16);
    }
    if let Some(arr) = array.as_any().downcast_ref::<Int32Array>() {
        return count_unique_primitive!(arr, i32);
    }
    if let Some(arr) = array.as_any().downcast_ref::<Int64Array>() {
        return count_unique_primitive!(arr, i64);
    }
    if let Some(arr) = array.as_any().downcast_ref::<UInt8Array>() {
        return count_unique_primitive!(arr, u8);
    }
    if let Some(arr) = array.as_any().downcast_ref::<UInt16Array>() {
        return count_unique_primitive!(arr, u16);
    }
    if let Some(arr) = array.as_any().downcast_ref::<UInt32Array>() {
        return count_unique_primitive!(arr, u32);
    }
    if let Some(arr) = array.as_any().downcast_ref::<UInt64Array>() {
        return count_unique_primitive!(arr, u64);
    }
    if let Some(arr) = array.as_any().downcast_ref::<Float16Array>() {
        // f16 does not implement Hash, so fall back to byte representation.
        let mut set: HashSet<u16> = HashSet::new();
        for i in 0..arr.len() {
            if arr.is_null(i) {
                continue;
            }
            set.insert(arr.value(i).to_bits());
        }
        return set.len();
    }
    if let Some(arr) = array.as_any().downcast_ref::<Float32Array>() {
        let mut set: HashSet<u32> = HashSet::new();
        for i in 0..arr.len() {
            if arr.is_null(i) {
                continue;
            }
            set.insert(arr.value(i).to_bits());
        }
        return set.len();
    }
    if let Some(arr) = array.as_any().downcast_ref::<Float64Array>() {
        let mut set: HashSet<u64> = HashSet::new();
        for i in 0..arr.len() {
            if arr.is_null(i) {
                continue;
            }
            set.insert(arr.value(i).to_bits());
        }
        return set.len();
    }

    // Fallback: collect debug-formatted rows.
    let mut set: HashSet<String> = HashSet::new();
    for i in 0..array.len() {
        if array.is_null(i) {
            continue;
        }
        set.insert(format!("{:?}", array));
    }
    set.len()
}
