//! Revenue Recognition Models (ASC 606 / IFRS 15).
//!
//! Implements the five-step model for revenue recognition:
//! 1. Identify the contract with a customer
//! 2. Identify the performance obligations
//! 3. Determine the transaction price
//! 4. Allocate the transaction price
//! 5. Recognize revenue when performance obligations are satisfied
//!
//! This module generates realistic customer contracts, performance obligations,
//! and revenue recognition schedules that comply with ASC 606 / IFRS 15.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::framework::AccountingFramework;

/// Customer contract for revenue recognition.
///
/// Represents Step 1 of the revenue recognition model: identifying
/// the contract with the customer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerContract {
    /// Unique contract identifier.
    pub contract_id: Uuid,

    /// Reference to the customer entity.
    pub customer_id: String,

    /// Customer name for reporting.
    pub customer_name: String,

    /// Company code this contract belongs to.
    pub company_code: String,

    /// Contract inception date.
    pub inception_date: NaiveDate,

    /// Contract end date (if determinable).
    pub end_date: Option<NaiveDate>,

    /// Total transaction price before allocation.
    #[serde(with = "rust_decimal::serde::str")]
    pub transaction_price: Decimal,

    /// Currency of the contract.
    pub currency: String,

    /// Contract status.
    pub status: ContractStatus,

    /// Performance obligations within this contract.
    pub performance_obligations: Vec<PerformanceObligation>,

    /// Variable consideration components.
    pub variable_consideration: Vec<VariableConsideration>,

    /// Whether contract contains a significant financing component.
    pub has_significant_financing: bool,

    /// Discount rate for significant financing component.
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub financing_rate: Option<Decimal>,

    /// Accounting framework applied.
    pub framework: AccountingFramework,

    /// Contract modification history.
    pub modifications: Vec<ContractModification>,

    /// Reference to related sales order (O2C integration).
    pub sales_order_id: Option<Uuid>,

    /// Reference to related journal entries.
    #[serde(default)]
    pub journal_entry_ids: Vec<Uuid>,
}

impl CustomerContract {
    /// Create a new customer contract.
    pub fn new(
        customer_id: impl Into<String>,
        customer_name: impl Into<String>,
        company_code: impl Into<String>,
        inception_date: NaiveDate,
        transaction_price: Decimal,
        currency: impl Into<String>,
        framework: AccountingFramework,
    ) -> Self {
        Self {
            contract_id: Uuid::now_v7(),
            customer_id: customer_id.into(),
            customer_name: customer_name.into(),
            company_code: company_code.into(),
            inception_date,
            end_date: None,
            transaction_price,
            currency: currency.into(),
            status: ContractStatus::Active,
            performance_obligations: Vec::new(),
            variable_consideration: Vec::new(),
            has_significant_financing: false,
            financing_rate: None,
            framework,
            modifications: Vec::new(),
            sales_order_id: None,
            journal_entry_ids: Vec::new(),
        }
    }

    /// Add a performance obligation to the contract.
    pub fn add_performance_obligation(&mut self, obligation: PerformanceObligation) {
        self.performance_obligations.push(obligation);
    }

    /// Add variable consideration component.
    pub fn add_variable_consideration(&mut self, vc: VariableConsideration) {
        self.variable_consideration.push(vc);
    }

    /// Calculate total allocated transaction price across all obligations.
    pub fn total_allocated_price(&self) -> Decimal {
        self.performance_obligations
            .iter()
            .map(|po| po.allocated_price)
            .sum()
    }

    /// Calculate total revenue recognized to date.
    pub fn total_revenue_recognized(&self) -> Decimal {
        self.performance_obligations
            .iter()
            .map(|po| po.revenue_recognized)
            .sum()
    }

    /// Calculate total deferred revenue (contract liability).
    pub fn total_deferred_revenue(&self) -> Decimal {
        self.performance_obligations
            .iter()
            .map(|po| po.deferred_revenue)
            .sum()
    }

    /// Check if contract is fully satisfied.
    pub fn is_fully_satisfied(&self) -> bool {
        self.performance_obligations
            .iter()
            .all(|po| po.is_satisfied())
    }
}

/// Contract status for lifecycle tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ContractStatus {
    /// Contract is pending approval or execution.
    Pending,
    /// Contract is active and obligations are being performed.
    #[default]
    Active,
    /// Contract has been modified (superseded by new contract).
    Modified,
    /// Contract is complete - all obligations satisfied.
    Complete,
    /// Contract has been terminated.
    Terminated,
    /// Contract is in dispute.
    Disputed,
}

