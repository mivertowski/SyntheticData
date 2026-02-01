//! Retail transaction types.

use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::super::common::{IndustryGlAccount, IndustryJournalLine, IndustryTransaction};

/// Point of sale transaction types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PosTransaction {
    /// Standard sale.
    Sale {
        transaction_id: String,
        store_id: String,
        register_id: String,
        cashier_id: String,
        items: Vec<SaleItem>,
        subtotal: Decimal,
        tax: Decimal,
        total: Decimal,
        payment_method: String,
        timestamp: NaiveDateTime,
    },
    /// Customer return.
    Return {
        transaction_id: String,
        original_transaction_id: String,
        store_id: String,
        register_id: String,
        cashier_id: String,
        items: Vec<ReturnItem>,
        refund_amount: Decimal,
        refund_method: String,
        reason_code: String,
        timestamp: NaiveDateTime,
    },
    /// Voided transaction.
    Void {
        transaction_id: String,
        voided_transaction_id: String,
        store_id: String,
        register_id: String,
        cashier_id: String,
        supervisor_id: Option<String>,
        void_reason: String,
        original_amount: Decimal,
        timestamp: NaiveDateTime,
    },
    /// Price override.
    PriceOverride {
        transaction_id: String,
        item_sku: String,
        original_price: Decimal,
        override_price: Decimal,
        reason_code: String,
        approver_id: Option<String>,
        timestamp: NaiveDateTime,
    },
    /// Employee discount applied.
    EmployeeDiscount {
        transaction_id: String,
        employee_id: String,
        discount_amount: Decimal,
        beneficiary_relationship: String,
        timestamp: NaiveDateTime,
    },
    /// Loyalty redemption.
    LoyaltyRedemption {
        transaction_id: String,
        customer_id: String,
        points_redeemed: u32,
        value_redeemed: Decimal,
        timestamp: NaiveDateTime,
    },
}

/// Sale item line.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaleItem {
    /// SKU number.
    pub sku: String,
    /// Product name.
    pub product_name: String,
    /// Quantity sold.
    pub quantity: u32,
    /// Unit price.
    pub unit_price: Decimal,
    /// Discount amount.
    pub discount: Decimal,
    /// Line total.
    pub line_total: Decimal,
    /// Department code.
    pub department: String,
    /// Category.
    pub category: String,
}

/// Return item line.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnItem {
    /// SKU number.
    pub sku: String,
    /// Quantity returned.
    pub quantity: u32,
    /// Refund price per unit.
    pub refund_price: Decimal,
    /// Return reason.
    pub reason: String,
    /// Condition (new, damaged, etc.).
    pub condition: String,
    /// Whether item is restockable.
    pub restockable: bool,
}

/// Inventory transaction types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InventoryTransaction {
    /// Inventory receipt.
    Receipt {
        receipt_id: String,
        po_id: String,
        store_id: String,
        items: Vec<ReceiptItem>,
        received_by: String,
        date: NaiveDate,
    },
    /// Inter-store transfer.
    Transfer {
        transfer_id: String,
        from_store: String,
        to_store: String,
        items: Vec<TransferItem>,
        status: String,
        date: NaiveDate,
    },
    /// Physical count adjustment.
    CountAdjustment {
        adjustment_id: String,
        store_id: String,
        sku: String,
        system_quantity: i32,
        physical_quantity: i32,
        variance: i32,
        unit_cost: Decimal,
        reason_code: String,
        approved_by: Option<String>,
        date: NaiveDate,
    },
    /// Shrinkage write-off.
    ShrinkageWriteOff {
        writeoff_id: String,
        store_id: String,
        category: String,
        amount: Decimal,
        reason: ShrinkageReason,
        date: NaiveDate,
    },
    /// Markdown.
    Markdown {
        markdown_id: String,
        store_id: String,
        sku: String,
        original_price: Decimal,
        markdown_price: Decimal,
        quantity_affected: u32,
        reason: String,
        date: NaiveDate,
    },
}

/// Receipt item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptItem {
    /// SKU number.
    pub sku: String,
    /// Quantity received.
    pub quantity_received: u32,
    /// Quantity ordered.
    pub quantity_ordered: u32,
    /// Unit cost.
    pub unit_cost: Decimal,
}

/// Transfer item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferItem {
    /// SKU number.
    pub sku: String,
    /// Quantity.
    pub quantity: u32,
    /// Unit cost.
    pub unit_cost: Decimal,
}

/// Shrinkage reason codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShrinkageReason {
    /// Employee theft.
    EmployeeTheft,
    /// External theft (shoplifting).
    ExternalTheft,
    /// Administrative error.
    AdminError,
    /// Vendor fraud.
    VendorFraud,
    /// Damage/spoilage.
    Damage,
    /// Unknown.
    Unknown,
}

impl ShrinkageReason {
    /// Returns the reason code.
    pub fn code(&self) -> &'static str {
        match self {
            ShrinkageReason::EmployeeTheft => "EMP",
            ShrinkageReason::ExternalTheft => "EXT",
            ShrinkageReason::AdminError => "ADM",
            ShrinkageReason::VendorFraud => "VND",
            ShrinkageReason::Damage => "DMG",
            ShrinkageReason::Unknown => "UNK",
        }
    }
}

