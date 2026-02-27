//! Balance relationship rules and coherence validation.
//!
//! Defines rules for validating relationships between balance sheet and
//! income statement items (DSO, DPO, gross margin, etc.).

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use super::account_balance::BalanceSnapshot;

/// Balance relationship rule types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    /// Days Sales Outstanding (AR / Revenue * 365).
    DaysSalesOutstanding,
    /// Days Payable Outstanding (AP / COGS * 365).
    DaysPayableOutstanding,
    /// Days Inventory Outstanding (Inventory / COGS * 365).
    DaysInventoryOutstanding,
    /// Cash Conversion Cycle (DSO + DIO - DPO).
    CashConversionCycle,
    /// Gross Margin ((Revenue - COGS) / Revenue).
    GrossMargin,
    /// Operating Margin (Operating Income / Revenue).
    OperatingMargin,
    /// Net Margin (Net Income / Revenue).
    NetMargin,
    /// Current Ratio (Current Assets / Current Liabilities).
    CurrentRatio,
    /// Quick Ratio ((Current Assets - Inventory) / Current Liabilities).
    QuickRatio,
    /// Debt to Equity (Total Liabilities / Total Equity).
    DebtToEquity,
    /// Interest Coverage (EBIT / Interest Expense).
    InterestCoverage,
    /// Asset Turnover (Revenue / Total Assets).
    AssetTurnover,
    /// Return on Assets (Net Income / Total Assets).
    ReturnOnAssets,
    /// Return on Equity (Net Income / Total Equity).
    ReturnOnEquity,
    /// Depreciation to Fixed Assets (Annual Depreciation / Gross Fixed Assets).
    DepreciationRate,
    /// Balance Sheet Equation (Assets = Liabilities + Equity).
    BalanceSheetEquation,
    /// Retained Earnings Roll-forward.
    RetainedEarningsRollForward,
}

impl RelationshipType {
    /// Get the display name for this relationship type.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::DaysSalesOutstanding => "Days Sales Outstanding",
            Self::DaysPayableOutstanding => "Days Payable Outstanding",
            Self::DaysInventoryOutstanding => "Days Inventory Outstanding",
            Self::CashConversionCycle => "Cash Conversion Cycle",
            Self::GrossMargin => "Gross Margin",
            Self::OperatingMargin => "Operating Margin",
            Self::NetMargin => "Net Margin",
            Self::CurrentRatio => "Current Ratio",
            Self::QuickRatio => "Quick Ratio",
            Self::DebtToEquity => "Debt-to-Equity Ratio",
            Self::InterestCoverage => "Interest Coverage Ratio",
            Self::AssetTurnover => "Asset Turnover",
            Self::ReturnOnAssets => "Return on Assets",
            Self::ReturnOnEquity => "Return on Equity",
            Self::DepreciationRate => "Depreciation Rate",
            Self::BalanceSheetEquation => "Balance Sheet Equation",
            Self::RetainedEarningsRollForward => "Retained Earnings Roll-forward",
        }
    }

    /// Is this a critical validation (must pass)?
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            Self::BalanceSheetEquation | Self::RetainedEarningsRollForward
        )
    }
}

/// A balance relationship rule definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceRelationshipRule {
    /// Rule identifier.
    pub rule_id: String,
    /// Rule name.
    pub name: String,
    /// Relationship type.
    pub relationship_type: RelationshipType,
    /// Target value (if applicable).
    pub target_value: Option<Decimal>,
    /// Minimum acceptable value.
    pub min_value: Option<Decimal>,
    /// Maximum acceptable value.
    pub max_value: Option<Decimal>,
    /// Tolerance for deviation from target.
    pub tolerance: Decimal,
    /// Is this rule enabled?
    pub enabled: bool,
    /// Severity if rule fails.
    pub severity: RuleSeverity,
    /// Account codes for numerator calculation.
    pub numerator_accounts: Vec<String>,
    /// Account codes for denominator calculation.
    pub denominator_accounts: Vec<String>,
    /// Multiplier (e.g., 365 for DSO).
    pub multiplier: Decimal,
}

