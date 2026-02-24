//! Enhanced customer generator with credit management and payment behavior.
//!
//! Also supports customer segmentation with:
//! - Value segments (Enterprise, Mid-Market, SMB, Consumer)
//! - Customer lifecycle stages
//! - Referral networks and corporate hierarchies
//! - Engagement metrics and churn analysis

use chrono::NaiveDate;
use datasynth_core::models::{
    ChurnReason, CreditRating, Customer, CustomerEngagement, CustomerLifecycleStage,
    CustomerPaymentBehavior, CustomerPool, CustomerValueSegment, PaymentTerms, RiskTrigger,
    SegmentedCustomer, SegmentedCustomerPool,
};
use datasynth_core::utils::seeded_rng;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use tracing::debug;

/// Configuration for customer generation.
#[derive(Debug, Clone)]
pub struct CustomerGeneratorConfig {
    /// Distribution of credit ratings (rating, probability)
    pub credit_rating_distribution: Vec<(CreditRating, f64)>,
    /// Distribution of payment behaviors (behavior, probability)
    pub payment_behavior_distribution: Vec<(CustomerPaymentBehavior, f64)>,
    /// Distribution of payment terms (terms, probability)
    pub payment_terms_distribution: Vec<(PaymentTerms, f64)>,
    /// Probability of customer being intercompany
    pub intercompany_rate: f64,
    /// Default country for customers
    pub default_country: String,
    /// Default currency
    pub default_currency: String,
    /// Credit limit ranges by rating (min, max)
    pub credit_limits: Vec<(CreditRating, Decimal, Decimal)>,
}

impl Default for CustomerGeneratorConfig {
    fn default() -> Self {
        Self {
            credit_rating_distribution: vec![
                (CreditRating::AAA, 0.05),
                (CreditRating::AA, 0.10),
                (CreditRating::A, 0.25),
                (CreditRating::BBB, 0.30),
                (CreditRating::BB, 0.15),
                (CreditRating::B, 0.10),
                (CreditRating::CCC, 0.04),
                (CreditRating::D, 0.01),
            ],
            payment_behavior_distribution: vec![
                (CustomerPaymentBehavior::EarlyPayer, 0.15),
                (CustomerPaymentBehavior::OnTime, 0.45),
                (CustomerPaymentBehavior::SlightlyLate, 0.25),
                (CustomerPaymentBehavior::OftenLate, 0.10),
                (CustomerPaymentBehavior::HighRisk, 0.05),
            ],
            payment_terms_distribution: vec![
                (PaymentTerms::Net30, 0.50),
                (PaymentTerms::Net60, 0.20),
                (PaymentTerms::TwoTenNet30, 0.20),
                (PaymentTerms::Net15, 0.05),
                (PaymentTerms::Immediate, 0.05),
            ],
            intercompany_rate: 0.05,
            default_country: "US".to_string(),
            default_currency: "USD".to_string(),
            credit_limits: vec![
                (
                    CreditRating::AAA,
                    Decimal::from(1_000_000),
                    Decimal::from(10_000_000),
                ),
                (
                    CreditRating::AA,
                    Decimal::from(500_000),
                    Decimal::from(2_000_000),
                ),
                (
                    CreditRating::A,
                    Decimal::from(250_000),
                    Decimal::from(1_000_000),
                ),
                (
                    CreditRating::BBB,
                    Decimal::from(100_000),
                    Decimal::from(500_000),
                ),
                (
                    CreditRating::BB,
                    Decimal::from(50_000),
                    Decimal::from(250_000),
                ),
                (
                    CreditRating::B,
                    Decimal::from(25_000),
                    Decimal::from(100_000),
                ),
                (
                    CreditRating::CCC,
                    Decimal::from(10_000),
                    Decimal::from(50_000),
                ),
                (CreditRating::D, Decimal::from(0), Decimal::from(10_000)),
            ],
        }
    }
}

/// Configuration for customer segmentation.
#[derive(Debug, Clone)]
pub struct CustomerSegmentationConfig {
    /// Enable customer segmentation
    pub enabled: bool,
    /// Value segment distribution
    pub segment_distribution: SegmentDistribution,
    /// Lifecycle stage distribution
    pub lifecycle_distribution: LifecycleDistribution,
    /// Referral network configuration
    pub referral_config: ReferralConfig,
    /// Corporate hierarchy configuration
    pub hierarchy_config: HierarchyConfig,
    /// Industry distribution
    pub industry_distribution: Vec<(String, f64)>,
}

impl Default for CustomerSegmentationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            segment_distribution: SegmentDistribution::default(),
            lifecycle_distribution: LifecycleDistribution::default(),
            referral_config: ReferralConfig::default(),
            hierarchy_config: HierarchyConfig::default(),
            industry_distribution: vec![
                ("Technology".to_string(), 0.20),
                ("Manufacturing".to_string(), 0.15),
                ("Retail".to_string(), 0.15),
                ("Healthcare".to_string(), 0.12),
                ("Financial".to_string(), 0.12),
                ("Energy".to_string(), 0.08),
                ("Transportation".to_string(), 0.08),
                ("Construction".to_string(), 0.10),
            ],
        }
    }
}

/// Distribution of customer value segments.
#[derive(Debug, Clone)]
pub struct SegmentDistribution {
    /// Enterprise segment (customer share)
    pub enterprise: f64,
    /// Mid-market segment (customer share)
    pub mid_market: f64,
    /// SMB segment (customer share)
    pub smb: f64,
    /// Consumer segment (customer share)
    pub consumer: f64,
}

impl Default for SegmentDistribution {
    fn default() -> Self {
        Self {
            enterprise: 0.05,
            mid_market: 0.20,
            smb: 0.50,
            consumer: 0.25,
        }
    }
}

impl SegmentDistribution {
    /// Validate that distribution sums to 1.0.
    pub fn validate(&self) -> Result<(), String> {
        let sum = self.enterprise + self.mid_market + self.smb + self.consumer;
        if (sum - 1.0).abs() > 0.01 {
            Err(format!("Segment distribution must sum to 1.0, got {}", sum))
        } else {
            Ok(())
        }
    }

