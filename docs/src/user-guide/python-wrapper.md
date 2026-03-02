# Python Wrapper Guide

This guide explains how to use the DataSynth Python wrapper for in-memory configuration, local CLI generation, and streaming generation through the server API.

## Installation

The wrapper lives in the repository under `python/`. Install it in development mode:

```bash
cd python
pip install -e ".[all]"
```

Or install just the core with specific extras:

```bash
pip install -e ".[cli]"      # For CLI generation (requires PyYAML)
pip install -e ".[memory]"   # For in-memory tables (requires pandas)
pip install -e ".[streaming]" # For streaming (requires websockets)
```

## Quick start (CLI generation)

```python
from datasynth_py import DataSynth, CompanyConfig, Config, GlobalSettings, ChartOfAccountsSettings

config = Config(
    global_settings=GlobalSettings(
        industry="retail",
        start_date="2024-01-01",
        period_months=12,
    ),
    companies=[
        CompanyConfig(code="C001", name="Retail Corp", currency="USD", country="US"),
    ],
    chart_of_accounts=ChartOfAccountsSettings(complexity="small"),
)

synth = DataSynth()
result = synth.generate(config=config, output={"format": "csv", "sink": "temp_dir"})

print(result.output_dir)  # Path to generated files
```

## Using blueprints

Blueprints provide preconfigured templates for common scenarios:

```python
from datasynth_py import DataSynth
from datasynth_py.config import blueprints

# List available blueprints
print(blueprints.list())
# ['retail_small', 'banking_medium', 'manufacturing_large',
#  'banking_aml', 'ml_training', 'with_graph_export']

# Create a retail configuration with 4 companies
config = blueprints.retail_small(companies=4, transactions=10000)

# Banking/AML focused configuration
config = blueprints.banking_aml(customers=1000, typologies=True)

# ML training optimized configuration
config = blueprints.ml_training(
    industry="manufacturing",
    anomaly_ratio=0.05,
)

# Add graph export to any configuration
config = blueprints.with_graph_export(
    base_config=blueprints.retail_small(),
    formats=["pytorch_geometric", "neo4j"],
)

synth = DataSynth()
result = synth.generate(config=config, output={"format": "parquet", "sink": "path", "path": "./output"})
```

## Configuration model

The configuration model matches the CLI schema:

```python
from datasynth_py import (
    ChartOfAccountsSettings,
    CompanyConfig,
    Config,
    FraudSettings,
    GlobalSettings,
)

config = Config(
    global_settings=GlobalSettings(
        industry="manufacturing",      # Industry sector
        start_date="2024-01-01",       # Simulation start date
        period_months=12,              # Number of months to simulate
        seed=42,                       # Random seed for reproducibility
        group_currency="USD",          # Base currency
    ),
    companies=[
        CompanyConfig(
            code="M001",
            name="Manufacturing Co",
            currency="USD",
            country="US",
            annual_transaction_volume="ten_k",  # Volume preset
        ),
        CompanyConfig(
            code="M002",
            name="Manufacturing EU",
            currency="EUR",
            country="DE",
            annual_transaction_volume="hundred_k",
        ),
    ],
    chart_of_accounts=ChartOfAccountsSettings(
        complexity="medium",           # small, medium, or large
    ),
    fraud=FraudSettings(
        enabled=True,
        rate=0.01,                     # 1% fraud rate
    ),
)
```

### Valid industry values

- `manufacturing`
- `retail`
- `financial_services`
- `healthcare`
- `technology`
- `professional_services`
- `energy`
- `transportation`
- `real_estate`
- `telecommunications`

### Transaction volume presets

- `ten_k` - 10,000 transactions/year
- `hundred_k` - 100,000 transactions/year
- `one_m` - 1,000,000 transactions/year
- `ten_m` - 10,000,000 transactions/year
- `hundred_m` - 100,000,000 transactions/year

### Extended configuration

Additional configuration sections for specialized scenarios:

```python
from datasynth_py.config.models import (
    Config,
    GlobalSettings,
    BankingSettings,
    ScenarioSettings,
    TemporalDriftSettings,
    DataQualitySettings,
    GraphExportSettings,
)

config = Config(
    global_settings=GlobalSettings(industry="financial_services"),

    # Banking/KYC/AML configuration
    banking=BankingSettings(
        enabled=True,
        retail_customers=1000,
        business_customers=200,
        typologies_enabled=True,  # Structuring, layering, mule patterns
    ),

    # ML training scenario
    scenario=ScenarioSettings(
        tags=["ml_training", "fraud_detection"],
        ml_training=True,
        target_anomaly_ratio=0.05,
    ),

    # Temporal drift for concept drift testing
    temporal=TemporalDriftSettings(
        enabled=True,
        amount_mean_drift=0.02,
        drift_type="gradual",  # gradual, sudden, recurring
    ),

    # Data quality issues for DQ model training
    data_quality=DataQualitySettings(
        enabled=True,
        missing_rate=0.05,
        typo_rate=0.02,
    ),

    # Graph export for GNN training
    graph_export=GraphExportSettings(
        enabled=True,
        formats=["pytorch_geometric", "neo4j"],
    ),
)
```

