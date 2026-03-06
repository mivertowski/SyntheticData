//! Intercompany transaction types and matched pairs.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Types of intercompany transactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ICTransactionType {
    /// Sale of goods between entities.
    #[default]
    GoodsSale,
    /// Services provided between entities.
    ServiceProvided,
    /// Intercompany loan.
    Loan,
    /// Loan interest.
    LoanInterest,
    /// Dividend distribution.
    Dividend,
    /// Management fee.
    ManagementFee,
    /// Royalty payment.
    Royalty,
    /// Cost sharing/allocation.
    CostSharing,
    /// Capital contribution.
    CapitalContribution,
    /// Recharge of expenses.
    ExpenseRecharge,
    /// Technical assistance fee.
    TechnicalAssistance,
    /// Guarantee fee.
    GuaranteeFee,
    /// License fee.
    LicenseFee,
    /// Research and development cost sharing.
    RDCostSharing,
}

impl ICTransactionType {
    /// Get the typical account categories for seller side.
    pub fn seller_accounts(&self) -> (&'static str, &'static str) {
        match self {
            Self::GoodsSale => ("IC Receivable", "IC Revenue - Goods"),
            Self::ServiceProvided => ("IC Receivable", "IC Revenue - Services"),
            Self::Loan => ("IC Loan Receivable", "Cash"),
            Self::LoanInterest => ("IC Receivable", "Interest Income"),
            Self::Dividend => ("Cash", "Dividend Income"),
            Self::ManagementFee => ("IC Receivable", "Management Fee Income"),
            Self::Royalty => ("IC Receivable", "Royalty Income"),
            Self::CostSharing => ("IC Receivable", "Cost Recovery"),
            Self::CapitalContribution => ("Investment in Subsidiary", "Cash"),
            Self::ExpenseRecharge => ("IC Receivable", "Cost Recovery"),
            Self::TechnicalAssistance => ("IC Receivable", "Technical Fee Income"),
            Self::GuaranteeFee => ("IC Receivable", "Guarantee Fee Income"),
            Self::LicenseFee => ("IC Receivable", "License Fee Income"),
            Self::RDCostSharing => ("IC Receivable", "R&D Cost Recovery"),
        }
    }

    /// Get the typical account categories for buyer side.
    pub fn buyer_accounts(&self) -> (&'static str, &'static str) {
        match self {
            Self::GoodsSale => ("Inventory/COGS", "IC Payable"),
            Self::ServiceProvided => ("Service Expense", "IC Payable"),
            Self::Loan => ("Cash", "IC Loan Payable"),
            Self::LoanInterest => ("Interest Expense", "IC Payable"),
            Self::Dividend => ("Retained Earnings", "Dividend Payable"),
            Self::ManagementFee => ("Management Fee Expense", "IC Payable"),
            Self::Royalty => ("Royalty Expense", "IC Payable"),
            Self::CostSharing => ("Allocated Cost", "IC Payable"),
            Self::CapitalContribution => ("Cash", "Additional Paid-in Capital"),
            Self::ExpenseRecharge => ("Operating Expense", "IC Payable"),
            Self::TechnicalAssistance => ("Technical Fee Expense", "IC Payable"),
            Self::GuaranteeFee => ("Guarantee Fee Expense", "IC Payable"),
            Self::LicenseFee => ("License Fee Expense", "IC Payable"),
            Self::RDCostSharing => ("R&D Expense", "IC Payable"),
        }
    }

    /// Check if this transaction type affects profit/loss.
    pub fn affects_pnl(&self) -> bool {
        !matches!(
            self,
            Self::Loan | Self::CapitalContribution | Self::Dividend
        )
    }

    /// Check if this transaction type requires transfer pricing documentation.
    pub fn requires_transfer_pricing(&self) -> bool {
        matches!(
            self,
            Self::GoodsSale
                | Self::ServiceProvided
                | Self::ManagementFee
                | Self::Royalty
                | Self::LoanInterest
                | Self::TechnicalAssistance
                | Self::LicenseFee
                | Self::RDCostSharing
        )
    }

