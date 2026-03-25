# Audit Data Completeness: Closing the Gaps

**Date**: 2026-03-25
**Status**: Approved
**Scope**: Address all 14 data gaps identified by the analytics inventory gap analysis so that every FSA audit procedure has the synthetic data it needs.

## Problem

Gap analysis of the data analytics inventory against SyntheticData's generators reveals:
- 3 data types completely missing (prior_year, industry_data, board_minutes)
- 5 data types partially covered (journal_entries, organizational, system_reports, management_reports, financial_statements)
- 3 analytical procedures partially covered (cutoff_test, completeness_check, expectation_model)

This means audit procedures that rely on these data types cannot produce realistic, evidence-backed artifacts.

## Solution

Eight work items, ordered by audit impact. Each produces testable, self-contained changes.

---

## WI-1: Journal Entry Audit Flags (Critical — ISA 240)

The JE fraud testing analytical procedure (the single most important audit data analytics test) requires flags that don't exist on `JournalEntry`:

| Field | Type | Purpose | ISA Reference |
|-------|------|---------|---------------|
| `is_manual` | `bool` | Manual entries are higher fraud risk | ISA 240.32(a) |
| `is_post_close` | `bool` | Post-closing entries require scrutiny | ISA 240.32(a) |
| `source` | `String` | Source system/module (e.g., "SAP-FI", "manual", "interface") | ISA 240 |
| `created_by` | `String` | User who created (vs `posted_by` who approved) | ISA 240.32(a) |
| `created_date` | `NaiveDateTime` | Timestamp of creation (may differ from posting) | ISA 240 |

### Changes
- `datasynth-core/src/models/journal_entry.rs` — add fields with `#[serde(default)]`
- `datasynth-generators/src/je_generator.rs` — populate flags:
  - `is_manual`: ~5% of entries (configurable), higher at period-end
  - `is_post_close`: entries posted after period-end date
  - `source`: selected from ["SAP-FI", "SAP-MM", "SAP-SD", "manual", "interface", "spreadsheet"]
  - `created_by`: from employee pool (may differ from poster for SoD analysis)
  - `created_date`: posting_date minus processing lag
- `datasynth-output` — include new fields in CSV/JSON export
- Tests: verify flag distributions, manual entry correlation with period-end

---

## WI-2: Prior-Year Data Generation (Critical — ISA 315/520)

Audit procedures compare current year against prior year for trend analysis, risk assessment, and analytical procedures. This is the single most impactful data type gap.

### Approach

Generate two fiscal periods instead of one. The first period becomes "prior year", the second "current year". Prior-year audit findings carry forward as `prior_issues`.

### Changes
- `datasynth-config/src/schema.rs` — add `prior_year: PriorYearConfig` section:
  ```yaml
  prior_year:
    enabled: true
    generate_prior_period: true  # generates PY-1 data
    carry_forward_findings: true
    prior_year_seed_offset: 10000
  ```
- `datasynth-runtime/src/enhanced_orchestrator.rs` — when enabled, run generation twice:
  1. First pass with date range shifted back 12 months → prior year data
  2. Second pass with normal dates → current year data
  3. Tag prior-year data with `period_type: "prior_year"` or separate output directory
- New model: `PriorYearComparative` in `datasynth-core`:
  ```rust
  pub struct PriorYearComparative {
      pub account: String,
      pub current_year_amount: Decimal,
      pub prior_year_amount: Decimal,
      pub variance: Decimal,
      pub variance_pct: f64,
  }
  ```
- Financial statements get `prior_year_amount` column populated
- Prior audit findings model: `PriorYearFinding` with status (remediated/open/recurring)
- Output: `prior_year/` subdirectory with same structure as current year

### Tests
- Two-period generation produces 2x the data
- Comparative report shows meaningful variances
- Prior-year findings carry forward

---

## WI-3: Industry Benchmark Data (Important — ISA 520)

Analytical procedures require industry comparisons (peer group metrics, industry medians). Currently the `industry_profiles.rs` has Retail/Manufacturing/Financial Services profiles but they're used internally for distribution parameters, not exported as reference data.

