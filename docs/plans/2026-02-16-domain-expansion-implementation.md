# Domain Expansion Implementation Plan: Tax, Treasury, Project Accounting, ESG

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add four complementary synthetic data domains (Tax Accounting, Treasury & Cash Management, Project Accounting, ESG Reporting) that create dense cross-linkages with existing DataSynth modules.

**Architecture:** Each domain follows the existing pattern: models in `datasynth-core/src/models/`, config in `datasynth-config/src/schema.rs`, generators in `datasynth-generators/src/`, exports in `datasynth-output/src/`. No new crates. The existing `project.rs` model will be extended (not replaced) for project accounting.

**Tech Stack:** Rust, serde, rust_decimal (with `#[serde(with = "rust_decimal::serde::str")]`), chrono, rand/rand_chacha (ChaCha8Rng for deterministic generation)

**Design doc:** `docs/plans/2026-02-16-domain-expansion-design.md`

---

## Phase 1: Tax Accounting & Compliance

Tax is implemented first because it decorates every existing transaction type with a new dimension and all other new domains need tax lines.

### Task 1: Tax Data Models

**Files:**
- Create: `crates/datasynth-core/src/models/tax.rs`
- Modify: `crates/datasynth-core/src/models/mod.rs:37` (add module declaration)
- Modify: `crates/datasynth-core/src/models/mod.rs:110` (add re-export)

**Step 1: Write failing test for TaxCode model**

Add to the bottom of `crates/datasynth-core/src/models/tax.rs`:

```rust
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;

    #[test]
    fn test_tax_code_creation() {
        let code = TaxCode {
            id: "TC-001".to_string(),
            code: "VAT-STD-DE".to_string(),
            description: "German standard VAT".to_string(),
            tax_type: TaxType::Vat,
            rate: Decimal::new(19, 2), // 0.19
            jurisdiction_id: "DE".to_string(),
            effective_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            expiry_date: None,
            is_reverse_charge: false,
            is_exempt: false,
        };
        assert_eq!(code.tax_amount(Decimal::from(1000)), Decimal::from(190));
        assert!(code.is_active(NaiveDate::from_ymd_opt(2025, 6, 15).unwrap()));
    }

    #[test]
    fn test_tax_line_creation() {
        let line = TaxLine {
            id: "TL-001".to_string(),
            document_type: TaxableDocumentType::VendorInvoice,
            document_id: "VI-001".to_string(),
            line_number: 1,
            tax_code_id: "TC-001".to_string(),
            jurisdiction_id: "DE".to_string(),
            taxable_amount: Decimal::from(1000),
            tax_amount: Decimal::from(190),
            is_deductible: true,
            is_reverse_charge: false,
            is_self_assessed: false,
        };
        assert!(line.is_deductible);
        assert_eq!(line.effective_rate(), Decimal::new(19, 2));
    }

    #[test]
    fn test_tax_return_net_payable() {
        let ret = TaxReturn {
            id: "TR-001".to_string(),
            entity_id: "C001".to_string(),
            jurisdiction_id: "DE".to_string(),
            period_start: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            period_end: NaiveDate::from_ymd_opt(2025, 3, 31).unwrap(),
            return_type: TaxReturnType::VatReturn,
            status: TaxReturnStatus::Draft,
            total_output_tax: Decimal::from(50000),
            total_input_tax: Decimal::from(30000),
            net_payable: Decimal::from(20000),
            filing_deadline: NaiveDate::from_ymd_opt(2025, 5, 10).unwrap(),
            actual_filing_date: None,
            is_late: false,
        };
        assert_eq!(ret.net_payable, ret.total_output_tax - ret.total_input_tax);
        assert!(!ret.is_filed());
    }

    #[test]
    fn test_tax_provision() {
        let prov = TaxProvision {
            id: "TP-001".to_string(),
            entity_id: "C001".to_string(),
            period: NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
            current_tax_expense: Decimal::from(210000),
            deferred_tax_asset: Decimal::from(15000),
            deferred_tax_liability: Decimal::from(25000),
            statutory_rate: Decimal::new(21, 2),
            effective_rate: Decimal::new(23, 2),
            rate_reconciliation: vec![
                RateReconciliationItem {
                    description: "State taxes".to_string(),
                    rate_impact: Decimal::new(2, 2),
                },
            ],
        };
        assert!(prov.effective_rate > prov.statutory_rate);
        assert_eq!(prov.net_deferred_tax(), Decimal::from(-10000)); // liability > asset
    }

    #[test]
    fn test_withholding_tax_record() {
        let wht = WithholdingTaxRecord {
            id: "WHT-001".to_string(),
            payment_id: "PAY-001".to_string(),
            vendor_id: "V-001".to_string(),
            withholding_type: WithholdingType::ServiceWithholding,
            treaty_rate: Some(Decimal::new(15, 2)),
            statutory_rate: Decimal::new(30, 2),
            applied_rate: Decimal::new(15, 2),
            base_amount: Decimal::from(10000),
            withheld_amount: Decimal::from(1500),
            certificate_number: Some("WHT-CERT-2025-001".to_string()),
        };
        assert!(wht.has_treaty_benefit());
        assert_eq!(wht.treaty_savings(), Decimal::from(1500)); // (30% - 15%) * 10000
    }

    #[test]
    fn test_uncertain_tax_position() {
        let utp = UncertainTaxPosition {
            id: "UTP-001".to_string(),
            entity_id: "C001".to_string(),
            description: "R&D credit position".to_string(),
            tax_benefit: Decimal::from(500000),
            recognition_threshold: Decimal::new(50, 2),
            recognized_amount: Decimal::from(350000),
            measurement_method: TaxMeasurementMethod::MostLikelyAmount,
        };
        assert!(utp.recognized_amount <= utp.tax_benefit);
        assert_eq!(utp.unrecognized_amount(), Decimal::from(150000));
    }

    #[test]
    fn test_jurisdiction_hierarchy() {
        let federal = TaxJurisdiction {
            id: "US".to_string(),
            name: "United States".to_string(),
            country_code: "US".to_string(),
            region_code: None,
            jurisdiction_type: JurisdictionType::Federal,
            parent_jurisdiction_id: None,
            vat_registered: false,
        };
        let state = TaxJurisdiction {
            id: "US-CA".to_string(),
            name: "California".to_string(),
            country_code: "US".to_string(),
            region_code: Some("CA".to_string()),
            jurisdiction_type: JurisdictionType::State,
            parent_jurisdiction_id: Some("US".to_string()),
            vat_registered: false,
        };
        assert!(federal.parent_jurisdiction_id.is_none());
        assert_eq!(state.parent_jurisdiction_id.as_deref(), Some("US"));
        assert!(state.is_subnational());
    }

    #[test]
    fn test_serde_roundtrip() {
        let code = TaxCode {
            id: "TC-001".to_string(),
            code: "VAT-STD-DE".to_string(),
            description: "German standard VAT".to_string(),
            tax_type: TaxType::Vat,
            rate: Decimal::new(19, 2),
            jurisdiction_id: "DE".to_string(),
            effective_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            expiry_date: None,
            is_reverse_charge: false,
            is_exempt: false,
        };
        let json = serde_json::to_string(&code).unwrap();
        let deserialized: TaxCode = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.rate, code.rate);
        assert_eq!(deserialized.tax_type, code.tax_type);
    }
}
```