## Configuration layering

Override configuration values:

```python
from datasynth_py import Config, GlobalSettings

base = Config(global_settings=GlobalSettings(industry="retail", start_date="2024-01-01"))
custom = base.override(
    fraud={"enabled": True, "rate": 0.02},
)
```

## Validation

Validation raises `ConfigValidationError` with structured error details:

```python
from datasynth_py import Config, GlobalSettings
from datasynth_py.config.validation import ConfigValidationError

try:
    Config(global_settings=GlobalSettings(period_months=0)).validate()
except ConfigValidationError as exc:
    for error in exc.errors:
        print(error.path, error.message, error.value)
```

## Output options

Control where and how data is generated:

```python
from datasynth_py import DataSynth, OutputSpec

synth = DataSynth()

# Write to a specific path
result = synth.generate(
    config=config,
    output=OutputSpec(format="csv", sink="path", path="./output"),
)

# Write to a temporary directory
result = synth.generate(
    config=config,
    output=OutputSpec(format="parquet", sink="temp_dir"),
)
print(result.output_dir)  # Temp directory path

# Load into memory (requires pandas)
result = synth.generate(
    config=config,
    output=OutputSpec(format="csv", sink="memory"),
)
print(result.tables["journal_entries"].head())
```

## Fingerprint Operations

The Python wrapper provides access to fingerprint extraction, validation, and evaluation:

```python
from datasynth_py import DataSynth

synth = DataSynth()

# Extract fingerprint from real data
synth.fingerprint.extract(
    input_path="./real_data/",
    output_path="./fingerprint.dsf",
    privacy_level="standard"  # minimal, standard, high, maximum
)

# Validate fingerprint file
is_valid, errors = synth.fingerprint.validate("./fingerprint.dsf")
if not is_valid:
    print(f"Validation errors: {errors}")

# Get fingerprint info
info = synth.fingerprint.info("./fingerprint.dsf", detailed=True)
print(f"Privacy level: {info.privacy_level}")
print(f"Epsilon spent: {info.epsilon_spent}")
print(f"Tables: {info.tables}")

# Evaluate synthetic data fidelity
report = synth.fingerprint.evaluate(
    fingerprint_path="./fingerprint.dsf",
    synthetic_path="./synthetic_data/",
    threshold=0.8
)
print(f"Overall score: {report.overall_score}")
print(f"Statistical fidelity: {report.statistical_fidelity}")
print(f"Correlation fidelity: {report.correlation_fidelity}")
print(f"Passes threshold: {report.passes}")
```

### FidelityReport Fields

| Field | Description |
|-------|-------------|
| `overall_score` | Weighted average of all fidelity metrics (0-1) |
| `statistical_fidelity` | KS statistic, Wasserstein distance, Benford MAD |
| `correlation_fidelity` | Correlation matrix RMSE |
| `schema_fidelity` | Column type match, row count ratio |
| `passes` | Whether the score meets the threshold |

## Streaming generation

Streaming uses the DataSynth server for real-time event generation. Start the server first:

```bash
cargo run -p datasynth-server -- --port 3000
```

Then stream events:

```python
import asyncio

from datasynth_py import DataSynth
from datasynth_py.config import blueprints


async def main() -> None:
    synth = DataSynth(server_url="http://localhost:3000")
    config = blueprints.retail_small(companies=2)
    session = synth.stream(config=config, events_per_second=100)

    async for event in session.events():
        print(event)
        break


asyncio.run(main())
```

### Stream controls

```python
session.pause()
session.resume()
session.stop()
```

### Pattern triggers

Trigger specific patterns during streaming to simulate real-world scenarios:

```python
# Trigger temporal patterns
session.trigger_month_end()    # Month-end volume spike
session.trigger_year_end()     # Year-end closing entries
session.trigger_pattern("quarter_end_spike")

# Trigger anomaly patterns
session.trigger_fraud_cluster()  # Clustered fraud transactions
session.trigger_pattern("dormant_account_activity")

# Available patterns:
# - period_end_spike
# - quarter_end_spike
# - year_end_spike
# - fraud_cluster
# - error_burst
# - dormant_account_activity
```

### Synchronous event consumption

For simpler use cases without async/await:

```python
def process_event(event):
    print(f"Received: {event['document_id']}")

session.sync_events(callback=process_event, max_events=1000)
```

## Runtime requirements

The wrapper shells out to the `datasynth-data` CLI for batch generation. Ensure the binary is available:

```bash
cargo build --release
export DATASYNTH_BINARY=target/release/datasynth-data
```

Alternatively, pass `binary_path` when creating the client:

```python
synth = DataSynth(binary_path="/path/to/datasynth-data")
```

## Troubleshooting