    /// Check if this is a recurring transaction type.
    pub fn is_recurring(&self) -> bool {
        matches!(
            self,
            Self::ManagementFee
                | Self::Royalty
                | Self::LoanInterest
                | Self::CostSharing
                | Self::GuaranteeFee
                | Self::LicenseFee
        )
    }

    /// Get the typical frequency for recurring transactions.
    pub fn typical_frequency(&self) -> Option<RecurringFrequency> {
        match self {
            Self::ManagementFee => Some(RecurringFrequency::Monthly),
            Self::Royalty => Some(RecurringFrequency::Quarterly),
            Self::LoanInterest => Some(RecurringFrequency::Monthly),
            Self::CostSharing => Some(RecurringFrequency::Monthly),
            Self::GuaranteeFee => Some(RecurringFrequency::Annually),
            Self::LicenseFee => Some(RecurringFrequency::Quarterly),
            _ => None,
        }
    }

    /// Check if withholding tax typically applies.
    pub fn has_withholding_tax(&self) -> bool {
        matches!(
            self,
            Self::Dividend
                | Self::Royalty
                | Self::LoanInterest
                | Self::ManagementFee
                | Self::TechnicalAssistance
                | Self::LicenseFee
        )
    }

    /// Get typical withholding tax rate (varies by jurisdiction).
    pub fn typical_withholding_rate(&self) -> Option<Decimal> {
        match self {
            Self::Dividend => Some(Decimal::new(15, 2)),      // 15%
            Self::Royalty => Some(Decimal::new(10, 2)),       // 10%
            Self::LoanInterest => Some(Decimal::new(10, 2)),  // 10%
            Self::ManagementFee => Some(Decimal::new(15, 2)), // 15%
            Self::TechnicalAssistance => Some(Decimal::new(15, 2)), // 15%
            Self::LicenseFee => Some(Decimal::new(10, 2)),    // 10%
            _ => None,
        }
    }
}

/// Frequency for recurring intercompany transactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecurringFrequency {
    /// Monthly recurring.
    Monthly,
    /// Quarterly recurring.
    Quarterly,
    /// Semi-annually.
    SemiAnnually,
    /// Annually.
    Annually,
}

/// A matched pair of intercompany journal entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ICMatchedPair {
    /// Unique IC reference number linking both sides.
    pub ic_reference: String,
    /// Transaction type.
    pub transaction_type: ICTransactionType,
    /// Seller company code.
    pub seller_company: String,
    /// Buyer company code.
    pub buyer_company: String,
    /// Transaction amount in transaction currency.
    pub amount: Decimal,
    /// Transaction currency.
    pub currency: String,
    /// Transaction date.
    pub transaction_date: NaiveDate,
    /// Posting date (may differ from transaction date).
    pub posting_date: NaiveDate,
    /// Seller entry document number.
    pub seller_document: String,
    /// Buyer entry document number.
    pub buyer_document: String,
    /// Description/reference text.
    pub description: String,
    /// Transfer pricing policy applied.
    pub transfer_pricing_policy: Option<String>,
    /// Withholding tax amount (if applicable).
    pub withholding_tax: Option<Decimal>,
    /// Settlement status.
    pub settlement_status: ICSettlementStatus,
    /// Settlement date (when the IC balance was cleared).
    pub settlement_date: Option<NaiveDate>,
    /// Netting reference (if settled via netting).
    pub netting_reference: Option<String>,
}

impl ICMatchedPair {
    /// Create a new IC matched pair.
    pub fn new(
        ic_reference: String,
        transaction_type: ICTransactionType,
        seller_company: String,
        buyer_company: String,
        amount: Decimal,
        currency: String,
        transaction_date: NaiveDate,
    ) -> Self {
        let description =
            format!("IC {transaction_type:?} from {seller_company} to {buyer_company}");

        Self {
            ic_reference,
            transaction_type,
            seller_company,
            buyer_company,
            amount,
            currency,
            transaction_date,
            posting_date: transaction_date,
            seller_document: String::new(),
            buyer_document: String::new(),
            description,
            transfer_pricing_policy: None,
            withholding_tax: None,
            settlement_status: ICSettlementStatus::Open,
            settlement_date: None,
            netting_reference: None,
        }
    }

