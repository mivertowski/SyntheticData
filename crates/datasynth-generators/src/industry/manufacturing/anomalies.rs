//! Manufacturing-specific anomalies.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::super::common::IndustryAnomaly;

/// Manufacturing-specific anomaly types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ManufacturingAnomaly {
    /// Reported yield is higher than actual (to meet targets).
    YieldManipulation {
        reported_yield: f64,
        actual_yield: f64,
        order_id: String,
    },
    /// Labor hours misallocated between orders.
    LaborMisallocation {
        from_order: String,
        to_order: String,
        hours: Decimal,
    },
    /// Production reported without actual production.
    PhantomProduction {
        order_id: String,
        quantity: u32,
        value: Decimal,
    },
    /// Obsolete inventory not written down.
    ObsoleteInventoryConcealment {
        material_id: String,
        age_months: u32,
        value: Decimal,
    },
    /// Standard costs inflated/deflated improperly.
    StandardCostManipulation {
        material_id: String,
        actual_market_cost: Decimal,
        recorded_standard: Decimal,
    },
    /// Overhead allocated to wrong products.
    OverheadMisallocation {
        from_product: String,
        to_product: String,
        amount: Decimal,
    },
    /// Scrap underreported to inflate yields.
    ScrapUnderreporting {
        order_id: String,
        actual_scrap: u32,
        reported_scrap: u32,
        value_hidden: Decimal,
    },
    /// WIP not properly transferred at period end.
    WipCutoffManipulation {
        order_id: String,
        completion_percentage: f64,
        recorded_percentage: f64,
    },
    /// Bill of materials quantity manipulation.
    BomQuantityManipulation {
        product_id: String,
        component_id: String,
        standard_qty: f64,
        actual_qty: f64,
    },
    /// Work center capacity overstated.
    CapacityOverstatement {
        work_center: String,
        actual_capacity: Decimal,
        reported_capacity: Decimal,
    },
    /// Inventory count manipulation.
    InventoryCountFraud {
        material_id: String,
        actual_quantity: u32,
        recorded_quantity: u32,
    },
    /// Quality records falsified.
    QualityRecordFalsification {
        order_id: String,
        actual_defect_rate: f64,
        reported_defect_rate: f64,
    },
}

impl IndustryAnomaly for ManufacturingAnomaly {
    fn anomaly_type(&self) -> &str {
        match self {
            ManufacturingAnomaly::YieldManipulation { .. } => "yield_manipulation",
            ManufacturingAnomaly::LaborMisallocation { .. } => "labor_misallocation",
            ManufacturingAnomaly::PhantomProduction { .. } => "phantom_production",
            ManufacturingAnomaly::ObsoleteInventoryConcealment { .. } => {
                "obsolete_inventory_concealment"
            }
            ManufacturingAnomaly::StandardCostManipulation { .. } => "standard_cost_manipulation",
            ManufacturingAnomaly::OverheadMisallocation { .. } => "overhead_misallocation",
            ManufacturingAnomaly::ScrapUnderreporting { .. } => "scrap_underreporting",
            ManufacturingAnomaly::WipCutoffManipulation { .. } => "wip_cutoff_manipulation",
            ManufacturingAnomaly::BomQuantityManipulation { .. } => "bom_quantity_manipulation",
            ManufacturingAnomaly::CapacityOverstatement { .. } => "capacity_overstatement",
            ManufacturingAnomaly::InventoryCountFraud { .. } => "inventory_count_fraud",
            ManufacturingAnomaly::QualityRecordFalsification { .. } => {
                "quality_record_falsification"
            }
        }
    }

    fn severity(&self) -> u8 {
        match self {
            ManufacturingAnomaly::LaborMisallocation { .. } => 3,
            ManufacturingAnomaly::OverheadMisallocation { .. } => 3,
            ManufacturingAnomaly::WipCutoffManipulation { .. } => 3,
            ManufacturingAnomaly::BomQuantityManipulation { .. } => 3,
            ManufacturingAnomaly::CapacityOverstatement { .. } => 3,
            ManufacturingAnomaly::YieldManipulation { .. } => 4,
            ManufacturingAnomaly::ScrapUnderreporting { .. } => 4,
            ManufacturingAnomaly::StandardCostManipulation { .. } => 4,
            ManufacturingAnomaly::ObsoleteInventoryConcealment { .. } => 4,
            ManufacturingAnomaly::PhantomProduction { .. } => 5,
            ManufacturingAnomaly::InventoryCountFraud { .. } => 5,
            ManufacturingAnomaly::QualityRecordFalsification { .. } => 5,
        }
    }

    fn detection_difficulty(&self) -> &str {
        match self {
            ManufacturingAnomaly::LaborMisallocation { .. } => "moderate",
            ManufacturingAnomaly::OverheadMisallocation { .. } => "moderate",
            ManufacturingAnomaly::BomQuantityManipulation { .. } => "moderate",
            ManufacturingAnomaly::YieldManipulation { .. } => "hard",
            ManufacturingAnomaly::ScrapUnderreporting { .. } => "hard",
            ManufacturingAnomaly::StandardCostManipulation { .. } => "hard",
            ManufacturingAnomaly::WipCutoffManipulation { .. } => "hard",
            ManufacturingAnomaly::CapacityOverstatement { .. } => "hard",
            ManufacturingAnomaly::ObsoleteInventoryConcealment { .. } => "hard",
            ManufacturingAnomaly::PhantomProduction { .. } => "expert",
            ManufacturingAnomaly::InventoryCountFraud { .. } => "expert",
            ManufacturingAnomaly::QualityRecordFalsification { .. } => "expert",
        }
    }

