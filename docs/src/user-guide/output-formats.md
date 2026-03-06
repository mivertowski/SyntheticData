# Output Formats

DataSynth generates multiple file types organized into categories.

## Output Directory Structure

```
output/
├── master_data/          # Entity master records
├── transactions/         # Journal entries and documents
├── subledgers/           # Subsidiary ledger records
├── period_close/         # Trial balances and closing
├── consolidation/        # Elimination entries
├── fx/                   # Exchange rates
├── banking/              # KYC profiles and bank transactions
├── process_mining/       # OCEL 2.0 event logs
├── audit/                # Audit engagements and workpapers
├── graphs/               # ML-ready graph exports
├── labels/               # Anomaly, fraud, and quality labels
└── controls/             # Internal control mappings
```

## File Formats

### CSV

Default format with standard conventions:
- UTF-8 encoding
- Comma-separated values
- Header row included
- Quoted strings when needed
- Decimal values serialized as strings (prevents floating-point artifacts)

**Example (journal_entries.csv):**
```csv
document_id,posting_date,company_code,account,description,debit,credit,is_fraud
abc-123,2024-01-15,1000,1100,Customer payment,"1000.00","0.00",false
abc-123,2024-01-15,1000,1200,Cash receipt,"0.00","1000.00",false
```

### JSON

Structured format with nested objects:

**Example (journal_entries.json):**
```json
[
  {
    "header": {
      "document_id": "abc-123",
      "posting_date": "2024-01-15",
      "company_code": "1000",
      "source": "Manual",
      "is_fraud": false
    },
    "lines": [
      {
        "account": "1100",
        "description": "Customer payment",
        "debit": "1000.00",
        "credit": "0.00"
      },
      {
        "account": "1200",
        "description": "Cash receipt",
        "debit": "0.00",
        "credit": "1000.00"
      }
    ]
  }
]
```

### ACDOCA (SAP HANA)

SAP Universal Journal format with simulation fields:

| Field | Description |
|-------|-------------|
| RCLNT | Client |
| RLDNR | Ledger |
| RBUKRS | Company code |
| GJAHR | Fiscal year |
| BELNR | Document number |
| DOCLN | Line item |
| RYEAR | Year |
| POPER | Posting period |
| RACCT | Account |
| DRCRK | Debit/Credit indicator |
| HSL | Amount in local currency |
| ZSIM_* | Simulation metadata fields |

## Master Data Files

### chart_of_accounts.csv

| Field | Description |
|-------|-------------|
| account_number | GL account code |
| account_name | Display name |
| account_type | Asset, Liability, Equity, Revenue, Expense |
| account_subtype | Detailed classification |
| is_control_account | Links to subledger |
| normal_balance | Debit or Credit |

### vendors.csv

| Field | Description |
|-------|-------------|
| vendor_id | Unique identifier |
| vendor_name | Company name |
| tax_id | Tax identification |
| payment_terms | Standard terms |
| currency | Transaction currency |
| is_intercompany | IC flag |

### customers.csv

| Field | Description |
|-------|-------------|
| customer_id | Unique identifier |
| customer_name | Company/person name |
| credit_limit | Maximum credit |
| credit_rating | Rating code |
| payment_behavior | Typical payment pattern |

### materials.csv

| Field | Description |
|-------|-------------|
| material_id | Unique identifier |
| description | Material name |
| material_type | Classification |
| valuation_method | FIFO, LIFO, Avg |
| standard_cost | Unit cost |

### employees.csv

| Field | Description |
|-------|-------------|
| employee_id | Unique identifier |
| name | Full name |
| department | Department code |
| manager_id | Hierarchy link |
| approval_limit | Maximum approval amount |
| transaction_codes | Authorized T-codes |

## Transaction Files

### journal_entries.csv

| Field | Description |
|-------|-------------|
| document_id | Entry identifier |
| company_code | Company |
| fiscal_year | Year |
| fiscal_period | Period |
| posting_date | Date posted |
| document_date | Original date |
| source | Transaction source |
| business_process | Process category |
| is_fraud | Fraud indicator |
| is_anomaly | Anomaly indicator |

