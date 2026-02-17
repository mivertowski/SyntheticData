# Tax Accounting

*New in v0.7.0*

DataSynth generates a complete tax accounting lifecycle, covering indirect taxes (VAT/GST/sales tax), income tax provisions, withholding taxes, and uncertain tax positions.

## Overview

The tax accounting module simulates the end-to-end tax process:

1. **Tax Code Master Data** — Jurisdictions and tax codes with effective date ranges
2. **Tax Line Decoration** — Automatic tax line attachment to source documents (vendor invoices, customer invoices, journal entries, payments, payroll runs)
3. **Tax Return Filing** — Aggregation by jurisdiction and period into VAT/GST/income/withholding/payroll returns
4. **Income Tax Provisions** — ASC 740 / IAS 12 provision computation with deferred tax tracking
5. **Uncertain Tax Positions** — FIN 48 / IFRIC 23 uncertain position modeling
6. **Withholding Tax** — Cross-border withholding with treaty benefit tracking

## Data Models

| Model | Description |
|-------|-------------|
| `TaxJurisdiction` | Tax authority definitions (Federal, State, Local, Municipal, Supranational) |
| `TaxCode` | Tax rate definitions with effective date ranges |
| `TaxLine` | Individual tax lines attached to source documents |
| `TaxReturn` | Periodic tax filings (VAT, GST, Income, Withholding, Payroll) |
| `TaxProvision` | ASC 740 / IAS 12 income tax provisions with deferred tax |
| `UncertainTaxPosition` | FIN 48 / IFRIC 23 uncertain tax positions |
| `WithholdingTaxRecord` | Cross-border withholding with treaty benefits |
| `RateReconciliationItem` | Statutory-to-effective rate reconciliation |

## Configuration

```yaml
tax:
  enabled: true
  jurisdictions:
    countries: [US, DE, GB]
    subnational_regions: [CA, TX, NY]  # US states with nexus
  vat_gst:
    enabled: true
    standard_rate: 0.19
    reduced_rate: 0.07
    exempt_categories: [financial_services, healthcare, education]
    reverse_charge: true
  sales_tax:
    enabled: true
    nexus_states: [CA, TX, NY]
  withholding:
    enabled: true
    treaty_network: [US-DE, US-GB, US-CH]
    default_rate: 0.30
  provisions:
    enabled: true
    statutory_rate: 0.21
    uncertain_position_count: 5
  payroll_tax:
    enabled: true
    employer_rate: 0.0765
    employee_rate: 0.0765
  anomaly_rate: 0.03
```

## Output Files

| File | Description |
|------|-------------|
| `tax_jurisdictions.csv` | Jurisdiction master data |
| `tax_codes.csv` | Tax code definitions |
| `tax_lines.csv` | Individual tax line items |
| `tax_returns.csv` | Tax return filings |
| `tax_provisions.csv` | Provision calculations |
| `rate_reconciliation.csv` | Rate reconciliation items |
| `uncertain_tax_positions.csv` | FIN 48 / IFRIC 23 uncertain positions |
| `withholding_records.csv` | Withholding tax records with treaty benefits |
| `tax_anomaly_labels.csv` | Data quality labels |

## Process Mining (OCPM)

The tax module contributes 2 object types and 8 activities to the OCEL 2.0 event log:

- **Object Types**: `tax_line`, `tax_return`
- **Activities**: Tax determination, tax line creation, tax return filing, assessment, payment, amendment
- **Lifecycle**: Tax returns follow draft → prepared → reviewed → filed → assessed → paid
