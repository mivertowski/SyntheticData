//! Enhanced vendor generator with realistic payment behavior and bank accounts.
//!
//! Now integrates with the realism module for sophisticated vendor naming
//! based on spend categories, industry patterns, and well-known brands.

use chrono::NaiveDate;
use datasynth_core::models::{BankAccount, PaymentTerms, Vendor, VendorBehavior, VendorPool};
use datasynth_core::templates::{
    AddressGenerator, AddressRegion, SpendCategory, VendorNameGenerator,
};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

/// Configuration for vendor generation.
#[derive(Debug, Clone)]
pub struct VendorGeneratorConfig {
    /// Distribution of payment terms (terms, probability)
    pub payment_terms_distribution: Vec<(PaymentTerms, f64)>,
    /// Distribution of vendor behaviors (behavior, probability)
    pub behavior_distribution: Vec<(VendorBehavior, f64)>,
    /// Probability of vendor being intercompany
    pub intercompany_rate: f64,
    /// Default country for vendors
    pub default_country: String,
    /// Default currency
    pub default_currency: String,
    /// Generate bank accounts
    pub generate_bank_accounts: bool,
    /// Probability of vendor having multiple bank accounts
    pub multiple_bank_account_rate: f64,
    /// Distribution of spend categories (category, probability)
    pub spend_category_distribution: Vec<(SpendCategory, f64)>,
    /// Primary region for address generation
    pub primary_region: AddressRegion,
    /// Use enhanced realistic naming (via realism module)
    pub use_enhanced_naming: bool,
}

impl Default for VendorGeneratorConfig {
    fn default() -> Self {
        Self {
            payment_terms_distribution: vec![
                (PaymentTerms::Net30, 0.40),
                (PaymentTerms::Net60, 0.20),
                (PaymentTerms::TwoTenNet30, 0.25),
                (PaymentTerms::Net15, 0.10),
                (PaymentTerms::Immediate, 0.05),
            ],
            behavior_distribution: vec![
                (VendorBehavior::Flexible, 0.60),
                (VendorBehavior::Strict, 0.25),
                (VendorBehavior::VeryFlexible, 0.10),
                (VendorBehavior::Aggressive, 0.05),
            ],
            intercompany_rate: 0.05,
            default_country: "US".to_string(),
            default_currency: "USD".to_string(),
            generate_bank_accounts: true,
            multiple_bank_account_rate: 0.20,
            spend_category_distribution: vec![
                (SpendCategory::OfficeSupplies, 0.15),
                (SpendCategory::ITServices, 0.12),
                (SpendCategory::ProfessionalServices, 0.12),
                (SpendCategory::Telecommunications, 0.08),
                (SpendCategory::Utilities, 0.08),
                (SpendCategory::RawMaterials, 0.10),
                (SpendCategory::Logistics, 0.10),
                (SpendCategory::Marketing, 0.08),
                (SpendCategory::Facilities, 0.07),
                (SpendCategory::Staffing, 0.05),
                (SpendCategory::Travel, 0.05),
            ],
            primary_region: AddressRegion::NorthAmerica,
            use_enhanced_naming: true,
        }
    }
}

/// Legacy vendor name templates by category (kept for backward compatibility).
/// New code should use VendorNameGenerator from the realism module.
#[allow(dead_code)]
const VENDOR_NAME_TEMPLATES_LEGACY: &[(&str, &[&str])] = &[
    (
        "Manufacturing",
        &[
            "Global Manufacturing Solutions",
            "Precision Parts Inc.",
            "Industrial Components Ltd.",
            "Advanced Materials Corp.",
        ],
    ),
    (
        "Services",
        &[
            "Professional Services Group",
            "Consulting Partners LLC",
            "Business Solutions Inc.",
            "Technical Services Corp.",
        ],
    ),
    (
        "Technology",
        &[
            "Tech Solutions Inc.",
            "Digital Systems Corp.",
            "Software Innovations LLC",
            "Cloud Services Partners",
        ],
    ),
];

/// Bank name templates.
const BANK_NAMES: &[&str] = &[
    "First National Bank",
    "Commerce Bank",
    "United Banking Corp",
    "Regional Trust Bank",
    "Merchants Bank",
    "Citizens Financial",
    "Pacific Coast Bank",
    "Atlantic Commerce Bank",
    "Midwest Trust Company",
    "Capital One Commercial",
];

/// Generator for vendor master data.
pub struct VendorGenerator {
    rng: ChaCha8Rng,
    seed: u64,
    config: VendorGeneratorConfig,
    vendor_counter: usize,
    /// Enhanced vendor name generator from realism module
    vendor_name_gen: VendorNameGenerator,
    /// Address generator for vendor addresses
    address_gen: AddressGenerator,
}

