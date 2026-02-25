//! Customer generator for banking data.

use chrono::{Datelike, NaiveDate};
use datasynth_core::models::banking::{
    BankingCustomerType, BusinessPersona, RetailPersona, RiskTier, TrustPersona,
};
use datasynth_core::CountryPack;
use datasynth_core::DeterministicUuidFactory;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;

use crate::config::BankingConfig;
use crate::models::{
    BankingCustomer, BeneficialOwner, ControlType, KycProfile, PepCategory, PersonaVariant,
    VerificationStatus,
};

/// Generator for banking customers.
pub struct CustomerGenerator {
    config: BankingConfig,
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
    start_date: NaiveDate,
    end_date: NaiveDate,
    /// Optional country pack for locale-aware phone, address, and ID generation.
    country_pack: Option<CountryPack>,
}

impl CustomerGenerator {
    /// Create a new customer generator.
    pub fn new(config: BankingConfig, seed: u64) -> Self {
        let start_date = NaiveDate::parse_from_str(&config.population.start_date, "%Y-%m-%d")
            .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid default date"));
        let end_date = start_date + chrono::Months::new(config.population.period_months);

        Self {
            config,
            rng: ChaCha8Rng::seed_from_u64(seed),
            uuid_factory: DeterministicUuidFactory::new(
                seed,
                datasynth_core::GeneratorType::Customer,
            ),
            start_date,
            end_date,
            country_pack: None,
        }
    }

    /// Set the country pack for locale-aware data generation.
    ///
    /// When set, phone numbers, addresses, and national IDs will be generated
    /// using country-pack-specific formats and templates.
    pub fn set_country_pack(&mut self, pack: CountryPack) {
        self.country_pack = Some(pack);
    }

    /// Generate all customers.
    pub fn generate_all(&mut self) -> Vec<BankingCustomer> {
        let mut customers = Vec::new();

        // Generate retail customers
        for _ in 0..self.config.population.retail_customers {
            customers.push(self.generate_retail_customer());
        }

        // Generate business customers
        for _ in 0..self.config.population.business_customers {
            customers.push(self.generate_business_customer());
        }

        // Generate trusts
        for _ in 0..self.config.population.trusts {
            customers.push(self.generate_trust_customer());
        }

        // Form households
        self.form_households(&mut customers);

        customers
    }

    /// Generate a single retail customer.
    pub fn generate_retail_customer(&mut self) -> BankingCustomer {
        let customer_id = self.uuid_factory.next();
        let persona = self.select_retail_persona();
        let (first_name, last_name) = self.generate_person_name();
        let country = self.select_country();
        let onboarding_date = self.random_onboarding_date();

        let mut customer = BankingCustomer::new_retail(
            customer_id,
            &first_name,
            &last_name,
            &country,
            onboarding_date,
        )
        .with_persona(PersonaVariant::Retail(persona));

        // Set risk tier based on persona and configuration
        let risk_tier = self.calculate_retail_risk_tier(persona, &country);
        customer.risk_tier = risk_tier;

        // Generate KYC profile
        customer.kyc_profile = self.generate_retail_kyc_profile(persona);

        // Possibly mark as PEP
        if self.rng.random::<f64>() < self.config.compliance.pep_rate {
            customer.is_pep = true;
            customer.pep_category = Some(self.select_pep_category());
            customer.risk_tier = RiskTier::High;
        }

        // Generate contact info — use country pack methods when available
        customer.email = Some(self.generate_email(&first_name, &last_name));
        let pack_clone = self.country_pack.clone();
        customer.phone = Some(if let Some(ref pack) = pack_clone {
            self.generate_phone_from_pack(pack)
        } else {
            self.generate_phone(&country)
        });
        customer.date_of_birth = Some(self.generate_birth_date(persona));

        // Generate address
        let (addr, city, state, postal) = if let Some(ref pack) = pack_clone {
            self.generate_address_from_pack(pack)
        } else {
            self.generate_address(&country)
        };
        customer.address_line1 = Some(addr);
        customer.city = Some(city);
        customer.state = Some(state);
        customer.postal_code = Some(postal);

        // Generate identification documents
        customer.national_id = Some(if let Some(ref pack) = pack_clone {
            self.generate_national_id_from_pack(pack)
        } else {
            self.generate_national_id(&country)
        });
        if self.rng.random::<f64>() < 0.4 {
            customer.passport_number = Some(self.generate_passport_number(&country));
        }

        customer
    }

