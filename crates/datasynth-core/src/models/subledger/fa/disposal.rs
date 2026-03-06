//! Asset disposal and retirement models.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{AssetClass, FixedAssetRecord};
use crate::models::subledger::GLReference;

/// Asset disposal transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetDisposal {
    /// Disposal ID.
    pub disposal_id: String,
    /// Asset number.
    pub asset_number: String,
    /// Sub-number.
    pub sub_number: String,
    /// Company code.
    pub company_code: String,
    /// Disposal date.
    pub disposal_date: NaiveDate,
    /// Posting date.
    pub posting_date: NaiveDate,
    /// Disposal type.
    pub disposal_type: DisposalType,
    /// Disposal reason.
    pub disposal_reason: DisposalReason,
    /// Asset description.
    pub asset_description: String,
    /// Asset class.
    pub asset_class: AssetClass,
    /// Original acquisition cost.
    pub acquisition_cost: Decimal,
    /// Accumulated depreciation at disposal.
    pub accumulated_depreciation: Decimal,
    /// Net book value at disposal.
    pub net_book_value: Decimal,
    /// Sale proceeds (if sold).
    pub sale_proceeds: Decimal,
    /// Disposal costs (removal, transport, etc.).
    pub disposal_costs: Decimal,
    /// Net proceeds.
    pub net_proceeds: Decimal,
    /// Gain or loss on disposal.
    pub gain_loss: Decimal,
    /// Is gain (vs loss).
    pub is_gain: bool,
    /// Customer (if sold).
    pub customer_id: Option<String>,
    /// Invoice reference.
    pub invoice_reference: Option<String>,
    /// GL references.
    pub gl_references: Vec<GLReference>,
    /// Approval status.
    pub approval_status: DisposalApprovalStatus,
    /// Approved by.
    pub approved_by: Option<String>,
    /// Approval date.
    pub approval_date: Option<NaiveDate>,
    /// Created by.
    pub created_by: String,
    /// Created at.
    pub created_at: DateTime<Utc>,
    /// Notes.
    pub notes: Option<String>,
}

impl AssetDisposal {
    /// Creates a new disposal record.
    pub fn new(
        disposal_id: String,
        asset: &FixedAssetRecord,
        disposal_date: NaiveDate,
        disposal_type: DisposalType,
        disposal_reason: DisposalReason,
        created_by: String,
    ) -> Self {
        Self {
            disposal_id,
            asset_number: asset.asset_number.clone(),
            sub_number: asset.sub_number.clone(),
            company_code: asset.company_code.clone(),
            disposal_date,
            posting_date: disposal_date,
            disposal_type,
            disposal_reason,
            asset_description: asset.description.clone(),
            asset_class: asset.asset_class,
            acquisition_cost: asset.acquisition_cost,
            accumulated_depreciation: asset.accumulated_depreciation,
            net_book_value: asset.net_book_value,
            sale_proceeds: Decimal::ZERO,
            disposal_costs: Decimal::ZERO,
            net_proceeds: Decimal::ZERO,
            gain_loss: Decimal::ZERO,
            is_gain: false,
            customer_id: None,
            invoice_reference: None,
            gl_references: Vec::new(),
            approval_status: DisposalApprovalStatus::Pending,
            approved_by: None,
            approval_date: None,
            created_by,
            created_at: Utc::now(),
            notes: None,
        }
    }

    /// Creates a sale disposal.
    pub fn sale(
        disposal_id: String,
        asset: &FixedAssetRecord,
        disposal_date: NaiveDate,
        sale_proceeds: Decimal,
        customer_id: String,
        created_by: String,
    ) -> Self {
        let mut disposal = Self::new(
            disposal_id,
            asset,
            disposal_date,
            DisposalType::Sale,
            DisposalReason::Sale,
            created_by,
        );

        disposal.sale_proceeds = sale_proceeds;
        disposal.customer_id = Some(customer_id);
        disposal.calculate_gain_loss();
        disposal
    }

    /// Creates a scrapping disposal.
    pub fn scrap(
        disposal_id: String,
        asset: &FixedAssetRecord,
        disposal_date: NaiveDate,
        reason: DisposalReason,
        created_by: String,
    ) -> Self {
        let mut disposal = Self::new(
            disposal_id,
            asset,
            disposal_date,
            DisposalType::Scrapping,
            reason,
            created_by,
        );
        disposal.calculate_gain_loss();
        disposal
    }

