//! Combined Risk Assessment (CRA) generator per ISA 315.
//!
//! For each entity the generator produces one `CombinedRiskAssessment` per
//! (account area, assertion) combination drawn from a set of 12 standard
//! account areas.  Inherent risk is driven by the economic nature of each
//! account area; control risk can be overridden from external control-
//! effectiveness data (e.g. from `InternalControl` records).
//!
//! # Significant risk rules (ISA 315.28 / ISA 240)
//!
//! The following are always flagged as significant risks, regardless of CRA level:
//! - Revenue / Occurrence (presumed fraud risk per ISA 240.26)
//! - Related Party / Occurrence (related-party transactions)
//! - Accounting Estimates / Valuation (high estimation uncertainty)

use datasynth_core::models::audit::risk_assessment_cra::{
    AuditAssertion, CombinedRiskAssessment, RiskRating,
};
use datasynth_core::utils::seeded_rng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use tracing::{debug, info};

// ---------------------------------------------------------------------------
// Account area definition
// ---------------------------------------------------------------------------

/// An account area with its default inherent risk and the assertions to assess.
#[derive(Debug, Clone)]
struct AccountAreaSpec {
    /// Human-readable name (e.g. "Revenue").
    name: &'static str,
    /// Default inherent risk when no other information is available.
    default_ir: RiskRating,
    /// Assertions to generate CRAs for.
    assertions: &'static [AuditAssertion],
    /// Whether Revenue/Occurrence significant-risk rule applies.
    always_significant_occurrence: bool,
}

/// Standard account areas per ISA 315 / typical audit scope.
static ACCOUNT_AREAS: &[AccountAreaSpec] = &[
    AccountAreaSpec {
        name: "Revenue",
        default_ir: RiskRating::High,
        assertions: &[
            AuditAssertion::Occurrence,
            AuditAssertion::Cutoff,
            AuditAssertion::Accuracy,
        ],
        always_significant_occurrence: true,
    },
    AccountAreaSpec {
        name: "Cost of Sales",
        default_ir: RiskRating::Medium,
        assertions: &[AuditAssertion::Occurrence, AuditAssertion::Accuracy],
        always_significant_occurrence: false,
    },
    AccountAreaSpec {
        name: "Trade Receivables",
        default_ir: RiskRating::High,
        assertions: &[
            AuditAssertion::Existence,
            AuditAssertion::ValuationAndAllocation,
        ],
        always_significant_occurrence: false,
    },
    AccountAreaSpec {
        name: "Inventory",
        default_ir: RiskRating::High,
        assertions: &[
            AuditAssertion::Existence,
            AuditAssertion::ValuationAndAllocation,
        ],
        always_significant_occurrence: false,
    },
    AccountAreaSpec {
        name: "Fixed Assets",
        default_ir: RiskRating::Medium,
        assertions: &[
            AuditAssertion::Existence,
            AuditAssertion::ValuationAndAllocation,
        ],
        always_significant_occurrence: false,
    },
    AccountAreaSpec {
        name: "Trade Payables",
        default_ir: RiskRating::Low,
        assertions: &[
            AuditAssertion::CompletenessBalance,
            AuditAssertion::Accuracy,
        ],
        always_significant_occurrence: false,
    },
    AccountAreaSpec {
        name: "Accruals",
        default_ir: RiskRating::Medium,
        assertions: &[
            AuditAssertion::CompletenessBalance,
            AuditAssertion::ValuationAndAllocation,
        ],
        always_significant_occurrence: false,
    },
    AccountAreaSpec {
        name: "Cash",
        default_ir: RiskRating::Low,
        assertions: &[
            AuditAssertion::Existence,
            AuditAssertion::CompletenessBalance,
        ],
        always_significant_occurrence: false,
    },
    AccountAreaSpec {
        name: "Tax",
        default_ir: RiskRating::Medium,
        assertions: &[
            AuditAssertion::Accuracy,
            AuditAssertion::ValuationAndAllocation,
        ],
        always_significant_occurrence: false,
    },
    AccountAreaSpec {
        name: "Equity",
        default_ir: RiskRating::Low,
        assertions: &[
            AuditAssertion::Existence,
            AuditAssertion::PresentationAndDisclosure,
        ],
        always_significant_occurrence: false,
    },
    AccountAreaSpec {
        name: "Provisions",
        default_ir: RiskRating::High,
        assertions: &[
            AuditAssertion::CompletenessBalance,
            AuditAssertion::ValuationAndAllocation,
        ],
        always_significant_occurrence: false,
    },
    AccountAreaSpec {
        name: "Related Parties",
        default_ir: RiskRating::High,
        assertions: &[AuditAssertion::Occurrence, AuditAssertion::Completeness],
        always_significant_occurrence: true,
    },
];

