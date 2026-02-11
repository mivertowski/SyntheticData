//! Retail-specific anomalies.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::super::common::IndustryAnomaly;

/// Retail-specific anomaly types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetailAnomaly {
    /// Employee gives unauthorized discounts to friends/family.
    Sweethearting {
        cashier_id: String,
        beneficiary: String,
        estimated_loss: Decimal,
        transaction_count: u32,
    },
    /// Cash stolen from register before recording.
    Skimming {
        register_id: String,
        store_id: String,
        amount: Decimal,
        detection_method: String,
    },
    /// Fraudulent refunds processed.
    RefundFraud {
        transaction_id: String,
        employee_id: String,
        refund_amount: Decimal,
        scheme_type: RefundFraudType,
    },
    /// Receiving fraud (short shipments, diversions).
    ReceivingFraud {
        po_id: String,
        employee_id: String,
        short_quantity: u32,
        value: Decimal,
    },
    /// Fraudulent inter-store transfers.
    TransferFraud {
        from_store: String,
        to_store: String,
        items_diverted: u32,
        value: Decimal,
    },
    /// Coupon/promotion fraud.
    CouponFraud {
        coupon_code: String,
        not_presented: bool,
        value: Decimal,
        transaction_count: u32,
    },
    /// Employee discount abuse.
    EmployeeDiscountAbuse {
        employee_id: String,
        non_employee_beneficiary: String,
        discount_value: Decimal,
        transaction_count: u32,
    },
    /// Void abuse.
    VoidAbuse {
        cashier_id: String,
        void_count: u32,
        void_total: Decimal,
        period_days: u32,
    },
    /// Price override abuse.
    PriceOverrideAbuse {
        employee_id: String,
        override_count: u32,
        total_discount: Decimal,
    },
    /// Gift card fraud.
    GiftCardFraud {
        scheme_type: GiftCardFraudType,
        amount: Decimal,
        cards_affected: u32,
    },
    /// Inventory manipulation.
    InventoryManipulation {
        store_id: String,
        manipulation_type: InventoryManipulationType,
        value: Decimal,
    },
    /// Fictitious vendor kickback.
    VendorKickback {
        vendor_id: String,
        buyer_id: String,
        kickback_amount: Decimal,
        scheme_duration_days: u32,
    },
}

/// Type of refund fraud.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RefundFraudType {
    /// Refund to personal card.
    RefundToPersonalCard,
    /// Fake merchandise return.
    FakeMerchandiseReturn,
    /// Return without receipt fraud.
    NoReceiptFraud,
    /// Cross-retailer return fraud.
    CrossRetailerFraud,
    /// Wardrobing (return after use).
    Wardrobing,
}

/// Type of gift card fraud.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GiftCardFraudType {
    /// Loading without payment.
    LoadingWithoutPayment,
    /// Balance transfer scheme.
    BalanceTransfer,
    /// Card number harvesting.
    CardNumberHarvesting,
    /// Return to gift card scheme.
    ReturnToGiftCard,
}

/// Type of inventory manipulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InventoryManipulationType {
    /// Phantom inventory.
    PhantomInventory,
    /// Concealed shrinkage.
    ConcealedShrinkage,
    /// Count manipulation.
    CountManipulation,
    /// Category shifting.
    CategoryShifting,
}

impl IndustryAnomaly for RetailAnomaly {
    fn anomaly_type(&self) -> &str {
        match self {
            RetailAnomaly::Sweethearting { .. } => "sweethearting",
            RetailAnomaly::Skimming { .. } => "skimming",
            RetailAnomaly::RefundFraud { .. } => "refund_fraud",
            RetailAnomaly::ReceivingFraud { .. } => "receiving_fraud",
            RetailAnomaly::TransferFraud { .. } => "transfer_fraud",
            RetailAnomaly::CouponFraud { .. } => "coupon_fraud",
            RetailAnomaly::EmployeeDiscountAbuse { .. } => "employee_discount_abuse",
            RetailAnomaly::VoidAbuse { .. } => "void_abuse",
            RetailAnomaly::PriceOverrideAbuse { .. } => "price_override_abuse",
            RetailAnomaly::GiftCardFraud { .. } => "gift_card_fraud",
            RetailAnomaly::InventoryManipulation { .. } => "inventory_manipulation",
            RetailAnomaly::VendorKickback { .. } => "vendor_kickback",
        }
    }

    fn severity(&self) -> u8 {
        match self {
            RetailAnomaly::EmployeeDiscountAbuse { .. } => 3,
            RetailAnomaly::CouponFraud { .. } => 3,
            RetailAnomaly::VoidAbuse { .. } => 3,
            RetailAnomaly::PriceOverrideAbuse { .. } => 3,
            RetailAnomaly::Sweethearting { .. } => 4,
            RetailAnomaly::RefundFraud { .. } => 4,
            RetailAnomaly::TransferFraud { .. } => 4,
            RetailAnomaly::Skimming { .. } => 5,
            RetailAnomaly::ReceivingFraud { .. } => 5,
            RetailAnomaly::GiftCardFraud { .. } => 4,
            RetailAnomaly::InventoryManipulation { .. } => 4,
            RetailAnomaly::VendorKickback { .. } => 5,
        }
    }

