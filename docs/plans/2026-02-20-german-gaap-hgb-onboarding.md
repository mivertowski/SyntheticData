# German GAAP (HGB) Onboarding — Specification & Implementation Plan

> **Status:** Draft
> **Date:** 2026-02-20
> **Author:** Claude Code
> **Scope:** Add first-class German GAAP (HGB) support, following the established French GAAP (PCG) onboarding pattern
> **Predecessor:** French GAAP PR #46 (commits `438ebf2`..`cd9b89f`)

---

## 1. Executive Summary

### 1.1 Objective

Onboard **German GAAP (HGB — Handelsgesetzbuch)** as a fully supported accounting framework in DataSynth, at parity with the existing French GAAP (PCG) implementation. This means:

1. A new `GermanGaap` variant in the `AccountingFramework` enum
2. A complete German chart of accounts based on **SKR04** (Standardkontenrahmen 04) with SKR03 as an alternative
3. HGB-specific accounting treatment rules (provisions, impairment, leases, low-value assets, depreciation)
4. A **GoBD-compliant audit export** format (the German equivalent of the French FEC)
5. Framework-specific settings and validation rules
6. E-Bilanz (XBRL taxonomy) awareness for tax reporting
7. Full orchestrator and CLI integration

### 1.2 Why HGB?

Germany is Europe's largest economy. The HGB governs all German commercial entities (§238 HGB). Unlike France where most entities follow PCG, Germany has a dual-track system:

- **Large/public companies**: IFRS for consolidated accounts (mandatory for listed entities per EU IAS Regulation)
- **All entities**: HGB for individual (statutory) accounts and tax filings (Steuerbilanz)

The existing `DE.json` country pack already declares `"local_gaap_name": "HGB"` and `"standard": "SKR04"`, but no generation logic exists. This plan closes that gap.

### 1.3 French GAAP Pattern (Reference)

The French GAAP implementation followed this pattern, which we replicate:

| Component | French GAAP | German GAAP (this plan) |
|---|---|---|
| Framework enum variant | `FrenchGaap` | `GermanGaap` |
| Config schema variant | `AccountingFrameworkConfig::FrenchGaap` | `AccountingFrameworkConfig::GermanGaap` |
| Chart of accounts data | `pcg_2024.json` (6,424 lines) | `skr04_2024.json` (new, ~3,000+ lines) |
| CoA constants module | `datasynth-core/src/pcg.rs` | `datasynth-core/src/skr.rs` (new) |
| CoA loader module | `datasynth-core/src/pcg_loader.rs` | `datasynth-core/src/skr_loader.rs` (new) |
| CoA generator flag | `use_french_pcg: bool` | `use_german_skr: bool` (new) |
| Audit export format | `fec.rs` (FEC 18-column) | `gobd.rs` (GoBD/GDPdU export, new) |
| Framework methods | `french_gaap()` on `FrameworkSettings` | `german_gaap()` on `FrameworkSettings` |
| Orchestrator wiring | Maps `FrenchGaap` → PCG CoA + FEC export | Maps `GermanGaap` → SKR CoA + GoBD export |
| CLI output | `fec.csv` written when `FrenchGaap` | `gobd_export/` written when `GermanGaap` |

---

## 2. German GAAP (HGB) — Domain Background

### 2.1 Legal Framework

The **Handelsgesetzbuch** (HGB, German Commercial Code) is the primary source of German statutory accounting law. Key sections:

| HGB Section | Topic |
|---|---|
| §238–263 | General bookkeeping obligations (Buchführungspflicht) |
| §264–289a | Supplementary rules for Kapitalgesellschaften (corporations) |
| §290–315e | Consolidated accounts (Konzernabschluss) |
| §316–324a | Audit requirements (Prüfung) |
| §325–329 | Disclosure and filing (Offenlegung) |

Key reforms:
- **BilMoG (2009)**: Bilanzrechtsmodernisierungsgesetz — modernized HGB to align closer to IFRS while retaining German-specific rules. Eliminated certain tax-driven provisions, introduced fair-value measurement for financial instruments, allowed capitalization of self-created intangible assets.
- **BilRUG (2015)**: Bilanzrichtlinie-Umsetzungsgesetz — implemented EU Accounting Directive 2013/34/EU. Changed revenue definition, size thresholds, notes requirements.
- **DiRUG (2022)**: Digitalization directive — electronic Handelsregister filing via Unternehmensregister.
- **MoPeG (2024)**: Modernization of partnership law — affects OHG/KG accounting obligations.
- **BEG IV (Oct 2024)**: Viertes Bürokratieentlastungsgesetz — reduced retention periods for Buchungsbelege from 10 to **8 years** (§257 Abs. 4 HGB).
- **Wachstumschancengesetz (Mar 2024)**: Introduced mandatory B2B e-invoicing (phased 2025–2028), improved loss carryforward rules, expanded partnership taxation options.
- **Annual Tax Act 2024 (Dec 2024)**: Raised §267 HGB size thresholds by ~25%, introduced income tax information report (§§342 ff. HGB).

**Updated Company Size Thresholds (§267 HGB, effective 2025):**

| Category | Balance Sheet Total | Annual Revenue | Employees |
|---|---|---|---|
| Micro (§267a) | ≤ EUR 450,000 | ≤ EUR 900,000 | ≤ 10 |
| Small | ≤ EUR 7,500,000 | ≤ EUR 15,000,000 | ≤ 50 |
| Medium | ≤ EUR 25,000,000 | ≤ EUR 50,000,000 | ≤ 250 |
| Large | Exceeds medium thresholds | | |

**Pending:** CSRD transposition into HGB (sustainability reporting in Lagebericht) — delayed by EU "Stop the Clock" directive to fiscal year 2027 (Wave 2) and 2028 (Wave 3).

### 2.2 SKR04 Chart of Accounts Structure

**SKR04** (Standardkontenrahmen 04) is the process-oriented chart of accounts published by DATEV, the dominant German accounting software cooperative. It is based on the **Abschlussgliederungsprinzip** (financial-statement-oriented structure), making it the natural choice for DataSynth's financial statement generation.

| Class | Range | German Name | English Translation | Equivalent |
|---|---|---|---|---|
| 0 | 0000–0999 | Anlagevermögen | Fixed Assets (non-current) | PCG Class 2 |
| 1 | 1000–1999 | Umlaufvermögen | Current Assets | PCG Class 3–5 |
| 2 | 2000–2999 | Eigenkapital | Equity | PCG Class 1 (partial) |
| 3 | 3000–3999 | Fremdkapital | Liabilities | PCG Class 1 (partial) + 4 |
| 4 | 4000–4999 | Betriebliche Erträge | Operating Revenue | PCG Class 7 |
| 5 | 5000–5999 | Betriebliche Aufwendungen | Operating Expenses | PCG Class 6 |
| 6 | 6000–6999 | Betriebliche Aufwendungen | Operating Expenses (continued) | PCG Class 6 |
| 7 | 7000–7999 | Weitere Erträge/Aufwendungen | Other Income/Expenses | PCG Class 6+7 |
| 8 | 8000–8999 | (reserved / unused in SKR04) | — | — |
| 9 | 9000–9999 | Vortrags- und statistische Konten | Carry-forward & statistical | PCG Class 9 |

**Key SKR04 control accounts:**

