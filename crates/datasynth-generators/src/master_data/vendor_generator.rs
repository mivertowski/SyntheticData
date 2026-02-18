//! Enhanced vendor generator with realistic payment behavior and bank accounts.
//!
//! Now integrates with the realism module for sophisticated vendor naming
//! based on spend categories, industry patterns, and well-known brands.
//!
//! Also supports multi-tier vendor network generation with:
//! - Supply chain tiers (Tier 1, 2, 3)
//! - Vendor clustering (Reliable, Standard, Transactional, Problematic)
//! - Strategic importance and spend tier classification
//! - Concentration analysis and dependency tracking

use chrono::NaiveDate;
use datasynth_core::models::{
    BankAccount, DeclineReason, PaymentHistory, PaymentTerms, SpendTier, StrategicLevel,
    Substitutability, SupplyChainTier, Vendor, VendorBehavior, VendorCluster, VendorDependency,
    VendorLifecycleStage, VendorNetwork, VendorPool, VendorQualityScore, VendorRelationship,
    VendorRelationshipType,
};
use datasynth_core::templates::{
    AddressGenerator, AddressRegion, SpendCategory, VendorNameGenerator,
};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use tracing::debug;

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

/// Configuration for vendor network generation.
#[derive(Debug, Clone)]
pub struct VendorNetworkConfig {
    /// Enable vendor network generation
    pub enabled: bool,
    /// Maximum depth of supply chain tiers (1-3)
    pub depth: u8,
    /// Number of Tier 1 vendors to generate
    pub tier1_count: TierCountConfig,
    /// Number of Tier 2 vendors per Tier 1 parent
    pub tier2_per_parent: TierCountConfig,
    /// Number of Tier 3 vendors per Tier 2 parent
    pub tier3_per_parent: TierCountConfig,
    /// Cluster distribution
    pub cluster_distribution: ClusterDistribution,
    /// Concentration limits
    pub concentration_limits: ConcentrationLimits,
    /// Strategic level distribution
    pub strategic_distribution: Vec<(StrategicLevel, f64)>,
    /// Single-source percentage
    pub single_source_percent: f64,
}

/// Count range for tier generation.
#[derive(Debug, Clone)]
pub struct TierCountConfig {
    /// Minimum count
    pub min: usize,
    /// Maximum count
    pub max: usize,
}

impl TierCountConfig {
    /// Create a new tier count config.
    pub fn new(min: usize, max: usize) -> Self {
        Self { min, max }
    }

    /// Sample a count from the range.
    pub fn sample(&self, rng: &mut impl Rng) -> usize {
        rng.gen_range(self.min..=self.max)
    }
}

/// Distribution of vendor clusters.
#[derive(Debug, Clone)]
pub struct ClusterDistribution {
    /// Reliable strategic vendors (default: 20%)
    pub reliable_strategic: f64,
    /// Standard operational vendors (default: 50%)
    pub standard_operational: f64,
    /// Transactional vendors (default: 25%)
    pub transactional: f64,
    /// Problematic vendors (default: 5%)
    pub problematic: f64,
}

impl Default for ClusterDistribution {
    fn default() -> Self {
        Self {
            reliable_strategic: 0.20,
            standard_operational: 0.50,
            transactional: 0.25,
            problematic: 0.05,
        }
    }
}

impl ClusterDistribution {
    /// Validate that distribution sums to 1.0.
    pub fn validate(&self) -> Result<(), String> {
        let sum = self.reliable_strategic
            + self.standard_operational
            + self.transactional
            + self.problematic;
        if (sum - 1.0).abs() > 0.01 {
            Err(format!("Cluster distribution must sum to 1.0, got {}", sum))
        } else {
            Ok(())
        }
    }

    /// Select a cluster based on the distribution.
    pub fn select(&self, roll: f64) -> VendorCluster {
        let mut cumulative = 0.0;

        cumulative += self.reliable_strategic;
        if roll < cumulative {
            return VendorCluster::ReliableStrategic;
        }

        cumulative += self.standard_operational;
        if roll < cumulative {
            return VendorCluster::StandardOperational;
        }

        cumulative += self.transactional;
        if roll < cumulative {
            return VendorCluster::Transactional;
        }

        VendorCluster::Problematic
    }
}

