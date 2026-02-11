"""MLflow experiment tracking for DataSynth synthetic data generation.

Provides:
- DataSynthMlflowTracker: Track generation runs, log metrics, and compare experiments
"""

from __future__ import annotations

import hashlib
import time
from pathlib import Path
from typing import Any, Dict, List, Optional

try:
    import mlflow
    from mlflow.entities import ViewType

    HAS_MLFLOW = True
except ImportError:
    HAS_MLFLOW = False
    mlflow = None  # type: ignore[assignment]
    ViewType = None  # type: ignore[assignment, misc]


class DataSynthMlflowTracker:
    """Track DataSynth generation runs with MLflow.

    Logs parameters, metrics, and artifacts for each generation run,
    enabling experiment comparison and reproducibility tracking.

    Args:
        experiment_name: MLflow experiment name. Default: "datasynth".
        tracking_uri: MLflow tracking server URI. If None, uses default.

    Raises:
        ImportError: If mlflow is not installed.

    Example::

        tracker = DataSynthMlflowTracker(experiment_name="my_experiment")
        run_info = tracker.track_generation("./output", config=my_config)

        # Or as a context manager:
        with DataSynthMlflowTracker() as tracker:
            tracker.log_quality_metrics({"completeness": 0.98})
    """

    def __init__(
        self,
        experiment_name: str = "datasynth",
        tracking_uri: Optional[str] = None,
    ):
        if not HAS_MLFLOW:
            raise ImportError(
                "MLflow is required for DataSynth MLflow integration. "
                "Install with: pip install 'datasynth-py[mlflow]'"
            )

        if tracking_uri is not None:
            mlflow.set_tracking_uri(tracking_uri)

        self.experiment_name = experiment_name
        mlflow.set_experiment(experiment_name)
        self._active_run = None

    def __enter__(self) -> DataSynthMlflowTracker:
        """Start an MLflow run as a context manager."""
        self._active_run = mlflow.start_run()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb) -> None:
        """End the active MLflow run."""
        if self._active_run is not None:
            mlflow.end_run()
            self._active_run = None

    def track_generation(
        self,
        output_path: str,
        config: Optional[Dict[str, Any]] = None,
        tags: Optional[Dict[str, str]] = None,
    ) -> Dict[str, Any]:
        """Track a DataSynth generation run in MLflow.

        Logs configuration parameters, output metrics, and config artifact.

        Args:
            output_path: Path to DataSynth output directory.
            config: Optional configuration dict used for generation.
            tags: Optional tags to attach to the run.

        Returns:
            Dict with run_id, experiment_id, and logged metrics.
        """
        output_dir = Path(output_path)

        with mlflow.start_run() as run:
            # Log tags
            if tags:
                mlflow.set_tags(tags)
            mlflow.set_tag("datasynth.version", "1.0.0")

            # Log configuration parameters
            if config is not None:
                self._log_config_params(config)
                self._log_config_artifact(config)

            # Log output metrics
            metrics = self._collect_output_metrics(output_dir)
            if metrics:
                mlflow.log_metrics(metrics)

            run_info = {
                "run_id": run.info.run_id,
                "experiment_id": run.info.experiment_id,
                "metrics": metrics,
            }

        return run_info

    def log_quality_metrics(self, metrics: Dict[str, Any]) -> None:
        """Log arbitrary quality metrics to the current active run.

        Should be called within a context manager or active run.

        Args:
            metrics: Dict of metric name -> numeric value.

        Raises:
            RuntimeError: If no active MLflow run exists.
        """
        active_run = mlflow.active_run()
        if active_run is None:
            raise RuntimeError(
                "No active MLflow run. Use within a context manager or call "
                "mlflow.start_run() first."
            )

        # Filter to numeric values and prefix with "quality."
        numeric_metrics = {}
        for key, value in metrics.items():
            if isinstance(value, (int, float)):
                safe_key = f"quality.{key.replace(' ', '_')}"
                numeric_metrics[safe_key] = value

        if numeric_metrics:
            mlflow.log_metrics(numeric_metrics)

    def compare_runs(
        self,
        experiment_name: Optional[str] = None,
        n: int = 5,
    ) -> List[Dict[str, Any]]:
        """Retrieve and compare recent generation runs.

        Args:
            experiment_name: Experiment to query. Defaults to this tracker's experiment.
            n: Number of recent runs to retrieve.

        Returns:
            List of dicts with run_id, start_time, params, and metrics.
        """
        exp_name = experiment_name or self.experiment_name
        experiment = mlflow.get_experiment_by_name(exp_name)

        if experiment is None:
            return []

        runs = mlflow.search_runs(
            experiment_ids=[experiment.experiment_id],
            max_results=n,
            order_by=["start_time DESC"],
            run_view_type=ViewType.ACTIVE_ONLY,
            output_format="list",
        )

        results = []
        for run in runs:
            results.append({
                "run_id": run.info.run_id,
                "start_time": run.info.start_time,
                "end_time": run.info.end_time,
                "status": run.info.status,
                "params": dict(run.data.params),
                "metrics": dict(run.data.metrics),
                "tags": {
                    k: v
                    for k, v in run.data.tags.items()
                    if not k.startswith("mlflow.")
                },
            })

        return results

    def _log_config_params(self, config: Dict[str, Any]) -> None:
        """Extract and log key configuration parameters."""
        # Config hash for deduplication
        config_str = str(sorted(config.items()))
        config_hash = hashlib.sha256(config_str.encode()).hexdigest()[:12]
        mlflow.log_param("config_hash", config_hash)

        # Global settings
        global_settings = config.get("global", config.get("global_settings", {}))
        if isinstance(global_settings, dict):
            if "seed" in global_settings:
                mlflow.log_param("seed", global_settings["seed"])
            if "industry" in global_settings:
                mlflow.log_param("industry", global_settings["industry"])
            if "period_months" in global_settings:
                mlflow.log_param("period_months", global_settings["period_months"])
            if "start_date" in global_settings:
                mlflow.log_param("start_date", str(global_settings["start_date"]))

        # Company count
        companies = config.get("companies", [])
        if isinstance(companies, list):
            mlflow.log_param("company_count", len(companies))

        # Chart of accounts complexity
        coa = config.get("chart_of_accounts", {})
        if isinstance(coa, dict) and "complexity" in coa:
            mlflow.log_param("coa_complexity", coa["complexity"])

        # Output format
        output = config.get("output", {})
        if isinstance(output, dict) and "format" in output:
            mlflow.log_param("output_format", output["format"])

    def _log_config_artifact(self, config: Dict[str, Any]) -> None:
        """Log the full configuration as a YAML artifact."""
        import tempfile

        try:
            import yaml

            with tempfile.NamedTemporaryFile(
                mode="w", suffix=".yaml", prefix="datasynth_config_", delete=False
            ) as f:
                yaml.dump(config, f, default_flow_style=False, sort_keys=False)
                temp_path = f.name

            mlflow.log_artifact(temp_path, artifact_path="config")

            # Clean up temp file
            Path(temp_path).unlink(missing_ok=True)
        except ImportError:
            # yaml not available, log as JSON instead
            import json

            with tempfile.NamedTemporaryFile(
                mode="w", suffix=".json", prefix="datasynth_config_", delete=False
            ) as f:
                json.dump(config, f, indent=2)
                temp_path = f.name

            mlflow.log_artifact(temp_path, artifact_path="config")
            Path(temp_path).unlink(missing_ok=True)

    @staticmethod
    def _collect_output_metrics(output_dir: Path) -> Dict[str, float]:
        """Collect metrics from the output directory."""
        metrics: Dict[str, float] = {}

        if not output_dir.exists():
            return metrics

        # Count files by type
        all_files = [f for f in output_dir.rglob("*") if f.is_file()]
        metrics["file_count"] = float(len(all_files))

        csv_files = [f for f in all_files if f.suffix == ".csv"]
        metrics["csv_file_count"] = float(len(csv_files))

        json_files = [f for f in all_files if f.suffix == ".json"]
        metrics["json_file_count"] = float(len(json_files))

        parquet_files = [f for f in all_files if f.suffix == ".parquet"]
        metrics["parquet_file_count"] = float(len(parquet_files))

        # Total size in bytes
        total_size = sum(f.stat().st_size for f in all_files)
        metrics["total_size_bytes"] = float(total_size)
        metrics["total_size_mb"] = round(total_size / (1024 * 1024), 2)

        # Row count estimate from CSV files (count lines minus header)
        total_rows = 0
        for csv_file in csv_files:
            try:
                with open(csv_file, "r") as f:
                    line_count = sum(1 for _ in f) - 1  # subtract header
                    total_rows += max(0, line_count)
            except (OSError, UnicodeDecodeError):
                continue
        metrics["estimated_total_rows"] = float(total_rows)

        return metrics
