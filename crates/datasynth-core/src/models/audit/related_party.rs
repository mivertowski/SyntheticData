//! Related party models per ISA 550.
//!
//! ISA 550 requires auditors to identify related party relationships and transactions
//! and assess their impact on the financial statements. Related party transactions
//! are inherently higher risk because they may not be conducted on arm's length terms
//! and can be used as a vehicle for management override or fraud.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Classification of the related party's relationship to the entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RelatedPartyType {
    /// Subsidiary — the entity controls the related party
    #[default]
    Subsidiary,
    /// Associate — the entity has significant influence over the related party
    Associate,
    /// Joint venture — the entity and one or more other parties control the related party jointly
    JointVenture,
    /// Key management personnel — those with authority and responsibility for planning,
    /// directing and controlling the activities of the entity
    KeyManagement,
    /// Close family members of key management personnel
    CloseFamily,
    /// Shareholder with significant influence (but not control) over the entity
    ShareholderSignificant,
    /// Entity controlled by a common director or key management person
    CommonDirector,
    /// Other related party relationship
    Other,
}

/// Legal or economic basis for the related party relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipBasis {
    /// Related through direct or indirect ownership
    #[default]
    Ownership,
    /// Related through the ability to control operating and financial policies
    Control,
    /// Related through significant influence without control
    SignificantInfluence,
    /// Related through key management personnel status
    KeyManagementPersonnel,
    /// Related through close family ties
    CloseFamily,
    /// Other basis for the relationship
    Other,
}

/// How the related party was identified during the audit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IdentificationSource {
    /// Disclosed by management in response to auditor inquiry
    #[default]
    ManagementDisclosure,
    /// Identified directly by the auditor through enquiry procedures
    AuditorInquiry,
    /// Identified through review of public registers or records
    PublicRecords,
    /// Identified through bank confirmation procedures
    BankConfirmation,
    /// Identified through review of legal agreements or correspondence
    LegalReview,
    /// Identified through a whistleblower tip or anonymous allegation
    WhistleblowerTip,
}

/// Type of related party transaction per ISA 550.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RptTransactionType {
    /// Sale of goods or services to a related party
    #[default]
    Sale,
    /// Purchase of goods or services from a related party
    Purchase,
    /// Lease of property or equipment to/from a related party
    Lease,
    /// Loan extended to or received from a related party
    Loan,
    /// Guarantee provided to or by a related party
    Guarantee,
    /// Management fee charged to or received from a related party
    ManagementFee,
    /// Dividend paid or received
    Dividend,
    /// Transfer of assets or liabilities
    Transfer,
    /// Ongoing service agreement with a related party
    ServiceAgreement,
    /// License or royalty arrangement
    LicenseRoyalty,
    /// Capital contribution made to or received from a related party
    CapitalContribution,
    /// Other type of related party transaction
    Other,
}

/// A related party identified during the audit per ISA 550.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedParty {
    /// Unique party ID
    pub party_id: Uuid,
    /// Human-readable reference (format: "RP-{first 8 hex chars of party_id}")
    pub party_ref: String,
    /// Engagement this related party is associated with
    pub engagement_id: Uuid,

    // === Identity ===
    /// Name of the related party
    pub party_name: String,
    /// Classification of the related party type
    pub party_type: RelatedPartyType,
    /// Legal or economic basis for the relationship
    pub relationship_basis: RelationshipBasis,

    // === Relationship Details ===
    /// Ownership percentage, if applicable
    pub ownership_percentage: Option<f64>,
    /// Whether the related party has board representation
    pub board_representation: bool,
    /// Whether the related party is or controls key management personnel
    pub key_management: bool,

    // === Disclosure Assessment ===
    /// Whether the related party has been disclosed in the financial statements
    pub disclosed_in_financials: bool,
    /// Whether the disclosure is adequate per the applicable financial reporting framework
    pub disclosure_adequate: Option<bool>,

    // === Identification ===
    /// How this related party was identified
    pub identified_by: IdentificationSource,

    // === Timestamps ===
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RelatedParty {
    /// Create a new related party record.
    pub fn new(
        engagement_id: Uuid,
        party_name: impl Into<String>,
        party_type: RelatedPartyType,
        relationship_basis: RelationshipBasis,
    ) -> Self {
        let now = Utc::now();
        let id = Uuid::new_v4();
        let party_ref = format!("RP-{}", &id.simple().to_string()[..8]);
        Self {
            party_id: id,
            party_ref,
            engagement_id,
            party_name: party_name.into(),
            party_type,
            relationship_basis,
            ownership_percentage: None,
            board_representation: false,
            key_management: false,
            disclosed_in_financials: true,
            disclosure_adequate: None,
            identified_by: IdentificationSource::ManagementDisclosure,
            created_at: now,
            updated_at: now,
        }
    }
}

