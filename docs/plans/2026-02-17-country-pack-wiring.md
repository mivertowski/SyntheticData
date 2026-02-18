# Country Pack Full Integration Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Wire the country pack system into all 17 integration points across the codebase so that every generator produces country-localized data when a country pack is available.

**Architecture:** The `CountryPackRegistry` already lives on `EnhancedOrchestrator` (line 739). The established pattern is: orchestrator resolves `&CountryPack` for the primary company's country, then passes it to each generator via builder methods (`with_country_pack_*`) or as a parameter to `generate_*_from_pack()`. Several generators already have these methods but they're not called from the orchestrator. This plan wires them all.

**Tech Stack:** Rust, serde, datasynth-core (CountryPack, CountryPackRegistry), datasynth-generators, datasynth-runtime, datasynth-banking, datasynth-graph, datasynth-ocpm, datasynth-fingerprint

---

## Batch 1: Wire Existing Country Pack Methods in Orchestrator (Gaps 2, 5, 7)

These generators already have `_from_pack` / `_from_country_pack` methods that are never called from the orchestrator. The fix is purely orchestrator-side wiring.

### Task 1: Wire JE generator temporal patterns from country pack (Gap 2)

**Context:** `JournalEntryGenerator::with_country_pack_temporal()` exists at `je_generator.rs:329` but the orchestrator only calls `with_country_pack_names()` at line 2959. The temporal method creates a `HolidayCalendar::from_country_pack()` and configures `BusinessDayCalculator` with pack holidays.

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs` (~line 2959)

**Step 1: Add temporal wiring after names wiring**

Find the JE generator setup around lines 2951-2975. After `.with_country_pack_names(je_pack)`, chain `.with_country_pack_temporal(...)`:

```rust
// Around line 2959, change from:
.with_country_pack_names(je_pack)
// To:
.with_country_pack_names(je_pack)
.with_country_pack_temporal(
    self.config.temporal_patterns.clone(),
    self.seed + 200,
    je_pack,
)
```

**Step 2: Verify compilation**

Run: `cargo check -p datasynth-runtime`
Expected: compiles clean

**Step 3: Run existing tests**

Run: `cargo test -p datasynth-runtime`
Expected: all existing tests pass

**Step 4: Commit**

```bash
git add crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "feat(runtime): wire JE temporal patterns from country pack"
```

---

### Task 2: Wire tax code generator from country pack (Gap 5)

**Context:** `TaxCodeGenerator::generate_from_country_pack()` exists at `tax_code_generator.rs:540` but the orchestrator calls `generate()` (the non-pack version). The pack method reads `pack.tax.vat.standard_rate`, `reduced_rates`, `subnational`, etc.

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs` (find `TaxCodeGenerator` usage)

**Step 1: Find tax code generation in orchestrator**

Search for `TaxCodeGenerator` in enhanced_orchestrator.rs. It will be in a tax-generation phase. Change the `generate()` call to `generate_from_country_pack()`, passing the primary country's pack.

```rust
// Replace the existing tax code generation with:
let primary_country = self.config.companies.first()
    .map(|c| c.country.as_str())
    .unwrap_or("US");
let tax_pack = self.country_pack_registry.get_by_str(primary_country);
let company_code = self.config.companies.first()
    .map(|c| c.code.as_str())
    .unwrap_or("1000");
let fiscal_year = /* extract from config start_date */;
let (jurisdictions, tax_codes) = tax_gen.generate_from_country_pack(
    tax_pack,
    company_code,
    fiscal_year,
);
```

If the pack has no meaningful tax data, `generate_from_country_pack` returns empty vectors, so add a fallback to `generate()` when the pack result is empty.

**Step 2: Verify compilation**

Run: `cargo check -p datasynth-runtime`

**Step 3: Run tests**

Run: `cargo test -p datasynth-runtime && cargo test -p datasynth-generators -- tax`

**Step 4: Commit**

```bash
git add crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "feat(runtime): wire tax code generator from country pack"
```

---

### Task 3: Wire emission generator from country pack (Gap 7)

**Context:** `EmissionGenerator::spend_emission_factor_from_pack()` exists at `emission_generator.rs:98` but `spend_emission_factor()` (hardcoded multipliers) is used instead. The pack method reads `pack.business_rules.emission_country_multiplier`.

