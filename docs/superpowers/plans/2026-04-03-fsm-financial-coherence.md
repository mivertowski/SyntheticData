# Audit FSM Financial Data Coherence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire real journal entry data through the audit FSM pipeline so that materiality, sampling, analytical procedures, and all other audit artifacts reference actual financial data with coherent values and traceable IDs.

**Architecture:** Three-layer enhancement: (1) fix financial metric computation in the orchestrator's FSM path, (2) pass full JE population into the FSM and wire it through sampling with real document IDs, (3) include JE and trial balance data in the ArtifactBag output. All changes are additive with backward-compatible fallbacks.

**Tech Stack:** Rust, datasynth-core (JournalEntry, Decimal), datasynth-generators (SamplingPlanGenerator), datasynth-audit-fsm (EngagementContext, ArtifactBag, StepDispatcher), datasynth-runtime (EnhancedOrchestrator)

**Spec:** `docs/superpowers/specs/2026-04-03-fsm-financial-coherence-design.md`

---

## File Structure

```
Modified files:
crates/datasynth-runtime/src/enhanced_orchestrator.rs    — Fix financial metrics, pass JEs, populate output
crates/datasynth-audit-fsm/src/context.rs                — Add journal_entries field to EngagementContext
crates/datasynth-audit-fsm/src/artifact.rs               — Add journal_entries + trial_balance to ArtifactBag
crates/datasynth-audit-fsm/src/dispatch.rs               — Wire JE data to sampling dispatcher
crates/datasynth-generators/src/audit/sampling_plan_generator.rs — Add JE-aware sampling method

New test files:
crates/datasynth-generators/tests/sampling_coherence_tests.rs    — Test JE-aware sampling
crates/datasynth-audit-fsm/tests/financial_coherence_tests.rs   — Test end-to-end coherence
```

---

### Task 1: Fix financial metric computation in FSM path

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs` (method `generate_audit_data_with_fsm`)

The FSM path currently hardcodes `pretax_income`, `equity`, `gross_profit`, `working_capital`, `operating_cash_flow`, `total_debt` to `Decimal::ZERO` and computes `total_revenue`/`total_assets` without account-range filters. The legacy path has correct logic. Copy and adapt it.

- [ ] **Step 1: Write the failing test**

Create `crates/datasynth-audit-fsm/tests/financial_coherence_tests.rs`:

```rust
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Helper: compute financial metrics from JE lines the same way the
/// orchestrator should.  We test the logic in isolation first.
#[test]
fn test_financial_metrics_from_je_account_ranges() {
    // Simulate JE lines with known account codes and amounts
    // Revenue (4xxx credits): 100,000
    // COGS (5xxx debits): 40,000
    // OpEx (6xxx debits): 30,000
    // Assets (1xxx debits): 200,000
    // Liabilities (2xxx credits): 80,000
    // Equity (3xxx credits): 120,000

    let lines: Vec<(&str, Decimal, Decimal)> = vec![
        // (account_code, debit, credit)
        ("4000", dec!(0), dec!(60000)),     // Product Revenue
        ("4100", dec!(0), dec!(40000)),     // Service Revenue
        ("5000", dec!(40000), dec!(0)),     // COGS
        ("6100", dec!(20000), dec!(0)),     // Salaries
        ("6300", dec!(10000), dec!(0)),     // Rent
        ("1000", dec!(50000), dec!(0)),     // Cash
        ("1100", dec!(80000), dec!(0)),     // AR
        ("1500", dec!(70000), dec!(0)),     // Fixed Assets
        ("2000", dec!(0), dec!(50000)),     // AP
        ("2100", dec!(0), dec!(30000)),     // Other liabilities
        ("3000", dec!(0), dec!(80000)),     // Share capital
        ("3100", dec!(0), dec!(40000)),     // Retained earnings
    ];

    let total_revenue: Decimal = lines.iter()
        .filter(|(acc, _, _)| acc.starts_with('4'))
        .map(|(_, _, credit)| *credit)
        .sum();
    assert_eq!(total_revenue, dec!(100000));

    let total_assets: Decimal = lines.iter()
        .filter(|(acc, _, _)| acc.starts_with('1'))
        .map(|(_, debit, _)| *debit)
        .sum();
    assert_eq!(total_assets, dec!(200000));

    let total_expenses: Decimal = lines.iter()
        .filter(|(acc, _, _)| acc.starts_with('5') || acc.starts_with('6'))
        .map(|(_, debit, _)| *debit)
        .sum();
    assert_eq!(total_expenses, dec!(70000));

    let equity: Decimal = lines.iter()
        .filter(|(acc, _, _)| acc.starts_with('3'))
        .map(|(_, _, credit)| *credit)
        .sum();
    assert_eq!(equity, dec!(120000));

    let pretax_income = total_revenue - total_expenses;
    assert_eq!(pretax_income, dec!(30000));

    let total_debt: Decimal = lines.iter()
        .filter(|(acc, _, _)| acc.starts_with('2'))
        .map(|(_, _, credit)| *credit)
        .sum();
    assert_eq!(total_debt, dec!(80000));

    let gross_profit = total_revenue * dec!(0.35);
    assert_eq!(gross_profit, dec!(35000));

    let working_capital = total_assets - total_debt;
    assert_eq!(working_capital, dec!(120000));
}
```

- [ ] **Step 2: Run test to verify it passes** (this tests the logic, not the orchestrator integration)

Run: `cargo test -p datasynth-audit-fsm --test financial_coherence_tests -- --test-threads=4`
Expected: PASS

- [ ] **Step 3: Fix the orchestrator's FSM path**

In `crates/datasynth-runtime/src/enhanced_orchestrator.rs`, find the `generate_audit_data_with_fsm` method. Locate the section where `total_revenue` and `total_assets` are computed (around line 11154-11166) and the section where the EngagementContext is constructed (around line 11253-11279).

Replace the unfiltered revenue/asset calculations and the six `Decimal::ZERO` values:

```rust
// Replace the existing total_revenue calculation (currently sums ALL credits):
let total_revenue: rust_decimal::Decimal = entries
    .iter()
    .flat_map(|e| e.lines.iter())
    .filter(|l| l.account_code.starts_with('4'))
    .map(|l| l.credit_amount)
    .sum();

