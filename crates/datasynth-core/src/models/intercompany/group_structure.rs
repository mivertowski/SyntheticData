//! Group structure ownership models for consolidated financial reporting.
//!
//! This module provides models for capturing parent-subsidiary relationships,
//! ownership percentages, and consolidation methods. It feeds into ISA 600
//! (component auditor scope), consolidated financial statements, and NCI
//! (non-controlling interest) calculations.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Complete ownership/consolidation structure for a corporate group.
///
/// Captures the parent entity and all subsidiaries and associates, with their
/// respective ownership percentages and consolidation methods.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupStructure {
    /// Code of the ultimate parent entity in the group.
    pub parent_entity: String,
    /// Subsidiary relationships (>50% owned or otherwise controlled entities).
    pub subsidiaries: Vec<SubsidiaryRelationship>,
    /// Associate relationships (20–50% owned entities, significant influence).
    pub associates: Vec<AssociateRelationship>,
}

impl GroupStructure {
    /// Create a new group structure with the given parent entity.
    pub fn new(parent_entity: String) -> Self {
        Self {
            parent_entity,
            subsidiaries: Vec::new(),
            associates: Vec::new(),
        }
    }

    /// Add a subsidiary relationship.
    pub fn add_subsidiary(&mut self, subsidiary: SubsidiaryRelationship) {
        self.subsidiaries.push(subsidiary);
    }

    /// Add an associate relationship.
    pub fn add_associate(&mut self, associate: AssociateRelationship) {
        self.associates.push(associate);
    }

    /// Return the total number of entities in the group (parent + subs + associates).
    pub fn entity_count(&self) -> usize {
        1 + self.subsidiaries.len() + self.associates.len()
    }
}

/// Relationship between the group parent and a subsidiary entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubsidiaryRelationship {
    /// Entity code of the subsidiary.
    pub entity_code: String,
    /// Percentage of shares held by the parent (0–100).
    pub ownership_percentage: Decimal,
    /// Percentage of voting rights held by the parent (0–100).
    pub voting_rights_percentage: Decimal,
    /// Accounting consolidation method applied to this subsidiary.
    pub consolidation_method: GroupConsolidationMethod,
    /// Date the parent acquired control of this subsidiary.
    pub acquisition_date: Option<NaiveDate>,
    /// Non-controlling interest percentage (= 100 − ownership_percentage).
    pub nci_percentage: Decimal,
    /// Functional currency code of the subsidiary (e.g. "USD", "EUR").
    pub functional_currency: String,
}

impl SubsidiaryRelationship {
    /// Create a fully-owned (100 %) subsidiary with full consolidation.
    pub fn new_full(entity_code: String, functional_currency: String) -> Self {
        Self {
            entity_code,
            ownership_percentage: Decimal::from(100),
            voting_rights_percentage: Decimal::from(100),
            consolidation_method: GroupConsolidationMethod::FullConsolidation,
            acquisition_date: None,
            nci_percentage: Decimal::ZERO,
            functional_currency,
        }
    }

    /// Create a subsidiary with a specified ownership percentage.
    ///
    /// The consolidation method and NCI are derived automatically from the
    /// ownership percentage using IFRS 10 / IAS 28 thresholds.
    pub fn new_with_ownership(
        entity_code: String,
        ownership_percentage: Decimal,
        functional_currency: String,
        acquisition_date: Option<NaiveDate>,
    ) -> Self {
        let consolidation_method = GroupConsolidationMethod::from_ownership(ownership_percentage);
        let nci_percentage = Decimal::from(100) - ownership_percentage;
        Self {
            entity_code,
            ownership_percentage,
            voting_rights_percentage: ownership_percentage,
            consolidation_method,
            acquisition_date,
            nci_percentage,
            functional_currency,
        }
    }
}

/// Consolidation method applied to a subsidiary or investee.
///
/// Distinct from the existing [`super::ConsolidationMethod`] in that it uses
/// IFRS-aligned terminology and adds a `FairValue` option for FVTPL investments.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupConsolidationMethod {
    /// Full line-by-line consolidation (IFRS 10, >50 % ownership / control).
    FullConsolidation,
    /// Equity method (IAS 28, 20–50 % ownership, significant influence).
    EquityMethod,
    /// Fair value through profit or loss (<20 % ownership, no influence).
    FairValue,
}