**Files:**
- Modify: `crates/datasynth-generators/src/esg/emission_generator.rs` (change internal calls)
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs` (pass pack to ESG phase)

**Step 1: Update ESG phase to pass country pack**

In the orchestrator's ESG generation phase, resolve the country pack and pass it to the emission generator. If the emission generator's `generate_scope3_purchased_goods()` currently calls `spend_emission_factor()` internally, add an optional `&CountryPack` parameter or set the pack on the generator struct.

The simplest approach: add a `with_country_pack(&mut self, pack: &CountryPack)` method to `EmissionGenerator` that stores the country multiplier, then use it in `spend_emission_factor()` instead of the hardcoded match.

**Step 2: Verify**

Run: `cargo test -p datasynth-generators -- esg`

**Step 3: Commit**

```bash
git add crates/datasynth-generators/src/esg/emission_generator.rs crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "feat(esg): wire emission country multiplier from country pack"
```

---

## Batch 2: Wire Banking Customer Generator (Gap 3)

### Task 4: Wire banking customer generator country pack methods (Gap 3)

**Context:** `CustomerGenerator` in `datasynth-banking` already has `generate_phone_from_pack()` (line 774), `generate_address_from_pack()` (line 835), and `generate_national_id_from_pack()` (line 919). But `generate_retail_customer()` calls the non-pack versions (`generate_phone(&country)`, `generate_address(&country)`, `generate_national_id(&country)`).

**Files:**
- Modify: `crates/datasynth-banking/src/generators/customer_generator.rs`
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs` (pass registry to banking)

**Step 1: Add CountryPackRegistry to CustomerGenerator**

Add an `Option<&CountryPackRegistry>` or store a reference to the registry on the generator. Then in `generate_retail_customer()`, `generate_business_customer()`, and `generate_trust_customer()`, after `select_country()` resolves the country code, look up the pack and use `_from_pack` methods when available.

Since we can't store references easily (lifetimes), the simplest approach is to add a `pub fn set_country_pack_registry(...)` or pass it via a new constructor. Alternative: add a `generate_all_with_packs(&mut self, registry: &CountryPackRegistry)` method.

**Step 2: Wire in orchestrator**

In the banking phase (`phase_banking_data`), pass `&self.country_pack_registry` to the banking customer generator.

**Step 3: Write test**

Add a test in `crates/datasynth-banking/tests/` that creates a `CustomerGenerator`, generates a customer with a DE country pack, and verifies the phone/address formats match German patterns.

**Step 4: Verify**

Run: `cargo test -p datasynth-banking`

**Step 5: Commit**

```bash
git add crates/datasynth-banking/src/generators/customer_generator.rs crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "feat(banking): wire customer generator country pack methods"
```

---

## Batch 3: LLM Enrichment Country Awareness (Gap 1)

### Task 5: Add country context to VendorLlmEnricher (Gap 1a)

**Context:** `VendorLlmEnricher::enrich_vendor_name()` takes `(industry, spend_category, country)` but the prompt doesn't include locale-specific guidance (e.g., "use German company suffix like GmbH"). The country pack has `vendor_templates.name_patterns` and `legal_entities.entity_types` that should inform the LLM.

**Files:**
- Modify: `crates/datasynth-generators/src/llm_enrichment/vendor_enricher.rs`

**Step 1: Add country pack context to prompt**

Add a new method `enrich_vendor_name_with_pack()` that takes an additional `&CountryPack` parameter. Build prompt context from:
- `pack.legal_entities.entity_types` → "Common suffixes: GmbH, AG, KG"
- `pack.vendor_templates.name_patterns` → "Use patterns like: {industry} {suffix}"
- `pack.locale.language_code` → "Generate name in {language}"

```rust
pub fn enrich_vendor_name_with_pack(
    &self,
    industry: &str,
    spend_category: &str,
    country: &str,
    pack: &CountryPack,
) -> Result<String, SynthError> {
    let entity_suffixes: Vec<&str> = pack.legal_entities.entity_types
        .iter()
        .map(|e| e.code.as_str())
        .filter(|s| !s.is_empty())
        .collect();
    let suffix_hint = if entity_suffixes.is_empty() {
        String::new()
    } else {
        format!(" Use one of these legal entity suffixes: {}.", entity_suffixes.join(", "))
    };
    let lang_hint = if pack.locale.language_code.is_empty() {
        String::new()
    } else {
        format!(" The company name should sound natural in {}.", pack.locale.language_name)
    };

    let prompt = format!(
        "Generate a single realistic vendor/supplier company name for a {} company \
         in {} that provides {}.{}{} Return ONLY the company name, nothing else.",
        industry, country, spend_category, suffix_hint, lang_hint
    );
    // ... rest is same as enrich_vendor_name
}
```