// Replace the existing total_assets calculation (currently sums ALL debits):
let total_assets: rust_decimal::Decimal = entries
    .iter()
    .flat_map(|e| e.lines.iter())
    .filter(|l| l.account_code.starts_with('1'))
    .map(|l| l.debit_amount)
    .sum();

// NEW: Compute the six missing metrics
let total_expenses: rust_decimal::Decimal = entries
    .iter()
    .flat_map(|e| e.lines.iter())
    .filter(|l| l.account_code.starts_with('5') || l.account_code.starts_with('6'))
    .map(|l| l.debit_amount)
    .sum();

let equity: rust_decimal::Decimal = entries
    .iter()
    .flat_map(|e| e.lines.iter())
    .filter(|l| l.account_code.starts_with('3'))
    .map(|l| l.credit_amount)
    .sum();

let total_debt: rust_decimal::Decimal = entries
    .iter()
    .flat_map(|e| e.lines.iter())
    .filter(|l| l.account_code.starts_with('2'))
    .map(|l| l.credit_amount)
    .sum();

let pretax_income = total_revenue - total_expenses;
let gross_profit = total_revenue * rust_decimal::Decimal::new(35, 2); // 35% margin
let working_capital = total_assets - total_debt;
let operating_cash_flow = pretax_income * rust_decimal::Decimal::new(85, 2); // 85% of PTI
```

Then in the EngagementContext construction, replace the six `Decimal::ZERO` lines:

```rust
pretax_income,
equity,
gross_profit,
working_capital,
operating_cash_flow,
total_debt,
```

- [ ] **Step 4: Verify existing tests still pass**

Run: `cargo test -p datasynth-audit-fsm -- --test-threads=4`
Expected: All existing tests pass

- [ ] **Step 5: Commit**

```bash
git add crates/datasynth-runtime/src/enhanced_orchestrator.rs crates/datasynth-audit-fsm/tests/financial_coherence_tests.rs
git commit -m "fix(audit-fsm): compute real financial metrics from JE account ranges in FSM path"
```

---

### Task 2: Add journal_entries to EngagementContext

**Files:**
- Modify: `crates/datasynth-audit-fsm/src/context.rs`
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs`

- [ ] **Step 1: Add field to EngagementContext**

In `crates/datasynth-audit-fsm/src/context.rs`, add after the `anomaly_refs` field:

```rust
    /// Full journal entry population for coherent sampling and analytics.
    /// When populated, sampling and analytical procedures use real JE data.
    pub journal_entries: Vec<datasynth_core::JournalEntry>,
```

- [ ] **Step 2: Add the import**

At the top of `context.rs`, ensure `datasynth_core` is imported. Check the crate's `Cargo.toml` — `datasynth-core` is already a dependency.

- [ ] **Step 3: Update all EngagementContext construction sites**

In `context.rs`, update `demo()` and `demo_with_anomalies()` methods to include:

```rust
journal_entries: Vec::new(),
```

In `crates/datasynth-runtime/src/enhanced_orchestrator.rs`, in the `generate_audit_data_with_fsm` method's EngagementContext construction, add:

```rust
journal_entries: entries.to_vec(),
```

- [ ] **Step 4: Verify compilation**

Run: `cargo check -p datasynth-audit-fsm && cargo check -p datasynth-runtime`
Expected: Compiles (may need to fix any other EngagementContext construction sites)

- [ ] **Step 5: Verify existing tests still pass**

Run: `cargo test -p datasynth-audit-fsm -- --test-threads=4`
Expected: All pass

- [ ] **Step 6: Commit**

```bash
git add crates/datasynth-audit-fsm/src/context.rs crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "feat(audit-fsm): add journal_entries field to EngagementContext"
```

---

### Task 3: Add JE and trial balance to ArtifactBag

**Files:**
- Modify: `crates/datasynth-audit-fsm/src/artifact.rs`

- [ ] **Step 1: Define TrialBalanceEntry struct**

Add at the top of `artifact.rs` (after the existing imports):

