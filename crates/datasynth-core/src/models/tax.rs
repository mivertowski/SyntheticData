//! Tax accounting data models.
//!
//! This module provides comprehensive tax data models including:
//! - Tax jurisdictions (federal, state, local, municipal, supranational)
//! - Tax codes with rates and effective date ranges
//! - Tax lines attached to source documents (invoices, JEs, payments)
//! - Tax returns (VAT, income tax, withholding remittance, payroll)
//! - Tax provisions under ASC 740 / IAS 12
//! - Uncertain tax positions under FIN 48 / IFRIC 23
//! - Withholding tax records with treaty benefit tracking

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Classification of a tax jurisdiction within a governmental hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum JurisdictionType {
    /// National / federal level (e.g., IRS, HMRC)
    #[default]
    Federal,
    /// State / province level
    State,
    /// County / city level
    Local,
    /// Municipal / district level
    Municipal,
    /// Supranational body (e.g., EU VAT)
    Supranational,
}

/// High-level classification of a tax.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaxType {
    /// Value Added Tax
    #[default]
    Vat,
    /// Goods and Services Tax
    Gst,
    /// Sales Tax (destination-based)
    SalesTax,
    /// Corporate / individual income tax
    IncomeTax,
    /// Withholding tax on cross-border payments
    WithholdingTax,
    /// Employer / employee payroll taxes
    PayrollTax,
    /// Excise / duty taxes
    ExciseTax,
}

/// The type of source document a tax line is attached to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaxableDocumentType {
    /// Accounts-payable vendor invoice
    #[default]
    VendorInvoice,
    /// Accounts-receivable customer invoice
    CustomerInvoice,
    /// Manual journal entry
    JournalEntry,
    /// Cash disbursement / receipt
    Payment,
    /// Payroll run
    PayrollRun,
}

/// Type of periodic tax return filed with an authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaxReturnType {
    /// VAT / GST return
    #[default]
    VatReturn,
    /// Corporate income tax return
    IncomeTax,
    /// Withholding tax remittance
    WithholdingRemittance,
    /// Payroll tax return
    PayrollTax,
}

/// Lifecycle status of a tax return.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaxReturnStatus {
    /// Return is being prepared
    #[default]
    Draft,
    /// Return has been submitted to the authority
    Filed,
    /// Authority has reviewed and issued assessment
    Assessed,
    /// Tax liability has been settled
    Paid,
    /// An amendment has been filed
    Amended,
}

/// Category of withholding tax applied to a payment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WithholdingType {
    /// Withholding on dividend distributions
    DividendWithholding,
    /// Withholding on royalty payments
    RoyaltyWithholding,
    /// Withholding on service fees
    #[default]
    ServiceWithholding,
}

/// Measurement method for uncertain tax positions (FIN 48 / IFRIC 23).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaxMeasurementMethod {
    /// Single most-likely outcome
    #[default]
    MostLikelyAmount,
    /// Probability-weighted expected value
    ExpectedValue,
}

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

/// A taxing authority at a specific level of government.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxJurisdiction {
    /// Unique jurisdiction identifier
    pub id: String,
    /// Human-readable name (e.g., "United States - Federal")
    pub name: String,
    /// ISO 3166-1 alpha-2 country code
    pub country_code: String,
    /// State / province / region code (ISO 3166-2 subdivision)
    pub region_code: Option<String>,
    /// Tier within the governmental hierarchy
    pub jurisdiction_type: JurisdictionType,
    /// Parent jurisdiction (e.g., state's parent is the federal jurisdiction)
    pub parent_jurisdiction_id: Option<String>,
    /// Whether the entity is VAT-registered in this jurisdiction
    pub vat_registered: bool,
}

impl TaxJurisdiction {
    /// Creates a new tax jurisdiction.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        country_code: impl Into<String>,
        jurisdiction_type: JurisdictionType,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            country_code: country_code.into(),
            region_code: None,
            jurisdiction_type,
            parent_jurisdiction_id: None,
            vat_registered: false,
        }
    }

    /// Sets the region code.
    pub fn with_region_code(mut self, region_code: impl Into<String>) -> Self {
        self.region_code = Some(region_code.into());
        self
    }

    /// Sets the parent jurisdiction ID.
    pub fn with_parent_jurisdiction_id(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_jurisdiction_id = Some(parent_id.into());
        self
    }

    /// Sets the VAT registration flag.
    pub fn with_vat_registered(mut self, registered: bool) -> Self {
        self.vat_registered = registered;
        self
    }

    /// Returns `true` if the jurisdiction is sub-national (state, local, or municipal).
    pub fn is_subnational(&self) -> bool {
        matches!(
            self.jurisdiction_type,
            JurisdictionType::State | JurisdictionType::Local | JurisdictionType::Municipal
        )
    }
}

