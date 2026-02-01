//! Audit Opinion Models (ISA 700/705/706/701).
//!
//! Implements audit opinion formation and reporting:
//! - ISA 700: Forming an Opinion
//! - ISA 701: Key Audit Matters
//! - ISA 705: Modifications to the Opinion
//! - ISA 706: Emphasis of Matter and Other Matter Paragraphs

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Audit opinion record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditOpinion {
    /// Unique opinion identifier.
    pub opinion_id: Uuid,

    /// Engagement ID.
    pub engagement_id: Uuid,

    /// Opinion date.
    pub opinion_date: NaiveDate,

    /// Type of opinion.
    pub opinion_type: OpinionType,

    /// Key Audit Matters (ISA 701).
    pub key_audit_matters: Vec<KeyAuditMatter>,

    /// Modification details (if modified opinion).
    pub modification: Option<OpinionModification>,

    /// Emphasis of Matter paragraphs (ISA 706).
    pub emphasis_of_matter: Vec<EmphasisOfMatter>,

    /// Other Matter paragraphs (ISA 706).
    pub other_matter: Vec<OtherMatter>,

    /// Going concern conclusion.
    pub going_concern_conclusion: GoingConcernConclusion,

    /// Material uncertainty related to going concern.
    pub material_uncertainty_going_concern: bool,

    /// PCAOB compliance elements (for US issuers).
    pub pcaob_compliance: Option<PcaobOpinionElements>,

    /// Financial statement period end.
    pub period_end_date: NaiveDate,

    /// Entity name.
    pub entity_name: String,

    /// Auditor name/firm.
    pub auditor_name: String,

    /// Engagement partner.
    pub engagement_partner: String,

    /// Whether EQCR was performed.
    pub eqcr_performed: bool,
}

impl AuditOpinion {
    /// Create a new audit opinion.
    pub fn new(
        engagement_id: Uuid,
        opinion_date: NaiveDate,
        opinion_type: OpinionType,
        entity_name: impl Into<String>,
        period_end_date: NaiveDate,
    ) -> Self {
        Self {
            opinion_id: Uuid::now_v7(),
            engagement_id,
            opinion_date,
            opinion_type,
            key_audit_matters: Vec::new(),
            modification: None,
            emphasis_of_matter: Vec::new(),
            other_matter: Vec::new(),
            going_concern_conclusion: GoingConcernConclusion::default(),
            material_uncertainty_going_concern: false,
            pcaob_compliance: None,
            period_end_date,
            entity_name: entity_name.into(),
            auditor_name: String::new(),
            engagement_partner: String::new(),
            eqcr_performed: false,
        }
    }

    /// Check if opinion is unmodified.
    pub fn is_unmodified(&self) -> bool {
        matches!(self.opinion_type, OpinionType::Unmodified)
    }

    /// Check if opinion is modified.
    pub fn is_modified(&self) -> bool {
        !self.is_unmodified()
    }

    /// Add a Key Audit Matter.
    pub fn add_kam(&mut self, kam: KeyAuditMatter) {
        self.key_audit_matters.push(kam);
    }

    /// Add Emphasis of Matter.
    pub fn add_eom(&mut self, eom: EmphasisOfMatter) {
        self.emphasis_of_matter.push(eom);
    }
}

/// Type of audit opinion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OpinionType {
    /// Clean opinion - financial statements are fairly presented.
    #[default]
    Unmodified,
    /// Material misstatement but not pervasive.
    Qualified,
    /// Material and pervasive misstatement.
    Adverse,
    /// Unable to obtain sufficient appropriate audit evidence.
    Disclaimer,
}

impl std::fmt::Display for OpinionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unmodified => write!(f, "Unmodified"),
            Self::Qualified => write!(f, "Qualified"),
            Self::Adverse => write!(f, "Adverse"),
            Self::Disclaimer => write!(f, "Disclaimer"),
        }
    }
}