### Line Items (embedded or separate)

| Field | Description |
|-------|-------------|
| line_number | Sequence |
| account_number | GL account |
| cost_center | Cost center |
| profit_center | Profit center |
| debit_amount | Debit |
| credit_amount | Credit |
| description | Line description |

### Document Flow Files

**purchase_orders.csv:**
- Order header with vendor, dates, status
- Line items with materials, quantities, prices

**goods_receipts.csv:**
- Receipt linked to PO
- Quantities received, variances

**vendor_invoices.csv:**
- Invoice with three-way match status
- Payment terms, due date

**payments.csv:**
- Payment documents
- Bank references, cleared invoices

**document_references.csv:**
- Links between documents (FollowOn, Payment, Reversal)
- Ensures complete document chains

## Subledger Files

### ar_open_items.csv

| Field | Description |
|-------|-------------|
| customer_id | Customer reference |
| invoice_number | Document number |
| invoice_date | Date issued |
| due_date | Payment due |
| original_amount | Invoice total |
| open_amount | Remaining balance |
| aging_bucket | 0-30, 31-60, 61-90, 90+ |

### ap_open_items.csv

Similar structure for payables.

### fa_register.csv

| Field | Description |
|-------|-------------|
| asset_id | Asset identifier |
| description | Asset name |
| acquisition_date | Purchase date |
| acquisition_cost | Original cost |
| useful_life_years | Depreciation period |
| depreciation_method | Straight-line, etc. |
| accumulated_depreciation | Total depreciation |
| net_book_value | Current value |

### inventory_positions.csv

| Field | Description |
|-------|-------------|
| material_id | Material reference |
| warehouse | Location |
| quantity | Units on hand |
| unit_cost | Current cost |
| total_value | Extended value |

## Period Close Files

### trial_balances/YYYY_MM.csv

| Field | Description |
|-------|-------------|
| account_number | GL account |
| account_name | Description |
| opening_balance | Period start |
| period_debits | Total debits |
| period_credits | Total credits |
| closing_balance | Period end |

### accruals.csv

Accrual entries with reversal dates.

### depreciation.csv

Monthly depreciation entries per asset.

## Banking Files

### banking_customers.csv

| Field | Description |
|-------|-------------|
| customer_id | Unique identifier |
| customer_type | retail, business, trust |
| name | Customer name |
| created_at | Account creation date |
| risk_score | Calculated risk score (0-100) |
| kyc_status | verified, pending, enhanced_due_diligence |
| pep_flag | Politically exposed person |
| sanctions_flag | Sanctions list match |

### bank_accounts.csv

| Field | Description |
|-------|-------------|
| account_id | Unique identifier |
| customer_id | Owner reference |
| account_type | checking, savings, money_market |
| currency | Account currency |
| opened_date | Opening date |
| balance | Current balance |
| status | active, dormant, closed |

### bank_transactions.csv

| Field | Description |
|-------|-------------|
| transaction_id | Unique identifier |
| account_id | Account reference |
| timestamp | Transaction time |
| amount | Transaction amount |
| currency | Transaction currency |
| direction | credit, debit |
| channel | branch, atm, online, wire, ach |
| category | Transaction category |
| counterparty_id | Counterparty reference |

### kyc_profiles.csv

| Field | Description |
|-------|-------------|
| customer_id | Customer reference |
| declared_turnover | Expected monthly volume |
| transaction_frequency | Expected transactions/month |
| source_of_funds | Declared income source |
| geographic_exposure | List of countries |
| cash_intensity | Expected cash ratio |
| beneficial_owner_complexity | Ownership layers |

### aml_typology_labels.csv

| Field | Description |
|-------|-------------|
| transaction_id | Transaction reference |
| typology | structuring, funnel, layering, mule, fraud |
| confidence | Confidence score (0-1) |
| pattern_id | Related pattern identifier |
| related_transactions | Comma-separated related IDs |

