//! Manufacturing transaction types.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::super::common::{IndustryGlAccount, IndustryJournalLine, IndustryTransaction};

/// Production order type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProductionOrderType {
    /// Standard production order.
    Standard,
    /// Rework order for defective products.
    Rework,
    /// Prototype/engineering order.
    Prototype,
    /// Repair order for customer returns.
    Repair,
    /// Refurbishment order.
    Refurbishment,
}

impl ProductionOrderType {
    /// Returns the order type code.
    pub fn code(&self) -> &'static str {
        match self {
            ProductionOrderType::Standard => "STD",
            ProductionOrderType::Rework => "RWK",
            ProductionOrderType::Prototype => "PRT",
            ProductionOrderType::Repair => "REP",
            ProductionOrderType::Refurbishment => "RFB",
        }
    }
}

/// Scrap reason codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScrapReason {
    /// Material defect.
    MaterialDefect,
    /// Machine malfunction.
    MachineMalfunction,
    /// Operator error.
    OperatorError,
    /// Design issue.
    DesignIssue,
    /// Quality spec failure.
    QualityFailure,
    /// Contamination.
    Contamination,
    /// Obsolescence.
    Obsolescence,
    /// Damage in handling.
    HandlingDamage,
}

impl ScrapReason {
    /// Returns the reason code.
    pub fn code(&self) -> &'static str {
        match self {
            ScrapReason::MaterialDefect => "MAT",
            ScrapReason::MachineMalfunction => "MCH",
            ScrapReason::OperatorError => "OPR",
            ScrapReason::DesignIssue => "DES",
            ScrapReason::QualityFailure => "QUA",
            ScrapReason::Contamination => "CON",
            ScrapReason::Obsolescence => "OBS",
            ScrapReason::HandlingDamage => "HND",
        }
    }
}

/// Variance type for production cost analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VarianceType {
    /// Material price variance (actual vs standard price).
    MaterialPrice,
    /// Material usage variance (actual vs standard quantity).
    MaterialUsage,
    /// Labor rate variance.
    LaborRate,
    /// Labor efficiency variance.
    LaborEfficiency,
    /// Variable overhead spending variance.
    VariableOverheadSpending,
    /// Variable overhead efficiency variance.
    VariableOverheadEfficiency,
    /// Fixed overhead budget variance.
    FixedOverheadBudget,
    /// Fixed overhead volume variance.
    FixedOverheadVolume,
}

impl VarianceType {
    /// Returns the variance type code.
    pub fn code(&self) -> &'static str {
        match self {
            VarianceType::MaterialPrice => "MPV",
            VarianceType::MaterialUsage => "MUV",
            VarianceType::LaborRate => "LRV",
            VarianceType::LaborEfficiency => "LEV",
            VarianceType::VariableOverheadSpending => "VOSV",
            VarianceType::VariableOverheadEfficiency => "VOEV",
            VarianceType::FixedOverheadBudget => "FOBV",
            VarianceType::FixedOverheadVolume => "FOVV",
        }
    }

    /// Returns the GL account suffix for this variance.
    pub fn account_suffix(&self) -> &'static str {
        match self {
            VarianceType::MaterialPrice => "510",
            VarianceType::MaterialUsage => "520",
            VarianceType::LaborRate => "530",
            VarianceType::LaborEfficiency => "540",
            VarianceType::VariableOverheadSpending => "550",
            VarianceType::VariableOverheadEfficiency => "560",
            VarianceType::FixedOverheadBudget => "570",
            VarianceType::FixedOverheadVolume => "580",
        }
    }
}

