//! Entity-aware anomaly injection.
//!
//! Provides rules for injecting anomalies based on entity characteristics,
//! such as vendor tenure, employee experience, and account types.

use chrono::NaiveDate;
use rand::Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use datasynth_core::models::{AnomalyType, ErrorType, FraudType, ProcessIssueType};

/// Configuration for entity-aware anomaly injection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityAwareConfig {
    /// Enable entity-aware injection.
    pub enabled: bool,
    /// Vendor-specific rules.
    pub vendor_rules: VendorAnomalyRules,
    /// Employee-specific rules.
    pub employee_rules: EmployeeAnomalyRules,
    /// Account-specific rules.
    pub account_rules: AccountAnomalyRules,
}

impl Default for EntityAwareConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            vendor_rules: VendorAnomalyRules::default(),
            employee_rules: EmployeeAnomalyRules::default(),
            account_rules: AccountAnomalyRules::default(),
        }
    }
}

/// Rules for vendor-specific anomaly patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorAnomalyRules {
    /// Threshold for "new" vendor in days.
    pub new_vendor_threshold_days: u32,
    /// Error rate multiplier for new vendors.
    pub new_vendor_error_multiplier: f64,
    /// Anomaly types more common with new vendors.
    pub new_vendor_error_types: Vec<AnomalyType>,
    /// Threshold for "strategic" vendor by total spend.
    pub strategic_vendor_spend_threshold: Decimal,
    /// Anomaly types for strategic vendors (typically fraud).
    pub strategic_vendor_types: Vec<AnomalyType>,
    /// International vendor FX/tax error types.
    pub international_error_types: Vec<AnomalyType>,
    /// Dormant vendor threshold in days.
    pub dormant_vendor_threshold_days: u32,
    /// Error multiplier for dormant vendor reactivation.
    pub dormant_reactivation_multiplier: f64,
}

impl Default for VendorAnomalyRules {
    fn default() -> Self {
        Self {
            new_vendor_threshold_days: 90,
            new_vendor_error_multiplier: 2.5,
            new_vendor_error_types: vec![
                AnomalyType::Error(ErrorType::MissingField),
                AnomalyType::Error(ErrorType::MisclassifiedAccount),
                AnomalyType::Error(ErrorType::MissingField),
                AnomalyType::ProcessIssue(ProcessIssueType::MissingDocumentation),
            ],
            strategic_vendor_spend_threshold: dec!(1000000),
            strategic_vendor_types: vec![
                AnomalyType::Fraud(FraudType::Kickback),
                AnomalyType::Fraud(FraudType::InvoiceManipulation),
                AnomalyType::Fraud(FraudType::SplitTransaction),
            ],
            international_error_types: vec![
                AnomalyType::Error(ErrorType::CurrencyError),
                AnomalyType::Error(ErrorType::TaxCalculationError),
                AnomalyType::Error(ErrorType::WrongPeriod),
            ],
            dormant_vendor_threshold_days: 180,
            dormant_reactivation_multiplier: 1.8,
        }
    }
}

impl VendorAnomalyRules {
    /// Checks if a vendor is considered "new".
    pub fn is_new_vendor(&self, creation_date: NaiveDate, current_date: NaiveDate) -> bool {
        let days = (current_date - creation_date).num_days();
        days >= 0 && days < self.new_vendor_threshold_days as i64
    }

    /// Checks if a vendor is "dormant" (no activity for threshold period).
    pub fn is_dormant_vendor(&self, last_activity: NaiveDate, current_date: NaiveDate) -> bool {
        let days = (current_date - last_activity).num_days();
        days >= self.dormant_vendor_threshold_days as i64
    }

    /// Checks if a vendor is "strategic" based on total spend.
    pub fn is_strategic_vendor(&self, total_spend: Decimal) -> bool {
        total_spend >= self.strategic_vendor_spend_threshold
    }

    /// Gets the error rate multiplier for a vendor.
    pub fn get_multiplier(&self, context: &VendorContext) -> f64 {
        let mut multiplier = 1.0;

        if context.is_new {
            multiplier *= self.new_vendor_error_multiplier;
        }

        if context.is_dormant_reactivation {
            multiplier *= self.dormant_reactivation_multiplier;
        }

        multiplier
    }

    /// Gets applicable anomaly types for a vendor context.
    pub fn get_applicable_types(&self, context: &VendorContext) -> Vec<AnomalyType> {
        let mut types = Vec::new();

        if context.is_new {
            types.extend(self.new_vendor_error_types.clone());
        }

        if context.is_strategic {
            types.extend(self.strategic_vendor_types.clone());
        }

        if context.is_international {
            types.extend(self.international_error_types.clone());
        }

        types
    }
}