### entity_risk_labels.csv

| Field | Description |
|-------|-------------|
| entity_id | Customer or account ID |
| entity_type | customer, account |
| risk_category | high, medium, low |
| risk_factors | Contributing factors |
| label_date | Label timestamp |

## Process Mining Files (OCEL 2.0)

### event_log.json

OCEL 2.0 format event log:

```json
{
  "ocel:global-log": {
    "ocel:version": "2.0",
    "ocel:ordering": "timestamp"
  },
  "ocel:events": {
    "e1": {
      "ocel:activity": "Create Purchase Order",
      "ocel:timestamp": "2024-01-15T10:30:00Z",
      "ocel:typedOmap": [
        {"ocel:oid": "PO-001", "ocel:qualifier": "order"}
      ]
    }
  },
  "ocel:objects": {
    "PO-001": {
      "ocel:type": "PurchaseOrder",
      "ocel:attributes": {
        "vendor": "VEND-001",
        "amount": "10000.00"
      }
    }
  }
}
```

### objects.json

Object instances with types and attributes.

### events.json

Event records with object relationships.

### process_variants.csv

| Field | Description |
|-------|-------------|
| variant_id | Unique identifier |
| activity_sequence | Ordered activity list |
| frequency | Occurrence count |
| avg_duration | Average case duration |

## Audit Files

### audit_engagements.csv

| Field | Description |
|-------|-------------|
| engagement_id | Unique identifier |
| client_name | Client entity |
| engagement_type | Financial, Compliance, Operational |
| fiscal_year | Audit period |
| materiality | Materiality threshold |
| status | Planning, Fieldwork, Completion |

### audit_workpapers.csv

| Field | Description |
|-------|-------------|
| workpaper_id | Unique identifier |
| engagement_id | Engagement reference |
| workpaper_type | Lead schedule, Substantive, etc. |
| prepared_by | Preparer ID |
| reviewed_by | Reviewer ID |
| status | Draft, Reviewed, Final |

### audit_evidence.csv

| Field | Description |
|-------|-------------|
| evidence_id | Unique identifier |
| workpaper_id | Workpaper reference |
| evidence_type | Document, Inquiry, Observation, etc. |
| source | Evidence source |
| reliability | High, Medium, Low |
| sufficiency | Sufficient, Insufficient |

### audit_risks.csv

| Field | Description |
|-------|-------------|
| risk_id | Unique identifier |
| engagement_id | Engagement reference |
| risk_description | Risk narrative |
| risk_level | High, Significant, Low |
| likelihood | Probable, Possible, Remote |
| response | Response strategy |

### audit_findings.csv

| Field | Description |
|-------|-------------|
| finding_id | Unique identifier |
| engagement_id | Engagement reference |
| finding_type | Deficiency, Significant, Material Weakness |
| description | Finding narrative |
| recommendation | Recommended action |
| management_response | Response text |

### audit_judgments.csv

| Field | Description |
|-------|-------------|
| judgment_id | Unique identifier |
| workpaper_id | Workpaper reference |
| judgment_area | Revenue recognition, Estimates, etc. |
| alternatives_considered | Options evaluated |
| conclusion | Selected approach |
| rationale | Reasoning documentation |

## Graph Export Files

### PyTorch Geometric

```
graphs/transaction_network/pytorch_geometric/
├── node_features.pt    # [num_nodes, features]
├── edge_index.pt       # [2, num_edges]
├── edge_attr.pt        # [num_edges, edge_features]
├── labels.pt           # Node/edge labels
├── train_mask.pt       # Training split
├── val_mask.pt         # Validation split
└── test_mask.pt        # Test split
```

### Neo4j

```
graphs/entity_relationship/neo4j/
├── nodes_account.csv
├── nodes_entity.csv
├── nodes_user.csv
├── edges_transaction.csv
├── edges_approval.csv
└── import.cypher        # Import script
```