**Step 2: Implement tax models**

Create `crates/datasynth-core/src/models/tax.rs` with all types. Key types:
- `TaxJurisdiction` (hierarchy: Federal > State > Local > Municipal > Supranational)
- `JurisdictionType` enum
- `TaxType` enum (Vat, Gst, SalesTax, IncomeTax, WithholdingTax, PayrollTax, ExciseTax)
- `TaxCode` with `tax_amount()` and `is_active()` methods
- `TaxableDocumentType` enum (VendorInvoice, CustomerInvoice, JournalEntry, Payment, PayrollRun)
- `TaxLine` with `effective_rate()` method
- `TaxReturnType`, `TaxReturnStatus` enums
- `TaxReturn` with `is_filed()`, `is_late()` methods
- `TaxProvision` with `net_deferred_tax()` method
- `RateReconciliationItem`
- `UncertainTaxPosition` with `unrecognized_amount()` method, `TaxMeasurementMethod` enum
- `WithholdingType` enum, `WithholdingTaxRecord` with `has_treaty_benefit()`, `treaty_savings()`

All Decimal fields use `#[serde(with = "rust_decimal::serde::str")]`. All enums use `#[serde(rename_all = "snake_case")]` and `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]` with `#[default]` on one variant.

**Step 3: Register in mod.rs**

In `crates/datasynth-core/src/models/mod.rs`:
- Add `mod tax;` after line 36 (after `mod user;`)
- Add `pub use tax::*;` after line 109 (after `pub use user::*;`)

**Step 4: Run tests to verify**

Run: `cargo test -p datasynth-core tax:: --lib`
Expected: All 8 tests pass

**Step 5: Commit**

```bash
git add crates/datasynth-core/src/models/tax.rs crates/datasynth-core/src/models/mod.rs
git commit -m "feat(models): add tax accounting data models

Tax jurisdictions, codes, lines, returns, provisions, uncertain positions,
and withholding records with full serde support and business logic methods."
```

---

### Task 2: Tax Config Schema

**Files:**
- Modify: `crates/datasynth-config/src/schema.rs:155-171` (add field to GeneratorConfig)
- Modify: `crates/datasynth-config/src/schema.rs` (add TaxConfig structs at end of file)

**Step 1: Write failing test for TaxConfig deserialization**

Add test to bottom of schema.rs (or a new test file):

```rust
#[test]
fn test_tax_config_defaults() {
    let config = TaxConfig::default();
    assert!(!config.enabled);
    assert!(config.vat_gst.standard_rates.is_empty());
    assert!(config.provisions.enabled); // provisions default to enabled when tax is enabled
}

#[test]
fn test_tax_config_from_yaml() {
    let yaml = r#"
tax:
  enabled: true
  jurisdictions:
    countries: [US, DE]
    include_subnational: true
  vat_gst:
    enabled: true
    standard_rates:
      DE: 0.19
      GB: 0.20
    reverse_charge: true
  withholding:
    enabled: true
    rates:
      default: 0.30
  anomaly_rate: 0.03
"#;
    let config: GeneratorConfig = serde_yaml::from_str(yaml).unwrap();
    assert!(config.tax.enabled);
    assert_eq!(config.tax.jurisdictions.countries.len(), 2);
    assert!(config.tax.vat_gst.reverse_charge);
}
```

**Step 2: Implement TaxConfig structs**

Add to `schema.rs` (before the closing of the file):

```rust
/// Tax accounting configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub jurisdictions: TaxJurisdictionConfig,
    #[serde(default)]
    pub vat_gst: VatGstConfig,
    #[serde(default)]
    pub sales_tax: SalesTaxConfig,
    #[serde(default)]
    pub withholding: WithholdingConfig,
    #[serde(default)]
    pub provisions: TaxProvisionConfig,
    #[serde(default)]
    pub payroll_tax: PayrollTaxConfig,
    #[serde(default = "default_tax_anomaly_rate")]
    pub anomaly_rate: f64,
}

impl Default for TaxConfig { /* all fields default, enabled: false */ }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxJurisdictionConfig {
    #[serde(default)]
    pub countries: Vec<String>,
    #[serde(default)]
    pub include_subnational: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VatGstConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub standard_rates: std::collections::HashMap<String, f64>,
    #[serde(default)]
    pub reduced_rates: std::collections::HashMap<String, f64>,
    #[serde(default)]
    pub exempt_categories: Vec<String>,
    #[serde(default)]
    pub reverse_charge: bool,
}

// + SalesTaxConfig, WithholdingConfig, TaxProvisionConfig, PayrollTaxConfig
// Each follows the same pattern: `enabled: bool` + domain-specific fields with defaults
```

Add field to `GeneratorConfig` (after line 170, before the closing `}`):

```rust
    /// Tax accounting configuration (jurisdictions, VAT/GST, withholding, provisions)
    #[serde(default)]
    pub tax: TaxConfig,
```

**Step 3: Run tests**

Run: `cargo test -p datasynth-config tax_config`
Expected: PASS

**Step 4: Verify existing tests still pass**

