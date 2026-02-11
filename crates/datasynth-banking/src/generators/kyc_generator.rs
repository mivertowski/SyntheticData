//! KYC profile generator for banking data.

use datasynth_core::models::banking::{
    CashIntensity, CountryExposure, CountryExposureType, CountryRiskCategory, FrequencyBand,
    SourceOfFunds, SourceOfWealth, TurnoverBand,
};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use crate::config::BankingConfig;
use crate::models::{BankingCustomer, ExpectedCategory, KycProfile, PersonaVariant};

/// Generator for KYC profiles.
pub struct KycGenerator {
    rng: ChaCha8Rng,
}

impl KycGenerator {
    /// Create a new KYC generator.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed.wrapping_add(4000)),
        }
    }

    /// Generate KYC profile for a customer.
    pub fn generate_profile(
        &mut self,
        customer: &BankingCustomer,
        _config: &BankingConfig,
    ) -> KycProfile {
        match &customer.persona {
            Some(PersonaVariant::Retail(p)) => self.generate_retail_profile(*p),
            Some(PersonaVariant::Business(p)) => self.generate_business_profile(*p),
            Some(PersonaVariant::Trust(p)) => self.generate_trust_profile(*p),
            None => KycProfile::default(),
        }
    }

    /// Generate retail KYC profile.
    fn generate_retail_profile(
        &mut self,
        persona: datasynth_core::models::banking::RetailPersona,
    ) -> KycProfile {
        use datasynth_core::models::banking::RetailPersona;

        let (turnover, frequency, source, cash_intensity) = match persona {
            RetailPersona::Student => (
                TurnoverBand::VeryLow,
                FrequencyBand::Low,
                SourceOfFunds::Other,
                CashIntensity::Low,
            ),
            RetailPersona::EarlyCareer => (
                TurnoverBand::Low,
                FrequencyBand::Medium,
                SourceOfFunds::Employment,
                CashIntensity::Low,
            ),
            RetailPersona::MidCareer => (
                TurnoverBand::Medium,
                FrequencyBand::Medium,
                SourceOfFunds::Employment,
                CashIntensity::VeryLow,
            ),
            RetailPersona::Retiree => (
                TurnoverBand::Low,
                FrequencyBand::Low,
                SourceOfFunds::Pension,
                CashIntensity::Moderate,
            ),
            RetailPersona::HighNetWorth => (
                TurnoverBand::VeryHigh,
                FrequencyBand::High,
                SourceOfFunds::Investments,
                CashIntensity::VeryLow,
            ),
            RetailPersona::GigWorker => (
                TurnoverBand::Low,
                FrequencyBand::High,
                SourceOfFunds::SelfEmployment,
                CashIntensity::Moderate,
            ),
            _ => (
                TurnoverBand::Low,
                FrequencyBand::Medium,
                SourceOfFunds::Employment,
                CashIntensity::Low,
            ),
        };

        let mut profile = KycProfile::new("Personal banking", source)
            .with_turnover(turnover)
            .with_frequency(frequency)
            .with_cash_intensity(cash_intensity);

        // Add expected categories
        profile.expected_categories = self.generate_retail_categories(persona);

        // Add geographic exposure
        profile.geographic_exposure = vec![CountryExposure {
            country_code: "US".to_string(),
            exposure_type: CountryExposureType::Residence,
            risk_category: CountryRiskCategory::Low,
        }];

        // Random completeness
        profile.completeness_score = self.rng.gen_range(0.90..1.0);

        profile
    }

    /// Generate business KYC profile.
    fn generate_business_profile(
        &mut self,
        persona: datasynth_core::models::banking::BusinessPersona,
    ) -> KycProfile {
        use datasynth_core::models::banking::BusinessPersona;

        let (turnover, cash_intensity) = match persona {
            BusinessPersona::SmallBusiness => (TurnoverBand::Medium, CashIntensity::Low),
            BusinessPersona::MidMarket => (TurnoverBand::High, CashIntensity::VeryLow),
            BusinessPersona::Enterprise => (TurnoverBand::UltraHigh, CashIntensity::VeryLow),
            BusinessPersona::CashIntensive => (TurnoverBand::High, CashIntensity::VeryHigh),
            BusinessPersona::ImportExport => (TurnoverBand::VeryHigh, CashIntensity::Low),
            _ => (TurnoverBand::Medium, CashIntensity::Moderate),
        };

        let mut profile = KycProfile::new("Business operations", SourceOfFunds::SelfEmployment)
            .with_turnover(turnover)
            .with_frequency(FrequencyBand::High)
            .with_cash_intensity(cash_intensity);

        // Business-specific settings
        profile.beneficial_owner_complexity = self.rng.gen_range(1..5);

        if matches!(persona, BusinessPersona::ImportExport) {
            profile.international_rate = 0.4;
            profile.geographic_exposure = vec![
                CountryExposure {
                    country_code: "US".to_string(),
                    exposure_type: CountryExposureType::BusinessOperations,
                    risk_category: CountryRiskCategory::Low,
                },
                CountryExposure {
                    country_code: "CN".to_string(),
                    exposure_type: CountryExposureType::TransactionHistory,
                    risk_category: CountryRiskCategory::Medium,
                },
            ];
        }

        profile
    }

    /// Generate trust KYC profile.
    fn generate_trust_profile(
        &mut self,
        _persona: datasynth_core::models::banking::TrustPersona,
    ) -> KycProfile {
        let mut profile = KycProfile::high_net_worth();
        profile.beneficial_owner_complexity = self.rng.gen_range(3..8);
        profile.source_of_wealth = Some(SourceOfWealth::Inheritance);
        profile
    }

    /// Generate expected categories for retail persona.
    fn generate_retail_categories(
        &self,
        persona: datasynth_core::models::banking::RetailPersona,
    ) -> Vec<ExpectedCategory> {
        use datasynth_core::models::banking::RetailPersona;

        match persona {
            RetailPersona::Student => vec![
                ExpectedCategory::new("Dining", 0.25),
                ExpectedCategory::new("Entertainment", 0.20),
                ExpectedCategory::new("Shopping", 0.20),
                ExpectedCategory::new("Transportation", 0.15),
            ],
            RetailPersona::MidCareer => vec![
                ExpectedCategory::new("Groceries", 0.25),
                ExpectedCategory::new("Dining", 0.15),
                ExpectedCategory::new("Utilities", 0.15),
                ExpectedCategory::new("Shopping", 0.20),
            ],
            RetailPersona::HighNetWorth => vec![
                ExpectedCategory::new("Investment", 0.30),
                ExpectedCategory::new("Luxury", 0.20),
                ExpectedCategory::new("Travel", 0.20),
            ],
            _ => vec![
                ExpectedCategory::new("Groceries", 0.25),
                ExpectedCategory::new("Shopping", 0.20),
                ExpectedCategory::new("Dining", 0.15),
            ],
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use uuid::Uuid;

    #[test]
    fn test_kyc_generation() {
        let config = BankingConfig::default();
        let mut gen = KycGenerator::new(12345);

        let customer = BankingCustomer::new_retail(
            Uuid::new_v4(),
            "Test",
            "User",
            "US",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        )
        .with_persona(PersonaVariant::Retail(
            datasynth_core::models::banking::RetailPersona::MidCareer,
        ));

        let profile = gen.generate_profile(&customer, &config);
        assert!(!profile.declared_purpose.is_empty());
    }
}
