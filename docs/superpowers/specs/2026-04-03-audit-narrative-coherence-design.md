# Audit Narrative Artifact Coherence

**Date**: 2026-04-03
**Status**: Draft
**Scope**: Enhance the four qualitative audit generators (subsequent events, judgments, evidence, workpapers) to reference actual audit artifacts from the ArtifactBag and real financial data from EngagementContext, producing interconnected narrative output.

## Problem

The qualitative audit generators produce artifacts disconnected from the actual audit engagement:

1. **Subsequent events** generate random event types with random $10k-$5M financial impacts, ignoring the company's actual risk profile and financial condition
2. **Professional judgments** produce generic conclusions about materiality, risk, and going concern without referencing the actual amounts, risk levels, or assessments already generated
3. **Evidence** records are randomly typed (confirmation, invoice, bank statement) without matching the workpaper's assertion or risk area, and amounts in AI-extracted data are synthetic
4. **Workpaper** objectives and procedures use generic templates without referencing actual account balances, materiality thresholds, or sampling parameters

The root cause: dispatch methods pass only `EngagementContext` (financial metadata) but not `ArtifactBag` (actual audit work products), so generators can't reference what the audit actually found.

## Solution

Enrich each generator's dispatch call to pass relevant bag artifacts and financial context. Each generator gets a new `_with_context` method that accepts the data it needs to produce coherent output. Existing methods remain as fallbacks.

### Enhancement 1: Subsequent Events — Risk-Weighted with Real Financial Scale

**Current**: `generate_for_entity(entity_code, period_end_date)` → random events, random amounts

**Enhanced**: `generate_for_entity_with_context(entity_code, period_end_date, input)` where input includes:
- `total_revenue: Decimal` — for scaling financial impact proportionally
- `total_assets: Decimal` — for scaling asset-related events
- `high_risk_areas: Vec<String>` — from CRAs with High/Moderate combined_risk, to weight event type selection
- `going_concern_doubt: bool` — from going concern assessment, increases probability and severity
- `pretax_income: Decimal` — negative income increases probability of adverse events

**Behavior changes**:
- Financial impact scaled as 1-5% of revenue/assets (not random $10k-$5M)
- Event types weighted toward high-risk CRA areas (if "Inventory" is high risk, favor AssetImpairment)
- Going concern doubt increases event count and adjusting-event probability
- Loss-making companies get higher probability of adverse events (CustomerBankruptcy, LitigationSettlement)

### Enhancement 2: Professional Judgments — Reference Actual Audit Results

**Current**: `generate_judgment(engagement, team_members)` → generic conclusions

**Enhanced**: `generate_judgment_with_context(engagement, team_members, context)` where context includes:
- `materiality: Option<&MaterialityCalculation>` — for materiality judgment: cite actual amount, basis, percentage
- `high_risk_cras: Vec<&CombinedRiskAssessment>` — for risk judgment: cite actual risk levels and account areas
- `going_concern: Option<&GoingConcernAssessment>` — for GC judgment: cite actual indicators and conclusion
- `finding_count: usize` — for misstatement judgment: cite actual number and nature of findings
- `sampling_exception_rate: Option<f64>` — for sampling judgment: cite actual exception rate

**Behavior changes**:
- Materiality judgment: "Performance materiality set at $X (Y% of Z benchmark)" using real numbers
- Risk judgment: "Inherent risk assessed as High for Revenue ($Xm) due to..." referencing actual CRA
- Going concern: conclusion matches actual assessment (not always "no doubt")
- Fraud risk: if findings exist with fraud indicators, judgment acknowledges them
- Sampling: references actual population size, sample size, exception count

### Enhancement 3: Evidence — Type-Matched to Assertion and Risk

**Current**: `generate_evidence_for_workpaper(workpaper, team, date)` → random types/amounts

**Enhanced**: `generate_evidence_for_workpaper_with_context(workpaper, team, date, context)` where context includes:
- `risk_level: Option<CraLevel>` — from CRA matching the workpaper's account area
- `account_balance: Option<f64>` — from account_balances for the workpaper's GL area
- `assertion: Option<String>` — from the workpaper's tested assertion

