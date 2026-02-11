# CLI Reference

The `datasynth-data` command-line tool provides commands for generating synthetic financial data and extracting fingerprints from real data.

## Installation

After building the project, the binary is at `target/release/datasynth-data`.

```bash
cargo build --release
./target/release/datasynth-data --help
```

## Global Options

| Option | Description |
|--------|-------------|
| `-h, --help` | Show help information |
| `-V, --version` | Show version number |
| `-v, --verbose` | Enable verbose output |
| `-q, --quiet` | Suppress non-error output |

## Commands

### generate

Generate synthetic financial data.

```bash
datasynth-data generate [OPTIONS]
```

**Options:**

| Option | Type | Description |
|--------|------|-------------|
| `--config <PATH>` | Path | Configuration YAML file |
| `--demo` | Flag | Use demo preset instead of config |
| `--output <DIR>` | Path | Output directory (required) |
| `--format <FMT>` | String | Output format: csv, json |
| `--seed <NUM>` | u64 | Override random seed |

**Examples:**

```bash
# Generate with configuration file
datasynth-data generate --config config.yaml --output ./output

# Use demo mode
datasynth-data generate --demo --output ./demo-output

# Override seed for reproducibility
datasynth-data generate --config config.yaml --output ./output --seed 12345

# JSON output format
datasynth-data generate --config config.yaml --output ./output --format json
```

### init

Create a new configuration file from industry presets.

```bash
datasynth-data init [OPTIONS]
```

**Options:**

| Option | Type | Description |
|--------|------|-------------|
| `--industry <NAME>` | String | Industry preset |
| `--complexity <LEVEL>` | String | small, medium, large |
| `-o, --output <PATH>` | Path | Output file path |

**Available Industries:**
- `manufacturing`
- `retail`
- `financial_services`
- `healthcare`
- `technology`

**Examples:**

```bash
# Create manufacturing config
datasynth-data init --industry manufacturing --complexity medium -o config.yaml

# Create large retail config
datasynth-data init --industry retail --complexity large -o retail.yaml
```

### validate

Validate a configuration file.

```bash
datasynth-data validate --config <PATH>
```

**Options:**

| Option | Type | Description |
|--------|------|-------------|
| `--config <PATH>` | Path | Configuration file to validate |

**Example:**

```bash
datasynth-data validate --config config.yaml
```

**Validation Checks:**
- Required fields present
- Value ranges (period_months: 1-120)
- Distribution weights sum to 1.0 (±0.01 tolerance)
- Date consistency
- Company code uniqueness
- Compression level: 1-9 when enabled
- All rate/percentage fields: 0.0-1.0
- Approval thresholds: strictly ascending order

### info

Display available presets and configuration options.

```bash
datasynth-data info
```

**Output includes:**
- Available industry presets
- Complexity levels
- Supported output formats
- Feature capabilities

### fingerprint

Privacy-preserving fingerprint extraction and evaluation. This command has several subcommands.

```bash
datasynth-data fingerprint <SUBCOMMAND>
```

#### fingerprint extract

Extract a fingerprint from real data with privacy controls.

```bash
datasynth-data fingerprint extract [OPTIONS]
```

**Options:**

| Option | Type | Description |
|--------|------|-------------|
| `--input <PATH>` | Path | Input CSV data file (required) |
| `--output <PATH>` | Path | Output .dsf fingerprint file (required) |
| `--privacy-level <LEVEL>` | String | Privacy level: minimal, standard, high, maximum |
| `--epsilon <FLOAT>` | f64 | Custom differential privacy epsilon (overrides level) |
| `--k <INT>` | usize | Custom k-anonymity threshold (overrides level) |

**Privacy Levels:**

| Level | Epsilon | k | Outlier % | Use Case |
|-------|---------|---|-----------|----------|
| minimal | 5.0 | 3 | 99% | Low privacy, high utility |
| standard | 1.0 | 5 | 95% | Balanced (default) |
| high | 0.5 | 10 | 90% | Higher privacy |
| maximum | 0.1 | 20 | 85% | Maximum privacy |

