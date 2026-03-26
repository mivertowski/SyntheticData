# GAM Blueprint Integration Plan

**Date**: 2026-03-25
**Status**: Pending (blocked on AuditMethodology repo alignment)
**Scope**: Load and run the full EY GAM blueprint (1,182 procedures, 3,035 steps) in the FSM engine.

## Pre-requisites (AuditMethodology Repo)

These must be fixed in the AuditMethodology exporter before SyntheticData can load the GAM blueprint cleanly:

### 1. Nest procedures under phases (BLOCKING)

**Current**: Procedures are in a flat `procedures: [1182]` list at the top level. Phases exist but contain `procedures: []` (empty).

**Required**: Each procedure has a `phase` field referencing its parent phase. The exporter should group procedures by phase and nest them, matching the FSA/IA pattern:

```yaml
phases:
  - id: initial_planning
    procedures:
      - id: proc_1682192_s142
        # ...
  - id: identify_and_assess_risks
    procedures:
      - id: proc_1682XXX
        # ...
```

**Where to fix**: `src/gam_scraper/export/datasynth_exporter.py` (or whichever exporter produces the blueprint YAML). Group `procedures` by their `phase` field before serialization.

### 2. Generate preconditions from phase ordering (BLOCKING)

**Current**: 0/1182 procedures have preconditions. No dependency ordering at all.

**Required**: At minimum, inter-phase preconditions based on the natural audit workflow:

```
initial_planning → identify_and_assess_risks → design_and_execute_responses → conclude_and_communicate
```

Within each phase, procedures should depend on earlier procedures in the same phase (sequential by section ordering in GAM).

**Where to fix**: Same exporter. For each procedure, set `preconditions` to include:
- The last procedure of the preceding phase (inter-phase dependency)
- Earlier procedures in the same GAM section (intra-phase dependency)

The GAM topic hierarchy provides natural ordering — topics within a section have an implicit sequence.

### 3. Align discriminator format (NICE TO HAVE)

**Current GAM**:
```yaml
discriminators:
  tiers: [complex, core, digital, non_complex]
  overlays: [group_audit, listed, pcaob_fs, pcaob_ia]
```

**FSA/IA pattern**:
```yaml
discriminators:
  tiers: [core]
  categories: [financial, operational]
```

The GAM uses "tiers" and "overlays" (EY-specific), while FSA/IA use "tiers" and "categories". The engine handles both via HashMap<String, Vec<String>>, but consistency would be cleaner.

---

## SyntheticData Implementation (after source fixes)

### Phase 1: Make it loadable

1. **Schema additions** — add `#[serde(default)]` for GAM-specific step fields:
   - `isa_mandate: Option<String>` — whether step is ISA-mandated
   - `form_refs: Vec<FormRef>` — links to EY forms
   - `deliverable_fields: Vec<String>` — expected output fields
   - `standard_field_trace: Vec<String>` — ISA paragraph tracing

2. **Feature-gated loading** — `gam-blueprint` feature flag:
   ```toml
   [features]
   gam-blueprint = []
   ```
   Load from filesystem (not `include_str!` — 13MB is too large for embedding):
   ```rust
   #[cfg(feature = "gam-blueprint")]
   pub fn load_gam_from_path(path: &Path) -> Result<BlueprintWithPreconditions, AuditFsmError>
   ```

3. **Evidence catalog handling** — GAM has 1,702 evidence templates (vs 10 for FSA, 28 for IA). The engine's evidence_states HashMap scales fine, but the enrichment lookups should be indexed.

4. **Validation** — verify GAM passes `validate_blueprint_with_preconditions()` after source fixes.

### Phase 2: Make it run

5. **Legal documents data type** — new model + generator:
   ```rust
   pub struct LegalDocument {
       pub document_id: Uuid,
       pub document_type: String,  // engagement_letter, management_rep, legal_opinion, regulatory_filing
       pub entity_code: String,
       pub date: NaiveDate,
       pub signatories: Vec<String>,
       pub key_terms: Vec<String>,
   }
   ```
   Used in 686/3,035 GAM steps. Generate ~5-10 per engagement.

6. **Command prefix dispatch** — 2,102 unique commands. Rather than mapping each individually, parse prefixes:
   | Prefix | Generator | Count |
   |--------|-----------|-------|
   | `provide_` | Workpaper | ~600 |
   | `evaluate_` | RiskAssessment/CRA | ~300 |
   | `determine_` | Materiality/Judgment | ~200 |
   | `perform_` | Sampling/Testing | ~250 |
   | `assess_` | Risk/CRA | ~150 |
   | `consider_` | Judgment | ~200 |
   | `obtain_` | Evidence/Confirmation | ~100 |
   | `document_` | Evidence | ~100 |
   | `review_` | Judgment | ~100 |
   | Other | Generic workpaper | ~100 |

7. **Run and verify** — full GAM engagement, check completion rate, event/artifact counts.

### Phase 3: Make it rich

8. **GAM analytics inventory** (7MB, 3,007 steps) — load and wire into StepDispatcher for data-driven content enrichment.

9. **Form-aware evidence** — use `form_refs` and `deliverable_fields` on steps to generate evidence with the correct field structure per EY form template.

10. **Standard dependencies** — 335 ISA cross-references for enhanced coverage reporting.

11. **Performance** — streaming artifact export for 50K+ artifacts. Consider:
    - Batched JSON write (1000 artifacts per flush)
    - Progress reporting for long runs
    - Memory profiling at 1,182 procedures

### Expected Metrics

| Metric | Estimate |
|--------|----------|
| Events | 6,000-8,000 |
| Artifacts | 50,000+ |
| Duration | 5-15 seconds |
| Memory | ~200-500MB peak |
| Output files | All existing + `legal_documents/` |

---

## Data Coverage Check

| GAM Data Type | SyntheticData Status |
|---------------|---------------------|
| general_ledger | Covered |
| journal_entries | Covered (with ISA 240 flags) |
| financial_statements | Covered (with comparatives) |
| sub_ledger | Covered |
| confirmations | Covered |
| contracts | Covered |
| management_reports | Covered |
| organizational | Covered |
| prior_year | Covered |
| **legal_documents** | **Not yet — Phase 2 deliverable** |

| GAM Analytical Procedure | SyntheticData Status |
|--------------------------|---------------------|
| trend_analysis | Covered |
| ratio_analysis | Covered |
| three_way_match | Covered |
| sampling | Covered |
| journal_entry_testing | Covered |
| stratification | Covered |
| variance_analysis | Covered |
| expectation_model | Covered |
| visualization | N/A (export format) |

**9/10 data types covered, 8/8 analytical procedures covered.**
Only `legal_documents` needs implementation (Phase 2).
