"""meti_profil — modern, Rust-powered data profiling with a Markdown report.

The public entry point is :class:`ProfileReport`. It accepts a path to a CSV,
Parquet or Excel file, or a pandas / polars DataFrame, and produces a hybrid
Markdown report (human-readable and agent-consumable).
"""

from __future__ import annotations

import os
import tempfile
from pathlib import Path
from typing import TYPE_CHECKING, Any, Optional, Union

from meti_profil._meti_profil import ProfileReport as _ProfileReport
from meti_profil._meti_profil import __version__

if TYPE_CHECKING:  # pragma: no cover
    import pandas as pd
    import polars as pl

__all__ = ["ProfileReport", "__version__"]

# File extensions the Rust engine can read directly from a path.
_NATIVE_SUFFIXES = {".csv", ".parquet", ".xlsx", ".xls"}


class ProfileReport:
    """A data profiling report.

    Parameters
    ----------
    source:
        A path to a CSV/Parquet/Excel file, or a pandas/polars DataFrame.
    title:
        Title of the report, written to the Markdown frontmatter.
    minimal:
        Reserved: reduce heavy analyses (reserved for a future release).
    explorative:
        Reserved: enable advanced analyses (reserved for a future release).
    """

    def __init__(
        self,
        source: Union[str, Path, "pd.DataFrame", "pl.DataFrame"],
        *,
        title: str = "Dataset Profile",
        minimal: bool = False,
        explorative: bool = True,
    ) -> None:
        if isinstance(source, (str, Path)):
            path = Path(source)
            if path.suffix.lower() not in _NATIVE_SUFFIXES:
                raise ValueError(
                    f"Unsupported file extension: {path.suffix!r}. "
                    f"Expected one of {sorted(_NATIVE_SUFFIXES)}."
                )
            self._report = _ProfileReport(
                str(path),
                title=title,
                source=str(source),
                minimal=minimal,
                explorative=explorative,
            )
            return

        # DataFrame input: bridge into Rust via a temporary Parquet file.
        source_label = _dataframe_label(source)
        fd, tmp_path = tempfile.mkstemp(suffix=".parquet")
        os.close(fd)
        try:
            _dataframe_to_parquet(source, tmp_path)
            self._report = _ProfileReport(
                tmp_path,
                title=title,
                source=source_label,
                minimal=minimal,
                explorative=explorative,
            )
        finally:
            if os.path.exists(tmp_path):
                os.remove(tmp_path)

    def to_file(self, path: Union[str, Path]) -> None:
        """Write the Markdown report to ``path``."""
        self._report.to_file(str(path))

    def to_markdown(self) -> str:
        """Return the Markdown report as a string."""
        return self._report.to_markdown()

    def get_summary(self) -> dict:
        """Return a dictionary of dataset-level summary metrics."""
        return self._report.get_summary()

    def get_column_info(self, name: str) -> Optional[dict]:
        """Return per-column schema information, or ``None`` if unknown."""
        return self._report.get_column_info(name)


def _dataframe_label(source: Any) -> str:
    module = type(source).__module__.split(".")[0]
    return f"<{module}.DataFrame>"


def _dataframe_to_parquet(source: Any, path: str) -> None:
    # Use snappy: fast, always supported by the Rust reader, and avoids relying
    # on the writer's default codec (pandas/polars default to zstd).
    module = type(source).__module__.split(".")[0]
    if module == "pandas":
        source.to_parquet(path, index=False, compression="snappy")
    elif module == "polars":
        source.write_parquet(path, compression="snappy")
    else:
        raise TypeError(
            "source must be a file path, a pandas.DataFrame or a polars.DataFrame, "
            f"got {type(source)!r}"
        )
