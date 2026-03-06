//! Banking customer model for KYC/AML simulation.

use chrono::NaiveDate;
use datasynth_core::models::banking::{
    BankingCustomerType, BusinessPersona, RetailPersona, RiskTier, TrustPersona,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{BeneficialOwner, KycProfile};

/// Customer name structure supporting various formats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerName {
    /// Full legal name
    pub legal_name: String,
    /// First name (for individuals)
    pub first_name: Option<String>,
    /// Last name (for individuals)
    pub last_name: Option<String>,
    /// Middle name (for individuals)
    pub middle_name: Option<String>,
    /// Trade name / DBA (for businesses)
    pub trade_name: Option<String>,
}

impl CustomerName {
    /// Create a new individual name.
    pub fn individual(first: &str, last: &str) -> Self {
        Self {
            legal_name: format!("{first} {last}"),
            first_name: Some(first.to_string()),
            last_name: Some(last.to_string()),
            middle_name: None,
            trade_name: None,
        }
    }

    /// Create a new individual name with middle name.
    pub fn individual_full(first: &str, middle: &str, last: &str) -> Self {
        Self {
            legal_name: format!("{first} {middle} {last}"),
            first_name: Some(first.to_string()),
            last_name: Some(last.to_string()),
            middle_name: Some(middle.to_string()),
            trade_name: None,
        }
    }

    /// Create a new business name.
    pub fn business(legal_name: &str) -> Self {
        Self {
            legal_name: legal_name.to_string(),
            first_name: None,
            last_name: None,
            middle_name: None,
            trade_name: None,
        }
    }

    /// Create a business name with trade name.
    pub fn business_with_dba(legal_name: &str, trade_name: &str) -> Self {
        Self {
            legal_name: legal_name.to_string(),
            first_name: None,
            last_name: None,
            middle_name: None,
            trade_name: Some(trade_name.to_string()),
        }
    }

    /// Get the display name.
    pub fn display_name(&self) -> &str {
        self.trade_name.as_deref().unwrap_or(&self.legal_name)
    }
}

/// Customer relationship for linked accounts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerRelationship {
    /// Related customer ID
    pub related_customer_id: Uuid,
    /// Relationship type
    pub relationship_type: RelationshipType,
    /// Start date of relationship
    pub start_date: NaiveDate,
    /// Whether relationship is still active
    pub is_active: bool,
}

/// Type of relationship between customers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    /// Spouse / domestic partner
    Spouse,
    /// Parent-child
    ParentChild,
    /// Sibling
    Sibling,
    /// Other family member
    Family,
    /// Business partner
    BusinessPartner,
    /// Employer-employee
    Employment,
    /// Authorized signer
    AuthorizedSigner,
    /// Beneficiary
    Beneficiary,
    /// Guarantor
    Guarantor,
    /// Attorney / power of attorney
    Attorney,
    /// Trust relationship
    TrustRelationship,
    /// Joint account holder
    JointAccountHolder,
}

impl RelationshipType {
    /// Risk weight for AML scoring.
    pub fn risk_weight(&self) -> f64 {
        match self {
            Self::Spouse | Self::ParentChild | Self::Sibling => 1.0,
            Self::Family => 1.1,
            Self::BusinessPartner => 1.3,
            Self::Employment => 0.8,
            Self::AuthorizedSigner | Self::JointAccountHolder => 1.2,
            Self::Beneficiary => 1.4,
            Self::Guarantor => 1.1,
            Self::Attorney => 1.5,
            Self::TrustRelationship => 1.6,
        }
    }
}

/// Persona variant for behavioral modeling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PersonaVariant {
    /// Retail customer persona
    Retail(RetailPersona),
    /// Business customer persona
    Business(BusinessPersona),
    /// Trust customer persona
    Trust(TrustPersona),
}

