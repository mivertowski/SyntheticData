# Global Settings

Global settings control overall generation behavior.

## Configuration

```yaml
global:
  seed: 42                           # Random seed for reproducibility
  industry: manufacturing            # Industry preset
  start_date: 2024-01-01             # Generation start date
  period_months: 12                  # Duration in months
  fiscal_year_months: 12             # Fiscal year length for multi-period (optional)
  group_currency: USD                # Base/reporting currency
  worker_threads: 4                  # Parallel workers (optional)
  memory_limit: 2147483648           # Memory limit in bytes (optional)
```

## Fields

### seed

Random number generator seed for reproducible output.

| Property | Value |
|----------|-------|
| Type | `u64` |
| Required | No |
| Default | Random |

```yaml
global:
  seed: 42  # Same seed = same output
```

**Use cases:**
- Reproducible test datasets
- Debugging
- Consistent benchmarks

### industry

Industry preset for domain-specific settings.

| Property | Value |
|----------|-------|
| Type | `string` |
| Required | Yes |
| Values | See below |

**Available industries:**

| Industry | Description |
|----------|-------------|
| `manufacturing` | Production, inventory, cost accounting |
| `retail` | High volume sales, seasonal patterns |
| `financial_services` | Complex IC, regulatory compliance |
| `healthcare` | Insurance billing, compliance |
| `technology` | SaaS revenue, R&D |
| `energy` | Long-term assets, commodity trading |
| `telecom` | Subscription revenue, network assets |
| `transportation` | Fleet assets, fuel costs |
| `hospitality` | Seasonal, revenue management |

### start_date

Beginning date for generated data.

| Property | Value |
|----------|-------|
| Type | `date` (YYYY-MM-DD) |
| Required | Yes |

```yaml
global:
  start_date: 2024-01-01
```

**Notes:**
- First transaction will be on or after this date
- Combined with `period_months` to determine date range

### period_months

Duration of generation period.

| Property | Value |
|----------|-------|
| Type | `u32` |
| Required | Yes |
| Range | 1-120 |

```yaml
global:
  period_months: 12    # One year
  period_months: 36    # Three years
  period_months: 1     # One month
```

**Considerations:**
- Longer periods = more data
- Period close features require at least 1 month
- Year-end close requires at least 12 months

### fiscal_year_months

Length of the fiscal year in months, used for multi-period generation sessions.

| Property | Value |
|----------|-------|
| Type | `u32` |
| Required | No |
| Default | `12` |
| Range | 1-120, must be ≤ `period_months` |

```yaml
global:
  fiscal_year_months: 12   # Standard 12-month fiscal year (default)
  fiscal_year_months: 6    # Semi-annual fiscal periods
```

When used with `--append --months N`, the generation session splits output into fiscal-year-aligned periods, carrying forward balances and entity state between periods via `.dss` checkpoint files.

### group_currency

Base currency for consolidation and reporting.

| Property | Value |
|----------|-------|
| Type | `string` (ISO 4217) |
| Required | Yes |

```yaml
global:
  group_currency: USD
  group_currency: EUR
  group_currency: CHF
```

**Used for:**
- Currency translation
- Consolidation
- Intercompany eliminations

### worker_threads

Number of parallel worker threads.

| Property | Value |
|----------|-------|
| Type | `usize` |
| Required | No |
| Default | Number of CPU cores |

```yaml
global:
  worker_threads: 4    # Use 4 threads
  worker_threads: 1    # Single-threaded
```

**Guidance:**
- Default (CPU cores) is usually optimal
- Reduce for memory-constrained systems
- Increase may not improve performance beyond CPU cores

### memory_limit

Maximum memory usage in bytes.

| Property | Value |
|----------|-------|
| Type | `u64` |
| Required | No |
| Default | None (system limit) |

```yaml
global:
  memory_limit: 1073741824    # 1 GB
  memory_limit: 2147483648    # 2 GB
  memory_limit: 4294967296    # 4 GB
```

**Behavior:**
- Soft limit: Generation slows down
- Hard limit: Generation pauses until memory freed
- Streaming output to reduce memory pressure

## Environment Variable Overrides

| Variable | Setting |
|----------|---------|
| `SYNTH_DATA_SEED` | `global.seed` |
| `SYNTH_DATA_THREADS` | `global.worker_threads` |
| `SYNTH_DATA_MEMORY_LIMIT` | `global.memory_limit` |

```bash
SYNTH_DATA_SEED=12345 datasynth-data generate --config config.yaml --output ./output
```

## Examples

### Minimal

```yaml
global:
  industry: manufacturing
  start_date: 2024-01-01
  period_months: 12
  group_currency: USD
```

### Full Control

```yaml
global:
  seed: 42
  industry: financial_services
  start_date: 2023-01-01
  period_months: 36
  fiscal_year_months: 12
  group_currency: USD
  worker_threads: 8
  memory_limit: 8589934592  # 8 GB
```

### Development/Testing

```yaml
global:
  seed: 42                # Reproducible
  industry: manufacturing
  start_date: 2024-01-01
  period_months: 1        # Short period
  group_currency: USD
  worker_threads: 1       # Single thread for debugging
```

## Validation

| Check | Rule |
|-------|------|
| `period_months` | 1 ≤ value ≤ 120 |
| `fiscal_year_months` | 1 ≤ value ≤ 120, must be ≤ `period_months` |
| `start_date` | Valid date |
| `industry` | Known industry preset |
| `group_currency` | Valid ISO 4217 code |

## See Also

- [Industry Presets](industry-presets.md)
- [Companies](companies.md)
- [Performance Tuning](../advanced/performance.md)
