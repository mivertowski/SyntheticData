//! Retail industry settings and configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of retail store.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StoreType {
    /// Flagship/anchor store.
    Flagship,
    /// Standard retail location.
    Standard,
    /// Outlet/discount location.
    Outlet,
    /// Mall kiosk.
    Kiosk,
    /// Online/e-commerce.
    Online,
    /// Pop-up/temporary store.
    PopUp,
    /// Warehouse/big-box.
    Warehouse,
}

impl StoreType {
    /// Returns average daily transactions for this store type.
    pub fn avg_daily_transactions(&self) -> u32 {
        match self {
            StoreType::Flagship => 500,
            StoreType::Standard => 200,
            StoreType::Outlet => 300,
            StoreType::Kiosk => 50,
            StoreType::Online => 1000,
            StoreType::PopUp => 75,
            StoreType::Warehouse => 150,
        }
    }

    /// Returns average transaction value for this store type.
    pub fn avg_transaction_value(&self) -> f64 {
        match self {
            StoreType::Flagship => 150.0,
            StoreType::Standard => 75.0,
            StoreType::Outlet => 45.0,
            StoreType::Kiosk => 25.0,
            StoreType::Online => 85.0,
            StoreType::PopUp => 60.0,
            StoreType::Warehouse => 200.0,
        }
    }
}

/// Type of promotion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PromotionType {
    /// Percentage discount.
    PercentOff,
    /// Fixed amount discount.
    AmountOff,
    /// Buy one get one.
    Bogo,
    /// Buy X get Y free.
    BuyXGetY,
    /// Bundle pricing.
    Bundle,
    /// Loyalty points multiplier.
    PointsMultiplier,
    /// Clearance markdown.
    Clearance,
    /// Seasonal sale.
    Seasonal,
}

impl PromotionType {
    /// Returns the discount code prefix for this promotion type.
    pub fn code_prefix(&self) -> &'static str {
        match self {
            PromotionType::PercentOff => "PCT",
            PromotionType::AmountOff => "AMT",
            PromotionType::Bogo => "BOG",
            PromotionType::BuyXGetY => "BXY",
            PromotionType::Bundle => "BND",
            PromotionType::PointsMultiplier => "PTS",
            PromotionType::Clearance => "CLR",
            PromotionType::Seasonal => "SEA",
        }
    }
}

/// Retail industry settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetailSettings {
    /// Store types to model.
    pub store_types: Vec<StoreType>,
    /// Number of stores to generate.
    pub store_count: u32,
    /// Average SKUs per store.
    pub avg_skus_per_store: u32,
    /// Return rate (0.0-1.0).
    pub return_rate: f64,
    /// Shrinkage rate (0.0-1.0).
    pub shrinkage_rate: f64,
    /// Promotion active rate (% of products on promotion).
    pub promotion_rate: f64,
    /// Employee discount rate.
    pub employee_discount_rate: f64,
    /// Loyalty program enabled.
    pub loyalty_program: bool,
    /// Payment method distribution.
    pub payment_methods: HashMap<String, f64>,
    /// Peak hours (24-hour format).
    pub peak_hours: Vec<u32>,
    /// Seasonal patterns enabled.
    pub seasonal_patterns: bool,
}

impl Default for RetailSettings {
    fn default() -> Self {
        let mut payment_methods = HashMap::new();
        payment_methods.insert("credit_card".to_string(), 0.45);
        payment_methods.insert("debit_card".to_string(), 0.30);
        payment_methods.insert("cash".to_string(), 0.15);
        payment_methods.insert("mobile_payment".to_string(), 0.08);
        payment_methods.insert("gift_card".to_string(), 0.02);

        Self {
            store_types: vec![StoreType::Standard, StoreType::Outlet, StoreType::Online],
            store_count: 50,
            avg_skus_per_store: 5000,
            return_rate: 0.08,
            shrinkage_rate: 0.015,
            promotion_rate: 0.20,
            employee_discount_rate: 0.20,
            loyalty_program: true,
            payment_methods,
            peak_hours: vec![12, 13, 17, 18, 19],
            seasonal_patterns: true,
        }
    }
}

impl RetailSettings {
    /// Returns the expected daily shrinkage value per store.
    pub fn expected_daily_shrinkage(&self, avg_daily_sales: f64) -> f64 {
        avg_daily_sales * self.shrinkage_rate
    }

    /// Returns the expected daily return value per store.
    pub fn expected_daily_returns(&self, avg_daily_sales: f64) -> f64 {
        avg_daily_sales * self.return_rate
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_store_type() {
        let flagship = StoreType::Flagship;
        assert_eq!(flagship.avg_daily_transactions(), 500);
        assert!(flagship.avg_transaction_value() > 100.0);

        let online = StoreType::Online;
        assert!(online.avg_daily_transactions() > flagship.avg_daily_transactions());
    }

    #[test]
    fn test_promotion_type() {
        let bogo = PromotionType::Bogo;
        assert_eq!(bogo.code_prefix(), "BOG");
    }

    #[test]
    fn test_retail_settings() {
        let settings = RetailSettings::default();

        assert!(settings.return_rate > 0.0 && settings.return_rate < 0.2);
        assert!(settings.shrinkage_rate > 0.0 && settings.shrinkage_rate < 0.05);
        assert!(settings.payment_methods.len() >= 4);

        let shrinkage = settings.expected_daily_shrinkage(10_000.0);
        assert!(shrinkage > 0.0 && shrinkage < 500.0);
    }
}
