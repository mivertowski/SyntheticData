//! Banking customer type definitions.

use serde::{Deserialize, Serialize};

/// Type of banking customer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BankingCustomerType {
    /// Individual retail customer
    #[default]
    Retail,
    /// Business/corporate customer
    Business,
    /// Trust or foundation
    Trust,
    /// Financial institution
    FinancialInstitution,
    /// Government entity
    Government,
    /// Non-profit organization
    NonProfit,
}

impl BankingCustomerType {
    /// Returns the category name.
    pub fn category(&self) -> &'static str {
        match self {
            Self::Retail => "Retail",
            Self::Business => "Business",
            Self::Trust => "Trust",
            Self::FinancialInstitution => "Financial Institution",
            Self::Government => "Government",
            Self::NonProfit => "Non-Profit",
        }
    }

    /// Returns whether this is a natural person (vs legal entity).
    pub fn is_natural_person(&self) -> bool {
        matches!(self, Self::Retail)
    }

    /// Returns whether enhanced due diligence is typically required.
    pub fn requires_enhanced_dd(&self) -> bool {
        matches!(self, Self::Trust | Self::FinancialInstitution)
    }

    /// Returns whether beneficial ownership analysis is required.
    pub fn requires_ubo_analysis(&self) -> bool {
        !matches!(self, Self::Retail | Self::Government)
    }
}

/// Retail customer persona for behavioral modeling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetailPersona {
    /// College/university student
    Student,
    /// Early career professional (25-35)
    EarlyCareer,
    /// Mid-career professional (35-55)
    MidCareer,
    /// Retiree (55+)
    Retiree,
    /// High net worth individual (>$1M assets)
    HighNetWorth,
    /// Gig economy worker / freelancer
    GigWorker,
    /// Seasonal worker
    SeasonalWorker,
    /// Unemployed / low activity
    LowActivity,
}

impl RetailPersona {
    /// Expected monthly income range (min, max) in local currency units.
    pub fn income_range(&self) -> (u32, u32) {
        match self {
            Self::Student => (0, 2_000),
            Self::EarlyCareer => (3_000, 8_000),
            Self::MidCareer => (6_000, 20_000),
            Self::Retiree => (2_000, 10_000),
            Self::HighNetWorth => (20_000, 200_000),
            Self::GigWorker => (1_000, 8_000),
            Self::SeasonalWorker => (0, 6_000),
            Self::LowActivity => (0, 1_500),
        }
    }

    /// Expected monthly transaction count range (min, max).
    pub fn transaction_frequency_range(&self) -> (u32, u32) {
        match self {
            Self::Student => (10, 50),
            Self::EarlyCareer => (30, 100),
            Self::MidCareer => (40, 150),
            Self::Retiree => (20, 60),
            Self::HighNetWorth => (50, 300),
            Self::GigWorker => (30, 120),
            Self::SeasonalWorker => (10, 80),
            Self::LowActivity => (5, 20),
        }
    }

    /// Base risk score (1-10, 10 being highest risk).
    pub fn base_risk_score(&self) -> u8 {
        match self {
            Self::Student => 2,
            Self::EarlyCareer => 2,
            Self::MidCareer => 2,
            Self::Retiree => 2,
            Self::HighNetWorth => 4,
            Self::GigWorker => 3,
            Self::SeasonalWorker => 3,
            Self::LowActivity => 2,
        }
    }

    /// Typical cash usage intensity (0.0-1.0).
    pub fn cash_intensity(&self) -> f64 {
        match self {
            Self::Student => 0.15,
            Self::EarlyCareer => 0.10,
            Self::MidCareer => 0.08,
            Self::Retiree => 0.20,
            Self::HighNetWorth => 0.05,
            Self::GigWorker => 0.25,
            Self::SeasonalWorker => 0.30,
            Self::LowActivity => 0.20,
        }
    }
}

/// Business customer persona for behavioral modeling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BusinessPersona {
    /// Small business (<$1M annual revenue)
    SmallBusiness,
    /// Medium business ($1M-$50M)
    MidMarket,
    /// Large enterprise (>$50M)
    Enterprise,
    /// Startup / early stage
    Startup,
    /// Cash-intensive business (retail, restaurants)
    CashIntensive,
    /// Import/export business
    ImportExport,
    /// Money services business
    MoneyServices,
    /// Professional services (law, accounting)
    ProfessionalServices,
}

