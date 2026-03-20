//! Audit Opinion Generator — ISA 700 / 701 / 705 / 706.
//!
//! Derives an `AuditOpinion` from the audit findings, going concern
//! assessment, and component auditor reports already produced in the
//! same generation run.
//!
//! # Opinion determination logic
//!
//! | Condition                                               | Opinion       |
//! |---------------------------------------------------------|---------------|
//! | No material findings (or all remediated)               | Unmodified    |
//! | 1–2 material findings not remediated                   | Qualified     |
//! | 3+ material findings not remediated                    | Adverse       |
//! | Scope limitations from component reports               | Disclaimer    |
//!
//! Going concern with material uncertainty → Emphasis of Matter paragraph
//! added regardless of opinion type.
//!
//! Key Audit Matters (ISA 701): 1–3 KAMs per entity, drawn from revenue
//! recognition, goodwill / asset impairment, and expected credit loss.

use chrono::NaiveDate;
use datasynth_core::models::audit::going_concern::{
    GoingConcernAssessment, GoingConcernConclusion,
};
use datasynth_core::models::audit::{
    AuditFinding, ComponentAuditorReport, FindingStatus, FindingType,
};
use datasynth_core::utils::seeded_rng;
use datasynth_standards::audit::opinion::{
    AuditOpinion, EmphasisOfMatter, EomMatter, IcfrOpinion, IcfrOpinionType, KeyAuditMatter,
    MaterialWeakness as OpinionMaterialWeakness, ModificationBasis, OpinionModification,
    OpinionType, PcaobOpinionElements, RiskLevel,
};
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Input data consumed by the generator.
#[derive(Debug, Clone, Default)]
pub struct AuditOpinionInput {
    /// Entity / company code.
    pub entity_code: String,
    /// Human-readable entity name.
    pub entity_name: String,
    /// Engagement UUID from which this opinion is derived.
    pub engagement_id: Uuid,
    /// Period end date of the financial statements.
    pub period_end: NaiveDate,
    /// All findings raised for this entity's engagement.
    pub findings: Vec<AuditFinding>,
    /// Going concern assessment for this entity (if available).
    pub going_concern: Option<GoingConcernAssessment>,
    /// Component auditor reports (ISA 600) received for this engagement.
    pub component_reports: Vec<ComponentAuditorReport>,
    /// Whether the engagement follows PCAOB / SOX (US issuer).
    pub is_us_listed: bool,
    /// Auditor firm name.
    pub auditor_name: String,
    /// Engagement partner name.
    pub engagement_partner: String,
}