/// Manufacturing transaction types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ManufacturingTransaction {
    // ========== Production Transactions ==========
    /// Creation of a production order.
    WorkOrderIssuance {
        order_id: String,
        product_id: String,
        quantity: u32,
        order_type: ProductionOrderType,
        date: NaiveDate,
    },
    /// Issuance of materials to production.
    MaterialRequisition {
        order_id: String,
        materials: Vec<MaterialLine>,
        date: NaiveDate,
    },
    /// Posting of labor hours to production.
    LaborBooking {
        order_id: String,
        work_center: String,
        hours: Decimal,
        labor_rate: Decimal,
        date: NaiveDate,
    },
    /// Overhead absorption posting.
    OverheadAbsorption {
        order_id: String,
        absorption_rate: Decimal,
        base_amount: Decimal,
        date: NaiveDate,
    },
    /// Scrap reporting.
    ScrapReporting {
        order_id: String,
        material_id: String,
        quantity: u32,
        reason: ScrapReason,
        scrap_value: Decimal,
        date: NaiveDate,
    },
    /// Rework order creation.
    ReworkOrder {
        original_order_id: String,
        rework_order_id: String,
        quantity: u32,
        estimated_cost: Decimal,
        date: NaiveDate,
    },
    /// Production variance posting.
    ProductionVariance {
        order_id: String,
        variance_type: VarianceType,
        amount: Decimal,
        date: NaiveDate,
    },
    /// Completion of production order.
    ProductionCompletion {
        order_id: String,
        product_id: String,
        quantity_completed: u32,
        total_cost: Decimal,
        date: NaiveDate,
    },

    // ========== Inventory Transactions ==========
    /// Receipt of raw materials.
    RawMaterialReceipt {
        po_id: String,
        material_id: String,
        quantity: u32,
        unit_cost: Decimal,
        date: NaiveDate,
    },
    /// Transfer between production stages.
    WipTransfer {
        from_center: String,
        to_center: String,
        order_id: String,
        quantity: u32,
        value: Decimal,
        date: NaiveDate,
    },
    /// Transfer of finished goods to inventory.
    FinishedGoodsTransfer {
        order_id: String,
        product_id: String,
        quantity: u32,
        location: String,
        unit_cost: Decimal,
        date: NaiveDate,
    },
    /// Cycle count adjustment.
    CycleCountAdjustment {
        material_id: String,
        location: String,
        variance_quantity: i32,
        unit_cost: Decimal,
        date: NaiveDate,
    },

    // ========== Costing Transactions ==========
    /// Standard cost revaluation.
    StandardCostRevaluation {
        material_id: String,
        old_cost: Decimal,
        new_cost: Decimal,
        inventory_quantity: u32,
        date: NaiveDate,
    },
    /// Purchase price variance.
    PurchasePriceVariance {
        material_id: String,
        po_id: String,
        standard_cost: Decimal,
        actual_cost: Decimal,
        quantity: u32,
        date: NaiveDate,
    },
}

/// Material line in a requisition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialLine {
    /// Material ID.
    pub material_id: String,
    /// Quantity issued.
    pub quantity: f64,
    /// Unit of measure.
    pub unit_of_measure: String,
    /// Standard cost per unit.
    pub standard_cost: Decimal,
    /// Storage location.
    pub location: String,
}

impl IndustryTransaction for ManufacturingTransaction {
    fn transaction_type(&self) -> &str {
        match self {
            ManufacturingTransaction::WorkOrderIssuance { .. } => "work_order_issuance",
            ManufacturingTransaction::MaterialRequisition { .. } => "material_requisition",
            ManufacturingTransaction::LaborBooking { .. } => "labor_booking",
            ManufacturingTransaction::OverheadAbsorption { .. } => "overhead_absorption",
            ManufacturingTransaction::ScrapReporting { .. } => "scrap_reporting",
            ManufacturingTransaction::ReworkOrder { .. } => "rework_order",
            ManufacturingTransaction::ProductionVariance { .. } => "production_variance",
            ManufacturingTransaction::ProductionCompletion { .. } => "production_completion",
            ManufacturingTransaction::RawMaterialReceipt { .. } => "raw_material_receipt",
            ManufacturingTransaction::WipTransfer { .. } => "wip_transfer",
            ManufacturingTransaction::FinishedGoodsTransfer { .. } => "finished_goods_transfer",
            ManufacturingTransaction::CycleCountAdjustment { .. } => "cycle_count_adjustment",
            ManufacturingTransaction::StandardCostRevaluation { .. } => "standard_cost_revaluation",
            ManufacturingTransaction::PurchasePriceVariance { .. } => "purchase_price_variance",
        }
    }