```rust
/// Single account balance for trial balance output within the audit context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialBalanceEntry {
    pub account_code: String,
    pub account_description: String,
    pub debit_balance: rust_decimal::Decimal,
    pub credit_balance: rust_decimal::Decimal,
    pub net_balance: rust_decimal::Decimal,
    pub entity_code: String,
    pub period: String,
}
```

- [ ] **Step 2: Add fields to ArtifactBag**

Add after the `confirmation_responses` field:

```rust
    /// Journal entry population referenced by sampling and analytical procedures.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub journal_entries: Vec<datasynth_core::JournalEntry>,

    /// Trial balance derived from journal entries.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub trial_balance_entries: Vec<TrialBalanceEntry>,
```

- [ ] **Step 3: Update total_artifacts()**

Add to the sum in `total_artifacts()`:

```rust
            + self.journal_entries.len()
            + self.trial_balance_entries.len()
```

- [ ] **Step 4: Verify compilation and tests**

Run: `cargo test -p datasynth-audit-fsm -- --test-threads=4`
Expected: All pass (Default derive handles Vec::new() for the new fields)

- [ ] **Step 5: Commit**

```bash
git add crates/datasynth-audit-fsm/src/artifact.rs
git commit -m "feat(audit-fsm): add journal_entries and trial_balance to ArtifactBag"
```

---

### Task 4: Populate ArtifactBag financial data in orchestrator

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs`

- [ ] **Step 1: Add trial balance computation helper**

Add a helper function (can be a free function or method) near the `generate_audit_data_with_fsm` method:

```rust
fn compute_trial_balance_entries(
    entries: &[datasynth_core::JournalEntry],
    entity_code: &str,
    fiscal_year: i32,
) -> Vec<datasynth_audit_fsm::artifact::TrialBalanceEntry> {
    use std::collections::BTreeMap;
    let mut balances: BTreeMap<String, (rust_decimal::Decimal, rust_decimal::Decimal)> = BTreeMap::new();

    for je in entries {
        for line in &je.lines {
            let entry = balances.entry(line.account_code.clone()).or_default();
            entry.0 += line.debit_amount;
            entry.1 += line.credit_amount;
        }
    }

    balances
        .into_iter()
        .map(|(account_code, (debit, credit))| {
            datasynth_audit_fsm::artifact::TrialBalanceEntry {
                account_code: account_code.clone(),
                account_description: account_code.clone(), // Use code as description fallback
                debit_balance: debit,
                credit_balance: credit,
                net_balance: debit - credit,
                entity_code: entity_code.to_string(),
                period: format!("FY{}", fiscal_year),
            }
        })
        .collect()
}
```

- [ ] **Step 2: Populate after engine run**

In `generate_audit_data_with_fsm`, after `engine.run_engagement(&context)` returns the result, add:

```rust
// Populate financial data in the artifact bag for downstream consumers.
result.artifacts.journal_entries = entries.to_vec();
result.artifacts.trial_balance_entries = compute_trial_balance_entries(
    entries,
    &company_code,
    start_date.year(),
);
```

Note: `result` here is the `EngagementResult` returned by the engine. Find the exact variable name in the existing code and adapt.

- [ ] **Step 3: Verify compilation and tests**

Run: `cargo test -p datasynth-runtime -- --test-threads=4`
Expected: All pass

- [ ] **Step 4: Commit**

```bash
git add crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "feat(audit-fsm): populate ArtifactBag with JE population and trial balance"
```

---

### Task 5: Implement JE-aware sampling in SamplingPlanGenerator

**Files:**
- Modify: `crates/datasynth-generators/src/audit/sampling_plan_generator.rs`
- Create: `crates/datasynth-generators/tests/sampling_coherence_tests.rs`

This is the core task — adding a new method that selects key items and representative items from real JE data.

- [ ] **Step 1: Write the failing test**

Create `crates/datasynth-generators/tests/sampling_coherence_tests.rs`:

```rust
use datasynth_core::models::JournalEntry;
use datasynth_generators::audit::sampling_plan_generator::SamplingPlanGenerator;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

/// Build a minimal JournalEntry with one line for testing.
fn make_je(document_id: &str, account_code: &str, debit: Decimal, credit: Decimal) -> JournalEntry {
    use chrono::{NaiveDate, Utc};
    use datasynth_core::models::journal_entry::*;
    use smallvec::smallvec;
    use uuid::Uuid;

    let doc_id = Uuid::parse_str(document_id)
        .unwrap_or_else(|_| Uuid::new_v4());

    JournalEntry {
        header: JournalEntryHeader {
            document_id: doc_id,
            company_code: "C001".to_string(),
            fiscal_year: 2024,
            fiscal_period: 6,
            posting_date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            document_date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            created_at: Utc::now(),
            document_type: "SA".to_string(),
            currency: "USD".to_string(),
            exchange_rate: dec!(1),
            reference: None,
            header_text: None,
            created_by: "TEST".to_string(),
            ..Default::default()
        },
        lines: smallvec![JournalEntryLine {
            document_id: doc_id,
            line_number: 1,
            gl_account: account_code.to_string(),
            account_code: account_code.to_string(),
            debit_amount: debit,
            credit_amount: credit,
            local_amount: debit - credit,
            ..Default::default()
        }],
    }
}