impl BalanceRelationshipRule {
    /// Create a new DSO rule.
    pub fn new_dso_rule(target_days: u32, tolerance_days: u32) -> Self {
        Self {
            rule_id: "DSO".to_string(),
            name: "Days Sales Outstanding".to_string(),
            relationship_type: RelationshipType::DaysSalesOutstanding,
            target_value: Some(Decimal::from(target_days)),
            min_value: Some(Decimal::from(target_days.saturating_sub(tolerance_days))),
            max_value: Some(Decimal::from(target_days + tolerance_days)),
            tolerance: Decimal::from(tolerance_days),
            enabled: true,
            severity: RuleSeverity::Warning,
            numerator_accounts: vec!["1200".to_string()], // AR accounts
            denominator_accounts: vec!["4100".to_string()], // Revenue accounts
            multiplier: dec!(365),
        }
    }

    /// Create a new DPO rule.
    pub fn new_dpo_rule(target_days: u32, tolerance_days: u32) -> Self {
        Self {
            rule_id: "DPO".to_string(),
            name: "Days Payable Outstanding".to_string(),
            relationship_type: RelationshipType::DaysPayableOutstanding,
            target_value: Some(Decimal::from(target_days)),
            min_value: Some(Decimal::from(target_days.saturating_sub(tolerance_days))),
            max_value: Some(Decimal::from(target_days + tolerance_days)),
            tolerance: Decimal::from(tolerance_days),
            enabled: true,
            severity: RuleSeverity::Warning,
            numerator_accounts: vec!["2100".to_string()], // AP accounts
            denominator_accounts: vec!["5100".to_string()], // COGS accounts
            multiplier: dec!(365),
        }
    }

    /// Create a new gross margin rule.
    pub fn new_gross_margin_rule(target_margin: Decimal, tolerance: Decimal) -> Self {
        Self {
            rule_id: "GROSS_MARGIN".to_string(),
            name: "Gross Margin".to_string(),
            relationship_type: RelationshipType::GrossMargin,
            target_value: Some(target_margin),
            min_value: Some(target_margin - tolerance),
            max_value: Some(target_margin + tolerance),
            tolerance,
            enabled: true,
            severity: RuleSeverity::Warning,
            numerator_accounts: vec!["4100".to_string(), "5100".to_string()], // Revenue - COGS
            denominator_accounts: vec!["4100".to_string()],                   // Revenue
            multiplier: Decimal::ONE,
        }
    }

    /// Create balance sheet equation rule.
    pub fn new_balance_equation_rule() -> Self {
        Self {
            rule_id: "BS_EQUATION".to_string(),
            name: "Balance Sheet Equation".to_string(),
            relationship_type: RelationshipType::BalanceSheetEquation,
            target_value: Some(Decimal::ZERO),
            min_value: Some(dec!(-0.01)),
            max_value: Some(dec!(0.01)),
            tolerance: dec!(0.01),
            enabled: true,
            severity: RuleSeverity::Critical,
            numerator_accounts: Vec::new(),
            denominator_accounts: Vec::new(),
            multiplier: Decimal::ONE,
        }
    }

    /// Check if a calculated value is within acceptable range.
    pub fn is_within_range(&self, value: Decimal) -> bool {
        let within_min = self.min_value.is_none_or(|min| value >= min);
        let within_max = self.max_value.is_none_or(|max| value <= max);
        within_min && within_max
    }

    /// Get the deviation from target.
    pub fn deviation_from_target(&self, value: Decimal) -> Option<Decimal> {
        self.target_value.map(|target| value - target)
    }
}

/// Severity level for rule violations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RuleSeverity {
    /// Informational only.
    Info,
    /// Warning - should be investigated.
    #[default]
    Warning,
    /// Error - significant issue.
    Error,
    /// Critical - must be resolved.
    Critical,
}

/// Result of validating a balance relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Rule that was validated.
    pub rule_id: String,
    /// Rule name.
    pub rule_name: String,
    /// Relationship type.
    pub relationship_type: RelationshipType,
    /// Calculated value.
    pub calculated_value: Decimal,
    /// Target value.
    pub target_value: Option<Decimal>,
    /// Is the value within acceptable range?
    pub is_valid: bool,
    /// Deviation from target.
    pub deviation: Option<Decimal>,
    /// Deviation percentage.
    pub deviation_percent: Option<Decimal>,
    /// Severity.
    pub severity: RuleSeverity,
    /// Validation message.
    pub message: String,
}