    /// Select a segment based on the distribution.
    pub fn select(&self, roll: f64) -> CustomerValueSegment {
        let mut cumulative = 0.0;

        cumulative += self.enterprise;
        if roll < cumulative {
            return CustomerValueSegment::Enterprise;
        }

        cumulative += self.mid_market;
        if roll < cumulative {
            return CustomerValueSegment::MidMarket;
        }

        cumulative += self.smb;
        if roll < cumulative {
            return CustomerValueSegment::Smb;
        }

        CustomerValueSegment::Consumer
    }
}

/// Distribution of lifecycle stages.
#[derive(Debug, Clone)]
pub struct LifecycleDistribution {
    /// Prospect rate
    pub prospect: f64,
    /// New customer rate
    pub new: f64,
    /// Growth stage rate
    pub growth: f64,
    /// Mature stage rate
    pub mature: f64,
    /// At-risk rate
    pub at_risk: f64,
    /// Churned rate
    pub churned: f64,
}

impl Default for LifecycleDistribution {
    fn default() -> Self {
        Self {
            prospect: 0.0, // Prospects not typically in active pool
            new: 0.10,
            growth: 0.15,
            mature: 0.60,
            at_risk: 0.10,
            churned: 0.05,
        }
    }
}

impl LifecycleDistribution {
    /// Validate that distribution sums to 1.0.
    pub fn validate(&self) -> Result<(), String> {
        let sum =
            self.prospect + self.new + self.growth + self.mature + self.at_risk + self.churned;
        if (sum - 1.0).abs() > 0.01 {
            Err(format!(
                "Lifecycle distribution must sum to 1.0, got {}",
                sum
            ))
        } else {
            Ok(())
        }
    }
}

/// Configuration for referral networks.
#[derive(Debug, Clone)]
pub struct ReferralConfig {
    /// Enable referral generation
    pub enabled: bool,
    /// Rate of customers acquired via referral
    pub referral_rate: f64,
    /// Maximum referrals per customer
    pub max_referrals_per_customer: usize,
}

impl Default for ReferralConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            referral_rate: 0.15,
            max_referrals_per_customer: 5,
        }
    }
}

/// Configuration for corporate hierarchies.
#[derive(Debug, Clone)]
pub struct HierarchyConfig {
    /// Enable corporate hierarchy generation
    pub enabled: bool,
    /// Rate of customers in hierarchies
    pub hierarchy_rate: f64,
    /// Maximum hierarchy depth
    pub max_depth: usize,
    /// Rate of billing consolidation
    pub billing_consolidation_rate: f64,
}

impl Default for HierarchyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            hierarchy_rate: 0.30,
            max_depth: 3,
            billing_consolidation_rate: 0.50,
        }
    }
}

/// Customer name templates by industry.
const CUSTOMER_NAME_TEMPLATES: &[(&str, &[&str])] = &[
    (
        "Retail",
        &[
            "Consumer Goods Corp.",
            "Retail Solutions Inc.",
            "Shop Direct Ltd.",
            "Market Leaders LLC",
            "Consumer Brands Group",
            "Retail Partners Co.",
            "Shopping Networks Inc.",
            "Direct Sales Corp.",
        ],
    ),
    (
        "Manufacturing",
        &[
            "Industrial Manufacturing Inc.",
            "Production Systems Corp.",
            "Assembly Technologies LLC",
            "Manufacturing Partners Group",
            "Factory Solutions Ltd.",
            "Production Line Inc.",
            "Industrial Works Corp.",
            "Manufacturing Excellence Co.",
        ],
    ),
    (
        "Healthcare",
        &[
            "Healthcare Systems Inc.",
            "Medical Solutions Corp.",
            "Health Partners LLC",
            "Medical Equipment Group",
            "Healthcare Providers Ltd.",
            "Clinical Services Inc.",
            "Health Networks Corp.",
            "Medical Supplies Co.",
        ],
    ),
    (
        "Technology",
        &[
            "Tech Innovations Inc.",
            "Digital Solutions Corp.",
            "Software Systems LLC",
            "Technology Partners Group",
            "IT Solutions Ltd.",
            "Tech Enterprises Inc.",
            "Digital Networks Corp.",
            "Innovation Labs Co.",
        ],
    ),
    (
        "Financial",
        &[
            "Financial Services Inc.",
            "Banking Solutions Corp.",
            "Investment Partners LLC",
            "Financial Networks Group",
            "Capital Services Ltd.",
            "Banking Partners Inc.",
            "Finance Solutions Corp.",
            "Investment Group Co.",
        ],
    ),
    (
        "Energy",
        &[
            "Energy Solutions Inc.",
            "Power Systems Corp.",
            "Renewable Partners LLC",
            "Energy Networks Group",
            "Utility Services Ltd.",
            "Power Generation Inc.",
            "Energy Partners Corp.",
            "Sustainable Energy Co.",
        ],
    ),
    (
        "Transportation",
        &[
            "Transport Solutions Inc.",
            "Logistics Systems Corp.",
            "Freight Partners LLC",
            "Transportation Networks Group",
            "Shipping Services Ltd.",
            "Fleet Management Inc.",
            "Logistics Partners Corp.",
            "Transport Dynamics Co.",
        ],
    ),
    (
        "Construction",
        &[
            "Construction Solutions Inc.",
            "Building Systems Corp.",
            "Development Partners LLC",
            "Construction Group Ltd.",
            "Building Services Inc.",
            "Property Development Corp.",
            "Construction Partners Co.",
            "Infrastructure Systems LLC",
        ],
    ),
];

/// Generator for customer master data.
pub struct CustomerGenerator {
    rng: ChaCha8Rng,
    seed: u64,
    config: CustomerGeneratorConfig,
    customer_counter: usize,
    /// Segmentation configuration
    segmentation_config: CustomerSegmentationConfig,
    /// Optional country pack for locale-aware generation
    country_pack: Option<datasynth_core::CountryPack>,
}