| Purpose | SKR04 Account | Description |
|---|---|---|
| AR Control | 1200 | Forderungen aus Lieferungen und Leistungen |
| AP Control | 3300 | Verbindlichkeiten aus Lieferungen und Leistungen |
| Bank | 1200–1289 | Bankkonten (main bank accounts) |
| Cash | 1600 | Kasse |
| Inventory | 1100–1199 | Vorräte |
| Fixed Assets | 0100–0899 | Sachanlagen |
| Accum. Depreciation | 0100–0899 (offset sub-accounts) | Kumulierte Abschreibungen |
| Revenue (products) | 4000–4399 | Umsatzerlöse |
| Revenue (services) | 4400–4599 | Erlöse aus Leistungen |
| COGS | 5000–5199 | Materialaufwand |
| Wages/Salaries | 6000–6199 | Personalaufwand – Löhne und Gehälter |
| Social contributions | 6200–6299 | Personalaufwand – Soziale Abgaben |
| Depreciation | 6200–6299 (alt: 7000s) | Abschreibungen |
| Interest expense | 7300–7399 | Zinsen und ähnliche Aufwendungen |
| Tax expense | 7600–7699 | Steuern vom Einkommen und Ertrag |
| GR/IR Clearing | 3350 | Wareneingangs-/Rechnungseingangskonto |

**SKR03 vs SKR04:** SKR03 follows the **Prozessgliederungsprinzip** (process-oriented), organizing accounts by business process rather than financial statement line. SKR03 is more common among small businesses; SKR04 among larger enterprises and those using SAP. We implement SKR04 as the primary chart, with SKR03 as a configuration option.

### 2.3 HGB-Specific Accounting Treatments

These are the key differences from IFRS that must be reflected in generated data:

#### 2.3.1 Provisions (Rückstellungen) — §249 HGB

HGB requires provisions for:
- Uncertain obligations to third parties (§249(1) S.1 HGB)
- Pending loss contracts (Drohverlustrückstellungen, §249(1) S.1 HGB) — **mandatory under HGB, prohibited under IFRS**
- Maintenance obligations (§249(1) S.2 Nr.1 HGB) — first 3 months of next fiscal year
- Warranty without legal obligation (§249(1) S.2 Nr.2 HGB)
- Internal obligation expenses (Aufwandsrückstellungen) — **prohibited since BilMoG** (§249(2) HGB old version removed)

**Key differences from IFRS (IAS 37):**
- **Probability threshold**: HGB allows recognition at probability **below 50%** (prudence principle); IFRS requires **>50%** probability
- **Expense provisions (Aufwandsrückstellungen)**: Restricted but still possible under HGB; **prohibited under IFRS**
- **Discounting**: §253(2) HGB requires provisions with >1 year maturity to be discounted using Bundesbank average market rate over past **7 years** (10 years for pension provisions). IFRS uses entity-specific pre-tax rate.
- **Overall effect**: Provision levels under HGB are generally **higher** than under IFRS

#### 2.3.2 Self-Created Intangible Assets — §248(2) HGB

BilMoG introduced an **optional** capitalization of self-created intangible fixed assets (Aktivierungswahlrecht). This is:
- **Optional under HGB** (§248(2) HGB) — many companies still expense R&D fully
- **Mandatory under IFRS** (IAS 38) when development criteria are met
- **Never for research costs** (§255(2a) HGB — only development costs eligible)
- If capitalized, a distribution restriction applies (§268(8) HGB — blocked for dividends)

#### 2.3.3 Goodwill (Geschäfts- oder Firmenwert)

- **HGB**: Mandatory amortization over useful economic life (max 10 years if life cannot be reliably estimated, §253(3) S.4 HGB)
- **IFRS**: No amortization; annual impairment testing under IAS 36 (IASB discussing reintroduction of amortization)

**Impact on DataSynth**: When `GermanGaap` is selected, goodwill should amortize over a configurable period (default 5 years), unlike IFRS where it remains on the balance sheet indefinitely.

#### 2.3.4 Impairment — §253(3)-(5) HGB

- **Write-down obligation**: Permanent impairment → mandatory write-down for all assets (§253(3) S.5 HGB)
- **Temporary impairment**: Only mandatory for financial fixed assets (Finanzanlagen) and current assets; optional for other fixed assets
- **Reversal (Wertaufholung)**: **Mandatory** under HGB (§253(5) HGB) — you must reverse write-downs when reasons no longer exist. Under IFRS, reversal is permitted but for goodwill. Under US GAAP, reversal is generally prohibited.
- **No goodwill exception**: Unlike IFRS, HGB requires reversal of goodwill impairment too (unless purchased goodwill is written off over useful life per §253(3) S.3 HGB)

#### 2.3.5 Leases — HGB vs IFRS 16

HGB has **no dedicated lease standard**. Lease classification follows tax guidance (Steuerliche Leasingerlasse, BMF 1971/2001):
- **Operating lease**: Off-balance-sheet for lessee (expense recognition only)
- **Finance lease**: On-balance-sheet only when economic ownership transfers to lessee
- Key test: Lease term ≥ 40% and ≤ 90% of useful life → operating lease (lessee off-balance)

This differs fundamentally from IFRS 16 (all leases on-balance for lessee) and US GAAP ASC 842 (bright-line tests).

**Impact on DataSynth**: When `GermanGaap` is selected, most leases should remain off-balance (operating), generating only rent expense entries rather than ROU assets and lease liabilities.

#### 2.3.6 Low-Value Assets (GWG) — §6(2) EStG

Unique to German tax/commercial law:
- Assets with acquisition cost ≤ **800 EUR** (net): Immediate full expense in year of acquisition (Sofortabschreibung)
- Assets 250.01–1,000 EUR: Option for **pool depreciation** (Sammelposten) over 5 years
- This is a **tax rule** (EStG) but commonly applied in HGB accounts due to Maßgeblichkeitsprinzip

#### 2.3.7 Depreciation — HGB-Specific Methods

- **Straight-line (linear, §7(1) EStG)**: Standard method. Building rates: 3% for post-2022 completions, 2% for post-1924, 2.5% pre-1925. Movable assets per AfA-Tabellen.
- **Declining balance (degressiv, §7(2) EStG)**: Tax law intermittently allows/disallows. Current rules:
  - 2020–2022 (Corona): max 2.5x linear, capped at 25%
  - 01.04.2024–31.12.2024: max 2x linear, capped at 20%
  - **01.07.2025–31.12.2027: max 3x linear, capped at 30%**
- **Sonderabschreibung (§7g(5) EStG)**: Special depreciation for SMEs (profit < EUR 200,000) — up to **40%** additional in first 5 years (increased from 20% for post-2023 acquisitions). Combinable with degressive for up to 60% in Year 1.
- **Degressive building AfA (§7(5a) EStG)**: 5–6% declining balance for residential buildings (construction start Oct 2023 – Sep 2029).
- **Useful life**: Follows the official **AfA-Tabellen** (depreciation tables) published by the Bundesfinanzministerium
- **Component approach**: Not required under HGB (unlike IFRS IAS 16)

#### 2.3.8 Revenue Definition — §277(1) HGB (post-BilRUG)

Since BilRUG (2016), HGB revenue (Umsatzerlöse) includes:
- Revenue from sale of products and goods
- Revenue from services
- Revenue from rental of assets
This is now aligned with IFRS 15 scope (previously HGB excluded some of these).

