use crate::dataframe::DataFrame;
use arrow::array::{Array, Float64Array, Int64Array};
use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
pub struct NumericStats {
    pub count: usize,
    pub missing: usize,
    pub mean: Option<f64>,
    pub std: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub q25: Option<f64>,
    pub median: Option<f64>,
    pub q75: Option<f64>,
    pub skewness: Option<f64>,
    pub kurtosis: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NumericAnalysis {
    pub columns: Vec<(String, NumericStats)>,
}

impl NumericAnalysis {
    pub fn analyze(df: &DataFrame) -> Self {
        let columns: Vec<(String, NumericStats)> = df
            .columns()
            .iter()
            .filter_map(|col| {
                let stats = analyze_array(&col.array)?;
                Some((col.name.clone(), stats))
            })
            .collect();
        Self { columns }
    }
}

fn analyze_array(array: &arrow::array::ArrayRef) -> Option<NumericStats> {
    let values: Vec<f64> = if let Some(arr) = array.as_any().downcast_ref::<Int64Array>() {
        arr.iter().flatten().map(|v| v as f64).collect()
    } else if let Some(arr) = array.as_any().downcast_ref::<Float64Array>() {
        arr.iter().flatten().collect()
    } else {
        return None;
    };

    if values.is_empty() {
        return Some(NumericStats {
            count: array.len(),
            missing: array.null_count(),
            ..Default::default()
        });
    }

    let mut sorted = values.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    let std = variance.sqrt();

    Some(NumericStats {
        count: array.len(),
        missing: array.null_count(),
        mean: Some(mean),
        std: Some(std),
        min: sorted.first().copied(),
        max: sorted.last().copied(),
        q25: Some(percentile(&sorted, 0.25)),
        median: Some(percentile(&sorted, 0.5)),
        q75: Some(percentile(&sorted, 0.75)),
        skewness: Some(skewness(&values, mean, std)),
        kurtosis: Some(kurtosis(&values, mean, std)),
    })
}

/// Linear-interpolated percentile over an already-sorted slice.
fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.len() == 1 {
        return sorted[0];
    }
    let idx = p * (sorted.len() - 1) as f64;
    let lower = idx.floor() as usize;
    let upper = idx.ceil() as usize;
    if lower == upper {
        sorted[lower]
    } else {
        sorted[lower] + (sorted[upper] - sorted[lower]) * (idx - lower as f64)
    }
}

/// Population skewness (third standardized moment).
fn skewness(values: &[f64], mean: f64, std: f64) -> f64 {
    if std == 0.0 {
        return 0.0;
    }
    let n = values.len() as f64;
    values
        .iter()
        .map(|v| ((v - mean) / std).powi(3))
        .sum::<f64>()
        / n
}

/// Population excess kurtosis (fourth standardized moment minus 3).
fn kurtosis(values: &[f64], mean: f64, std: f64) -> f64 {
    if std == 0.0 {
        return 0.0;
    }
    let n = values.len() as f64;
    values
        .iter()
        .map(|v| ((v - mean) / std).powi(4))
        .sum::<f64>()
        / n
        - 3.0
}