#[test]
fn test_key_items_use_real_je_document_ids() {
    // Create JEs with known amounts on revenue accounts (4xxx)
    let je_big = make_je("00000000-0000-0000-0000-000000000001", "4000", dec!(0), dec!(500000));
    let je_medium = make_je("00000000-0000-0000-0000-000000000002", "4000", dec!(0), dec!(100000));
    let je_small = make_je("00000000-0000-0000-0000-000000000003", "4000", dec!(0), dec!(5000));

    let entries = vec![je_big, je_medium, je_small];

    // Create a CRA for revenue with tolerable error = 50,000
    // je_big (500,000) and je_medium (100,000) should be key items
    // je_small (5,000) should NOT be a key item
    let cra = datasynth_core::models::audit::CombinedRiskAssessment {
        id: "CRA-C001-REVENUE-Occurrence".to_string(),
        entity_code: "C001".to_string(),
        account_area: "Revenue".to_string(),
        assertion: datasynth_core::models::audit::AuditAssertion::Occurrence,
        combined_risk: datasynth_core::models::audit::CraLevel::High,
        ..Default::default()
    };

    let account_balances: HashMap<String, f64> = HashMap::from([
        ("4000".to_string(), -605000.0), // credit balance
    ]);

    let mut gen = SamplingPlanGenerator::new(42);
    let (plans, items) = gen.generate_for_cras_with_population(
        &[cra],
        Some(dec!(50000)),
        &entries,
        &account_balances,
    );

    assert!(!plans.is_empty(), "Should generate at least one plan");
    assert!(!items.is_empty(), "Should generate sampled items");

    // Key items should use real JE document IDs
    let key_items: Vec<_> = items.iter()
        .filter(|i| i.selection_type == datasynth_core::models::audit::SelectionType::KeyItem)
        .collect();

    for ki in &key_items {
        // item_id should be a real UUID from our JEs, not a synthetic slug
        assert!(
            ki.item_id.contains("00000000-0000-0000-0000-"),
            "Key item ID should be a real JE document_id, got: {}",
            ki.item_id
        );
        // Key item amount should be > tolerable error
        assert!(
            ki.amount > dec!(50000),
            "Key item amount {} should be > tolerable error 50000",
            ki.amount
        );
    }
}

#[test]
fn test_representative_items_use_real_je_document_ids() {
    // Create 50 JEs on AR accounts (1100) with small amounts
    let entries: Vec<JournalEntry> = (0..50)
        .map(|i| {
            let id = format!("00000000-0000-0000-0000-{:012}", i);
            make_je(&id, "1100", dec!(1000) + Decimal::from(i * 100), dec!(0))
        })
        .collect();

    let cra = datasynth_core::models::audit::CombinedRiskAssessment {
        id: "CRA-C001-TRADE_RECEIVABLES-Existence".to_string(),
        entity_code: "C001".to_string(),
        account_area: "Trade Receivables".to_string(),
        assertion: datasynth_core::models::audit::AuditAssertion::Existence,
        combined_risk: datasynth_core::models::audit::CraLevel::Moderate,
        ..Default::default()
    };

    let account_balances: HashMap<String, f64> = HashMap::from([
        ("1100".to_string(), 72500.0), // sum of debit balances
    ]);

    let mut gen = SamplingPlanGenerator::new(42);
    let (plans, items) = gen.generate_for_cras_with_population(
        &[cra],
        Some(dec!(100000)), // High TE so no key items
        &entries,
        &account_balances,
    );

    assert!(!items.is_empty(), "Should have representative items");

    let rep_items: Vec<_> = items.iter()
        .filter(|i| i.selection_type == datasynth_core::models::audit::SelectionType::Representative)
        .collect();

    for ri in &rep_items {
        // item_id should be a real JE document_id
        assert!(
            ri.item_id.contains("00000000-0000-0000-0000-"),
            "Representative item ID should be a real JE document_id, got: {}",
            ri.item_id
        );
    }
}

