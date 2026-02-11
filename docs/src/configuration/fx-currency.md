# FX & Currency

SyntheticData generates realistic foreign exchange rates, currency translation entries, and cumulative translation adjustments (CTA) for multi-currency enterprise simulation.

## Overview

The FX module in `datasynth-generators` provides three generators:

| Generator | Purpose | Output |
|-----------|---------|--------|
| **FX Rate Service** | Daily exchange rates via Ornstein-Uhlenbeck process | `fx/daily_rates.csv`, `fx/period_rates.csv` |
| **Currency Translator** | Translate foreign-currency financials to reporting currency | `consolidation/currency_translation.csv` |
| **CTA Generator** | Cumulative Translation Adjustment for consolidation | `consolidation/cta_entries.csv` |

## Configuration

```yaml
fx:
  enabled: true
  base_currency: USD                    # Reporting/functional currency
  currencies:
    - code: EUR
      initial_rate: 1.10
      volatility: 0.08
      mean_reversion: 0.05
    - code: GBP
      initial_rate: 1.27
      volatility: 0.07
      mean_reversion: 0.04
    - code: JPY
      initial_rate: 0.0067
      volatility: 0.10
      mean_reversion: 0.06
    - code: CHF
      initial_rate: 1.12
      volatility: 0.06
      mean_reversion: 0.03

  translation:
    method: current_rate                # current_rate, temporal, monetary_non_monetary
    equity_at_historical: true
    income_at_average: true

  cta:
    enabled: true
    equity_account: "3900"              # CTA equity account
```

## FX Rate Service

### Ornstein-Uhlenbeck Process

Exchange rates are generated using a mean-reverting stochastic process (Ornstein-Uhlenbeck), which models the tendency of exchange rates to revert toward a long-term equilibrium:

```
dX(t) = θ(μ - X(t))dt + σdW(t)

where:
  X(t)  = log exchange rate at time t
  θ     = mean reversion speed (mean_reversion config)
  μ     = long-term mean (derived from initial_rate)
  σ     = volatility
  dW(t) = Wiener process (random walk)
```

This produces rates that:
- **Mean-revert**: Rates drift back toward the initial level over time
- **Have realistic volatility**: Day-to-day movements match configurable volatility targets
- **Are serially correlated**: Today's rate depends on yesterday's rate (not i.i.d.)
- **Are deterministic**: Given the same seed, rates are exactly reproducible

### Rate Types

| Rate Type | Usage | Calculation |
|-----------|-------|-------------|
| **Daily spot** | Transaction-date rates | O-U process output for each business day |
| **Period average** | Income statement translation | Arithmetic mean of daily rates within the period |
| **Period closing** | Balance sheet translation | Last business day rate in the period |
| **Historical** | Equity items | Rate at the date equity was contributed |

### Output: daily_rates.csv

| Field | Description |
|-------|-------------|
| `date` | Business day |
| `from_currency` | Source currency (e.g., EUR) |
| `to_currency` | Target currency (e.g., USD) |
| `spot_rate` | Daily spot rate |
| `inverse_rate` | 1 / spot_rate |

### Output: period_rates.csv

| Field | Description |
|-------|-------------|
| `period` | Fiscal period (YYYY-MM) |
| `from_currency` | Source currency |
| `to_currency` | Target currency |
| `average_rate` | Period average |
| `closing_rate` | Period-end closing rate |

---

## Currency Translation

### Translation Methods

SyntheticData supports three standard currency translation methods:

#### Current Rate Method (ASC 830 / IAS 21 — default)

The most common method for foreign subsidiaries with functional currency different from reporting currency:

| Item | Rate Used |
|------|-----------|
| Assets | Closing rate |
| Liabilities | Closing rate |
| Equity (contributed capital) | Historical rate |
| Equity (retained earnings) | Rolled-forward |
| Revenue | Average rate |
| Expenses | Average rate |
| Dividends | Rate on declaration date |
| **CTA** | Balancing item → Equity |

#### Temporal Method (ASC 830)

Used when the foreign operation's functional currency is the parent's currency (e.g., highly inflationary economies):

| Item | Rate Used |
|------|-----------|
| Monetary assets/liabilities | Closing rate |
| Non-monetary assets (at cost) | Historical rate |
| Non-monetary assets (at fair value) | Rate at fair value date |
| Revenue | Average rate |
| Expenses | Average rate |
| Depreciation | Historical rate of related asset |
| **Remeasurement gain/loss** | Income statement |

#### Monetary/Non-Monetary Method

| Item | Rate Used |
|------|-----------|
| Monetary items | Closing rate |
| Non-monetary items | Historical rate |

### Translation Configuration

```yaml
fx:
  translation:
    method: current_rate      # current_rate | temporal | monetary_non_monetary
    equity_at_historical: true
    income_at_average: true
```

---

## CTA Generator

The Cumulative Translation Adjustment arises because assets/liabilities are translated at closing rates while equity is at historical rates. The CTA is posted to Other Comprehensive Income (OCI) in equity:

```
CTA = Translated Net Assets (at closing rate)
    - Translated Equity (at historical rates)
    - Translated Net Income (at average rate)
```

### CTA Journal Entry

| Debit | Credit | Description |
|-------|--------|-------------|
| CTA (Equity 3900) | Various BS accounts | Translation adjustment for period |

The CTA accumulates over time and is only recycled to the income statement when a foreign subsidiary is disposed of.

### Configuration

```yaml
fx:
  cta:
    enabled: true
    equity_account: "3900"    # OCI - CTA account
```

---

## Multi-Currency Company Configuration

Multi-currency scenarios require companies with different functional currencies:

```yaml
companies:
  - code: C001
    name: "US Parent Corp"
    currency: USD
    country: US

  - code: C002
    name: "European Subsidiary"
    currency: EUR
    country: DE

  - code: C003
    name: "UK Subsidiary"
    currency: GBP
    country: GB

  - code: C004
    name: "Japan Subsidiary"
    currency: JPY
    country: JP

fx:
  enabled: true
  base_currency: USD
  currencies:
    - { code: EUR, initial_rate: 1.10, volatility: 0.08, mean_reversion: 0.05 }
    - { code: GBP, initial_rate: 1.27, volatility: 0.07, mean_reversion: 0.04 }
    - { code: JPY, initial_rate: 0.0067, volatility: 0.10, mean_reversion: 0.06 }

intercompany:
  enabled: true
  # IC transactions generate FX exposure
```

## Output Files

| File | Content |
|------|---------|
| `fx/daily_rates.csv` | Daily spot rates for all currency pairs |
| `fx/period_rates.csv` | Period average and closing rates |
| `consolidation/currency_translation.csv` | Translation entries per entity/period |
| `consolidation/cta_entries.csv` | CTA adjustments (if CTA enabled) |
| `consolidation/consolidated_trial_balance.csv` | Translated and consolidated TB |

## See Also

- [Financial Settings](financial-settings.md) — Intercompany and consolidation config
- [Intercompany Processing](../advanced/intercompany.md) — IC matching and elimination
- [Subledgers](subledgers.md) — Multi-currency subledger records
- [Period Close Engine](../advanced/period-close.md) — Month-end FX revaluation
