//! Banking transaction type definitions.

use serde::{Deserialize, Serialize};

/// Transaction channel (how the transaction was initiated).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TransactionChannel {
    /// Point of sale / card present
    #[default]
    CardPresent,
    /// Card not present (online, phone)
    CardNotPresent,
    /// ATM transaction
    Atm,
    /// ACH/direct debit
    Ach,
    /// Wire transfer
    Wire,
    /// Internal transfer (same bank)
    InternalTransfer,
    /// Mobile banking
    Mobile,
    /// Online banking
    Online,
    /// Branch (in-person)
    Branch,
    /// Cash (deposit or withdrawal)
    Cash,
    /// Check
    Check,
    /// Real-time payment (RTP, FedNow)
    RealTimePayment,
    /// SWIFT international transfer
    Swift,
    /// Peer-to-peer (Zelle, Venmo, etc.)
    PeerToPeer,
}

impl TransactionChannel {
    /// Risk weight for AML scoring.
    pub fn risk_weight(&self) -> f64 {
        match self {
            Self::CardPresent => 0.8,
            Self::CardNotPresent => 1.0,
            Self::Atm => 1.2,
            Self::Ach => 0.9,
            Self::Wire => 1.5,
            Self::InternalTransfer => 0.6,
            Self::Mobile => 1.0,
            Self::Online => 1.0,
            Self::Branch => 0.7,
            Self::Cash => 1.8,
            Self::Check => 1.0,
            Self::RealTimePayment => 1.3,
            Self::Swift => 1.8,
            Self::PeerToPeer => 1.2,
        }
    }

    /// Whether this channel supports cross-border transactions.
    pub fn supports_cross_border(&self) -> bool {
        matches!(self, Self::Wire | Self::Swift | Self::CardNotPresent)
    }

    /// Typical processing time in hours.
    pub fn typical_processing_hours(&self) -> u32 {
        match self {
            Self::CardPresent | Self::CardNotPresent => 0,
            Self::Atm => 0,
            Self::Ach => 48,
            Self::Wire => 4,
            Self::InternalTransfer => 0,
            Self::Mobile | Self::Online => 0,
            Self::Branch => 0,
            Self::Cash => 0,
            Self::Check => 48,
            Self::RealTimePayment => 0,
            Self::Swift => 24,
            Self::PeerToPeer => 0,
        }
    }
}

/// Transaction direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    /// Money coming in (credit)
    #[default]
    Inbound,
    /// Money going out (debit)
    Outbound,
}

impl Direction {
    /// Returns the opposite direction.
    pub fn opposite(&self) -> Self {
        match self {
            Self::Inbound => Self::Outbound,
            Self::Outbound => Self::Inbound,
        }
    }
}

/// Transaction category for behavioral analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionCategory {
    // Income categories
    /// Regular salary/wage deposit
    Salary,
    /// Freelance/contract income
    FreelanceIncome,
    /// Investment income (dividends, interest)
    InvestmentIncome,
    /// Government benefits
    GovernmentBenefit,
    /// Pension/retirement income
    Pension,
    /// Rental income
    RentalIncome,
    /// Refund/rebate
    Refund,
    /// Transfer from own account
    TransferIn,
    /// Cash deposit
    CashDeposit,
    /// Check deposit
    CheckDeposit,

    // Expense categories
    /// Housing (rent, mortgage)
    Housing,
    /// Utilities (electric, gas, water)
    Utilities,
    /// Telecommunications (phone, internet)
    Telecommunications,
    /// Insurance
    Insurance,
    /// Groceries
    Groceries,
    /// Dining/restaurants
    Dining,
    /// Transportation (gas, public transit)
    Transportation,
    /// Healthcare
    Healthcare,
    /// Education
    Education,
    /// Entertainment
    Entertainment,
    /// Shopping (retail)
    Shopping,
    /// Subscription services
    Subscription,
    /// Transfer to own account
    TransferOut,
    /// ATM withdrawal
    AtmWithdrawal,
    /// Loan payment
    LoanPayment,
    /// Credit card payment
    CreditCardPayment,
    /// Investment/trading
    Investment,
    /// Tax payment
    TaxPayment,
    /// Charitable donation
    Charity,
    /// International transfer
    InternationalTransfer,
    /// Peer-to-peer payment
    P2PPayment,
    /// Other/uncategorized
    Other,
}

impl TransactionCategory {
    /// Whether this is an income category.
    pub fn is_income(&self) -> bool {
        matches!(
            self,
            Self::Salary
                | Self::FreelanceIncome
                | Self::InvestmentIncome
                | Self::GovernmentBenefit
                | Self::Pension
                | Self::RentalIncome
                | Self::Refund
                | Self::TransferIn
                | Self::CashDeposit
                | Self::CheckDeposit
        )
    }

