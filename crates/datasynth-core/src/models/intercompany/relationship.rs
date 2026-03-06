//! Intercompany relationship and ownership structure models.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents an intercompany relationship between two legal entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntercompanyRelationship {
    /// Unique identifier for this relationship.
    pub relationship_id: String,
    /// Parent/investing company code.
    pub parent_company: String,
    /// Subsidiary/investee company code.
    pub subsidiary_company: String,
    /// Ownership percentage (0.0 to 100.0).
    pub ownership_percentage: Decimal,
    /// Consolidation method based on ownership and control.
    pub consolidation_method: ConsolidationMethod,
    /// Transfer pricing policy identifier.
    pub transfer_pricing_policy: Option<String>,
    /// Date the relationship became effective.
    pub effective_date: NaiveDate,
    /// Date the relationship ended (if applicable).
    pub end_date: Option<NaiveDate>,
    /// Whether this is a direct or indirect holding.
    pub holding_type: HoldingType,
    /// Functional currency of the subsidiary.
    pub functional_currency: String,
    /// Whether elimination entries are required.
    pub requires_elimination: bool,
    /// Segment or business unit for reporting.
    pub reporting_segment: Option<String>,
}

impl IntercompanyRelationship {
    /// Create a new intercompany relationship.
    pub fn new(
        relationship_id: String,
        parent_company: String,
        subsidiary_company: String,
        ownership_percentage: Decimal,
        effective_date: NaiveDate,
    ) -> Self {
        let consolidation_method = ConsolidationMethod::from_ownership(ownership_percentage);
        let requires_elimination = consolidation_method != ConsolidationMethod::Equity;

        Self {
            relationship_id,
            parent_company,
            subsidiary_company,
            ownership_percentage,
            consolidation_method,
            transfer_pricing_policy: None,
            effective_date,
            end_date: None,
            holding_type: HoldingType::Direct,
            functional_currency: "USD".to_string(),
            requires_elimination,
            reporting_segment: None,
        }
    }

    /// Check if the relationship is active on a given date.
    pub fn is_active_on(&self, date: NaiveDate) -> bool {
        date >= self.effective_date && self.end_date.is_none_or(|end| date <= end)
    }

    /// Check if this represents a controlling interest.
    pub fn is_controlling(&self) -> bool {
        self.ownership_percentage > Decimal::from(50)
    }

    /// Check if this represents a significant influence.
    pub fn has_significant_influence(&self) -> bool {
        self.ownership_percentage >= Decimal::from(20)
    }
}

/// Consolidation method based on level of control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConsolidationMethod {
    /// Full consolidation (>50% ownership, control).
    #[default]
    Full,
    /// Proportional consolidation (joint ventures).
    Proportional,
    /// Equity method (20-50% ownership, significant influence).
    Equity,
    /// Cost method (<20% ownership, no significant influence).
    Cost,
}

impl ConsolidationMethod {
    /// Determine consolidation method based on ownership percentage.
    pub fn from_ownership(ownership_pct: Decimal) -> Self {
        if ownership_pct > Decimal::from(50) {
            Self::Full
        } else if ownership_pct >= Decimal::from(20) {
            Self::Equity
        } else {
            Self::Cost
        }
    }

    /// Check if full elimination is required.
    pub fn requires_full_elimination(&self) -> bool {
        matches!(self, Self::Full)
    }

    /// Check if proportional elimination is required.
    pub fn requires_proportional_elimination(&self) -> bool {
        matches!(self, Self::Proportional)
    }

    /// Returns the string representation of the consolidation method.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Full => "Full",
            Self::Proportional => "Proportional",
            Self::Equity => "Equity",
            Self::Cost => "Cost",
        }
    }
}

/// Type of holding in the ownership structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum HoldingType {
    /// Direct ownership by the parent.
    #[default]
    Direct,
    /// Indirect ownership through another subsidiary.
    Indirect,
    /// Reciprocal/cross-holding.
    Reciprocal,
}

/// Represents the complete ownership structure of a corporate group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipStructure {
    /// The ultimate parent company code.
    pub ultimate_parent: String,
    /// All intercompany relationships.
    pub relationships: Vec<IntercompanyRelationship>,
    /// Effective ownership percentages (company -> effective %).
    effective_ownership: HashMap<String, Decimal>,
    /// Direct subsidiaries by parent.
    subsidiaries_by_parent: HashMap<String, Vec<String>>,
}

impl OwnershipStructure {
    /// Create a new ownership structure.
    pub fn new(ultimate_parent: String) -> Self {
        Self {
            ultimate_parent,
            relationships: Vec::new(),
            effective_ownership: HashMap::new(),
            subsidiaries_by_parent: HashMap::new(),
        }
    }

