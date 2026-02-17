# Domain Expansion Design: Tax, Treasury, Project Accounting, ESG

**Date**: 2026-02-16
**Status**: Approved
**Scope**: Four new complementary domains for DataSynth synthetic data generation

## Motivation

DataSynth covers ~85-90% of mid-market enterprise transaction needs. The four domains selected here were chosen because they are **complementary, not orthogonal** -- each creates dense linkages with existing modules rather than standing as an isolated silo.

| Domain | Existing Modules Touched | Strategic Value |
|---|---|---|
| Tax Accounting | 8+ (GL, AP, AR, IC, payroll, reporting, audit, close) | Decorates every existing transaction with a new dimension |
| Treasury & Cash Mgmt | 7+ (banking, FX, AP, AR, reporting, IC, payroll) | Completes the cash lifecycle already half-built |
| Project Accounting | 6+ (HR/time, expenses, P2P, revenue, budgets, mfg) | Cheapest to build (reuses existing generators), opens 3+ industries |
| ESG Reporting | 6+ (vendor network, mfg, HR, reporting, audit, controls) | Increasingly mandatory, strong forward-looking positioning |

## Architecture

All four domains follow the existing pattern. No new crates required.

```
datasynth-core/src/models/
  ├── tax.rs
  ├── treasury.rs
  ├── project.rs
  └── esg.rs

datasynth-generators/src/
  ├── tax/
  │   ├── mod.rs
  │   ├── tax_code_generator.rs
  │   ├── tax_line_generator.rs       # Decorator: attaches tax lines to AP/AR/JE
  │   ├── tax_return_generator.rs
  │   ├── tax_provision_generator.rs  # ASC 740 / IAS 12
  │   └── withholding_generator.rs
  ├── treasury/
  │   ├── mod.rs
  │   ├── cash_position_generator.rs  # Aggregation: sums AP/AR/payroll flows
  │   ├── cash_forecast_generator.rs
  │   ├── hedging_generator.rs        # Extends existing FX module
  │   ├── debt_generator.rs
  │   └── cash_pool_generator.rs
  ├── project/
  │   ├── mod.rs
  │   ├── project_generator.rs        # Projects + WBS hierarchies
  │   ├── project_cost_generator.rs   # Linking: tags time/expense/PO with project refs
  │   ├── revenue_generator.rs        # PoC / milestone-based
  │   ├── earned_value_generator.rs
  │   └── change_order_generator.rs
  └── esg/
      ├── mod.rs
      ├── emission_generator.rs       # Scope 1/2/3 from activity data
      ├── energy_generator.rs
      ├── workforce_generator.rs      # Derives from employee master data
      ├── supplier_esg_generator.rs   # Scores existing vendors
      └── disclosure_generator.rs     # Maps to CSRD/GRI/SASB frameworks

datasynth-config/src/schema.rs        # Four new config sections
datasynth-runtime/src/orchestrator.rs  # Phase ordering for new generators
```

### Generator Patterns

Each domain uses a distinct generator pattern that determines when it runs in the orchestration pipeline:

| Pattern | Domains | Description |
|---|---|---|
| **Decorator** | Tax (tax_line_generator) | Runs after AP/AR/O2C generators, attaches data to existing documents |
| **Aggregation** | Treasury (cash_position_generator) | Runs after all payment generators, sums flows into positions |
| **Linking** | Project (project_cost_generator) | Tags a percentage of existing time/expense/PO records with project references |
| **Derivation** | ESG (workforce_generator, emission_generator) | Computes metrics from existing master data and operational records |
| **Standalone** | Tax (tax_code), Treasury (debt), Project (project), ESG (disclosure) | Creates new entities with no dependency on existing records |

## Domain 1: Tax Accounting & Compliance

### Data Models

```rust
// --- tax.rs ---

/// Tax jurisdiction hierarchy: Country > State/Province > City/County
pub struct TaxJurisdiction {
    pub id: String,
    pub name: String,
    pub country_code: String,
    pub region_code: Option<String>,
    pub jurisdiction_type: JurisdictionType,  // Federal, State, Local, Municipal, Supranational
    pub parent_jurisdiction_id: Option<String>,
    pub vat_registered: bool,
}

/// Tax rate by type, jurisdiction, and effective date
pub struct TaxCode {
    pub id: String,
    pub code: String,
    pub description: String,
    pub tax_type: TaxType,  // Vat, Gst, SalesTax, IncomeTax, WithholdingTax, PayrollTax, ExciseTax
    pub rate: Decimal,
    pub jurisdiction_id: String,
    pub effective_date: NaiveDate,
    pub expiry_date: Option<NaiveDate>,
    pub is_reverse_charge: bool,
    pub is_exempt: bool,
}

/// Attached to AP invoices, AR invoices, JEs
pub struct TaxLine {
    pub id: String,
    pub document_type: DocumentType,
    pub document_id: String,
    pub line_number: u32,
    pub tax_code_id: String,
    pub jurisdiction_id: String,
    pub taxable_amount: Decimal,
    pub tax_amount: Decimal,
    pub is_deductible: bool,
    pub is_reverse_charge: bool,
    pub is_self_assessed: bool,
}

/// Periodic filing (VAT return, income tax, withholding remittance)
pub struct TaxReturn {
    pub id: String,
    pub entity_id: String,
    pub jurisdiction_id: String,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub return_type: TaxReturnType,  // VatReturn, IncomeTax, WithholdingRemittance, PayrollTax
    pub status: TaxReturnStatus,     // Draft, Filed, Assessed, Paid, Amended
    pub total_output_tax: Decimal,
    pub total_input_tax: Decimal,
    pub net_payable: Decimal,
    pub filing_deadline: NaiveDate,
    pub actual_filing_date: Option<NaiveDate>,
    pub is_late: bool,
}

/// ASC 740 / IAS 12 tax provision
pub struct TaxProvision {
    pub id: String,
    pub entity_id: String,
    pub period: NaiveDate,
    pub current_tax_expense: Decimal,
    pub deferred_tax_asset: Decimal,
    pub deferred_tax_liability: Decimal,
    pub statutory_rate: Decimal,
    pub effective_rate: Decimal,
    pub rate_reconciliation: Vec<RateReconciliationItem>,
}

pub struct RateReconciliationItem {
    pub description: String,  // "State taxes", "Permanent differences", "R&D credits"
    pub rate_impact: Decimal,
}

/// FIN 48 / IFRIC 23 uncertain tax positions
pub struct UncertainTaxPosition {
    pub id: String,
    pub entity_id: String,
    pub description: String,
    pub tax_benefit: Decimal,
    pub recognition_threshold: Decimal,
    pub recognized_amount: Decimal,
    pub measurement_method: MostLikelyAmount | ExpectedValue,
}

/// Cross-border withholding on vendor payments
pub struct WithholdingTaxRecord {
    pub id: String,
    pub payment_id: String,
    pub vendor_id: String,
    pub withholding_type: WithholdingType,  // Dividend, Royalty, Service
    pub treaty_rate: Option<Decimal>,
    pub statutory_rate: Decimal,
    pub applied_rate: Decimal,
    pub base_amount: Decimal,
    pub withheld_amount: Decimal,
    pub certificate_number: Option<String>,
}
```

