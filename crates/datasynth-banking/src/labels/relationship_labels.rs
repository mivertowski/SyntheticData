//! Relationship-level label generation.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{BankingCustomer, BeneficialOwner, CustomerRelationship};

/// Relationship type for labeling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationshipType {
    /// Family relationship
    Family,
    /// Employer-employee relationship
    Employment,
    /// Business partner
    BusinessPartner,
    /// Vendor relationship
    Vendor,
    /// Customer relationship (B2B)
    Customer,
    /// Beneficial ownership
    BeneficialOwnership,
    /// Transaction counterparty
    TransactionCounterparty,
    /// Money mule link
    MuleLink,
    /// Shell company link
    ShellLink,
    /// Unknown/other
    Unknown,
}

/// Relationship-level labels for ML training.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipLabel {
    /// Source entity ID
    pub source_id: Uuid,
    /// Target entity ID
    pub target_id: Uuid,
    /// Relationship type
    pub relationship_type: RelationshipType,
    /// Is this a mule network link?
    pub is_mule_link: bool,
    /// Is this a shell company link?
    pub is_shell_link: bool,
    /// Ownership percentage (for UBO edges)
    pub ownership_percent: Option<f64>,
    /// Number of transactions between entities
    pub transaction_count: u32,
    /// Total transaction volume
    pub transaction_volume: f64,
    /// Relationship strength score (0.0-1.0)
    pub strength: f64,
    /// Associated case ID
    pub case_id: Option<String>,
    /// Confidence score
    pub confidence: f64,
}

impl RelationshipLabel {
    /// Create a new relationship label.
    pub fn new(source_id: Uuid, target_id: Uuid, relationship_type: RelationshipType) -> Self {
        Self {
            source_id,
            target_id,
            relationship_type,
            is_mule_link: false,
            is_shell_link: false,
            ownership_percent: None,
            transaction_count: 0,
            transaction_volume: 0.0,
            strength: 0.5,
            case_id: None,
            confidence: 1.0,
        }
    }

    /// Mark as mule link.
    pub fn as_mule_link(mut self) -> Self {
        self.is_mule_link = true;
        self.relationship_type = RelationshipType::MuleLink;
        self
    }

    /// Mark as shell link.
    pub fn as_shell_link(mut self) -> Self {
        self.is_shell_link = true;
        self.relationship_type = RelationshipType::ShellLink;
        self
    }

    /// Set ownership percentage.
    pub fn with_ownership(mut self, percent: f64) -> Self {
        self.ownership_percent = Some(percent);
        self.relationship_type = RelationshipType::BeneficialOwnership;
        self
    }

    /// Set transaction statistics.
    pub fn with_transactions(mut self, count: u32, volume: f64) -> Self {
        self.transaction_count = count;
        self.transaction_volume = volume;
        // Compute strength based on transaction frequency
        self.strength = (count as f64 / 100.0).min(1.0);
        self
    }

    /// Set case ID.
    pub fn with_case(mut self, case_id: &str) -> Self {
        self.case_id = Some(case_id.to_string());
        self
    }
}

/// Relationship label extractor.
pub struct RelationshipLabelExtractor;

impl RelationshipLabelExtractor {
    /// Extract relationship labels from customers.
    pub fn extract_from_customers(customers: &[BankingCustomer]) -> Vec<RelationshipLabel> {
        let mut labels = Vec::new();

        for customer in customers {
            // Extract explicit relationships
            for rel in &customer.relationships {
                let label = Self::from_customer_relationship(customer.customer_id, rel);
                labels.push(label);
            }

            // Extract beneficial ownership relationships
            for bo in &customer.beneficial_owners {
                let label = Self::from_beneficial_owner(customer.customer_id, bo);
                labels.push(label);
            }
        }

        labels
    }

    /// Create label from customer relationship.
    fn from_customer_relationship(
        customer_id: Uuid,
        relationship: &CustomerRelationship,
    ) -> RelationshipLabel {
        use crate::models::RelationshipType as CustRelType;

        let rel_type = match relationship.relationship_type {
            CustRelType::Spouse
            | CustRelType::ParentChild
            | CustRelType::Sibling
            | CustRelType::Family => RelationshipType::Family,
            CustRelType::Employment => RelationshipType::Employment,
            CustRelType::BusinessPartner => RelationshipType::BusinessPartner,
            CustRelType::AuthorizedSigner | CustRelType::JointAccountHolder => {
                RelationshipType::Family
            }
            CustRelType::Beneficiary | CustRelType::TrustRelationship => {
                RelationshipType::BeneficialOwnership
            }
            CustRelType::Guarantor | CustRelType::Attorney => RelationshipType::Unknown,
        };

        RelationshipLabel::new(customer_id, relationship.related_customer_id, rel_type)
    }

