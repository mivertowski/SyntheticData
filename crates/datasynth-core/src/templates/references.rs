//! Reference number generation for journal entries.
//!
//! Generates realistic document reference numbers like invoice numbers,
//! purchase orders, sales orders, etc.

use crate::models::BusinessProcess;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Types of reference numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceType {
    /// Invoice number (vendor or customer)
    Invoice,
    /// Purchase order number
    PurchaseOrder,
    /// Sales order number
    SalesOrder,
    /// Goods receipt number
    GoodsReceipt,
    /// Payment reference
    PaymentReference,
    /// Asset tag number
    AssetTag,
    /// Project number
    ProjectNumber,
    /// Expense report number
    ExpenseReport,
    /// Contract number
    ContractNumber,
    /// Batch number
    BatchNumber,
    /// Internal document number
    InternalDocument,
}

impl ReferenceType {
    /// Get the default prefix for this reference type.
    pub fn default_prefix(&self) -> &'static str {
        match self {
            Self::Invoice => "INV",
            Self::PurchaseOrder => "PO",
            Self::SalesOrder => "SO",
            Self::GoodsReceipt => "GR",
            Self::PaymentReference => "PAY",
            Self::AssetTag => "FA",
            Self::ProjectNumber => "PRJ",
            Self::ExpenseReport => "EXP",
            Self::ContractNumber => "CTR",
            Self::BatchNumber => "BATCH",
            Self::InternalDocument => "DOC",
        }
    }

    /// Get the typical reference type for a business process.
    pub fn for_business_process(process: BusinessProcess) -> Self {
        match process {
            BusinessProcess::O2C => Self::SalesOrder,
            BusinessProcess::P2P => Self::PurchaseOrder,
            BusinessProcess::R2R => Self::InternalDocument,
            BusinessProcess::H2R => Self::ExpenseReport,
            BusinessProcess::A2R => Self::AssetTag,
            BusinessProcess::S2C => Self::PurchaseOrder,
            BusinessProcess::Mfg => Self::InternalDocument,
            BusinessProcess::Bank => Self::PaymentReference,
            BusinessProcess::Audit => Self::InternalDocument,
            BusinessProcess::Treasury => Self::PaymentReference,
            BusinessProcess::Tax => Self::InternalDocument,
            BusinessProcess::Intercompany => Self::InternalDocument,
            BusinessProcess::ProjectAccounting => Self::ProjectNumber,
            BusinessProcess::Esg => Self::InternalDocument,
        }
    }
}

/// Format for reference numbers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum ReferenceFormat {
    /// Simple sequential: PREFIX-000001
    Sequential,
    /// Year-prefixed: PREFIX-YYYY-000001
    #[default]
    YearPrefixed,
    /// Year-month: PREFIX-YYYYMM-00001
    YearMonthPrefixed,
    /// Random alphanumeric: PREFIX-XXXXXX
    Random,
    /// Company-year: PREFIX-COMP-YYYY-00001
    CompanyYearPrefixed,
}

/// Configuration for a reference type.
#[derive(Debug, Clone)]
pub struct ReferenceConfig {
    /// Prefix for the reference
    pub prefix: String,
    /// Format to use
    pub format: ReferenceFormat,
    /// Number of digits in the sequence
    pub sequence_digits: usize,
    /// Starting sequence number
    pub start_sequence: u64,
}

impl Default for ReferenceConfig {
    fn default() -> Self {
        Self {
            prefix: "REF".to_string(),
            format: ReferenceFormat::YearPrefixed,
            sequence_digits: 6,
            start_sequence: 1,
        }
    }
}

/// Generator for reference numbers.
#[derive(Debug)]
pub struct ReferenceGenerator {
    /// Configuration by reference type
    configs: HashMap<ReferenceType, ReferenceConfig>,
    /// Counters by reference type and year
    counters: HashMap<(ReferenceType, Option<i32>), AtomicU64>,
    /// Default year for generation
    default_year: i32,
    /// Company code for company-prefixed formats
    company_code: String,
}

impl Default for ReferenceGenerator {
    fn default() -> Self {
        Self::new(2024, "1000")
    }
}

impl ReferenceGenerator {
    /// Create a new reference generator.
    pub fn new(year: i32, company_code: &str) -> Self {
        let mut configs = HashMap::new();

        // Set up default configurations for each type
        for ref_type in [
            ReferenceType::Invoice,
            ReferenceType::PurchaseOrder,
            ReferenceType::SalesOrder,
            ReferenceType::GoodsReceipt,
            ReferenceType::PaymentReference,
            ReferenceType::AssetTag,
            ReferenceType::ProjectNumber,
            ReferenceType::ExpenseReport,
            ReferenceType::ContractNumber,
            ReferenceType::BatchNumber,
            ReferenceType::InternalDocument,
        ] {
            configs.insert(
                ref_type,
                ReferenceConfig {
                    prefix: ref_type.default_prefix().to_string(),
                    format: ReferenceFormat::YearPrefixed,
                    sequence_digits: 6,
                    start_sequence: 1,
                },
            );
        }

        Self {
            configs,
            counters: HashMap::new(),
            default_year: year,
            company_code: company_code.to_string(),
        }
    }

