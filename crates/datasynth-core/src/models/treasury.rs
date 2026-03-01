//! Treasury and cash management data models.
//!
//! This module provides comprehensive treasury models including:
//! - Daily cash positions per entity/account/currency
//! - Forward-looking cash forecasts with probability-weighted items
//! - Cash pooling structures (physical, notional, zero-balancing)
//! - Hedging instruments (FX forwards, IR swaps) under ASC 815 / IFRS 9
//! - Hedge relationship designations with effectiveness testing
//! - Debt instruments with amortization schedules and covenant monitoring
//! - Bank guarantees and letters of credit
//! - Intercompany netting runs with multilateral settlement

use std::collections::HashMap;

use chrono::{NaiveDate, NaiveTime};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::graph_properties::{GraphPropertyValue, ToNodeProperties};

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Category of a cash flow item in a treasury forecast.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TreasuryCashFlowCategory {
    /// Accounts receivable collection
    #[default]
    ArCollection,
    /// Accounts payable payment
    ApPayment,
    /// Payroll disbursement
    PayrollDisbursement,
    /// Tax payment to authority
    TaxPayment,
    /// Debt principal and interest service
    DebtService,
    /// Capital expenditure
    CapitalExpenditure,
    /// Intercompany settlement
    IntercompanySettlement,
    /// Project milestone payment
    ProjectMilestone,
    /// Other / unclassified cash flow
    Other,
}

/// Type of cash pool structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PoolType {
    /// Physical sweeping of balances to a header account
    #[default]
    PhysicalPooling,
    /// Balances remain in sub-accounts; interest calculated on notional aggregate
    NotionalPooling,
    /// Sub-accounts are swept to zero daily
    ZeroBalancing,
}

/// Type of hedging instrument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum HedgeInstrumentType {
    /// Foreign exchange forward contract
    #[default]
    FxForward,
    /// Foreign exchange option
    FxOption,
    /// Interest rate swap
    InterestRateSwap,
    /// Commodity forward contract
    CommodityForward,
    /// Cross-currency interest rate swap
    CrossCurrencySwap,
}

/// Lifecycle status of a hedging instrument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InstrumentStatus {
    /// Instrument is live and outstanding
    #[default]
    Active,
    /// Instrument has reached maturity date
    Matured,
    /// Instrument was terminated early
    Terminated,
    /// Instrument was novated to a new counterparty
    Novated,
}

/// Type of hedged item under ASC 815 / IFRS 9.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum HedgedItemType {
    /// Highly probable future transaction
    #[default]
    ForecastedTransaction,
    /// Binding contractual commitment
    FirmCommitment,
    /// Asset or liability already on balance sheet
    RecognizedAsset,
    /// Net investment in a foreign operation
    NetInvestment,
}

/// Hedge accounting classification under ASC 815 / IFRS 9.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum HedgeType {
    /// Fair value hedge — hedges the fair value of an asset/liability
    #[default]
    FairValueHedge,
    /// Cash flow hedge — hedges variability of future cash flows
    CashFlowHedge,
    /// Net investment hedge — hedges FX risk in foreign subsidiaries
    NetInvestmentHedge,
}

/// Method used to test hedge effectiveness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EffectivenessMethod {
    /// Dollar-offset method (ratio of cumulative changes)
    #[default]
    DollarOffset,
    /// Statistical regression analysis
    Regression,
    /// Critical terms match (qualitative)
    CriticalTerms,
}

/// Type of debt instrument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DebtType {
    /// Amortizing term loan
    #[default]
    TermLoan,
    /// Revolving credit facility
    RevolvingCredit,
    /// Bond issuance
    Bond,
    /// Commercial paper (short-term)
    CommercialPaper,
    /// Bridge loan (interim financing)
    BridgeLoan,
}

/// Interest rate type on a debt instrument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InterestRateType {
    /// Fixed interest rate for the life of the instrument
    #[default]
    Fixed,
    /// Floating rate (index + spread)
    Variable,
}

/// Type of financial covenant on a debt instrument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CovenantType {
    /// Total debt / total equity
    #[default]
    DebtToEquity,
    /// EBIT / interest expense
    InterestCoverage,
    /// Current assets / current liabilities
    CurrentRatio,
    /// Minimum net worth requirement
    NetWorth,
    /// Total debt / EBITDA
    DebtToEbitda,
    /// (EBITDA - CapEx) / fixed charges
    FixedChargeCoverage,
}

/// Measurement frequency for covenant testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Frequency {
    /// Monthly measurement
    Monthly,
    /// Quarterly measurement
    #[default]
    Quarterly,
    /// Annual measurement
    Annual,
}

/// Type of bank guarantee or letter of credit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum GuaranteeType {
    /// Commercial letter of credit (trade finance)
    #[default]
    CommercialLc,
    /// Standby letter of credit (financial guarantee)
    StandbyLc,
    /// Bank guarantee
    BankGuarantee,
    /// Performance bond
    PerformanceBond,
}

/// Lifecycle status of a bank guarantee.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum GuaranteeStatus {
    /// Guarantee is active
    #[default]
    Active,
    /// Guarantee has been drawn upon
    Drawn,
    /// Guarantee has expired
    Expired,
    /// Guarantee was cancelled
    Cancelled,
}

/// Netting cycle frequency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NettingCycle {
    /// Daily netting
    Daily,
    /// Weekly netting
    Weekly,
    /// Monthly netting
    #[default]
    Monthly,
}

/// Settlement direction for a netting position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PayOrReceive {
    /// Entity must pay the net amount
    #[default]
    Pay,
    /// Entity will receive the net amount
    Receive,
    /// Entity's position is zero
    Flat,
}

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

/// Daily cash position per entity / bank account / currency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashPosition {
    /// Unique position identifier
    pub id: String,
    /// Legal entity
    pub entity_id: String,
    /// Bank account holding the cash
    pub bank_account_id: String,
    /// Position currency
    pub currency: String,
    /// Position date
    pub date: NaiveDate,
    /// Balance at start of day
    #[serde(with = "rust_decimal::serde::str")]
    pub opening_balance: Decimal,
    /// Total inflows during the day
    #[serde(with = "rust_decimal::serde::str")]
    pub inflows: Decimal,
    /// Total outflows during the day
    #[serde(with = "rust_decimal::serde::str")]
    pub outflows: Decimal,
    /// Balance at end of day (opening + inflows - outflows)
    #[serde(with = "rust_decimal::serde::str")]
    pub closing_balance: Decimal,
    /// Available balance (after holds, pending transactions)
    #[serde(with = "rust_decimal::serde::str")]
    pub available_balance: Decimal,
    /// Value-date balance (settlement-adjusted)
    #[serde(with = "rust_decimal::serde::str")]
    pub value_date_balance: Decimal,
}

impl CashPosition {
    /// Creates a new cash position.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        entity_id: impl Into<String>,
        bank_account_id: impl Into<String>,
        currency: impl Into<String>,
        date: NaiveDate,
        opening_balance: Decimal,
        inflows: Decimal,
        outflows: Decimal,
    ) -> Self {
        let closing = (opening_balance + inflows - outflows).round_dp(2);
        Self {
            id: id.into(),
            entity_id: entity_id.into(),
            bank_account_id: bank_account_id.into(),
            currency: currency.into(),
            date,
            opening_balance,
            inflows,
            outflows,
            closing_balance: closing,
            available_balance: closing,
            value_date_balance: closing,
        }
    }

    /// Overrides the available balance.
    pub fn with_available_balance(mut self, balance: Decimal) -> Self {
        self.available_balance = balance;
        self
    }

    /// Overrides the value-date balance.
    pub fn with_value_date_balance(mut self, balance: Decimal) -> Self {
        self.value_date_balance = balance;
        self
    }

    /// Computes closing balance from opening + inflows - outflows.
    pub fn computed_closing_balance(&self) -> Decimal {
        (self.opening_balance + self.inflows - self.outflows).round_dp(2)
    }
}