/// Union type for all retail transactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetailTransaction {
    /// Point of sale transaction.
    Pos(PosTransaction),
    /// Inventory transaction.
    Inventory(InventoryTransaction),
}

impl IndustryTransaction for RetailTransaction {
    fn transaction_type(&self) -> &str {
        match self {
            RetailTransaction::Pos(pos) => match pos {
                PosTransaction::Sale { .. } => "pos_sale",
                PosTransaction::Return { .. } => "pos_return",
                PosTransaction::Void { .. } => "pos_void",
                PosTransaction::PriceOverride { .. } => "price_override",
                PosTransaction::EmployeeDiscount { .. } => "employee_discount",
                PosTransaction::LoyaltyRedemption { .. } => "loyalty_redemption",
            },
            RetailTransaction::Inventory(inv) => match inv {
                InventoryTransaction::Receipt { .. } => "inventory_receipt",
                InventoryTransaction::Transfer { .. } => "inventory_transfer",
                InventoryTransaction::CountAdjustment { .. } => "count_adjustment",
                InventoryTransaction::ShrinkageWriteOff { .. } => "shrinkage_writeoff",
                InventoryTransaction::Markdown { .. } => "markdown",
            },
        }
    }

    fn date(&self) -> NaiveDate {
        match self {
            RetailTransaction::Pos(pos) => match pos {
                PosTransaction::Sale { timestamp, .. }
                | PosTransaction::Return { timestamp, .. }
                | PosTransaction::Void { timestamp, .. }
                | PosTransaction::PriceOverride { timestamp, .. }
                | PosTransaction::EmployeeDiscount { timestamp, .. }
                | PosTransaction::LoyaltyRedemption { timestamp, .. } => timestamp.date(),
            },
            RetailTransaction::Inventory(inv) => match inv {
                InventoryTransaction::Receipt { date, .. }
                | InventoryTransaction::Transfer { date, .. }
                | InventoryTransaction::CountAdjustment { date, .. }
                | InventoryTransaction::ShrinkageWriteOff { date, .. }
                | InventoryTransaction::Markdown { date, .. } => *date,
            },
        }
    }

    fn amount(&self) -> Option<Decimal> {
        match self {
            RetailTransaction::Pos(pos) => match pos {
                PosTransaction::Sale { total, .. } => Some(*total),
                PosTransaction::Return { refund_amount, .. } => Some(*refund_amount),
                PosTransaction::Void {
                    original_amount, ..
                } => Some(*original_amount),
                PosTransaction::PriceOverride {
                    original_price,
                    override_price,
                    ..
                } => Some(*original_price - *override_price),
                PosTransaction::EmployeeDiscount {
                    discount_amount, ..
                } => Some(*discount_amount),
                PosTransaction::LoyaltyRedemption { value_redeemed, .. } => Some(*value_redeemed),
            },
            RetailTransaction::Inventory(inv) => match inv {
                InventoryTransaction::ShrinkageWriteOff { amount, .. } => Some(*amount),
                InventoryTransaction::CountAdjustment {
                    variance,
                    unit_cost,
                    ..
                } => Some(Decimal::from(*variance) * *unit_cost),
                _ => None,
            },
        }
    }

    fn accounts(&self) -> Vec<String> {
        match self {
            RetailTransaction::Pos(pos) => match pos {
                PosTransaction::Sale { .. } => {
                    vec!["1100".to_string(), "4100".to_string(), "2300".to_string()]
                }
                PosTransaction::Return { .. } => {
                    vec!["4200".to_string(), "1100".to_string()]
                }
                _ => Vec::new(),
            },
            RetailTransaction::Inventory(inv) => match inv {
                InventoryTransaction::ShrinkageWriteOff { .. } => {
                    vec!["5300".to_string(), "1400".to_string()]
                }
                InventoryTransaction::CountAdjustment { .. } => {
                    vec!["5310".to_string(), "1400".to_string()]
                }
                _ => Vec::new(),
            },
        }
    }

    fn to_journal_lines(&self) -> Vec<IndustryJournalLine> {
        match self {
            RetailTransaction::Pos(PosTransaction::Sale {
                total,
                tax,
                store_id,
                ..
            }) => {
                let pretax = *total - *tax;
                vec![
                    IndustryJournalLine::debit("1100", *total, "Cash/AR from sales")
                        .with_dimension("store", store_id),
                    IndustryJournalLine::credit("4100", pretax, "Sales Revenue"),
                    IndustryJournalLine::credit("2300", *tax, "Sales Tax Payable"),
                ]
            }
            RetailTransaction::Pos(PosTransaction::Return { refund_amount, .. }) => {
                vec![
                    IndustryJournalLine::debit("4200", *refund_amount, "Sales Returns"),
                    IndustryJournalLine::credit("1100", *refund_amount, "Cash/AR refund"),
                ]
            }
            RetailTransaction::Inventory(InventoryTransaction::ShrinkageWriteOff {
                amount,
                reason,
                store_id,
                ..
            }) => {
                vec![
                    IndustryJournalLine::debit(
                        "5300",
                        *amount,
                        format!("Shrinkage - {:?}", reason),
                    )
                    .with_dimension("store", store_id),
                    IndustryJournalLine::credit("1400", *amount, "Inventory reduction"),
                ]
            }
            _ => Vec::new(),
        }
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("industry".to_string(), "retail".to_string());
        meta.insert(
            "transaction_type".to_string(),
            self.transaction_type().to_string(),
        );
        meta
    }
}