Run: `cargo test -p datasynth-config`
Expected: All existing tests pass (the new field has `#[serde(default)]` so existing configs won't break)

**Step 5: Commit**

```bash
git add crates/datasynth-config/src/schema.rs
git commit -m "feat(config): add tax accounting configuration schema

Supports jurisdiction hierarchy, VAT/GST rates, sales tax nexus,
withholding tax treaties, tax provisions (ASC 740), and anomaly injection."
```

---

### Task 3: Tax Code Generator

**Files:**
- Create: `crates/datasynth-generators/src/tax/mod.rs`
- Create: `crates/datasynth-generators/src/tax/tax_code_generator.rs`
- Modify: `crates/datasynth-generators/src/lib.rs:49` (add module)

**Step 1: Write failing test**

In `tax_code_generator.rs`:

```rust
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_tax_codes_for_countries() {
        let config = TaxCodeGeneratorConfig {
            countries: vec!["US".to_string(), "DE".to_string()],
            include_subnational: true,
            ..Default::default()
        };
        let mut gen = TaxCodeGenerator::new(42, config);
        let (jurisdictions, codes) = gen.generate();

        // Should have federal jurisdictions for both countries
        assert!(jurisdictions.iter().any(|j| j.country_code == "US"));
        assert!(jurisdictions.iter().any(|j| j.country_code == "DE"));

        // DE should have VAT codes
        assert!(codes.iter().any(|c| c.tax_type == TaxType::Vat && c.jurisdiction_id == "DE"));

        // US should have no VAT (uses sales tax)
        assert!(!codes.iter().any(|c| c.tax_type == TaxType::Vat && c.jurisdiction_id == "US"));

        // All codes should have valid rates
        for code in &codes {
            assert!(code.rate >= Decimal::ZERO);
            assert!(code.rate <= Decimal::ONE);
        }
    }

    #[test]
    fn test_deterministic() {
        let config = TaxCodeGeneratorConfig::default();
        let mut gen1 = TaxCodeGenerator::new(42, config.clone());
        let mut gen2 = TaxCodeGenerator::new(42, config);
        let (j1, c1) = gen1.generate();
        let (j2, c2) = gen2.generate();
        assert_eq!(j1.len(), j2.len());
        assert_eq!(c1.len(), c2.len());
    }
}
```

**Step 2: Implement TaxCodeGenerator**

```rust
pub struct TaxCodeGenerator {
    rng: ChaCha8Rng,
    config: TaxCodeGeneratorConfig,
}

impl TaxCodeGenerator {
    pub fn new(seed: u64, config: TaxCodeGeneratorConfig) -> Self { /* ... */ }
    pub fn generate(&mut self) -> (Vec<TaxJurisdiction>, Vec<TaxCode>) { /* ... */ }
}
```

Generates jurisdictions and tax codes based on configured countries. Includes built-in rate tables for common jurisdictions (US, DE, GB, FR, SG, JP, AU, BR, IN, etc.). For each country:
- Creates federal jurisdiction
- If `include_subnational`, creates state/province jurisdictions
- Creates appropriate tax codes (VAT for EU, GST for AU/SG/IN, Sales Tax for US states)

**Step 3: Register in lib.rs**

Add to `crates/datasynth-generators/src/lib.rs`:
- `pub mod tax;` after line 48 (after `pub mod sourcing;`)
- `pub use tax::*;` after line 81 (after `pub use standards::*;`)

**Step 4: Run tests**

Run: `cargo test -p datasynth-generators tax_code`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/datasynth-generators/src/tax/ crates/datasynth-generators/src/lib.rs
git commit -m "feat(generators): add tax code generator

Jurisdiction-aware tax code generation for VAT/GST/Sales Tax with
built-in rate tables for 10+ countries and subnational jurisdictions."
```

---

### Task 4: Tax Line Generator (Decorator Pattern)

**Files:**
- Create: `crates/datasynth-generators/src/tax/tax_line_generator.rs`
- Modify: `crates/datasynth-generators/src/tax/mod.rs`

This is the key decorator generator that attaches tax lines to existing AP/AR documents.

**Step 1: Write failing test**

```rust
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_generate_tax_lines_for_vendor_invoice() {
        let tax_codes = vec![
            TaxCode {
                id: "TC-VAT-STD".to_string(),
                code: "VAT-19".to_string(),
                description: "Standard VAT".to_string(),
                tax_type: TaxType::Vat,
                rate: Decimal::new(19, 2),
                jurisdiction_id: "DE".to_string(),
                effective_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                expiry_date: None,
                is_reverse_charge: false,
                is_exempt: false,
            },
        ];

        let mut gen = TaxLineGenerator::new(42, tax_codes, TaxLineGeneratorConfig::default());

        let lines = gen.generate_for_document(
            TaxableDocumentType::VendorInvoice,
            "VI-001",
            "DE",       // vendor country
            "DE",       // buyer country
            Decimal::from(10000),
            NaiveDate::from_ymd_opt(2025, 3, 15).unwrap(),
            None,       // no product category override
        );

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].tax_amount, Decimal::from(1900));
        assert!(lines[0].is_deductible); // input VAT on AP invoice is deductible
        assert!(!lines[0].is_reverse_charge);
    }

    #[test]
    fn test_reverse_charge_cross_border_eu() {
        // Vendor in DE, buyer in FR -> reverse charge (no VAT charged by vendor)
        let tax_codes = make_eu_tax_codes();
        let mut gen = TaxLineGenerator::new(42, tax_codes, TaxLineGeneratorConfig::default());

        let lines = gen.generate_for_document(
            TaxableDocumentType::VendorInvoice,
            "VI-002",
            "DE",  // vendor country
            "FR",  // buyer country (different EU country)
            Decimal::from(5000),
            NaiveDate::from_ymd_opt(2025, 3, 15).unwrap(),
            None,
        );

        assert_eq!(lines.len(), 1);
        assert!(lines[0].is_reverse_charge);
        assert!(lines[0].is_self_assessed); // buyer self-assesses
    }

    #[test]
    fn test_exempt_category() {
        let tax_codes = make_eu_tax_codes();
        let config = TaxLineGeneratorConfig {
            exempt_categories: vec!["financial_services".to_string()],
            ..Default::default()
        };
        let mut gen = TaxLineGenerator::new(42, tax_codes, config);

        let lines = gen.generate_for_document(
            TaxableDocumentType::CustomerInvoice,
            "CI-001",
            "DE",
            "DE",
            Decimal::from(10000),
            NaiveDate::from_ymd_opt(2025, 3, 15).unwrap(),
            Some("financial_services"),
        );

        assert!(lines.is_empty() || lines[0].is_exempt());
    }
}
```

**Step 2: Implement TaxLineGenerator**

```rust
pub struct TaxLineGenerator {
    rng: ChaCha8Rng,
    tax_codes: Vec<TaxCode>,
    config: TaxLineGeneratorConfig,
    eu_countries: HashSet<String>,  // for reverse charge determination
    counter: u64,
}

