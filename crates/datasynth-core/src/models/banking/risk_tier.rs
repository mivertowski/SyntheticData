//! Risk tier definitions for KYC/AML.

use serde::{Deserialize, Serialize};

/// Customer risk tier for KYC purposes.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum RiskTier {
    /// Low risk - simplified due diligence allowed
    Low,
    /// Medium risk - standard due diligence
    #[default]
    Medium,
    /// High risk - enhanced due diligence required
    High,
    /// Very high risk - senior management approval required
    VeryHigh,
    /// Prohibited - relationship not allowed
    Prohibited,
}

impl RiskTier {
    /// Numeric score for calculations (0-100).
    pub fn score(&self) -> u8 {
        match self {
            Self::Low => 20,
            Self::Medium => 40,
            Self::High => 65,
            Self::VeryHigh => 85,
            Self::Prohibited => 100,
        }
    }

    /// Whether enhanced due diligence is required.
    pub fn requires_enhanced_dd(&self) -> bool {
        matches!(self, Self::High | Self::VeryHigh)
    }

    /// Whether senior management approval is required.
    pub fn requires_senior_approval(&self) -> bool {
        matches!(self, Self::VeryHigh)
    }

    /// Review frequency in months.
    pub fn review_frequency_months(&self) -> u8 {
        match self {
            Self::Low => 36,
            Self::Medium => 24,
            Self::High => 12,
            Self::VeryHigh => 6,
            Self::Prohibited => 0,
        }
    }

    /// Transaction monitoring intensity.
    pub fn monitoring_intensity(&self) -> MonitoringIntensity {
        match self {
            Self::Low => MonitoringIntensity::Standard,
            Self::Medium => MonitoringIntensity::Standard,
            Self::High => MonitoringIntensity::Enhanced,
            Self::VeryHigh => MonitoringIntensity::Intensive,
            Self::Prohibited => MonitoringIntensity::Intensive,
        }
    }

    /// Create from a numeric score (0-100).
    pub fn from_score(score: u8) -> Self {
        match score {
            0..=30 => Self::Low,
            31..=50 => Self::Medium,
            51..=75 => Self::High,
            76..=95 => Self::VeryHigh,
            _ => Self::Prohibited,
        }
    }
}

/// Transaction monitoring intensity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MonitoringIntensity {
    /// Standard automated monitoring
    #[default]
    Standard,
    /// Enhanced monitoring with lower thresholds
    Enhanced,
    /// Intensive monitoring with manual review
    Intensive,
}

impl MonitoringIntensity {
    /// Alert threshold multiplier (1.0 = standard).
    pub fn threshold_multiplier(&self) -> f64 {
        match self {
            Self::Standard => 1.0,
            Self::Enhanced => 0.7,
            Self::Intensive => 0.5,
        }
    }

    /// Whether manual review is required for alerts.
    pub fn requires_manual_review(&self) -> bool {
        matches!(self, Self::Intensive)
    }
}

/// Source of funds classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceOfFunds {
    /// Regular employment income
    Employment,
    /// Self-employment / business income
    SelfEmployment,
    /// Investment returns
    Investments,
    /// Inheritance
    Inheritance,
    /// Gift
    Gift,
    /// Sale of property
    PropertySale,
    /// Pension / retirement
    Pension,
    /// Government benefits
    GovernmentBenefits,
    /// Lottery / gambling winnings
    GamblingWinnings,
    /// Legal settlement
    LegalSettlement,
    /// Loan proceeds
    Loan,
    /// Insurance payout
    Insurance,
    /// Crypto / digital assets
    CryptoAssets,
    /// Other
    Other,
    /// Unknown / undeclared
    Unknown,
}

impl SourceOfFunds {
    /// Risk weight (1.0 = standard).
    pub fn risk_weight(&self) -> f64 {
        match self {
            Self::Employment | Self::Pension | Self::GovernmentBenefits => 0.7,
            Self::SelfEmployment => 1.2,
            Self::Investments => 1.0,
            Self::Inheritance | Self::Gift => 1.3,
            Self::PropertySale => 1.1,
            Self::GamblingWinnings => 2.0,
            Self::LegalSettlement => 1.5,
            Self::Loan => 1.0,
            Self::Insurance => 0.9,
            Self::CryptoAssets => 2.0,
            Self::Other => 1.5,
            Self::Unknown => 2.5,
        }
    }

    /// Whether documentation is typically required.
    pub fn requires_documentation(&self) -> bool {
        !matches!(
            self,
            Self::Employment | Self::Pension | Self::GovernmentBenefits
        )
    }
}

