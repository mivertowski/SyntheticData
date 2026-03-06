//! Common types shared across all subledgers.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Status of a subledger document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SubledgerDocumentStatus {
    /// Document is open/outstanding.
    #[default]
    Open,
    /// Document is partially cleared.
    PartiallyCleared,
    /// Document is fully cleared.
    Cleared,
    /// Document is reversed/cancelled.
    Reversed,
    /// Document is on hold.
    OnHold,
    /// Document is in dispute.
    InDispute,
}

/// Clearing information for a subledger document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearingInfo {
    /// Document that cleared this item.
    pub clearing_document: String,
    /// Date of clearing.
    pub clearing_date: NaiveDate,
    /// Amount cleared.
    pub clearing_amount: Decimal,
    /// Clearing type.
    pub clearing_type: ClearingType,
}

/// Type of clearing transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClearingType {
    /// Payment received/made.
    Payment,
    /// Credit/debit memo applied.
    Memo,
    /// Write-off.
    WriteOff,
    /// Netting between AR and AP.
    Netting,
    /// Manual clearing.
    Manual,
    /// Reversal.
    Reversal,
}

/// Reference to GL posting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GLReference {
    /// Journal entry ID.
    pub journal_entry_id: String,
    /// Posting date in GL.
    pub posting_date: NaiveDate,
    /// GL account code.
    pub gl_account: String,
    /// Amount posted to GL.
    pub amount: Decimal,
    /// Debit or credit indicator.
    pub debit_credit: DebitCredit,
}

/// Debit or credit indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebitCredit {
    Debit,
    Credit,
}

/// Tax information for a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxInfo {
    /// Tax code.
    pub tax_code: String,
    /// Tax rate percentage.
    pub tax_rate: Decimal,
    /// Tax base amount.
    pub tax_base: Decimal,
    /// Tax amount.
    pub tax_amount: Decimal,
    /// Tax jurisdiction.
    pub jurisdiction: Option<String>,
}

impl TaxInfo {
    /// Creates new tax info with calculated tax amount.
    pub fn new(tax_code: String, tax_rate: Decimal, tax_base: Decimal) -> Self {
        let tax_amount = (tax_base * tax_rate / dec!(100)).round_dp(2);
        Self {
            tax_code,
            tax_rate,
            tax_base,
            tax_amount,
            jurisdiction: None,
        }
    }

    /// Creates tax info with explicit tax amount.
    pub fn with_amount(
        tax_code: String,
        tax_rate: Decimal,
        tax_base: Decimal,
        tax_amount: Decimal,
    ) -> Self {
        Self {
            tax_code,
            tax_rate,
            tax_base,
            tax_amount,
            jurisdiction: None,
        }
    }

    /// Sets the tax jurisdiction.
    pub fn with_jurisdiction(mut self, jurisdiction: String) -> Self {
        self.jurisdiction = Some(jurisdiction);
        self
    }
}

/// Payment terms for AR/AP documents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentTerms {
    /// Terms code (e.g., "NET30", "2/10NET30").
    pub terms_code: String,
    /// Description.
    pub description: String,
    /// Net due days from baseline date.
    pub net_due_days: u32,
    /// Discount percentage if paid early.
    pub discount_percent: Option<Decimal>,
    /// Days to qualify for discount.
    pub discount_days: Option<u32>,
    /// Second discount tier percentage.
    pub discount_percent_2: Option<Decimal>,
    /// Days for second discount tier.
    pub discount_days_2: Option<u32>,
}

impl PaymentTerms {
    /// Creates standard net terms.
    pub fn net(days: u32) -> Self {
        Self {
            terms_code: format!("NET{days}"),
            description: format!("Net {days} days"),
            net_due_days: days,
            discount_percent: None,
            discount_days: None,
            discount_percent_2: None,
            discount_days_2: None,
        }
    }