**Behavior changes**:
- High-risk areas get more external evidence (confirmations, specialist reports) — higher reliability per ISA 500
- Evidence amounts anchored to real account balances (bank statement shows real cash balance, not random)
- Assertion-matched types: Existence → confirmation/observation; Completeness → analytical/cutoff test; Valuation → specialist report/recalculation

### Enhancement 4: Workpapers — Parameterized Objectives with Real Data

**Current**: `generate_workpaper(engagement, section, date, team)` → generic titles/objectives

**Enhanced**: `generate_workpaper_with_context(engagement, section, date, team, context)` where context includes:
- `account_area: Option<String>` — from the procedure being executed
- `account_balance: Option<Decimal>` — real GL balance for the area
- `risk_level: Option<CraLevel>` — from matching CRA
- `materiality: Option<Decimal>` — performance materiality
- `sampling_plan: Option<&SamplingPlan>` — for testing workpapers: population, sample size, method

**Behavior changes**:
- Title: "Revenue Substantive Testing — $2.5M (High Risk)" instead of generic "Revenue Cutoff Testing"
- Objective: "Obtain evidence that revenue ($2.5M, 35% of total revenue) is not materially misstated. Performance materiality: $150K."
- Procedure: "Selected sample of 45 items from population of 850 revenue transactions using MUS" (from actual SamplingPlan)
- Scope: Derived from actual CRA level (High → 100% coverage, Medium → 80%, Low → 60%)

## Dispatch Changes

Each dispatch method gets an enhanced path that reads from the bag:

```rust
fn dispatch_subsequent_events(&mut self, ctx: &EngagementContext, bag: &mut ArtifactBag) {
    let high_risk_areas: Vec<String> = bag.combined_risk_assessments.iter()
        .filter(|c| c.combined_risk >= CraLevel::Moderate)
        .map(|c| c.account_area.clone())
        .collect();
    let gc_doubt = bag.going_concern_assessments.last()
        .map(|gc| gc.has_material_uncertainty)
        .unwrap_or(false);
    
    let input = SubsequentEventInput {
        total_revenue: ctx.total_revenue,
        total_assets: ctx.total_assets,
        pretax_income: ctx.pretax_income,
        high_risk_areas,
        going_concern_doubt: gc_doubt,
    };
    let events = self.se_gen.generate_for_entity_with_context(
        &ctx.company_code, ctx.report_date, &input);
    bag.subsequent_events.extend(events);
}
```

Similar pattern for judgment, evidence, and workpaper dispatchers — extract what's needed from `bag` and `ctx`, build an input struct, call the `_with_context` method.

## Files Changed

| File | Change |
|------|--------|
| `crates/datasynth-generators/src/audit/subsequent_event_generator.rs` | Add `SubsequentEventInput`, `generate_for_entity_with_context` method |
| `crates/datasynth-generators/src/audit/judgment_generator.rs` | Add `JudgmentContext`, `generate_judgment_with_context` method |
| `crates/datasynth-generators/src/audit/evidence_generator.rs` | Add `EvidenceContext`, `generate_evidence_for_workpaper_with_context` method |
| `crates/datasynth-generators/src/audit/workpaper_generator.rs` | Add `WorkpaperContext`, `generate_workpaper_with_context` method |
| `crates/datasynth-audit-fsm/src/dispatch.rs` | Update 4 dispatch methods to extract bag context and call enhanced methods |

## Testing Strategy

| Test | What It Verifies |
|------|------------------|
| Subsequent event financial impact scales with revenue | Impact is 1-5% of revenue, not random $10k-$5M |
| Subsequent event types weighted by CRA risk areas | High-risk inventory → more AssetImpairment events |
| Judgment materiality references real amount | Output contains actual materiality figure |
| Judgment going concern matches assessment | Doubt → judgment acknowledges doubt |
| Evidence type matches assertion | Existence assertion → confirmation/observation evidence |
| Evidence amounts anchored to GL balance | Bank evidence amount near real cash balance |
| Workpaper objective contains real balance | "$2.5M" appears in objective text |
| Workpaper sampling references real plan | Population/sample size from actual SamplingPlan |
| Backward compat: empty bag falls back | No CRAs/materiality → original synthetic behavior |