/// A banking customer with full KYC information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankingCustomer {
    /// Unique customer identifier
    pub customer_id: Uuid,
    /// Customer type (retail, business, trust)
    pub customer_type: BankingCustomerType,
    /// Customer name
    pub name: CustomerName,
    /// Behavioral persona
    pub persona: Option<PersonaVariant>,
    /// Country of residence (ISO 3166-1 alpha-2)
    pub residence_country: String,
    /// Country of citizenship (for individuals)
    pub citizenship_country: Option<String>,
    /// Date of birth (for individuals) or incorporation (for entities)
    pub date_of_birth: Option<NaiveDate>,
    /// Tax identification number
    pub tax_id: Option<String>,
    /// National ID number
    pub national_id: Option<String>,
    /// Passport number
    pub passport_number: Option<String>,
    /// Customer onboarding date
    pub onboarding_date: NaiveDate,
    /// KYC profile with expected activity
    pub kyc_profile: KycProfile,
    /// Risk tier assigned
    pub risk_tier: RiskTier,
    /// Account IDs owned by this customer
    pub account_ids: Vec<Uuid>,
    /// Relationships with other customers
    pub relationships: Vec<CustomerRelationship>,
    /// Beneficial owners (for entities/trusts)
    pub beneficial_owners: Vec<BeneficialOwner>,
    /// Primary contact email
    pub email: Option<String>,
    /// Primary contact phone
    pub phone: Option<String>,
    /// Address line 1
    pub address_line1: Option<String>,
    /// Address line 2
    pub address_line2: Option<String>,
    /// City
    pub city: Option<String>,
    /// State/province
    pub state: Option<String>,
    /// Postal code
    pub postal_code: Option<String>,
    /// Whether customer is active
    pub is_active: bool,
    /// Whether customer is a PEP (Politically Exposed Person)
    pub is_pep: bool,
    /// PEP category if applicable
    pub pep_category: Option<PepCategory>,
    /// Industry/occupation (NAICS code for businesses)
    pub industry_code: Option<String>,
    /// Industry description
    pub industry_description: Option<String>,
    /// Household ID for linked retail customers
    pub household_id: Option<Uuid>,
    /// Date of last KYC review
    pub last_kyc_review: Option<NaiveDate>,
    /// Next scheduled KYC review
    pub next_kyc_review: Option<NaiveDate>,
    /// Cross-reference to core enterprise customer ID (from master data)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enterprise_customer_id: Option<String>,

    // Ground truth labels (for ML)
    /// Whether this is a mule account (ground truth)
    pub is_mule: bool,
    /// Whether KYC information is truthful
    pub kyc_truthful: bool,
    /// True source of funds if different from declared
    pub true_source_of_funds: Option<datasynth_core::models::banking::SourceOfFunds>,
}

impl BankingCustomer {
    /// Create a new retail customer.
    pub fn new_retail(
        customer_id: Uuid,
        first_name: &str,
        last_name: &str,
        residence_country: &str,
        onboarding_date: NaiveDate,
    ) -> Self {
        Self {
            customer_id,
            customer_type: BankingCustomerType::Retail,
            name: CustomerName::individual(first_name, last_name),
            persona: None,
            residence_country: residence_country.to_string(),
            citizenship_country: Some(residence_country.to_string()),
            date_of_birth: None,
            tax_id: None,
            national_id: None,
            passport_number: None,
            onboarding_date,
            kyc_profile: KycProfile::default(),
            risk_tier: RiskTier::default(),
            account_ids: Vec::new(),
            relationships: Vec::new(),
            beneficial_owners: Vec::new(),
            email: None,
            phone: None,
            address_line1: None,
            address_line2: None,
            city: None,
            state: None,
            postal_code: None,
            is_active: true,
            is_pep: false,
            pep_category: None,
            industry_code: None,
            industry_description: None,
            household_id: None,
            last_kyc_review: Some(onboarding_date),
            next_kyc_review: None,
            enterprise_customer_id: None,
            is_mule: false,
            kyc_truthful: true,
            true_source_of_funds: None,
        }
    }

