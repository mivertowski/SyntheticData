//! Beneficial ownership structures for KYC/AML.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A beneficial owner (UBO) of an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeneficialOwner {
    /// Unique beneficial owner identifier
    pub ubo_id: Uuid,
    /// Name of beneficial owner
    pub name: String,
    /// Date of birth (for individuals)
    pub date_of_birth: Option<NaiveDate>,
    /// Country of residence
    pub country_of_residence: String,
    /// Country of citizenship
    pub citizenship_country: Option<String>,
    /// Ownership percentage
    #[serde(with = "rust_decimal::serde::str")]
    pub ownership_percentage: Decimal,
    /// Control type
    pub control_type: ControlType,
    /// Is this a direct or indirect owner
    pub is_direct: bool,
    /// Intermediary entity (if indirect)
    pub intermediary_entity: Option<IntermediaryEntity>,
    /// Is a PEP
    pub is_pep: bool,
    /// Is on sanctions list
    pub is_sanctioned: bool,
    /// Verification status
    pub verification_status: VerificationStatus,
    /// Verification date
    pub verification_date: Option<NaiveDate>,
    /// Source of wealth
    pub source_of_wealth: Option<String>,

    // Ground truth
    /// Whether this is a true UBO (vs nominee)
    pub is_true_ubo: bool,
    /// Hidden behind shell structure
    pub is_hidden: bool,
}

impl BeneficialOwner {
    /// Create a new beneficial owner.
    pub fn new(ubo_id: Uuid, name: &str, country: &str, ownership_percentage: Decimal) -> Self {
        Self {
            ubo_id,
            name: name.to_string(),
            date_of_birth: None,
            country_of_residence: country.to_string(),
            citizenship_country: Some(country.to_string()),
            ownership_percentage,
            control_type: ControlType::OwnershipInterest,
            is_direct: true,
            intermediary_entity: None,
            is_pep: false,
            is_sanctioned: false,
            verification_status: VerificationStatus::Verified,
            verification_date: None,
            source_of_wealth: None,
            is_true_ubo: true,
            is_hidden: false,
        }
    }

    /// Set as indirect ownership.
    pub fn with_intermediary(mut self, intermediary: IntermediaryEntity) -> Self {
        self.is_direct = false;
        self.intermediary_entity = Some(intermediary);
        self
    }

    /// Set as PEP.
    pub fn as_pep(mut self) -> Self {
        self.is_pep = true;
        self
    }

    /// Mark as nominee (not true UBO).
    pub fn as_nominee(mut self) -> Self {
        self.is_true_ubo = false;
        self
    }

    /// Mark as hidden behind shell structure.
    pub fn as_hidden(mut self) -> Self {
        self.is_hidden = true;
        self.is_true_ubo = false;
        self
    }

    /// Calculate risk score.
    pub fn calculate_risk_score(&self) -> u8 {
        let mut score = 0.0;

        // Ownership level risk
        let ownership_f64: f64 = self.ownership_percentage.try_into().unwrap_or(0.0);
        if ownership_f64 >= 25.0 {
            score += 20.0;
        } else if ownership_f64 >= 10.0 {
            score += 10.0;
        }

        // Control type risk
        score += self.control_type.risk_weight() * 10.0;

        // Indirect ownership risk
        if !self.is_direct {
            score += 15.0;
        }

        // PEP risk
        if self.is_pep {
            score += 25.0;
        }

        // Sanctions risk
        if self.is_sanctioned {
            score += 50.0;
        }

        // Verification status
        if self.verification_status != VerificationStatus::Verified {
            score += 10.0;
        }

        // Hidden structure risk
        if self.is_hidden {
            score += 30.0;
        }

        score.min(100.0) as u8
    }
}

/// Type of control over entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ControlType {
    /// Direct ownership interest
    #[default]
    OwnershipInterest,
    /// Voting rights
    VotingRights,
    /// Board control
    BoardControl,
    /// Management control
    ManagementControl,
    /// Contract-based control
    ContractualControl,
    /// Family relationship control
    FamilyRelationship,
    /// Trust arrangement
    TrustArrangement,
    /// Nominee arrangement
    NomineeArrangement,
}

impl ControlType {
    /// Risk weight for AML scoring.
    pub fn risk_weight(&self) -> f64 {
        match self {
            Self::OwnershipInterest => 1.0,
            Self::VotingRights => 1.1,
            Self::BoardControl | Self::ManagementControl => 1.2,
            Self::ContractualControl => 1.4,
            Self::FamilyRelationship => 1.3,
            Self::TrustArrangement => 1.6,
            Self::NomineeArrangement => 2.0,
        }
    }
}

/// Verification status for UBO.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    /// Fully verified
    #[default]
    Verified,
    /// Partially verified
    PartiallyVerified,
    /// Pending verification
    Pending,
    /// Unable to verify
    UnableToVerify,
    /// Verification expired
    Expired,
}