/// Key Audit Matter (ISA 701).
///
/// Matters that were of most significance in the audit and required
/// significant auditor attention.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyAuditMatter {
    /// KAM identifier.
    pub kam_id: Uuid,

    /// Title/heading of the KAM.
    pub title: String,

    /// Description of why this matter was significant.
    pub significance_explanation: String,

    /// How the matter was addressed in the audit.
    pub audit_response: String,

    /// Related financial statement area.
    pub financial_statement_area: String,

    /// Risk of material misstatement level.
    pub romm_level: RiskLevel,

    /// Related findings (if any).
    pub related_finding_ids: Vec<Uuid>,

    /// Workpaper references.
    pub workpaper_references: Vec<String>,
}

impl KeyAuditMatter {
    /// Create a new Key Audit Matter.
    pub fn new(
        title: impl Into<String>,
        significance_explanation: impl Into<String>,
        audit_response: impl Into<String>,
        financial_statement_area: impl Into<String>,
    ) -> Self {
        Self {
            kam_id: Uuid::now_v7(),
            title: title.into(),
            significance_explanation: significance_explanation.into(),
            audit_response: audit_response.into(),
            financial_statement_area: financial_statement_area.into(),
            romm_level: RiskLevel::High,
            related_finding_ids: Vec::new(),
            workpaper_references: Vec::new(),
        }
    }
}

/// Risk level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    #[default]
    Medium,
    High,
    VeryHigh,
}

/// Opinion modification details (ISA 705).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpinionModification {
    /// Basis for modification.
    pub basis: ModificationBasis,

    /// Description of the matter.
    pub matter_description: String,

    /// Financial statement effects.
    pub financial_effects: FinancialEffects,

    /// Whether matter is pervasive.
    pub is_pervasive: bool,

    /// Affected financial statement areas.
    pub affected_areas: Vec<String>,

    /// Misstatement amount (if quantifiable).
    pub misstatement_amount: Option<rust_decimal::Decimal>,

    /// Related to prior period.
    pub relates_to_prior_period: bool,

    /// Related to going concern.
    pub relates_to_going_concern: bool,
}

impl OpinionModification {
    /// Create a new opinion modification.
    pub fn new(basis: ModificationBasis, matter_description: impl Into<String>) -> Self {
        Self {
            basis,
            matter_description: matter_description.into(),
            financial_effects: FinancialEffects::default(),
            is_pervasive: false,
            affected_areas: Vec::new(),
            misstatement_amount: None,
            relates_to_prior_period: false,
            relates_to_going_concern: false,
        }
    }
}

/// Basis for opinion modification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModificationBasis {
    /// Material misstatement in the financial statements.
    MaterialMisstatement,
    /// Inability to obtain sufficient appropriate audit evidence.
    InabilityToObtainEvidence,
    /// Both misstatement and scope limitation.
    Both,
}

impl std::fmt::Display for ModificationBasis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MaterialMisstatement => write!(f, "Material Misstatement"),
            Self::InabilityToObtainEvidence => write!(f, "Inability to Obtain Evidence"),
            Self::Both => write!(f, "Material Misstatement and Scope Limitation"),
        }
    }
}

/// Financial effects of modification matter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FinancialEffects {
    /// Effect on assets.
    pub assets_effect: String,
    /// Effect on liabilities.
    pub liabilities_effect: String,
    /// Effect on equity.
    pub equity_effect: String,
    /// Effect on revenue.
    pub revenue_effect: String,
    /// Effect on expenses.
    pub expenses_effect: String,
    /// Effect on cash flows.
    pub cash_flows_effect: String,
    /// Effect on disclosures.
    pub disclosures_effect: String,
}

/// Emphasis of Matter paragraph (ISA 706).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmphasisOfMatter {
    /// EOM identifier.
    pub eom_id: Uuid,

    /// Matter being emphasized.
    pub matter: EomMatter,

    /// Description of the matter.
    pub description: String,

    /// Reference to relevant notes.
    pub note_reference: String,

    /// Whether matter is appropriately presented and disclosed.
    pub appropriately_presented: bool,
}

impl EmphasisOfMatter {
    /// Create a new Emphasis of Matter.
    pub fn new(matter: EomMatter, description: impl Into<String>) -> Self {
        Self {
            eom_id: Uuid::now_v7(),
            matter,
            description: description.into(),
            note_reference: String::new(),
            appropriately_presented: true,
        }
    }
}

