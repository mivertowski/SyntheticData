//! Tax Line Generator (Decorator Pattern).
//!
//! Generates [`TaxLine`] records for existing AP/AR/JE documents. This is a
//! "decorator" generator — it runs **after** invoice generators and enriches
//! documents with tax information based on the seller/buyer countries, product
//! category exemptions, and EU reverse-charge rules.

use chrono::NaiveDate;
use datasynth_core::utils::seeded_rng;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use std::collections::{HashMap, HashSet};

use datasynth_core::models::{TaxCode, TaxLine, TaxableDocumentType};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for tax line generation behavior.
#[derive(Debug, Clone)]
pub struct TaxLineGeneratorConfig {
    /// Product categories exempt from tax (e.g., "financial_services", "education").
    pub exempt_categories: Vec<String>,
    /// EU member country codes (for reverse-charge determination).
    pub eu_countries: HashSet<String>,
}

impl Default for TaxLineGeneratorConfig {
    fn default() -> Self {
        Self {
            exempt_categories: Vec::new(),
            eu_countries: HashSet::from([
                "DE".into(),
                "FR".into(),
                "IT".into(),
                "ES".into(),
                "NL".into(),
                "BE".into(),
                "AT".into(),
                "PT".into(),
                "IE".into(),
                "FI".into(),
                "SE".into(),
                "DK".into(),
                "PL".into(),
                "CZ".into(),
                "RO".into(),
                "HU".into(),
                "BG".into(),
                "HR".into(),
                "SK".into(),
                "SI".into(),
                "LT".into(),
                "LV".into(),
                "EE".into(),
                "CY".into(),
                "LU".into(),
                "MT".into(),
                "GR".into(),
            ]),
        }
    }
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Generates tax lines for documents (AP invoices, AR invoices, JEs).
///
/// This is a decorator generator that takes a set of pre-generated
/// [`TaxCode`]s and produces [`TaxLine`] records for source documents.
///
/// # Tax Determination Logic
///
/// 1. **Exempt check** — if the product category is in `config.exempt_categories`,
///    no tax lines are emitted.
/// 2. **Jurisdiction determination** — for `VendorInvoice` (input tax) the
///    seller country is the taxing jurisdiction; for `CustomerInvoice` (output
///    tax) the buyer country is used (destination-based); for `JournalEntry`
///    the seller country is used.
/// 3. **EU reverse charge** — when both countries are in the EU and differ,
///    reverse-charge rules apply: the buyer self-assesses at their own rate.
/// 4. **Deductibility** — vendor invoices are deductible (input VAT is
///    recoverable), customer invoices and journal entries are not.
///
/// # Examples
///
/// ```
/// use datasynth_generators::tax::TaxLineGenerator;
/// use datasynth_generators::tax::TaxLineGeneratorConfig;
/// use datasynth_generators::tax::TaxCodeGenerator;
///
/// let mut code_gen = TaxCodeGenerator::new(42);
/// let (_jurisdictions, codes) = code_gen.generate();
///
/// let mut gen = TaxLineGenerator::new(TaxLineGeneratorConfig::default(), codes, 42);
/// ```
pub struct TaxLineGenerator {
    rng: ChaCha8Rng,
    /// Tax codes indexed by jurisdiction_id for fast lookup.
    tax_codes_by_jurisdiction: HashMap<String, Vec<TaxCode>>,
    config: TaxLineGeneratorConfig,
    counter: u64,
}

impl TaxLineGenerator {
    /// Creates a new tax line generator.
    ///
    /// `tax_codes` are indexed by their `jurisdiction_id` for O(1) lookup.
    pub fn new(config: TaxLineGeneratorConfig, tax_codes: Vec<TaxCode>, seed: u64) -> Self {
        let mut tax_codes_by_jurisdiction: HashMap<String, Vec<TaxCode>> = HashMap::new();
        for code in tax_codes {
            tax_codes_by_jurisdiction
                .entry(code.jurisdiction_id.clone())
                .or_default()
                .push(code);
        }

        Self {
            rng: seeded_rng(seed, 0),
            tax_codes_by_jurisdiction,
            config,
            counter: 0,
        }
    }