#[test]
fn test_fallback_to_synthetic_when_no_matching_jes() {
    // Create JEs only on cash accounts (1000) but CRA is for revenue (4xxx)
    let entries: Vec<JournalEntry> = (0..10)
        .map(|i| {
            let id = format!("00000000-0000-0000-0000-{:012}", i);
            make_je(&id, "1000", dec!(10000), dec!(0))
        })
        .collect();

    let cra = datasynth_core::models::audit::CombinedRiskAssessment {
        id: "CRA-C001-REVENUE-Occurrence".to_string(),
        entity_code: "C001".to_string(),
        account_area: "Revenue".to_string(),
        assertion: datasynth_core::models::audit::AuditAssertion::Occurrence,
        combined_risk: datasynth_core::models::audit::CraLevel::High,
        ..Default::default()
    };

    let account_balances = HashMap::new();

    let mut gen = SamplingPlanGenerator::new(42);
    let (plans, items) = gen.generate_for_cras_with_population(
        &[cra],
        Some(dec!(50000)),
        &entries,
        &account_balances,
    );

    // Should still generate plans (fallback to synthetic)
    assert!(!plans.is_empty(), "Should generate plans even with no matching JEs");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p datasynth-generators --test sampling_coherence_tests -- --test-threads=4`
Expected: Compilation error — `generate_for_cras_with_population` doesn't exist yet

- [ ] **Step 3: Add account_area_to_prefixes helper**

In `crates/datasynth-generators/src/audit/sampling_plan_generator.rs`, add this helper function:

```rust
/// Map an audit account area name to GL account code prefixes.
/// Uses the CoA conventions: 1xxx Assets, 2xxx Liabilities, 3xxx Equity,
/// 4xxx Revenue, 5xxx COGS, 6xxx Expenses.
fn account_area_to_prefixes(account_area: &str) -> Vec<&'static str> {
    let lower = account_area.to_lowercase();
    if lower.contains("revenue") || lower.contains("sales") {
        vec!["4"]
    } else if lower.contains("receivable") {
        vec!["11"]
    } else if lower.contains("payable") {
        vec!["20"]
    } else if lower.contains("inventory") || lower.contains("stock") {
        vec!["12", "13"]
    } else if lower.contains("cash") || lower.contains("bank") {
        vec!["10"]
    } else if lower.contains("fixed asset") || lower.contains("ppe") || lower.contains("property") {
        vec!["14", "15", "16"]
    } else if lower.contains("equity") || lower.contains("capital") {
        vec!["3"]
    } else if lower.contains("expense") || lower.contains("cost") {
        vec!["5", "6"]
    } else if lower.contains("debt") || lower.contains("loan") || lower.contains("borrow") {
        vec!["23", "24"]
    } else if lower.contains("tax") {
        vec!["17", "25"]
    } else if lower.contains("provision") {
        vec!["26"]
    } else if lower.contains("intangible") || lower.contains("goodwill") {
        vec!["19"]
    } else {
        vec![] // Empty = use all JE lines
    }
}

/// Filter JE lines to those matching the account area's GL prefixes.
fn filter_je_lines_for_area<'a>(
    entries: &'a [datasynth_core::JournalEntry],
    account_area: &str,
) -> Vec<(&'a datasynth_core::JournalEntry, &'a datasynth_core::models::journal_entry::JournalEntryLine)> {
    let prefixes = account_area_to_prefixes(account_area);
    let mut results = Vec::new();

    for je in entries {
        for line in &je.lines {
            let matches = if prefixes.is_empty() {
                true // No filter — include all
            } else {
                prefixes.iter().any(|p| line.account_code.starts_with(p))
            };
            if matches {
                let amount = (line.debit_amount - line.credit_amount).abs();
                if amount > rust_decimal::Decimal::ZERO {
                    results.push((je, line));
                }
            }
        }
    }
    results
}
```

- [ ] **Step 4: Implement generate_for_cras_with_population**

Add the new method to `impl SamplingPlanGenerator`:

```rust
/// Generate sampling plans using real journal entry population data.
///
/// Key items are selected from JE lines with amount > tolerable_error.
/// Representative items are sampled from the remaining JE population.
/// Falls back to synthetic generation for CRAs with no matching JE lines.
pub fn generate_for_cras_with_population(
    &mut self,
    cras: &[CombinedRiskAssessment],
    tolerable_error: Option<Decimal>,
    journal_entries: &[datasynth_core::JournalEntry],
    account_balances: &std::collections::HashMap<String, f64>,
) -> (Vec<SamplingPlan>, Vec<SampledItem>) {
    use tracing::info;

    info!(
        "Generating JE-aware sampling plans for {} CRAs from {} journal entries",
        cras.len(),
        journal_entries.len()
    );

    let mut plans = Vec::new();
    let mut all_items = Vec::new();

    for cra in cras {
        if cra.combined_risk < CraLevel::Moderate {
            continue;
        }

        let te = tolerable_error
            .unwrap_or_else(|| self.config.base_population_value * dec!(0.05));

        // Filter JE lines for this CRA's account area
        let matching_lines = filter_je_lines_for_area(journal_entries, &cra.account_area);

        if matching_lines.is_empty() {
            // Fallback to synthetic generation
            let (plan, items) = self.generate_plan(cra, te);
            all_items.extend(items);
            plans.push(plan);
            continue;
        }

        let (plan, items) = self.generate_plan_from_population(cra, te, &matching_lines);
        all_items.extend(items);
        plans.push(plan);
    }

    info!(
        "Generated {} sampling plans with {} sampled items (JE-aware)",
        plans.len(),
        all_items.len()
    );
    (plans, all_items)
}

