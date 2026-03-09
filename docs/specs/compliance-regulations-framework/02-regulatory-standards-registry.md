# Part 2: Regulatory Standards Registry

> **Parent:** [Compliance & Regulations Framework](00-index.md)
> **Status:** Draft | **Date:** 2026-03-09

---

## 2.1 Overview

The **Standard Registry** is the central catalog of all compliance standards supported by DataSynth. It provides:

- **Canonical identification** for every standard via `StandardId`
- **Version history** with effective dates and change impact
- **Cross-reference maps** linking related standards across bodies
- **Supersession chains** tracking standard evolution over time
- **Jurisdiction mapping** showing where each standard is mandatory or permitted

The registry is the single source of truth that all generators consult. It is populated at compile time with built-in standards and can be extended at runtime with user-defined YAML files.

---

## 2.2 Built-In Standards Catalog

### 2.2.1 Accounting Standards — IFRS

| StandardId | Title | Effective | Key Domain |
|------------|-------|-----------|------------|
| `IFRS-1` | First-time Adoption of IFRS | 2004-01-01 | Transition |
| `IFRS-2` | Share-based Payment | 2005-01-01 | Equity compensation |
| `IFRS-3` | Business Combinations | 2004-03-31 | M&A, goodwill |
| `IFRS-5` | Non-current Assets Held for Sale | 2005-01-01 | Asset classification |
| `IFRS-7` | Financial Instruments: Disclosures | 2007-01-01 | Risk disclosures |
| `IFRS-8` | Operating Segments | 2009-01-01 | Segment reporting |
| `IFRS-9` | Financial Instruments | 2018-01-01 | Classification, impairment, hedging |
| `IFRS-10` | Consolidated Financial Statements | 2013-01-01 | Group accounting |
| `IFRS-13` | Fair Value Measurement | 2013-01-01 | Valuation hierarchy |
| `IFRS-15` | Revenue from Contracts with Customers | 2018-01-01 | 5-step revenue model |
| `IFRS-16` | Leases | 2019-01-01 | ROU assets, lease liabilities |
| `IFRS-17` | Insurance Contracts | 2023-01-01 | Insurance measurement |
| `IFRS-18` | Presentation and Disclosure in Financial Statements | 2027-01-01 | Replaces IAS 1 |

**Supersession chains:**
- `IAS-39` → `IFRS-9` (2018-01-01)
- `IAS-17` → `IFRS-16` (2019-01-01)
- `IFRS-4` → `IFRS-17` (2023-01-01)
- `IAS-18` + `IAS-11` → `IFRS-15` (2018-01-01)
- `IAS-1` → `IFRS-18` (2027-01-01, upcoming)

### 2.2.2 Accounting Standards — US GAAP (ASC)

| StandardId | Title | Effective | Key Domain |
|------------|-------|-----------|------------|
| `ASC-606` | Revenue from Contracts with Customers | 2018-01-01 | Revenue recognition |
| `ASC-842` | Leases | 2019-01-01 | Lease accounting |
| `ASC-815` | Derivatives and Hedging | 2001-01-01 | Hedge accounting |
| `ASC-820` | Fair Value Measurement | 2008-01-01 | Fair value hierarchy |
| `ASC-326` | Financial Instruments — Credit Losses (CECL) | 2020-01-01 | Expected credit losses |
| `ASC-350` | Intangibles — Goodwill and Other | 2002-01-01 | Goodwill impairment |
| `ASC-360` | Property, Plant, and Equipment | 2005-01-01 | Long-lived asset impairment |
| `ASC-718` | Compensation — Stock Compensation | 2006-01-01 | Share-based payment |
| `ASC-740` | Income Taxes | 1992-01-01 | Deferred tax |
| `ASC-805` | Business Combinations | 2009-01-01 | M&A accounting |
| `ASC-810` | Consolidation | 2010-01-01 | VIE, consolidation |
| `ASC-842` | Leases | 2019-01-01 | ROU assets |

**Cross-references (IFRS ↔ US GAAP):**

