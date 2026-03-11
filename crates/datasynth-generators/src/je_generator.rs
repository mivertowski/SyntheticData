//! Journal Entry generator with statistical distributions.

use chrono::{Datelike, NaiveDate};
use datasynth_core::utils::seeded_rng;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use std::sync::Arc;

use tracing::debug;

use datasynth_config::schema::{
    FraudConfig, GeneratorConfig, TemplateConfig, TemporalPatternsConfig, TransactionConfig,
};
use datasynth_core::distributions::{
    BusinessDayCalculator, CrossDayConfig, DriftAdjustments, DriftConfig, DriftController,
    EventType, LagDistribution, PeriodEndConfig, PeriodEndDynamics, PeriodEndModel,
    ProcessingLagCalculator, ProcessingLagConfig, *,
};
use datasynth_core::models::*;
use datasynth_core::templates::{
    descriptions::DescriptionContext, DescriptionGenerator, ReferenceGenerator, ReferenceType,
};
use datasynth_core::traits::Generator;
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use datasynth_core::CountryPack;

use crate::company_selector::WeightedCompanySelector;
use crate::user_generator::{UserGenerator, UserGeneratorConfig};

/// Generator for realistic journal entries.
pub struct JournalEntryGenerator {
    rng: ChaCha8Rng,
    seed: u64,
    config: TransactionConfig,
    coa: Arc<ChartOfAccounts>,
    companies: Vec<String>,
    company_selector: WeightedCompanySelector,
    line_sampler: LineItemSampler,
    amount_sampler: AmountSampler,
    temporal_sampler: TemporalSampler,
    start_date: NaiveDate,
    end_date: NaiveDate,
    count: u64,
    uuid_factory: DeterministicUuidFactory,
    // Enhanced features
    user_pool: Option<UserPool>,
    description_generator: DescriptionGenerator,
    reference_generator: ReferenceGenerator,
    template_config: TemplateConfig,
    vendor_pool: VendorPool,
    customer_pool: CustomerPool,
    // Material pool for realistic material references
    material_pool: Option<MaterialPool>,
    // Flag indicating whether we're using real master data vs defaults
    using_real_master_data: bool,
    // Fraud generation
    fraud_config: FraudConfig,
    // Persona-based error injection
    persona_errors_enabled: bool,
    // Approval threshold enforcement
    approval_enabled: bool,
    approval_threshold: rust_decimal::Decimal,
    // SOD violation rate for approval tracking (0.0 to 1.0)
    sod_violation_rate: f64,
    // Batching behavior - humans often process similar items together
    batch_state: Option<BatchState>,
    // Temporal drift controller for simulating distribution changes over time
    drift_controller: Option<DriftController>,
    // Temporal patterns components
    business_day_calculator: Option<BusinessDayCalculator>,
    processing_lag_calculator: Option<ProcessingLagCalculator>,
    temporal_patterns_config: Option<TemporalPatternsConfig>,
}

/// State for tracking batch processing behavior.
///
/// When humans process transactions, they often batch similar items together
/// (e.g., processing all invoices from one vendor, entering similar expenses).
#[derive(Clone)]
struct BatchState {
    /// The base entry template to vary
    base_account_number: String,
    base_amount: rust_decimal::Decimal,
    base_business_process: Option<BusinessProcess>,
    base_posting_date: NaiveDate,
    /// Remaining entries in this batch
    remaining: u8,
}

impl JournalEntryGenerator {
    /// Create a new journal entry generator.
    pub fn new_with_params(
        config: TransactionConfig,
        coa: Arc<ChartOfAccounts>,
        companies: Vec<String>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        seed: u64,
    ) -> Self {
        Self::new_with_full_config(
            config,
            coa,
            companies,
            start_date,
            end_date,
            seed,
            TemplateConfig::default(),
            None,
        )
    }