impl BusinessPersona {
    /// Expected monthly turnover range in local currency units.
    pub fn turnover_range(&self) -> (u64, u64) {
        match self {
            Self::SmallBusiness => (10_000, 100_000),
            Self::MidMarket => (100_000, 5_000_000),
            Self::Enterprise => (5_000_000, 100_000_000),
            Self::Startup => (0, 50_000),
            Self::CashIntensive => (20_000, 500_000),
            Self::ImportExport => (50_000, 10_000_000),
            Self::MoneyServices => (100_000, 50_000_000),
            Self::ProfessionalServices => (30_000, 1_000_000),
        }
    }

    /// Base risk score (1-10).
    pub fn base_risk_score(&self) -> u8 {
        match self {
            Self::SmallBusiness => 3,
            Self::MidMarket => 3,
            Self::Enterprise => 2,
            Self::Startup => 4,
            Self::CashIntensive => 5,
            Self::ImportExport => 6,
            Self::MoneyServices => 8,
            Self::ProfessionalServices => 3,
        }
    }

    /// Whether this business type requires enhanced due diligence.
    pub fn requires_enhanced_dd(&self) -> bool {
        matches!(
            self,
            Self::CashIntensive | Self::ImportExport | Self::MoneyServices
        )
    }

    /// Typical cash deposit frequency (transactions per month).
    pub fn cash_deposit_frequency(&self) -> (u32, u32) {
        match self {
            Self::CashIntensive => (20, 100),
            Self::SmallBusiness => (2, 10),
            Self::MidMarket => (0, 5),
            Self::Enterprise => (0, 2),
            Self::Startup => (0, 3),
            Self::ImportExport => (0, 5),
            Self::MoneyServices => (50, 500),
            Self::ProfessionalServices => (0, 5),
        }
    }
}

/// Trust customer persona.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustPersona {
    /// Family trust for estate planning
    FamilyTrust,
    /// Private foundation
    PrivateFoundation,
    /// Charitable trust
    CharitableTrust,
    /// Investment holding structure
    InvestmentHolding,
    /// Special purpose vehicle
    SpecialPurposeVehicle,
}

impl TrustPersona {
    /// Base risk score (1-10).
    pub fn base_risk_score(&self) -> u8 {
        match self {
            Self::FamilyTrust => 4,
            Self::PrivateFoundation => 5,
            Self::CharitableTrust => 4,
            Self::InvestmentHolding => 6,
            Self::SpecialPurposeVehicle => 7,
        }
    }

    /// Typical number of beneficial owners.
    pub fn typical_ubo_count(&self) -> (u8, u8) {
        match self {
            Self::FamilyTrust => (1, 5),
            Self::PrivateFoundation => (1, 10),
            Self::CharitableTrust => (1, 7),
            Self::InvestmentHolding => (1, 20),
            Self::SpecialPurposeVehicle => (1, 3),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_customer_type_properties() {
        assert!(BankingCustomerType::Retail.is_natural_person());
        assert!(!BankingCustomerType::Business.is_natural_person());
        assert!(BankingCustomerType::Trust.requires_enhanced_dd());
        assert!(BankingCustomerType::Business.requires_ubo_analysis());
        assert!(!BankingCustomerType::Government.requires_ubo_analysis());
    }

    #[test]
    fn test_retail_persona_properties() {
        let (min, max) = RetailPersona::HighNetWorth.income_range();
        assert!(max > min);
        assert!(
            RetailPersona::HighNetWorth.base_risk_score()
                > RetailPersona::Student.base_risk_score()
        );
    }

    #[test]
    fn test_business_persona_properties() {
        assert!(BusinessPersona::MoneyServices.requires_enhanced_dd());
        assert!(!BusinessPersona::Enterprise.requires_enhanced_dd());
        assert!(
            BusinessPersona::MoneyServices.base_risk_score()
                > BusinessPersona::Enterprise.base_risk_score()
        );
    }
}