impl ValidationResult {
    /// Create a passing result.
    pub fn pass(rule: &BalanceRelationshipRule, calculated_value: Decimal) -> Self {
        let deviation = rule.deviation_from_target(calculated_value);
        let deviation_percent = rule.target_value.and_then(|target| {
            if target != Decimal::ZERO {
                Some((calculated_value - target) / target * dec!(100))
            } else {
                None
            }
        });

        Self {
            rule_id: rule.rule_id.clone(),
            rule_name: rule.name.clone(),
            relationship_type: rule.relationship_type,
            calculated_value,
            target_value: rule.target_value,
            is_valid: true,
            deviation,
            deviation_percent,
            severity: RuleSeverity::Info,
            message: format!(
                "{} = {:.2} (within acceptable range)",
                rule.name, calculated_value
            ),
        }
    }

    /// Create a failing result.
    pub fn fail(
        rule: &BalanceRelationshipRule,
        calculated_value: Decimal,
        message: String,
    ) -> Self {
        let deviation = rule.deviation_from_target(calculated_value);
        let deviation_percent = rule.target_value.and_then(|target| {
            if target != Decimal::ZERO {
                Some((calculated_value - target) / target * dec!(100))
            } else {
                None
            }
        });

        Self {
            rule_id: rule.rule_id.clone(),
            rule_name: rule.name.clone(),
            relationship_type: rule.relationship_type,
            calculated_value,
            target_value: rule.target_value,
            is_valid: false,
            deviation,
            deviation_percent,
            severity: rule.severity,
            message,
        }
    }
}

/// Balance coherence validator.
pub struct BalanceCoherenceValidator {
    /// Rules to validate.
    rules: Vec<BalanceRelationshipRule>,
}

impl BalanceCoherenceValidator {
    /// Create a new validator with default rules.
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Add a rule to the validator.
    pub fn add_rule(&mut self, rule: BalanceRelationshipRule) {
        self.rules.push(rule);
    }

    /// Add standard rules for an industry.
    pub fn add_standard_rules(&mut self, target_dso: u32, target_dpo: u32, target_margin: Decimal) {
        self.rules
            .push(BalanceRelationshipRule::new_dso_rule(target_dso, 10));
        self.rules
            .push(BalanceRelationshipRule::new_dpo_rule(target_dpo, 10));
        self.rules
            .push(BalanceRelationshipRule::new_gross_margin_rule(
                target_margin,
                dec!(0.05),
            ));
        self.rules
            .push(BalanceRelationshipRule::new_balance_equation_rule());
    }

    /// Validate a balance snapshot against all rules.
    pub fn validate_snapshot(&self, snapshot: &BalanceSnapshot) -> Vec<ValidationResult> {
        let mut results = Vec::new();

        for rule in &self.rules {
            if !rule.enabled {
                continue;
            }

            let result = self.validate_rule(rule, snapshot);
            results.push(result);
        }

        results
    }

