//! Industry-specific amount distribution profiles.
//!
//! Pre-configured distribution profiles for different industries based on
//! typical transaction patterns observed in each sector.

use super::mixture::{LogNormalComponent, LogNormalMixtureConfig};
use super::pareto::ParetoConfig;
use super::weibull::WeibullConfig;
use serde::{Deserialize, Serialize};

/// Industry type for profile selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IndustryType {
    /// Retail industry (B2C, POS transactions)
    Retail,
    /// Manufacturing industry (B2B, production)
    #[default]
    Manufacturing,
    /// Financial services (banking, insurance)
    FinancialServices,
    /// Healthcare
    Healthcare,
    /// Technology / SaaS
    Technology,
    /// Wholesale / Distribution
    Wholesale,
    /// Professional Services
    ProfessionalServices,
    /// Construction
    Construction,
}

/// Complete industry amount profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustryAmountProfile {
    /// Industry type
    pub industry: IndustryType,
    /// Sales/revenue transaction amounts
    pub sales_amounts: LogNormalMixtureConfig,
    /// Purchase transaction amounts
    pub purchase_amounts: LogNormalMixtureConfig,
    /// Payroll transaction amounts
    pub payroll_amounts: LogNormalMixtureConfig,
    /// Capital expenditure amounts (heavy-tailed)
    pub capex_amounts: ParetoConfig,
    /// Days-to-payment distribution
    pub days_to_payment: WeibullConfig,
    /// Seasonality multipliers by month (Jan=0, Dec=11)
    pub seasonality: [f64; 12],
    /// Typical line item count range
    pub line_item_range: (u8, u8),
    /// Average transaction volume per day
    pub avg_daily_transactions: u32,
}

impl Default for IndustryAmountProfile {
    fn default() -> Self {
        Self::manufacturing()
    }
}

impl IndustryAmountProfile {
    /// Create a retail industry profile.
    ///
    /// Characteristics:
    /// - High volume of small POS transactions
    /// - Mixture of cash registers, online, and returns
    /// - Strong seasonality (Q4 spike)
    /// - Fast payment terms
    pub fn retail() -> Self {
        Self {
            industry: IndustryType::Retail,
            sales_amounts: LogNormalMixtureConfig {
                components: vec![
                    // POS transactions (60%) - small purchases
                    LogNormalComponent::with_label(0.60, 3.5, 1.0, "pos_small"),
                    // Medium purchases (25%) - grocery, apparel
                    LogNormalComponent::with_label(0.25, 4.5, 0.8, "medium"),
                    // Large purchases (10%) - electronics, furniture
                    LogNormalComponent::with_label(0.10, 6.0, 1.2, "large"),
                    // High-value (5%) - luxury items
                    LogNormalComponent::with_label(0.05, 7.5, 0.9, "luxury"),
                ],
                min_value: 0.01,
                max_value: Some(50_000.0),
                decimal_places: 2,
            },
            purchase_amounts: LogNormalMixtureConfig {
                components: vec![
                    // Regular inventory orders (70%)
                    LogNormalComponent::with_label(0.70, 7.0, 1.5, "inventory"),
                    // Large bulk orders (25%)
                    LogNormalComponent::with_label(0.25, 9.0, 1.0, "bulk"),
                    // Special/seasonal (5%)
                    LogNormalComponent::with_label(0.05, 10.0, 0.8, "seasonal"),
                ],
                min_value: 100.0,
                max_value: Some(1_000_000.0),
                decimal_places: 2,
            },
            payroll_amounts: LogNormalMixtureConfig {
                components: vec![
                    // Hourly/part-time (60%)
                    LogNormalComponent::with_label(0.60, 6.5, 0.6, "hourly"),
                    // Full-time staff (35%)
                    LogNormalComponent::with_label(0.35, 7.5, 0.5, "salary"),
                    // Management (5%)
                    LogNormalComponent::with_label(0.05, 8.5, 0.4, "management"),
                ],
                min_value: 200.0,
                max_value: Some(50_000.0),
                decimal_places: 2,
            },
            capex_amounts: ParetoConfig {
                alpha: 2.0,
                x_min: 5_000.0,
                max_value: Some(500_000.0),
                decimal_places: 2,
            },
            days_to_payment: WeibullConfig::days_to_payment(),
            seasonality: [
                0.75, // Jan - post-holiday lull
                0.70, // Feb
                0.85, // Mar
                0.90, // Apr
                0.95, // May
                0.90, // Jun
                0.85, // Jul
                0.90, // Aug - back to school
                0.95, // Sep
                1.10, // Oct
                1.40, // Nov - Black Friday
                1.75, // Dec - Holiday peak
            ],
            line_item_range: (1, 50),
            avg_daily_transactions: 500,
        }
    }