    /// Sets sale proceeds.
    pub fn with_sale_proceeds(mut self, proceeds: Decimal) -> Self {
        self.sale_proceeds = proceeds;
        self.calculate_gain_loss();
        self
    }

    /// Sets disposal costs.
    pub fn with_disposal_costs(mut self, costs: Decimal) -> Self {
        self.disposal_costs = costs;
        self.calculate_gain_loss();
        self
    }

    /// Calculates gain or loss on disposal.
    pub fn calculate_gain_loss(&mut self) {
        self.net_proceeds = self.sale_proceeds - self.disposal_costs;
        self.gain_loss = self.net_proceeds - self.net_book_value;
        self.is_gain = self.gain_loss >= Decimal::ZERO;
    }

    /// Approves the disposal.
    pub fn approve(&mut self, approver: String, approval_date: NaiveDate) {
        self.approval_status = DisposalApprovalStatus::Approved;
        self.approved_by = Some(approver);
        self.approval_date = Some(approval_date);
    }

    /// Rejects the disposal.
    pub fn reject(&mut self, reason: String) {
        self.approval_status = DisposalApprovalStatus::Rejected;
        self.notes = Some(format!(
            "{}Rejected: {}",
            self.notes
                .as_ref()
                .map(|n| format!("{n}. "))
                .unwrap_or_default(),
            reason
        ));
    }

    /// Posts the disposal.
    pub fn post(&mut self) {
        self.approval_status = DisposalApprovalStatus::Posted;
    }

    /// Adds a GL reference.
    pub fn add_gl_reference(&mut self, reference: GLReference) {
        self.gl_references.push(reference);
    }

    /// Gets the gain (or zero if loss).
    pub fn gain(&self) -> Decimal {
        if self.is_gain {
            self.gain_loss
        } else {
            Decimal::ZERO
        }
    }

    /// Gets the loss (or zero if gain).
    pub fn loss(&self) -> Decimal {
        if !self.is_gain {
            self.gain_loss.abs()
        } else {
            Decimal::ZERO
        }
    }

    /// Requires approval based on threshold.
    pub fn requires_approval(&self, threshold: Decimal) -> bool {
        self.net_book_value > threshold || self.gain_loss.abs() > threshold
    }
}

/// Type of disposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DisposalType {
    /// Sale to external party.
    #[default]
    Sale,
    /// Intercompany transfer.
    IntercompanyTransfer,
    /// Scrapping/write-off.
    Scrapping,
    /// Trade-in.
    TradeIn,
    /// Donation.
    Donation,
    /// Loss (theft, destruction).
    Loss,
    /// Partial disposal.
    PartialDisposal,
}

/// Reason for disposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DisposalReason {
    /// Normal sale.
    #[default]
    Sale,
    /// End of useful life.
    EndOfLife,
    /// Obsolete.
    Obsolescence,
    /// Damaged beyond repair.
    Damage,
    /// Theft or loss.
    TheftLoss,
    /// Replaced by new asset.
    Replacement,
    /// Business restructuring.
    Restructuring,
    /// Compliance/regulatory.
    Compliance,
    /// Environmental disposal.
    Environmental,
    /// Donated to charity.
    Donated,
    /// Other.
    Other,
}

/// Disposal approval status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DisposalApprovalStatus {
    /// Pending approval.
    #[default]
    Pending,
    /// Approved.
    Approved,
    /// Rejected.
    Rejected,
    /// Posted.
    Posted,
    /// Cancelled.
    Cancelled,
}

/// Asset transfer (between locations or entities).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetTransfer {
    /// Transfer ID.
    pub transfer_id: String,
    /// Asset number.
    pub asset_number: String,
    /// Sub-number.
    pub sub_number: String,
    /// Transfer date.
    pub transfer_date: NaiveDate,
    /// Transfer type.
    pub transfer_type: TransferType,
    /// From company code.
    pub from_company: String,
    /// To company code.
    pub to_company: String,
    /// From cost center.
    pub from_cost_center: Option<String>,
    /// To cost center.
    pub to_cost_center: Option<String>,
    /// From location.
    pub from_location: Option<String>,
    /// To location.
    pub to_location: Option<String>,
    /// Transfer value.
    pub transfer_value: Decimal,
    /// Accumulated depreciation transferred.
    pub accumulated_depreciation: Decimal,
    /// Status.
    pub status: TransferStatus,
    /// Created by.
    pub created_by: String,
    /// Created at.
    pub created_at: DateTime<Utc>,
    /// Notes.
    pub notes: Option<String>,
}