| IFRS | US GAAP | Convergence Level |
|------|---------|-------------------|
| IFRS-15 | ASC-606 | High (joint standard) |
| IFRS-16 | ASC-842 | Medium (different classification) |
| IFRS-9 | ASC-326 | Low (different impairment models) |
| IFRS-13 | ASC-820 | High (substantially converged) |
| IFRS-3 | ASC-805 | High (joint standard) |
| IAS-12 | ASC-740 | Medium (different deferred tax logic) |

### 2.2.3 Accounting Standards — Local GAAP

| StandardId | Title | Jurisdiction | Key Characteristics |
|------------|-------|-------------|---------------------|
| `HGB-252` | General Valuation Principles | DE | Prudence principle, lower-of-cost |
| `HGB-253` | Depreciation and Write-downs | DE | Mandatory reversal, Degressiv method |
| `HGB-264` | Annual Financial Statements | DE | Publication requirements |
| `PCG-99` | Plan Comptable Général | FR | 8-class account structure |
| `PCG-A47` | FEC Export Requirements (Article A47 A-1) | FR | 18-column mandated export |
| `FRS-102` | Financial Reporting Standard | GB | UK GAAP for medium entities |
| `FRS-101` | Reduced Disclosure Framework | GB | Qualifying entities |
| `JGAAP-ASBJ` | Japanese GAAP | JP | ASBJ standards |
| `IND-AS` | Indian Accounting Standards | IN | IFRS-converged with carve-outs |
| `CPC-00` | Conceptual Framework | BR | CPC/IFRS-converged |
| `SFRS-I` | Singapore FRS (International) | SG | IFRS-identical |
| `AASB-101` | Presentation of Financial Statements | AU | IFRS-based with local additions |
| `K-IFRS` | Korean IFRS | KR | IFRS-adopted |

### 2.2.4 Auditing Standards — ISA

| StandardId | Title | Series | Key Focus |
|------------|-------|--------|-----------|
| `ISA-200` | Overall Objectives of the Auditor | General | Reasonable assurance |
| `ISA-210` | Agreeing Terms of Audit Engagements | General | Engagement letter |
| `ISA-220` | Quality Management for an Audit | General | Engagement quality |
| `ISA-230` | Audit Documentation | General | Working papers |
| `ISA-240` | The Auditor's Responsibilities Relating to Fraud | General | Fraud procedures |
| `ISA-250` | Consideration of Laws and Regulations | General | Legal compliance |
| `ISA-260` | Communication with Those Charged with Governance | General | Governance comms |
| `ISA-265` | Communicating Deficiencies in Internal Control | General | Control deficiencies |
| `ISA-300` | Planning an Audit | Planning | Audit strategy |
| `ISA-315` | Identifying and Assessing Risks of Material Misstatement | Risk | Risk assessment |
| `ISA-320` | Materiality in Planning and Performing an Audit | Planning | Materiality |
| `ISA-330` | The Auditor's Responses to Assessed Risks | Risk | Risk response |
| `ISA-402` | Audit Considerations for Service Organizations | Special | SOC reports |
| `ISA-450` | Evaluation of Misstatements | Evaluation | Misstatement aggregation |
| `ISA-500` | Audit Evidence | Evidence | Sufficiency, appropriateness |
| `ISA-505` | External Confirmations | Evidence | Third-party confirmations |
| `ISA-520` | Analytical Procedures | Evidence | Trend/ratio analysis |
| `ISA-530` | Audit Sampling | Evidence | Statistical sampling |
| `ISA-540` | Auditing Accounting Estimates | Evidence | Estimates, judgments |
| `ISA-550` | Related Parties | Evidence | Related party transactions |
| `ISA-560` | Subsequent Events | Evidence | Post-balance-sheet events |
| `ISA-570` | Going Concern | Evidence | Continuity assessment |
| `ISA-580` | Written Representations | Evidence | Management representations |
| `ISA-600` | Special Considerations — Group Audits | Group | Component auditors |
| `ISA-610` | Using the Work of Internal Auditors | Others | Internal audit reliance |
| `ISA-620` | Using the Work of an Auditor's Expert | Others | Expert valuation |
| `ISA-700` | Forming an Opinion | Reporting | Audit opinion |
| `ISA-701` | Communicating Key Audit Matters | Reporting | KAM |
| `ISA-705` | Modifications to the Opinion | Reporting | Qualified/adverse/disclaimer |
| `ISA-706` | Emphasis of Matter and Other Matter | Reporting | Emphasis paragraphs |
| `ISA-710` | Comparative Information | Reporting | Prior period comparatives |
| `ISA-720` | The Auditor's Responsibilities Relating to Other Information | Reporting | Annual report other info |

