# Part 3: Country-Specific Compliance Profiles

> **Parent:** [Compliance & Regulations Framework](00-index.md)
> **Status:** Implemented | **Date:** 2026-03-09

---

## 3.1 Overview

Country-specific compliance profiles are the mechanism by which the framework translates abstract standards into jurisdiction-specific requirements. Each profile answers: "For an entity incorporated in country X, what accounting, auditing, regulatory, and reporting requirements apply?"

Profiles are **compositional**: they compose global standards (IFRS, ISA) with local requirements (HGB, SOX, FEC), local effective dates, local modifications (carve-outs, delayed adoption), and filing obligations.

---

## 3.2 Profile Structure

```rust
pub struct CountryComplianceProfile {
    /// ISO 3166-1 alpha-2
    pub country_code: String,
    /// Display name
    pub country_name: String,
    /// Supranational memberships affecting regulatory obligations
    pub memberships: Vec<SupranationalMembership>,
    /// Accounting layer
    pub accounting: AccountingProfile,
    /// Auditing layer
    pub auditing: AuditingProfile,
    /// Regulatory layer
    pub regulatory: RegulatoryProfile,
    /// Tax layer
    pub tax: TaxProfile,
    /// Reporting/filing layer
    pub reporting: ReportingProfile,
    /// Sustainability/ESG layer
    pub sustainability: SustainabilityProfile,
}
```

---

## 3.3 Supported Country Profiles

### 3.3.1 United States (US)

```yaml
country_code: US
country_name: "United States of America"
memberships: []

accounting:
  primary_framework: us_gaap
  standards_body: FASB
  chart_of_accounts: flexible  # No mandated chart
  ifrs_permitted: true          # SEC allows IFRS for foreign private issuers
  key_standards:
    - { id: "ASC-606", local_name: "ASC 606", effective: "2018-01-01" }
    - { id: "ASC-842", local_name: "ASC 842", effective: "2019-01-01" }
    - { id: "ASC-326", local_name: "ASC 326 (CECL)", effective: "2020-01-01" }
    - { id: "ASC-820", local_name: "ASC 820", effective: "2008-01-01" }
    - { id: "ASC-740", local_name: "ASC 740", effective: "1992-01-01" }

auditing:
  framework: pcaob  # For SEC registrants
  alternative_framework: aicpa_clarified  # For non-public entities
  standards_body: PCAOB
  key_standards:
    - { id: "PCAOB-AS-2201", context: "ICFR audit (SOX 404)" }
    - { id: "PCAOB-AS-3101", context: "Auditor's report with CAM" }
  integrated_audit: true  # Financial statements + ICFR
  cam_required: true       # Critical Audit Matters

regulatory:
  securities_regulator: SEC
  stock_exchange: [NYSE, NASDAQ]
  key_regulations:
    - { id: "SOX-302", scope: "all_sec_registrants" }
    - { id: "SOX-404", scope: "accelerated_filers" }
    - { id: "SOX-906", scope: "all_sec_registrants" }
  filing_currency: USD
  fiscal_year_flexibility: true

tax:
  tax_authority: IRS
  corporate_tax_rate: 0.21
  key_regulations:
    - { id: "FATCA", scope: "foreign_financial_institutions" }
    - { id: "OECD-BEPS-13", scope: "mnc_above_threshold" }
  transfer_pricing: section_482
  withholding_tax: true

reporting:
  formats:
    - { type: "10-K", frequency: annual, regulator: SEC }
    - { type: "10-Q", frequency: quarterly, regulator: SEC }
    - { type: "8-K", frequency: event_driven, regulator: SEC }
    - { type: "20-F", frequency: annual, scope: "foreign_private_issuers" }
  electronic_filing: EDGAR
  xbrl_required: true
  inline_xbrl: true  # Since 2021

sustainability:
  frameworks:
    - { id: "SEC-CLIMATE", effective: "2026-01-01", scope: "large_accelerated_filers" }
  voluntary: [GRI, SASB, TCFD]
```

### 3.3.2 Germany (DE)