### DGL (Deep Graph Library)

```
graphs/transaction_network/dgl/
├── graph.bin           # DGL binary format
├── node_features.npy   # NumPy arrays
└── edge_features.npy
```

## Label Files

### anomaly_labels.csv

| Field | Description |
|-------|-------------|
| document_id | Entry reference |
| anomaly_id | Unique anomaly ID |
| anomaly_type | Classification |
| anomaly_category | Fraud, Error, Process, Statistical, Relational |
| severity | Low, Medium, High |
| description | Human-readable explanation |

### fraud_labels.csv

| Field | Description |
|-------|-------------|
| document_id | Entry reference |
| fraud_type | Specific fraud pattern (20+ types) |
| detection_difficulty | Easy, Medium, Hard |
| description | Fraud scenario description |

### quality_labels.csv

| Field | Description |
|-------|-------------|
| record_id | Record reference |
| field_name | Affected field |
| issue_type | MissingValue, Typo, FormatVariation, Duplicate |
| issue_subtype | Detailed classification |
| original_value | Value before modification |
| modified_value | Value after modification |
| severity | Severity level (1-5) |

## Control Files

### internal_controls.csv

| Field | Description |
|-------|-------------|
| control_id | Unique identifier |
| control_name | Description |
| control_type | Preventive, Detective |
| frequency | Continuous, Daily, etc. |
| assertions | Completeness, Accuracy, etc. |

### control_account_mappings.csv

| Field | Description |
|-------|-------------|
| control_id | Control reference |
| account_number | GL account |
| threshold | Monetary threshold |

### sod_rules.csv

Segregation of duties conflict definitions.

### sod_conflict_pairs.csv

Actual SoD violations detected in generated data.

## Parquet Format

Apache Parquet columnar format for large analytical datasets:

```yaml
output:
  format: parquet
  compression: snappy      # snappy, gzip, zstd
```

**Benefits:**
- Columnar storage — efficient for queries touching few columns
- Built-in compression — typically 5-10x smaller than CSV
- Schema embedding — self-describing files with full type information
- Predicate pushdown — query engines skip irrelevant row groups

**Use with:** Apache Spark, DuckDB, Polars, pandas, BigQuery, Snowflake, Databricks.

## ERP-Specific Formats

DataSynth can export in native ERP table schemas:

| Format | Target ERP | Tables |
|--------|-----------|--------|
| `sap` | SAP S/4HANA | BKPF, BSEG, ACDOCA, LFA1, KNA1, MARA, CSKS, CEPC |
| `oracle` | Oracle EBS | GL_JE_HEADERS, GL_JE_LINES, GL_JE_BATCHES |
| `netsuite` | NetSuite | Journal entries with subsidiary, multi-book, custom fields |

See [ERP Output Formats](erp-output-formats.md) for field mappings and configuration.

## Compression Options

| Option | Extension | Use Case |
|--------|-----------|----------|
| none | .csv/.json | Development, small datasets |
| gzip | .csv.gz | General compression |
| zstd | .csv.zst | High performance |
| snappy | .parquet | Parquet default (fast) |

## Configuration

```yaml
output:
  format: csv              # csv, json, jsonl, parquet, sap, oracle, netsuite
  compression: none        # none, gzip, zstd (CSV/JSON) or snappy/gzip/zstd (Parquet)
  compression_level: 6     # 1-9 (if compression enabled)
  streaming: false         # Enable streaming mode for large outputs
```

## See Also

- [ERP Output Formats](erp-output-formats.md) — SAP, Oracle, NetSuite table exports
- [Streaming Output](streaming-output.md) — Real-time streaming sinks
- [Configuration](../configuration/output-settings.md) — Output settings reference
- [Graph Export](../advanced/graph-export.md)
- [Anomaly Injection](../advanced/anomaly-injection.md)
- [AML/KYC Testing](../use-cases/aml-kyc-testing.md)
- [Process Mining](../use-cases/process-mining.md)