### Changes
- New model: `IndustryBenchmark` in `datasynth-core`:
  ```rust
  pub struct IndustryBenchmark {
      pub industry: String,
      pub metric: String,
      pub value: Decimal,
      pub source: String,
      pub period: String,
  }
  ```
- New generator: `industry_benchmark_generator.rs` in `datasynth-generators`:
  - Generates benchmarks per industry preset (retail, manufacturing, financial_services, healthcare, technology)
  - Metrics: median_revenue, gross_margin, net_margin, current_ratio, debt_to_equity, revenue_growth, employee_count_median, interest_rates, inflation_rate
  - Values derived from industry_profiles.rs parameters + controlled randomness
- Output: `benchmarks/industry_benchmarks.json`
- Tests: benchmarks generated for configured industry, metrics are reasonable

---

## WI-4: IT System Reports (Important — ITGC)

ITGC (IT General Controls) testing requires access logs, change management records, and backup verification — none currently generated.

### Changes
- New models in `datasynth-core/src/models/`:
  ```rust
  pub struct AccessLog {
      pub log_id: Uuid,
      pub timestamp: NaiveDateTime,
      pub user_id: String,
      pub system: String,
      pub action: String,        // login, logout, failed_login, privilege_escalation
      pub ip_address: String,
      pub success: bool,
  }

  pub struct ChangeManagementRecord {
      pub change_id: Uuid,
      pub system: String,
      pub change_type: String,   // config, code, access, patch
      pub requested_by: String,
      pub approved_by: Option<String>,
      pub implemented_date: NaiveDateTime,
      pub tested: bool,
      pub rollback_plan: bool,
  }
  ```
- New generator: `it_controls_generator.rs` in `datasynth-generators`:
  - Generates access logs (10-50 per user per month), change records (5-20 per month)
  - Anomalies: failed logins at unusual hours, unapproved changes, missing test evidence
- Output: `system_reports/access_logs.json`, `system_reports/change_management.json`
- Wire into existing `InternalControl` framework (ITGC controls reference these records)

---

## WI-5: Board Minutes & Governance Documents (Moderate — ISA 260)

Communication with Those Charged With Governance (TCWG) requires board minutes as evidence.

### Changes
- New model: `BoardMinutes` in `datasynth-core`:
  ```rust
  pub struct BoardMinutes {
      pub meeting_id: Uuid,
      pub meeting_date: NaiveDate,
      pub meeting_type: String,     // regular, special, audit_committee
      pub attendees: Vec<String>,
      pub key_decisions: Vec<String>,
      pub risk_discussions: Vec<String>,
      pub audit_committee_matters: Vec<String>,
  }
  ```
- New generator: `governance_generator.rs` in `datasynth-generators`:
  - Generates quarterly board meetings + monthly audit committee meetings
  - Key decisions reference actual company events (period-close, new contracts, impairments)
  - Risk discussions reference actual risk categories from the engagement
- Output: `governance/board_minutes.json`

---

## WI-6: Organizational Data Enrichment (Moderate — ISA 315)

Entity understanding requires data fields not currently generated.

### Changes
- Enrich company config or add `OrganizationalProfile` model:
  ```rust
  pub struct OrganizationalProfile {
      pub entity_code: String,
      pub it_systems: Vec<ItSystem>,
      pub regulatory_environment: Vec<String>,
      pub prior_auditor: Option<String>,
      pub org_structure: String,      // description
  }

  pub struct ItSystem {
      pub name: String,
      pub vendor: String,
      pub module: String,           // ERP, CRM, HCM, etc.
      pub go_live_date: NaiveDate,
      pub interfaces: Vec<String>,  // other systems it connects to
  }
  ```
- Generate per company from config (industry determines typical IT landscape)
- Output: `master_data/organizational_profile.json`

---

## WI-7: Management Report Enrichment (Lower — ISA 520)

KPI dashboards and cash flow projections as management-formatted reports.