/// Context information about a vendor.
#[derive(Debug, Clone, Default)]
pub struct VendorContext {
    /// Vendor ID.
    pub vendor_id: String,
    /// Whether vendor is new (< threshold days).
    pub is_new: bool,
    /// Whether vendor is strategic (high spend).
    pub is_strategic: bool,
    /// Whether vendor is international.
    pub is_international: bool,
    /// Whether this is a dormant vendor reactivation.
    pub is_dormant_reactivation: bool,
    /// Total spend with this vendor.
    pub total_spend: Decimal,
    /// Days since vendor creation.
    pub days_since_creation: i64,
    /// Days since last activity.
    pub days_since_last_activity: i64,
}

/// Rules for employee-specific anomaly patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeAnomalyRules {
    /// Threshold for "new" employee in days.
    pub new_employee_threshold_days: u32,
    /// Error rate for new employees.
    pub new_employee_error_rate: f64,
    /// Transaction volume threshold for fatigue.
    pub volume_fatigue_threshold: u32,
    /// Error multiplier when volume exceeds threshold.
    pub volume_fatigue_multiplier: f64,
    /// Error multiplier when covering for absent approver.
    pub coverage_error_multiplier: f64,
    /// Overtime hours threshold for error spike.
    pub overtime_hours_threshold: f64,
    /// Error multiplier during overtime.
    pub overtime_error_multiplier: f64,
    /// Error types more common with new employees.
    pub new_employee_error_types: Vec<AnomalyType>,
    /// Error types from fatigue.
    pub fatigue_error_types: Vec<AnomalyType>,
}

impl Default for EmployeeAnomalyRules {
    fn default() -> Self {
        Self {
            new_employee_threshold_days: 180,
            new_employee_error_rate: 0.05,
            volume_fatigue_threshold: 50,
            volume_fatigue_multiplier: 1.5,
            coverage_error_multiplier: 1.3,
            overtime_hours_threshold: 45.0,
            overtime_error_multiplier: 1.4,
            new_employee_error_types: vec![
                AnomalyType::Error(ErrorType::MisclassifiedAccount),
                AnomalyType::Error(ErrorType::WrongPeriod),
                AnomalyType::Error(ErrorType::MissingField),
                AnomalyType::ProcessIssue(ProcessIssueType::IncompleteApprovalChain),
            ],
            fatigue_error_types: vec![
                AnomalyType::Error(ErrorType::DuplicateEntry),
                AnomalyType::Error(ErrorType::TransposedDigits),
                AnomalyType::Error(ErrorType::ReversedAmount),
                AnomalyType::ProcessIssue(ProcessIssueType::SkippedApproval),
            ],
        }
    }
}

impl EmployeeAnomalyRules {
    /// Checks if an employee is considered "new".
    pub fn is_new_employee(&self, hire_date: NaiveDate, current_date: NaiveDate) -> bool {
        let days = (current_date - hire_date).num_days();
        days >= 0 && days < self.new_employee_threshold_days as i64
    }

    /// Checks if employee is experiencing volume fatigue.
    pub fn is_volume_fatigue(&self, daily_transaction_count: u32) -> bool {
        daily_transaction_count > self.volume_fatigue_threshold
    }

    /// Checks if employee is in overtime.
    pub fn is_overtime(&self, weekly_hours: f64) -> bool {
        weekly_hours > self.overtime_hours_threshold
    }

    /// Gets the error rate multiplier for an employee.
    pub fn get_multiplier(&self, context: &EmployeeContext) -> f64 {
        let mut multiplier = 1.0;

        if context.is_new {
            multiplier *= 1.0 + self.new_employee_error_rate * 10.0;
        }

        if context.is_volume_fatigued {
            multiplier *= self.volume_fatigue_multiplier;
        }

        if context.is_covering {
            multiplier *= self.coverage_error_multiplier;
        }

        if context.is_overtime {
            multiplier *= self.overtime_error_multiplier;
        }

        multiplier
    }

    /// Gets applicable anomaly types for an employee context.
    pub fn get_applicable_types(&self, context: &EmployeeContext) -> Vec<AnomalyType> {
        let mut types = Vec::new();

        if context.is_new {
            types.extend(self.new_employee_error_types.clone());
        }

        if context.is_volume_fatigued || context.is_overtime {
            types.extend(self.fatigue_error_types.clone());
        }

        types
    }
}

