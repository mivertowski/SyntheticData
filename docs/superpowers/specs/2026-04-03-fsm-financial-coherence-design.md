# Audit FSM Financial Data Coherence

**Date**: 2026-04-03
**Status**: Draft
**Scope**: Wire real financial data through the entire audit FSM pipeline so that every artifact — materiality, risk assessment, sampling, analytical procedures, findings, and opinions — references actual financial data with coherent values, traceable IDs, and consistent amounts across the full data chain.

## Problem

The audit FSM engine generates audit artifacts (sampling plans, sampled items, analytical procedures, materiality calculations) in isolation from the actual journal entry population. Specifically:

1. **Materiality uses zero values** — `pretax_income`, `equity`, `gross_profit`, `working_capital`, `operating_cash_flow`, `total_debt` are all `Decimal::ZERO` in the FSM path's `EngagementContext`, despite real JE data being available in the orchestrator. Materiality calculations produce numbers disconnected from reality.

2. **Sampled items have synthetic IDs** — `SampledItem.item_id` uses generated slugs like `"C001-TRADE_RECEIVABLES-KEY-001"` that don't correspond to any actual `JournalEntry.document_id`. Key item amounts are randomly generated rather than drawn from real JE lines above tolerable error.

3. **No financial data in output** — `ArtifactBag` and `EngagementResult` contain zero journal entries or trial balance data. Downstream consumers (AssureTwin) can't do full population analytics because the population isn't included.

4. **Revenue/asset calculations are unfiltered** — The FSM path sums ALL credit amounts as "revenue" and ALL debit amounts as "assets", rather than filtering by account range (4xxx for revenue, 1xxx for assets) as the legacy path does.

The legacy (non-FSM) audit path correctly computes financial metrics from JE data using account-range filters. The FSM path was built without this wiring.

## Solution

Three layers of enhancement, each building on the previous:

### Layer 1: Fix EngagementContext Financial Metrics

**Where**: `crates/datasynth-runtime/src/enhanced_orchestrator.rs`, method `generate_audit_data_with_fsm()`

Replace the six `Decimal::ZERO` assignments with real computations from the `entries: &[JournalEntry]` parameter, using the same account-range logic as the legacy path:

| Metric | Computation | Account Filter |
|--------|-------------|----------------|
| `total_revenue` | Sum of credit amounts | `account_code.starts_with('4')` |
| `total_assets` | Sum of debit amounts | `account_code.starts_with('1')` |
| `pretax_income` | `revenue - expenses` | expenses: debit on `'5'` or `'6'` |
| `equity` | Sum of credit amounts | `account_code.starts_with('3')` |
| `gross_profit` | `revenue * 0.35` | (derived) |
| `working_capital` | Current assets - current liabilities | `'1'` debits - `'2'` credits (simplified) |
| `operating_cash_flow` | `pretax_income * 0.85` | (approximation, consistent with legacy) |
| `total_debt` | Sum of credit amounts | `account_code.starts_with('2')` |

Also fix the existing `total_revenue` and `total_assets` calculations to use account-range filters (currently unfiltered).

Apply per-company filtering when multiple companies exist, with volume_weight fallback for companies with no JE data (matching legacy path logic).

### Layer 2: Wire Real JEs into Sampling

**Where**: Multiple files across `datasynth-generators` and `datasynth-audit-fsm`

#### 2a. Expand EngagementContext with full JE data

Add to `EngagementContext` (`crates/datasynth-audit-fsm/src/context.rs`):

```rust
/// Full journal entry population for sampling and population analytics.
/// Populated by the orchestrator from the already-generated JE data.
pub journal_entries: Vec<JournalEntry>,
```

The orchestrator passes the complete `entries` slice (not just 50 IDs). This is the same data already available in `generate_audit_data_with_fsm(entries)`.

#### 2b. Enhance SamplingPlanGenerator with JE-aware methods

Add to `SamplingPlanGenerator` (`crates/datasynth-generators/src/audit/sampling_plan_generator.rs`):