    /// Generates tax lines for a single document.
    ///
    /// Determines the applicable tax code based on:
    /// - `seller_country` / `buyer_country` to select jurisdiction
    /// - Cross-border EU B2B transactions trigger reverse charge
    /// - `product_category` may trigger an exemption
    /// - `doc_type` determines input (AP) vs output (AR) tax treatment
    pub fn generate_for_document(
        &mut self,
        doc_type: TaxableDocumentType,
        doc_id: &str,
        seller_country: &str,
        buyer_country: &str,
        taxable_amount: Decimal,
        date: NaiveDate,
        product_category: Option<&str>,
    ) -> Vec<TaxLine> {
        // 1. Exempt check
        if let Some(cat) = product_category {
            if self
                .config
                .exempt_categories
                .iter()
                .any(|e| e.eq_ignore_ascii_case(cat))
            {
                return Vec::new();
            }
        }

        // 2. Determine taxing jurisdiction
        let jurisdiction_country = match doc_type {
            TaxableDocumentType::VendorInvoice => seller_country,
            TaxableDocumentType::CustomerInvoice => {
                // Destination-based: use buyer country
                // But if same country, still use that country
                buyer_country
            }
            TaxableDocumentType::JournalEntry => seller_country,
            // Payment and PayrollRun not typically decorated with tax lines here
            _ => seller_country,
        };

        // 3. EU cross-border reverse charge
        let is_eu_cross_border = seller_country != buyer_country
            && self.config.eu_countries.contains(seller_country)
            && self.config.eu_countries.contains(buyer_country);

        if is_eu_cross_border {
            return self.generate_reverse_charge_line(
                doc_type,
                doc_id,
                buyer_country,
                taxable_amount,
                date,
            );
        }

        // 4. US Sales Tax special case: use buyer state (e.g., "US-CA" -> "JUR-US-CA")
        let jurisdiction_id = self.resolve_jurisdiction_id(jurisdiction_country);

        // 5. Look up tax codes for the jurisdiction
        let tax_code = match self.find_standard_code(&jurisdiction_id, date) {
            Some(code) => code,
            None => return Vec::new(), // No matching code -> no tax
        };

        // 6. Compute tax
        let tax_amount = tax_code.tax_amount(taxable_amount);
        let is_deductible = matches!(doc_type, TaxableDocumentType::VendorInvoice);

        let line = self.build_tax_line(
            doc_type,
            doc_id,
            &tax_code.id,
            &jurisdiction_id,
            taxable_amount,
            tax_amount,
            is_deductible,
            false, // not reverse charge
            false, // not self-assessed
        );

        vec![line]
    }