    /// Create a manufacturing industry profile.
    ///
    /// Characteristics:
    /// - Mix of raw materials, components, and finished goods
    /// - Larger average transaction sizes
    /// - B2B payment terms (Net 30-60)
    /// - Production-driven seasonality
    pub fn manufacturing() -> Self {
        Self {
            industry: IndustryType::Manufacturing,
            sales_amounts: LogNormalMixtureConfig {
                components: vec![
                    // Standard product orders (50%)
                    LogNormalComponent::with_label(0.50, 8.0, 1.5, "standard"),
                    // Large orders (35%)
                    LogNormalComponent::with_label(0.35, 10.0, 1.0, "large"),
                    // Enterprise/contract orders (15%)
                    LogNormalComponent::with_label(0.15, 12.0, 0.8, "enterprise"),
                ],
                min_value: 500.0,
                max_value: Some(10_000_000.0),
                decimal_places: 2,
            },
            purchase_amounts: LogNormalMixtureConfig {
                components: vec![
                    // Raw materials (55%)
                    LogNormalComponent::with_label(0.55, 8.5, 1.5, "raw_materials"),
                    // Components/parts (30%)
                    LogNormalComponent::with_label(0.30, 7.5, 1.2, "components"),
                    // Equipment/tooling (15%)
                    LogNormalComponent::with_label(0.15, 10.0, 1.0, "equipment"),
                ],
                min_value: 100.0,
                max_value: Some(5_000_000.0),
                decimal_places: 2,
            },
            payroll_amounts: LogNormalMixtureConfig {
                components: vec![
                    // Production workers (50%)
                    LogNormalComponent::with_label(0.50, 7.5, 0.5, "production"),
                    // Technical staff (30%)
                    LogNormalComponent::with_label(0.30, 8.0, 0.4, "technical"),
                    // Management (15%)
                    LogNormalComponent::with_label(0.15, 9.0, 0.5, "management"),
                    // Executive (5%)
                    LogNormalComponent::with_label(0.05, 10.0, 0.4, "executive"),
                ],
                min_value: 1000.0,
                max_value: Some(100_000.0),
                decimal_places: 2,
            },
            capex_amounts: ParetoConfig {
                alpha: 1.5, // Heavier tail - large equipment purchases
                x_min: 25_000.0,
                max_value: Some(10_000_000.0),
                decimal_places: 2,
            },
            days_to_payment: WeibullConfig {
                shape: 2.0,
                scale: 45.0, // Net 45 typical
                min_value: 5.0,
                max_value: Some(90.0),
                round_to_integer: true,
            },
            seasonality: [
                0.90, // Jan
                0.95, // Feb
                1.00, // Mar
                1.05, // Apr
                1.00, // May
                0.95, // Jun
                0.85, // Jul - summer slowdown
                0.90, // Aug
                1.05, // Sep
                1.10, // Oct
                1.05, // Nov
                0.85, // Dec - holiday shutdown
            ],
            line_item_range: (2, 25),
            avg_daily_transactions: 50,
        }
    }