    /// Add a relationship to the ownership structure.
    pub fn add_relationship(&mut self, relationship: IntercompanyRelationship) {
        // Update subsidiaries index
        self.subsidiaries_by_parent
            .entry(relationship.parent_company.clone())
            .or_default()
            .push(relationship.subsidiary_company.clone());

        self.relationships.push(relationship);

        // Recalculate effective ownership
        self.calculate_effective_ownership();
    }

    /// Get all relationships for a specific parent company.
    pub fn get_relationships_for_parent(&self, parent: &str) -> Vec<&IntercompanyRelationship> {
        self.relationships
            .iter()
            .filter(|r| r.parent_company == parent)
            .collect()
    }

    /// Get all relationships for a specific subsidiary.
    pub fn get_relationships_for_subsidiary(
        &self,
        subsidiary: &str,
    ) -> Vec<&IntercompanyRelationship> {
        self.relationships
            .iter()
            .filter(|r| r.subsidiary_company == subsidiary)
            .collect()
    }

    /// Get the direct parent of a company.
    pub fn get_direct_parent(&self, company: &str) -> Option<&str> {
        self.relationships
            .iter()
            .find(|r| r.subsidiary_company == company && r.holding_type == HoldingType::Direct)
            .map(|r| r.parent_company.as_str())
    }

    /// Get direct subsidiaries of a company.
    pub fn get_direct_subsidiaries(&self, parent: &str) -> Vec<&str> {
        self.subsidiaries_by_parent
            .get(parent)
            .map(|subs| subs.iter().map(std::string::String::as_str).collect())
            .unwrap_or_default()
    }

    /// Get all companies in the group.
    pub fn get_all_companies(&self) -> Vec<&str> {
        let mut companies: Vec<&str> = vec![self.ultimate_parent.as_str()];
        for rel in &self.relationships {
            if !companies.contains(&rel.subsidiary_company.as_str()) {
                companies.push(rel.subsidiary_company.as_str());
            }
        }
        companies
    }

    /// Get effective ownership percentage from ultimate parent.
    pub fn get_effective_ownership(&self, company: &str) -> Decimal {
        if company == self.ultimate_parent {
            Decimal::from(100)
        } else {
            self.effective_ownership
                .get(company)
                .copied()
                .unwrap_or(Decimal::ZERO)
        }
    }

    /// Check if two companies are related (share a common parent).
    pub fn are_related(&self, company1: &str, company2: &str) -> bool {
        if company1 == company2 {
            return true;
        }
        // Both are in the same group if they have effective ownership
        let has1 =
            company1 == self.ultimate_parent || self.effective_ownership.contains_key(company1);
        let has2 =
            company2 == self.ultimate_parent || self.effective_ownership.contains_key(company2);
        has1 && has2
    }

    /// Get the consolidation method for a company.
    pub fn get_consolidation_method(&self, company: &str) -> Option<ConsolidationMethod> {
        self.relationships
            .iter()
            .find(|r| r.subsidiary_company == company)
            .map(|r| r.consolidation_method)
    }

    /// Calculate effective ownership percentages through the chain.
    fn calculate_effective_ownership(&mut self) {
        self.effective_ownership.clear();

        // Start from ultimate parent's direct subsidiaries
        let mut to_process: Vec<(String, Decimal)> = self
            .get_direct_subsidiaries(&self.ultimate_parent)
            .iter()
            .filter_map(|sub| {
                self.relationships
                    .iter()
                    .find(|r| {
                        r.parent_company == self.ultimate_parent && r.subsidiary_company == *sub
                    })
                    .map(|r| (sub.to_string(), r.ownership_percentage))
            })
            .collect();

        // Process in order, calculating effective ownership
        while let Some((company, effective_pct)) = to_process.pop() {
            self.effective_ownership
                .insert(company.clone(), effective_pct);

            // Add this company's subsidiaries
            for sub in self.get_direct_subsidiaries(&company) {
                if let Some(rel) = self
                    .relationships
                    .iter()
                    .find(|r| r.parent_company == company && r.subsidiary_company == sub)
                {
                    let sub_effective =
                        effective_pct * rel.ownership_percentage / Decimal::from(100);
                    to_process.push((sub.to_string(), sub_effective));
                }
            }
        }
    }

    /// Get relationships that are active on a given date.
    pub fn get_active_relationships(&self, date: NaiveDate) -> Vec<&IntercompanyRelationship> {
        self.relationships
            .iter()
            .filter(|r| r.is_active_on(date))
            .collect()
    }

