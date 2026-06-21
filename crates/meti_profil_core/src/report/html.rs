// The renderer deliberately writes newline-terminated HTML lines for output
// readability; `writeln!` would obscure that intent here.
#![allow(clippy::write_with_newline)]

use crate::report::model::Report;
use std::collections::HashSet;
use std::fmt::Write as _;

const CSS: &str = include_str!("assets/report.css");
const JS: &str = include_str!("assets/report.js");

/// Renders a [`Report`] into a self-contained, interactive HTML document.
///
/// The document embeds its own CSS and JavaScript (no external resources), so it
/// works fully offline and can be shared as a single file. Charts are drawn as
/// interactive SVG from a JSON payload embedded in the page.
pub struct HtmlRenderer;

impl HtmlRenderer {
    pub fn render(report: &Report) -> String {
        let body = Self::render_body(report);
        let payload = json_payload(report);
        let mut out = String::new();
        out.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
        out.push_str("<meta charset=\"utf-8\">\n");
        out.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
        let _ = write!(out, "<title>{}</title>\n", esc(&report.title));
        let _ = write!(out, "<style>\n{CSS}\n</style>\n");
        out.push_str("</head>\n<body>\n");
        out.push_str(&body);
        let _ = write!(out, "<script>window.METI_PROFIL = {payload};</script>\n");
        let _ = write!(out, "<script>\n{JS}\n</script>\n");
        out.push_str("</body>\n</html>\n");
        out
    }

    fn render_body(report: &Report) -> String {
        let mut b = String::new();
        b.push_str("<div class=\"mp-wrap\">\n");
        Self::header(report, &mut b);
        Self::cards(report, &mut b);
        Self::schema(report, &mut b);
        Self::numeric(report, &mut b);
        Self::categorical(report, &mut b);
        Self::missing(report, &mut b);
        Self::correlations(report, &mut b);
        Self::footer(report, &mut b);
        b.push_str("</div>\n");
        b
    }

    fn header(report: &Report, b: &mut String) {
        b.push_str("<header class=\"mp-head\">\n");
        let _ = write!(b, "<h1>{}</h1>\n", esc(&report.title));
        b.push_str("<div class=\"mp-sub\">");
        if let Some(src) = &report.source {
            let _ = write!(b, "source <code>{}</code> &middot; ", esc(src));
        }
        let _ = write!(b, "meti_profil v{}</div>\n", esc(&report.version));
        b.push_str("</header>\n");
    }

    fn cards(report: &Report, b: &mut String) {
        b.push_str("<div class=\"mp-cards\">\n");
        card(b, &report.schema.row_count.to_string(), "Rows", "");
        card(b, &report.schema.column_count.to_string(), "Columns", "");
        card(
            b,
            &report.missing.missing_cells.to_string(),
            "Missing cells",
            &format!("{:.2}%", report.missing.missing_pct),
        );
        card(
            b,
            &report.duplicates.duplicate_rows.to_string(),
            "Duplicate rows",
            &format!("{:.2}%", report.duplicates.duplicate_pct),
        );
        b.push_str("</div>\n");
    }

    fn schema(report: &Report, b: &mut String) {
        b.push_str("<section class=\"mp-sec\">\n<h2>Schema</h2>\n");
        b.push_str("<table class=\"mp-tbl\">\n<thead><tr>");
        b.push_str("<th>Column</th><th>Type</th><th>Arrow type</th>");
        b.push_str(
            "<th class=\"num\">Unique</th><th class=\"num\">Missing</th></tr></thead>\n<tbody>\n",
        );
        let rows = report.schema.row_count.max(1) as f64;
        for col in &report.schema.columns {
            let ty = format!("{:?}", col.detected_type);
            let pct = (col.null_count as f64 / rows) * 100.0;
            let _ = write!(
                b,
                "<tr><td>{}</td><td><span class=\"mp-pill t-{}\">{}</span></td><td>{}</td>\
                 <td class=\"num\">{}</td><td class=\"num\">{} ({:.1}%)</td></tr>\n",
                esc(&col.name),
                esc(&ty),
                esc(&ty),
                esc(&col.arrow_type),
                col.unique_count,
                col.null_count,
                pct,
            );
        }
        b.push_str("</tbody>\n</table>\n</section>\n");
    }

    fn numeric(report: &Report, b: &mut String) {
        let keep: HashSet<&str> = report.numeric_column_names().into_iter().collect();
        let cols: Vec<_> = report
            .numeric
            .columns
            .iter()
            .filter(|(name, _)| keep.contains(name.as_str()))
            .collect();
        if cols.is_empty() {
            return;
        }
        b.push_str("<section class=\"mp-sec\">\n<h2>Numeric columns</h2>\n");
        for (name, s) in cols {
            let _ = write!(b, "<div class=\"mp-col\">\n<h3>{}</h3>\n", esc(name));
            b.push_str("<div class=\"mp-col-grid\">\n");
            // Left: stat table.
            b.push_str("<table class=\"mp-tbl\">\n<tbody>\n");
            stat_row(b, "count", &s.count.to_string());
            stat_row(b, "missing", &s.missing.to_string());
            opt_row(b, "mean", s.mean);
            opt_row(b, "std", s.std);
            opt_row(b, "min", s.min);
            opt_row(b, "25%", s.q25);
            opt_row(b, "median", s.median);
            opt_row(b, "75%", s.q75);
            opt_row(b, "max", s.max);
            opt_row(b, "skewness", s.skewness);
            opt_row(b, "kurtosis", s.kurtosis);
            b.push_str("</tbody>\n</table>\n");
            // Right: histogram placeholder.
            let _ = write!(
                b,
                "<div class=\"mp-chart\" data-chart=\"hist\" data-col=\"{}\"></div>\n",
                esc_attr(name)
            );
            b.push_str("</div>\n</div>\n");
        }
        b.push_str("</section>\n");
    }