/// A single item in a cash forecast.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashForecastItem {
    /// Unique item identifier
    pub id: String,
    /// Expected date of the cash flow
    pub date: NaiveDate,
    /// Category of the forecast item
    pub category: TreasuryCashFlowCategory,
    /// Expected amount (positive = inflow, negative = outflow)
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,
    /// Probability of occurrence (0.0 to 1.0)
    #[serde(with = "rust_decimal::serde::str")]
    pub probability: Decimal,
    /// Source document type (e.g., "SalesOrder", "PurchaseOrder")
    pub source_document_type: Option<String>,
    /// Source document identifier
    pub source_document_id: Option<String>,
}

/// Forward-looking cash forecast for an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashForecast {
    /// Unique forecast identifier
    pub id: String,
    /// Legal entity
    pub entity_id: String,
    /// Forecast currency
    pub currency: String,
    /// Date the forecast was prepared
    pub forecast_date: NaiveDate,
    /// Number of days the forecast covers
    pub horizon_days: u32,
    /// Individual forecast line items
    pub items: Vec<CashForecastItem>,
    /// Net position (sum of probability-weighted amounts)
    #[serde(with = "rust_decimal::serde::str")]
    pub net_position: Decimal,
    /// Confidence level for the forecast (0.0 to 1.0)
    #[serde(with = "rust_decimal::serde::str")]
    pub confidence_level: Decimal,
}

impl CashForecast {
    /// Creates a new cash forecast.
    pub fn new(
        id: impl Into<String>,
        entity_id: impl Into<String>,
        currency: impl Into<String>,
        forecast_date: NaiveDate,
        horizon_days: u32,
        items: Vec<CashForecastItem>,
        confidence_level: Decimal,
    ) -> Self {
        let net_position = items
            .iter()
            .map(|item| (item.amount * item.probability).round_dp(2))
            .sum::<Decimal>()
            .round_dp(2);
        Self {
            id: id.into(),
            entity_id: entity_id.into(),
            currency: currency.into(),
            forecast_date,
            horizon_days,
            items,
            net_position,
            confidence_level,
        }
    }

    /// Recomputes the net position from the probability-weighted items.
    pub fn computed_net_position(&self) -> Decimal {
        self.items
            .iter()
            .map(|item| (item.amount * item.probability).round_dp(2))
            .sum::<Decimal>()
            .round_dp(2)
    }
}

/// Cash pool grouping entity bank accounts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashPool {
    /// Unique pool identifier
    pub id: String,
    /// Descriptive name
    pub name: String,
    /// Type of pooling structure
    pub pool_type: PoolType,
    /// Master / header account receiving sweeps
    pub header_account_id: String,
    /// Participant sub-account identifiers
    pub participant_accounts: Vec<String>,
    /// Time of day when sweeps occur
    pub sweep_time: NaiveTime,
    /// Interest rate benefit from pooling (bps or decimal fraction)
    #[serde(with = "rust_decimal::serde::str")]
    pub interest_rate_benefit: Decimal,
}

impl CashPool {
    /// Creates a new cash pool.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        pool_type: PoolType,
        header_account_id: impl Into<String>,
        sweep_time: NaiveTime,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            pool_type,
            header_account_id: header_account_id.into(),
            participant_accounts: Vec::new(),
            sweep_time,
            interest_rate_benefit: Decimal::ZERO,
        }
    }

    /// Adds a participant account.
    pub fn with_participant(mut self, account_id: impl Into<String>) -> Self {
        self.participant_accounts.push(account_id.into());
        self
    }

    /// Sets the interest rate benefit.
    pub fn with_interest_rate_benefit(mut self, benefit: Decimal) -> Self {
        self.interest_rate_benefit = benefit;
        self
    }

    /// Returns the total number of accounts in the pool (header + participants).
    pub fn total_accounts(&self) -> usize {
        1 + self.participant_accounts.len()
    }
}

/// A single sweep transaction within a cash pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashPoolSweep {
    /// Unique sweep identifier
    pub id: String,
    /// Pool this sweep belongs to
    pub pool_id: String,
    /// Date of the sweep
    pub date: NaiveDate,
    /// Source account (balance swept from)
    pub from_account_id: String,
    /// Destination account (balance swept to)
    pub to_account_id: String,
    /// Amount swept
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,
    /// Currency of the sweep
    pub currency: String,
}

/// A hedging instrument (derivative contract).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HedgingInstrument {
    /// Unique instrument identifier
    pub id: String,
    /// Type of derivative
    pub instrument_type: HedgeInstrumentType,
    /// Notional / face amount
    #[serde(with = "rust_decimal::serde::str")]
    pub notional_amount: Decimal,
    /// Primary currency
    pub currency: String,
    /// Currency pair for FX instruments (e.g., "EUR/USD")
    pub currency_pair: Option<String>,
    /// Fixed rate (for swaps, forwards)
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub fixed_rate: Option<Decimal>,
    /// Floating rate index name (e.g., "SOFR", "EURIBOR")
    pub floating_index: Option<String>,
    /// Strike rate for options
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub strike_rate: Option<Decimal>,
    /// Trade date
    pub trade_date: NaiveDate,
    /// Maturity / expiry date
    pub maturity_date: NaiveDate,
    /// Counterparty name
    pub counterparty: String,
    /// Current fair value (mark-to-market)
    #[serde(with = "rust_decimal::serde::str")]
    pub fair_value: Decimal,
    /// Current lifecycle status
    pub status: InstrumentStatus,
}

impl HedgingInstrument {
    /// Creates a new hedging instrument.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        instrument_type: HedgeInstrumentType,
        notional_amount: Decimal,
        currency: impl Into<String>,
        trade_date: NaiveDate,
        maturity_date: NaiveDate,
        counterparty: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            instrument_type,
            notional_amount,
            currency: currency.into(),
            currency_pair: None,
            fixed_rate: None,
            floating_index: None,
            strike_rate: None,
            trade_date,
            maturity_date,
            counterparty: counterparty.into(),
            fair_value: Decimal::ZERO,
            status: InstrumentStatus::Active,
        }
    }

    /// Sets the currency pair.
    pub fn with_currency_pair(mut self, pair: impl Into<String>) -> Self {
        self.currency_pair = Some(pair.into());
        self
    }

    /// Sets the fixed rate.
    pub fn with_fixed_rate(mut self, rate: Decimal) -> Self {
        self.fixed_rate = Some(rate);
        self
    }

    /// Sets the floating rate index.
    pub fn with_floating_index(mut self, index: impl Into<String>) -> Self {
        self.floating_index = Some(index.into());
        self
    }

    /// Sets the strike rate.
    pub fn with_strike_rate(mut self, rate: Decimal) -> Self {
        self.strike_rate = Some(rate);
        self
    }

    /// Sets the fair value.
    pub fn with_fair_value(mut self, value: Decimal) -> Self {
        self.fair_value = value;
        self
    }

    /// Sets the status.
    pub fn with_status(mut self, status: InstrumentStatus) -> Self {
        self.status = status;
        self
    }

    /// Returns `true` if the instrument is still outstanding.
    pub fn is_active(&self) -> bool {
        self.status == InstrumentStatus::Active
    }

    /// Returns the remaining tenor in days from the given date.
    /// Returns 0 if the instrument has already matured.
    pub fn remaining_tenor_days(&self, as_of: NaiveDate) -> i64 {
        (self.maturity_date - as_of).num_days().max(0)
    }
}