### Integration Points

| Existing Module | Integration |
|---|---|
| AP Invoices | Each invoice gets `Vec<TaxLine>` -- input VAT/GST, use tax (US), reverse charge (EU B2B) |
| AR/Customer Invoices | Each invoice gets `Vec<TaxLine>` -- output VAT/GST, sales tax by ship-to jurisdiction |
| Payments | Cross-border payments get `WithholdingTaxRecord` with treaty rates |
| Payroll | Payroll line items get tax withholding breakdown (federal/state/local/FICA) |
| Intercompany | Transfer pricing docs get arm's-length validation; IC invoices get withholding |
| Journal Entries | Tax provision entries, deferred tax adjustments |
| Financial Statements | Tax expense line, deferred tax assets/liabilities on balance sheet |
| Audit | Tax-specific risk assessments, uncertain tax position review |
| Period Close | Tax provision calculation as a close step |

### Generator Logic

- **tax_code_generator**: Creates jurisdiction-aware tax codes from country config. US gets sales tax by nexus state; EU gets VAT with reverse charge; APAC gets GST.
- **tax_line_generator** (decorator): Runs after AP/AR/O2C generators. For each invoice, determines applicable tax code from vendor/customer country + product category + jurisdiction rules. Attaches tax lines. Handles exempt categories (financial services, healthcare, education).
- **tax_return_generator**: Aggregates tax lines by jurisdiction and period. Generates VAT/GST returns monthly/quarterly. Flags late filings.
- **tax_provision_generator**: Runs during period close. Computes current tax from pre-tax income * statutory rate, then adjusts for permanent differences, temporary differences (creating deferred tax), and credits. Generates rate reconciliation.
- **withholding_generator**: For cross-border vendor payments, applies withholding based on vendor country, treaty network, and payment type.

### ML Labels

| Label | Description |
|---|---|
| `IncorrectTaxCode` | Wrong rate applied for jurisdiction/product combination |
| `MissingTaxLine` | Taxable transaction with no tax line |
| `RateArbitrage` | Artificial routing through low-tax jurisdictions |
| `LateFilingRisk` | Filing pattern trending toward deadline |
| `TransferPricingDeviation` | Price outside arm's-length range |
| `WithholdingUnderstatement` | Applied rate below statutory without treaty justification |

### Configuration

```yaml
tax:
  enabled: true
  jurisdictions:
    countries: [US, DE, GB, SG]
    include_subnational: true
  vat_gst:
    enabled: true
    standard_rates: { US: 0.0, DE: 0.19, GB: 0.20, SG: 0.09 }
    reduced_rates: { DE: 0.07, GB: 0.05 }
    exempt_categories: [financial_services, healthcare, education]
    reverse_charge: true
  sales_tax:
    enabled: true
    nexus_states: [CA, NY, TX, FL]
  withholding:
    enabled: true
    treaty_network: true
    rates: { default: 0.30, treaty_reduced: 0.15 }
  provisions:
    enabled: true
    statutory_rate: 0.21
    uncertain_positions: true
  payroll_tax:
    enabled: true
  anomaly_rate: 0.03
```

### Export Files

`tax_codes.csv`, `tax_jurisdictions.csv`, `tax_lines.csv`, `tax_returns.csv`, `tax_provisions.csv`, `rate_reconciliation.csv`, `uncertain_tax_positions.csv`, `withholding_records.csv`, `tax_anomaly_labels.csv`

---

## Domain 2: Treasury & Cash Management

### Data Models