/// A tax code defining a rate for a specific tax type and jurisdiction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxCode {
    /// Unique tax code identifier
    pub id: String,
    /// Short mnemonic (e.g., "VAT-STD-20", "WHT-SVC-15")
    pub code: String,
    /// Human-readable description
    pub description: String,
    /// Category of tax
    pub tax_type: TaxType,
    /// Tax rate as a decimal fraction (e.g., 0.20 for 20%)
    #[serde(with = "rust_decimal::serde::str")]
    pub rate: Decimal,
    /// Jurisdiction this code applies to
    pub jurisdiction_id: String,
    /// Date from which the code is effective (inclusive)
    pub effective_date: NaiveDate,
    /// Date after which the code is no longer effective (exclusive)
    pub expiry_date: Option<NaiveDate>,
    /// Whether the reverse-charge mechanism applies
    pub is_reverse_charge: bool,
    /// Whether transactions under this code are tax-exempt
    pub is_exempt: bool,
}

impl TaxCode {
    /// Creates a new tax code.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        code: impl Into<String>,
        description: impl Into<String>,
        tax_type: TaxType,
        rate: Decimal,
        jurisdiction_id: impl Into<String>,
        effective_date: NaiveDate,
    ) -> Self {
        Self {
            id: id.into(),
            code: code.into(),
            description: description.into(),
            tax_type,
            rate,
            jurisdiction_id: jurisdiction_id.into(),
            effective_date,
            expiry_date: None,
            is_reverse_charge: false,
            is_exempt: false,
        }
    }

    /// Sets the expiry date.
    pub fn with_expiry_date(mut self, expiry: NaiveDate) -> Self {
        self.expiry_date = Some(expiry);
        self
    }

    /// Sets the reverse-charge flag.
    pub fn with_reverse_charge(mut self, reverse_charge: bool) -> Self {
        self.is_reverse_charge = reverse_charge;
        self
    }

    /// Sets the exempt flag.
    pub fn with_exempt(mut self, exempt: bool) -> Self {
        self.is_exempt = exempt;
        self
    }

    /// Computes the tax amount for a given taxable base.
    ///
    /// Returns `taxable_amount * rate`, rounded to 2 decimal places.
    /// Exempt codes always return zero.
    pub fn tax_amount(&self, taxable_amount: Decimal) -> Decimal {
        if self.is_exempt {
            return Decimal::ZERO;
        }
        (taxable_amount * self.rate).round_dp(2)
    }

    /// Returns `true` if the tax code is active on the given `date`.
    ///
    /// A code is active when `effective_date <= date` and either no expiry
    /// is set or `date < expiry_date`.
    pub fn is_active(&self, date: NaiveDate) -> bool {
        if date < self.effective_date {
            return false;
        }
        match self.expiry_date {
            Some(expiry) => date < expiry,
            None => true,
        }
    }
}

/// A single tax line attached to a source document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxLine {
    /// Unique tax line identifier
    pub id: String,
    /// Type of the source document
    pub document_type: TaxableDocumentType,
    /// Source document identifier
    pub document_id: String,
    /// Line number within the document
    pub line_number: u32,
    /// Tax code applied
    pub tax_code_id: String,
    /// Jurisdiction the tax is assessed in
    pub jurisdiction_id: String,
    /// Base amount subject to tax
    #[serde(with = "rust_decimal::serde::str")]
    pub taxable_amount: Decimal,
    /// Computed tax amount
    #[serde(with = "rust_decimal::serde::str")]
    pub tax_amount: Decimal,
    /// Whether the input tax is deductible (reclaimable)
    pub is_deductible: bool,
    /// Whether the reverse-charge mechanism applies
    pub is_reverse_charge: bool,
    /// Whether the tax was self-assessed by the buyer
    pub is_self_assessed: bool,
}

