//! Retail industry transaction generation.
//!
//! Provides retail-specific:
//! - POS transactions, returns, promotions
//! - Inventory management and shrinkage
//! - Master data (stores, products, categories)
//! - Anomalies (sweethearting, skimming, refund fraud)

mod anomalies;
mod settings;
mod transactions;

pub use anomalies::RetailAnomaly;
pub use settings::{PromotionType, RetailSettings, StoreType};
pub use transactions::{
    InventoryTransaction, PosTransaction, RetailTransaction, RetailTransactionGenerator,
};
