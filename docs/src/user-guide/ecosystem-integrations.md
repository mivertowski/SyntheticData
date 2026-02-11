# Ecosystem Integrations

> **New in v0.5.0**

DataSynth's Python wrapper includes optional integrations with popular data engineering and ML platforms for seamless pipeline orchestration.

## Installation

```bash
# Install all integrations
pip install datasynth-py[integrations]

# Install specific integrations
pip install datasynth-py[airflow]
pip install datasynth-py[dbt]
pip install datasynth-py[mlflow]
pip install datasynth-py[spark]
```

## Apache Airflow

The Airflow integration provides custom operators and sensors for orchestrating synthetic data generation in Airflow DAGs.

### DataSynthOperator

Generates synthetic data as an Airflow task:

```python
from datasynth_py.integrations import DataSynthOperator

generate = DataSynthOperator(
    task_id="generate_synthetic_data",
    config={
        "global": {"industry": "retail", "start_date": "2024-01-01", "period_months": 12},
        "transactions": {"target_count": 50000},
        "output": {"format": "csv"},
    },
    output_path="/data/synthetic/{{ ds }}",
)
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `task_id` | str | Airflow task identifier |
| `config` | dict | Generation configuration (inline) |
| `config_path` | str | Path to YAML config file (alternative to `config`) |
| `output_path` | str | Output directory (supports Jinja templates) |

### DataSynthSensor

Waits for synthetic data generation to complete:

```python
from datasynth_py.integrations import DataSynthSensor

wait = DataSynthSensor(
    task_id="wait_for_data",
    output_path="/data/synthetic/{{ ds }}",
    poke_interval=30,
    timeout=600,
)
```

### DataSynthValidateOperator

Validates a configuration file before generation:

```python
from datasynth_py.integrations import DataSynthValidateOperator

validate = DataSynthValidateOperator(
    task_id="validate_config",
    config_path="/configs/retail.yaml",
)
```

### Complete DAG Example

```python
from airflow import DAG
from airflow.utils.dates import days_ago
from datasynth_py.integrations import (
    DataSynthOperator,
    DataSynthSensor,
    DataSynthValidateOperator,
)

with DAG(
    "weekly_synthetic_data",
    start_date=days_ago(1),
    schedule_interval="@weekly",
    catchup=False,
) as dag:

    validate = DataSynthValidateOperator(
        task_id="validate",
        config_path="/configs/retail.yaml",
    )

    generate = DataSynthOperator(
        task_id="generate",
        config_path="/configs/retail.yaml",
        output_path="/data/synthetic/{{ ds }}",
    )

    wait = DataSynthSensor(
        task_id="wait",
        output_path="/data/synthetic/{{ ds }}",
    )

    validate >> generate >> wait
```

## dbt Integration

Generate dbt-compatible project structures from synthetic data output.

### DbtSourceGenerator

```python
from datasynth_py.integrations import DbtSourceGenerator

gen = DbtSourceGenerator()
```

#### Generate sources.yml

Creates a dbt `sources.yml` file pointing to synthetic data tables:

```python
sources_path = gen.generate_sources_yaml(
    output_dir="./synthetic_output",
    dbt_project_dir="./my_dbt_project",
)
# Creates ./my_dbt_project/models/sources.yml
```

#### Generate Seeds

Copies synthetic CSV files as dbt seeds:

```python
seeds_dir = gen.generate_seeds(
    output_dir="./synthetic_output",
    dbt_project_dir="./my_dbt_project",
)
# Copies CSVs to ./my_dbt_project/seeds/
```

### create_dbt_project

Creates a complete dbt project structure from synthetic output:

```python
from datasynth_py.integrations import create_dbt_project

project = create_dbt_project(
    output_dir="./synthetic_output",
    project_name="synthetic_test",
)
```

This creates:
```
synthetic_test/
├── dbt_project.yml
├── models/
│   └── sources.yml
├── seeds/
│   ├── journal_entries.csv
│   ├── vendors.csv
│   ├── customers.csv
│   └── ...
└── tests/
```

### Usage with dbt CLI

```bash
cd synthetic_test
dbt seed      # Load synthetic CSVs
dbt run       # Run transformations
dbt test      # Run data tests
```

## MLflow Integration

Track synthetic data generation runs as MLflow experiments for comparison and reproducibility.

### DataSynthMlflowTracker

```python
from datasynth_py.integrations import DataSynthMlflowTracker