    /// Validate a single rule against a snapshot.
    fn validate_rule(
        &self,
        rule: &BalanceRelationshipRule,
        snapshot: &BalanceSnapshot,
    ) -> ValidationResult {
        match rule.relationship_type {
            RelationshipType::BalanceSheetEquation => {
                // A = L + E + Net Income
                let equation_diff = snapshot.balance_difference;
                if snapshot.is_balanced {
                    ValidationResult::pass(rule, equation_diff)
                } else {
                    ValidationResult::fail(
                        rule,
                        equation_diff,
                        format!("Balance sheet is out of balance by {:.2}", equation_diff),
                    )
                }
            }
            RelationshipType::CurrentRatio => {
                let current_assets = snapshot.total_assets; // Simplified
                let current_liabilities = snapshot.total_liabilities;

                if current_liabilities == Decimal::ZERO {
                    ValidationResult::fail(
                        rule,
                        Decimal::ZERO,
                        "No current liabilities".to_string(),
                    )
                } else {
                    let ratio = current_assets / current_liabilities;
                    if rule.is_within_range(ratio) {
                        ValidationResult::pass(rule, ratio)
                    } else {
                        ValidationResult::fail(
                            rule,
                            ratio,
                            format!("Current ratio {:.2} is outside acceptable range", ratio),
                        )
                    }
                }
            }
            RelationshipType::DebtToEquity => {
                if snapshot.total_equity == Decimal::ZERO {
                    ValidationResult::fail(rule, Decimal::ZERO, "No equity".to_string())
                } else {
                    let ratio = snapshot.total_liabilities / snapshot.total_equity;
                    if rule.is_within_range(ratio) {
                        ValidationResult::pass(rule, ratio)
                    } else {
                        ValidationResult::fail(
                            rule,
                            ratio,
                            format!(
                                "Debt-to-equity ratio {:.2} is outside acceptable range",
                                ratio
                            ),
                        )
                    }
                }
            }
            RelationshipType::GrossMargin => {
                if snapshot.total_revenue == Decimal::ZERO {
                    ValidationResult::pass(rule, Decimal::ZERO) // No revenue is technically valid
                } else {
                    let gross_profit = snapshot.total_revenue - snapshot.total_expenses; // Simplified
                    let margin = gross_profit / snapshot.total_revenue;
                    if rule.is_within_range(margin) {
                        ValidationResult::pass(rule, margin)
                    } else {
                        ValidationResult::fail(
                            rule,
                            margin,
                            format!(
                                "Gross margin {:.1}% is outside target range",
                                margin * dec!(100)
                            ),
                        )
                    }
                }
            }
            _ => {
                // Default calculation using rule accounts
                let numerator: Decimal = rule
                    .numerator_accounts
                    .iter()
                    .filter_map(|code| snapshot.get_balance(code))
                    .map(|b| b.closing_balance)
                    .sum();

                let denominator: Decimal = rule
                    .denominator_accounts
                    .iter()
                    .filter_map(|code| snapshot.get_balance(code))
                    .map(|b| b.closing_balance)
                    .sum();

                if denominator == Decimal::ZERO {
                    ValidationResult::fail(rule, Decimal::ZERO, "Denominator is zero".to_string())
                } else {
                    let value = numerator / denominator * rule.multiplier;
                    if rule.is_within_range(value) {
                        ValidationResult::pass(rule, value)
                    } else {
                        ValidationResult::fail(
                            rule,
                            value,
                            format!("{} = {:.2} is outside acceptable range", rule.name, value),
                        )
                    }
                }
            }
        }
    }

    /// Get a summary of validation results.
    pub fn summarize_results(results: &[ValidationResult]) -> ValidationSummary {
        let total = results.len();
        let passed = results.iter().filter(|r| r.is_valid).count();
        let failed = total - passed;

        let critical_failures = results
            .iter()
            .filter(|r| !r.is_valid && r.severity == RuleSeverity::Critical)
            .count();

        let error_failures = results
            .iter()
            .filter(|r| !r.is_valid && r.severity == RuleSeverity::Error)
            .count();

        let warning_failures = results
            .iter()
            .filter(|r| !r.is_valid && r.severity == RuleSeverity::Warning)
            .count();

        ValidationSummary {
            total_rules: total,
            passed,
            failed,
            critical_failures,
            error_failures,
            warning_failures,
            is_coherent: critical_failures == 0,
        }
    }
}

impl Default for BalanceCoherenceValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of validation results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// Total number of rules validated.
    pub total_rules: usize,
    /// Rules that passed.
    pub passed: usize,
    /// Rules that failed.
    pub failed: usize,
    /// Critical failures.
    pub critical_failures: usize,
    /// Error failures.
    pub error_failures: usize,
    /// Warning failures.
    pub warning_failures: usize,
    /// Overall coherence (no critical failures).
    pub is_coherent: bool,
}

/// Account groupings for ratio calculations.
#[derive(Debug, Clone, Default)]
pub struct AccountGroups {
    /// Current asset accounts.
    pub current_assets: Vec<String>,
    /// Non-current asset accounts.
    pub non_current_assets: Vec<String>,
    /// Current liability accounts.
    pub current_liabilities: Vec<String>,
    /// Non-current liability accounts.
    pub non_current_liabilities: Vec<String>,
    /// Equity accounts.
    pub equity: Vec<String>,
    /// Revenue accounts.
    pub revenue: Vec<String>,
    /// COGS accounts.
    pub cogs: Vec<String>,
    /// Operating expense accounts.
    pub operating_expenses: Vec<String>,
    /// AR accounts.
    pub accounts_receivable: Vec<String>,
    /// AP accounts.
    pub accounts_payable: Vec<String>,
    /// Inventory accounts.
    pub inventory: Vec<String>,
    /// Fixed asset accounts.
    pub fixed_assets: Vec<String>,
    /// Accumulated depreciation accounts.
    pub accumulated_depreciation: Vec<String>,
}