/// Generate a single sampling plan from real JE population data.
fn generate_plan_from_population(
    &mut self,
    cra: &CombinedRiskAssessment,
    tolerable_error: Decimal,
    lines: &[(&datasynth_core::JournalEntry, &datasynth_core::models::journal_entry::JournalEntryLine)],
) -> (SamplingPlan, Vec<SampledItem>) {
    let pop_size = lines.len();
    let pop_value: Decimal = lines
        .iter()
        .map(|(_, l)| (l.debit_amount - l.credit_amount).abs())
        .sum();

    let plan_id = format!(
        "SP-{}-{}-{}",
        cra.entity_code,
        cra.account_area.replace(' ', "_").to_uppercase(),
        format!("{:?}", cra.assertion)
    );

    let methodology = Self::methodology_for_cra(cra);
    let rep_sample_size = Self::sample_size_for_cra(cra);
    let misstatement_p = Self::misstatement_rate(cra.combined_risk);

    // Select key items: JE lines with amount > tolerable_error
    let mut key_items = Vec::new();
    let mut key_je_ids = std::collections::HashSet::new();
    let mut sorted_lines: Vec<_> = lines.iter()
        .map(|(je, l)| {
            let amount = (l.debit_amount - l.credit_amount).abs();
            (je, l, amount)
        })
        .collect();
    sorted_lines.sort_by(|a, b| b.2.cmp(&a.2)); // Descending by amount

    for (je, _line, amount) in &sorted_lines {
        if *amount <= tolerable_error {
            break; // Sorted descending, so no more key items
        }
        if key_items.len() >= 20 {
            break; // Cap at 20
        }
        let je_id = je.header.document_id.to_string();
        if key_je_ids.contains(&je_id) {
            continue; // Skip duplicates from same JE
        }
        key_je_ids.insert(je_id.clone());
        key_items.push(KeyItem {
            item_id: je_id,
            amount: *amount,
            reason: self.pick_key_item_reason(cra, key_items.len()),
        });
    }

    let key_items_value: Decimal = key_items.iter().map(|k| k.amount).sum();
    let remaining_value = pop_value - key_items_value;

    // Convert key items to SampledItem
    let mut sampled_items: Vec<SampledItem> = key_items
        .iter()
        .map(|ki| {
            let misstatement_found = self.rng.random::<f64>() < misstatement_p;
            let misstatement_amount = if misstatement_found {
                let pct = Decimal::try_from(self.rng.random_range(0.01_f64..=0.30_f64))
                    .unwrap_or(dec!(0.05));
                Some((ki.amount * pct).round_dp(2))
            } else {
                None
            };
            SampledItem {
                item_id: ki.item_id.clone(),
                sampling_plan_id: plan_id.clone(),
                amount: ki.amount,
                selection_type: SelectionType::KeyItem,
                tested: true,
                misstatement_found,
                misstatement_amount,
            }
        })
        .collect();

    // Select representative items from remaining JE population
    let remaining_lines: Vec<_> = sorted_lines
        .iter()
        .filter(|(je, _, _)| !key_je_ids.contains(&je.header.document_id.to_string()))
        .collect();

    let actual_rep_size = rep_sample_size.min(remaining_lines.len());
    if actual_rep_size > 0 && !remaining_lines.is_empty() {
        // Systematic selection from remaining population
        let step = remaining_lines.len() / actual_rep_size;
        let start = if step > 0 { self.rng.random_range(0..step) } else { 0 };

        for i in 0..actual_rep_size {
            let idx = (start + i * step).min(remaining_lines.len() - 1);
            let (je, _line, amount) = remaining_lines[idx];
            let je_id = je.header.document_id.to_string();

            let misstatement_found = self.rng.random::<f64>() < misstatement_p;
            let misstatement_amount = if misstatement_found {
                let pct = Decimal::try_from(self.rng.random_range(0.01_f64..=0.30_f64))
                    .unwrap_or(dec!(0.05));
                Some((*amount * pct).round_dp(2))
            } else {
                None
            };

            sampled_items.push(SampledItem {
                item_id: je_id,
                sampling_plan_id: plan_id.clone(),
                amount: *amount,
                selection_type: SelectionType::Representative,
                tested: true,
                misstatement_found,
                misstatement_amount,
            });
        }
    }

    let sampling_interval = if actual_rep_size > 0 {
        remaining_value / Decimal::from(actual_rep_size as i64)
    } else {
        Decimal::ZERO
    };

    let plan = SamplingPlan {
        id: plan_id,
        entity_code: cra.entity_code.clone(),
        account_area: cra.account_area.clone(),
        assertion: format!("{:?}", cra.assertion),
        methodology,
        population_size: pop_size,
        population_value: pop_value,
        key_items,
        key_items_value,
        remaining_population_value: remaining_value,
        sample_size: sampled_items.len(),
        sampling_interval,
        cra_level: format!("{:?}", cra.combined_risk),
        tolerable_error,
    };

    (plan, sampled_items)
}
```

Note: `methodology_for_cra`, `sample_size_for_cra`, and `misstatement_rate` are existing private methods. If they're currently methods on `self`, adjust accordingly. If they're associated functions, call them as `Self::method_name(...)`.

- [ ] **Step 5: Add necessary imports to sampling_plan_generator.rs**

At the top of the file, add:

```rust
use datasynth_core::JournalEntry;
```

And ensure `datasynth-core` is in `datasynth-generators/Cargo.toml` dependencies (it should be already).

- [ ] **Step 6: Run tests**

Run: `cargo test -p datasynth-generators --test sampling_coherence_tests -- --test-threads=4`
Expected: All 3 tests pass

- [ ] **Step 7: Run all existing sampling tests**

Run: `cargo test -p datasynth-generators sampling -- --test-threads=4`
Expected: All existing tests still pass

- [ ] **Step 8: Commit**

```bash
git add crates/datasynth-generators/src/audit/sampling_plan_generator.rs crates/datasynth-generators/tests/sampling_coherence_tests.rs
git commit -m "feat(sampling): implement JE-aware sampling with real document IDs and amounts"
```

---

### Task 6: Wire JE-aware sampling into StepDispatcher

**Files:**
- Modify: `crates/datasynth-audit-fsm/src/dispatch.rs`

- [ ] **Step 1: Update dispatch_sampling to use JE data when available**

Replace the existing `dispatch_sampling` method:

```rust
fn dispatch_sampling(&mut self, ctx: &EngagementContext, bag: &mut ArtifactBag) {
    if bag.combined_risk_assessments.is_empty() {
        warn!("dispatch_sampling: no CRAs in bag — generating CRAs first");
        self.dispatch_cra(ctx, bag);
    }

    let tolerable_error = bag
        .materiality_calculations
        .first()
        .map(|m| m.performance_materiality);

    if !ctx.journal_entries.is_empty() {
        // Coherent path: use real JE population
        let (plans, items) = self.sampling_gen.generate_for_cras_with_population(
            &bag.combined_risk_assessments,
            tolerable_error,
            &ctx.journal_entries,
            &ctx.account_balances,
        );
        bag.sampling_plans.extend(plans);
        bag.sampled_items.extend(items);
    } else {
        // Fallback: synthetic generation (backward compatible)
        let (plans, items) = self
            .sampling_gen
            .generate_for_cras(&bag.combined_risk_assessments, tolerable_error);
        bag.sampling_plans.extend(plans);
        bag.sampled_items.extend(items);
    }
}
```

- [ ] **Step 2: Add HashMap import if needed**

Ensure `use std::collections::HashMap;` is imported at the top of `dispatch.rs` (it likely already is since `EngagementContext` uses it).

- [ ] **Step 3: Verify compilation**

Run: `cargo check -p datasynth-audit-fsm`
Expected: Compiles

- [ ] **Step 4: Run all FSM tests**

Run: `cargo test -p datasynth-audit-fsm -- --test-threads=4`
Expected: All pass

- [ ] **Step 5: Commit**

```bash
git add crates/datasynth-audit-fsm/src/dispatch.rs
git commit -m "feat(audit-fsm): wire JE-aware sampling into StepDispatcher"
```

---

### Task 7: End-to-end coherence integration test

**Files:**
- Modify: `crates/datasynth-audit-fsm/tests/financial_coherence_tests.rs`

- [ ] **Step 1: Write end-to-end coherence test**

Add to `crates/datasynth-audit-fsm/tests/financial_coherence_tests.rs`:

```rust
use datasynth_audit_fsm::context::EngagementContext;
use datasynth_audit_fsm::engine::AuditFsmEngine;
use datasynth_audit_fsm::loader::{BlueprintWithPreconditions, BuiltinOverlay, OverlaySource, load_overlay};
use datasynth_core::models::JournalEntry;
use datasynth_core::models::journal_entry::*;
use chrono::{NaiveDate, Utc};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rust_decimal_macros::dec;
use smallvec::smallvec;
use std::collections::HashMap;
use uuid::Uuid;