```yaml
country_code: DE
country_name: "Federal Republic of Germany"
memberships: [EU, EEA, EUROZONE]

accounting:
  primary_framework: german_gaap  # HGB for statutory
  ifrs_required_for: listed_entities  # § 315e HGB
  standards_body: DRSC
  chart_of_accounts: skr04  # or skr03
  key_standards:
    - { id: "HGB-252", local_name: "§252 HGB General Valuation", effective: "1985-01-01" }
    - { id: "HGB-253", local_name: "§253 HGB Depreciation", effective: "1985-01-01" }
    - { id: "HGB-264", local_name: "§264 HGB Annual Statements", effective: "1985-01-01" }
  local_rules:
    prudence_principle: true        # Vorsichtsprinzip
    reverse_impairment: mandatory   # §253(5) HGB
    goodwill_amortization: true     # max 10 years §253(3)
    gwg_threshold: 800              # EUR, immediate expense
    degressiv_depreciation: true    # 3x SL, max 30%
    pending_loss_provisions: true   # Drohverlustrückstellungen

auditing:
  framework: isa  # ISA adopted via EU
  standards_body: IDW
  key_standards:
    - { id: "ISA-315", local_name: "ISA [DE] 315 (Revised)" }
    - { id: "ISA-700", local_name: "ISA [DE] 700 (Revised)" }
  local_additions:
    - "IDW PS 201: Principles of audit reporting"
    - "IDW PS 261: Risk-based audit approach"
    - "IDW PS 880: IT audit"

regulatory:
  securities_regulator: BaFin
  stock_exchange: [XETRA, FSE]
  key_regulations:
    - { id: "EU-AR-537", scope: "public_interest_entities" }
    - { id: "EU-AD-2014", scope: "all_statutory_audits" }
  audit_export:
    format: gobd
    columns: 13
    separator: semicolon
    index_file: xml

tax:
  tax_authority: BZSt
  corporate_tax_rate: 0.15  # KSt, plus ~14% GewSt + 5.5% SolZ
  key_regulations:
    - { id: "GoBD", scope: "all_entities", description: "Digital record-keeping requirements" }
    - { id: "OECD-BEPS-13", scope: "mnc_above_threshold" }
    - { id: "CRS", scope: "financial_institutions" }
  e_invoicing: mandatory_2025  # B2B from 2025

reporting:
  formats:
    - { type: "Jahresabschluss", frequency: annual, regulator: Bundesanzeiger }
    - { type: "E-Bilanz", frequency: annual, regulator: BZSt, format: xbrl }
  electronic_filing: eBilanz
  publication: Bundesanzeiger

sustainability:
  frameworks:
    - { id: "EU-CSRD", effective: "2024-01-01", scope: "large_entities" }
    - { id: "EU-TAX", effective: "2022-01-01", scope: "all_entities" }
  reporting_standard: ESRS  # European Sustainability Reporting Standards
```

### 3.3.3 United Kingdom (GB)

```yaml
country_code: GB
country_name: "United Kingdom"
memberships: []  # Post-Brexit

accounting:
  primary_framework: uk_gaap  # FRS 102 for most
  ifrs_required_for: listed_entities  # AIM optional
  standards_body: FRC
  key_standards:
    - { id: "FRS-102", local_name: "FRS 102", effective: "2015-01-01" }
    - { id: "FRS-101", local_name: "FRS 101 Reduced Disclosure", effective: "2015-01-01" }
  local_rules:
    section_1A_micro: true   # Micro-entity regime
    true_and_fair_override: true

auditing:
  framework: isa_uk  # ISA (UK) with local additions
  standards_body: FRC
  key_standards:
    - { id: "ISA-315", local_name: "ISA (UK) 315 (Revised 2019)" }
    - { id: "ISA-700", local_name: "ISA (UK) 700 (Revised 2019)" }
    - { id: "ISA-701", local_name: "ISA (UK) 701" }
  local_additions:
    - "Extended auditor's report for listed entities"
    - "Viability statement audit (UK Corporate Governance Code)"

regulatory:
  securities_regulator: FCA
  stock_exchange: [LSE, AIM]
  key_regulations:
    - { id: "UK-CA-2006", scope: "all_companies", description: "Companies Act 2006" }
    - { id: "UK-CGC", scope: "premium_listed", description: "UK Corporate Governance Code" }
  audit_rotation: 20_years_with_tender  # For FTSE 350

tax:
  tax_authority: HMRC
  corporate_tax_rate: 0.25  # From April 2023
  key_regulations:
    - { id: "MTD", scope: "vat_registered", description: "Making Tax Digital" }
    - { id: "CRS", scope: "financial_institutions" }
  e_invoicing: voluntary

reporting:
  formats:
    - { type: "Annual Return", frequency: annual, regulator: Companies_House }
    - { type: "CT600", frequency: annual, regulator: HMRC }
  electronic_filing: Companies_House_online
  iXBRL_required: true  # Corporation tax

sustainability:
  frameworks:
    - { id: "UK-SDR", effective: "2024-01-01", scope: "large_entities" }
    - { id: "TCFD", effective: "2022-04-06", scope: "premium_listed" }
```