/// Context information about an employee.
#[derive(Debug, Clone, Default)]
pub struct EmployeeContext {
    /// Employee ID.
    pub employee_id: String,
    /// Whether employee is new (< threshold days).
    pub is_new: bool,
    /// Whether employee is experiencing volume fatigue.
    pub is_volume_fatigued: bool,
    /// Whether employee is covering for an absent approver.
    pub is_covering: bool,
    /// Whether employee is in overtime.
    pub is_overtime: bool,
    /// Daily transaction count.
    pub daily_transaction_count: u32,
    /// Weekly hours worked.
    pub weekly_hours: f64,
    /// Days since hire.
    pub days_since_hire: i64,
}

/// Rules for account-specific anomaly patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountAnomalyRules {
    /// High-risk account prefixes (e.g., "8" for suspense).
    pub high_risk_prefixes: Vec<String>,
    /// Error multiplier for high-risk accounts.
    pub high_risk_multiplier: f64,
    /// Reconciliation account patterns.
    pub reconciliation_account_patterns: Vec<String>,
    /// Error types for reconciliation accounts.
    pub reconciliation_error_types: Vec<AnomalyType>,
    /// Revenue account patterns.
    pub revenue_account_patterns: Vec<String>,
    /// Fraud types for revenue accounts.
    pub revenue_fraud_types: Vec<AnomalyType>,
    /// Intercompany account patterns.
    pub intercompany_account_patterns: Vec<String>,
    /// Error types for intercompany accounts.
    pub intercompany_error_types: Vec<AnomalyType>,
}

impl Default for AccountAnomalyRules {
    fn default() -> Self {
        Self {
            high_risk_prefixes: vec!["8".to_string(), "9".to_string()],
            high_risk_multiplier: 2.0,
            reconciliation_account_patterns: vec![
                "1290".to_string(), // Clearing accounts
                "2990".to_string(), // Suspense accounts
            ],
            reconciliation_error_types: vec![
                AnomalyType::Error(ErrorType::UnbalancedEntry),
                AnomalyType::Error(ErrorType::MisclassifiedAccount),
            ],
            revenue_account_patterns: vec![
                "4".to_string(), // Revenue accounts typically start with 4
            ],
            revenue_fraud_types: vec![
                AnomalyType::Fraud(FraudType::RevenueManipulation),
                AnomalyType::Fraud(FraudType::PrematureRevenue),
                AnomalyType::Fraud(FraudType::ChannelStuffing),
            ],
            intercompany_account_patterns: vec![
                "1310".to_string(), // IC receivables
                "2310".to_string(), // IC payables
            ],
            intercompany_error_types: vec![
                AnomalyType::Error(ErrorType::WrongCompanyCode),
                AnomalyType::Error(ErrorType::WrongPeriod),
            ],
        }
    }
}

impl AccountAnomalyRules {
    /// Checks if an account is high-risk.
    pub fn is_high_risk(&self, account_code: &str) -> bool {
        self.high_risk_prefixes
            .iter()
            .any(|prefix| account_code.starts_with(prefix))
    }

    /// Checks if an account is a reconciliation account.
    pub fn is_reconciliation_account(&self, account_code: &str) -> bool {
        self.reconciliation_account_patterns
            .iter()
            .any(|pattern| account_code.starts_with(pattern))
    }

    /// Checks if an account is a revenue account.
    pub fn is_revenue_account(&self, account_code: &str) -> bool {
        self.revenue_account_patterns
            .iter()
            .any(|pattern| account_code.starts_with(pattern))
    }

    /// Checks if an account is an intercompany account.
    pub fn is_intercompany_account(&self, account_code: &str) -> bool {
        self.intercompany_account_patterns
            .iter()
            .any(|pattern| account_code.starts_with(pattern))
    }

    /// Gets the error rate multiplier for an account.
    pub fn get_multiplier(&self, context: &AccountContext) -> f64 {
        let mut multiplier = 1.0;

        if context.is_high_risk {
            multiplier *= self.high_risk_multiplier;
        }

        multiplier
    }

    /// Gets applicable anomaly types for an account context.
    pub fn get_applicable_types(&self, context: &AccountContext) -> Vec<AnomalyType> {
        let mut types = Vec::new();

        if context.is_reconciliation {
            types.extend(self.reconciliation_error_types.clone());
        }

        if context.is_revenue {
            types.extend(self.revenue_fraud_types.clone());
        }

        if context.is_intercompany {
            types.extend(self.intercompany_error_types.clone());
        }

        types
    }
}

