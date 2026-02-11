//! Banking account type definitions.

use serde::{Deserialize, Serialize};

/// Type of bank account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BankAccountType {
    /// Standard checking/current account
    #[default]
    Checking,
    /// Savings account
    Savings,
    /// Money market account
    MoneyMarket,
    /// Certificate of deposit
    CertificateOfDeposit,
    /// Business operating account
    BusinessOperating,
    /// Business savings account
    BusinessSavings,
    /// Trust account
    TrustAccount,
    /// Investment account
    Investment,
    /// Foreign currency account
    ForeignCurrency,
    /// Escrow account
    Escrow,
    /// Payroll account
    Payroll,
}

impl BankAccountType {
    /// Whether this account type typically has high transaction volume.
    pub fn is_high_volume(&self) -> bool {
        matches!(
            self,
            Self::Checking | Self::BusinessOperating | Self::Payroll
        )
    }

    /// Whether this is a business account type.
    pub fn is_business(&self) -> bool {
        matches!(
            self,
            Self::BusinessOperating
                | Self::BusinessSavings
                | Self::Payroll
                | Self::TrustAccount
                | Self::Escrow
        )
    }

    /// Typical minimum balance requirement.
    pub fn typical_min_balance(&self) -> u32 {
        match self {
            Self::Checking => 0,
            Self::Savings => 100,
            Self::MoneyMarket => 2_500,
            Self::CertificateOfDeposit => 1_000,
            Self::BusinessOperating => 0,
            Self::BusinessSavings => 500,
            Self::TrustAccount => 10_000,
            Self::Investment => 5_000,
            Self::ForeignCurrency => 1_000,
            Self::Escrow => 0,
            Self::Payroll => 0,
        }
    }

    /// Risk weight for AML scoring.
    pub fn risk_weight(&self) -> f64 {
        match self {
            Self::Checking => 1.0,
            Self::Savings => 0.8,
            Self::MoneyMarket => 0.9,
            Self::CertificateOfDeposit => 0.5,
            Self::BusinessOperating => 1.2,
            Self::BusinessSavings => 0.9,
            Self::TrustAccount => 1.5,
            Self::Investment => 1.1,
            Self::ForeignCurrency => 1.8,
            Self::Escrow => 1.3,
            Self::Payroll => 0.7,
        }
    }
}

/// Account status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AccountStatus {
    /// Account is active and in good standing
    #[default]
    Active,
    /// Account is dormant (no activity for extended period)
    Dormant,
    /// Account is frozen due to suspicious activity
    Frozen,
    /// Account is under review
    UnderReview,
    /// Account is closed
    Closed,
    /// Account is pending activation
    PendingActivation,
    /// Account has restrictions
    Restricted,
}

impl AccountStatus {
    /// Whether transactions are allowed on this account.
    pub fn allows_transactions(&self) -> bool {
        matches!(self, Self::Active)
    }

    /// Whether the account can be reopened.
    pub fn can_reopen(&self) -> bool {
        matches!(self, Self::Dormant | Self::Frozen | Self::Restricted)
    }

    /// Risk indicator for status.
    pub fn risk_indicator(&self) -> f64 {
        match self {
            Self::Active => 0.0,
            Self::Dormant => 0.3,
            Self::Frozen => 1.0,
            Self::UnderReview => 0.8,
            Self::Closed => 0.0,
            Self::PendingActivation => 0.2,
            Self::Restricted => 0.7,
        }
    }
}

/// Product features associated with an account.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountFeatures {
    /// Debit card enabled
    pub debit_card: bool,
    /// Online banking enabled
    pub online_banking: bool,
    /// Mobile banking enabled
    pub mobile_banking: bool,
    /// International transfers enabled
    pub international_transfers: bool,
    /// Wire transfers enabled
    pub wire_transfers: bool,
    /// ACH transfers enabled
    pub ach_transfers: bool,
    /// Check writing enabled
    pub check_writing: bool,
    /// Overdraft protection enabled
    pub overdraft_protection: bool,
    /// ATM access enabled
    pub atm_access: bool,
    /// Cash deposits allowed
    pub cash_deposits: bool,
    /// Daily withdrawal limit
    pub daily_withdrawal_limit: Option<u64>,
    /// Daily transfer limit
    pub daily_transfer_limit: Option<u64>,
}

impl AccountFeatures {
    /// Standard retail account features.
    pub fn retail_standard() -> Self {
        Self {
            debit_card: true,
            online_banking: true,
            mobile_banking: true,
            international_transfers: false,
            wire_transfers: false,
            ach_transfers: true,
            check_writing: true,
            overdraft_protection: false,
            atm_access: true,
            cash_deposits: true,
            daily_withdrawal_limit: Some(500),
            daily_transfer_limit: Some(5_000),
        }
    }

    /// Premium retail account features.
    pub fn retail_premium() -> Self {
        Self {
            debit_card: true,
            online_banking: true,
            mobile_banking: true,
            international_transfers: true,
            wire_transfers: true,
            ach_transfers: true,
            check_writing: true,
            overdraft_protection: true,
            atm_access: true,
            cash_deposits: true,
            daily_withdrawal_limit: Some(2_000),
            daily_transfer_limit: Some(50_000),
        }
    }

    /// Standard business account features.
    pub fn business_standard() -> Self {
        Self {
            debit_card: true,
            online_banking: true,
            mobile_banking: true,
            international_transfers: true,
            wire_transfers: true,
            ach_transfers: true,
            check_writing: true,
            overdraft_protection: true,
            atm_access: true,
            cash_deposits: true,
            daily_withdrawal_limit: Some(10_000),
            daily_transfer_limit: Some(250_000),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_account_type_properties() {
        assert!(BankAccountType::Checking.is_high_volume());
        assert!(!BankAccountType::Savings.is_high_volume());
        assert!(BankAccountType::BusinessOperating.is_business());
        assert!(!BankAccountType::Checking.is_business());
    }

    #[test]
    fn test_account_status_properties() {
        assert!(AccountStatus::Active.allows_transactions());
        assert!(!AccountStatus::Frozen.allows_transactions());
        assert!(AccountStatus::Frozen.can_reopen());
        assert!(!AccountStatus::Closed.can_reopen());
    }

    #[test]
    fn test_account_features() {
        let retail = AccountFeatures::retail_standard();
        assert!(retail.debit_card);
        assert!(!retail.international_transfers);

        let business = AccountFeatures::business_standard();
        assert!(business.international_transfers);
        assert!(business.wire_transfers);
    }
}
