# meti_profil

A modern, Rust-powered data profiling library with Python bindings. It reads
CSV, Parquet, and Excel files (or pandas / polars DataFrames) and generates a
hybrid Markdown report that is readable by humans and structured for consumption
by code agents.

## Installation

```bash
pip install meti_profil
```

## Quick start

```python
import meti_profil as mp

# From a file
report = mp.ProfileReport("data.csv", title="My dataset")
report.to_file("profile.md")

# From a pandas DataFrame
import pandas as pd
df = pd.read_csv("data.csv")
report = mp.ProfileReport(df)

# Programmatic access
print(report.get_summary())          # dataset-level metrics
print(report.get_column_info("age")) # per-column schema info
markdown = report.to_markdown()
```

### `ProfileReport` parameters

| Parameter     | Type                                   | Default             | Description                                  |
|---------------|----------------------------------------|---------------------|----------------------------------------------|
| `source`      | `str`, `Path`, pandas/polars DataFrame | required            | Data source.                                 |
| `title`       | `str`                                  | `"Dataset Profile"` | Report title (written to the frontmatter).   |
| `minimal`     | `bool`                                 | `False`             | Reserved: reduce heavy analyses.             |
| `explorative` | `bool`                                 | `True`              | Reserved: enable advanced analyses.          |

## Report format

The Markdown report starts with a YAML frontmatter block (rows, columns,
missing cells, duplicates, version) followed by normalized `## ` sections:
`Overview`, `Schema`, `Numeric Columns`, `Categorical Columns`, `Missing
Values`, `Duplicate Rows`, and `Correlations`.

## Features

- Fast Rust engine backed by Apache Arrow.
- Reads CSV, Parquet (snappy/zstd/lz4/brotli/gzip), and Excel files.
- Accepts pandas and polars DataFrames.
- Schema/type detection, descriptive numeric statistics, categorical
  frequencies, missing-value and duplicate-row analysis, and Pearson
  correlations.
- Clean Markdown reports optimized for both humans and code agents.

## Development

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install maturin pytest pandas polars pyarrow

# Build the extension in-place
maturin develop

# Run the test suites
cargo test --workspace
pytest tests/python -v
```

## License

MIT