### 2.4 German Financial Statement Formats

#### Bilanz (Balance Sheet) — §266 HGB

Must be prepared in **Kontoform** (account form): Aktiva left, Passiva right.

**Aktiva (Assets):**
```
A. Anlagevermögen (Fixed Assets)
   I.   Immaterielle Vermögensgegenstände
   II.  Sachanlagen
   III. Finanzanlagen
B. Umlaufvermögen (Current Assets)
   I.   Vorräte
   II.  Forderungen und sonstige Vermögensgegenstände
   III. Wertpapiere
   IV.  Kassenbestand, Bankguthaben
C. Rechnungsabgrenzungsposten (Prepaid expenses)
D. Aktive latente Steuern
```

**Passiva (Equity & Liabilities):**
```
A. Eigenkapital
   I.   Gezeichnetes Kapital
   II.  Kapitalrücklage
   III. Gewinnrücklagen
   IV.  Gewinnvortrag / Verlustvortrag
   V.   Jahresüberschuss / Jahresfehlbetrag
B. Rückstellungen (Provisions)
C. Verbindlichkeiten (Liabilities)
D. Rechnungsabgrenzungsposten (Deferred income)
E. Passive latente Steuern
```

Small corporations (§267(1)): only letters + Roman numerals. Micro corporations (§267a): only letters.

#### Gewinn- und Verlustrechnung (GuV / Income Statement) — §275 HGB

Prepared in **Staffelform** (vertical format). Two methods available:

**Gesamtkostenverfahren (GKV, §275(2)) — Nature of Expense Method** (~90% of German companies):
1. Umsatzerlöse → 2. Bestandsveränderungen → 3. Aktivierte Eigenleistungen → 4. Sonstige betriebliche Erträge → 5. **Materialaufwand** → 6. **Personalaufwand** → 7. **Abschreibungen** → 8. Sonstige betriebliche Aufwendungen → 9–13. Finanzergebnis → 14. Steuern → 17. Jahresüberschuss/-fehlbetrag

**Umsatzkostenverfahren (UKV, §275(3)) — Cost of Sales Method:**
1. Umsatzerlöse → 2. **Herstellungskosten** (COGS) → 3. Bruttoergebnis → 4. **Vertriebskosten** → 5. **Verwaltungskosten** → 6–7. Sonstige → 8–12. Finanzergebnis → 13. Steuern → 16. Jahresüberschuss/-fehlbetrag

When using UKV, §285 Nr. 8 HGB requires Materialaufwand and Personalaufwand disclosure in notes (Anhang).

**Impact on DataSynth**: The financial statement generator should support both formats; default to GKV. SKR04 account structure maps directly to GKV line items.

### 2.5 GoBD Audit Export (German FEC Equivalent)

The **GoBD** (Grundsätze zur ordnungsmäßigen Führung und Aufbewahrung von Büchern, Aufzeichnungen und Unterlagen in elektronischer Form) replaces the older GDPdU and defines requirements for:

1. **Digital bookkeeping**: Completeness, correctness, timely recording, ordering, immutability (Unveränderbarkeit)
2. **Data retention**: 10 years for financial statements, **8 years** for Buchungsbelege (reduced from 10 by BEG IV, effective 2025), 6 years for business correspondence
3. **Data access for tax audits**: Three levels — Z1 (direct read-only access), Z2 (indirect/query-based), Z3 (data carrier handover)
4. **Procedural documentation (Verfahrensdokumentation)**: Every IT system must document content, structure, process flow, and controls

**Version history:** First published 2014 (replaced GDPdU/GoBS), updated 2020, editorially amended Mar 2024, **second amendment 14 Jul 2025** (aligned with mandatory B2B e-invoicing — clarifies XML archiving for ZUGFeRD, removes mandatory separate PDF storage).

**Non-compliance penalties:** Up to EUR 250,000 for failure to provide data during audit; potential criminal tax law consequences.

#### GoBD Export Format (Z3 — Datenträgerüberlassung)

The GoBD export uses a structured XML index file (`index.xml`) plus CSV/fixed-width data files:

| File | Content |
|---|---|
| `index.xml` | Metadata: taxpayer, fiscal year, data descriptions, table schemas |
| `journal.csv` | Journal entries (Buchungsjournal) |
| `accounts.csv` | Chart of accounts (Kontenplan) |
| `account_balances.csv` | Account balances (Kontensalden) |
| `customers.csv` | Customer master data (Debitorenstamm) |
| `vendors.csv` | Vendor master data (Kreditorenstamm) |
| `fixed_assets.csv` | Fixed asset register (Anlagenverzeichnis) |

The `index.xml` follows the **IDEA/AIS TaxAudit** schema (or the newer **DSFinV-K** for cash registers). For journal entries, the minimum columns are:

| # | Field | German | Description |
|---|---|---|---|
| 1 | Belegdatum | Document date | Date of the source document |
| 2 | Buchungsdatum | Posting date | Date posted to GL |
| 3 | Belegnummer | Document number | Unique document reference |
| 4 | Buchungstext | Posting text | Description of the entry |
| 5 | Kontonummer | Account number | GL account (SKR04) |
| 6 | Gegenkontonummer | Offset account | Contra account |
| 7 | Sollbetrag | Debit amount | Debit in local currency |
| 8 | Habenbetrag | Credit amount | Credit in local currency |
| 9 | Steuerschlüssel | Tax code | VAT code |
| 10 | Steuerbetrag | Tax amount | VAT amount |
| 11 | Währung | Currency | ISO currency code |
| 12 | Kostenstelle | Cost center | Cost center code |
| 13 | Belegnummernkreis | Document number range | Journal/number range ID |

### 2.5 E-Bilanz (Electronic Tax Balance Sheet)

Since 2013 (§5b EStG), all German taxpayers must electronically submit their tax balance sheet to the Finanzamt using **XBRL** via the ELSTER/ERiC infrastructure:

- **Current taxonomy**: Version 6.8 (fiscal years after 31.12.2024), Version 6.9 (after 31.12.2025)
- **Mandatory submissions**: Commercial/tax balance sheet, P&L, tax-to-commercial reconciliation (Überleitungsrechnung), profit appropriation, capital account development (partnerships)
- **Industry taxonomies**: Core + supplements for banking, insurance, housing, agriculture, hospitals

**Impact on DataSynth**: The SKR04 account structure should be designed so that accounts map cleanly to E-Bilanz taxonomy positions. This is a metadata concern (not a generation concern) but the account hierarchy should reflect it.

### 2.6 E-Invoicing: ZUGFeRD and XRechnung

- **ZUGFeRD** (Zentraler User Guide des Forums elektronische Rechnung Deutschland): Hybrid PDF/A-3 + XML format (EN 16931 / CII syntax). ZUGFeRD 2.2+ includes an XRECHNUNG profile for B2G compatibility.
- **XRechnung**: Pure XML format (EN 16931 / UBL 2.1) maintained by KoSIT. Mandatory for B2G at federal level since Nov 2020. Transmitted via E-Rechnungsportal Bund (Peppol-enabled).

**B2B E-Invoicing Mandate Timeline (Wachstumschancengesetz):**