/// Common Emphasis of Matter topics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EomMatter {
    /// Significant uncertainty about entity's ability to continue.
    GoingConcern,
    /// Major catastrophe affecting operations.
    MajorCatastrophe,
    /// Significant related party transactions.
    RelatedPartyTransactions,
    /// Significant subsequent event.
    SubsequentEvent,
    /// Change in accounting policy.
    AccountingPolicyChange,
    /// New accounting standard adoption.
    NewStandardAdoption,
    /// Unusually important litigation.
    Litigation,
    /// Regulatory action.
    RegulatoryAction,
    /// Other significant matter.
    Other,
}

impl std::fmt::Display for EomMatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GoingConcern => write!(f, "Going Concern"),
            Self::MajorCatastrophe => write!(f, "Major Catastrophe"),
            Self::RelatedPartyTransactions => write!(f, "Related Party Transactions"),
            Self::SubsequentEvent => write!(f, "Subsequent Event"),
            Self::AccountingPolicyChange => write!(f, "Accounting Policy Change"),
            Self::NewStandardAdoption => write!(f, "New Standard Adoption"),
            Self::Litigation => write!(f, "Litigation"),
            Self::RegulatoryAction => write!(f, "Regulatory Action"),
            Self::Other => write!(f, "Other"),
        }
    }
}

/// Other Matter paragraph (ISA 706).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtherMatter {
    /// OM identifier.
    pub om_id: Uuid,

    /// Type of other matter.
    pub matter_type: OtherMatterType,

    /// Description.
    pub description: String,
}

impl OtherMatter {
    /// Create a new Other Matter.
    pub fn new(matter_type: OtherMatterType, description: impl Into<String>) -> Self {
        Self {
            om_id: Uuid::now_v7(),
            matter_type,
            description: description.into(),
        }
    }
}

/// Types of Other Matter paragraphs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OtherMatterType {
    /// Prior period financial statements audited by another auditor.
    PredecessorAuditor,
    /// Prior period not audited.
    PriorPeriodNotAudited,
    /// Supplementary information.
    SupplementaryInformation,
    /// Other matter relevant to users.
    Other,
}

/// Going concern conclusion (ISA 570).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GoingConcernConclusion {
    /// Conclusion on going concern.
    pub conclusion: GoingConcernAssessment,

    /// Material uncertainty exists.
    pub material_uncertainty_exists: bool,

    /// If uncertainty exists, is it adequately disclosed.
    pub adequately_disclosed: bool,

    /// Events/conditions identified.
    pub events_conditions: Vec<String>,

    /// Management's plans to address.
    pub management_plans: String,

    /// Auditor's evaluation of management's plans.
    pub auditor_evaluation: String,

    /// Impact on opinion.
    pub opinion_impact: Option<OpinionType>,
}

/// Going concern assessment outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum GoingConcernAssessment {
    /// No material uncertainty identified.
    #[default]
    NoMaterialUncertainty,
    /// Material uncertainty exists, adequately disclosed.
    MaterialUncertaintyAdequatelyDisclosed,
    /// Material uncertainty exists, not adequately disclosed.
    MaterialUncertaintyNotAdequatelyDisclosed,
    /// Going concern basis inappropriate.
    GoingConcernInappropriate,
}

impl std::fmt::Display for GoingConcernAssessment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoMaterialUncertainty => write!(f, "No Material Uncertainty"),
            Self::MaterialUncertaintyAdequatelyDisclosed => {
                write!(f, "Material Uncertainty - Adequately Disclosed")
            }
            Self::MaterialUncertaintyNotAdequatelyDisclosed => {
                write!(f, "Material Uncertainty - Not Adequately Disclosed")
            }
            Self::GoingConcernInappropriate => write!(f, "Going Concern Basis Inappropriate"),
        }
    }
}

/// PCAOB-specific opinion elements for US issuers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcaobOpinionElements {
    /// Whether this is an integrated audit (SOX 404).
    pub is_integrated_audit: bool,

    /// ICFR opinion (for integrated audits).
    pub icfr_opinion: Option<IcfrOpinion>,

    /// Critical Audit Matters (PCAOB equivalent of KAMs).
    pub critical_audit_matters: Vec<KeyAuditMatter>,

    /// Auditor tenure disclosure.
    pub auditor_tenure_years: Option<u32>,

    /// PCAOB registration number.
    pub pcaob_registration_number: Option<String>,
}