impl TaxLine {
    /// Creates a new tax line.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        document_type: TaxableDocumentType,
        document_id: impl Into<String>,
        line_number: u32,
        tax_code_id: impl Into<String>,
        jurisdiction_id: impl Into<String>,
        taxable_amount: Decimal,
        tax_amount: Decimal,
    ) -> Self {
        Self {
            id: id.into(),
            document_type,
            document_id: document_id.into(),
            line_number,
            tax_code_id: tax_code_id.into(),
            jurisdiction_id: jurisdiction_id.into(),
            taxable_amount,
            tax_amount,
            is_deductible: true,
            is_reverse_charge: false,
            is_self_assessed: false,
        }
    }

    /// Sets the deductible flag.
    pub fn with_deductible(mut self, deductible: bool) -> Self {
        self.is_deductible = deductible;
        self
    }

    /// Sets the reverse-charge flag.
    pub fn with_reverse_charge(mut self, reverse_charge: bool) -> Self {
        self.is_reverse_charge = reverse_charge;
        self
    }

    /// Sets the self-assessed flag.
    pub fn with_self_assessed(mut self, self_assessed: bool) -> Self {
        self.is_self_assessed = self_assessed;
        self
    }

    /// Computes the effective tax rate for this line.
    ///
    /// Returns `tax_amount / taxable_amount`, or `Decimal::ZERO` when the
    /// taxable amount is zero (avoids division by zero).
    pub fn effective_rate(&self) -> Decimal {
        if self.taxable_amount.is_zero() {
            Decimal::ZERO
        } else {
            (self.tax_amount / self.taxable_amount).round_dp(6)
        }
    }
}

/// A periodic tax return filed with a jurisdiction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxReturn {
    /// Unique return identifier
    pub id: String,
    /// Legal entity filing the return
    pub entity_id: String,
    /// Jurisdiction the return is filed with
    pub jurisdiction_id: String,
    /// Start of the reporting period
    pub period_start: NaiveDate,
    /// End of the reporting period
    pub period_end: NaiveDate,
    /// Type of return
    pub return_type: TaxReturnType,
    /// Current lifecycle status
    pub status: TaxReturnStatus,
    /// Total output tax (tax collected / charged)
    #[serde(with = "rust_decimal::serde::str")]
    pub total_output_tax: Decimal,
    /// Total input tax (tax paid / reclaimable)
    #[serde(with = "rust_decimal::serde::str")]
    pub total_input_tax: Decimal,
    /// Net amount payable to the authority (output - input)
    #[serde(with = "rust_decimal::serde::str")]
    pub net_payable: Decimal,
    /// Statutory filing deadline
    pub filing_deadline: NaiveDate,
    /// Actual date the return was submitted
    pub actual_filing_date: Option<NaiveDate>,
    /// Whether the return was filed after the deadline
    pub is_late: bool,
}

impl TaxReturn {
    /// Creates a new tax return.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        entity_id: impl Into<String>,
        jurisdiction_id: impl Into<String>,
        period_start: NaiveDate,
        period_end: NaiveDate,
        return_type: TaxReturnType,
        total_output_tax: Decimal,
        total_input_tax: Decimal,
        filing_deadline: NaiveDate,
    ) -> Self {
        let net_payable = (total_output_tax - total_input_tax).round_dp(2);
        Self {
            id: id.into(),
            entity_id: entity_id.into(),
            jurisdiction_id: jurisdiction_id.into(),
            period_start,
            period_end,
            return_type,
            status: TaxReturnStatus::Draft,
            total_output_tax,
            total_input_tax,
            net_payable,
            filing_deadline,
            actual_filing_date: None,
            is_late: false,
        }
    }

    /// Records the actual filing date and derives lateness.
    pub fn with_filing(mut self, filing_date: NaiveDate) -> Self {
        self.actual_filing_date = Some(filing_date);
        self.is_late = filing_date > self.filing_deadline;
        self.status = TaxReturnStatus::Filed;
        self
    }

    /// Sets the return status.
    pub fn with_status(mut self, status: TaxReturnStatus) -> Self {
        self.status = status;
        self
    }

    /// Returns `true` if the return has been submitted (Filed, Assessed, or Paid).
    pub fn is_filed(&self) -> bool {
        matches!(
            self.status,
            TaxReturnStatus::Filed | TaxReturnStatus::Assessed | TaxReturnStatus::Paid
        )
    }
}