| Date | Requirement |
|---|---|
| **1 Jan 2025** | All businesses must be able to **receive** e-invoices |
| 2025–2026 | Transition: paper/PDF still allowed with recipient consent |
| **1 Jan 2027** | Companies with turnover > EUR 800,000 must **issue** e-invoices |
| **1 Jan 2028** | **All** businesses must issue e-invoices for domestic B2B |

**GoBD 2025 amendment impact**: Only the XML component of ZUGFeRD invoices must be archived (separate PDF storage no longer mandatory). Plain PDF is no longer a valid e-invoice.

**Impact on DataSynth**: The existing `DE.json` country pack already declares `"e_invoice_format": "ZUGFeRD"`. No additional code needed, but document generation should be aware of the format when producing invoice-like outputs.

---

## 3. Implementation Plan

### Phase 0: Foundation — Framework Enum & Config (Estimated: 1 batch)

#### Task 0.1: Add `GermanGaap` to `AccountingFramework` enum

**Files:**
- `crates/datasynth-standards/src/framework.rs`

**Changes:**
1. Add `GermanGaap` variant to `AccountingFramework` enum with doc comment:
   ```rust
   /// German GAAP (Handelsgesetzbuch – HGB).
   ///
   /// German statutory accounting framework:
   /// - SKR03/SKR04 chart of accounts
   /// - LIFO prohibited (§256 HGB)
   /// - Mandatory impairment reversal (§253(5) HGB)
   /// - Optional capitalization of self-created intangibles (§248(2) HGB)
   /// - Off-balance operating leases (no dedicated lease standard)
   /// - Provisions for pending losses mandatory (§249(1) HGB)
   GermanGaap,
   ```

2. Extend all `match` arms in `AccountingFramework` impl:
   - `revenue_standard()` → `"HGB §277 / BilRUG"`
   - `lease_standard()` → `"HGB / BMF-Leasingerlasse"`
   - `fair_value_standard()` → `"HGB §253(1) / §255(4)"`
   - `impairment_standard()` → `"HGB §253(3)-(5)"`
   - `allows_lifo()` → `false` (LIFO prohibited under HGB since BilMoG)
   - `requires_development_capitalization()` → `false` (optional under §248(2) HGB)
   - `allows_ppe_revaluation()` → `false` (HGB uses cost model, no revaluation above cost)
   - `allows_impairment_reversal()` → `true` (mandatory under §253(5) HGB)
   - `uses_brightline_lease_tests()` → `false`
   - `Display` → `"German GAAP (HGB)"`

3. Add HGB-specific framework methods:
   ```rust
   /// Returns whether pending-loss provisions (Drohverlustrückstellungen) are mandatory.
   pub fn requires_pending_loss_provisions(&self) -> bool {
       matches!(self, Self::GermanGaap)
   }

   /// Returns whether low-value asset immediate expensing is applicable.
   pub fn allows_low_value_asset_expensing(&self) -> bool {
       matches!(self, Self::GermanGaap)
   }

   /// Returns whether operating leases remain off-balance-sheet for lessees.
   pub fn operating_leases_off_balance(&self) -> bool {
       matches!(self, Self::GermanGaap)
   }
   ```

4. Add `FrameworkSettings::german_gaap()` constructor:
   ```rust
   pub fn german_gaap() -> Self {
       Self {
           framework: AccountingFramework::GermanGaap,
           use_lifo_inventory: false,           // Prohibited under HGB
           capitalize_development_costs: false,  // Optional (§248(2)), default off
           use_ppe_revaluation: false,           // Not permitted under HGB
           allow_impairment_reversal: true,      // Mandatory under §253(5)
           ..Default::default()
       }
   }
   ```

5. Update `validate()` — LIFO under `GermanGaap` should fail (same as IFRS/French GAAP):
   ```rust
   if self.use_lifo_inventory
       && matches!(
           self.framework,
           AccountingFramework::Ifrs
               | AccountingFramework::FrenchGaap
               | AccountingFramework::GermanGaap
       )
   {
       return Err(FrameworkValidationError::LifoNotPermittedUnderIfrs);
   }
   ```
   (Consider renaming the error variant to `LifoNotPermitted` for clarity.)

6. Add tests for `GermanGaap` variant (serialization roundtrip, features, settings validation).

**Verification:** `cargo test -p datasynth-standards`

#### Task 0.2: Add `GermanGaap` to config schema

**Files:**
- `crates/datasynth-config/src/schema.rs`

**Changes:**
1. Add `GermanGaap` to `AccountingFrameworkConfig` enum:
   ```rust
   /// German GAAP (Handelsgesetzbuch – HGB)
   GermanGaap,
   ```

2. Update doc comments on `AccountingStandardsConfig` to mention HGB.

3. Add serialization test for the new variant.

**Verification:** `cargo test -p datasynth-config`

---

### Phase 1: Chart of Accounts — SKR04 Data & Loader (Estimated: 1 batch)

#### Task 1.1: Create SKR04 reference data (`skr04_2024.json`)

**File (new):**
- `crates/datasynth-core/resources/skr04_2024.json`

**Structure:** Follow the same tree structure as `pcg_2024.json`:
```json
[
  {
    "number": 0,
    "label": "Anlagevermögen",
    "system": "base",
    "accounts": [
      {
        "number": 10,
        "label": "Immaterielle Vermögensgegenstände",
        "system": "condensed",
        "accounts": [
          { "number": 100, "label": "Konzessionen", "system": "base", "accounts": [] },
          { "number": 110, "label": "Gewerbliche Schutzrechte", "system": "base", "accounts": [] },
          { "number": 135, "label": "Selbst geschaffene immaterielle VG", "system": "developed", "accounts": [] },
          ...
        ]
      },
      ...
    ]
  },
  ...
]
```

**Source:** The SKR04 account plan is published by DATEV and is widely available in structured form. The JSON should contain ~500–1,500 accounts covering all 10 classes (0–9), with `system` field indicating `"base"`, `"condensed"`, or `"developed"` for complexity filtering.

**Key accounts to ensure are present:**

| SKR04 # | Description | Type |
|---|---|---|
| 0200–0499 | Sachanlagen (tangible fixed assets) | Asset |
| 0500–0699 | Finanzanlagen (financial fixed assets) | Asset |
| 0700–0899 | Kumulierte Abschreibungen | Contra-Asset |
| 1000–1099 | Fertige/unfertige Erzeugnisse | Asset (Inventory) |
| 1200 | Forderungen aus L+L | Asset (AR) |
| 1400 | Forderungen gg. verb. Unternehmen | Asset (IC-AR) |
| 1600 | Kasse | Asset (Cash) |
| 1800 | Bank | Asset (Cash) |
| 2000 | Gezeichnetes Kapital | Equity |
| 2900 | Jahresüberschuss/-fehlbetrag | Equity |
| 3000–3099 | Rückstellungen | Liability (Provisions) |
| 3100–3199 | Verbindlichkeiten gg. Kreditinstituten | Liability (Debt) |
| 3300 | Verbindlichkeiten aus L+L | Liability (AP) |
| 3500 | Verbindlichkeiten gg. verb. Unternehmen | Liability (IC-AP) |
| 3800 | Umsatzsteuer | Liability (VAT) |
| 4000–4399 | Umsatzerlöse | Revenue |
| 4400–4599 | Erlöse (Leistungen) | Revenue |
| 4900 | Sonstige betriebliche Erträge | Revenue |
| 5000–5199 | Materialaufwand | Expense (COGS) |
| 5900–5999 | Bestandsveränderungen | Expense |
| 6000–6099 | Löhne und Gehälter | Expense (Personnel) |
| 6100–6199 | Soziale Abgaben | Expense (Personnel) |
| 6200–6299 | Abschreibungen | Expense (Depreciation) |
| 6300–6499 | Sonstige betriebliche Aufwendungen | Expense (Operating) |
| 7000–7099 | Zinserträge | Revenue (Financial) |
| 7300–7399 | Zinsaufwendungen | Expense (Financial) |
| 7600–7699 | Steuern vom Einkommen und Ertrag | Expense (Tax) |
| 9000 | Saldenvorträge Sachkonten | Statistical |