    fn detection_difficulty(&self) -> &str {
        match self {
            RetailAnomaly::VoidAbuse { .. } => "easy",
            RetailAnomaly::PriceOverrideAbuse { .. } => "easy",
            RetailAnomaly::EmployeeDiscountAbuse { .. } => "moderate",
            RetailAnomaly::CouponFraud { .. } => "moderate",
            RetailAnomaly::RefundFraud { .. } => "moderate",
            RetailAnomaly::TransferFraud { .. } => "moderate",
            RetailAnomaly::Sweethearting { .. } => "hard",
            RetailAnomaly::GiftCardFraud { .. } => "hard",
            RetailAnomaly::InventoryManipulation { .. } => "hard",
            RetailAnomaly::Skimming { .. } => "expert",
            RetailAnomaly::ReceivingFraud { .. } => "hard",
            RetailAnomaly::VendorKickback { .. } => "expert",
        }
    }

    fn indicators(&self) -> Vec<String> {
        match self {
            RetailAnomaly::Sweethearting { .. } => vec![
                "high_no_sale_rate".to_string(),
                "frequent_price_overrides".to_string(),
                "repeat_customer_discounts".to_string(),
                "lower_avg_transaction".to_string(),
            ],
            RetailAnomaly::Skimming { .. } => vec![
                "register_short".to_string(),
                "cash_variance_pattern".to_string(),
                "transaction_gaps".to_string(),
            ],
            RetailAnomaly::RefundFraud { .. } => vec![
                "high_refund_rate".to_string(),
                "refunds_to_same_card".to_string(),
                "refunds_without_receipt".to_string(),
                "customer_not_present_refunds".to_string(),
            ],
            RetailAnomaly::VoidAbuse { .. } => vec![
                "high_void_rate".to_string(),
                "voids_after_tender".to_string(),
                "pattern_of_small_voids".to_string(),
            ],
            RetailAnomaly::GiftCardFraud { .. } => vec![
                "gift_cards_activated_without_sale".to_string(),
                "unusual_gift_card_patterns".to_string(),
                "gift_card_balance_anomalies".to_string(),
            ],
            RetailAnomaly::InventoryManipulation { .. } => vec![
                "shrinkage_pattern_anomaly".to_string(),
                "count_timing_manipulation".to_string(),
                "category_variance_spike".to_string(),
            ],
            _ => vec!["general_retail_anomaly".to_string()],
        }
    }

    fn regulatory_concerns(&self) -> Vec<String> {
        match self {
            RetailAnomaly::Skimming { .. }
            | RetailAnomaly::ReceivingFraud { .. }
            | RetailAnomaly::VendorKickback { .. } => vec![
                "financial_statement_fraud".to_string(),
                "employee_theft".to_string(),
                "internal_controls".to_string(),
            ],
            RetailAnomaly::InventoryManipulation { .. } => vec![
                "inventory_valuation".to_string(),
                "asc_330".to_string(),
                "sox_section_404".to_string(),
            ],
            _ => vec![
                "employee_theft".to_string(),
                "internal_controls".to_string(),
            ],
        }
    }
}

impl RetailAnomaly {
    /// Returns the financial impact of this anomaly.
    pub fn financial_impact(&self) -> Decimal {
        match self {
            RetailAnomaly::Sweethearting { estimated_loss, .. } => *estimated_loss,
            RetailAnomaly::Skimming { amount, .. } => *amount,
            RetailAnomaly::RefundFraud { refund_amount, .. } => *refund_amount,
            RetailAnomaly::ReceivingFraud { value, .. } => *value,
            RetailAnomaly::TransferFraud { value, .. } => *value,
            RetailAnomaly::CouponFraud { value, .. } => *value,
            RetailAnomaly::EmployeeDiscountAbuse { discount_value, .. } => *discount_value,
            RetailAnomaly::VoidAbuse { void_total, .. } => *void_total,
            RetailAnomaly::PriceOverrideAbuse { total_discount, .. } => *total_discount,
            RetailAnomaly::GiftCardFraud { amount, .. } => *amount,
            RetailAnomaly::InventoryManipulation { value, .. } => *value,
            RetailAnomaly::VendorKickback {
                kickback_amount, ..
            } => *kickback_amount,
        }
    }

    /// Returns whether this involves collusion.
    pub fn involves_collusion(&self) -> bool {
        matches!(
            self,
            RetailAnomaly::ReceivingFraud { .. }
                | RetailAnomaly::TransferFraud { .. }
                | RetailAnomaly::VendorKickback { .. }
        )
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_sweethearting() {
        let anomaly = RetailAnomaly::Sweethearting {
            cashier_id: "C001".to_string(),
            beneficiary: "Friend".to_string(),
            estimated_loss: Decimal::new(500, 0),
            transaction_count: 20,
        };

        assert_eq!(anomaly.anomaly_type(), "sweethearting");
        assert_eq!(anomaly.severity(), 4);
        assert_eq!(anomaly.detection_difficulty(), "hard");
        assert_eq!(anomaly.financial_impact(), Decimal::new(500, 0));
    }

    #[test]
    fn test_skimming() {
        let anomaly = RetailAnomaly::Skimming {
            register_id: "R01".to_string(),
            store_id: "S001".to_string(),
            amount: Decimal::new(1000, 0),
            detection_method: "variance_analysis".to_string(),
        };

        assert_eq!(anomaly.severity(), 5);
        assert_eq!(anomaly.detection_difficulty(), "expert");
    }

    #[test]
    fn test_collusion() {
        let kickback = RetailAnomaly::VendorKickback {
            vendor_id: "V001".to_string(),
            buyer_id: "B001".to_string(),
            kickback_amount: Decimal::new(5000, 0),
            scheme_duration_days: 180,
        };

        assert!(kickback.involves_collusion());

        let skimming = RetailAnomaly::Skimming {
            register_id: "R01".to_string(),
            store_id: "S001".to_string(),
            amount: Decimal::new(500, 0),
            detection_method: "".to_string(),
        };

        assert!(!skimming.involves_collusion());
    }
}