impl CustomerGenerator {
    /// Create a new customer generator.
    pub fn new(seed: u64) -> Self {
        Self::with_config(seed, CustomerGeneratorConfig::default())
    }

    /// Create a new customer generator with custom configuration.
    pub fn with_config(seed: u64, config: CustomerGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            seed,
            config,
            customer_counter: 0,
            segmentation_config: CustomerSegmentationConfig::default(),
            country_pack: None,
        }
    }

    /// Create a new customer generator with segmentation configuration.
    pub fn with_segmentation_config(
        seed: u64,
        config: CustomerGeneratorConfig,
        segmentation_config: CustomerSegmentationConfig,
    ) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            seed,
            config,
            customer_counter: 0,
            segmentation_config,
            country_pack: None,
        }
    }

    /// Set segmentation configuration.
    pub fn set_segmentation_config(&mut self, segmentation_config: CustomerSegmentationConfig) {
        self.segmentation_config = segmentation_config;
    }

    /// Set the country pack for locale-aware generation.
    pub fn set_country_pack(&mut self, pack: datasynth_core::CountryPack) {
        self.country_pack = Some(pack);
    }

    /// Generate a single customer.
    pub fn generate_customer(
        &mut self,
        company_code: &str,
        _effective_date: NaiveDate,
    ) -> Customer {
        self.customer_counter += 1;

        let customer_id = format!("C-{:06}", self.customer_counter);
        let (_industry, name) = self.select_customer_name();

        let mut customer = Customer::new(
            &customer_id,
            name,
            datasynth_core::models::CustomerType::Corporate,
        );

        customer.country = self.config.default_country.clone();
        customer.currency = self.config.default_currency.clone();
        // Note: industry and effective_date are not fields on Customer

        // Set credit rating and limit
        customer.credit_rating = self.select_credit_rating();
        customer.credit_limit = self.generate_credit_limit(&customer.credit_rating);

        // Set payment behavior
        customer.payment_behavior = self.select_payment_behavior();

        // Set payment terms
        customer.payment_terms = self.select_payment_terms();

        // Check if intercompany
        if self.rng.gen::<f64>() < self.config.intercompany_rate {
            customer.is_intercompany = true;
            customer.intercompany_code = Some(format!("IC-{}", company_code));
        }

        // Note: address, contact_name, contact_email are not fields on Customer

        customer
    }

    /// Generate an intercompany customer (always intercompany).
    pub fn generate_intercompany_customer(
        &mut self,
        company_code: &str,
        partner_company_code: &str,
        effective_date: NaiveDate,
    ) -> Customer {
        let mut customer = self.generate_customer(company_code, effective_date);
        customer.is_intercompany = true;
        customer.intercompany_code = Some(partner_company_code.to_string());
        customer.name = format!("{} - IC", partner_company_code);
        customer.credit_rating = CreditRating::AAA; // IC always highest rating
        customer.credit_limit = Decimal::from(100_000_000); // High limit for IC
        customer.payment_behavior = CustomerPaymentBehavior::OnTime;
        customer
    }

    /// Generate a customer with specific credit profile.
    pub fn generate_customer_with_credit(
        &mut self,
        company_code: &str,
        credit_rating: CreditRating,
        credit_limit: Decimal,
        effective_date: NaiveDate,
    ) -> Customer {
        let mut customer = self.generate_customer(company_code, effective_date);
        customer.credit_rating = credit_rating;
        customer.credit_limit = credit_limit;

        // Adjust payment behavior based on credit rating
        customer.payment_behavior = match credit_rating {
            CreditRating::AAA | CreditRating::AA => {
                if self.rng.gen::<f64>() < 0.7 {
                    CustomerPaymentBehavior::EarlyPayer
                } else {
                    CustomerPaymentBehavior::OnTime
                }
            }
            CreditRating::A | CreditRating::BBB => CustomerPaymentBehavior::OnTime,
            CreditRating::BB | CreditRating::B => CustomerPaymentBehavior::SlightlyLate,
            CreditRating::CCC | CreditRating::CC => CustomerPaymentBehavior::OftenLate,
            CreditRating::C | CreditRating::D => CustomerPaymentBehavior::HighRisk,
        };

        customer
    }

    /// Generate a customer pool with specified count.
    /// Uses counter-based name selection for variety and appends customer ID when a duplicate would occur.
    pub fn generate_customer_pool(
        &mut self,
        count: usize,
        company_code: &str,
        effective_date: NaiveDate,
    ) -> CustomerPool {
        debug!(count, company_code, %effective_date, "Generating customer pool");
        let mut pool = CustomerPool::new();
        let mut used_names = std::collections::HashSet::new();

        for _ in 0..count {
            let mut customer = self.generate_customer(company_code, effective_date);
            let name = std::mem::take(&mut customer.name);
            let unique_name =
                Self::dedupe_customer_name(&name, &customer.customer_id, &mut used_names);
            customer.name = unique_name;
            pool.add_customer(customer);
        }

        pool
    }

    /// Generate a customer pool with intercompany customers.
    /// Ensures unique customer names by appending customer ID when a duplicate would occur.
    pub fn generate_customer_pool_with_ic(
        &mut self,
        count: usize,
        company_code: &str,
        partner_company_codes: &[String],
        effective_date: NaiveDate,
    ) -> CustomerPool {
        let mut pool = CustomerPool::new();
        let mut used_names = std::collections::HashSet::new();

        let regular_count = count.saturating_sub(partner_company_codes.len());
        for _ in 0..regular_count {
            let mut customer = self.generate_customer(company_code, effective_date);
            let name = std::mem::take(&mut customer.name);
            let unique_name =
                Self::dedupe_customer_name(&name, &customer.customer_id, &mut used_names);
            customer.name = unique_name;
            pool.add_customer(customer);
        }

        for partner in partner_company_codes {
            let mut customer =
                self.generate_intercompany_customer(company_code, partner, effective_date);
            let name = std::mem::take(&mut customer.name);
            let unique_name =
                Self::dedupe_customer_name(&name, &customer.customer_id, &mut used_names);
            customer.name = unique_name;
            pool.add_customer(customer);
        }

        pool
    }

    /// Generate a diverse customer pool with various credit profiles.
    /// Ensures unique customer names by appending customer ID when a duplicate would occur.
    pub fn generate_diverse_pool(
        &mut self,
        count: usize,
        company_code: &str,
        effective_date: NaiveDate,
    ) -> CustomerPool {
        let mut pool = CustomerPool::new();
        let mut used_names = std::collections::HashSet::new();

        let rating_counts = [
            (CreditRating::AAA, (count as f64 * 0.05) as usize),
            (CreditRating::AA, (count as f64 * 0.10) as usize),
            (CreditRating::A, (count as f64 * 0.20) as usize),
            (CreditRating::BBB, (count as f64 * 0.30) as usize),
            (CreditRating::BB, (count as f64 * 0.15) as usize),
            (CreditRating::B, (count as f64 * 0.10) as usize),
            (CreditRating::CCC, (count as f64 * 0.07) as usize),
            (CreditRating::D, (count as f64 * 0.03) as usize),
        ];

        for (rating, rating_count) in rating_counts {
            for _ in 0..rating_count {
                let credit_limit = self.generate_credit_limit(&rating);
                let mut customer = self.generate_customer_with_credit(
                    company_code,
                    rating,
                    credit_limit,
                    effective_date,
                );
                let name = std::mem::take(&mut customer.name);
                let unique_name =
                    Self::dedupe_customer_name(&name, &customer.customer_id, &mut used_names);
                customer.name = unique_name;
                pool.add_customer(customer);
            }
        }

        while pool.customers.len() < count {
            let mut customer = self.generate_customer(company_code, effective_date);
            let name = std::mem::take(&mut customer.name);
            let unique_name =
                Self::dedupe_customer_name(&name, &customer.customer_id, &mut used_names);
            customer.name = unique_name;
            pool.add_customer(customer);
        }

        pool
    }

    /// Total number of distinct customer names across all industries (for cycling).
    fn total_customer_name_slots() -> usize {
        CUSTOMER_NAME_TEMPLATES
            .iter()
            .map(|(_, names)| names.len())
            .sum()
    }

    /// Select a customer name from templates. Uses customer_counter to cycle through all names
    /// so the first N customers get unique names (N = total names across industries).
    fn select_customer_name(&mut self) -> (&'static str, &'static str) {
        let total = Self::total_customer_name_slots();
        let idx = (self.customer_counter - 1) % total;
        let mut remaining = idx;
        for (industry, names) in CUSTOMER_NAME_TEMPLATES {
            if remaining < names.len() {
                return (industry, names[remaining]);
            }
            remaining -= names.len();
        }
        (
            CUSTOMER_NAME_TEMPLATES[0].0,
            CUSTOMER_NAME_TEMPLATES[0].1[0],
        )
    }

    /// Return a unique name: if `name` is already in `used_names`, append ` (id)` so it is unique.
    fn dedupe_customer_name(
        name: &str,
        id: &str,
        used_names: &mut std::collections::HashSet<String>,
    ) -> String {
        let candidate = if used_names.contains(name) {
            format!("{} ({})", name, id)
        } else {
            name.to_string()
        };
        used_names.insert(candidate.clone());
        candidate
    }

    /// Select credit rating based on distribution.
    fn select_credit_rating(&mut self) -> CreditRating {
        let roll: f64 = self.rng.gen();
        let mut cumulative = 0.0;

        for (rating, prob) in &self.config.credit_rating_distribution {
            cumulative += prob;
            if roll < cumulative {
                return *rating;
            }
        }

        CreditRating::BBB
    }

    /// Generate credit limit for rating.
    fn generate_credit_limit(&mut self, rating: &CreditRating) -> Decimal {
        for (r, min, max) in &self.config.credit_limits {
            if r == rating {
                let range = (*max - *min).to_string().parse::<f64>().unwrap_or(0.0);
                let offset = Decimal::from_f64_retain(self.rng.gen::<f64>() * range)
                    .unwrap_or(Decimal::ZERO);
                return *min + offset;
            }
        }

        Decimal::from(100_000)
    }

    /// Select payment behavior based on distribution.
    fn select_payment_behavior(&mut self) -> CustomerPaymentBehavior {
        let roll: f64 = self.rng.gen();
        let mut cumulative = 0.0;

        for (behavior, prob) in &self.config.payment_behavior_distribution {
            cumulative += prob;
            if roll < cumulative {
                return *behavior;
            }
        }

        CustomerPaymentBehavior::OnTime
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

    /// Reset the generator.
    pub fn reset(&mut self) {
        self.rng = seeded_rng(self.seed, 0);
        self.customer_counter = 0;
    }

    // ===== Customer Segmentation Generation =====

    /// Generate a segmented customer pool with value segments, lifecycle stages, and networks.
    pub fn generate_segmented_pool(
        &mut self,
        count: usize,
        company_code: &str,
        effective_date: NaiveDate,
        total_annual_revenue: Decimal,
    ) -> SegmentedCustomerPool {
        let mut pool = SegmentedCustomerPool::new();

        if !self.segmentation_config.enabled {
            return pool;
        }

        // Calculate counts by segment
        let segment_counts = self.calculate_segment_counts(count);

        // Generate customers by segment
        let mut all_customer_ids: Vec<String> = Vec::new();
        let mut parent_candidates: Vec<String> = Vec::new();

        for (segment, segment_count) in segment_counts {
            for _ in 0..segment_count {
                let customer = self.generate_customer(company_code, effective_date);
                let customer_id = customer.customer_id.clone();

                let mut segmented =
                    self.create_segmented_customer(&customer, segment, effective_date);

                // Assign industry
                segmented.industry = Some(self.select_industry());

                // Assign annual contract value based on segment
                segmented.annual_contract_value =
                    self.generate_acv(segment, total_annual_revenue, count);

                // Enterprise customers are candidates for parent relationships
                if segment == CustomerValueSegment::Enterprise {
                    parent_candidates.push(customer_id.clone());
                }

                all_customer_ids.push(customer_id);
                pool.add_customer(segmented);
            }
        }

        // Build referral networks
        if self.segmentation_config.referral_config.enabled {
            self.build_referral_networks(&mut pool, &all_customer_ids);
        }

        // Build corporate hierarchies
        if self.segmentation_config.hierarchy_config.enabled {
            self.build_corporate_hierarchies(&mut pool, &all_customer_ids, &parent_candidates);
        }

        // Calculate engagement metrics and churn risk
        self.populate_engagement_metrics(&mut pool, effective_date);

        // Calculate statistics
        pool.calculate_statistics();

        pool
    }

    /// Calculate customer counts by segment.
    fn calculate_segment_counts(
        &mut self,
        total_count: usize,
    ) -> Vec<(CustomerValueSegment, usize)> {
        let dist = &self.segmentation_config.segment_distribution;
        vec![
            (
                CustomerValueSegment::Enterprise,
                (total_count as f64 * dist.enterprise) as usize,
            ),
            (
                CustomerValueSegment::MidMarket,
                (total_count as f64 * dist.mid_market) as usize,
            ),
            (
                CustomerValueSegment::Smb,
                (total_count as f64 * dist.smb) as usize,
            ),
            (
                CustomerValueSegment::Consumer,
                (total_count as f64 * dist.consumer) as usize,
            ),
        ]
    }

    /// Create a segmented customer from a base customer.
    fn create_segmented_customer(
        &mut self,
        customer: &Customer,
        segment: CustomerValueSegment,
        effective_date: NaiveDate,
    ) -> SegmentedCustomer {
        let lifecycle_stage = self.generate_lifecycle_stage(effective_date);

        SegmentedCustomer::new(
            &customer.customer_id,
            &customer.name,
            segment,
            effective_date,
        )
        .with_lifecycle_stage(lifecycle_stage)
    }

    /// Generate lifecycle stage based on distribution.
    fn generate_lifecycle_stage(&mut self, effective_date: NaiveDate) -> CustomerLifecycleStage {
        let dist = &self.segmentation_config.lifecycle_distribution;
        let roll: f64 = self.rng.gen();
        let mut cumulative = 0.0;

        cumulative += dist.prospect;
        if roll < cumulative {
            return CustomerLifecycleStage::Prospect {
                conversion_probability: self.rng.gen_range(0.1..0.4),
                source: Some("Marketing".to_string()),
                first_contact_date: effective_date
                    - chrono::Duration::days(self.rng.gen_range(1..90)),
            };
        }

        cumulative += dist.new;
        if roll < cumulative {
            return CustomerLifecycleStage::New {
                first_order_date: effective_date
                    - chrono::Duration::days(self.rng.gen_range(1..90)),
                onboarding_complete: self.rng.gen::<f64>() > 0.3,
            };
        }

        cumulative += dist.growth;
        if roll < cumulative {
            return CustomerLifecycleStage::Growth {
                since: effective_date - chrono::Duration::days(self.rng.gen_range(90..365)),
                growth_rate: self.rng.gen_range(0.10..0.50),
            };
        }

        cumulative += dist.mature;
        if roll < cumulative {
            return CustomerLifecycleStage::Mature {
                stable_since: effective_date
                    - chrono::Duration::days(self.rng.gen_range(365..1825)),
                avg_annual_spend: Decimal::from(self.rng.gen_range(10000..500000)),
            };
        }

        cumulative += dist.at_risk;
        if roll < cumulative {
            let triggers = self.generate_risk_triggers();
            return CustomerLifecycleStage::AtRisk {
                triggers,
                flagged_date: effective_date - chrono::Duration::days(self.rng.gen_range(7..60)),
                churn_probability: self.rng.gen_range(0.3..0.8),
            };
        }

        // Churned
        CustomerLifecycleStage::Churned {
            last_activity: effective_date - chrono::Duration::days(self.rng.gen_range(90..365)),
            win_back_probability: self.rng.gen_range(0.05..0.25),
            reason: Some(self.generate_churn_reason()),
        }
    }

    /// Generate risk triggers for at-risk customers.
    fn generate_risk_triggers(&mut self) -> Vec<RiskTrigger> {
        let all_triggers = [
            RiskTrigger::DecliningOrderFrequency,
            RiskTrigger::DecliningOrderValue,
            RiskTrigger::PaymentIssues,
            RiskTrigger::Complaints,
            RiskTrigger::ReducedEngagement,
            RiskTrigger::ContractExpiring,
        ];

        let count = self.rng.gen_range(1..=3);
        let mut triggers = Vec::new();

        for _ in 0..count {
            let idx = self.rng.gen_range(0..all_triggers.len());
            triggers.push(all_triggers[idx].clone());
        }

        triggers
    }

    /// Generate churn reason.
    fn generate_churn_reason(&mut self) -> ChurnReason {
        let roll: f64 = self.rng.gen();
        if roll < 0.30 {
            ChurnReason::Competitor
        } else if roll < 0.50 {
            ChurnReason::Price
        } else if roll < 0.65 {
            ChurnReason::ServiceQuality
        } else if roll < 0.75 {
            ChurnReason::BudgetConstraints
        } else if roll < 0.85 {
            ChurnReason::ProductFit
        } else if roll < 0.92 {
            ChurnReason::Consolidation
        } else {
            ChurnReason::Unknown
        }
    }

    /// Select an industry based on distribution.
    fn select_industry(&mut self) -> String {
        let roll: f64 = self.rng.gen();
        let mut cumulative = 0.0;

        for (industry, prob) in &self.segmentation_config.industry_distribution {
            cumulative += prob;
            if roll < cumulative {
                return industry.clone();
            }
        }

        "Other".to_string()
    }

    /// Generate annual contract value based on segment.
    fn generate_acv(
        &mut self,
        segment: CustomerValueSegment,
        total_revenue: Decimal,
        total_customers: usize,
    ) -> Decimal {
        // Calculate expected revenue per customer in this segment
        let segment_revenue_share = segment.revenue_share();
        let segment_customer_share = segment.customer_share();
        let expected_customers_in_segment =
            (total_customers as f64 * segment_customer_share) as usize;
        let segment_total_revenue = total_revenue
            * Decimal::from_f64_retain(segment_revenue_share).unwrap_or(Decimal::ZERO);

        let avg_acv = if expected_customers_in_segment > 0 {
            segment_total_revenue / Decimal::from(expected_customers_in_segment)
        } else {
            Decimal::from(10000)
        };

        // Add variance (±50%)
        let variance = self.rng.gen_range(0.5..1.5);
        avg_acv * Decimal::from_f64_retain(variance).unwrap_or(Decimal::ONE)
    }

    /// Build referral networks among customers.
    fn build_referral_networks(
        &mut self,
        pool: &mut SegmentedCustomerPool,
        customer_ids: &[String],
    ) {
        let referral_rate = self.segmentation_config.referral_config.referral_rate;
        let max_referrals = self
            .segmentation_config
            .referral_config
            .max_referrals_per_customer;

        // Track referral counts per customer
        let mut referral_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        // Create customer ID to index mapping
        let id_to_idx: std::collections::HashMap<String, usize> = customer_ids
            .iter()
            .enumerate()
            .map(|(idx, id)| (id.clone(), idx))
            .collect();

        for i in 0..pool.customers.len() {
            if self.rng.gen::<f64>() < referral_rate {
                // This customer was referred - find a referrer
                let potential_referrers: Vec<usize> = customer_ids
                    .iter()
                    .enumerate()
                    .filter(|(j, id)| {
                        *j != i && referral_counts.get(*id).copied().unwrap_or(0) < max_referrals
                    })
                    .map(|(j, _)| j)
                    .collect();

                if !potential_referrers.is_empty() {
                    let referrer_idx =
                        potential_referrers[self.rng.gen_range(0..potential_referrers.len())];
                    let referrer_id = customer_ids[referrer_idx].clone();
                    let customer_id = pool.customers[i].customer_id.clone();

                    // Update the referred customer
                    pool.customers[i].network_position.referred_by = Some(referrer_id.clone());

                    // Update the referrer's referral list
                    if let Some(&ref_idx) = id_to_idx.get(&referrer_id) {
                        pool.customers[ref_idx]
                            .network_position
                            .referrals_made
                            .push(customer_id.clone());
                    }

                    *referral_counts.entry(referrer_id).or_insert(0) += 1;
                }
            }
        }
    }

    /// Build corporate hierarchies among customers.
    fn build_corporate_hierarchies(
        &mut self,
        pool: &mut SegmentedCustomerPool,
        customer_ids: &[String],
        parent_candidates: &[String],
    ) {
        let hierarchy_rate = self.segmentation_config.hierarchy_config.hierarchy_rate;
        let billing_consolidation_rate = self
            .segmentation_config
            .hierarchy_config
            .billing_consolidation_rate;

        // Create customer ID to index mapping
        let id_to_idx: std::collections::HashMap<String, usize> = customer_ids
            .iter()
            .enumerate()
            .map(|(idx, id)| (id.clone(), idx))
            .collect();

        for i in 0..pool.customers.len() {
            // Skip enterprise customers (they are parents) and already-hierarchied customers
            if pool.customers[i].segment == CustomerValueSegment::Enterprise
                || pool.customers[i].network_position.parent_customer.is_some()
            {
                continue;
            }

            if self.rng.gen::<f64>() < hierarchy_rate && !parent_candidates.is_empty() {
                // Assign a parent
                let parent_idx = self.rng.gen_range(0..parent_candidates.len());
                let parent_id = parent_candidates[parent_idx].clone();
                let customer_id = pool.customers[i].customer_id.clone();

                // Update the child
                pool.customers[i].network_position.parent_customer = Some(parent_id.clone());
                pool.customers[i].network_position.billing_consolidation =
                    self.rng.gen::<f64>() < billing_consolidation_rate;

                // Update the parent's child list
                if let Some(&parent_idx) = id_to_idx.get(&parent_id) {
                    pool.customers[parent_idx]
                        .network_position
                        .child_customers
                        .push(customer_id);
                }
            }
        }
    }

    /// Populate engagement metrics for customers.
    fn populate_engagement_metrics(
        &mut self,
        pool: &mut SegmentedCustomerPool,
        effective_date: NaiveDate,
    ) {
        for customer in &mut pool.customers {
            // Generate engagement based on lifecycle stage and segment
            let (base_orders, base_revenue) = match customer.lifecycle_stage {
                CustomerLifecycleStage::Mature {
                    avg_annual_spend, ..
                } => {
                    let orders = self.rng.gen_range(12..48);
                    (orders, avg_annual_spend)
                }
                CustomerLifecycleStage::Growth { growth_rate, .. } => {
                    let orders = self.rng.gen_range(6..24);
                    let rev = Decimal::from(orders * self.rng.gen_range(5000..20000));
                    (
                        orders,
                        rev * Decimal::from_f64_retain(1.0 + growth_rate).unwrap_or(Decimal::ONE),
                    )
                }
                CustomerLifecycleStage::New { .. } => {
                    let orders = self.rng.gen_range(1..6);
                    (
                        orders,
                        Decimal::from(orders * self.rng.gen_range(2000..10000)),
                    )
                }
                CustomerLifecycleStage::AtRisk { .. } => {
                    let orders = self.rng.gen_range(2..12);
                    (
                        orders,
                        Decimal::from(orders * self.rng.gen_range(3000..15000)),
                    )
                }
                CustomerLifecycleStage::Churned { .. } => (0, Decimal::ZERO),
                _ => (0, Decimal::ZERO),
            };

            customer.engagement = CustomerEngagement {
                total_orders: base_orders as u32,
                orders_last_12_months: (base_orders as f64 * 0.5) as u32,
                lifetime_revenue: base_revenue,
                revenue_last_12_months: base_revenue
                    * Decimal::from_f64_retain(0.5).unwrap_or(Decimal::ZERO),
                average_order_value: if base_orders > 0 {
                    base_revenue / Decimal::from(base_orders)
                } else {
                    Decimal::ZERO
                },
                days_since_last_order: match &customer.lifecycle_stage {
                    CustomerLifecycleStage::Churned { last_activity, .. } => {
                        (effective_date - *last_activity).num_days().max(0) as u32
                    }
                    CustomerLifecycleStage::AtRisk { .. } => self.rng.gen_range(30..120),
                    _ => self.rng.gen_range(1..30),
                },
                last_order_date: Some(
                    effective_date - chrono::Duration::days(self.rng.gen_range(1..90)),
                ),
                first_order_date: Some(
                    effective_date - chrono::Duration::days(self.rng.gen_range(180..1825)),
                ),
                products_purchased: base_orders as u32 * self.rng.gen_range(1..5),
                support_tickets: self.rng.gen_range(0..10),
                nps_score: Some(self.rng.gen_range(-20..80) as i8),
            };

            // Calculate churn risk
            customer.calculate_churn_risk();

            // Calculate upsell potential based on segment and engagement
            customer.upsell_potential = match customer.segment {
                CustomerValueSegment::Enterprise => 0.3 + self.rng.gen_range(0.0..0.2),
                CustomerValueSegment::MidMarket => 0.4 + self.rng.gen_range(0.0..0.3),
                CustomerValueSegment::Smb => 0.5 + self.rng.gen_range(0.0..0.3),
                CustomerValueSegment::Consumer => 0.2 + self.rng.gen_range(0.0..0.3),
            };
        }
    }

    /// Generate a combined output of CustomerPool and SegmentedCustomerPool.
    pub fn generate_pool_with_segmentation(
        &mut self,
        count: usize,
        company_code: &str,
        effective_date: NaiveDate,
        total_annual_revenue: Decimal,
    ) -> (CustomerPool, SegmentedCustomerPool) {
        let segmented_pool =
            self.generate_segmented_pool(count, company_code, effective_date, total_annual_revenue);

        // Create a regular CustomerPool from the segmented customers
        let mut pool = CustomerPool::new();
        for _segmented in &segmented_pool.customers {
            let customer = self.generate_customer(company_code, effective_date);
            pool.add_customer(customer);
        }

        (pool, segmented_pool)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_customer_generation() {
        let mut gen = CustomerGenerator::new(42);
        let customer = gen.generate_customer("1000", NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

        assert!(!customer.customer_id.is_empty());
        assert!(!customer.name.is_empty());
        assert!(customer.credit_limit > Decimal::ZERO);
    }

    #[test]
    fn test_customer_pool_generation() {
        let mut gen = CustomerGenerator::new(42);
        let pool =
            gen.generate_customer_pool(20, "1000", NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

        assert_eq!(pool.customers.len(), 20);
    }

    #[test]
    fn test_intercompany_customer() {
        let mut gen = CustomerGenerator::new(42);
        let customer = gen.generate_intercompany_customer(
            "1000",
            "2000",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        assert!(customer.is_intercompany);
        assert_eq!(customer.intercompany_code, Some("2000".to_string()));
        assert_eq!(customer.credit_rating, CreditRating::AAA);
    }

    #[test]
    fn test_diverse_pool() {
        let mut gen = CustomerGenerator::new(42);
        let pool =
            gen.generate_diverse_pool(100, "1000", NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

        // Should have customers with various credit ratings
        let aaa_count = pool
            .customers
            .iter()
            .filter(|c| c.credit_rating == CreditRating::AAA)
            .count();
        let d_count = pool
            .customers
            .iter()
            .filter(|c| c.credit_rating == CreditRating::D)
            .count();

        assert!(aaa_count > 0);
        assert!(d_count > 0);
    }

    #[test]
    fn test_deterministic_generation() {
        let mut gen1 = CustomerGenerator::new(42);
        let mut gen2 = CustomerGenerator::new(42);

        let customer1 =
            gen1.generate_customer("1000", NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        let customer2 =
            gen2.generate_customer("1000", NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

        assert_eq!(customer1.customer_id, customer2.customer_id);
        assert_eq!(customer1.name, customer2.name);
        assert_eq!(customer1.credit_rating, customer2.credit_rating);
    }

    #[test]
    fn test_customer_with_specific_credit() {
        let mut gen = CustomerGenerator::new(42);
        let customer = gen.generate_customer_with_credit(
            "1000",
            CreditRating::D,
            Decimal::from(5000),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        assert_eq!(customer.credit_rating, CreditRating::D);
        assert_eq!(customer.credit_limit, Decimal::from(5000));
        assert_eq!(customer.payment_behavior, CustomerPaymentBehavior::HighRisk);
    }

    // ===== Customer Segmentation Tests =====

    #[test]
    fn test_segmented_pool_generation() {
        let segmentation_config = CustomerSegmentationConfig {
            enabled: true,
            ..Default::default()
        };

        let mut gen = CustomerGenerator::with_segmentation_config(
            42,
            CustomerGeneratorConfig::default(),
            segmentation_config,
        );

        let pool = gen.generate_segmented_pool(
            100,
            "1000",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            Decimal::from(10_000_000),
        );

        assert_eq!(pool.customers.len(), 100);
        assert!(!pool.customers.is_empty());
    }

    #[test]
    fn test_segment_distribution() {
        let segmentation_config = CustomerSegmentationConfig {
            enabled: true,
            segment_distribution: SegmentDistribution {
                enterprise: 0.05,
                mid_market: 0.20,
                smb: 0.50,
                consumer: 0.25,
            },
            ..Default::default()
        };

        let mut gen = CustomerGenerator::with_segmentation_config(
            42,
            CustomerGeneratorConfig::default(),
            segmentation_config,
        );

        let pool = gen.generate_segmented_pool(
            200,
            "1000",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            Decimal::from(10_000_000),
        );

        // Count by segment
        let enterprise_count = pool
            .customers
            .iter()
            .filter(|c| c.segment == CustomerValueSegment::Enterprise)
            .count();
        let smb_count = pool
            .customers
            .iter()
            .filter(|c| c.segment == CustomerValueSegment::Smb)
            .count();

        // Enterprise should be ~5% (10 of 200)
        assert!((5..=20).contains(&enterprise_count));
        // SMB should be ~50% (100 of 200)
        assert!((80..=120).contains(&smb_count));
    }

    #[test]
    fn test_referral_network() {
        let segmentation_config = CustomerSegmentationConfig {
            enabled: true,
            referral_config: ReferralConfig {
                enabled: true,
                referral_rate: 0.30, // Higher rate for testing
                max_referrals_per_customer: 5,
            },
            ..Default::default()
        };

        let mut gen = CustomerGenerator::with_segmentation_config(
            42,
            CustomerGeneratorConfig::default(),
            segmentation_config,
        );

        let pool = gen.generate_segmented_pool(
            50,
            "1000",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            Decimal::from(5_000_000),
        );

        // Count customers who were referred
        let referred_count = pool
            .customers
            .iter()
            .filter(|c| c.network_position.was_referred())
            .count();

        // Should have some referred customers
        assert!(referred_count > 0);
    }

    #[test]
    fn test_corporate_hierarchy() {
        let segmentation_config = CustomerSegmentationConfig {
            enabled: true,
            segment_distribution: SegmentDistribution {
                enterprise: 0.10, // More enterprise for testing
                mid_market: 0.30,
                smb: 0.40,
                consumer: 0.20,
            },
            hierarchy_config: HierarchyConfig {
                enabled: true,
                hierarchy_rate: 0.50, // Higher rate for testing
                max_depth: 3,
                billing_consolidation_rate: 0.50,
            },
            ..Default::default()
        };

        let mut gen = CustomerGenerator::with_segmentation_config(
            42,
            CustomerGeneratorConfig::default(),
            segmentation_config,
        );

        let pool = gen.generate_segmented_pool(
            50,
            "1000",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            Decimal::from(5_000_000),
        );

        // Count customers in hierarchies (have a parent)
        let in_hierarchy_count = pool
            .customers
            .iter()
            .filter(|c| c.network_position.parent_customer.is_some())
            .count();

        // Should have some customers in hierarchies
        assert!(in_hierarchy_count > 0);

        // Count enterprise customers with children
        let parents_with_children = pool
            .customers
            .iter()
            .filter(|c| {
                c.segment == CustomerValueSegment::Enterprise
                    && !c.network_position.child_customers.is_empty()
            })
            .count();

        assert!(parents_with_children > 0);
    }

    #[test]
    fn test_lifecycle_stages() {
        let segmentation_config = CustomerSegmentationConfig {
            enabled: true,
            lifecycle_distribution: LifecycleDistribution {
                prospect: 0.0,
                new: 0.20,
                growth: 0.20,
                mature: 0.40,
                at_risk: 0.15,
                churned: 0.05,
            },
            ..Default::default()
        };

        let mut gen = CustomerGenerator::with_segmentation_config(
            42,
            CustomerGeneratorConfig::default(),
            segmentation_config,
        );

        let pool = gen.generate_segmented_pool(
            100,
            "1000",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            Decimal::from(10_000_000),
        );

        // Count at-risk customers
        let at_risk_count = pool
            .customers
            .iter()
            .filter(|c| matches!(c.lifecycle_stage, CustomerLifecycleStage::AtRisk { .. }))
            .count();

        // Should be roughly 15%
        assert!((5..=30).contains(&at_risk_count));

        // Count mature customers
        let mature_count = pool
            .customers
            .iter()
            .filter(|c| matches!(c.lifecycle_stage, CustomerLifecycleStage::Mature { .. }))
            .count();

        // Should be roughly 40%
        assert!((25..=55).contains(&mature_count));
    }

    #[test]
    fn test_engagement_metrics() {
        let segmentation_config = CustomerSegmentationConfig {
            enabled: true,
            ..Default::default()
        };

        let mut gen = CustomerGenerator::with_segmentation_config(
            42,
            CustomerGeneratorConfig::default(),
            segmentation_config,
        );

        let pool = gen.generate_segmented_pool(
            20,
            "1000",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            Decimal::from(2_000_000),
        );

        // All customers should have engagement data populated
        for customer in &pool.customers {
            // Churned customers may have 0 orders
            if !matches!(
                customer.lifecycle_stage,
                CustomerLifecycleStage::Churned { .. }
            ) {
                // Active customers should have some orders
                assert!(
                    customer.engagement.total_orders > 0
                        || matches!(
                            customer.lifecycle_stage,
                            CustomerLifecycleStage::Prospect { .. }
                        )
                );
            }

            // Churn risk should be calculated
            assert!(customer.churn_risk_score >= 0.0 && customer.churn_risk_score <= 1.0);
        }
    }

    #[test]
    fn test_segment_distribution_validation() {
        let valid = SegmentDistribution::default();
        assert!(valid.validate().is_ok());

        let invalid = SegmentDistribution {
            enterprise: 0.5,
            mid_market: 0.5,
            smb: 0.5,
            consumer: 0.5,
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_segmentation_disabled() {
        let segmentation_config = CustomerSegmentationConfig {
            enabled: false,
            ..Default::default()
        };

        let mut gen = CustomerGenerator::with_segmentation_config(
            42,
            CustomerGeneratorConfig::default(),
            segmentation_config,
        );

        let pool = gen.generate_segmented_pool(
            20,
            "1000",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            Decimal::from(2_000_000),
        );

        // Should return empty pool when disabled
        assert!(pool.customers.is_empty());
    }
}
