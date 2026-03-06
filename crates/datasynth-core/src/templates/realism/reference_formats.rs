//! Enhanced reference number format generation.
//!
//! Provides ERP-style reference number generation with multiple format
//! options and realistic patterns.

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

/// Reference format types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EnhancedReferenceFormat {
    /// Standard format: PREFIX-YYYY-NNNNNN
    #[default]
    Standard,
    /// SAP-style: 10-digit number (e.g., 4500000001)
    SapStyle,
    /// Oracle-style: PREFIX-ORG-YYYY-NNNNN
    OracleStyle,
    /// NetSuite-style: PREFIX-NNNNN
    NetSuiteStyle,
    /// Random alphanumeric: AAANNNNNNA
    Alphanumeric,
    /// UUID-based short reference
    ShortUuid,
    /// Date-based: YYYYMMDD-NNNN
    DateBased,
    /// Vendor invoice style (external): Various formats
    VendorInvoice,
    /// Bank reference: BANK-DATE-NNNN
    BankReference,
    /// Check number: 6-digit sequential
    CheckNumber,
}

/// Reference style configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceStyle {
    #[default]
    Modern,
    Legacy,
    Erp,
    Simple,
}

/// Enhanced reference generator with multiple format support.
#[derive(Debug)]
pub struct EnhancedReferenceGenerator {
    counters: Mutex<HashMap<(EnhancedReferenceFormat, i32), AtomicU64>>,
    sap_counter: AtomicU64,
    check_counter: AtomicU64,
}

impl Clone for EnhancedReferenceGenerator {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl Default for EnhancedReferenceGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl EnhancedReferenceGenerator {
    /// Create a new reference generator.
    pub fn new() -> Self {
        Self {
            counters: Mutex::new(HashMap::new()),
            sap_counter: AtomicU64::new(4500000001),
            check_counter: AtomicU64::new(100001),
        }
    }

    /// Generate a reference number.
    pub fn generate(
        &self,
        format: EnhancedReferenceFormat,
        year: i32,
        rng: &mut impl Rng,
    ) -> String {
        match format {
            EnhancedReferenceFormat::Standard => self.generate_standard(year),
            EnhancedReferenceFormat::SapStyle => self.generate_sap_style(),
            EnhancedReferenceFormat::OracleStyle => self.generate_oracle_style(year),
            EnhancedReferenceFormat::NetSuiteStyle => self.generate_netsuite_style(),
            EnhancedReferenceFormat::Alphanumeric => self.generate_alphanumeric(rng),
            EnhancedReferenceFormat::ShortUuid => self.generate_short_uuid(rng),
            EnhancedReferenceFormat::DateBased => self.generate_date_based(year, rng),
            EnhancedReferenceFormat::VendorInvoice => self.generate_vendor_invoice(rng),
            EnhancedReferenceFormat::BankReference => self.generate_bank_reference(year, rng),
            EnhancedReferenceFormat::CheckNumber => self.generate_check_number(),
        }
    }

    /// Generate a reference for a specific document type.
    pub fn generate_for_document(
        &self,
        doc_type: DocumentType,
        year: i32,
        _rng: &mut impl Rng,
    ) -> String {
        let prefix = doc_type.prefix();
        let seq = self.next_sequence(EnhancedReferenceFormat::Standard, year);
        format!("{prefix}-{year}-{seq:06}")
    }

    /// Generate an external reference (vendor/bank style).
    pub fn generate_external(&self, rng: &mut impl Rng) -> String {
        self.generate_vendor_invoice(rng)
    }

    fn generate_standard(&self, year: i32) -> String {
        let seq = self.next_sequence(EnhancedReferenceFormat::Standard, year);
        format!("DOC-{year}-{seq:06}")
    }

    fn generate_sap_style(&self) -> String {
        let num = self.sap_counter.fetch_add(1, Ordering::Relaxed);
        format!("{num:010}")
    }

    fn generate_oracle_style(&self, year: i32) -> String {
        let seq = self.next_sequence(EnhancedReferenceFormat::OracleStyle, year);
        format!("ORG1-{year}-{seq:05}")
    }

    fn generate_netsuite_style(&self) -> String {
        let seq = self.next_sequence(EnhancedReferenceFormat::NetSuiteStyle, 0);
        format!("INV{seq:05}")
    }

    fn generate_alphanumeric(&self, rng: &mut impl Rng) -> String {
        let letters: String = (0..3)
            .map(|_| (b'A' + rng.random_range(0..26)) as char)
            .collect();
        let numbers = rng.random_range(100000..999999);
        let check = (b'A' + rng.random_range(0..26)) as char;
        format!("{letters}{numbers:06}{check}")
    }