### 2.2.5 Auditing Standards — PCAOB

| StandardId | Title | Key Focus |
|------------|-------|-----------|
| `PCAOB-AS-1101` | Responsibilities and Functions of the Independent Auditor | General responsibilities |
| `PCAOB-AS-1301` | Communications with Audit Committees | Governance communication |
| `PCAOB-AS-2101` | Audit Planning | Planning requirements |
| `PCAOB-AS-2110` | Identifying and Assessing Risks of Material Misstatement | Risk assessment |
| `PCAOB-AS-2201` | An Audit of ICFR Integrated with Financial Statement Audit | SOX 404 integration |
| `PCAOB-AS-2301` | The Auditor's Responses to Risks of Material Misstatement | Risk response |
| `PCAOB-AS-2401` | Consideration of Fraud | Fraud procedures |
| `PCAOB-AS-2501` | Auditing Accounting Estimates | Estimates |
| `PCAOB-AS-2502` | Auditing Fair Value Measurements | Fair value |
| `PCAOB-AS-2601` | Consideration of an Entity's Use of a Service Organization | SOC reports |
| `PCAOB-AS-3101` | The Auditor's Report on an Audit | Reporting (with CAM) |
| `PCAOB-AS-3105` | Departures from Unqualified Opinions | Modified opinions |

**Cross-references (ISA ↔ PCAOB):**

| ISA | PCAOB AS | Divergence |
|-----|----------|------------|
| ISA-315 | AS-2110 | Aligned (risk assessment) |
| ISA-330 | AS-2301 | Aligned (risk response) |
| ISA-701 (KAM) | AS-3101 (CAM) | Different scope: KAM broader, CAM more specific |
| ISA-265 | AS-2201 | Major: ISA communicates deficiencies; PCAOB requires ICFR opinion |
| ISA-402 | AS-2601 | Aligned (service organizations) |

### 2.2.6 Regulatory Standards — SOX

| StandardId | Title | Key Requirements |
|------------|-------|-----------------|
| `SOX-302` | Corporate Responsibility for Financial Reports | CEO/CFO certification of disclosure controls |
| `SOX-404` | Management Assessment of Internal Controls | ICFR assessment + external auditor attestation |
| `SOX-906` | Corporate Responsibility for Financial Reports (Criminal) | Criminal penalties for false certifications |
| `SOX-802` | Criminal Penalties for Altering Documents | Document retention requirements |
| `SOX-806` | Protection for Employees (Whistleblower) | Whistleblower protections |

### 2.2.7 Regulatory Standards — EU & Supranational

| StandardId | Title | Effective | Scope |
|------------|-------|-----------|-------|
| `EU-AR-537` | EU Audit Regulation (No 537/2014) | 2016-06-17 | PIE audit requirements, rotation, non-audit services |
| `EU-AD-2014` | EU Audit Directive (2014/56/EU) | 2016-06-17 | Statutory audit framework |
| `EU-CSRD` | Corporate Sustainability Reporting Directive | 2024-01-01 | ESG reporting (phased by entity size) |
| `EU-TAX` | EU Taxonomy Regulation | 2022-01-01 | Sustainable activity classification |
| `EU-SFDR` | Sustainable Finance Disclosure Regulation | 2021-03-10 | Investment product ESG disclosure |
| `EU-AMLD-6` | 6th Anti-Money Laundering Directive | 2021-12-03 | AML compliance |
| `EU-MiCA` | Markets in Crypto-Assets Regulation | 2024-12-30 | Crypto-asset regulation |
| `EU-DORA` | Digital Operational Resilience Act | 2025-01-17 | ICT risk management for financial entities |