#### Task 1.2: Create SKR constants module (`skr.rs`)

**File (new):**
- `crates/datasynth-core/src/skr.rs`

**Pattern:** Mirror `pcg.rs` structure. Define control account constants for SKR04:

```rust
//! SKR04 (Standardkontenrahmen 04) — German GAAP chart of accounts constants.
//!
//! SKR04 follows the Abschlussgliederungsprinzip (financial-statement-oriented):
//! - Class 0: Non-current assets (Anlagevermögen)
//! - Class 1: Current assets (Umlaufvermögen)
//! - Class 2: Equity (Eigenkapital)
//! - Class 3: Liabilities (Fremdkapital)
//! - Class 4: Operating revenue (Betriebliche Erträge)
//! - Class 5: Operating expenses – materials (Materialaufwand)
//! - Class 6: Operating expenses – personnel & other (Personalaufwand + sonstige)
//! - Class 7: Other income/expenses (Finanzergebnis + Steuern)
//! - Class 8: (reserved)
//! - Class 9: Statistical/carry-forward (Vortragskonten)

pub mod control_accounts {
    pub const AR_CONTROL: &str = "1200";
    pub const AP_CONTROL: &str = "3300";
    pub const INVENTORY: &str = "1100";
    pub const FIXED_ASSETS: &str = "0200";
    pub const ACCUMULATED_DEPRECIATION: &str = "0700";
    pub const GR_IR_CLEARING: &str = "3350";
    pub const IC_AR_CLEARING: &str = "1400";
    pub const IC_AP_CLEARING: &str = "3500";
}

pub mod cash_accounts {
    pub const OPERATING_CASH: &str = "1800";
    pub const BANK_ACCOUNT: &str = "1810";
    pub const PETTY_CASH: &str = "1600";
}

pub mod revenue_accounts {
    pub const PRODUCT_REVENUE: &str = "4000";
    pub const SERVICE_REVENUE: &str = "4400";
    pub const OTHER_REVENUE: &str = "4900";
    pub const SALES_DISCOUNTS: &str = "4730";
}

pub mod expense_accounts {
    pub const COGS: &str = "5000";
    pub const DEPRECIATION: &str = "6220";
    pub const SALARIES_WAGES: &str = "6000";
    pub const RENT: &str = "6310";
    pub const INTEREST_EXPENSE: &str = "7300";
}

pub mod equity_liability_accounts {
    pub const COMMON_STOCK: &str = "2000";
    pub const RETAINED_EARNINGS: &str = "2970";
    pub const PROVISIONS: &str = "3000";
    pub const SHORT_TERM_DEBT: &str = "3150";
    pub const LONG_TERM_DEBT: &str = "3160";
}

pub mod tax_accounts {
    pub const INPUT_VAT: &str = "1570";   // Vorsteuer
    pub const OUTPUT_VAT: &str = "3800";  // Umsatzsteuer
    pub const TAX_EXPENSE: &str = "7600"; // Steuern vom Einkommen und Ertrag
}

pub mod personnel_accounts {
    pub const WAGES_PAYABLE: &str = "3720"; // Verbindlichkeiten Löhne/Gehälter
}

/// Return the SKR04 class (0–9) from an account number string.
#[inline]
pub fn skr_class(account: &str) -> Option<u8> {
    let first = account.chars().next()?;
    first.to_digit(10).map(|d| d as u8)
}
```

#### Task 1.3: Create SKR loader module (`skr_loader.rs`)

**File (new):**
- `crates/datasynth-core/src/skr_loader.rs`

**Pattern:** Mirror `pcg_loader.rs`. Key differences:

1. **Account number format**: SKR04 uses 4-digit base accounts (not 6-digit like PCG). Normalize to 4 digits (right-pad with zeros).
2. **Class mapping**: Map classes 0–9 to `AccountType` and `AccountSubType`:
   - Class 0 → `Asset` / `FixedAssets` (0–6) or `AccumulatedDepreciation` (07–09)
   - Class 1 → `Asset` / various (Inventory 10–11, AR 12, Cash 16+18)
   - Class 2 → `Equity` / various
   - Class 3 → `Liability` / various
   - Class 4 → `Revenue` / various
   - Class 5 → `Expense` / `CostOfGoodsSold`
   - Class 6 → `Expense` / `OperatingExpenses`
   - Class 7 → mixed (Revenue + Expense for financial items and taxes)
   - Class 9 → `Asset` / `SuspenseClearing`

3. **Embedded JSON**: `include_str!("../resources/skr04_2024.json")`

4. **Public API**: `build_chart_of_accounts_from_skr04(complexity, industry) -> Result<ChartOfAccounts, ...>`

5. **Tests**: Verify tree loads, correct account count per complexity level, 4-digit format, class coverage.

#### Task 1.4: Wire SKR into `datasynth-core/src/lib.rs`

**File:**
- `crates/datasynth-core/src/lib.rs`

**Changes:** Add `pub mod skr;` and `pub mod skr_loader;` (mirroring the existing `pub mod pcg;` and `pub mod pcg_loader;`).

**Verification:** `cargo test -p datasynth-core`

---

### Phase 2: CoA Generator Integration (Estimated: 1 batch)

#### Task 2.1: Extend `ChartOfAccountsGenerator` with SKR support

**File:**
- `crates/datasynth-generators/src/coa_generator.rs`

**Changes:**
1. Add `use_german_skr: bool` field (mirroring `use_french_pcg`).
2. Add builder method:
   ```rust
   pub fn with_german_skr(mut self, use_skr: bool) -> Self {
       self.use_german_skr = use_skr;
       self
   }
   ```
3. Extend `generate()` to check `use_german_skr` before `use_french_pcg`:
   ```rust
   pub fn generate(&mut self) -> ChartOfAccounts {
       self.count += 1;
       if self.use_german_skr {
           self.generate_skr()
       } else if self.use_french_pcg {
           self.generate_pcg()
       } else {
           self.generate_default()
       }
   }
   ```
4. Add `generate_skr()` method that delegates to `skr_loader::build_chart_of_accounts_from_skr04()` with a `generate_skr_fallback()` that manually creates key accounts (same pattern as `generate_pcg()` / `generate_pcg_fallback()`).

5. Add tests: verify SKR CoA has `country == "DE"`, 4-digit accounts, correct class range.

**Verification:** `cargo test -p datasynth-generators`

---

### Phase 3: GoBD Audit Export (Estimated: 1 batch)

#### Task 3.1: Create GoBD export module

**File (new):**
- `crates/datasynth-output/src/formats/gobd.rs`