// ---------------------------------------------------------------------------
// Risk factors by account area
// ---------------------------------------------------------------------------

fn risk_factors_for(area: &str, assertion: AuditAssertion) -> Vec<String> {
    let mut factors: Vec<String> = Vec::new();

    match area {
        "Revenue" => {
            factors.push(
                "Revenue recognition involves judgment in identifying performance obligations"
                    .into(),
            );
            if assertion == AuditAssertion::Occurrence {
                factors.push(
                    "Presumed fraud risk per ISA 240 — incentive to overstate revenue".into(),
                );
            }
            if assertion == AuditAssertion::Cutoff {
                factors.push(
                    "Cut-off risk heightened near period-end due to shipping arrangements".into(),
                );
            }
        }
        "Trade Receivables" => {
            factors
                .push("Collectability assessment involves significant management judgment".into());
            if assertion == AuditAssertion::ValuationAndAllocation {
                factors.push(
                    "ECL provisioning methodology may be complex under IFRS 9 / ASC 310".into(),
                );
            }
        }
        "Inventory" => {
            factors.push("Physical quantities require verification through observation".into());
            if assertion == AuditAssertion::ValuationAndAllocation {
                factors
                    .push("NRV impairment requires management's forward-looking estimates".into());
            }
        }
        "Fixed Assets" => {
            factors
                .push("Capitalisation vs. expensing judgments affect reported asset values".into());
            if assertion == AuditAssertion::ValuationAndAllocation {
                factors
                    .push("Depreciation method and useful life estimates involve judgment".into());
            }
        }
        "Provisions" => {
            factors.push("Provisions are inherently uncertain and require estimation".into());
            factors.push("Completeness depends on management identifying all obligations".into());
        }
        "Related Parties" => {
            factors.push("Related party transactions may not be conducted at arm's length".into());
            factors.push(
                "Completeness depends on management disclosing all related party relationships"
                    .into(),
            );
        }
        "Accruals" => {
            factors.push(
                "Accrual completeness relies on management's identification of liabilities".into(),
            );
        }
        "Tax" => {
            factors
                .push("Tax provisions involve complex legislation and management judgment".into());
            factors.push(
                "Deferred tax calculation depends on timing difference identification".into(),
            );
        }
        _ => {
            factors.push(format!("{area} — standard inherent risk factors apply"));
        }
    }

    factors
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for the CRA generator.
#[derive(Debug, Clone)]
pub struct CraGeneratorConfig {
    /// Probability that control risk is Low (effective controls in place).
    pub effective_controls_probability: f64,
    /// Probability that control risk is Medium (partially effective).
    pub partial_controls_probability: f64,
    // Note: no_controls_probability = 1 - effective - partial
}

impl Default for CraGeneratorConfig {
    fn default() -> Self {
        Self {
            effective_controls_probability: 0.40,
            partial_controls_probability: 0.45,
        }
    }
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Generator for Combined Risk Assessments per ISA 315.
pub struct CraGenerator {
    rng: ChaCha8Rng,
    config: CraGeneratorConfig,
}

impl CraGenerator {
    /// Create a new generator with the given seed and default configuration.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0x315), // discriminator for ISA 315
            config: CraGeneratorConfig::default(),
        }
    }

    /// Create a new generator with custom configuration.
    pub fn with_config(seed: u64, config: CraGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0x315),
            config,
        }
    }

    /// Generate CRAs for all standard account areas for a single entity.
    ///
    /// # Arguments
    /// * `entity_code` — The entity being assessed.
    /// * `control_effectiveness` — Optional map from account area name to
    ///   control risk override.  When `None` for an area the generator picks
    ///   control risk randomly using the configured probabilities.
    pub fn generate_for_entity(
        &mut self,
        entity_code: &str,
        control_effectiveness: Option<&std::collections::HashMap<String, RiskRating>>,
    ) -> Vec<CombinedRiskAssessment> {
        info!("Generating CRAs for entity {}", entity_code);
        let mut results = Vec::new();

        for spec in ACCOUNT_AREAS {
            for &assertion in spec.assertions {
                let ir = self.jitter_inherent_risk(spec.default_ir);
                let cr = self.assess_control_risk(spec.name, control_effectiveness);

                // Determine significant risk flag
                let is_significant = self.is_significant_risk(spec, assertion, ir, cr);

                debug!(
                    "CRA: {} {:?} -> IR={:?} CR={:?} significant={}",
                    spec.name, assertion, ir, cr, is_significant
                );

                let risk_factors = risk_factors_for(spec.name, assertion);

                let cra = CombinedRiskAssessment::new(
                    entity_code,
                    spec.name,
                    assertion,
                    ir,
                    cr,
                    is_significant,
                    risk_factors,
                );

                results.push(cra);
            }
        }

        info!(
            "Generated {} CRAs for entity {}",
            results.len(),
            entity_code
        );
        results
    }

    /// Apply small random jitter to the default inherent risk so outputs vary.
    ///
    /// There is a 15% chance of moving one step up/down from the default,
    /// ensuring most assessments reflect the expected risk profile while
    /// allowing realistic variation.
    fn jitter_inherent_risk(&mut self, default: RiskRating) -> RiskRating {
        let roll: f64 = self.rng.random();
        match default {
            RiskRating::Low => {
                if roll > 0.85 {
                    RiskRating::Medium
                } else {
                    RiskRating::Low
                }
            }
            RiskRating::Medium => {
                if roll < 0.10 {
                    RiskRating::Low
                } else if roll > 0.85 {
                    RiskRating::High
                } else {
                    RiskRating::Medium
                }
            }
            RiskRating::High => {
                if roll > 0.85 {
                    RiskRating::Medium
                } else {
                    RiskRating::High
                }
            }
        }
    }

    /// Determine control risk for an account area.
    ///
    /// Uses the supplied override map if present, otherwise draws randomly
    /// according to the configured probabilities.
    fn assess_control_risk(
        &mut self,
        area: &str,
        overrides: Option<&std::collections::HashMap<String, RiskRating>>,
    ) -> RiskRating {
        if let Some(map) = overrides {
            if let Some(&cr) = map.get(area) {
                return cr;
            }
        }
        let roll: f64 = self.rng.random();
        if roll < self.config.effective_controls_probability {
            RiskRating::Low
        } else if roll
            < self.config.effective_controls_probability + self.config.partial_controls_probability
        {
            RiskRating::Medium
        } else {
            RiskRating::High
        }
    }

    /// Apply the significant risk rules per ISA 315.28, ISA 240, and ISA 501.
    fn is_significant_risk(
        &self,
        spec: &AccountAreaSpec,
        assertion: AuditAssertion,
        ir: RiskRating,
        _cr: RiskRating,
    ) -> bool {
        // Per ISA 240.26 — revenue occurrence is always presumed fraud risk
        if spec.always_significant_occurrence && assertion == AuditAssertion::Occurrence {
            return true;
        }
        // Per ISA 501 — inventory existence requires physical observation (always significant
        // when inherent risk is High, as quantities cannot be confirmed by other means).
        if spec.name == "Inventory"
            && assertion == AuditAssertion::Existence
            && ir == RiskRating::High
        {
            return true;
        }
        // High IR on high-judgment areas (Provisions, Estimates) is significant
        if ir == RiskRating::High
            && matches!(
                spec.name,
                "Provisions" | "Accruals" | "Trade Receivables" | "Inventory"
            )
            && assertion == AuditAssertion::ValuationAndAllocation
        {
            return true;
        }
        false
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn generates_cras_for_entity() {
        let mut gen = CraGenerator::new(42);
        let cras = gen.generate_for_entity("C001", None);
        // Should produce at least 12 CRAs (2 assertions × 12 areas minimum)
        assert!(!cras.is_empty());
        assert!(cras.len() >= 12);
    }

    #[test]
    fn revenue_occurrence_always_significant() {
        let mut gen = CraGenerator::new(42);
        let cras = gen.generate_for_entity("C001", None);
        let rev_occurrence = cras
            .iter()
            .find(|c| c.account_area == "Revenue" && c.assertion == AuditAssertion::Occurrence);
        assert!(
            rev_occurrence.is_some(),
            "Revenue/Occurrence CRA should exist"
        );
        assert!(
            rev_occurrence.unwrap().significant_risk,
            "Revenue/Occurrence must always be significant per ISA 240"
        );
    }

    #[test]
    fn related_party_occurrence_is_significant() {
        let mut gen = CraGenerator::new(42);
        let cras = gen.generate_for_entity("C001", None);
        let rp = cras.iter().find(|c| {
            c.account_area == "Related Parties" && c.assertion == AuditAssertion::Occurrence
        });
        assert!(rp.is_some());
        assert!(rp.unwrap().significant_risk);
    }

    #[test]
    fn cra_ids_are_unique() {
        let mut gen = CraGenerator::new(42);
        let cras = gen.generate_for_entity("C001", None);
        let ids: std::collections::HashSet<&str> = cras.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(ids.len(), cras.len(), "CRA IDs should be unique");
    }

    #[test]
    fn control_override_respected() {
        let mut overrides = std::collections::HashMap::new();
        overrides.insert("Cash".into(), RiskRating::Low);
        let mut gen = CraGenerator::new(42);
        let cras = gen.generate_for_entity("C001", Some(&overrides));
        let cash_cras: Vec<_> = cras.iter().filter(|c| c.account_area == "Cash").collect();
        for c in &cash_cras {
            assert_eq!(
                c.control_risk,
                RiskRating::Low,
                "Control override should apply"
            );
        }
    }
}
