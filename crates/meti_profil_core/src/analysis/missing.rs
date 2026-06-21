use crate::dataframe::DataFrame;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ColumnMissing {
    pub name: String,
    pub missing_count: usize,
    pub missing_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MissingAnalysis {
    pub total_cells: usize,
    pub missing_cells: usize,
    pub missing_pct: f64,
    pub columns: Vec<ColumnMissing>,
}

impl MissingAnalysis {
    pub fn analyze(df: &DataFrame) -> Self {
        let total_cells = df.row_count() * df.column_count();
        let mut missing_cells = 0usize;
        let columns: Vec<ColumnMissing> = df
            .columns()
            .iter()
            .map(|col| {
                let count = col.array.null_count();
                missing_cells += count;
                let pct = if df.row_count() == 0 {
                    0.0
                } else {
                    (count as f64 / df.row_count() as f64) * 100.0
                };
                ColumnMissing {
                    name: col.name.clone(),
                    missing_count: count,
                    missing_pct: pct,
                }
            })
            .collect();
        let missing_pct = if total_cells == 0 {
            0.0
        } else {
            (missing_cells as f64 / total_cells as f64) * 100.0
        };
        Self {
            total_cells,
            missing_cells,
            missing_pct,
            columns,
        }
    }
}