    /// Set the company code.
    pub fn with_company_code(mut self, code: &str) -> Self {
        self.company_code = code.to_string();
        self
    }

    /// Set the default year.
    pub fn with_year(mut self, year: i32) -> Self {
        self.default_year = year;
        self
    }

    /// Set configuration for a reference type.
    pub fn set_config(&mut self, ref_type: ReferenceType, config: ReferenceConfig) {
        self.configs.insert(ref_type, config);
    }

    /// Set a custom prefix for a reference type.
    pub fn set_prefix(&mut self, ref_type: ReferenceType, prefix: &str) {
        if let Some(config) = self.configs.get_mut(&ref_type) {
            config.prefix = prefix.to_string();
        }
    }

    /// Get the next sequence number for a reference type and optional year.
    fn next_sequence(&mut self, ref_type: ReferenceType, year: Option<i32>) -> u64 {
        let key = (ref_type, year);
        let config = self.configs.get(&ref_type).cloned().unwrap_or_default();

        let counter = self
            .counters
            .entry(key)
            .or_insert_with(|| AtomicU64::new(config.start_sequence));

        counter.fetch_add(1, Ordering::SeqCst)
    }

    /// Generate a reference number.
    pub fn generate(&mut self, ref_type: ReferenceType) -> String {
        self.generate_for_year(ref_type, self.default_year)
    }

    /// Generate a reference number for a specific year.
    pub fn generate_for_year(&mut self, ref_type: ReferenceType, year: i32) -> String {
        let config = self.configs.get(&ref_type).cloned().unwrap_or_default();
        let seq = self.next_sequence(ref_type, Some(year));

        match config.format {
            ReferenceFormat::Sequential => {
                format!(
                    "{}-{:0width$}",
                    config.prefix,
                    seq,
                    width = config.sequence_digits
                )
            }
            ReferenceFormat::YearPrefixed => {
                format!(
                    "{}-{}-{:0width$}",
                    config.prefix,
                    year,
                    seq,
                    width = config.sequence_digits
                )
            }
            ReferenceFormat::YearMonthPrefixed => {
                // Use a default month; in practice, pass month as parameter
                format!(
                    "{}-{}01-{:0width$}",
                    config.prefix,
                    year,
                    seq,
                    width = config.sequence_digits - 1
                )
            }
            ReferenceFormat::Random => {
                // Generate random alphanumeric suffix
                let suffix: String = (0..config.sequence_digits)
                    .map(|_| {
                        let idx = rand::thread_rng().gen_range(0..36);
                        if idx < 10 {
                            (b'0' + idx) as char
                        } else {
                            (b'A' + idx - 10) as char
                        }
                    })
                    .collect();
                format!("{}-{}", config.prefix, suffix)
            }
            ReferenceFormat::CompanyYearPrefixed => {
                format!(
                    "{}-{}-{}-{:0width$}",
                    config.prefix,
                    self.company_code,
                    year,
                    seq,
                    width = config.sequence_digits
                )
            }
        }
    }

    /// Generate a reference for a business process.
    pub fn generate_for_process(&mut self, process: BusinessProcess) -> String {
        let ref_type = ReferenceType::for_business_process(process);
        self.generate(ref_type)
    }

    /// Generate a reference for a business process and year.
    pub fn generate_for_process_year(&mut self, process: BusinessProcess, year: i32) -> String {
        let ref_type = ReferenceType::for_business_process(process);
        self.generate_for_year(ref_type, year)
    }

    /// Generate an external reference (vendor invoice, etc.) with random elements.
    pub fn generate_external_reference(&self, rng: &mut impl Rng) -> String {
        // External references often have different formats
        let formats = [
            // Vendor invoice formats
            |rng: &mut dyn rand::RngCore| format!("INV{:08}", rng.gen_range(10000000u64..99999999)),
            |rng: &mut dyn rand::RngCore| {
                format!("{:010}", rng.gen_range(1000000000u64..9999999999))
            },
            |rng: &mut dyn rand::RngCore| {
                format!(
                    "V{}-{:06}",
                    rng.gen_range(100..999),
                    rng.gen_range(1..999999)
                )
            },
            |rng: &mut dyn rand::RngCore| {
                format!(
                    "{}{:07}",
                    (b'A' + rng.gen_range(0..26)) as char,
                    rng.gen_range(1000000..9999999)
                )
            },
        ];

        let idx = rng.gen_range(0..formats.len());
        formats[idx](rng)
    }
}