    fn date(&self) -> NaiveDate {
        match self {
            ManufacturingTransaction::WorkOrderIssuance { date, .. }
            | ManufacturingTransaction::MaterialRequisition { date, .. }
            | ManufacturingTransaction::LaborBooking { date, .. }
            | ManufacturingTransaction::OverheadAbsorption { date, .. }
            | ManufacturingTransaction::ScrapReporting { date, .. }
            | ManufacturingTransaction::ReworkOrder { date, .. }
            | ManufacturingTransaction::ProductionVariance { date, .. }
            | ManufacturingTransaction::ProductionCompletion { date, .. }
            | ManufacturingTransaction::RawMaterialReceipt { date, .. }
            | ManufacturingTransaction::WipTransfer { date, .. }
            | ManufacturingTransaction::FinishedGoodsTransfer { date, .. }
            | ManufacturingTransaction::CycleCountAdjustment { date, .. }
            | ManufacturingTransaction::StandardCostRevaluation { date, .. }
            | ManufacturingTransaction::PurchasePriceVariance { date, .. } => *date,
        }
    }

    fn amount(&self) -> Option<Decimal> {
        match self {
            ManufacturingTransaction::LaborBooking {
                hours, labor_rate, ..
            } => Some(*hours * *labor_rate),
            ManufacturingTransaction::ScrapReporting { scrap_value, .. } => Some(*scrap_value),
            ManufacturingTransaction::ProductionVariance { amount, .. } => Some(*amount),
            ManufacturingTransaction::ProductionCompletion { total_cost, .. } => Some(*total_cost),
            ManufacturingTransaction::RawMaterialReceipt {
                quantity,
                unit_cost,
                ..
            } => Some(Decimal::from(*quantity) * *unit_cost),
            ManufacturingTransaction::WipTransfer { value, .. } => Some(*value),
            ManufacturingTransaction::FinishedGoodsTransfer {
                quantity,
                unit_cost,
                ..
            } => Some(Decimal::from(*quantity) * *unit_cost),
            ManufacturingTransaction::CycleCountAdjustment {
                variance_quantity,
                unit_cost,
                ..
            } => Some(Decimal::from(*variance_quantity) * *unit_cost),
            ManufacturingTransaction::StandardCostRevaluation {
                old_cost,
                new_cost,
                inventory_quantity,
                ..
            } => Some((*new_cost - *old_cost) * Decimal::from(*inventory_quantity)),
            ManufacturingTransaction::PurchasePriceVariance {
                standard_cost,
                actual_cost,
                quantity,
                ..
            } => Some((*actual_cost - *standard_cost) * Decimal::from(*quantity)),
            _ => None,
        }
    }