**Pattern:** Mirror `fec.rs`. The GoBD export produces:

1. **`gobd_journal.csv`** — semicolon-separated, UTF-8, 13+ columns (see §2.4)
2. **`gobd_accounts.csv`** — Chart of accounts export (Kontonummer, Kontobezeichnung, Kontotyp, Saldo)
3. **`gobd_index.xml`** — Metadata index file per GoBD specification

Key differences from FEC:
- Semicolon-separated (same as FEC)
- Includes **Gegenkontonummer** (offset account) — FEC doesn't have this
- Includes **Steuerschlüssel** and **Steuerbetrag** (tax code + tax amount) — FEC doesn't have these
- Date format: `YYYYMMDD` (same as FEC)
- Amounts: period as decimal separator (German convention) or dot — configurable

```rust
pub fn write_gobd_journal_csv(
    path: &Path,
    entries: &[JournalEntry],
    coa: &ChartOfAccounts,
) -> SynthResult<()> { ... }

pub fn write_gobd_accounts_csv(
    path: &Path,
    coa: &ChartOfAccounts,
) -> SynthResult<()> { ... }

pub fn write_gobd_index_xml(
    path: &Path,
    company_code: &str,
    fiscal_year: i32,
    tables: &[&str],
) -> SynthResult<()> { ... }
```

#### Task 3.2: Register GoBD in output module

**File:**
- `crates/datasynth-output/src/formats/mod.rs`
- `crates/datasynth-output/src/lib.rs`

**Changes:** Add `pub mod gobd;` and re-export `write_gobd_journal_csv`, `write_gobd_accounts_csv`, `write_gobd_index_xml`.

**Verification:** `cargo test -p datasynth-output`

---

### Phase 4: Orchestrator & CLI Wiring (Estimated: 1 batch)

#### Task 4.1: Wire `GermanGaap` in enhanced orchestrator

**File:**
- `crates/datasynth-runtime/src/enhanced_orchestrator.rs`

**Changes:**

1. In the framework resolution logic (~line 3663), add `GermanGaap` arm:
   ```rust
   Some(datasynth_config::schema::AccountingFrameworkConfig::GermanGaap) => {
       datasynth_standards::framework::AccountingFramework::GermanGaap
   }
   ```

2. In the country pack auto-detection (~line 3675), add `"german_gaap"` or `"hgb"`:
   ```rust
   "german_gaap" | "hgb" => {
       datasynth_standards::framework::AccountingFramework::GermanGaap
   }
   ```

3. In `generate_coa()` (~line 4913), add SKR detection:
   ```rust
   let use_german_skr = self.config.accounting_standards.enabled
       && matches!(
           self.config.accounting_standards.framework,
           Some(datasynth_config::schema::AccountingFrameworkConfig::GermanGaap)
       );
   ```
   Then chain `.with_german_skr(use_german_skr)` on the generator builder.

#### Task 4.2: Wire `GermanGaap` in streaming orchestrator

**File:**
- `crates/datasynth-runtime/src/streaming_orchestrator.rs`

**Changes:** Same pattern as enhanced orchestrator — detect `GermanGaap` config and set `use_german_skr` flag.

#### Task 4.3: Wire GoBD export in CLI

**File:**
- `crates/datasynth-cli/src/main.rs`

**Changes:** After the FEC export block (~line 589), add GoBD export block:
```rust
// Write GoBD (German GAAP) audit export
if matches!(
    config_for_manifest.accounting_standards.framework,
    Some(AccountingFrameworkConfig::GermanGaap)
) && !result.journal_entries.is_empty()
{
    let gobd_dir = output.join("gobd_export");
    std::fs::create_dir_all(&gobd_dir)?;

    // Journal entries
    let journal_path = gobd_dir.join("gobd_journal.csv");
    write_gobd_journal_csv(&journal_path, &result.journal_entries, &result.chart_of_accounts)?;

    // Chart of accounts
    let accounts_path = gobd_dir.join("gobd_accounts.csv");
    write_gobd_accounts_csv(&accounts_path, &result.chart_of_accounts)?;

    // Index XML
    let company_code = config_for_manifest.companies.first()
        .map(|c| c.code.as_str()).unwrap_or("1000");
    let fiscal_year = /* extract from start_date */;
    write_gobd_index_xml(
        &gobd_dir.join("index.xml"),
        company_code,
        fiscal_year,
        &["gobd_journal.csv", "gobd_accounts.csv"],
    )?;

    tracing::info!("GoBD export written to: {}", gobd_dir.display());
}
```

**Verification:** `cargo test -p datasynth-cli` and `cargo test -p datasynth-runtime`

---

### Phase 5: HGB-Specific Accounting Treatments (Estimated: 2 batches)

#### Task 5.1: Impairment generator — HGB rules

**File:**
- `crates/datasynth-standards/src/accounting/impairment.rs`

**Changes:** The impairment generator already has an `AccountingFramework` parameter. Add `GermanGaap` arms:
- Impairment reversal: **mandatory** (not just permitted)
- No two-step test (HGB uses direct comparison of carrying amount vs recoverable amount)
- Goodwill impairment: reversible under HGB

#### Task 5.2: Lease generator — HGB off-balance treatment

**File:**
- `crates/datasynth-standards/src/accounting/leases.rs`

**Changes:** When `GermanGaap` is selected:
- Skip ROU asset and lease liability generation
- Generate only operating lease expense entries (Mietaufwand / Leasingaufwand)
- Use German classification test: 40%–90% of useful life → operating
- `finance_lease_percent` config should be overridden to a low value (~5–10%) under HGB

#### Task 5.3: Provisions generator — HGB Drohverlustrückstellungen

**File (new consideration):**
- `crates/datasynth-generators/src/standards/provision_generator.rs` (or extend existing accruals generator)

**Changes:** When `GermanGaap`:
- Generate provisions for pending losses (Drohverlustrückstellungen) for a configurable percentage of open purchase commitments
- Apply Bundesbank discount rate for long-term provisions (§253(2) HGB)
- Generate corresponding JEs: Dr. Expense / Cr. Provisions (Class 3 accounts in SKR04)

#### Task 5.4: Low-value assets — GWG immediate expensing

**File:**
- `crates/datasynth-generators/src/master_data/asset_generator.rs` (or period_close)

**Changes:** When `GermanGaap`:
- Assets with acquisition cost ≤ 800 EUR → generate immediate write-off JE (full depreciation in Year 1)
- Assets 250–1,000 EUR → optionally generate pool depreciation (Sammelposten) over 5 years
- Tag these in the fixed asset register with `gwg: true`

#### Task 5.5: Goodwill amortization — HGB mandatory

**File:**
- `crates/datasynth-generators/src/period_close/depreciation.rs` (or a new goodwill module)

**Changes:** When `GermanGaap`:
- Goodwill must amortize over useful life (§253(3) S.4 HGB, max 10 years if life unestimable)
- Default to 5 years straight-line
- Unlike IFRS (impairment-only, no amortization)

#### Task 5.6: Depreciation — German methods and AfA tables

**File:**
- `crates/datasynth-generators/src/period_close/depreciation.rs`