    /// Create label from beneficial owner.
    fn from_beneficial_owner(entity_id: Uuid, bo: &BeneficialOwner) -> RelationshipLabel {
        let ownership_pct: f64 = bo.ownership_percentage.try_into().unwrap_or(0.0);
        let mut label =
            RelationshipLabel::new(bo.ubo_id, entity_id, RelationshipType::BeneficialOwnership)
                .with_ownership(ownership_pct);

        // Check for shell company indicators (hidden ownership or indirect with intermediary)
        if bo.is_hidden || bo.intermediary_entity.is_some() {
            label = label.as_shell_link();
        }

        label
    }

    /// Extract transaction-based relationships.
    pub fn extract_from_transactions(
        transactions: &[crate::models::BankTransaction],
    ) -> Vec<RelationshipLabel> {
        use std::collections::HashMap;

        // Group by account-counterparty pairs
        let mut pairs: HashMap<(Uuid, String), (u32, f64, bool)> = HashMap::new();

        for txn in transactions {
            let key = (txn.account_id, txn.counterparty.name.clone());
            let entry = pairs.entry(key).or_insert((0, 0.0, false));
            entry.0 += 1;
            entry.1 += txn.amount.try_into().unwrap_or(0.0);
            if txn.is_suspicious {
                entry.2 = true;
            }
        }

        pairs
            .into_iter()
            .filter(|(_, (count, _, _))| *count >= 2) // Only significant relationships
            .map(
                |((account_id, _counterparty), (count, volume, suspicious))| {
                    let mut label = RelationshipLabel::new(
                        account_id,
                        Uuid::new_v4(), // Counterparty UUID would come from counterparty pool
                        RelationshipType::TransactionCounterparty,
                    )
                    .with_transactions(count, volume);

                    if suspicious {
                        label.is_mule_link = true;
                    }

                    label
                },
            )
            .collect()
    }

    /// Get relationship label summary.
    pub fn summarize(labels: &[RelationshipLabel]) -> RelationshipLabelSummary {
        let total = labels.len();
        let mule_links = labels.iter().filter(|l| l.is_mule_link).count();
        let shell_links = labels.iter().filter(|l| l.is_shell_link).count();
        let ownership_links = labels
            .iter()
            .filter(|l| l.ownership_percent.is_some())
            .count();

        let mut by_type = std::collections::HashMap::new();
        for label in labels {
            *by_type.entry(label.relationship_type).or_insert(0) += 1;
        }

        RelationshipLabelSummary {
            total_relationships: total,
            mule_link_count: mule_links,
            mule_link_rate: mule_links as f64 / total.max(1) as f64,
            shell_link_count: shell_links,
            shell_link_rate: shell_links as f64 / total.max(1) as f64,
            ownership_link_count: ownership_links,
            by_type,
        }
    }
}

/// Relationship label summary.
#[derive(Debug, Clone)]
pub struct RelationshipLabelSummary {
    /// Total relationships
    pub total_relationships: usize,
    /// Number of mule links
    pub mule_link_count: usize,
    /// Mule link rate
    pub mule_link_rate: f64,
    /// Number of shell links
    pub shell_link_count: usize,
    /// Shell link rate
    pub shell_link_rate: f64,
    /// Number of ownership links
    pub ownership_link_count: usize,
    /// Counts by relationship type
    pub by_type: std::collections::HashMap<RelationshipType, usize>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_relationship_label() {
        let source = Uuid::new_v4();
        let target = Uuid::new_v4();

        let label = RelationshipLabel::new(source, target, RelationshipType::Family);

        assert_eq!(label.source_id, source);
        assert_eq!(label.target_id, target);
        assert!(!label.is_mule_link);
    }

    #[test]
    fn test_mule_link() {
        let source = Uuid::new_v4();
        let target = Uuid::new_v4();

        let label =
            RelationshipLabel::new(source, target, RelationshipType::Unknown).as_mule_link();

        assert!(label.is_mule_link);
        assert_eq!(label.relationship_type, RelationshipType::MuleLink);
    }

    #[test]
    fn test_ownership_label() {
        let owner = Uuid::new_v4();
        let entity = Uuid::new_v4();

        let label =
            RelationshipLabel::new(owner, entity, RelationshipType::Unknown).with_ownership(25.0);

        assert_eq!(label.ownership_percent, Some(25.0));
        assert_eq!(
            label.relationship_type,
            RelationshipType::BeneficialOwnership
        );
    }
}
