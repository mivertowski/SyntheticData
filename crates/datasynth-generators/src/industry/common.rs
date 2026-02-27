//! Common traits and types for industry-specific modules.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Trait for industry-specific transactions.
pub trait IndustryTransaction: std::fmt::Debug + Send + Sync {
    /// Returns the transaction type name.
    fn transaction_type(&self) -> &str;

    /// Returns the transaction date.
    fn date(&self) -> NaiveDate;

    /// Returns the transaction amount (if applicable).
    fn amount(&self) -> Option<Decimal>;

    /// Returns the GL account(s) impacted.
    fn accounts(&self) -> Vec<String>;

    /// Converts to journal entry line items.
    fn to_journal_lines(&self) -> Vec<IndustryJournalLine>;

    /// Returns metadata for the transaction.
    fn metadata(&self) -> HashMap<String, String>;
}

/// Journal line generated from industry transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustryJournalLine {
    /// GL account number.
    pub account: String,
    /// Debit amount (zero if credit).
    pub debit: Decimal,
    /// Credit amount (zero if debit).
    pub credit: Decimal,
    /// Description.
    pub description: String,
    /// Cost center (if applicable).
    pub cost_center: Option<String>,
    /// Additional dimensions.
    pub dimensions: HashMap<String, String>,
}

impl IndustryJournalLine {
    /// Creates a debit line.
    pub fn debit(
        account: impl Into<String>,
        amount: Decimal,
        description: impl Into<String>,
    ) -> Self {
        Self {
            account: account.into(),
            debit: amount,
            credit: Decimal::ZERO,
            description: description.into(),
            cost_center: None,
            dimensions: HashMap::new(),
        }
    }

    /// Creates a credit line.
    pub fn credit(
        account: impl Into<String>,
        amount: Decimal,
        description: impl Into<String>,
    ) -> Self {
        Self {
            account: account.into(),
            debit: Decimal::ZERO,
            credit: amount,
            description: description.into(),
            cost_center: None,
            dimensions: HashMap::new(),
        }
    }

    /// Sets the cost center.
    pub fn with_cost_center(mut self, cost_center: impl Into<String>) -> Self {
        self.cost_center = Some(cost_center.into());
        self
    }

    /// Adds a dimension.
    pub fn with_dimension(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.dimensions.insert(key.into(), value.into());
        self
    }
}

/// Trait for industry-specific anomalies.
pub trait IndustryAnomaly: std::fmt::Debug + Send + Sync {
    /// Returns the anomaly type name.
    fn anomaly_type(&self) -> &str;

    /// Returns the severity (1-5).
    fn severity(&self) -> u8;

    /// Returns detection difficulty.
    fn detection_difficulty(&self) -> &str;

    /// Returns indicators that should trigger detection.
    fn indicators(&self) -> Vec<String>;

    /// Returns related regulatory concerns.
    fn regulatory_concerns(&self) -> Vec<String>;
}

/// Trait for industry-specific transaction generators.
///
/// This is the intended future API for pluggable industry modules.
/// Concrete implementations will be added as each industry vertical is built out.
#[allow(unused)]
pub trait IndustryTransactionGenerator: Send + Sync {
    /// The transaction type produced by this generator.
    type Transaction: IndustryTransaction;

    /// The anomaly type produced by this generator.
    type Anomaly: IndustryAnomaly;

    /// Generates transactions for a period.
    fn generate_transactions(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
        count: usize,
    ) -> Vec<Self::Transaction>;

    /// Generates industry-specific anomalies.
    fn generate_anomalies(&self, transactions: &[Self::Transaction]) -> Vec<Self::Anomaly>;

    /// Returns industry-specific GL accounts.
    fn gl_accounts(&self) -> Vec<IndustryGlAccount>;
}

/// Industry-specific GL account definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustryGlAccount {
    /// Account number.
    pub account_number: String,
    /// Account name.
    pub name: String,
    /// Account type (Asset, Liability, Revenue, Expense, Equity).
    pub account_type: String,
    /// Industry-specific category.
    pub category: String,
    /// Whether this is a control account.
    pub is_control: bool,
    /// Normal balance (Debit or Credit).
    pub normal_balance: String,
}

impl IndustryGlAccount {
    /// Creates a new GL account.
    pub fn new(
        number: impl Into<String>,
        name: impl Into<String>,
        account_type: impl Into<String>,
        category: impl Into<String>,
    ) -> Self {
        Self {
            account_number: number.into(),
            name: name.into(),
            account_type: account_type.into(),
            category: category.into(),
            is_control: false,
            normal_balance: "Debit".to_string(),
        }
    }

    /// Marks as control account.
    pub fn into_control(mut self) -> Self {
        self.is_control = true;
        self
    }

    /// Sets normal balance.
    pub fn with_normal_balance(mut self, balance: impl Into<String>) -> Self {
        self.normal_balance = balance.into();
        self
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_journal_line() {
        let debit = IndustryJournalLine::debit("1000", Decimal::new(1000, 0), "Test debit")
            .with_cost_center("CC001")
            .with_dimension("project", "P001");

        assert_eq!(debit.account, "1000");
        assert_eq!(debit.debit, Decimal::new(1000, 0));
        assert_eq!(debit.credit, Decimal::ZERO);
        assert_eq!(debit.cost_center, Some("CC001".to_string()));
        assert_eq!(debit.dimensions.get("project"), Some(&"P001".to_string()));

        let credit = IndustryJournalLine::credit("2000", Decimal::new(1000, 0), "Test credit");
        assert_eq!(credit.debit, Decimal::ZERO);
        assert_eq!(credit.credit, Decimal::new(1000, 0));
    }

    #[test]
    fn test_gl_account() {
        let account =
            IndustryGlAccount::new("5100", "Cost of Goods Sold", "Expense", "Manufacturing")
                .into_control()
                .with_normal_balance("Debit");

        assert_eq!(account.account_number, "5100");
        assert!(account.is_control);
        assert_eq!(account.normal_balance, "Debit");
    }
}
