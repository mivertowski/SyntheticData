//! Sales quote models for the Quote-to-Cash process.
//!
//! These models represent sales quotations and their line items,
//! supporting the full quote lifecycle from draft through win/loss.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Status of a sales quote through the quotation lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum QuoteStatus {
    /// Initial draft, not yet sent to customer
    #[default]
    Draft,
    /// Quote has been sent to the customer
    Sent,
    /// Quote is under active negotiation
    Negotiating,
    /// Customer accepted the quote
    Won,
    /// Customer rejected the quote
    Lost,
    /// Quote validity period has elapsed
    Expired,
    /// Quote was cancelled before resolution
    Cancelled,
}

/// A sales quotation issued to a prospective or existing customer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesQuote {
    /// Unique quote identifier
    pub quote_id: String,
    /// Company code issuing the quote
    pub company_code: String,
    /// Customer identifier
    pub customer_id: String,
    /// Customer name
    pub customer_name: String,
    /// Date the quote was created
    pub quote_date: NaiveDate,
    /// Date the quote expires
    pub valid_until: NaiveDate,
    /// Current status of the quote
    pub status: QuoteStatus,
    /// Individual line items on the quote
    pub line_items: Vec<QuoteLineItem>,
    /// Total quoted amount before discount
    #[serde(with = "rust_decimal::serde::str")]
    pub total_amount: Decimal,
    /// Currency code (e.g., USD, EUR)
    pub currency: String,
    /// Discount percentage applied (0.0 to 1.0)
    pub discount_percent: f64,
    /// Calculated discount amount
    #[serde(with = "rust_decimal::serde::str")]
    pub discount_amount: Decimal,
    /// Sales representative responsible for the quote
    pub sales_rep_id: Option<String>,
    /// Linked sales order identifier (populated when status is Won)
    pub sales_order_id: Option<String>,
    /// Reason the quote was lost (populated when status is Lost)
    pub lost_reason: Option<String>,
    /// Free-text notes on the quote
    pub notes: Option<String>,
}

/// An individual line item within a sales quote.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteLineItem {
    /// Deterministic UUID for this line item
    pub id: String,
    /// Sequential item number within the quote
    pub item_number: u32,
    /// Material or product identifier
    pub material_id: String,
    /// Description of the quoted item
    pub description: String,
    /// Quoted quantity
    #[serde(with = "rust_decimal::serde::str")]
    pub quantity: Decimal,
    /// Unit price for the item
    #[serde(with = "rust_decimal::serde::str")]
    pub unit_price: Decimal,
    /// Total line amount (quantity * unit_price)
    #[serde(with = "rust_decimal::serde::str")]
    pub line_amount: Decimal,
}