    /// Generate a single business customer.
    pub fn generate_business_customer(&mut self) -> BankingCustomer {
        let customer_id = self.uuid_factory.next();
        let persona = self.select_business_persona();
        let name = self.generate_business_name(persona);
        let country = self.select_country();
        let onboarding_date = self.random_onboarding_date();

        let mut customer =
            BankingCustomer::new_business(customer_id, &name, &country, onboarding_date)
                .with_persona(PersonaVariant::Business(persona));

        // Set risk tier
        let risk_tier = self.calculate_business_risk_tier(persona, &country);
        customer.risk_tier = risk_tier;

        // Generate KYC profile
        customer.kyc_profile = self.generate_business_kyc_profile(persona);

        // Generate contact info — use country pack methods when available
        customer.email = Some(format!("info@{}.com", name.to_lowercase().replace(' ', "")));
        let pack_clone = self.country_pack.clone();
        customer.phone = Some(if let Some(ref pack) = pack_clone {
            self.generate_phone_from_pack(pack)
        } else {
            self.generate_phone(&country)
        });

        // Set industry
        customer.industry_description = Some(self.get_industry_description(persona));

        // Generate address
        let (addr, city, state, postal) = if let Some(ref pack) = pack_clone {
            self.generate_address_from_pack(pack)
        } else {
            self.generate_address(&country)
        };
        customer.address_line1 = Some(addr);
        customer.city = Some(city);
        customer.state = Some(state);
        customer.postal_code = Some(postal);

        // Generate beneficial owners (business entities must have UBOs)
        let ubo_count = self.rng.random_range(1..=3);
        let mut remaining_pct = 100.0_f64;
        for i in 0..ubo_count {
            let (first, last) = self.generate_person_name();
            let pct = if i == ubo_count - 1 {
                remaining_pct
            } else {
                let share = self
                    .rng
                    .random_range(15.0..=50.0_f64)
                    .min(remaining_pct - 10.0);
                remaining_pct -= share;
                share
            };
            let ubo = BeneficialOwner::new(
                self.uuid_factory.next(),
                &format!("{} {}", first, last),
                &country,
                Decimal::from_f64_retain(pct).unwrap_or(Decimal::from(25)),
            );
            customer.beneficial_owners.push(ubo);
        }

        customer
    }

    /// Generate a trust customer.
    pub fn generate_trust_customer(&mut self) -> BankingCustomer {
        let customer_id = self.uuid_factory.next();
        let persona = self.select_trust_persona();
        let name = self.generate_trust_name(persona);
        let country = self.select_country();
        let onboarding_date = self.random_onboarding_date();

        let mut customer =
            BankingCustomer::new_business(customer_id, &name, &country, onboarding_date)
                .with_persona(PersonaVariant::Trust(persona));

        customer.customer_type = BankingCustomerType::Trust;

        // Trusts are typically higher risk
        customer.risk_tier = RiskTier::High;

        // Generate KYC profile
        customer.kyc_profile = KycProfile::high_net_worth()
            .with_turnover(datasynth_core::models::banking::TurnoverBand::VeryHigh);

        // Generate address — use country pack when available
        let pack_clone = self.country_pack.clone();
        let (addr, city, state, postal) = if let Some(ref pack) = pack_clone {
            self.generate_address_from_pack(pack)
        } else {
            self.generate_address(&country)
        };
        customer.address_line1 = Some(addr);
        customer.city = Some(city);
        customer.state = Some(state);
        customer.postal_code = Some(postal);

        // Generate beneficial owners (trusts always have UBOs)
        let ubo_count = self.rng.random_range(1..=4);
        let mut remaining_pct = 100.0_f64;
        for i in 0..ubo_count {
            let (first, last) = self.generate_person_name();
            let pct = if i == ubo_count - 1 {
                remaining_pct
            } else {
                let share = self
                    .rng
                    .random_range(10.0..=40.0_f64)
                    .min(remaining_pct - 5.0);
                remaining_pct -= share;
                share
            };
            let mut ubo = BeneficialOwner::new(
                self.uuid_factory.next(),
                &format!("{} {}", first, last),
                &country,
                Decimal::from_f64_retain(pct).unwrap_or(Decimal::from(25)),
            );
            ubo.control_type = ControlType::TrustArrangement;
            ubo.verification_status = if self.rng.random::<f64>() < 0.7 {
                VerificationStatus::Verified
            } else {
                VerificationStatus::PartiallyVerified
            };
            customer.beneficial_owners.push(ubo);
        }

        customer
    }