    fn accounts(&self) -> Vec<String> {
        match self {
            ManufacturingTransaction::MaterialRequisition { .. } => {
                vec!["1400".to_string(), "1300".to_string()] // WIP, Raw Materials
            }
            ManufacturingTransaction::LaborBooking { .. } => {
                vec!["1400".to_string(), "2100".to_string()] // WIP, Wages Payable
            }
            ManufacturingTransaction::OverheadAbsorption { .. } => {
                vec!["1400".to_string(), "5400".to_string()] // WIP, Overhead Applied
            }
            ManufacturingTransaction::ScrapReporting { .. } => {
                vec!["5200".to_string(), "1400".to_string()] // Scrap Loss, WIP
            }
            ManufacturingTransaction::ProductionVariance { variance_type, .. } => {
                vec![
                    format!("5{}", variance_type.account_suffix()),
                    "1400".to_string(),
                ]
            }
            ManufacturingTransaction::ProductionCompletion { .. } => {
                vec!["1500".to_string(), "1400".to_string()] // FG Inventory, WIP
            }
            ManufacturingTransaction::RawMaterialReceipt { .. } => {
                vec!["1300".to_string(), "2000".to_string()] // Raw Materials, AP
            }
            ManufacturingTransaction::FinishedGoodsTransfer { .. } => {
                vec!["1500".to_string(), "1400".to_string()] // FG Inventory, WIP
            }
            ManufacturingTransaction::CycleCountAdjustment { .. } => {
                vec!["5300".to_string(), "1300".to_string()] // Inventory Adjustment, Inventory
            }
            ManufacturingTransaction::StandardCostRevaluation { .. } => {
                vec!["1300".to_string(), "5510".to_string()] // Inventory, Revaluation
            }
            ManufacturingTransaction::PurchasePriceVariance { .. } => {
                vec!["5510".to_string(), "2000".to_string()] // PPV, AP
            }
            _ => Vec::new(),
        }
    }

    fn to_journal_lines(&self) -> Vec<IndustryJournalLine> {
        match self {
            ManufacturingTransaction::MaterialRequisition { materials, .. } => {
                let total: Decimal = materials
                    .iter()
                    .map(|m| {
                        m.standard_cost
                            * Decimal::from_f64_retain(m.quantity).unwrap_or(Decimal::ONE)
                    })
                    .sum();

                vec![
                    IndustryJournalLine::debit("1400", total, "WIP - Material Issue"),
                    IndustryJournalLine::credit("1300", total, "Raw Materials Inventory"),
                ]
            }
            ManufacturingTransaction::LaborBooking {
                hours,
                labor_rate,
                work_center,
                ..
            } => {
                let amount = *hours * *labor_rate;
                vec![
                    IndustryJournalLine::debit("1400", amount, "WIP - Direct Labor")
                        .with_cost_center(work_center),
                    IndustryJournalLine::credit("2100", amount, "Wages Payable"),
                ]
            }
            ManufacturingTransaction::ProductionCompletion {
                total_cost,
                product_id,
                quantity_completed,
                ..
            } => {
                vec![
                    IndustryJournalLine::debit("1500", *total_cost, "Finished Goods Inventory")
                        .with_dimension("product", product_id)
                        .with_dimension("quantity", quantity_completed.to_string()),
                    IndustryJournalLine::credit("1400", *total_cost, "WIP - Completion"),
                ]
            }
            ManufacturingTransaction::ProductionVariance {
                variance_type,
                amount,
                order_id,
                ..
            } => {
                let account = format!("5{}", variance_type.account_suffix());
                let desc = format!("{:?} - Order {}", variance_type, order_id);

                if *amount >= Decimal::ZERO {
                    vec![
                        IndustryJournalLine::debit(&account, *amount, &desc),
                        IndustryJournalLine::credit("1400", *amount, "WIP - Variance"),
                    ]
                } else {
                    vec![
                        IndustryJournalLine::debit("1400", amount.abs(), "WIP - Variance"),
                        IndustryJournalLine::credit(&account, amount.abs(), &desc),
                    ]
                }
            }
            _ => Vec::new(),
        }
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("industry".to_string(), "manufacturing".to_string());
        meta.insert(
            "transaction_type".to_string(),
            self.transaction_type().to_string(),
        );
        meta
    }
}

/// Generator for manufacturing transactions.
#[derive(Debug, Clone)]
pub struct ManufacturingTransactionGenerator {
    /// Production order types to generate.
    pub order_types: Vec<ProductionOrderType>,
    /// Average orders per day.
    pub avg_orders_per_day: f64,
    /// Average materials per order.
    pub avg_materials_per_order: u32,
    /// Scrap rate (0.0-1.0).
    pub scrap_rate: f64,
    /// Variance rate (0.0-1.0).
    pub variance_rate: f64,
}