```rust
// --- treasury.rs ---

/// Daily cash position per entity/account/currency
pub struct CashPosition {
    pub id: String,
    pub entity_id: String,
    pub bank_account_id: String,
    pub currency: String,
    pub date: NaiveDate,
    pub opening_balance: Decimal,
    pub inflows: Decimal,
    pub outflows: Decimal,
    pub closing_balance: Decimal,
    pub available_balance: Decimal,
    pub value_date_balance: Decimal,
}

/// Forward-looking cash forecast
pub struct CashForecast {
    pub id: String,
    pub entity_id: String,
    pub currency: String,
    pub forecast_date: NaiveDate,
    pub horizon_days: u32,
    pub items: Vec<CashForecastItem>,
    pub net_position: Decimal,
    pub confidence_level: Decimal,
}

pub struct CashForecastItem {
    pub id: String,
    pub date: NaiveDate,
    pub category: CashFlowCategory,  // ArCollection, ApPayment, PayrollDisbursement,
                                     // TaxPayment, DebtService, CapitalExpenditure,
                                     // IntercompanySettlement, ProjectMilestone, Other
    pub amount: Decimal,
    pub probability: Decimal,
    pub source_document_type: Option<String>,
    pub source_document_id: Option<String>,
}

/// Cash pooling structures
pub struct CashPool {
    pub id: String,
    pub name: String,
    pub pool_type: PoolType,  // PhysicalPooling, NotionalPooling, ZeroBalancing
    pub header_account_id: String,
    pub participant_accounts: Vec<String>,
    pub sweep_time: NaiveTime,
    pub interest_rate_benefit: Decimal,
}

pub struct CashPoolSweep {
    pub id: String,
    pub pool_id: String,
    pub date: NaiveDate,
    pub from_account_id: String,
    pub to_account_id: String,
    pub amount: Decimal,
    pub currency: String,
}

/// Hedging instruments (extends existing FX module)
pub struct HedgingInstrument {
    pub id: String,
    pub instrument_type: HedgeInstrumentType,  // FxForward, FxOption, InterestRateSwap,
                                                // CommodityForward, CrossCurrencySwap
    pub notional_amount: Decimal,
    pub currency: String,
    pub currency_pair: Option<String>,
    pub fixed_rate: Option<Decimal>,
    pub floating_index: Option<String>,  // "SOFR", "EURIBOR"
    pub strike_rate: Option<Decimal>,
    pub trade_date: NaiveDate,
    pub maturity_date: NaiveDate,
    pub counterparty: String,
    pub fair_value: Decimal,
    pub status: InstrumentStatus,  // Active, Matured, Terminated, Novated
}

/// ASC 815 / IFRS 9 hedge accounting designation
pub struct HedgeRelationship {
    pub id: String,
    pub hedged_item_type: HedgedItemType,  // ForecastedTransaction, FirmCommitment,
                                            // RecognizedAsset, NetInvestment
    pub hedged_item_description: String,
    pub hedging_instrument_id: String,
    pub hedge_type: HedgeType,  // FairValueHedge, CashFlowHedge, NetInvestmentHedge
    pub designation_date: NaiveDate,
    pub effectiveness_test_method: EffectivenessMethod,  // DollarOffset, Regression, CriticalTerms
    pub effectiveness_ratio: Decimal,
    pub is_effective: bool,
    pub ineffectiveness_amount: Decimal,
}

/// Debt instruments with covenant monitoring
pub struct DebtInstrument {
    pub id: String,
    pub entity_id: String,
    pub instrument_type: DebtType,  // TermLoan, RevolvingCredit, Bond, CommercialPaper, BridgeLoan
    pub lender: String,
    pub principal: Decimal,
    pub currency: String,
    pub interest_rate: Decimal,
    pub rate_type: RateType,  // Fixed, Variable { spread, index }
    pub origination_date: NaiveDate,
    pub maturity_date: NaiveDate,
    pub amortization_schedule: Vec<AmortizationPayment>,
    pub covenants: Vec<DebtCovenant>,
    pub drawn_amount: Decimal,
    pub facility_limit: Decimal,
}

pub struct AmortizationPayment {
    pub date: NaiveDate,
    pub principal_payment: Decimal,
    pub interest_payment: Decimal,
    pub balance_after: Decimal,
}

pub struct DebtCovenant {
    pub id: String,
    pub covenant_type: CovenantType,  // DebtToEquity, InterestCoverage, CurrentRatio,
                                       // NetWorth, DebtToEbitda, FixedChargeCoverage
    pub threshold: Decimal,
    pub measurement_frequency: Frequency,  // Monthly, Quarterly, Annual
    pub actual_value: Decimal,
    pub measurement_date: NaiveDate,
    pub is_compliant: bool,
    pub headroom: Decimal,
    pub waiver_obtained: bool,
}

/// Letters of credit and bank guarantees
pub struct BankGuarantee {
    pub id: String,
    pub entity_id: String,
    pub guarantee_type: GuaranteeType,  // CommercialLC, StandbyLC, BankGuarantee, PerformanceBond
    pub amount: Decimal,
    pub currency: String,
    pub beneficiary: String,
    pub issuing_bank: String,
    pub issue_date: NaiveDate,
    pub expiry_date: NaiveDate,
    pub status: GuaranteeStatus,  // Active, Drawn, Expired, Cancelled
    pub linked_contract_id: Option<String>,
    pub linked_project_id: Option<String>,
}

/// Intercompany netting runs
pub struct NettingRun {
    pub id: String,
    pub netting_date: NaiveDate,
    pub cycle: NettingCycle,  // Daily, Weekly, Monthly
    pub participating_entities: Vec<String>,
    pub gross_receivables: Decimal,
    pub gross_payables: Decimal,
    pub net_settlement: Decimal,
    pub settlement_currency: String,
    pub positions: Vec<NettingPosition>,
}

pub struct NettingPosition {
    pub entity_id: String,
    pub gross_receivable: Decimal,
    pub gross_payable: Decimal,
    pub net_position: Decimal,
    pub settlement_direction: PayOrReceive,
}
```

### Integration Points