```rust
/// Generate sampling plans with real JE population data.
/// Key items are actual JE lines with amount > tolerable_error.
/// Representative items are sampled from the remaining JE population.
pub fn generate_for_cras_with_population(
    &mut self,
    cras: &[CombinedRiskAssessment],
    tolerable_error: Option<Decimal>,
    journal_entries: &[JournalEntry],
    account_balances: &HashMap<String, f64>,
) -> (Vec<SamplingPlan>, Vec<SampledItem>)
```

**Key item selection from real JEs:**

For each CRA (filtered to Moderate+ risk):
1. Identify the CRA's `account_area` → map to GL account ranges (e.g., "Revenue" → `4xxx`, "Trade Receivables" → `1100`-`1199`)
2. Filter JE lines to those matching the account range
3. Select lines where `abs(debit_amount - credit_amount) > tolerable_error` as key items
4. Use real `JournalEntry.header.document_id` as `SampledItem.item_id`
5. Use real line amounts as `SampledItem.amount`
6. Cap key items at 20 per CRA (take largest by amount)

**Representative item selection from real JEs:**
1. From remaining JE lines (same account filter, excluding key items), sample `rep_sample_size` items
2. Use Monetary Unit Sampling (MUS) for balance assertions, systematic selection for transaction assertions (matching existing methodology selection)
3. Use real document IDs and amounts

**Account area to GL range mapping:**

```rust
fn account_range_for_area(account_area: &str) -> Vec<&str> {
    match account_area.to_lowercase().as_str() {
        s if s.contains("revenue") || s.contains("sales") => vec!["4"],
        s if s.contains("receivable") => vec!["11"],
        s if s.contains("payable") => vec!["20"],
        s if s.contains("inventory") || s.contains("stock") => vec!["12", "13"],
        s if s.contains("cash") || s.contains("bank") => vec!["10"],
        s if s.contains("fixed asset") || s.contains("ppe") => vec!["14", "15", "16"],
        s if s.contains("equity") => vec!["3"],
        s if s.contains("expense") || s.contains("cost") => vec!["5", "6"],
        s if s.contains("debt") || s.contains("loan") || s.contains("borrow") => vec!["23", "24"],
        s if s.contains("tax") => vec!["17", "25"],
        s if s.contains("provision") => vec!["26"],
        _ => vec![], // Fallback: use all JE lines
    }
}
```

This mapping uses the CoA structure already in the codebase (GL constants in `datasynth-core/src/accounts.rs`).

**Backward compatibility**: The existing `generate_for_cras()` method stays unchanged. The new method is called when JE data is available; the old method remains the fallback.

#### 2c. Update StepDispatcher

In `dispatch_sampling()` (`crates/datasynth-audit-fsm/src/dispatch.rs`):

```rust
fn dispatch_sampling(&mut self, ctx: &EngagementContext, bag: &mut ArtifactBag) {
    // ... existing CRA check ...
    let tolerable_error = bag.materiality_calculations.first()
        .map(|m| m.performance_materiality);

    if ctx.journal_entries.is_empty() {
        // Fallback: existing synthetic generation
        let (plans, items) = self.sampling_gen
            .generate_for_cras(&bag.combined_risk_assessments, tolerable_error);
        bag.sampling_plans.extend(plans);
        bag.sampled_items.extend(items);
    } else {
        // Coherent: real JE-based sampling
        let (plans, items) = self.sampling_gen
            .generate_for_cras_with_population(
                &bag.combined_risk_assessments,
                tolerable_error,
                &ctx.journal_entries,
                &ctx.account_balances,
            );
        bag.sampling_plans.extend(plans);
        bag.sampled_items.extend(items);
    }
}
```

### Layer 3: Include Financial Data in Output

**Where**: `crates/datasynth-audit-fsm/src/artifact.rs` and orchestrator

#### 3a. Add financial data fields to ArtifactBag

```rust
#[derive(Debug, Clone, Default, Serialize)]
pub struct ArtifactBag {
    // ... existing 20 fields ...

    /// Journal entry population referenced by sampling and analytical procedures.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub journal_entries: Vec<JournalEntry>,

    /// Trial balance derived from journal entries.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub trial_balance_entries: Vec<TrialBalanceEntry>,
}
```