impl std::fmt::Display for ContractStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Active => write!(f, "Active"),
            Self::Modified => write!(f, "Modified"),
            Self::Complete => write!(f, "Complete"),
            Self::Terminated => write!(f, "Terminated"),
            Self::Disputed => write!(f, "Disputed"),
        }
    }
}

/// Performance obligation within a customer contract.
///
/// Represents Step 2 of the revenue recognition model: identifying
/// distinct goods or services promised in the contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceObligation {
    /// Unique obligation identifier.
    pub obligation_id: Uuid,

    /// Parent contract ID.
    pub contract_id: Uuid,

    /// Sequence number within contract.
    pub sequence: u32,

    /// Description of the promised good or service.
    pub description: String,

    /// Type of obligation.
    pub obligation_type: ObligationType,

    /// Pattern of satisfaction.
    pub satisfaction_pattern: SatisfactionPattern,

    /// Method for measuring progress (for over-time recognition).
    pub progress_method: Option<ProgressMethod>,

    /// Standalone selling price for allocation.
    #[serde(with = "rust_decimal::serde::str")]
    pub standalone_selling_price: Decimal,

    /// Allocated transaction price (Step 4).
    #[serde(with = "rust_decimal::serde::str")]
    pub allocated_price: Decimal,

    /// Percentage complete (0-100).
    #[serde(with = "rust_decimal::serde::str")]
    pub progress_percent: Decimal,

    /// Revenue recognized to date.
    #[serde(with = "rust_decimal::serde::str")]
    pub revenue_recognized: Decimal,

    /// Deferred revenue (contract liability).
    #[serde(with = "rust_decimal::serde::str")]
    pub deferred_revenue: Decimal,

    /// Unbilled receivable (contract asset).
    #[serde(with = "rust_decimal::serde::str")]
    pub contract_asset: Decimal,

    /// Date obligation was satisfied (if complete).
    pub satisfaction_date: Option<NaiveDate>,

    /// Expected satisfaction date.
    pub expected_satisfaction_date: Option<NaiveDate>,

    /// Material right granted to customer.
    pub material_right: Option<MaterialRight>,
}

impl PerformanceObligation {
    /// Create a new performance obligation.
    pub fn new(
        contract_id: Uuid,
        sequence: u32,
        description: impl Into<String>,
        obligation_type: ObligationType,
        satisfaction_pattern: SatisfactionPattern,
        standalone_selling_price: Decimal,
    ) -> Self {
        Self {
            obligation_id: Uuid::now_v7(),
            contract_id,
            sequence,
            description: description.into(),
            obligation_type,
            satisfaction_pattern,
            progress_method: match satisfaction_pattern {
                SatisfactionPattern::OverTime => Some(ProgressMethod::default()),
                SatisfactionPattern::PointInTime => None,
            },
            standalone_selling_price,
            allocated_price: Decimal::ZERO,
            progress_percent: Decimal::ZERO,
            revenue_recognized: Decimal::ZERO,
            deferred_revenue: Decimal::ZERO,
            contract_asset: Decimal::ZERO,
            satisfaction_date: None,
            expected_satisfaction_date: None,
            material_right: None,
        }
    }

    /// Check if obligation is fully satisfied.
    pub fn is_satisfied(&self) -> bool {
        self.satisfaction_date.is_some() || self.progress_percent >= Decimal::from(100)
    }

    /// Update progress and calculate revenue to recognize.
    pub fn update_progress(&mut self, new_progress: Decimal, as_of_date: NaiveDate) {
        let old_revenue = self.revenue_recognized;
        self.progress_percent = new_progress.min(Decimal::from(100));

        // Calculate revenue based on progress
        let target_revenue = self.allocated_price * self.progress_percent / Decimal::from(100);
        self.revenue_recognized = target_revenue;
        self.deferred_revenue = self.allocated_price - self.revenue_recognized;

        // Mark as satisfied if 100% complete
        if self.progress_percent >= Decimal::from(100) && self.satisfaction_date.is_none() {
            self.satisfaction_date = Some(as_of_date);
        }

        // Contract asset exists when revenue recognized exceeds billing
        // This would need billing information to calculate accurately
        let _ = old_revenue; // Used for incremental calculations
    }
}

/// Type of performance obligation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ObligationType {
    /// Physical product delivery.
    #[default]
    Good,
    /// Service performance.
    Service,
    /// License grant (functional or symbolic).
    License,
    /// Series of distinct goods/services that are substantially the same.
    Series,
    /// Warranty beyond assurance-type.
    ServiceTypeWarranty,
    /// Option that provides material right.
    MaterialRight,
}