/// An item in the statutory-to-effective rate reconciliation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateReconciliationItem {
    /// Description of the reconciling item (e.g., "State taxes", "R&D credits")
    pub description: String,
    /// Impact on the effective rate (positive increases, negative decreases)
    #[serde(with = "rust_decimal::serde::str")]
    pub rate_impact: Decimal,
}

/// Income tax provision computed under ASC 740 / IAS 12.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxProvision {
    /// Unique provision identifier
    pub id: String,
    /// Legal entity the provision relates to
    pub entity_id: String,
    /// Period end date
    pub period: NaiveDate,
    /// Current period income tax expense
    #[serde(with = "rust_decimal::serde::str")]
    pub current_tax_expense: Decimal,
    /// Deferred tax asset balance
    #[serde(with = "rust_decimal::serde::str")]
    pub deferred_tax_asset: Decimal,
    /// Deferred tax liability balance
    #[serde(with = "rust_decimal::serde::str")]
    pub deferred_tax_liability: Decimal,
    /// Statutory tax rate
    #[serde(with = "rust_decimal::serde::str")]
    pub statutory_rate: Decimal,
    /// Effective tax rate after permanent and temporary differences
    #[serde(with = "rust_decimal::serde::str")]
    pub effective_rate: Decimal,
    /// Rate reconciliation from statutory to effective rate
    pub rate_reconciliation: Vec<RateReconciliationItem>,
}

impl TaxProvision {
    /// Creates a new tax provision.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        entity_id: impl Into<String>,
        period: NaiveDate,
        current_tax_expense: Decimal,
        deferred_tax_asset: Decimal,
        deferred_tax_liability: Decimal,
        statutory_rate: Decimal,
        effective_rate: Decimal,
    ) -> Self {
        Self {
            id: id.into(),
            entity_id: entity_id.into(),
            period,
            current_tax_expense,
            deferred_tax_asset,
            deferred_tax_liability,
            statutory_rate,
            effective_rate,
            rate_reconciliation: Vec::new(),
        }
    }

    /// Adds a rate reconciliation item.
    pub fn with_reconciliation_item(
        mut self,
        description: impl Into<String>,
        rate_impact: Decimal,
    ) -> Self {
        self.rate_reconciliation.push(RateReconciliationItem {
            description: description.into(),
            rate_impact,
        });
        self
    }

    /// Computes the net deferred tax position.
    ///
    /// Positive value indicates a net deferred tax asset; negative indicates
    /// a net deferred tax liability.
    pub fn net_deferred_tax(&self) -> Decimal {
        self.deferred_tax_asset - self.deferred_tax_liability
    }
}

/// An uncertain tax position evaluated under FIN 48 / IFRIC 23.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncertainTaxPosition {
    /// Unique UTP identifier
    pub id: String,
    /// Legal entity
    pub entity_id: String,
    /// Description of the tax position
    pub description: String,
    /// Total gross tax benefit claimed
    #[serde(with = "rust_decimal::serde::str")]
    pub tax_benefit: Decimal,
    /// Recognition threshold (typically 0.50 for "more-likely-than-not")
    #[serde(with = "rust_decimal::serde::str")]
    pub recognition_threshold: Decimal,
    /// Amount recognized in the financial statements
    #[serde(with = "rust_decimal::serde::str")]
    pub recognized_amount: Decimal,
    /// Measurement method used to determine the recognized amount
    pub measurement_method: TaxMeasurementMethod,
}

impl UncertainTaxPosition {
    /// Creates a new uncertain tax position.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        entity_id: impl Into<String>,
        description: impl Into<String>,
        tax_benefit: Decimal,
        recognition_threshold: Decimal,
        recognized_amount: Decimal,
        measurement_method: TaxMeasurementMethod,
    ) -> Self {
        Self {
            id: id.into(),
            entity_id: entity_id.into(),
            description: description.into(),
            tax_benefit,
            recognition_threshold,
            recognized_amount,
            measurement_method,
        }
    }

    /// Returns the portion of the tax benefit that has **not** been recognized.
    pub fn unrecognized_amount(&self) -> Decimal {
        self.tax_benefit - self.recognized_amount
    }
}