- **MissingDependencyError**: Install the required optional dependency (`PyYAML`, `pandas`, or `websockets`).
- **CLI not found**: Build the `datasynth-data` binary and set `DATASYNTH_BINARY` or pass `binary_path`.
- **ConfigValidationError**: Check the error details for invalid configuration values.
- **Streaming errors**: Verify the server is running and reachable at the configured URL.

## Ecosystem Integrations (v0.5.0)

DataSynth includes optional integrations with popular data engineering and ML platforms. Install with:

```bash
pip install datasynth-py[integrations]
# Or install specific integrations
pip install datasynth-py[airflow,dbt,mlflow,spark]
```

### Apache Airflow

Use the `DataSynthOperator` to generate data as part of Airflow DAGs:

```python
from datasynth_py.integrations import DataSynthOperator, DataSynthSensor, DataSynthValidateOperator

# Generate data
generate = DataSynthOperator(
    task_id="generate_data",
    config=config,
    output_path="/data/synthetic/output",
)

# Wait for completion
sensor = DataSynthSensor(
    task_id="wait_for_data",
    output_path="/data/synthetic/output",
)

# Validate config
validate = DataSynthValidateOperator(
    task_id="validate_config",
    config_path="/data/configs/config.yaml",
)
```

### dbt Integration

Generate dbt sources and seeds from synthetic data:

```python
from datasynth_py.integrations import DbtSourceGenerator, create_dbt_project

gen = DbtSourceGenerator()

# Generate sources.yml for dbt
sources_path = gen.generate_sources_yaml("./output", "./my_dbt_project")

# Generate seed CSVs
seeds_dir = gen.generate_seeds("./output", "./my_dbt_project")

# Create complete dbt project from synthetic output
project = create_dbt_project("./output", "my_dbt_project")
```

### MLflow Tracking

Track generation runs as MLflow experiments:

```python
from datasynth_py.integrations import DataSynthMlflowTracker

tracker = DataSynthMlflowTracker(experiment_name="synthetic_data_runs")

# Track a generation run
run_info = tracker.track_generation("./output", config=cfg)

# Log quality metrics
tracker.log_quality_metrics({
    "completeness": 0.98,
    "benford_mad": 0.008,
    "correlation_preservation": 0.95,
})

# Compare recent runs
comparison = tracker.compare_runs(n=5)
```

### Apache Spark

Read synthetic data as Spark DataFrames:

```python
from datasynth_py.integrations import DataSynthSparkReader

reader = DataSynthSparkReader()

# Read a single table
df = reader.read_table(spark, "./output", "journal_entries")

# Read all tables
tables = reader.read_all_tables(spark, "./output")

# Create temporary views for SQL queries
views = reader.create_temp_views(spark, "./output")
spark.sql("SELECT * FROM journal_entries WHERE amount > 10000").show()
```

For comprehensive integration documentation, see the [Ecosystem Integrations](ecosystem-integrations.md) guide.

## Fraud Scenario Packs (v1.8.0)

Apply pre-configured fraud pattern bundles using the `with_fraud_packs()` blueprint:

```python
from datasynth_py import DataSynth
from datasynth_py.config import blueprints

# Apply fraud packs to a base config
config = blueprints.retail_small()
config = blueprints.with_fraud_packs(
    config,
    packs=["revenue_fraud", "vendor_kickback"],
    fraud_rate=0.03,
)

synth = DataSynth()
result = synth.generate(config=config, output={"format": "csv", "sink": "temp_dir"})
```

You can also pass fraud packs directly to `generate()`:

```python
result = synth.generate(
    config=config,
    fraud_scenario=["comprehensive"],
    fraud_rate=0.05,
    output={"format": "csv", "sink": "temp_dir"},
)
```

Available packs: `revenue_fraud`, `payroll_ghost`, `vendor_kickback`, `management_override`, `comprehensive`.

See [Fraud Scenario Packs](../advanced/fraud-scenario-packs.md) for full documentation.

## Counterfactual Scenarios (v1.8.0)

Generate paired baseline and counterfactual datasets using the `with_scenarios()` blueprint:

```python
from datasynth_py.config import blueprints

config = blueprints.retail_small()
config = blueprints.with_scenarios(
    config,
    template="fraud_detection",
    with_interventions=True,
)

synth = DataSynth()
result = synth.generate(config=config, output={"format": "csv", "sink": "temp_dir"})
```

Available scenario templates include `fraud_detection`, `recession_impact`, and `control_failure`. See [Counterfactual Scenarios](../advanced/counterfactual-scenarios.md) for full documentation.

## Streaming Output (v1.8.0)

Configure phase-aware streaming output using the `with_streaming()` blueprint:

```python
from datasynth_py.config import blueprints

config = blueprints.retail_small()
config = blueprints.with_streaming(
    config,
    buffer_size=5000,
    backpressure="block",
)

synth = DataSynth()
result = synth.generate(
    config=config,
    stream_file="./output/stream.jsonl",
    output={"format": "csv", "sink": "path", "path": "./output"},
)
```

The `stream_file` parameter writes JSONL output alongside the standard batch output. See [Streaming Pipeline](../advanced/streaming-pipeline.md) for full documentation.
