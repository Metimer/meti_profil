use crate::dataframe::DataFrame;
use arrow::array::{Array, GenericStringArray, LargeStringArray, OffsetSizeTrait, StringArray};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct Frequency {
    pub value: String,
    pub count: usize,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CategoricalStats {
    pub count: usize,
    pub missing: usize,
    pub unique_count: usize,
    pub top_values: Vec<Frequency>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CategoricalAnalysis {
    pub columns: Vec<(String, CategoricalStats)>,
}

impl CategoricalAnalysis {
    pub fn analyze(df: &DataFrame) -> Self {
        let columns: Vec<(String, CategoricalStats)> = df
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

fn analyze_array(array: &arrow::array::ArrayRef) -> Option<CategoricalStats> {
    if let Some(arr) = array.as_any().downcast_ref::<StringArray>() {
        Some(analyze_string_array(arr))
    } else {
        array
            .as_any()
            .downcast_ref::<LargeStringArray>()
            .map(analyze_string_array)
    }
}

fn analyze_string_array<O: OffsetSizeTrait>(arr: &GenericStringArray<O>) -> CategoricalStats {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for i in 0..arr.len() {
        if arr.is_null(i) {
            continue;
        }
        *counts.entry(arr.value(i)).or_insert(0) += 1;
    }

    let mut freqs: Vec<Frequency> = counts
        .iter()
        .map(|(value, count)| {
            let percentage = (*count as f64 / arr.len() as f64) * 100.0;
            Frequency {
                value: (*value).to_string(),
                count: *count,
                percentage,
            }
        })
        .collect();
    // Sort by descending count, then value for a stable, deterministic order.
    freqs.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.value.cmp(&b.value)));
    freqs.truncate(10);

    CategoricalStats {
        count: arr.len(),
        missing: arr.null_count(),
        unique_count: counts.len(),
        top_values: freqs,
    }
}