    fn generate_short_uuid(&self, rng: &mut impl Rng) -> String {
        let chars: String = (0..8)
            .map(|_| {
                let idx = rng.random_range(0..36);
                if idx < 10 {
                    (b'0' + idx) as char
                } else {
                    (b'A' + idx - 10) as char
                }
            })
            .collect();
        chars
    }

    fn generate_date_based(&self, year: i32, rng: &mut impl Rng) -> String {
        let month = rng.random_range(1..=12);
        let day = rng.random_range(1..=28);
        let seq = rng.random_range(1..=9999);
        format!("{year}{month:02}{day:02}-{seq:04}")
    }

    fn generate_vendor_invoice(&self, rng: &mut impl Rng) -> String {
        let style = rng.random_range(0..8);
        match style {
            0 => {
                // INV-NNNNNNNN
                format!("INV-{:08}", rng.random_range(10000000..99999999))
            }
            1 => {
                // Pure numbers
                format!("{:010}", rng.random_range(1000000000u64..9999999999))
            }
            2 => {
                // V-NNN-NNNNNN
                format!(
                    "V{:03}-{:06}",
                    rng.random_range(100..999),
                    rng.random_range(100000..999999)
                )
            }
            3 => {
                // Letter + numbers
                let letter = (b'A' + rng.random_range(0..26)) as char;
                format!("{}{:07}", letter, rng.random_range(1000000..9999999))
            }
            4 => {
                // YYYY-NNNNNN
                let year = rng.random_range(2020..=2025);
                format!("{}-{:06}", year, rng.random_range(1..999999))
            }
            5 => {
                // PO-based
                format!("PO{:08}", rng.random_range(10000000..99999999))
            }
            6 => {
                // Short alphanumeric
                let alpha: String = (0..2)
                    .map(|_| (b'A' + rng.random_range(0..26)) as char)
                    .collect();
                format!("{}{:06}", alpha, rng.random_range(100000..999999))
            }
            _ => {
                // UUID-like
                format!(
                    "{:04X}-{:04X}",
                    rng.random_range(0..0xFFFF),
                    rng.random_range(0..0xFFFF)
                )
            }
        }
    }

    fn generate_bank_reference(&self, year: i32, rng: &mut impl Rng) -> String {
        let month = rng.random_range(1..=12);
        let day = rng.random_range(1..=28);
        let seq = rng.random_range(1..=999999);
        format!("BNK{year}{month:02}{day:02}{seq:06}")
    }

    fn generate_check_number(&self) -> String {
        let num = self.check_counter.fetch_add(1, Ordering::Relaxed);
        format!("{num:06}")
    }

    fn next_sequence(&self, format: EnhancedReferenceFormat, year: i32) -> u64 {
        let mut counters = self.counters.lock().expect("mutex poisoned");
        let counter = counters
            .entry((format, year))
            .or_insert_with(|| AtomicU64::new(1));
        counter.fetch_add(1, Ordering::Relaxed)
    }

    /// Reset all counters (useful for testing).
    pub fn reset(&self) {
        self.counters.lock().expect("mutex poisoned").clear();
        self.sap_counter.store(4500000001, Ordering::Relaxed);
        self.check_counter.store(100001, Ordering::Relaxed);
    }
}

/// Document types for reference generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocumentType {
    Invoice,
    PurchaseOrder,
    SalesOrder,
    GoodsReceipt,
    Payment,
    JournalEntry,
    CreditMemo,
    DebitMemo,
    Delivery,
    Return,
    Adjustment,
    Transfer,
}