    /// Risk weight for AML scoring.
    pub fn risk_weight(&self) -> f64 {
        match self {
            Self::Salary | Self::Pension | Self::GovernmentBenefit => 0.5,
            Self::CashDeposit => 2.0,
            Self::InternationalTransfer => 1.8,
            Self::Investment => 1.3,
            Self::P2PPayment => 1.2,
            Self::Charity => 1.1,
            Self::FreelanceIncome => 1.1,
            Self::RentalIncome => 1.0,
            _ => 1.0,
        }
    }

    /// Whether this category is typically recurring.
    pub fn is_typically_recurring(&self) -> bool {
        matches!(
            self,
            Self::Salary
                | Self::Pension
                | Self::GovernmentBenefit
                | Self::Housing
                | Self::Utilities
                | Self::Telecommunications
                | Self::Insurance
                | Self::Subscription
                | Self::LoanPayment
        )
    }
}

/// Merchant Category Code (MCC) classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MerchantCategoryCode(pub u16);

impl MerchantCategoryCode {
    // Common MCC codes
    pub const GROCERY_STORES: Self = Self(5411);
    pub const RESTAURANTS: Self = Self(5812);
    pub const GAS_STATIONS: Self = Self(5542);
    pub const AIRLINES: Self = Self(3000);
    pub const HOTELS: Self = Self(3500);
    pub const CAR_RENTAL: Self = Self(3351);
    pub const DEPARTMENT_STORES: Self = Self(5311);
    pub const DRUG_STORES: Self = Self(5912);
    pub const UTILITIES: Self = Self(4900);
    pub const TELECOM: Self = Self(4814);
    pub const INSURANCE: Self = Self(6300);
    pub const MEDICAL: Self = Self(8011);
    pub const EDUCATION: Self = Self(8299);
    pub const GOVERNMENT: Self = Self(9311);
    pub const MONEY_TRANSFER: Self = Self(4829);
    pub const WIRE_TRANSFER: Self = Self(4829);
    pub const GAMBLING: Self = Self(7995);
    pub const CRYPTO: Self = Self(6051);

    /// Risk weight for AML scoring.
    pub fn risk_weight(&self) -> f64 {
        match self.0 {
            7995 => 3.0,        // Gambling
            6051 => 2.5,        // Crypto
            4829 => 2.0,        // Money transfer
            5933 => 1.8,        // Pawn shops
            5944 => 1.5,        // Jewelry
            6010..=6012 => 1.3, // Financial institutions
            _ => 1.0,
        }
    }

    /// Category description.
    pub fn description(&self) -> &'static str {
        match self.0 {
            5411 => "Grocery Stores",
            5812 => "Restaurants",
            5542 => "Gas Stations",
            3000..=3299 => "Airlines",
            3500..=3799 => "Hotels",
            3351..=3499 => "Car Rental",
            5311 => "Department Stores",
            5912 => "Drug Stores",
            4900 => "Utilities",
            4814 => "Telecommunications",
            6300 => "Insurance",
            8011 => "Medical Services",
            8299 => "Education",
            9311 => "Government Services",
            4829 => "Money Transfer",
            7995 => "Gambling",
            6051 => "Cryptocurrency",
            _ => "Other",
        }
    }

    /// Whether this is a high-risk MCC.
    pub fn is_high_risk(&self) -> bool {
        matches!(self.0, 7995 | 6051 | 4829 | 5933 | 5944 | 6010..=6012)
    }
}

impl Default for MerchantCategoryCode {
    fn default() -> Self {
        Self(5999) // Miscellaneous retail
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_channel_properties() {
        assert!(TransactionChannel::Wire.supports_cross_border());
        assert!(!TransactionChannel::Atm.supports_cross_border());
        assert!(
            TransactionChannel::Cash.risk_weight() > TransactionChannel::CardPresent.risk_weight()
        );
    }

    #[test]
    fn test_direction_opposite() {
        assert_eq!(Direction::Inbound.opposite(), Direction::Outbound);
        assert_eq!(Direction::Outbound.opposite(), Direction::Inbound);
    }

    #[test]
    fn test_transaction_category() {
        assert!(TransactionCategory::Salary.is_income());
        assert!(!TransactionCategory::Groceries.is_income());
        assert!(TransactionCategory::Housing.is_typically_recurring());
        assert!(!TransactionCategory::Shopping.is_typically_recurring());
    }

    #[test]
    fn test_mcc_properties() {
        assert!(MerchantCategoryCode::GAMBLING.is_high_risk());
        assert!(!MerchantCategoryCode::GROCERY_STORES.is_high_risk());
    }
}
