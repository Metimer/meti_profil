import pandas as pd

import meti_profil as mp


def _df():
    return pd.DataFrame(
        {
            "age": [25, 30, None, 42, 25, 31, 29, 40, 22, 35],
            "city": ["paris", "lyon", "paris", "nice", "paris", "lyon", "nice", "paris", "lyon", "nice"],
            "score": [88.5, 92.0, 75.5, None, 88.5, 79.1, 95.2, 60.0, 84.3, 90.0],
        }
    )


def test_to_html_string():
    md = mp.ProfileReport(_df(), title="HTML Demo")
    html = md.to_html()
    assert isinstance(html, str)
    assert html.startswith("<!DOCTYPE html>")
    assert "<title>HTML Demo</title>" in html
    assert "window.METI_PROFIL" in html
    assert 'data-chart="hist"' in html
    assert html.rstrip().endswith("</html>")


def test_to_html_file(tmp_path):
    report = mp.ProfileReport(_df())
    out = tmp_path / "report.html"
    result = report.to_html(out)
    assert result is None
    assert out.exists()
    assert out.read_text().startswith("<!DOCTYPE html>")


def test_repr_html_is_sandboxed_iframe():
    report = mp.ProfileReport(_df())
    html = report._repr_html_()
    assert html.startswith("<iframe srcdoc=")
    assert 'sandbox="allow-scripts"' in html
    # The inner document's quotes are escaped for the attribute.
    assert "&quot;" in html
    assert "</iframe>" in html
