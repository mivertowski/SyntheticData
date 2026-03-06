# Demo Mode

Demo mode provides a quick way to explore DataSynth without creating a configuration file. It uses sensible defaults to generate a complete synthetic dataset.

## Running Demo Mode

```bash
datasynth-data generate --demo --output ./demo-output
```

## What Demo Mode Generates

Demo mode creates a comprehensive dataset with:

| Category | Contents |
|----------|----------|
| **Master Data** | Vendors, customers, materials, employees |
| **Transactions** | ~10,000 journal entries |
| **Document Flows** | P2P and O2C process documents |
| **Subledgers** | AR and AP records |
| **Period Close** | Trial balances |
| **Controls** | Internal control mappings |

## Demo Configuration

Demo mode uses these defaults:

```yaml
global:
  industry: manufacturing
  start_date: 2024-01-01
  period_months: 3
  group_currency: USD

companies:
  - code: "1000"
    name: "Demo Company"
    currency: USD
    country: US

chart_of_accounts:
  complexity: medium              # ~400 accounts

transactions:
  target_count: 10000

fraud:
  enabled: true
  fraud_rate: 0.005

anomaly_injection:
  enabled: true
  total_rate: 0.01
  generate_labels: true

output:
  format: csv
```

## Output Structure

After running demo mode, explore the output:

```bash
tree demo-output/
```

```
demo-output/
├── master_data/
│   ├── chart_of_accounts.csv     # GL accounts
│   ├── vendors.csv               # Vendor master
│   ├── customers.csv             # Customer master
│   ├── materials.csv             # Material/product master
│   └── employees.csv             # Employee/user master
├── transactions/
│   ├── journal_entries.csv       # Main JE file
│   ├── acdoca.csv                # SAP HANA format
│   ├── purchase_orders.csv       # P2P documents
│   ├── goods_receipts.csv
│   ├── vendor_invoices.csv
│   ├── payments.csv
│   ├── sales_orders.csv          # O2C documents
│   ├── deliveries.csv
│   ├── customer_invoices.csv
│   └── customer_receipts.csv
├── subledgers/
│   ├── ar_open_items.csv
│   ├── ap_open_items.csv
│   └── inventory_positions.csv
├── period_close/
│   └── trial_balances/
│       ├── 2024_01.csv
│       ├── 2024_02.csv
│       └── 2024_03.csv
├── labels/
│   ├── anomaly_labels.csv        # For ML training
│   └── fraud_labels.csv
└── controls/
    ├── internal_controls.csv
    └── sod_rules.csv
```

## Exploring the Data

### Journal Entries

```bash
head -5 demo-output/transactions/journal_entries.csv
```

Key fields:
- `document_id`: Unique transaction identifier
- `posting_date`: When the entry was posted
- `company_code`: Company identifier
- `account_number`: GL account
- `debit_amount` / `credit_amount`: Entry amounts
- `is_fraud`: Fraud label (true/false)
- `is_anomaly`: Anomaly label

### Fraud Labels

```bash
# View fraud transactions
grep "true" demo-output/labels/fraud_labels.csv | head
```

### Trial Balance

```bash
# Check balance coherence
head demo-output/period_close/trial_balances/2024_01.csv
```

## Customizing Demo Output

You can combine demo mode with some options:

```bash
# Change output directory
datasynth-data generate --demo --output ./my-demo

# Use demo as starting point, then create config
datasynth-data init --industry manufacturing --complexity medium -o config.yaml
# Edit config.yaml as needed
datasynth-data generate --config config.yaml --output ./output
```

## Use Cases for Demo Mode

### Quick Exploration
Test DataSynth's capabilities before creating a custom configuration.

### Development Testing
Generate test data quickly for development purposes.

### Training & Workshops
Provide sample data for training sessions without complex setup.

### Benchmarking
Establish baseline performance metrics.

## Moving Beyond Demo Mode

When you're ready for more control:

1. **Create a configuration file:**
   ```bash
   datasynth-data init --industry <your-industry> -o config.yaml
   ```

2. **Customize settings:**
   - Adjust transaction volume
   - Configure multiple companies
   - Enable graph export
   - Fine-tune fraud/anomaly rates

3. **Generate with your config:**
   ```bash
   datasynth-data generate --config config.yaml --output ./output
   ```

## Next Steps

- Review [Quick Start](quick-start.md) for custom configurations
- Learn the [CLI Reference](../user-guide/cli-reference.md)
- Explore [Configuration Options](../configuration/README.md)
- See [Use Cases](../use-cases/README.md) for real-world examples
