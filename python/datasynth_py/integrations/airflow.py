"""Apache Airflow operators for DataSynth synthetic data generation.

Provides:
- DataSynthOperator: Generate synthetic data as an Airflow task
- DataSynthSensor: Wait for generation completion
- DataSynthValidateOperator: Validate configuration
- create_synthetic_data_pipeline: DAG factory function
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import time
from pathlib import Path
from typing import Any, Dict, List, Optional

try:
    from airflow.models import BaseOperator
    from airflow.sensors.base import BaseSensorOperator
    from airflow.utils.context import Context

    HAS_AIRFLOW = True
except ImportError:
    HAS_AIRFLOW = False

    # Stub classes for when airflow is not installed
    class BaseOperator:  # type: ignore[no-redef]
        """Stub BaseOperator when airflow is not installed."""
        def __init__(self, *args, **kwargs):
            raise ImportError(
                "Apache Airflow is required for DataSynth Airflow integration. "
                "Install with: pip install 'datasynth-py[airflow]'"
            )

    class BaseSensorOperator:  # type: ignore[no-redef]
        """Stub BaseSensorOperator when airflow is not installed."""
        def __init__(self, *args, **kwargs):
            raise ImportError(
                "Apache Airflow is required for DataSynth Airflow integration. "
                "Install with: pip install 'datasynth-py[airflow]'"
            )

    class Context:  # type: ignore[no-redef]
        pass


def _find_datasynth_binary() -> str:
    """Find the datasynth-data binary."""
    # Check PATH first
    binary = shutil.which("datasynth-data")
    if binary:
        return binary

    # Check common locations
    for path in [
        "./target/release/datasynth-data",
        "./target/debug/datasynth-data",
        os.path.expanduser("~/.cargo/bin/datasynth-data"),
    ]:
        if os.path.isfile(path):
            return path

    raise FileNotFoundError(
        "datasynth-data binary not found. Build with 'cargo build --release' "
        "or ensure it's on PATH."
    )


class DataSynthOperator(BaseOperator):
    """Airflow operator that generates synthetic data using DataSynth.

    Args:
        config: Configuration dict or path to YAML config file.
        output_path: Directory for generated output.
        output_format: Output format (csv, parquet, json). Default: parquet.
        quality_gate: Optional quality gate configuration.
        datasynth_binary: Path to datasynth-data binary (auto-detected if None).
        task_id: Airflow task ID.
    """

    template_fields = ("config", "output_path", "output_format")

    def __init__(
        self,
        config: str | Dict[str, Any],
        output_path: str,
        output_format: str = "parquet",
        quality_gate: Optional[Dict[str, Any]] = None,
        datasynth_binary: Optional[str] = None,
        **kwargs: Any,
    ):
        super().__init__(**kwargs)
        self.config = config
        self.output_path = output_path
        self.output_format = output_format
        self.quality_gate = quality_gate
        self.datasynth_binary = datasynth_binary

    def execute(self, context: Context) -> Dict[str, Any]:
        """Execute synthetic data generation."""
        binary = self.datasynth_binary or _find_datasynth_binary()

        # Write config to temp file if dict
        if isinstance(self.config, dict):
            import tempfile
            import yaml

            config_path = os.path.join(tempfile.mkdtemp(), "config.yaml")
            with open(config_path, "w") as f:
                yaml.dump(self.config, f)
        else:
            config_path = self.config

        cmd = [
            binary,
            "generate",
            "--config", config_path,
            "--output", self.output_path,
        ]

        start_time = time.time()
        result = subprocess.run(cmd, capture_output=True, text=True, check=False)
        duration = time.time() - start_time

        if result.returncode != 0:
            raise RuntimeError(
                f"DataSynth generation failed (exit code {result.returncode}): "
                f"{result.stderr}"
            )

        # Count output files
        output_dir = Path(self.output_path)
        file_count = sum(1 for _ in output_dir.rglob("*") if _.is_file()) if output_dir.exists() else 0

        metrics = {
            "duration_seconds": duration,
            "output_path": self.output_path,
            "file_count": file_count,
            "exit_code": result.returncode,
        }

        # Push metrics to XCom
        if hasattr(context, "get") or isinstance(context, dict):
            ti = context.get("ti") if isinstance(context, dict) else None
            if ti is not None:
                ti.xcom_push(key="generation_metrics", value=metrics)

        return metrics


class DataSynthSensor(BaseSensorOperator):
    """Sensor that waits for DataSynth generation output to appear.

    Polls the output directory until expected files are found.
    """

    template_fields = ("output_path",)

    def __init__(
        self,
        output_path: str,
        expected_files: Optional[List[str]] = None,
        **kwargs: Any,
    ):
        super().__init__(**kwargs)
        self.output_path = output_path
        self.expected_files = expected_files or ["journal_entries.csv"]

    def poke(self, context: Context) -> bool:
        """Check if output files exist."""
        output_dir = Path(self.output_path)
        if not output_dir.exists():
            return False

        for expected in self.expected_files:
            if not list(output_dir.rglob(expected)):
                return False

        return True


class DataSynthValidateOperator(BaseOperator):
    """Operator that validates a DataSynth configuration file.

    Fails the task if validation fails.
    """

    template_fields = ("config_path",)

    def __init__(self, config_path: str, datasynth_binary: Optional[str] = None, **kwargs: Any):
        super().__init__(**kwargs)
        self.config_path = config_path
        self.datasynth_binary = datasynth_binary

    def execute(self, context: Context) -> Dict[str, Any]:
        """Validate configuration."""
        binary = self.datasynth_binary or _find_datasynth_binary()

        result = subprocess.run(
            [binary, "validate", "--config", self.config_path],
            capture_output=True,
            text=True,
            check=False,
        )

        if result.returncode != 0:
            raise RuntimeError(
                f"Config validation failed: {result.stderr}"
            )

        return {"valid": True, "config_path": self.config_path}


def create_synthetic_data_pipeline(
    config: str | Dict[str, Any],
    schedule: str,
    output_path: str,
    dag_id: str = "datasynth_pipeline",
    **dag_kwargs: Any,
):
    """Create an Airflow DAG for synthetic data generation.

    Args:
        config: Config dict or path to YAML file.
        schedule: Cron schedule expression.
        output_path: Output directory.
        dag_id: DAG identifier.
        **dag_kwargs: Additional DAG arguments.

    Returns:
        Configured Airflow DAG.
    """
    if not HAS_AIRFLOW:
        raise ImportError("Apache Airflow is required. Install with: pip install 'datasynth-py[airflow]'")

    from airflow import DAG
    from datetime import datetime

    default_args = dag_kwargs.pop("default_args", {
        "owner": "datasynth",
        "retries": 1,
    })

    dag = DAG(
        dag_id=dag_id,
        default_args=default_args,
        schedule_interval=schedule,
        start_date=dag_kwargs.pop("start_date", datetime(2024, 1, 1)),
        catchup=False,
        **dag_kwargs,
    )

    with dag:
        validate = DataSynthValidateOperator(
            task_id="validate_config",
            config_path=config if isinstance(config, str) else "/tmp/datasynth_config.yaml",
        )

        generate = DataSynthOperator(
            task_id="generate_data",
            config=config,
            output_path=output_path,
        )

        check = DataSynthSensor(
            task_id="check_output",
            output_path=output_path,
            poke_interval=30,
            timeout=3600,
        )

        validate >> generate >> check

    return dag