    /// Select a retail persona based on configured weights.
    fn select_retail_persona(&mut self) -> RetailPersona {
        let weights = &self.config.population.retail_persona_weights;
        let roll: f64 = self.rng.random();
        let mut cumulative = 0.0;

        for (name, weight) in weights {
            cumulative += weight;
            if roll < cumulative {
                return match name.as_str() {
                    "student" => RetailPersona::Student,
                    "early_career" => RetailPersona::EarlyCareer,
                    "mid_career" => RetailPersona::MidCareer,
                    "retiree" => RetailPersona::Retiree,
                    "high_net_worth" => RetailPersona::HighNetWorth,
                    "gig_worker" => RetailPersona::GigWorker,
                    "seasonal_worker" => RetailPersona::SeasonalWorker,
                    "low_activity" => RetailPersona::LowActivity,
                    _ => RetailPersona::MidCareer,
                };
            }
        }
        RetailPersona::MidCareer
    }

    /// Select a business persona based on configured weights.
    fn select_business_persona(&mut self) -> BusinessPersona {
        let weights = &self.config.population.business_persona_weights;
        let roll: f64 = self.rng.random();
        let mut cumulative = 0.0;

        for (name, weight) in weights {
            cumulative += weight;
            if roll < cumulative {
                return match name.as_str() {
                    "small_business" => BusinessPersona::SmallBusiness,
                    "mid_market" => BusinessPersona::MidMarket,
                    "enterprise" => BusinessPersona::Enterprise,
                    "startup" => BusinessPersona::Startup,
                    "cash_intensive" => BusinessPersona::CashIntensive,
                    "import_export" => BusinessPersona::ImportExport,
                    "money_services" => BusinessPersona::MoneyServices,
                    "professional_services" => BusinessPersona::ProfessionalServices,
                    _ => BusinessPersona::SmallBusiness,
                };
            }
        }
        BusinessPersona::SmallBusiness
    }

    /// Select a trust persona.
    fn select_trust_persona(&mut self) -> TrustPersona {
        let options = [
            TrustPersona::FamilyTrust,
            TrustPersona::PrivateFoundation,
            TrustPersona::CharitableTrust,
            TrustPersona::InvestmentHolding,
            TrustPersona::SpecialPurposeVehicle,
        ];
        *options.choose(&mut self.rng).expect("non-empty array")
    }

    /// Generate a person name.
    fn generate_person_name(&mut self) -> (String, String) {
        let first_names = [
            "James",
            "Mary",
            "John",
            "Patricia",
            "Robert",
            "Jennifer",
            "Michael",
            "Linda",
            "William",
            "Barbara",
            "David",
            "Elizabeth",
            "Richard",
            "Susan",
            "Joseph",
            "Jessica",
            "Thomas",
            "Sarah",
            "Charles",
            "Karen",
            "Christopher",
            "Nancy",
            "Daniel",
            "Lisa",
            "Matthew",
            "Betty",
            "Anthony",
            "Margaret",
            "Mark",
            "Sandra",
        ];
        let last_names = [
            "Smith",
            "Johnson",
            "Williams",
            "Brown",
            "Jones",
            "Garcia",
            "Miller",
            "Davis",
            "Rodriguez",
            "Martinez",
            "Hernandez",
            "Lopez",
            "Gonzalez",
            "Wilson",
            "Anderson",
            "Thomas",
            "Taylor",
            "Moore",
            "Jackson",
            "Martin",
            "Lee",
            "Perez",
            "Thompson",
            "White",
            "Harris",
            "Sanchez",
            "Clark",
            "Ramirez",
            "Lewis",
            "Robinson",
        ];

        let first = first_names.choose(&mut self.rng).expect("non-empty array");
        let last = last_names.choose(&mut self.rng).expect("non-empty array");
        (first.to_string(), last.to_string())
    }

    /// Generate a business name.
    fn generate_business_name(&mut self, persona: BusinessPersona) -> String {
        let prefixes = [
            "Acme", "Global", "Premier", "Advanced", "Pacific", "Summit", "Atlas", "Apex",
        ];
        let industries = match persona {
            BusinessPersona::SmallBusiness => ["Services", "Solutions", "Group", "LLC"],
            BusinessPersona::MidMarket => ["Industries", "Corporation", "Enterprises", "Holdings"],
            BusinessPersona::Enterprise => ["International", "Global Corp", "Worldwide", "Inc"],
            BusinessPersona::CashIntensive => ["Retail", "Restaurant", "Store", "Shop"],
            BusinessPersona::ImportExport => {
                ["Trading", "Import Export", "Commerce", "International"]
            }
            BusinessPersona::ProfessionalServices => {
                ["Consulting", "Advisors", "Partners", "Associates"]
            }
            _ => ["Company", "Business", "Firm", "LLC"],
        };

        let prefix = prefixes.choose(&mut self.rng).expect("non-empty array");
        let suffix = industries.choose(&mut self.rng).expect("non-empty array");
        format!("{} {}", prefix, suffix)
    }