/// A withholding tax record associated with a cross-border payment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithholdingTaxRecord {
    /// Unique record identifier
    pub id: String,
    /// Payment document this withholding relates to
    pub payment_id: String,
    /// Vendor / payee subject to withholding
    pub vendor_id: String,
    /// Category of withholding
    pub withholding_type: WithholdingType,
    /// Reduced rate under an applicable tax treaty
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub treaty_rate: Option<Decimal>,
    /// Domestic statutory withholding rate
    #[serde(with = "rust_decimal::serde::str")]
    pub statutory_rate: Decimal,
    /// Rate actually applied (may equal treaty_rate or statutory_rate)
    #[serde(with = "rust_decimal::serde::str")]
    pub applied_rate: Decimal,
    /// Gross payment amount subject to withholding
    #[serde(with = "rust_decimal::serde::str")]
    pub base_amount: Decimal,
    /// Amount withheld (base_amount * applied_rate)
    #[serde(with = "rust_decimal::serde::str")]
    pub withheld_amount: Decimal,
    /// Tax certificate / receipt number from the authority
    pub certificate_number: Option<String>,
}

impl WithholdingTaxRecord {
    /// Creates a new withholding tax record.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        payment_id: impl Into<String>,
        vendor_id: impl Into<String>,
        withholding_type: WithholdingType,
        statutory_rate: Decimal,
        applied_rate: Decimal,
        base_amount: Decimal,
    ) -> Self {
        let withheld_amount = (base_amount * applied_rate).round_dp(2);
        Self {
            id: id.into(),
            payment_id: payment_id.into(),
            vendor_id: vendor_id.into(),
            withholding_type,
            treaty_rate: None,
            statutory_rate,
            applied_rate,
            base_amount,
            withheld_amount,
            certificate_number: None,
        }
    }

    /// Sets the treaty rate.
    pub fn with_treaty_rate(mut self, rate: Decimal) -> Self {
        self.treaty_rate = Some(rate);
        self
    }

    /// Sets the certificate number.
    pub fn with_certificate_number(mut self, number: impl Into<String>) -> Self {
        self.certificate_number = Some(number.into());
        self
    }

    /// Returns `true` if a tax-treaty benefit has been applied.
    ///
    /// A treaty benefit exists when a treaty rate is present **and** the
    /// applied rate is strictly less than the statutory rate.
    pub fn has_treaty_benefit(&self) -> bool {
        self.treaty_rate.is_some() && self.applied_rate < self.statutory_rate
    }

    /// Computes the savings achieved through the treaty benefit.
    ///
    /// `(statutory_rate - applied_rate) * base_amount`, rounded to 2 dp.
    pub fn treaty_savings(&self) -> Decimal {
        ((self.statutory_rate - self.applied_rate) * self.base_amount).round_dp(2)
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
    fn test_tax_code_creation() {
        let code = TaxCode::new(
            "TC-001",
            "VAT-STD-20",
            "Standard VAT 20%",
            TaxType::Vat,
            dec!(0.20),
            "JUR-UK",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        )
        .with_expiry_date(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap());

        // tax_amount computation
        assert_eq!(code.tax_amount(dec!(1000.00)), dec!(200.00));
        assert_eq!(code.tax_amount(dec!(0)), dec!(0.00));

        // is_active within range
        assert!(code.is_active(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()));
        // before effective date
        assert!(!code.is_active(NaiveDate::from_ymd_opt(2023, 12, 31).unwrap()));
        // on expiry date (exclusive)
        assert!(!code.is_active(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()));
        // well after expiry
        assert!(!code.is_active(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()));
    }

    #[test]
    fn test_tax_code_exempt() {
        let code = TaxCode::new(
            "TC-002",
            "VAT-EX",
            "VAT Exempt",
            TaxType::Vat,
            dec!(0.20),
            "JUR-UK",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        )
        .with_exempt(true);

        assert_eq!(code.tax_amount(dec!(5000.00)), dec!(0));
    }

    #[test]
    fn test_tax_line_creation() {
        let line = TaxLine::new(
            "TL-001",
            TaxableDocumentType::VendorInvoice,
            "INV-001",
            1,
            "TC-001",
            "JUR-UK",
            dec!(1000.00),
            dec!(200.00),
        );

        assert_eq!(line.effective_rate(), dec!(0.200000));

        // Zero taxable amount should return zero rate
        let zero_line = TaxLine::new(
            "TL-002",
            TaxableDocumentType::VendorInvoice,
            "INV-002",
            1,
            "TC-001",
            "JUR-UK",
            dec!(0),
            dec!(0),
        );
        assert_eq!(zero_line.effective_rate(), dec!(0));
    }

    #[test]
    fn test_tax_return_net_payable() {
        let ret = TaxReturn::new(
            "TR-001",
            "ENT-001",
            "JUR-UK",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 3, 31).unwrap(),
            TaxReturnType::VatReturn,
            dec!(50000),
            dec!(30000),
            NaiveDate::from_ymd_opt(2024, 4, 30).unwrap(),
        );

        // Draft is not filed
        assert!(!ret.is_filed());
        assert_eq!(ret.net_payable, dec!(20000));

        // Filed
        let filed = ret.with_filing(NaiveDate::from_ymd_opt(2024, 4, 15).unwrap());
        assert!(filed.is_filed());
        assert!(!filed.is_late);

        // Assessed
        let assessed = TaxReturn::new(
            "TR-002",
            "ENT-001",
            "JUR-UK",
            NaiveDate::from_ymd_opt(2024, 4, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 6, 30).unwrap(),
            TaxReturnType::VatReturn,
            dec!(60000),
            dec!(40000),
            NaiveDate::from_ymd_opt(2024, 7, 31).unwrap(),
        )
        .with_status(TaxReturnStatus::Assessed);
        assert!(assessed.is_filed());

        // Paid
        let paid = TaxReturn::new(
            "TR-003",
            "ENT-001",
            "JUR-UK",
            NaiveDate::from_ymd_opt(2024, 7, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 9, 30).unwrap(),
            TaxReturnType::IncomeTax,
            dec!(100000),
            dec!(0),
            NaiveDate::from_ymd_opt(2024, 10, 31).unwrap(),
        )
        .with_status(TaxReturnStatus::Paid);
        assert!(paid.is_filed());

        // Amended is not in the "filed" set
        let amended = TaxReturn::new(
            "TR-004",
            "ENT-001",
            "JUR-UK",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 3, 31).unwrap(),
            TaxReturnType::VatReturn,
            dec!(50000),
            dec!(30000),
            NaiveDate::from_ymd_opt(2024, 4, 30).unwrap(),
        )
        .with_status(TaxReturnStatus::Amended);
        assert!(!amended.is_filed());
    }

    #[test]
    fn test_tax_provision() {
        let provision = TaxProvision::new(
            "TP-001",
            "ENT-001",
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            dec!(250000),
            dec!(80000),
            dec!(120000),
            dec!(0.21),
            dec!(0.245),
        )
        .with_reconciliation_item("State taxes", dec!(0.03))
        .with_reconciliation_item("R&D credits", dec!(-0.015));

        // net deferred tax: 80,000 - 120,000 = -40,000 (net liability)
        assert_eq!(provision.net_deferred_tax(), dec!(-40000));
        assert_eq!(provision.rate_reconciliation.len(), 2);
    }

    #[test]
    fn test_withholding_tax_record() {
        let wht = WithholdingTaxRecord::new(
            "WHT-001",
            "PAY-001",
            "V-100",
            WithholdingType::RoyaltyWithholding,
            dec!(0.30),   // statutory 30%
            dec!(0.10),   // applied 10% (treaty)
            dec!(100000), // base amount
        )
        .with_treaty_rate(dec!(0.10))
        .with_certificate_number("CERT-2024-001");

        assert!(wht.has_treaty_benefit());
        // savings: (0.30 - 0.10) * 100,000 = 20,000
        assert_eq!(wht.treaty_savings(), dec!(20000.00));
        assert_eq!(wht.withheld_amount, dec!(10000.00));
        assert_eq!(wht.certificate_number, Some("CERT-2024-001".to_string()));
    }

    #[test]
    fn test_withholding_no_treaty() {
        let wht = WithholdingTaxRecord::new(
            "WHT-002",
            "PAY-002",
            "V-200",
            WithholdingType::ServiceWithholding,
            dec!(0.25),
            dec!(0.25),
            dec!(50000),
        );

        assert!(!wht.has_treaty_benefit());
        // No savings when applied == statutory
        assert_eq!(wht.treaty_savings(), dec!(0.00));
    }

    #[test]
    fn test_uncertain_tax_position() {
        let utp = UncertainTaxPosition::new(
            "UTP-001",
            "ENT-001",
            "R&D credit claim for software development",
            dec!(500000), // total benefit claimed
            dec!(0.50),   // more-likely-than-not threshold
            dec!(350000), // recognized
            TaxMeasurementMethod::MostLikelyAmount,
        );

        // unrecognized: 500,000 - 350,000 = 150,000
        assert_eq!(utp.unrecognized_amount(), dec!(150000));
    }

    #[test]
    fn test_jurisdiction_hierarchy() {
        let federal = TaxJurisdiction::new(
            "JUR-US",
            "United States - Federal",
            "US",
            JurisdictionType::Federal,
        );
        assert!(!federal.is_subnational());

        let state = TaxJurisdiction::new("JUR-US-CA", "California", "US", JurisdictionType::State)
            .with_region_code("CA")
            .with_parent_jurisdiction_id("JUR-US");
        assert!(state.is_subnational());
        assert_eq!(state.region_code, Some("CA".to_string()));
        assert_eq!(state.parent_jurisdiction_id, Some("JUR-US".to_string()));

        let local = TaxJurisdiction::new(
            "JUR-US-CA-SF",
            "San Francisco",
            "US",
            JurisdictionType::Local,
        )
        .with_parent_jurisdiction_id("JUR-US-CA");
        assert!(local.is_subnational());

        let municipal = TaxJurisdiction::new(
            "JUR-US-NY-NYC",
            "New York City",
            "US",
            JurisdictionType::Municipal,
        )
        .with_parent_jurisdiction_id("JUR-US-NY");
        assert!(municipal.is_subnational());

        let supra = TaxJurisdiction::new(
            "JUR-EU",
            "European Union",
            "EU",
            JurisdictionType::Supranational,
        );
        assert!(!supra.is_subnational());
    }

    #[test]
    fn test_serde_roundtrip() {
        let code = TaxCode::new(
            "TC-SERDE",
            "VAT-STD-20",
            "Standard VAT 20%",
            TaxType::Vat,
            dec!(0.20),
            "JUR-UK",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        )
        .with_expiry_date(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap())
        .with_reverse_charge(true);

        let json = serde_json::to_string_pretty(&code).unwrap();
        let deserialized: TaxCode = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, code.id);
        assert_eq!(deserialized.code, code.code);
        assert_eq!(deserialized.rate, code.rate);
        assert_eq!(deserialized.tax_type, code.tax_type);
        assert_eq!(deserialized.is_reverse_charge, code.is_reverse_charge);
        assert_eq!(deserialized.effective_date, code.effective_date);
        assert_eq!(deserialized.expiry_date, code.expiry_date);
    }

    #[test]
    fn test_withholding_serde_roundtrip() {
        // With treaty rate (Some)
        let wht = WithholdingTaxRecord::new(
            "WHT-SERDE-1",
            "PAY-001",
            "V-001",
            WithholdingType::RoyaltyWithholding,
            dec!(0.30),
            dec!(0.15),
            dec!(50000),
        )
        .with_treaty_rate(dec!(0.10));

        let json = serde_json::to_string_pretty(&wht).unwrap();
        let deserialized: WithholdingTaxRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.treaty_rate, Some(dec!(0.10)));
        assert_eq!(deserialized.statutory_rate, dec!(0.30));
        assert_eq!(deserialized.applied_rate, dec!(0.15));
        assert_eq!(deserialized.base_amount, dec!(50000));
        assert_eq!(deserialized.withheld_amount, wht.withheld_amount);

        // Without treaty rate (None)
        let wht_no_treaty = WithholdingTaxRecord::new(
            "WHT-SERDE-2",
            "PAY-002",
            "V-002",
            WithholdingType::ServiceWithholding,
            dec!(0.30),
            dec!(0.30),
            dec!(10000),
        );

        let json2 = serde_json::to_string_pretty(&wht_no_treaty).unwrap();
        let deserialized2: WithholdingTaxRecord = serde_json::from_str(&json2).unwrap();
        assert_eq!(deserialized2.treaty_rate, None);
        assert_eq!(deserialized2.statutory_rate, dec!(0.30));
    }
}