/// ASC 815 / IFRS 9 hedge relationship designation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HedgeRelationship {
    /// Unique relationship identifier
    pub id: String,
    /// Type of hedged item
    pub hedged_item_type: HedgedItemType,
    /// Description of what is being hedged
    pub hedged_item_description: String,
    /// Hedging instrument linked to this relationship
    pub hedging_instrument_id: String,
    /// Hedge accounting classification
    pub hedge_type: HedgeType,
    /// Date the hedge was designated
    pub designation_date: NaiveDate,
    /// Method used for effectiveness testing
    pub effectiveness_test_method: EffectivenessMethod,
    /// Effectiveness ratio (hedging instrument change / hedged item change)
    #[serde(with = "rust_decimal::serde::str")]
    pub effectiveness_ratio: Decimal,
    /// Whether the hedge qualifies as effective (ratio within 80-125%)
    pub is_effective: bool,
    /// Ineffectiveness amount recognized in P&L
    #[serde(with = "rust_decimal::serde::str")]
    pub ineffectiveness_amount: Decimal,
}

impl HedgeRelationship {
    /// Creates a new hedge relationship.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        hedged_item_type: HedgedItemType,
        hedged_item_description: impl Into<String>,
        hedging_instrument_id: impl Into<String>,
        hedge_type: HedgeType,
        designation_date: NaiveDate,
        effectiveness_test_method: EffectivenessMethod,
        effectiveness_ratio: Decimal,
    ) -> Self {
        let is_effective = Self::check_effectiveness(effectiveness_ratio);
        Self {
            id: id.into(),
            hedged_item_type,
            hedged_item_description: hedged_item_description.into(),
            hedging_instrument_id: hedging_instrument_id.into(),
            hedge_type,
            designation_date,
            effectiveness_test_method,
            effectiveness_ratio,
            is_effective,
            ineffectiveness_amount: Decimal::ZERO,
        }
    }

    /// Sets the ineffectiveness amount.
    pub fn with_ineffectiveness_amount(mut self, amount: Decimal) -> Self {
        self.ineffectiveness_amount = amount;
        self
    }

    /// Checks whether the effectiveness ratio is within the 80-125% corridor.
    ///
    /// Under ASC 815 / IAS 39, a hedge is considered highly effective if the
    /// ratio of changes in the hedging instrument to changes in the hedged item
    /// falls within 0.80 to 1.25.
    pub fn check_effectiveness(ratio: Decimal) -> bool {
        let lower = Decimal::new(80, 2); // 0.80
        let upper = Decimal::new(125, 2); // 1.25
        ratio >= lower && ratio <= upper
    }

    /// Recomputes the `is_effective` flag from the current ratio.
    pub fn update_effectiveness(&mut self) {
        self.is_effective = Self::check_effectiveness(self.effectiveness_ratio);
    }
}

/// A single payment in a debt amortization schedule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmortizationPayment {
    /// Payment date
    pub date: NaiveDate,
    /// Principal portion of the payment
    #[serde(with = "rust_decimal::serde::str")]
    pub principal_payment: Decimal,
    /// Interest portion of the payment
    #[serde(with = "rust_decimal::serde::str")]
    pub interest_payment: Decimal,
    /// Outstanding balance after this payment
    #[serde(with = "rust_decimal::serde::str")]
    pub balance_after: Decimal,
}

impl AmortizationPayment {
    /// Total payment (principal + interest).
    pub fn total_payment(&self) -> Decimal {
        (self.principal_payment + self.interest_payment).round_dp(2)
    }
}

/// A financial covenant attached to a debt instrument.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtCovenant {
    /// Unique covenant identifier
    pub id: String,
    /// Type of financial ratio being tested
    pub covenant_type: CovenantType,
    /// Covenant threshold value
    #[serde(with = "rust_decimal::serde::str")]
    pub threshold: Decimal,
    /// How often the covenant is tested
    pub measurement_frequency: Frequency,
    /// Most recent actual measured value
    #[serde(with = "rust_decimal::serde::str")]
    pub actual_value: Decimal,
    /// Date the measurement was taken
    pub measurement_date: NaiveDate,
    /// Whether the entity is in compliance
    pub is_compliant: bool,
    /// Distance from the covenant threshold (positive = headroom, negative = breach)
    #[serde(with = "rust_decimal::serde::str")]
    pub headroom: Decimal,
    /// Whether a waiver was obtained for a breach
    pub waiver_obtained: bool,

    // -- Standalone fields for graph export (DS-003) --
    /// Back-reference to the parent debt instrument ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub facility_id: Option<String>,
    /// Entity / company code
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity_code: Option<String>,
    /// Debt facility name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub facility_name: Option<String>,
    /// Outstanding principal at measurement date
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outstanding_principal: Option<Decimal>,
    /// Currency of the facility
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// Fiscal period (e.g. "2024-06")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period: Option<String>,
}

impl DebtCovenant {
    /// Creates a new debt covenant.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        covenant_type: CovenantType,
        threshold: Decimal,
        measurement_frequency: Frequency,
        actual_value: Decimal,
        measurement_date: NaiveDate,
    ) -> Self {
        let (is_compliant, headroom) =
            Self::evaluate_compliance(covenant_type, threshold, actual_value);
        Self {
            id: id.into(),
            covenant_type,
            threshold,
            measurement_frequency,
            actual_value,
            measurement_date,
            is_compliant,
            headroom,
            waiver_obtained: false,
            facility_id: None,
            entity_code: None,
            facility_name: None,
            outstanding_principal: None,
            currency: None,
            period: None,
        }
    }

    /// Sets the waiver flag.
    pub fn with_waiver(mut self, waiver: bool) -> Self {
        self.waiver_obtained = waiver;
        self
    }

    /// Set the parent debt facility details for standalone graph export.
    pub fn with_facility(
        mut self,
        facility_id: impl Into<String>,
        entity_code: impl Into<String>,
        facility_name: impl Into<String>,
        outstanding_principal: Decimal,
        currency: impl Into<String>,
        period: impl Into<String>,
    ) -> Self {
        self.facility_id = Some(facility_id.into());
        self.entity_code = Some(entity_code.into());
        self.facility_name = Some(facility_name.into());
        self.outstanding_principal = Some(outstanding_principal);
        self.currency = Some(currency.into());
        self.period = Some(period.into());
        self
    }

    /// Evaluates compliance and computes headroom.
    ///
    /// For "maximum" covenants (DebtToEquity, DebtToEbitda): actual must be ≤ threshold.
    /// For "minimum" covenants (InterestCoverage, CurrentRatio, NetWorth, FixedChargeCoverage):
    /// actual must be ≥ threshold.
    fn evaluate_compliance(
        covenant_type: CovenantType,
        threshold: Decimal,
        actual_value: Decimal,
    ) -> (bool, Decimal) {
        match covenant_type {
            // Maximum covenants: actual <= threshold means compliant
            CovenantType::DebtToEquity | CovenantType::DebtToEbitda => {
                let headroom = (threshold - actual_value).round_dp(4);
                (actual_value <= threshold, headroom)
            }
            // Minimum covenants: actual >= threshold means compliant
            CovenantType::InterestCoverage
            | CovenantType::CurrentRatio
            | CovenantType::NetWorth
            | CovenantType::FixedChargeCoverage => {
                let headroom = (actual_value - threshold).round_dp(4);
                (actual_value >= threshold, headroom)
            }
        }
    }

    /// Recomputes compliance and headroom from current values.
    pub fn update_compliance(&mut self) {
        let (compliant, headroom) =
            Self::evaluate_compliance(self.covenant_type, self.threshold, self.actual_value);
        self.is_compliant = compliant;
        self.headroom = headroom;
    }
}