/// Intermediary entity in ownership chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntermediaryEntity {
    /// Entity identifier
    pub entity_id: Uuid,
    /// Entity name
    pub name: String,
    /// Entity type
    pub entity_type: IntermediaryType,
    /// Jurisdiction (country)
    pub jurisdiction: String,
    /// Ownership percentage through this entity
    #[serde(with = "rust_decimal::serde::str")]
    pub ownership_percentage: Decimal,
    /// Is this a shell company
    pub is_shell: bool,
    /// Registration number
    pub registration_number: Option<String>,
}

impl IntermediaryEntity {
    /// Create a new intermediary entity.
    pub fn new(
        entity_id: Uuid,
        name: &str,
        entity_type: IntermediaryType,
        jurisdiction: &str,
        ownership_percentage: Decimal,
    ) -> Self {
        Self {
            entity_id,
            name: name.to_string(),
            entity_type,
            jurisdiction: jurisdiction.to_string(),
            ownership_percentage,
            is_shell: false,
            registration_number: None,
        }
    }

    /// Mark as shell company.
    pub fn as_shell(mut self) -> Self {
        self.is_shell = true;
        self
    }
}

/// Type of intermediary entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntermediaryType {
    /// Holding company
    HoldingCompany,
    /// Special purpose vehicle
    SPV,
    /// Trust
    Trust,
    /// Foundation
    Foundation,
    /// Limited partnership
    LimitedPartnership,
    /// LLC
    LLC,
    /// Other corporate entity
    Other,
}

impl IntermediaryType {
    /// Risk weight for AML scoring.
    pub fn risk_weight(&self) -> f64 {
        match self {
            Self::HoldingCompany => 1.2,
            Self::SPV => 1.5,
            Self::Trust => 1.6,
            Self::Foundation => 1.5,
            Self::LimitedPartnership => 1.3,
            Self::LLC => 1.2,
            Self::Other => 1.4,
        }
    }
}

/// Ownership chain for complex structures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipChain {
    /// Ultimate beneficial owner
    pub ultimate_owner: BeneficialOwner,
    /// Chain of intermediaries (from UBO to entity)
    pub intermediaries: Vec<IntermediaryEntity>,
    /// Total layers in ownership structure
    pub total_layers: u8,
    /// Effective ownership percentage
    #[serde(with = "rust_decimal::serde::str")]
    pub effective_ownership: Decimal,
}

impl OwnershipChain {
    /// Create a new ownership chain.
    pub fn new(owner: BeneficialOwner) -> Self {
        let effective = owner.ownership_percentage;
        Self {
            ultimate_owner: owner,
            intermediaries: Vec::new(),
            total_layers: 1,
            effective_ownership: effective,
        }
    }

    /// Add an intermediary layer.
    pub fn add_intermediary(&mut self, intermediary: IntermediaryEntity) {
        // Adjust effective ownership
        let intermediary_pct: f64 = intermediary
            .ownership_percentage
            .try_into()
            .unwrap_or(100.0);
        let current_effective: f64 = self.effective_ownership.try_into().unwrap_or(0.0);
        self.effective_ownership =
            Decimal::from_f64_retain(current_effective * intermediary_pct / 100.0)
                .unwrap_or(Decimal::ZERO);

        self.intermediaries.push(intermediary);
        self.total_layers += 1;
    }

    /// Calculate complexity score.
    pub fn complexity_score(&self) -> u8 {
        let mut score = (self.total_layers as f64 - 1.0) * 10.0;

        for intermediary in &self.intermediaries {
            score += intermediary.entity_type.risk_weight() * 5.0;
            if intermediary.is_shell {
                score += 20.0;
            }
        }

        if !self.ultimate_owner.is_true_ubo {
            score += 30.0;
        }

        score.min(100.0) as u8
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_beneficial_owner() {
        let owner = BeneficialOwner::new(Uuid::new_v4(), "John Doe", "US", Decimal::from(50));
        assert!(owner.is_true_ubo);
        assert!(owner.is_direct);
    }

    #[test]
    fn test_ownership_chain() {
        let owner = BeneficialOwner::new(Uuid::new_v4(), "John Doe", "US", Decimal::from(100));
        let mut chain = OwnershipChain::new(owner);

        let holding = IntermediaryEntity::new(
            Uuid::new_v4(),
            "Holding Co Ltd",
            IntermediaryType::HoldingCompany,
            "KY",
            Decimal::from(80),
        );
        chain.add_intermediary(holding);

        assert_eq!(chain.total_layers, 2);
        assert!(chain.effective_ownership < Decimal::from(100));
    }

    #[test]
    fn test_risk_scoring() {
        let base_owner = BeneficialOwner::new(Uuid::new_v4(), "Jane Doe", "US", Decimal::from(30));
        let base_score = base_owner.calculate_risk_score();

        let pep_owner =
            BeneficialOwner::new(Uuid::new_v4(), "Minister Smith", "US", Decimal::from(30))
                .as_pep();
        let pep_score = pep_owner.calculate_risk_score();

        assert!(pep_score > base_score);
    }
}
