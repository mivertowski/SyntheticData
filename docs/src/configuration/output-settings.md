# Output Settings

Output settings control file formats and organization.

## Configuration

```yaml
output:
  format: csv
  compression: none
  compression_level: 6

  files:
    journal_entries: true
    acdoca: true
    master_data: true
    documents: true
    subledgers: true
    trial_balances: true
    labels: true
    controls: true
```

## Format

Output file format selection.

```yaml
output:
  format: csv        # CSV format (default)
  format: json       # JSON format
  format: jsonl      # Newline-delimited JSON
  format: parquet    # Apache Parquet columnar
  format: sap        # SAP S/4HANA table format
  format: oracle     # Oracle EBS GL tables
  format: netsuite   # NetSuite journal entries
```

### CSV Format

Standard comma-separated values:

```csv
document_id,posting_date,company_code,account,debit,credit
abc-123,2024-01-15,1000,1100,"1000.00","0.00"
abc-123,2024-01-15,1000,4000,"0.00","1000.00"
```

**Characteristics:**
- UTF-8 encoding
- Header row included
- Quoted strings when needed
- Decimals as strings

### JSON Format

Structured JSON with nested objects:

```json
[
  {
    "header": {
      "document_id": "abc-123",
      "posting_date": "2024-01-15",
      "company_code": "1000"
    },
    "lines": [
      {"account": "1100", "debit": "1000.00", "credit": "0.00"},
      {"account": "4000", "debit": "0.00", "credit": "1000.00"}
    ]
  }
]
```

### Parquet Format

Apache Parquet columnar format for analytics:

```yaml
output:
  format: parquet
  compression: snappy     # snappy (default), gzip, zstd
```

Parquet files are self-describing with embedded schema and support columnar compression. Ideal for Spark, DuckDB, Polars, pandas, and cloud data warehouses.

### ERP Formats

Export in native ERP table schemas for load testing and integration validation:

```yaml
# SAP S/4HANA
output:
  format: sap
  sap:
    tables: [bkpf, bseg, acdoca, lfa1, kna1, mara, csks, cepc]
    client: "100"

# Oracle EBS
output:
  format: oracle
  oracle:
    ledger_id: 1

# NetSuite
output:
  format: netsuite
  netsuite:
    subsidiary_id: 1
    include_custom_fields: true
```

See [ERP Output Formats](../user-guide/erp-output-formats.md) for full field mappings.

### Streaming Mode

Enable streaming output for memory-efficient generation of large datasets:

```yaml
output:
  format: csv           # Any format
  streaming: true       # Enable streaming mode
```

See [Streaming Output](../user-guide/streaming-output.md) for details.

## Compression

File compression options.

```yaml
output:
  compression: none     # No compression
  compression: gzip     # Gzip compression (.gz)
  compression: zstd     # Zstandard compression (.zst)
```

### Compression Level

When compression is enabled:

```yaml
output:
  compression: gzip
  compression_level: 6    # 1-9, higher = smaller + slower
```

| Level | Speed | Size | Use Case |
|-------|-------|------|----------|
| 1 | Fastest | Largest | Quick compression |
| 6 | Balanced | Medium | General use (default) |
| 9 | Slowest | Smallest | Maximum compression |

### Compression Comparison

| Compression | Extension | Speed | Ratio |
|-------------|-----------|-------|-------|
| `none` | `.csv` | N/A | 1.0 |
| `gzip` | `.csv.gz` | Medium | ~0.15 |
| `zstd` | `.csv.zst` | Fast | ~0.12 |

## File Selection

Control which files are generated:

```yaml
output:
  files:
    # Core transaction data
    journal_entries: true    # journal_entries.csv
    acdoca: true             # acdoca.csv (SAP format)

    # Master data
    master_data: true        # vendors.csv, customers.csv, etc.

    # Document flow
    documents: true          # purchase_orders.csv, invoices.csv, etc.

    # Subsidiary ledgers
    subledgers: true         # ar_open_items.csv, ap_open_items.csv, etc.

    # Period close
    trial_balances: true     # trial_balances/*.csv

    # ML labels
    labels: true             # anomaly_labels.csv, fraud_labels.csv

    # Controls
    controls: true           # internal_controls.csv, sod_rules.csv
```

## Output Directory Structure

With all files enabled:

```
output/
├── master_data/
│   ├── chart_of_accounts.csv
│   ├── vendors.csv
│   ├── customers.csv
│   ├── materials.csv
│   ├── fixed_assets.csv
│   └── employees.csv
├── transactions/
│   ├── journal_entries.csv
│   └── acdoca.csv
├── documents/
│   ├── purchase_orders.csv
│   ├── goods_receipts.csv
│   ├── vendor_invoices.csv
│   ├── payments.csv
│   ├── sales_orders.csv
│   ├── deliveries.csv
│   ├── customer_invoices.csv
│   └── customer_receipts.csv
├── subledgers/
│   ├── ar_open_items.csv
│   ├── ar_aging.csv
│   ├── ap_open_items.csv
│   ├── ap_aging.csv
│   ├── fa_register.csv
│   ├── fa_depreciation.csv
│   ├── inventory_positions.csv
│   └── inventory_movements.csv
├── period_close/
│   └── trial_balances/
│       ├── 2024_01.csv
│       ├── 2024_02.csv
│       └── ...
├── consolidation/
│   ├── eliminations.csv
│   └── currency_translation.csv
├── fx/
│   ├── daily_rates.csv
│   └── period_rates.csv
├── graphs/                      # If graph_export enabled
│   ├── pytorch_geometric/
│   └── neo4j/
├── labels/
│   ├── anomaly_labels.csv
│   └── fraud_labels.csv
└── controls/
    ├── internal_controls.csv
    ├── control_mappings.csv
    └── sod_rules.csv
```

## Examples

### Development (Fast)

```yaml
output:
  format: csv
  compression: none
  files:
    journal_entries: true
    master_data: true
    labels: true
```

### Production (Compact)

```yaml
output:
  format: csv
  compression: zstd
  compression_level: 6
  files:
    journal_entries: true
    acdoca: true
    master_data: true
    documents: true
    subledgers: true
    trial_balances: true
    labels: true
    controls: true
```

### ML Training Focus

```yaml
output:
  format: csv
  compression: gzip
  files:
    journal_entries: true
    labels: true                 # Important for supervised learning
    master_data: true            # For feature engineering
```

### SAP Integration

```yaml
output:
  format: csv
  compression: none
  files:
    journal_entries: false
    acdoca: true                 # SAP ACDOCA format
    master_data: true
    documents: true
```

## Validation

| Check | Rule |
|-------|------|
| `format` | `csv` or `json` |
| `compression` | `none`, `gzip`, or `zstd` |
| `compression_level` | 1-9 (only when compression enabled) |

## See Also

- [Output Formats](../user-guide/output-formats.md)
- [Graph Export](../advanced/graph-export.md)
- [datasynth-output Crate](../crates/datasynth-output.md)