/// A debt instrument (loan, bond, credit facility).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtInstrument {
    /// Unique instrument identifier
    pub id: String,
    /// Legal entity borrower
    pub entity_id: String,
    /// Type of debt instrument
    pub instrument_type: DebtType,
    /// Lender / creditor name
    pub lender: String,
    /// Original principal amount
    #[serde(with = "rust_decimal::serde::str")]
    pub principal: Decimal,
    /// Denomination currency
    pub currency: String,
    /// Interest rate (annual, as decimal fraction)
    #[serde(with = "rust_decimal::serde::str")]
    pub interest_rate: Decimal,
    /// Fixed or variable rate
    pub rate_type: InterestRateType,
    /// Date the instrument was originated
    pub origination_date: NaiveDate,
    /// Contractual maturity date
    pub maturity_date: NaiveDate,
    /// Amortization schedule (empty for bullet / revolving)
    pub amortization_schedule: Vec<AmortizationPayment>,
    /// Associated financial covenants
    pub covenants: Vec<DebtCovenant>,
    /// Current drawn amount (for revolving facilities)
    #[serde(with = "rust_decimal::serde::str")]
    pub drawn_amount: Decimal,
    /// Committed facility limit (for revolving facilities)
    #[serde(with = "rust_decimal::serde::str")]
    pub facility_limit: Decimal,
}

impl DebtInstrument {
    /// Creates a new debt instrument.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        entity_id: impl Into<String>,
        instrument_type: DebtType,
        lender: impl Into<String>,
        principal: Decimal,
        currency: impl Into<String>,
        interest_rate: Decimal,
        rate_type: InterestRateType,
        origination_date: NaiveDate,
        maturity_date: NaiveDate,
    ) -> Self {
        Self {
            id: id.into(),
            entity_id: entity_id.into(),
            instrument_type,
            lender: lender.into(),
            principal,
            currency: currency.into(),
            interest_rate,
            rate_type,
            origination_date,
            maturity_date,
            amortization_schedule: Vec::new(),
            covenants: Vec::new(),
            drawn_amount: principal,
            facility_limit: principal,
        }
    }

    /// Sets the amortization schedule.
    pub fn with_amortization_schedule(mut self, schedule: Vec<AmortizationPayment>) -> Self {
        self.amortization_schedule = schedule;
        self
    }

    /// Adds a covenant.
    pub fn with_covenant(mut self, covenant: DebtCovenant) -> Self {
        self.covenants.push(covenant);
        self
    }

    /// Sets the drawn amount (for revolving facilities).
    pub fn with_drawn_amount(mut self, amount: Decimal) -> Self {
        self.drawn_amount = amount;
        self
    }

    /// Sets the facility limit (for revolving facilities).
    pub fn with_facility_limit(mut self, limit: Decimal) -> Self {
        self.facility_limit = limit;
        self
    }

    /// Returns the total principal payments across the amortization schedule.
    pub fn total_principal_payments(&self) -> Decimal {
        self.amortization_schedule
            .iter()
            .map(|p| p.principal_payment)
            .sum::<Decimal>()
            .round_dp(2)
    }

    /// Returns the total interest payments across the amortization schedule.
    pub fn total_interest_payments(&self) -> Decimal {
        self.amortization_schedule
            .iter()
            .map(|p| p.interest_payment)
            .sum::<Decimal>()
            .round_dp(2)
    }

    /// Returns available capacity on a revolving credit facility.
    pub fn available_capacity(&self) -> Decimal {
        (self.facility_limit - self.drawn_amount).round_dp(2)
    }

    /// Returns `true` if all covenants are compliant.
    pub fn all_covenants_compliant(&self) -> bool {
        self.covenants
            .iter()
            .all(|c| c.is_compliant || c.waiver_obtained)
    }
}

/// A bank guarantee or letter of credit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankGuarantee {
    /// Unique guarantee identifier
    pub id: String,
    /// Legal entity that obtained the guarantee
    pub entity_id: String,
    /// Type of guarantee
    pub guarantee_type: GuaranteeType,
    /// Face amount of the guarantee
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,
    /// Denomination currency
    pub currency: String,
    /// Party in whose favour the guarantee is issued
    pub beneficiary: String,
    /// Bank that issued the guarantee
    pub issuing_bank: String,
    /// Issue date
    pub issue_date: NaiveDate,
    /// Expiry date
    pub expiry_date: NaiveDate,
    /// Current lifecycle status
    pub status: GuaranteeStatus,
    /// Linked procurement contract (if applicable)
    pub linked_contract_id: Option<String>,
    /// Linked project (if applicable)
    pub linked_project_id: Option<String>,
}

impl BankGuarantee {
    /// Creates a new bank guarantee.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        entity_id: impl Into<String>,
        guarantee_type: GuaranteeType,
        amount: Decimal,
        currency: impl Into<String>,
        beneficiary: impl Into<String>,
        issuing_bank: impl Into<String>,
        issue_date: NaiveDate,
        expiry_date: NaiveDate,
    ) -> Self {
        Self {
            id: id.into(),
            entity_id: entity_id.into(),
            guarantee_type,
            amount,
            currency: currency.into(),
            beneficiary: beneficiary.into(),
            issuing_bank: issuing_bank.into(),
            issue_date,
            expiry_date,
            status: GuaranteeStatus::Active,
            linked_contract_id: None,
            linked_project_id: None,
        }
    }

    /// Sets the status.
    pub fn with_status(mut self, status: GuaranteeStatus) -> Self {
        self.status = status;
        self
    }

    /// Links to a procurement contract.
    pub fn with_linked_contract(mut self, contract_id: impl Into<String>) -> Self {
        self.linked_contract_id = Some(contract_id.into());
        self
    }

    /// Links to a project.
    pub fn with_linked_project(mut self, project_id: impl Into<String>) -> Self {
        self.linked_project_id = Some(project_id.into());
        self
    }

    /// Returns `true` if the guarantee is active on the given date.
    pub fn is_active_on(&self, date: NaiveDate) -> bool {
        self.status == GuaranteeStatus::Active
            && date >= self.issue_date
            && date <= self.expiry_date
    }

    /// Returns the remaining validity in days from the given date.
    pub fn remaining_days(&self, as_of: NaiveDate) -> i64 {
        (self.expiry_date - as_of).num_days().max(0)
    }
}

/// A netting position for a single entity within a netting run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NettingPosition {
    /// Entity identifier
    pub entity_id: String,
    /// Gross amount receivable from other entities
    #[serde(with = "rust_decimal::serde::str")]
    pub gross_receivable: Decimal,
    /// Gross amount payable to other entities
    #[serde(with = "rust_decimal::serde::str")]
    pub gross_payable: Decimal,
    /// Net position (receivable - payable)
    #[serde(with = "rust_decimal::serde::str")]
    pub net_position: Decimal,
    /// Whether this entity pays or receives
    pub settlement_direction: PayOrReceive,
}

/// An intercompany netting run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NettingRun {
    /// Unique netting run identifier
    pub id: String,
    /// Settlement date
    pub netting_date: NaiveDate,
    /// Netting cycle frequency
    pub cycle: NettingCycle,
    /// List of participating entity IDs
    pub participating_entities: Vec<String>,
    /// Total gross receivables across all entities
    #[serde(with = "rust_decimal::serde::str")]
    pub gross_receivables: Decimal,
    /// Total gross payables across all entities
    #[serde(with = "rust_decimal::serde::str")]
    pub gross_payables: Decimal,
    /// Net settlement amount (sum of absolute net positions / 2)
    #[serde(with = "rust_decimal::serde::str")]
    pub net_settlement: Decimal,
    /// Settlement currency
    pub settlement_currency: String,
    /// Per-entity positions
    pub positions: Vec<NettingPosition>,
}