    /// Generate a trust name.
    fn generate_trust_name(&mut self, persona: TrustPersona) -> String {
        let (first_name, last_name) = self.generate_person_name();
        match persona {
            TrustPersona::FamilyTrust => format!("{} Family Trust", last_name),
            TrustPersona::PrivateFoundation => format!("{} {} Foundation", first_name, last_name),
            TrustPersona::CharitableTrust => format!("{} Charitable Trust", last_name),
            TrustPersona::InvestmentHolding => format!("{} Holdings Ltd", last_name),
            TrustPersona::SpecialPurposeVehicle => format!("{} SPV LLC", last_name),
        }
    }

    /// Select a country (weighted towards US).
    fn select_country(&mut self) -> String {
        let roll: f64 = self.rng.random();
        if roll < 0.8 {
            "US".to_string()
        } else if roll < 0.85 {
            "GB".to_string()
        } else if roll < 0.90 {
            "CA".to_string()
        } else if roll < 0.93 {
            "DE".to_string()
        } else if roll < 0.96 {
            "FR".to_string()
        } else {
            let countries = ["JP", "AU", "SG", "CH", "NL"];
            countries
                .choose(&mut self.rng)
                .expect("non-empty array")
                .to_string()
        }
    }

    /// Generate a random onboarding date within the simulation period.
    fn random_onboarding_date(&mut self) -> NaiveDate {
        // 70% onboarded before simulation, 30% during
        if self.rng.random::<f64>() < 0.7 {
            // Onboarded 1-5 years before simulation start
            let years_before: i64 = self.rng.random_range(1..=5);
            let days_offset: i64 = self.rng.random_range(0..365);
            self.start_date - chrono::Duration::days(years_before * 365 + days_offset)
        } else {
            // Onboarded during simulation
            let sim_days = (self.end_date - self.start_date).num_days();
            let offset = self.rng.random_range(0..sim_days);
            self.start_date + chrono::Duration::days(offset)
        }
    }

    /// Calculate risk tier for retail customer.
    fn calculate_retail_risk_tier(&mut self, persona: RetailPersona, country: &str) -> RiskTier {
        let base_score = persona.base_risk_score();
        let mut score = base_score as f64 * 10.0;

        // Country risk
        if !["US", "GB", "CA", "DE", "FR", "JP", "AU"].contains(&country) {
            score += 20.0;
        }

        // Risk appetite adjustment
        score *= self.config.compliance.risk_appetite.high_risk_multiplier();

        // Random variation
        score += self.rng.random_range(-10.0..10.0);

        RiskTier::from_score(score.clamp(0.0, 100.0) as u8)
    }

    /// Calculate risk tier for business customer.
    fn calculate_business_risk_tier(
        &mut self,
        persona: BusinessPersona,
        country: &str,
    ) -> RiskTier {
        let base_score = persona.base_risk_score();
        let mut score = base_score as f64 * 10.0;

        // Enhanced DD requirement
        if persona.requires_enhanced_dd() {
            score += 15.0;
        }

        // Country risk
        if !["US", "GB", "CA", "DE", "FR", "JP", "AU"].contains(&country) {
            score += 25.0;
        }

        // Risk appetite adjustment
        score *= self.config.compliance.risk_appetite.high_risk_multiplier();

        // Random variation
        score += self.rng.random_range(-10.0..10.0);

        RiskTier::from_score(score.clamp(0.0, 100.0) as u8)
    }

