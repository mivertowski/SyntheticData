//! Banking transaction model for KYC/AML simulation.

#![allow(clippy::too_many_arguments)]

use chrono::{DateTime, Utc};
use datasynth_core::models::banking::{
    AmlTypology, Direction, LaunderingStage, MerchantCategoryCode, TransactionCategory,
    TransactionChannel,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A bank transaction with full metadata and ground truth labels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankTransaction {
    /// Unique transaction identifier
    pub transaction_id: Uuid,
    /// Account ID
    pub account_id: Uuid,
    /// Timestamp when transaction was initiated
    pub timestamp_initiated: DateTime<Utc>,
    /// Timestamp when transaction was booked
    pub timestamp_booked: DateTime<Utc>,
    /// Timestamp when transaction was settled
    pub timestamp_settled: Option<DateTime<Utc>>,
    /// Transaction amount (always positive)
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,
    /// Transaction currency (ISO 4217)
    pub currency: String,
    /// Transaction direction (inbound/outbound)
    pub direction: Direction,
    /// Transaction channel
    pub channel: TransactionChannel,
    /// Transaction category
    pub category: TransactionCategory,
    /// Counterparty reference
    pub counterparty: CounterpartyRef,
    /// Merchant category code (for card transactions)
    pub mcc: Option<MerchantCategoryCode>,
    /// Transaction reference/description
    pub reference: String,
    /// Balance before transaction
    #[serde(with = "rust_decimal::serde::str_option")]
    pub balance_before: Option<Decimal>,
    /// Balance after transaction
    #[serde(with = "rust_decimal::serde::str_option")]
    pub balance_after: Option<Decimal>,
    /// Original currency (if FX conversion)
    pub original_currency: Option<String>,
    /// Original amount (if FX conversion)
    #[serde(with = "rust_decimal::serde::str_option")]
    pub original_amount: Option<Decimal>,
    /// FX rate applied
    #[serde(with = "rust_decimal::serde::str_option")]
    pub fx_rate: Option<Decimal>,
    /// Location (country code)
    pub location_country: Option<String>,
    /// Location (city)
    pub location_city: Option<String>,
    /// Device fingerprint (for online/mobile)
    pub device_id: Option<String>,
    /// IP address (masked for output)
    pub ip_address: Option<String>,
    /// Whether transaction was authorized
    pub is_authorized: bool,
    /// Authorization code
    pub auth_code: Option<String>,
    /// Transaction status
    pub status: TransactionStatus,
    /// Parent transaction ID (for reversals, fees)
    pub parent_transaction_id: Option<Uuid>,

    // Ground truth labels for ML
    /// Whether transaction is suspicious (ground truth)
    pub is_suspicious: bool,
    /// Suspicion reason (AML typology)
    pub suspicion_reason: Option<AmlTypology>,
    /// Money laundering stage
    pub laundering_stage: Option<LaunderingStage>,
    /// Case ID linking suspicious transactions
    pub case_id: Option<String>,
    /// Whether transaction is spoofed (adversarial mode)
    pub is_spoofed: bool,
    /// Spoofing intensity (0.0-1.0)
    pub spoofing_intensity: Option<f64>,
    /// Scenario ID for linked transactions
    pub scenario_id: Option<String>,
    /// Transaction sequence number within scenario
    pub scenario_sequence: Option<u32>,
}

impl BankTransaction {
    /// Create a new transaction.
    pub fn new(
        transaction_id: Uuid,
        account_id: Uuid,
        amount: Decimal,
        currency: &str,
        direction: Direction,
        channel: TransactionChannel,
        category: TransactionCategory,
        counterparty: CounterpartyRef,
        reference: &str,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            transaction_id,
            account_id,
            timestamp_initiated: timestamp,
            timestamp_booked: timestamp,
            timestamp_settled: None,
            amount,
            currency: currency.to_string(),
            direction,
            channel,
            category,
            counterparty,
            mcc: None,
            reference: reference.to_string(),
            balance_before: None,
            balance_after: None,
            original_currency: None,
            original_amount: None,
            fx_rate: None,
            location_country: None,
            location_city: None,
            device_id: None,
            ip_address: None,
            is_authorized: true,
            auth_code: None,
            status: TransactionStatus::Completed,
            parent_transaction_id: None,
            is_suspicious: false,
            suspicion_reason: None,
            laundering_stage: None,
            case_id: None,
            is_spoofed: false,
            spoofing_intensity: None,
            scenario_id: None,
            scenario_sequence: None,
        }
    }

    /// Mark as suspicious.
    pub fn mark_suspicious(mut self, reason: AmlTypology, case_id: &str) -> Self {
        self.is_suspicious = true;
        self.suspicion_reason = Some(reason);
        self.case_id = Some(case_id.to_string());
        self
    }

    /// Set laundering stage.
    pub fn with_laundering_stage(mut self, stage: LaunderingStage) -> Self {
        self.laundering_stage = Some(stage);
        self
    }

    /// Mark as spoofed.
    pub fn mark_spoofed(mut self, intensity: f64) -> Self {
        self.is_spoofed = true;
        self.spoofing_intensity = Some(intensity);
        self
    }

    /// Set scenario information.
    pub fn with_scenario(mut self, scenario_id: &str, sequence: u32) -> Self {
        self.scenario_id = Some(scenario_id.to_string());
        self.scenario_sequence = Some(sequence);
        self
    }

    /// Set MCC.
    pub fn with_mcc(mut self, mcc: MerchantCategoryCode) -> Self {
        self.mcc = Some(mcc);
        self
    }

    /// Set location.
    pub fn with_location(mut self, country: &str, city: Option<&str>) -> Self {
        self.location_country = Some(country.to_string());
        self.location_city = city.map(|c| c.to_string());
        self
    }

    /// Set FX conversion.
    pub fn with_fx_conversion(
        mut self,
        original_currency: &str,
        original_amount: Decimal,
        rate: Decimal,
    ) -> Self {
        self.original_currency = Some(original_currency.to_string());
        self.original_amount = Some(original_amount);
        self.fx_rate = Some(rate);
        self
    }

    /// Set balance information.
    pub fn with_balance(mut self, before: Decimal, after: Decimal) -> Self {
        self.balance_before = Some(before);
        self.balance_after = Some(after);
        self
    }

    /// Calculate risk score for the transaction.
    pub fn calculate_risk_score(&self) -> u8 {
        let mut score = 0.0;

        // Channel risk
        score += self.channel.risk_weight() * 10.0;

        // Category risk
        score += self.category.risk_weight() * 10.0;

        // Amount risk (log scale)
        let amount_f64: f64 = self.amount.try_into().unwrap_or(0.0);
        if amount_f64 > 10_000.0 {
            score += ((amount_f64 / 10_000.0).ln() * 5.0).min(20.0);
        }

        // MCC risk
        if let Some(mcc) = self.mcc {
            score += mcc.risk_weight() * 5.0;
        }

        // Cross-border risk
        if self.original_currency.is_some() {
            score += 10.0;
        }

        // Ground truth (if available, would dominate)
        if self.is_suspicious {
            score += 50.0;
        }

        score.min(100.0) as u8
    }

    /// Check if this is a cash transaction.
    pub fn is_cash(&self) -> bool {
        matches!(
            self.channel,
            TransactionChannel::Cash | TransactionChannel::Atm
        )
    }

    /// Check if this is a cross-border transaction.
    pub fn is_cross_border(&self) -> bool {
        self.original_currency.is_some() || matches!(self.channel, TransactionChannel::Swift)
    }
}