/// Generator for retail transactions.
#[derive(Debug, Clone)]
pub struct RetailTransactionGenerator {
    /// Average transactions per store per day.
    pub avg_daily_transactions: u32,
    /// Return rate (0.0-1.0).
    pub return_rate: f64,
    /// Void rate (0.0-1.0).
    pub void_rate: f64,
    /// Price override rate (0.0-1.0).
    pub override_rate: f64,
    /// Shrinkage rate (0.0-1.0).
    pub shrinkage_rate: f64,
}

impl Default for RetailTransactionGenerator {
    fn default() -> Self {
        Self {
            avg_daily_transactions: 200,
            return_rate: 0.08,
            void_rate: 0.02,
            override_rate: 0.05,
            shrinkage_rate: 0.015,
        }
    }
}

impl RetailTransactionGenerator {
    /// Returns retail-specific GL accounts.
    pub fn gl_accounts() -> Vec<IndustryGlAccount> {
        vec![
            IndustryGlAccount::new("1100", "Cash and Cash Equivalents", "Asset", "Cash")
                .into_control(),
            IndustryGlAccount::new("1400", "Merchandise Inventory", "Asset", "Inventory")
                .into_control(),
            IndustryGlAccount::new("2300", "Sales Tax Payable", "Liability", "Tax")
                .with_normal_balance("Credit"),
            IndustryGlAccount::new("4100", "Sales Revenue", "Revenue", "Sales")
                .with_normal_balance("Credit"),
            IndustryGlAccount::new("4200", "Sales Returns and Allowances", "Revenue", "Sales"),
            IndustryGlAccount::new("4300", "Sales Discounts", "Revenue", "Sales"),
            IndustryGlAccount::new("5100", "Cost of Goods Sold", "Expense", "COGS"),
            IndustryGlAccount::new("5200", "Freight In", "Expense", "COGS"),
            IndustryGlAccount::new("5300", "Inventory Shrinkage", "Expense", "Shrinkage"),
            IndustryGlAccount::new("5310", "Inventory Adjustments", "Expense", "Shrinkage"),
            IndustryGlAccount::new("5400", "Markdown Expense", "Expense", "Markdown"),
            IndustryGlAccount::new("5500", "Employee Discount Expense", "Expense", "Discount"),
            IndustryGlAccount::new("5600", "Loyalty Program Expense", "Expense", "Loyalty"),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pos_sale() {
        let timestamp = NaiveDate::from_ymd_opt(2024, 1, 15)
            .unwrap()
            .and_hms_opt(14, 30, 0)
            .unwrap();

        let tx = RetailTransaction::Pos(PosTransaction::Sale {
            transaction_id: "TRX001".to_string(),
            store_id: "S001".to_string(),
            register_id: "R01".to_string(),
            cashier_id: "C001".to_string(),
            items: vec![SaleItem {
                sku: "SKU001".to_string(),
                product_name: "Widget".to_string(),
                quantity: 2,
                unit_price: Decimal::new(1999, 2),
                discount: Decimal::ZERO,
                line_total: Decimal::new(3998, 2),
                department: "D001".to_string(),
                category: "Widgets".to_string(),
            }],
            subtotal: Decimal::new(3998, 2),
            tax: Decimal::new(320, 2),
            total: Decimal::new(4318, 2),
            payment_method: "credit_card".to_string(),
            timestamp,
        });

        assert_eq!(tx.transaction_type(), "pos_sale");
        assert_eq!(tx.amount(), Some(Decimal::new(4318, 2)));
        assert_eq!(tx.accounts().len(), 3);
    }

    #[test]
    fn test_shrinkage_writeoff() {
        let tx = RetailTransaction::Inventory(InventoryTransaction::ShrinkageWriteOff {
            writeoff_id: "WO001".to_string(),
            store_id: "S001".to_string(),
            category: "Electronics".to_string(),
            amount: Decimal::new(500, 0),
            reason: ShrinkageReason::ExternalTheft,
            date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        });

        assert_eq!(tx.transaction_type(), "shrinkage_writeoff");
        assert_eq!(tx.amount(), Some(Decimal::new(500, 0)));

        let lines = tx.to_journal_lines();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].debit, Decimal::new(500, 0));
    }

    #[test]
    fn test_gl_accounts() {
        let accounts = RetailTransactionGenerator::gl_accounts();
        assert!(accounts.len() >= 10);

        let inventory = accounts.iter().find(|a| a.account_number == "1400");
        assert!(inventory.is_some());
    }
}