impl Default for ManufacturingTransactionGenerator {
    fn default() -> Self {
        Self {
            order_types: vec![ProductionOrderType::Standard, ProductionOrderType::Rework],
            avg_orders_per_day: 5.0,
            avg_materials_per_order: 4,
            scrap_rate: 0.02,
            variance_rate: 0.15,
        }
    }
}

impl ManufacturingTransactionGenerator {
    /// Returns manufacturing-specific GL accounts.
    pub fn gl_accounts() -> Vec<IndustryGlAccount> {
        vec![
            IndustryGlAccount::new("1300", "Raw Materials Inventory", "Asset", "Inventory")
                .into_control(),
            IndustryGlAccount::new("1400", "Work in Process", "Asset", "Inventory").into_control(),
            IndustryGlAccount::new("1500", "Finished Goods Inventory", "Asset", "Inventory")
                .into_control(),
            IndustryGlAccount::new("5100", "Cost of Goods Sold", "Expense", "COGS"),
            IndustryGlAccount::new("5200", "Scrap and Spoilage", "Expense", "Manufacturing"),
            IndustryGlAccount::new("5300", "Inventory Adjustments", "Expense", "Manufacturing"),
            IndustryGlAccount::new(
                "5400",
                "Manufacturing Overhead Applied",
                "Expense",
                "Overhead",
            )
            .with_normal_balance("Credit"),
            IndustryGlAccount::new("5510", "Material Price Variance", "Expense", "Variance"),
            IndustryGlAccount::new("5520", "Material Usage Variance", "Expense", "Variance"),
            IndustryGlAccount::new("5530", "Labor Rate Variance", "Expense", "Variance"),
            IndustryGlAccount::new("5540", "Labor Efficiency Variance", "Expense", "Variance"),
            IndustryGlAccount::new("5550", "Variable OH Spending Var", "Expense", "Variance"),
            IndustryGlAccount::new("5560", "Variable OH Efficiency Var", "Expense", "Variance"),
            IndustryGlAccount::new("5570", "Fixed OH Budget Variance", "Expense", "Variance"),
            IndustryGlAccount::new("5580", "Fixed OH Volume Variance", "Expense", "Variance"),
        ]
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_production_order_type() {
        let std = ProductionOrderType::Standard;
        assert_eq!(std.code(), "STD");

        let rework = ProductionOrderType::Rework;
        assert_eq!(rework.code(), "RWK");
    }

    #[test]
    fn test_variance_type() {
        let mpv = VarianceType::MaterialPrice;
        assert_eq!(mpv.code(), "MPV");
        assert_eq!(mpv.account_suffix(), "510");
    }

    #[test]
    fn test_manufacturing_transaction() {
        let tx = ManufacturingTransaction::ProductionCompletion {
            order_id: "PO001".to_string(),
            product_id: "FG001".to_string(),
            quantity_completed: 100,
            total_cost: Decimal::new(5000, 0),
            date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        };

        assert_eq!(tx.transaction_type(), "production_completion");
        assert_eq!(tx.amount(), Some(Decimal::new(5000, 0)));
        assert_eq!(tx.accounts().len(), 2);
    }

    #[test]
    fn test_journal_lines() {
        let tx = ManufacturingTransaction::ProductionVariance {
            order_id: "PO001".to_string(),
            variance_type: VarianceType::MaterialPrice,
            amount: Decimal::new(500, 0),
            date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        };

        let lines = tx.to_journal_lines();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].debit, Decimal::new(500, 0));
        assert_eq!(lines[1].credit, Decimal::new(500, 0));
    }

    #[test]
    fn test_gl_accounts() {
        let accounts = ManufacturingTransactionGenerator::gl_accounts();
        assert!(accounts.len() >= 10);

        let wip = accounts.iter().find(|a| a.account_number == "1400");
        assert!(wip.is_some());
        assert!(wip.unwrap().is_control);
    }
}
