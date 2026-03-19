//! Stock-based compensation models — ASC 718 / IFRS 2.
//!
//! This module provides data models for equity-settled share-based payment
//! arrangements, including stock option grants, restricted stock units (RSUs),
//! performance share units (PSUs), vesting schedules, and period expense
//! recognition.
//!
//! # Framework references
//!
//! | Topic                     | ASC 718 (US GAAP)                  | IFRS 2                            |
//! |---------------------------|------------------------------------|-----------------------------------|
//! | Measurement date          | Grant date (equity awards)         | Grant date (equity awards)        |
//! | Fair value model          | Option pricing model required      | Option pricing model required     |
//! | Expense recognition       | Straight-line or graded            | Straight-line (tranche-by-tranche)|
//! | Forfeiture estimate       | Estimate at grant; true-up         | Estimate at grant; true-up        |
//! | Vesting conditions        | Service, performance, market       | Service, performance, market      |

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Type of equity instrument granted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InstrumentType {
    /// Stock options — right to purchase shares at the exercise price.
    Options,
    /// Restricted Stock Units — shares vest on service / time conditions.
    #[default]
    RSUs,
    /// Performance Share Units — vest subject to performance conditions.
    PSUs,
}

impl std::fmt::Display for InstrumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Options => write!(f, "Options"),
            Self::RSUs => write!(f, "RSUs"),
            Self::PSUs => write!(f, "PSUs"),
        }
    }
}

/// Method used to determine the vesting pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VestingType {
    /// All shares vest at a single date (100% cliff).
    Cliff,
    /// Shares vest in equal tranches over multiple periods.
    #[default]
    Graded,
    /// Vesting depends on achievement of performance targets.
    Performance,
}

// ---------------------------------------------------------------------------
// Vesting schedule
// ---------------------------------------------------------------------------

/// A single vesting event within a schedule.
///
/// Each entry captures the percentage of the total grant that vests on the
/// given date, together with the cumulative percentage vested to that point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VestingEntry {
    /// Sequential period number (1-indexed, e.g. Year 1 = 1, Year 2 = 2 …).
    pub period: u32,
    /// Date on which this tranche vests.
    pub vesting_date: NaiveDate,
    /// Percentage of the total grant vesting in this period (e.g. 0.25 = 25%).
    #[serde(with = "rust_decimal::serde::str")]
    pub percentage: Decimal,
    /// Cumulative percentage vested through this entry (e.g. 0.50 after Year 2 of 4).
    #[serde(with = "rust_decimal::serde::str")]
    pub cumulative_percentage: Decimal,
}

/// Vesting schedule attached to a stock grant.
///
/// For graded vesting the entries have equal `percentage` each year.
/// For cliff vesting there is a single entry with `percentage = 1.00`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VestingSchedule {
    /// Vesting pattern type.
    pub vesting_type: VestingType,
    /// Total number of vesting periods (e.g. 4 for a standard 4-year schedule).
    pub total_periods: u32,
    /// Cliff period count — periods before any vesting occurs (may be 0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cliff_periods: Option<u32>,
    /// Ordered list of vesting events; percentages must sum to 1.00.
    pub vesting_entries: Vec<VestingEntry>,
}

// ---------------------------------------------------------------------------
// Stock grant
// ---------------------------------------------------------------------------

/// A single stock-based compensation grant awarded to an employee.
///
/// One `StockGrant` corresponds to one award agreement.  For option grants
/// the `exercise_price` field is populated; RSUs and PSUs typically have
/// no exercise price.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockGrant {
    /// Unique grant identifier (e.g. "GRANT-1000-EMP001-2024").
    pub id: String,
    /// Company / entity code that issued the grant.
    pub entity_code: String,
    /// Employee who received the grant.
    pub employee_id: String,
    /// Date on which the grant was approved and the fair value is fixed.
    pub grant_date: NaiveDate,
    /// Type of instrument (Options, RSUs, or PSUs).
    pub instrument_type: InstrumentType,
    /// Number of shares / units granted.
    pub quantity: u32,
    /// Strike / exercise price per share (only applicable to Options).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "rust_decimal::serde::str_option"
    )]
    pub exercise_price: Option<Decimal>,
    /// Fair value per share / unit at the grant date (measurement basis for expense).
    #[serde(with = "rust_decimal::serde::str")]
    pub fair_value_at_grant: Decimal,
    /// Total grant-date fair value (`quantity × fair_value_at_grant`).
    #[serde(with = "rust_decimal::serde::str")]
    pub total_grant_value: Decimal,
    /// Vesting schedule defining when each tranche vests.
    pub vesting_schedule: VestingSchedule,
    /// Expiration date of options (None for RSUs/PSUs which expire on vesting).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiration_date: Option<NaiveDate>,
    /// Estimated annual forfeiture rate applied to reduce total grant expense.
    #[serde(with = "rust_decimal::serde::str")]
    pub forfeiture_rate: Decimal,
    /// Reporting currency code (e.g. "USD").
    pub currency: String,
}

// ---------------------------------------------------------------------------
// Period expense record
// ---------------------------------------------------------------------------

/// Stock-based compensation expense recognised for a grant in one period.
///
/// Generated for each active vesting period.  The `cumulative_recognized`
/// plus `remaining_unrecognized` equals the total expense budget for this
/// grant after applying the forfeiture estimate.
///
/// # Identities
///
/// `cumulative_recognized + remaining_unrecognized
///   ≈ total_grant_value × (1 − forfeiture_rate)`  (within rounding)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockCompExpense {
    /// Foreign key to `StockGrant.id`.
    pub grant_id: String,
    /// Company / entity code.
    pub entity_code: String,
    /// Period label (e.g. "2024-Q1" or "2024-12").
    pub period: String,
    /// Expense recognised in this period.
    #[serde(with = "rust_decimal::serde::str")]
    pub expense_amount: Decimal,
    /// Cumulative expense recognised through the end of this period.
    #[serde(with = "rust_decimal::serde::str")]
    pub cumulative_recognized: Decimal,
    /// Remaining unrecognised expense after this period.
    #[serde(with = "rust_decimal::serde::str")]
    pub remaining_unrecognized: Decimal,
    /// Forfeiture rate applied to this grant (snapshot at grant date).
    #[serde(with = "rust_decimal::serde::str")]
    pub forfeiture_rate: Decimal,
}