### Changes
- New model: `ManagementReport` wrapping existing KPIs + budget data into a consolidated report format:
  ```rust
  pub struct ManagementReport {
      pub report_id: Uuid,
      pub report_type: String,      // monthly_pack, board_report, forecast
      pub period: String,
      pub entity_code: String,
      pub kpis: Vec<ManagementKpi>,
      pub budget_variances: Vec<BudgetVariance>,
      pub cash_flow_projection: Option<CashFlowProjection>,
  }
  ```
- Aggregates existing ManagementKpi + Budget data into report-level documents
- Output: `management_reports/management_reports.json`

---

## WI-8: Financial Statement Comparative Fields (Lower)

Add `prior_year_amount` and `assumptions` to FinancialStatementLineItem.

### Changes
- `datasynth-core/src/models/financial_reporting.rs` — add fields:
  ```rust
  pub prior_year_amount: Option<Decimal>,
  pub assumptions: Option<String>,
  ```
- When WI-2 (prior year) is enabled, populate `prior_year_amount` from the prior period data
- `assumptions` populated for significant estimates (e.g., impairment, provisions)

---

## Integrity, Quality, Statistics & Coherence Guardrails

Every new generator and model must satisfy these invariants:

### Data Integrity
- All generated IDs are deterministic (via `DeterministicUuidFactory` or seeded RNG)
- All cross-references resolve: if a finding cites a JE ID, that JE exists in the output
- If `is_post_close == true`, the entry's `posting_date` must be after `period_end_date`
- If `is_manual == true`, the `source` must be "manual" or "spreadsheet"
- Access logs must reference employees that exist in the master data
- Board minutes attendees must be drawn from the generated employee pool
- Prior-year amounts must match the actual prior-period generation output

### Statistical Properties
- `is_manual` rate follows configurable distribution (default ~5%), with period-end spike
- Access log timestamps follow intraday patterns (business hours weighted, overnight sparse)
- Failed login rate follows realistic distribution (2-5% of attempts)
- Industry benchmarks have appropriate variance around real-world ranges per industry
- Prior-year variances follow log-normal distribution (most small, few large)
- Change management records cluster around release cycles (monthly/quarterly)

### Coherence Across Generators
- JE `created_by` must exist in employee master data
- JE `source` must reference an IT system generated in `OrganizationalProfile`
- Change management records reference the same IT systems as `OrganizationalProfile`
- Board meeting decisions reference actual events (if impairment was generated, board discussed it)
- Prior-year findings should correlate with actual risk areas (if revenue is significant, prior year has revenue-related findings)
- Industry benchmarks must be consistent with the generated company's financial data (company metrics should be within 2 standard deviations of industry median)

### Benford's Law Compliance
- New amount fields (prior_year_amount, variance) must maintain Benford compliance
- Industry benchmark values must follow realistic first-digit distributions

### Balance/Reconciliation Integrity
- Prior-year closing balances must equal current-year opening balances
- Comparative financial statements must cross-foot (PY + variance = CY)
- Sum of all access log entries per user must not exceed 24 hours per day

## Testing Strategy

Each WI gets:
- **Unit tests** for the new model/generator
- **Integrity tests** verifying cross-references resolve (JE IDs, employee IDs, IT system names)
- **Statistical tests** verifying distributions match expected profiles (Benford's, rates, temporal patterns)
- **Coherence tests** verifying consistency across generators (sources match IT systems, attendees match employees)
- **Integration test** verifying the data appears in output
- **Gap analysis rerun** verifying audit procedure data requirements are met

Final validation: re-run the gap analysis script and verify coverage improves from 42% → 85%+.

## Implementation Order

| WI | Priority | Effort | Dependencies |
|----|----------|--------|-------------|
| 1 — JE flags | Critical | Small | None |
| 2 — Prior year | Critical | Large | WI-8 |
| 3 — Industry benchmarks | Important | Small | None |
| 4 — IT system reports | Important | Medium | None |
| 5 — Board minutes | Moderate | Small | None |
| 6 — Org profile | Moderate | Small | None |
| 7 — Management reports | Lower | Small | None |
| 8 — FS comparatives | Lower | Small | WI-2 |

WI-1, WI-3, WI-4, WI-5, WI-6 can all be done in parallel. WI-2 is the largest piece.