impl NettingRun {
    /// Creates a new netting run.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        netting_date: NaiveDate,
        cycle: NettingCycle,
        settlement_currency: impl Into<String>,
        positions: Vec<NettingPosition>,
    ) -> Self {
        let participating_entities: Vec<String> =
            positions.iter().map(|p| p.entity_id.clone()).collect();
        let gross_receivables = positions
            .iter()
            .map(|p| p.gross_receivable)
            .sum::<Decimal>()
            .round_dp(2);
        let gross_payables = positions
            .iter()
            .map(|p| p.gross_payable)
            .sum::<Decimal>()
            .round_dp(2);
        let net_settlement = positions
            .iter()
            .map(|p| p.net_position.abs())
            .sum::<Decimal>()
            .round_dp(2)
            / Decimal::TWO;
        Self {
            id: id.into(),
            netting_date,
            cycle,
            participating_entities,
            gross_receivables,
            gross_payables,
            net_settlement: net_settlement.round_dp(2),
            settlement_currency: settlement_currency.into(),
            positions,
        }
    }

    /// Payment savings from netting: gross flows eliminated.
    ///
    /// `savings = max(gross_receivables, gross_payables) - net_settlement`
    pub fn savings(&self) -> Decimal {
        let gross_max = self.gross_receivables.max(self.gross_payables);
        (gross_max - self.net_settlement).round_dp(2)
    }

    /// Savings as a percentage of gross flows.
    pub fn savings_pct(&self) -> Decimal {
        let gross_max = self.gross_receivables.max(self.gross_payables);
        if gross_max.is_zero() {
            return Decimal::ZERO;
        }
        (self.savings() / gross_max * Decimal::ONE_HUNDRED).round_dp(2)
    }
}

// ---------------------------------------------------------------------------
// ToNodeProperties implementations
// ---------------------------------------------------------------------------

impl ToNodeProperties for CashPosition {
    fn node_type_name(&self) -> &'static str {
        "cash_position"
    }
    fn node_type_code(&self) -> u16 {
        420
    }
    fn to_node_properties(&self) -> HashMap<String, GraphPropertyValue> {
        let mut p = HashMap::new();
        p.insert(
            "entityCode".into(),
            GraphPropertyValue::String(self.entity_id.clone()),
        );
        p.insert(
            "bankId".into(),
            GraphPropertyValue::String(self.bank_account_id.clone()),
        );
        p.insert(
            "currency".into(),
            GraphPropertyValue::String(self.currency.clone()),
        );
        p.insert("asOfDate".into(), GraphPropertyValue::Date(self.date));
        p.insert(
            "openingBalance".into(),
            GraphPropertyValue::Decimal(self.opening_balance),
        );
        p.insert("inflows".into(), GraphPropertyValue::Decimal(self.inflows));
        p.insert(
            "outflows".into(),
            GraphPropertyValue::Decimal(self.outflows),
        );
        p.insert(
            "balance".into(),
            GraphPropertyValue::Decimal(self.closing_balance),
        );
        p.insert(
            "availableBalance".into(),
            GraphPropertyValue::Decimal(self.available_balance),
        );
        p.insert(
            "valueDateBalance".into(),
            GraphPropertyValue::Decimal(self.value_date_balance),
        );
        p
    }
}

impl ToNodeProperties for CashForecast {
    fn node_type_name(&self) -> &'static str {
        "cash_forecast"
    }
    fn node_type_code(&self) -> u16 {
        421
    }
    fn to_node_properties(&self) -> HashMap<String, GraphPropertyValue> {
        let mut p = HashMap::new();
        p.insert(
            "entityCode".into(),
            GraphPropertyValue::String(self.entity_id.clone()),
        );
        p.insert(
            "currency".into(),
            GraphPropertyValue::String(self.currency.clone()),
        );
        p.insert(
            "forecastDate".into(),
            GraphPropertyValue::Date(self.forecast_date),
        );
        p.insert(
            "horizonDays".into(),
            GraphPropertyValue::Int(self.horizon_days as i64),
        );
        p.insert(
            "itemCount".into(),
            GraphPropertyValue::Int(self.items.len() as i64),
        );
        p.insert(
            "netPosition".into(),
            GraphPropertyValue::Decimal(self.net_position),
        );
        p.insert(
            "certaintyLevel".into(),
            GraphPropertyValue::Float(self.confidence_level.to_f64().unwrap_or(0.0)),
        );
        p
    }
}

impl ToNodeProperties for CashPool {
    fn node_type_name(&self) -> &'static str {
        "cash_pool"
    }
    fn node_type_code(&self) -> u16 {
        422
    }
    fn to_node_properties(&self) -> HashMap<String, GraphPropertyValue> {
        let mut p = HashMap::new();
        p.insert("name".into(), GraphPropertyValue::String(self.name.clone()));
        p.insert(
            "poolType".into(),
            GraphPropertyValue::String(format!("{:?}", self.pool_type)),
        );
        p.insert(
            "headerAccount".into(),
            GraphPropertyValue::String(self.header_account_id.clone()),
        );
        p.insert(
            "participantCount".into(),
            GraphPropertyValue::Int(self.participant_accounts.len() as i64),
        );
        p.insert(
            "interestBenefit".into(),
            GraphPropertyValue::Decimal(self.interest_rate_benefit),
        );
        p
    }
}

impl ToNodeProperties for CashPoolSweep {
    fn node_type_name(&self) -> &'static str {
        "cash_pool_sweep"
    }
    fn node_type_code(&self) -> u16 {
        423
    }
    fn to_node_properties(&self) -> HashMap<String, GraphPropertyValue> {
        let mut p = HashMap::new();
        p.insert(
            "poolId".into(),
            GraphPropertyValue::String(self.pool_id.clone()),
        );
        p.insert("date".into(), GraphPropertyValue::Date(self.date));
        p.insert(
            "fromAccount".into(),
            GraphPropertyValue::String(self.from_account_id.clone()),
        );
        p.insert(
            "toAccount".into(),
            GraphPropertyValue::String(self.to_account_id.clone()),
        );
        p.insert("amount".into(), GraphPropertyValue::Decimal(self.amount));
        p.insert(
            "currency".into(),
            GraphPropertyValue::String(self.currency.clone()),
        );
        p
    }
}

impl ToNodeProperties for HedgingInstrument {
    fn node_type_name(&self) -> &'static str {
        "hedging_instrument"
    }
    fn node_type_code(&self) -> u16 {
        424
    }
    fn to_node_properties(&self) -> HashMap<String, GraphPropertyValue> {
        let mut p = HashMap::new();
        p.insert(
            "hedgeType".into(),
            GraphPropertyValue::String(format!("{:?}", self.instrument_type)),
        );
        p.insert(
            "notionalAmount".into(),
            GraphPropertyValue::Decimal(self.notional_amount),
        );
        p.insert(
            "currency".into(),
            GraphPropertyValue::String(self.currency.clone()),
        );
        if let Some(ref cp) = self.currency_pair {
            p.insert(
                "currencyPair".into(),
                GraphPropertyValue::String(cp.clone()),
            );
        }
        p.insert(
            "tradeDate".into(),
            GraphPropertyValue::Date(self.trade_date),
        );
        p.insert(
            "maturityDate".into(),
            GraphPropertyValue::Date(self.maturity_date),
        );
        p.insert(
            "counterparty".into(),
            GraphPropertyValue::String(self.counterparty.clone()),
        );
        p.insert(
            "fairValue".into(),
            GraphPropertyValue::Decimal(self.fair_value),
        );
        p.insert(
            "status".into(),
            GraphPropertyValue::String(format!("{:?}", self.status)),
        );
        p
    }
}