### 3.3.4 France (FR)

```yaml
country_code: FR
country_name: "French Republic"
memberships: [EU, EEA, EUROZONE]

accounting:
  primary_framework: french_gaap  # PCG
  ifrs_required_for: listed_entities
  standards_body: ANC
  chart_of_accounts: pcg  # Mandatory 8-class structure
  key_standards:
    - { id: "PCG-99", local_name: "Plan Comptable Général" }
    - { id: "PCG-A47", local_name: "Article A47 A-1 (FEC)" }
  local_rules:
    fec_export: mandatory
    auxiliary_accounts: true  # 401XXXX / 411XXXX
    provision_reglementees: true

auditing:
  framework: isa  # Adopted via EU
  standards_body: H3C
  key_standards:
    - { id: "ISA-315", local_name: "NEP 315" }
    - { id: "ISA-700", local_name: "NEP 700" }
  local_additions:
    - "CAC (Commissaire aux comptes) specific requirements"
    - "Two-CAC requirement for large entities"

regulatory:
  securities_regulator: AMF
  stock_exchange: [EURONEXT_PARIS]
  key_regulations:
    - { id: "EU-AR-537", scope: "public_interest_entities" }
    - { id: "EU-AD-2014", scope: "all_statutory_audits" }
  audit_export:
    format: fec
    columns: 18
    separator: pipe_or_tab
    encoding: ISO-8859-1_or_UTF-8

tax:
  tax_authority: DGFiP
  corporate_tax_rate: 0.25
  key_regulations:
    - { id: "FEC", scope: "all_entities", description: "Fichier des Écritures Comptables" }
    - { id: "OECD-BEPS-13", scope: "mnc_above_threshold" }
  e_invoicing: mandatory_2026  # Phased rollout

reporting:
  formats:
    - { type: "Liasse fiscale", frequency: annual, regulator: DGFiP }
    - { type: "Rapport de gestion", frequency: annual }
  electronic_filing: DGFiP_platform

sustainability:
  frameworks:
    - { id: "EU-CSRD", effective: "2024-01-01" }
    - { id: "DPEF", effective: "2017-01-01", scope: "large_entities" }
```

### 3.3.5 Japan (JP)