    /// Creates terms with early payment discount.
    pub fn with_discount(net_days: u32, discount_percent: Decimal, discount_days: u32) -> Self {
        Self {
            terms_code: format!("{discount_percent}/{discount_days}NET{net_days}"),
            description: format!(
                "{discount_percent}% discount if paid within {discount_days} days, net {net_days} days"
            ),
            net_due_days: net_days,
            discount_percent: Some(discount_percent),
            discount_days: Some(discount_days),
            discount_percent_2: None,
            discount_days_2: None,
        }
    }

    /// Common payment terms.
    pub fn net_30() -> Self {
        Self::net(30)
    }

    pub fn net_60() -> Self {
        Self::net(60)
    }

    pub fn net_90() -> Self {
        Self::net(90)
    }

    pub fn two_ten_net_30() -> Self {
        Self::with_discount(30, dec!(2), 10)
    }

    pub fn one_ten_net_30() -> Self {
        Self::with_discount(30, dec!(1), 10)
    }

    /// Calculates due date from baseline date.
    pub fn calculate_due_date(&self, baseline_date: NaiveDate) -> NaiveDate {
        baseline_date + chrono::Duration::days(self.net_due_days as i64)
    }

    /// Calculates discount due date.
    pub fn calculate_discount_date(&self, baseline_date: NaiveDate) -> Option<NaiveDate> {
        self.discount_days
            .map(|days| baseline_date + chrono::Duration::days(days as i64))
    }

    /// Calculates discount amount for a given base amount.
    pub fn calculate_discount(
        &self,
        base_amount: Decimal,
        payment_date: NaiveDate,
        baseline_date: NaiveDate,
    ) -> Decimal {
        if let (Some(discount_percent), Some(discount_days)) =
            (self.discount_percent, self.discount_days)
        {
            let discount_deadline = baseline_date + chrono::Duration::days(discount_days as i64);
            if payment_date <= discount_deadline {
                return (base_amount * discount_percent / dec!(100)).round_dp(2);
            }
        }
        Decimal::ZERO
    }
}

/// Reconciliation status between subledger and GL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationStatus {
    /// Company code.
    pub company_code: String,
    /// GL control account.
    pub gl_account: String,
    /// Subledger type.
    pub subledger_type: SubledgerType,
    /// As-of date.
    pub as_of_date: NaiveDate,
    /// GL balance.
    pub gl_balance: Decimal,
    /// Subledger balance.
    pub subledger_balance: Decimal,
    /// Difference.
    pub difference: Decimal,
    /// Is reconciled (within tolerance).
    pub is_reconciled: bool,
    /// Reconciliation timestamp.
    pub reconciled_at: DateTime<Utc>,
    /// Unreconciled items.
    pub unreconciled_items: Vec<UnreconciledItem>,
}

impl ReconciliationStatus {
    /// Creates new reconciliation status.
    pub fn new(
        company_code: String,
        gl_account: String,
        subledger_type: SubledgerType,
        as_of_date: NaiveDate,
        gl_balance: Decimal,
        subledger_balance: Decimal,
        tolerance: Decimal,
    ) -> Self {
        let difference = gl_balance - subledger_balance;
        let is_reconciled = difference.abs() <= tolerance;

        Self {
            company_code,
            gl_account,
            subledger_type,
            as_of_date,
            gl_balance,
            subledger_balance,
            difference,
            is_reconciled,
            reconciled_at: Utc::now(),
            unreconciled_items: Vec::new(),
        }
    }

    /// Adds an unreconciled item.
    pub fn add_unreconciled_item(&mut self, item: UnreconciledItem) {
        self.unreconciled_items.push(item);
    }
}

/// Type of subledger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubledgerType {
    /// Accounts Receivable.
    AR,
    /// Accounts Payable.
    AP,
    /// Fixed Assets.
    FA,
    /// Inventory.
    Inventory,
}

/// An item that doesn't reconcile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnreconciledItem {
    /// Document number.
    pub document_number: String,
    /// Document type.
    pub document_type: String,
    /// Amount in subledger.
    pub subledger_amount: Decimal,
    /// Amount in GL.
    pub gl_amount: Decimal,
    /// Difference.
    pub difference: Decimal,
    /// Reason for discrepancy.
    pub reason: Option<String>,
}

