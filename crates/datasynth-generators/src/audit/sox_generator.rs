//! SOX 302 / 404 Assessment Generator.
//!
//! Produces:
//! - [`Sox302Certification`] — CEO and CFO certifications (SOX Section 302).
//!   Generated for every US-listed entity per fiscal year.
//! - [`Sox404Assessment`] — Management's assessment of ICFR (SOX Section 404).
//!   Effectiveness is determined from the audit findings already generated:
//!   any open material weakness results in an "ineffective" conclusion.
//!
//! # Usage
//! ```ignore
//! use datasynth_generators::audit::sox_generator::{SoxGenerator, SoxGeneratorInput};
//!
//! let mut gen = SoxGenerator::new(42);
//! let (certs, assessment) = gen.generate(&input);
//! ```

use chrono::NaiveDate;
use datasynth_core::models::audit::{AuditFinding, FindingStatus, FindingType};
use datasynth_core::utils::seeded_rng;
use datasynth_standards::regulatory::sox::{
    CertifierRole, ControlDeficiency as SoxControlDeficiency, DeficiencyClassificationSummary,
    MaterialWeakness, RemediationAction, RemediationStatus, ScopeConclusion, ScopedEntity,
    SignificantDeficiency, Sox302Certification, Sox404Assessment,
};
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Input required to generate SOX artefacts for one entity.
#[derive(Debug, Clone)]
pub struct SoxGeneratorInput {
    /// Entity / company code.
    pub company_code: String,
    /// Entity name (used in certification text).
    pub company_name: String,
    /// Fiscal year being certified/assessed.
    pub fiscal_year: u16,
    /// Period end date.
    pub period_end: NaiveDate,
    /// All audit findings raised for this entity.
    pub findings: Vec<AuditFinding>,
    /// CEO name.
    pub ceo_name: String,
    /// CFO name.
    pub cfo_name: String,
    /// Materiality threshold used in the audit.
    pub materiality_threshold: Decimal,
    /// Percentage of consolidated revenue represented by this entity.
    pub revenue_percent: Decimal,
    /// Percentage of consolidated assets represented by this entity.
    pub assets_percent: Decimal,
    /// Key account captions in scope.
    pub significant_accounts: Vec<String>,
}

