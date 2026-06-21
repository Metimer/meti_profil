use crate::report::model::Report;
use std::collections::HashSet;
use std::fmt::Write as _;

/// Renders a [`Report`] into a hybrid Markdown document: readable by humans and
/// structured (stable headings + YAML frontmatter) for consumption by agents.
pub struct MarkdownRenderer;

impl MarkdownRenderer {
    pub fn render(report: &Report) -> String {
        let mut out = String::new();
        Self::render_frontmatter(report, &mut out);
        writeln!(out, "\n# {}", report.title).unwrap();
        Self::render_overview(report, &mut out);
        Self::render_schema(report, &mut out);
        Self::render_numeric(report, &mut out);
        Self::render_categorical(report, &mut out);
        Self::render_missing(report, &mut out);
        Self::render_duplicates(report, &mut out);
        Self::render_correlations(report, &mut out);
        out
    }

    fn render_frontmatter(report: &Report, out: &mut String) {
        writeln!(out, "---").unwrap();
        writeln!(out, "title: {}", report.title).unwrap();
        if let Some(source) = &report.source {
            writeln!(out, "source: {}", source).unwrap();
        }
        writeln!(out, "rows: {}", report.schema.row_count).unwrap();
        writeln!(out, "columns: {}", report.schema.column_count).unwrap();
        writeln!(out, "missing_cells: {}", report.missing.missing_cells).unwrap();
        writeln!(out, "missing_cells_pct: {:.2}", report.missing.missing_pct).unwrap();
        writeln!(out, "duplicate_rows: {}", report.duplicates.duplicate_rows).unwrap();
        writeln!(
            out,
            "duplicate_rows_pct: {:.2}",
            report.duplicates.duplicate_pct
        )
        .unwrap();
        writeln!(out, "meti_profil_version: {}", report.version).unwrap();
        writeln!(out, "---").unwrap();
    }

    fn render_overview(report: &Report, out: &mut String) {
        writeln!(out, "\n## Overview").unwrap();
        writeln!(out, "\n| Metric | Value |").unwrap();
        writeln!(out, "|--------|------:|").unwrap();
        writeln!(out, "| Rows | {} |", report.schema.row_count).unwrap();
        writeln!(out, "| Columns | {} |", report.schema.column_count).unwrap();
        writeln!(
            out,
            "| Missing cells | {} ({:.2}%) |",
            report.missing.missing_cells, report.missing.missing_pct
        )
        .unwrap();
        writeln!(
            out,
            "| Duplicate rows | {} ({:.2}%) |",
            report.duplicates.duplicate_rows, report.duplicates.duplicate_pct
        )
        .unwrap();
    }

    fn render_schema(report: &Report, out: &mut String) {
        writeln!(out, "\n## Schema").unwrap();
        writeln!(
            out,
            "\n| Column | Detected type | Arrow type | Unique | Missing |"
        )
        .unwrap();
        writeln!(
            out,
            "|--------|---------------|------------|-------:|--------:|"
        )
        .unwrap();
        let rows = report.schema.row_count.max(1) as f64;
        for col in &report.schema.columns {
            writeln!(
                out,
                "| {} | {:?} | {} | {} | {} ({:.1}%) |",
                col.name,
                col.detected_type,
                col.arrow_type,
                col.unique_count,
                col.null_count,
                (col.null_count as f64 / rows) * 100.0
            )
            .unwrap();
        }
    }

    fn render_numeric(report: &Report, out: &mut String) {
        let keep: HashSet<&str> = report.numeric_column_names().into_iter().collect();
        let columns: Vec<_> = report
            .numeric
            .columns
            .iter()
            .filter(|(name, _)| keep.contains(name.as_str()))
            .collect();
        if columns.is_empty() {
            return;
        }
        writeln!(out, "\n## Numeric Columns").unwrap();
        for (name, stats) in columns {
            writeln!(out, "\n### {}", name).unwrap();
            writeln!(out, "\n| Statistic | Value |").unwrap();
            writeln!(out, "|-----------|------:|").unwrap();
            writeln!(out, "| count | {} |", stats.count).unwrap();
            writeln!(out, "| missing | {} |", stats.missing).unwrap();
            Self::write_opt(out, "mean", stats.mean);
            Self::write_opt(out, "std", stats.std);
            Self::write_opt(out, "min", stats.min);
            Self::write_opt(out, "25%", stats.q25);
            Self::write_opt(out, "median", stats.median);
            Self::write_opt(out, "75%", stats.q75);
            Self::write_opt(out, "max", stats.max);
            Self::write_opt(out, "skewness", stats.skewness);
            Self::write_opt(out, "kurtosis", stats.kurtosis);
        }
    }

    fn write_opt(out: &mut String, label: &str, value: Option<f64>) {
        match value {
            Some(v) => writeln!(out, "| {} | {:.4} |", label, v).unwrap(),
            None => writeln!(out, "| {} | - |", label).unwrap(),
        }
    }

    fn render_categorical(report: &Report, out: &mut String) {
        let keep: HashSet<&str> = report.categorical_column_names().into_iter().collect();
        let columns: Vec<_> = report
            .categorical
            .columns
            .iter()
            .filter(|(name, _)| keep.contains(name.as_str()))
            .collect();
        if columns.is_empty() {
            return;
        }
        writeln!(out, "\n## Categorical Columns").unwrap();
        for (name, stats) in columns {
            writeln!(out, "\n### {}", name).unwrap();
            writeln!(out, "\nUnique values: {}", stats.unique_count).unwrap();
            writeln!(out, "\n| Value | Count | Percentage |").unwrap();
            writeln!(out, "|-------|------:|-----------:|").unwrap();
            for freq in &stats.top_values {
                writeln!(
                    out,
                    "| {} | {} | {:.1}% |",
                    freq.value, freq.count, freq.percentage
                )
                .unwrap();
            }
        }
    }

    fn render_missing(report: &Report, out: &mut String) {
        writeln!(out, "\n## Missing Values").unwrap();
        writeln!(out, "\n| Column | Missing | Percentage |").unwrap();
        writeln!(out, "|--------|--------:|-----------:|").unwrap();
        for col in &report.missing.columns {
            writeln!(
                out,
                "| {} | {} | {:.2}% |",
                col.name, col.missing_count, col.missing_pct
            )
            .unwrap();
        }
    }

    fn render_duplicates(report: &Report, out: &mut String) {
        writeln!(out, "\n## Duplicate Rows").unwrap();
        writeln!(
            out,
            "\nTotal duplicate rows: {} ({:.2}%)",
            report.duplicates.duplicate_rows, report.duplicates.duplicate_pct
        )
        .unwrap();
    }

    fn render_correlations(report: &Report, out: &mut String) {
        if report.correlations.pairs.is_empty() {
            return;
        }
        writeln!(out, "\n## Correlations").unwrap();
        writeln!(out, "\nHighly correlated pairs (|Pearson| > 0.9):").unwrap();
        writeln!(out, "\n| Column A | Column B | Pearson |").unwrap();
        writeln!(out, "|----------|----------|--------:|").unwrap();
        for pair in &report.correlations.pairs {
            writeln!(
                out,
                "| {} | {} | {:.4} |",
                pair.column_a, pair.column_b, pair.pearson
            )
            .unwrap();
        }
    }
}
