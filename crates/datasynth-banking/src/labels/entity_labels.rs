//! Entity-level label generation.

use datasynth_core::models::banking::{RiskTier, SourceOfFunds, TurnoverBand};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{BankAccount, BankingCustomer};

/// Customer-level labels for ML training.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerLabel {
    /// Customer ID
    pub customer_id: Uuid,
    /// Risk tier
    pub risk_tier: RiskTier,
    /// Expected monthly turnover band
    pub expected_turnover: TurnoverBand,
    /// Beneficial owner complexity score (1-10)
    pub bo_complexity: u8,
    /// Is known money mule?
    pub is_mule: bool,
    /// True vs declared source of funds match?
    pub sof_truthful: bool,
    /// True source of funds (if different from declared)
    pub true_sof: Option<SourceOfFunds>,
    /// Declared source of funds
    pub declared_sof: SourceOfFunds,
    /// KYC completeness score (0.0-1.0)
    pub kyc_completeness: f64,
    /// Customer type risk weight
    pub type_risk_weight: f64,
    /// Associated case IDs
    pub case_ids: Vec<String>,
    /// Confidence score for the label
    pub confidence: f64,
}

impl CustomerLabel {
    /// Create a new customer label from a customer.
    pub fn from_customer(customer: &BankingCustomer) -> Self {
        Self {
            customer_id: customer.customer_id,
            risk_tier: customer.risk_tier,
            expected_turnover: customer.kyc_profile.expected_monthly_turnover,
            bo_complexity: customer.kyc_profile.beneficial_owner_complexity,
            is_mule: customer.is_mule,
            sof_truthful: customer.kyc_truthful,
            true_sof: customer.kyc_profile.true_source_of_funds,
            declared_sof: customer.kyc_profile.source_of_funds,
            kyc_completeness: customer.kyc_profile.completeness_score,
            type_risk_weight: Self::customer_type_risk_weight(&customer.customer_type),
            case_ids: Vec::new(),
            confidence: 1.0,
        }
    }

    /// Get risk weight for customer type.
    fn customer_type_risk_weight(
        customer_type: &datasynth_core::models::banking::BankingCustomerType,
    ) -> f64 {
        use datasynth_core::models::banking::BankingCustomerType;
        match customer_type {
            BankingCustomerType::Retail => 1.0,
            BankingCustomerType::Business => 1.2,
            BankingCustomerType::Trust => 1.5,
            BankingCustomerType::FinancialInstitution => 1.8,
            BankingCustomerType::Government => 0.8,
            BankingCustomerType::NonProfit => 1.0,
        }
    }

    /// Add case ID to customer label.
    pub fn with_case(mut self, case_id: &str) -> Self {
        self.case_ids.push(case_id.to_string());
        self
    }
}

/// Account-level labels for ML training.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountLabel {
    /// Account ID
    pub account_id: Uuid,
    /// Owner customer ID
    pub customer_id: Uuid,
    /// Is funnel account?
    pub is_funnel: bool,
    /// Is mule account?
    pub is_mule_account: bool,
    /// Account risk weight
    pub risk_weight: f64,
    /// Expected transaction count per month
    pub expected_tx_count: u32,
    /// Expected average transaction amount
    pub expected_avg_amount: f64,
    /// Associated case ID
    pub case_id: Option<String>,
    /// Account age in days
    pub account_age_days: u32,
    /// Is dormant (no activity in 90+ days)?
    pub is_dormant: bool,
    /// Confidence score
    pub confidence: f64,
}

impl AccountLabel {
    /// Create a new account label from an account.
    pub fn from_account(account: &BankAccount) -> Self {
        let today = chrono::Utc::now().date_naive();
        let age_days = (today - account.opening_date).num_days().max(0) as u32;

        Self {
            account_id: account.account_id,
            customer_id: account.primary_owner_id,
            is_funnel: account.is_funnel_account,
            is_mule_account: account.is_mule_account,
            risk_weight: account.account_type.risk_weight(),
            expected_tx_count: Self::estimate_tx_count(&account.account_type),
            expected_avg_amount: Self::estimate_avg_amount(&account.account_type),
            case_id: account.case_id.clone(),
            account_age_days: age_days,
            is_dormant: account.days_dormant > 90,
            confidence: 1.0,
        }
    }

    /// Estimate expected transaction count.
    fn estimate_tx_count(account_type: &datasynth_core::models::banking::BankAccountType) -> u32 {
        use datasynth_core::models::banking::BankAccountType;

        match account_type {
            BankAccountType::Checking => 30,
            BankAccountType::Savings => 5,
            BankAccountType::MoneyMarket => 3,
            BankAccountType::CertificateOfDeposit => 1,
            BankAccountType::BusinessOperating => 100,
            BankAccountType::BusinessSavings => 10,
            BankAccountType::Payroll => 50,
            BankAccountType::TrustAccount => 5,
            BankAccountType::Escrow => 3,
            BankAccountType::Investment => 10,
            BankAccountType::ForeignCurrency => 20,
        }
    }