| Existing Module | Integration |
|---|---|
| AP Payments | Each payment creates a cash outflow in CashPosition; scheduled payments feed CashForecast |
| AR Collections | Each receipt creates a cash inflow; AR aging feeds forecast with probability weighting |
| Banking | Bank accounts are the atomic unit of CashPosition; transactions reconcile to positions |
| FX | HedgingInstruments extend the FX module; FX exposures drive hedge decisions |
| Intercompany | NettingRun multilateralizes bilateral IC settlements |
| Payroll | Payroll schedule creates predictable cash outflows in forecast |
| Tax (new) | Tax payment deadlines create forecast items; refunds create inflows |
| Financial Reporting | Debt disclosures, hedge accounting P&L entries, cash flow statement detail |
| Audit | Covenant compliance review, hedge effectiveness testing, cash forecast accuracy |
| Period Close | Interest accruals on debt, hedge mark-to-market, covenant measurement |
| Project (new) | Milestone payments create forecast items; performance bonds link to projects |

### Generator Logic

- **cash_position_generator** (aggregation): Runs after AP/AR/payroll/banking generators. For each bank account, sums inflows and outflows per day from payment records and bank transactions. Computes opening/closing/available balances.
- **cash_forecast_generator**: Looks forward from AP aging (scheduled payments → high probability outflows), AR aging (overdue = lower probability inflows), payroll schedule (100% probability), tax deadlines, and debt service.
- **hedging_generator**: Identifies FX exposures from existing multi-currency AP/AR balances and forecast. Creates FX forwards to cover a configurable percentage. Designates hedge relationships and tests effectiveness.
- **debt_generator** (standalone): Creates loan structures with amortization schedules. Monitors covenants against actual ratios computed from financial statements.
- **cash_pool_generator**: Groups entity bank accounts into pools. Generates daily sweeps moving excess balances to header account.

### ML Labels

| Label | Description |
|---|---|
| `CashForecastMiss` | Actual vs. forecast deviation exceeds threshold |
| `CovenantBreachRisk` | Headroom trending toward zero over time |
| `HedgeIneffectiveness` | Effectiveness ratio outside 80-125% corridor |
| `UnusualCashMovement` | Large unexpected flow (cross-links to AML in banking) |
| `LiquidityCrisis` | Available cash below minimum balance policy |
| `CounterpartyConcentration` | Excessive hedging exposure to single counterparty |

### Configuration

```yaml
treasury:
  enabled: true
  cash_positioning:
    enabled: true
    frequency: daily
    minimum_balance_policy: 100000.0
  cash_forecasting:
    enabled: true
    horizon_days: 90
    ar_collection_probability_curve: aging
    confidence_interval: 0.90
  cash_pooling:
    enabled: true
    pool_type: zero_balancing
    sweep_time: "16:00"
  hedging:
    enabled: true
    hedge_ratio: 0.75
    instruments: [fx_forward, interest_rate_swap]
    hedge_accounting: true
    effectiveness_method: regression_analysis
  debt:
    enabled: true
    instruments:
      - { type: term_loan, principal: 5000000, rate: 0.055, maturity_months: 60 }
      - { type: revolving_credit, facility: 2000000, rate: 0.045 }
    covenants:
      - { type: debt_to_ebitda, threshold: 3.5 }
      - { type: interest_coverage, threshold: 3.0 }
  netting:
    enabled: true
    cycle: monthly
  bank_guarantees:
    enabled: true
    count: 5
  anomaly_rate: 0.02
```

### Export Files

`cash_positions.csv`, `cash_forecasts.csv`, `cash_forecast_items.csv`, `cash_pool_sweeps.csv`, `hedging_instruments.csv`, `hedge_relationships.csv`, `debt_instruments.csv`, `debt_covenants.csv`, `amortization_schedules.csv`, `bank_guarantees.csv`, `netting_runs.csv`, `netting_positions.csv`, `treasury_anomaly_labels.csv`

---

## Domain 3: Project Accounting

### Data Models

