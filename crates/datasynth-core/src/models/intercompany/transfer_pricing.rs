//! Transfer pricing models and policies for intercompany transactions.

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Transfer pricing method for intercompany transactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TransferPricingMethod {
    /// Cost plus a markup percentage.
    #[default]
    CostPlus,
    /// Comparable uncontrolled price (market-based).
    ComparableUncontrolled,
    /// Resale price minus a margin.
    ResalePrice,
    /// Transactional net margin method.
    TransactionalNetMargin,
    /// Profit split method.
    ProfitSplit,
    /// Fixed fee arrangement.
    FixedFee,
}

impl TransferPricingMethod {
    /// Get the typical markup/margin range for this method.
    pub fn typical_margin_range(&self) -> (Decimal, Decimal) {
        match self {
            Self::CostPlus => (Decimal::new(3, 2), Decimal::new(15, 2)), // 3-15%
            Self::ComparableUncontrolled => (Decimal::ZERO, Decimal::ZERO), // Market price
            Self::ResalePrice => (Decimal::new(10, 2), Decimal::new(30, 2)), // 10-30%
            Self::TransactionalNetMargin => (Decimal::new(2, 2), Decimal::new(10, 2)), // 2-10%
            Self::ProfitSplit => (Decimal::new(40, 2), Decimal::new(60, 2)), // 40-60% split
            Self::FixedFee => (Decimal::ZERO, Decimal::ZERO),            // Fixed amount
        }
    }

    /// Check if this method is cost-based.
    pub fn is_cost_based(&self) -> bool {
        matches!(self, Self::CostPlus | Self::TransactionalNetMargin)
    }

    /// Check if this method requires comparable data.
    pub fn requires_comparables(&self) -> bool {
        matches!(
            self,
            Self::ComparableUncontrolled | Self::ResalePrice | Self::TransactionalNetMargin
        )
    }
}

/// A transfer pricing policy applicable to intercompany transactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferPricingPolicy {
    /// Unique policy identifier.
    pub policy_id: String,
    /// Policy name/description.
    pub name: String,
    /// Transfer pricing method used.
    pub method: TransferPricingMethod,
    /// Markup or margin percentage (interpretation depends on method).
    pub markup_percent: Decimal,
    /// Minimum markup (for range-based policies).
    pub min_markup_percent: Option<Decimal>,
    /// Maximum markup (for range-based policies).
    pub max_markup_percent: Option<Decimal>,
    /// Transaction types this policy applies to.
    pub applicable_transaction_types: Vec<String>,
    /// Effective date of the policy.
    pub effective_date: NaiveDate,
    /// End date of the policy (if replaced).
    pub end_date: Option<NaiveDate>,
    /// Currency for fixed fee policies.
    pub fee_currency: Option<String>,
    /// Fixed fee amount (for FixedFee method).
    pub fixed_fee_amount: Option<Decimal>,
    /// Documentation requirements.
    pub documentation_requirements: DocumentationLevel,
    /// Whether annual benchmarking is required.
    pub requires_annual_benchmarking: bool,
}

impl TransferPricingPolicy {
    /// Create a new cost-plus policy.
    pub fn new_cost_plus(
        policy_id: String,
        name: String,
        markup_percent: Decimal,
        effective_date: NaiveDate,
    ) -> Self {
        Self {
            policy_id,
            name,
            method: TransferPricingMethod::CostPlus,
            markup_percent,
            min_markup_percent: None,
            max_markup_percent: None,
            applicable_transaction_types: Vec::new(),
            effective_date,
            end_date: None,
            fee_currency: None,
            fixed_fee_amount: None,
            documentation_requirements: DocumentationLevel::Standard,
            requires_annual_benchmarking: false,
        }
    }

    /// Create a new fixed fee policy.
    pub fn new_fixed_fee(
        policy_id: String,
        name: String,
        fee_amount: Decimal,
        currency: String,
        effective_date: NaiveDate,
    ) -> Self {
        Self {
            policy_id,
            name,
            method: TransferPricingMethod::FixedFee,
            markup_percent: Decimal::ZERO,
            min_markup_percent: None,
            max_markup_percent: None,
            applicable_transaction_types: Vec::new(),
            effective_date,
            end_date: None,
            fee_currency: Some(currency),
            fixed_fee_amount: Some(fee_amount),
            documentation_requirements: DocumentationLevel::Standard,
            requires_annual_benchmarking: false,
        }
    }

    /// Check if the policy is active on a given date.
    pub fn is_active_on(&self, date: NaiveDate) -> bool {
        date >= self.effective_date && self.end_date.is_none_or(|end| date <= end)
    }