```yaml
country_code: JP
country_name: "Japan"
memberships: []

accounting:
  primary_framework: j_gaap  # ASBJ
  ifrs_permitted: true  # Voluntary adoption since 2010
  standards_body: ASBJ
  chart_of_accounts: flexible
  key_standards:
    - { id: "JGAAP-ASBJ", local_name: "企業会計基準" }
  local_rules:
    revenue_standard: "ASBJ Statement No. 29"  # Converged with IFRS 15
    lease_standard: "ASBJ Statement No. 13"    # Operating leases still off-balance
    goodwill_amortization: true  # Max 20 years

auditing:
  framework: isa_local  # Based on ISA with local modifications
  standards_body: JICPA
  key_standards:
    - { id: "ISA-315", local_name: "監査基準委員会報告書315" }
    - { id: "ISA-700", local_name: "監査基準委員会報告書700" }
  local_additions:
    - "KAM required for listed entities from 2021"
    - "Internal control audit (J-SOX)"

regulatory:
  securities_regulator: FSA_JFSA
  stock_exchange: [TSE]
  key_regulations:
    - { id: "J-SOX", scope: "listed_entities", description: "Financial Instruments and Exchange Act" }
  j_sox:
    effective: "2008-04-01"
    icfr_assessment: true
    auditor_attestation: true
    direct_reporting: false  # Unlike US SOX

tax:
  tax_authority: NTA
  corporate_tax_rate: 0.2315  # National, plus local rates
  qualified_invoice: true  # From October 2023

reporting:
  formats:
    - { type: "有価証券報告書", frequency: annual, regulator: FSA }
    - { type: "四半期報告書", frequency: quarterly, regulator: FSA }
  electronic_filing: EDINET
  xbrl_required: true
```

### 3.3.6 India (IN)

```yaml
country_code: IN
country_name: "Republic of India"
memberships: []

accounting:
  primary_framework: ind_as  # IFRS-converged
  standards_body: ICAI
  key_standards:
    - { id: "IND-AS", local_name: "Ind AS" }
  local_rules:
    ifrs_carve_outs:
      - "Ind AS 101: Deemed cost exemption for PPE"
      - "Ind AS 109: Irrevocable FVOCI for equity investments (scope limitation)"
      - "Ind AS 116: Lease modifications — no reassessment of lease term"
    schedule_iii: true  # Companies Act Schedule III format

auditing:
  framework: isa_local
  standards_body: ICAI
  key_standards:
    - { id: "ISA-315", local_name: "SA 315" }
    - { id: "ISA-700", local_name: "SA 700" }
  local_additions:
    - "CARO 2020 reporting requirements"
    - "Tax audit (Section 44AB)"
    - "Internal financial controls (IFC) reporting"

regulatory:
  securities_regulator: SEBI
  stock_exchange: [BSE, NSE]
  key_regulations:
    - { id: "IN-CA-2013", scope: "all_companies", description: "Companies Act 2013" }
    - { id: "IN-IFC", scope: "all_companies", description: "Internal Financial Controls" }
  ifc:
    section_143_3i: true  # Auditor opinion on IFC
    based_on: coso

tax:
  tax_authority: CBDT
  corporate_tax_rate: 0.2517  # 25.17% for new manufacturing
  gst: true
  e_invoicing: mandatory_above_threshold

reporting:
  formats:
    - { type: "Annual Return (MGT-7)", frequency: annual, regulator: MCA }
    - { type: "Financial Statements", frequency: annual, format: "Schedule III" }
  electronic_filing: MCA21
```

### 3.3.7 Additional Country Profiles (Summary)

| Country | Code | Accounting | Audit | Key Local Features |
|---------|------|-----------|-------|-------------------|
| **Brazil** | BR | BR GAAP (CPCs, IFRS-converged) | NBC TA (ISA-based) | SPED/EFD electronic bookkeeping, CVM for listed |
| **Singapore** | SG | SFRS(I) (IFRS-identical) | SSA (ISA-identical) | ACRA filing, XBRL required |
| **Australia** | AU | AASB (IFRS-based) | ASA (ISA-based) | ASIC reporting, reduced disclosure for Tier 2 |
| **Canada** | CA | IFRS (public) / ASPE (private) | CAS (ISA-based) | CSA requirements, bilingual reporting |
| **South Korea** | KR | K-IFRS | KSA (ISA-based) | DART electronic filing, K-SOX |
| **Italy** | IT | OIC / IFRS for listed | ISA Italia | Collegio Sindacale oversight, CONSOB |
| **Spain** | ES | PGC / IFRS for listed | NIA-ES (ISA-based) | ICAC standards, CNMV |
| **Mexico** | MX | NIF (CINIF) | NIA (ISA-based) | CNBV requirements, SAT e-invoicing |
| **China** | CN | CAS (converging with IFRS) | CSAE (ISA-based) | CSRC requirements, unique fair value rules |
| **Switzerland** | CH | Swiss GAAP FER / IFRS | ISA (via RAB) | SIX exchange rules, OR (Code of Obligations) |
| **Netherlands** | NL | Dutch GAAP / IFRS | ISA (via NBA) | AFM oversight, Dutch Civil Code Title 9 |
| **Saudi Arabia** | SA | IFRS (mandatory since 2017) | ISA | SOCPA oversight, Zakat requirements |
| **UAE** | AE | IFRS (mandatory) | ISA | Corporate tax from 2023, free zone rules |