/// Source of wealth classification (for HNW customers).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceOfWealth {
    /// Built through employment/career
    CareerEarnings,
    /// Inherited wealth
    Inheritance,
    /// Business ownership
    BusinessOwnership,
    /// Investment appreciation
    Investments,
    /// Real estate appreciation
    RealEstate,
    /// Sale of business
    BusinessSale,
    /// IPO / equity event
    EquityEvent,
    /// Professional practice (doctor, lawyer)
    ProfessionalPractice,
    /// Entertainment / sports
    Entertainment,
    /// Crypto / digital assets
    CryptoAssets,
    /// Other
    Other,
}

impl SourceOfWealth {
    /// Risk weight (1.0 = standard).
    pub fn risk_weight(&self) -> f64 {
        match self {
            Self::CareerEarnings | Self::ProfessionalPractice => 0.8,
            Self::Inheritance => 1.2,
            Self::BusinessOwnership | Self::BusinessSale => 1.3,
            Self::Investments | Self::RealEstate => 1.0,
            Self::EquityEvent => 1.1,
            Self::Entertainment => 1.4,
            Self::CryptoAssets => 2.0,
            Self::Other => 1.5,
        }
    }
}

/// Country risk classification for geographic exposure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryExposure {
    /// Country code (ISO 3166-1 alpha-2)
    pub country_code: String,
    /// Exposure type
    pub exposure_type: CountryExposureType,
    /// Risk category
    pub risk_category: CountryRiskCategory,
}

/// Type of country exposure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CountryExposureType {
    /// Country of residence
    Residence,
    /// Country of citizenship
    Citizenship,
    /// Country of birth
    Birth,
    /// Business operations in country
    BusinessOperations,
    /// Regular transactions with country
    TransactionHistory,
}

/// Country risk category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CountryRiskCategory {
    /// Low risk (FATF compliant, low corruption)
    Low,
    /// Medium risk
    #[default]
    Medium,
    /// High risk (weak AML framework)
    High,
    /// Very high risk (FATF grey list)
    VeryHigh,
    /// Sanctioned / prohibited (FATF black list, OFAC)
    Sanctioned,
}

impl CountryRiskCategory {
    /// Risk weight (1.0 = standard).
    pub fn risk_weight(&self) -> f64 {
        match self {
            Self::Low => 0.7,
            Self::Medium => 1.0,
            Self::High => 1.5,
            Self::VeryHigh => 2.5,
            Self::Sanctioned => 10.0,
        }
    }

    /// Whether transactions should be blocked.
    pub fn is_prohibited(&self) -> bool {
        matches!(self, Self::Sanctioned)
    }
}

/// Cash intensity classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CashIntensity {
    /// Very low cash usage (<5%)
    VeryLow,
    /// Low cash usage (5-15%)
    #[default]
    Low,
    /// Moderate cash usage (15-30%)
    Moderate,
    /// High cash usage (30-50%)
    High,
    /// Very high cash usage (>50%)
    VeryHigh,
}

impl CashIntensity {
    /// Risk weight (1.0 = standard).
    pub fn risk_weight(&self) -> f64 {
        match self {
            Self::VeryLow => 0.8,
            Self::Low => 1.0,
            Self::Moderate => 1.3,
            Self::High => 1.8,
            Self::VeryHigh => 2.5,
        }
    }

    /// Expected cash transaction percentage.
    pub fn expected_percentage(&self) -> (f64, f64) {
        match self {
            Self::VeryLow => (0.0, 0.05),
            Self::Low => (0.05, 0.15),
            Self::Moderate => (0.15, 0.30),
            Self::High => (0.30, 0.50),
            Self::VeryHigh => (0.50, 1.0),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_tier_ordering() {
        assert!(RiskTier::Low < RiskTier::Medium);
        assert!(RiskTier::Medium < RiskTier::High);
        assert!(RiskTier::High < RiskTier::VeryHigh);
    }

    #[test]
    fn test_risk_tier_from_score() {
        assert_eq!(RiskTier::from_score(10), RiskTier::Low);
        assert_eq!(RiskTier::from_score(40), RiskTier::Medium);
        assert_eq!(RiskTier::from_score(60), RiskTier::High);
        assert_eq!(RiskTier::from_score(90), RiskTier::VeryHigh);
    }

    #[test]
    fn test_source_of_funds_risk() {
        assert!(
            SourceOfFunds::CryptoAssets.risk_weight() > SourceOfFunds::Employment.risk_weight()
        );
        assert!(SourceOfFunds::Employment.risk_weight() < 1.0);
    }

    #[test]
    fn test_country_risk_category() {
        assert!(CountryRiskCategory::Sanctioned.is_prohibited());
        assert!(!CountryRiskCategory::High.is_prohibited());
    }

    #[test]
    fn test_cash_intensity() {
        let (min, max) = CashIntensity::High.expected_percentage();
        assert!(min >= 0.30);
        assert!(max <= 0.50);
    }
}