/// Reference to a counterparty.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterpartyRef {
    /// Counterparty type
    pub counterparty_type: CounterpartyType,
    /// Counterparty ID (if known)
    pub counterparty_id: Option<Uuid>,
    /// Counterparty name
    pub name: String,
    /// Account identifier (masked)
    pub account_identifier: Option<String>,
    /// Bank identifier (BIC/SWIFT)
    pub bank_identifier: Option<String>,
    /// Country (ISO 3166-1 alpha-2)
    pub country: Option<String>,
}

impl CounterpartyRef {
    /// Create a merchant counterparty.
    pub fn merchant(id: Uuid, name: &str) -> Self {
        Self {
            counterparty_type: CounterpartyType::Merchant,
            counterparty_id: Some(id),
            name: name.to_string(),
            account_identifier: None,
            bank_identifier: None,
            country: None,
        }
    }

    /// Create an employer counterparty.
    pub fn employer(id: Uuid, name: &str) -> Self {
        Self {
            counterparty_type: CounterpartyType::Employer,
            counterparty_id: Some(id),
            name: name.to_string(),
            account_identifier: None,
            bank_identifier: None,
            country: None,
        }
    }

    /// Create a peer-to-peer counterparty.
    pub fn peer(name: &str, account: Option<&str>) -> Self {
        Self {
            counterparty_type: CounterpartyType::Peer,
            counterparty_id: None,
            name: name.to_string(),
            account_identifier: account.map(|a| a.to_string()),
            bank_identifier: None,
            country: None,
        }
    }

    /// Create an ATM counterparty.
    pub fn atm(location: &str) -> Self {
        Self {
            counterparty_type: CounterpartyType::Atm,
            counterparty_id: None,
            name: format!("ATM - {}", location),
            account_identifier: None,
            bank_identifier: None,
            country: None,
        }
    }

    /// Create a self-transfer counterparty.
    pub fn self_account(account_id: Uuid, account_name: &str) -> Self {
        Self {
            counterparty_type: CounterpartyType::SelfAccount,
            counterparty_id: Some(account_id),
            name: account_name.to_string(),
            account_identifier: None,
            bank_identifier: None,
            country: None,
        }
    }

    /// Create an unknown counterparty.
    pub fn unknown(name: &str) -> Self {
        Self {
            counterparty_type: CounterpartyType::Unknown,
            counterparty_id: None,
            name: name.to_string(),
            account_identifier: None,
            bank_identifier: None,
            country: None,
        }
    }

