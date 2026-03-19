//! IAS 21 currency translation result models.
//!
//! These models capture the output of translating an entity's financial
//! statements from its functional currency into the group presentation currency.
//!
//! Under the **current-rate method** (most common for foreign operations):
//! - Balance sheet monetary items → closing rate
//! - Balance sheet non-monetary items (PP&E, equity) → historical rate
//! - P&L items → average rate for the period
//! - The balancing difference is the **Currency Translation Adjustment (CTA)**,
//!   recognised as Other Comprehensive Income (OCI).

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// The overall IAS 21 translation method applied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ias21TranslationMethod {
    /// Current-rate method — all balance-sheet items at closing rate except
    /// equity which uses historical rates; P&L at average rate.
    CurrentRate,
    /// Temporal method — monetary items at closing rate, non-monetary at
    /// historical rate; P&L at average rate.
    Temporal,
}

impl std::fmt::Display for Ias21TranslationMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CurrentRate => write!(f, "current_rate"),
            Self::Temporal => write!(f, "temporal"),
        }
    }
}

/// Which exchange rate was used to translate a particular line item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranslationRateType {
    /// Period-end (closing) rate — used for balance-sheet monetary items.
    ClosingRate,
    /// Rate prevailing at the original transaction date — used for equity and
    /// non-monetary balance-sheet items.
    HistoricalRate,
    /// Weighted average rate for the period — used for P&L items.
    AverageRate,
    /// One-to-one (no translation required; functional == presentation).
    NoTranslation,
}

impl std::fmt::Display for TranslationRateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ClosingRate => write!(f, "closing_rate"),
            Self::HistoricalRate => write!(f, "historical_rate"),
            Self::AverageRate => write!(f, "average_rate"),
            Self::NoTranslation => write!(f, "no_translation"),
        }
    }
}

/// A single translated line item (one GL account / account group).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslatedLineItem {
    /// GL account code.
    pub account: String,
    /// Account type label (e.g. "Asset", "Liability", "Revenue").
    pub account_type: String,
    /// Amount in the entity's functional currency.
    pub functional_amount: Decimal,
    /// Exchange rate applied.
    pub rate_used: Decimal,
    /// Category of rate used.
    pub rate_type: TranslationRateType,
    /// Translated amount in the presentation currency.
    pub presentation_amount: Decimal,
}

/// Full IAS 21 translation result for one entity and one reporting period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyTranslationResult {
    /// Entity (company) code.
    pub entity_code: String,
    /// Functional currency of the entity (ISO 4217).
    pub functional_currency: String,
    /// Presentation (group reporting) currency (ISO 4217).
    pub presentation_currency: String,
    /// Period label (e.g. "2024-12").
    pub period: String,
    /// Translation method applied.
    pub translation_method: Ias21TranslationMethod,
    /// All translated line items.
    pub translated_items: Vec<TranslatedLineItem>,
    /// Currency Translation Adjustment recognised in OCI.
    ///
    /// Positive = OCI gain (foreign currency strengthened relative to
    /// presentation currency); negative = OCI loss.
    pub cta_amount: Decimal,
    /// Closing rate used for balance-sheet monetary items.
    pub closing_rate: Decimal,
    /// Average rate used for P&L items.
    pub average_rate: Decimal,
    /// Total balance-sheet amount translated (functional currency).
    pub total_balance_sheet_functional: Decimal,
    /// Total balance-sheet amount translated (presentation currency).
    pub total_balance_sheet_presentation: Decimal,
    /// Total P&L amount translated (functional currency).
    pub total_pnl_functional: Decimal,
    /// Total P&L amount translated (presentation currency).
    pub total_pnl_presentation: Decimal,
}
