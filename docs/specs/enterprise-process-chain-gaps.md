# Enterprise Process Chain Gaps — Specification & Implementation Hints

> **Status:** Draft
> **Date:** 2026-02-11
> **Companion to:** `s2p-process-chain-spec.md`
> **Scope:** All process chains beyond S2P that have gaps

---

## 1. Coverage Landscape

```
Process Chain               Coverage   Gap Priority   Interconnections
────────────────────────────────────────────────────────────────────────
O2C  Order-to-Cash            93%      Low            R2R, INV, BANK
R2R  Record-to-Report         78%      Medium         ALL chains
H2R  Hire-to-Retire           30%      High           R2R, S2P, BANK
A2R  Acquire-to-Retire        70%      Medium         S2P, R2R, BANK
INV  Inventory Management     55%      Medium         S2P, O2C, MFG
BANK Banking / Treasury       65%      High           S2P, O2C, H2R, A2R
MFG  Plan-to-Produce          20%      High           S2P, INV, R2R
```

Each section below follows the same structure:
1. What exists today (brief summary)
2. What's missing
3. New models needed (Rust struct sketches)
4. Generator logic
5. Config additions
6. Coherence rules
7. Anomaly injection points
8. Implementation hints (where to place files, what existing code to reuse)

---

## 2. Order-to-Cash (O2C) — Closing the Last 7%

### 2.1 What Exists

The O2C chain is nearly complete: `SalesOrder` (9 types) → `Delivery` (6 types, pick/pack/ship) → `CustomerInvoice` (7 types, dunning, disputes) → `Payment` (5 behaviors: partial, short, on-account, corrections, NSF). Revenue recognition models exist (`CustomerContract`, `PerformanceObligation` per ASC 606/IFRS 15). OCPM has 10 O2C activities.

### 2.2 What's Missing

| Gap | Impact |
|-----|--------|
| **Customer Quote / Sales Quote** | No quote-to-order conversion tracking; `SalesOrder.quote_id` is an orphan FK |
| **Revenue Recognition Generator** | `CustomerContract` + `PerformanceObligation` models exist but no generator wires them to SO/Invoice data |

### 2.3 New Models

#### `SalesQuote` — `datasynth-core/src/models/documents/sales_quote.rs`

```rust
pub enum QuoteStatus {
    Draft,
    Sent,
    UnderReview,
    Accepted,        // Converts to SO
    Rejected,
    Expired,
    Superseded,      // Replaced by newer quote
}

pub struct SalesQuoteItem {
    pub base: DocumentLineItem,
    pub discount_percent: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub alternative_item: bool,          // Optional alternative for customer
    pub is_selected: bool,               // Customer selected this alternative
}

pub struct SalesQuote {
    pub header: DocumentHeader,
    pub quote_type: QuoteType,           // Standard, Rush, Framework
    pub status: QuoteStatus,
    pub customer_id: String,
    pub contact_person: Option<String>,
    pub items: Vec<SalesQuoteItem>,
    pub total_net_amount: Decimal,
    pub total_tax_amount: Decimal,
    pub total_gross_amount: Decimal,
    pub currency: String,
    pub payment_terms: String,
    pub incoterms: Option<String>,
    pub validity_start: NaiveDate,
    pub validity_end: NaiveDate,
    pub sales_org: String,
    pub distribution_channel: String,
    pub sales_employee_id: Option<String>,
    pub rfq_reference: Option<String>,       // Links to S2P RfxEvent (customer's RFQ)
    pub converted_to_so_id: Option<String>,  // FK to SalesOrder
    pub conversion_date: Option<NaiveDate>,
    pub rejection_reason: Option<String>,
    pub competitor_info: Option<String>,      // Lost-to competitor
    pub win_probability: Option<f64>,        // Sales pipeline probability
    pub revision_number: u16,
    pub previous_quote_id: Option<String>,   // For superseded quotes
}
```

### 2.4 Generator Logic

**Quote Generator** — `datasynth-generators/src/document_flow/quote_generator.rs`

