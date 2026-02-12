# Period Close Engine

Generate period-end accounting processes.

## Overview

The period close engine simulates:

- Monthly close (accruals, depreciation)
- Quarterly close (IC elimination, translation)
- Annual close (closing entries, retained earnings)

## Configuration

```yaml
period_close:
  enabled: true

  monthly:
    accruals: true
    depreciation: true
    reconciliation: true

  quarterly:
    intercompany_elimination: true
    currency_translation: true

  annual:
    closing_entries: true
    retained_earnings: true
```

## Monthly Close

### Accruals

Generate reversing accrual entries:

```yaml
period_close:
  monthly:
    accruals:
      enabled: true
      auto_reverse: true             # Reverse next period

      categories:
        expense_accrual: 0.4
        revenue_accrual: 0.2
        payroll_accrual: 0.3
        other: 0.1
```

**Expense Accrual:**
```
Period 1 (accrue):
    Dr Expense                     10,000
        Cr Accrued Liabilities     10,000

Period 2 (reverse):
    Dr Accrued Liabilities         10,000
        Cr Expense                 10,000
```

### Depreciation

Calculate and post monthly depreciation:

```yaml
period_close:
  monthly:
    depreciation:
      enabled: true
      run_date: last_day            # When in period

      methods:
        straight_line: 0.7
        declining_balance: 0.2
        units_of_production: 0.1
```

**Depreciation Entry:**
```
    Dr Depreciation Expense          5,000
        Cr Accumulated Depreciation  5,000
```

### Subledger Reconciliation

Verify subledger-to-GL control accounts:

```yaml
period_close:
  monthly:
    reconciliation:
      enabled: true

      checks:
        - subledger: ar
          control_account: "1100"
        - subledger: ap
          control_account: "2000"
        - subledger: inventory
          control_account: "1200"
```

**Reconciliation Report:**
| Subledger | Control Account | Subledger Balance | GL Balance | Difference |
|-----------|-----------------|-------------------|------------|------------|
| AR | 1100 | 500,000 | 500,000 | 0 |
| AP | 2000 | (300,000) | (300,000) | 0 |

## Quarterly Close

### IC Elimination

Generate consolidation eliminations:

```yaml
period_close:
  quarterly:
    intercompany_elimination:
      enabled: true

      types:
        - revenue_expense            # Eliminate IC sales
        - unrealized_profit          # Eliminate IC inventory profit
        - receivable_payable         # Eliminate IC balances
        - dividends                  # Eliminate IC dividends
```

See [Intercompany Processing](intercompany.md) for details.

### Currency Translation

Translate foreign subsidiary balances:

```yaml
period_close:
  quarterly:
    currency_translation:
      enabled: true
      method: current_rate           # current_rate, temporal

      rate_mapping:
        assets: closing_rate
        liabilities: closing_rate
        equity: historical_rate
        revenue: average_rate
        expense: average_rate

      cta_account: "3500"            # CTA equity account
```

**Translation Entry (CTA):**
```
If foreign currency strengthened:
    Dr Foreign Subsidiary Investment  10,000
        Cr CTA (Other Comprehensive)  10,000
```

## Annual Close

### Closing Entries

Close temporary accounts to retained earnings:

```yaml
period_close:
  annual:
    closing_entries:
      enabled: true
      close_revenue: true
      close_expense: true
      income_summary_account: "3900"
```

**Closing Sequence:**
```
1. Close Revenue:
    Dr Revenue accounts (all)      1,000,000
        Cr Income Summary          1,000,000

2. Close Expenses:
    Dr Income Summary                800,000
        Cr Expense accounts (all)    800,000

3. Close Income Summary:
    Dr Income Summary                200,000
        Cr Retained Earnings         200,000
```

### Retained Earnings

Update retained earnings:

```yaml
period_close:
  annual:
    retained_earnings:
      enabled: true
      account: "3100"
      dividend_account: "3150"
```

### Year-End Adjustments

Additional adjusting entries:

```yaml
period_close:
  annual:
    adjustments:
      - type: bad_debt_provision
        rate: 0.02                   # 2% of AR

      - type: inventory_reserve
        rate: 0.01                   # 1% of inventory

      - type: bonus_accrual
        rate: 0.10                   # 10% of salary expense
```

## Financial Statements (v0.6.0)

The period close engine can now generate full financial statement sets from the adjusted trial balance. This is controlled by the `financial_reporting` configuration section.

### Balance Sheet

Generates a statement of financial position with current/non-current asset and liability classifications:

```
Assets                              Liabilities & Equity
├── Current Assets                  ├── Current Liabilities
│   ├── Cash & Equivalents          │   ├── Accounts Payable
│   ├── Accounts Receivable         │   ├── Accrued Liabilities
│   └── Inventory                   │   └── Current Debt
├── Non-Current Assets              ├── Non-Current Liabilities
│   ├── Fixed Assets (net)          │   └── Long-Term Debt
│   └── Intangibles                 └── Equity
Total Assets = Total L + E              ├── Common Stock
                                        └── Retained Earnings
```