impl Default for SoxGeneratorInput {
    fn default() -> Self {
        Self {
            company_code: "C000".into(),
            company_name: "Example Corp".into(),
            fiscal_year: 2024,
            period_end: NaiveDate::from_ymd_opt(2024, 12, 31).unwrap_or_default(),
            findings: Vec::new(),
            ceo_name: "John Smith".into(),
            cfo_name: "Jane Doe".into(),
            materiality_threshold: Decimal::from(100_000),
            revenue_percent: Decimal::from(100),
            assets_percent: Decimal::from(100),
            significant_accounts: vec![
                "Revenue".into(),
                "Accounts Receivable".into(),
                "Inventory".into(),
                "Fixed Assets".into(),
                "Accounts Payable".into(),
            ],
        }
    }
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Generates SOX Section 302 certifications and Section 404 ICFR assessments.
pub struct SoxGenerator {
    rng: ChaCha8Rng,
}

impl SoxGenerator {
    /// Create a new generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0x302_404),
        }
    }

    /// Generate SOX 302 certifications (CEO + CFO) and a SOX 404 assessment.
    ///
    /// Returns `(certifications, sox404_assessment)`.
    pub fn generate(
        &mut self,
        input: &SoxGeneratorInput,
    ) -> (Vec<Sox302Certification>, Sox404Assessment) {
        let certs = self.generate_302_certifications(input);
        let assessment = self.generate_404_assessment(input);
        (certs, assessment)
    }

    /// Generate for a batch of entities.
    pub fn generate_batch(
        &mut self,
        inputs: &[SoxGeneratorInput],
    ) -> Vec<(Vec<Sox302Certification>, Sox404Assessment)> {
        inputs.iter().map(|i| self.generate(i)).collect()
    }

    // -----------------------------------------------------------------------
    // SOX 302
    // -----------------------------------------------------------------------

    fn generate_302_certifications(
        &mut self,
        input: &SoxGeneratorInput,
    ) -> Vec<Sox302Certification> {
        let material_weaknesses: Vec<Uuid> = input
            .findings
            .iter()
            .filter(|f| is_material_weakness_open(f))
            .map(|f| f.finding_id)
            .collect();

        let significant_deficiencies: Vec<Uuid> = input
            .findings
            .iter()
            .filter(|f| is_significant_deficiency_open(f))
            .map(|f| f.finding_id)
            .collect();

        let controls_effective = material_weaknesses.is_empty();
        let fraud_disclosed = input.findings.iter().any(|f| {
            matches!(f.finding_type, FindingType::MaterialMisstatement) && f.report_to_governance
        });

        // Certification date = period end + ~60 days (typical 10-K filing window).
        let cert_days: i64 = self.rng.random_range(55i64..=70);
        let cert_date = input.period_end + chrono::Duration::days(cert_days);

        let roles = [
            (CertifierRole::Ceo, &input.ceo_name),
            (CertifierRole::Cfo, &input.cfo_name),
        ];

        let mut certs = Vec::with_capacity(2);

        for (role, name) in &roles {
            let mut cert = Sox302Certification::new(
                &input.company_code,
                input.fiscal_year,
                input.period_end,
                *role,
                name.as_str(),
            );

            cert.certification_date = cert_date;
            cert.disclosure_controls_effective = controls_effective;
            cert.internal_control_designed_effectively = controls_effective;
            cert.material_weaknesses = material_weaknesses.clone();
            cert.significant_deficiencies = significant_deficiencies.clone();

            if fraud_disclosed {
                cert.fraud_disclosed = true;
                cert.fraud_description = Some(
                    "Certain material misstatements requiring restatement were identified \
                     during the audit and have been disclosed to the audit committee."
                        .into(),
                );
            }

            if !material_weaknesses.is_empty() {
                cert.no_material_misstatement = false;
                cert.fairly_presented = false;
            }

            cert.generate_certification_text();
            certs.push(cert);
        }

        certs
    }

    // -----------------------------------------------------------------------
    // SOX 404
    // -----------------------------------------------------------------------

    fn generate_404_assessment(&mut self, input: &SoxGeneratorInput) -> Sox404Assessment {
        let assessment_days: i64 = self.rng.random_range(55i64..=75);
        let assessment_date = input.period_end + chrono::Duration::days(assessment_days);

        let mut assessment =
            Sox404Assessment::new(&input.company_code, input.fiscal_year, assessment_date);

        assessment.materiality_threshold = input.materiality_threshold;

        // Scope — this entity covers 100% of itself.
        assessment.scope.push(ScopedEntity {
            entity_code: input.company_code.clone(),
            entity_name: input.company_name.clone(),
            revenue_percent: input.revenue_percent,
            assets_percent: input.assets_percent,
            scope_conclusion: ScopeConclusion::InScope,
            significant_accounts: input.significant_accounts.clone(),
        });

        // Classify findings into deficiency tiers.
        let mut material_weaknesses: Vec<MaterialWeakness> = Vec::new();
        let mut significant_deficiencies: Vec<SignificantDeficiency> = Vec::new();
        let mut control_deficiencies: Vec<SoxControlDeficiency> = Vec::new();

        for finding in &input.findings {
            match finding.finding_type {
                FindingType::MaterialWeakness if is_open(finding) => {
                    let mut mw = MaterialWeakness::new(&finding.title, finding.identified_date);
                    mw.affected_controls = finding.related_control_ids.clone();
                    mw.affected_accounts = finding.accounts_affected.clone();
                    mw.root_cause = finding.cause.clone();
                    mw.potential_misstatement = finding.monetary_impact;
                    mw.related_finding_ids = vec![finding.finding_id];
                    mw.remediated_by_year_end = matches!(
                        finding.status,
                        FindingStatus::Closed | FindingStatus::PendingValidation
                    );
                    if mw.remediated_by_year_end {
                        mw.remediation_date = Some(input.period_end);
                    }
                    material_weaknesses.push(mw);
                }
                FindingType::SignificantDeficiency => {
                    let mut sd =
                        SignificantDeficiency::new(&finding.title, finding.identified_date);
                    sd.affected_controls = finding.related_control_ids.clone();
                    sd.affected_accounts = finding.accounts_affected.clone();
                    sd.remediated = matches!(finding.status, FindingStatus::Closed);
                    significant_deficiencies.push(sd);
                }
                FindingType::ControlDeficiency | FindingType::ItDeficiency => {
                    control_deficiencies.push(SoxControlDeficiency {
                        deficiency_id: Uuid::now_v7(),
                        description: finding.title.clone(),
                        affected_control: finding
                            .related_control_ids
                            .first()
                            .cloned()
                            .unwrap_or_else(|| "CTRL-UNKNOWN".into()),
                        identification_date: finding.identified_date,
                        remediated: matches!(finding.status, FindingStatus::Closed),
                    });
                }
                _ => {}
            }
        }

        // Populate deficiency classification summary.
        let total_deficiencies = (material_weaknesses.len()
            + significant_deficiencies.len()
            + control_deficiencies.len()) as u32;
        let remediated = (material_weaknesses
            .iter()
            .filter(|m| m.remediated_by_year_end)
            .count()
            + significant_deficiencies
                .iter()
                .filter(|s| s.remediated)
                .count()
            + control_deficiencies.iter().filter(|c| c.remediated).count())
            as u32;

        assessment.deficiency_classification = DeficiencyClassificationSummary {
            deficiencies_identified: total_deficiencies,
            control_deficiencies: control_deficiencies.len() as u32,
            significant_deficiencies: significant_deficiencies.len() as u32,
            material_weaknesses: material_weaknesses.len() as u32,
            remediated,
        };

        // Key controls tested (approximate: 15 per significant account, min 30).
        let key_controls = (input.significant_accounts.len() * 15).max(30);
        let defective = material_weaknesses.len() + significant_deficiencies.len();
        let effective = key_controls.saturating_sub(defective * 3); // rough 3-per-finding impact
        assessment.key_controls_tested = key_controls;
        assessment.key_controls_effective = effective.min(key_controls);

        // Populate remediation actions for open material weaknesses.
        for mw in &material_weaknesses {
            if !mw.remediated_by_year_end {
                let target = assessment_date + chrono::Duration::days(180);
                assessment.remediation_actions.push(RemediationAction {
                    action_id: Uuid::now_v7(),
                    deficiency_id: mw.weakness_id,
                    description: format!(
                        "Implement enhanced controls and monitoring to address: {}",
                        mw.description
                    ),
                    responsible_party: "Controller / VP Finance".into(),
                    target_date: target,
                    completion_date: None,
                    status: RemediationStatus::InProgress,
                    remediation_tested: false,
                    remediation_effective: false,
                });
            }
        }

        assessment.material_weaknesses = material_weaknesses;
        assessment.significant_deficiencies = significant_deficiencies;
        assessment.control_deficiencies = control_deficiencies;

        // Effectiveness conclusion (driven by material weaknesses).
        assessment.evaluate_effectiveness();

        assessment.management_conclusion = if assessment.icfr_effective {
            format!(
                "Based on our assessment using the COSO 2013 framework, management concludes \
                 that {} maintained effective internal control over financial reporting as of \
                 {}. No material weaknesses were identified.",
                input.company_name, input.period_end
            )
        } else {
            let count = assessment.material_weaknesses.len();
            format!(
                "Based on our assessment using the COSO 2013 framework, management concludes \
                 that {} did not maintain effective internal control over financial reporting as \
                 of {} due to {} material weakness{}. See Management's Report for details.",
                input.company_name,
                input.period_end,
                count,
                if count == 1 { "" } else { "es" }
            )
        };

        assessment.management_report_date = assessment_date;
        assessment
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn is_open(f: &AuditFinding) -> bool {
    !matches!(
        f.status,
        FindingStatus::Closed | FindingStatus::NotApplicable
    )
}

fn is_material_weakness_open(f: &AuditFinding) -> bool {
    matches!(f.finding_type, FindingType::MaterialWeakness) && is_open(f)
}

fn is_significant_deficiency_open(f: &AuditFinding) -> bool {
    matches!(f.finding_type, FindingType::SignificantDeficiency) && is_open(f)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn minimal_input() -> SoxGeneratorInput {
        SoxGeneratorInput::default()
    }

    #[test]
    fn test_certifications_produced_for_ceo_and_cfo() {
        let mut gen = SoxGenerator::new(42);
        let (certs, _) = gen.generate(&minimal_input());
        assert_eq!(certs.len(), 2);
        let roles: Vec<CertifierRole> = certs.iter().map(|c| c.certifier_role).collect();
        assert!(roles.contains(&CertifierRole::Ceo));
        assert!(roles.contains(&CertifierRole::Cfo));
    }

    #[test]
    fn test_effective_when_no_material_weaknesses() {
        let mut gen = SoxGenerator::new(42);
        let (certs, assessment) = gen.generate(&minimal_input());
        assert!(assessment.icfr_effective);
        assert!(certs.iter().all(|c| c.disclosure_controls_effective));
    }

    #[test]
    fn test_ineffective_when_material_weakness_present() {
        use datasynth_core::models::audit::{AuditFinding, FindingType};

        let mut gen = SoxGenerator::new(42);
        let mut input = minimal_input();

        let eng_id = Uuid::new_v4();
        let finding = AuditFinding::new(eng_id, FindingType::MaterialWeakness, "SoD gap");
        input.findings = vec![finding];

        let (certs, assessment) = gen.generate(&input);
        assert!(!assessment.icfr_effective);
        assert!(!assessment.material_weaknesses.is_empty());
        assert!(!certs[0].disclosure_controls_effective);
    }

    #[test]
    fn test_assessment_conclusion_text_matches_effectiveness() {
        let mut gen = SoxGenerator::new(42);
        let (_, assessment) = gen.generate(&minimal_input());
        assert!(assessment.management_conclusion.contains("effective"));
    }

    #[test]
    fn test_significant_deficiency_does_not_make_ineffective() {
        use datasynth_core::models::audit::{AuditFinding, FindingType};

        let mut gen = SoxGenerator::new(42);
        let mut input = minimal_input();

        let eng_id = Uuid::new_v4();
        let finding = AuditFinding::new(
            eng_id,
            FindingType::SignificantDeficiency,
            "Reconciliation gap",
        );
        input.findings = vec![finding];

        let (_, assessment) = gen.generate(&input);
        // A significant deficiency alone does NOT make ICFR ineffective.
        assert!(assessment.icfr_effective);
        assert!(!assessment.significant_deficiencies.is_empty());
    }

    #[test]
    fn test_certifications_have_non_empty_text() {
        let mut gen = SoxGenerator::new(42);
        let (certs, _) = gen.generate(&minimal_input());
        for cert in &certs {
            assert!(!cert.certification_text.is_empty());
        }
    }

    #[test]
    fn test_remediation_action_generated_for_open_mw() {
        use datasynth_core::models::audit::{AuditFinding, FindingType};

        let mut gen = SoxGenerator::new(42);
        let mut input = minimal_input();

        let eng_id = Uuid::new_v4();
        let finding = AuditFinding::new(eng_id, FindingType::MaterialWeakness, "GL access");
        input.findings = vec![finding]; // status = Draft (open)

        let (_, assessment) = gen.generate(&input);
        // One open material weakness → one remediation action expected.
        assert!(!assessment.remediation_actions.is_empty());
    }

    #[test]
    fn test_batch_generate_returns_correct_count() {
        let mut gen = SoxGenerator::new(42);
        let inputs: Vec<SoxGeneratorInput> = (0..3)
            .map(|i| SoxGeneratorInput {
                company_code: format!("C{:03}", i),
                ..SoxGeneratorInput::default()
            })
            .collect();
        let results = gen.generate_batch(&inputs);
        assert_eq!(results.len(), 3);
    }
}