    /// Generate KYC profile for retail customer.
    fn generate_retail_kyc_profile(&mut self, persona: RetailPersona) -> KycProfile {
        use datasynth_core::models::banking::{
            CashIntensity, FrequencyBand, SourceOfFunds, TurnoverBand,
        };

        let (income_min, income_max) = persona.income_range();
        let (freq_min, freq_max) = persona.transaction_frequency_range();
        let avg_income = (income_min + income_max) / 2;
        let avg_freq = (freq_min + freq_max) / 2;

        let turnover = match avg_income {
            0..=2000 => TurnoverBand::VeryLow,
            2001..=5000 => TurnoverBand::Low,
            5001..=25000 => TurnoverBand::Medium,
            25001..=100000 => TurnoverBand::High,
            _ => TurnoverBand::VeryHigh,
        };

        let frequency = match avg_freq {
            0..=10 => FrequencyBand::VeryLow,
            11..=30 => FrequencyBand::Low,
            31..=100 => FrequencyBand::Medium,
            101..=300 => FrequencyBand::High,
            _ => FrequencyBand::VeryHigh,
        };

        let source = match persona {
            RetailPersona::Student => SourceOfFunds::Other,
            RetailPersona::Retiree => SourceOfFunds::Pension,
            RetailPersona::HighNetWorth => SourceOfFunds::Investments,
            RetailPersona::GigWorker | RetailPersona::SeasonalWorker => {
                SourceOfFunds::SelfEmployment
            }
            _ => SourceOfFunds::Employment,
        };

        let cash_intensity_level = if persona.cash_intensity() < 0.1 {
            CashIntensity::VeryLow
        } else if persona.cash_intensity() < 0.2 {
            CashIntensity::Low
        } else if persona.cash_intensity() < 0.35 {
            CashIntensity::Moderate
        } else {
            CashIntensity::High
        };

        KycProfile::new("Personal banking", source)
            .with_turnover(turnover)
            .with_frequency(frequency)
            .with_cash_intensity(cash_intensity_level)
    }

    /// Generate KYC profile for business customer.
    fn generate_business_kyc_profile(&mut self, persona: BusinessPersona) -> KycProfile {
        use datasynth_core::models::banking::{
            CashIntensity, FrequencyBand, SourceOfFunds, TurnoverBand,
        };

        let (turnover_min, turnover_max) = persona.turnover_range();
        let avg_turnover = (turnover_min + turnover_max) / 2;

        let turnover = match avg_turnover {
            0..=10_000 => TurnoverBand::VeryLow,
            10_001..=100_000 => TurnoverBand::Medium,
            100_001..=500_000 => TurnoverBand::High,
            500_001..=5_000_000 => TurnoverBand::VeryHigh,
            _ => TurnoverBand::UltraHigh,
        };

        let (_cash_min, cash_max) = persona.cash_deposit_frequency();
        let cash_intensity = if cash_max > 50 {
            CashIntensity::VeryHigh
        } else if cash_max > 20 {
            CashIntensity::High
        } else if cash_max > 5 {
            CashIntensity::Moderate
        } else {
            CashIntensity::Low
        };

        KycProfile::new("Business operations", SourceOfFunds::SelfEmployment)
            .with_turnover(turnover)
            .with_frequency(FrequencyBand::High)
            .with_cash_intensity(cash_intensity)
    }

    /// Select a PEP category.
    fn select_pep_category(&mut self) -> PepCategory {
        let categories = [
            PepCategory::SeniorGovernment,
            PepCategory::SeniorPolitical,
            PepCategory::FamilyMember,
            PepCategory::CloseAssociate,
            PepCategory::StateEnterprise,
        ];
        *categories.choose(&mut self.rng).expect("non-empty array")
    }

    /// Generate email address.
    fn generate_email(&mut self, first: &str, last: &str) -> String {
        let domains = [
            "gmail.com",
            "yahoo.com",
            "outlook.com",
            "hotmail.com",
            "icloud.com",
        ];
        let domain = domains.choose(&mut self.rng).expect("non-empty array");
        let num: u32 = self.rng.random_range(1..999);
        format!(
            "{}.{}{}@{}",
            first.to_lowercase(),
            last.to_lowercase(),
            num,
            domain
        )
    }

    /// Generate phone number.
    fn generate_phone(&self, country: &str) -> String {
        match country {
            "US" | "CA" => format!(
                "+1-555-{:03}-{:04}",
                rand::random::<u16>() % 1000,
                rand::random::<u16>() % 10000
            ),
            "GB" => format!(
                "+44-7{:03}-{:06}",
                rand::random::<u16>() % 1000,
                rand::random::<u32>() % 1000000
            ),
            _ => format!(
                "+{}-{:010}",
                rand::random::<u8>() % 90 + 10,
                rand::random::<u64>() % 10000000000
            ),
        }
    }