/// Context information about an account.
#[derive(Debug, Clone, Default)]
pub struct AccountContext {
    /// Account code.
    pub account_code: String,
    /// Whether account is high-risk.
    pub is_high_risk: bool,
    /// Whether account is a reconciliation account.
    pub is_reconciliation: bool,
    /// Whether account is a revenue account.
    pub is_revenue: bool,
    /// Whether account is an intercompany account.
    pub is_intercompany: bool,
}

/// Entity-aware anomaly injector.
pub struct EntityAwareInjector {
    config: EntityAwareConfig,
}

impl Default for EntityAwareInjector {
    fn default() -> Self {
        Self::new(EntityAwareConfig::default())
    }
}

impl EntityAwareInjector {
    /// Creates a new entity-aware injector.
    pub fn new(config: EntityAwareConfig) -> Self {
        Self { config }
    }

    /// Gets the combined rate multiplier for a transaction context.
    pub fn get_rate_multiplier(
        &self,
        vendor_ctx: Option<&VendorContext>,
        employee_ctx: Option<&EmployeeContext>,
        account_ctx: Option<&AccountContext>,
    ) -> f64 {
        if !self.config.enabled {
            return 1.0;
        }

        let mut multiplier = 1.0;

        if let Some(ctx) = vendor_ctx {
            multiplier *= self.config.vendor_rules.get_multiplier(ctx);
        }

        if let Some(ctx) = employee_ctx {
            multiplier *= self.config.employee_rules.get_multiplier(ctx);
        }

        if let Some(ctx) = account_ctx {
            multiplier *= self.config.account_rules.get_multiplier(ctx);
        }

        multiplier
    }

    /// Gets applicable anomaly types for a transaction context.
    pub fn get_applicable_types(
        &self,
        vendor_ctx: Option<&VendorContext>,
        employee_ctx: Option<&EmployeeContext>,
        account_ctx: Option<&AccountContext>,
    ) -> Vec<AnomalyType> {
        if !self.config.enabled {
            return Vec::new();
        }

        let mut types = Vec::new();

        if let Some(ctx) = vendor_ctx {
            types.extend(self.config.vendor_rules.get_applicable_types(ctx));
        }

        if let Some(ctx) = employee_ctx {
            types.extend(self.config.employee_rules.get_applicable_types(ctx));
        }

        if let Some(ctx) = account_ctx {
            types.extend(self.config.account_rules.get_applicable_types(ctx));
        }

        // Remove duplicates
        types.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
        types.dedup();

        types
    }

    /// Determines if an anomaly should be injected based on context.
    pub fn should_inject<R: Rng>(
        &self,
        base_rate: f64,
        vendor_ctx: Option<&VendorContext>,
        employee_ctx: Option<&EmployeeContext>,
        account_ctx: Option<&AccountContext>,
        rng: &mut R,
    ) -> bool {
        let multiplier = self.get_rate_multiplier(vendor_ctx, employee_ctx, account_ctx);
        let adjusted_rate = (base_rate * multiplier).min(1.0);
        rng.random::<f64>() < adjusted_rate
    }

    /// Builds a vendor context from entity data.
    pub fn build_vendor_context(
        &self,
        vendor_id: impl Into<String>,
        creation_date: NaiveDate,
        last_activity: NaiveDate,
        current_date: NaiveDate,
        total_spend: Decimal,
        is_international: bool,
    ) -> VendorContext {
        let is_new = self
            .config
            .vendor_rules
            .is_new_vendor(creation_date, current_date);
        let is_dormant_reactivation = self
            .config
            .vendor_rules
            .is_dormant_vendor(last_activity, current_date);
        let is_strategic = self.config.vendor_rules.is_strategic_vendor(total_spend);

        VendorContext {
            vendor_id: vendor_id.into(),
            is_new,
            is_strategic,
            is_international,
            is_dormant_reactivation,
            total_spend,
            days_since_creation: (current_date - creation_date).num_days(),
            days_since_last_activity: (current_date - last_activity).num_days(),
        }
    }