impl AssetTransfer {
    /// Creates a new asset transfer.
    pub fn new(
        transfer_id: String,
        asset: &FixedAssetRecord,
        transfer_date: NaiveDate,
        transfer_type: TransferType,
        to_company: String,
        created_by: String,
    ) -> Self {
        Self {
            transfer_id,
            asset_number: asset.asset_number.clone(),
            sub_number: asset.sub_number.clone(),
            transfer_date,
            transfer_type,
            from_company: asset.company_code.clone(),
            to_company,
            from_cost_center: asset.cost_center.clone(),
            to_cost_center: None,
            from_location: asset.location.clone(),
            to_location: None,
            transfer_value: asset.net_book_value,
            accumulated_depreciation: asset.accumulated_depreciation,
            status: TransferStatus::Draft,
            created_by,
            created_at: Utc::now(),
            notes: None,
        }
    }

    /// Sets destination cost center.
    pub fn to_cost_center(mut self, cost_center: String) -> Self {
        self.to_cost_center = Some(cost_center);
        self
    }

    /// Sets destination location.
    pub fn to_location(mut self, location: String) -> Self {
        self.to_location = Some(location);
        self
    }

    /// Submits for approval.
    pub fn submit(&mut self) {
        self.status = TransferStatus::Submitted;
    }

    /// Approves the transfer.
    pub fn approve(&mut self) {
        self.status = TransferStatus::Approved;
    }

    /// Completes the transfer.
    pub fn complete(&mut self) {
        self.status = TransferStatus::Completed;
    }

    /// Is intercompany transfer.
    pub fn is_intercompany(&self) -> bool {
        self.from_company != self.to_company
    }
}

/// Type of transfer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferType {
    /// Within same company, different cost center.
    IntraCompany,
    /// Between legal entities.
    InterCompany,
    /// Physical location change only.
    LocationChange,
    /// Reorganization.
    Reorganization,
}

/// Transfer status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferStatus {
    /// Draft.
    Draft,
    /// Submitted for approval.
    Submitted,
    /// Approved.
    Approved,
    /// Completed.
    Completed,
    /// Rejected.
    Rejected,
    /// Cancelled.
    Cancelled,
}

/// Asset impairment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetImpairment {
    /// Impairment ID.
    pub impairment_id: String,
    /// Asset number.
    pub asset_number: String,
    /// Company code.
    pub company_code: String,
    /// Impairment date.
    pub impairment_date: NaiveDate,
    /// Net book value before impairment.
    pub nbv_before: Decimal,
    /// Fair value/recoverable amount.
    pub fair_value: Decimal,
    /// Impairment loss.
    pub impairment_loss: Decimal,
    /// Net book value after impairment.
    pub nbv_after: Decimal,
    /// Impairment reason.
    pub reason: ImpairmentReason,
    /// Is reversal (impairment recovery).
    pub is_reversal: bool,
    /// GL reference.
    pub gl_reference: Option<GLReference>,
    /// Created by.
    pub created_by: String,
    /// Created at.
    pub created_at: DateTime<Utc>,
    /// Notes.
    pub notes: Option<String>,
}

impl AssetImpairment {
    /// Creates a new impairment.
    pub fn new(
        impairment_id: String,
        asset: &FixedAssetRecord,
        impairment_date: NaiveDate,
        fair_value: Decimal,
        reason: ImpairmentReason,
        created_by: String,
    ) -> Self {
        let impairment_loss = (asset.net_book_value - fair_value).max(Decimal::ZERO);

        Self {
            impairment_id,
            asset_number: asset.asset_number.clone(),
            company_code: asset.company_code.clone(),
            impairment_date,
            nbv_before: asset.net_book_value,
            fair_value,
            impairment_loss,
            nbv_after: fair_value,
            reason,
            is_reversal: false,
            gl_reference: None,
            created_by,
            created_at: Utc::now(),
            notes: None,
        }
    }