    /// Get companies that require full consolidation.
    pub fn get_fully_consolidated_companies(&self) -> Vec<&str> {
        self.relationships
            .iter()
            .filter(|r| r.consolidation_method == ConsolidationMethod::Full)
            .map(|r| r.subsidiary_company.as_str())
            .collect()
    }

    /// Get companies accounted for under equity method.
    pub fn get_equity_method_companies(&self) -> Vec<&str> {
        self.relationships
            .iter()
            .filter(|r| r.consolidation_method == ConsolidationMethod::Equity)
            .map(|r| r.subsidiary_company.as_str())
            .collect()
    }
}

/// Intercompany account mapping for a relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntercompanyAccountMapping {
    /// The relationship this mapping applies to.
    pub relationship_id: String,
    /// IC Receivable account (seller side).
    pub ic_receivable_account: String,
    /// IC Payable account (buyer side).
    pub ic_payable_account: String,
    /// IC Revenue account (seller side).
    pub ic_revenue_account: String,
    /// IC Expense account (buyer side).
    pub ic_expense_account: String,
    /// IC Investment account (parent, for equity method).
    pub ic_investment_account: Option<String>,
    /// IC Equity account (subsidiary, for eliminations).
    pub ic_equity_account: Option<String>,
}

impl IntercompanyAccountMapping {
    /// Create a new IC account mapping with standard accounts.
    pub fn new_standard(relationship_id: String, company_code: &str) -> Self {
        Self {
            relationship_id,
            ic_receivable_account: format!("1310{company_code}"),
            ic_payable_account: format!("2110{company_code}"),
            ic_revenue_account: format!("4100{company_code}"),
            ic_expense_account: format!("5100{company_code}"),
            ic_investment_account: Some(format!("1510{company_code}")),
            ic_equity_account: Some(format!("3100{company_code}")),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_consolidation_method_from_ownership() {
        assert_eq!(
            ConsolidationMethod::from_ownership(dec!(100)),
            ConsolidationMethod::Full
        );
        assert_eq!(
            ConsolidationMethod::from_ownership(dec!(51)),
            ConsolidationMethod::Full
        );
        assert_eq!(
            ConsolidationMethod::from_ownership(dec!(50)),
            ConsolidationMethod::Equity
        );
        assert_eq!(
            ConsolidationMethod::from_ownership(dec!(20)),
            ConsolidationMethod::Equity
        );
        assert_eq!(
            ConsolidationMethod::from_ownership(dec!(19)),
            ConsolidationMethod::Cost
        );
    }

    #[test]
    fn test_relationship_is_controlling() {
        let rel = IntercompanyRelationship::new(
            "REL001".to_string(),
            "1000".to_string(),
            "1100".to_string(),
            dec!(100),
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        );
        assert!(rel.is_controlling());

        let rel2 = IntercompanyRelationship::new(
            "REL002".to_string(),
            "1000".to_string(),
            "2000".to_string(),
            dec!(30),
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        );
        assert!(!rel2.is_controlling());
        assert!(rel2.has_significant_influence());
    }

    #[test]
    fn test_ownership_structure() {
        let mut structure = OwnershipStructure::new("1000".to_string());

        structure.add_relationship(IntercompanyRelationship::new(
            "REL001".to_string(),
            "1000".to_string(),
            "1100".to_string(),
            dec!(100),
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        ));

        structure.add_relationship(IntercompanyRelationship::new(
            "REL002".to_string(),
            "1100".to_string(),
            "1110".to_string(),
            dec!(80),
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        ));

        assert_eq!(structure.get_effective_ownership("1000"), dec!(100));
        assert_eq!(structure.get_effective_ownership("1100"), dec!(100));
        assert_eq!(structure.get_effective_ownership("1110"), dec!(80));

        assert!(structure.are_related("1000", "1100"));
        assert!(structure.are_related("1100", "1110"));

        let subs = structure.get_direct_subsidiaries("1000");
        assert_eq!(subs, vec!["1100"]);
    }

    #[test]
    fn test_relationship_active_date() {
        let mut rel = IntercompanyRelationship::new(
            "REL001".to_string(),
            "1000".to_string(),
            "1100".to_string(),
            dec!(100),
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        );
        rel.end_date = Some(NaiveDate::from_ymd_opt(2023, 12, 31).unwrap());

        assert!(rel.is_active_on(NaiveDate::from_ymd_opt(2022, 6, 15).unwrap()));
        assert!(rel.is_active_on(NaiveDate::from_ymd_opt(2023, 12, 31).unwrap()));
        assert!(!rel.is_active_on(NaiveDate::from_ymd_opt(2021, 12, 31).unwrap()));
        assert!(!rel.is_active_on(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()));
    }
}
