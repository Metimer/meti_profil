use crate::dataframe::{Column, DataFrame};
use arrow::array::{Array, Float64Array, Int64Array};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CorrelationPair {
    pub column_a: String,
    pub column_b: String,
    pub pearson: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CorrelationAnalysis {
    /// Numeric column names, in the order used by `matrix`.
    pub columns: Vec<String>,
    /// Full N×N Pearson matrix (`None` where a coefficient is undefined).
    pub matrix: Vec<Vec<Option<f64>>>,
    /// Pairs whose absolute correlation exceeds [`HIGH_CORRELATION_THRESHOLD`].
    pub pairs: Vec<CorrelationPair>,
}

/// Absolute Pearson correlation above which a pair is flagged as highly correlated.
const HIGH_CORRELATION_THRESHOLD: f64 = 0.9;

impl CorrelationAnalysis {
    pub fn analyze(df: &DataFrame) -> Self {
        let numeric: Vec<&Column> = df.columns().iter().filter(|c| is_numeric(c)).collect();
        let n = numeric.len();
        let columns: Vec<String> = numeric.iter().map(|c| c.name.clone()).collect();

        let mut matrix: Vec<Vec<Option<f64>>> = vec![vec![None; n]; n];
        let mut pairs = Vec::new();
        for i in 0..n {
            matrix[i][i] = Some(1.0);
            for j in (i + 1)..n {
                let coeff = pearson(numeric[i], numeric[j]);
                matrix[i][j] = coeff;
                matrix[j][i] = coeff;
                if let Some(pearson) = coeff {
                    if pearson.abs() > HIGH_CORRELATION_THRESHOLD {
                        pairs.push(CorrelationPair {
                            column_a: numeric[i].name.clone(),
                            column_b: numeric[j].name.clone(),
                            pearson,
                        });
                    }
                }
            }
        }
        Self {
            columns,
            matrix,
            pairs,
        }
    }
}

fn is_numeric(col: &Column) -> bool {
    col.array.as_any().downcast_ref::<Int64Array>().is_some()
        || col.array.as_any().downcast_ref::<Float64Array>().is_some()
}

/// Extract values together with a per-row validity mask so paired columns can
/// drop rows where either side is null.
fn values_with_mask(col: &Column) -> Option<(Vec<f64>, Vec<bool>)> {
    if let Some(arr) = col.array.as_any().downcast_ref::<Int64Array>() {
        let values = (0..arr.len())
            .map(|i| {
                if arr.is_null(i) {
                    0.0
                } else {
                    arr.value(i) as f64
                }
            })
            .collect();
        let mask = (0..arr.len()).map(|i| !arr.is_null(i)).collect();
        Some((values, mask))
    } else if let Some(arr) = col.array.as_any().downcast_ref::<Float64Array>() {
        let values = (0..arr.len())
            .map(|i| if arr.is_null(i) { 0.0 } else { arr.value(i) })
            .collect();
        let mask = (0..arr.len()).map(|i| !arr.is_null(i)).collect();
        Some((values, mask))
    } else {
        None
    }
}

fn pearson(a: &Column, b: &Column) -> Option<f64> {
    let (xs, xmask) = values_with_mask(a)?;
    let (ys, ymask) = values_with_mask(b)?;
    if xs.len() != ys.len() {
        return None;
    }

    // Keep only rows where both columns are non-null.
    let paired: Vec<(f64, f64)> = xs
        .iter()
        .zip(&ys)
        .zip(xmask.iter().zip(&ymask))
        .filter_map(|((x, y), (&mx, &my))| if mx && my { Some((*x, *y)) } else { None })
        .collect();
    if paired.len() < 2 {
        return None;
    }

    let n = paired.len() as f64;
    let mean_x = paired.iter().map(|(x, _)| x).sum::<f64>() / n;
    let mean_y = paired.iter().map(|(_, y)| y).sum::<f64>() / n;
    let mut num = 0.0;
    let mut denom_x = 0.0;
    let mut denom_y = 0.0;
    for (x, y) in &paired {
        let dx = x - mean_x;
        let dy = y - mean_y;
        num += dx * dy;
        denom_x += dx * dx;
        denom_y += dy * dy;
    }
    let denom = denom_x.sqrt() * denom_y.sqrt();
    if denom == 0.0 {
        return None;
    }
    Some(num / denom)
}