    /// Create a financial services industry profile.
    ///
    /// Characteristics:
    /// - High-value wire transfers and ACH
    /// - Fee-based income (many small transactions)
    /// - Regulatory-driven patterns
    /// - Month-end/quarter-end spikes
    pub fn financial_services() -> Self {
        Self {
            industry: IndustryType::FinancialServices,
            sales_amounts: LogNormalMixtureConfig {
                components: vec![
                    // ACH/small transfers (40%)
                    LogNormalComponent::with_label(0.40, 6.0, 1.5, "ach_small"),
                    // Medium transactions (30%)
                    LogNormalComponent::with_label(0.30, 9.0, 1.5, "medium"),
                    // Large wire transfers (20%)
                    LogNormalComponent::with_label(0.20, 12.0, 2.0, "wire_large"),
                    // Institutional (10%)
                    LogNormalComponent::with_label(0.10, 15.0, 1.5, "institutional"),
                ],
                min_value: 1.0,
                max_value: Some(100_000_000.0),
                decimal_places: 2,
            },
            purchase_amounts: LogNormalMixtureConfig {
                components: vec![
                    // Software/licenses (40%)
                    LogNormalComponent::with_label(0.40, 7.0, 1.0, "software"),
                    // Professional services (35%)
                    LogNormalComponent::with_label(0.35, 9.0, 1.2, "professional"),
                    // Technology infrastructure (25%)
                    LogNormalComponent::with_label(0.25, 11.0, 1.0, "infrastructure"),
                ],
                min_value: 500.0,
                max_value: Some(10_000_000.0),
                decimal_places: 2,
            },
            payroll_amounts: LogNormalMixtureConfig {
                components: vec![
                    // Operations/support (30%)
                    LogNormalComponent::with_label(0.30, 8.0, 0.5, "operations"),
                    // Analysts/associates (35%)
                    LogNormalComponent::with_label(0.35, 9.0, 0.4, "analyst"),
                    // Senior professionals (25%)
                    LogNormalComponent::with_label(0.25, 10.0, 0.4, "senior"),
                    // Executives (10%)
                    LogNormalComponent::with_label(0.10, 11.5, 0.5, "executive"),
                ],
                min_value: 2000.0,
                max_value: Some(500_000.0),
                decimal_places: 2,
            },
            capex_amounts: ParetoConfig {
                alpha: 1.8,
                x_min: 50_000.0,
                max_value: Some(50_000_000.0),
                decimal_places: 2,
            },
            days_to_payment: WeibullConfig {
                shape: 3.0,  // More predictable
                scale: 10.0, // Fast payment cycles
                min_value: 1.0,
                max_value: Some(30.0),
                round_to_integer: true,
            },
            seasonality: [
                1.05, // Jan - year start
                0.95, // Feb
                1.15, // Mar - quarter end
                1.00, // Apr
                0.95, // May
                1.15, // Jun - quarter end
                0.90, // Jul
                0.90, // Aug
                1.15, // Sep - quarter end
                1.00, // Oct
                0.95, // Nov
                1.25, // Dec - year end
            ],
            line_item_range: (1, 10),
            avg_daily_transactions: 1000,
        }
    }

    /// Create a healthcare industry profile.
    ///
    /// Characteristics:
    /// - Insurance claims and patient payments
    /// - Medical supply procurement
    /// - Regulatory and compliance costs
    /// - Seasonal illness patterns
    pub fn healthcare() -> Self {
        Self {
            industry: IndustryType::Healthcare,
            sales_amounts: LogNormalMixtureConfig {
                components: vec![
                    // Copays/small claims (40%)
                    LogNormalComponent::with_label(0.40, 4.0, 1.0, "copay"),
                    // Standard procedures (35%)
                    LogNormalComponent::with_label(0.35, 7.0, 1.5, "procedures"),
                    // Specialist services (20%)
                    LogNormalComponent::with_label(0.20, 9.0, 1.2, "specialist"),
                    // Major treatments (5%)
                    LogNormalComponent::with_label(0.05, 11.0, 1.0, "major"),
                ],
                min_value: 10.0,
                max_value: Some(1_000_000.0),
                decimal_places: 2,
            },
            purchase_amounts: LogNormalMixtureConfig {
                components: vec![
                    // Consumable supplies (45%)
                    LogNormalComponent::with_label(0.45, 6.0, 1.2, "consumables"),
                    // Pharmaceuticals (35%)
                    LogNormalComponent::with_label(0.35, 8.0, 1.5, "pharma"),
                    // Medical equipment (20%)
                    LogNormalComponent::with_label(0.20, 10.0, 1.0, "equipment"),
                ],
                min_value: 50.0,
                max_value: Some(5_000_000.0),
                decimal_places: 2,
            },
            payroll_amounts: LogNormalMixtureConfig {
                components: vec![
                    // Support staff (35%)
                    LogNormalComponent::with_label(0.35, 7.5, 0.5, "support"),
                    // Nurses/technicians (35%)
                    LogNormalComponent::with_label(0.35, 8.5, 0.4, "clinical"),
                    // Physicians (25%)
                    LogNormalComponent::with_label(0.25, 10.0, 0.5, "physician"),
                    // Specialists (5%)
                    LogNormalComponent::with_label(0.05, 11.0, 0.4, "specialist"),
                ],
                min_value: 1500.0,
                max_value: Some(200_000.0),
                decimal_places: 2,
            },
            capex_amounts: ParetoConfig {
                alpha: 1.6,
                x_min: 10_000.0,
                max_value: Some(20_000_000.0),
                decimal_places: 2,
            },
            days_to_payment: WeibullConfig {
                shape: 1.5,  // More variance due to insurance
                scale: 60.0, // Insurance processing time
                min_value: 10.0,
                max_value: Some(180.0),
                round_to_integer: true,
            },
            seasonality: [
                1.15, // Jan - flu season
                1.10, // Feb
                1.00, // Mar
                0.95, // Apr
                0.90, // May
                0.90, // Jun
                0.85, // Jul
                0.90, // Aug
                0.95, // Sep
                1.00, // Oct
                1.05, // Nov
                1.10, // Dec - holiday injuries/illness
            ],
            line_item_range: (1, 30),
            avg_daily_transactions: 200,
        }
    }