```rust
// --- project.rs ---

pub struct Project {
    pub id: String,
    pub code: String,
    pub name: String,
    pub description: String,
    pub project_type: ProjectType,  // FixedPrice, TimeAndMaterials, CostPlus, UnitPrice, Internal
    pub customer_id: Option<String>,
    pub project_manager_id: String,
    pub department_id: String,
    pub start_date: NaiveDate,
    pub planned_end_date: NaiveDate,
    pub actual_end_date: Option<NaiveDate>,
    pub status: ProjectStatus,  // Planning, Active, OnHold, Completed, Cancelled
    pub contract_value: Option<Decimal>,
    pub budget_total: Decimal,
    pub currency: String,
    pub completion_percentage: Decimal,
}

pub struct WbsElement {
    pub id: String,
    pub project_id: String,
    pub parent_id: Option<String>,
    pub code: String,        // "1.2.3"
    pub name: String,
    pub level: u8,
    pub budget_labor: Decimal,
    pub budget_material: Decimal,
    pub budget_subcontractor: Decimal,
    pub budget_overhead: Decimal,
    pub budget_total: Decimal,
    pub is_billing_element: bool,
    pub responsible_person_id: Option<String>,
    pub status: WbsStatus,  // Open, Closed, Locked
}

/// Every cost tracked to a WBS element
pub struct ProjectCostLine {
    pub id: String,
    pub project_id: String,
    pub wbs_element_id: String,
    pub cost_category: CostCategory,  // Labor, Material, Subcontractor, Overhead, Equipment, Travel
    pub source_type: CostSourceType,  // TimeEntry, ExpenseReport, PurchaseOrder, VendorInvoice, JournalEntry
    pub source_document_id: String,
    pub posting_date: NaiveDate,
    pub quantity: Option<Decimal>,
    pub unit_rate: Option<Decimal>,
    pub amount: Decimal,
    pub currency: String,
    pub is_billable: bool,
    pub employee_id: Option<String>,
}

/// Revenue recognition per period
pub struct ProjectRevenue {
    pub id: String,
    pub project_id: String,
    pub wbs_element_id: String,
    pub period: NaiveDate,
    pub recognition_method: RevenueMethod,  // PercentageOfCompletion, CompletedContract, MilestoneBased
    pub completion_measure: CompletionMeasure,  // CostToTotal, UnitsDelivered, MilestoneAchieved, LaborHours
    pub total_contract_revenue: Decimal,
    pub cumulative_cost_incurred: Decimal,
    pub estimated_total_cost: Decimal,
    pub completion_percentage: Decimal,
    pub cumulative_recognized_revenue: Decimal,
    pub current_period_revenue: Decimal,
    pub unbilled_revenue: Decimal,  // contract asset (WIP)
    pub deferred_revenue: Decimal,  // contract liability (overbilling)
}

pub struct ProjectMilestone {
    pub id: String,
    pub project_id: String,
    pub wbs_element_id: Option<String>,
    pub name: String,
    pub planned_date: NaiveDate,
    pub actual_date: Option<NaiveDate>,
    pub billing_amount: Option<Decimal>,
    pub completion_criteria: String,
    pub status: MilestoneStatus,  // Pending, Achieved, Invoiced, Paid
    pub deliverable_description: Option<String>,
}

pub struct ChangeOrder {
    pub id: String,
    pub project_id: String,
    pub description: String,
    pub requested_by: String,
    pub requested_date: NaiveDate,
    pub approved_date: Option<NaiveDate>,
    pub approved_by: Option<String>,
    pub status: ChangeOrderStatus,  // Requested, UnderReview, Approved, Rejected, Withdrawn
    pub cost_impact: Decimal,
    pub revenue_impact: Decimal,
    pub schedule_impact_days: i32,
    pub change_reason: ChangeReason,  // ScopeChange, DesignChange, SiteCondition, Regulatory, ClientRequest
}

pub struct Retainage {
    pub id: String,
    pub project_id: String,
    pub invoice_id: String,
    pub retainage_rate: Decimal,
    pub retained_amount: Decimal,
    pub release_conditions: String,
    pub scheduled_release_date: NaiveDate,
    pub actual_release_date: Option<NaiveDate>,
    pub released_amount: Decimal,
    pub status: RetainageStatus,  // Held, PartiallyReleased, Released
}

/// Earned Value Management metrics per period
pub struct EarnedValueMetric {
    pub id: String,
    pub project_id: String,
    pub wbs_element_id: Option<String>,
    pub period_date: NaiveDate,
    pub planned_value: Decimal,     // BCWS
    pub earned_value: Decimal,      // BCWP
    pub actual_cost: Decimal,       // ACWP
    pub schedule_variance: Decimal, // EV - PV
    pub cost_variance: Decimal,     // EV - AC
    pub spi: Decimal,               // EV / PV
    pub cpi: Decimal,               // EV / AC
    pub estimate_at_completion: Decimal,
    pub estimate_to_complete: Decimal,
    pub variance_at_completion: Decimal,
    pub tcpi: Decimal,              // to-complete performance index
}
```

### Integration Points

| Existing Module | Integration |
|---|---|
| Time Entries (HR) | Optional `project_id` + `wbs_element_id` fields → `ProjectCostLine` (Labor) |
| Expense Reports (HR) | Optional `project_id` on expense line items → `ProjectCostLine` (Travel) |
| Purchase Orders (P2P) | POs can reference a project → `ProjectCostLine` (Material/Subcontractor) |
| Vendor Invoices (P2P) | Subcontractor invoices link to project; retainage withheld |
| Revenue Recognition (Standards) | Project revenue uses ASC 606 "over time" model |
| Budgets | Project budgets integrate with master budget as cost center-level details |
| Customer (Master Data) | Fixed-price projects link to customer |
| Financial Reporting | WIP (contract assets/liabilities) on balance sheet; project revenue on P&L |
| AR | Milestone billing creates AR invoices; retainage creates long-term receivables |
| Audit | Revenue recognition review, PoC estimation review, change order scrutiny |
| Treasury (new) | Milestone payments create forecast items; performance bonds link to projects |
| Tax (new) | Project revenue in foreign jurisdictions triggers withholding |

### Generator Logic

- **project_generator** (standalone): Creates projects with realistic WBS hierarchies (3-5 levels deep). Project types distributed per config. Budget allocated top-down through WBS.
- **project_cost_generator** (linking): Tags a configurable percentage of existing time entries, expense reports, and POs with project/WBS references. This reuses existing generator output rather than creating new transactions.
- **revenue_generator**: Computes PoC based on cost-to-total ratios per period. Generates revenue recognition entries. Handles overbilling (deferred revenue) and underbilling (WIP/contract assets).
- **earned_value_generator**: Computes EVM metrics from planned vs. actual cost and schedule data. Generates SPI, CPI, EAC, ETC, TCPI.
- **change_order_generator**: Injects change orders with configurable probability. Adjusts project budget and schedule. Tracks approval workflow.

### ML Labels

| Label | Description |
|---|---|
| `CostOverrun` | CPI below threshold, graded by severity |
| `ScheduleSlippage` | SPI below threshold |
| `AggressiveCompletion` | PoC jumps inconsistent with cost curve (revenue manipulation red flag) |
| `ChangeOrderChurning` | Excessive change orders relative to contract value |
| `RetainageManipulation` | Early release or waived retainage |
| `UnbilledRevenueAccumulation` | Growing WIP without billing (cash flow risk) |
| `EstimateRevision` | Large EAC revisions (estimation problems or fraud) |

### Configuration