    /// Generate birth date based on persona.
    fn generate_birth_date(&mut self, persona: RetailPersona) -> NaiveDate {
        let base_year = self.start_date.year();
        let age_range = match persona {
            RetailPersona::Student => (18, 25),
            RetailPersona::EarlyCareer => (25, 35),
            RetailPersona::MidCareer => (35, 55),
            RetailPersona::Retiree => (55, 80),
            RetailPersona::HighNetWorth => (40, 70),
            RetailPersona::GigWorker => (20, 40),
            RetailPersona::SeasonalWorker => (18, 50),
            RetailPersona::LowActivity => (20, 70),
        };

        let age: i32 = self.rng.random_range(age_range.0..=age_range.1);
        let month: u32 = self.rng.random_range(1..=12);
        let day: u32 = self.rng.random_range(1..=28);

        NaiveDate::from_ymd_opt(base_year - age, month, day).unwrap_or_else(|| {
            NaiveDate::from_ymd_opt(base_year - age, 1, 1).expect("valid fallback date")
        })
    }

    /// Get industry description for business persona.
    fn get_industry_description(&self, persona: BusinessPersona) -> String {
        match persona {
            BusinessPersona::SmallBusiness => "Small Business Services",
            BusinessPersona::MidMarket => "Mid-Market Corporation",
            BusinessPersona::Enterprise => "Large Enterprise",
            BusinessPersona::Startup => "Technology Startup",
            BusinessPersona::CashIntensive => "Retail / Restaurant",
            BusinessPersona::ImportExport => "International Trade",
            BusinessPersona::MoneyServices => "Money Services Business",
            BusinessPersona::ProfessionalServices => "Professional Services",
        }
        .to_string()
    }

    /// Generate a realistic address.
    fn generate_address(&mut self, country: &str) -> (String, String, String, String) {
        let number: u32 = self.rng.random_range(1..=9999);
        let streets = [
            "Main St",
            "Oak Ave",
            "Maple Dr",
            "Broadway",
            "Park Ave",
            "Cedar Ln",
            "Elm St",
            "Washington Blvd",
            "Market St",
            "High St",
        ];
        let street = streets.choose(&mut self.rng).expect("non-empty array");
        let addr = format!("{} {}", number, street);

        let (city, state, postal) = match country {
            "US" => {
                let cities = [
                    ("New York", "NY"),
                    ("Los Angeles", "CA"),
                    ("Chicago", "IL"),
                    ("Houston", "TX"),
                    ("Phoenix", "AZ"),
                    ("Philadelphia", "PA"),
                    ("San Antonio", "TX"),
                    ("San Diego", "CA"),
                    ("Dallas", "TX"),
                    ("Austin", "TX"),
                ];
                let (c, s) = cities.choose(&mut self.rng).expect("non-empty array");
                let zip: u32 = self.rng.random_range(10001..=99999);
                (c.to_string(), s.to_string(), format!("{:05}", zip))
            }
            "GB" => {
                let cities = [
                    ("London", "England"),
                    ("Manchester", "England"),
                    ("Birmingham", "England"),
                    ("Edinburgh", "Scotland"),
                ];
                let (c, s) = cities.choose(&mut self.rng).expect("non-empty array");
                let area: char = (b'A' + self.rng.random_range(0..26)) as char;
                let num: u8 = self.rng.random_range(1..=9);
                (
                    c.to_string(),
                    s.to_string(),
                    format!("{}{}  {}AA", area, num, self.rng.random_range(1..=9)),
                )
            }
            "CA" => {
                let cities = [
                    ("Toronto", "ON"),
                    ("Vancouver", "BC"),
                    ("Montreal", "QC"),
                    ("Calgary", "AB"),
                ];
                let (c, s) = cities.choose(&mut self.rng).expect("non-empty array");
                let l1: char = (b'A' + self.rng.random_range(0..26)) as char;
                let d1: u8 = self.rng.random_range(1..=9);
                let l2: char = (b'A' + self.rng.random_range(0..26)) as char;
                (
                    c.to_string(),
                    s.to_string(),
                    format!("{}{}{} {}{}{}", l1, d1, l2, d1, l1, d1),
                )
            }
            _ => {
                let zip: u32 = self.rng.random_range(10000..=99999);
                ("City".to_string(), "State".to_string(), format!("{}", zip))
            }
        };
        (addr, city, state, postal)
    }

    /// Generate a national ID number.
    fn generate_national_id(&mut self, country: &str) -> String {
        match country {
            "US" => format!(
                "{:03}-{:02}-{:04}",
                self.rng.random_range(100..=999),
                self.rng.random_range(10..=99),
                self.rng.random_range(1000..=9999)
            ),
            "GB" => format!("AB{:06}C", self.rng.random_range(100000..=999999)),
            _ => format!(
                "ID-{:010}",
                self.rng.random_range(1000000000_u64..=9999999999)
            ),
        }
    }

