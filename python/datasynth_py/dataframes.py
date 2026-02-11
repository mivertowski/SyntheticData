"""DataFrame integration for DataSynth output.

Provides utilities to convert generation results into pandas or polars DataFrames.
"""

from __future__ import annotations

import os
from typing import List, Optional


def list_tables(result) -> List[str]:
    """List available output tables from a generation result.

    Args:
        result: Generation result from DataSynth.generate()

    Returns:
        List of table names (CSV filenames without extension).
    """
    output_dir = _get_output_dir(result)
    if not output_dir or not os.path.isdir(output_dir):
        return []

    tables = []
    for f in sorted(os.listdir(output_dir)):
        if f.endswith(".csv"):
            tables.append(f[:-4])  # Remove .csv extension
    return tables


def to_pandas(result, table_name: str, **kwargs):
    """Convert a generation output table to a pandas DataFrame.

    Args:
        result: Generation result from DataSynth.generate()
        table_name: Name of the table (e.g., "journal_entries", "vendors")
        **kwargs: Additional arguments passed to pandas.read_csv()

    Returns:
        pandas.DataFrame

    Raises:
        ImportError: If pandas is not installed.
        FileNotFoundError: If the table CSV file doesn't exist.
    """
    try:
        import pandas as pd
    except ImportError:
        raise ImportError(
            "pandas is required for to_pandas(). "
            "Install it with: pip install 'datasynth-py[pandas]'"
        )

    csv_path = _resolve_table_path(result, table_name)
    # Skip comment lines that start with #  (synthetic content markers)
    return pd.read_csv(csv_path, comment="#", **kwargs)


def to_polars(result, table_name: str, **kwargs):
    """Convert a generation output table to a polars DataFrame.

    Args:
        result: Generation result from DataSynth.generate()
        table_name: Name of the table (e.g., "journal_entries", "vendors")
        **kwargs: Additional arguments passed to polars.read_csv()

    Returns:
        polars.DataFrame

    Raises:
        ImportError: If polars is not installed.
        FileNotFoundError: If the table CSV file doesn't exist.
    """
    try:
        import polars as pl
    except ImportError:
        raise ImportError(
            "polars is required for to_polars(). "
            "Install it with: pip install 'datasynth-py[polars]'"
        )

    csv_path = _resolve_table_path(result, table_name)
    return pl.read_csv(csv_path, comment_prefix="#", **kwargs)


def _get_output_dir(result) -> Optional[str]:
    """Extract output directory from a generation result."""
    if isinstance(result, dict):
        return result.get("output_dir") or result.get("output_directory")
    if hasattr(result, "output_dir"):
        return result.output_dir
    return None


def _resolve_table_path(result, table_name: str) -> str:
    """Resolve the full path to a table CSV file."""
    output_dir = _get_output_dir(result)
    if not output_dir:
        raise ValueError(
            "Cannot determine output directory from result. "
            "Ensure the result contains 'output_dir' or 'output_directory'."
        )

    # Try with and without .csv extension
    if table_name.endswith(".csv"):
        csv_path = os.path.join(output_dir, table_name)
    else:
        csv_path = os.path.join(output_dir, f"{table_name}.csv")

    if not os.path.isfile(csv_path):
        # Also check in subdirectories (e.g., trial_balances/)
        for subdir in os.listdir(output_dir):
            subpath = os.path.join(output_dir, subdir)
            if os.path.isdir(subpath):
                candidate = os.path.join(subpath, f"{table_name}.csv")
                if os.path.isfile(candidate):
                    return candidate

        available = list_tables(result)
        raise FileNotFoundError(
            f"Table '{table_name}' not found in {output_dir}. "
            f"Available tables: {available}"
        )

    return csv_path