---

## 3.4 Supranational Memberships

Memberships propagate regulatory obligations:

```rust
pub enum SupranationalBody {
    EU,         // EU Audit Regulation, CSRD, EU Taxonomy
    EEA,        // Extends EU to Norway, Iceland, Liechtenstein
    Eurozone,   // ECB supervision, TARGET2
    ASEAN,      // ASEAN CPA framework
    GCC,        // Gulf Cooperation Council
    Mercosur,   // Southern Common Market
}
```

**EU membership automatically includes:**
- EU Audit Regulation (537/2014) for PIEs
- EU Audit Directive (2014/56/EU) for statutory audits
- CSRD for sustainability reporting (phased)
- EU Taxonomy for sustainable classification
- AMLD-6 for AML compliance
- CRS for automatic exchange

---

## 3.5 Applicability Criteria

Not all standards apply to all entities within a jurisdiction. The framework models applicability via criteria:

```rust
pub enum ApplicabilityCriteria {
    /// Applies to all entities in the jurisdiction
    AllEntities,
    /// Only listed/public entities
    ListedEntities,
    /// Only Public Interest Entities (PIEs)
    PublicInterestEntities,
    /// Only entities above revenue/asset thresholds
    AboveThreshold { revenue: Option<Decimal>, assets: Option<Decimal>, employees: Option<u32> },
    /// Only entities in specific sectors
    SectorSpecific(Vec<String>),
    /// Only financial institutions
    FinancialInstitutions,
    /// Only SEC registrants (US-specific)
    SecRegistrants,
    /// Accelerated filers (US-specific)
    AcceleratedFilers,
    /// Custom criteria expression
    Custom(String),
}
```

**Example:** SOX-404 auditor attestation applies to `AcceleratedFilers` but not to non-accelerated filers or emerging growth companies. The framework generates SOX data only for entities matching the criteria.

---

## 3.6 Configuration

### Per-Company Jurisdiction Override

```yaml
companies:
  - code: "C001"
    name: "US Parent Corp"
    country: US
    compliance_overrides:
      additional_standards: ["EU-CSRD"]  # US company with EU reporting needs
      exclude_standards: ["SOX-906"]     # Not applicable
      entity_type: accelerated_filer

  - code: "C002"
    name: "German Subsidiary GmbH"
    country: DE
    compliance_overrides:
      entity_type: large_entity
      ifrs_reporting: true  # Listed parent requires IFRS package
```

### Global Jurisdiction Selection

```yaml
compliance_regulations:
  jurisdictions:
    - US
    - DE
    - GB
    - JP
  # Or use "auto" to derive from company country codes:
  jurisdiction_mode: auto  # auto | explicit
```

---

## 3.7 Profile Resolution Algorithm

```
resolve_profile(company, target_date):
  1. Load base profile for company.country
  2. Apply supranational memberships (add EU regulations if EU member)
  3. Filter standards by applicability criteria (entity_type, sector, thresholds)
  4. Apply company-specific overrides (additional/excluded standards)
  5. Resolve temporal versions (which version of each standard at target_date)
  6. Apply local effective dates (may differ from global effective dates)
  7. Return: ResolvedComplianceProfile {
       applicable_standards: Vec<ResolvedStandard>,
       filing_requirements: Vec<FilingRequirement>,
       audit_requirements: AuditRequirements,
       tax_requirements: TaxRequirements,
     }
```