```yaml
project_accounting:
  enabled: true
  project_count: 25
  project_types:
    fixed_price: 0.40
    time_and_materials: 0.35
    cost_plus: 0.15
    internal: 0.10
  wbs:
    max_depth: 4
    elements_per_level: [1, 3, 5, 8]
  cost_allocation:
    time_entry_project_rate: 0.60
    expense_project_rate: 0.40
    po_project_rate: 0.25
  revenue_recognition:
    method: percentage_of_completion
    completion_measure: cost_to_total
  milestones:
    avg_per_project: 5
    billing_milestone_rate: 0.60
  change_orders:
    probability: 0.30
    avg_per_project: 2
  retainage:
    enabled: true
    default_rate: 0.10
  earned_value:
    enabled: true
    reporting_frequency: monthly
  anomaly_rate: 0.04
```

### Export Files

`projects.csv`, `wbs_elements.csv`, `project_cost_lines.csv`, `project_revenue.csv`, `project_milestones.csv`, `change_orders.csv`, `retainage.csv`, `earned_value_metrics.csv`, `project_anomaly_labels.csv`

---

## Domain 4: ESG / Sustainability Reporting

### Data Models

```rust
// --- esg.rs ---

// === Environmental ===

pub struct EmissionRecord {
    pub id: String,
    pub entity_id: String,
    pub facility_id: Option<String>,
    pub scope: EmissionScope,  // Scope1, Scope2, Scope3
    pub scope3_category: Option<Scope3Category>,  // 15 GHG Protocol categories
    pub activity_type: EmissionActivity,
    pub activity_data: Decimal,
    pub activity_unit: String,
    pub emission_factor: Decimal,
    pub emission_factor_source: String,
    pub co2_tonnes: Decimal,
    pub ch4_tonnes: Decimal,
    pub n2o_tonnes: Decimal,
    pub co2e_tonnes: Decimal,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub data_quality_score: u8,  // 1-5 per GHG Protocol
    pub estimation_method: EstimationMethod,  // DirectMeasurement, ActivityBased, SpendBased, AverageBased
}

pub enum Scope3Category {
    PurchasedGoods,        // Cat 1
    CapitalGoods,          // Cat 2
    FuelAndEnergy,         // Cat 3
    UpstreamTransport,     // Cat 4
    Waste,                 // Cat 5
    BusinessTravel,        // Cat 6
    EmployeeCommuting,     // Cat 7
    UpstreamLeased,        // Cat 8
    DownstreamTransport,   // Cat 9
    Processing,            // Cat 10
    UseOfProducts,         // Cat 11
    EndOfLife,             // Cat 12
    DownstreamLeased,      // Cat 13
    Franchises,            // Cat 14
    Investments,           // Cat 15
}

pub struct EnergyConsumption {
    pub id: String,
    pub entity_id: String,
    pub facility_id: Option<String>,
    pub energy_type: EnergyType,
    pub is_renewable: bool,
    pub consumption_kwh: Decimal,
    pub cost: Decimal,
    pub currency: String,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub source: DataSource,  // Metered, Estimated, InvoiceBased
}

pub struct WaterUsage {
    pub id: String,
    pub entity_id: String,
    pub facility_id: Option<String>,
    pub source: WaterSource,  // Municipal, Groundwater, SurfaceWater, Recycled, Rainwater
    pub withdrawal_m3: Decimal,
    pub consumption_m3: Decimal,
    pub discharge_m3: Decimal,
    pub water_stress_area: bool,
    pub period: NaiveDate,
}

pub struct WasteRecord {
    pub id: String,
    pub entity_id: String,
    pub facility_id: Option<String>,
    pub waste_type: WasteType,  // Hazardous, NonHazardous, Recyclable, Organic, EWaste
    pub quantity_tonnes: Decimal,
    pub disposal_method: DisposalMethod,  // Landfill, Recycling, Incineration, Composting, Reuse
    pub diversion_rate: Decimal,
    pub period: NaiveDate,
}

// === Social ===

pub struct WorkforceDiversityMetric {
    pub id: String,
    pub entity_id: String,
    pub period: NaiveDate,
    pub dimension: DiversityDimension,  // Gender, Ethnicity, AgeGroup, Disability, Veteran
    pub category: String,
    pub level: OrganizationLevel,  // AllEmployees, Management, SeniorManagement, Executive, Board
    pub headcount: u32,
    pub percentage: Decimal,
}

pub struct PayEquityMetric {
    pub id: String,
    pub entity_id: String,
    pub period: NaiveDate,
    pub dimension: DiversityDimension,
    pub category_a: String,
    pub category_b: String,
    pub median_pay_ratio: Decimal,
    pub adjusted_pay_ratio: Decimal,
    pub job_level: Option<String>,
}

pub struct SafetyIncident {
    pub id: String,
    pub entity_id: String,
    pub facility_id: Option<String>,
    pub date: NaiveDate,
    pub incident_type: IncidentType,  // Injury, NearMiss, Fatality, Illness, EnvironmentalRelease
    pub severity: Severity,
    pub lost_days: u32,
    pub root_cause: String,
    pub corrective_action: String,
    pub is_recordable: bool,
}

pub struct SafetyMetric {
    pub id: String,
    pub entity_id: String,
    pub period: NaiveDate,
    pub total_hours_worked: Decimal,
    pub recordable_incidents: u32,
    pub lost_time_incidents: u32,
    pub trir: Decimal,
    pub ltir: Decimal,
    pub dart_rate: Decimal,
}

// === Governance ===

pub struct GovernanceMetric {
    pub id: String,
    pub entity_id: String,
    pub period: NaiveDate,
    pub board_size: u8,
    pub independent_directors: u8,
    pub female_board_members: u8,
    pub board_meetings_attended_pct: Decimal,
    pub ceo_to_median_pay_ratio: Decimal,
    pub ethics_training_completion_pct: Decimal,
    pub whistleblower_reports: u32,
    pub data_privacy_incidents: u32,
    pub anti_corruption_training_pct: Decimal,
}

// === Supply Chain ESG ===

pub struct SupplierEsgAssessment {
    pub id: String,
    pub vendor_id: String,
    pub assessment_date: NaiveDate,
    pub environmental_score: Decimal,
    pub social_score: Decimal,
    pub governance_score: Decimal,
    pub overall_score: Decimal,
    pub scope3_emission_estimate: Decimal,
    pub risk_flags: Vec<EsgRiskFlag>,
    pub corrective_actions_required: Vec<String>,
    pub next_review_date: NaiveDate,
    pub assessment_method: AssessmentMethod,
}

pub enum EsgRiskFlag {
    ChildLabor, ForcedLabor, Deforestation, WaterPollution,
    ExcessiveEmissions, SafetyViolation, CorruptionRisk,
    DataPrivacyBreach, ConflictMinerals,
}

// === Reporting ===

pub struct EsgDisclosure {
    pub id: String,
    pub entity_id: String,
    pub framework: EsgFramework,  // Csrd, Gri, Sasb, Tcfd, Issb, SecClimate
    pub standard_id: String,
    pub metric_name: String,
    pub metric_value: Decimal,
    pub metric_unit: String,
    pub period: NaiveDate,
    pub assurance_level: AssuranceLevel,  // None, Limited, Reasonable
    pub restated: bool,
    pub notes: Option<String>,
}

pub struct MaterialityAssessment {
    pub id: String,
    pub entity_id: String,
    pub topic: String,
    pub financial_materiality: Decimal,
    pub impact_materiality: Decimal,
    pub is_material: bool,
    pub stakeholder_priority: Decimal,
    pub assessment_date: NaiveDate,
}

pub struct ClimateScenario {
    pub id: String,
    pub entity_id: String,
    pub scenario: ScenarioType,  // OnePointFive, TwoDegree, ThreeDegree, CurrentPolicies
    pub time_horizon: TimeHorizon,  // Short2030, Medium2040, Long2050
    pub physical_risk_score: Decimal,
    pub transition_risk_score: Decimal,
    pub financial_impact_low: Decimal,
    pub financial_impact_high: Decimal,
    pub stranded_asset_risk: Decimal,
    pub carbon_price_assumption: Decimal,
}
```