Also update `fallback_vendor_name` to use entity suffixes from pack when available.

**Step 2: Update existing tests**

Existing tests should still pass. Add a new test for `enrich_vendor_name_with_pack()` using a mock provider.

**Step 3: Commit**

```bash
git add crates/datasynth-generators/src/llm_enrichment/vendor_enricher.rs
git commit -m "feat(llm): add country pack context to vendor name enrichment"
```

---

### Task 6: Add country context to TransactionLlmEnricher (Gap 1b)

**Files:**
- Modify: `crates/datasynth-generators/src/llm_enrichment/transaction_enricher.rs`

**Step 1: Add pack-aware methods**

Add `enrich_description_with_pack()` and `enrich_memo_with_pack()` that include:
- `pack.locale.language_code` → localize the language of descriptions
- `pack.document_texts.journal_entry.posting_texts` → provide example posting texts
- `pack.locale.default_currency` → mention currency context

**Step 2: Update fallback methods**

Update `fallback_description()` and `fallback_memo()` to optionally pull from `pack.document_texts` templates when available.

**Step 3: Tests and commit**

Run: `cargo test -p datasynth-generators -- llm`

```bash
git add crates/datasynth-generators/src/llm_enrichment/transaction_enricher.rs
git commit -m "feat(llm): add country pack context to transaction description enrichment"
```

---

### Task 7: Add country context to AnomalyLlmExplainer (Gap 1c)

**Files:**
- Modify: `crates/datasynth-generators/src/llm_enrichment/anomaly_explainer.rs`

**Step 1: Add pack-aware explain method**

Add `explain_with_pack()` that includes:
- `pack.locale.language_name` → "Explain in {language}"
- `pack.accounting.framework` → mention whether this is IFRS or US GAAP context
- `pack.locale.default_currency` → for amount references

**Step 2: Tests and commit**

```bash
git add crates/datasynth-generators/src/llm_enrichment/anomaly_explainer.rs
git commit -m "feat(llm): add country pack context to anomaly explanations"
```

---