    /// Calculate the transfer price for a given cost.
    pub fn calculate_transfer_price(&self, cost: Decimal) -> Decimal {
        match self.method {
            TransferPricingMethod::CostPlus => {
                cost * (Decimal::ONE + self.markup_percent / Decimal::from(100))
            }
            TransferPricingMethod::FixedFee => self.fixed_fee_amount.unwrap_or(Decimal::ZERO),
            TransferPricingMethod::ResalePrice => {
                // For resale price, markup is actually a margin to subtract
                cost / (Decimal::ONE - self.markup_percent / Decimal::from(100))
            }
            TransferPricingMethod::TransactionalNetMargin => {
                cost * (Decimal::ONE + self.markup_percent / Decimal::from(100))
            }
            TransferPricingMethod::ProfitSplit => {
                // Profit split: apply the markup_percent as the seller's profit share
                // For example, if markup_percent is 50, seller keeps 50% of total profit
                // We approximate total profit as a percentage of cost (typical 10-20% industry margin)
                let industry_margin = Decimal::new(15, 2); // Assume 15% industry profit
                let total_profit = cost * industry_margin;
                let seller_share = total_profit * self.markup_percent / Decimal::from(100);
                cost + seller_share
            }
            TransferPricingMethod::ComparableUncontrolled => {
                // Comparable uncontrolled price: use market adjustment
                // The markup_percent represents market price premium/discount vs cost
                // Positive = market price above cost, negative = below cost
                cost * (Decimal::ONE + self.markup_percent / Decimal::from(100))
            }
        }
    }

    /// Calculate the markup amount for a given cost.
    pub fn calculate_markup(&self, cost: Decimal) -> Decimal {
        self.calculate_transfer_price(cost) - cost
    }
}

/// Level of documentation required for transfer pricing compliance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DocumentationLevel {
    /// Minimal documentation.
    Minimal,
    /// Standard documentation.
    #[default]
    Standard,
    /// Comprehensive documentation (for high-risk transactions).
    Comprehensive,
    /// Country-by-country reporting level.
    CbCR,
}

/// Result of a transfer pricing calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferPriceCalculation {
    /// The policy used for calculation.
    pub policy_id: String,
    /// Original cost/base amount.
    pub base_amount: Decimal,
    /// Calculated transfer price.
    pub transfer_price: Decimal,
    /// Markup/margin amount.
    pub markup_amount: Decimal,
    /// Effective markup percentage.
    pub effective_markup_percent: Decimal,
    /// Currency of the amounts.
    pub currency: String,
    /// Date of calculation.
    pub calculation_date: NaiveDate,
    /// Whether the price is within arm's length range.
    pub is_arms_length: bool,
}

impl TransferPriceCalculation {
    /// Create a new transfer price calculation.
    pub fn new(
        policy: &TransferPricingPolicy,
        base_amount: Decimal,
        currency: String,
        calculation_date: NaiveDate,
    ) -> Self {
        let transfer_price = policy.calculate_transfer_price(base_amount);
        let markup_amount = transfer_price - base_amount;
        let effective_markup_percent = if base_amount != Decimal::ZERO {
            (markup_amount / base_amount) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };

        // Check if within arm's length range
        let is_arms_length = match (policy.min_markup_percent, policy.max_markup_percent) {
            (Some(min), Some(max)) => {
                effective_markup_percent >= min && effective_markup_percent <= max
            }
            _ => true, // No range specified, assume compliant
        };

        Self {
            policy_id: policy.policy_id.clone(),
            base_amount,
            transfer_price,
            markup_amount,
            effective_markup_percent,
            currency,
            calculation_date,
            is_arms_length,
        }
    }
}

/// Arm's length range for benchmarking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmsLengthRange {
    /// Lower quartile (25th percentile).
    pub lower_quartile: Decimal,
    /// Median (50th percentile).
    pub median: Decimal,
    /// Upper quartile (75th percentile).
    pub upper_quartile: Decimal,
    /// Interquartile range.
    pub iqr: Decimal,
    /// Number of comparables used.
    pub comparable_count: usize,
    /// Date of the benchmarking study.
    pub study_date: NaiveDate,
    /// Validity period in months.
    pub validity_months: u32,
}

impl ArmsLengthRange {
    /// Create a new arm's length range from benchmarking data.
    pub fn new(
        lower_quartile: Decimal,
        median: Decimal,
        upper_quartile: Decimal,
        comparable_count: usize,
        study_date: NaiveDate,
    ) -> Self {
        Self {
            lower_quartile,
            median,
            upper_quartile,
            iqr: upper_quartile - lower_quartile,
            comparable_count,
            study_date,
            validity_months: 36, // Typical 3-year validity
        }
    }

    /// Check if a margin falls within the arm's length range.
    pub fn is_within_range(&self, margin: Decimal) -> bool {
        margin >= self.lower_quartile && margin <= self.upper_quartile
    }