    /// Calculate withholding tax if applicable.
    pub fn calculate_withholding_tax(&mut self) {
        if let Some(rate) = self.transaction_type.typical_withholding_rate() {
            self.withholding_tax = Some(self.amount * rate);
        }
    }

    /// Get the net amount after withholding tax.
    pub fn net_amount(&self) -> Decimal {
        self.amount - self.withholding_tax.unwrap_or(Decimal::ZERO)
    }

    /// Mark as settled.
    pub fn settle(&mut self, settlement_date: NaiveDate, netting_reference: Option<String>) {
        self.settlement_status = if netting_reference.is_some() {
            ICSettlementStatus::SettledViaNettin
        } else {
            ICSettlementStatus::SettledViaCash
        };
        self.settlement_date = Some(settlement_date);
        self.netting_reference = netting_reference;
    }

    /// Check if the transaction is still open.
    pub fn is_open(&self) -> bool {
        self.settlement_status == ICSettlementStatus::Open
    }
}

/// Settlement status for IC transactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ICSettlementStatus {
    /// Open/unsettled.
    #[default]
    Open,
    /// Partially settled.
    PartiallySettled,
    /// Settled via cash payment.
    SettledViaCash,
    /// Settled via intercompany netting.
    SettledViaNettin,
    /// Written off.
    WrittenOff,
}

/// Intercompany netting arrangement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ICNettingArrangement {
    /// Netting reference number.
    pub netting_reference: String,
    /// Netting center company (if applicable).
    pub netting_center: Option<String>,
    /// Companies included in netting.
    pub participating_companies: Vec<String>,
    /// Netting period start date.
    pub period_start: NaiveDate,
    /// Netting period end date.
    pub period_end: NaiveDate,
    /// Settlement date.
    pub settlement_date: NaiveDate,
    /// Currency for netting settlement.
    pub settlement_currency: String,
    /// Individual company positions before netting.
    pub gross_positions: Vec<ICNettingPosition>,
    /// Net positions after netting.
    pub net_positions: Vec<ICNettingPosition>,
    /// Total gross receivables.
    pub total_gross_receivables: Decimal,
    /// Total gross payables.
    pub total_gross_payables: Decimal,
    /// Net settlement amount.
    pub net_settlement_amount: Decimal,
    /// Netting efficiency (percentage reduction).
    pub netting_efficiency: Decimal,
}

impl ICNettingArrangement {
    /// Create a new netting arrangement.
    pub fn new(
        netting_reference: String,
        participating_companies: Vec<String>,
        period_start: NaiveDate,
        period_end: NaiveDate,
        settlement_date: NaiveDate,
        settlement_currency: String,
    ) -> Self {
        Self {
            netting_reference,
            netting_center: None,
            participating_companies,
            period_start,
            period_end,
            settlement_date,
            settlement_currency,
            gross_positions: Vec::new(),
            net_positions: Vec::new(),
            total_gross_receivables: Decimal::ZERO,
            total_gross_payables: Decimal::ZERO,
            net_settlement_amount: Decimal::ZERO,
            netting_efficiency: Decimal::ZERO,
        }
    }

    /// Calculate netting efficiency.
    pub fn calculate_efficiency(&mut self) {
        let total_gross = self.total_gross_receivables + self.total_gross_payables;
        if total_gross > Decimal::ZERO {
            let reduction = total_gross - self.net_settlement_amount.abs() * Decimal::from(2);
            self.netting_efficiency = reduction / total_gross * Decimal::from(100);
        }
    }
}

/// Individual company position in IC netting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ICNettingPosition {
    /// Company code.
    pub company: String,
    /// Gross receivables.
    pub gross_receivables: Decimal,
    /// Gross payables.
    pub gross_payables: Decimal,
    /// Net position (positive = net receiver).
    pub net_position: Decimal,
    /// Currency.
    pub currency: String,
}