### 2.2.8 Prudential Regulations — Basel

| StandardId | Title | Effective | Key Area |
|------------|-------|-----------|----------|
| `BASEL-III-CAP` | Capital Requirements | 2013-01-01 (phased) | CET1, Tier 1, Total Capital ratios |
| `BASEL-III-LCR` | Liquidity Coverage Ratio | 2015-01-01 | 30-day stress liquidity |
| `BASEL-III-NSFR` | Net Stable Funding Ratio | 2018-01-01 | 1-year structural liquidity |
| `BASEL-III-LEV` | Leverage Ratio | 2018-01-01 | Non-risk-weighted capital |
| `BASEL-IV-SA` | Standardized Approach (revised) | 2025-01-01 | Credit risk (output floor) |
| `BASEL-IV-FRTB` | Fundamental Review of the Trading Book | 2025-01-01 | Market risk |

### 2.2.9 Tax & Reporting Standards

| StandardId | Title | Scope |
|------------|-------|-------|
| `OECD-BEPS-13` | Transfer Pricing Documentation (CbCR) | Country-by-Country Reporting |
| `OECD-BEPS-15` | Multilateral Instrument | Treaty modification |
| `CRS` | Common Reporting Standard | Automatic exchange of financial account info |
| `FATCA` | Foreign Account Tax Compliance Act (US) | US person reporting |
| `GRI-2021` | GRI Universal Standards | Sustainability reporting |
| `ISSB-S1` | General Requirements for Sustainability-related Disclosures | IFRS sustainability baseline |
| `ISSB-S2` | Climate-related Disclosures | Climate-specific reporting |

---

## 2.3 Cross-Reference Maps

Cross-references enable the framework to generate interlinked compliance data. When a control maps to SOX-404, the framework automatically cross-references to ISA-315 (risk assessment context) and COSO (framework context).

### 2.3.1 Control Framework Cross-References

```
SOX-404 (ICFR Assessment)
  ├── COSO-2013 (Framework)
  │   ├── Control Environment ──── ISA-315.14-24
  │   ├── Risk Assessment ──────── ISA-315.25-31
  │   ├── Control Activities ───── ISA-315.32-35
  │   ├── Info & Communication ── ISA-315.36-41
  │   └── Monitoring ───────────── ISA-315.42-44
  ├── PCAOB-AS-2201 (Audit of ICFR)
  │   ├── Planning ─────────────── ISA-300
  │   ├── Top-Down Approach ────── ISA-315
  │   ├── Testing Controls ─────── ISA-330
  │   └── Evaluating Deficiencies ── ISA-265
  └── Entity-Level Controls
      ├── IT General Controls ──── ISACA COBIT
      └── Process-Level Controls ── ISA-315 Appendix
```

### 2.3.2 Accounting Standard Cross-References

```
Revenue Recognition
  IFRS-15 ◄──────────────► ASC-606
     │                         │
     ├── ISA-540 (Estimates)   ├── PCAOB-AS-2501
     ├── ISA-550 (Related)     ├── PCAOB-AS-2410
     └── IFRS-9 (Impairment    └── ASC-326 (CECL)
         of contract assets)       of contract assets)

Lease Accounting
  IFRS-16 ◄──────────────► ASC-842
     │                         │
     ├── IAS-36 (Impairment)   ├── ASC-360 (Impairment)
     └── IFRS-13 (Fair Value)  └── ASC-820 (Fair Value)
```

### 2.3.3 Registry API