impl TaxLineGenerator {
    pub fn new(seed: u64, tax_codes: Vec<TaxCode>, config: TaxLineGeneratorConfig) -> Self;

    /// Generate tax lines for a single document.
    pub fn generate_for_document(
        &mut self,
        doc_type: TaxableDocumentType,
        doc_id: &str,
        seller_country: &str,
        buyer_country: &str,
        taxable_amount: Decimal,
        date: NaiveDate,
        product_category: Option<&str>,
    ) -> Vec<TaxLine>;

    /// Batch-decorate a slice of vendor invoices.
    pub fn decorate_vendor_invoices(&mut self, invoices: &[(String, String, Decimal, NaiveDate)])
        -> Vec<TaxLine>;

    /// Batch-decorate a slice of customer invoices.
    pub fn decorate_customer_invoices(&mut self, invoices: &[(String, String, Decimal, NaiveDate)])
        -> Vec<TaxLine>;
}
```

Logic:
1. Look up applicable tax codes by jurisdiction (seller country for output, buyer country for input)
2. Check if cross-border EU → apply reverse charge
3. Check if product category is exempt → skip or mark exempt
4. Compute tax amount = taxable_amount * rate
5. Set deductibility (input VAT on AP is deductible; output VAT on AR is collected)

**Step 3: Run tests**

Run: `cargo test -p datasynth-generators tax_line`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/datasynth-generators/src/tax/
git commit -m "feat(generators): add tax line decorator generator

Attaches tax lines to AP/AR documents with jurisdiction-aware VAT/GST
determination, EU reverse charge handling, and exempt category support."
```

---

### Task 5: Tax Return and Provision Generators

**Files:**
- Create: `crates/datasynth-generators/src/tax/tax_return_generator.rs`
- Create: `crates/datasynth-generators/src/tax/tax_provision_generator.rs`
- Create: `crates/datasynth-generators/src/tax/withholding_generator.rs`
- Modify: `crates/datasynth-generators/src/tax/mod.rs`

**Step 1: Write failing tests for each generator**

- `TaxReturnGenerator`: Takes `Vec<TaxLine>` + period range → produces `Vec<TaxReturn>` aggregated by jurisdiction
- `TaxProvisionGenerator`: Takes pre-tax income + statutory rate → produces `TaxProvision` with rate reconciliation
- `WithholdingGenerator`: Takes cross-border payments + vendor countries → produces `Vec<WithholdingTaxRecord>`

Each test should verify:
- Correct aggregation (return totals match sum of lines)
- Determinism (same seed → same output)
- Business rules (late filing detection, effective rate calculation, treaty rate application)

**Step 2: Implement all three generators**

Follow the same struct pattern:
```rust
pub struct TaxReturnGenerator { rng: ChaCha8Rng, config: TaxReturnConfig }
pub struct TaxProvisionGenerator { rng: ChaCha8Rng, config: TaxProvisionConfig }
pub struct WithholdingGenerator { rng: ChaCha8Rng, config: WithholdingConfig, treaty_network: HashMap<(String, String), Decimal> }
```

**Step 3: Update tax/mod.rs with new re-exports**

**Step 4: Run all tax tests**

Run: `cargo test -p datasynth-generators tax_return && cargo test -p datasynth-generators tax_provision && cargo test -p datasynth-generators withholding`
Expected: All PASS

**Step 5: Commit**

```bash
git add crates/datasynth-generators/src/tax/
git commit -m "feat(generators): add tax return, provision, and withholding generators

Tax return aggregation by jurisdiction/period, ASC 740 tax provision
with rate reconciliation, and cross-border withholding with treaty network."
```

---

### Task 6: Tax Anomaly Labels

**Files:**
- Create: `crates/datasynth-generators/src/tax/tax_anomaly.rs`
- Modify: `crates/datasynth-generators/src/tax/mod.rs`

**Step 1: Write failing test**

```rust
#[test]
fn test_tax_anomaly_injection() {
    let mut injector = TaxAnomalyInjector::new(42, 0.05); // 5% anomaly rate
    let mut lines = generate_normal_tax_lines(100);
    let labels = injector.inject(&mut lines);

    assert!(!labels.is_empty());
    assert!(labels.len() <= 10); // roughly 5% of 100, with some variance
    for label in &labels {
        assert!(matches!(label.anomaly_type,
            TaxAnomalyType::IncorrectTaxCode | TaxAnomalyType::MissingTaxLine |
            TaxAnomalyType::RateArbitrage | TaxAnomalyType::WithholdingUnderstatement
        ));
    }
}
```

**Step 2: Implement TaxAnomalyInjector**

Anomaly types: `IncorrectTaxCode`, `MissingTaxLine`, `RateArbitrage`, `LateFilingRisk`, `TransferPricingDeviation`, `WithholdingUnderstatement`

Each produces a labeled anomaly with type, severity, and description.

**Step 3: Run tests, commit**

---

### Task 7: Tax Export

**Files:**
- Create: `crates/datasynth-output/src/tax_export.rs`
- Modify: `crates/datasynth-output/src/lib.rs`

**Step 1: Write failing test**

```rust
#[test]
fn test_tax_export_creates_csv_files() {
    let dir = tempfile::tempdir().unwrap();
    let exporter = TaxExporter::new(dir.path());

    let codes = vec![/* test TaxCode */];
    let lines = vec![/* test TaxLine */];
    let returns = vec![/* test TaxReturn */];

    let summary = exporter.export_all(&codes, &[], &lines, &returns, &[], &[], &[]).unwrap();

    assert!(dir.path().join("tax_codes.csv").exists());
    assert!(dir.path().join("tax_lines.csv").exists());
    assert!(dir.path().join("tax_returns.csv").exists());
    assert_eq!(summary.tax_codes_count, codes.len());
}
```

**Step 2: Implement TaxExporter**

Follow the `ControlExporter` pattern. Export files:
- `tax_codes.csv`, `tax_jurisdictions.csv`, `tax_lines.csv`
- `tax_returns.csv`, `tax_provisions.csv`, `rate_reconciliation.csv`
- `uncertain_tax_positions.csv`, `withholding_records.csv`
- `tax_anomaly_labels.csv`