impl std::fmt::Display for ObligationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Good => write!(f, "Good"),
            Self::Service => write!(f, "Service"),
            Self::License => write!(f, "License"),
            Self::Series => write!(f, "Series"),
            Self::ServiceTypeWarranty => write!(f, "Service-Type Warranty"),
            Self::MaterialRight => write!(f, "Material Right"),
        }
    }
}

/// Pattern for satisfying performance obligations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SatisfactionPattern {
    /// Revenue recognized at a point in time (e.g., delivery).
    #[default]
    PointInTime,
    /// Revenue recognized over time as performance occurs.
    OverTime,
}

impl std::fmt::Display for SatisfactionPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PointInTime => write!(f, "Point in Time"),
            Self::OverTime => write!(f, "Over Time"),
        }
    }
}

/// Method for measuring progress toward completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProgressMethod {
    /// Output methods (units produced, milestones, surveys).
    #[default]
    Output,
    /// Input methods (costs incurred, resources consumed, time elapsed).
    Input,
    /// Straight-line method (for series with similar effort).
    StraightLine,
}

impl std::fmt::Display for ProgressMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Output => write!(f, "Output Method"),
            Self::Input => write!(f, "Input Method"),
            Self::StraightLine => write!(f, "Straight-Line"),
        }
    }
}

/// Variable consideration component.
///
/// Represents amounts that can vary based on future events (discounts,
/// rebates, refunds, incentives, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableConsideration {
    /// Unique identifier.
    pub vc_id: Uuid,

    /// Parent contract ID.
    pub contract_id: Uuid,

    /// Type of variable consideration.
    pub vc_type: VariableConsiderationType,

    /// Estimated amount (expected value or most likely amount).
    #[serde(with = "rust_decimal::serde::str")]
    pub estimated_amount: Decimal,

    /// Constrained amount included in transaction price.
    #[serde(with = "rust_decimal::serde::str")]
    pub constrained_amount: Decimal,

    /// Estimation method used.
    pub estimation_method: EstimationMethod,

    /// Probability that estimate is reliable.
    #[serde(with = "rust_decimal::serde::str")]
    pub probability: Decimal,

    /// Description of the variable component.
    pub description: String,

    /// Resolution date (when uncertainty is resolved).
    pub resolution_date: Option<NaiveDate>,

    /// Actual amount (after resolution).
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub actual_amount: Option<Decimal>,
}

impl VariableConsideration {
    /// Create a new variable consideration component.
    pub fn new(
        contract_id: Uuid,
        vc_type: VariableConsiderationType,
        estimated_amount: Decimal,
        description: impl Into<String>,
    ) -> Self {
        Self {
            vc_id: Uuid::now_v7(),
            contract_id,
            vc_type,
            estimated_amount,
            constrained_amount: estimated_amount,
            estimation_method: EstimationMethod::ExpectedValue,
            probability: Decimal::from(80),
            description: description.into(),
            resolution_date: None,
            actual_amount: None,
        }
    }

    /// Apply constraint to prevent significant revenue reversal.
    pub fn apply_constraint(&mut self, constraint_threshold: Decimal) {
        // Constrain to amount highly probable not to result in reversal
        self.constrained_amount = self.estimated_amount * constraint_threshold;
    }
}

/// Type of variable consideration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VariableConsiderationType {
    /// Volume or trade discount.
    Discount,
    /// Rebate based on volume or other criteria.
    Rebate,
    /// Right of return (reduces transaction price).
    RightOfReturn,
    /// Performance bonus or incentive.
    IncentiveBonus,
    /// Penalty for non-performance.
    Penalty,
    /// Price concession.
    PriceConcession,
    /// Royalty based on sales or usage.
    Royalty,
    /// Contingent payment.
    ContingentPayment,
}

impl std::fmt::Display for VariableConsiderationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Discount => write!(f, "Discount"),
            Self::Rebate => write!(f, "Rebate"),
            Self::RightOfReturn => write!(f, "Right of Return"),
            Self::IncentiveBonus => write!(f, "Incentive Bonus"),
            Self::Penalty => write!(f, "Penalty"),
            Self::PriceConcession => write!(f, "Price Concession"),
            Self::Royalty => write!(f, "Royalty"),
            Self::ContingentPayment => write!(f, "Contingent Payment"),
        }
    }
}

/// Method for estimating variable consideration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EstimationMethod {
    /// Expected value (probability-weighted average).
    #[default]
    ExpectedValue,
    /// Most likely amount (single most likely outcome).
    MostLikelyAmount,
}