impl DocumentType {
    /// Get the standard prefix for this document type.
    pub fn prefix(&self) -> &'static str {
        match self {
            DocumentType::Invoice => "INV",
            DocumentType::PurchaseOrder => "PO",
            DocumentType::SalesOrder => "SO",
            DocumentType::GoodsReceipt => "GR",
            DocumentType::Payment => "PMT",
            DocumentType::JournalEntry => "JE",
            DocumentType::CreditMemo => "CM",
            DocumentType::DebitMemo => "DM",
            DocumentType::Delivery => "DL",
            DocumentType::Return => "RET",
            DocumentType::Adjustment => "ADJ",
            DocumentType::Transfer => "TRF",
        }
    }

    /// Get an alternative SAP-style document type code.
    pub fn sap_code(&self) -> &'static str {
        match self {
            DocumentType::Invoice => "RE",
            DocumentType::PurchaseOrder => "NB",
            DocumentType::SalesOrder => "TA",
            DocumentType::GoodsReceipt => "WE",
            DocumentType::Payment => "ZP",
            DocumentType::JournalEntry => "SA",
            DocumentType::CreditMemo => "KR",
            DocumentType::DebitMemo => "DR",
            DocumentType::Delivery => "LF",
            DocumentType::Return => "AF",
            DocumentType::Adjustment => "AB",
            DocumentType::Transfer => "UE",
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_standard_format() {
        let gen = EnhancedReferenceGenerator::new();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let ref1 = gen.generate(EnhancedReferenceFormat::Standard, 2024, &mut rng);
        assert!(ref1.starts_with("DOC-2024-"));
        assert!(ref1.len() == 15);

        let ref2 = gen.generate(EnhancedReferenceFormat::Standard, 2024, &mut rng);
        assert_ne!(ref1, ref2); // Sequential
    }

    #[test]
    fn test_sap_style_format() {
        let gen = EnhancedReferenceGenerator::new();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let ref1 = gen.generate(EnhancedReferenceFormat::SapStyle, 2024, &mut rng);
        assert!(ref1.len() == 10);
        assert!(ref1.starts_with("4500"));

        let ref2 = gen.generate(EnhancedReferenceFormat::SapStyle, 2024, &mut rng);
        assert_ne!(ref1, ref2);
    }

    #[test]
    fn test_oracle_style_format() {
        let gen = EnhancedReferenceGenerator::new();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let ref1 = gen.generate(EnhancedReferenceFormat::OracleStyle, 2024, &mut rng);
        assert!(ref1.starts_with("ORG1-2024-"));
    }

    #[test]
    fn test_alphanumeric_format() {
        let gen = EnhancedReferenceGenerator::new();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let ref1 = gen.generate(EnhancedReferenceFormat::Alphanumeric, 2024, &mut rng);
        assert!(ref1.len() == 10);
        assert!(ref1.chars().take(3).all(|c| c.is_ascii_uppercase()));
        assert!(ref1.chars().last().unwrap().is_ascii_uppercase());
    }

    #[test]
    fn test_vendor_invoice_variety() {
        let gen = EnhancedReferenceGenerator::new();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let mut formats = std::collections::HashSet::new();
        for _ in 0..100 {
            let ref1 = gen.generate(EnhancedReferenceFormat::VendorInvoice, 2024, &mut rng);
            // Check first 3 chars pattern
            let pattern: String = ref1.chars().take(3).collect();
            formats.insert(pattern);
        }

        // Should have variety in formats
        assert!(formats.len() > 3);
    }

    #[test]
    fn test_document_type_generation() {
        let gen = EnhancedReferenceGenerator::new();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let inv = gen.generate_for_document(DocumentType::Invoice, 2024, &mut rng);
        assert!(inv.starts_with("INV-2024-"));

        let po = gen.generate_for_document(DocumentType::PurchaseOrder, 2024, &mut rng);
        assert!(po.starts_with("PO-2024-"));

        let je = gen.generate_for_document(DocumentType::JournalEntry, 2024, &mut rng);
        assert!(je.starts_with("JE-2024-"));
    }

    #[test]
    fn test_check_number_sequential() {
        let gen = EnhancedReferenceGenerator::new();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let check1 = gen.generate(EnhancedReferenceFormat::CheckNumber, 2024, &mut rng);
        let check2 = gen.generate(EnhancedReferenceFormat::CheckNumber, 2024, &mut rng);
        let check3 = gen.generate(EnhancedReferenceFormat::CheckNumber, 2024, &mut rng);

        // Should be sequential
        let num1: u64 = check1.parse().unwrap();
        let num2: u64 = check2.parse().unwrap();
        let num3: u64 = check3.parse().unwrap();

        assert_eq!(num2, num1 + 1);
        assert_eq!(num3, num2 + 1);
    }

    #[test]
    fn test_document_type_prefixes() {
        assert_eq!(DocumentType::Invoice.prefix(), "INV");
        assert_eq!(DocumentType::PurchaseOrder.prefix(), "PO");
        assert_eq!(DocumentType::JournalEntry.prefix(), "JE");
        assert_eq!(DocumentType::Payment.prefix(), "PMT");
    }

    #[test]
    fn test_sap_codes() {
        assert_eq!(DocumentType::Invoice.sap_code(), "RE");
        assert_eq!(DocumentType::PurchaseOrder.sap_code(), "NB");
        assert_eq!(DocumentType::GoodsReceipt.sap_code(), "WE");
    }

    #[test]
    fn test_reset_counters() {
        let gen = EnhancedReferenceGenerator::new();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let ref1 = gen.generate(EnhancedReferenceFormat::Standard, 2024, &mut rng);
        gen.reset();
        let mut rng2 = ChaCha8Rng::seed_from_u64(42);
        let ref2 = gen.generate(EnhancedReferenceFormat::Standard, 2024, &mut rng2);

        assert_eq!(ref1, ref2);
    }
}