    /// Estimate expected average amount.
    fn estimate_avg_amount(account_type: &datasynth_core::models::banking::BankAccountType) -> f64 {
        use datasynth_core::models::banking::BankAccountType;

        match account_type {
            BankAccountType::Checking => 250.0,
            BankAccountType::Savings => 1000.0,
            BankAccountType::MoneyMarket => 5000.0,
            BankAccountType::CertificateOfDeposit => 10000.0,
            BankAccountType::BusinessOperating => 2500.0,
            BankAccountType::BusinessSavings => 10000.0,
            BankAccountType::Payroll => 3500.0,
            BankAccountType::TrustAccount => 50000.0,
            BankAccountType::Escrow => 25000.0,
            BankAccountType::Investment => 15000.0,
            BankAccountType::ForeignCurrency => 5000.0,
        }
    }
}

/// Entity label extractor.
pub struct EntityLabelExtractor;

impl EntityLabelExtractor {
    /// Extract customer labels.
    pub fn extract_customers(customers: &[BankingCustomer]) -> Vec<CustomerLabel> {
        customers.iter().map(CustomerLabel::from_customer).collect()
    }

    /// Extract account labels.
    pub fn extract_accounts(accounts: &[BankAccount]) -> Vec<AccountLabel> {
        accounts.iter().map(AccountLabel::from_account).collect()
    }

    /// Get customer label summary.
    pub fn summarize_customers(labels: &[CustomerLabel]) -> CustomerLabelSummary {
        let total = labels.len();
        let mules = labels.iter().filter(|l| l.is_mule).count();
        let deceptive = labels.iter().filter(|l| !l.sof_truthful).count();

        let mut by_risk_tier = std::collections::HashMap::new();
        for label in labels {
            *by_risk_tier.entry(label.risk_tier).or_insert(0) += 1;
        }

        CustomerLabelSummary {
            total_customers: total,
            mule_count: mules,
            mule_rate: mules as f64 / total as f64,
            deceptive_count: deceptive,
            deceptive_rate: deceptive as f64 / total as f64,
            by_risk_tier,
        }
    }

    /// Get account label summary.
    pub fn summarize_accounts(labels: &[AccountLabel]) -> AccountLabelSummary {
        let total = labels.len();
        let funnel = labels.iter().filter(|l| l.is_funnel).count();
        let mule = labels.iter().filter(|l| l.is_mule_account).count();
        let dormant = labels.iter().filter(|l| l.is_dormant).count();

        AccountLabelSummary {
            total_accounts: total,
            funnel_count: funnel,
            funnel_rate: funnel as f64 / total as f64,
            mule_count: mule,
            mule_rate: mule as f64 / total as f64,
            dormant_count: dormant,
            dormant_rate: dormant as f64 / total as f64,
        }
    }
}

/// Customer label summary.
#[derive(Debug, Clone)]
pub struct CustomerLabelSummary {
    /// Total customers
    pub total_customers: usize,
    /// Number of mules
    pub mule_count: usize,
    /// Mule rate
    pub mule_rate: f64,
    /// Number with deceptive KYC
    pub deceptive_count: usize,
    /// Deceptive rate
    pub deceptive_rate: f64,
    /// Counts by risk tier
    pub by_risk_tier: std::collections::HashMap<RiskTier, usize>,
}

/// Account label summary.
#[derive(Debug, Clone)]
pub struct AccountLabelSummary {
    /// Total accounts
    pub total_accounts: usize,
    /// Number of funnel accounts
    pub funnel_count: usize,
    /// Funnel rate
    pub funnel_rate: f64,
    /// Number of mule accounts
    pub mule_count: usize,
    /// Mule rate
    pub mule_rate: f64,
    /// Number of dormant accounts
    pub dormant_count: usize,
    /// Dormant rate
    pub dormant_rate: f64,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_customer_label() {
        let customer = BankingCustomer::new_retail(
            Uuid::new_v4(),
            "Test",
            "User",
            "US",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        let label = CustomerLabel::from_customer(&customer);

        assert_eq!(label.customer_id, customer.customer_id);
        assert!(!label.is_mule);
        assert!(label.sof_truthful);
    }

    #[test]
    fn test_account_label() {
        let account = BankAccount::new(
            Uuid::new_v4(),
            "****1234".to_string(),
            datasynth_core::models::banking::BankAccountType::Checking,
            Uuid::new_v4(),
            "USD",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        let label = AccountLabel::from_account(&account);

        assert_eq!(label.account_id, account.account_id);
        assert!(!label.is_funnel);
        assert!(!label.is_mule_account);
    }
}
