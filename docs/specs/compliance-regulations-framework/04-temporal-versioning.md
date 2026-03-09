# Part 4: Temporal Versioning & Change Management

> **Parent:** [Compliance & Regulations Framework](00-index.md)
> **Status:** Draft | **Date:** 2026-03-09

---

## 4.1 Overview

Regulatory standards are not static — they evolve through amendments, replacements, and new issuances. The temporal versioning system ensures that DataSynth generates compliance data that accurately reflects the regulatory landscape **at any point in time**. This is critical for:

- **Training ML models** on historical data with historically-correct regulatory context
- **Testing compliance systems** against past, current, and future regulatory states
- **Simulating transition periods** where old and new standards coexist
- **Backtesting** audit analytics against known regulatory regimes

---

## 4.2 Temporal Model

### 4.2.1 Core Concepts

```
Timeline:  ────────────────────────────────────────────────────────▶

IAS 17:    ═══════════════════════════════════════╗
           1997-01-01                    2019-01-01│ (superseded)
                                                   │
IFRS 16:                        ┌────────early─────╠════════════▶
                          2016-01-13        2019-01-01
                          (issued)    (effective / mandatory)

           ◄─── IAS 17 active ────►◄── transition ►◄── IFRS 16 ──►
```

**Key temporal concepts:**

| Concept | Description |
|---------|------------|
| **Issuance Date** | When the standard was published by the issuing body |
| **Early Adoption Date** | Earliest date an entity may voluntarily apply the standard |
| **Effective Date (Global)** | Date the standard becomes mandatory globally |
| **Effective Date (Local)** | Jurisdiction-specific effective date (may differ from global) |
| **Transition Period** | Window between early adoption and mandatory application |
| **Superseded Date** | Date the old standard ceases to be applicable |
| **Amendment Date** | Date a targeted change to an existing standard takes effect |
| **Sunset Date** | Date a transitional provision or exemption expires |

### 4.2.2 Data Model

```rust
/// A temporal version of a standard.
pub struct TemporalVersion {
    /// Version identifier (e.g., "2018", "2020-amended", "2023-revised")
    pub version_id: String,
    /// Date this version was issued/published
    pub issued_date: NaiveDate,
    /// Date this version becomes available for early adoption
    pub early_adoption_from: Option<NaiveDate>,
    /// Date this version becomes mandatory (global default)
    pub effective_from: NaiveDate,
    /// Date this version is superseded (None = currently active)
    pub superseded_at: Option<NaiveDate>,
    /// Per-jurisdiction effective date overrides
    pub jurisdiction_overrides: HashMap<String, NaiveDate>,
    /// Transitional provisions and their sunset dates
    pub transitional_provisions: Vec<TransitionalProvision>,
    /// Key changes from the previous version
    pub changes: Vec<StandardChange>,
    /// Impact level
    pub impact: ChangeImpact,
}

/// A specific change within a standard version.
pub struct StandardChange {
    /// Change identifier
    pub id: String,
    /// Description of the change
    pub description: String,
    /// Affected requirements
    pub affected_requirements: Vec<String>,
    /// Impact on generated data
    pub data_impact: DataImpact,
}

/// How a change affects generated data.
pub enum DataImpact {
    /// New fields or records are generated
    NewOutput { fields: Vec<String> },
    /// Existing field values change
    ModifiedValues { fields: Vec<String>, description: String },
    /// Fields or records are removed
    RemovedOutput { fields: Vec<String> },
    /// Classification logic changes
    ReclassificationRequired { description: String },
    /// Measurement methodology changes
    MeasurementChange { description: String },
    /// Disclosure-only (no measurement impact)
    DisclosureOnly,
}

/// A transitional provision with an expiration date.
pub struct TransitionalProvision {
    /// Provision identifier
    pub id: String,
    /// Description
    pub description: String,
    /// When this provision becomes available
    pub available_from: NaiveDate,
    /// When this provision expires
    pub sunset_date: NaiveDate,
    /// Entities that elected this provision
    pub election_probability: f64,
}
```

---

## 4.3 Timeline of Major Standard Changes

The registry includes the following temporal events, enabling generation for any target date:

### 4.3.1 IFRS Timeline