    /// Create a person/individual counterparty.
    pub fn person(name: &str) -> Self {
        Self {
            counterparty_type: CounterpartyType::Peer,
            counterparty_id: None,
            name: name.to_string(),
            account_identifier: None,
            bank_identifier: None,
            country: None,
        }
    }

    /// Create a business counterparty.
    pub fn business(name: &str) -> Self {
        Self {
            counterparty_type: CounterpartyType::Unknown,
            counterparty_id: None,
            name: name.to_string(),
            account_identifier: None,
            bank_identifier: None,
            country: None,
        }
    }

    /// Create an international counterparty.
    pub fn international(name: &str) -> Self {
        Self {
            counterparty_type: CounterpartyType::FinancialInstitution,
            counterparty_id: None,
            name: name.to_string(),
            account_identifier: None,
            bank_identifier: None,
            country: Some("XX".to_string()), // Unknown foreign country
        }
    }

    /// Create a crypto exchange counterparty.
    pub fn crypto_exchange(name: &str) -> Self {
        Self {
            counterparty_type: CounterpartyType::CryptoExchange,
            counterparty_id: None,
            name: name.to_string(),
            account_identifier: None,
            bank_identifier: None,
            country: None,
        }
    }

    /// Create a service provider counterparty.
    pub fn service(name: &str) -> Self {
        Self {
            counterparty_type: CounterpartyType::Unknown,
            counterparty_id: None,
            name: name.to_string(),
            account_identifier: None,
            bank_identifier: None,
            country: None,
        }
    }

    /// Create a merchant counterparty by name only.
    pub fn merchant_by_name(name: &str, _mcc: &str) -> Self {
        Self {
            counterparty_type: CounterpartyType::Merchant,
            counterparty_id: None,
            name: name.to_string(),
            account_identifier: None,
            bank_identifier: None,
            country: None,
        }
    }
}

/// Type of counterparty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CounterpartyType {
    /// Merchant / retailer
    Merchant,
    /// Employer (salary source)
    Employer,
    /// Utility company
    Utility,
    /// Government agency
    Government,
    /// Financial institution
    FinancialInstitution,
    /// Peer (another individual)
    Peer,
    /// ATM
    Atm,
    /// Own account (transfer)
    SelfAccount,
    /// Investment platform
    Investment,
    /// Cryptocurrency exchange
    CryptoExchange,
    /// Unknown
    Unknown,
}

impl CounterpartyType {
    /// Risk weight for AML scoring.
    pub fn risk_weight(&self) -> f64 {
        match self {
            Self::Merchant => 1.0,
            Self::Employer => 0.5,
            Self::Utility | Self::Government => 0.3,
            Self::FinancialInstitution => 1.2,
            Self::Peer => 1.5,
            Self::Atm => 1.3,
            Self::SelfAccount => 0.8,
            Self::Investment => 1.2,
            Self::CryptoExchange => 2.0,
            Self::Unknown => 1.8,
        }
    }
}

/// Transaction status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    /// Pending authorization
    Pending,
    /// Authorized but not settled
    Authorized,
    /// Completed/settled
    #[default]
    Completed,
    /// Failed
    Failed,
    /// Declined
    Declined,
    /// Reversed
    Reversed,
    /// Disputed
    Disputed,
    /// On hold for review
    OnHold,
}

impl TransactionStatus {
    /// Whether the transaction is finalized.
    pub fn is_final(&self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Declined | Self::Reversed
        )
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_creation() {
        let txn = BankTransaction::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Decimal::from(100),
            "USD",
            Direction::Outbound,
            TransactionChannel::CardPresent,
            TransactionCategory::Shopping,
            CounterpartyRef::merchant(Uuid::new_v4(), "Test Store"),
            "Purchase at Test Store",
            Utc::now(),
        );

        assert!(!txn.is_suspicious);
        assert!(!txn.is_cross_border());
    }

    #[test]
    fn test_suspicious_transaction() {
        let txn = BankTransaction::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Decimal::from(9500),
            "USD",
            Direction::Inbound,
            TransactionChannel::Cash,
            TransactionCategory::CashDeposit,
            CounterpartyRef::atm("Main Branch"),
            "Cash deposit",
            Utc::now(),
        )
        .mark_suspicious(AmlTypology::Structuring, "CASE-001");

        assert!(txn.is_suspicious);
        assert_eq!(txn.suspicion_reason, Some(AmlTypology::Structuring));
    }

    #[test]
    fn test_risk_score() {
        let low_risk = BankTransaction::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Decimal::from(50),
            "USD",
            Direction::Outbound,
            TransactionChannel::CardPresent,
            TransactionCategory::Groceries,
            CounterpartyRef::merchant(Uuid::new_v4(), "Grocery Store"),
            "Groceries",
            Utc::now(),
        );

        let high_risk = BankTransaction::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Decimal::from(50000),
            "USD",
            Direction::Outbound,
            TransactionChannel::Wire,
            TransactionCategory::InternationalTransfer,
            CounterpartyRef::unknown("Unknown Recipient"),
            "Wire transfer",
            Utc::now(),
        );

        assert!(high_risk.calculate_risk_score() > low_risk.calculate_risk_score());
    }
}
