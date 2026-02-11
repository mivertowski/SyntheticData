"""DataSynth ecosystem integrations.

Provides optional integrations with:
- Apache Airflow (operators and sensors)
- dbt (source generation and project scaffolding)
- MLflow (experiment tracking)
- Apache Spark (DataFrame reader)

Each integration is lazily imported to avoid requiring the dependency at import time.
"""


def __getattr__(name):
    """Lazy import integrations to avoid requiring optional dependencies."""
    if name == "DataSynthOperator":
        from .airflow import DataSynthOperator
        return DataSynthOperator
    elif name == "DataSynthSensor":
        from .airflow import DataSynthSensor
        return DataSynthSensor
    elif name == "DataSynthValidateOperator":
        from .airflow import DataSynthValidateOperator
        return DataSynthValidateOperator
    elif name == "DbtSourceGenerator":
        from .dbt import DbtSourceGenerator
        return DbtSourceGenerator
    elif name == "DataSynthMlflowTracker":
        from .mlflow_tracker import DataSynthMlflowTracker
        return DataSynthMlflowTracker
    elif name == "DataSynthSparkReader":
        from .spark import DataSynthSparkReader
        return DataSynthSparkReader
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")