    /// Create a new journal entry generator with full configuration.
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_full_config(
        config: TransactionConfig,
        coa: Arc<ChartOfAccounts>,
        companies: Vec<String>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        seed: u64,
        template_config: TemplateConfig,
        user_pool: Option<UserPool>,
    ) -> Self {
        // Initialize user pool if not provided
        let user_pool = user_pool.or_else(|| {
            if template_config.names.generate_realistic_names {
                let user_gen_config = UserGeneratorConfig {
                    culture_distribution: vec![
                        (
                            datasynth_core::templates::NameCulture::WesternUs,
                            template_config.names.culture_distribution.western_us,
                        ),
                        (
                            datasynth_core::templates::NameCulture::Hispanic,
                            template_config.names.culture_distribution.hispanic,
                        ),
                        (
                            datasynth_core::templates::NameCulture::German,
                            template_config.names.culture_distribution.german,
                        ),
                        (
                            datasynth_core::templates::NameCulture::French,
                            template_config.names.culture_distribution.french,
                        ),
                        (
                            datasynth_core::templates::NameCulture::Chinese,
                            template_config.names.culture_distribution.chinese,
                        ),
                        (
                            datasynth_core::templates::NameCulture::Japanese,
                            template_config.names.culture_distribution.japanese,
                        ),
                        (
                            datasynth_core::templates::NameCulture::Indian,
                            template_config.names.culture_distribution.indian,
                        ),
                    ],
                    email_domain: template_config.names.email_domain.clone(),
                    generate_realistic_names: true,
                };
                let mut user_gen = UserGenerator::with_config(seed + 100, user_gen_config);
                Some(user_gen.generate_standard(&companies))
            } else {
                None
            }
        });

        // Initialize reference generator
        let mut ref_gen = ReferenceGenerator::new(
            start_date.year(),
            companies
                .first()
                .map(std::string::String::as_str)
                .unwrap_or("1000"),
        );
        ref_gen.set_prefix(
            ReferenceType::Invoice,
            &template_config.references.invoice_prefix,
        );
        ref_gen.set_prefix(
            ReferenceType::PurchaseOrder,
            &template_config.references.po_prefix,
        );
        ref_gen.set_prefix(
            ReferenceType::SalesOrder,
            &template_config.references.so_prefix,
        );

        // Create weighted company selector (uniform weights for this constructor)
        let company_selector = WeightedCompanySelector::uniform(companies.clone());

        Self {
            rng: seeded_rng(seed, 0),
            seed,
            config: config.clone(),
            coa,
            companies,
            company_selector,
            line_sampler: LineItemSampler::with_config(
                seed + 1,
                config.line_item_distribution.clone(),
                config.even_odd_distribution.clone(),
                config.debit_credit_distribution.clone(),
            ),
            amount_sampler: AmountSampler::with_config(seed + 2, config.amounts.clone()),
            temporal_sampler: TemporalSampler::with_config(
                seed + 3,
                config.seasonality.clone(),
                WorkingHoursConfig::default(),
                Vec::new(),
            ),
            start_date,
            end_date,
            count: 0,
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::JournalEntry),
            user_pool,
            description_generator: DescriptionGenerator::new(),
            reference_generator: ref_gen,
            template_config,
            vendor_pool: VendorPool::standard(),
            customer_pool: CustomerPool::standard(),
            material_pool: None,
            using_real_master_data: false,
            fraud_config: FraudConfig::default(),
            persona_errors_enabled: true, // Enable by default for realism
            approval_enabled: true,       // Enable by default for realism
            approval_threshold: rust_decimal::Decimal::new(10000, 0), // $10,000 default threshold
            sod_violation_rate: 0.10,  // 10% default SOD violation rate
            batch_state: None,
            drift_controller: None,
            // Always provide a basic BusinessDayCalculator so that weekend/holiday
            // filtering is active even when temporal_patterns is not explicitly enabled.
            business_day_calculator: Some(BusinessDayCalculator::new(HolidayCalendar::new(
                Region::US,
                start_date.year(),
            ))),
            processing_lag_calculator: None,
            temporal_patterns_config: None,
        }
    }

    /// Create from a full GeneratorConfig.
    ///
    /// This constructor uses the volume_weight from company configs
    /// for weighted company selection, and fraud config from GeneratorConfig.
    pub fn from_generator_config(
        full_config: &GeneratorConfig,
        coa: Arc<ChartOfAccounts>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        seed: u64,
    ) -> Self {
        let companies: Vec<String> = full_config
            .companies
            .iter()
            .map(|c| c.code.clone())
            .collect();

        // Create weighted selector using volume_weight from company configs
        let company_selector = WeightedCompanySelector::from_configs(&full_config.companies);

        let mut generator = Self::new_with_full_config(
            full_config.transactions.clone(),
            coa,
            companies,
            start_date,
            end_date,
            seed,
            full_config.templates.clone(),
            None,
        );

        // Override the uniform selector with weighted selector
        generator.company_selector = company_selector;

        // Set fraud config
        generator.fraud_config = full_config.fraud.clone();

        // Configure temporal patterns if enabled
        let temporal_config = &full_config.temporal_patterns;
        if temporal_config.enabled {
            generator = generator.with_temporal_patterns(temporal_config.clone(), seed);
        }

        generator
    }

    /// Configure temporal patterns including business day calculations and processing lags.
    ///
    /// This enables realistic temporal behavior including:
    /// - Business day awareness (no postings on weekends/holidays)
    /// - Processing lag modeling (event-to-posting delays)
    /// - Period-end dynamics (volume spikes at month/quarter/year end)
    pub fn with_temporal_patterns(mut self, config: TemporalPatternsConfig, seed: u64) -> Self {
        // Create business day calculator if enabled
        if config.business_days.enabled {
            let region = config
                .calendars
                .regions
                .first()
                .map(|r| Self::parse_region(r))
                .unwrap_or(Region::US);

            let calendar = HolidayCalendar::new(region, self.start_date.year());
            self.business_day_calculator = Some(BusinessDayCalculator::new(calendar));
        }

        // Create processing lag calculator if enabled
        if config.processing_lags.enabled {
            let lag_config = Self::convert_processing_lag_config(&config.processing_lags);
            self.processing_lag_calculator =
                Some(ProcessingLagCalculator::with_config(seed, lag_config));
        }

        // Create period-end dynamics if configured
        let model = config.period_end.model.as_deref().unwrap_or("flat");
        if model != "flat"
            || config
                .period_end
                .month_end
                .as_ref()
                .is_some_and(|m| m.peak_multiplier.unwrap_or(1.0) != 1.0)
        {
            let dynamics = Self::convert_period_end_config(&config.period_end);
            self.temporal_sampler.set_period_end_dynamics(dynamics);
        }

        self.temporal_patterns_config = Some(config);
        self
    }

    /// Configure temporal patterns using a [`CountryPack`] for the holiday calendar.
    ///
    /// This is an alternative to [`with_temporal_patterns`] that derives the
    /// holiday calendar from a country-pack definition rather than the built-in
    /// region-based calendars.  All other temporal behaviour (business-day
    /// adjustment, processing lags, period-end dynamics) is configured
    /// identically.
    pub fn with_country_pack_temporal(
        mut self,
        config: TemporalPatternsConfig,
        seed: u64,
        pack: &CountryPack,
    ) -> Self {
        // Create business day calculator using the country pack calendar
        if config.business_days.enabled {
            let calendar = HolidayCalendar::from_country_pack(pack, self.start_date.year());
            self.business_day_calculator = Some(BusinessDayCalculator::new(calendar));
        }

        // Create processing lag calculator if enabled
        if config.processing_lags.enabled {
            let lag_config = Self::convert_processing_lag_config(&config.processing_lags);
            self.processing_lag_calculator =
                Some(ProcessingLagCalculator::with_config(seed, lag_config));
        }

        // Create period-end dynamics if configured
        let model = config.period_end.model.as_deref().unwrap_or("flat");
        if model != "flat"
            || config
                .period_end
                .month_end
                .as_ref()
                .is_some_and(|m| m.peak_multiplier.unwrap_or(1.0) != 1.0)
        {
            let dynamics = Self::convert_period_end_config(&config.period_end);
            self.temporal_sampler.set_period_end_dynamics(dynamics);
        }

        self.temporal_patterns_config = Some(config);
        self
    }

    /// Convert schema processing lag config to core config.
    fn convert_processing_lag_config(
        schema: &datasynth_config::schema::ProcessingLagSchemaConfig,
    ) -> ProcessingLagConfig {
        let mut config = ProcessingLagConfig {
            enabled: schema.enabled,
            ..Default::default()
        };

        // Helper to convert lag schema to distribution
        let convert_lag = |lag: &datasynth_config::schema::LagDistributionSchemaConfig| {
            let mut dist = LagDistribution::log_normal(lag.mu, lag.sigma);
            if let Some(min) = lag.min_hours {
                dist.min_lag_hours = min;
            }
            if let Some(max) = lag.max_hours {
                dist.max_lag_hours = max;
            }
            dist
        };

        // Apply event-specific lags
        if let Some(ref lag) = schema.sales_order_lag {
            config
                .event_lags
                .insert(EventType::SalesOrder, convert_lag(lag));
        }
        if let Some(ref lag) = schema.purchase_order_lag {
            config
                .event_lags
                .insert(EventType::PurchaseOrder, convert_lag(lag));
        }
        if let Some(ref lag) = schema.goods_receipt_lag {
            config
                .event_lags
                .insert(EventType::GoodsReceipt, convert_lag(lag));
        }
        if let Some(ref lag) = schema.invoice_receipt_lag {
            config
                .event_lags
                .insert(EventType::InvoiceReceipt, convert_lag(lag));
        }
        if let Some(ref lag) = schema.invoice_issue_lag {
            config
                .event_lags
                .insert(EventType::InvoiceIssue, convert_lag(lag));
        }
        if let Some(ref lag) = schema.payment_lag {
            config
                .event_lags
                .insert(EventType::Payment, convert_lag(lag));
        }
        if let Some(ref lag) = schema.journal_entry_lag {
            config
                .event_lags
                .insert(EventType::JournalEntry, convert_lag(lag));
        }

        // Apply cross-day posting config
        if let Some(ref cross_day) = schema.cross_day_posting {
            config.cross_day = CrossDayConfig {
                enabled: cross_day.enabled,
                probability_by_hour: cross_day.probability_by_hour.clone(),
                ..Default::default()
            };
        }

        config
    }

    /// Convert schema period-end config to core PeriodEndDynamics.
    fn convert_period_end_config(
        schema: &datasynth_config::schema::PeriodEndSchemaConfig,
    ) -> PeriodEndDynamics {
        let model_type = schema.model.as_deref().unwrap_or("exponential");

        // Helper to convert period config
        let convert_period =
            |period: Option<&datasynth_config::schema::PeriodEndModelSchemaConfig>,
             default_peak: f64|
             -> PeriodEndConfig {
                if let Some(p) = period {
                    let model = match model_type {
                        "flat" => PeriodEndModel::FlatMultiplier {
                            multiplier: p.peak_multiplier.unwrap_or(default_peak),
                        },
                        "extended_crunch" => PeriodEndModel::ExtendedCrunch {
                            start_day: p.start_day.unwrap_or(-10),
                            sustained_high_days: p.sustained_high_days.unwrap_or(3),
                            peak_multiplier: p.peak_multiplier.unwrap_or(default_peak),
                            ramp_up_days: 3, // Default ramp-up period
                        },
                        _ => PeriodEndModel::ExponentialAcceleration {
                            start_day: p.start_day.unwrap_or(-10),
                            base_multiplier: p.base_multiplier.unwrap_or(1.0),
                            peak_multiplier: p.peak_multiplier.unwrap_or(default_peak),
                            decay_rate: p.decay_rate.unwrap_or(0.3),
                        },
                    };
                    PeriodEndConfig {
                        enabled: true,
                        model,
                        additional_multiplier: p.additional_multiplier.unwrap_or(1.0),
                    }
                } else {
                    PeriodEndConfig {
                        enabled: true,
                        model: PeriodEndModel::ExponentialAcceleration {
                            start_day: -10,
                            base_multiplier: 1.0,
                            peak_multiplier: default_peak,
                            decay_rate: 0.3,
                        },
                        additional_multiplier: 1.0,
                    }
                }
            };

        PeriodEndDynamics::new(
            convert_period(schema.month_end.as_ref(), 2.0),
            convert_period(schema.quarter_end.as_ref(), 3.5),
            convert_period(schema.year_end.as_ref(), 5.0),
        )
    }

    /// Parse a region string into a Region enum.
    fn parse_region(region_str: &str) -> Region {
        match region_str.to_uppercase().as_str() {
            "US" => Region::US,
            "DE" => Region::DE,
            "GB" => Region::GB,
            "CN" => Region::CN,
            "JP" => Region::JP,
            "IN" => Region::IN,
            "BR" => Region::BR,
            "MX" => Region::MX,
            "AU" => Region::AU,
            "SG" => Region::SG,
            "KR" => Region::KR,
            "FR" => Region::FR,
            "IT" => Region::IT,
            "ES" => Region::ES,
            "CA" => Region::CA,
            _ => Region::US,
        }
    }

    /// Set a custom company selector.
    pub fn set_company_selector(&mut self, selector: WeightedCompanySelector) {
        self.company_selector = selector;
    }

    /// Get the current company selector.
    pub fn company_selector(&self) -> &WeightedCompanySelector {
        &self.company_selector
    }

    /// Set fraud configuration.
    pub fn set_fraud_config(&mut self, config: FraudConfig) {
        self.fraud_config = config;
    }

    /// Set vendors from generated master data.
    ///
    /// This replaces the default vendor pool with actual generated vendors,
    /// ensuring JEs reference real master data entities.
    pub fn with_vendors(mut self, vendors: &[Vendor]) -> Self {
        if !vendors.is_empty() {
            self.vendor_pool = VendorPool::from_vendors(vendors.to_vec());
            self.using_real_master_data = true;
        }
        self
    }

    /// Set customers from generated master data.
    ///
    /// This replaces the default customer pool with actual generated customers,
    /// ensuring JEs reference real master data entities.
    pub fn with_customers(mut self, customers: &[Customer]) -> Self {
        if !customers.is_empty() {
            self.customer_pool = CustomerPool::from_customers(customers.to_vec());
            self.using_real_master_data = true;
        }
        self
    }

    /// Set materials from generated master data.
    ///
    /// This provides material references for JEs that involve inventory movements.
    pub fn with_materials(mut self, materials: &[Material]) -> Self {
        if !materials.is_empty() {
            self.material_pool = Some(MaterialPool::from_materials(materials.to_vec()));
            self.using_real_master_data = true;
        }
        self
    }

    /// Set all master data at once for convenience.
    ///
    /// This is the recommended way to configure the JE generator with
    /// generated master data to ensure data coherence.
    pub fn with_master_data(
        self,
        vendors: &[Vendor],
        customers: &[Customer],
        materials: &[Material],
    ) -> Self {
        self.with_vendors(vendors)
            .with_customers(customers)
            .with_materials(materials)
    }

    /// Replace the user pool with one generated from a [`CountryPack`].
    ///
    /// This is an alternative to the default name-culture distribution that
    /// derives name pools and weights from the country-pack's `names` section.
    /// The existing user pool (if any) is discarded and regenerated using
    /// [`MultiCultureNameGenerator::from_country_pack`].
    pub fn with_country_pack_names(mut self, pack: &CountryPack) -> Self {
        let name_gen =
            datasynth_core::templates::MultiCultureNameGenerator::from_country_pack(pack);
        let config = UserGeneratorConfig {
            // The culture distribution is embedded in the name generator
            // itself, so we use an empty list here.
            culture_distribution: Vec::new(),
            email_domain: name_gen.email_domain().to_string(),
            generate_realistic_names: true,
        };
        let mut user_gen = UserGenerator::with_name_generator(self.seed + 100, config, name_gen);
        self.user_pool = Some(user_gen.generate_standard(&self.companies));
        self
    }

    /// Check if the generator is using real master data.
    pub fn is_using_real_master_data(&self) -> bool {
        self.using_real_master_data
    }

    /// Determine if this transaction should be fraudulent.
    fn determine_fraud(&mut self) -> Option<FraudType> {
        if !self.fraud_config.enabled {
            return None;
        }

        // Roll for fraud based on fraud rate
        if self.rng.random::<f64>() >= self.fraud_config.fraud_rate {
            return None;
        }

        // Select fraud type based on distribution
        Some(self.select_fraud_type())
    }

    /// Select a fraud type based on the configured distribution.
    fn select_fraud_type(&mut self) -> FraudType {
        let dist = &self.fraud_config.fraud_type_distribution;
        let roll: f64 = self.rng.random();

        let mut cumulative = 0.0;

        cumulative += dist.suspense_account_abuse;
        if roll < cumulative {
            return FraudType::SuspenseAccountAbuse;
        }

        cumulative += dist.fictitious_transaction;
        if roll < cumulative {
            return FraudType::FictitiousTransaction;
        }

        cumulative += dist.revenue_manipulation;
        if roll < cumulative {
            return FraudType::RevenueManipulation;
        }

        cumulative += dist.expense_capitalization;
        if roll < cumulative {
            return FraudType::ExpenseCapitalization;
        }

        cumulative += dist.split_transaction;
        if roll < cumulative {
            return FraudType::SplitTransaction;
        }

        cumulative += dist.timing_anomaly;
        if roll < cumulative {
            return FraudType::TimingAnomaly;
        }

        cumulative += dist.unauthorized_access;
        if roll < cumulative {
            return FraudType::UnauthorizedAccess;
        }

        // Default fallback
        FraudType::DuplicatePayment
    }

    /// Map a fraud type to an amount pattern for suspicious amounts.
    fn fraud_type_to_amount_pattern(&self, fraud_type: FraudType) -> FraudAmountPattern {
        match fraud_type {
            FraudType::SplitTransaction | FraudType::JustBelowThreshold => {
                FraudAmountPattern::ThresholdAdjacent
            }
            FraudType::FictitiousTransaction
            | FraudType::FictitiousEntry
            | FraudType::SuspenseAccountAbuse
            | FraudType::RoundDollarManipulation => FraudAmountPattern::ObviousRoundNumbers,
            FraudType::RevenueManipulation
            | FraudType::ExpenseCapitalization
            | FraudType::ImproperCapitalization
            | FraudType::ReserveManipulation
            | FraudType::UnauthorizedAccess
            | FraudType::PrematureRevenue
            | FraudType::UnderstatedLiabilities
            | FraudType::OverstatedAssets
            | FraudType::ChannelStuffing => FraudAmountPattern::StatisticallyImprobable,
            FraudType::DuplicatePayment
            | FraudType::TimingAnomaly
            | FraudType::SelfApproval
            | FraudType::ExceededApprovalLimit
            | FraudType::SegregationOfDutiesViolation
            | FraudType::UnauthorizedApproval
            | FraudType::CollusiveApproval
            | FraudType::FictitiousVendor
            | FraudType::ShellCompanyPayment
            | FraudType::Kickback
            | FraudType::KickbackScheme
            | FraudType::InvoiceManipulation
            | FraudType::AssetMisappropriation
            | FraudType::InventoryTheft
            | FraudType::GhostEmployee => FraudAmountPattern::Normal,
            // Accounting Standards Fraud Types (ASC 606/IFRS 15 - Revenue)
            FraudType::ImproperRevenueRecognition
            | FraudType::ImproperPoAllocation
            | FraudType::VariableConsiderationManipulation
            | FraudType::ContractModificationMisstatement => {
                FraudAmountPattern::StatisticallyImprobable
            }
            // Accounting Standards Fraud Types (ASC 842/IFRS 16 - Leases)
            FraudType::LeaseClassificationManipulation
            | FraudType::OffBalanceSheetLease
            | FraudType::LeaseLiabilityUnderstatement
            | FraudType::RouAssetMisstatement => FraudAmountPattern::StatisticallyImprobable,
            // Accounting Standards Fraud Types (ASC 820/IFRS 13 - Fair Value)
            FraudType::FairValueHierarchyManipulation
            | FraudType::Level3InputManipulation
            | FraudType::ValuationTechniqueManipulation => {
                FraudAmountPattern::StatisticallyImprobable
            }
            // Accounting Standards Fraud Types (ASC 360/IAS 36 - Impairment)
            FraudType::DelayedImpairment
            | FraudType::ImpairmentTestAvoidance
            | FraudType::CashFlowProjectionManipulation
            | FraudType::ImproperImpairmentReversal => FraudAmountPattern::StatisticallyImprobable,
            // Sourcing/Procurement Fraud
            FraudType::BidRigging
            | FraudType::PhantomVendorContract
            | FraudType::ConflictOfInterestSourcing => FraudAmountPattern::Normal,
            FraudType::SplitContractThreshold => FraudAmountPattern::ThresholdAdjacent,
            // HR/Payroll Fraud
            FraudType::GhostEmployeePayroll
            | FraudType::PayrollInflation
            | FraudType::DuplicateExpenseReport
            | FraudType::FictitiousExpense => FraudAmountPattern::Normal,
            FraudType::SplitExpenseToAvoidApproval => FraudAmountPattern::ThresholdAdjacent,
            // O2C Fraud
            FraudType::RevenueTimingManipulation => FraudAmountPattern::StatisticallyImprobable,
            FraudType::QuotePriceOverride => FraudAmountPattern::Normal,
        }
    }

    /// Generate a deterministic UUID using the factory.
    #[inline]
    fn generate_deterministic_uuid(&self) -> uuid::Uuid {
        self.uuid_factory.next()
    }

    /// Cost center pool used for expense account enrichment.
    const COST_CENTER_POOL: &'static [&'static str] =
        &["CC1000", "CC2000", "CC3000", "CC4000", "CC5000"];

    /// Enrich journal entry line items with account descriptions, cost centers,
    /// profit centers, value dates, line text, and assignment fields.
    ///
    /// This populates the sparse optional fields that `JournalEntryLine::debit()`
    /// and `::credit()` leave as `None`.
    fn enrich_line_items(&self, entry: &mut JournalEntry) {
        let posting_date = entry.header.posting_date;
        let company_code = &entry.header.company_code;
        let header_text = entry.header.header_text.clone();
        let business_process = entry.header.business_process;

        // Derive a deterministic index from the document_id for cost center selection
        let doc_id_bytes = entry.header.document_id.as_bytes();
        let mut cc_seed: usize = 0;
        for &b in doc_id_bytes {
            cc_seed = cc_seed.wrapping_add(b as usize);
        }

        for (i, line) in entry.lines.iter_mut().enumerate() {
            // 1. account_description: look up from CoA
            if line.account_description.is_none() {
                line.account_description = self
                    .coa
                    .get_account(&line.gl_account)
                    .map(|a| a.short_description.clone());
            }

            // 2. cost_center: assign to expense accounts (5xxx/6xxx)
            if line.cost_center.is_none() {
                let first_char = line.gl_account.chars().next().unwrap_or('0');
                if first_char == '5' || first_char == '6' {
                    let idx = cc_seed.wrapping_add(i) % Self::COST_CENTER_POOL.len();
                    line.cost_center = Some(Self::COST_CENTER_POOL[idx].to_string());
                }
            }

            // 3. profit_center: derive from company code + business process
            if line.profit_center.is_none() {
                let suffix = match business_process {
                    Some(BusinessProcess::P2P) => "-P2P",
                    Some(BusinessProcess::O2C) => "-O2C",
                    Some(BusinessProcess::R2R) => "-R2R",
                    Some(BusinessProcess::H2R) => "-H2R",
                    _ => "",
                };
                line.profit_center = Some(format!("PC-{company_code}{suffix}"));
            }

            // 4. line_text: fall back to header_text if not already set
            if line.line_text.is_none() {
                line.line_text = header_text.clone();
            }

            // 5. value_date: set to posting_date for AR/AP accounts
            if line.value_date.is_none()
                && (line.gl_account.starts_with("1100") || line.gl_account.starts_with("2000"))
            {
                line.value_date = Some(posting_date);
            }

            // 6. assignment: set to vendor/customer reference for AP/AR lines
            if line.assignment.is_none() {
                if line.gl_account.starts_with("2000") {
                    // AP line - use vendor reference from header
                    if let Some(ref ht) = header_text {
                        // Try to extract vendor ID from header text patterns like "... - V-001"
                        if let Some(vendor_part) = ht.rsplit(" - ").next() {
                            if vendor_part.starts_with("V-")
                                || vendor_part.starts_with("VENDOR")
                                || vendor_part.starts_with("Vendor")
                            {
                                line.assignment = Some(vendor_part.to_string());
                            }
                        }
                    }
                } else if line.gl_account.starts_with("1100") {
                    // AR line - use customer reference from header
                    if let Some(ref ht) = header_text {
                        if let Some(customer_part) = ht.rsplit(" - ").next() {
                            if customer_part.starts_with("C-")
                                || customer_part.starts_with("CUST")
                                || customer_part.starts_with("Customer")
                            {
                                line.assignment = Some(customer_part.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    /// Generate a single journal entry.
    pub fn generate(&mut self) -> JournalEntry {
        debug!(
            count = self.count,
            companies = self.companies.len(),
            start_date = %self.start_date,
            end_date = %self.end_date,
            "Generating journal entry"
        );

        // Check if we're in a batch - if so, generate a batched entry
        if let Some(ref state) = self.batch_state {
            if state.remaining > 0 {
                return self.generate_batched_entry();
            }
        }

        self.count += 1;

        // Generate deterministic document ID
        let document_id = self.generate_deterministic_uuid();

        // Sample posting date
        let mut posting_date = self
            .temporal_sampler
            .sample_date(self.start_date, self.end_date);

        // Adjust posting date to be a business day if business day calculator is configured
        if let Some(ref calc) = self.business_day_calculator {
            if !calc.is_business_day(posting_date) {
                // Move to next business day
                posting_date = calc.next_business_day(posting_date, false);
                // Ensure we don't exceed end_date
                if posting_date > self.end_date {
                    posting_date = calc.prev_business_day(self.end_date, true);
                }
            }
        }

        // Select company using weighted selector
        let company_code = self.company_selector.select(&mut self.rng).to_string();

        // Sample line item specification
        let line_spec = self.line_sampler.sample();

        // Determine source type using full 4-way distribution
        let source = self.select_source();
        let is_automated = matches!(
            source,
            TransactionSource::Automated | TransactionSource::Recurring
        );

        // Select business process
        let business_process = self.select_business_process();

        // Determine if this is a fraudulent transaction
        let fraud_type = self.determine_fraud();
        let is_fraud = fraud_type.is_some();

        // Sample time based on source
        let time = self.temporal_sampler.sample_time(!is_automated);
        let created_at = posting_date.and_time(time).and_utc();

        // Select user from pool or generate generic
        let (created_by, user_persona) = self.select_user(is_automated);

        // Create header with deterministic UUID
        let mut header =
            JournalEntryHeader::with_deterministic_id(company_code, posting_date, document_id);
        header.created_at = created_at;
        header.source = source;
        header.created_by = created_by;
        header.user_persona = user_persona;
        header.business_process = Some(business_process);
        header.document_type = Self::document_type_for_process(business_process).to_string();
        header.is_fraud = is_fraud;
        header.fraud_type = fraud_type;

        // Generate description context
        let mut context =
            DescriptionContext::with_period(posting_date.month(), posting_date.year());

        // Add vendor/customer context based on business process
        match business_process {
            BusinessProcess::P2P => {
                if let Some(vendor) = self.vendor_pool.random_vendor(&mut self.rng) {
                    context.vendor_name = Some(vendor.name.clone());
                }
            }
            BusinessProcess::O2C => {
                if let Some(customer) = self.customer_pool.random_customer(&mut self.rng) {
                    context.customer_name = Some(customer.name.clone());
                }
            }
            _ => {}
        }

        // Generate header text if enabled
        if self.template_config.descriptions.generate_header_text {
            header.header_text = Some(self.description_generator.generate_header_text(
                business_process,
                &context,
                &mut self.rng,
            ));
        }

        // Generate reference if enabled
        if self.template_config.references.generate_references {
            header.reference = Some(
                self.reference_generator
                    .generate_for_process_year(business_process, posting_date.year()),
            );
        }

        // Derive typed source document from reference prefix
        header.source_document = header
            .reference
            .as_deref()
            .and_then(DocumentRef::parse)
            .or_else(|| {
                if header.source == TransactionSource::Manual {
                    Some(DocumentRef::Manual)
                } else {
                    None
                }
            });

        // Generate line items
        let mut entry = JournalEntry::new(header);

        // Generate amount - use fraud pattern if this is a fraudulent transaction
        let base_amount = if let Some(ft) = fraud_type {
            let pattern = self.fraud_type_to_amount_pattern(ft);
            self.amount_sampler.sample_fraud(pattern)
        } else {
            self.amount_sampler.sample()
        };

        // Apply temporal drift if configured
        let drift_adjusted_amount = {
            let drift = self.get_drift_adjustments(posting_date);
            if drift.amount_mean_multiplier != 1.0 {
                // Apply drift multiplier (includes seasonal factor if enabled)
                let multiplier = drift.amount_mean_multiplier * drift.seasonal_factor;
                let adjusted = base_amount.to_f64().unwrap_or(1.0) * multiplier;
                Decimal::from_f64_retain(adjusted).unwrap_or(base_amount)
            } else {
                base_amount
            }
        };

        // Apply human variation to amounts for non-automated transactions
        let total_amount = if is_automated {
            drift_adjusted_amount // Automated systems use exact amounts
        } else {
            self.apply_human_variation(drift_adjusted_amount)
        };

        // Generate debit lines
        let debit_amounts = self
            .amount_sampler
            .sample_summing_to(line_spec.debit_count, total_amount);
        for (i, amount) in debit_amounts.into_iter().enumerate() {
            let account_number = self.select_debit_account().account_number.clone();
            let mut line = JournalEntryLine::debit(
                entry.header.document_id,
                (i + 1) as u32,
                account_number.clone(),
                amount,
            );

            // Generate line text if enabled
            if self.template_config.descriptions.generate_line_text {
                line.line_text = Some(self.description_generator.generate_line_text(
                    &account_number,
                    &context,
                    &mut self.rng,
                ));
            }

            entry.add_line(line);
        }

        // Generate credit lines - use the SAME amounts to ensure balance
        let credit_amounts = self
            .amount_sampler
            .sample_summing_to(line_spec.credit_count, total_amount);
        for (i, amount) in credit_amounts.into_iter().enumerate() {
            let account_number = self.select_credit_account().account_number.clone();
            let mut line = JournalEntryLine::credit(
                entry.header.document_id,
                (line_spec.debit_count + i + 1) as u32,
                account_number.clone(),
                amount,
            );

            // Generate line text if enabled
            if self.template_config.descriptions.generate_line_text {
                line.line_text = Some(self.description_generator.generate_line_text(
                    &account_number,
                    &context,
                    &mut self.rng,
                ));
            }

            entry.add_line(line);
        }

        // Enrich line items with account descriptions, cost centers, etc.
        self.enrich_line_items(&mut entry);

        // Apply persona-based errors if enabled and it's a human user
        if self.persona_errors_enabled && !is_automated {
            self.maybe_inject_persona_error(&mut entry);
        }

        // Apply approval workflow if enabled and amount exceeds threshold
        if self.approval_enabled {
            self.maybe_apply_approval_workflow(&mut entry, posting_date);
        }

        // Populate approved_by / approval_date from the approval workflow
        self.populate_approval_fields(&mut entry, posting_date);

        // Maybe start a batch of similar entries for realism
        self.maybe_start_batch(&entry);

        entry
    }

    /// Enable or disable persona-based error injection.
    ///
    /// When enabled, entries created by human personas have a chance
    /// to contain realistic human errors based on their experience level.
    pub fn with_persona_errors(mut self, enabled: bool) -> Self {
        self.persona_errors_enabled = enabled;
        self
    }

    /// Set fraud configuration for fraud injection.
    ///
    /// When fraud is enabled in the config, transactions have a chance
    /// to be marked as fraudulent based on the configured fraud rate.
    pub fn with_fraud_config(mut self, config: FraudConfig) -> Self {
        self.fraud_config = config;
        self
    }

    /// Check if persona errors are enabled.
    pub fn persona_errors_enabled(&self) -> bool {
        self.persona_errors_enabled
    }

    /// Enable or disable batch processing behavior.
    ///
    /// When enabled (default), the generator will occasionally produce batches
    /// of similar entries, simulating how humans batch similar work together.
    pub fn with_batching(mut self, enabled: bool) -> Self {
        if !enabled {
            self.batch_state = None;
        }
        self
    }

    /// Check if batch processing is enabled.
    pub fn batching_enabled(&self) -> bool {
        // Batching is implicitly enabled when not explicitly disabled
        true
    }

    /// Maybe start a batch based on the current entry.
    ///
    /// Humans often batch similar work: processing invoices from one vendor,
    /// entering expense reports for a trip, reconciling similar items.
    fn maybe_start_batch(&mut self, entry: &JournalEntry) {
        // Only start batch for non-automated, non-fraud entries
        if entry.header.source == TransactionSource::Automated || entry.header.is_fraud {
            return;
        }

        // 15% chance to start a batch (most work is not batched)
        if self.rng.random::<f64>() > 0.15 {
            return;
        }

        // Extract key attributes for batching
        let base_account = entry
            .lines
            .first()
            .map(|l| l.gl_account.clone())
            .unwrap_or_default();

        let base_amount = entry.total_debit();

        self.batch_state = Some(BatchState {
            base_account_number: base_account,
            base_amount,
            base_business_process: entry.header.business_process,
            base_posting_date: entry.header.posting_date,
            remaining: self.rng.random_range(2..7), // 2-6 more similar entries
        });
    }

    /// Generate an entry that's part of the current batch.
    ///
    /// Batched entries have:
    /// - Same or very similar business process
    /// - Same posting date (batched work done together)
    /// - Similar amounts (within ±15%)
    /// - Same debit account (processing similar items)
    fn generate_batched_entry(&mut self) -> JournalEntry {
        use rust_decimal::Decimal;

        // Decrement batch counter
        if let Some(ref mut state) = self.batch_state {
            state.remaining = state.remaining.saturating_sub(1);
        }

        let Some(batch) = self.batch_state.clone() else {
            // This is a programming error - batch_state should be set before calling this method.
            // Clear state and fall back to generating a standard entry instead of panicking.
            tracing::warn!(
                "generate_batched_entry called without batch_state; generating standard entry"
            );
            self.batch_state = None;
            return self.generate();
        };

        // Use the batch's posting date (work done on same day)
        let posting_date = batch.base_posting_date;

        self.count += 1;
        let document_id = self.generate_deterministic_uuid();

        // Select same company (batched work is usually same company)
        let company_code = self.company_selector.select(&mut self.rng).to_string();

        // Use simplified line spec for batched entries (usually 2-line)
        let _line_spec = LineItemSpec {
            total_count: 2,
            debit_count: 1,
            credit_count: 1,
            split_type: DebitCreditSplit::Equal,
        };

        // Batched entries are always manual
        let source = TransactionSource::Manual;

        // Use the batch's business process
        let business_process = batch.base_business_process.unwrap_or(BusinessProcess::R2R);

        // Sample time
        let time = self.temporal_sampler.sample_time(true);
        let created_at = posting_date.and_time(time).and_utc();

        // Same user for batched work
        let (created_by, user_persona) = self.select_user(false);

        // Create header
        let mut header =
            JournalEntryHeader::with_deterministic_id(company_code, posting_date, document_id);
        header.created_at = created_at;
        header.source = source;
        header.created_by = created_by;
        header.user_persona = user_persona;
        header.business_process = Some(business_process);
        header.document_type = Self::document_type_for_process(business_process).to_string();

        // Batched manual entries have Manual source document
        header.source_document = Some(DocumentRef::Manual);

        // Generate similar amount (within ±15% of base)
        let variation = self.rng.random_range(-0.15..0.15);
        let varied_amount =
            batch.base_amount * (Decimal::ONE + Decimal::try_from(variation).unwrap_or_default());
        let total_amount = varied_amount.round_dp(2).max(Decimal::from(1));

        // Create the entry
        let mut entry = JournalEntry::new(header);

        // Use same debit account as batch base
        let debit_line = JournalEntryLine::debit(
            entry.header.document_id,
            1,
            batch.base_account_number.clone(),
            total_amount,
        );
        entry.add_line(debit_line);

        // Select a credit account
        let credit_account = self.select_credit_account().account_number.clone();
        let credit_line =
            JournalEntryLine::credit(entry.header.document_id, 2, credit_account, total_amount);
        entry.add_line(credit_line);

        // Enrich line items with account descriptions, cost centers, etc.
        self.enrich_line_items(&mut entry);

        // Apply persona-based errors if enabled
        if self.persona_errors_enabled {
            self.maybe_inject_persona_error(&mut entry);
        }

        // Apply approval workflow if enabled
        if self.approval_enabled {
            self.maybe_apply_approval_workflow(&mut entry, posting_date);
        }

        // Populate approved_by / approval_date from the approval workflow
        self.populate_approval_fields(&mut entry, posting_date);

        // Clear batch state if no more entries remaining
        if batch.remaining <= 1 {
            self.batch_state = None;
        }

        entry
    }

    /// Maybe inject a persona-appropriate error based on the persona's error rate.
    fn maybe_inject_persona_error(&mut self, entry: &mut JournalEntry) {
        // Parse persona from the entry header
        let persona_str = &entry.header.user_persona;
        let persona = match persona_str.to_lowercase().as_str() {
            s if s.contains("junior") => UserPersona::JuniorAccountant,
            s if s.contains("senior") => UserPersona::SeniorAccountant,
            s if s.contains("controller") => UserPersona::Controller,
            s if s.contains("manager") => UserPersona::Manager,
            s if s.contains("executive") => UserPersona::Executive,
            _ => return, // Don't inject errors for unknown personas
        };

        // Get base error rate from persona
        let base_error_rate = persona.error_rate();

        // Apply stress factors based on posting date
        let adjusted_rate = self.apply_stress_factors(base_error_rate, entry.header.posting_date);

        // Check if error should occur based on adjusted rate
        if self.rng.random::<f64>() >= adjusted_rate {
            return; // No error this time
        }

        // Select and inject persona-appropriate error
        self.inject_human_error(entry, persona);
    }

    /// Apply contextual stress factors to the base error rate.
    ///
    /// Stress factors increase error likelihood during:
    /// - Month-end (day >= 28): 1.5x more errors due to deadline pressure
    /// - Quarter-end (Mar, Jun, Sep, Dec): additional 25% boost
    /// - Year-end (December 28-31): 2.0x more errors due to audit pressure
    /// - Monday morning (catch-up work): 20% more errors
    /// - Friday afternoon (rushing to leave): 30% more errors
    fn apply_stress_factors(&self, base_rate: f64, posting_date: chrono::NaiveDate) -> f64 {
        use chrono::Datelike;

        let mut rate = base_rate;
        let day = posting_date.day();
        let month = posting_date.month();

        // Year-end stress (December 28-31): double the error rate
        if month == 12 && day >= 28 {
            rate *= 2.0;
            return rate.min(0.5); // Cap at 50% to keep it realistic
        }

        // Quarter-end stress (last days of Mar, Jun, Sep, Dec)
        if matches!(month, 3 | 6 | 9 | 12) && day >= 28 {
            rate *= 1.75; // 75% more errors at quarter end
            return rate.min(0.4);
        }

        // Month-end stress (last 3 days of month)
        if day >= 28 {
            rate *= 1.5; // 50% more errors at month end
        }

        // Day-of-week stress effects
        let weekday = posting_date.weekday();
        match weekday {
            chrono::Weekday::Mon => {
                // Monday: catching up, often rushed
                rate *= 1.2;
            }
            chrono::Weekday::Fri => {
                // Friday: rushing to finish before weekend
                rate *= 1.3;
            }
            _ => {}
        }

        // Cap at 40% to keep it realistic
        rate.min(0.4)
    }

    /// Apply human-like variation to an amount.
    ///
    /// Humans don't enter perfectly calculated amounts - they:
    /// - Round amounts differently
    /// - Estimate instead of calculating exactly
    /// - Make small input variations
    ///
    /// This applies small variations (typically ±2%) to make amounts more realistic.
    fn apply_human_variation(&mut self, amount: rust_decimal::Decimal) -> rust_decimal::Decimal {
        use rust_decimal::Decimal;

        // Automated transactions or very small amounts don't get variation
        if amount < Decimal::from(10) {
            return amount;
        }

        // 70% chance of human variation being applied
        if self.rng.random::<f64>() > 0.70 {
            return amount;
        }

        // Decide which type of human variation to apply
        let variation_type: u8 = self.rng.random_range(0..4);

        match variation_type {
            0 => {
                // ±2% variation (common for estimated amounts)
                let variation_pct = self.rng.random_range(-0.02..0.02);
                let variation = amount * Decimal::try_from(variation_pct).unwrap_or_default();
                (amount + variation).round_dp(2)
            }
            1 => {
                // Round to nearest $10
                let ten = Decimal::from(10);
                (amount / ten).round() * ten
            }
            2 => {
                // Round to nearest $100 (for larger amounts)
                if amount >= Decimal::from(500) {
                    let hundred = Decimal::from(100);
                    (amount / hundred).round() * hundred
                } else {
                    amount
                }
            }
            3 => {
                // Slight under/over payment (±$0.01 to ±$1.00)
                let cents = Decimal::new(self.rng.random_range(-100..100), 2);
                (amount + cents).max(Decimal::ZERO).round_dp(2)
            }
            _ => amount,
        }
    }

    /// Rebalance an entry after a one-sided amount modification.
    ///
    /// When an error modifies one line's amount, this finds a line on the opposite
    /// side (credit if modified was debit, or vice versa) and adjusts it by the
    /// same impact to maintain balance.
    fn rebalance_entry(entry: &mut JournalEntry, modified_was_debit: bool, impact: Decimal) {
        // Find a line on the opposite side to adjust
        let balancing_idx = entry.lines.iter().position(|l| {
            if modified_was_debit {
                l.credit_amount > Decimal::ZERO
            } else {
                l.debit_amount > Decimal::ZERO
            }
        });

        if let Some(idx) = balancing_idx {
            if modified_was_debit {
                entry.lines[idx].credit_amount += impact;
            } else {
                entry.lines[idx].debit_amount += impact;
            }
        }
    }

    /// Inject a human-like error based on the persona.
    ///
    /// All error types maintain balance - amount modifications are applied to both sides.
    /// Entries are marked with [HUMAN_ERROR:*] tags in header_text for ML detection.
    fn inject_human_error(&mut self, entry: &mut JournalEntry, persona: UserPersona) {
        use rust_decimal::Decimal;

        // Different personas make different types of errors
        let error_type: u8 = match persona {
            UserPersona::JuniorAccountant => {
                // Junior accountants make more varied errors
                self.rng.random_range(0..5)
            }
            UserPersona::SeniorAccountant => {
                // Senior accountants mainly make transposition errors
                self.rng.random_range(0..3)
            }
            UserPersona::Controller | UserPersona::Manager => {
                // Controllers/managers mainly make rounding or cutoff errors
                self.rng.random_range(3..5)
            }
            _ => return,
        };

        match error_type {
            0 => {
                // Transposed digits in an amount
                if let Some(line) = entry.lines.get_mut(0) {
                    let is_debit = line.debit_amount > Decimal::ZERO;
                    let original_amount = if is_debit {
                        line.debit_amount
                    } else {
                        line.credit_amount
                    };

                    // Simple digit swap in the string representation
                    let s = original_amount.to_string();
                    if s.len() >= 2 {
                        let chars: Vec<char> = s.chars().collect();
                        let pos = self.rng.random_range(0..chars.len().saturating_sub(1));
                        if chars[pos].is_ascii_digit()
                            && chars.get(pos + 1).is_some_and(char::is_ascii_digit)
                        {
                            let mut new_chars = chars;
                            new_chars.swap(pos, pos + 1);
                            if let Ok(new_amount) =
                                new_chars.into_iter().collect::<String>().parse::<Decimal>()
                            {
                                let impact = new_amount - original_amount;

                                // Apply to the modified line
                                if is_debit {
                                    entry.lines[0].debit_amount = new_amount;
                                } else {
                                    entry.lines[0].credit_amount = new_amount;
                                }

                                // Rebalance the entry
                                Self::rebalance_entry(entry, is_debit, impact);

                                entry.header.header_text = Some(
                                    entry.header.header_text.clone().unwrap_or_default()
                                        + " [HUMAN_ERROR:TRANSPOSITION]",
                                );
                            }
                        }
                    }
                }
            }
            1 => {
                // Wrong decimal place (off by factor of 10)
                if let Some(line) = entry.lines.get_mut(0) {
                    let is_debit = line.debit_amount > Decimal::ZERO;
                    let original_amount = if is_debit {
                        line.debit_amount
                    } else {
                        line.credit_amount
                    };

                    let new_amount = original_amount * Decimal::new(10, 0);
                    let impact = new_amount - original_amount;

                    // Apply to the modified line
                    if is_debit {
                        entry.lines[0].debit_amount = new_amount;
                    } else {
                        entry.lines[0].credit_amount = new_amount;
                    }

                    // Rebalance the entry
                    Self::rebalance_entry(entry, is_debit, impact);

                    entry.header.header_text = Some(
                        entry.header.header_text.clone().unwrap_or_default()
                            + " [HUMAN_ERROR:DECIMAL_SHIFT]",
                    );
                }
            }
            2 => {
                // Typo in description (doesn't affect balance)
                if let Some(ref mut text) = entry.header.header_text {
                    let typos = ["teh", "adn", "wiht", "taht", "recieve"];
                    let correct = ["the", "and", "with", "that", "receive"];
                    let idx = self.rng.random_range(0..typos.len());
                    if text.to_lowercase().contains(correct[idx]) {
                        *text = text.replace(correct[idx], typos[idx]);
                        *text = format!("{text} [HUMAN_ERROR:TYPO]");
                    }
                }
            }
            3 => {
                // Rounding to round number
                if let Some(line) = entry.lines.get_mut(0) {
                    let is_debit = line.debit_amount > Decimal::ZERO;
                    let original_amount = if is_debit {
                        line.debit_amount
                    } else {
                        line.credit_amount
                    };

                    let new_amount =
                        (original_amount / Decimal::new(100, 0)).round() * Decimal::new(100, 0);
                    let impact = new_amount - original_amount;

                    // Apply to the modified line
                    if is_debit {
                        entry.lines[0].debit_amount = new_amount;
                    } else {
                        entry.lines[0].credit_amount = new_amount;
                    }

                    // Rebalance the entry
                    Self::rebalance_entry(entry, is_debit, impact);

                    entry.header.header_text = Some(
                        entry.header.header_text.clone().unwrap_or_default()
                            + " [HUMAN_ERROR:ROUNDED]",
                    );
                }
            }
            4 => {
                // Late posting marker (document date much earlier than posting date)
                // This doesn't create an imbalance
                if entry.header.document_date == entry.header.posting_date {
                    let days_late = self.rng.random_range(5..15);
                    entry.header.document_date =
                        entry.header.posting_date - chrono::Duration::days(days_late);
                    entry.header.header_text = Some(
                        entry.header.header_text.clone().unwrap_or_default()
                            + " [HUMAN_ERROR:LATE_POSTING]",
                    );
                }
            }
            _ => {}
        }
    }

    /// Apply approval workflow for high-value transactions.
    ///
    /// If the entry amount exceeds the approval threshold, simulate an
    /// approval workflow with appropriate approvers based on amount.
    fn maybe_apply_approval_workflow(
        &mut self,
        entry: &mut JournalEntry,
        _posting_date: NaiveDate,
    ) {
        use rust_decimal::Decimal;

        let amount = entry.total_debit();

        // Skip if amount is below threshold
        if amount <= self.approval_threshold {
            // Auto-approved below threshold
            let workflow = ApprovalWorkflow::auto_approved(
                entry.header.created_by.clone(),
                entry.header.user_persona.clone(),
                amount,
                entry.header.created_at,
            );
            entry.header.approval_workflow = Some(workflow);
            return;
        }

        // Mark as SOX relevant for high-value transactions
        entry.header.sox_relevant = true;

        // Determine required approval levels based on amount
        let required_levels = if amount > Decimal::new(100000, 0) {
            3 // Executive approval required
        } else if amount > Decimal::new(50000, 0) {
            2 // Senior management approval
        } else {
            1 // Manager approval
        };

        // Create the approval workflow
        let mut workflow = ApprovalWorkflow::new(
            entry.header.created_by.clone(),
            entry.header.user_persona.clone(),
            amount,
        );
        workflow.required_levels = required_levels;

        // Simulate submission
        let submit_time = entry.header.created_at;
        let submit_action = ApprovalAction::new(
            entry.header.created_by.clone(),
            entry.header.user_persona.clone(),
            self.parse_persona(&entry.header.user_persona),
            ApprovalActionType::Submit,
            0,
        )
        .with_timestamp(submit_time);

        workflow.actions.push(submit_action);
        workflow.status = ApprovalStatus::Pending;
        workflow.submitted_at = Some(submit_time);

        // Simulate approvals with realistic delays
        let mut current_time = submit_time;
        for level in 1..=required_levels {
            // Add delay for approval (1-3 business hours per level)
            let delay_hours = self.rng.random_range(1..4);
            current_time += chrono::Duration::hours(delay_hours);

            // Skip weekends
            while current_time.weekday() == chrono::Weekday::Sat
                || current_time.weekday() == chrono::Weekday::Sun
            {
                current_time += chrono::Duration::days(1);
            }

            // Generate approver based on level
            let (approver_id, approver_role) = self.select_approver(level);

            let approve_action = ApprovalAction::new(
                approver_id.clone(),
                approver_role.to_string(),
                approver_role,
                ApprovalActionType::Approve,
                level,
            )
            .with_timestamp(current_time);

            workflow.actions.push(approve_action);
            workflow.current_level = level;
        }

        // Mark as approved
        workflow.status = ApprovalStatus::Approved;
        workflow.approved_at = Some(current_time);

        entry.header.approval_workflow = Some(workflow);
    }

    /// Select an approver based on the required level.
    fn select_approver(&mut self, level: u8) -> (String, UserPersona) {
        let persona = match level {
            1 => UserPersona::Manager,
            2 => UserPersona::Controller,
            _ => UserPersona::Executive,
        };

        // Try to get from user pool first
        if let Some(ref pool) = self.user_pool {
            if let Some(user) = pool.get_random_user(persona, &mut self.rng) {
                return (user.user_id.clone(), persona);
            }
        }

        // Fallback to generated approver
        let approver_id = match persona {
            UserPersona::Manager => format!("MGR{:04}", self.rng.random_range(1..100)),
            UserPersona::Controller => format!("CTRL{:04}", self.rng.random_range(1..20)),
            UserPersona::Executive => format!("EXEC{:04}", self.rng.random_range(1..10)),
            _ => format!("USR{:04}", self.rng.random_range(1..1000)),
        };

        (approver_id, persona)
    }

    /// Parse user persona from string.
    fn parse_persona(&self, persona_str: &str) -> UserPersona {
        match persona_str.to_lowercase().as_str() {
            s if s.contains("junior") => UserPersona::JuniorAccountant,
            s if s.contains("senior") => UserPersona::SeniorAccountant,
            s if s.contains("controller") => UserPersona::Controller,
            s if s.contains("manager") => UserPersona::Manager,
            s if s.contains("executive") => UserPersona::Executive,
            s if s.contains("automated") || s.contains("system") => UserPersona::AutomatedSystem,
            _ => UserPersona::JuniorAccountant, // Default
        }
    }

    /// Enable or disable approval workflow.
    pub fn with_approval(mut self, enabled: bool) -> Self {
        self.approval_enabled = enabled;
        self
    }

    /// Set the approval threshold amount.
    pub fn with_approval_threshold(mut self, threshold: rust_decimal::Decimal) -> Self {
        self.approval_threshold = threshold;
        self
    }

    /// Set the SOD violation rate for approval tracking.
    ///
    /// When a transaction is approved, there is a `rate` probability (0.0 to 1.0)
    /// that the approver is the same as the creator, which constitutes a SOD violation.
    /// Default is 0.10 (10%).
    pub fn with_sod_violation_rate(mut self, rate: f64) -> Self {
        self.sod_violation_rate = rate;
        self
    }

    /// Populate `approved_by` and `approval_date` from the approval workflow,
    /// and flag SOD violations when the approver matches the creator.
    fn populate_approval_fields(&mut self, entry: &mut JournalEntry, posting_date: NaiveDate) {
        if let Some(ref workflow) = entry.header.approval_workflow {
            // Extract the last approver from the workflow actions
            let last_approver = workflow
                .actions
                .iter()
                .rev()
                .find(|a| matches!(a.action, ApprovalActionType::Approve));

            if let Some(approver_action) = last_approver {
                entry.header.approved_by = Some(approver_action.actor_id.clone());
                entry.header.approval_date = Some(approver_action.action_timestamp.date_naive());
            } else {
                // No explicit approver (auto-approved); use the preparer
                entry.header.approved_by = Some(workflow.preparer_id.clone());
                entry.header.approval_date = Some(posting_date);
            }

            // Inject SOD violation: with configured probability, set approver = creator
            if self.rng.random::<f64>() < self.sod_violation_rate {
                let creator = entry.header.created_by.clone();
                entry.header.approved_by = Some(creator);
                entry.header.sod_violation = true;
                entry.header.sod_conflict_type =
                    Some(SodConflictType::PreparerApprover);
            }
        }
    }

    /// Set the temporal drift controller for simulating distribution changes over time.
    ///
    /// When drift is enabled, amounts and other distributions will shift based on
    /// the period (month) to simulate realistic temporal evolution like inflation
    /// or increasing fraud rates.
    pub fn with_drift_controller(mut self, controller: DriftController) -> Self {
        self.drift_controller = Some(controller);
        self
    }

    /// Set drift configuration directly.
    ///
    /// Creates a drift controller from the config. Total periods is calculated
    /// from the date range.
    pub fn with_drift_config(mut self, config: DriftConfig, seed: u64) -> Self {
        if config.enabled {
            let total_periods = self.calculate_total_periods();
            self.drift_controller = Some(DriftController::new(config, seed, total_periods));
        }
        self
    }

    /// Calculate total periods (months) in the date range.
    fn calculate_total_periods(&self) -> u32 {
        let start_year = self.start_date.year();
        let start_month = self.start_date.month();
        let end_year = self.end_date.year();
        let end_month = self.end_date.month();

        ((end_year - start_year) * 12 + (end_month as i32 - start_month as i32) + 1).max(1) as u32
    }

    /// Calculate the period number (0-indexed) for a given date.
    fn date_to_period(&self, date: NaiveDate) -> u32 {
        let start_year = self.start_date.year();
        let start_month = self.start_date.month() as i32;
        let date_year = date.year();
        let date_month = date.month() as i32;

        ((date_year - start_year) * 12 + (date_month - start_month)).max(0) as u32
    }

    /// Get drift adjustments for a given date.
    fn get_drift_adjustments(&self, date: NaiveDate) -> DriftAdjustments {
        if let Some(ref controller) = self.drift_controller {
            let period = self.date_to_period(date);
            controller.compute_adjustments(period)
        } else {
            DriftAdjustments::none()
        }
    }

    /// Select a user from the pool or generate a generic user ID.
    #[inline]
    fn select_user(&mut self, is_automated: bool) -> (String, String) {
        if let Some(ref pool) = self.user_pool {
            let persona = if is_automated {
                UserPersona::AutomatedSystem
            } else {
                // Random distribution among human personas
                let roll: f64 = self.rng.random();
                if roll < 0.4 {
                    UserPersona::JuniorAccountant
                } else if roll < 0.7 {
                    UserPersona::SeniorAccountant
                } else if roll < 0.85 {
                    UserPersona::Controller
                } else {
                    UserPersona::Manager
                }
            };

            if let Some(user) = pool.get_random_user(persona, &mut self.rng) {
                return (user.user_id.clone(), user.persona.to_string());
            }
        }

        // Fallback to generic format
        if is_automated {
            (
                format!("BATCH{:04}", self.rng.random_range(1..=20)),
                "automated_system".to_string(),
            )
        } else {
            (
                format!("USER{:04}", self.rng.random_range(1..=40)),
                "senior_accountant".to_string(),
            )
        }
    }

    /// Select transaction source based on configuration weights.
    #[inline]
    fn select_source(&mut self) -> TransactionSource {
        let roll: f64 = self.rng.random();
        let dist = &self.config.source_distribution;

        if roll < dist.manual {
            TransactionSource::Manual
        } else if roll < dist.manual + dist.automated {
            TransactionSource::Automated
        } else if roll < dist.manual + dist.automated + dist.recurring {
            TransactionSource::Recurring
        } else {
            TransactionSource::Adjustment
        }
    }

    /// Select a business process based on configuration weights.
    #[inline]
    /// Map a business process to a SAP-style document type code.
    ///
    /// - P2P → "KR" (vendor invoice)
    /// - O2C → "DR" (customer invoice)
    /// - R2R → "SA" (general journal)
    /// - H2R → "HR" (HR posting)
    /// - A2R → "AA" (asset posting)
    /// - others → "SA"
    fn document_type_for_process(process: BusinessProcess) -> &'static str {
        match process {
            BusinessProcess::P2P => "KR",
            BusinessProcess::O2C => "DR",
            BusinessProcess::R2R => "SA",
            BusinessProcess::H2R => "HR",
            BusinessProcess::A2R => "AA",
            _ => "SA",
        }
    }

    fn select_business_process(&mut self) -> BusinessProcess {
        let roll: f64 = self.rng.random();

        // Default weights: O2C=35%, P2P=30%, R2R=20%, H2R=10%, A2R=5%
        if roll < 0.35 {
            BusinessProcess::O2C
        } else if roll < 0.65 {
            BusinessProcess::P2P
        } else if roll < 0.85 {
            BusinessProcess::R2R
        } else if roll < 0.95 {
            BusinessProcess::H2R
        } else {
            BusinessProcess::A2R
        }
    }

    #[inline]
    fn select_debit_account(&mut self) -> &GLAccount {
        let accounts = self.coa.get_accounts_by_type(AccountType::Asset);
        let expense_accounts = self.coa.get_accounts_by_type(AccountType::Expense);

        // 60% asset, 40% expense for debits
        let all: Vec<_> = if self.rng.random::<f64>() < 0.6 {
            accounts
        } else {
            expense_accounts
        };

        all.choose(&mut self.rng).copied().unwrap_or_else(|| {
            tracing::warn!(
                "Account selection returned empty list, falling back to first COA account"
            );
            &self.coa.accounts[0]
        })
    }

    #[inline]
    fn select_credit_account(&mut self) -> &GLAccount {
        let liability_accounts = self.coa.get_accounts_by_type(AccountType::Liability);
        let revenue_accounts = self.coa.get_accounts_by_type(AccountType::Revenue);

        // 60% liability, 40% revenue for credits
        let all: Vec<_> = if self.rng.random::<f64>() < 0.6 {
            liability_accounts
        } else {
            revenue_accounts
        };

        all.choose(&mut self.rng).copied().unwrap_or_else(|| {
            tracing::warn!(
                "Account selection returned empty list, falling back to first COA account"
            );
            &self.coa.accounts[0]
        })
    }
}

impl Generator for JournalEntryGenerator {
    type Item = JournalEntry;
    type Config = (
        TransactionConfig,
        Arc<ChartOfAccounts>,
        Vec<String>,
        NaiveDate,
        NaiveDate,
    );

    fn new(config: Self::Config, seed: u64) -> Self {
        Self::new_with_params(config.0, config.1, config.2, config.3, config.4, seed)
    }

    fn generate_one(&mut self) -> Self::Item {
        self.generate()
    }

    fn reset(&mut self) {
        self.rng = seeded_rng(self.seed, 0);
        self.line_sampler.reset(self.seed + 1);
        self.amount_sampler.reset(self.seed + 2);
        self.temporal_sampler.reset(self.seed + 3);
        self.count = 0;
        self.uuid_factory.reset();

        // Reset reference generator by recreating it
        let mut ref_gen = ReferenceGenerator::new(
            self.start_date.year(),
            self.companies
                .first()
                .map(std::string::String::as_str)
                .unwrap_or("1000"),
        );
        ref_gen.set_prefix(
            ReferenceType::Invoice,
            &self.template_config.references.invoice_prefix,
        );
        ref_gen.set_prefix(
            ReferenceType::PurchaseOrder,
            &self.template_config.references.po_prefix,
        );
        ref_gen.set_prefix(
            ReferenceType::SalesOrder,
            &self.template_config.references.so_prefix,
        );
        self.reference_generator = ref_gen;
    }

    fn count(&self) -> u64 {
        self.count
    }

    fn seed(&self) -> u64 {
        self.seed
    }
}

use datasynth_core::traits::ParallelGenerator;

impl ParallelGenerator for JournalEntryGenerator {
    /// Split this generator into `parts` independent sub-generators.
    ///
    /// Each sub-generator gets a deterministic seed derived from the parent seed
    /// and its partition index, plus a partitioned UUID factory to avoid contention.
    /// The results are deterministic for a given partition count.
    fn split(self, parts: usize) -> Vec<Self> {
        let parts = parts.max(1);
        (0..parts)
            .map(|i| {
                // Derive a unique seed per partition using a golden-ratio constant
                let sub_seed = self
                    .seed
                    .wrapping_add((i as u64).wrapping_mul(0x9E3779B97F4A7C15));

                let mut gen = JournalEntryGenerator::new_with_full_config(
                    self.config.clone(),
                    Arc::clone(&self.coa),
                    self.companies.clone(),
                    self.start_date,
                    self.end_date,
                    sub_seed,
                    self.template_config.clone(),
                    self.user_pool.clone(),
                );

                // Copy over configuration state
                gen.company_selector = self.company_selector.clone();
                gen.vendor_pool = self.vendor_pool.clone();
                gen.customer_pool = self.customer_pool.clone();
                gen.material_pool = self.material_pool.clone();
                gen.using_real_master_data = self.using_real_master_data;
                gen.fraud_config = self.fraud_config.clone();
                gen.persona_errors_enabled = self.persona_errors_enabled;
                gen.approval_enabled = self.approval_enabled;
                gen.approval_threshold = self.approval_threshold;
                gen.sod_violation_rate = self.sod_violation_rate;

                // Use partitioned UUID factory to eliminate atomic contention
                gen.uuid_factory = DeterministicUuidFactory::for_partition(
                    sub_seed,
                    GeneratorType::JournalEntry,
                    i as u8,
                );

                // Copy temporal patterns if configured
                if let Some(ref config) = self.temporal_patterns_config {
                    gen.temporal_patterns_config = Some(config.clone());
                    // Rebuild business day calculator from the stored config
                    if config.business_days.enabled {
                        if let Some(ref bdc) = self.business_day_calculator {
                            gen.business_day_calculator = Some(bdc.clone());
                        }
                    }
                    // Rebuild processing lag calculator with partition seed
                    if config.processing_lags.enabled {
                        let lag_config =
                            Self::convert_processing_lag_config(&config.processing_lags);
                        gen.processing_lag_calculator =
                            Some(ProcessingLagCalculator::with_config(sub_seed, lag_config));
                    }
                }

                // Copy drift controller if present
                if let Some(ref dc) = self.drift_controller {
                    gen.drift_controller = Some(dc.clone());
                }

                gen
            })
            .collect()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::ChartOfAccountsGenerator;

    #[test]
    fn test_generate_balanced_entries() {
        let mut coa_gen =
            ChartOfAccountsGenerator::new(CoAComplexity::Small, IndustrySector::Manufacturing, 42);
        let coa = Arc::new(coa_gen.generate());

        let mut je_gen = JournalEntryGenerator::new_with_params(
            TransactionConfig::default(),
            coa,
            vec!["1000".to_string()],
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            42,
        );

        let mut balanced_count = 0;
        for _ in 0..100 {
            let entry = je_gen.generate();

            // Skip entries with human errors as they may be intentionally unbalanced
            let has_human_error = entry
                .header
                .header_text
                .as_ref()
                .map(|t| t.contains("[HUMAN_ERROR:"))
                .unwrap_or(false);

            if !has_human_error {
                assert!(
                    entry.is_balanced(),
                    "Entry {:?} is not balanced",
                    entry.header.document_id
                );
                balanced_count += 1;
            }
            assert!(entry.line_count() >= 2, "Entry has fewer than 2 lines");
        }

        // Ensure most entries are balanced (human errors are rare)
        assert!(
            balanced_count >= 80,
            "Expected at least 80 balanced entries, got {}",
            balanced_count
        );
    }

    #[test]
    fn test_deterministic_generation() {
        let mut coa_gen =
            ChartOfAccountsGenerator::new(CoAComplexity::Small, IndustrySector::Manufacturing, 42);
        let coa = Arc::new(coa_gen.generate());

        let mut gen1 = JournalEntryGenerator::new_with_params(
            TransactionConfig::default(),
            Arc::clone(&coa),
            vec!["1000".to_string()],
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            42,
        );

        let mut gen2 = JournalEntryGenerator::new_with_params(
            TransactionConfig::default(),
            coa,
            vec!["1000".to_string()],
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            42,
        );

        for _ in 0..50 {
            let e1 = gen1.generate();
            let e2 = gen2.generate();
            assert_eq!(e1.header.document_id, e2.header.document_id);
            assert_eq!(e1.total_debit(), e2.total_debit());
        }
    }

    #[test]
    fn test_templates_generate_descriptions() {
        let mut coa_gen =
            ChartOfAccountsGenerator::new(CoAComplexity::Small, IndustrySector::Manufacturing, 42);
        let coa = Arc::new(coa_gen.generate());

        // Enable all template features
        let template_config = TemplateConfig {
            names: datasynth_config::schema::NameTemplateConfig {
                generate_realistic_names: true,
                email_domain: "test.com".to_string(),
                culture_distribution: datasynth_config::schema::CultureDistribution::default(),
            },
            descriptions: datasynth_config::schema::DescriptionTemplateConfig {
                generate_header_text: true,
                generate_line_text: true,
            },
            references: datasynth_config::schema::ReferenceTemplateConfig {
                generate_references: true,
                invoice_prefix: "TEST-INV".to_string(),
                po_prefix: "TEST-PO".to_string(),
                so_prefix: "TEST-SO".to_string(),
            },
        };

        let mut je_gen = JournalEntryGenerator::new_with_full_config(
            TransactionConfig::default(),
            coa,
            vec!["1000".to_string()],
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            42,
            template_config,
            None,
        )
        .with_persona_errors(false); // Disable for template testing

        for _ in 0..10 {
            let entry = je_gen.generate();

            // Verify header text is populated
            assert!(
                entry.header.header_text.is_some(),
                "Header text should be populated"
            );

            // Verify reference is populated
            assert!(
                entry.header.reference.is_some(),
                "Reference should be populated"
            );

            // Verify business process is set
            assert!(
                entry.header.business_process.is_some(),
                "Business process should be set"
            );

            // Verify line text is populated
            for line in &entry.lines {
                assert!(line.line_text.is_some(), "Line text should be populated");
            }

            // Entry should still be balanced
            assert!(entry.is_balanced());
        }
    }

    #[test]
    fn test_user_pool_integration() {
        let mut coa_gen =
            ChartOfAccountsGenerator::new(CoAComplexity::Small, IndustrySector::Manufacturing, 42);
        let coa = Arc::new(coa_gen.generate());

        let companies = vec!["1000".to_string()];

        // Generate user pool
        let mut user_gen = crate::UserGenerator::new(42);
        let user_pool = user_gen.generate_standard(&companies);

        let mut je_gen = JournalEntryGenerator::new_with_full_config(
            TransactionConfig::default(),
            coa,
            companies,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            42,
            TemplateConfig::default(),
            Some(user_pool),
        );

        // Generate entries and verify user IDs are from pool
        for _ in 0..20 {
            let entry = je_gen.generate();

            // User ID should not be generic BATCH/USER format when pool is used
            // (though it may still fall back if random selection misses)
            assert!(!entry.header.created_by.is_empty());
        }
    }

    #[test]
    fn test_master_data_connection() {
        let mut coa_gen =
            ChartOfAccountsGenerator::new(CoAComplexity::Small, IndustrySector::Manufacturing, 42);
        let coa = Arc::new(coa_gen.generate());

        // Create test vendors
        let vendors = vec![
            Vendor::new("V-TEST-001", "Test Vendor Alpha", VendorType::Supplier),
            Vendor::new("V-TEST-002", "Test Vendor Beta", VendorType::Technology),
        ];

        // Create test customers
        let customers = vec![
            Customer::new("C-TEST-001", "Test Customer One", CustomerType::Corporate),
            Customer::new(
                "C-TEST-002",
                "Test Customer Two",
                CustomerType::SmallBusiness,
            ),
        ];

        // Create test materials
        let materials = vec![Material::new(
            "MAT-TEST-001",
            "Test Material A",
            MaterialType::RawMaterial,
        )];

        // Create generator with master data
        let generator = JournalEntryGenerator::new_with_params(
            TransactionConfig::default(),
            coa,
            vec!["1000".to_string()],
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            42,
        );

        // Without master data
        assert!(!generator.is_using_real_master_data());

        // Connect master data
        let generator_with_data = generator
            .with_vendors(&vendors)
            .with_customers(&customers)
            .with_materials(&materials);

        // Should now be using real master data
        assert!(generator_with_data.is_using_real_master_data());
    }

    #[test]
    fn test_with_master_data_convenience_method() {
        let mut coa_gen =
            ChartOfAccountsGenerator::new(CoAComplexity::Small, IndustrySector::Manufacturing, 42);
        let coa = Arc::new(coa_gen.generate());

        let vendors = vec![Vendor::new("V-001", "Vendor One", VendorType::Supplier)];
        let customers = vec![Customer::new(
            "C-001",
            "Customer One",
            CustomerType::Corporate,
        )];
        let materials = vec![Material::new(
            "MAT-001",
            "Material One",
            MaterialType::RawMaterial,
        )];

        let generator = JournalEntryGenerator::new_with_params(
            TransactionConfig::default(),
            coa,
            vec!["1000".to_string()],
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            42,
        )
        .with_master_data(&vendors, &customers, &materials);

        assert!(generator.is_using_real_master_data());
    }

    #[test]
    fn test_stress_factors_increase_error_rate() {
        let mut coa_gen =
            ChartOfAccountsGenerator::new(CoAComplexity::Small, IndustrySector::Manufacturing, 42);
        let coa = Arc::new(coa_gen.generate());

        let generator = JournalEntryGenerator::new_with_params(
            TransactionConfig::default(),
            coa,
            vec!["1000".to_string()],
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            42,
        );

        let base_rate = 0.1;

        // Regular day - no stress factors
        let regular_day = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(); // Mid-June Wednesday
        let regular_rate = generator.apply_stress_factors(base_rate, regular_day);
        assert!(
            (regular_rate - base_rate).abs() < 0.01,
            "Regular day should have minimal stress factor adjustment"
        );

        // Month end - 50% more errors
        let month_end = NaiveDate::from_ymd_opt(2024, 6, 29).unwrap(); // June 29 (Saturday)
        let month_end_rate = generator.apply_stress_factors(base_rate, month_end);
        assert!(
            month_end_rate > regular_rate,
            "Month end should have higher error rate than regular day"
        );

        // Year end - double the error rate
        let year_end = NaiveDate::from_ymd_opt(2024, 12, 30).unwrap(); // December 30
        let year_end_rate = generator.apply_stress_factors(base_rate, year_end);
        assert!(
            year_end_rate > month_end_rate,
            "Year end should have highest error rate"
        );

        // Friday stress
        let friday = NaiveDate::from_ymd_opt(2024, 6, 14).unwrap(); // Friday
        let friday_rate = generator.apply_stress_factors(base_rate, friday);
        assert!(
            friday_rate > regular_rate,
            "Friday should have higher error rate than mid-week"
        );

        // Monday stress
        let monday = NaiveDate::from_ymd_opt(2024, 6, 17).unwrap(); // Monday
        let monday_rate = generator.apply_stress_factors(base_rate, monday);
        assert!(
            monday_rate > regular_rate,
            "Monday should have higher error rate than mid-week"
        );
    }

    #[test]
    fn test_batching_produces_similar_entries() {
        let mut coa_gen =
            ChartOfAccountsGenerator::new(CoAComplexity::Small, IndustrySector::Manufacturing, 42);
        let coa = Arc::new(coa_gen.generate());

        // Use seed 123 which is more likely to trigger batching
        let mut je_gen = JournalEntryGenerator::new_with_params(
            TransactionConfig::default(),
            coa,
            vec!["1000".to_string()],
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            123,
        )
        .with_persona_errors(false); // Disable to ensure balanced entries

        // Generate many entries - at 15% batch rate, should see some batches
        let entries: Vec<JournalEntry> = (0..200).map(|_| je_gen.generate()).collect();

        // Check that all entries are balanced (batched or not)
        for entry in &entries {
            assert!(
                entry.is_balanced(),
                "All entries including batched should be balanced"
            );
        }

        // Count entries with same-day posting dates (batch indicator)
        let mut date_counts: std::collections::HashMap<NaiveDate, usize> =
            std::collections::HashMap::new();
        for entry in &entries {
            *date_counts.entry(entry.header.posting_date).or_insert(0) += 1;
        }

        // With batching, some dates should have multiple entries
        let dates_with_multiple = date_counts.values().filter(|&&c| c > 1).count();
        assert!(
            dates_with_multiple > 0,
            "With batching, should see some dates with multiple entries"
        );
    }

    #[test]
    fn test_temporal_patterns_business_days() {
        use datasynth_config::schema::{
            BusinessDaySchemaConfig, CalendarSchemaConfig, TemporalPatternsConfig,
        };

        let mut coa_gen =
            ChartOfAccountsGenerator::new(CoAComplexity::Small, IndustrySector::Manufacturing, 42);
        let coa = Arc::new(coa_gen.generate());

        // Create temporal patterns config with business days enabled
        let temporal_config = TemporalPatternsConfig {
            enabled: true,
            business_days: BusinessDaySchemaConfig {
                enabled: true,
                ..Default::default()
            },
            calendars: CalendarSchemaConfig {
                regions: vec!["US".to_string()],
                custom_holidays: vec![],
            },
            ..Default::default()
        };

        let mut je_gen = JournalEntryGenerator::new_with_params(
            TransactionConfig::default(),
            coa,
            vec!["1000".to_string()],
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 3, 31).unwrap(), // Q1 2024
            42,
        )
        .with_temporal_patterns(temporal_config, 42)
        .with_persona_errors(false);

        // Generate entries and verify none fall on weekends
        let entries: Vec<JournalEntry> = (0..100).map(|_| je_gen.generate()).collect();

        for entry in &entries {
            let weekday = entry.header.posting_date.weekday();
            assert!(
                weekday != chrono::Weekday::Sat && weekday != chrono::Weekday::Sun,
                "Posting date {:?} should not be a weekend",
                entry.header.posting_date
            );
        }
    }

    #[test]
    fn test_default_generation_filters_weekends() {
        // Verify that weekend entries are <5% even when temporal_patterns is NOT enabled.
        // This tests the fix where new_with_full_config always creates a default
        // BusinessDayCalculator with US holidays as a fallback.
        let mut coa_gen =
            ChartOfAccountsGenerator::new(CoAComplexity::Small, IndustrySector::Manufacturing, 42);
        let coa = Arc::new(coa_gen.generate());

        let mut je_gen = JournalEntryGenerator::new_with_params(
            TransactionConfig::default(),
            coa,
            vec!["1000".to_string()],
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            42,
        )
        .with_persona_errors(false);

        let total = 500;
        let entries: Vec<JournalEntry> = (0..total).map(|_| je_gen.generate()).collect();

        let weekend_count = entries
            .iter()
            .filter(|e| {
                let wd = e.header.posting_date.weekday();
                wd == chrono::Weekday::Sat || wd == chrono::Weekday::Sun
            })
            .count();

        let weekend_pct = weekend_count as f64 / total as f64;
        assert!(
            weekend_pct < 0.05,
            "Expected weekend entries <5% of total without temporal_patterns enabled, \
             but got {:.1}% ({}/{})",
            weekend_pct * 100.0,
            weekend_count,
            total
        );
    }

    #[test]
    fn test_document_type_derived_from_business_process() {
        let mut coa_gen =
            ChartOfAccountsGenerator::new(CoAComplexity::Small, IndustrySector::Manufacturing, 42);
        let coa = Arc::new(coa_gen.generate());

        let mut je_gen = JournalEntryGenerator::new_with_params(
            TransactionConfig::default(),
            coa,
            vec!["1000".to_string()],
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            99,
        )
        .with_persona_errors(false)
        .with_batching(false);

        let total = 200;
        let mut doc_types = std::collections::HashSet::new();
        let mut sa_count = 0_usize;

        for _ in 0..total {
            let entry = je_gen.generate();
            let dt = &entry.header.document_type;
            doc_types.insert(dt.clone());
            if dt == "SA" {
                sa_count += 1;
            }
        }

        // Should have more than 3 distinct document types
        assert!(
            doc_types.len() > 3,
            "Expected >3 distinct document types, got {} ({:?})",
            doc_types.len(),
            doc_types,
        );

        // "SA" should be less than 50% (R2R is 20% of the weight)
        let sa_pct = sa_count as f64 / total as f64;
        assert!(
            sa_pct < 0.50,
            "Expected SA <50%, got {:.1}% ({}/{})",
            sa_pct * 100.0,
            sa_count,
            total,
        );
    }

    #[test]
    fn test_enrich_line_items_account_description() {
        let mut coa_gen =
            ChartOfAccountsGenerator::new(CoAComplexity::Small, IndustrySector::Manufacturing, 42);
        let coa = Arc::new(coa_gen.generate());

        let mut je_gen = JournalEntryGenerator::new_with_params(
            TransactionConfig::default(),
            coa,
            vec!["1000".to_string()],
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            42,
        )
        .with_persona_errors(false);

        let total = 200;
        let entries: Vec<JournalEntry> = (0..total).map(|_| je_gen.generate()).collect();

        // Count lines with account_description populated
        let total_lines: usize = entries.iter().map(|e| e.lines.len()).sum();
        let lines_with_desc: usize = entries
            .iter()
            .flat_map(|e| &e.lines)
            .filter(|l| l.account_description.is_some())
            .count();

        let desc_pct = lines_with_desc as f64 / total_lines as f64;
        assert!(
            desc_pct > 0.95,
            "Expected >95% of lines to have account_description, got {:.1}% ({}/{})",
            desc_pct * 100.0,
            lines_with_desc,
            total_lines,
        );
    }

    #[test]
    fn test_enrich_line_items_cost_center_for_expense_accounts() {
        let mut coa_gen =
            ChartOfAccountsGenerator::new(CoAComplexity::Small, IndustrySector::Manufacturing, 42);
        let coa = Arc::new(coa_gen.generate());

        let mut je_gen = JournalEntryGenerator::new_with_params(
            TransactionConfig::default(),
            coa,
            vec!["1000".to_string()],
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            42,
        )
        .with_persona_errors(false);

        let total = 300;
        let entries: Vec<JournalEntry> = (0..total).map(|_| je_gen.generate()).collect();

        // Count expense account lines (5xxx/6xxx) with cost_center populated
        let expense_lines: Vec<&JournalEntryLine> = entries
            .iter()
            .flat_map(|e| &e.lines)
            .filter(|l| {
                let first = l.gl_account.chars().next().unwrap_or('0');
                first == '5' || first == '6'
            })
            .collect();

        if !expense_lines.is_empty() {
            let with_cc = expense_lines
                .iter()
                .filter(|l| l.cost_center.is_some())
                .count();
            let cc_pct = with_cc as f64 / expense_lines.len() as f64;
            assert!(
                cc_pct > 0.80,
                "Expected >80% of expense lines to have cost_center, got {:.1}% ({}/{})",
                cc_pct * 100.0,
                with_cc,
                expense_lines.len(),
            );
        }
    }

    #[test]
    fn test_enrich_line_items_profit_center_and_line_text() {
        let mut coa_gen =
            ChartOfAccountsGenerator::new(CoAComplexity::Small, IndustrySector::Manufacturing, 42);
        let coa = Arc::new(coa_gen.generate());

        let mut je_gen = JournalEntryGenerator::new_with_params(
            TransactionConfig::default(),
            coa,
            vec!["1000".to_string()],
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            42,
        )
        .with_persona_errors(false);

        let total = 100;
        let entries: Vec<JournalEntry> = (0..total).map(|_| je_gen.generate()).collect();

        let total_lines: usize = entries.iter().map(|e| e.lines.len()).sum();

        // All lines should have profit_center
        let with_pc = entries
            .iter()
            .flat_map(|e| &e.lines)
            .filter(|l| l.profit_center.is_some())
            .count();
        let pc_pct = with_pc as f64 / total_lines as f64;
        assert!(
            pc_pct > 0.95,
            "Expected >95% of lines to have profit_center, got {:.1}% ({}/{})",
            pc_pct * 100.0,
            with_pc,
            total_lines,
        );

        // All lines should have line_text (either from template or header fallback)
        let with_text = entries
            .iter()
            .flat_map(|e| &e.lines)
            .filter(|l| l.line_text.is_some())
            .count();
        let text_pct = with_text as f64 / total_lines as f64;
        assert!(
            text_pct > 0.95,
            "Expected >95% of lines to have line_text, got {:.1}% ({}/{})",
            text_pct * 100.0,
            with_text,
            total_lines,
        );
    }
}