    /// Create a technology/SaaS industry profile.
    ///
    /// Characteristics:
    /// - Subscription-based revenue
    /// - High R&D and cloud costs
    /// - Fast growth patterns
    /// - Minimal seasonality
    pub fn technology() -> Self {
        Self {
            industry: IndustryType::Technology,
            sales_amounts: LogNormalMixtureConfig {
                components: vec![
                    // SMB subscriptions (50%)
                    LogNormalComponent::with_label(0.50, 5.5, 1.0, "smb"),
                    // Mid-market (30%)
                    LogNormalComponent::with_label(0.30, 8.0, 1.0, "midmarket"),
                    // Enterprise contracts (15%)
                    LogNormalComponent::with_label(0.15, 10.5, 1.2, "enterprise"),
                    // Large deals (5%)
                    LogNormalComponent::with_label(0.05, 13.0, 0.8, "strategic"),
                ],
                min_value: 10.0,
                max_value: Some(10_000_000.0),
                decimal_places: 2,
            },
            purchase_amounts: LogNormalMixtureConfig {
                components: vec![
                    // SaaS tools (40%)
                    LogNormalComponent::with_label(0.40, 6.0, 1.0, "saas"),
                    // Cloud infrastructure (35%)
                    LogNormalComponent::with_label(0.35, 8.5, 1.5, "cloud"),
                    // Hardware/equipment (15%)
                    LogNormalComponent::with_label(0.15, 7.5, 1.0, "hardware"),
                    // Contractors (10%)
                    LogNormalComponent::with_label(0.10, 9.0, 1.0, "contractors"),
                ],
                min_value: 50.0,
                max_value: Some(5_000_000.0),
                decimal_places: 2,
            },
            payroll_amounts: LogNormalMixtureConfig {
                components: vec![
                    // Junior engineers (25%)
                    LogNormalComponent::with_label(0.25, 8.5, 0.4, "junior"),
                    // Mid-level (40%)
                    LogNormalComponent::with_label(0.40, 9.2, 0.3, "mid"),
                    // Senior engineers (25%)
                    LogNormalComponent::with_label(0.25, 10.0, 0.3, "senior"),
                    // Leadership (10%)
                    LogNormalComponent::with_label(0.10, 11.0, 0.4, "leadership"),
                ],
                min_value: 3000.0,
                max_value: Some(300_000.0),
                decimal_places: 2,
            },
            capex_amounts: ParetoConfig {
                alpha: 2.2, // Less extreme tail
                x_min: 10_000.0,
                max_value: Some(2_000_000.0),
                decimal_places: 2,
            },
            days_to_payment: WeibullConfig {
                shape: 2.5,  // Predictable (often auto-pay)
                scale: 15.0, // Fast cycles
                min_value: 0.0,
                max_value: Some(45.0),
                round_to_integer: true,
            },
            seasonality: [
                0.95, // Jan
                0.95, // Feb
                1.00, // Mar
                1.00, // Apr
                1.00, // May
                1.00, // Jun
                0.95, // Jul
                0.95, // Aug
                1.05, // Sep
                1.05, // Oct
                1.00, // Nov
                1.05, // Dec
            ],
            line_item_range: (1, 15),
            avg_daily_transactions: 100,
        }
    }