### Task 8: Wire LLM enrichers to use country packs from orchestrator

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs` (LLM enrichment phase)

**Step 1: In phase_llm_enrichment, pass country pack**

Find the LLM enrichment phase. When calling vendor enrichment, pass the country pack. When calling transaction enrichment, pass the pack. Use the `_with_pack` methods created in Tasks 5-7.

**Step 2: Verify**

Run: `cargo test -p datasynth-runtime`

**Step 3: Commit**

```bash
git add crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "feat(runtime): wire LLM enrichers with country pack context"
```

---

## Batch 4: Master Data Generators (Gap 4)

### Task 9: Add country pack address generation to vendor generator

**Context:** `VendorGenerator` uses `AddressGenerator::for_region(config.primary_region)` with the legacy `AddressRegion` enum. The country pack has rich address data in `pack.address.components` (street_names, city_names, state_codes, postal_code format).

**Files:**
- Modify: `crates/datasynth-generators/src/master_data/vendor_generator.rs`

**Step 1: Add optional CountryPack field**

Add `country_pack: Option<CountryPack>` to `VendorGeneratorConfig` (cloned, since we can't store references). When set, use `pack.address` data instead of `AddressRegion`.

**Step 2: Update generate_vendor() to use pack**

When `country_pack` is `Some(pack)`, generate addresses using pack's street_names, city_names, state_codes, and postal_code format (reuse the expansion logic from banking's `expand_postal_format`).

For vendor names: use `pack.vendor_templates.name_patterns` and `pack.legal_entities.entity_types` to generate more realistic names.

**Step 3: Wire in orchestrator**

In `phase_master_data`, resolve the country pack for the primary company and set it on the vendor generator config.

**Step 4: Tests**

Run: `cargo test -p datasynth-generators -- vendor`

**Step 5: Commit**

```bash
git add crates/datasynth-generators/src/master_data/vendor_generator.rs crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "feat(generators): wire vendor generator with country pack addresses"
```

---

### Task 10: Add country pack address generation to customer generator

**Files:**
- Modify: `crates/datasynth-generators/src/master_data/customer_generator.rs`

**Step 1: Mirror Task 9 for CustomerGeneratorConfig**

Add `country_pack: Option<CountryPack>` to `CustomerGeneratorConfig`. When set, use pack address data and customer_templates for name generation.

**Step 2: Wire in orchestrator and test**

**Step 3: Commit**

```bash
git add crates/datasynth-generators/src/master_data/customer_generator.rs crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "feat(generators): wire customer generator with country pack addresses"
```

---

## Batch 5: Document Text Localization (Gaps 9, 10)

### Task 11: Use country pack document_texts in P2P/O2C generators (Gap 9)

**Context:** The country pack has `document_texts.purchase_order`, `document_texts.invoice`, and `document_texts.journal_entry`, each with `header_templates`, `line_descriptions`, and `posting_texts`. Currently, P2P and O2C generators don't use these.

**Files:**
- Modify: `crates/datasynth-generators/src/document_flow/p2p_generator.rs`
- Modify: `crates/datasynth-generators/src/document_flow/o2c_generator.rs`

**Step 1: Add optional document texts to P2P config**

Add `document_texts: Option<DocumentTextsConfig>` to the P2P generator's config. When generating PO header text or line descriptions, pull from `document_texts.purchase_order.header_templates` or `line_descriptions` instead of using material descriptions alone.

**Step 2: Same for O2C**

**Step 3: Wire in orchestrator**

In document flow phases, pass `pack.document_texts.clone()` to the generator configs.

**Step 4: Test and commit**

```bash
git add crates/datasynth-generators/src/document_flow/p2p_generator.rs crates/datasynth-generators/src/document_flow/o2c_generator.rs crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "feat(generators): use country pack document texts in P2P/O2C"
```

---

### Task 12: Use country pack legal_entities for company name suffixes (Gap 10)

**Context:** Company and vendor names don't use country-specific legal entity suffixes (GmbH, Ltd, Inc, S.A.). The country pack has `legal_entities.entity_types` with code, name, and weight.

**Files:**
- Modify: `crates/datasynth-generators/src/master_data/vendor_generator.rs` (if not already done in Task 9)
- Modify: `crates/datasynth-config/src/presets.rs`

**Step 1: Use entity_types weights for suffix selection**

In vendor name generation, when a country pack is available, append a legal entity suffix selected by weight from `pack.legal_entities.entity_types`.

**Step 2: Test and commit**

```bash
git add crates/datasynth-generators/src/master_data/vendor_generator.rs
git commit -m "feat(generators): use country pack legal entity suffixes in vendor names"
```

---

## Batch 6: Standards & Payroll Enhancements (Gaps 6, 11)

### Task 13: Use country pack accounting framework for standards generation (Gap 6)

**Context:** The standards generator reads `config.accounting_standards.framework` (UsGaap/Ifrs/Dual) but doesn't consult `pack.accounting.framework`. For multi-country generation, each company should use its country's framework.

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs` (standards phase)

**Step 1: Resolve framework from country pack**

In `phase_accounting_standards`, if the config's `framework` is set, use it. Otherwise, read from the country pack's `accounting.framework` field. Map "us_gaap" → UsGaap, "ifrs" → Ifrs, etc.

**Step 2: Test and commit**

```bash
git add crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "feat(runtime): resolve accounting framework from country pack"
```

---

### Task 14: Enhance payroll with country pack deduction labels (Gap 11)

**Context:** Payroll generation already uses `generate_with_country_pack()` (wired at orchestrator line 2306). However, the payroll line items still use US-centric labels ("Federal Income Tax", "FICA"). The pack has `payroll.statutory_deductions[].name` and `name_en` with localized labels.

**Files:**
- Modify: `crates/datasynth-generators/src/hr/payroll_generator.rs`

**Step 1: Use pack deduction names in line items**

In `generate_with_country_pack()` → `rates_from_country_pack()`, when building `PayrollLineItem` entries, use the deduction's `name` (localized) or `name_en` instead of hardcoded English labels.

**Step 2: Test and commit**

```bash
git add crates/datasynth-generators/src/hr/payroll_generator.rs
git commit -m "feat(hr): use country pack deduction labels in payroll line items"
```

---

## Batch 7: Presets & Data Quality (Gaps 8, 15)

### Task 15: Enhance presets with country pack awareness (Gap 8)

**Context:** `generate_companies()` in `presets.rs` has hardcoded country/currency pairs per industry. While this works, the preset could validate that country packs exist for the assigned countries.