**Examples:**

```bash
# Extract with standard privacy
datasynth-data fingerprint extract \
    --input ./real_data.csv \
    --output ./fingerprint.dsf \
    --privacy-level standard

# Extract with custom privacy parameters
datasynth-data fingerprint extract \
    --input ./real_data.csv \
    --output ./fingerprint.dsf \
    --epsilon 0.75 \
    --k 8
```

#### fingerprint validate

Validate a fingerprint file's integrity and structure.

```bash
datasynth-data fingerprint validate <PATH>
```

**Arguments:**

| Argument | Type | Description |
|----------|------|-------------|
| `<PATH>` | Path | Path to .dsf fingerprint file |

**Validation Checks:**
- DSF file structure (ZIP archive with required components)
- SHA-256 checksums for all components
- Required fields in manifest, schema, statistics
- Privacy audit completeness

**Example:**

```bash
datasynth-data fingerprint validate ./fingerprint.dsf
```

#### fingerprint info

Display fingerprint metadata and statistics.

```bash
datasynth-data fingerprint info <PATH> [OPTIONS]
```

**Arguments:**

| Argument | Type | Description |
|----------|------|-------------|
| `<PATH>` | Path | Path to .dsf fingerprint file |

**Options:**

| Option | Type | Description |
|--------|------|-------------|
| `--detailed` | Flag | Show detailed statistics |
| `--json` | Flag | Output as JSON |

**Examples:**

```bash
# Basic info
datasynth-data fingerprint info ./fingerprint.dsf

# Detailed statistics
datasynth-data fingerprint info ./fingerprint.dsf --detailed

# JSON output for scripting
datasynth-data fingerprint info ./fingerprint.dsf --json
```

#### fingerprint diff

Compare two fingerprints.

```bash
datasynth-data fingerprint diff <PATH1> <PATH2>
```

**Arguments:**

| Argument | Type | Description |
|----------|------|-------------|
| `<PATH1>` | Path | First .dsf fingerprint file |
| `<PATH2>` | Path | Second .dsf fingerprint file |

**Output includes:**
- Schema differences (columns added/removed/changed)
- Statistical distribution changes
- Correlation matrix differences

**Example:**

```bash
datasynth-data fingerprint diff ./fp_v1.dsf ./fp_v2.dsf
```

#### fingerprint evaluate

Evaluate synthetic data fidelity against a fingerprint.

```bash
datasynth-data fingerprint evaluate [OPTIONS]
```

**Options:**

| Option | Type | Description |
|--------|------|-------------|
| `--fingerprint <PATH>` | Path | Reference .dsf fingerprint file (required) |
| `--synthetic <PATH>` | Path | Directory containing synthetic data (required) |
| `--threshold <FLOAT>` | f64 | Minimum fidelity score (0.0-1.0, default 0.8) |
| `--report <PATH>` | Path | Output report file (HTML or JSON based on extension) |

**Fidelity Metrics:**
- **Statistical**: KS statistic, Wasserstein distance, Benford MAD
- **Correlation**: Correlation matrix RMSE
- **Schema**: Column type match, row count ratio
- **Rules**: Balance equation compliance rate

**Examples:**

```bash
# Basic evaluation
datasynth-data fingerprint evaluate \
    --fingerprint ./fingerprint.dsf \
    --synthetic ./synthetic_data/ \
    --threshold 0.8

# Generate HTML report
datasynth-data fingerprint evaluate \
    --fingerprint ./fingerprint.dsf \
    --synthetic ./synthetic_data/ \
    --threshold 0.85 \
    --report ./fidelity_report.html
```

## diffusion (v0.5.0)

Train and evaluate diffusion models for statistical data generation.

### diffusion train

Train a diffusion model from a fingerprint file.