**Changes:** When `GermanGaap`:
- Support declining balance (degressiv) depreciation: max 3x linear, capped at 30% (Jul 2025–Dec 2027 per §7(2) EStG)
- Support Sonderabschreibung for SMEs: up to 40% additional in first 5 years (§7g(5) EStG)
- Default useful lives from AfA-Tabellen (e.g., office equipment 13 years, computers 3 years, vehicles 6 years, buildings 33 years)
- No component approach (unlike IFRS)

---

### Phase 6: UI, Docs & Tests (Estimated: 1 batch)

#### Task 6.1: Update desktop UI framework selector

**File:**
- `crates/datasynth-ui/src/routes/config/accounting-standards/+page.svelte`

**Changes:** Add `"german_gaap"` option to the framework dropdown (mirroring the existing `"french_gaap"` option).

#### Task 6.2: Update documentation

**Files:**
- `docs/src/advanced/accounting-standards.md`
- `CLAUDE.md` (if needed)
- `CHANGELOG.md`

**Changes:** Document `german_gaap` as a valid `framework` option, describe SKR04, GoBD export, HGB-specific treatments.

#### Task 6.3: Integration tests

**Files:**
- `crates/datasynth-standards/tests/integration_tests.rs` — add `GermanGaap` tests
- `crates/datasynth-runtime/tests/generation_integration.rs` — add end-to-end test generating with `GermanGaap` framework

**Test cases:**
1. `test_german_gaap_settings()` — validate `FrameworkSettings::german_gaap()`
2. `test_german_gaap_coa_generation()` — generate SKR04 CoA, verify DE country, 4-digit accounts, class coverage
3. `test_german_gaap_framework_features()` — verify `allows_lifo() == false`, `allows_impairment_reversal() == true`, `requires_pending_loss_provisions() == true`
4. `test_german_gaap_gobd_export()` — generate entries → write GoBD CSV → verify column count and content
5. `test_german_gaap_end_to_end()` — full orchestrator run with `framework: german_gaap`, verify SKR04 accounts and GoBD output

#### Task 6.4: Update Python wrapper

**File:**
- `python/datasynth_py/config/models.py`

**Changes:** Add `"german_gaap"` to the framework enum options (mirroring `"french_gaap"`).

---

### Phase 7: Update DE Country Pack (Estimated: minor)

#### Task 7.1: Set DE.json primary framework to support HGB

**File:**
- `crates/datasynth-core/country-packs/DE.json`

**Changes:** Update the `accounting` section:
```json
{
  "accounting": {
    "framework": "german_gaap",
    "secondary_framework": "ifrs",
    "local_gaap_name": "HGB",
    "chart_of_accounts": {
      "standard": "SKR04",
      "alternative_standard": "SKR03",
      "numbering_length": 4,
      "account_ranges": {
        "fixed_assets": { "from": "0", "to": "0" },
        "current_assets": { "from": "1", "to": "1" },
        "equity": { "from": "2", "to": "2" },
        "liabilities": { "from": "3", "to": "3" },
        "revenue": { "from": "4", "to": "4" },
        "expenses_materials": { "from": "5", "to": "5" },
        "expenses_other": { "from": "6", "to": "6" },
        "financial_other": { "from": "7", "to": "7" },
        "statistical": { "from": "9", "to": "9" }
      }
    },
    ...
  }
}
```

---

## 4. File Change Summary

| File | Action | Description |
|---|---|---|
| `crates/datasynth-standards/src/framework.rs` | Modify | Add `GermanGaap` variant + methods |
| `crates/datasynth-config/src/schema.rs` | Modify | Add `GermanGaap` to config enum |
| `crates/datasynth-core/resources/skr04_2024.json` | **New** | SKR04 reference data (~3,000 lines) |
| `crates/datasynth-core/src/skr.rs` | **New** | SKR04 control account constants |
| `crates/datasynth-core/src/skr_loader.rs` | **New** | SKR04 tree loader + CoA builder |
| `crates/datasynth-core/src/lib.rs` | Modify | Export `skr` and `skr_loader` modules |
| `crates/datasynth-generators/src/coa_generator.rs` | Modify | Add `use_german_skr` flag + generation |
| `crates/datasynth-output/src/formats/gobd.rs` | **New** | GoBD journal/accounts/index export |
| `crates/datasynth-output/src/formats/mod.rs` | Modify | Register `gobd` module |
| `crates/datasynth-output/src/lib.rs` | Modify | Re-export GoBD functions |
| `crates/datasynth-runtime/src/enhanced_orchestrator.rs` | Modify | Wire `GermanGaap` framework + SKR CoA |
| `crates/datasynth-runtime/src/streaming_orchestrator.rs` | Modify | Wire `GermanGaap` in streaming path |
| `crates/datasynth-cli/src/main.rs` | Modify | GoBD export output + import |
| `crates/datasynth-standards/src/accounting/impairment.rs` | Modify | HGB impairment rules |
| `crates/datasynth-standards/src/accounting/leases.rs` | Modify | HGB off-balance operating leases |
| `crates/datasynth-generators/src/period_close/depreciation.rs` | Modify | German depreciation methods |
| `crates/datasynth-generators/src/master_data/asset_generator.rs` | Modify | GWG low-value asset logic |
| `crates/datasynth-core/country-packs/DE.json` | Modify | Update framework to `german_gaap` |
| `crates/datasynth-ui/.../+page.svelte` | Modify | Add `german_gaap` UI option |
| `crates/datasynth-standards/tests/integration_tests.rs` | Modify | HGB integration tests |
| `python/datasynth_py/config/models.py` | Modify | Add `german_gaap` to Python enum |
| `docs/src/advanced/accounting-standards.md` | Modify | Document HGB support |

**Total: 10 new files, 12 modified files**

---

## 5. Configuration Example

```yaml
global:
  industry: manufacturing
  start_date: "2025-01-01"
  period_months: 12
  seed: 42

companies:
  - code: "DE01"
    name: "Müller Maschinenbau GmbH"
    currency: "EUR"
    country: "DE"

chart_of_accounts:
  complexity: medium

accounting_standards:
  enabled: true
  framework: german_gaap
  revenue_recognition:
    enabled: true
  leases:
    enabled: true
    lease_count: 30
    finance_lease_percent: 0.05  # Most leases off-balance under HGB
  fair_value:
    enabled: false  # Less relevant under HGB
  impairment:
    enabled: true   # Mandatory reversal under §253(5) HGB

audit_standards:
  enabled: true
  isa_compliance:
    enabled: true
    compliance_level: standard
    framework: isa   # Germany uses ISA (not PCAOB)
  sox:
    enabled: false   # SOX not applicable to German companies

output:
  format: csv
  output_directory: "./output"
```

**Expected output directory:**
```
output/
├── journal_entries.csv        # Standard JE export
├── chart_of_accounts.csv      # SKR04 accounts (4-digit)
├── gobd_export/               # GoBD audit files
│   ├── index.xml              # GoBD metadata index
│   ├── gobd_journal.csv       # GoBD-compliant journal (13+ columns)
│   └── gobd_accounts.csv      # Account plan export
├── vendors.csv
├── customers.csv
├── ...                        # All other standard outputs
```

---

## 6. Risk Assessment & Open Questions

### Risks