**Step 3: Register in lib.rs**

Add `pub mod tax_export;` and `pub use tax_export::*;` to `crates/datasynth-output/src/lib.rs`.

**Step 4: Run tests, commit**

```bash
git commit -m "feat(output): add tax data CSV exporter

Exports tax codes, jurisdictions, lines, returns, provisions,
uncertain positions, withholding records, and anomaly labels."
```

---

### Task 8: Tax Integration Tests

**Files:**
- Create: `crates/datasynth-generators/tests/tax_validation_tests.rs`

**Step 1: Write integration tests**

```rust
//! Integration tests for tax data generation pipeline.
//!
//! Tests the full flow: jurisdiction setup → tax code generation → tax line decoration
//! → tax return aggregation → provision calculation → anomaly injection.

#[test]
fn test_full_tax_pipeline() {
    // 1. Generate jurisdictions and tax codes
    let mut code_gen = TaxCodeGenerator::new(42, config);
    let (jurisdictions, codes) = code_gen.generate();

    // 2. Generate tax lines for mock invoices
    let mut line_gen = TaxLineGenerator::new(43, codes.clone(), TaxLineGeneratorConfig::default());
    let invoice_data = mock_vendor_invoices(50);
    let lines = line_gen.decorate_vendor_invoices(&invoice_data);

    // 3. Aggregate into returns
    let mut return_gen = TaxReturnGenerator::new(44, TaxReturnConfig::default());
    let returns = return_gen.generate(&lines, period_start, period_end);

    // 4. Verify coherence
    for ret in &returns {
        let jurisdiction_lines: Vec<_> = lines.iter()
            .filter(|l| l.jurisdiction_id == ret.jurisdiction_id)
            .collect();
        let expected_output: Decimal = jurisdiction_lines.iter()
            .filter(|l| l.document_type == TaxableDocumentType::CustomerInvoice)
            .map(|l| l.tax_amount)
            .sum();
        assert_eq!(ret.total_output_tax, expected_output,
            "Return output tax should match sum of output tax lines");
    }
}

#[test]
fn test_tax_determinism_full_pipeline() { /* same seed → identical results */ }
```

**Step 2: Run integration tests**

Run: `cargo test -p datasynth-generators --test tax_validation_tests`
Expected: PASS

**Step 3: Run full workspace check**

Run: `cargo clippy --workspace && cargo test --workspace`
Expected: All pass (except known protoc warning)

**Step 4: Commit**

```bash
git commit -m "test(tax): add integration tests for tax generation pipeline

Full pipeline test: jurisdictions → codes → lines → returns → provisions.
Verifies coherence, determinism, and anomaly injection."
```

---

## Phase 2: Treasury & Cash Management

Treasury runs second because it aggregates from AP/AR/payroll (existing) + tax (Phase 1).

### Task 9: Treasury Data Models

**Files:**
- Create: `crates/datasynth-core/src/models/treasury.rs`
- Modify: `crates/datasynth-core/src/models/mod.rs`

**Step 1: Write failing tests**

Test `CashPosition` (opening + inflows - outflows = closing), `CashForecast` (net position = sum of items), `DebtInstrument` (amortization sum = principal), `DebtCovenant` (compliance check), `HedgingInstrument` (status transitions), `HedgeRelationship` (effectiveness 80-125% corridor), `NettingRun` (gross - net balance), `BankGuarantee`, `CashPool`, `CashPoolSweep`.

**Step 2: Implement all treasury models**

Follow same derives and serde patterns as tax.rs. Key methods:
- `CashPosition::closing_balance()` — computed from opening + inflows - outflows
- `DebtCovenant::headroom()` — distance from threshold
- `HedgeRelationship::is_effective()` — ratio within 0.80..=1.25
- `NettingRun::savings()` — gross - net (bilateral reduction)

**Step 3: Register in mod.rs, run tests, commit**

```bash
git commit -m "feat(models): add treasury & cash management data models

Cash positions, forecasts, pooling, hedging instruments (ASC 815/IFRS 9),
debt instruments with covenants, bank guarantees, and IC netting runs."
```

---

### Task 10: Treasury Config Schema

**Files:**
- Modify: `crates/datasynth-config/src/schema.rs`

Add `TreasuryConfig` with sub-configs: `CashPositioningConfig`, `CashForecastingConfig`, `CashPoolingConfig`, `HedgingConfig`, `DebtConfig`, `NettingConfig`, `BankGuaranteeConfig`.

Add `pub treasury: TreasuryConfig` to `GeneratorConfig`.

Run: `cargo test -p datasynth-config`, commit.

---

### Task 11: Cash Position Generator (Aggregation Pattern)

**Files:**
- Create: `crates/datasynth-generators/src/treasury/mod.rs`
- Create: `crates/datasynth-generators/src/treasury/cash_position_generator.rs`
- Modify: `crates/datasynth-generators/src/lib.rs`

**Step 1: Write failing test**

```rust
#[test]
fn test_cash_positions_from_payment_flows() {
    let mut gen = CashPositionGenerator::new(42, CashPositionConfig::default());
    let flows = vec![
        CashFlow { date: d("2025-01-15"), account_id: "BA-001".into(), amount: dec!(5000), direction: Inflow },
        CashFlow { date: d("2025-01-15"), account_id: "BA-001".into(), amount: dec!(2000), direction: Outflow },
        CashFlow { date: d("2025-01-16"), account_id: "BA-001".into(), amount: dec!(1000), direction: Outflow },
    ];
    let positions = gen.generate("C001", &flows, d("2025-01-15"), d("2025-01-16"), dec!(10000));

    assert_eq!(positions.len(), 2); // one per day
    assert_eq!(positions[0].opening_balance, dec!(10000));
    assert_eq!(positions[0].inflows, dec!(5000));
    assert_eq!(positions[0].outflows, dec!(2000));
    assert_eq!(positions[0].closing_balance, dec!(13000));
    assert_eq!(positions[1].opening_balance, dec!(13000));
}
```

**Step 2: Implement**

The generator takes a `Vec<CashFlow>` (abstracted from AP payments, AR receipts, payroll, tax payments) and aggregates into daily `CashPosition` records per bank account.

**Step 3: Run tests, commit**

---

