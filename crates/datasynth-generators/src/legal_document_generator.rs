//! Legal document generator for audit engagements.
//!
//! Generates realistic legal documents (engagement letters, management
//! representation letters, legal opinions, regulatory filings, and board
//! resolutions) that support GAM audit procedures.

use chrono::NaiveDate;
use datasynth_core::models::LegalDocument;
use datasynth_core::utils::seeded_rng;
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

// ---------------------------------------------------------------------------
// Template pools
// ---------------------------------------------------------------------------

const ENGAGEMENT_LETTER_TERMS: &[&str] = &[
    "Scope of audit procedures",
    "Management responsibilities for financial statements",
    "Auditor responsibilities under ISA",
    "Expected form and content of audit report",
    "Fee arrangements and billing schedule",
    "Access to records and personnel",
    "Confidentiality and data protection obligations",
    "Communication of internal control deficiencies",
    "Anti-money laundering compliance requirements",
    "Independence requirements and safeguards",
];

const MANAGEMENT_REP_TERMS: &[&str] = &[
    "Financial statements prepared in accordance with applicable framework",
    "All transactions recorded and reflected in financial statements",
    "Internal controls designed and maintained for reliable reporting",
    "All known fraud or suspected fraud communicated to auditor",
    "All related party relationships and transactions disclosed",
    "No material subsequent events requiring adjustment or disclosure",
    "Going concern assessment provided to auditor",
    "All known litigation and claims disclosed",
    "Compliance with laws and regulations confirmed",
    "Uncorrected misstatements assessed as immaterial",
];

const LEGAL_OPINION_TERMS: &[&str] = &[
    "Entity duly incorporated and in good standing",
    "Authorization of transactions under applicable law",
    "No pending litigation materially affecting financial position",
    "Compliance with contractual obligations",
    "Regulatory approval obtained for disclosed transactions",
    "Tax position supportable under applicable legislation",
];

const REGULATORY_FILING_TERMS: &[&str] = &[
    "Annual financial statements filed with regulator",
    "Tax return submitted to competent authority",
    "Securities disclosure requirements satisfied",
    "Capital adequacy ratio reported to banking authority",
    "Environmental compliance report submitted",
    "Anti-money laundering annual report filed",
    "Data protection annual assessment filed",
    "Corporate governance statement submitted",
];

const BOARD_RESOLUTION_TERMS: &[&str] = &[
    "Appointment of external auditor approved",
    "Audit committee composition confirmed",
    "Financial statements approved for issuance",
    "Dividend distribution authorized",
    "Related party transactions ratified",
    "Internal audit charter approved",
    "Risk appetite statement adopted",
    "Compliance program reviewed and endorsed",
];

const SENIORITY_TITLES: &[&str] = &[
    "Chief Executive Officer",
    "Chief Financial Officer",
    "General Counsel",
    "Board Chairperson",
    "Audit Committee Chair",
    "Chief Compliance Officer",
    "Company Secretary",
    "Head of Internal Audit",
    "Controller",
    "VP of Finance",
];

/// Configuration for the legal document generator.
pub struct LegalDocumentGeneratorConfig {
    /// Minimum legal opinions per engagement (default: 0).
    pub legal_opinion_min: u32,
    /// Maximum legal opinions per engagement (default: 2).
    pub legal_opinion_max: u32,
    /// Minimum regulatory filings per engagement (default: 1).
    pub regulatory_filing_min: u32,
    /// Maximum regulatory filings per engagement (default: 3).
    pub regulatory_filing_max: u32,
    /// Minimum board resolutions per engagement (default: 1).
    pub board_resolution_min: u32,
    /// Maximum board resolutions per engagement (default: 2).
    pub board_resolution_max: u32,
}

impl Default for LegalDocumentGeneratorConfig {
    fn default() -> Self {
        Self {
            legal_opinion_min: 0,
            legal_opinion_max: 2,
            regulatory_filing_min: 1,
            regulatory_filing_max: 3,
            board_resolution_min: 1,
            board_resolution_max: 2,
        }
    }
}

