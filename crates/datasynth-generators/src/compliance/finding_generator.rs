//! Compliance finding generator.
//!
//! Generates compliance findings tied to audit procedures, with
//! deficiency classification per SOX/ISA and remediation tracking.

use chrono::NaiveDate;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;

use datasynth_core::models::compliance::{
    ComplianceAssertion, ComplianceFinding, DeficiencyLevel, FindingSeverity, RemediationStatus,
    StandardId,
};
use datasynth_core::utils::seeded_rng;

use super::procedure_generator::AuditProcedureRecord;

/// Configuration for compliance finding generation.
#[derive(Debug, Clone)]
pub struct ComplianceFindingGeneratorConfig {
    /// Rate of findings per procedure (0.0-1.0)
    pub finding_rate: f64,
    /// Rate of material weaknesses among findings
    pub material_weakness_rate: f64,
    /// Rate of significant deficiencies among findings
    pub significant_deficiency_rate: f64,
    /// Whether to generate remediation plans
    pub generate_remediation: bool,
}

impl Default for ComplianceFindingGeneratorConfig {
    fn default() -> Self {
        Self {
            finding_rate: 0.05,
            material_weakness_rate: 0.02,
            significant_deficiency_rate: 0.08,
            generate_remediation: true,
        }
    }
}

/// Finding templates with condition/criteria/cause/effect structure.
const FINDING_TEMPLATES: &[(&str, &str, &str)] = &[
    (
        "Revenue cutoff exception",
        "Revenue was recognized in the incorrect period due to delayed shipment recording",
        "Cutoff",
    ),
    (
        "Three-way match failure",
        "Purchase order, goods receipt, and invoice amounts did not agree within tolerance",
        "Accuracy",
    ),
    (
        "Segregation of duties violation",
        "Same user created and approved the transaction, violating SoD policy",
        "Occurrence",
    ),
    (
        "Inadequate journal entry review",
        "Manual journal entries were posted without required supervisory approval",
        "Occurrence",
    ),
    (
        "Inventory valuation discrepancy",
        "Physical inventory count differed from book records by more than tolerable threshold",
        "ValuationAndAllocation",
    ),
    (
        "Fixed asset existence",
        "Selected fixed assets could not be physically verified during inspection",
        "Existence",
    ),
    (
        "Related party disclosure gap",
        "Related party transactions were not fully disclosed in the financial statements",
        "CompletenessDisclosure",
    ),
    (
        "Lease classification error",
        "Operating lease incorrectly classified as finance lease under ASC 842/IFRS 16",
        "Classification",
    ),
    (
        "Revenue recognition timing",
        "Performance obligation satisfied over time incorrectly recognized at point in time",
        "Accuracy",
    ),
    (
        "Bank reconciliation delay",
        "Bank reconciliations not completed within 5 business days of month-end",
        "Timeliness",
    ),
];

/// Generator for compliance findings.
pub struct ComplianceFindingGenerator {
    rng: ChaCha8Rng,
    config: ComplianceFindingGeneratorConfig,
    counter: u32,
}

impl ComplianceFindingGenerator {
    /// Creates a new generator.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config: ComplianceFindingGeneratorConfig::default(),
            counter: 0,
        }
    }

    /// Creates a generator with custom configuration.
    pub fn with_config(seed: u64, config: ComplianceFindingGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config,
            counter: 0,
        }
    }

    /// Generates findings for a set of audit procedures.
    pub fn generate_findings(
        &mut self,
        procedures: &[AuditProcedureRecord],
        company_code: &str,
        reference_date: NaiveDate,
    ) -> Vec<ComplianceFinding> {
        let mut findings = Vec::new();

        for procedure in procedures {
            if self.rng.random::<f64>() > self.config.finding_rate {
                continue;
            }

            self.counter += 1;
            let template_idx = self.counter as usize % FINDING_TEMPLATES.len();
            let (title, description, assertion_str) = FINDING_TEMPLATES[template_idx];

            let deficiency_level = self.determine_deficiency_level();
            let severity = match deficiency_level {
                DeficiencyLevel::MaterialWeakness => FindingSeverity::High,
                DeficiencyLevel::SignificantDeficiency => FindingSeverity::Moderate,
                DeficiencyLevel::ControlDeficiency => FindingSeverity::Low,
            };

            let assertion = match assertion_str {
                "Occurrence" => ComplianceAssertion::Occurrence,
                "Completeness" => ComplianceAssertion::Completeness,
                "Accuracy" => ComplianceAssertion::Accuracy,
                "Cutoff" => ComplianceAssertion::Cutoff,
                "Classification" => ComplianceAssertion::Classification,
                "Existence" => ComplianceAssertion::Existence,
                "ValuationAndAllocation" => ComplianceAssertion::ValuationAndAllocation,
                "CompletenessDisclosure" => ComplianceAssertion::CompletenessDisclosure,
                "Timeliness" => ComplianceAssertion::Timeliness,
                _ => ComplianceAssertion::Occurrence,
            };

            let standard_id = StandardId::parse(&procedure.standard_id);

            let financial_impact = if matches!(
                deficiency_level,
                DeficiencyLevel::MaterialWeakness | DeficiencyLevel::SignificantDeficiency
            ) {
                let amount = self.rng.random_range(5_000i64..500_000i64);
                Some(Decimal::from(amount))
            } else {
                None
            };

            let remediation_status = if self.config.generate_remediation {
                let r: f64 = self.rng.random();
                if r < 0.3 {
                    RemediationStatus::Remediated
                } else if r < 0.7 {
                    RemediationStatus::InProgress
                } else {
                    RemediationStatus::Open
                }
            } else {
                RemediationStatus::Open
            };

            let is_repeat = self.rng.random::<f64>() < 0.15;

            let mut finding = ComplianceFinding::new(
                company_code,
                title,
                severity,
                deficiency_level,
                reference_date,
            )
            .with_description(description)
            .identified_by(&procedure.procedure_id)
            .with_assertion(assertion)
            .with_standard(standard_id)
            .with_remediation(remediation_status);

            if is_repeat {
                finding = finding.as_repeat();
            }

            if let Some(impact) = financial_impact {
                finding.financial_impact = Some(impact);
            }

            findings.push(finding);
        }

        findings
    }

    fn determine_deficiency_level(&mut self) -> DeficiencyLevel {
        let r: f64 = self.rng.random();
        if r < self.config.material_weakness_rate {
            DeficiencyLevel::MaterialWeakness
        } else if r < self.config.material_weakness_rate + self.config.significant_deficiency_rate {
            DeficiencyLevel::SignificantDeficiency
        } else {
            DeficiencyLevel::ControlDeficiency
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::compliance::ProcedureGenerator;
    use datasynth_standards::registry::StandardRegistry;

    #[test]
    fn test_generate_findings() {
        let registry = StandardRegistry::with_built_in();
        let date = NaiveDate::from_ymd_opt(2025, 6, 30).unwrap();

        let mut proc_gen = ProcedureGenerator::new(42);
        let procedures = proc_gen.generate_procedures(&registry, "US", date);

        // Use a high finding rate for testing
        let config = ComplianceFindingGeneratorConfig {
            finding_rate: 1.0, // 100% for test
            ..Default::default()
        };
        let mut finding_gen = ComplianceFindingGenerator::with_config(42, config);
        let findings = finding_gen.generate_findings(&procedures, "C001", date);

        assert!(!findings.is_empty(), "Should generate findings");
        for f in &findings {
            assert_eq!(f.company_code, "C001");
            assert!(!f.related_standards.is_empty());
        }
    }
}