    /// Creates an impairment reversal.
    pub fn reversal(
        impairment_id: String,
        asset: &FixedAssetRecord,
        impairment_date: NaiveDate,
        new_fair_value: Decimal,
        max_reversal: Decimal,
        created_by: String,
    ) -> Self {
        let reversal_amount = (new_fair_value - asset.net_book_value).min(max_reversal);

        Self {
            impairment_id,
            asset_number: asset.asset_number.clone(),
            company_code: asset.company_code.clone(),
            impairment_date,
            nbv_before: asset.net_book_value,
            fair_value: new_fair_value,
            impairment_loss: -reversal_amount, // Negative for reversal
            nbv_after: asset.net_book_value + reversal_amount,
            reason: ImpairmentReason::ValueRecovery,
            is_reversal: true,
            gl_reference: None,
            created_by,
            created_at: Utc::now(),
            notes: None,
        }
    }
}

/// Reason for impairment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImpairmentReason {
    /// Physical damage.
    PhysicalDamage,
    /// Market decline.
    MarketDecline,
    /// Technological obsolescence.
    TechnologyObsolescence,
    /// Legal or regulatory changes.
    RegulatoryChange,
    /// Business restructuring.
    Restructuring,
    /// Asset held for sale.
    HeldForSale,
    /// Value recovery (reversal).
    ValueRecovery,
    /// Other.
    Other,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_asset() -> FixedAssetRecord {
        let mut asset = FixedAssetRecord::new(
            "ASSET001".to_string(),
            "1000".to_string(),
            AssetClass::MachineryEquipment,
            "Production Machine".to_string(),
            NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            dec!(100000),
            "USD".to_string(),
        );
        asset.accumulated_depreciation = dec!(60000);
        asset.net_book_value = dec!(40000);
        asset
    }

    #[test]
    fn test_disposal_sale_gain() {
        let asset = create_test_asset();
        let disposal = AssetDisposal::sale(
            "DISP001".to_string(),
            &asset,
            NaiveDate::from_ymd_opt(2024, 6, 30).unwrap(),
            dec!(50000),
            "CUST001".to_string(),
            "USER1".to_string(),
        );

        assert_eq!(disposal.net_book_value, dec!(40000));
        assert_eq!(disposal.sale_proceeds, dec!(50000));
        assert_eq!(disposal.gain_loss, dec!(10000));
        assert!(disposal.is_gain);
    }

    #[test]
    fn test_disposal_sale_loss() {
        let asset = create_test_asset();
        let disposal = AssetDisposal::sale(
            "DISP002".to_string(),
            &asset,
            NaiveDate::from_ymd_opt(2024, 6, 30).unwrap(),
            dec!(30000),
            "CUST001".to_string(),
            "USER1".to_string(),
        );

        assert_eq!(disposal.gain_loss, dec!(-10000));
        assert!(!disposal.is_gain);
        assert_eq!(disposal.loss(), dec!(10000));
    }

    #[test]
    fn test_disposal_scrapping() {
        let asset = create_test_asset();
        let disposal = AssetDisposal::scrap(
            "DISP003".to_string(),
            &asset,
            NaiveDate::from_ymd_opt(2024, 6, 30).unwrap(),
            DisposalReason::EndOfLife,
            "USER1".to_string(),
        );

        assert_eq!(disposal.sale_proceeds, Decimal::ZERO);
        assert_eq!(disposal.gain_loss, dec!(-40000));
        assert!(!disposal.is_gain);
    }

    #[test]
    fn test_asset_transfer() {
        let asset = create_test_asset();
        let transfer = AssetTransfer::new(
            "TRF001".to_string(),
            &asset,
            NaiveDate::from_ymd_opt(2024, 7, 1).unwrap(),
            TransferType::InterCompany,
            "2000".to_string(),
            "USER1".to_string(),
        )
        .to_cost_center("CC200".to_string());

        assert!(transfer.is_intercompany());
        assert_eq!(transfer.transfer_value, dec!(40000));
    }

    #[test]
    fn test_impairment() {
        let asset = create_test_asset();
        let impairment = AssetImpairment::new(
            "IMP001".to_string(),
            &asset,
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            dec!(25000),
            ImpairmentReason::MarketDecline,
            "USER1".to_string(),
        );

        assert_eq!(impairment.nbv_before, dec!(40000));
        assert_eq!(impairment.fair_value, dec!(25000));
        assert_eq!(impairment.impairment_loss, dec!(15000));
        assert_eq!(impairment.nbv_after, dec!(25000));
    }
}
