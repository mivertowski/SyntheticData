//! Banking account model for KYC/AML simulation.

use chrono::{DateTime, NaiveDate, Utc};
use datasynth_core::models::banking::{AccountFeatures, AccountStatus, BankAccountType};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A bank account with full metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankAccount {
    /// Unique account identifier
    pub account_id: Uuid,
    /// Account number (masked for output)
    pub account_number: String,
    /// Account type
    pub account_type: BankAccountType,
    /// Primary owner customer ID
    pub primary_owner_id: Uuid,
    /// Joint owner customer IDs
    pub joint_owner_ids: Vec<Uuid>,
    /// Account status
    pub status: AccountStatus,
    /// Account currency (ISO 4217)
    pub currency: String,
    /// Account opening date
    pub opening_date: NaiveDate,
    /// Account closing date (if closed)
    pub closing_date: Option<NaiveDate>,
    /// Current balance
    #[serde(with = "rust_decimal::serde::str")]
    pub current_balance: Decimal,
    /// Available balance (may differ due to holds)
    #[serde(with = "rust_decimal::serde::str")]
    pub available_balance: Decimal,
    /// Account features/capabilities
    pub features: AccountFeatures,
    /// IBAN (for international accounts)
    pub iban: Option<String>,
    /// BIC/SWIFT code
    pub swift_bic: Option<String>,
    /// Routing number (for US accounts)
    pub routing_number: Option<String>,
    /// Branch code
    pub branch_code: Option<String>,
    /// Interest rate (for savings/CD)
    pub interest_rate: Option<Decimal>,
    /// Overdraft limit
    #[serde(with = "rust_decimal::serde::str")]
    pub overdraft_limit: Decimal,
    /// Last activity timestamp
    pub last_activity: Option<DateTime<Utc>>,
    /// Days dormant (calculated field)
    pub days_dormant: u32,
    /// Is this a nominee account
    pub is_nominee: bool,
    /// Linked card numbers (masked)
    pub linked_cards: Vec<String>,
    /// Purpose of account (declared)
    pub declared_purpose: Option<String>,
    /// Source account for linked funding
    pub funding_source_account: Option<Uuid>,
    /// FK → GL account number in the chart of accounts (e.g. "1010" for Cash at Bank).
    /// `None` when the account has not yet been mapped to the general ledger.
    pub gl_account: Option<String>,

    // Ground truth labels
    /// Whether this is a mule account
    pub is_mule_account: bool,
    /// Whether this is a funnel account
    pub is_funnel_account: bool,
    /// Associated case ID for suspicious activity
    pub case_id: Option<String>,
}

impl BankAccount {
    /// Create a new account.
    pub fn new(
        account_id: Uuid,
        account_number: String,
        account_type: BankAccountType,
        primary_owner_id: Uuid,
        currency: &str,
        opening_date: NaiveDate,
    ) -> Self {
        let features = match account_type {
            BankAccountType::Checking => AccountFeatures::retail_standard(),
            BankAccountType::BusinessOperating => AccountFeatures::business_standard(),
            _ => AccountFeatures::default(),
        };

        Self {
            account_id,
            account_number,
            account_type,
            primary_owner_id,
            joint_owner_ids: Vec::new(),
            status: AccountStatus::Active,
            currency: currency.to_string(),
            opening_date,
            closing_date: None,
            current_balance: Decimal::ZERO,
            available_balance: Decimal::ZERO,
            features,
            iban: None,
            swift_bic: None,
            routing_number: None,
            branch_code: None,
            interest_rate: None,
            overdraft_limit: Decimal::ZERO,
            last_activity: None,
            days_dormant: 0,
            is_nominee: false,
            linked_cards: Vec::new(),
            declared_purpose: None,
            funding_source_account: None,
            is_mule_account: false,
            is_funnel_account: false,
            case_id: None,
            gl_account: None,
        }
    }

    /// Check if account can process transactions.
    pub fn can_transact(&self) -> bool {
        self.status.allows_transactions()
    }

    /// Check if account has sufficient funds for debit.
    pub fn has_sufficient_funds(&self, amount: Decimal) -> bool {
        self.available_balance + self.overdraft_limit >= amount
    }

    /// Apply a debit (outgoing transaction).
    pub fn apply_debit(&mut self, amount: Decimal, timestamp: DateTime<Utc>) -> bool {
        if !self.has_sufficient_funds(amount) {
            return false;
        }
        self.current_balance -= amount;
        self.available_balance -= amount;
        self.last_activity = Some(timestamp);
        self.days_dormant = 0;
        true
    }