impl ToNodeProperties for HedgeRelationship {
    fn node_type_name(&self) -> &'static str {
        "hedge_relationship"
    }
    fn node_type_code(&self) -> u16 {
        425
    }
    fn to_node_properties(&self) -> HashMap<String, GraphPropertyValue> {
        let mut p = HashMap::new();
        p.insert(
            "hedgedItemType".into(),
            GraphPropertyValue::String(format!("{:?}", self.hedged_item_type)),
        );
        p.insert(
            "hedgedItemDescription".into(),
            GraphPropertyValue::String(self.hedged_item_description.clone()),
        );
        p.insert(
            "instrumentId".into(),
            GraphPropertyValue::String(self.hedging_instrument_id.clone()),
        );
        p.insert(
            "hedgeType".into(),
            GraphPropertyValue::String(format!("{:?}", self.hedge_type)),
        );
        p.insert(
            "designationDate".into(),
            GraphPropertyValue::Date(self.designation_date),
        );
        p.insert(
            "effectivenessMethod".into(),
            GraphPropertyValue::String(format!("{:?}", self.effectiveness_test_method)),
        );
        p.insert(
            "effectivenessRatio".into(),
            GraphPropertyValue::Float(self.effectiveness_ratio.to_f64().unwrap_or(0.0)),
        );
        p.insert(
            "isEffective".into(),
            GraphPropertyValue::Bool(self.is_effective),
        );
        p.insert(
            "ineffectivenessAmount".into(),
            GraphPropertyValue::Decimal(self.ineffectiveness_amount),
        );
        p
    }
}

impl ToNodeProperties for DebtInstrument {
    fn node_type_name(&self) -> &'static str {
        "debt_instrument"
    }
    fn node_type_code(&self) -> u16 {
        426
    }
    fn to_node_properties(&self) -> HashMap<String, GraphPropertyValue> {
        let mut p = HashMap::new();
        p.insert(
            "entityCode".into(),
            GraphPropertyValue::String(self.entity_id.clone()),
        );
        p.insert(
            "instrumentType".into(),
            GraphPropertyValue::String(format!("{:?}", self.instrument_type)),
        );
        p.insert(
            "lender".into(),
            GraphPropertyValue::String(self.lender.clone()),
        );
        p.insert(
            "principal".into(),
            GraphPropertyValue::Decimal(self.principal),
        );
        p.insert(
            "currency".into(),
            GraphPropertyValue::String(self.currency.clone()),
        );
        p.insert(
            "interestRate".into(),
            GraphPropertyValue::Decimal(self.interest_rate),
        );
        p.insert(
            "rateType".into(),
            GraphPropertyValue::String(format!("{:?}", self.rate_type)),
        );
        p.insert(
            "originationDate".into(),
            GraphPropertyValue::Date(self.origination_date),
        );
        p.insert(
            "maturityDate".into(),
            GraphPropertyValue::Date(self.maturity_date),
        );
        p.insert(
            "drawnAmount".into(),
            GraphPropertyValue::Decimal(self.drawn_amount),
        );
        p.insert(
            "facilityLimit".into(),
            GraphPropertyValue::Decimal(self.facility_limit),
        );
        p.insert(
            "covenantCount".into(),
            GraphPropertyValue::Int(self.covenants.len() as i64),
        );
        p
    }
}