    fn indicators(&self) -> Vec<String> {
        match self {
            ManufacturingAnomaly::YieldManipulation { .. } => vec![
                "yield_exceeds_theoretical_maximum".to_string(),
                "yield_variance_from_historical".to_string(),
                "material_usage_inconsistent_with_yield".to_string(),
            ],
            ManufacturingAnomaly::PhantomProduction { .. } => vec![
                "production_without_material_issues".to_string(),
                "labor_hours_inconsistent_with_output".to_string(),
                "inventory_physical_count_variance".to_string(),
            ],
            ManufacturingAnomaly::ObsoleteInventoryConcealment { .. } => vec![
                "aged_inventory_without_reserve".to_string(),
                "no_movement_extended_period".to_string(),
                "market_price_below_cost".to_string(),
            ],
            ManufacturingAnomaly::StandardCostManipulation { .. } => vec![
                "standard_cost_variance_from_market".to_string(),
                "unusually_favorable_ppv".to_string(),
                "standard_cost_change_without_justification".to_string(),
            ],
            ManufacturingAnomaly::ScrapUnderreporting { .. } => vec![
                "scrap_rate_below_industry_average".to_string(),
                "material_variance_without_scrap".to_string(),
                "quality_rejects_not_matching_scrap".to_string(),
            ],
            ManufacturingAnomaly::InventoryCountFraud { .. } => vec![
                "count_adjusted_without_investigation".to_string(),
                "consistent_small_adjustments".to_string(),
                "cycle_count_variance_pattern".to_string(),
            ],
            ManufacturingAnomaly::QualityRecordFalsification { .. } => vec![
                "defect_rate_inconsistent_with_customer_returns".to_string(),
                "quality_records_modified_after_fact".to_string(),
                "statistical_process_control_anomalies".to_string(),
            ],
            _ => vec!["general_manufacturing_anomaly".to_string()],
        }
    }

    fn regulatory_concerns(&self) -> Vec<String> {
        match self {
            ManufacturingAnomaly::PhantomProduction { .. }
            | ManufacturingAnomaly::InventoryCountFraud { .. } => vec![
                "financial_statement_fraud".to_string(),
                "sox_section_302".to_string(),
                "sox_section_404".to_string(),
            ],
            ManufacturingAnomaly::ObsoleteInventoryConcealment { .. }
            | ManufacturingAnomaly::StandardCostManipulation { .. } => vec![
                "gaap_inventory_valuation".to_string(),
                "asc_330".to_string(),
                "ias_2".to_string(),
            ],
            ManufacturingAnomaly::QualityRecordFalsification { .. } => vec![
                "product_safety".to_string(),
                "fda_cgmp".to_string(),
                "iso_9001".to_string(),
            ],
            _ => vec!["general_gaap".to_string()],
        }
    }
}

impl ManufacturingAnomaly {
    /// Returns the financial impact of this anomaly.
    pub fn financial_impact(&self) -> Option<Decimal> {
        match self {
            ManufacturingAnomaly::PhantomProduction { value, .. } => Some(*value),
            ManufacturingAnomaly::ObsoleteInventoryConcealment { value, .. } => Some(*value),
            ManufacturingAnomaly::OverheadMisallocation { amount, .. } => Some(*amount),
            ManufacturingAnomaly::ScrapUnderreporting { value_hidden, .. } => Some(*value_hidden),
            ManufacturingAnomaly::StandardCostManipulation {
                actual_market_cost,
                recorded_standard,
                ..
            } => Some((*recorded_standard - *actual_market_cost).abs()),
            ManufacturingAnomaly::LaborMisallocation { hours, .. } => {
                // Assuming $25/hr labor rate
                Some(*hours * Decimal::new(25, 0))
            }
            _ => None,
        }
    }

    /// Returns whether this anomaly affects financial statements.
    pub fn affects_financials(&self) -> bool {
        !matches!(
            self,
            ManufacturingAnomaly::CapacityOverstatement { .. }
                | ManufacturingAnomaly::QualityRecordFalsification { .. }
        )
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_yield_manipulation() {
        let anomaly = ManufacturingAnomaly::YieldManipulation {
            reported_yield: 0.98,
            actual_yield: 0.92,
            order_id: "PO001".to_string(),
        };

        assert_eq!(anomaly.anomaly_type(), "yield_manipulation");
        assert_eq!(anomaly.severity(), 4);
        assert_eq!(anomaly.detection_difficulty(), "hard");
        assert!(!anomaly.indicators().is_empty());
    }

    #[test]
    fn test_phantom_production() {
        let anomaly = ManufacturingAnomaly::PhantomProduction {
            order_id: "PO002".to_string(),
            quantity: 100,
            value: Decimal::new(10_000, 0),
        };

        assert_eq!(anomaly.severity(), 5);
        assert_eq!(anomaly.detection_difficulty(), "expert");
        assert_eq!(anomaly.financial_impact(), Some(Decimal::new(10_000, 0)));
        assert!(anomaly.affects_financials());
    }

    #[test]
    fn test_inventory_fraud() {
        let anomaly = ManufacturingAnomaly::InventoryCountFraud {
            material_id: "MAT001".to_string(),
            actual_quantity: 100,
            recorded_quantity: 150,
        };

        assert!(anomaly
            .regulatory_concerns()
            .contains(&"sox_section_404".to_string()));
    }

    #[test]
    fn test_quality_falsification() {
        let anomaly = ManufacturingAnomaly::QualityRecordFalsification {
            order_id: "PO003".to_string(),
            actual_defect_rate: 0.08,
            reported_defect_rate: 0.02,
        };

        assert!(anomaly
            .regulatory_concerns()
            .contains(&"fda_cgmp".to_string()));
        assert!(!anomaly.affects_financials());
    }
}
