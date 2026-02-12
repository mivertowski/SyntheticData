//! Supplier bid models for RFx responses.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Status of a supplier bid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BidStatus {
    /// Bid submitted, awaiting evaluation
    #[default]
    Submitted,
    /// Under evaluation
    UnderEvaluation,
    /// Bid accepted (winner)
    Accepted,
    /// Bid rejected
    Rejected,
    /// Bid withdrawn by vendor
    Withdrawn,
    /// Technically disqualified
    Disqualified,
}

/// Line item within a supplier bid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidLineItem {
    /// Item number (matches RFx line item)
    pub item_number: u16,
    /// Offered unit price
    #[serde(with = "rust_decimal::serde::str")]
    pub unit_price: Decimal,
    /// Offered quantity
    pub quantity: Decimal,
    /// Total line amount
    #[serde(with = "rust_decimal::serde::str")]
    pub total_amount: Decimal,
    /// Lead time in days
    pub lead_time_days: u32,
    /// Vendor's notes for this item
    pub notes: Option<String>,
}

/// A supplier's bid in response to an RFx.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierBid {
    /// Unique bid identifier
    pub bid_id: String,
    /// RFx this bid responds to
    pub rfx_id: String,
    /// Vendor submitting the bid
    pub vendor_id: String,
    /// Company code
    pub company_code: String,
    /// Bid status
    pub status: BidStatus,
    /// Submission date
    pub submission_date: NaiveDate,
    /// Bid line items
    pub line_items: Vec<BidLineItem>,
    /// Total bid amount
    #[serde(with = "rust_decimal::serde::str")]
    pub total_amount: Decimal,
    /// Validity period (days from submission)
    pub validity_days: u32,
    /// Payment terms offered
    pub payment_terms: String,
    /// Delivery terms (incoterms)
    pub delivery_terms: Option<String>,
    /// Technical proposal summary
    pub technical_summary: Option<String>,
    /// Whether the bid was submitted on time
    pub is_on_time: bool,
    /// Whether the bid is technically compliant
    pub is_compliant: bool,
}
