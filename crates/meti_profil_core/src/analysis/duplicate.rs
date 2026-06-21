use crate::dataframe::DataFrame;
use arrow::array::Array;
use arrow_cast::display::array_value_to_string;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize)]
pub struct DuplicateAnalysis {
    pub total_rows: usize,
    pub duplicate_rows: usize,
    pub duplicate_pct: f64,
}

impl DuplicateAnalysis {
    pub fn analyze(df: &DataFrame) -> Self {
        let total_rows = df.row_count();
        if total_rows == 0 {
            return Self {
                total_rows: 0,
                duplicate_rows: 0,
                duplicate_pct: 0.0,
            };
        }

        let mut seen: HashSet<String> = HashSet::new();
        let mut duplicates = 0usize;
        for row_idx in 0..total_rows {
            let mut key = String::new();
            for col in df.columns() {
                if col.array.is_null(row_idx) {
                    key.push_str("\u{0}NULL\u{0}");
                } else if let Ok(s) = array_value_to_string(col.array.as_ref(), row_idx) {
                    key.push_str(&s);
                } else {
                    key.push_str("\u{0}?\u{0}");
                }
                // Field separator to avoid collisions between adjacent columns.
                key.push('\u{1f}');
            }
            if !seen.insert(key) {
                duplicates += 1;
            }
        }

        Self {
            total_rows,
            duplicate_rows: duplicates,
            duplicate_pct: (duplicates as f64 / total_rows as f64) * 100.0,
        }
    }
}