    /// Create a new business customer.
    pub fn new_business(
        customer_id: Uuid,
        legal_name: &str,
        residence_country: &str,
        onboarding_date: NaiveDate,
    ) -> Self {
        Self {
            customer_id,
            customer_type: BankingCustomerType::Business,
            name: CustomerName::business(legal_name),
            persona: None,
            residence_country: residence_country.to_string(),
            citizenship_country: None,
            date_of_birth: None,
            tax_id: None,
            national_id: None,
            passport_number: None,
            onboarding_date,
            kyc_profile: KycProfile::default(),
            risk_tier: RiskTier::default(),
            account_ids: Vec::new(),
            relationships: Vec::new(),
            beneficial_owners: Vec::new(),
            email: None,
            phone: None,
            address_line1: None,
            address_line2: None,
            city: None,
            state: None,
            postal_code: None,
            is_active: true,
            is_pep: false,
            pep_category: None,
            industry_code: None,
            industry_description: None,
            household_id: None,
            last_kyc_review: Some(onboarding_date),
            next_kyc_review: None,
            enterprise_customer_id: None,
            is_mule: false,
            kyc_truthful: true,
            true_source_of_funds: None,
        }
    }

    /// Set the persona.
    pub fn with_persona(mut self, persona: PersonaVariant) -> Self {
        self.persona = Some(persona);
        self
    }

    /// Set the risk tier.
    pub fn with_risk_tier(mut self, tier: RiskTier) -> Self {
        self.risk_tier = tier;
        self
    }

    /// Add an account.
    pub fn add_account(&mut self, account_id: Uuid) {
        self.account_ids.push(account_id);
    }

    /// Add a relationship.
    pub fn add_relationship(&mut self, relationship: CustomerRelationship) {
        self.relationships.push(relationship);
    }

    /// Add a beneficial owner.
    pub fn add_beneficial_owner(&mut self, owner: BeneficialOwner) {
        self.beneficial_owners.push(owner);
    }

    /// Calculate composite risk score.
    pub fn calculate_risk_score(&self) -> u8 {
        let mut score = self.risk_tier.score() as f64;

        // Adjust for customer type
        if self.customer_type.requires_enhanced_dd() {
            score *= 1.2;
        }

        // Adjust for PEP status
        if self.is_pep {
            score *= 1.5;
        }

        // Adjust for KYC truthfulness
        if !self.kyc_truthful {
            score *= 1.3;
        }

        // Adjust for mule status
        if self.is_mule {
            score *= 2.0;
        }

        score.min(100.0) as u8
    }
}

/// PEP (Politically Exposed Person) category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PepCategory {
    /// Head of state / government
    HeadOfState,
    /// Senior government official
    SeniorGovernment,
    /// Senior judicial official
    SeniorJudicial,
    /// Senior military official
    SeniorMilitary,
    /// Senior political party official
    SeniorPolitical,
    /// Senior executive of state-owned enterprise
    StateEnterprise,
    /// International organization official
    InternationalOrganization,
    /// Family member of PEP
    FamilyMember,
    /// Close associate of PEP
    CloseAssociate,
}

impl PepCategory {
    /// Risk weight for AML scoring.
    pub fn risk_weight(&self) -> f64 {
        match self {
            Self::HeadOfState | Self::SeniorGovernment => 3.0,
            Self::SeniorJudicial | Self::SeniorMilitary => 2.5,
            Self::SeniorPolitical | Self::StateEnterprise => 2.0,
            Self::InternationalOrganization => 1.8,
            Self::FamilyMember => 2.0,
            Self::CloseAssociate => 1.8,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_customer_name() {
        let name = CustomerName::individual("John", "Doe");
        assert_eq!(name.legal_name, "John Doe");
        assert_eq!(name.first_name, Some("John".to_string()));

        let biz = CustomerName::business_with_dba("Acme Corp LLC", "Acme Store");
        assert_eq!(biz.display_name(), "Acme Store");
    }

    #[test]
    fn test_banking_customer() {
        let customer = BankingCustomer::new_retail(
            Uuid::new_v4(),
            "Jane",
            "Smith",
            "US",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        assert_eq!(customer.customer_type, BankingCustomerType::Retail);
        assert!(customer.is_active);
        assert!(!customer.is_mule);
        assert!(customer.kyc_truthful);
    }

    #[test]
    fn test_risk_score_calculation() {
        let mut customer = BankingCustomer::new_retail(
            Uuid::new_v4(),
            "Test",
            "User",
            "US",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        let base_score = customer.calculate_risk_score();

        customer.is_pep = true;
        let pep_score = customer.calculate_risk_score();
        assert!(pep_score > base_score);

        customer.is_mule = true;
        let mule_score = customer.calculate_risk_score();
        assert!(mule_score > pep_score);
    }
}