    /// Apply a credit (incoming transaction).
    pub fn apply_credit(&mut self, amount: Decimal, timestamp: DateTime<Utc>) {
        self.current_balance += amount;
        self.available_balance += amount;
        self.last_activity = Some(timestamp);
        self.days_dormant = 0;
    }

    /// Place a hold on funds.
    pub fn place_hold(&mut self, amount: Decimal) {
        self.available_balance -= amount;
    }

    /// Release a hold on funds.
    pub fn release_hold(&mut self, amount: Decimal) {
        self.available_balance += amount;
    }

    /// Close the account.
    pub fn close(&mut self, close_date: NaiveDate) {
        self.status = AccountStatus::Closed;
        self.closing_date = Some(close_date);
    }

    /// Freeze the account.
    pub fn freeze(&mut self) {
        self.status = AccountStatus::Frozen;
    }

    /// Mark as dormant.
    pub fn mark_dormant(&mut self, days: u32) {
        self.days_dormant = days;
        if days > 365 {
            self.status = AccountStatus::Dormant;
        }
    }

    /// Add a joint owner.
    pub fn add_joint_owner(&mut self, owner_id: Uuid) {
        if !self.joint_owner_ids.contains(&owner_id) {
            self.joint_owner_ids.push(owner_id);
        }
    }

    /// Get all owner IDs (primary + joint).
    pub fn all_owner_ids(&self) -> Vec<Uuid> {
        let mut owners = vec![self.primary_owner_id];
        owners.extend(&self.joint_owner_ids);
        owners
    }

    /// Calculate risk score for the account.
    pub fn calculate_risk_score(&self) -> u8 {
        let mut score = self.account_type.risk_weight() * 30.0;

        // Status risk
        score += self.status.risk_indicator() * 20.0;

        // Feature risk
        if self.features.international_transfers {
            score += 10.0;
        }
        if self.features.wire_transfers {
            score += 5.0;
        }
        if self.features.cash_deposits {
            score += 5.0;
        }

        // Ground truth
        if self.is_mule_account {
            score += 50.0;
        }
        if self.is_funnel_account {
            score += 40.0;
        }

        score.min(100.0) as u8
    }
}

/// Account holder summary for reporting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountHolder {
    /// Customer ID
    pub customer_id: Uuid,
    /// Holder type
    pub holder_type: AccountHolderType,
    /// Ownership percentage (for joint accounts)
    pub ownership_percent: Option<u8>,
    /// Date added as holder
    pub added_date: NaiveDate,
}

/// Type of account holder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountHolderType {
    /// Primary account owner
    Primary,
    /// Joint owner with full rights
    JointOwner,
    /// Authorized signer (no ownership)
    AuthorizedSigner,
    /// Beneficiary
    Beneficiary,
    /// Power of attorney
    PowerOfAttorney,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_account_creation() {
        let account = BankAccount::new(
            Uuid::new_v4(),
            "****1234".to_string(),
            BankAccountType::Checking,
            Uuid::new_v4(),
            "USD",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        assert!(account.can_transact());
        assert_eq!(account.current_balance, Decimal::ZERO);
    }

    #[test]
    fn test_account_transactions() {
        let mut account = BankAccount::new(
            Uuid::new_v4(),
            "****1234".to_string(),
            BankAccountType::Checking,
            Uuid::new_v4(),
            "USD",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        let now = Utc::now();

        // Credit
        account.apply_credit(Decimal::from(1000), now);
        assert_eq!(account.current_balance, Decimal::from(1000));

        // Debit
        assert!(account.apply_debit(Decimal::from(500), now));
        assert_eq!(account.current_balance, Decimal::from(500));

        // Insufficient funds
        assert!(!account.apply_debit(Decimal::from(1000), now));
    }

    #[test]
    fn test_account_freeze() {
        let mut account = BankAccount::new(
            Uuid::new_v4(),
            "****1234".to_string(),
            BankAccountType::Checking,
            Uuid::new_v4(),
            "USD",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        account.freeze();
        assert!(!account.can_transact());
    }

    #[test]
    fn test_joint_owners() {
        let mut account = BankAccount::new(
            Uuid::new_v4(),
            "****1234".to_string(),
            BankAccountType::Checking,
            Uuid::new_v4(),
            "USD",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        let joint_owner = Uuid::new_v4();
        account.add_joint_owner(joint_owner);

        let owners = account.all_owner_ids();
        assert_eq!(owners.len(), 2);
    }
}