impl ICNettingPosition {
    /// Create a new netting position.
    pub fn new(company: String, currency: String) -> Self {
        Self {
            company,
            gross_receivables: Decimal::ZERO,
            gross_payables: Decimal::ZERO,
            net_position: Decimal::ZERO,
            currency,
        }
    }

    /// Add a receivable to this position.
    pub fn add_receivable(&mut self, amount: Decimal) {
        self.gross_receivables += amount;
        self.net_position = self.gross_receivables - self.gross_payables;
    }

    /// Add a payable to this position.
    pub fn add_payable(&mut self, amount: Decimal) {
        self.gross_payables += amount;
        self.net_position = self.gross_receivables - self.gross_payables;
    }
}

/// Intercompany loan details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ICLoan {
    /// Loan identifier.
    pub loan_id: String,
    /// Lender company.
    pub lender_company: String,
    /// Borrower company.
    pub borrower_company: String,
    /// Principal amount.
    pub principal: Decimal,
    /// Outstanding balance.
    pub outstanding_balance: Decimal,
    /// Currency.
    pub currency: String,
    /// Interest rate (annual).
    pub interest_rate: Decimal,
    /// Interest calculation method.
    pub interest_method: InterestMethod,
    /// Loan start date.
    pub start_date: NaiveDate,
    /// Maturity date.
    pub maturity_date: NaiveDate,
    /// Repayment schedule.
    pub repayment_schedule: RepaymentSchedule,
    /// Interest payment frequency.
    pub interest_frequency: RecurringFrequency,
    /// Accrued interest.
    pub accrued_interest: Decimal,
    /// Last interest payment date.
    pub last_interest_date: Option<NaiveDate>,
    /// Is the loan subordinated?
    pub is_subordinated: bool,
    /// Transfer pricing documentation reference.
    pub transfer_pricing_doc: Option<String>,
}

impl ICLoan {
    /// Create a new intercompany loan.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        loan_id: String,
        lender_company: String,
        borrower_company: String,
        principal: Decimal,
        currency: String,
        interest_rate: Decimal,
        start_date: NaiveDate,
        maturity_date: NaiveDate,
    ) -> Self {
        Self {
            loan_id,
            lender_company,
            borrower_company,
            principal,
            outstanding_balance: principal,
            currency,
            interest_rate,
            interest_method: InterestMethod::SimpleInterest,
            start_date,
            maturity_date,
            repayment_schedule: RepaymentSchedule::BulletRepayment,
            interest_frequency: RecurringFrequency::Quarterly,
            accrued_interest: Decimal::ZERO,
            last_interest_date: Some(start_date),
            is_subordinated: false,
            transfer_pricing_doc: None,
        }
    }

    /// Calculate interest for a period.
    pub fn calculate_interest(&self, from_date: NaiveDate, to_date: NaiveDate) -> Decimal {
        let days = (to_date - from_date).num_days();
        match self.interest_method {
            InterestMethod::SimpleInterest => {
                self.outstanding_balance * self.interest_rate / Decimal::from(100)
                    * Decimal::from(days)
                    / Decimal::from(365)
            }
            InterestMethod::ActualActual => {
                self.outstanding_balance * self.interest_rate / Decimal::from(100)
                    * Decimal::from(days)
                    / Decimal::from(365)
            }
            InterestMethod::Actual360 => {
                self.outstanding_balance * self.interest_rate / Decimal::from(100)
                    * Decimal::from(days)
                    / Decimal::from(360)
            }
            InterestMethod::ThirtyThreeSixty => {
                // Simplified 30/360 calculation
                self.outstanding_balance * self.interest_rate / Decimal::from(100)
                    * Decimal::from(days)
                    / Decimal::from(360)
            }
        }
    }

    /// Accrue interest up to a date.
    pub fn accrue_interest(&mut self, to_date: NaiveDate) {
        if let Some(last_date) = self.last_interest_date {
            if to_date > last_date {
                self.accrued_interest += self.calculate_interest(last_date, to_date);
            }
        }
    }

    /// Record interest payment.
    pub fn pay_interest(&mut self, payment_date: NaiveDate) -> Decimal {
        let interest_paid = self.accrued_interest;
        self.accrued_interest = Decimal::ZERO;
        self.last_interest_date = Some(payment_date);
        interest_paid
    }

    /// Record principal repayment.
    pub fn repay_principal(&mut self, amount: Decimal) {
        self.outstanding_balance -= amount;
        if self.outstanding_balance < Decimal::ZERO {
            self.outstanding_balance = Decimal::ZERO;
        }
    }

    /// Check if the loan is fully repaid.
    pub fn is_repaid(&self) -> bool {
        self.outstanding_balance == Decimal::ZERO
    }
}