### Task 12: Cash Forecast, Hedging, Debt, Pool Generators

**Files:**
- Create: `crates/datasynth-generators/src/treasury/cash_forecast_generator.rs`
- Create: `crates/datasynth-generators/src/treasury/hedging_generator.rs`
- Create: `crates/datasynth-generators/src/treasury/debt_generator.rs`
- Create: `crates/datasynth-generators/src/treasury/cash_pool_generator.rs`
- Modify: `crates/datasynth-generators/src/treasury/mod.rs`

Follow the same TDD pattern for each:

1. **CashForecastGenerator**: Takes AR aging + AP aging + payroll schedule + tax deadlines → produces forecast items with probability weighting. Test that overdue AR gets lower probability.

2. **HedgingGenerator**: Takes FX exposures (from multi-currency AP/AR) → creates FX forward instruments for configurable hedge ratio. Designates hedge relationships. Tests effectiveness ratio computation.

3. **DebtGenerator**: Creates term loans, revolving credit with amortization schedules. Tests covenant monitoring against financial ratios. Generates `AmortizationPayment` vectors that sum to principal.

4. **CashPoolGenerator**: Groups bank accounts into pools by entity. Generates daily sweeps. Tests zero-balancing logic (all participant balances → 0, header gets net).

**Step by step**: Write tests for each, implement, verify, commit after each generator.

```bash
git commit -m "feat(generators): add cash forecast generator"
git commit -m "feat(generators): add hedging instrument generator with ASC 815 effectiveness"
git commit -m "feat(generators): add debt instrument and covenant generator"
git commit -m "feat(generators): add cash pool sweep generator"
```

---

### Task 13: Treasury Anomaly Labels

**Files:**
- Create: `crates/datasynth-generators/src/treasury/treasury_anomaly.rs`

Anomaly types: `CashForecastMiss`, `CovenantBreachRisk`, `HedgeIneffectiveness`, `UnusualCashMovement`, `LiquidityCrisis`, `CounterpartyConcentration`

Follow same pattern as Task 6.

---

### Task 14: Treasury Export

**Files:**
- Create: `crates/datasynth-output/src/treasury_export.rs`
- Modify: `crates/datasynth-output/src/lib.rs`

Export: `cash_positions.csv`, `cash_forecasts.csv`, `cash_forecast_items.csv`, `cash_pool_sweeps.csv`, `hedging_instruments.csv`, `hedge_relationships.csv`, `debt_instruments.csv`, `debt_covenants.csv`, `amortization_schedules.csv`, `bank_guarantees.csv`, `netting_runs.csv`, `netting_positions.csv`, `treasury_anomaly_labels.csv`

---

### Task 15: Treasury Integration Tests

**Files:**
- Create: `crates/datasynth-generators/tests/treasury_validation_tests.rs`

Test full pipeline: mock AP/AR payments → cash positions → forecast → hedging → covenant check.
Verify: position balances chain correctly day-to-day, forecast probabilities decay with aging, hedge effectiveness within corridor.

```bash
git commit -m "test(treasury): add integration tests for treasury pipeline"
```

---

## Phase 3: Project Accounting

Project accounting extends the existing `project.rs` model and reuses existing time entry, expense, and PO generators.

### Task 16: Extended Project Accounting Models

**Files:**
- Create: `crates/datasynth-core/src/models/project_accounting.rs`
- Modify: `crates/datasynth-core/src/models/mod.rs`

**Important:** The existing `crates/datasynth-core/src/models/project.rs` already has `Project`, `WbsElement`, `ProjectPool`, `ProjectType`, and `ProjectStatus`. The new file `project_accounting.rs` adds the accounting-specific types that build on top:

- `ProjectCostLine` (links time/expense/PO to WBS)
- `CostCategory` enum (Labor, Material, Subcontractor, Overhead, Equipment, Travel)
- `CostSourceType` enum (TimeEntry, ExpenseReport, PurchaseOrder, VendorInvoice, JournalEntry)
- `ProjectRevenue` (PoC recognition)
- `RevenueMethod`, `CompletionMeasure` enums
- `ProjectMilestone`, `MilestoneStatus` enum
- `ChangeOrder`, `ChangeOrderStatus`, `ChangeReason` enums
- `Retainage`, `RetainageStatus` enum
- `EarnedValueMetric` (BCWS, BCWP, ACWP, SPI, CPI, EAC, ETC, TCPI)

Tests verify: EVM formulas (SV = EV - PV, CV = EV - AC, SPI = EV/PV, CPI = EV/AC), PoC completion percentage calculation, retainage math.

```bash
git commit -m "feat(models): add project accounting models (cost lines, revenue, EVM, milestones)"
```

---

### Task 17: Project Accounting Config Schema

**Files:**
- Modify: `crates/datasynth-config/src/schema.rs`

Add `ProjectAccountingConfig` with: `project_count`, `project_types` (distribution), `wbs` config, `cost_allocation` rates (what % of time entries/expenses/POs get tagged), `revenue_recognition`, `milestones`, `change_orders`, `retainage`, `earned_value`, `anomaly_rate`.

Add `pub project_accounting: ProjectAccountingConfig` to `GeneratorConfig`.

---

### Task 18: Project Cost Generator (Linking Pattern)

**Files:**
- Create: `crates/datasynth-generators/src/project_accounting/mod.rs`
- Create: `crates/datasynth-generators/src/project_accounting/project_generator.rs`
- Create: `crates/datasynth-generators/src/project_accounting/project_cost_generator.rs`
- Modify: `crates/datasynth-generators/src/lib.rs`

**Step 1: Write failing test**

```rust
#[test]
fn test_project_cost_linking() {
    let projects = generate_test_projects(5);
    let time_entries = generate_test_time_entries(100);
    let config = ProjectCostConfig { time_entry_project_rate: 0.60, ..Default::default() };

    let mut gen = ProjectCostGenerator::new(42, config);
    let cost_lines = gen.link_time_entries(&projects, &time_entries);

    // ~60% of time entries should be linked
    let linked_count = cost_lines.len();
    assert!(linked_count >= 40 && linked_count <= 80,
        "Expected ~60 linked, got {}", linked_count);

    // All linked entries should reference valid projects and WBS elements
    for line in &cost_lines {
        assert!(projects.iter().any(|p| p.project_id == line.project_id));
        assert_eq!(line.cost_category, CostCategory::Labor);
        assert_eq!(line.source_type, CostSourceType::TimeEntry);
    }
}
```