```bash
datasynth-data diffusion train \
    --fingerprint ./fingerprint.dsf \
    --output ./model.json \
    --n-steps 1000 \
    --schedule cosine
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--fingerprint` | path | (required) | Path to .dsf fingerprint file |
| `--output` | path | (required) | Output path for trained model |
| `--n-steps` | integer | `1000` | Number of diffusion steps |
| `--schedule` | string | `linear` | Noise schedule: `linear`, `cosine`, `sigmoid` |

### diffusion evaluate

Evaluate a trained diffusion model's fit quality.

```bash
datasynth-data diffusion evaluate \
    --model ./model.json \
    --samples 5000
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--model` | path | (required) | Path to trained model |
| `--samples` | integer | `1000` | Number of evaluation samples |

## causal (v0.5.0)

Generate data with causal structure, run interventions, and produce counterfactuals.

### causal generate

Generate data following a causal graph structure.

```bash
datasynth-data causal generate \
    --template fraud_detection \
    --samples 10000 \
    --seed 42 \
    --output ./causal_output
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--template` | string | (required) | Built-in template (`fraud_detection`, `revenue_cycle`) or path to custom YAML |
| `--samples` | integer | `1000` | Number of samples to generate |
| `--seed` | integer | (random) | Random seed for reproducibility |
| `--output` | path | (required) | Output directory |

### causal intervene

Run do-calculus interventions ("what-if" scenarios).

```bash
datasynth-data causal intervene \
    --template fraud_detection \
    --variable transaction_amount \
    --value 50000 \
    --samples 5000 \
    --output ./intervention_output
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--template` | string | (required) | Causal template or YAML path |
| `--variable` | string | (required) | Variable to intervene on |
| `--value` | float | (required) | Value to set the variable to |
| `--samples` | integer | `1000` | Number of intervention samples |
| `--output` | path | (required) | Output directory |

### causal validate

Validate that generated data preserves causal structure.

```bash
datasynth-data causal validate \
    --data ./causal_output \
    --template fraud_detection
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--data` | path | (required) | Path to generated data |
| `--template` | string | (required) | Causal template to validate against |

## fingerprint federated (v0.5.0)

Aggregate fingerprints from multiple distributed sources without centralizing raw data.

```bash
datasynth-data fingerprint federated \
    --sources ./source_a.dsf ./source_b.dsf ./source_c.dsf \
    --output ./aggregated.dsf \
    --method weighted_average \
    --max-epsilon 5.0
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--sources` | paths | (required) | Two or more .dsf fingerprint files |
| `--output` | path | (required) | Output path for aggregated fingerprint |
| `--method` | string | `weighted_average` | Aggregation method: `weighted_average`, `median`, `trimmed_mean` |
| `--max-epsilon` | float | `5.0` | Maximum epsilon budget per source |

## init --from-description (v0.5.0)

Generate configuration from a natural language description using LLM.

```bash
datasynth-data init \
    --from-description "Generate 1 year of retail data for a mid-size US company with fraud patterns" \
    -o config.yaml
```

Uses the configured LLM provider (defaults to Mock) to parse the description and generate an appropriate YAML configuration.

## generate --certificate (v0.5.0)

Attach a synthetic data certificate to the generated output.

```bash
datasynth-data generate \
    --config config.yaml \
    --output ./output \
    --certificate
```

Produces a `certificate.json` in the output directory containing DP guarantees, quality metrics, and an HMAC-SHA256 signature.

## Signal Handling (Unix)

On Unix systems, you can pause and resume generation:

```bash
# Start generation in background
datasynth-data generate --config config.yaml --output ./output &

# Pause generation
kill -USR1 $(pgrep datasynth-data)

# Resume generation (send SIGUSR1 again)
kill -USR1 $(pgrep datasynth-data)
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Configuration error |
| 3 | I/O error |
| 4 | Validation error |
| 5 | Fingerprint error |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `SYNTH_DATA_LOG` | Log level (error, warn, info, debug, trace) |
| `SYNTH_DATA_THREADS` | Number of worker threads |

**Example:**

```bash
SYNTH_DATA_LOG=debug datasynth-data generate --config config.yaml --output ./output
```

## Configuration File Location

The tool looks for configuration files in this order:
1. Path specified with `--config`
2. `./datasynth-data.yaml` in current directory
3. `~/.config/datasynth-data/config.yaml`

## Output Directory Structure

Generation creates this structure:

```
output/
├── master_data/          Vendors, customers, materials, assets, employees
├── transactions/         Journal entries, purchase orders, invoices, payments
├── subledgers/           AR, AP, FA, inventory detail records
├── period_close/         Trial balances, accruals, closing entries
├── consolidation/        Eliminations, currency translation
├── fx/                   Exchange rates, CTA adjustments
├── banking/              KYC profiles, bank transactions, AML typology labels
├── process_mining/       OCEL 2.0 event logs, process variants
├── audit/                Engagements, workpapers, findings, risk assessments
├── graphs/               PyTorch Geometric, Neo4j, DGL exports (if enabled)
├── labels/               Anomaly, fraud, and data quality labels for ML
└── controls/             Internal control mappings, SoD rules
```

## Scripting Examples

### Batch Generation

```bash
#!/bin/bash
for industry in manufacturing retail healthcare; do
    datasynth-data init --industry $industry --complexity medium -o ${industry}.yaml
    datasynth-data generate --config ${industry}.yaml --output ./output/${industry}
done
```

### CI/CD Pipeline

```yaml
# GitHub Actions example
- name: Generate Test Data
  run: |
    cargo build --release
    ./target/release/datasynth-data generate --demo --output ./test-data

- name: Validate Generation
  run: |
    # Check output files exist
    test -f ./test-data/transactions/journal_entries.csv
    test -f ./test-data/master_data/vendors.csv
```

### Reproducible Generation

```bash
# Same seed produces identical output
datasynth-data generate --config config.yaml --output ./run1 --seed 42
datasynth-data generate --config config.yaml --output ./run2 --seed 42
diff -r run1 run2  # No differences
```

### Fingerprint Pipeline

```bash
#!/bin/bash
# Extract fingerprint from real data
datasynth-data fingerprint extract \
    --input ./real_data.csv \
    --output ./fingerprint.dsf \
    --privacy-level high

# Validate the fingerprint
datasynth-data fingerprint validate ./fingerprint.dsf

# Generate synthetic data matching the fingerprint
# (fingerprint informs config generation)
datasynth-data generate --config generated_config.yaml --output ./synthetic

# Evaluate fidelity
datasynth-data fingerprint evaluate \
    --fingerprint ./fingerprint.dsf \
    --synthetic ./synthetic \
    --threshold 0.85 \
    --report ./fidelity_report.html
```

## Troubleshooting

### Common Issues

**"Configuration file not found"**
```bash
# Check file path
ls -la config.yaml
# Use absolute path
datasynth-data generate --config /full/path/to/config.yaml --output ./output
```

**"Invalid configuration"**
```bash
# Validate first
datasynth-data validate --config config.yaml
```

**"Permission denied"**
```bash
# Check output directory permissions
mkdir -p ./output
chmod 755 ./output
```

**"Out of memory"**

The generator includes memory guards that prevent OOM conditions. If you still encounter issues:
- Reduce transaction count in configuration
- The system will automatically reduce batch sizes under memory pressure
- Check `memory_guard` settings in configuration

**"Fingerprint validation failed"**
```bash
# Check DSF file integrity
datasynth-data fingerprint validate ./fingerprint.dsf

# View detailed info
datasynth-data fingerprint info ./fingerprint.dsf --detailed
```

**"Low fidelity score"**

If synthetic data fidelity is below threshold:
- Review the fidelity report for specific metrics
- Adjust configuration to better match fingerprint statistics
- Consider using the evaluation framework's auto-tuning recommendations

## See Also

- [Quick Start](../getting-started/quick-start.md)
- [Configuration Reference](../configuration/README.md)
- [Output Formats](output-formats.md)
- [Fingerprinting Guide](../advanced/fingerprinting.md)
- [Python Wrapper](python-wrapper.md)