    /// Generate a phone number from a country pack's phone configuration.
    ///
    /// Reads `pack.phone.formats` (landline / mobile / freephone) and picks one
    /// at random to use as a template.  Inside the template every `{xxx…}`
    /// placeholder is replaced with the corresponding number of random digits.
    ///
    /// Falls back to [`Self::generate_phone`] when no format templates are
    /// configured in the pack.
    pub fn generate_phone_from_pack(&self, pack: &CountryPack) -> String {
        // Collect all non-empty format strings from the pack.
        let mut formats: Vec<&str> = Vec::new();
        if !pack.phone.formats.landline.is_empty() {
            formats.push(&pack.phone.formats.landline);
        }
        if !pack.phone.formats.mobile.is_empty() {
            formats.push(&pack.phone.formats.mobile);
        }
        if !pack.phone.formats.freephone.is_empty() {
            formats.push(&pack.phone.formats.freephone);
        }

        if formats.is_empty() {
            // No formats configured -- delegate to hardcoded logic.
            return self.generate_phone(&pack.country_code);
        }

        // Pick a random format template.
        let mut thread_rng = rand::rng();
        let template = *formats.choose(&mut thread_rng).expect("non-empty vec");

        // Replace every `{x…}` placeholder with random digits.
        let mut result = String::with_capacity(template.len());
        let mut chars = template.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '{' {
                // Count the 'x' characters inside the braces.
                let mut digit_count: usize = 0;
                for inner in chars.by_ref() {
                    if inner == '}' {
                        break;
                    }
                    if inner == 'x' || inner == 'X' {
                        digit_count += 1;
                    }
                }
                // Emit that many random digits.
                for _ in 0..digit_count {
                    let d = rand::random::<u8>() % 10;
                    result.push((b'0' + d) as char);
                }
            } else {
                result.push(ch);
            }
        }
        result
    }

    /// Generate a realistic address from a country pack's address configuration.
    ///
    /// Reads cities from `pack.address.components.city_names`, states from
    /// `pack.address.components.state_names` / `state_codes`, streets from
    /// `pack.address.components.street_names`, and generates a postal code
    /// matching `pack.address.postal_code.format` (where `X` becomes a random
    /// digit and `A` becomes a random uppercase letter).
    ///
    /// Returns `(address_line, city, state, postal_code)`.
    ///
    /// Falls back to [`Self::generate_address`] when the pack has no address
    /// data configured.
    pub fn generate_address_from_pack(
        &mut self,
        pack: &CountryPack,
    ) -> (String, String, String, String) {
        let cities = &pack.address.components.city_names;
        let postal_format = &pack.address.postal_code.format;

        // If neither cities nor postal format are available, fall back.
        if cities.is_empty() && postal_format.is_empty() {
            return self.generate_address(&pack.country_code);
        }

        // --- Street address ---
        let number: u32 = self.rng.random_range(1..=9999);
        let street_names = &pack.address.components.street_names;
        let street = if street_names.is_empty() {
            let default_streets = [
                "Main St",
                "Oak Ave",
                "Maple Dr",
                "Broadway",
                "Park Ave",
                "Cedar Ln",
                "Elm St",
                "Washington Blvd",
                "Market St",
                "High St",
            ];
            default_streets
                .choose(&mut self.rng)
                .expect("non-empty array")
                .to_string()
        } else {
            street_names
                .choose(&mut self.rng)
                .expect("non-empty vec")
                .clone()
        };
        let addr = format!("{} {}", number, street);

        // --- City ---
        let city = if cities.is_empty() {
            "City".to_string()
        } else {
            cities.choose(&mut self.rng).expect("non-empty vec").clone()
        };

        // --- State ---
        let state_codes = &pack.address.components.state_codes;
        let state_names = &pack.address.components.state_names;
        let state = if !state_codes.is_empty() {
            state_codes
                .choose(&mut self.rng)
                .expect("non-empty vec")
                .clone()
        } else if !state_names.is_empty() {
            state_names
                .choose(&mut self.rng)
                .expect("non-empty vec")
                .clone()
        } else {
            "State".to_string()
        };

        // --- Postal code ---
        let postal = if postal_format.is_empty() {
            let zip: u32 = self.rng.random_range(10000..=99999);
            format!("{}", zip)
        } else {
            self.expand_postal_format(postal_format)
        };

        (addr, city, state, postal)
    }

    /// Generate a national ID number from a country pack's legal-entities
    /// configuration.
    ///
    /// Reads `pack.legal_entities.tax_id_format.format` (e.g. `"xxx-xx-xxxx"`
    /// or `"ABxxxxxxC"`).  In the format string every lowercase `x` is replaced
    /// with a random digit and every uppercase letter (`A`-`Z`) is replaced
    /// with a random uppercase letter.
    ///
    /// Falls back to [`Self::generate_national_id`] when the format is empty.
    pub fn generate_national_id_from_pack(&mut self, pack: &CountryPack) -> String {
        let fmt = &pack.legal_entities.tax_id_format.format;
        if fmt.is_empty() {
            return self.generate_national_id(&pack.country_code);
        }
        self.expand_id_format(fmt)
    }

    // ------------------------------------------------------------------
    // Private helpers for country-pack expansion
    // ------------------------------------------------------------------

    /// Expand a postal-code format string.
    ///
    /// `X` (uppercase) is replaced with a random digit (0-9).
    /// `A` (uppercase) is replaced with a random uppercase letter (A-Z).
    /// All other characters are kept verbatim.
    fn expand_postal_format(&mut self, format: &str) -> String {
        let mut result = String::with_capacity(format.len());
        for ch in format.chars() {
            match ch {
                'X' => {
                    let d: u8 = self.rng.random_range(0..10);
                    result.push((b'0' + d) as char);
                }
                'A' => {
                    let l: u8 = self.rng.random_range(0..26);
                    result.push((b'A' + l) as char);
                }
                other => result.push(other),
            }
        }
        result
    }

    /// Expand a national-ID format string.
    ///
    /// `x` (lowercase) is replaced with a random digit (0-9).
    /// Any uppercase letter (`A`-`Z`) is replaced with a random uppercase
    /// letter.
    /// All other characters (dashes, spaces, etc.) are kept verbatim.
    fn expand_id_format(&mut self, format: &str) -> String {
        let mut result = String::with_capacity(format.len());
        for ch in format.chars() {
            if ch == 'x' {
                let d: u8 = self.rng.random_range(0..10);
                result.push((b'0' + d) as char);
            } else if ch.is_ascii_uppercase() {
                let l: u8 = self.rng.random_range(0..26);
                result.push((b'A' + l) as char);
            } else {
                result.push(ch);
            }
        }
        result
    }

    /// Generate a passport number.
    fn generate_passport_number(&mut self, country: &str) -> String {
        match country {
            "US" => format!("{:09}", self.rng.random_range(100000000_u64..=999999999)),
            "GB" => format!("{:09}", self.rng.random_range(100000000_u64..=999999999)),
            _ => format!("P{:08}", self.rng.random_range(10000000_u64..=99999999)),
        }
    }

    /// Form households from retail customers.
    fn form_households(&mut self, customers: &mut [BankingCustomer]) {
        use uuid::Uuid;

        let retail_indices: Vec<usize> = customers
            .iter()
            .enumerate()
            .filter(|(_, c)| c.customer_type == BankingCustomerType::Retail)
            .map(|(i, _)| i)
            .collect();

        let household_count = (retail_indices.len() as f64 * self.config.population.household_rate
            / self.config.population.avg_household_size) as usize;

        for _ in 0..household_count {
            let household_id = Uuid::new_v4();
            let size = self.rng.random_range(2..=4).min(retail_indices.len());

            // Select random customers for household
            let selected: Vec<usize> = retail_indices
                .choose_multiple(&mut self.rng, size)
                .copied()
                .collect();

            for idx in selected {
                customers[idx].household_id = Some(household_id);
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_customer_generation() {
        let config = BankingConfig::small();
        let mut generator = CustomerGenerator::new(config, 12345);
        let customers = generator.generate_all();

        assert!(!customers.is_empty());

        let retail_count = customers
            .iter()
            .filter(|c| c.customer_type == BankingCustomerType::Retail)
            .count();
        let business_count = customers
            .iter()
            .filter(|c| c.customer_type == BankingCustomerType::Business)
            .count();

        assert!(retail_count > 0);
        assert!(business_count > 0);
    }

    #[test]
    fn test_persona_distribution() {
        let config = BankingConfig::small();
        let mut generator = CustomerGenerator::new(config, 12345);

        // Generate many personas and check distribution
        let mut personas = std::collections::HashMap::new();
        for _ in 0..1000 {
            let persona = generator.select_retail_persona();
            *personas.entry(format!("{:?}", persona)).or_insert(0) += 1;
        }

        // Should have multiple different personas
        assert!(personas.len() > 3);
    }
}
