# Domain Models

Core data structures representing enterprise financial concepts.

## Model Categories

| Category | Models |
|----------|--------|
| [Accounting](#accounting) | JournalEntry, ChartOfAccounts, ACDOCA |
| [Master Data](#master-data) | Vendor, Customer, Material, FixedAsset, Employee |
| [Documents](#documents) | PurchaseOrder, Invoice, Payment, etc. |
| [Financial](#financial) | TrialBalance, FxRate, AccountBalance |
| [Financial Reporting](#financial-reporting) | FinancialStatement, CashFlowItem, BankReconciliation, BankStatementLine |
| [Sourcing (S2C)](#sourcing-s2c) | SourcingProject, SupplierQualification, RfxEvent, Bid, BidEvaluation, ProcurementContract, CatalogItem, SupplierScorecard, SpendAnalysis |
| [HR / Payroll](#hr--payroll) | PayrollRun, PayrollLineItem, TimeEntry, ExpenseReport, ExpenseLineItem, BenefitEnrollment |
| [Manufacturing](#manufacturing) | ProductionOrder, QualityInspection, CycleCount, BomComponent, InventoryMovement |
| [Sales Quotes](#sales-quotes) | SalesQuote, QuoteLineItem |
| [Tax](#tax) | TaxJurisdiction, TaxCode, TaxLine, TaxReturn, TaxProvision, WithholdingTaxRecord, UncertainTaxPosition |
| [Treasury](#treasury) | CashPosition, CashForecast, CashPool, CashPoolSweep, HedgingInstrument, HedgeRelationship, DebtInstrument, DebtCovenant |
| [ESG](#esg) | EmissionRecord, EnergyConsumption, WaterUsage, WasteRecord, WorkforceDiversityMetric, PayEquityMetric, SafetyIncident, SafetyMetric, GovernanceMetric, SupplierEsgAssessment, MaterialityAssessment, EsgDisclosure, ClimateScenario |
| [Project Accounting](#project-accounting) | Project, ProjectCostLine, ProjectRevenue, EarnedValueMetric, ChangeOrder, ProjectMilestone |
| [Compliance](#compliance) | InternalControl, SoDRule, LabeledAnomaly |
| [Graph Properties](#graph-properties) | ToNodeProperties, GraphPropertyValue, GraphEntityType (51 types), RelationshipType (28 edge types), EdgeConstraint |

---

## Accounting

### JournalEntry

The core accounting record.

```rust
pub struct JournalEntry {
    pub header: JournalEntryHeader,
    pub lines: Vec<JournalEntryLine>,
}

pub struct JournalEntryHeader {
    pub document_id: Uuid,
    pub company_code: String,
    pub fiscal_year: u16,
    pub fiscal_period: u8,
    pub posting_date: NaiveDate,
    pub document_date: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub source: TransactionSource,
    pub business_process: Option<BusinessProcess>,

    // Document references
    pub source_document_type: Option<DocumentType>,
    pub source_document_id: Option<String>,

    // Labels
    pub is_fraud: bool,
    pub fraud_type: Option<FraudType>,
    pub is_anomaly: bool,
    pub anomaly_type: Option<AnomalyType>,

    // Control markers
    pub control_ids: Vec<String>,
    pub sox_relevant: bool,
    pub sod_violation: bool,
}

pub struct JournalEntryLine {
    pub line_number: u32,
    pub account_number: String,
    pub cost_center: Option<String>,
    pub profit_center: Option<String>,
    pub debit_amount: Decimal,
    pub credit_amount: Decimal,
    pub description: String,
    pub tax_code: Option<String>,
}
```

**Invariant:** Sum of debits must equal sum of credits.

### ChartOfAccounts

GL account structure.

```rust
pub struct ChartOfAccounts {
    pub accounts: Vec<Account>,
}

pub struct Account {
    pub account_number: String,
    pub name: String,
    pub account_type: AccountType,
    pub account_subtype: AccountSubType,
    pub is_control_account: bool,
    pub normal_balance: NormalBalance,
    pub is_active: bool,
}

pub enum AccountType {
    Asset,
    Liability,
    Equity,
    Revenue,
    Expense,
}

pub enum AccountSubType {
    // Assets
    Cash, AccountsReceivable, Inventory, FixedAsset,
    // Liabilities
    AccountsPayable, AccruedLiabilities, LongTermDebt,
    // Equity
    CommonStock, RetainedEarnings,
    // Revenue
    SalesRevenue, ServiceRevenue,
    // Expense
    CostOfGoodsSold, OperatingExpense,
    // ...
}
```

### ACDOCA

SAP HANA Universal Journal format.

```rust
pub struct AcdocaEntry {
    pub rclnt: String,           // Client
    pub rldnr: String,           // Ledger
    pub rbukrs: String,          // Company code
    pub gjahr: u16,              // Fiscal year
    pub belnr: String,           // Document number
    pub docln: u32,              // Line item
    pub ryear: u16,              // Year
    pub poper: u8,               // Posting period
    pub racct: String,           // Account
    pub drcrk: DebitCreditIndicator,
    pub hsl: Decimal,            // Amount in local currency
    pub ksl: Decimal,            // Amount in group currency

    // Simulation fields
    pub zsim_fraud: bool,
    pub zsim_anomaly: bool,
    pub zsim_source: String,
}
```

---

## Master Data

### Vendor

Supplier master record.

```rust
pub struct Vendor {
    pub vendor_id: String,
    pub vendor_name: String,
    pub tax_id: Option<String>,
    pub currency: String,
    pub country: String,
    pub payment_terms: PaymentTerms,
    pub bank_account: Option<BankAccount>,
    pub is_intercompany: bool,
    pub behavior: VendorBehavior,
    pub valid_from: NaiveDate,
    pub valid_to: Option<NaiveDate>,
}

pub struct VendorBehavior {
    pub late_payment_tendency: f64,
    pub discount_usage_rate: f64,
}
```

### Customer

Customer master record.

```rust
pub struct Customer {
    pub customer_id: String,
    pub customer_name: String,
    pub currency: String,
    pub country: String,
    pub credit_limit: Decimal,
    pub credit_rating: CreditRating,
    pub payment_behavior: PaymentBehavior,
    pub is_intercompany: bool,
    pub valid_from: NaiveDate,
}

pub struct PaymentBehavior {
    pub on_time_rate: f64,
    pub early_payment_rate: f64,
    pub late_payment_rate: f64,
    pub average_days_late: u32,
}
```

### Material

Product/material master.

```rust
pub struct Material {
    pub material_id: String,
    pub description: String,
    pub material_type: MaterialType,
    pub unit_of_measure: String,
    pub valuation_method: ValuationMethod,
    pub standard_cost: Decimal,
    pub gl_account: String,
}

pub enum MaterialType {
    RawMaterial,
    WorkInProgress,
    FinishedGoods,
    Service,
}

pub enum ValuationMethod {
    Fifo,
    Lifo,
    WeightedAverage,
    StandardCost,
}
```

### FixedAsset

Capital asset record.

```rust
pub struct FixedAsset {
    pub asset_id: String,
    pub description: String,
    pub asset_class: AssetClass,
    pub acquisition_date: NaiveDate,
    pub acquisition_cost: Decimal,
    pub useful_life_years: u32,
    pub depreciation_method: DepreciationMethod,
    pub salvage_value: Decimal,
    pub accumulated_depreciation: Decimal,
    pub disposal_date: Option<NaiveDate>,
}
```

### Employee

User/employee record.

```rust
pub struct Employee {
    pub employee_id: String,
    pub name: String,
    pub department: String,
    pub role: String,
    pub manager_id: Option<String>,
    pub approval_limit: Decimal,
    pub transaction_codes: Vec<String>,
    pub hire_date: NaiveDate,
}
```

---

## Documents

### PurchaseOrder

P2P initiating document.

```rust
pub struct PurchaseOrder {
    pub po_number: String,
    pub vendor_id: String,
    pub company_code: String,
    pub order_date: NaiveDate,
    pub items: Vec<PoLineItem>,
    pub total_amount: Decimal,
    pub currency: String,
    pub status: PoStatus,
}

pub struct PoLineItem {
    pub line_number: u32,
    pub material_id: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub gl_account: String,
}
```

### VendorInvoice

AP invoice with three-way match.

```rust
pub struct VendorInvoice {
    pub invoice_number: String,
    pub vendor_id: String,
    pub po_number: Option<String>,
    pub gr_number: Option<String>,
    pub invoice_date: NaiveDate,
    pub due_date: NaiveDate,
    pub total_amount: Decimal,
    pub match_status: MatchStatus,
}

pub enum MatchStatus {
    Matched,
    QuantityVariance,
    PriceVariance,
    Blocked,
}
```

### DocumentReference

Links documents in flows.

```rust
pub struct DocumentReference {
    pub from_document_type: DocumentType,
    pub from_document_id: String,
    pub to_document_type: DocumentType,
    pub to_document_id: String,
    pub reference_type: ReferenceType,
}

pub enum ReferenceType {
    FollowsFrom,     // Normal flow
    PaymentFor,      // Payment → Invoice
    ReversalOf,      // Reversal/credit memo
}
```

---

## Financial

### TrialBalance

Period-end balances.

```rust
pub struct TrialBalance {
    pub company_code: String,
    pub fiscal_year: u16,
    pub fiscal_period: u8,
    pub accounts: Vec<TrialBalanceRow>,
}

pub struct TrialBalanceRow {
    pub account_number: String,
    pub account_name: String,
    pub opening_balance: Decimal,
    pub period_debits: Decimal,
    pub period_credits: Decimal,
    pub closing_balance: Decimal,
}
```

### FxRate

Exchange rate record.

```rust
pub struct FxRate {
    pub from_currency: String,
    pub to_currency: String,
    pub rate_date: NaiveDate,
    pub rate_type: RateType,
    pub rate: Decimal,
}

pub enum RateType {
    Spot,
    Closing,
    Average,
}
```

---

## Compliance

### LabeledAnomaly

ML training label.

```rust
pub struct LabeledAnomaly {
    pub document_id: Uuid,
    pub anomaly_id: String,
    pub anomaly_type: AnomalyType,
    pub category: AnomalyCategory,
    pub severity: Severity,
    pub description: String,
    pub detection_difficulty: DetectionDifficulty,
}

pub enum AnomalyType {
    Fraud,
    Error,
    ProcessIssue,
    Statistical,
    Relational,
}
```

### InternalControl

SOX control definition.

```rust
pub struct InternalControl {
    pub control_id: String,
    pub name: String,
    pub description: String,
    pub control_type: ControlType,
    pub frequency: ControlFrequency,
    pub assertions: Vec<Assertion>,
}
```

---

## Financial Reporting

### FinancialStatement

Period-end financial statement with line items.

```rust
pub enum StatementType {
    BalanceSheet,
    IncomeStatement,
    CashFlowStatement,
    ChangesInEquity,
}

pub struct FinancialStatementLineItem {
    pub line_code: String,
    pub label: String,
    pub section: String,
    pub sort_order: u32,
    pub amount: Decimal,
    pub amount_prior: Option<Decimal>,
    pub indent_level: u8,
    pub is_total: bool,
    pub gl_accounts: Vec<String>,
}

pub struct CashFlowItem {
    pub item_code: String,
    pub label: String,
    pub category: CashFlowCategory,  // Operating, Investing, Financing
    pub amount: Decimal,
}
```

### BankReconciliation

Bank statement reconciliation with auto-matching.

```rust
pub struct BankStatementLine {
    pub line_id: String,
    pub statement_date: NaiveDate,
    pub direction: Direction,         // Inflow, Outflow
    pub amount: Decimal,
    pub description: String,
    pub match_status: MatchStatus,    // Unmatched, AutoMatched, ManuallyMatched, BankCharge, Interest
    pub matched_payment_id: Option<String>,
}

pub struct BankReconciliation {
    pub reconciliation_id: String,
    pub company_code: String,
    pub bank_account: String,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub opening_balance: Decimal,
    pub closing_balance: Decimal,
    pub status: ReconciliationStatus, // InProgress, Completed, CompletedWithExceptions
}
```

---

## Sourcing (S2C)

Source-to-Contract models for the procurement pipeline.

### SourcingProject

Top-level sourcing initiative.

```rust
pub struct SourcingProject {
    pub project_id: String,
    pub title: String,
    pub category: String,
    pub status: SourcingProjectStatus,
    pub estimated_spend: Decimal,
    pub start_date: NaiveDate,
    pub target_award_date: NaiveDate,
}
```

### RfxEvent

Request for Information/Proposal/Quote.

```rust
pub struct RfxEvent {
    pub rfx_id: String,
    pub project_id: String,
    pub rfx_type: RfxType,       // Rfi, Rfp, Rfq
    pub title: String,
    pub issue_date: NaiveDate,
    pub close_date: NaiveDate,
    pub invited_suppliers: Vec<String>,
}
```

### ProcurementContract

Awarded contract resulting from bid evaluation.

```rust
pub struct ProcurementContract {
    pub contract_id: String,
    pub vendor_id: String,
    pub rfx_id: Option<String>,
    pub contract_value: Decimal,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub auto_renew: bool,
}
```

Additional S2C models include `SpendAnalysis`, `SupplierQualification`, `Bid`, `BidEvaluation`, `CatalogItem`, and `SupplierScorecard`.

---

## HR / Payroll

Hire-to-Retire (H2R) process models.

### PayrollRun

A complete pay cycle for a company.

```rust
pub struct PayrollRun {
    pub payroll_id: String,
    pub company_code: String,
    pub pay_period_start: NaiveDate,
    pub pay_period_end: NaiveDate,
    pub run_date: NaiveDate,
    pub status: PayrollRunStatus,     // Draft, Calculated, Approved, Posted, Reversed
    pub total_gross: Decimal,
    pub total_deductions: Decimal,
    pub total_net: Decimal,
    pub total_employer_cost: Decimal,
    pub employee_count: u32,
}
```

### TimeEntry

Employee time tracking record.

```rust
pub struct TimeEntry {
    pub entry_id: String,
    pub employee_id: String,
    pub date: NaiveDate,
    pub hours_regular: f64,
    pub hours_overtime: f64,
    pub hours_pto: f64,
    pub hours_sick: f64,
    pub project_id: Option<String>,
    pub cost_center: Option<String>,
    pub approval_status: TimeApprovalStatus,  // Pending, Approved, Rejected
}
```

### ExpenseReport

Employee expense reimbursement.

```rust
pub struct ExpenseReport {
    pub report_id: String,
    pub employee_id: String,
    pub submission_date: NaiveDate,
    pub status: ExpenseStatus,        // Draft, Submitted, Approved, Rejected, Paid
    pub total_amount: Decimal,
    pub line_items: Vec<ExpenseLineItem>,
}

pub enum ExpenseCategory {
    Travel, Meals, Lodging, Transportation,
    Office, Entertainment, Training, Other,
}
```

### BenefitEnrollment

Employee benefit plan enrollment record.

```rust
pub struct BenefitEnrollment {
    pub id: String,
    pub entity_code: String,
    pub employee_id: String,
    pub employee_name: String,
    pub plan_type: BenefitPlanType,       // Health, Dental, Vision, Retirement401k, StockPurchase, LifeInsurance, Disability
    pub plan_name: String,
    pub enrollment_date: NaiveDate,
    pub effective_date: NaiveDate,
    pub employee_contribution: Decimal,
    pub employer_contribution: Decimal,
    pub currency: String,
    pub status: BenefitStatus,            // Active, Pending, Terminated, OnLeave
    pub is_active: bool,
}
```

---

## Manufacturing

Production and quality process models.

### ProductionOrder

Manufacturing production order linked to materials.

```rust
pub struct ProductionOrder {
    pub order_id: String,
    pub material_id: String,
    pub planned_quantity: Decimal,
    pub actual_quantity: Decimal,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub status: ProductionOrderStatus,
}
```

### QualityInspection

Quality control inspection record.

```rust
pub struct QualityInspection {
    pub inspection_id: String,
    pub production_order_id: String,
    pub inspection_date: NaiveDate,
    pub result: InspectionResult,     // Pass, Fail, Conditional
    pub defect_count: u32,
}
```

### CycleCount

Inventory cycle count with variance tracking.

```rust
pub struct CycleCount {
    pub count_id: String,
    pub material_id: String,
    pub warehouse: String,
    pub count_date: NaiveDate,
    pub system_quantity: Decimal,
    pub counted_quantity: Decimal,
    pub variance: Decimal,
}
```

### BomComponent

Bill-of-materials component linking parent materials to child components.

```rust
pub struct BomComponent {
    pub id: String,
    pub entity_code: String,
    pub parent_material: String,
    pub component_material: String,
    pub component_description: String,
    pub level: u32,
    pub quantity_per: Decimal,
    pub unit: String,
    pub scrap_rate: Decimal,
    pub is_phantom: bool,
}
```

### InventoryMovement

Goods movement record for receipts, issues, transfers, returns, scrap, and adjustments.

```rust
pub struct InventoryMovement {
    pub id: String,
    pub entity_code: String,
    pub material_code: String,
    pub movement_date: NaiveDate,
    pub movement_type: MovementType,  // GoodsReceipt, GoodsIssue, Transfer, Return, Scrap, Adjustment
    pub quantity: Decimal,
    pub value: Decimal,
    pub currency: String,
    pub storage_location: String,
    pub reference_doc: String,
}
```

---

## Tax

Tax accounting models (ASC 740/IAS 12). See [Tax Accounting](../advanced/tax-accounting.md) for full details.

### Key Models

- **TaxJurisdiction**: Tax authority with country/region codes, VAT registration status
- **TaxCode**: Tax rate definitions with type classification and exemption flags
- **TaxLine**: Individual line items on tax returns with amounts by jurisdiction
- **TaxReturn**: Filing records with total tax, amount paid, balance due, and filing status
- **TaxProvision**: Current/deferred tax provisions with effective rates by jurisdiction
- **WithholdingTaxRecord**: Withholding tax with base amounts, applied rates, and treaty applicability
- **UncertainTaxPosition**: UTP records with recognition thresholds and measurement methods

---

## Treasury

Treasury and cash management models. See [Treasury](../advanced/treasury.md) for full details.

### Key Models

- **CashPosition**: Daily bank account balances with closing, available, and value-date balances
- **CashForecast**: Forward-looking cash flow projections with confidence levels
- **CashPool**: Notional or physical cash pooling with header accounts and participant aggregation
- **CashPoolSweep**: Automated sweep transactions between pool participants and header
- **HedgingInstrument**: FX forwards, interest rate swaps, options with notional amounts
- **HedgeRelationship**: Hedge accounting designations with effectiveness testing (ASC 815/IFRS 9)
- **DebtInstrument**: Loan facilities with outstanding principal, rates, and maturity dates
- **DebtCovenant**: Financial covenant thresholds with actual values, headroom, and compliance status

---

## ESG

Environmental, Social, and Governance models. See [ESG/Sustainability](../advanced/esg-sustainability.md) for full details.

### Key Models

- **EmissionRecord**: GHG Scope 1/2/3 emissions with CO2e tonnes and emission factors
- **EnergyConsumption**: Energy usage by source (renewable/non-renewable) with MWh tracking
- **WaterUsage**: Water withdrawal, consumption, and discharge by source
- **WasteRecord**: Waste generation by type (hazardous/non-hazardous) with disposal methods
- **WorkforceDiversityMetric**: Demographic representation by category and level
- **PayEquityMetric**: Compensation equity ratios by demographic group
- **SafetyIncident**: Workplace incidents with severity and root cause classification
- **SafetyMetric**: Aggregate safety rates (TRIR, DART, lost time)
- **GovernanceMetric**: Board composition, independence, and oversight metrics
- **SupplierEsgAssessment**: Vendor ESG scores with environmental/social/governance breakdown
- **MaterialityAssessment**: Double materiality assessments by ESG topic
- **EsgDisclosure**: Framework-specific disclosures (GRI, SASB, TCFD) with completion tracking
- **ClimateScenario**: TCFD climate scenarios with warming pathways and financial impact

---

## Project Accounting

Project accounting models with WBS, earned value, and cost tracking. See [Project Accounting](../advanced/project-accounting.md) for full details.

### Key Models

- **Project**: Project master with WBS structure, status, and budget
- **ProjectCostLine**: Cost postings to WBS elements with budget/actual/variance
- **ProjectRevenue**: Revenue recognition records (PoC or milestone-based)
- **EarnedValueMetric**: EVM metrics (BCWS, BCWP, ACWP, CPI, SPI) per period
- **ChangeOrder**: Project scope/budget change requests with approval workflow
- **ProjectMilestone**: Deliverable milestones with payment amounts and completion status

---

## Sales Quotes

Quote-to-order pipeline models.

### SalesQuote

Sales quotation record.

```rust
pub struct SalesQuote {
    pub quote_id: String,
    pub customer_id: String,
    pub quote_date: NaiveDate,
    pub valid_until: NaiveDate,
    pub total_amount: Decimal,
    pub status: QuoteStatus,          // Draft, Sent, Won, Lost, Expired
    pub converted_order_id: Option<String>,
}
```

---

## Graph Properties

### ToNodeProperties Trait

All 51 entity types implement `ToNodeProperties` for graph property mapping:

```rust
pub trait ToNodeProperties {
    fn node_type_name(&self) -> &'static str;  // e.g. "uncertain_tax_position"
    fn node_type_code(&self) -> u16;           // e.g. 416
    fn to_node_properties(&self) -> HashMap<String, GraphPropertyValue>;
}
```

Property maps use camelCase keys (e.g. `entityCode`, `totalAmount`, `isApproved`) for downstream graph consumers.

### GraphPropertyValue

```rust
pub enum GraphPropertyValue {
    String(String), Int(i64), Float(f64), Decimal(Decimal),
    Bool(bool), Date(NaiveDate), StringList(Vec<String>),
}
```

### GraphEntityType

Registry of 51 entity types with numeric codes (100-504), 2-letter codes, snake_case names, and category helpers (`is_tax()`, `is_treasury()`, `is_esg()`, `is_project()`, `is_h2r()`, `is_mfg()`, `is_governance()`).

### RelationshipType

Registry of 28+ typed edge variants with `EdgeConstraint` validation (source/target entity types, `Cardinality`).

See [Graph Export](../advanced/graph-export.md) for the full entity type code table and edge registry.

---

## Decimal Handling

All monetary amounts use `rust_decimal::Decimal`:

```rust
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

let amount = dec!(1234.56);
let tax = amount * dec!(0.077);
```

Serialized as strings to prevent IEEE 754 issues:

```json
{"amount": "1234.56"}
```

## See Also

- [datasynth-core Crate](../crates/datasynth-core.md)
- [Data Flow](data-flow.md)
- [Generation Pipeline](generation-pipeline.md)
