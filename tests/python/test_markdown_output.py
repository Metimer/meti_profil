import pandas as pd

import meti_profil as mp


def test_markdown_contains_frontmatter():
    df = pd.DataFrame({"a": [1, 2, 3], "b": [4, 5, 6]})
    report = mp.ProfileReport(df, title="My Report")
    md = report.to_markdown()
    assert md.startswith("---")
    assert "title: My Report" in md
    assert "meti_profil_version:" in md


def test_markdown_contains_sections():
    df = pd.DataFrame(
        {
            "age": [25, 30, None, 42, 25],
            "name": ["alice", "bob", "claire", "david", "alice"],
            "score": [88.5, 92.0, 75.5, None, 88.5],
        }
    )
    report = mp.ProfileReport(df)
    md = report.to_markdown()
    assert "## Overview" in md
    assert "## Schema" in md
    assert "## Numeric Columns" in md
    assert "## Categorical Columns" in md
    assert "## Missing Values" in md
    assert "## Duplicate Rows" in md


def test_markdown_has_consistent_row_count():
    df = pd.DataFrame({"x": range(100)})
    report = mp.ProfileReport(df)
    md = report.to_markdown()
    assert "rows: 100" in md
    assert "| Rows | 100 |" in md
