//! KYC profile model for expected activity envelope.

use datasynth_core::models::banking::{
    CashIntensity, CountryExposure, FrequencyBand, SourceOfFunds, SourceOfWealth, TurnoverBand,
};
use serde::{Deserialize, Serialize};

/// KYC profile defining expected customer activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycProfile {
    /// Declared purpose of the account
    pub declared_purpose: String,
    /// Expected monthly turnover band
    pub expected_monthly_turnover: TurnoverBand,
    /// Expected transaction frequency band
    pub expected_transaction_frequency: FrequencyBand,
    /// Expected merchant/transaction categories
    pub expected_categories: Vec<ExpectedCategory>,
    /// Declared source of funds
    pub source_of_funds: SourceOfFunds,
    /// Declared source of wealth (for HNW)
    pub source_of_wealth: Option<SourceOfWealth>,
    /// Geographic exposure (countries)
    pub geographic_exposure: Vec<CountryExposure>,
    /// Expected cash intensity
    pub cash_intensity: CashIntensity,
    /// Beneficial owner complexity score (0-10)
    pub beneficial_owner_complexity: u8,
    /// Expected international transaction rate (0.0-1.0)
    pub international_rate: f64,
    /// Expected large transaction rate (0.0-1.0)
    pub large_transaction_rate: f64,
    /// Threshold for "large" transaction
    pub large_transaction_threshold: u64,
    /// KYC completeness score (0.0-1.0)
    pub completeness_score: f64,

    // Ground truth (for deception modeling)
    /// True source of funds (if different from declared)
    pub true_source_of_funds: Option<SourceOfFunds>,
    /// True expected turnover (if different from declared)
    pub true_turnover: Option<TurnoverBand>,
    /// Whether the KYC profile is truthful
    pub is_truthful: bool,
}

impl Default for KycProfile {
    fn default() -> Self {
        Self {
            declared_purpose: "Personal banking".to_string(),
            expected_monthly_turnover: TurnoverBand::default(),
            expected_transaction_frequency: FrequencyBand::default(),
            expected_categories: Vec::new(),
            source_of_funds: SourceOfFunds::Employment,
            source_of_wealth: None,
            geographic_exposure: Vec::new(),
            cash_intensity: CashIntensity::default(),
            beneficial_owner_complexity: 0,
            international_rate: 0.05,
            large_transaction_rate: 0.02,
            large_transaction_threshold: 10_000,
            completeness_score: 1.0,
            true_source_of_funds: None,
            true_turnover: None,
            is_truthful: true,
        }
    }
}

impl KycProfile {
    /// Create a new KYC profile.
    pub fn new(purpose: &str, source_of_funds: SourceOfFunds) -> Self {
        Self {
            declared_purpose: purpose.to_string(),
            source_of_funds,
            ..Default::default()
        }
    }

    /// Create a profile for a retail customer.
    pub fn retail_standard() -> Self {
        Self {
            declared_purpose: "Personal checking and savings".to_string(),
            expected_monthly_turnover: TurnoverBand::Low,
            expected_transaction_frequency: FrequencyBand::Medium,
            source_of_funds: SourceOfFunds::Employment,
            cash_intensity: CashIntensity::Low,
            ..Default::default()
        }
    }

    /// Create a profile for a high net worth customer.
    pub fn high_net_worth() -> Self {
        Self {
            declared_purpose: "Wealth management and investment".to_string(),
            expected_monthly_turnover: TurnoverBand::VeryHigh,
            expected_transaction_frequency: FrequencyBand::High,
            source_of_funds: SourceOfFunds::Investments,
            source_of_wealth: Some(SourceOfWealth::BusinessOwnership),
            cash_intensity: CashIntensity::VeryLow,
            international_rate: 0.20,
            large_transaction_rate: 0.15,
            large_transaction_threshold: 50_000,
            ..Default::default()
        }
    }

    /// Create a profile for a small business.
    pub fn small_business() -> Self {
        Self {
            declared_purpose: "Business operations".to_string(),
            expected_monthly_turnover: TurnoverBand::Medium,
            expected_transaction_frequency: FrequencyBand::High,
            source_of_funds: SourceOfFunds::SelfEmployment,
            cash_intensity: CashIntensity::Moderate,
            large_transaction_rate: 0.05,
            large_transaction_threshold: 25_000,
            ..Default::default()
        }
    }

    /// Create a profile for a cash-intensive business.
    pub fn cash_intensive_business() -> Self {
        Self {
            declared_purpose: "Retail business operations".to_string(),
            expected_monthly_turnover: TurnoverBand::High,
            expected_transaction_frequency: FrequencyBand::VeryHigh,
            source_of_funds: SourceOfFunds::SelfEmployment,
            cash_intensity: CashIntensity::VeryHigh,
            large_transaction_rate: 0.01,
            large_transaction_threshold: 10_000,
            ..Default::default()
        }
    }