### Integration Points

| Existing Module | Integration |
|---|---|
| Vendor Network | Every vendor gets `SupplierEsgAssessment`; Scope 3 Cat 1 allocated from vendor spend |
| Manufacturing | Production orders → Scope 1 emissions; energy per production run |
| HR/Payroll | Employee data → `WorkforceDiversityMetric` and `PayEquityMetric` |
| Expenses | Business travel expenses → Scope 3 Cat 6 emissions |
| Fixed Assets | Asset energy efficiency; green capex tracking; stranded asset identification |
| Financial Reporting | ESG disclosures as supplementary schedules |
| Audit | ESG assurance engagements (ISAE 3000/3410) |
| Internal Controls | Governance metrics extend COSO framework |
| KPIs | ESG KPIs alongside financial KPIs |
| Tax (new) | Carbon tax creates tax line items; green incentives reduce tax provision |
| Treasury (new) | Green bonds with ESG covenants; ESG compliance costs in cash forecast |
| Project (new) | Construction project emissions; project ESG impact assessment |

### Generator Logic

- **emission_generator** (derivation): Derives Scope 1 from manufacturing energy data, Scope 2 from purchased electricity (energy consumption), Scope 3 from vendor spend (Cat 1), business travel expenses (Cat 6), and employee count (Cat 7 commuting). Applies region-specific emission factors.
- **energy_generator** (standalone + derivation): Creates facility-level energy records. For manufacturing companies, correlates energy with production volume.
- **workforce_generator** (derivation): Aggregates existing employee master data by gender, ethnicity, age, disability, and org level. Computes pay equity ratios from salary data.
- **supplier_esg_generator** (derivation): Scores existing vendors based on industry, country risk, vendor quality scores (from vendor network module). High-risk flags based on country and industry combination.
- **disclosure_generator**: Maps all calculated metrics to framework-specific disclosure IDs (GRI 305-1, ESRS E1-6, etc.). Checks materiality assessment to determine which disclosures are required.

### ML Labels

| Label | Description |
|---|---|
| `GreenwashingIndicator` | Reported emissions inconsistent with activity data |
| `DiversityStagnation` | No improvement trend despite stated targets |
| `SupplyChainRisk` | High Scope 3 concentration in high-risk regions |
| `DataQualityGap` | Excessive estimated vs. measured data |
| `MissingDisclosure` | Material topics without corresponding disclosures |
| `ScenarioInconsistency` | Climate assumptions contradict actual capital expenditure patterns |

### Configuration

```yaml
esg:
  enabled: true
  environmental:
    emissions:
      enabled: true
      scope1: true
      scope2: true
      scope3:
        enabled: true
        categories: [1, 3, 4, 5, 6, 7]
        allocation_method: spend_based
      emission_factor_source: epa
    energy:
      enabled: true
      renewable_percentage: 0.35
    water:
      enabled: true
      stress_area_facilities_pct: 0.20
    waste:
      enabled: true
      diversion_target: 0.75
  social:
    diversity:
      enabled: true
      dimensions: [gender, ethnicity, age_group]
      levels: [all, management, executive, board]
    pay_equity:
      enabled: true
      gender_pay_gap: 0.13
      adjusted_gap: 0.03
    safety:
      enabled: true
      target_trir: 1.5
      near_miss_ratio: 10
  governance:
    enabled: true
    board_independence_pct: 0.75
    ceo_pay_ratio: 250
  supply_chain:
    enabled: true
    assessment_coverage_pct: 0.80
    high_risk_threshold: 40
  reporting:
    frameworks: [csrd, gri]
    assurance_level: limited
    double_materiality: true
  climate_scenarios:
    enabled: true
    scenarios: [1.5_degree, 2_degree, current_policies]
  anomaly_rate: 0.03
```