/// Generates [`LegalDocument`] records for audit engagements.
pub struct LegalDocumentGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
    config: LegalDocumentGeneratorConfig,
}

impl LegalDocumentGenerator {
    /// Create a new generator with the given seed and default configuration.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::LegalDocument),
            config: LegalDocumentGeneratorConfig::default(),
        }
    }

    /// Create a new generator with explicit configuration.
    pub fn with_config(seed: u64, config: LegalDocumentGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::LegalDocument),
            config,
        }
    }

    /// Generate legal documents for a single engagement.
    ///
    /// Produces:
    /// - 1 engagement letter
    /// - 1 management representation letter
    /// - 0-2 legal opinions
    /// - 1-3 regulatory filings
    /// - 1-2 board resolutions
    ///
    /// Signatories are drawn from `employee_names`; if empty, generic
    /// titles are used as fallback.
    pub fn generate(
        &mut self,
        entity_code: &str,
        fiscal_year: i32,
        employee_names: &[String],
    ) -> Vec<LegalDocument> {
        let mut docs = Vec::new();

        // Engagement letter (always 1, near start of fiscal year)
        if let Some(date) = NaiveDate::from_ymd_opt(fiscal_year, 1, 15) {
            docs.push(self.make_document(
                "engagement_letter",
                entity_code,
                date,
                &format!("Engagement Letter — {} FY{}", entity_code, fiscal_year),
                ENGAGEMENT_LETTER_TERMS,
                employee_names,
                "signed",
                2,
                3,
            ));
        }

        // Management representation letter (always 1, near year-end close)
        if let Some(date) = NaiveDate::from_ymd_opt(fiscal_year, 12, 20) {
            docs.push(self.make_document(
                "management_rep",
                entity_code,
                date,
                &format!(
                    "Management Representation Letter — {} FY{}",
                    entity_code, fiscal_year
                ),
                MANAGEMENT_REP_TERMS,
                employee_names,
                "signed",
                2,
                4,
            ));
        }

        // Legal opinions (0-2)
        let opinion_count = self
            .rng
            .random_range(self.config.legal_opinion_min..=self.config.legal_opinion_max);
        for i in 0..opinion_count {
            let month = self.rng.random_range(3u32..=11);
            if let Some(date) = NaiveDate::from_ymd_opt(fiscal_year, month, 10) {
                docs.push(self.make_document(
                    "legal_opinion",
                    entity_code,
                    date,
                    &format!(
                        "Legal Opinion #{} — {} FY{}",
                        i + 1,
                        entity_code,
                        fiscal_year
                    ),
                    LEGAL_OPINION_TERMS,
                    employee_names,
                    "final",
                    1,
                    2,
                ));
            }
        }

        // Regulatory filings (1-3)
        let filing_count = self
            .rng
            .random_range(self.config.regulatory_filing_min..=self.config.regulatory_filing_max);
        for i in 0..filing_count {
            let month = self.rng.random_range(1u32..=12);
            let day = self.rng.random_range(1u32..=28);
            if let Some(date) = NaiveDate::from_ymd_opt(fiscal_year, month, day) {
                docs.push(self.make_document(
                    "regulatory_filing",
                    entity_code,
                    date,
                    &format!(
                        "Regulatory Filing #{} — {} FY{}",
                        i + 1,
                        entity_code,
                        fiscal_year
                    ),
                    REGULATORY_FILING_TERMS,
                    employee_names,
                    "signed",
                    1,
                    2,
                ));
            }
        }

        // Board resolutions (1-2)
        let resolution_count = self
            .rng
            .random_range(self.config.board_resolution_min..=self.config.board_resolution_max);
        for i in 0..resolution_count {
            let month = self.rng.random_range(1u32..=12);
            if let Some(date) = NaiveDate::from_ymd_opt(fiscal_year, month, 25) {
                docs.push(self.make_document(
                    "board_resolution",
                    entity_code,
                    date,
                    &format!(
                        "Board Resolution #{} — {} FY{}",
                        i + 1,
                        entity_code,
                        fiscal_year
                    ),
                    BOARD_RESOLUTION_TERMS,
                    employee_names,
                    "signed",
                    3,
                    5,
                ));
            }
        }

        // Sort chronologically
        docs.sort_by_key(|d| d.date);
        docs
    }

    /// Build a single legal document.
    #[allow(clippy::too_many_arguments)]
    fn make_document(
        &mut self,
        doc_type: &str,
        entity_code: &str,
        date: NaiveDate,
        title: &str,
        terms_pool: &[&str],
        employee_names: &[String],
        status: &str,
        signatories_min: usize,
        signatories_max: usize,
    ) -> LegalDocument {
        let signatories = self.pick_signatories(employee_names, signatories_min, signatories_max);
        let key_terms = self.pick_items(terms_pool, 3, terms_pool.len().min(6));

        LegalDocument {
            document_id: self.uuid_factory.next(),
            document_type: doc_type.to_string(),
            entity_code: entity_code.to_string(),
            date,
            title: title.to_string(),
            signatories,
            key_terms,
            status: status.to_string(),
        }
    }

    /// Pick signatories from the employee pool (or use generic titles as fallback).
    fn pick_signatories(&mut self, pool: &[String], min: usize, max: usize) -> Vec<String> {
        let source: Vec<String> = if pool.is_empty() {
            SENIORITY_TITLES.iter().map(|s| (*s).to_string()).collect()
        } else {
            pool.to_vec()
        };
        let count = self.rng.random_range(min..=max).min(source.len());
        let mut indices: Vec<usize> = (0..source.len()).collect();
        indices.shuffle(&mut self.rng);
        indices.truncate(count);
        indices.sort_unstable();
        indices.iter().map(|&i| source[i].clone()).collect()
    }

    /// Randomly pick `min..=max` items from a template pool.
    fn pick_items(&mut self, pool: &[&str], min: usize, max: usize) -> Vec<String> {
        let count = self.rng.random_range(min..=max).min(pool.len());
        let mut indices: Vec<usize> = (0..pool.len()).collect();
        indices.shuffle(&mut self.rng);
        indices.truncate(count);
        indices.sort_unstable();
        indices.iter().map(|&i| pool[i].to_string()).collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::Datelike;

    fn sample_employees() -> Vec<String> {
        (1..=15).map(|i| format!("Employee_{:03}", i)).collect()
    }

    #[test]
    fn test_generates_non_empty_output() {
        let mut gen = LegalDocumentGenerator::new(42);
        let docs = gen.generate("C001", 2025, &sample_employees());
        assert!(!docs.is_empty(), "should produce legal documents");
    }

    #[test]
    fn test_document_count_range() {
        let mut gen = LegalDocumentGenerator::new(42);
        let docs = gen.generate("C001", 2025, &sample_employees());
        // Min: 1 engagement + 1 mgmt rep + 0 legal + 1 filing + 1 resolution = 4
        // Max: 1 engagement + 1 mgmt rep + 2 legal + 3 filing + 2 resolution = 9
        assert!(
            docs.len() >= 4 && docs.len() <= 9,
            "expected 4-9 documents, got {}",
            docs.len()
        );
    }

    #[test]
    fn test_has_engagement_letter() {
        let mut gen = LegalDocumentGenerator::new(42);
        let docs = gen.generate("C001", 2025, &sample_employees());
        let engagement = docs
            .iter()
            .filter(|d| d.document_type == "engagement_letter")
            .count();
        assert_eq!(engagement, 1, "should have exactly 1 engagement letter");
    }

    #[test]
    fn test_has_management_rep() {
        let mut gen = LegalDocumentGenerator::new(42);
        let docs = gen.generate("C001", 2025, &sample_employees());
        let mgmt_rep = docs
            .iter()
            .filter(|d| d.document_type == "management_rep")
            .count();
        assert_eq!(mgmt_rep, 1, "should have exactly 1 management rep letter");
    }

    #[test]
    fn test_document_types_correct() {
        let mut gen = LegalDocumentGenerator::new(42);
        let docs = gen.generate("C001", 2025, &sample_employees());
        let valid_types = [
            "engagement_letter",
            "management_rep",
            "legal_opinion",
            "regulatory_filing",
            "board_resolution",
        ];
        for doc in &docs {
            assert!(
                valid_types.contains(&doc.document_type.as_str()),
                "unexpected document type: {}",
                doc.document_type
            );
        }
    }

    #[test]
    fn test_entity_code_propagated() {
        let mut gen = LegalDocumentGenerator::new(42);
        let docs = gen.generate("TEST_ENTITY", 2025, &sample_employees());
        for doc in &docs {
            assert_eq!(
                doc.entity_code, "TEST_ENTITY",
                "entity_code should match input"
            );
        }
    }

    #[test]
    fn test_dates_within_fiscal_year() {
        let mut gen = LegalDocumentGenerator::new(42);
        let docs = gen.generate("C001", 2025, &sample_employees());
        for doc in &docs {
            assert_eq!(doc.date.year(), 2025, "document date should be in FY2025");
        }
    }

    #[test]
    fn test_dates_sorted() {
        let mut gen = LegalDocumentGenerator::new(42);
        let docs = gen.generate("C001", 2025, &sample_employees());
        for w in docs.windows(2) {
            assert!(
                w[0].date <= w[1].date,
                "documents should be sorted chronologically"
            );
        }
    }

    #[test]
    fn test_unique_ids() {
        let mut gen = LegalDocumentGenerator::new(42);
        let docs = gen.generate("C001", 2025, &sample_employees());
        let ids: std::collections::HashSet<_> = docs.iter().map(|d| d.document_id).collect();
        assert_eq!(ids.len(), docs.len(), "all document IDs should be unique");
    }

    #[test]
    fn test_signatories_present() {
        let mut gen = LegalDocumentGenerator::new(42);
        let docs = gen.generate("C001", 2025, &sample_employees());
        for doc in &docs {
            assert!(
                !doc.signatories.is_empty(),
                "document {} should have signatories",
                doc.document_type
            );
        }
    }

    #[test]
    fn test_key_terms_present() {
        let mut gen = LegalDocumentGenerator::new(42);
        let docs = gen.generate("C001", 2025, &sample_employees());
        for doc in &docs {
            assert!(
                !doc.key_terms.is_empty(),
                "document {} should have key terms",
                doc.document_type
            );
        }
    }

    #[test]
    fn test_empty_employee_pool_fallback() {
        let mut gen = LegalDocumentGenerator::new(42);
        let docs = gen.generate("C001", 2025, &[]);
        assert!(!docs.is_empty(), "should produce docs with empty pool");
        for doc in &docs {
            assert!(
                !doc.signatories.is_empty(),
                "should have fallback signatories"
            );
        }
    }

    #[test]
    fn test_deterministic_with_same_seed() {
        let employees = sample_employees();

        let mut gen1 = LegalDocumentGenerator::new(999);
        let d1 = gen1.generate("C001", 2025, &employees);

        let mut gen2 = LegalDocumentGenerator::new(999);
        let d2 = gen2.generate("C001", 2025, &employees);

        assert_eq!(d1.len(), d2.len());
        for (a, b) in d1.iter().zip(d2.iter()) {
            assert_eq!(a.document_id, b.document_id);
            assert_eq!(a.document_type, b.document_type);
            assert_eq!(a.date, b.date);
            assert_eq!(a.title, b.title);
            assert_eq!(a.key_terms, b.key_terms);
        }
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut gen = LegalDocumentGenerator::new(42);
        let docs = gen.generate("C001", 2025, &sample_employees());
        let json = serde_json::to_string(&docs).expect("serialize");
        let parsed: Vec<LegalDocument> = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(docs.len(), parsed.len());
        for (orig, rt) in docs.iter().zip(parsed.iter()) {
            assert_eq!(orig.document_id, rt.document_id);
            assert_eq!(orig.document_type, rt.document_type);
            assert_eq!(orig.date, rt.date);
        }
    }
}