impl PcaobOpinionElements {
    /// Create new PCAOB elements.
    pub fn new(is_integrated_audit: bool) -> Self {
        Self {
            is_integrated_audit,
            icfr_opinion: None,
            critical_audit_matters: Vec::new(),
            auditor_tenure_years: None,
            pcaob_registration_number: None,
        }
    }
}

/// ICFR (Internal Control over Financial Reporting) opinion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IcfrOpinion {
    /// ICFR opinion type.
    pub opinion_type: IcfrOpinionType,

    /// Material weaknesses identified.
    pub material_weaknesses: Vec<MaterialWeakness>,

    /// Significant deficiencies identified.
    pub significant_deficiencies: Vec<String>,

    /// Scope limitations (if any).
    pub scope_limitations: Vec<String>,
}

/// ICFR opinion type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IcfrOpinionType {
    /// ICFR is effective.
    #[default]
    Effective,
    /// Adverse opinion due to material weakness.
    Adverse,
    /// Disclaimer due to scope limitation.
    Disclaimer,
}

/// Material weakness in ICFR.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialWeakness {
    /// Weakness identifier.
    pub weakness_id: Uuid,

    /// Description.
    pub description: String,

    /// Affected controls.
    pub affected_controls: Vec<String>,

    /// Affected accounts.
    pub affected_accounts: Vec<String>,

    /// Potential misstatement.
    pub potential_misstatement: String,
}

impl MaterialWeakness {
    /// Create a new material weakness.
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            weakness_id: Uuid::now_v7(),
            description: description.into(),
            affected_controls: Vec::new(),
            affected_accounts: Vec::new(),
            potential_misstatement: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_opinion_creation() {
        let opinion = AuditOpinion::new(
            Uuid::now_v7(),
            NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
            OpinionType::Unmodified,
            "Test Company Inc.",
            NaiveDate::from_ymd_opt(2023, 12, 31).unwrap(),
        );

        assert!(opinion.is_unmodified());
        assert!(!opinion.is_modified());
        assert!(opinion.key_audit_matters.is_empty());
    }

    #[test]
    fn test_modified_opinion() {
        let mut opinion = AuditOpinion::new(
            Uuid::now_v7(),
            NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
            OpinionType::Qualified,
            "Test Company Inc.",
            NaiveDate::from_ymd_opt(2023, 12, 31).unwrap(),
        );

        opinion.modification = Some(OpinionModification::new(
            ModificationBasis::MaterialMisstatement,
            "Inventory was not properly valued",
        ));

        assert!(opinion.is_modified());
        assert!(opinion.modification.is_some());
    }

    #[test]
    fn test_key_audit_matter() {
        let kam = KeyAuditMatter::new(
            "Revenue Recognition",
            "Complex multi-element arrangements require significant judgment",
            "We tested controls over revenue recognition and performed substantive testing",
            "Revenue",
        );

        assert_eq!(kam.title, "Revenue Recognition");
        assert_eq!(kam.romm_level, RiskLevel::High);
    }

    #[test]
    fn test_going_concern() {
        let gc = GoingConcernConclusion {
            conclusion: GoingConcernAssessment::MaterialUncertaintyAdequatelyDisclosed,
            material_uncertainty_exists: true,
            adequately_disclosed: true,
            ..Default::default()
        };

        assert!(gc.material_uncertainty_exists);
        assert!(gc.adequately_disclosed);
    }

    #[test]
    fn test_pcaob_elements() {
        let mut pcaob = PcaobOpinionElements::new(true);
        pcaob.auditor_tenure_years = Some(5);
        pcaob.icfr_opinion = Some(IcfrOpinion {
            opinion_type: IcfrOpinionType::Effective,
            material_weaknesses: Vec::new(),
            significant_deficiencies: Vec::new(),
            scope_limitations: Vec::new(),
        });

        assert!(pcaob.is_integrated_audit);
        assert!(pcaob.icfr_opinion.is_some());
    }
}
