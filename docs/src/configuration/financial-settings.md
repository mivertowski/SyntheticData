# Financial Settings

Financial settings control balance, subledger, FX, and period close.

## Balance Configuration

```yaml
balance:
  opening_balance:
    enabled: true
    total_assets: 10000000

  coherence_check:
    enabled: true
    tolerance: 0.01
```

### Opening Balance

Generate coherent opening balance sheet:

```yaml
balance:
  opening_balance:
    enabled: true
    total_assets: 10000000           # Total asset value

    structure:                        # Balance sheet structure
      current_assets: 0.3
      fixed_assets: 0.5
      other_assets: 0.2

      current_liabilities: 0.2
      long_term_debt: 0.3
      equity: 0.5
```

### Balance Coherence

Verify accounting equation:

```yaml
balance:
  coherence_check:
    enabled: true                    # Verify Assets = L + E
    tolerance: 0.01                  # Allowed rounding variance
    frequency: monthly               # When to check
```

---

## Subledger Configuration

```yaml
subledger:
  ar:
    enabled: true
    aging_buckets: [30, 60, 90, 120]

  ap:
    enabled: true
    aging_buckets: [30, 60, 90]

  fixed_assets:
    enabled: true
    depreciation_methods:
      - straight_line
      - declining_balance

  inventory:
    enabled: true
    valuation_methods:
      - fifo
      - weighted_average
```

### Accounts Receivable

```yaml
subledger:
  ar:
    enabled: true
    aging_buckets: [30, 60, 90, 120]  # Aging period boundaries

    collection:
      on_time_rate: 0.7               # % paid within terms
      write_off_rate: 0.02            # % written off

    reconciliation:
      enabled: true                   # Reconcile to GL
      control_account: "1100"         # AR control account
```

### Accounts Payable

```yaml
subledger:
  ap:
    enabled: true
    aging_buckets: [30, 60, 90]

    payment:
      discount_usage_rate: 0.3        # % taking early pay discount
      late_payment_rate: 0.1          # % paid late

    reconciliation:
      enabled: true
      control_account: "2000"         # AP control account
```

### Fixed Assets

```yaml
subledger:
  fixed_assets:
    enabled: true

    depreciation_methods:
      - method: straight_line
        weight: 0.7
      - method: declining_balance
        rate: 0.2
        weight: 0.2
      - method: units_of_production
        weight: 0.1

    disposal:
      rate: 0.05                      # Annual disposal rate
      gain_loss_account: "8000"       # Gain/loss account

    reconciliation:
      enabled: true
      control_accounts:
        asset: "1500"
        depreciation: "1510"
```

### Inventory

```yaml
subledger:
  inventory:
    enabled: true

    valuation_methods:
      - method: fifo
        weight: 0.3
      - method: weighted_average
        weight: 0.5
      - method: standard_cost
        weight: 0.2

    movements:
      receipt_weight: 0.4
      issue_weight: 0.4
      adjustment_weight: 0.1
      transfer_weight: 0.1

    reconciliation:
      enabled: true
      control_account: "1200"
```

---

## FX Configuration

```yaml
fx:
  enabled: true
  base_currency: USD

  currency_pairs:
    - EUR
    - GBP
    - CHF
    - JPY

  volatility: 0.01

  translation:
    method: current_rate
```

### Exchange Rates

```yaml
fx:
  enabled: true
  base_currency: USD                  # Reporting currency

  currency_pairs:                     # Currencies to generate
    - EUR
    - GBP
    - CHF

  rate_types:
    - spot                            # Daily spot rates
    - closing                         # Period closing rates
    - average                         # Period average rates

  volatility: 0.01                    # Daily volatility
  mean_reversion: 0.1                 # Ornstein-Uhlenbeck parameter
```

### Currency Translation

```yaml
fx:
  translation:
    method: current_rate              # current_rate, temporal

    rate_mapping:
      assets: closing_rate
      liabilities: closing_rate
      equity: historical_rate
      revenue: average_rate
      expense: average_rate

    cta_account: "3500"               # CTA equity account
```