- Generate quotes for a configurable percentage of sales orders (default: 60% of SOs have a preceding quote)
- Quote-to-SO conversion rate: configurable (default: 40% — many quotes don't convert)
- Quotes per customer request: 1-3 (revisions)
- Timeline: Quote validity 30-90 days, conversion within validity window
- Pricing: Quote amounts 5-15% higher than final SO (negotiation discount)
- Alternative items: 10-20% of quotes include alternatives

**Revenue Recognition Generator** — `datasynth-generators/src/document_flow/revrec_generator.rs`

- Post-process O2C chains: for each `CustomerInvoice`, create a `CustomerContract` with `PerformanceObligation`s
- Simple cases (single delivery = single PO): 1 contract, 1 PO, full recognition at delivery
- Complex cases (multi-element arrangements): multiple POs with standalone selling price allocation
- Variable consideration: volume discounts, rebates, right-of-return (use existing model fields)
- Links `CustomerContract.sales_order_id` to `SalesOrder.id`

**Implementation hint:** The `CustomerContract` struct already has `sales_order_id: Option<Uuid>` in `revenue.rs`. The generator just needs to populate it and create matching `PerformanceObligation` entries with `recognition_pattern` set based on delivery type.

### 2.5 Config Additions

```yaml
document_flows:
  o2c:
    # Existing fields unchanged...
    quotes:
      enabled: true
      quote_to_so_rate: 0.60              # % of SOs that have a preceding quote
      conversion_rate: 0.40               # % of quotes that convert to SO
      avg_validity_days: 60
      revision_rate: 0.25                 # % of quotes that go through revision
      negotiation_discount_percent: 0.08  # Avg discount from quote to SO price
    revenue_recognition:
      enabled: true
      multi_element_rate: 0.15            # % of contracts with multiple POs
      variable_consideration_rate: 0.10   # % with volume discounts / rebates
      right_of_return_rate: 0.05          # % with return rights affecting recognition
```

### 2.6 Coherence Rules

| Rule ID | Constraint |
|---------|-----------|
| O2C-001 | `SalesOrder.quote_id` (when set) must reference a valid `SalesQuote` with status `Accepted` |
| O2C-002 | `SalesQuote.converted_to_so_id` must match `SalesOrder.quote_id` (bidirectional) |
| O2C-003 | `SalesQuote.validity_end` ≥ `SalesOrder.document_date` |
| O2C-004 | `CustomerContract.sales_order_id` must reference a valid SO |
| O2C-005 | Sum of `PerformanceObligation.allocated_price` must equal `CustomerContract.transaction_price` |

### 2.7 Anomalies

| Type | Category | Description |
|------|----------|-------------|
| `QuotePriceOverride` | Process | SO price exceeds quote price (unapproved price increase) |
| `ExpiredQuoteConversion` | Process | SO created from expired quote |
| `RevenueTimingManipulation` | Fraud | PO recognized before delivery (channel stuffing) |

### 2.8 Implementation Hints

- Add `sales_quote.rs` to `crates/datasynth-core/src/models/documents/`
- Export in `documents/mod.rs`
- Add `QuoteType` to `DocumentType` enum in `document_chain.rs`
- OCPM: Add 3 new activities: `create_quote`, `send_quote`, `convert_quote`
- Reuse `DocumentHeader` and `DocumentLineItem` base structs for consistency
- The `O2CDocumentChain` in `o2c_generator.rs` should gain a `quote: Option<SalesQuote>` field

---

## 3. Record-to-Report (R2R) — Closing the Last 22%

### 3.1 What Exists

JE recording (balanced, SOX-tracked), intercompany (7 IC types, transfer pricing, matching, elimination), period-end close (task DAG with 20 `CloseTask` variants, accruals with auto-reversal, depreciation), currency translation (ASC 821/IFRS, CTA), consolidation (4 elimination types), trial balance (unadjusted/adjusted/post-closing). OCPM has 15 R2R activities. Full audit module (ISA, PCAOB, SOX, opinions).

### 3.2 What's Missing

| Gap | Impact |
|-----|--------|
| **Financial Statements** | No Balance Sheet, Income Statement, Cash Flow Statement, or Statement of Changes in Equity |
| **Management Reporting / KPIs** | No ratio calculations, variance analysis, segment reporting |
| **Budgets** | No budget/forecast data for variance analysis |

### 3.3 New Models

#### Financial Statements — `datasynth-core/src/models/financial_statements.rs`

```rust
pub enum StatementType {
    BalanceSheet,
    IncomeStatement,
    CashFlowStatement,
    ChangesInEquity,
}

pub enum StatementBasis {
    Individual,        // Single entity
    Consolidated,      // Group consolidation
}

pub struct FinancialStatementLineItem {
    pub line_id: String,
    pub label: String,
    pub gl_account_range: Option<(String, String)>,  // Account range mapping
    pub amount_current: Decimal,
    pub amount_prior: Decimal,
    pub amount_budget: Option<Decimal>,
    pub variance_amount: Option<Decimal>,
    pub variance_percent: Option<f64>,
    pub indent_level: u8,                            // For grouping/subtotals
    pub is_subtotal: bool,
    pub is_total: bool,
    pub notes_reference: Option<String>,             // "Note 1", "Note 2", etc.
}

pub struct FinancialStatement {
    pub statement_id: String,
    pub statement_type: StatementType,
    pub basis: StatementBasis,
    pub company_code: String,
    pub fiscal_year: u16,
    pub fiscal_period: u8,
    pub period_end_date: NaiveDate,
    pub currency: String,
    pub line_items: Vec<FinancialStatementLineItem>,
    pub total_amount: Decimal,                       // Net income / total assets / etc.
    pub is_audited: bool,
    pub prepared_by: Option<String>,
    pub approved_by: Option<String>,
    pub approval_date: Option<NaiveDate>,
}

/// Cash flow classification for indirect method.
pub enum CashFlowCategory {
    Operating,
    Investing,
    Financing,
}

pub struct CashFlowItem {
    pub category: CashFlowCategory,
    pub label: String,
    pub amount: Decimal,
    pub is_non_cash_adjustment: bool,    // For indirect method (depreciation, etc.)
}
```

#### Management KPIs — `datasynth-core/src/models/management_kpi.rs`

```rust
pub struct FinancialKpi {
    pub kpi_id: String,
    pub company_code: String,
    pub fiscal_year: u16,
    pub fiscal_period: u8,
    pub period_end_date: NaiveDate,
    // Liquidity
    pub current_ratio: f64,
    pub quick_ratio: f64,
    pub cash_ratio: f64,
    // Profitability
    pub gross_margin: f64,
    pub operating_margin: f64,
    pub net_margin: f64,
    pub return_on_assets: f64,
    pub return_on_equity: f64,
    // Efficiency
    pub dso_days: f64,                   // Days Sales Outstanding
    pub dpo_days: f64,                   // Days Payable Outstanding
    pub dio_days: f64,                   // Days Inventory Outstanding
    pub cash_conversion_cycle: f64,      // DSO + DIO - DPO
    pub asset_turnover: f64,
    pub inventory_turnover: f64,
    // Leverage
    pub debt_to_equity: f64,
    pub interest_coverage: f64,
    // Procurement (links to S2P)
    pub contract_coverage_rate: Option<f64>,
    pub maverick_spend_rate: Option<f64>,
    pub supplier_on_time_delivery: Option<f64>,
}

pub struct BudgetLine {
    pub gl_account: String,
    pub cost_center: Option<String>,
    pub fiscal_year: u16,
    pub fiscal_period: u8,
    pub budget_amount: Decimal,
    pub actual_amount: Decimal,
    pub variance: Decimal,
    pub variance_percent: f64,
    pub is_favorable: bool,
}
```

### 3.4 Generator Logic

**Financial Statement Generator** — `datasynth-generators/src/period_close/financial_statement_generator.rs`

- Consumes adjusted trial balance (already generated by `TrialBalanceGenerator`)
- Maps GL accounts to statement line items using CoA account categories (existing `AccountCategory` enum: Assets, Liabilities, Equity, Revenue, Expense)
- Balance Sheet: Group by account category, compute subtotals (Current Assets, Non-Current Assets, etc.)
- Income Statement: Revenue − COGS = Gross Profit − OpEx = Operating Income − Interest/Tax = Net Income
- Cash Flow (indirect method): Start with Net Income, add back non-cash items (depreciation from `DepreciationRunGenerator`), adjust for working capital changes (AR, AP, Inventory deltas from period-over-period trial balance)
- Changes in Equity: Beginning balance + Net Income − Dividends + OCI (CTA from `CTAGenerator`)
- Prior period comparison: Use previous period's trial balance for `amount_prior`

**KPI Generator** — `datasynth-generators/src/period_close/kpi_generator.rs`

- Computes ratios from financial statement line items
- DSO = (AR / Revenue) × Days
- DPO = (AP / COGS) × Days
- DIO = (Inventory / COGS) × Days
- Current Ratio = Current Assets / Current Liabilities
- Existing `BalanceConfig` already has `target_dso_days`, `target_dpo_days`, `target_current_ratio`, `target_debt_to_equity` — these can serve as validation targets

**Budget Generator** — `datasynth-generators/src/period_close/budget_generator.rs`

- Generate budgets as prior-year actuals × (1 + growth_rate) with noise
- Revenue budget: prior year + configurable growth (default: 5%)
- Expense budget: prior year + inflation (default: 3%)
- Variance = Actual − Budget (favorable if revenue above or expense below)

### 3.5 Config Additions

```yaml
financial_reporting:
  enabled: true
  generate_balance_sheet: true
  generate_income_statement: true
  generate_cash_flow: true            # Indirect method
  generate_changes_in_equity: true
  generate_segment_reporting: false   # By company_code / department
  comparative_periods: 1              # Number of prior periods to include
  management_kpis:
    enabled: true
    frequency: monthly                # monthly, quarterly
  budgets:
    enabled: true
    revenue_growth_rate: 0.05
    expense_inflation_rate: 0.03
    variance_noise: 0.10              # ±10% random deviation from budget
```

### 3.6 Coherence Rules

| Rule ID | Constraint |
|---------|-----------|
| R2R-001 | Balance Sheet: Total Assets = Total Liabilities + Total Equity |
| R2R-002 | Income Statement Net Income = Change in Retained Earnings (adjusted for dividends) |
| R2R-003 | Cash Flow ending cash = Balance Sheet cash position |
| R2R-004 | KPI values derivable from statement line items (no orphan metrics) |
| R2R-005 | Budget variance = Actual − Budget (sign convention consistent) |

### 3.7 Implementation Hints

- Place statement models in `datasynth-core/src/models/financial_statements.rs`
- Generators go into `datasynth-generators/src/period_close/` alongside `close_engine.rs`
- Run order: TrialBalance → FinancialStatements → KPIs (sequential in `CloseEngine`)
- Add `GenerateFinancialStatements` and `CalculateKpis` to existing `CloseTask` enum (already has `GenerateFinancialStatements` at line 537 of `period_close.rs` — just wire it)
- Output: `balance_sheet.csv`, `income_statement.csv`, `cash_flow_statement.csv`, `financial_kpis.csv`, `budget_variance.csv`

---

## 4. Hire-to-Retire (H2R) — From 30% to Useful

### 4.1 What Exists

Rich `Employee` model (job levels, departments, approval limits, 13 system roles, transaction code auth, working hours). `EmployeeGenerator` with department definitions, cultural name generation, hierarchy building. `GhostEmployee` and `ExpenseReimbursement` anomaly types. `EmployeeStatus` lifecycle (Active → OnLeave → Terminated → Retired).

### 4.2 What's Missing

| Gap | Impact |
|-----|--------|
| **Payroll** | No salary structure, tax withholding, deductions, payroll runs, payslips |
| **Time & Attendance** | No time entries, attendance records, overtime tracking |
| **Expense Management** | No expense reports (only anomaly type exists, no legitimate data) |

### 4.3 New Models

#### Payroll — `datasynth-core/src/models/payroll.rs`

```rust
pub enum PayFrequency {
    Weekly,
    Biweekly,
    SemiMonthly,
    Monthly,
}

pub enum DeductionType {
    FederalTax,
    StateTax,
    LocalTax,
    SocialSecurity,
    Medicare,
    HealthInsurance,
    DentalInsurance,
    VisionInsurance,
    Retirement401k,
    LifeInsurance,
    UnionDues,
    GarnishmentOrLevy,
    ParkingBenefit,
    Other(String),
}

pub enum EarningType {
    BaseSalary,
    HourlyWage,
    Overtime,
    Bonus,
    Commission,
    HolidayPay,
    SickPay,
    VacationPay,
    Allowance,
    Reimbursement,
}

pub struct PayrollRun {
    pub run_id: String,
    pub company_code: String,
    pub pay_period_start: NaiveDate,
    pub pay_period_end: NaiveDate,
    pub payment_date: NaiveDate,
    pub pay_frequency: PayFrequency,
    pub status: PayrollRunStatus,        // Draft, Calculated, Approved, Posted, Paid
    pub total_gross: Decimal,
    pub total_deductions: Decimal,
    pub total_employer_taxes: Decimal,
    pub total_net: Decimal,
    pub employee_count: u32,
    pub journal_entry_id: Option<String>,
    pub approved_by: Option<String>,
    pub posted_by: Option<String>,
}

pub struct PayslipEntry {
    pub payslip_id: String,
    pub payroll_run_id: String,
    pub employee_id: String,
    pub company_code: String,
    pub pay_period_start: NaiveDate,
    pub pay_period_end: NaiveDate,
    pub payment_date: NaiveDate,
    // Earnings
    pub earnings: Vec<EarningLine>,
    pub total_gross: Decimal,
    // Deductions
    pub deductions: Vec<DeductionLine>,
    pub total_deductions: Decimal,
    // Employer costs (not on payslip but needed for JE)
    pub employer_taxes: Vec<EmployerTaxLine>,
    pub total_employer_cost: Decimal,
    // Net
    pub net_pay: Decimal,
    pub payment_method: PaymentMethod,   // Reuse existing enum
    pub bank_account_last4: Option<String>,
    // YTD
    pub ytd_gross: Decimal,
    pub ytd_deductions: Decimal,
    pub ytd_net: Decimal,
    // Cost allocation
    pub cost_center: String,
    pub department_id: String,
}

pub struct EarningLine {
    pub earning_type: EarningType,
    pub hours: Option<Decimal>,
    pub rate: Option<Decimal>,
    pub amount: Decimal,
}

pub struct DeductionLine {
    pub deduction_type: DeductionType,
    pub amount: Decimal,
    pub is_pre_tax: bool,
    pub ytd_amount: Decimal,
}

pub struct EmployerTaxLine {
    pub tax_type: String,           // "FICA_ER", "FUTA", "SUTA", etc.
    pub amount: Decimal,
}
```

#### Time & Attendance — `datasynth-core/src/models/time_entry.rs`

```rust
pub enum TimeEntryType {
    Regular,
    Overtime,
    DoubleTime,
    OnCall,
    Training,
    Travel,
    Holiday,
    SickLeave,
    VacationLeave,
}

pub struct TimeEntry {
    pub entry_id: String,
    pub employee_id: String,
    pub date: NaiveDate,
    pub entry_type: TimeEntryType,
    pub hours: Decimal,
    pub cost_center: Option<String>,
    pub project_id: Option<String>,       // For project-based time
    pub wbs_element: Option<String>,      // Work Breakdown Structure
    pub approval_status: ApprovalStatus,  // Pending, Approved, Rejected
    pub approved_by: Option<String>,
    pub notes: Option<String>,
}
```

#### Expense Management — `datasynth-core/src/models/expense_report.rs`

```rust
pub enum ExpenseCategory {
    Travel,
    Meals,
    Lodging,
    Transportation,
    OfficeSupplies,
    ClientEntertainment,
    Training,
    Mileage,
    Telecommunications,
    Miscellaneous,
}

pub struct ExpenseLineItem {
    pub line_number: u32,
    pub date: NaiveDate,
    pub category: ExpenseCategory,
    pub description: String,
    pub amount: Decimal,
    pub currency: String,
    pub receipt_attached: bool,
    pub merchant_name: Option<String>,
    pub is_billable: bool,
    pub project_id: Option<String>,
    pub tax_amount: Option<Decimal>,
}

pub struct ExpenseReport {
    pub report_id: String,
    pub employee_id: String,
    pub company_code: String,
    pub title: String,
    pub submission_date: NaiveDate,
    pub period_from: NaiveDate,
    pub period_to: NaiveDate,
    pub status: ExpenseReportStatus,     // Draft, Submitted, Approved, Paid, Rejected
    pub items: Vec<ExpenseLineItem>,
    pub total_amount: Decimal,
    pub total_reimbursable: Decimal,
    pub advance_amount: Decimal,         // Cash advance already received
    pub amount_due_employee: Decimal,    // Total − Advance
    pub cost_center: String,
    pub approved_by: Option<String>,
    pub approval_date: Option<NaiveDate>,
    pub payment_date: Option<NaiveDate>,
    pub payment_id: Option<String>,      // FK to Payment
    pub journal_entry_id: Option<String>,
    pub policy_violations: Vec<String>,  // "Over per-diem", "Missing receipt", etc.
}
```

### 4.4 Generator Logic

**Payroll Generator** — `datasynth-generators/src/hr/payroll_generator.rs`

- Run monthly for each active employee
- Base salary derived from `JobLevel` (Staff: $50K-$70K, Senior: $70K-$100K, Manager: $100K-$150K, Director: $150K-$220K, VP: $220K-$350K, Executive: $350K-$500K)
- Tax rates: Federal ~22% effective, State ~5%, FICA 7.65% (employee + employer)
- Deductions: 60% have health insurance ($200-$600/mo), 45% have 401k (3-10% of gross)
- Generate journal entries:
  - DR Salary Expense (by cost center) — `6100`
  - DR Employer Payroll Tax Expense — `6150`
  - CR Payroll Payable — `2300`
  - CR Tax Withholding Payable — `2310`
  - CR Benefits Payable — `2320`

**Implementation hint:** Reuse the existing `EmployeePool` for iteration. The `Employee.cost_center` field drives cost allocation. JE generation follows the same pattern as `DocumentFlowJeGenerator`.

**Time Entry Generator** — `datasynth-generators/src/hr/time_generator.rs`

- Generate 8-hour regular entries for business days (use existing `BusinessDayCalculator`)
- Overtime: 5-15% of employees, 2-8 hours/week, correlated with `PeriodEndDynamics` (more OT near month-end)
- Leave: Use existing `leave_rate` config (default 5%)

**Expense Generator** — `datasynth-generators/src/hr/expense_generator.rs`

- 20-40% of employees submit expense reports per month
- Amount distribution: Log-normal (median $200, 95th percentile $2,000)
- Approval: Reports > $500 require manager approval, > $5,000 require director
- Policy violations: 5-10% of line items have issues (missing receipt, over per-diem)

### 4.5 Config Additions

```yaml
hr:
  enabled: false                         # Off by default
  payroll:
    enabled: true
    pay_frequency: monthly
    salary_ranges:
      staff: { min: 50000, max: 70000 }
      senior: { min: 70000, max: 100000 }
      manager: { min: 100000, max: 150000 }
      director: { min: 150000, max: 220000 }
    tax_rates:
      federal_effective: 0.22
      state_effective: 0.05
      fica_employee: 0.0765
      fica_employer: 0.0765
    benefits_enrollment_rate: 0.60
    retirement_participation_rate: 0.45
  time_attendance:
    enabled: true
    overtime_rate: 0.10
    avg_overtime_hours: 4.0
  expenses:
    enabled: true
    submission_rate: 0.30                # % of employees per month
    avg_report_amount: 500.0
    policy_violation_rate: 0.08
    receipt_missing_rate: 0.05
```

### 4.6 Coherence Rules

| Rule ID | Constraint |
|---------|-----------|
| H2R-001 | Every `PayslipEntry.employee_id` must reference an Active or OnLeave employee |
| H2R-002 | `PayrollRun.total_gross` = sum of `PayslipEntry.total_gross` for that run |
| H2R-003 | `PayslipEntry.net_pay` = `total_gross` − `total_deductions` |
| H2R-004 | `PayrollRun.journal_entry_id` JE must balance (DR Expense = CR Payable) |
| H2R-005 | No payslips for `Terminated` employees after `termination_date` |
| H2R-006 | `GhostEmployee` anomaly: payslips exist for employee with no time entries |
| H2R-007 | `ExpenseReport.approved_by` must be a manager at or above submitter's level |
| H2R-008 | `TimeEntry.hours` summed per employee per week ≤ 80 (sanity check) |

### 4.7 Anomalies

| Type | Category | Description |
|------|----------|-------------|
| `GhostEmployeePayroll` | Fraud | Payslips for terminated/non-existent employees (enhances existing `GhostEmployee`) |
| `PayrollInflation` | Fraud | Salary amount exceeds band for job level |
| `DuplicateExpenseReport` | Fraud | Same expenses submitted across multiple reports |
| `FictitiousExpense` | Fraud | Expense line items with no corresponding business justification |
| `SplitExpenseToAvoidApproval` | Fraud | Multiple reports just below approval threshold |
| `OvertimeAnomaly` | Statistical | Employee with consistent high overtime but no corresponding output |

### 4.8 Implementation Hints

- Create new module: `crates/datasynth-generators/src/hr/` with `mod.rs`, `payroll_generator.rs`, `time_generator.rs`, `expense_generator.rs`
- Models go into `crates/datasynth-core/src/models/` (payroll.rs, time_entry.rs, expense_report.rs)
- Orchestration: H2R generators run after master data (need employee pool) but before period close (payroll JEs feed into trial balance)
- Ghost employee enhancement: Cross-reference payslip employees against `EmployeePool`; anomaly = payslip with no matching active employee
- Output: `payroll_runs.csv`, `payslips.csv`, `time_entries.csv`, `expense_reports.csv`, `expense_line_items.csv`

---

## 5. Acquire-to-Retire (A2R) — Closing the Last 30%

### 5.1 What Exists

`FixedAssetRecord` (12 classes), 7 depreciation methods (incl. MACRS with IRS tables), `DepreciationRunGenerator`, `AssetDisposal` (4 types, gain/loss calculation), `AssetAccountDetermination`. Standards module has `ImpairmentTest` (ASC 360/IAS 36) with full model. OCPM has `run_depr` and `impair_test` activities.

### 5.2 What's Missing

| Gap | Impact |
|-----|--------|
| **Impairment Generator** | Model exists, no generator wires it to assets |
| **Capitalization Threshold** | No policy enforcement for capitalize vs. expense |
| **CIP Capitalization** | Asset class exists, no WIP cost accumulation workflow |
| **Asset-PO Link** | `vendor_id` and `po_reference` exist but aren't coherently populated from P2P |

### 5.3 New Models

No new models needed — the existing `ImpairmentTest` in `datasynth-standards` and `FixedAssetRecord` are sufficient. Only enhancements:

```rust
// Add to FixedAssetRecord:
pub capitalization_threshold_applied: bool,  // Was threshold rule used?
pub original_expense_je_id: Option<String>,  // If below threshold, expensed instead

// Add to CIP tracking (could be a new lightweight struct):
pub struct CipCostAccumulation {
    pub asset_number: String,               // CIP asset
    pub cost_entry_id: String,
    pub posting_date: NaiveDate,
    pub vendor_id: Option<String>,
    pub po_reference: Option<String>,
    pub cost_type: CipCostType,             // Material, Labor, Overhead, Service
    pub amount: Decimal,
    pub description: String,
    pub journal_entry_id: String,
}

pub enum CipCostType {
    Material,
    Labor,
    Overhead,
    ExternalService,
    InternalLabor,
}
```

### 5.4 Generator Logic

**Impairment Generator** — `datasynth-generators/src/subledger/impairment_generator.rs`

- Run annually (or when indicators present)
- Select 5-15% of assets for testing (configurable)
- Impairment indicators: asset age > 70% of useful life, asset class = Software or IT (technology risk), random market-driven triggers
- `recoverable_amount` = max(`fair_value_less_costs`, `value_in_use`)
- If `carrying_amount` > `recoverable_amount` → impairment loss
- Generate JE: DR Impairment Loss (expense), CR Accumulated Depreciation (or Asset directly)
- Update `FixedAssetRecord.net_book_value`

**Implementation hint:** The `ImpairmentTest` struct in `datasynth-standards/src/accounting/impairment.rs` has all fields. The generator populates it, then generates a JE using the same pattern as `DepreciationRunGenerator`.

**Capitalization Threshold** — Add to `FixedAssetGenerator`

- Config parameter: `capitalization_threshold` (default: $5,000)
- During asset creation from P2P (capital PO): if unit price < threshold, generate expense JE instead of capitalization JE
- Set `capitalization_threshold_applied = true` on the record

**CIP Flow** — `datasynth-generators/src/subledger/cip_generator.rs`

- For `AssetClass::ConstructionInProgress` assets: accumulate costs over 3-18 months
- On completion: reclassify to target asset class, begin depreciation
- Generate JE chain: DR CIP (accumulation) → DR Target Asset, CR CIP (capitalization)

### 5.5 Config Additions

```yaml
master_data:
  fixed_assets:
    # Existing fields...
    capitalization_threshold: 5000.0
    impairment:
      enabled: true
      annual_test_rate: 0.10          # % of assets tested annually
      impairment_rate: 0.03           # % of tested assets actually impaired
      technology_risk_multiplier: 2.0 # IT/Software assets tested more often
    cip:
      enabled: true
      cip_percent: 0.05              # % of assets are CIP
      avg_construction_months: 9
      cost_entries_per_month: 3
```

### 5.6 Coherence Rules

| Rule ID | Constraint |
|---------|-----------|
| A2R-001 | `ImpairmentTest.carrying_amount` must match `FixedAssetRecord.net_book_value` at test date |
| A2R-002 | `ImpairmentTest.impairment_loss` = `carrying_amount` − `recoverable_amount` (when positive) |
| A2R-003 | CIP assets must not have depreciation entries until capitalization |
| A2R-004 | Asset `po_reference` (when set) must reference a valid PO with asset-relevant line items |
| A2R-005 | Sum of `CipCostAccumulation.amount` for an asset must equal its `acquisition_cost` at capitalization |

---

## 6. Inventory Management — Closing the Last 45%

### 6.1 What Exists

`Material` (8 types, BOM, ABC classification, 5 valuation methods), `InventoryPosition` (with `last_count_date`, `safety_stock`, `reorder_point`, `BatchStock`, `SerialNumber`), `InventoryMovement` (23 movement types incl. `PhysicalInventory`, `InventoryAdjustmentIn/Out`), `InventoryGenerator` with GR/GI. `StockStatus` already has `Obsolete` variant.

### 6.2 What's Missing

| Gap | Impact |
|-----|--------|
| **Cycle Counting** | No count records despite `last_count_date` and `PhysicalInventory` movement type existing |
| **Quality Inspection** | No QA records despite `StockType::QualityInspection` and `TransferToInspection` movement existing |
| **Obsolescence / Write-down** | `StockStatus::Obsolete` exists but no aging analysis or write-down proposals |

### 6.3 New Models

#### Cycle Count — `datasynth-core/src/models/subledger/inventory/cycle_count.rs`

```rust
pub struct CycleCountPlan {
    pub plan_id: String,
    pub company_code: String,
    pub plant: String,
    pub fiscal_year: u16,
    pub frequency_a_items: u8,           // Counts per year for A items (e.g., 4)
    pub frequency_b_items: u8,           // Counts per year for B items (e.g., 2)
    pub frequency_c_items: u8,           // Counts per year for C items (e.g., 1)
    pub scheduled_counts: Vec<ScheduledCount>,
}

pub struct CycleCountRecord {
    pub count_id: String,
    pub plan_id: Option<String>,
    pub material_id: String,
    pub plant: String,
    pub storage_location: String,
    pub count_date: NaiveDate,
    pub counted_by: String,              // employee_id
    pub book_quantity: Decimal,
    pub counted_quantity: Decimal,
    pub variance_quantity: Decimal,       // counted − book
    pub variance_value: Decimal,
    pub variance_percent: f64,
    pub unit: String,
    pub batch_number: Option<String>,
    pub status: CountStatus,             // Planned, Counted, Reviewed, Posted, Recounted
    pub recount_required: bool,          // If variance > threshold
    pub adjustment_movement_id: Option<String>,  // FK to InventoryMovement
    pub approved_by: Option<String>,
    pub notes: Option<String>,
}
```

#### Quality Inspection — `datasynth-core/src/models/subledger/inventory/inspection.rs`

```rust
pub enum InspectionTrigger {
    GoodsReceipt,         // Triggered by PO GR
    Production,           // Triggered by production completion
    ReturnFromCustomer,   // Triggered by sales return
    Periodic,             // Routine quality check
}

pub enum InspectionResult {
    Accepted,
    AcceptedWithDeviation,
    Rejected,
    UsageDecisionPending,
}

pub struct InspectionCharacteristic {
    pub name: String,                    // "Dimension", "Weight", "Color", "Hardness"
    pub target_value: Option<Decimal>,
    pub lower_limit: Option<Decimal>,
    pub upper_limit: Option<Decimal>,
    pub measured_value: Option<Decimal>,
    pub is_qualitative: bool,            // true = visual/subjective
    pub qualitative_result: Option<String>, // "OK", "Minor defect", etc.
    pub passed: bool,
}

pub struct QualityInspectionLot {
    pub lot_id: String,
    pub material_id: String,
    pub plant: String,
    pub trigger: InspectionTrigger,
    pub source_document_id: String,      // GR ID, Production Order ID, etc.
    pub vendor_id: Option<String>,
    pub batch_number: Option<String>,
    pub inspection_date: NaiveDate,
    pub inspector_id: String,
    pub sample_size: Decimal,
    pub lot_size: Decimal,
    pub characteristics: Vec<InspectionCharacteristic>,
    pub result: InspectionResult,
    pub defect_count: u32,
    pub defect_rate: f64,
    pub usage_decision: UsageDecision,   // Accept, Rework, Scrap, ReturnToVendor
    pub movement_id: Option<String>,     // Resulting stock movement
    pub notes: Option<String>,
}

pub enum UsageDecision {
    AcceptToUnrestricted,
    AcceptToBlocked,
    Rework,
    Scrap,
    ReturnToVendor,
}
```

### 6.4 Generator Logic

**Cycle Count Generator** — `datasynth-generators/src/subledger/cycle_count_generator.rs`

- Generate count plan based on ABC classification (A: 4×/yr, B: 2×/yr, C: 1×/yr)
- Count accuracy: 95-98% of counts match book quantity
- Variances: Log-normal distribution, most small (1-3%), few large (>10%)
- Auto-generate `InventoryAdjustmentIn/Out` movements for posted counts
- Update `InventoryPosition.last_count_date`

**Implementation hint:** The `InventoryMovement` already has `PhysicalInventory`, `InventoryAdjustmentIn`, `InventoryAdjustmentOut` movement types. The generator creates a `CycleCountRecord` and then calls existing `InventoryGenerator` methods to create the adjustment movement.

**Quality Inspection Generator** — `datasynth-generators/src/subledger/inspection_generator.rs`

- Triggered by GR: configurable `inspection_rate` (default: 20% of GRs)
- Inspection pass rate varies by `VendorCluster`: ReliableStrategic 99%, StandardOperational 95%, Transactional 90%, Problematic 80%
- Failed inspection → `ReturnToVendor` or `Scrap` movement (feeds back to vendor quality score)
- Use existing `TransferToInspection`/`TransferFromInspection` movement types

**Obsolescence Generator** — `datasynth-generators/src/subledger/obsolescence_generator.rs`

- Monthly: flag materials with no movement in > 180 days as slow-moving
- > 365 days with no movement → propose write-down (50% of value)
- > 730 days → propose full write-off
- Generate JE: DR Inventory Write-Down Expense, CR Inventory Valuation Allowance

### 6.5 Config Additions

```yaml
subledger:
  inventory:
    cycle_counting:
      enabled: true
      frequency_a: 4               # Counts/year for A items
      frequency_b: 2
      frequency_c: 1
      accuracy_rate: 0.96
      variance_threshold: 0.05    # >5% triggers recount
    quality_inspection:
      enabled: true
      inspection_rate: 0.20       # % of GRs inspected
      defect_rate_by_cluster:
        reliable_strategic: 0.01
        standard_operational: 0.05
        transactional: 0.10
        problematic: 0.20
    obsolescence:
      enabled: true
      slow_moving_days: 180
      write_down_days: 365
      write_off_days: 730
      write_down_percent: 0.50
```

### 6.6 Coherence Rules

| Rule ID | Constraint |
|---------|-----------|
| INV-001 | `CycleCountRecord.book_quantity` must match `InventoryPosition.quantity_on_hand` at count date |
| INV-002 | Count adjustment movements must reconcile with variance quantity |
| INV-003 | Inspection lot `source_document_id` must reference a valid GR or production completion |
| INV-004 | Failed inspection with `ReturnToVendor` must have corresponding `ReturnToVendor` movement |
| INV-005 | Obsolete items must have no movements within configured `slow_moving_days` window |

---

## 7. Banking / Treasury — Closing the Last 35%

### 7.1 What Exists

`BankAccount` (full IBAN/SWIFT, balances, features), `BankTransaction` (10+ channels, FX, counterparty, AML ground truth), KYC profiles (risk tiers, PEP checks), 6 AML typologies, 8+ fraud types.

### 7.2 What's Missing — Bank Reconciliation (Priority Gap)

| Gap | Impact |
|-----|--------|
| **Bank Reconciliation** | No matching of bank statement lines to AP/AR ledger entries |
| **Cash Forecasting** | No forward-looking cash position |
| **FX Contract Management** | FX rates exist but no hedge accounting |

### 7.3 New Models

#### Bank Reconciliation — `datasynth-core/src/models/bank_reconciliation.rs`

```rust
pub enum ReconciliationStatus {
    Open,
    InProgress,
    Reconciled,
    ReconciledException,  // Reconciled with exceptions
    Closed,
}

pub enum MatchStatus {
    Matched,               // 1:1 match
    PartialMatch,          // Amount mismatch within tolerance
    MultiMatch,            // 1:N or N:1 match
    Unmatched,             // No match found
    ManualMatch,           // Manually matched by user
}

pub struct BankStatementLine {
    pub line_id: String,
    pub statement_id: String,
    pub value_date: NaiveDate,
    pub posting_date: NaiveDate,
    pub amount: Decimal,
    pub currency: String,
    pub direction: Direction,            // Inbound / Outbound
    pub reference: String,               // Bank reference (check #, wire ref)
    pub counterparty_name: Option<String>,
    pub counterparty_account: Option<String>,
    pub transaction_code: String,        // Bank transaction type code
    pub description: String,
    pub match_status: MatchStatus,
    pub matched_ledger_ids: Vec<String>, // FK to Payment.id or GL entry
    pub match_difference: Option<Decimal>,
    pub match_date: Option<NaiveDate>,
    pub matched_by: Option<String>,      // Auto or employee_id
}

pub struct BankReconciliation {
    pub reconciliation_id: String,
    pub company_code: String,
    pub bank_account_id: String,
    pub statement_date: NaiveDate,
    pub period_from: NaiveDate,
    pub period_to: NaiveDate,
    pub status: ReconciliationStatus,
    pub statement_opening_balance: Decimal,
    pub statement_closing_balance: Decimal,
    pub ledger_opening_balance: Decimal,
    pub ledger_closing_balance: Decimal,
    pub total_matched: u32,
    pub total_unmatched_bank: u32,       // Bank items not in ledger (timing)
    pub total_unmatched_ledger: u32,     // Ledger items not in bank (outstanding checks)
    pub reconciling_items: Vec<ReconcilingItem>,
    pub net_difference: Decimal,         // Should be zero after reconciliation
    pub prepared_by: String,
    pub reviewed_by: Option<String>,
    pub review_date: Option<NaiveDate>,
}

pub struct ReconcilingItem {
    pub item_type: ReconcilingItemType,
    pub description: String,
    pub amount: Decimal,
    pub expected_clear_date: Option<NaiveDate>,
    pub reference: String,
}

pub enum ReconcilingItemType {
    OutstandingCheck,       // Check issued, not yet cashed
    DepositInTransit,       // Deposit made, not yet credited by bank
    BankCharge,             // Bank fee not yet recorded in books
    InterestEarned,         // Interest not yet recorded
    NsfCheck,               // Check returned — need reversal
    Error,                  // Bank or book error
}
```

### 7.4 Generator Logic

**Bank Reconciliation Generator** — `datasynth-generators/src/banking/reconciliation_generator.rs` (or in `datasynth-banking`)

- For each `Payment` (AP payment or AR receipt), create a matching `BankStatementLine`
- Auto-match rate: 85-95% (most match by amount + date + reference)
- Timing differences: 5-10% of items are outstanding (check not yet cashed, deposit in transit)
- Generate reconciling items for:
  - Outstanding checks: payments posted but not yet cleared (3-15 days lag)
  - Deposits in transit: AR receipts posted but not credited by bank (1-3 days lag)
  - Bank charges: monthly fees, wire fees (generate from bank features config)
- `net_difference` should be zero when all timing items are accounted for

**Implementation hint:** The existing `Payment` model has `status` (Pending, Approved, Sent, Cleared). The reconciliation generator creates statement lines for `Cleared` payments and tracks `Sent` ones as outstanding. Match on `Payment.id` ↔ `BankStatementLine.matched_ledger_ids`.

### 7.5 Config Additions

```yaml
banking:
  # Existing fields...
  reconciliation:
    enabled: true
    auto_match_rate: 0.90
    outstanding_check_rate: 0.08
    deposit_in_transit_rate: 0.05
    avg_clearing_days: 3
    bank_fee_per_statement: 25.0
    tolerance_amount: 0.50            # Match tolerance in currency units
```

### 7.6 Coherence Rules

| Rule ID | Constraint |
|---------|-----------|
| BANK-001 | Every `Cleared` Payment must have a matching `BankStatementLine` |
| BANK-002 | `Reconciliation.net_difference` = 0 after all reconciling items |
| BANK-003 | `statement_closing_balance` = `statement_opening_balance` + sum(statement lines) |
| BANK-004 | Outstanding checks: `BankStatementLine` with `Unmatched` status must clear within 90 days |

---

## 8. Plan-to-Produce (Manufacturing) — From 20% to Viable

### 8.1 What Exists

`Material` with BOM (multi-level, scrap, yield), `ManufacturingSettings` (BOM depth, JIT, quality framework), `BomComponent` with effective quantity calculation, industry-specific master data generation.

### 8.2 What's Missing

| Gap | Impact |
|-----|--------|
| **Production Order** | No work orders, no routing, no operation sequences |
| **WIP Costing** | No work-in-progress accounting |
| **Variance Analysis** | No material/labor/overhead variance |
| **Standard Cost Rollup** | `calculate_bom_cost()` exists but no periodic update |

### 8.3 New Models

#### Production Order — `datasynth-core/src/models/production_order.rs`

```rust
pub enum ProductionOrderType {
    Standard,
    Rework,
    Prototype,
    DiscreteManufacturing,
    ProcessManufacturing,
}

pub enum ProductionOrderStatus {
    Planned,
    Released,
    InProgress,
    PartiallyComplete,
    Completed,
    TechnicallyComplete,   // Production done, costs still pending
    Closed,                 // Variance settled
    Cancelled,
}

pub struct RoutingOperation {
    pub operation_number: u32,          // 0010, 0020, 0030
    pub work_center_id: String,
    pub description: String,
    pub setup_time_hours: Decimal,
    pub run_time_per_unit_hours: Decimal,
    pub machine_time_hours: Decimal,
    pub labor_count: u8,                // Number of workers
    pub status: OperationStatus,        // NotStarted, InProgress, Completed
    pub actual_start: Option<NaiveDate>,
    pub actual_end: Option<NaiveDate>,
    pub actual_labor_hours: Option<Decimal>,
    pub actual_machine_hours: Option<Decimal>,
    pub scrap_quantity: Decimal,
    pub yield_quantity: Decimal,
}

pub struct ProductionOrder {
    pub order_id: String,
    pub order_type: ProductionOrderType,
    pub status: ProductionOrderStatus,
    pub material_id: String,            // Finished good
    pub plant: String,
    pub planned_quantity: Decimal,
    pub actual_quantity: Decimal,        // Quantity completed
    pub scrap_quantity: Decimal,
    pub unit: String,
    pub bom_id: String,
    pub routing: Vec<RoutingOperation>,
    pub planned_start_date: NaiveDate,
    pub planned_end_date: NaiveDate,
    pub actual_start_date: Option<NaiveDate>,
    pub actual_end_date: Option<NaiveDate>,
    // Costing
    pub planned_material_cost: Decimal,
    pub planned_labor_cost: Decimal,
    pub planned_overhead_cost: Decimal,
    pub planned_total_cost: Decimal,
    pub actual_material_cost: Decimal,
    pub actual_labor_cost: Decimal,
    pub actual_overhead_cost: Decimal,
    pub actual_total_cost: Decimal,
    // Variances (populated at settlement)
    pub material_variance: Decimal,      // Actual − Standard
    pub labor_rate_variance: Decimal,
    pub labor_efficiency_variance: Decimal,
    pub overhead_variance: Decimal,
    pub total_variance: Decimal,
    // References
    pub cost_center: String,
    pub sales_order_id: Option<String>,  // Make-to-order
    pub journal_entry_ids: Vec<String>,
    pub goods_receipt_id: Option<String>, // GR for finished good
    pub component_issues: Vec<ComponentIssue>,
}

pub struct ComponentIssue {
    pub material_id: String,
    pub planned_quantity: Decimal,
    pub actual_quantity: Decimal,
    pub movement_id: String,             // FK to InventoryMovement (GoodsIssueProduction)
    pub variance_quantity: Decimal,
    pub variance_value: Decimal,
}
```

### 8.4 Generator Logic

**Production Order Generator** — `datasynth-generators/src/manufacturing/production_order_generator.rs`

- Generate production orders for `FinishedGood` and `SemiFinished` materials
- Explode BOM to determine component requirements
- Component issues: create `GoodsIssueProduction` inventory movements
- Finished good receipt: create `GoodsReceiptProduction` inventory movement
- JE chain:
  1. Component issue: DR WIP Material (`1400`), CR Raw Material Inventory (`1300`)
  2. Labor posting: DR WIP Labor (`1410`), CR Payroll Accrual (`2300`)
  3. Overhead: DR WIP Overhead (`1420`), CR Overhead Applied (`5500`)
  4. GR finished good: DR Finished Goods Inventory (`1200`), CR WIP (`1400+1410+1420`)
  5. Variance settlement: DR/CR Production Variance accounts
- Yield rate: use `ManufacturingSettings.target_yield_rate` (default 0.97)
- Scrap: (1 − yield) × planned quantity

**Implementation hint:** Reuse existing `InventoryGenerator.generate_goods_receipt()` and `generate_goods_issue()` for stock movements. The BOM explosion uses `Material.bom_components` which already has `effective_quantity()` with scrap. Production orders connect S2P to manufacturing: raw material POs feed inventory → production order issues → finished good receipts.

### 8.5 Config Additions

```yaml
manufacturing:
  enabled: false                          # Off by default
  production_orders:
    enabled: true
    orders_per_month: 50
    avg_batch_size: 100
    yield_rate: 0.97
    make_to_order_rate: 0.20             # % of orders linked to sales order
    rework_rate: 0.03
  costing:
    labor_rate_per_hour: 35.00
    overhead_rate: 1.50                   # Multiplier on direct labor
    standard_cost_update_frequency: quarterly
  routing:
    avg_operations: 4
    setup_time_hours: 1.5
    run_time_variation: 0.15             # ±15% of planned run time
```

### 8.6 Coherence Rules

| Rule ID | Constraint |
|---------|-----------|
| MFG-001 | Sum of `ComponentIssue.actual_quantity` must support `actual_quantity` × BOM per-unit requirements (±scrap) |
| MFG-002 | `actual_material_cost` = sum of component issue values |
| MFG-003 | `material_variance` = `actual_material_cost` − (`planned_material_cost` × actual_qty / planned_qty) |
| MFG-004 | Production GR quantity must match `actual_quantity` |
| MFG-005 | Finished good GR creates stock; verify `InventoryPosition` updated |

---

## 9. Cross-Process Integration Points

These are places where closing one gap enables coherence with another chain:

| Integration | From Chain | To Chain | Link Mechanism |
|-------------|-----------|----------|----------------|
| Raw material demand | MFG | S2P | Production order BOM → Purchase Requisition |
| Asset acquisition | S2P | A2R | Capital PO → GR → Fixed Asset Record |
| Payroll JEs | H2R | R2R | PayrollRun → JournalEntry → Trial Balance |
| Payment clearing | S2P / O2C | BANK | Payment.status=Cleared → BankStatementLine |
| Inventory bridge | S2P | O2C | GR increases stock; Delivery decreases stock |
| Production costing | MFG | R2R | WIP JEs → Trial Balance → Income Statement (COGS) |
| Quality → Scorecard | INV | S2P | Inspection failure → Vendor quality score → Supplier scorecard |
| Expense reimburse | H2R | S2P | Expense payment → AP Payment → Bank |
| Revenue recognition | O2C | R2R | CustomerContract → Revenue JEs → Income Statement |
| Impairment | A2R | R2R | Impairment loss JE → Trial Balance → Balance Sheet |

---

## 10. Recommended Implementation Order

Considering dependencies and the interconnection map above:

```
Wave 1 (Foundation — enables everything else):
  ├─ S2C completion (separate spec)
  ├─ Bank Reconciliation (validates all payment chains)
  └─ Financial Statement Generator (consumes all JEs)

Wave 2 (Core process chains):
  ├─ Payroll + Time (H2R core, enables SoD analysis)
  ├─ Revenue Recognition Generator (wires existing models)
  └─ Impairment Generator (wires existing models)

Wave 3 (Operational depth):
  ├─ Production Orders + WIP Costing (MFG core)
  ├─ Cycle Counting + Quality Inspection (INV completeness)
  └─ Expense Management (H2R extension)

Wave 4 (Polish):
  ├─ Sales Quote model (O2C upstream)
  ├─ Cash Forecasting (Treasury)
  ├─ Management KPIs + Budget Variance (R2R reporting)
  └─ Obsolescence Management (INV)
```

Each wave is independently deployable. Within a wave, items can be parallelized.