/// Concentration limits for vendor spend.
#[derive(Debug, Clone)]
pub struct ConcentrationLimits {
    /// Maximum concentration for a single vendor (default: 15%)
    pub max_single_vendor: f64,
    /// Maximum concentration for top 5 vendors (default: 45%)
    pub max_top5: f64,
}

impl Default for ConcentrationLimits {
    fn default() -> Self {
        Self {
            max_single_vendor: 0.15,
            max_top5: 0.45,
        }
    }
}

impl Default for VendorNetworkConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            depth: 3,
            tier1_count: TierCountConfig::new(50, 100),
            tier2_per_parent: TierCountConfig::new(4, 10),
            tier3_per_parent: TierCountConfig::new(2, 5),
            cluster_distribution: ClusterDistribution::default(),
            concentration_limits: ConcentrationLimits::default(),
            strategic_distribution: vec![
                (StrategicLevel::Critical, 0.05),
                (StrategicLevel::Important, 0.15),
                (StrategicLevel::Standard, 0.50),
                (StrategicLevel::Transactional, 0.30),
            ],
            single_source_percent: 0.05,
        }
    }
}

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
    /// Network configuration
    network_config: VendorNetworkConfig,
    /// Optional country pack for locale-aware generation
    country_pack: Option<datasynth_core::CountryPack>,
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
            network_config: VendorNetworkConfig::default(),
            country_pack: None,
        }
    }

    /// Create a new vendor generator with network configuration.
    pub fn with_network_config(
        seed: u64,
        config: VendorGeneratorConfig,
        network_config: VendorNetworkConfig,
    ) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            seed,
            vendor_name_gen: VendorNameGenerator::new(),
            address_gen: AddressGenerator::for_region(config.primary_region),
            config,
            vendor_counter: 0,
            network_config,
            country_pack: None,
        }
    }

    /// Set network configuration.
    pub fn set_network_config(&mut self, network_config: VendorNetworkConfig) {
        self.network_config = network_config;
    }

    /// Set the country pack for locale-aware generation.
    pub fn set_country_pack(&mut self, pack: datasynth_core::CountryPack) {
        self.country_pack = Some(pack);
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
        debug!(count, company_code, %effective_date, "Generating vendor pool");
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

    /// Reset the generator.
    pub fn reset(&mut self) {
        self.rng = ChaCha8Rng::seed_from_u64(self.seed);
        self.vendor_counter = 0;
        self.vendor_name_gen = VendorNameGenerator::new();
        self.address_gen = AddressGenerator::for_region(self.config.primary_region);
    }

    // ===== Vendor Network Generation =====

    /// Generate a complete vendor network with tiered supply chain.
    pub fn generate_vendor_network(
        &mut self,
        company_code: &str,
        effective_date: NaiveDate,
        total_annual_spend: Decimal,
    ) -> VendorNetwork {
        let mut network = VendorNetwork::new(company_code);
        network.created_date = Some(effective_date);

        if !self.network_config.enabled {
            return network;
        }

        // Generate Tier 1 vendors
        let tier1_count = self.network_config.tier1_count.sample(&mut self.rng);
        let tier1_ids = self.generate_tier_vendors(
            company_code,
            effective_date,
            tier1_count,
            SupplyChainTier::Tier1,
            None,
            &mut network,
        );

        // Generate Tier 2 vendors (if depth >= 2)
        if self.network_config.depth >= 2 {
            for tier1_id in &tier1_ids {
                let tier2_count = self.network_config.tier2_per_parent.sample(&mut self.rng);
                let tier2_ids = self.generate_tier_vendors(
                    company_code,
                    effective_date,
                    tier2_count,
                    SupplyChainTier::Tier2,
                    Some(tier1_id.clone()),
                    &mut network,
                );

                // Update Tier 1 vendor's children
                if let Some(rel) = network.get_relationship_mut(tier1_id) {
                    rel.child_vendors = tier2_ids.clone();
                }

                // Generate Tier 3 vendors (if depth >= 3)
                if self.network_config.depth >= 3 {
                    for tier2_id in &tier2_ids {
                        let tier3_count =
                            self.network_config.tier3_per_parent.sample(&mut self.rng);
                        let tier3_ids = self.generate_tier_vendors(
                            company_code,
                            effective_date,
                            tier3_count,
                            SupplyChainTier::Tier3,
                            Some(tier2_id.clone()),
                            &mut network,
                        );

                        // Update Tier 2 vendor's children
                        if let Some(rel) = network.get_relationship_mut(tier2_id) {
                            rel.child_vendors = tier3_ids;
                        }
                    }
                }
            }
        }

        // Assign annual spend to vendors
        self.assign_annual_spend(&mut network, total_annual_spend);

        // Calculate network statistics
        network.calculate_statistics(effective_date);

        network
    }

    /// Generate vendors for a specific tier.
    fn generate_tier_vendors(
        &mut self,
        company_code: &str,
        effective_date: NaiveDate,
        count: usize,
        tier: SupplyChainTier,
        parent_id: Option<String>,
        network: &mut VendorNetwork,
    ) -> Vec<String> {
        let mut vendor_ids = Vec::with_capacity(count);

        for _ in 0..count {
            // Generate base vendor
            let vendor = self.generate_vendor(company_code, effective_date);
            let vendor_id = vendor.vendor_id.clone();

            // Create relationship
            let mut relationship = VendorRelationship::new(
                vendor_id.clone(),
                self.select_relationship_type(),
                tier,
                self.generate_relationship_start_date(effective_date),
            );

            // Set parent if applicable
            if let Some(ref parent) = parent_id {
                relationship = relationship.with_parent(parent.clone());
            }

            // Assign cluster, strategic level, and spend tier
            relationship = relationship
                .with_cluster(self.select_cluster())
                .with_strategic_importance(self.select_strategic_level())
                .with_spend_tier(self.select_spend_tier());

            // Set lifecycle stage
            relationship.lifecycle_stage = self.generate_lifecycle_stage(effective_date);

            // Initialize quality score based on cluster
            relationship.quality_score = self.generate_quality_score(&relationship.cluster);

            // Initialize payment history based on cluster
            relationship.payment_history = self.generate_payment_history(&relationship.cluster);

            // Generate dependency analysis for Tier 1 vendors
            if tier == SupplyChainTier::Tier1 {
                relationship.dependency = Some(self.generate_dependency(&vendor_id, &vendor.name));
            }

            network.add_relationship(relationship);
            vendor_ids.push(vendor_id);
        }

        vendor_ids
    }

    /// Select a relationship type.
    fn select_relationship_type(&mut self) -> VendorRelationshipType {
        let roll: f64 = self.rng.gen();
        if roll < 0.40 {
            VendorRelationshipType::DirectSupplier
        } else if roll < 0.55 {
            VendorRelationshipType::ServiceProvider
        } else if roll < 0.70 {
            VendorRelationshipType::RawMaterialSupplier
        } else if roll < 0.80 {
            VendorRelationshipType::Manufacturer
        } else if roll < 0.88 {
            VendorRelationshipType::Distributor
        } else if roll < 0.94 {
            VendorRelationshipType::Contractor
        } else {
            VendorRelationshipType::OemPartner
        }
    }

    /// Select a cluster based on distribution.
    fn select_cluster(&mut self) -> VendorCluster {
        let roll: f64 = self.rng.gen();
        self.network_config.cluster_distribution.select(roll)
    }

    /// Select a strategic level based on distribution.
    fn select_strategic_level(&mut self) -> StrategicLevel {
        let roll: f64 = self.rng.gen();
        let mut cumulative = 0.0;

        for (level, prob) in &self.network_config.strategic_distribution {
            cumulative += prob;
            if roll < cumulative {
                return *level;
            }
        }

        StrategicLevel::Standard
    }

    /// Select a spend tier.
    fn select_spend_tier(&mut self) -> SpendTier {
        let roll: f64 = self.rng.gen();
        if roll < 0.05 {
            SpendTier::Platinum
        } else if roll < 0.20 {
            SpendTier::Gold
        } else if roll < 0.50 {
            SpendTier::Silver
        } else {
            SpendTier::Bronze
        }
    }

    /// Generate a relationship start date (before effective date).
    fn generate_relationship_start_date(&mut self, effective_date: NaiveDate) -> NaiveDate {
        let days_back: i64 = self.rng.gen_range(90..3650); // 3 months to 10 years
        effective_date - chrono::Duration::days(days_back)
    }

    /// Generate lifecycle stage based on probabilities.
    fn generate_lifecycle_stage(&mut self, effective_date: NaiveDate) -> VendorLifecycleStage {
        let roll: f64 = self.rng.gen();
        if roll < 0.05 {
            VendorLifecycleStage::Onboarding {
                started: effective_date - chrono::Duration::days(self.rng.gen_range(1..60)),
                expected_completion: effective_date
                    + chrono::Duration::days(self.rng.gen_range(30..90)),
            }
        } else if roll < 0.12 {
            VendorLifecycleStage::RampUp {
                started: effective_date - chrono::Duration::days(self.rng.gen_range(60..180)),
                target_volume_percent: self.rng.gen_range(50..80) as u8,
            }
        } else if roll < 0.85 {
            VendorLifecycleStage::SteadyState {
                since: effective_date - chrono::Duration::days(self.rng.gen_range(180..1825)),
            }
        } else if roll < 0.95 {
            VendorLifecycleStage::Decline {
                started: effective_date - chrono::Duration::days(self.rng.gen_range(30..180)),
                reason: DeclineReason::QualityIssues,
            }
        } else {
            VendorLifecycleStage::SteadyState {
                since: effective_date - chrono::Duration::days(self.rng.gen_range(365..1825)),
            }
        }
    }

    /// Generate quality score based on cluster.
    fn generate_quality_score(&mut self, cluster: &VendorCluster) -> VendorQualityScore {
        let (base_delivery, base_quality, base_invoice, base_response) = match cluster {
            VendorCluster::ReliableStrategic => (0.97, 0.96, 0.98, 0.95),
            VendorCluster::StandardOperational => (0.92, 0.90, 0.93, 0.85),
            VendorCluster::Transactional => (0.85, 0.82, 0.88, 0.75),
            VendorCluster::Problematic => (0.70, 0.68, 0.75, 0.60),
        };

        // Add some variance
        let delivery_variance: f64 = self.rng.gen_range(-0.05..0.05);
        let quality_variance: f64 = self.rng.gen_range(-0.05..0.05);
        let invoice_variance: f64 = self.rng.gen_range(-0.05..0.05);
        let response_variance: f64 = self.rng.gen_range(-0.05..0.05);

        VendorQualityScore {
            delivery_score: (base_delivery + delivery_variance).clamp(0.0_f64, 1.0_f64),
            quality_score: (base_quality + quality_variance).clamp(0.0_f64, 1.0_f64),
            invoice_accuracy_score: (base_invoice + invoice_variance).clamp(0.0_f64, 1.0_f64),
            responsiveness_score: (base_response + response_variance).clamp(0.0_f64, 1.0_f64),
            last_evaluation: NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid default date"),
            evaluation_count: self.rng.gen_range(1..20),
        }
    }

    /// Generate payment history based on cluster.
    fn generate_payment_history(&mut self, cluster: &VendorCluster) -> PaymentHistory {
        let total = self.rng.gen_range(10..200) as u32;
        let on_time_rate = cluster.invoice_accuracy_probability();
        let on_time = (total as f64 * on_time_rate) as u32;
        let early = (total as f64 * self.rng.gen_range(0.05..0.20)) as u32;
        let late = total.saturating_sub(on_time).saturating_sub(early);

        PaymentHistory {
            total_invoices: total,
            on_time_payments: on_time,
            early_payments: early,
            late_payments: late,
            total_amount: Decimal::from(total) * Decimal::from(self.rng.gen_range(1000..50000)),
            average_days_to_pay: self.rng.gen_range(20.0..45.0),
            last_payment_date: None,
            total_discounts: Decimal::from(early) * Decimal::from(self.rng.gen_range(50..500)),
        }
    }

    /// Generate vendor dependency analysis.
    fn generate_dependency(&mut self, vendor_id: &str, vendor_name: &str) -> VendorDependency {
        let is_single_source = self.rng.gen::<f64>() < self.network_config.single_source_percent;

        let substitutability = {
            let roll: f64 = self.rng.gen();
            if roll < 0.60 {
                Substitutability::Easy
            } else if roll < 0.90 {
                Substitutability::Moderate
            } else {
                Substitutability::Difficult
            }
        };

        let mut dep = VendorDependency::new(vendor_id, self.infer_spend_category(vendor_name));
        dep.is_single_source = is_single_source;
        dep.substitutability = substitutability;
        dep.concentration_percent = self.rng.gen_range(0.01..0.20);

        // Generate alternative vendors if not single source
        if !is_single_source {
            let alt_count = self.rng.gen_range(1..4);
            for i in 0..alt_count {
                dep.alternatives.push(format!("ALT-{}-{:03}", vendor_id, i));
            }
        }

        dep
    }

    /// Infer spend category from vendor name (simplified).
    fn infer_spend_category(&self, _vendor_name: &str) -> String {
        "General".to_string()
    }

    /// Assign annual spend to vendors based on Pareto principle.
    fn assign_annual_spend(&mut self, network: &mut VendorNetwork, total_spend: Decimal) {
        let tier1_count = network.tier1_vendors.len();
        if tier1_count == 0 {
            return;
        }

        // Generate Pareto-distributed weights
        let mut weights: Vec<f64> = (0..tier1_count)
            .map(|_| {
                // Pareto distribution with alpha = 1.5
                let u: f64 = self.rng.gen_range(0.01..1.0);
                u.powf(-1.0 / 1.5)
            })
            .collect();

        let total_weight: f64 = weights.iter().sum();
        for w in &mut weights {
            *w /= total_weight;
        }

        // Sort to ensure highest weights get assigned to first vendors
        weights.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

        // Assign spend to Tier 1 vendors
        for (idx, vendor_id) in network.tier1_vendors.clone().iter().enumerate() {
            if let Some(rel) = network.get_relationship_mut(vendor_id) {
                let weight = weights.get(idx).copied().unwrap_or(0.01);
                let spend = total_spend * Decimal::from_f64_retain(weight).unwrap_or(Decimal::ZERO);
                rel.annual_spend = spend;

                // Update spend tier based on actual spend
                if let Some(dep) = &mut rel.dependency {
                    dep.concentration_percent = weight;
                }
            }
        }

        // Assign smaller amounts to Tier 2/3 (they supply to Tier 1, not directly)
        let tier2_avg_spend = total_spend / Decimal::from(network.tier2_vendors.len().max(1))
            * Decimal::from_f64_retain(0.05).unwrap_or(Decimal::ZERO);
        for vendor_id in &network.tier2_vendors.clone() {
            if let Some(rel) = network.get_relationship_mut(vendor_id) {
                rel.annual_spend = tier2_avg_spend
                    * Decimal::from_f64_retain(self.rng.gen_range(0.5..1.5))
                        .unwrap_or(Decimal::ONE);
            }
        }

        let tier3_avg_spend = total_spend / Decimal::from(network.tier3_vendors.len().max(1))
            * Decimal::from_f64_retain(0.01).unwrap_or(Decimal::ZERO);
        for vendor_id in &network.tier3_vendors.clone() {
            if let Some(rel) = network.get_relationship_mut(vendor_id) {
                rel.annual_spend = tier3_avg_spend
                    * Decimal::from_f64_retain(self.rng.gen_range(0.5..1.5))
                        .unwrap_or(Decimal::ONE);
            }
        }
    }

    /// Generate a vendor pool with network relationships.
    pub fn generate_vendor_pool_with_network(
        &mut self,
        company_code: &str,
        effective_date: NaiveDate,
        total_annual_spend: Decimal,
    ) -> (VendorPool, VendorNetwork) {
        let network =
            self.generate_vendor_network(company_code, effective_date, total_annual_spend);

        // Create VendorPool from network relationships
        let mut pool = VendorPool::new();
        for _vendor_id in network
            .tier1_vendors
            .iter()
            .chain(network.tier2_vendors.iter())
            .chain(network.tier3_vendors.iter())
        {
            // Generate a basic vendor for each relationship
            // In practice, you'd want to store the full Vendor in the relationship
            let vendor = self.generate_vendor(company_code, effective_date);
            pool.add_vendor(vendor);
        }

        (pool, network)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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

    // ===== Vendor Network Tests =====

    #[test]
    fn test_vendor_network_generation() {
        let network_config = VendorNetworkConfig {
            enabled: true,
            depth: 2,
            tier1_count: TierCountConfig::new(5, 10),
            tier2_per_parent: TierCountConfig::new(2, 4),
            tier3_per_parent: TierCountConfig::new(1, 2),
            ..Default::default()
        };

        let mut gen = VendorGenerator::with_network_config(
            42,
            VendorGeneratorConfig::default(),
            network_config,
        );

        let network = gen.generate_vendor_network(
            "1000",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            Decimal::from(10_000_000),
        );

        assert!(!network.tier1_vendors.is_empty());
        assert!(!network.tier2_vendors.is_empty());
        assert!(network.tier1_vendors.len() >= 5);
        assert!(network.tier1_vendors.len() <= 10);
    }

    #[test]
    fn test_vendor_network_relationships() {
        let network_config = VendorNetworkConfig {
            enabled: true,
            depth: 2,
            tier1_count: TierCountConfig::new(3, 3),
            tier2_per_parent: TierCountConfig::new(2, 2),
            ..Default::default()
        };

        let mut gen = VendorGenerator::with_network_config(
            42,
            VendorGeneratorConfig::default(),
            network_config,
        );

        let network = gen.generate_vendor_network(
            "1000",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            Decimal::from(5_000_000),
        );

        // Check that Tier 2 vendors have parents
        for tier2_id in &network.tier2_vendors {
            let rel = network.get_relationship(tier2_id).unwrap();
            assert!(rel.parent_vendor.is_some());
            assert_eq!(rel.tier, SupplyChainTier::Tier2);
        }

        // Check that Tier 1 vendors have children
        for tier1_id in &network.tier1_vendors {
            let rel = network.get_relationship(tier1_id).unwrap();
            assert!(!rel.child_vendors.is_empty());
            assert_eq!(rel.tier, SupplyChainTier::Tier1);
        }
    }

    #[test]
    fn test_vendor_network_spend_distribution() {
        let network_config = VendorNetworkConfig {
            enabled: true,
            depth: 1,
            tier1_count: TierCountConfig::new(10, 10),
            ..Default::default()
        };

        let mut gen = VendorGenerator::with_network_config(
            42,
            VendorGeneratorConfig::default(),
            network_config,
        );

        let total_spend = Decimal::from(10_000_000);
        let network = gen.generate_vendor_network(
            "1000",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            total_spend,
        );

        // Check that spend is distributed
        let total_assigned: Decimal = network.relationships.values().map(|r| r.annual_spend).sum();

        assert!(total_assigned > Decimal::ZERO);
    }

    #[test]
    fn test_vendor_network_cluster_distribution() {
        let network_config = VendorNetworkConfig {
            enabled: true,
            depth: 1,
            tier1_count: TierCountConfig::new(100, 100),
            cluster_distribution: ClusterDistribution::default(),
            ..Default::default()
        };

        let mut gen = VendorGenerator::with_network_config(
            42,
            VendorGeneratorConfig::default(),
            network_config,
        );

        let network = gen.generate_vendor_network(
            "1000",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            Decimal::from(10_000_000),
        );

        // Count clusters
        let mut cluster_counts: std::collections::HashMap<VendorCluster, usize> =
            std::collections::HashMap::new();
        for rel in network.relationships.values() {
            *cluster_counts.entry(rel.cluster).or_insert(0) += 1;
        }

        // ReliableStrategic should be roughly 20%
        let reliable = cluster_counts
            .get(&VendorCluster::ReliableStrategic)
            .unwrap_or(&0);
        assert!(*reliable >= 10 && *reliable <= 35);
    }

    #[test]
    fn test_cluster_distribution_validation() {
        let valid = ClusterDistribution::default();
        assert!(valid.validate().is_ok());

        let invalid = ClusterDistribution {
            reliable_strategic: 0.5,
            standard_operational: 0.5,
            transactional: 0.5,
            problematic: 0.5,
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_vendor_network_disabled() {
        let network_config = VendorNetworkConfig {
            enabled: false,
            ..Default::default()
        };

        let mut gen = VendorGenerator::with_network_config(
            42,
            VendorGeneratorConfig::default(),
            network_config,
        );

        let network = gen.generate_vendor_network(
            "1000",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            Decimal::from(10_000_000),
        );

        assert!(network.tier1_vendors.is_empty());
        assert!(network.relationships.is_empty());
    }
}