/// Builder for configuring reference generation.
#[derive(Debug, Clone, Default)]
pub struct ReferenceGeneratorBuilder {
    year: Option<i32>,
    company_code: Option<String>,
    invoice_prefix: Option<String>,
    po_prefix: Option<String>,
    so_prefix: Option<String>,
}

impl ReferenceGeneratorBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the year.
    pub fn year(mut self, year: i32) -> Self {
        self.year = Some(year);
        self
    }

    /// Set the company code.
    pub fn company_code(mut self, code: &str) -> Self {
        self.company_code = Some(code.to_string());
        self
    }

    /// Set invoice prefix.
    pub fn invoice_prefix(mut self, prefix: &str) -> Self {
        self.invoice_prefix = Some(prefix.to_string());
        self
    }

    /// Set PO prefix.
    pub fn po_prefix(mut self, prefix: &str) -> Self {
        self.po_prefix = Some(prefix.to_string());
        self
    }

    /// Set SO prefix.
    pub fn so_prefix(mut self, prefix: &str) -> Self {
        self.so_prefix = Some(prefix.to_string());
        self
    }

    /// Build the generator.
    pub fn build(self) -> ReferenceGenerator {
        let year = self.year.unwrap_or(2024);
        let company = self.company_code.as_deref().unwrap_or("1000");

        let mut gen = ReferenceGenerator::new(year, company);

        if let Some(prefix) = self.invoice_prefix {
            gen.set_prefix(ReferenceType::Invoice, &prefix);
        }
        if let Some(prefix) = self.po_prefix {
            gen.set_prefix(ReferenceType::PurchaseOrder, &prefix);
        }
        if let Some(prefix) = self.so_prefix {
            gen.set_prefix(ReferenceType::SalesOrder, &prefix);
        }

        gen
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_sequential_generation() {
        let mut gen = ReferenceGenerator::new(2024, "1000");

        let ref1 = gen.generate(ReferenceType::Invoice);
        let ref2 = gen.generate(ReferenceType::Invoice);
        let ref3 = gen.generate(ReferenceType::Invoice);

        assert!(ref1.starts_with("INV-2024-"));
        assert!(ref2.starts_with("INV-2024-"));
        assert!(ref3.starts_with("INV-2024-"));

        // Should be sequential
        assert_ne!(ref1, ref2);
        assert_ne!(ref2, ref3);
    }

    #[test]
    fn test_different_types() {
        let mut gen = ReferenceGenerator::new(2024, "1000");

        let inv = gen.generate(ReferenceType::Invoice);
        let po = gen.generate(ReferenceType::PurchaseOrder);
        let so = gen.generate(ReferenceType::SalesOrder);

        assert!(inv.starts_with("INV-"));
        assert!(po.starts_with("PO-"));
        assert!(so.starts_with("SO-"));
    }

    #[test]
    fn test_year_based_counters() {
        let mut gen = ReferenceGenerator::new(2024, "1000");

        let ref_2024 = gen.generate_for_year(ReferenceType::Invoice, 2024);
        let ref_2025 = gen.generate_for_year(ReferenceType::Invoice, 2025);

        assert!(ref_2024.contains("2024"));
        assert!(ref_2025.contains("2025"));

        // Different years should have independent counters
        assert!(ref_2024.ends_with("000001"));
        assert!(ref_2025.ends_with("000001"));
    }

    #[test]
    fn test_business_process_mapping() {
        let mut gen = ReferenceGenerator::new(2024, "1000");

        let o2c_ref = gen.generate_for_process(BusinessProcess::O2C);
        let p2p_ref = gen.generate_for_process(BusinessProcess::P2P);

        assert!(o2c_ref.starts_with("SO-")); // Sales Order
        assert!(p2p_ref.starts_with("PO-")); // Purchase Order
    }

    #[test]
    fn test_custom_prefix() {
        let mut gen = ReferenceGenerator::new(2024, "ACME");
        gen.set_prefix(ReferenceType::Invoice, "ACME-INV");

        let inv = gen.generate(ReferenceType::Invoice);
        assert!(inv.starts_with("ACME-INV-"));
    }

    #[test]
    fn test_builder() {
        let mut gen = ReferenceGeneratorBuilder::new()
            .year(2025)
            .company_code("CORP")
            .invoice_prefix("CORP-INV")
            .build();

        let inv = gen.generate(ReferenceType::Invoice);
        assert!(inv.starts_with("CORP-INV-2025-"));
    }

    #[test]
    fn test_external_reference() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;

        let gen = ReferenceGenerator::new(2024, "1000");
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let ext_ref = gen.generate_external_reference(&mut rng);
        assert!(!ext_ref.is_empty());
    }
}