    /// Set turnover band.
    pub fn with_turnover(mut self, turnover: TurnoverBand) -> Self {
        self.expected_monthly_turnover = turnover;
        self
    }

    /// Set frequency band.
    pub fn with_frequency(mut self, frequency: FrequencyBand) -> Self {
        self.expected_transaction_frequency = frequency;
        self
    }

    /// Add expected category.
    pub fn with_expected_category(mut self, category: ExpectedCategory) -> Self {
        self.expected_categories.push(category);
        self
    }

    /// Add geographic exposure.
    pub fn with_country_exposure(mut self, exposure: CountryExposure) -> Self {
        self.geographic_exposure.push(exposure);
        self
    }

    /// Set cash intensity.
    pub fn with_cash_intensity(mut self, intensity: CashIntensity) -> Self {
        self.cash_intensity = intensity;
        self
    }

    /// Set as deceptive (ground truth differs from declared).
    pub fn with_deception(
        mut self,
        true_source: SourceOfFunds,
        true_turnover: Option<TurnoverBand>,
    ) -> Self {
        self.true_source_of_funds = Some(true_source);
        self.true_turnover = true_turnover;
        self.is_truthful = false;
        self
    }

    /// Calculate risk score based on profile.
    pub fn calculate_risk_score(&self) -> u8 {
        let mut score = 0.0;

        // Source of funds risk
        score += self.source_of_funds.risk_weight() * 15.0;

        // Turnover risk (higher turnover = higher risk)
        let (_, max_turnover) = self.expected_monthly_turnover.range();
        if max_turnover > 100_000 {
            score += 15.0;
        } else if max_turnover > 25_000 {
            score += 10.0;
        } else if max_turnover > 5_000 {
            score += 5.0;
        }

        // Cash intensity risk
        score += self.cash_intensity.risk_weight() * 10.0;

        // International exposure risk
        score += self.international_rate * 20.0;

        // UBO complexity risk
        score += (self.beneficial_owner_complexity as f64) * 2.0;

        // Deception risk (if ground truth available)
        if !self.is_truthful {
            score += 25.0;
        }

        // Completeness penalty
        score += (1.0 - self.completeness_score) * 10.0;

        score.min(100.0) as u8
    }

    /// Check if actual activity matches expected.
    pub fn is_within_expected_turnover(&self, actual_monthly: u64) -> bool {
        let (min, max) = self.expected_monthly_turnover.range();
        actual_monthly >= min && actual_monthly <= max * 2 // Allow some tolerance
    }

    /// Check if transaction frequency is within expected.
    pub fn is_within_expected_frequency(&self, actual_count: u32) -> bool {
        let (min, max) = self.expected_transaction_frequency.range();
        actual_count >= min / 2 && actual_count <= max * 2
    }
}

/// Expected transaction category with weight.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedCategory {
    /// Category name
    pub category: String,
    /// Expected percentage of transactions (0.0-1.0)
    pub expected_percentage: f64,
    /// Tolerance for deviation
    pub tolerance: f64,
}

impl ExpectedCategory {
    /// Create a new expected category.
    pub fn new(category: &str, percentage: f64) -> Self {
        Self {
            category: category.to_string(),
            expected_percentage: percentage,
            tolerance: 0.1, // 10% tolerance
        }
    }

    /// Check if actual matches expected.
    pub fn matches(&self, actual_percentage: f64) -> bool {
        (actual_percentage - self.expected_percentage).abs() <= self.tolerance
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_kyc_profile_default() {
        let profile = KycProfile::default();
        assert!(profile.is_truthful);
        assert_eq!(profile.source_of_funds, SourceOfFunds::Employment);
    }

    #[test]
    fn test_kyc_profile_presets() {
        let retail = KycProfile::retail_standard();
        assert_eq!(retail.expected_monthly_turnover, TurnoverBand::Low);

        let hnw = KycProfile::high_net_worth();
        assert_eq!(hnw.expected_monthly_turnover, TurnoverBand::VeryHigh);
        assert!(hnw.source_of_wealth.is_some());
    }

    #[test]
    fn test_deceptive_profile() {
        let profile = KycProfile::retail_standard()
            .with_deception(SourceOfFunds::CryptoAssets, Some(TurnoverBand::VeryHigh));

        assert!(!profile.is_truthful);
        assert!(profile.true_source_of_funds.is_some());

        let base_score = KycProfile::retail_standard().calculate_risk_score();
        let deceptive_score = profile.calculate_risk_score();
        assert!(deceptive_score > base_score);
    }

    #[test]
    fn test_turnover_check() {
        let profile = KycProfile::default().with_turnover(TurnoverBand::Medium);
        // Medium is 5,000 - 25,000
        assert!(profile.is_within_expected_turnover(10_000));
        assert!(profile.is_within_expected_turnover(40_000)); // Within 2x tolerance
        assert!(!profile.is_within_expected_turnover(100_000));
    }
}
