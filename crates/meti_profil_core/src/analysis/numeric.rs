use crate::dataframe::DataFrame;
use arrow::array::{Array, Float64Array, Int64Array};
use serde::Serialize;

/// Default number of bins used for numeric histograms.
const HISTOGRAM_BINS: usize = 10;

/// A single histogram bin covering the half-open interval `[lower, upper)`
/// (the last bin is closed on both ends).
#[derive(Debug, Clone, Serialize)]
pub struct HistogramBin {
    pub lower: f64,
    pub upper: f64,
    pub count: usize,
}

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
    #[serde(default)]
    pub histogram: Vec<HistogramBin>,
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

    let min = sorted.first().copied();
    let max = sorted.last().copied();
    let histogram = match (min, max) {
        (Some(lo), Some(hi)) => histogram(&sorted, lo, hi),
        _ => Vec::new(),
    };

    Some(NumericStats {
        count: array.len(),
        missing: array.null_count(),
        mean: Some(mean),
        std: Some(std),
        min,
        max,
        q25: Some(percentile(&sorted, 0.25)),
        median: Some(percentile(&sorted, 0.5)),
        q75: Some(percentile(&sorted, 0.75)),
        skewness: Some(skewness(&values, mean, std)),
        kurtosis: Some(kurtosis(&values, mean, std)),
        histogram,
    })
}

/// Build an equal-width histogram over `[min, max]` from a sorted slice.
fn histogram(sorted: &[f64], min: f64, max: f64) -> Vec<HistogramBin> {
    if sorted.is_empty() {
        return Vec::new();
    }
    // Degenerate range (all values equal): a single bin holding everything.
    if max <= min {
        return vec![HistogramBin {
            lower: min,
            upper: max,
            count: sorted.len(),
        }];
    }
    let bins = HISTOGRAM_BINS;
    let width = (max - min) / bins as f64;
    let mut counts = vec![0usize; bins];
    for &v in sorted {
        let mut idx = ((v - min) / width).floor() as usize;
        if idx >= bins {
            idx = bins - 1; // the maximum value falls in the last bin
        }
        counts[idx] += 1;
    }
    counts
        .into_iter()
        .enumerate()
        .map(|(i, count)| HistogramBin {
            lower: min + width * i as f64,
            upper: min + width * (i + 1) as f64,
            count,
        })
        .collect()
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