**Step 2: Implement**

- `ProjectGenerator`: Creates projects with WBS hierarchies based on config. Extends the existing `ProjectPool::standard()` with more detailed project types including customer-facing projects with contract values.
- `ProjectCostGenerator`: The linking generator. For each time entry/expense/PO, probabilistically assigns to a project+WBS based on `time_entry_project_rate` etc. Creates `ProjectCostLine` records.

**Step 3: Run tests, commit**

---

### Task 19: Project Revenue and Earned Value Generators

**Files:**
- Create: `crates/datasynth-generators/src/project_accounting/revenue_generator.rs`
- Create: `crates/datasynth-generators/src/project_accounting/earned_value_generator.rs`
- Create: `crates/datasynth-generators/src/project_accounting/change_order_generator.rs`

1. **RevenueGenerator**: Takes project cost lines + project contracts → computes PoC (cost incurred / estimated total cost) → generates `ProjectRevenue` records per period. Tests: cumulative revenue increases monotonically, unbilled revenue = recognized - billed.

2. **EarnedValueGenerator**: Takes WBS budgets + actual costs + schedule → computes EVM metrics. Tests: SPI = EV/PV, CPI = EV/AC, EAC formulas.

3. **ChangeOrderGenerator**: Probabilistically injects change orders. Tests: cost/revenue/schedule impacts are coherent.

```bash
git commit -m "feat(generators): add project revenue (PoC) and earned value generators"
git commit -m "feat(generators): add change order and retainage generators"
```

---

### Task 20: Project Accounting Export and Integration Tests

**Files:**
- Create: `crates/datasynth-output/src/project_accounting_export.rs`
- Create: `crates/datasynth-generators/tests/project_accounting_validation_tests.rs`

Export: `projects.csv`, `wbs_elements.csv`, `project_cost_lines.csv`, `project_revenue.csv`, `project_milestones.csv`, `change_orders.csv`, `retainage.csv`, `earned_value_metrics.csv`, `project_anomaly_labels.csv`

Integration test: Full pipeline from project creation → cost linking → revenue recognition → EVM. Verify: total cost lines by source match the linking rates, revenue increases with completion, EVM formulas hold.

```bash
git commit -m "feat(output): add project accounting CSV exporter"
git commit -m "test(project): add integration tests for project accounting pipeline"
```

---

## Phase 4: ESG / Sustainability Reporting

ESG is last because it derives data from manufacturing (Scope 1), HR (diversity), vendor network (Scope 3), and can also integrate tax (carbon tax) and treasury (green bonds).

### Task 21: ESG Data Models

**Files:**
- Create: `crates/datasynth-core/src/models/esg.rs`
- Modify: `crates/datasynth-core/src/models/mod.rs`

Models split into four sections:

**Environmental:** `EmissionRecord`, `EmissionScope`, `Scope3Category` (15 GHG Protocol categories), `EmissionActivity`, `EstimationMethod`, `EnergyConsumption`, `EnergyType`, `WaterUsage`, `WaterSource`, `WasteRecord`, `WasteType`, `DisposalMethod`

**Social:** `WorkforceDiversityMetric`, `DiversityDimension`, `OrganizationLevel`, `PayEquityMetric`, `SafetyIncident`, `IncidentType`, `SafetyMetric`

**Governance:** `GovernanceMetric`

**Supply Chain & Reporting:** `SupplierEsgAssessment`, `EsgRiskFlag`, `AssessmentMethod`, `EsgDisclosure`, `EsgFramework`, `AssuranceLevel`, `MaterialityAssessment`, `ClimateScenario`, `ScenarioType`, `TimeHorizon`

Tests verify: emission factor calculation (activity_data * emission_factor = co2e_tonnes), TRIR formula (recordable_incidents * 200000 / total_hours_worked), materiality double-threshold logic, serde roundtrip.

```bash
git commit -m "feat(models): add ESG sustainability reporting data models

Environmental (GHG Scope 1/2/3, energy, water, waste), social (diversity,
pay equity, safety), governance, supply chain ESG, and disclosure models."
```

---

### Task 22: ESG Config Schema

**Files:**
- Modify: `crates/datasynth-config/src/schema.rs`

Add `EsgConfig` with nested: `EnvironmentalConfig` (emissions scope1/2/3, energy, water, waste), `SocialConfig` (diversity, pay_equity, safety), `GovernanceConfig`, `SupplyChainEsgConfig`, `EsgReportingConfig`, `ClimateScenarioConfig`.

Add `pub esg: EsgConfig` to `GeneratorConfig`.

---

### Task 23: Emission Generator (Derivation Pattern)

**Files:**
- Create: `crates/datasynth-generators/src/esg/mod.rs`
- Create: `crates/datasynth-generators/src/esg/emission_generator.rs`
- Modify: `crates/datasynth-generators/src/lib.rs`

**Step 1: Write failing test**

```rust
#[test]
fn test_scope1_emissions_from_energy() {
    let energy_data = vec![
        EnergyInput { facility_id: "F-001".into(), energy_type: NaturalGas, consumption_kwh: dec!(100000), period: d("2025-01") },
    ];
    let mut gen = EmissionGenerator::new(42, EmissionConfig::default());
    let records = gen.generate_scope1("C001", &energy_data);

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].scope, EmissionScope::Scope1);
    assert!(records[0].co2e_tonnes > Decimal::ZERO);
    assert_eq!(records[0].estimation_method, EstimationMethod::ActivityBased);
}

#[test]
fn test_scope3_from_vendor_spend() {
    let vendor_spend = vec![
        VendorSpendInput { vendor_id: "V-001".into(), category: "office_supplies".into(), spend: dec!(50000), country: "US".into() },
        VendorSpendInput { vendor_id: "V-002".into(), category: "manufacturing".into(), spend: dec!(200000), country: "CN".into() },
    ];
    let mut gen = EmissionGenerator::new(42, EmissionConfig::default());
    let records = gen.generate_scope3_purchased_goods("C001", &vendor_spend, d("2025-01"), d("2025-12"));

    assert_eq!(records.len(), 2);
    assert!(records.iter().all(|r| r.scope == EmissionScope::Scope3));
    assert!(records.iter().all(|r| r.scope3_category == Some(Scope3Category::PurchasedGoods)));
    // Higher spend + manufacturing = higher emissions
    assert!(records[1].co2e_tonnes > records[0].co2e_tonnes);
}
```