impl VendorGenerator {
    /// Create a new vendor generator.
    pub fn new(seed: u64) -> Self {
        Self::with_config(seed, VendorGeneratorConfig::default())
    }

    /// Create a new vendor generator with custom configuration.
    pub fn with_config(seed: u64, config: VendorGeneratorConfig) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            seed,
            vendor_name_gen: VendorNameGenerator::new(),
            address_gen: AddressGenerator::for_region(config.primary_region),
            config,
            vendor_counter: 0,
        }
    }

    /// Generate a single vendor.
    pub fn generate_vendor(&mut self, company_code: &str, _effective_date: NaiveDate) -> Vendor {
        self.vendor_counter += 1;

        let vendor_id = format!("V-{:06}", self.vendor_counter);
        let (category, name) = self.select_vendor_name();
        let tax_id = self.generate_tax_id();
        let _address = self.address_gen.generate_commercial(&mut self.rng);

        // Store the spend category for potential future use
        let _spend_category = category;

        let mut vendor = Vendor::new(
            &vendor_id,
            &name,
            datasynth_core::models::VendorType::Supplier,
        );
        vendor.tax_id = Some(tax_id);
        vendor.country = self.config.default_country.clone();
        vendor.currency = self.config.default_currency.clone();
        // Note: category, effective_date, address are not fields on Vendor

        // Set payment terms
        vendor.payment_terms = self.select_payment_terms();

        // Set behavior
        vendor.behavior = self.select_vendor_behavior();

        // Check if intercompany
        if self.rng.gen::<f64>() < self.config.intercompany_rate {
            vendor.is_intercompany = true;
            vendor.intercompany_code = Some(format!("IC-{}", company_code));
        }

        // Generate bank accounts
        if self.config.generate_bank_accounts {
            let bank_account = self.generate_bank_account(&vendor.vendor_id);
            vendor.bank_accounts.push(bank_account);

            if self.rng.gen::<f64>() < self.config.multiple_bank_account_rate {
                let bank_account2 = self.generate_bank_account(&vendor.vendor_id);
                vendor.bank_accounts.push(bank_account2);
            }
        }

        vendor
    }

    /// Generate an intercompany vendor (always intercompany).
    pub fn generate_intercompany_vendor(
        &mut self,
        company_code: &str,
        partner_company_code: &str,
        effective_date: NaiveDate,
    ) -> Vendor {
        let mut vendor = self.generate_vendor(company_code, effective_date);
        vendor.is_intercompany = true;
        vendor.intercompany_code = Some(partner_company_code.to_string());
        vendor.name = format!("{} - IC", partner_company_code);
        vendor.payment_terms = PaymentTerms::Immediate; // IC usually immediate
        vendor
    }

    /// Generate a vendor pool with specified count.
    pub fn generate_vendor_pool(
        &mut self,
        count: usize,
        company_code: &str,
        effective_date: NaiveDate,
    ) -> VendorPool {
        let mut pool = VendorPool::new();

        for _ in 0..count {
            let vendor = self.generate_vendor(company_code, effective_date);
            pool.add_vendor(vendor);
        }

        pool
    }

    /// Generate a vendor pool with intercompany vendors.
    pub fn generate_vendor_pool_with_ic(
        &mut self,
        count: usize,
        company_code: &str,
        partner_company_codes: &[String],
        effective_date: NaiveDate,
    ) -> VendorPool {
        let mut pool = VendorPool::new();

        // Generate regular vendors
        let regular_count = count.saturating_sub(partner_company_codes.len());
        for _ in 0..regular_count {
            let vendor = self.generate_vendor(company_code, effective_date);
            pool.add_vendor(vendor);
        }

        // Generate IC vendors for each partner
        for partner in partner_company_codes {
            let vendor = self.generate_intercompany_vendor(company_code, partner, effective_date);
            pool.add_vendor(vendor);
        }

        pool
    }

    /// Select a spend category based on distribution.
    fn select_spend_category(&mut self) -> SpendCategory {
        let roll: f64 = self.rng.gen();
        let mut cumulative = 0.0;

        for (category, prob) in &self.config.spend_category_distribution {
            cumulative += prob;
            if roll < cumulative {
                return *category;
            }
        }

        SpendCategory::OfficeSupplies
    }

    /// Select a vendor name using the enhanced realism module or legacy templates.
    fn select_vendor_name(&mut self) -> (SpendCategory, String) {
        let category = self.select_spend_category();

        if self.config.use_enhanced_naming {
            // Use the enhanced VendorNameGenerator from the realism module
            let name = self.vendor_name_gen.generate(category, &mut self.rng);
            (category, name)
        } else {
            // Fallback to simple category-based names
            let name = format!("{:?} Vendor {}", category, self.vendor_counter);
            (category, name)
        }
    }

    /// Select payment terms based on distribution.
    fn select_payment_terms(&mut self) -> PaymentTerms {
        let roll: f64 = self.rng.gen();
        let mut cumulative = 0.0;

        for (terms, prob) in &self.config.payment_terms_distribution {
            cumulative += prob;
            if roll < cumulative {
                return *terms;
            }
        }

        PaymentTerms::Net30
    }

    /// Select vendor behavior based on distribution.
    fn select_vendor_behavior(&mut self) -> VendorBehavior {
        let roll: f64 = self.rng.gen();
        let mut cumulative = 0.0;

        for (behavior, prob) in &self.config.behavior_distribution {
            cumulative += prob;
            if roll < cumulative {
                return *behavior;
            }
        }

        VendorBehavior::Flexible
    }

    /// Generate a tax ID.
    fn generate_tax_id(&mut self) -> String {
        format!(
            "{:02}-{:07}",
            self.rng.gen_range(10..99),
            self.rng.gen_range(1000000..9999999)
        )
    }

    /// Generate a bank account.
    fn generate_bank_account(&mut self, vendor_id: &str) -> BankAccount {
        let bank_idx = self.rng.gen_range(0..BANK_NAMES.len());
        let bank_name = BANK_NAMES[bank_idx];

        let routing = format!("{:09}", self.rng.gen_range(100000000u64..999999999));
        let account = format!("{:010}", self.rng.gen_range(1000000000u64..9999999999));

        BankAccount {
            bank_name: bank_name.to_string(),
            bank_country: "US".to_string(),
            account_number: account,
            routing_code: routing,
            holder_name: format!("Vendor {}", vendor_id),
            is_primary: self.vendor_counter == 1,
        }
    }

    /// Generate an address using the enhanced address generator.
    #[allow(dead_code)]
    fn generate_address(&mut self) -> String {
        use datasynth_core::templates::AddressStyle;
        let address = self.address_gen.generate_commercial(&mut self.rng);
        address.format(AddressStyle::SingleLine)
    }

    /// Reset the generator.
    pub fn reset(&mut self) {
        self.rng = ChaCha8Rng::seed_from_u64(self.seed);
        self.vendor_counter = 0;
        self.vendor_name_gen = VendorNameGenerator::new();
        self.address_gen = AddressGenerator::for_region(self.config.primary_region);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vendor_generation() {
        let mut gen = VendorGenerator::new(42);
        let vendor = gen.generate_vendor("1000", NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

        assert!(!vendor.vendor_id.is_empty());
        assert!(!vendor.name.is_empty());
        assert!(vendor.tax_id.is_some());
        assert!(!vendor.bank_accounts.is_empty());
    }

    #[test]
    fn test_vendor_pool_generation() {
        let mut gen = VendorGenerator::new(42);
        let pool =
            gen.generate_vendor_pool(10, "1000", NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

        assert_eq!(pool.vendors.len(), 10);
    }

    #[test]
    fn test_intercompany_vendor() {
        let mut gen = VendorGenerator::new(42);
        let vendor = gen.generate_intercompany_vendor(
            "1000",
            "2000",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        assert!(vendor.is_intercompany);
        assert_eq!(vendor.intercompany_code, Some("2000".to_string()));
    }

    #[test]
    fn test_deterministic_generation() {
        let mut gen1 = VendorGenerator::new(42);
        let mut gen2 = VendorGenerator::new(42);

        let vendor1 = gen1.generate_vendor("1000", NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        let vendor2 = gen2.generate_vendor("1000", NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

        assert_eq!(vendor1.vendor_id, vendor2.vendor_id);
        assert_eq!(vendor1.name, vendor2.name);
    }

    #[test]
    fn test_vendor_pool_with_ic() {
        // Use config with 0 intercompany_rate to test explicit IC vendors only
        let config = VendorGeneratorConfig {
            intercompany_rate: 0.0,
            ..Default::default()
        };
        let mut gen = VendorGenerator::with_config(42, config);
        let pool = gen.generate_vendor_pool_with_ic(
            10,
            "1000",
            &["2000".to_string(), "3000".to_string()],
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        assert_eq!(pool.vendors.len(), 10);

        let ic_vendors: Vec<_> = pool.vendors.iter().filter(|v| v.is_intercompany).collect();
        assert_eq!(ic_vendors.len(), 2);
    }

    #[test]
    fn test_enhanced_vendor_names() {
        let mut gen = VendorGenerator::new(42);
        let vendor = gen.generate_vendor("1000", NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

        // Enhanced naming should produce more varied, realistic names
        assert!(!vendor.name.is_empty());
        // Should not be a simple generic name format
        assert!(!vendor.name.starts_with("Vendor "));
    }
}