### Income Statement

Generates a multi-step income statement:

```
Revenue
- Cost of Goods Sold
= Gross Profit
- Operating Expenses
= Operating Income
+/- Other Income/Expense
= Income Before Tax
- Income Tax
= Net Income
```

### Cash Flow Statement

Generates an indirect-method cash flow statement with three categories:

```yaml
financial_reporting:
  generate_cash_flow: true
```

Categories:
- **Operating**: Net income + non-cash adjustments + working capital changes
- **Investing**: Capital expenditures, asset disposals
- **Financing**: Debt proceeds/repayments, equity transactions, dividends

### Statement of Changes in Equity

Tracks equity movements across the period:

- Opening retained earnings
- Net income for the period
- Dividends declared
- Other comprehensive income (CTA, unrealized gains)
- Closing retained earnings

### Management KPIs

When `financial_reporting.management_kpis` is enabled, computes financial ratios:

- **Liquidity**: Current ratio, quick ratio, cash ratio
- **Profitability**: Gross margin, operating margin, net margin, ROA, ROE
- **Efficiency**: Inventory turnover, AR turnover, AP turnover, days sales outstanding
- **Leverage**: Debt-to-equity, debt-to-assets, interest coverage

### Budgets

When `financial_reporting.budgets` is enabled, generates budget records with variance analysis:

```yaml
financial_reporting:
  budgets:
    enabled: true
    variance_threshold: 0.10    # Flag variances > 10%
```

Produces budget vs. actual comparisons by account and period, with favorable/unfavorable variance flags.

## Output Files

### trial_balances/YYYY_MM.csv

| Field | Description |
|-------|-------------|
| `account_number` | GL account |
| `account_name` | Account description |
| `opening_balance` | Period start |
| `period_debits` | Total debits |
| `period_credits` | Total credits |
| `closing_balance` | Period end |

### accruals.csv

| Field | Description |
|-------|-------------|
| `accrual_id` | Unique ID |
| `accrual_type` | Category |
| `period` | Accrual period |
| `amount` | Accrual amount |
| `reversal_period` | When reversed |
| `entry_id` | Related JE ID |

### depreciation.csv

| Field | Description |
|-------|-------------|
| `asset_id` | Fixed asset |
| `period` | Depreciation period |
| `method` | Depreciation method |
| `depreciation_amount` | Period expense |
| `accumulated_total` | Running total |
| `net_book_value` | Remaining value |

### closing_entries.csv

| Field | Description |
|-------|-------------|
| `entry_id` | Closing entry ID |
| `entry_type` | Revenue, expense, summary |
| `account` | Account closed |
| `amount` | Closing amount |
| `fiscal_year` | Year closed |

### financial_statements.csv (v0.6.0)

| Field | Description |
|-------|-------------|
| `statement_id` | Unique statement identifier |
| `statement_type` | balance_sheet, income_statement, cash_flow, changes_in_equity |
| `company_code` | Company code |
| `period_end` | Statement date |
| `basis` | us_gaap, ifrs, statutory |
| `line_code` | Line item code |
| `label` | Display label |
| `section` | Statement section |
| `amount` | Current period amount |
| `amount_prior` | Prior period amount |

### bank_reconciliations.csv (v0.6.0)

| Field | Description |
|-------|-------------|
| `reconciliation_id` | Unique reconciliation ID |
| `company_code` | Company code |
| `bank_account` | Bank account identifier |
| `period_start` | Reconciliation period start |
| `period_end` | Reconciliation period end |
| `opening_balance` | Opening bank balance |
| `closing_balance` | Closing bank balance |
| `status` | in_progress, completed, completed_with_exceptions |

### management_kpis.csv (v0.6.0)

| Field | Description |
|-------|-------------|
| `company_code` | Company code |
| `period` | Reporting period |
| `kpi_name` | Ratio name (e.g., current_ratio, gross_margin) |
| `kpi_value` | Computed ratio value |
| `category` | liquidity, profitability, efficiency, leverage |

## Close Schedule

```
Month 1-11:
├── Accruals
├── Depreciation
└── Reconciliation

Month 3, 6, 9:
├── IC Elimination
└── Currency Translation

Month 12:
├── All monthly tasks
├── All quarterly tasks
├── Year-end adjustments
└── Closing entries
```

## Example Configuration

### Full Close Cycle

```yaml
global:
  start_date: 2024-01-01
  period_months: 12

period_close:
  enabled: true

  monthly:
    accruals:
      enabled: true
      auto_reverse: true
    depreciation:
      enabled: true
    reconciliation:
      enabled: true

  quarterly:
    intercompany_elimination:
      enabled: true
    currency_translation:
      enabled: true

  annual:
    closing_entries:
      enabled: true
    retained_earnings:
      enabled: true
    adjustments:
      - type: bad_debt_provision
        rate: 0.02
```

## See Also

- [Financial Settings](../configuration/financial-settings.md)
- [Intercompany Processing](intercompany.md)
- [datasynth-generators](../crates/datasynth-generators.md)
