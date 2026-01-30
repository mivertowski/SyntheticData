# datasynth-py

Python wrapper for the DataSynth synthetic data generator.

## Installation

### From PyPI

```bash
pip install datasynth-py[all]
```

Or install specific extras:

```bash
pip install datasynth-py           # Core only (no dependencies)
pip install datasynth-py[cli]      # CLI generation (PyYAML)
pip install datasynth-py[memory]   # In-memory tables (pandas)
pip install datasynth-py[streaming] # Streaming (websockets)
pip install datasynth-py[all]      # All optional dependencies
```

### From Source

```bash
cd python
pip install -e ".[all]"
```

## Quick Start

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
print(result.output_dir)
```

## Using Blueprints

```python
from datasynth_py import DataSynth
from datasynth_py.config import blueprints

config = blueprints.retail_small(companies=4, transactions=10000)
synth = DataSynth()
result = synth.generate(config=config, output={"format": "parquet", "sink": "path", "path": "./output"})
```

## Statistical Distributions (v0.3.0+)

```python
from datasynth_py.config.models import (
    Config,
    AdvancedDistributionSettings,
    MixtureDistributionConfig,
    MixtureComponentConfig,
    CorrelationConfig,
    CorrelationFieldConfig,
    RegimeChangeConfig,
    EconomicCycleConfig,
    StatisticalValidationConfig,
    StatisticalTestConfig,
)

config = Config(
    # ... other settings ...

    # Advanced statistical distributions
    distributions=AdvancedDistributionSettings(
        enabled=True,
        industry_profile="retail",

        # Mixture model for transaction amounts
        amounts=MixtureDistributionConfig(
            enabled=True,
            distribution_type="lognormal",
            components=[
                MixtureComponentConfig(weight=0.60, mu=6.0, sigma=1.5, label="routine"),
                MixtureComponentConfig(weight=0.30, mu=8.5, sigma=1.0, label="significant"),
                MixtureComponentConfig(weight=0.10, mu=11.0, sigma=0.8, label="major"),
            ],
            benford_compliance=True,
        ),

        # Cross-field correlations via copulas
        correlations=CorrelationConfig(
            enabled=True,
            copula_type="gaussian",  # gaussian, clayton, gumbel, frank, student_t
            fields=[
                CorrelationFieldConfig(name="amount", distribution_type="lognormal"),
                CorrelationFieldConfig(name="line_items", distribution_type="normal", min_value=1, max_value=20),
            ],
            matrix=[[1.0, 0.65], [0.65, 1.0]],
        ),

        # Economic regime changes
        regime_changes=RegimeChangeConfig(
            enabled=True,
            economic_cycle=EconomicCycleConfig(
                enabled=True,
                cycle_period_months=48,
                amplitude=0.15,
                recession_probability=0.1,
            ),
        ),

        # Statistical validation tests
        validation=StatisticalValidationConfig(
            enabled=True,
            tests=[
                StatisticalTestConfig(test_type="benford_first_digit", threshold_mad=0.015),
                StatisticalTestConfig(test_type="distribution_fit", target_distribution="lognormal", significance=0.05),
            ],
            fail_on_violation=False,
        ),
    ),
)
```

### Distribution Blueprints

```python
from datasynth_py.config import blueprints

# ML training with realistic distributions
config = blueprints.ml_training(with_distributions=True)

# Statistical validation preset
config = blueprints.statistical_validation()

# Add distributions to any config
config = blueprints.with_distributions(base_config)

# Retail with realistic names
config = blueprints.retail_small(realistic_names=True)
```

## Integration Features (v0.2.2+)

```python
from datasynth_py import (
    Config,
    StreamingSettings,
    RateLimitSettings,
    TemporalAttributeSettings,
    RelationshipSettings,
    GraphExportSettings,
)

config = Config(
    # ... other settings ...

    # Streaming output with backpressure
    streaming=StreamingSettings(
        enabled=True,
        buffer_size=1000,
        backpressure="block",  # block, drop_oldest, drop_newest, buffer
    ),

    # Rate limiting for controlled throughput
    rate_limit=RateLimitSettings(
        enabled=True,
        entities_per_second=10000.0,
        burst_size=100,
    ),

    # Bi-temporal data support
    temporal_attributes=TemporalAttributeSettings(
        enabled=True,
        generate_version_chains=True,
        avg_versions_per_entity=1.5,
    ),

    # Relationship generation with cardinality rules
    relationships=RelationshipSettings(
        enabled=True,
        allow_orphans=True,
        orphan_probability=0.01,
    ),

    # Graph export including RustGraph format
    graph_export=GraphExportSettings(
        enabled=True,
        formats=["pytorch_geometric", "rustgraph"],
    ),
)
```

## Requirements

The wrapper shells out to the `datasynth-data` CLI binary. Build it with:

```bash
cargo build --release
export DATASYNTH_BINARY=target/release/datasynth-data
```

Or pass `binary_path` when creating the client:

```python
synth = DataSynth(binary_path="/path/to/datasynth-data")
```

## Documentation

See the [Python Wrapper Guide](../docs/src/user-guide/python-wrapper.md) for complete documentation.

## License

Apache 2.0 License - see the main project LICENSE file.