    /// Builds an employee context from entity data.
    pub fn build_employee_context(
        &self,
        employee_id: impl Into<String>,
        hire_date: NaiveDate,
        current_date: NaiveDate,
        daily_transaction_count: u32,
        weekly_hours: f64,
        is_covering: bool,
    ) -> EmployeeContext {
        let is_new = self
            .config
            .employee_rules
            .is_new_employee(hire_date, current_date);
        let is_volume_fatigued = self
            .config
            .employee_rules
            .is_volume_fatigue(daily_transaction_count);
        let is_overtime = self.config.employee_rules.is_overtime(weekly_hours);

        EmployeeContext {
            employee_id: employee_id.into(),
            is_new,
            is_volume_fatigued,
            is_covering,
            is_overtime,
            daily_transaction_count,
            weekly_hours,
            days_since_hire: (current_date - hire_date).num_days(),
        }
    }

    /// Builds an account context from account code.
    pub fn build_account_context(&self, account_code: impl Into<String>) -> AccountContext {
        let code = account_code.into();
        let is_high_risk = self.config.account_rules.is_high_risk(&code);
        let is_reconciliation = self.config.account_rules.is_reconciliation_account(&code);
        let is_revenue = self.config.account_rules.is_revenue_account(&code);
        let is_intercompany = self.config.account_rules.is_intercompany_account(&code);

        AccountContext {
            account_code: code,
            is_high_risk,
            is_reconciliation,
            is_revenue,
            is_intercompany,
        }
    }

    /// Returns the configuration.
    pub fn config(&self) -> &EntityAwareConfig {
        &self.config
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_vendor_rules_new_vendor() {
        let rules = VendorAnomalyRules::default();
        let creation = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let current = NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(); // 31 days

        assert!(rules.is_new_vendor(creation, current));

        let current_later = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(); // 152 days
        assert!(!rules.is_new_vendor(creation, current_later));
    }

    #[test]
    fn test_vendor_rules_strategic() {
        let rules = VendorAnomalyRules::default();

        assert!(!rules.is_strategic_vendor(dec!(500000)));
        assert!(rules.is_strategic_vendor(dec!(1500000)));
    }

    #[test]
    fn test_employee_rules_new_employee() {
        let rules = EmployeeAnomalyRules::default();
        let hire = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let current = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(); // 60 days

        assert!(rules.is_new_employee(hire, current));

        let current_later = NaiveDate::from_ymd_opt(2024, 9, 1).unwrap(); // 244 days
        assert!(!rules.is_new_employee(hire, current_later));
    }

    #[test]
    fn test_employee_rules_fatigue() {
        let rules = EmployeeAnomalyRules::default();

        assert!(!rules.is_volume_fatigue(30));
        assert!(rules.is_volume_fatigue(60));
    }

    #[test]
    fn test_account_rules() {
        let rules = AccountAnomalyRules::default();

        assert!(rules.is_high_risk("8100"));
        assert!(rules.is_high_risk("9000"));
        assert!(!rules.is_high_risk("4100"));

        assert!(rules.is_revenue_account("4100"));
        assert!(!rules.is_revenue_account("5100"));

        assert!(rules.is_intercompany_account("1310"));
        assert!(rules.is_intercompany_account("2310"));
    }

    #[test]
    fn test_entity_aware_injector() {
        let injector = EntityAwareInjector::default();

        let vendor_ctx = VendorContext {
            vendor_id: "V001".to_string(),
            is_new: true,
            is_strategic: false,
            is_international: false,
            is_dormant_reactivation: false,
            total_spend: dec!(50000),
            days_since_creation: 30,
            days_since_last_activity: 5,
        };

        let multiplier = injector.get_rate_multiplier(Some(&vendor_ctx), None, None);
        assert!(multiplier > 1.0); // New vendor should increase rate
    }

    #[test]
    fn test_build_vendor_context() {
        let injector = EntityAwareInjector::default();

        let ctx = injector.build_vendor_context(
            "V001",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 5, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            dec!(2000000),
            true,
        );

        assert!(!ctx.is_new); // 152 days > 90 threshold
        assert!(ctx.is_strategic); // 2M > 1M threshold
        assert!(ctx.is_international);
        assert!(!ctx.is_dormant_reactivation); // 31 days < 180 threshold
    }

    #[test]
    fn test_combined_multiplier() {
        let injector = EntityAwareInjector::default();

        let vendor_ctx = VendorContext {
            is_new: true,
            ..Default::default()
        };

        let employee_ctx = EmployeeContext {
            is_volume_fatigued: true,
            ..Default::default()
        };

        let multiplier = injector.get_rate_multiplier(Some(&vendor_ctx), Some(&employee_ctx), None);

        // Should be new_vendor_multiplier * volume_fatigue_multiplier
        // 2.5 * 1.5 = 3.75
        assert!((multiplier - 3.75).abs() < 0.01);
    }
}