    /// Get the industry profile for a given industry type.
    pub fn for_industry(industry: IndustryType) -> Self {
        match industry {
            IndustryType::Retail => Self::retail(),
            IndustryType::Manufacturing => Self::manufacturing(),
            IndustryType::FinancialServices => Self::financial_services(),
            IndustryType::Healthcare => Self::healthcare(),
            IndustryType::Technology => Self::technology(),
            IndustryType::Wholesale => Self::manufacturing(), // Similar to manufacturing
            IndustryType::ProfessionalServices => Self::technology(), // Similar to tech
            IndustryType::Construction => Self::manufacturing(), // Similar pattern
        }
    }

    /// Get the seasonality multiplier for a given month (0 = January).
    pub fn seasonality_multiplier(&self, month: u8) -> f64 {
        self.seasonality[(month % 12) as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retail_profile() {
        let profile = IndustryAmountProfile::retail();
        assert_eq!(profile.industry, IndustryType::Retail);
        assert!(profile.sales_amounts.validate().is_ok());
        assert!(profile.purchase_amounts.validate().is_ok());
    }

    #[test]
    fn test_manufacturing_profile() {
        let profile = IndustryAmountProfile::manufacturing();
        assert_eq!(profile.industry, IndustryType::Manufacturing);
        assert!(profile.sales_amounts.validate().is_ok());
    }

    #[test]
    fn test_financial_services_profile() {
        let profile = IndustryAmountProfile::financial_services();
        assert_eq!(profile.industry, IndustryType::FinancialServices);
        assert!(profile.sales_amounts.validate().is_ok());
    }

    #[test]
    fn test_healthcare_profile() {
        let profile = IndustryAmountProfile::healthcare();
        assert_eq!(profile.industry, IndustryType::Healthcare);
        assert!(profile.sales_amounts.validate().is_ok());
    }

    #[test]
    fn test_technology_profile() {
        let profile = IndustryAmountProfile::technology();
        assert_eq!(profile.industry, IndustryType::Technology);
        assert!(profile.sales_amounts.validate().is_ok());
    }

    #[test]
    fn test_seasonality() {
        let retail = IndustryAmountProfile::retail();

        // December should be highest for retail
        assert_eq!(retail.seasonality_multiplier(11), 1.75);

        // February should be lowest
        assert_eq!(retail.seasonality_multiplier(1), 0.70);

        // Seasonality factors should be reasonable
        for month in 0..12 {
            let factor = retail.seasonality_multiplier(month);
            assert!(factor > 0.5 && factor < 2.0);
        }
    }

    #[test]
    fn test_for_industry() {
        let retail = IndustryAmountProfile::for_industry(IndustryType::Retail);
        assert_eq!(retail.industry, IndustryType::Retail);

        let tech = IndustryAmountProfile::for_industry(IndustryType::Technology);
        assert_eq!(tech.industry, IndustryType::Technology);
    }

    #[test]
    fn test_component_weights_sum() {
        let profiles = [
            IndustryAmountProfile::retail(),
            IndustryAmountProfile::manufacturing(),
            IndustryAmountProfile::financial_services(),
            IndustryAmountProfile::healthcare(),
            IndustryAmountProfile::technology(),
        ];

        for profile in &profiles {
            let sales_sum: f64 = profile
                .sales_amounts
                .components
                .iter()
                .map(|c| c.weight)
                .sum();
            assert!(
                (sales_sum - 1.0).abs() < 0.01,
                "Sales weights should sum to 1.0"
            );

            let purchase_sum: f64 = profile
                .purchase_amounts
                .components
                .iter()
                .map(|c| c.weight)
                .sum();
            assert!(
                (purchase_sum - 1.0).abs() < 0.01,
                "Purchase weights should sum to 1.0"
            );

            let payroll_sum: f64 = profile
                .payroll_amounts
                .components
                .iter()
                .map(|c| c.weight)
                .sum();
            assert!(
                (payroll_sum - 1.0).abs() < 0.01,
                "Payroll weights should sum to 1.0"
            );
        }
    }
}
