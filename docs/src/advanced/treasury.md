# Treasury & Cash Management

*New in v0.7.0*

DataSynth generates comprehensive treasury operations data covering cash positioning, forecasting, hedging, debt management, and intercompany netting.

## Overview

The treasury module simulates corporate treasury operations:

1. **Cash Positioning** — Daily cash balance aggregation by entity, account, and currency
2. **Cash Forecasting** — Probability-weighted forward-looking cash projections
3. **Cash Pooling** — Physical, notional, and zero-balance pooling structures with sweep transactions
4. **Hedging** — FX forwards, interest rate swaps, and options with ASC 815 / IFRS 9 hedge designations and effectiveness testing
5. **Debt Management** — Loans, bonds, and credit facilities with covenants and amortization schedules
6. **Netting** — Intercompany multilateral netting runs with per-entity settlement positions

## Data Models

| Model | Description |
|-------|-------------|
| `CashPosition` | Daily cash balance by entity/account/currency |
| `CashForecast` / `CashForecastItem` | Probability-weighted forward-looking projections |
| `CashPool` / `CashPoolSweep` | Pooling structures (physical/notional/zero-balance) |
| `HedgingInstrument` | Derivatives (FX forwards, IR swaps, options) |
| `HedgeRelationship` | ASC 815 / IFRS 9 hedge designations with effectiveness testing |
| `DebtInstrument` | Loans, bonds, credit facilities |
| `AmortizationPayment` | Individual payments in amortization schedule |
| `DebtCovenant` | Financial covenants (Debt/Equity, Interest Coverage, etc.) |
| `BankGuarantee` | Letters of credit and bank guarantees |
| `NettingRun` / `NettingPosition` | Intercompany multilateral netting |

## Configuration

```yaml
treasury:
  enabled: true
  cash_positioning:
    enabled: true
    frequency: daily
  cash_forecasting:
    enabled: true
    horizon_months: 6
    confidence_levels: [0.50, 0.75, 0.90]
  cash_pooling:
    enabled: true
    pool_type: physical          # physical, notional, zero_balance
  hedging:
    enabled: true
    fx_forwards: true
    ir_swaps: true
    options: true
    effectiveness_method: dollar_offset  # dollar_offset, regression, hypothetical_derivative
  debt:
    enabled: true
    term_loans: true
    bonds: true
    revolving_facilities: true
  netting:
    enabled: true
    frequency: monthly
  bank_guarantees:
    enabled: true
  anomaly_rate: 0.02
```

## Output Files

| File | Description |
|------|-------------|
| `cash_positions.csv` | Daily cash balances |
| `cash_forecasts.csv` | Forecast headers |
| `cash_forecast_items.csv` | Forecast line items (probability-weighted) |
| `cash_pool_sweeps.csv` | Physical sweep transactions |
| `hedging_instruments.csv` | Derivative contracts |
| `hedge_relationships.csv` | Hedge designations & effectiveness |
| `debt_instruments.csv` | Loans & bonds |
| `debt_covenants.csv` | Financial covenants |
| `amortization_schedules.csv` | Principal & interest payments |
| `bank_guarantees.csv` | LC & guarantees |
| `netting_runs.csv` | Intercompany netting runs |
| `netting_positions.csv` | Per-entity settlement positions |
| `treasury_anomaly_labels.csv` | Data quality labels |

## Process Mining (OCPM)

The treasury module contributes 4 object types and 4 activities:

- **Object Types**: `cash_position`, `cash_forecast`, `hedge_instrument`, `debt_instrument`
- **Activities**: Cash position calculation, forecast generation, hedge designation, debt issuance
- **Lifecycle**: Instruments follow creation → active → matured/terminated paths