/// Calculate DSO from balances.
pub fn calculate_dso(ar_balance: Decimal, annual_revenue: Decimal) -> Option<Decimal> {
    if annual_revenue == Decimal::ZERO {
        None
    } else {
        Some(ar_balance / annual_revenue * dec!(365))
    }
}

/// Calculate DPO from balances.
pub fn calculate_dpo(ap_balance: Decimal, annual_cogs: Decimal) -> Option<Decimal> {
    if annual_cogs == Decimal::ZERO {
        None
    } else {
        Some(ap_balance / annual_cogs * dec!(365))
    }
}

/// Calculate DIO from balances.
pub fn calculate_dio(inventory_balance: Decimal, annual_cogs: Decimal) -> Option<Decimal> {
    if annual_cogs == Decimal::ZERO {
        None
    } else {
        Some(inventory_balance / annual_cogs * dec!(365))
    }
}

/// Calculate cash conversion cycle.
pub fn calculate_ccc(dso: Decimal, dio: Decimal, dpo: Decimal) -> Decimal {
    dso + dio - dpo
}

/// Calculate gross margin.
pub fn calculate_gross_margin(revenue: Decimal, cogs: Decimal) -> Option<Decimal> {
    if revenue == Decimal::ZERO {
        None
    } else {
        Some((revenue - cogs) / revenue)
    }
}

/// Calculate operating margin.
pub fn calculate_operating_margin(revenue: Decimal, operating_income: Decimal) -> Option<Decimal> {
    if revenue == Decimal::ZERO {
        None
    } else {
        Some(operating_income / revenue)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_dso_calculation() {
        let ar = dec!(123288); // AR balance
        let revenue = dec!(1000000); // Annual revenue

        let dso = calculate_dso(ar, revenue).unwrap();
        // 123288 / 1000000 * 365 = 45.0
        assert!((dso - dec!(45)).abs() < dec!(1));
    }

    #[test]
    fn test_dpo_calculation() {
        let ap = dec!(58904); // AP balance
        let cogs = dec!(650000); // Annual COGS

        let dpo = calculate_dpo(ap, cogs).unwrap();
        // 58904 / 650000 * 365 ≈ 33.1
        assert!((dpo - dec!(33)).abs() < dec!(2));
    }

    #[test]
    fn test_gross_margin_calculation() {
        let revenue = dec!(1000000);
        let cogs = dec!(650000);

        let margin = calculate_gross_margin(revenue, cogs).unwrap();
        // (1000000 - 650000) / 1000000 = 0.35
        assert_eq!(margin, dec!(0.35));
    }

    #[test]
    fn test_ccc_calculation() {
        let dso = dec!(45);
        let dio = dec!(60);
        let dpo = dec!(30);

        let ccc = calculate_ccc(dso, dio, dpo);
        // 45 + 60 - 30 = 75 days
        assert_eq!(ccc, dec!(75));
    }

    #[test]
    fn test_dso_rule() {
        let rule = BalanceRelationshipRule::new_dso_rule(45, 10);

        assert!(rule.is_within_range(dec!(45)));
        assert!(rule.is_within_range(dec!(35)));
        assert!(rule.is_within_range(dec!(55)));
        assert!(!rule.is_within_range(dec!(30)));
        assert!(!rule.is_within_range(dec!(60)));
    }

    #[test]
    fn test_gross_margin_rule() {
        let rule = BalanceRelationshipRule::new_gross_margin_rule(dec!(0.35), dec!(0.05));

        assert!(rule.is_within_range(dec!(0.35)));
        assert!(rule.is_within_range(dec!(0.30)));
        assert!(rule.is_within_range(dec!(0.40)));
        assert!(!rule.is_within_range(dec!(0.25)));
        assert!(!rule.is_within_range(dec!(0.45)));
    }

    #[test]
    fn test_validation_summary() {
        let rule1 = BalanceRelationshipRule::new_balance_equation_rule();
        let rule2 = BalanceRelationshipRule::new_dso_rule(45, 10);

        let results = vec![
            ValidationResult::pass(&rule1, Decimal::ZERO),
            ValidationResult::fail(&rule2, dec!(60), "DSO too high".to_string()),
        ];

        let summary = BalanceCoherenceValidator::summarize_results(&results);

        assert_eq!(summary.total_rules, 2);
        assert_eq!(summary.passed, 1);
        assert_eq!(summary.failed, 1);
        assert!(summary.is_coherent); // No critical failures
    }
}