**Files:**
- Modify: `crates/datasynth-config/src/presets.rs`

**Step 1: Add country pack config to presets**

In `create_preset()`, add a `country_packs: None` entry to the returned `GeneratorConfig`. This ensures the orchestrator will load builtin packs. No logic change needed since the orchestrator already handles `None` by loading builtins.

The key enhancement: ensure generated companies use countries that have builtin packs (US, DE, GB) or `_DEFAULT`. Review existing presets and confirm they use US/DE/GB/JP/CN/etc. - for countries without explicit packs, `_DEFAULT` applies automatically.

**Step 2: Commit**

```bash
git add crates/datasynth-config/src/presets.rs
git commit -m "feat(config): ensure presets include country_packs config"
```

---

### Task 16: Add country-aware format variations (Gap 15 / data quality)

**Context:** `format_variations.rs` has `DateFormat` (ISO, US, EU, EUDot) and `AmountFormat` (Plain, USComma, EUFormat, etc.) but no mapping from country code to preferred format. The country pack has `locale.date_format` and `locale.number_format`.

**Files:**
- Modify: `crates/datasynth-generators/src/data_quality/format_variations.rs`

**Step 1: Add country-to-format mapping**

Add a function `preferred_formats_for_pack(pack: &CountryPack) -> (DateFormat, AmountFormat)` that maps:
- `pack.locale.date_format.short == "MM/DD/YYYY"` → `DateFormat::Us`
- `pack.locale.date_format.short == "DD.MM.YYYY"` → `DateFormat::EuDot`
- `pack.locale.date_format.short == "DD/MM/YYYY"` → `DateFormat::Eu`
- `pack.locale.number_format.decimal_separator == ","` → `AmountFormat::EuFormat`
- `pack.locale.number_format.thousands_separator == " "` → `AmountFormat::SpaceSeparator`
- Otherwise → default

**Step 2: Wire from orchestrator**

In the data quality injection phase, resolve the country pack and pass preferred formats.

**Step 3: Test and commit**

```bash
git add crates/datasynth-generators/src/data_quality/format_variations.rs crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "feat(data-quality): add country-aware format variation preferences"
```

---

## Batch 8: Graph, OCPM, Fingerprint Metadata (Gaps 12, 13, 14)

### Task 17: Add country metadata to graph nodes (Gap 14)

**Context:** Graph nodes (`AccountNode`, entity nodes) don't carry country metadata. For ML training, country is a useful feature.

**Files:**
- Modify: `crates/datasynth-graph/src/models/nodes.rs`
- Modify: `crates/datasynth-graph/src/builders/transaction_graph.rs`
- Modify: `crates/datasynth-graph/src/builders/entity_graph.rs`

**Step 1: Add optional country field to node types**

Add `pub country: Option<String>` to `CompanyNode` and similar node structs.

**Step 2: Populate from company config**

In the graph builders, when constructing nodes, read `company.country` from the config and set it on the node.

**Step 3: Test and commit**

```bash
git add crates/datasynth-graph/src/
git commit -m "feat(graph): add country metadata to graph nodes"
```

---

### Task 18: Add country_code to OCPM event metadata (Gap 13)

**Context:** OCEL 2.0 events don't carry country context. Adding it helps process mining tools analyze regional variations.

**Files:**
- Modify: `crates/datasynth-ocpm/src/generator/p2p_generator.rs`
- Modify: `crates/datasynth-ocpm/src/generator/o2c_generator.rs`

**Step 1: Add country_code to event attributes**

Add an optional `country_code` attribute to generated events. This can be a simple addition to the event's attribute map.

**Step 2: Wire from orchestrator**

In `phase_ocpm_events`, pass the primary company's country code to the OCPM generators.

**Step 3: Test and commit**

```bash
git add crates/datasynth-ocpm/src/
git commit -m "feat(ocpm): add country_code to OCEL 2.0 event attributes"
```

---

### Task 19: Add country_code to fingerprint metadata (Gap 12)

**Context:** `SourceMetadata` in `manifest.rs` has `industry` and `metadata: HashMap<String, String>` but no explicit country field.

**Files:**
- Modify: `crates/datasynth-fingerprint/src/models/manifest.rs`

**Step 1: Add country_code to SourceMetadata**

Add `pub country_code: Option<String>` to `SourceMetadata`. When extracting a fingerprint, read the company's country from config and store it.

**Step 2: Test and commit**