/// Output produced by the generator.
#[derive(Debug, Clone)]
pub struct GeneratedAuditOpinion {
    /// The formed audit opinion per ISA 700 / 705 / 706.
    pub opinion: AuditOpinion,
    /// Key Audit Matters per ISA 701 (also embedded in `opinion`).
    pub key_audit_matters: Vec<KeyAuditMatter>,
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Generator for audit opinions (ISA 700/701/705/706).
pub struct AuditOpinionGenerator {
    rng: ChaCha8Rng,
}

impl AuditOpinionGenerator {
    /// Create a new generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0x700),
        }
    }

    /// Generate an audit opinion from the supplied input.
    pub fn generate(&mut self, input: &AuditOpinionInput) -> GeneratedAuditOpinion {
        // Opinion date = period end + 60–90 days (typical sign-off window).
        let opinion_days: i64 = self.rng.random_range(60i64..=90);
        let opinion_date = input.period_end + chrono::Duration::days(opinion_days);

        // Classify material findings that have NOT been remediated.
        let open_material_findings: Vec<&AuditFinding> = input
            .findings
            .iter()
            .filter(|f| is_material_open(f))
            .collect();

        // Check for scope limitations in component reports.
        let has_scope_limitation = input.component_reports.iter().any(is_scope_limited);

        // Determine opinion type.
        let opinion_type = if has_scope_limitation {
            OpinionType::Disclaimer
        } else {
            match open_material_findings.len() {
                0 => OpinionType::Unmodified,
                1 | 2 => OpinionType::Qualified,
                _ => OpinionType::Adverse,
            }
        };

        let mut opinion = AuditOpinion::new(
            input.engagement_id,
            opinion_date,
            opinion_type,
            &input.entity_name,
            input.period_end,
        );

        opinion.auditor_name = input.auditor_name.clone();
        opinion.engagement_partner = input.engagement_partner.clone();
        // EQCR required for listed entities and whenever opinion is modified.
        opinion.eqcr_performed = input.is_us_listed || opinion.is_modified();

        // Populate modification details for non-unmodified opinions.
        if opinion.is_modified() {
            opinion.modification = Some(self.build_modification(
                opinion_type,
                &open_material_findings,
                has_scope_limitation,
            ));
        }

        // Going concern — populate conclusion and add EOM paragraph.
        if let Some(gc) = &input.going_concern {
            let has_uncertainty = !matches!(
                gc.auditor_conclusion,
                GoingConcernConclusion::NoMaterialUncertainty
            );
            opinion.material_uncertainty_going_concern = has_uncertainty;
            opinion.going_concern_conclusion.material_uncertainty_exists = has_uncertainty;
            opinion.going_concern_conclusion.events_conditions = gc
                .indicators
                .iter()
                .map(|i| i.description.clone())
                .collect();
            opinion.going_concern_conclusion.management_plans = gc.management_plans.join("; ");

            if has_uncertainty {
                opinion.add_eom(EmphasisOfMatter::new(
                    EomMatter::GoingConcern,
                    format!(
                        "We draw attention to Note X in the financial statements which indicates \
                         that {} has identified conditions that raise material uncertainty about \
                         the entity's ability to continue as a going concern.  \
                         Our opinion is not modified in respect of this matter.",
                        input.entity_name
                    ),
                ));
            }
        }

        // Generate 1–3 Key Audit Matters (ISA 701).
        let kams = self.generate_key_audit_matters(input);
        for kam in &kams {
            opinion.add_kam(kam.clone());
        }

        // PCAOB / SOX integrated audit for US-listed entities.
        if input.is_us_listed {
            let mws: Vec<OpinionMaterialWeakness> = open_material_findings
                .iter()
                .filter(|f| matches!(f.finding_type, FindingType::MaterialWeakness))
                .map(|f| {
                    let mut mw = OpinionMaterialWeakness::new(&f.title);
                    mw.affected_controls = f.related_control_ids.clone();
                    mw.affected_accounts = f.accounts_affected.clone();
                    mw.potential_misstatement = f
                        .monetary_impact
                        .map(|d| d.to_string())
                        .unwrap_or_else(|| "Not quantified".into());
                    mw
                })
                .collect();

            let icfr_type = if mws.is_empty() {
                IcfrOpinionType::Effective
            } else {
                IcfrOpinionType::Adverse
            };

            let mut pcaob = PcaobOpinionElements::new(true);
            pcaob.icfr_opinion = Some(IcfrOpinion {
                opinion_type: icfr_type,
                material_weaknesses: mws,
                significant_deficiencies: open_material_findings
                    .iter()
                    .filter(|f| matches!(f.finding_type, FindingType::SignificantDeficiency))
                    .map(|f| f.title.clone())
                    .collect(),
                scope_limitations: if has_scope_limitation {
                    vec!["Scope limitation from component auditor(s)".into()]
                } else {
                    Vec::new()
                },
            });
            // Clone KAMs for critical audit matters section.
            pcaob.critical_audit_matters = kams.clone();
            opinion.pcaob_compliance = Some(pcaob);
        }

        GeneratedAuditOpinion {
            key_audit_matters: kams,
            opinion,
        }
    }

    /// Generate opinions for a batch of entities.
    pub fn generate_batch(&mut self, inputs: &[AuditOpinionInput]) -> Vec<GeneratedAuditOpinion> {
        inputs.iter().map(|i| self.generate(i)).collect()
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn build_modification(
        &mut self,
        opinion_type: OpinionType,
        findings: &[&AuditFinding],
        scope_limited: bool,
    ) -> OpinionModification {
        let basis = if scope_limited && !findings.is_empty() {
            ModificationBasis::Both
        } else if scope_limited {
            ModificationBasis::InabilityToObtainEvidence
        } else {
            ModificationBasis::MaterialMisstatement
        };

        // Describe the matters giving rise to modification.
        let matter_description = if findings.is_empty() {
            "Scope limitation arising from component auditor reports prevented the group \
             auditor from obtaining sufficient appropriate audit evidence."
                .to_string()
        } else {
            let titles: Vec<&str> = findings.iter().map(|f| f.title.as_str()).collect();
            format!(
                "The following matters gave rise to a modification of our opinion: {}.",
                titles.join("; ")
            )
        };

        let mut modification = OpinionModification::new(basis, matter_description);
        modification.is_pervasive = matches!(opinion_type, OpinionType::Adverse);

        // Aggregate affected accounts.
        modification.affected_areas = findings
            .iter()
            .flat_map(|f| f.accounts_affected.iter().cloned())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // Aggregate monetary impact.
        let total_impact: rust_decimal::Decimal =
            findings.iter().filter_map(|f| f.monetary_impact).sum();
        if total_impact > rust_decimal::Decimal::ZERO {
            modification.misstatement_amount = Some(total_impact);
        }

        modification
    }

    fn generate_key_audit_matters(&mut self, input: &AuditOpinionInput) -> Vec<KeyAuditMatter> {
        // ISA 701 applies to listed entities; for non-listed, generate fewer KAMs.
        let max_kams: usize = if input.is_us_listed {
            3
        } else {
            self.rng.random_range(1usize..=2)
        };

        let candidates: Vec<KamTemplate> = vec![
            KamTemplate {
                title: "Revenue Recognition".into(),
                area: "Revenue".into(),
                significance: "Revenue recognition involves significant judgment in identifying \
                    performance obligations and determining the appropriate timing and method of \
                    revenue recognition under IFRS 15 / ASC 606, particularly for complex \
                    multi-element arrangements and variable consideration."
                    .into(),
                response: "We obtained and evaluated management's revenue recognition policies \
                    and assessed their compliance with applicable standards. We performed \
                    substantive testing on a sample of significant revenue contracts and tested \
                    controls over the order-to-cash process. We challenged management's \
                    assumptions on variable consideration using analytical procedures."
                    .into(),
                romm: RiskLevel::High,
            },
            KamTemplate {
                title: "Goodwill and Intangible Asset Impairment".into(),
                area: "Intangible Assets".into(),
                significance: "The assessment of the recoverability of goodwill and other \
                    intangible assets requires significant management judgment, particularly \
                    regarding the estimation of future cash flows, discount rates, and growth \
                    rates used in value-in-use calculations under IAS 36 / ASC 350."
                    .into(),
                response: "We assessed the appropriateness of management's impairment model \
                    and the methodology applied. We used internal specialists to evaluate \
                    the reasonableness of the discount rate and growth rate assumptions. \
                    We performed sensitivity analyses to assess the impact of reasonably \
                    possible changes in key assumptions."
                    .into(),
                romm: RiskLevel::High,
            },
            KamTemplate {
                title: "Expected Credit Loss Provisioning".into(),
                area: "Loans and Receivables".into(),
                significance: "The determination of expected credit losses (ECL) under \
                    IFRS 9 / ASC 326 involves significant estimation uncertainty, including \
                    the identification of significant increases in credit risk, the selection \
                    of appropriate models, and the incorporation of forward-looking \
                    macroeconomic information."
                    .into(),
                response: "We evaluated the design and operating effectiveness of controls \
                    over the ECL process. We engaged our credit modelling specialists to \
                    assess the appropriateness of key assumptions and model parameters. \
                    We tested the completeness and accuracy of data inputs and challenged \
                    management's macroeconomic scenarios against independent forecasts."
                    .into(),
                romm: RiskLevel::VeryHigh,
            },
        ];

        let count = candidates.len().min(max_kams);
        // Shuffle and take the first `count` candidates.
        let mut indices: Vec<usize> = (0..candidates.len()).collect();
        for i in (1..indices.len()).rev() {
            let j = self.rng.random_range(0..=i);
            indices.swap(i, j);
        }

        indices
            .into_iter()
            .take(count)
            .map(|idx| {
                let t = &candidates[idx];
                let mut kam = KeyAuditMatter::new(&t.title, &t.significance, &t.response, &t.area);
                kam.romm_level = t.romm;

                // Link relevant findings to the KAM.
                let related: Vec<Uuid> = input
                    .findings
                    .iter()
                    .filter(|f| {
                        f.accounts_affected
                            .iter()
                            .any(|a| a.to_lowercase().contains(&t.area.to_lowercase()))
                    })
                    .map(|f| f.finding_id)
                    .collect();
                kam.related_finding_ids = related;
                kam.workpaper_references = vec![format!(
                    "WP-{}-{}",
                    t.area.chars().next().unwrap_or('X'),
                    self.rng.random_range(100u32..999)
                )];
                kam
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Returns `true` when a finding is material and not yet closed/remediated.
fn is_material_open(f: &AuditFinding) -> bool {
    let is_material = matches!(
        f.finding_type,
        FindingType::MaterialWeakness | FindingType::MaterialMisstatement
    );
    let is_open = !matches!(
        f.status,
        FindingStatus::Closed | FindingStatus::NotApplicable | FindingStatus::PendingValidation
    );
    is_material && is_open
}

/// Returns `true` when a component auditor report indicates a scope limitation.
fn is_scope_limited(report: &ComponentAuditorReport) -> bool {
    // A component report has a scope limitation when the auditor could not
    // complete their work or explicitly flagged a limitation.
    !report.scope_limitations.is_empty()
        || report.significant_findings.iter().any(|s| {
            let lower = s.to_lowercase();
            lower.contains("scope limitation")
                || lower.contains("unable to obtain")
                || lower.contains("insufficient evidence")
        })
}

// ---------------------------------------------------------------------------
// Internal template struct
// ---------------------------------------------------------------------------

struct KamTemplate {
    title: String,
    area: String,
    significance: String,
    response: String,
    romm: RiskLevel,
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::audit::FindingType;

    fn make_period_end() -> NaiveDate {
        NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()
    }

    fn make_engagement_id() -> Uuid {
        Uuid::new_v4()
    }

    fn minimal_input(entity_code: &str) -> AuditOpinionInput {
        AuditOpinionInput {
            entity_code: entity_code.to_string(),
            entity_name: format!("{entity_code} Ltd"),
            engagement_id: make_engagement_id(),
            period_end: make_period_end(),
            findings: Vec::new(),
            going_concern: None,
            component_reports: Vec::new(),
            is_us_listed: false,
            auditor_name: "Big Four & Co LLP".into(),
            engagement_partner: "Jane Auditor".into(),
        }
    }

    #[test]
    fn test_unmodified_when_no_material_findings() {
        let mut gen = AuditOpinionGenerator::new(42);
        let input = minimal_input("C001");
        let result = gen.generate(&input);
        assert_eq!(result.opinion.opinion_type, OpinionType::Unmodified);
        assert!(result.opinion.modification.is_none());
    }

    #[test]
    fn test_qualified_with_one_material_finding() {
        use datasynth_core::models::audit::{AuditFinding, FindingType};

        let mut gen = AuditOpinionGenerator::new(42);
        let eng_id = make_engagement_id();
        let mut finding = AuditFinding::new(eng_id, FindingType::MaterialWeakness, "Seg of duties");
        finding.status = datasynth_core::models::audit::FindingStatus::Draft;

        let mut input = minimal_input("C002");
        input.engagement_id = eng_id;
        input.findings = vec![finding];

        let result = gen.generate(&input);
        assert_eq!(result.opinion.opinion_type, OpinionType::Qualified);
        assert!(result.opinion.modification.is_some());
    }

    #[test]
    fn test_adverse_with_three_or_more_material_findings() {
        use datasynth_core::models::audit::{AuditFinding, FindingType};

        let mut gen = AuditOpinionGenerator::new(42);
        let eng_id = make_engagement_id();
        let findings: Vec<AuditFinding> = (0..3)
            .map(|i| AuditFinding::new(eng_id, FindingType::MaterialWeakness, &format!("MW {i}")))
            .collect();

        let mut input = minimal_input("C003");
        input.engagement_id = eng_id;
        input.findings = findings;

        let result = gen.generate(&input);
        assert_eq!(result.opinion.opinion_type, OpinionType::Adverse);
        assert!(result.opinion.modification.as_ref().unwrap().is_pervasive);
    }

    #[test]
    fn test_disclaimer_when_scope_limited() {
        use datasynth_core::models::audit::ComponentAuditorReport;

        let mut gen = AuditOpinionGenerator::new(42);
        let eng_id = make_engagement_id();

        // Build a component report with a scope limitation.
        let report = ComponentAuditorReport {
            id: "RPT-001".into(),
            instruction_id: "INST-001".into(),
            component_auditor_id: "CA-001".into(),
            entity_code: "C004".into(),
            misstatements_identified: Vec::new(),
            scope_limitations: vec!["Unable to obtain sufficient appropriate audit evidence from third-party confirmation".into()],
            significant_findings: Vec::new(),
            conclusion: "Scope limitation prevents complete reporting.".into(),
        };

        let mut input = minimal_input("C004");
        input.engagement_id = eng_id;
        input.component_reports = vec![report];

        let result = gen.generate(&input);
        assert_eq!(result.opinion.opinion_type, OpinionType::Disclaimer);
    }

    #[test]
    fn test_going_concern_emphasis_of_matter() {
        use datasynth_core::models::audit::going_concern::{
            GoingConcernAssessment, GoingConcernConclusion,
        };

        let mut gen = AuditOpinionGenerator::new(42);
        let gc = GoingConcernAssessment {
            entity_code: "C005".into(),
            assessment_date: make_period_end(),
            assessment_period: "FY2024".into(),
            indicators: Vec::new(),
            management_plans: Vec::new(),
            auditor_conclusion: GoingConcernConclusion::MaterialUncertaintyExists,
            material_uncertainty_exists: true,
        };

        let mut input = minimal_input("C005");
        input.going_concern = Some(gc);

        let result = gen.generate(&input);
        assert!(result.opinion.material_uncertainty_going_concern);
        assert!(!result.opinion.emphasis_of_matter.is_empty());
        assert!(result
            .opinion
            .emphasis_of_matter
            .iter()
            .any(|e| matches!(e.matter, EomMatter::GoingConcern)));
    }

    #[test]
    fn test_key_audit_matters_generated() {
        let mut gen = AuditOpinionGenerator::new(42);
        let input = minimal_input("C006");
        let result = gen.generate(&input);
        // Non-listed entity should have 1–2 KAMs.
        assert!(!result.key_audit_matters.is_empty());
        assert!(result.key_audit_matters.len() <= 3);
        // KAMs are also embedded in the opinion struct.
        assert_eq!(
            result.opinion.key_audit_matters.len(),
            result.key_audit_matters.len()
        );
    }

    #[test]
    fn test_us_listed_pcaob_elements_present() {
        let mut gen = AuditOpinionGenerator::new(42);
        let mut input = minimal_input("C007");
        input.is_us_listed = true;

        let result = gen.generate(&input);
        assert!(result.opinion.pcaob_compliance.is_some());
        let pcaob = result.opinion.pcaob_compliance.as_ref().unwrap();
        assert!(pcaob.is_integrated_audit);
        assert!(pcaob.icfr_opinion.is_some());
    }

    #[test]
    fn test_remediated_finding_does_not_trigger_modification() {
        use datasynth_core::models::audit::{AuditFinding, FindingStatus, FindingType};

        let mut gen = AuditOpinionGenerator::new(42);
        let eng_id = make_engagement_id();
        let mut finding = AuditFinding::new(eng_id, FindingType::MaterialWeakness, "Old MW");
        finding.status = FindingStatus::Closed; // already remediated

        let mut input = minimal_input("C008");
        input.engagement_id = eng_id;
        input.findings = vec![finding];

        let result = gen.generate(&input);
        // Remediated finding should NOT trigger a qualified / adverse opinion.
        assert_eq!(result.opinion.opinion_type, OpinionType::Unmodified);
    }

    #[test]
    fn test_batch_generate() {
        let mut gen = AuditOpinionGenerator::new(99);
        let inputs: Vec<AuditOpinionInput> = (0..5)
            .map(|i| minimal_input(&format!("C{:03}", i)))
            .collect();
        let results = gen.generate_batch(&inputs);
        assert_eq!(results.len(), 5);
    }
}