### Export Files

`emission_records.csv`, `energy_consumption.csv`, `water_usage.csv`, `waste_records.csv`, `workforce_diversity.csv`, `pay_equity_metrics.csv`, `safety_incidents.csv`, `safety_metrics.csv`, `governance_metrics.csv`, `supplier_esg_assessments.csv`, `esg_disclosures.csv`, `materiality_assessments.csv`, `climate_scenarios.csv`, `esg_anomaly_labels.csv`

---

## Cross-Domain Linkage Map

```
                    ┌──────────────────────────────────────┐
                    │          EXISTING MODULES             │
                    │                                       │
  ┌─────────┐      │  AP ─── AR ─── GL ─── Financial Stmts │
  │   TAX   │◄────►│  │      │      │       │              │
  │         │      │  Payments  Banking  FX  Period Close   │
  │ tax lines│     │  │      │      │       │              │
  │ on every │     │  P2P    O2C   HR/Payroll  Mfg        │
  │ document │     │  │      │      │       │              │
  └────┬────┘      │  Vendors Customers Employees Assets   │
       │           │  │      │      │       │              │
       │           │  Intercompany  Controls  Audit        │
       │           └──┬──────┬──────┬───────┬──────────────┘
       │              │      │      │       │
       ▼              ▼      ▼      ▼       ▼
  ┌─────────┐   ┌─────────┐  ┌──────────┐  ┌─────────┐
  │TREASURY │◄─►│ PROJECT │  │   ESG    │  │  TAX    │
  │         │   │  ACCTG  │  │          │  │(return) │
  │cash pos.│   │         │  │emissions │  │         │
  │hedging  │   │ WBS     │  │from mfg  │  │tax on   │
  │debt mgmt│   │ costs   │  │diversity │  │project  │
  │covenants│   │ from HR │  │from HR   │  │revenue  │
  │netting  │   │ revenue │  │supply    │  │         │
  │         │   │ from AR │  │chain ESG │  │         │
  └────┬────┘   └────┬────┘  │from vend.│  └─────────┘
       │              │       └────┬─────┘
       │              │            │
       └──────────────┴────────────┘
```

### Cross-Linkages Between the Four New Domains

| From → To | Linkage |
|---|---|
| Tax → Treasury | Tax payment deadlines create cash forecast items; foreign tax refunds create inflows |
| Treasury → Tax | Hedging gains/losses have tax implications; debt interest is tax-deductible |
| Project → Tax | Project revenue in foreign jurisdictions triggers withholding; retainage tax treatment |
| Project → Treasury | Milestone payments create forecast items; project financing uses debt instruments; performance bonds |
| ESG → Tax | Carbon tax creates tax line items; green incentives reduce tax provision |
| ESG → Treasury | Green bonds (debt with ESG covenants); ESG compliance costs in cash forecast |
| ESG → Project | Construction projects generate Scope 1 emissions; project ESG impact assessment |
| Tax → ESG | Carbon tax payments appear in both tax returns and emission cost tracking |

---

## Implementation Sequence

Based on dependency analysis:

1. **Tax** -- decorates existing data; all other new domains need tax lines
2. **Treasury** -- aggregates from AP/AR/payroll + tax; needed for project cash forecasting
3. **Project Accounting** -- uses tax lines, feeds treasury forecasts, reuses existing generators
4. **ESG** -- most independent; draws from all other modules including the new three

Each domain follows the same implementation pattern:
1. Models in `datasynth-core`
2. Config schema in `datasynth-config`
3. Generators in `datasynth-generators`
4. Orchestrator integration in `datasynth-runtime`
5. Export integration in `datasynth-output`
6. Tests at each layer

---

## OCEL 2.0 Process Mining Integration

Each domain adds process variants to the existing OCPM module:

| Domain | Process Events |
|---|---|
| Tax | TaxDetermination → TaxLineCreated → ReturnFiled → ReturnAssessed → TaxPaid |
| Treasury | CashPositionCalculated → ForecastGenerated → HedgeDesignated → CovenantMeasured |
| Project | ProjectCreated → CostPosted → MilestoneAchieved → RevenueRecognized → ChangeOrderProcessed |
| ESG | DataCollected → EmissionCalculated → DisclosurePrepared → AssuranceCompleted |

---

## Presets Impact

Existing presets gain new capabilities:

| Preset | Tax | Treasury | Project | ESG |
|---|---|---|---|---|
| Manufacturing | VAT + payroll tax | Cash pooling, FX hedging | Internal projects (capex) | Full (Scope 1/2/3, safety) |
| Retail | Sales tax by state | Cash positioning | Minimal | Supply chain ESG, waste |
| Financial Services | Withholding + VAT exempt | Full treasury suite | Minimal | Governance-heavy |
| Healthcare | Sales tax exempt | Cash positioning | Research projects | Safety, diversity |
| Technology | R&D tax credits | Cash forecasting | T&M projects, internal R&D | Scope 2/3, diversity |
| Construction (new) | Retainage tax, multi-jurisdiction | Performance bonds, project financing | Full project accounting | Scope 1 (construction), safety |
| Professional Services (new) | Multi-jurisdiction services tax | Cash forecasting | Full T&M project accounting | Diversity, governance |