/// Interest calculation method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InterestMethod {
    /// Simple interest.
    #[default]
    SimpleInterest,
    /// Actual/Actual day count.
    ActualActual,
    /// Actual/360 day count.
    Actual360,
    /// 30/360 day count.
    ThirtyThreeSixty,
}

/// Loan repayment schedule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RepaymentSchedule {
    /// Single bullet repayment at maturity.
    #[default]
    BulletRepayment,
    /// Equal periodic installments.
    EqualInstallments,
    /// Equal principal payments.
    EqualPrincipal,
    /// Custom schedule.
    Custom,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_ic_transaction_type_accounts() {
        let (dr, cr) = ICTransactionType::GoodsSale.seller_accounts();
        assert_eq!(dr, "IC Receivable");
        assert_eq!(cr, "IC Revenue - Goods");

        let (dr, cr) = ICTransactionType::GoodsSale.buyer_accounts();
        assert_eq!(dr, "Inventory/COGS");
        assert_eq!(cr, "IC Payable");
    }

    #[test]
    fn test_ic_matched_pair() {
        let mut pair = ICMatchedPair::new(
            "IC2022-001".to_string(),
            ICTransactionType::ManagementFee,
            "1000".to_string(),
            "1100".to_string(),
            dec!(50000),
            "USD".to_string(),
            NaiveDate::from_ymd_opt(2022, 6, 30).unwrap(),
        );

        assert!(pair.is_open());
        assert_eq!(pair.net_amount(), dec!(50000));

        pair.calculate_withholding_tax();
        assert_eq!(pair.withholding_tax, Some(dec!(7500))); // 15%
        assert_eq!(pair.net_amount(), dec!(42500));
    }

    #[test]
    fn test_ic_loan_interest() {
        let loan = ICLoan::new(
            "LOAN001".to_string(),
            "1000".to_string(),
            "1100".to_string(),
            dec!(1000000),
            "USD".to_string(),
            dec!(5), // 5% annual
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        );

        // Calculate interest for 90 days
        let interest = loan.calculate_interest(
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2022, 4, 1).unwrap(),
        );

        // Expected: 1,000,000 * 0.05 * 90/365 ≈ 12,328.77
        assert!(interest > dec!(12000) && interest < dec!(13000));
    }

    #[test]
    fn test_ic_netting_position() {
        let mut position = ICNettingPosition::new("1000".to_string(), "USD".to_string());

        position.add_receivable(dec!(100000));
        position.add_payable(dec!(60000));

        assert_eq!(position.gross_receivables, dec!(100000));
        assert_eq!(position.gross_payables, dec!(60000));
        assert_eq!(position.net_position, dec!(40000));
    }

    #[test]
    fn test_transaction_type_properties() {
        assert!(ICTransactionType::GoodsSale.affects_pnl());
        assert!(!ICTransactionType::Loan.affects_pnl());
        assert!(!ICTransactionType::CapitalContribution.affects_pnl());

        assert!(ICTransactionType::ManagementFee.requires_transfer_pricing());
        assert!(!ICTransactionType::Dividend.requires_transfer_pricing());

        assert!(ICTransactionType::Royalty.has_withholding_tax());
        assert!(!ICTransactionType::GoodsSale.has_withholding_tax());
    }
}