| Date | Event | Impact |
|------|-------|--------|
| 2005-01-01 | EU mandates IFRS for listed entities | Framework selection |
| 2009-11-12 | IFRS 9 Phase 1 (Classification) issued | Financial instrument classification |
| 2013-01-01 | IFRS 10/11/12 effective (Consolidation package) | Group accounting |
| 2013-01-01 | IFRS 13 effective (Fair Value) | Valuation hierarchy |
| 2014-05-28 | IFRS 15 issued (Revenue) | 5-step revenue model |
| 2016-01-13 | IFRS 16 issued (Leases) | ROU asset recognition |
| 2018-01-01 | **IFRS 9 effective** — replaces IAS 39 | Expected credit loss model |
| 2018-01-01 | **IFRS 15 effective** — replaces IAS 18 + IAS 11 | Revenue recognition overhaul |
| 2019-01-01 | **IFRS 16 effective** — replaces IAS 17 | Lease on-balance-sheet |
| 2020-05-14 | IFRS 16 COVID-19 rent concession amendment | Lease modification simplification |
| 2023-01-01 | **IFRS 17 effective** — replaces IFRS 4 | Insurance contract measurement |
| 2023-01-23 | IAS 12 amendment: Pillar Two (deferred tax exemption) | BEPS impact |
| 2025-01-01 | IFRS 18 early adoption permitted | P&L presentation changes |
| 2027-01-01 | **IFRS 18 effective** — replaces IAS 1 | New P&L categories |

### 4.3.2 US GAAP Timeline

| Date | Event | Impact |
|------|-------|--------|
| 2002-07-30 | Sarbanes-Oxley Act enacted | ICFR requirements |
| 2006-01-01 | ASC 718 effective (Share-based Payment) | Stock comp expense |
| 2008-01-01 | ASC 820 effective (Fair Value) | 3-level hierarchy |
| 2009-01-01 | ASC 805/810 effective (Business Combinations) | Acquisition accounting |
| 2018-01-01 | **ASC 606 effective** (public entities) | Revenue recognition |
| 2019-01-01 | **ASC 842 effective** (public entities) | Lease accounting |
| 2020-01-01 | **ASC 326 effective** (large public) — CECL | Expected credit losses |
| 2023-01-01 | ASC 326 effective for smaller reporting companies | Broader CECL adoption |
| 2024-12-15 | ASC 842-10-65-8 (embedded leases update) | Lease identification |

### 4.3.3 Audit Standards Timeline

