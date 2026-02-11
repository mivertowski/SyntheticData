//! Customer generator for banking data.

use chrono::{Datelike, NaiveDate};
use datasynth_core::models::banking::{
    BankingCustomerType, BusinessPersona, RetailPersona, RiskTier, TrustPersona,
};
use datasynth_core::DeterministicUuidFactory;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use crate::config::BankingConfig;
use crate::models::{BankingCustomer, KycProfile, PepCategory, PersonaVariant};

/// Generator for banking customers.
pub struct CustomerGenerator {
    config: BankingConfig,
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
    start_date: NaiveDate,
    end_date: NaiveDate,
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
        }
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
        if self.rng.gen::<f64>() < self.config.compliance.pep_rate {
            customer.is_pep = true;
            customer.pep_category = Some(self.select_pep_category());
            customer.risk_tier = RiskTier::High;
        }

        // Generate contact info
        customer.email = Some(self.generate_email(&first_name, &last_name));
        customer.phone = Some(self.generate_phone(&country));
        customer.date_of_birth = Some(self.generate_birth_date(persona));

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

        // Generate contact info
        customer.email = Some(format!("info@{}.com", name.to_lowercase().replace(' ', "")));
        customer.phone = Some(self.generate_phone(&country));

        // Set industry
        customer.industry_description = Some(self.get_industry_description(persona));

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

        customer
    }

    /// Select a retail persona based on configured weights.
    fn select_retail_persona(&mut self) -> RetailPersona {
        let weights = &self.config.population.retail_persona_weights;
        let roll: f64 = self.rng.gen();
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
        let roll: f64 = self.rng.gen();
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
        let roll: f64 = self.rng.gen();
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
        if self.rng.gen::<f64>() < 0.7 {
            // Onboarded 1-5 years before simulation start
            let years_before: i64 = self.rng.gen_range(1..=5);
            let days_offset: i64 = self.rng.gen_range(0..365);
            self.start_date - chrono::Duration::days(years_before * 365 + days_offset)
        } else {
            // Onboarded during simulation
            let sim_days = (self.end_date - self.start_date).num_days();
            let offset = self.rng.gen_range(0..sim_days);
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
        score += self.rng.gen_range(-10.0..10.0);

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
        score += self.rng.gen_range(-10.0..10.0);

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
        let num: u32 = self.rng.gen_range(1..999);
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

        let age: i32 = self.rng.gen_range(age_range.0..=age_range.1);
        let month: u32 = self.rng.gen_range(1..=12);
        let day: u32 = self.rng.gen_range(1..=28);

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
            let size = self.rng.gen_range(2..=4).min(retail_indices.len());

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
