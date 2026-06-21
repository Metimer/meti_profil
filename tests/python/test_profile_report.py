import pandas as pd
import polars as pl
import pytest

import meti_profil as mp


@pytest.fixture
def csv_path(tmp_path):
    path = tmp_path / "data.csv"
    path.write_text(
        "age,name,score\n"
        "25,alice,88.5\n"
        "30,bob,92.0\n"
        ",claire,75.5\n"
        "42,david,\n"
        "25,alice,88.5\n"
    )
    return path


def test_profile_report_from_csv(csv_path):
    report = mp.ProfileReport(csv_path, title="CSV Test")
    md = report.to_markdown()
    assert "## Schema" in md
    assert "## Numeric Columns" in md
    assert "CSV Test" in md


def test_profile_report_from_pandas():
    df = pd.DataFrame(
        {
            "age": [25, 30, None, 42, 25],
            "name": ["alice", "bob", "claire", "david", "alice"],
            "score": [88.5, 92.0, 75.5, None, 88.5],
        }
    )
    report = mp.ProfileReport(df)
    summary = report.get_summary()
    assert summary["rows"] == 5
    assert summary["columns"] == 3


def test_profile_report_from_polars():
    df = pl.DataFrame(
        {
            "age": [25, 30, 40, 42, 25],
            "name": ["alice", "bob", "claire", "david", "alice"],
        }
    )
    report = mp.ProfileReport(df)
    summary = report.get_summary()
    assert summary["rows"] == 5
    assert summary["columns"] == 2


def test_profile_report_to_file(tmp_path):
    df = pd.DataFrame({"a": [1, 2, 3]})
    report = mp.ProfileReport(df)
    out = tmp_path / "report.md"
    report.to_file(out)
    assert out.exists()
    assert "## Schema" in out.read_text()


def test_get_column_info():
    df = pd.DataFrame({"age": [25, 30, 40, 42, 25]})
    report = mp.ProfileReport(df)
    info = report.get_column_info("age")
    assert info is not None
    assert info["name"] == "age"
    assert report.get_column_info("does_not_exist") is None


def test_unsupported_extension(tmp_path):
    bad = tmp_path / "data.json"
    bad.write_text("{}")
    with pytest.raises(ValueError):
        mp.ProfileReport(bad)