fn make_test_je(account_code: &str, debit: rust_decimal::Decimal, credit: rust_decimal::Decimal) -> JournalEntry {
    let doc_id = Uuid::new_v4();
    JournalEntry {
        header: JournalEntryHeader {
            document_id: doc_id,
            company_code: "C001".to_string(),
            fiscal_year: 2024,
            fiscal_period: 6,
            posting_date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            document_date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            created_at: Utc::now(),
            document_type: "SA".to_string(),
            currency: "USD".to_string(),
            exchange_rate: dec!(1),
            created_by: "TEST".to_string(),
            ..Default::default()
        },
        lines: smallvec![JournalEntryLine {
            document_id: doc_id,
            line_number: 1,
            gl_account: account_code.to_string(),
            account_code: account_code.to_string(),
            debit_amount: debit,
            credit_amount: credit,
            local_amount: debit - credit,
            ..Default::default()
        }],
    }
}

#[test]
fn test_fsm_sampling_uses_real_je_ids() {
    // Create a realistic JE population
    let mut entries = Vec::new();

    // Revenue JEs (4000) — mix of large and small
    for _ in 0..5 {
        entries.push(make_test_je("4000", dec!(0), dec!(200000))); // Large revenue
    }
    for _ in 0..20 {
        entries.push(make_test_je("4000", dec!(0), dec!(15000))); // Small revenue
    }
    // AR JEs (1100)
    for _ in 0..30 {
        entries.push(make_test_je("1100", dec!(25000), dec!(0)));
    }
    // Expense JEs
    for _ in 0..15 {
        entries.push(make_test_je("5000", dec!(30000), dec!(0)));
        entries.push(make_test_je("6100", dec!(10000), dec!(0)));
    }

    // Collect all JE document IDs for later verification
    let all_je_ids: std::collections::HashSet<String> = entries
        .iter()
        .map(|e| e.header.document_id.to_string())
        .collect();

    // Build account balances
    let mut account_balances = HashMap::new();
    for je in &entries {
        for line in &je.lines {
            let d: f64 = line.debit_amount.to_string().parse().unwrap_or(0.0);
            let c: f64 = line.credit_amount.to_string().parse().unwrap_or(0.0);
            *account_balances.entry(line.account_code.clone()).or_insert(0.0) += d - c;
        }
    }

    // Build context with real JE data
    let context = EngagementContext {
        company_code: "C001".to_string(),
        company_name: "Test Corp".to_string(),
        fiscal_year: 2024,
        currency: "USD".to_string(),
        total_revenue: dec!(1300000),
        total_assets: dec!(750000),
        engagement_start: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        report_date: NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        pretax_income: dec!(400000),
        equity: dec!(500000),
        gross_profit: dec!(455000),
        working_capital: dec!(300000),
        operating_cash_flow: dec!(340000),
        total_debt: dec!(200000),
        team_member_ids: vec!["EMP001".to_string()],
        team_member_pairs: vec![("EMP001".to_string(), "Auditor One".to_string())],
        accounts: vec!["4000".to_string(), "1100".to_string(), "5000".to_string(), "6100".to_string()],
        vendor_names: vec!["Vendor A".to_string()],
        customer_names: vec!["Customer B".to_string()],
        journal_entry_ids: entries.iter().take(50).map(|e| e.header.document_id.to_string()).collect(),
        account_balances,
        control_ids: vec![],
        anomaly_refs: vec![],
        is_us_listed: false,
        entity_codes: vec!["C001".to_string()],
        journal_entries: entries,
    };

    // Run FSM with builtin FSA blueprint
    let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
    bwp.validate().unwrap();
    let overlay = load_overlay(&OverlaySource::Builtin(BuiltinOverlay::Default)).unwrap();
    let rng = ChaCha8Rng::seed_from_u64(42);
    let mut engine = AuditFsmEngine::new(bwp, overlay, rng);
    let result = engine.run_engagement(&context).unwrap();

    // COHERENCE CHECK: Every sampled item ID should be a real JE document_id
    let sampled = &result.artifacts.sampled_items;
    if !sampled.is_empty() {
        let mut real_id_count = 0;
        for item in sampled {
            if all_je_ids.contains(&item.item_id) {
                real_id_count += 1;
            }
        }
        // At least some items should reference real JE IDs
        // (some CRAs may fall back to synthetic if no matching JEs)
        assert!(
            real_id_count > 0,
            "Expected at least some sampled items to reference real JE document_ids, but none did. {} items total.",
            sampled.len()
        );

        // Key items should all have amounts > performance materiality (if materiality was computed)
        if let Some(mat) = result.artifacts.materiality_calculations.first() {
            let te = mat.performance_materiality;
            let key_items: Vec<_> = sampled.iter()
                .filter(|i| i.selection_type == datasynth_core::models::audit::SelectionType::KeyItem)
                .filter(|i| all_je_ids.contains(&i.item_id)) // Only check real-JE key items
                .collect();
            for ki in &key_items {
                assert!(
                    ki.amount > te,
                    "Key item {} has amount {} which should be > tolerable error {}",
                    ki.item_id, ki.amount, te
                );
            }
        }
    }
}
```

- [ ] **Step 2: Run the coherence test**

Run: `cargo test -p datasynth-audit-fsm --test financial_coherence_tests -- --test-threads=4`
Expected: All tests pass

- [ ] **Step 3: Run full crate test suite**

Run: `cargo test -p datasynth-audit-fsm -- --test-threads=4`
Expected: All tests pass (including existing ones)

- [ ] **Step 4: Commit**

```bash
git add crates/datasynth-audit-fsm/tests/financial_coherence_tests.rs
git commit -m "test(audit-fsm): add end-to-end financial coherence integration test"
```

---

### Task 8: Verify full workspace and run clippy

**Files:**
- No new files

- [ ] **Step 1: Run clippy on affected crates**

Run: `cargo clippy -p datasynth-generators -p datasynth-audit-fsm -p datasynth-runtime -- -D warnings 2>&1 | tail -20`
Expected: No errors (fix any warnings)

- [ ] **Step 2: Run all tests across affected crates**

Run: `cargo test -p datasynth-generators -p datasynth-audit-fsm -- --test-threads=4`
Expected: All pass

- [ ] **Step 3: Run cargo fmt**

Run: `cargo fmt --check`
Expected: No formatting issues (or run `cargo fmt` to fix)

- [ ] **Step 4: Commit any fixes**

```bash
git add -A
git commit -m "style: fix clippy warnings and formatting"
```

---

## What's Next

This plan covers the core coherence wiring (Layers 1-3 from the spec). Layer 4 (full data chain coherence for risk assessments, analytical procedures, going concern, and opinions) can be done as a follow-up plan since those are additive enhancements that don't affect the core data flow established here.