    /// Batch-generates tax lines for multiple documents.
    ///
    /// Each tuple element: `(doc_id, seller_country, buyer_country, amount, date, optional category)`.
    pub fn generate_batch(
        &mut self,
        doc_type: TaxableDocumentType,
        documents: &[(String, String, String, Decimal, NaiveDate, Option<String>)],
    ) -> Vec<TaxLine> {
        let mut result = Vec::new();
        for (doc_id, seller, buyer, amount, date, category) in documents {
            let lines = self.generate_for_document(
                doc_type,
                doc_id,
                seller,
                buyer,
                *amount,
                *date,
                category.as_deref(),
            );
            result.extend(lines);
        }
        result
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Generates a reverse-charge tax line for EU cross-border transactions.
    ///
    /// The buyer self-assesses at their own country's standard rate.
    fn generate_reverse_charge_line(
        &mut self,
        doc_type: TaxableDocumentType,
        doc_id: &str,
        buyer_country: &str,
        taxable_amount: Decimal,
        date: NaiveDate,
    ) -> Vec<TaxLine> {
        let buyer_jurisdiction_id = self.resolve_jurisdiction_id(buyer_country);

        let tax_code = match self.find_standard_code(&buyer_jurisdiction_id, date) {
            Some(code) => code,
            None => return Vec::new(),
        };

        let tax_amount = tax_code.tax_amount(taxable_amount);
        let is_deductible = matches!(doc_type, TaxableDocumentType::VendorInvoice);

        let line = self.build_tax_line(
            doc_type,
            doc_id,
            &tax_code.id,
            &buyer_jurisdiction_id,
            taxable_amount,
            tax_amount,
            is_deductible,
            true, // reverse charge
            true, // self-assessed
        );

        vec![line]
    }

    /// Resolves a country code to a jurisdiction ID.
    ///
    /// For US state codes like "US-CA", maps to "JUR-US-CA".
    /// For country codes like "DE", maps to "JUR-DE".
    fn resolve_jurisdiction_id(&self, country_or_state: &str) -> String {
        if let Some(state_code) = country_or_state.strip_prefix("US-") {
            // US state-level: "US-CA" -> "JUR-US-CA"
            format!("JUR-US-{state_code}")
        } else {
            format!("JUR-{country_or_state}")
        }
    }

    /// Finds the standard-rate (non-exempt, non-reduced) tax code for a
    /// jurisdiction that is active on the given date.
    ///
    /// Selection priority:
    /// 1. Non-exempt code with the highest rate (standard rate)
    /// 2. Falls back to any active code
    fn find_standard_code(&self, jurisdiction_id: &str, date: NaiveDate) -> Option<TaxCode> {
        let codes = self.tax_codes_by_jurisdiction.get(jurisdiction_id)?;

        // Filter to active, non-exempt codes
        let mut candidates: Vec<&TaxCode> = codes
            .iter()
            .filter(|c| c.is_active(date) && !c.is_exempt)
            .collect();

        if candidates.is_empty() {
            return None;
        }

        // Sort by rate descending to pick the standard (highest) rate
        candidates.sort_by(|a, b| b.rate.cmp(&a.rate));

        Some(candidates[0].clone())
    }

    /// Builds a [`TaxLine`] with the next sequential ID.
    #[allow(clippy::too_many_arguments)]
    fn build_tax_line(
        &mut self,
        doc_type: TaxableDocumentType,
        doc_id: &str,
        tax_code_id: &str,
        jurisdiction_id: &str,
        taxable_amount: Decimal,
        tax_amount: Decimal,
        is_deductible: bool,
        is_reverse_charge: bool,
        is_self_assessed: bool,
    ) -> TaxLine {
        self.counter += 1;
        let line_id = format!("TXLN-{:06}", self.counter);

        // Use rng to slightly vary line_number for realism in future extensions
        let _noise: f64 = self.rng.gen();

        TaxLine::new(
            line_id,
            doc_type,
            doc_id,
            1, // line_number: one tax line per document call
            tax_code_id,
            jurisdiction_id,
            taxable_amount,
            tax_amount,
        )
        .with_deductible(is_deductible)
        .with_reverse_charge(is_reverse_charge)
        .with_self_assessed(is_self_assessed)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::tax::TaxCodeGenerator;
    use datasynth_config::schema::TaxConfig;
    use rust_decimal_macros::dec;

    /// Helper: generate tax codes for DE, FR, GB, US (with subnational).
    fn make_tax_codes() -> Vec<TaxCode> {
        let mut config = TaxConfig::default();
        config.jurisdictions.countries = vec!["DE".into(), "FR".into(), "GB".into(), "US".into()];
        config.jurisdictions.include_subnational = true;

        let mut gen = TaxCodeGenerator::with_config(42, config);
        let (_jurisdictions, codes) = gen.generate();
        codes
    }

    fn test_date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()
    }

    #[test]
    fn test_domestic_vendor_invoice() {
        let codes = make_tax_codes();
        let config = TaxLineGeneratorConfig::default();
        let mut gen = TaxLineGenerator::new(config, codes, 42);

        let lines = gen.generate_for_document(
            TaxableDocumentType::VendorInvoice,
            "INV-001",
            "DE", // seller
            "DE", // buyer
            dec!(10000),
            test_date(),
            None,
        );

        assert_eq!(lines.len(), 1, "Should produce one tax line");
        let line = &lines[0];
        assert_eq!(line.document_id, "INV-001");
        assert_eq!(line.jurisdiction_id, "JUR-DE");
        // DE standard VAT is 19%
        assert_eq!(line.tax_amount, dec!(1900.00));
        assert_eq!(line.taxable_amount, dec!(10000));
        assert!(line.is_deductible, "Vendor invoice input VAT is deductible");
        assert!(!line.is_reverse_charge);
        assert!(!line.is_self_assessed);
    }

    #[test]
    fn test_domestic_customer_invoice() {
        let codes = make_tax_codes();
        let config = TaxLineGeneratorConfig::default();
        let mut gen = TaxLineGenerator::new(config, codes, 42);

        let lines = gen.generate_for_document(
            TaxableDocumentType::CustomerInvoice,
            "CINV-001",
            "DE", // seller
            "DE", // buyer (destination = DE)
            dec!(5000),
            test_date(),
            None,
        );

        assert_eq!(lines.len(), 1);
        let line = &lines[0];
        assert_eq!(line.document_id, "CINV-001");
        assert_eq!(line.jurisdiction_id, "JUR-DE");
        // DE standard VAT 19%
        assert_eq!(line.tax_amount, dec!(950.00));
        assert!(
            !line.is_deductible,
            "Customer invoice output VAT is not deductible"
        );
        assert!(!line.is_reverse_charge);
    }

    #[test]
    fn test_eu_cross_border_reverse_charge() {
        let codes = make_tax_codes();
        let config = TaxLineGeneratorConfig::default();
        let mut gen = TaxLineGenerator::new(config, codes, 42);

        let lines = gen.generate_for_document(
            TaxableDocumentType::VendorInvoice,
            "INV-EU-001",
            "DE", // seller (EU)
            "FR", // buyer (EU, different country)
            dec!(20000),
            test_date(),
            None,
        );

        assert_eq!(lines.len(), 1, "Should produce one reverse-charge line");
        let line = &lines[0];
        assert_eq!(line.document_id, "INV-EU-001");
        // Buyer self-assesses at FR rate (20%)
        assert_eq!(line.jurisdiction_id, "JUR-FR");
        assert_eq!(line.tax_amount, dec!(4000.00));
        assert!(line.is_reverse_charge, "Should be reverse charge");
        assert!(line.is_self_assessed, "Buyer should self-assess");
        assert!(
            line.is_deductible,
            "Vendor invoice reverse charge is still deductible"
        );
    }

    #[test]
    fn test_exempt_category() {
        let codes = make_tax_codes();
        let config = TaxLineGeneratorConfig {
            exempt_categories: vec!["financial_services".into(), "education".into()],
            ..Default::default()
        };
        let mut gen = TaxLineGenerator::new(config, codes, 42);

        let lines = gen.generate_for_document(
            TaxableDocumentType::VendorInvoice,
            "INV-EXEMPT",
            "DE",
            "DE",
            dec!(50000),
            test_date(),
            Some("financial_services"),
        );

        assert!(
            lines.is_empty(),
            "Exempt category should produce no tax lines"
        );

        // Case-insensitive check
        let lines2 = gen.generate_for_document(
            TaxableDocumentType::VendorInvoice,
            "INV-EXEMPT-2",
            "DE",
            "DE",
            dec!(50000),
            test_date(),
            Some("FINANCIAL_SERVICES"),
        );
        assert!(
            lines2.is_empty(),
            "Exempt category check should be case-insensitive"
        );
    }

    #[test]
    fn test_non_eu_cross_border() {
        let codes = make_tax_codes();
        let config = TaxLineGeneratorConfig::default();
        let mut gen = TaxLineGenerator::new(config, codes, 42);

        // US seller -> DE buyer: NOT EU cross-border, no reverse charge
        // For VendorInvoice, jurisdiction = seller = US
        let lines = gen.generate_for_document(
            TaxableDocumentType::VendorInvoice,
            "INV-XBORDER",
            "US", // seller (non-EU)
            "DE", // buyer (EU)
            dec!(10000),
            test_date(),
            None,
        );

        // US federal has no sales tax codes (only state-level), so no tax line
        // unless there are federal-level codes. With our setup, JUR-US has no codes.
        // This is correct: cross-border non-EU -> seller country jurisdiction.
        // US federal has no tax codes -> empty result.
        assert!(
            lines.is_empty() || lines.iter().all(|l| !l.is_reverse_charge),
            "Non-EU cross-border should NOT use reverse charge"
        );
    }

    #[test]
    fn test_us_sales_tax() {
        let codes = make_tax_codes();
        let config = TaxLineGeneratorConfig::default();
        let mut gen = TaxLineGenerator::new(config, codes, 42);

        // Customer invoice: destination-based, buyer is in US-CA
        let lines = gen.generate_for_document(
            TaxableDocumentType::CustomerInvoice,
            "CINV-US-001",
            "US",    // seller
            "US-CA", // buyer (California)
            dec!(1000),
            test_date(),
            None,
        );

        assert_eq!(lines.len(), 1, "Should produce one sales tax line");
        let line = &lines[0];
        assert_eq!(line.jurisdiction_id, "JUR-US-CA");
        // California sales tax: 7.25%
        assert_eq!(line.tax_amount, dec!(72.50));
        assert!(!line.is_deductible, "Customer invoice not deductible");
    }

    #[test]
    fn test_no_matching_code() {
        let codes = make_tax_codes();
        let config = TaxLineGeneratorConfig::default();
        let mut gen = TaxLineGenerator::new(config, codes, 42);

        // Unknown jurisdiction -> no tax codes -> empty result
        let lines = gen.generate_for_document(
            TaxableDocumentType::VendorInvoice,
            "INV-UNKNOWN",
            "ZZ", // unknown country
            "ZZ",
            dec!(10000),
            test_date(),
            None,
        );

        assert!(
            lines.is_empty(),
            "Unknown jurisdiction should produce no tax lines"
        );
    }

    #[test]
    fn test_batch_generation() {
        let codes = make_tax_codes();
        let config = TaxLineGeneratorConfig::default();
        let mut gen = TaxLineGenerator::new(config, codes, 42);
        let date = test_date();

        let documents = vec![
            (
                "INV-B1".into(),
                "DE".into(),
                "DE".into(),
                dec!(1000),
                date,
                None,
            ),
            (
                "INV-B2".into(),
                "FR".into(),
                "FR".into(),
                dec!(2000),
                date,
                None,
            ),
            (
                "INV-B3".into(),
                "GB".into(),
                "GB".into(),
                dec!(3000),
                date,
                None,
            ),
        ];

        let lines = gen.generate_batch(TaxableDocumentType::VendorInvoice, &documents);

        assert_eq!(lines.len(), 3, "Should produce one line per document");

        // Verify each document got its own line
        let doc_ids: Vec<&str> = lines.iter().map(|l| l.document_id.as_str()).collect();
        assert!(doc_ids.contains(&"INV-B1"));
        assert!(doc_ids.contains(&"INV-B2"));
        assert!(doc_ids.contains(&"INV-B3"));

        // DE: 19%, FR: 20%, GB: 20%
        let de_line = lines.iter().find(|l| l.document_id == "INV-B1").unwrap();
        assert_eq!(de_line.tax_amount, dec!(190.00));

        let fr_line = lines.iter().find(|l| l.document_id == "INV-B2").unwrap();
        assert_eq!(fr_line.tax_amount, dec!(400.00));

        let gb_line = lines.iter().find(|l| l.document_id == "INV-B3").unwrap();
        assert_eq!(gb_line.tax_amount, dec!(600.00));
    }

    #[test]
    fn test_deterministic() {
        let codes1 = make_tax_codes();
        let codes2 = make_tax_codes();
        let config1 = TaxLineGeneratorConfig::default();
        let config2 = TaxLineGeneratorConfig::default();
        let date = test_date();

        let mut gen1 = TaxLineGenerator::new(config1, codes1, 999);
        let mut gen2 = TaxLineGenerator::new(config2, codes2, 999);

        let lines1 = gen1.generate_for_document(
            TaxableDocumentType::VendorInvoice,
            "INV-DET",
            "DE",
            "DE",
            dec!(5000),
            date,
            None,
        );
        let lines2 = gen2.generate_for_document(
            TaxableDocumentType::VendorInvoice,
            "INV-DET",
            "DE",
            "DE",
            dec!(5000),
            date,
            None,
        );

        assert_eq!(lines1.len(), lines2.len());
        for (l1, l2) in lines1.iter().zip(lines2.iter()) {
            assert_eq!(l1.id, l2.id);
            assert_eq!(l1.tax_code_id, l2.tax_code_id);
            assert_eq!(l1.tax_amount, l2.tax_amount);
            assert_eq!(l1.jurisdiction_id, l2.jurisdiction_id);
            assert_eq!(l1.is_deductible, l2.is_deductible);
            assert_eq!(l1.is_reverse_charge, l2.is_reverse_charge);
        }
    }

    #[test]
    fn test_line_counter_increments() {
        let codes = make_tax_codes();
        let config = TaxLineGeneratorConfig::default();
        let mut gen = TaxLineGenerator::new(config, codes, 42);
        let date = test_date();

        let lines1 = gen.generate_for_document(
            TaxableDocumentType::VendorInvoice,
            "INV-C1",
            "DE",
            "DE",
            dec!(1000),
            date,
            None,
        );
        let lines2 = gen.generate_for_document(
            TaxableDocumentType::VendorInvoice,
            "INV-C2",
            "DE",
            "DE",
            dec!(2000),
            date,
            None,
        );
        let lines3 = gen.generate_for_document(
            TaxableDocumentType::VendorInvoice,
            "INV-C3",
            "DE",
            "DE",
            dec!(3000),
            date,
            None,
        );

        assert_eq!(lines1[0].id, "TXLN-000001");
        assert_eq!(lines2[0].id, "TXLN-000002");
        assert_eq!(lines3[0].id, "TXLN-000003");
    }
}