/// A transaction with a related party identified during the audit per ISA 550.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedPartyTransaction {
    /// Unique transaction ID
    pub transaction_id: Uuid,
    /// Human-readable reference (format: "RPT-{first 8 hex chars of transaction_id}")
    pub transaction_ref: String,
    /// Engagement this transaction belongs to
    pub engagement_id: Uuid,
    /// The related party involved in this transaction
    pub related_party_id: Uuid,

    // === Transaction Details ===
    /// Type of related party transaction
    pub transaction_type: RptTransactionType,
    /// Description of the transaction
    pub description: String,
    /// Transaction amount
    pub amount: Decimal,
    /// Currency of the transaction (ISO 4217, e.g., "USD")
    pub currency: String,
    /// Date the transaction was entered into
    pub transaction_date: NaiveDate,
    /// Description of the terms and conditions
    pub terms_description: String,

    // === Arm's Length Assessment ===
    /// Whether the transaction was conducted on arm's length terms
    pub arms_length: Option<bool>,
    /// Evidence supporting the arm's length determination
    pub arms_length_evidence: Option<String>,
    /// Business rationale for the transaction
    pub business_rationale: Option<String>,
    /// Who approved the transaction (e.g., audit committee, board)
    pub approved_by: Option<String>,

    // === Disclosure Assessment ===
    /// Whether this transaction has been disclosed in the financial statements
    pub disclosed_in_financials: bool,
    /// Whether the disclosure is adequate per the applicable framework
    pub disclosure_adequate: Option<bool>,

    // === Risk Assessment ===
    /// Whether this transaction poses a management override risk
    pub management_override_risk: bool,

    // === Timestamps ===
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RelatedPartyTransaction {
    /// Create a new related party transaction record.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        engagement_id: Uuid,
        related_party_id: Uuid,
        transaction_type: RptTransactionType,
        description: impl Into<String>,
        amount: Decimal,
        currency: impl Into<String>,
        transaction_date: NaiveDate,
    ) -> Self {
        let now = Utc::now();
        let id = Uuid::new_v4();
        let transaction_ref = format!("RPT-{}", &id.simple().to_string()[..8]);
        Self {
            transaction_id: id,
            transaction_ref,
            engagement_id,
            related_party_id,
            transaction_type,
            description: description.into(),
            amount,
            currency: currency.into(),
            transaction_date,
            terms_description: String::new(),
            arms_length: None,
            arms_length_evidence: None,
            business_rationale: None,
            approved_by: None,
            disclosed_in_financials: true,
            disclosure_adequate: None,
            management_override_risk: false,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn sample_date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn test_new_related_party() {
        let eng = Uuid::new_v4();
        let rp = RelatedParty::new(
            eng,
            "Acme Holdings Ltd",
            RelatedPartyType::Subsidiary,
            RelationshipBasis::Ownership,
        );

        assert_eq!(rp.engagement_id, eng);
        assert_eq!(rp.party_name, "Acme Holdings Ltd");
        assert_eq!(rp.party_type, RelatedPartyType::Subsidiary);
        assert_eq!(rp.relationship_basis, RelationshipBasis::Ownership);
        assert!(rp.disclosed_in_financials);
        assert!(!rp.board_representation);
        assert!(!rp.key_management);
        assert!(rp.ownership_percentage.is_none());
        assert!(rp.disclosure_adequate.is_none());
        assert_eq!(rp.identified_by, IdentificationSource::ManagementDisclosure);
        assert!(rp.party_ref.starts_with("RP-"));
        assert_eq!(rp.party_ref.len(), 11); // "RP-" + 8 hex chars
    }

    #[test]
    fn test_new_rpt() {
        let eng = Uuid::new_v4();
        let party = Uuid::new_v4();
        let rpt = RelatedPartyTransaction::new(
            eng,
            party,
            RptTransactionType::ManagementFee,
            "Annual management fee for shared services",
            dec!(250_000),
            "USD",
            sample_date(2025, 6, 30),
        );

        assert_eq!(rpt.engagement_id, eng);
        assert_eq!(rpt.related_party_id, party);
        assert_eq!(rpt.transaction_type, RptTransactionType::ManagementFee);
        assert_eq!(rpt.amount, dec!(250_000));
        assert_eq!(rpt.currency, "USD");
        assert!(rpt.disclosed_in_financials);
        assert!(!rpt.management_override_risk);
        assert!(rpt.arms_length.is_none());
        assert!(rpt.terms_description.is_empty());
        assert!(rpt.transaction_ref.starts_with("RPT-"));
        assert_eq!(rpt.transaction_ref.len(), 12); // "RPT-" + 8 hex chars
    }

    #[test]
    fn test_related_party_type_serde() {
        let variants = [
            RelatedPartyType::Subsidiary,
            RelatedPartyType::Associate,
            RelatedPartyType::JointVenture,
            RelatedPartyType::KeyManagement,
            RelatedPartyType::CloseFamily,
            RelatedPartyType::ShareholderSignificant,
            RelatedPartyType::CommonDirector,
            RelatedPartyType::Other,
        ];
        for v in variants {
            let json = serde_json::to_string(&v).unwrap();
            let rt: RelatedPartyType = serde_json::from_str(&json).unwrap();
            assert_eq!(v, rt);
        }
        assert_eq!(
            serde_json::to_string(&RelatedPartyType::JointVenture).unwrap(),
            "\"joint_venture\""
        );
        assert_eq!(
            serde_json::to_string(&RelatedPartyType::ShareholderSignificant).unwrap(),
            "\"shareholder_significant\""
        );
        assert_eq!(
            serde_json::to_string(&RelatedPartyType::CommonDirector).unwrap(),
            "\"common_director\""
        );
    }

    #[test]
    fn test_relationship_basis_serde() {
        let variants = [
            RelationshipBasis::Ownership,
            RelationshipBasis::Control,
            RelationshipBasis::SignificantInfluence,
            RelationshipBasis::KeyManagementPersonnel,
            RelationshipBasis::CloseFamily,
            RelationshipBasis::Other,
        ];
        for v in variants {
            let json = serde_json::to_string(&v).unwrap();
            let rt: RelationshipBasis = serde_json::from_str(&json).unwrap();
            assert_eq!(v, rt);
        }
        assert_eq!(
            serde_json::to_string(&RelationshipBasis::SignificantInfluence).unwrap(),
            "\"significant_influence\""
        );
        assert_eq!(
            serde_json::to_string(&RelationshipBasis::KeyManagementPersonnel).unwrap(),
            "\"key_management_personnel\""
        );
    }

    #[test]
    fn test_identification_source_serde() {
        let variants = [
            IdentificationSource::ManagementDisclosure,
            IdentificationSource::AuditorInquiry,
            IdentificationSource::PublicRecords,
            IdentificationSource::BankConfirmation,
            IdentificationSource::LegalReview,
            IdentificationSource::WhistleblowerTip,
        ];
        for v in variants {
            let json = serde_json::to_string(&v).unwrap();
            let rt: IdentificationSource = serde_json::from_str(&json).unwrap();
            assert_eq!(v, rt);
        }
        assert_eq!(
            serde_json::to_string(&IdentificationSource::ManagementDisclosure).unwrap(),
            "\"management_disclosure\""
        );
        assert_eq!(
            serde_json::to_string(&IdentificationSource::WhistleblowerTip).unwrap(),
            "\"whistleblower_tip\""
        );
    }

    #[test]
    fn test_rpt_transaction_type_serde() {
        // Test round-trip for all 12 variants
        let variants = [
            RptTransactionType::Sale,
            RptTransactionType::Purchase,
            RptTransactionType::Lease,
            RptTransactionType::Loan,
            RptTransactionType::Guarantee,
            RptTransactionType::ManagementFee,
            RptTransactionType::Dividend,
            RptTransactionType::Transfer,
            RptTransactionType::ServiceAgreement,
            RptTransactionType::LicenseRoyalty,
            RptTransactionType::CapitalContribution,
            RptTransactionType::Other,
        ];
        for v in variants {
            let json = serde_json::to_string(&v).unwrap();
            let rt: RptTransactionType = serde_json::from_str(&json).unwrap();
            assert_eq!(v, rt);
        }
        assert_eq!(
            serde_json::to_string(&RptTransactionType::Sale).unwrap(),
            "\"sale\""
        );
        assert_eq!(
            serde_json::to_string(&RptTransactionType::ManagementFee).unwrap(),
            "\"management_fee\""
        );
        assert_eq!(
            serde_json::to_string(&RptTransactionType::ServiceAgreement).unwrap(),
            "\"service_agreement\""
        );
        assert_eq!(
            serde_json::to_string(&RptTransactionType::LicenseRoyalty).unwrap(),
            "\"license_royalty\""
        );
        assert_eq!(
            serde_json::to_string(&RptTransactionType::CapitalContribution).unwrap(),
            "\"capital_contribution\""
        );
    }

    #[test]
    fn test_rpt_all_12_transaction_types() {
        let eng = Uuid::new_v4();
        let party = Uuid::new_v4();
        let date = sample_date(2025, 1, 15);

        let all_types = [
            RptTransactionType::Sale,
            RptTransactionType::Purchase,
            RptTransactionType::Lease,
            RptTransactionType::Loan,
            RptTransactionType::Guarantee,
            RptTransactionType::ManagementFee,
            RptTransactionType::Dividend,
            RptTransactionType::Transfer,
            RptTransactionType::ServiceAgreement,
            RptTransactionType::LicenseRoyalty,
            RptTransactionType::CapitalContribution,
            RptTransactionType::Other,
        ];

        assert_eq!(
            all_types.len(),
            12,
            "must have exactly 12 transaction types"
        );

        for txn_type in all_types {
            let rpt = RelatedPartyTransaction::new(
                eng,
                party,
                txn_type,
                "Test transaction",
                dec!(1_000),
                "USD",
                date,
            );
            // Verify each variant serializes and deserialises correctly
            let json = serde_json::to_string(&rpt.transaction_type).unwrap();
            let rt: RptTransactionType = serde_json::from_str(&json).unwrap();
            assert_eq!(rpt.transaction_type, rt);
        }
    }

    #[test]
    fn test_management_override_risk_default() {
        let eng = Uuid::new_v4();
        let party = Uuid::new_v4();
        let rpt = RelatedPartyTransaction::new(
            eng,
            party,
            RptTransactionType::Loan,
            "Intercompany loan",
            dec!(1_000_000),
            "GBP",
            sample_date(2025, 3, 31),
        );
        // Default must be false per the spec
        assert!(!rpt.management_override_risk);
    }
}
