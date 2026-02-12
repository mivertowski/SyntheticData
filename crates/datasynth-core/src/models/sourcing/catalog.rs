//! Catalog item models for contract-based ordering.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// A catalog item available for ordering under a contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogItem {
    /// Unique catalog item identifier
    pub catalog_item_id: String,
    /// Contract ID this item belongs to
    pub contract_id: String,
    /// Contract line item number
    pub contract_line_number: u16,
    /// Vendor ID
    pub vendor_id: String,
    /// Material ID
    pub material_id: Option<String>,
    /// Item description
    pub description: String,
    /// Catalog price (from contract)
    #[serde(with = "rust_decimal::serde::str")]
    pub catalog_price: Decimal,
    /// Unit of measure
    pub uom: String,
    /// Whether this is the preferred item for this material
    pub is_preferred: bool,
    /// Category for search/browse
    pub category: String,
    /// Minimum order quantity
    pub min_order_quantity: Option<Decimal>,
    /// Lead time in days
    pub lead_time_days: Option<u32>,
    /// Is item active and orderable
    pub is_active: bool,
}