tracker = DataSynthMlflowTracker(experiment_name="synthetic_data_experiments")
```

#### Track a Generation Run

```python
run_info = tracker.track_generation(
    output_dir="./output",
    config=config,
)
# Logs: config parameters, output file counts, generation metadata
```

#### Log Quality Metrics

```python
tracker.log_quality_metrics({
    "completeness": 0.98,
    "benford_mad": 0.008,
    "correlation_preservation": 0.95,
    "statistical_fidelity": 0.92,
})
```

#### Compare Runs

```python
comparison = tracker.compare_runs(n=5)
for run in comparison:
    print(f"Run {run['run_id']}: {run['metrics']}")
```

### Experiment Comparison

Use MLflow to compare different generation configurations:

```python
import mlflow

configs = {
    "baseline": baseline_config,
    "with_diffusion": diffusion_config,
    "high_fraud": high_fraud_config,
}

for name, cfg in configs.items():
    with mlflow.start_run(run_name=name):
        result = synth.generate(config=cfg, output={"format": "csv", "sink": "temp_dir"})
        tracker.track_generation(result.output_dir, config=cfg)
        tracker.log_quality_metrics(evaluate_quality(result.output_dir))
```

View results in the MLflow UI:
```bash
mlflow ui --port 5000
# Open http://localhost:5000
```

## Apache Spark

Read synthetic data output directly as Spark DataFrames for large-scale analysis.

### DataSynthSparkReader

```python
from datasynth_py.integrations import DataSynthSparkReader

reader = DataSynthSparkReader()
```

#### Read a Single Table

```python
df = reader.read_table(spark, "./output", "journal_entries")
df.printSchema()
df.show(5)
```

#### Read All Tables

```python
tables = reader.read_all_tables(spark, "./output")
for name, df in tables.items():
    print(f"{name}: {df.count()} rows, {len(df.columns)} columns")
```

#### Create Temporary Views

```python
views = reader.create_temp_views(spark, "./output")

# Now use SQL
spark.sql("""
    SELECT
        v.vendor_id,
        v.name,
        COUNT(p.document_id) as payment_count,
        SUM(p.amount) as total_paid
    FROM vendors v
    JOIN payments p ON v.vendor_id = p.vendor_id
    GROUP BY v.vendor_id, v.name
    ORDER BY total_paid DESC
    LIMIT 10
""").show()
```

### Spark + DataSynth Pipeline

```python
from pyspark.sql import SparkSession
from datasynth_py import DataSynth
from datasynth_py.config import blueprints
from datasynth_py.integrations import DataSynthSparkReader

# Generate
synth = DataSynth()
config = blueprints.retail_small(transactions=100000)
result = synth.generate(config=config, output={"format": "csv", "sink": "temp_dir"})

# Load into Spark
spark = SparkSession.builder.appName("SyntheticAnalysis").getOrCreate()
reader = DataSynthSparkReader()
reader.create_temp_views(spark, result.output_dir)

# Analyze
spark.sql("""
    SELECT fiscal_period, COUNT(*) as entry_count, SUM(amount) as total_amount
    FROM journal_entries
    GROUP BY fiscal_period
    ORDER BY fiscal_period
""").show()
```

## Integration Dependencies

| Integration | Required Package | Version |
|-------------|-----------------|---------|
| Airflow | `apache-airflow` | >= 2.5 |
| dbt | `dbt-core` | >= 1.5 |
| MLflow | `mlflow` | >= 2.0 |
| Spark | `pyspark` | >= 3.3 |

All integrations are optional — install only what you need.

## See Also

- [Python Wrapper Guide](python-wrapper.md)
- [Pipeline Orchestration Use Case](../use-cases/pipeline-orchestration.md)
- [Server API](server-api.md)