**Step 2: Implement**

EmissionGenerator takes operational data and applies emission factors:
- Scope 1: Fuel combustion (natural gas, diesel from energy data), process emissions (from manufacturing)
- Scope 2: Purchased electricity (from energy consumption records)
- Scope 3: Spend-based allocation (vendor spend * industry emission factor), business travel (from expense reports), employee commuting (headcount * avg commute)

Built-in emission factor tables by source (EPA, DEFRA) with region-specific factors.

**Step 3: Run tests, commit**

---

### Task 24: Energy, Workforce, Supplier ESG, Disclosure Generators

**Files:**
- Create: `crates/datasynth-generators/src/esg/energy_generator.rs`
- Create: `crates/datasynth-generators/src/esg/workforce_generator.rs`
- Create: `crates/datasynth-generators/src/esg/supplier_esg_generator.rs`
- Create: `crates/datasynth-generators/src/esg/disclosure_generator.rs`

1. **EnergyGenerator**: Creates facility-level energy records. For manufacturing: correlates with production volume. Renewable percentage configurable. Tests: total = renewable + non-renewable.

2. **WorkforceGenerator** (derivation): Takes employee master data → computes diversity metrics by dimension and level, pay equity ratios, safety metrics (TRIR, LTIR, DART). Tests: percentages sum to 100%, pay equity ratio calculations.

3. **SupplierEsgGenerator** (derivation): Takes existing vendor list + vendor quality scores → assigns ESG scores correlated with quality. High-risk flags based on country + industry. Tests: coverage matches config, scores in 0-100 range.

4. **DisclosureGenerator**: Maps calculated metrics to framework-specific standard IDs (GRI 305-1, ESRS E1-6, etc.). Checks materiality to determine required disclosures. Tests: all material topics have disclosures, framework IDs are valid.

```bash
git commit -m "feat(generators): add energy consumption generator"
git commit -m "feat(generators): add workforce diversity and safety metrics generator"
git commit -m "feat(generators): add supplier ESG assessment generator"
git commit -m "feat(generators): add ESG disclosure and materiality generator"
```

---

### Task 25: ESG Anomaly Labels

**Files:**
- Create: `crates/datasynth-generators/src/esg/esg_anomaly.rs`

Anomaly types: `GreenwashingIndicator`, `DiversityStagnation`, `SupplyChainRisk`, `DataQualityGap`, `MissingDisclosure`, `ScenarioInconsistency`

---

### Task 26: ESG Export and Integration Tests

**Files:**
- Create: `crates/datasynth-output/src/esg_export.rs`
- Create: `crates/datasynth-generators/tests/esg_validation_tests.rs`

Export 14 files: `emission_records.csv`, `energy_consumption.csv`, `water_usage.csv`, `waste_records.csv`, `workforce_diversity.csv`, `pay_equity_metrics.csv`, `safety_incidents.csv`, `safety_metrics.csv`, `governance_metrics.csv`, `supplier_esg_assessments.csv`, `esg_disclosures.csv`, `materiality_assessments.csv`, `climate_scenarios.csv`, `esg_anomaly_labels.csv`

Integration test: Full pipeline with mock operational data → emissions + energy + workforce + disclosures. Verify: Scope 1+2+3 totals are coherent, TRIR formula correct, all material topics have disclosures.

```bash
git commit -m "feat(output): add ESG sustainability CSV exporter"
git commit -m "test(esg): add integration tests for ESG generation pipeline"
```

---

## Phase 5: Cross-Domain Integration

### Task 27: OCPM Process Mining Events

**Files:**
- Modify: `crates/datasynth-ocpm/src/` (add process variants for new domains)

Add OCEL 2.0 event types for each domain:
- Tax: `TaxDetermination → TaxLineCreated → ReturnFiled → ReturnAssessed → TaxPaid`
- Treasury: `CashPositionCalculated → ForecastGenerated → HedgeDesignated → CovenantMeasured`
- Project: `ProjectCreated → CostPosted → MilestoneAchieved → RevenueRecognized → ChangeOrderProcessed`
- ESG: `DataCollected → EmissionCalculated → DisclosurePrepared → AssuranceCompleted`

```bash
git commit -m "feat(ocpm): add process mining events for tax, treasury, project, and ESG"
```

---

### Task 28: Preset Updates

**Files:**
- Modify: preset configuration files to include new domain defaults per industry

| Preset | Tax | Treasury | Project | ESG |
|---|---|---|---|---|
| manufacturing | VAT + payroll | Cash pooling, FX hedge | Internal capex projects | Full (Scope 1/2/3, safety) |
| retail | Sales tax by state | Cash positioning | Minimal | Supply chain, waste |
| financial_services | Withholding + VAT exempt | Full treasury | Minimal | Governance-heavy |
| healthcare | Exempt categories | Cash positioning | Research projects | Safety, diversity |
| technology | R&D credits | Cash forecasting | T&M + internal R&D | Scope 2/3, diversity |

```bash
git commit -m "feat(config): update industry presets with tax, treasury, project, and ESG defaults"
```

---

### Task 29: Final Workspace Validation

**Step 1: Run full test suite**

Run: `cargo test --workspace`
Expected: All tests pass (2000+ existing + new tests)

**Step 2: Run clippy**

Run: `cargo clippy --workspace`
Expected: Clean (only protoc warning)

**Step 3: Run formatter**

Run: `cargo fmt --all`

**Step 4: Verify demo generation**

Run: `cargo run -p datasynth-cli -- generate --demo --output /tmp/datasynth-demo-test`
Verify new CSV files appear in output directory.

**Step 5: Final commit**

```bash
git commit -m "chore: cargo fmt and final validation"
```

---

## Summary

| Phase | Tasks | New Files | New Tests |
|---|---|---|---|
| Tax Accounting | 1-8 | ~10 | ~30 |
| Treasury | 9-15 | ~10 | ~25 |
| Project Accounting | 16-20 | ~8 | ~20 |
| ESG Reporting | 21-26 | ~10 | ~25 |
| Cross-Domain | 27-29 | ~3 | ~10 |
| **Total** | **29 tasks** | **~41 files** | **~110 tests** |

Implementation order respects dependencies: Tax → Treasury → Project → ESG → Cross-domain.