impl GroupConsolidationMethod {
    /// Derive the consolidation method from the ownership percentage.
    ///
    /// Uses standard IFRS 10 / IAS 28 thresholds:
    /// - > 50 % → FullConsolidation
    /// - 20–50 % → EquityMethod
    /// - < 20 % → FairValue
    pub fn from_ownership(ownership_pct: Decimal) -> Self {
        if ownership_pct > Decimal::from(50) {
            Self::FullConsolidation
        } else if ownership_pct >= Decimal::from(20) {
            Self::EquityMethod
        } else {
            Self::FairValue
        }
    }
}

/// Relationship between the group parent and an associate entity.
///
/// Associates are accounted for under the equity method (IAS 28).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssociateRelationship {
    /// Entity code of the associate.
    pub entity_code: String,
    /// Percentage of shares held by the investor (typically 20–50 %).
    pub ownership_percentage: Decimal,
    /// Share of the associate's profit/(loss) recognised in the period.
    pub equity_pickup: Decimal,
}

impl AssociateRelationship {
    /// Create a new associate relationship with zero equity pickup.
    pub fn new(entity_code: String, ownership_percentage: Decimal) -> Self {
        Self {
            entity_code,
            ownership_percentage,
            equity_pickup: Decimal::ZERO,
        }
    }
}

// ---------------------------------------------------------------------------
// NCI Measurement
// ---------------------------------------------------------------------------

/// Non-controlling interest measurement for a subsidiary.
///
/// Captures the NCI share of net assets and current-period profit/loss,
/// computed from the subsidiary's `nci_percentage` in [`SubsidiaryRelationship`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NciMeasurement {
    /// Entity code of the subsidiary carrying an NCI.
    pub entity_code: String,
    /// NCI percentage (= 100 − parent ownership percentage).
    #[serde(with = "rust_decimal::serde::str")]
    pub nci_percentage: Decimal,
    /// NCI share of the subsidiary's net assets at period-end.
    #[serde(with = "rust_decimal::serde::str")]
    pub nci_share_net_assets: Decimal,
    /// NCI share of the subsidiary's net income/(loss) for the period.
    #[serde(with = "rust_decimal::serde::str")]
    pub nci_share_profit: Decimal,
    /// Total NCI recognised in the consolidated balance sheet
    /// (opening NCI + share of profit − NCI dividends).
    #[serde(with = "rust_decimal::serde::str")]
    pub total_nci: Decimal,
}

impl NciMeasurement {
    /// Compute NCI measurement from subsidiary inputs.
    ///
    /// # Arguments
    /// * `entity_code` — entity code of the subsidiary.
    /// * `nci_percentage` — NCI percentage (0–100).
    /// * `net_assets` — subsidiary net assets at period-end (before NCI split).
    /// * `net_income` — subsidiary net income/(loss) for the period.
    pub fn compute(
        entity_code: String,
        nci_percentage: Decimal,
        net_assets: Decimal,
        net_income: Decimal,
    ) -> Self {
        let hundred = Decimal::from(100);
        let nci_pct_fraction = nci_percentage / hundred;
        let nci_share_net_assets = net_assets * nci_pct_fraction;
        let nci_share_profit = net_income * nci_pct_fraction;
        // Simplified total NCI = share of net assets (already includes accumulated earnings).
        let total_nci = nci_share_net_assets;

        Self {
            entity_code,
            nci_percentage,
            nci_share_net_assets,
            nci_share_profit,
            total_nci,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_group_consolidation_method_from_ownership() {
        assert_eq!(
            GroupConsolidationMethod::from_ownership(dec!(100)),
            GroupConsolidationMethod::FullConsolidation
        );
        assert_eq!(
            GroupConsolidationMethod::from_ownership(dec!(51)),
            GroupConsolidationMethod::FullConsolidation
        );
        assert_eq!(
            GroupConsolidationMethod::from_ownership(dec!(50)),
            GroupConsolidationMethod::EquityMethod
        );
        assert_eq!(
            GroupConsolidationMethod::from_ownership(dec!(20)),
            GroupConsolidationMethod::EquityMethod
        );
        assert_eq!(
            GroupConsolidationMethod::from_ownership(dec!(19)),
            GroupConsolidationMethod::FairValue
        );
        assert_eq!(
            GroupConsolidationMethod::from_ownership(dec!(0)),
            GroupConsolidationMethod::FairValue
        );
    }
}