| Risk | Mitigation |
|---|---|
| SKR04 data quality — public sources may have errors | Cross-reference multiple sources; add automated validation test (class range, account uniqueness) |
| SKR03 demand — some users may require SKR03 instead of SKR04 | Design loader to be chart-agnostic; add `skr03_2024.json` in a follow-up phase |
| GoBD format compliance — no official machine-readable spec | Follow DATEV export format as de facto standard; document deviations |
| Interaction with existing `DE.json` country pack framework field | The country pack currently says `"framework": "ifrs"`. Changing to `"german_gaap"` may affect auto-detection logic for IFRS-first users. Solution: only override when user explicitly sets `framework: german_gaap` in YAML config |
| E-Bilanz XBRL taxonomy mapping | Out of scope for initial implementation; add as metadata annotation in a future phase |

### Open Questions

1. **SKR03 support**: Should we ship both SKR03 and SKR04 from day one, or start with SKR04 only and add SKR03 later? **Recommendation**: SKR04 first (simpler, financial-statement-oriented), SKR03 in a follow-up.

2. **DE.json framework default**: Should we change `DE.json`'s `"framework"` from `"ifrs"` to `"german_gaap"`? This would affect users who set `country: "DE"` without explicitly choosing a framework. **Recommendation**: Keep `"ifrs"` as default in the country pack (since large German companies do use IFRS); only activate HGB when user explicitly sets `framework: german_gaap`.

3. **Dual reporting (HGB + IFRS)**: Many German companies maintain both HGB statutory books and IFRS reporting. Should we add a `HgbIfrs` dual reporting mode? **Recommendation**: Defer to a follow-up; the existing `DualReporting` (US GAAP + IFRS) pattern can be extended.

4. **GWG threshold**: The 800 EUR GWG threshold changes periodically in German tax law. Should it be hardcoded or configurable? **Recommendation**: Configurable via `FrameworkSettings` with 800 EUR default.

---

## 7. Acceptance Criteria

- [ ] `cargo build --release` succeeds with no new warnings
- [ ] `cargo test` passes all existing + new tests
- [ ] `cargo fmt && cargo clippy` clean
- [ ] `framework: german_gaap` in YAML config → SKR04 CoA with 4-digit DE accounts
- [ ] Generated journal entries reference valid SKR04 account numbers
- [ ] `gobd_export/` directory created with valid `index.xml`, `gobd_journal.csv`, `gobd_accounts.csv`
- [ ] GoBD journal CSV has ≥13 columns with correct headers
- [ ] Impairment generator produces mandatory reversals under `GermanGaap`
- [ ] Lease generator produces off-balance operating leases by default under `GermanGaap`
- [ ] UI framework dropdown includes `German GAAP (HGB)` option
- [ ] Python wrapper accepts `"german_gaap"` as framework value
- [ ] `AccountingFramework::GermanGaap` serializes/deserializes as `"german_gaap"`
- [ ] All HGB-specific methods (`requires_pending_loss_provisions()`, `allows_low_value_asset_expensing()`, `operating_leases_off_balance()`) return correct values

---

## 8. Key HGB Section Reference

| Section | Topic |
|---|---|
| §238–263 HGB | General bookkeeping and accounting obligations |
| §242 HGB | Obligation to prepare Bilanz and GuV |
| §246 HGB | Completeness, prohibition of offsetting |
| §247 HGB | Content of the balance sheet (fixed/current assets) |
| §248 HGB | Self-created intangible assets (capitalization option) |
| §249 HGB | Provisions (Rückstellungen) |
| §252 HGB | General valuation principles (GoB, prudence, going concern) |
| §253 HGB | Valuation at acquisition/production cost, depreciation, discounting |
| §255 HGB | Definitions: acquisition cost, production cost |
| §256a HGB | Foreign currency translation |
| §257 HGB | Document retention periods (8/10 years) |
| §264 HGB | Annual financial statements of Kapitalgesellschaften |
| §266 HGB | Balance sheet format (Bilanz) |
| §267 HGB | Size classification (micro/small/medium/large) |
| §274 HGB | Deferred taxes |
| §275 HGB | GuV format (GKV and UKV methods) |
| §277 HGB | Revenue definition (post-BilRUG) |
| §285 HGB | Notes disclosure requirements |
| §5b EStG | E-Bilanz requirement |
| §6(2) EStG | GWG (low-value assets) |
| §7 EStG | Depreciation methods (AfA) |
| §7g(5) EStG | Sonderabschreibung for SMEs |
| §§146–147 AO | Electronic bookkeeping and retention (GoBD basis) |

---

## 9. References

- [HGB Official English Translation (PDF)](https://www.gesetze-im-internet.de/englisch_hgb/englisch_hgb.pdf)
- [IFRS vs German GAAP — EY Scout Comparison (2022)](https://www.ey.com/content/dam/ey-unified-site/ey-com/de-de/technical/ifrs-ver%C3%B6ffentlichungen/documents/ey-de-ifrs-vs-german-gaap-march-2022.pdf)
- [IFRS compared to German GAAP and Dutch GAAP — KPMG (2024)](https://assets.kpmg.com/content/dam/kpmg/nl/pdf/2024/services/IFRS-dutch-german-GAAP.pdf)
- [German GAAP (HGB) vs IFRS Overview — GlobalConnect](https://globalconnectadmin.com/german-gaap-handelsgesetzbuch-hgb-vs-ifrs-understanding-germanys-accounting-framework-3/)
- [New HGB Thresholds 2025 — Mauer WPG](https://www.mauer-wpg.com/en/insights/new-thresholds-in-the-german-commercial-code-hgb-implications-for-your-corporation)
- [Changes in German Tax and Commercial Law 2025 — Ebner Stolz](https://www.ebnerstolz.de/en/insights/changes-german-tax-and-commercial-law-2025-72775.html)
- [SKR03 and SKR04 Differences — Ralf100M](https://ralf100m.de/en/chart-of-accounts-skr-03-and-skr-04-what-are-the-differences/)
- [GoBD Explained — Fiskaly](https://www.fiskaly.com/blog/understanding-gobd-compliant-archiving)
- [Germany Updates GoBD Rules for 2025 E-Invoicing — Dynatos](https://www.dynatos.com/blog/germany-updates-gobd-rules-for-2025-e-invoicing-mandate/)
- [GDPdU File Structure — MindBridge](https://support.mindbridge.ai/hc/en-us/articles/16638376071831-Formatting-GDPdU-How-do-GDPdU-Files-Work)
- [German Audit File (GDPdU/GoBD) — Microsoft Dynamics 365](https://learn.microsoft.com/en-us/dynamics365/finance/localizations/germany/emea-deu-gdpdu-audit-data-export)
- [E-Bilanz Overview — firma.de](https://www.firma.de/en/accountancy/e-bilanz-what-is-the-e-balance-sheet-in-germany/)
- [Germany E-Invoicing B2B Mandate Timeline — VATupdate](https://www.vatupdate.com/2025/11/12/germany-e-invoicing-b2b-mandate-timeline-and-compliance/)
- [XRechnung Guide — Invoice-Converter](https://www.invoice-converter.com/en/blog/xrechnung-guide-2025)
- [German Lease Accounting under HGB — Nakisa](https://nakisa.com/blog/know-your-local-gaap-accounting-for-leases-under-german-gaap-bilanzrechtsmodernisierungsgesetz-bilmog/)
- [Impairment Test under IFRS & HGB — Rödl & Partner](https://www.roedl.com/insights/reporting-trends-solutions/2024-1/pressure-is-on-impairment-test-according-to-ifrs-hgb)