/// Currency amount with original and local currency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyAmount {
    /// Amount in document currency.
    pub document_amount: Decimal,
    /// Document currency code.
    pub document_currency: String,
    /// Amount in local currency.
    pub local_amount: Decimal,
    /// Local currency code.
    pub local_currency: String,
    /// Exchange rate used.
    pub exchange_rate: Decimal,
}

impl CurrencyAmount {
    /// Creates amount in single currency.
    pub fn single_currency(amount: Decimal, currency: String) -> Self {
        Self {
            document_amount: amount,
            document_currency: currency.clone(),
            local_amount: amount,
            local_currency: currency,
            exchange_rate: Decimal::ONE,
        }
    }

    /// Creates amount with currency conversion.
    pub fn with_conversion(
        document_amount: Decimal,
        document_currency: String,
        local_currency: String,
        exchange_rate: Decimal,
    ) -> Self {
        let local_amount = (document_amount * exchange_rate).round_dp(2);
        Self {
            document_amount,
            document_currency,
            local_amount,
            local_currency,
            exchange_rate,
        }
    }
}

/// Baseline date type for payment terms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BaselineDateType {
    /// Document date.
    #[default]
    DocumentDate,
    /// Posting date.
    PostingDate,
    /// Entry date.
    EntryDate,
    /// Goods receipt date.
    GoodsReceiptDate,
    /// Custom date.
    CustomDate,
}

/// Dunning information for AR items.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DunningInfo {
    /// Current dunning level (0 = not dunned).
    pub dunning_level: u8,
    /// Maximum dunning level reached.
    pub max_dunning_level: u8,
    /// Last dunning date.
    pub last_dunning_date: Option<NaiveDate>,
    /// Last dunning run ID.
    pub last_dunning_run: Option<String>,
    /// Is blocked for dunning.
    pub dunning_blocked: bool,
    /// Block reason.
    pub block_reason: Option<String>,
}

impl DunningInfo {
    /// Advances to next dunning level.
    pub fn advance_level(&mut self, dunning_date: NaiveDate, run_id: String) {
        if !self.dunning_blocked {
            self.dunning_level += 1;
            if self.dunning_level > self.max_dunning_level {
                self.max_dunning_level = self.dunning_level;
            }
            self.last_dunning_date = Some(dunning_date);
            self.last_dunning_run = Some(run_id);
        }
    }

    /// Blocks dunning.
    pub fn block(&mut self, reason: String) {
        self.dunning_blocked = true;
        self.block_reason = Some(reason);
    }

    /// Unblocks dunning.
    pub fn unblock(&mut self) {
        self.dunning_blocked = false;
        self.block_reason = None;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_terms_due_date() {
        let terms = PaymentTerms::net_30();
        let baseline = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let due_date = terms.calculate_due_date(baseline);
        assert_eq!(due_date, NaiveDate::from_ymd_opt(2024, 2, 14).unwrap());
    }

    #[test]
    fn test_payment_terms_discount() {
        let terms = PaymentTerms::two_ten_net_30();
        let baseline = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let amount = dec!(1000);

        // Payment within discount period
        let early_payment = NaiveDate::from_ymd_opt(2024, 1, 20).unwrap();
        let discount = terms.calculate_discount(amount, early_payment, baseline);
        assert_eq!(discount, dec!(20)); // 2% of 1000

        // Payment after discount period
        let late_payment = NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();
        let no_discount = terms.calculate_discount(amount, late_payment, baseline);
        assert_eq!(no_discount, Decimal::ZERO);
    }

    #[test]
    fn test_tax_info() {
        let tax = TaxInfo::new("VAT".to_string(), dec!(20), dec!(1000));
        assert_eq!(tax.tax_amount, dec!(200));
    }

    #[test]
    fn test_currency_conversion() {
        let amount = CurrencyAmount::with_conversion(
            dec!(1000),
            "EUR".to_string(),
            "USD".to_string(),
            dec!(1.10),
        );
        assert_eq!(amount.local_amount, dec!(1100));
    }
}