    /// Check if the range is still valid.
    pub fn is_valid_on(&self, date: NaiveDate) -> bool {
        let months_elapsed = (date.year() - self.study_date.year()) * 12
            + (date.month() as i32 - self.study_date.month() as i32);
        months_elapsed >= 0 && (months_elapsed as u32) <= self.validity_months
    }

    /// Get adjustment needed to bring margin within range.
    pub fn get_adjustment(&self, margin: Decimal) -> Option<Decimal> {
        if margin < self.lower_quartile || margin > self.upper_quartile {
            Some(self.median - margin)
        } else {
            None
        }
    }
}

/// Transfer pricing adjustment for year-end true-up.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferPricingAdjustment {
    /// Adjustment identifier.
    pub adjustment_id: String,
    /// Related policy.
    pub policy_id: String,
    /// Seller company.
    pub seller_company: String,
    /// Buyer company.
    pub buyer_company: String,
    /// Fiscal year of adjustment.
    pub fiscal_year: i32,
    /// Original aggregate transfer prices.
    pub original_amount: Decimal,
    /// Adjusted aggregate transfer prices.
    pub adjusted_amount: Decimal,
    /// Net adjustment amount.
    pub adjustment_amount: Decimal,
    /// Currency.
    pub currency: String,
    /// Reason for adjustment.
    pub adjustment_reason: AdjustmentReason,
    /// Date of adjustment.
    pub adjustment_date: NaiveDate,
}

/// Reason for transfer pricing adjustment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdjustmentReason {
    /// Year-end true-up to target margin.
    YearEndTrueUp,
    /// Benchmarking study update.
    BenchmarkingUpdate,
    /// Tax authority adjustment.
    TaxAuthorityAdjustment,
    /// Advance pricing agreement.
    ApaPricing,
    /// Competent authority resolution.
    CompetentAuthority,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_cost_plus_calculation() {
        let policy = TransferPricingPolicy::new_cost_plus(
            "TP001".to_string(),
            "Standard Cost Plus".to_string(),
            dec!(5), // 5% markup
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        );

        let cost = dec!(1000);
        let transfer_price = policy.calculate_transfer_price(cost);
        assert_eq!(transfer_price, dec!(1050));

        let markup = policy.calculate_markup(cost);
        assert_eq!(markup, dec!(50));
    }

    #[test]
    fn test_fixed_fee_policy() {
        let policy = TransferPricingPolicy::new_fixed_fee(
            "TP002".to_string(),
            "Management Fee".to_string(),
            dec!(50000),
            "USD".to_string(),
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        );

        // Fixed fee ignores cost
        assert_eq!(policy.calculate_transfer_price(dec!(0)), dec!(50000));
        assert_eq!(policy.calculate_transfer_price(dec!(100000)), dec!(50000));
    }

    #[test]
    fn test_transfer_price_calculation() {
        let policy = TransferPricingPolicy::new_cost_plus(
            "TP001".to_string(),
            "Cost Plus 8%".to_string(),
            dec!(8),
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        );

        let calc = TransferPriceCalculation::new(
            &policy,
            dec!(10000),
            "USD".to_string(),
            NaiveDate::from_ymd_opt(2022, 6, 15).unwrap(),
        );

        assert_eq!(calc.base_amount, dec!(10000));
        assert_eq!(calc.transfer_price, dec!(10800));
        assert_eq!(calc.markup_amount, dec!(800));
        assert_eq!(calc.effective_markup_percent, dec!(8));
        assert!(calc.is_arms_length);
    }

    #[test]
    fn test_arms_length_range() {
        let range = ArmsLengthRange::new(
            dec!(3),
            dec!(5),
            dec!(8),
            15,
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        );

        assert!(range.is_within_range(dec!(5)));
        assert!(range.is_within_range(dec!(3)));
        assert!(range.is_within_range(dec!(8)));
        assert!(!range.is_within_range(dec!(2)));
        assert!(!range.is_within_range(dec!(10)));

        // Check adjustment
        assert_eq!(range.get_adjustment(dec!(1)), Some(dec!(4))); // Need to increase by 4
        assert_eq!(range.get_adjustment(dec!(5)), None); // Within range
    }

    #[test]
    fn test_policy_active_date() {
        let mut policy = TransferPricingPolicy::new_cost_plus(
            "TP001".to_string(),
            "Test Policy".to_string(),
            dec!(5),
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        );
        policy.end_date = Some(NaiveDate::from_ymd_opt(2023, 12, 31).unwrap());

        assert!(policy.is_active_on(NaiveDate::from_ymd_opt(2022, 6, 15).unwrap()));
        assert!(!policy.is_active_on(NaiveDate::from_ymd_opt(2021, 12, 31).unwrap()));
        assert!(!policy.is_active_on(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()));
    }
}
