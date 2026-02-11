# Pipeline Orchestration

> **New in v0.5.0**

Integrate DataSynth into data engineering pipelines using Apache Airflow, dbt, MLflow, and Apache Spark.

## Overview

DataSynth's Python wrapper includes optional integrations for popular data engineering platforms, enabling synthetic data generation as part of automated workflows.

```bash
pip install datasynth-py[integrations]
```

## Apache Airflow

### Generate Data in a DAG

```python
from airflow import DAG
from airflow.utils.dates import days_ago
from datasynth_py.integrations import (
    DataSynthOperator,
    DataSynthSensor,
    DataSynthValidateOperator,
)

config = {
    "global": {"industry": "retail", "start_date": "2024-01-01", "period_months": 12},
    "transactions": {"target_count": 50000},
}

with DAG("synthetic_data_pipeline", start_date=days_ago(1), schedule_interval="@weekly") as dag:

    validate = DataSynthValidateOperator(
        task_id="validate_config",
        config_path="/configs/retail.yaml",
    )

    generate = DataSynthOperator(
        task_id="generate_data",
        config=config,
        output_path="/data/synthetic/{{ ds }}",
    )

    wait = DataSynthSensor(
        task_id="wait_for_output",
        output_path="/data/synthetic/{{ ds }}",
    )

    validate >> generate >> wait
```

## dbt Integration

### Generate dbt Sources from Synthetic Data

```python
from datasynth_py.integrations import DbtSourceGenerator, create_dbt_project

# Generate sources.yml pointing to synthetic CSV files
gen = DbtSourceGenerator()
gen.generate_sources_yaml("./synthetic_output", "./my_dbt_project")

# Generate seed CSVs for dbt
gen.generate_seeds("./synthetic_output", "./my_dbt_project")

# Or create a complete dbt project structure
project = create_dbt_project("./synthetic_output", "my_dbt_project")
```

This creates:
- `models/sources.yml` with table definitions
- `seeds/` directory with CSV files
- Standard dbt project structure

### Testing dbt Models with Synthetic Data

```bash
# 1. Generate synthetic data
datasynth-data generate --config retail.yaml --output ./synthetic

# 2. Create dbt project from output
python -c "from datasynth_py.integrations import create_dbt_project; create_dbt_project('./synthetic', 'test_project')"

# 3. Run dbt
cd test_project && dbt seed && dbt run && dbt test
```

## MLflow Tracking

### Track Generation Experiments

```python
from datasynth_py.integrations import DataSynthMlflowTracker

tracker = DataSynthMlflowTracker(experiment_name="data_generation")

# Track a generation run (logs config, metrics, artifacts)
run_info = tracker.track_generation("./output", config=config)

# Log additional quality metrics
tracker.log_quality_metrics({
    "benford_mad": 0.008,
    "correlation_preservation": 0.95,
    "completeness": 0.99,
})

# Compare recent runs
comparison = tracker.compare_runs(n=10)
for run in comparison:
    print(f"Run {run['run_id']}: quality={run['metrics'].get('statistical_fidelity', 'N/A')}")
```

### A/B Testing Generation Configs

```python
configs = [
    ("baseline", baseline_config),
    ("with_diffusion", diffusion_config),
    ("with_llm", llm_config),
]

for name, cfg in configs:
    with mlflow.start_run(run_name=name):
        result = synth.generate(config=cfg, output={"format": "csv", "sink": "temp_dir"})
        tracker.track_generation(result.output_dir, config=cfg)
```

## Apache Spark

### Read Synthetic Data as DataFrames

```python
from datasynth_py.integrations import DataSynthSparkReader

reader = DataSynthSparkReader()

# Read a single table
je_df = reader.read_table(spark, "./output", "journal_entries")
je_df.show(5)

# Read all tables at once
tables = reader.read_all_tables(spark, "./output")
for name, df in tables.items():
    print(f"{name}: {df.count()} rows")

# Create temporary SQL views
reader.create_temp_views(spark, "./output")
spark.sql("""
    SELECT posting_date, SUM(amount) as total
    FROM journal_entries
    WHERE fiscal_period = 12
    GROUP BY posting_date
    ORDER BY posting_date
""").show()
```

## End-to-End Pipeline Example

```python
"""
Complete pipeline: Generate → Track → Load → Transform → Test
"""
from datasynth_py import DataSynth
from datasynth_py.config import blueprints
from datasynth_py.integrations import (
    DataSynthMlflowTracker,
    DataSynthSparkReader,
    DbtSourceGenerator,
)

# 1. Generate
synth = DataSynth()
config = blueprints.retail_small(transactions=50000)
result = synth.generate(config=config, output={"format": "csv", "sink": "temp_dir"})

# 2. Track with MLflow
tracker = DataSynthMlflowTracker(experiment_name="pipeline_test")
tracker.track_generation(result.output_dir, config=config)

# 3. Load into Spark
reader = DataSynthSparkReader()
reader.create_temp_views(spark, result.output_dir)

# 4. Create dbt project for transformation testing
gen = DbtSourceGenerator()
gen.generate_sources_yaml(result.output_dir, "./dbt_project")
```

## See Also

- [Ecosystem Integrations Guide](../user-guide/ecosystem-integrations.md)
- [Python Wrapper](../user-guide/python-wrapper.md)
- [Server API](../user-guide/server-api.md)