Use `skip_serializing_if` to keep output lean when financial data isn't included (backward compatibility for non-FSM path).

`TrialBalanceEntry` is a lightweight struct (not the full `PeriodTrialBalance`):

```rust
/// Single account balance for trial balance output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialBalanceEntry {
    pub account_code: String,
    pub account_description: String,
    pub debit_balance: Decimal,
    pub credit_balance: Decimal,
    pub net_balance: Decimal,
    pub entity_code: String,
    pub period: String,
}
```

#### 3b. Populate in orchestrator

After running the FSM engine, the orchestrator populates the financial data fields:

```rust
// After engine.run_engagement(&context)
result.artifacts.journal_entries = entries.to_vec();
result.artifacts.trial_balance_entries = compute_trial_balance(entries, &company_code, fiscal_year);
```

The `compute_trial_balance` function aggregates JE lines by account code into debit/credit balances.

#### 3c. Update total_artifacts()

Update `ArtifactBag::total_artifacts()` to include the new fields in the count.

## Layer 4: Full Data Chain Coherence

Beyond sampling, every artifact the FSM produces must be grounded in the same financial reality. This layer ensures the entire simulated audit process — from risk assessment through to the opinion — produces internally consistent data.

### 4a. Risk Assessment coherence

`CombinedRiskAssessment` uses `account_area` and `assertion` to identify what's being assessed. The risk factors and planned responses should reference actual account balances:

- **Enhance CRA generation** in `dispatch_cra()`: when JE data is available, set `CRA.population_value` from real account balances (sum of JE lines for the account area). Currently this is a synthetic estimate.
- **Risk factors** should reference actual data patterns (e.g., "Revenue account 4100 has 847 postings totaling $2.3M" rather than generic text).

### 4b. Analytical Procedures coherence

`AnalyticalProcedureResult` has `actual_value`, `expectation`, `variance`, and `requires_investigation`. Currently:
- `actual_value` is synthetic
- `expectation` is synthetic

**Enhancement**: When JE data is available:
- `actual_value` = real account balance from JE aggregation for the procedure's `account_or_area`
- `expectation` = derived from prior period logic or budget (can use actual_value * (1 ± variance_pct) for simulation)
- `variance` and `requires_investigation` computed from real numbers

Update `dispatch_analytical_procedures()` to pass JE-derived account balances to the analytical generator.

### 4c. Findings coherence

`AuditFinding` currently enriches findings with `"Supporting JE: {je_id}"` text. Enhance:
- `finding.amount` should reference actual JE amounts when the finding relates to a sampled item
- `finding.account` should reference real GL accounts from the JE population
- Cross-reference: if a finding stems from a sampled item, `finding.related_item_id` should match a real `SampledItem.item_id` which in turn matches a real `JournalEntry.document_id`

### 4d. Going Concern coherence

`GoingConcernAssessment` uses financial ratios. Currently synthetic. Enhance:
- Current ratio = real current assets / real current liabilities (from JE account balances)
- Debt-to-equity = real total_debt / real equity
- Cash flow indicators from real operating_cash_flow
- These are already available once Layer 1 fixes EngagementContext metrics

### 4e. Audit Opinion coherence

`AuditOpinion` should be informed by actual materiality, actual misstatement totals from sampling, and actual findings. The opinion type (unmodified/qualified/adverse/disclaimer) should be consistent with:
- Total misstatements found in sampling vs. materiality threshold
- Severity and count of findings
- Going concern indicators

### 4f. Confirmation coherence

`ExternalConfirmation` for accounts receivable/payable should reference real account balances and, where possible, real customer/vendor names from master data already in EngagementContext.

## Coherence Guarantees

After all layers, the following coherence properties hold across the entire data chain:

| Property | Guarantee |
|----------|-----------|
| **Financial Metrics** | |
| Materiality benchmark amounts | Derived from actual JE account balances using account-range filters |
| Tolerable error | Computed from real materiality (which uses real financials) |
| Trial balance | Derived from same JE population included in output |
| **Risk Assessment** | |
| CRA population values | Real account balances from JE data |
| Risk factors | Reference actual transaction volumes and amounts |
| **Sampling** | |
| Key item identification | JE lines genuinely exceeding tolerable error |
| Key item document IDs | Real `JournalEntry.header.document_id` values |
| Key item amounts | Real JE line amounts |
| Representative item IDs | Real JE document IDs |
| Representative item amounts | Real JE line amounts |
| Population total | Matches actual JE population for that account area |
| **Analytical Procedures** | |
| Actual values | Real account balances from JE aggregation |
| Variances | Computed from real numbers |
| Investigation triggers | Based on real materiality thresholds |
| **Findings & Opinion** | |
| Finding amounts | From real sampled items / real JE amounts |
| Finding references | Traceable to real document IDs |
| Misstatement totals | Aggregated from real sampling results |
| Opinion basis | Consistent with real materiality vs. real misstatements |
| **Confirmations** | |
| Confirmation amounts | From real AR/AP account balances |
| Confirmation parties | From real vendor/customer master data |
| **Output Completeness** | |
| JE population included | Full population in ArtifactBag for downstream analytics |
| Trial balance included | Derived from same JE data, consistent with all references |

## Files Changed

| File | Change |
|------|--------|
| `crates/datasynth-runtime/src/enhanced_orchestrator.rs` | Fix financial metric computation in FSM path; populate ArtifactBag with JE/TB data |
| `crates/datasynth-audit-fsm/src/context.rs` | Add `journal_entries: Vec<JournalEntry>` to EngagementContext |
| `crates/datasynth-audit-fsm/src/artifact.rs` | Add `journal_entries` and `trial_balance_entries` to ArtifactBag; add TrialBalanceEntry struct |
| `crates/datasynth-audit-fsm/src/dispatch.rs` | Update `dispatch_sampling()`, `dispatch_analytical_procedures()`, `dispatch_cra()`, `dispatch_going_concern()`, `dispatch_confirmations()` to use real JE data when available |
| `crates/datasynth-generators/src/audit/sampling_plan_generator.rs` | Add `generate_for_cras_with_population()` method; add account-area-to-GL mapping |
| `crates/datasynth-generators/src/audit/analytical_procedure_generator.rs` | Accept real account balances for actual_value computation |

## Testing Strategy

| Test | Scope | What It Verifies |
|------|-------|------------------|
| Unit: financial metric computation | orchestrator | Revenue, expenses, equity, pretax_income computed correctly from JE account ranges |
| Unit: account area mapping | sampling generator | "Revenue" maps to 4xxx, "Trade Receivables" to 11xx, etc. |
| Unit: key item selection | sampling generator | Items above TE selected, real document IDs used, capped at 20 |
| Unit: representative selection | sampling generator | Correct sample size, real IDs, amounts from JE population |
| Unit: analytical actual_value | analytical generator | actual_value matches JE account balance |
| Integration: end-to-end coherence | full pipeline | Every sampled item ID exists in JE population; every key item amount > tolerable error; materiality derived from real financials; trial balance totals match JE sums; finding amounts trace to real JEs |
| Integration: backward compat | full pipeline | Empty journal_entries falls back to existing synthetic generation |
| Existing: all 10 blueprints | regression | All existing audit blueprint tests still pass |

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Large JE population in EngagementContext | Memory — thousands of JEs cloned | Use `Arc<Vec<JournalEntry>>` or pass by reference; skip clone when possible |
| Account area mapping misses | Some CRAs may not find matching JEs | Fallback to synthetic generation for CRAs with no matching JE lines |
| Breaking existing tests | High | Layer 2b is additive (new method); dispatcher fallback preserves old behavior |
| Performance regression | Medium | JE filtering is O(n) per CRA; acceptable for typical populations (<100K JEs) |
| Analytical generator API change | Medium | Add optional parameter; existing callers unaffected |