/// Material right granted to customer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialRight {
    /// Type of material right.
    pub right_type: MaterialRightType,

    /// Standalone selling price of the right.
    #[serde(with = "rust_decimal::serde::str")]
    pub standalone_selling_price: Decimal,

    /// Exercise probability.
    #[serde(with = "rust_decimal::serde::str")]
    pub exercise_probability: Decimal,

    /// Expiration date.
    pub expiration_date: Option<NaiveDate>,
}

/// Type of material right.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialRightType {
    /// Option to renew at discount.
    RenewalOption,
    /// Loyalty points or customer rewards.
    LoyaltyPoints,
    /// Free or discounted future products/services.
    FutureDiscount,
}

/// Contract modification record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractModification {
    /// Modification identifier.
    pub modification_id: Uuid,

    /// Date of modification.
    pub modification_date: NaiveDate,

    /// Type of modification treatment.
    pub treatment: ModificationTreatment,

    /// Change in transaction price.
    #[serde(with = "rust_decimal::serde::str")]
    pub price_change: Decimal,

    /// Description of modification.
    pub description: String,
}

/// Treatment for contract modifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModificationTreatment {
    /// Treat as separate contract.
    SeparateContract,
    /// Terminate existing and create new contract.
    TerminateAndCreate,
    /// Cumulative catch-up adjustment.
    CumulativeCatchUp,
    /// Prospective adjustment.
    Prospective,
}

/// Revenue recognition schedule entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueRecognitionEntry {
    /// Parent contract ID.
    pub contract_id: Uuid,

    /// Performance obligation ID.
    pub obligation_id: Uuid,

    /// Recognition period (month end date).
    pub period_date: NaiveDate,

    /// Revenue recognized in this period.
    #[serde(with = "rust_decimal::serde::str")]
    pub revenue_amount: Decimal,

    /// Cumulative revenue recognized.
    #[serde(with = "rust_decimal::serde::str")]
    pub cumulative_revenue: Decimal,

    /// Deferred revenue balance.
    #[serde(with = "rust_decimal::serde::str")]
    pub deferred_revenue_balance: Decimal,

    /// Contract asset balance.
    #[serde(with = "rust_decimal::serde::str")]
    pub contract_asset_balance: Decimal,

    /// Progress percentage at period end.
    #[serde(with = "rust_decimal::serde::str")]
    pub progress_percent: Decimal,

    /// Journal entry reference.
    pub journal_entry_id: Option<Uuid>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_contract_creation() {
        let contract = CustomerContract::new(
            "CUST001",
            "Acme Corp",
            "1000",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            dec!(100000),
            "USD",
            AccountingFramework::UsGaap,
        );

        assert_eq!(contract.customer_id, "CUST001");
        assert_eq!(contract.transaction_price, dec!(100000));
        assert_eq!(contract.status, ContractStatus::Active);
        assert!(contract.performance_obligations.is_empty());
    }

    #[test]
    fn test_performance_obligation() {
        let contract_id = Uuid::now_v7();
        let mut po = PerformanceObligation::new(
            contract_id,
            1,
            "Software License",
            ObligationType::License,
            SatisfactionPattern::PointInTime,
            dec!(50000),
        );

        po.allocated_price = dec!(50000);
        po.update_progress(dec!(100), NaiveDate::from_ymd_opt(2024, 3, 31).unwrap());

        assert!(po.is_satisfied());
        assert_eq!(po.revenue_recognized, dec!(50000));
        assert_eq!(po.deferred_revenue, dec!(0));
    }

    #[test]
    fn test_over_time_recognition() {
        let contract_id = Uuid::now_v7();
        let mut po = PerformanceObligation::new(
            contract_id,
            1,
            "Consulting Services",
            ObligationType::Service,
            SatisfactionPattern::OverTime,
            dec!(120000),
        );

        po.allocated_price = dec!(120000);

        // 25% complete
        po.update_progress(dec!(25), NaiveDate::from_ymd_opt(2024, 1, 31).unwrap());
        assert_eq!(po.revenue_recognized, dec!(30000));
        assert_eq!(po.deferred_revenue, dec!(90000));

        // 50% complete
        po.update_progress(dec!(50), NaiveDate::from_ymd_opt(2024, 2, 29).unwrap());
        assert_eq!(po.revenue_recognized, dec!(60000));
        assert_eq!(po.deferred_revenue, dec!(60000));
    }

    #[test]
    fn test_variable_consideration_constraint() {
        let contract_id = Uuid::now_v7();
        let mut vc = VariableConsideration::new(
            contract_id,
            VariableConsiderationType::Rebate,
            dec!(10000),
            "Volume rebate",
        );

        vc.apply_constraint(dec!(0.80));
        assert_eq!(vc.constrained_amount, dec!(8000));
    }
}