    fn categorical(report: &Report, b: &mut String) {
        let keep: HashSet<&str> = report.categorical_column_names().into_iter().collect();
        let cols: Vec<_> = report
            .categorical
            .columns
            .iter()
            .filter(|(name, _)| keep.contains(name.as_str()))
            .collect();
        if cols.is_empty() {
            return;
        }
        b.push_str("<section class=\"mp-sec\">\n<h2>Categorical columns</h2>\n");
        for (name, s) in cols {
            let _ = write!(b, "<div class=\"mp-col\">\n<h3>{}</h3>\n", esc(name));
            let _ = write!(
                b,
                "<div class=\"mp-sub\" style=\"margin-bottom:8px\">{} unique values</div>\n",
                s.unique_count
            );
            let _ = write!(
                b,
                "<div class=\"mp-chart\" data-chart=\"cat\" data-col=\"{}\"></div>\n",
                esc_attr(name)
            );
            b.push_str("</div>\n");
        }
        b.push_str("</section>\n");
    }

    fn missing(report: &Report, b: &mut String) {
        b.push_str("<section class=\"mp-sec\">\n<h2>Missing values</h2>\n");
        let _ = write!(
            b,
            "<div class=\"mp-sub\" style=\"margin-bottom:10px\">{} missing cells ({:.2}% of all cells)</div>\n",
            report.missing.missing_cells, report.missing.missing_pct
        );
        b.push_str("<div class=\"mp-chart\" data-chart=\"missing\"></div>\n");
        b.push_str("</section>\n");
    }

    fn correlations(report: &Report, b: &mut String) {
        if report.correlations.columns.len() < 2 {
            return;
        }
        b.push_str("<section class=\"mp-sec\">\n<h2>Correlations</h2>\n");
        b.push_str("<div class=\"mp-sub\" style=\"margin-bottom:10px\">Pearson correlation between numeric columns</div>\n");
        b.push_str("<div class=\"mp-chart\" data-chart=\"corr\"></div>\n");
        if !report.correlations.pairs.is_empty() {
            b.push_str("<table class=\"mp-tbl\" style=\"margin-top:14px\">\n");
            b.push_str("<thead><tr><th>Highly correlated pair</th><th class=\"num\">Pearson</th></tr></thead>\n<tbody>\n");
            for p in &report.correlations.pairs {
                let _ = write!(
                    b,
                    "<tr><td>{} &times; {}</td><td class=\"num\">{:.4}</td></tr>\n",
                    esc(&p.column_a),
                    esc(&p.column_b),
                    p.pearson
                );
            }
            b.push_str("</tbody>\n</table>\n");
        }
        b.push_str("</section>\n");
    }

    fn footer(report: &Report, b: &mut String) {
        let _ = write!(
            b,
            "<footer class=\"mp-foot\">Generated by <a href=\"https://github.com/Metimer/meti_profil\">meti_profil</a> v{}</footer>\n",
            esc(&report.version)
        );
    }
}

fn card(b: &mut String, value: &str, key: &str, sub: &str) {
    b.push_str("<div class=\"mp-card\">");
    let _ = write!(b, "<div class=\"v\">{}</div>", esc(value));
    let _ = write!(b, "<div class=\"k\">{}</div>", esc(key));
    if !sub.is_empty() {
        let _ = write!(b, "<div class=\"s\">{}</div>", esc(sub));
    }
    b.push_str("</div>\n");
}

fn stat_row(b: &mut String, label: &str, value: &str) {
    let _ = write!(
        b,
        "<tr><td>{}</td><td class=\"num\">{}</td></tr>\n",
        esc(label),
        esc(value)
    );
}

fn opt_row(b: &mut String, label: &str, value: Option<f64>) {
    let v = match value {
        Some(v) => format!("{v:.4}"),
        None => "–".to_string(),
    };
    stat_row(b, label, &v);
}

/// Serialize the report to JSON, made safe to embed inside a `<script>` tag by
/// neutralizing any `</` sequence.
fn json_payload(report: &Report) -> String {
    let json = serde_json::to_string(report).unwrap_or_else(|_| "{}".to_string());
    json.replace("</", "<\\/")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}

/// Escape text for use in HTML element content.
fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

/// Escape text for use in a double-quoted HTML attribute.
fn esc_attr(s: &str) -> String {
    esc(s).replace('"', "&quot;")
}