impl ToNodeProperties for DebtCovenant {
    fn node_type_name(&self) -> &'static str {
        "debt_covenant"
    }
    fn node_type_code(&self) -> u16 {
        427
    }
    fn to_node_properties(&self) -> HashMap<String, GraphPropertyValue> {
        let mut p = HashMap::new();
        p.insert(
            "covenantType".into(),
            GraphPropertyValue::String(format!("{:?}", self.covenant_type)),
        );
        p.insert(
            "threshold".into(),
            GraphPropertyValue::Decimal(self.threshold),
        );
        p.insert(
            "frequency".into(),
            GraphPropertyValue::String(format!("{:?}", self.measurement_frequency)),
        );
        p.insert(
            "actualValue".into(),
            GraphPropertyValue::Decimal(self.actual_value),
        );
        p.insert(
            "testDate".into(),
            GraphPropertyValue::Date(self.measurement_date),
        );
        p.insert(
            "complianceStatus".into(),
            GraphPropertyValue::Bool(self.is_compliant),
        );
        p.insert(
            "headroom".into(),
            GraphPropertyValue::Decimal(self.headroom),
        );
        p.insert(
            "waiverObtained".into(),
            GraphPropertyValue::Bool(self.waiver_obtained),
        );
        if let Some(ref fid) = self.facility_id {
            p.insert("facilityId".into(), GraphPropertyValue::String(fid.clone()));
        }
        if let Some(ref ec) = self.entity_code {
            p.insert("entityCode".into(), GraphPropertyValue::String(ec.clone()));
        }
        if let Some(ref fn_) = self.facility_name {
            p.insert(
                "facilityName".into(),
                GraphPropertyValue::String(fn_.clone()),
            );
        }
        if let Some(op) = self.outstanding_principal {
            p.insert(
                "outstandingPrincipal".into(),
                GraphPropertyValue::Decimal(op),
            );
        }
        if let Some(ref cur) = self.currency {
            p.insert("currency".into(), GraphPropertyValue::String(cur.clone()));
        }
        if let Some(ref per) = self.period {
            p.insert("period".into(), GraphPropertyValue::String(per.clone()));
        }
        p
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_cash_position_closing_balance() {
        let pos = CashPosition::new(
            "CP-001",
            "C001",
            "BA-001",
            "USD",
            NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            dec!(10000),
            dec!(5000),
            dec!(2000),
        );
        // closing = 10000 + 5000 - 2000 = 13000
        assert_eq!(pos.closing_balance, dec!(13000));
        assert_eq!(pos.computed_closing_balance(), dec!(13000));
        assert_eq!(pos.available_balance, dec!(13000)); // defaults to closing
    }

    #[test]
    fn test_cash_position_with_overrides() {
        let pos = CashPosition::new(
            "CP-002",
            "C001",
            "BA-001",
            "USD",
            NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            dec!(10000),
            dec!(5000),
            dec!(2000),
        )
        .with_available_balance(dec!(12000))
        .with_value_date_balance(dec!(12500));

        assert_eq!(pos.closing_balance, dec!(13000));
        assert_eq!(pos.available_balance, dec!(12000));
        assert_eq!(pos.value_date_balance, dec!(12500));
    }

    #[test]
    fn test_cash_forecast_net_position() {
        let items = vec![
            CashForecastItem {
                id: "CFI-001".to_string(),
                date: NaiveDate::from_ymd_opt(2025, 2, 1).unwrap(),
                category: TreasuryCashFlowCategory::ArCollection,
                amount: dec!(50000),
                probability: dec!(0.90),
                source_document_type: Some("SalesOrder".to_string()),
                source_document_id: Some("SO-001".to_string()),
            },
            CashForecastItem {
                id: "CFI-002".to_string(),
                date: NaiveDate::from_ymd_opt(2025, 2, 5).unwrap(),
                category: TreasuryCashFlowCategory::ApPayment,
                amount: dec!(-30000),
                probability: dec!(1.00),
                source_document_type: Some("PurchaseOrder".to_string()),
                source_document_id: Some("PO-001".to_string()),
            },
            CashForecastItem {
                id: "CFI-003".to_string(),
                date: NaiveDate::from_ymd_opt(2025, 2, 15).unwrap(),
                category: TreasuryCashFlowCategory::TaxPayment,
                amount: dec!(-10000),
                probability: dec!(1.00),
                source_document_type: None,
                source_document_id: None,
            },
        ];
        let forecast = CashForecast::new(
            "CF-001",
            "C001",
            "USD",
            NaiveDate::from_ymd_opt(2025, 1, 31).unwrap(),
            30,
            items,
            dec!(0.90),
        );

        // net = (50000 * 0.90) + (-30000 * 1.00) + (-10000 * 1.00)
        //     = 45000 - 30000 - 10000 = 5000
        assert_eq!(forecast.net_position, dec!(5000));
        assert_eq!(forecast.computed_net_position(), dec!(5000));
        assert_eq!(forecast.items.len(), 3);
    }

    #[test]
    fn test_cash_pool_total_accounts() {
        let pool = CashPool::new(
            "POOL-001",
            "EUR Cash Pool",
            PoolType::ZeroBalancing,
            "BA-HEADER",
            NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
        )
        .with_participant("BA-001")
        .with_participant("BA-002")
        .with_participant("BA-003")
        .with_interest_rate_benefit(dec!(0.0025));

        assert_eq!(pool.total_accounts(), 4); // header + 3 participants
        assert_eq!(pool.interest_rate_benefit, dec!(0.0025));
        assert_eq!(pool.pool_type, PoolType::ZeroBalancing);
    }

    #[test]
    fn test_hedging_instrument_lifecycle() {
        let instr = HedgingInstrument::new(
            "HI-001",
            HedgeInstrumentType::FxForward,
            dec!(1000000),
            "EUR",
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 6, 30).unwrap(),
            "Deutsche Bank",
        )
        .with_currency_pair("EUR/USD")
        .with_fixed_rate(dec!(1.0850))
        .with_fair_value(dec!(15000));

        assert!(instr.is_active());
        assert_eq!(
            instr.remaining_tenor_days(NaiveDate::from_ymd_opt(2025, 3, 15).unwrap()),
            107 // 2025-03-15 to 2025-06-30
        );
        assert_eq!(instr.currency_pair, Some("EUR/USD".to_string()));
        assert_eq!(instr.fixed_rate, Some(dec!(1.0850)));

        // Terminate
        let terminated = instr.with_status(InstrumentStatus::Terminated);
        assert!(!terminated.is_active());
    }

    #[test]
    fn test_hedging_instrument_remaining_tenor_past_maturity() {
        let instr = HedgingInstrument::new(
            "HI-002",
            HedgeInstrumentType::InterestRateSwap,
            dec!(5000000),
            "USD",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            "JPMorgan",
        );

        // Past maturity → 0 days
        assert_eq!(
            instr.remaining_tenor_days(NaiveDate::from_ymd_opt(2025, 6, 1).unwrap()),
            0
        );
    }

    #[test]
    fn test_hedge_relationship_effectiveness() {
        // Effective: ratio = 0.95 (within 80-125%)
        let effective = HedgeRelationship::new(
            "HR-001",
            HedgedItemType::ForecastedTransaction,
            "Forecasted EUR revenue Q2 2025",
            "HI-001",
            HedgeType::CashFlowHedge,
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            EffectivenessMethod::Regression,
            dec!(0.95),
        );
        assert!(effective.is_effective);
        assert!(HedgeRelationship::check_effectiveness(dec!(0.80))); // boundary
        assert!(HedgeRelationship::check_effectiveness(dec!(1.25))); // boundary

        // Ineffective: ratio = 0.75 (below 80%)
        let ineffective = HedgeRelationship::new(
            "HR-002",
            HedgedItemType::FirmCommitment,
            "Committed USD purchase",
            "HI-002",
            HedgeType::FairValueHedge,
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            EffectivenessMethod::DollarOffset,
            dec!(0.75),
        )
        .with_ineffectiveness_amount(dec!(25000));
        assert!(!ineffective.is_effective);
        assert_eq!(ineffective.ineffectiveness_amount, dec!(25000));

        // Boundaries
        assert!(!HedgeRelationship::check_effectiveness(dec!(0.79)));
        assert!(!HedgeRelationship::check_effectiveness(dec!(1.26)));
    }

    #[test]
    fn test_debt_covenant_compliance() {
        // Maximum covenant (DebtToEbitda): actual 2.8 <= threshold 3.5 → compliant
        let compliant = DebtCovenant::new(
            "COV-001",
            CovenantType::DebtToEbitda,
            dec!(3.5),
            Frequency::Quarterly,
            dec!(2.8),
            NaiveDate::from_ymd_opt(2025, 3, 31).unwrap(),
        );
        assert!(compliant.is_compliant);
        assert_eq!(compliant.headroom, dec!(0.7)); // 3.5 - 2.8

        // Maximum covenant breached: actual 4.0 > threshold 3.5
        let breached = DebtCovenant::new(
            "COV-002",
            CovenantType::DebtToEbitda,
            dec!(3.5),
            Frequency::Quarterly,
            dec!(4.0),
            NaiveDate::from_ymd_opt(2025, 3, 31).unwrap(),
        );
        assert!(!breached.is_compliant);
        assert_eq!(breached.headroom, dec!(-0.5)); // negative = breach

        // Minimum covenant (InterestCoverage): actual 4.5 >= threshold 3.0 → compliant
        let min_compliant = DebtCovenant::new(
            "COV-003",
            CovenantType::InterestCoverage,
            dec!(3.0),
            Frequency::Quarterly,
            dec!(4.5),
            NaiveDate::from_ymd_opt(2025, 3, 31).unwrap(),
        );
        assert!(min_compliant.is_compliant);
        assert_eq!(min_compliant.headroom, dec!(1.5)); // 4.5 - 3.0

        // Minimum covenant breached: actual 2.5 < threshold 3.0
        let min_breached = DebtCovenant::new(
            "COV-004",
            CovenantType::InterestCoverage,
            dec!(3.0),
            Frequency::Quarterly,
            dec!(2.5),
            NaiveDate::from_ymd_opt(2025, 3, 31).unwrap(),
        );
        assert!(!min_breached.is_compliant);
        assert_eq!(min_breached.headroom, dec!(-0.5));

        // With waiver
        let waived = DebtCovenant::new(
            "COV-005",
            CovenantType::DebtToEquity,
            dec!(2.0),
            Frequency::Annual,
            dec!(2.5),
            NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
        )
        .with_waiver(true);
        assert!(!waived.is_compliant); // technically breached
        assert!(waived.waiver_obtained); // but waiver obtained
    }

    #[test]
    fn test_debt_instrument_amortization() {
        let schedule = vec![
            AmortizationPayment {
                date: NaiveDate::from_ymd_opt(2025, 3, 31).unwrap(),
                principal_payment: dec!(250000),
                interest_payment: dec!(68750),
                balance_after: dec!(4750000),
            },
            AmortizationPayment {
                date: NaiveDate::from_ymd_opt(2025, 6, 30).unwrap(),
                principal_payment: dec!(250000),
                interest_payment: dec!(65312.50),
                balance_after: dec!(4500000),
            },
            AmortizationPayment {
                date: NaiveDate::from_ymd_opt(2025, 9, 30).unwrap(),
                principal_payment: dec!(250000),
                interest_payment: dec!(61875),
                balance_after: dec!(4250000),
            },
            AmortizationPayment {
                date: NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
                principal_payment: dec!(250000),
                interest_payment: dec!(58437.50),
                balance_after: dec!(4000000),
            },
        ];

        let debt = DebtInstrument::new(
            "DEBT-001",
            "C001",
            DebtType::TermLoan,
            "First National Bank",
            dec!(5000000),
            "USD",
            dec!(0.055),
            InterestRateType::Fixed,
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2030, 1, 1).unwrap(),
        )
        .with_amortization_schedule(schedule);

        assert_eq!(debt.total_principal_payments(), dec!(1000000));
        assert_eq!(debt.total_interest_payments(), dec!(254375));
        assert_eq!(debt.amortization_schedule[0].total_payment(), dec!(318750));
    }

    #[test]
    fn test_debt_instrument_revolving_credit() {
        let revolver = DebtInstrument::new(
            "DEBT-002",
            "C001",
            DebtType::RevolvingCredit,
            "Wells Fargo",
            dec!(0),
            "USD",
            dec!(0.045),
            InterestRateType::Variable,
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2028, 1, 1).unwrap(),
        )
        .with_drawn_amount(dec!(800000))
        .with_facility_limit(dec!(2000000));

        assert_eq!(revolver.available_capacity(), dec!(1200000));
    }

    #[test]
    fn test_debt_instrument_all_covenants_compliant() {
        let debt = DebtInstrument::new(
            "DEBT-003",
            "C001",
            DebtType::TermLoan,
            "Citibank",
            dec!(3000000),
            "USD",
            dec!(0.05),
            InterestRateType::Fixed,
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2030, 1, 1).unwrap(),
        )
        .with_covenant(DebtCovenant::new(
            "COV-A",
            CovenantType::DebtToEbitda,
            dec!(3.5),
            Frequency::Quarterly,
            dec!(2.5),
            NaiveDate::from_ymd_opt(2025, 3, 31).unwrap(),
        ))
        .with_covenant(DebtCovenant::new(
            "COV-B",
            CovenantType::InterestCoverage,
            dec!(3.0),
            Frequency::Quarterly,
            dec!(5.0),
            NaiveDate::from_ymd_opt(2025, 3, 31).unwrap(),
        ));

        assert!(debt.all_covenants_compliant());

        // Add a breached covenant with waiver
        let debt_waived = debt.with_covenant(
            DebtCovenant::new(
                "COV-C",
                CovenantType::CurrentRatio,
                dec!(1.5),
                Frequency::Quarterly,
                dec!(1.2), // breached
                NaiveDate::from_ymd_opt(2025, 3, 31).unwrap(),
            )
            .with_waiver(true),
        );
        assert!(debt_waived.all_covenants_compliant()); // waiver counts
    }

    #[test]
    fn test_bank_guarantee_active_check() {
        let guarantee = BankGuarantee::new(
            "BG-001",
            "C001",
            GuaranteeType::PerformanceBond,
            dec!(500000),
            "USD",
            "Construction Corp",
            "HSBC",
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
        )
        .with_linked_project("PROJ-001");

        // Active within range
        assert!(guarantee.is_active_on(NaiveDate::from_ymd_opt(2025, 6, 15).unwrap()));
        // Before issue
        assert!(!guarantee.is_active_on(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()));
        // After expiry
        assert!(!guarantee.is_active_on(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()));
        // On expiry (inclusive)
        assert!(guarantee.is_active_on(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()));

        // Remaining days
        assert_eq!(
            guarantee.remaining_days(NaiveDate::from_ymd_opt(2025, 6, 15).unwrap()),
            199
        );
        assert_eq!(
            guarantee.remaining_days(NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
            0 // past expiry
        );

        // Drawn status
        let drawn = BankGuarantee::new(
            "BG-002",
            "C001",
            GuaranteeType::StandbyLc,
            dec!(200000),
            "EUR",
            "Supplier GmbH",
            "Deutsche Bank",
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
        )
        .with_status(GuaranteeStatus::Drawn);
        assert!(!drawn.is_active_on(NaiveDate::from_ymd_opt(2025, 6, 15).unwrap()));
    }

    #[test]
    fn test_netting_run_savings() {
        let positions = vec![
            NettingPosition {
                entity_id: "C001".to_string(),
                gross_receivable: dec!(100000),
                gross_payable: dec!(60000),
                net_position: dec!(40000),
                settlement_direction: PayOrReceive::Receive,
            },
            NettingPosition {
                entity_id: "C002".to_string(),
                gross_receivable: dec!(80000),
                gross_payable: dec!(90000),
                net_position: dec!(-10000),
                settlement_direction: PayOrReceive::Pay,
            },
            NettingPosition {
                entity_id: "C003".to_string(),
                gross_receivable: dec!(50000),
                gross_payable: dec!(80000),
                net_position: dec!(-30000),
                settlement_direction: PayOrReceive::Pay,
            },
        ];

        let run = NettingRun::new(
            "NR-001",
            NaiveDate::from_ymd_opt(2025, 1, 31).unwrap(),
            NettingCycle::Monthly,
            "USD",
            positions,
        );

        assert_eq!(run.gross_receivables, dec!(230000));
        assert_eq!(run.gross_payables, dec!(230000));
        // net_settlement = sum(|net_position|) / 2 = (40000 + 10000 + 30000) / 2 = 40000
        assert_eq!(run.net_settlement, dec!(40000));
        // savings = max(230000, 230000) - 40000 = 190000
        assert_eq!(run.savings(), dec!(190000));
        assert_eq!(run.participating_entities.len(), 3);
    }

    #[test]
    fn test_netting_run_savings_pct() {
        let positions = vec![
            NettingPosition {
                entity_id: "C001".to_string(),
                gross_receivable: dec!(100000),
                gross_payable: dec!(0),
                net_position: dec!(100000),
                settlement_direction: PayOrReceive::Receive,
            },
            NettingPosition {
                entity_id: "C002".to_string(),
                gross_receivable: dec!(0),
                gross_payable: dec!(100000),
                net_position: dec!(-100000),
                settlement_direction: PayOrReceive::Pay,
            },
        ];

        let run = NettingRun::new(
            "NR-002",
            NaiveDate::from_ymd_opt(2025, 2, 28).unwrap(),
            NettingCycle::Monthly,
            "EUR",
            positions,
        );

        // No savings when perfectly bilateral
        assert_eq!(run.net_settlement, dec!(100000));
        assert_eq!(run.savings(), dec!(0));
        assert_eq!(run.savings_pct(), dec!(0));
    }

    #[test]
    fn test_cash_pool_sweep() {
        let sweep = CashPoolSweep {
            id: "SWP-001".to_string(),
            pool_id: "POOL-001".to_string(),
            date: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            from_account_id: "BA-001".to_string(),
            to_account_id: "BA-HEADER".to_string(),
            amount: dec!(50000),
            currency: "EUR".to_string(),
        };

        assert_eq!(sweep.amount, dec!(50000));
        assert_eq!(sweep.pool_id, "POOL-001");
    }

    #[test]
    fn test_serde_roundtrip_cash_position() {
        let pos = CashPosition::new(
            "CP-SERDE",
            "C001",
            "BA-001",
            "USD",
            NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            dec!(10000.50),
            dec!(5000.25),
            dec!(2000.75),
        );

        let json = serde_json::to_string_pretty(&pos).unwrap();
        let deserialized: CashPosition = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.opening_balance, pos.opening_balance);
        assert_eq!(deserialized.closing_balance, pos.closing_balance);
        assert_eq!(deserialized.date, pos.date);
    }

    #[test]
    fn test_serde_roundtrip_hedging_instrument() {
        let instr = HedgingInstrument::new(
            "HI-SERDE",
            HedgeInstrumentType::InterestRateSwap,
            dec!(5000000),
            "USD",
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2030, 1, 1).unwrap(),
            "JPMorgan",
        )
        .with_fixed_rate(dec!(0.0425))
        .with_floating_index("SOFR")
        .with_fair_value(dec!(-35000));

        let json = serde_json::to_string_pretty(&instr).unwrap();
        let deserialized: HedgingInstrument = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.fixed_rate, Some(dec!(0.0425)));
        assert_eq!(deserialized.floating_index, Some("SOFR".to_string()));
        assert_eq!(deserialized.strike_rate, None);
        assert_eq!(deserialized.fair_value, dec!(-35000));
    }
}