```rust
impl StandardRegistry {
    /// Get the active version of a standard at a given date.
    pub fn active_version(&self, id: &StandardId, at: NaiveDate) -> Option<&TemporalVersion>;

    /// Get all standards applicable in a jurisdiction at a given date.
    pub fn standards_for_jurisdiction(
        &self,
        country: &str,
        at: NaiveDate,
    ) -> Vec<&ComplianceStandard>;

    /// Get cross-references for a standard.
    pub fn cross_references(&self, id: &StandardId) -> Vec<CrossReference>;

    /// Get the full supersession chain (oldest → newest).
    pub fn supersession_chain(&self, id: &StandardId) -> Vec<&ComplianceStandard>;

    /// Query standards by category and domain.
    pub fn query(&self, category: StandardCategory, domain: ComplianceDomain)
        -> Vec<&ComplianceStandard>;

    /// Register a user-defined standard from YAML.
    pub fn register_custom(&mut self, yaml_path: &Path) -> Result<StandardId>;

    /// Get all standards of a specific issuing body.
    pub fn by_body(&self, body: &IssuingBody) -> Vec<&ComplianceStandard>;
}
```

---

## 2.4 Supersession Chain Resolution

When generating data for a target date, the registry resolves which version of each standard is active. This handles:

1. **Simple supersession:** IAS-39 → IFRS-9 (after 2018-01-01, always use IFRS-9)
2. **Partial supersession:** IAS-39 still applies for hedge accounting that entities haven't transitioned (configurable)
3. **Delayed adoption:** A jurisdiction may adopt IFRS-16 later than the global effective date (e.g., some jurisdictions delayed to 2020)
4. **Early adoption:** Some standards permit early adoption (e.g., IFRS-17 from 2021 with early adoption)

```rust
/// Resolution algorithm:
/// 1. Look up the standard in the registry
/// 2. Find the version where effective_from <= target_date < superseded_at
/// 3. If the standard is superseded, check if the jurisdiction has a delayed adoption date
/// 4. If early adoption is configured, use the early_adoption_from date
/// 5. Return the resolved version with its specific requirements
pub fn resolve_standard(
    &self,
    id: &StandardId,
    country: &str,
    target_date: NaiveDate,
    config: &ComplianceConfig,
) -> ResolvedStandard {
    // ... resolution logic
}
```

---

## 2.5 User-Defined Standards

Users can extend the registry with custom standards defined in YAML:

```yaml
# custom-standards/my-industry-standard.yaml
standard:
  id: "CUSTOM-FINTECH-001"
  title: "Fintech Operational Resilience Standard"
  issuing_body: custom
  category: regulatory_requirement
  domain: risk_management
  versions:
    - version_id: "2024"
      effective_from: "2024-01-01"
      change_summary:
        - "Initial release"
      impact: high
  cross_references:
    - standard_id: "EU-DORA"
      relationship: derived_from
    - standard_id: "BASEL-III-CAP"
      relationship: complementary
  requirements:
    - id: "FINTECH-001.R1"
      title: "ICT Risk Assessment"
      description: "Annual ICT risk assessment required"
      assertions: [existence, completeness]
    - id: "FINTECH-001.R2"
      title: "Incident Reporting"
      description: "Major ICT incidents reported within 4 hours"
      assertions: [timeliness, completeness]
  mandatory_jurisdictions: []
  permitted_jurisdictions: ["*"]  # Available everywhere
```

Configuration to load custom standards:

```yaml
compliance_regulations:
  custom_standards_dir: "./custom-standards"
  # All YAML files in this directory are loaded into the registry
```

---

## 2.6 Output: Standards Metadata Export

When compliance generation runs, the registry exports metadata files alongside the generated data:

```
output/compliance/
├── standards_registry.json          # All applicable standards with versions
├── cross_reference_map.json         # Standard-to-standard cross-references
├── jurisdiction_profile.json        # Per-company jurisdiction resolution
├── supersession_chains.json         # Active supersession chains
└── standard_coverage_matrix.csv     # Standards × controls coverage matrix
```

**standards_registry.json** (excerpt):
```json
{
  "target_date": "2025-06-30",
  "resolved_standards": [
    {
      "id": "IFRS-16",
      "title": "Leases",
      "active_version": "2019-amended-2024",
      "effective_from": "2019-01-01",
      "issuing_body": "IASB",
      "category": "accounting_standard",
      "cross_references": ["ASC-842", "IAS-36", "IFRS-13"],
      "supersedes": ["IAS-17"],
      "applicable_entities": ["C001", "C003"]
    }
  ]
}
```
