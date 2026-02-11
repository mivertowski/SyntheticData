"""dbt integration for DataSynth synthetic data.

Provides:
- DbtSourceGenerator: Generate dbt sources.yml from DataSynth output
- DbtProfile: Generate dbt profile for DataSynth output
- create_dbt_project: Scaffold a complete dbt project
"""

from __future__ import annotations

import os
import shutil
from pathlib import Path
from typing import Any, Dict, List, Optional

try:
    import yaml

    HAS_YAML = True
except ImportError:
    HAS_YAML = False


# Column type mapping from DataSynth CSV headers to dbt/SQL types
_TYPE_MAP = {
    "id": "varchar",
    "uuid": "varchar",
    "code": "varchar",
    "name": "varchar",
    "text": "varchar",
    "description": "varchar",
    "reference": "varchar",
    "type": "varchar",
    "status": "varchar",
    "currency": "varchar(3)",
    "country": "varchar(2)",
    "date": "date",
    "posting_date": "date",
    "document_date": "date",
    "created_at": "timestamp",
    "timestamp": "timestamp",
    "amount": "decimal(18,2)",
    "debit_amount": "decimal(18,2)",
    "credit_amount": "decimal(18,2)",
    "rate": "decimal(18,6)",
    "weight": "decimal(10,4)",
    "count": "integer",
    "year": "integer",
    "period": "integer",
    "month": "integer",
    "is_": "boolean",
    "enabled": "boolean",
    "active": "boolean",
}


def _infer_column_type(column_name: str) -> str:
    """Infer SQL type from column name."""
    col_lower = column_name.lower()
    for pattern, sql_type in _TYPE_MAP.items():
        if pattern in col_lower:
            return sql_type
    return "varchar"


def _read_csv_headers(csv_path: Path) -> List[str]:
    """Read column headers from a CSV file."""
    with open(csv_path, "r") as f:
        first_line = f.readline().strip()
    return [col.strip().strip('"') for col in first_line.split(",")]


class DbtSourceGenerator:
    """Generate dbt source definitions from DataSynth output."""

    def __init__(self, source_name: str = "datasynth"):
        self.source_name = source_name

    def generate_sources_yaml(
        self,
        output_path: str | Path,
        project_path: str | Path,
        schema: str = "public",
    ) -> Path:
        """Generate a dbt sources.yml file from DataSynth CSV output.

        Args:
            output_path: Path to DataSynth output directory.
            project_path: Path to dbt project root.
            schema: Database schema name.

        Returns:
            Path to generated sources.yml file.
        """
        output_dir = Path(output_path)
        project_dir = Path(project_path)
        models_dir = project_dir / "models"
        models_dir.mkdir(parents=True, exist_ok=True)

        csv_files = sorted(output_dir.rglob("*.csv"))

        tables = []
        for csv_file in csv_files:
            table_name = csv_file.stem
            headers = _read_csv_headers(csv_file)

            columns = []
            for header in headers:
                columns.append({
                    "name": header,
                    "description": f"Auto-generated from DataSynth {table_name}",
                    "data_type": _infer_column_type(header),
                })

            tables.append({
                "name": table_name,
                "description": f"Synthetic {table_name} from DataSynth",
                "columns": columns,
            })

        sources_config = {
            "version": 2,
            "sources": [{
                "name": self.source_name,
                "schema": schema,
                "tables": tables,
            }],
        }

        sources_path = models_dir / "sources.yml"
        with open(sources_path, "w") as f:
            yaml.dump(sources_config, f, default_flow_style=False, sort_keys=False)

        return sources_path

    def generate_seeds(
        self,
        output_path: str | Path,
        project_path: str | Path,
        max_files: int = 50,
    ) -> Path:
        """Copy DataSynth CSV outputs as dbt seeds.

        Args:
            output_path: Path to DataSynth output directory.
            project_path: Path to dbt project root.
            max_files: Maximum number of seed files.

        Returns:
            Path to seeds directory.
        """
        output_dir = Path(output_path)
        project_dir = Path(project_path)
        seeds_dir = project_dir / "seeds"
        seeds_dir.mkdir(parents=True, exist_ok=True)

        csv_files = sorted(output_dir.rglob("*.csv"))[:max_files]

        seed_configs = []
        for csv_file in csv_files:
            dest = seeds_dir / csv_file.name
            shutil.copy2(csv_file, dest)
            seed_configs.append({"name": csv_file.stem})

        # Generate schema.yml for seeds
        schema = {
            "version": 2,
            "seeds": seed_configs,
        }

        schema_path = seeds_dir / "schema.yml"
        with open(schema_path, "w") as f:
            yaml.dump(schema, f, default_flow_style=False, sort_keys=False)

        return seeds_dir


class DbtProfile:
    """Generate dbt profile configurations for DataSynth output."""

    @staticmethod
    def duckdb_profile(
        output_path: str | Path,
        profile_name: str = "datasynth",
    ) -> Dict[str, Any]:
        """Generate a dbt profile for DuckDB pointing at DataSynth output.

        Args:
            output_path: Path to DataSynth output directory.
            profile_name: Profile name for profiles.yml.

        Returns:
            Profile configuration dict.
        """
        return {
            profile_name: {
                "target": "dev",
                "outputs": {
                    "dev": {
                        "type": "duckdb",
                        "path": str(Path(output_path) / "datasynth.duckdb"),
                        "schema": "main",
                    }
                },
            }
        }


def create_dbt_project(
    output_path: str | Path,
    project_name: str = "datasynth_project",
    target_path: str | Path = ".",
) -> Path:
    """Scaffold a complete dbt project with DataSynth synthetic data.

    Args:
        output_path: Path to DataSynth output directory.
        project_name: Name for the dbt project.
        target_path: Where to create the project directory.

    Returns:
        Path to the created dbt project.
    """
    project_dir = Path(target_path) / project_name
    project_dir.mkdir(parents=True, exist_ok=True)

    # Create dbt_project.yml
    dbt_project = {
        "name": project_name,
        "version": "1.0.0",
        "config-version": 2,
        "profile": "datasynth",
        "model-paths": ["models"],
        "seed-paths": ["seeds"],
        "clean-targets": ["target", "dbt_packages"],
    }

    with open(project_dir / "dbt_project.yml", "w") as f:
        yaml.dump(dbt_project, f, default_flow_style=False, sort_keys=False)

    # Create profiles.yml
    profile = DbtProfile.duckdb_profile(output_path)
    with open(project_dir / "profiles.yml", "w") as f:
        yaml.dump(profile, f, default_flow_style=False, sort_keys=False)

    # Generate sources and seeds
    generator = DbtSourceGenerator()
    generator.generate_sources_yaml(output_path, project_dir)
    generator.generate_seeds(output_path, project_dir)

    return project_dir