---

## Period Close Configuration

```yaml
period_close:
  enabled: true

  monthly:
    accruals: true
    depreciation: true

  quarterly:
    intercompany_elimination: true

  annual:
    closing_entries: true
    retained_earnings: true
```

### Monthly Close

```yaml
period_close:
  monthly:
    accruals:
      enabled: true
      auto_reverse: true              # Reverse in next period
      categories:
        - expense_accrual
        - revenue_accrual
        - payroll_accrual

    depreciation:
      enabled: true
      run_date: last_day              # When to run

    reconciliation:
      enabled: true
      subledger_to_gl: true
```

### Quarterly Close

```yaml
period_close:
  quarterly:
    intercompany_elimination:
      enabled: true
      types:
        - intercompany_sales
        - intercompany_profit
        - intercompany_dividends

    currency_translation:
      enabled: true
```

### Annual Close

```yaml
period_close:
  annual:
    closing_entries:
      enabled: true
      close_revenue: true
      close_expense: true

    retained_earnings:
      enabled: true
      account: "3100"

    year_end_adjustments:
      - bad_debt_provision
      - inventory_reserve
      - bonus_accrual
```

---

## Combined Example

```yaml
balance:
  opening_balance:
    enabled: true
    total_assets: 50000000
  coherence_check:
    enabled: true

subledger:
  ar:
    enabled: true
    aging_buckets: [30, 60, 90, 120, 180]
  ap:
    enabled: true
    aging_buckets: [30, 60, 90]
  fixed_assets:
    enabled: true
  inventory:
    enabled: true

fx:
  enabled: true
  base_currency: USD
  currency_pairs: [EUR, GBP, CHF, JPY, CNY]
  volatility: 0.012

period_close:
  enabled: true
  monthly:
    accruals: true
    depreciation: true
  quarterly:
    intercompany_elimination: true
  annual:
    closing_entries: true
    retained_earnings: true
```

## Financial Reporting (v0.6.0)

The `financial_reporting` section generates structured financial statements, management KPIs, and budgets derived from the underlying journal entries, trial balances, and period close data.

### Financial Statements

```yaml
financial_reporting:
  enabled: true
  generate_balance_sheet: true         # Balance sheet
  generate_income_statement: true      # Income statement / P&L
  generate_cash_flow: true             # Cash flow statement
  generate_changes_in_equity: true     # Statement of changes in equity
  comparative_periods: 1               # Number of prior-period comparatives
```

When enabled, the generator produces financial statements at each period close. The `comparative_periods` setting controls how many prior periods are included for comparative analysis. Statements are aggregated from the trial balance and subledger data, ensuring consistency with the underlying journal entries.

### Management KPIs

```yaml
financial_reporting:
  management_kpis:
    enabled: true
    frequency: "monthly"               # monthly or quarterly
```

Management KPIs include ratios and metrics computed from the generated financial data:

| KPI Category | Examples |
|-------------|----------|
| Liquidity | Current ratio, quick ratio, cash conversion cycle |
| Profitability | Gross margin, operating margin, ROE, ROA |
| Efficiency | Inventory turnover, receivables turnover, asset turnover |
| Leverage | Debt-to-equity, interest coverage |

### Budgets

```yaml
financial_reporting:
  budgets:
    enabled: true
    revenue_growth_rate: 0.05          # 5% expected growth
    expense_inflation_rate: 0.03       # 3% cost inflation
    variance_noise: 0.10               # 10% random noise on actuals vs budget
```

Budget generation creates a budget line for each GL account based on prior-period actuals, adjusted by the configured growth and inflation rates. The `variance_noise` parameter controls the spread between budget and actual figures, producing realistic budget-to-actual variance reports.

---

## See Also

- [Companies](companies.md)
- [Document Flows](document-flows.md)
- [Period Close Engine](../advanced/period-close.md)