| Date | Event | Impact |
|------|-------|--------|
| 2002-07-30 | PCAOB established (SOX) | US audit oversight |
| 2007-12-20 | PCAOB AS 2201 effective (ICFR audit) | Integrated audit |
| 2016-06-17 | EU Audit Reform effective | PIE requirements |
| 2017-06-01 | PCAOB AS 3101 effective (new auditor's report with CAM) | CAM reporting |
| 2019-12-15 | ISA 315 (Revised 2019) effective | Enhanced risk assessment |
| 2020-12-15 | ISA 220 (Revised 2020) effective | Quality management |
| 2022-12-15 | ISA 600 (Revised) effective | Group audits |
| 2023-12-15 | ISQM 1 and ISQM 2 effective | Quality management systems |

### 4.3.4 Regulatory Timeline

| Date | Event | Impact |
|------|-------|--------|
| 2013-01-01 | Basel III capital requirements phased in | CET1 ratios |
| 2015-01-01 | Basel III LCR effective | Liquidity requirements |
| 2016-06-17 | EU Audit Regulation effective | Audit rotation, non-audit services |
| 2018-01-01 | MiFID II effective | Investment services |
| 2018-05-25 | GDPR effective | Data protection |
| 2021-03-10 | EU SFDR effective | Sustainability disclosure |
| 2022-01-01 | EU Taxonomy effective | Green classification |
| 2024-01-01 | EU CSRD effective (Phase 1: large PIEs) | Sustainability reporting |
| 2025-01-01 | **Basel IV / Basel III Final** effective | Output floors, revised SA |
| 2025-01-01 | EU CSRD Phase 2 (large non-PIE entities) | Broader ESG reporting |
| 2025-01-17 | EU DORA effective | ICT resilience |
| 2026-01-01 | EU CSRD Phase 3 (listed SMEs) | SME ESG reporting |
| 2028-06-30 | Basel IV full phase-in | Complete output floor |

---

## 4.4 Temporal Resolution Algorithm

```rust
impl TemporalResolver {
    /// Resolve the applicable regulatory state for a given date and jurisdiction.
    ///
    /// Returns the set of active standards, each pinned to a specific version.
    pub fn resolve(
        &self,
        jurisdiction: &str,
        target_date: NaiveDate,
        entity_config: &EntityConfig,
    ) -> ResolvedRegulatoryState {
        let mut state = ResolvedRegulatoryState::new(target_date);

        // 1. Get jurisdiction profile
        let profile = self.registry.jurisdiction_profile(jurisdiction);

        // 2. For each mandatory standard in the profile
        for js in &profile.mandatory_standards {
            // 2a. Check applicability criteria
            if !js.applicability.matches(entity_config) {
                continue;
            }

            // 2b. Determine effective date (local override or global)
            let effective = js.local_effective_date;

            // 2c. Skip if not yet effective
            if target_date < effective {
                continue;
            }

            // 2d. Resolve the active version at target_date
            let standard = self.registry.get(&js.standard_id);
            let version = self.resolve_version(standard, jurisdiction, target_date);

            // 2e. Check for supersession
            if let Some(superseded_by) = &standard.superseded_by {
                let successor = self.registry.get(superseded_by);
                let successor_effective = self.effective_date(successor, jurisdiction);

                if target_date >= successor_effective {
                    // Old standard is no longer active; successor takes over
                    // But check for transitional provisions
                    let transitions = self.active_transitions(standard, target_date);
                    if transitions.is_empty() {
                        continue; // Fully superseded
                    }
                    // Partial supersession: old standard still active for transition items
                    state.add_transitional(js.standard_id.clone(), transitions);
                    continue;
                }
            }

            // 2f. Add to resolved state
            state.add_standard(ResolvedStandard {
                id: js.standard_id.clone(),
                version: version.clone(),
                local_designation: js.local_designation.clone(),
                modifications: js.modifications.clone(),
            });
        }

        // 3. Apply supranational regulations
        for membership in &profile.memberships {
            let supra_standards = self.supranational_standards(membership, target_date);
            for std in supra_standards {
                if !state.contains(&std.id) {
                    state.add_standard(std);
                }
            }
        }

        // 4. Apply entity-specific overrides
        if let Some(overrides) = &entity_config.compliance_overrides {
            for additional in &overrides.additional_standards {
                if let Some(std) = self.registry.get_optional(additional) {
                    state.add_standard(self.resolve_standard(std, jurisdiction, target_date));
                }
            }
            for excluded in &overrides.exclude_standards {
                state.remove_standard(excluded);
            }
        }

        state
    }
}
```

---

## 4.5 Transition Period Modeling

### 4.5.1 Parallel Reporting

During transition periods, entities may need to report under both old and new standards. The framework models this:

```yaml
compliance_regulations:
  temporal:
    target_date: "2018-06-30"
    transition_modeling:
      enabled: true
      # Generate data under both IAS 39 and IFRS 9 for entities in transition
      parallel_standards:
        - old: "IAS-39"
          new: "IFRS-9"
          transition_probability: 0.60  # 60% of entities have transitioned by this date
```

### 4.5.2 Early Adoption Simulation

```yaml
compliance_regulations:
  temporal:
    early_adoption:
      enabled: true
      # Probability that an entity early-adopts each standard
      standards:
        - id: "IFRS-17"
          early_adopt_probability: 0.05  # 5% early adopt
        - id: "IFRS-18"
          early_adopt_probability: 0.02  # 2% early adopt
```

### 4.5.3 Amendment Propagation

When a standard is amended, the framework tracks which requirements are affected:

```rust
/// An amendment to an existing standard version.
pub struct StandardAmendment {
    /// Amendment identifier
    pub id: String,
    /// Title of the amendment
    pub title: String,
    /// Effective date
    pub effective_from: NaiveDate,
    /// Which version this amends
    pub amends_version: String,
    /// Specific requirements affected
    pub affected_requirements: Vec<String>,
    /// How the amendment changes generation
    pub generation_changes: Vec<GenerationChange>,
}

pub enum GenerationChange {
    /// A parameter value changes
    ParameterChange {
        parameter: String,
        old_value: String,
        new_value: String,
    },
    /// A new field is added to output
    NewField {
        entity: String,
        field_name: String,
        field_type: String,
    },
    /// A classification rule changes
    ClassificationChange {
        entity: String,
        old_rule: String,
        new_rule: String,
    },
    /// A threshold changes
    ThresholdChange {
        threshold_name: String,
        old_value: f64,
        new_value: f64,
    },
}
```

---

## 4.6 Temporal Generation Examples

### Example 1: Pre-IFRS 16 (Target Date: 2018-12-31)

When `target_date: "2018-12-31"` and the entity is IFRS-reporting:

- **Lease accounting**: IAS 17 (operating leases off-balance-sheet, finance lease on-balance)
- **Revenue**: IFRS 15 (already effective)
- **Financial instruments**: IFRS 9 (already effective)
- **Insurance**: IFRS 4 (IFRS 17 not yet effective)
- **Output**: No ROU assets, no lease liabilities for operating leases

### Example 2: IFRS 16 Transition (Target Date: 2019-06-30)

When `target_date: "2019-06-30"`:

- **Lease accounting**: IFRS 16 active
- **Transition artifacts**:
  - Modified retrospective approach (most common)
  - ROU asset = lease liability at transition
  - Cumulative adjustment to retained earnings
  - Transition disclosures generated
- **Output**: ROU assets, lease liabilities, transition reconciliation

### Example 3: Multi-Jurisdiction (Target Date: 2025-03-01)

For a group with entities in US, DE, JP:

| Entity | Country | Accounting | Audit | Key Local Rules |
|--------|---------|-----------|-------|----------------|
| US Parent | US | US GAAP | PCAOB | SOX 302/404, CAM, CECL |
| DE Sub | DE | HGB + IFRS | ISA (IDW) | GoBD, CSRD, EU Audit Reg |
| JP Sub | JP | J-GAAP | JICPA | J-SOX, KAM |

The framework generates jurisdiction-appropriate data for each entity while maintaining group-level consistency (intercompany eliminations, consolidated reporting).

---

## 4.7 Temporal Metadata in Output

All generated compliance data carries temporal metadata:

```json
{
  "compliance_temporal_context": {
    "generation_date": "2026-03-09",
    "target_date": "2025-06-30",
    "regulatory_state": {
      "C001_US": {
        "accounting_framework": "us_gaap",
        "active_standards": ["ASC-606@2018", "ASC-842@2019", "ASC-326@2020"],
        "audit_framework": "pcaob",
        "regulatory": ["SOX-302", "SOX-404"]
      },
      "C002_DE": {
        "accounting_framework": "german_gaap",
        "active_standards": ["HGB-253@1985", "IFRS-16@2019-amended-2024"],
        "audit_framework": "isa",
        "regulatory": ["EU-AR-537", "EU-CSRD@2024-phase2"]
      }
    },
    "transition_items": [],
    "upcoming_changes": [
      {
        "standard": "IFRS-18",
        "effective": "2027-01-01",
        "impact": "P&L presentation restructuring"
      }
    ]
  }
}
```

---

## 4.8 Drift and Regime Change Integration

The temporal versioning system integrates with the existing `DriftConfig` and `RegimeChange` infrastructure in `datasynth-core/src/distributions/drift.rs`:

```yaml
compliance_regulations:
  temporal:
    regulatory_regime_changes:
      enabled: true
      # Model how regulatory changes affect transaction patterns
      ifrs9_transition:
        date: "2018-01-01"
        effects:
          - { field: "credit_loss_provision", multiplier: 1.35, description: "IFRS 9 ECL typically higher than IAS 39 IL" }
          - { field: "provision_frequency", multiplier: 1.2, description: "More frequent re-measurement" }
      ifrs16_transition:
        date: "2019-01-01"
        effects:
          - { field: "rou_asset_count", from: 0, to: "auto", description: "Operating leases move on-balance" }
          - { field: "depreciation_expense", multiplier: 1.1, description: "ROU depreciation added" }
          - { field: "interest_expense", multiplier: 1.05, description: "Lease liability interest" }
          - { field: "ebitda", multiplier: 1.15, description: "Operating lease expense reclassified" }
```

This allows the generation engine to model the financial impact of regulatory transitions, creating more realistic temporal patterns in the synthetic data.