```bash
git add crates/datasynth-fingerprint/src/
git commit -m "feat(fingerprint): add country_code to fingerprint metadata"
```

---

## Batch 9: Orchestrator Per-Company Dispatch & Holiday Calendar (Gaps 16, 17)

### Task 20: Refactor orchestrator to resolve pack per-company (Gap 16)

**Context:** Currently the orchestrator does `config.companies.first()` to get the primary country and resolves one pack. For multi-company scenarios, each company should get its own pack. However, most generators are single-pass, so the practical approach is to create a helper `fn primary_pack(&self) -> &CountryPack` and a `fn pack_for_company(&self, company: &CompanyConfig) -> &CountryPack`.

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs`

**Step 1: Add helper methods**

```rust
/// Resolve the country pack for the primary (first) company.
fn primary_pack(&self) -> &datasynth_core::CountryPack {
    let country = self.config.companies.first()
        .map(|c| c.country.as_str())
        .unwrap_or("US");
    self.country_pack_registry.get_by_str(country)
}

/// Resolve the country pack for a specific company.
fn pack_for_company(&self, company: &CompanyConfig) -> &datasynth_core::CountryPack {
    self.country_pack_registry.get_by_str(&company.country)
}
```

**Step 2: Replace repeated `config.companies.first()` patterns**

Search for the duplicated pattern:
```rust
let primary_country = self.config.companies.first()
    .map(|c| c.country.as_str())
    .unwrap_or("US");
let pack = self.country_pack_registry.get_by_str(primary_country);
```

Replace all occurrences with `let pack = self.primary_pack();`

**Step 3: Test and commit**

```bash
git add crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "refactor(runtime): add primary_pack() helper to reduce country pack boilerplate"
```

---

### Task 21: Replace hardcoded holiday calendars with country pack holidays (Gap 17)

**Context:** `holidays.rs` has 11 hardcoded region implementations (~1250 lines). `HolidayCalendar::from_country_pack()` already exists (line 159) and converts pack holiday data into a working calendar. The legacy `for_region()` method should fall back to `from_country_pack()` when a registry is available.

This is already partially done - the JE generator uses `from_country_pack()` via `with_country_pack_temporal()`. The remaining gap is other generators that still call `HolidayCalendar::for_region()` directly.

**Files:**
- Modify: `crates/datasynth-generators/src/je_generator.rs` (already done via Task 1)
- Audit: other generators that call `HolidayCalendar::for_region()`

**Step 1: Search for remaining for_region() calls**

```bash
grep -rn "for_region" crates/datasynth-generators/src/ --include="*.rs"
grep -rn "for_region" crates/datasynth-runtime/src/ --include="*.rs"
```

For each call found, evaluate if it can be replaced with `from_country_pack()` using a pack passed from the orchestrator.

**Step 2: Replace where feasible**

**Step 3: Test and commit**

```bash
git commit -m "feat(generators): prefer country pack holidays over hardcoded region calendars"
```

---

## Batch 10: Final Verification

### Task 22: Full workspace verification

**Step 1: Run all tests**

Run: `cargo test --workspace --exclude datasynth-ui`
Expected: all tests pass (2000+ existing + new tests)

**Step 2: Run clippy**

Run: `cargo clippy --workspace`
Expected: no new warnings (only expected `protoc not found` from datasynth-server)

**Step 3: Run fmt**

Run: `cargo fmt --check`
Expected: clean

**Step 4: Final commit if any formatting fixes needed**

```bash
cargo fmt
git add -A
git commit -m "style: apply cargo fmt after country pack wiring"
```

---

## Summary

| Batch | Tasks | Gaps Covered | Complexity |
|-------|-------|-------------|------------|
| 1: Wire existing methods | 1-3 | 2, 5, 7 | Low (orchestrator changes only) |
| 2: Banking wiring | 4 | 3 | Medium |
| 3: LLM enrichment | 5-8 | 1 | Medium |
| 4: Master data | 9-10 | 4 | Medium |
| 5: Document texts | 11-12 | 9, 10 | Medium |
| 6: Standards & payroll | 13-14 | 6, 11 | Low |
| 7: Presets & data quality | 15-16 | 8, 15 | Low-Medium |
| 8: Metadata | 17-19 | 12, 13, 14 | Low |
| 9: Orchestrator refactor | 20-21 | 16, 17 | Low |
| 10: Verification | 22 | All | Verification only |

**Total: 22 tasks across 10 batches covering all 17 integration gaps.**
