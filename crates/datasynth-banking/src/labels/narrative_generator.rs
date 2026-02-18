//! Case narrative generator for SAR reports.

use chrono::NaiveDate;
use datasynth_core::models::banking::{AmlTypology, LaunderingStage, Sophistication};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::models::{
    AmlScenario, CaseNarrative, CaseRecommendation, RedFlag, RedFlagCategory, ViolatedExpectation,
};
use crate::seed_offsets::NARRATIVE_GENERATOR_SEED_OFFSET;

/// Narrative generator for AML cases.
pub struct NarrativeGenerator {
    rng: ChaCha8Rng,
}

impl NarrativeGenerator {
    /// Create a new narrative generator.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed.wrapping_add(NARRATIVE_GENERATOR_SEED_OFFSET)),
        }
    }

    /// Generate narrative for an AML scenario.
    pub fn generate(&mut self, scenario: &AmlScenario) -> CaseNarrative {
        let storyline = self.generate_storyline(scenario);
        let mut narrative = CaseNarrative::new(&storyline);

        // Add evidence points
        for evidence in self.generate_evidence_points(scenario) {
            narrative.add_evidence(&evidence);
        }

        // Add violated expectations
        for ve in self.generate_violated_expectations(scenario) {
            narrative.add_violated_expectation(ve);
        }

        // Add red flags
        let today = chrono::Utc::now().date_naive();
        for rf in self.generate_red_flags(scenario, today) {
            narrative.add_red_flag(rf);
        }

        // Set recommendation
        let recommendation = self.recommend_action(scenario);
        narrative.with_recommendation(recommendation)
    }

    /// Generate main storyline.
    fn generate_storyline(&mut self, scenario: &AmlScenario) -> String {
        let typology_desc = self.typology_description(scenario.typology);
        let sophistication_desc = self.sophistication_description(scenario.sophistication);
        let stage_desc = self.stages_description(&scenario.stages);

        format!(
            "Investigation identified {} activity pattern involving {} sophistication level. \
             The activity appears consistent with the {} stage(s) of money laundering. \
             Analysis period: {} to {}. \
             Total {} accounts involved in the suspicious activity cluster.",
            typology_desc,
            sophistication_desc,
            stage_desc,
            scenario.start_date.format("%Y-%m-%d"),
            scenario.end_date.format("%Y-%m-%d"),
            scenario.involved_accounts.len()
        )
    }

    /// Get typology description.
    fn typology_description(&self, typology: AmlTypology) -> &'static str {
        match typology {
            AmlTypology::Structuring => "cash deposit structuring",
            AmlTypology::Smurfing => "smurfing/structuring",
            AmlTypology::CuckooSmurfing => "cuckoo smurfing",
            AmlTypology::FunnelAccount => "funnel account aggregation",
            AmlTypology::ConcentrationAccount => "concentration account abuse",
            AmlTypology::PouchActivity => "pouch activity",
            AmlTypology::Layering => "complex layering chain",
            AmlTypology::RapidMovement => "rapid fund movement",
            AmlTypology::ShellCompany => "shell company network",
            AmlTypology::RoundTripping => "round-tripping fund movement",
            AmlTypology::TradeBasedML => "trade-based money laundering",
            AmlTypology::InvoiceManipulation => "invoice manipulation",
            AmlTypology::MoneyMule => "money mule operation",
            AmlTypology::RomanceScam => "romance scam activity",
            AmlTypology::AdvanceFeeFraud => "advance fee fraud",
            AmlTypology::RealEstateIntegration => "real estate-based integration",
            AmlTypology::LuxuryGoods => "luxury goods integration",
            AmlTypology::CasinoIntegration => "casino-based integration",
            AmlTypology::CryptoIntegration => "cryptocurrency integration",
            AmlTypology::AccountTakeover => "account takeover fraud",
            AmlTypology::SyntheticIdentity => "synthetic identity fraud",
            AmlTypology::FirstPartyFraud => "first-party fraud",
            AmlTypology::AuthorizedPushPayment => "authorized push payment fraud",
            AmlTypology::BusinessEmailCompromise => "business email compromise",
            AmlTypology::FakeVendor => "fake vendor fraud",
            AmlTypology::TerroristFinancing => "potential terrorist financing",
            AmlTypology::SanctionsEvasion => "potential sanctions evasion",
            AmlTypology::TaxEvasion => "tax evasion",
            AmlTypology::HumanTrafficking => "human trafficking related",
            AmlTypology::DrugTrafficking => "drug trafficking related",
            AmlTypology::Corruption => "corruption/PEP-related activity",
            AmlTypology::Custom(_) => "custom suspicious pattern",
        }
    }

    /// Get sophistication description.
    fn sophistication_description(&self, sophistication: Sophistication) -> &'static str {
        match sophistication {
            Sophistication::Basic => "basic/amateur",
            Sophistication::Standard => "standard/organized",
            Sophistication::Professional => "professional/systematic",
            Sophistication::Advanced => "advanced/coordinated network",
            Sophistication::StateLevel => "state-level/highly sophisticated",
        }
    }

    /// Get stages description.
    fn stages_description(&self, stages: &[LaunderingStage]) -> String {
        if stages.is_empty() {
            return "unclassified".to_string();
        }

        stages
            .iter()
            .map(|s| match s {
                LaunderingStage::Placement => "placement",
                LaunderingStage::Layering => "layering",
                LaunderingStage::Integration => "integration",
                LaunderingStage::NotApplicable => "N/A",
            })
            .collect::<Vec<_>>()
            .join("/")
    }

    /// Generate evidence points.
    fn generate_evidence_points(&mut self, scenario: &AmlScenario) -> Vec<String> {
        let mut points = Vec::new();

        // Add typology-specific evidence
        match scenario.typology {
            AmlTypology::Structuring | AmlTypology::Smurfing => {
                let deposit_count = self.rng.gen_range(5..20);
                let threshold = 10_000;
                points.push(format!(
                    "{} cash deposits below ${} reporting threshold within {} days",
                    deposit_count,
                    threshold,
                    (scenario.end_date - scenario.start_date).num_days()
                ));
                points.push("Deposits made at multiple branch locations".to_string());
                points.push("Immediate consolidation transfer following deposits".to_string());
            }
            AmlTypology::FunnelAccount => {
                let source_count = self.rng.gen_range(8..25);
                points.push(format!(
                    "{} unrelated inbound transfers from different sources",
                    source_count
                ));
                points.push("Rapid outward transfers within 24-48 hours of receipt".to_string());
                points.push("No business relationship with senders documented".to_string());
            }
            AmlTypology::Layering => {
                let hop_count = self.rng.gen_range(3..8);
                points.push(format!(
                    "Funds traced through {} intermediary accounts",
                    hop_count
                ));
                points.push("Systematic splitting and recombination of amounts".to_string());
                points.push("Time delays inserted between hops to avoid detection".to_string());
            }
            AmlTypology::MoneyMule => {
                points.push("New account with limited prior transaction history".to_string());
                points.push("Pattern of receive-and-forward within short timeframe".to_string());
                points.push(
                    "Cash withdrawals/wire transfers immediately following deposits".to_string(),
                );
                points.push("Small retention amount consistent with mule compensation".to_string());
            }
            _ => {
                points.push("Unusual transaction pattern identified".to_string());
                points.push("Activity inconsistent with stated account purpose".to_string());
            }
        }

        // Add sophistication-based evidence
        if matches!(
            scenario.sophistication,
            Sophistication::Professional | Sophistication::Advanced | Sophistication::StateLevel
        ) {
            points.push("Use of intermediary entities to obscure beneficial ownership".to_string());
        }
        if matches!(
            scenario.sophistication,
            Sophistication::Advanced | Sophistication::StateLevel
        ) {
            points.push(
                "Coordinated activity across multiple accounts and jurisdictions".to_string(),
            );
        }

        points
    }

    /// Generate violated expectations.
    fn generate_violated_expectations(
        &mut self,
        scenario: &AmlScenario,
    ) -> Vec<ViolatedExpectation> {
        let mut violations = Vec::new();

        // Transaction frequency violation
        let expected_freq = self.rng.gen_range(5..15);
        let actual_freq = self.rng.gen_range(25..100);
        violations.push(ViolatedExpectation::new(
            "Monthly transaction count",
            &format!("{}", expected_freq),
            &format!("{}", actual_freq),
            (actual_freq as f64 - expected_freq as f64) / expected_freq as f64 * 100.0,
        ));

        // Cash activity violation
        if matches!(
            scenario.typology,
            AmlTypology::Structuring | AmlTypology::Smurfing | AmlTypology::MoneyMule
        ) {
            let expected_cash = self.rng.gen_range(5..15);
            let actual_cash = self.rng.gen_range(40..80);
            violations.push(ViolatedExpectation::new(
                "Cash activity percentage",
                &format!("{}%", expected_cash),
                &format!("{}%", actual_cash),
                (actual_cash - expected_cash) as f64,
            ));
        }

        // Volume violation
        let expected_vol = self.rng.gen_range(5000..15000);
        let actual_vol = self.rng.gen_range(50000..250000);
        violations.push(ViolatedExpectation::new(
            "Monthly transaction volume",
            &format!("${}", expected_vol),
            &format!("${}", actual_vol),
            (actual_vol as f64 - expected_vol as f64) / expected_vol as f64 * 100.0,
        ));

        violations
    }

    /// Generate red flags.
    fn generate_red_flags(&mut self, scenario: &AmlScenario, date: NaiveDate) -> Vec<RedFlag> {
        let mut flags = Vec::new();

        // Common red flags
        flags.push(RedFlag::new(
            RedFlagCategory::ActivityPattern,
            "Funds moved rapidly through account with minimal dwell time",
            8,
            date,
        ));

        // Typology-specific red flags
        match scenario.typology {
            AmlTypology::Structuring | AmlTypology::Smurfing => {
                flags.push(RedFlag::new(
                    RedFlagCategory::TransactionCharacteristic,
                    "Multiple transactions just below $10,000 reporting threshold",
                    9,
                    date,
                ));
            }
            AmlTypology::FunnelAccount => {
                flags.push(RedFlag::new(
                    RedFlagCategory::ThirdParty,
                    "Multiple unrelated senders with no apparent business connection",
                    7,
                    date,
                ));
            }
            AmlTypology::MoneyMule => {
                flags.push(RedFlag::new(
                    RedFlagCategory::AccountCharacteristic,
                    "New account with unusually high activity",
                    7,
                    date,
                ));
                flags.push(RedFlag::new(
                    RedFlagCategory::ActivityPattern,
                    "Immediate cash withdrawals following electronic deposits",
                    9,
                    date,
                ));
            }
            _ => {}
        }

        // Sophistication-based flags
        if matches!(
            scenario.sophistication,
            Sophistication::Professional | Sophistication::Advanced | Sophistication::StateLevel
        ) {
            flags.push(RedFlag::new(
                RedFlagCategory::CustomerBehavior,
                "Complex ownership structure obscures beneficial owner",
                6,
                date,
            ));
        }

        flags
    }

    /// Recommend action based on scenario.
    fn recommend_action(&self, scenario: &AmlScenario) -> CaseRecommendation {
        // High severity typologies
        if matches!(
            scenario.typology,
            AmlTypology::SanctionsEvasion
                | AmlTypology::TerroristFinancing
                | AmlTypology::Corruption
        ) {
            return CaseRecommendation::ReportLawEnforcement;
        }

        // Sophisticated activity warrants SAR
        if matches!(
            scenario.sophistication,
            Sophistication::Professional | Sophistication::Advanced | Sophistication::StateLevel
        ) {
            return CaseRecommendation::FileSar;
        }

        // Mule accounts should be closed
        if scenario.typology == AmlTypology::MoneyMule {
            return CaseRecommendation::CloseAccount;
        }

        // Standard suspicious activity
        if matches!(scenario.sophistication, Sophistication::Standard) {
            return CaseRecommendation::FileSar;
        }

        // Basic activity - enhanced monitoring
        CaseRecommendation::EnhancedMonitoring
    }
}

/// Exported case narrative with full details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedNarrative {
    /// Case ID
    pub case_id: String,
    /// Main storyline
    pub storyline: String,
    /// Evidence points
    pub evidence_points: Vec<String>,
    /// Violated expectations
    pub violated_expectations: Vec<ExportedViolation>,
    /// Red flags
    pub red_flags: Vec<ExportedRedFlag>,
    /// Recommendation
    pub recommendation: String,
    /// Scenario metadata
    pub metadata: NarrativeMetadata,
}

/// Exported violated expectation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedViolation {
    pub expectation_type: String,
    pub expected: String,
    pub actual: String,
    pub deviation_percent: f64,
}

/// Exported red flag.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedRedFlag {
    pub category: String,
    pub description: String,
    pub severity: u8,
}

/// Narrative metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeMetadata {
    /// Typology
    pub typology: String,
    /// Sophistication level
    pub sophistication: String,
    /// Laundering stages
    pub stages: Vec<String>,
    /// Start date
    pub start_date: String,
    /// End date
    pub end_date: String,
    /// Account count
    pub account_count: usize,
    /// Detectability score
    pub detectability: f64,
}

impl ExportedNarrative {
    /// Create from scenario and narrative.
    pub fn from_scenario(scenario: &AmlScenario, narrative: &CaseNarrative) -> Self {
        Self {
            case_id: scenario.scenario_id.clone(),
            storyline: narrative.storyline.clone(),
            evidence_points: narrative.evidence_points.clone(),
            violated_expectations: narrative
                .violated_expectations
                .iter()
                .map(|ve| ExportedViolation {
                    expectation_type: ve.expectation_type.clone(),
                    expected: ve.expected_value.clone(),
                    actual: ve.actual_value.clone(),
                    deviation_percent: ve.deviation_percentage,
                })
                .collect(),
            red_flags: narrative
                .red_flags
                .iter()
                .map(|rf| ExportedRedFlag {
                    category: format!("{:?}", rf.category),
                    description: rf.description.clone(),
                    severity: rf.severity,
                })
                .collect(),
            recommendation: format!("{:?}", narrative.recommendation),
            metadata: NarrativeMetadata {
                typology: format!("{:?}", scenario.typology),
                sophistication: format!("{:?}", scenario.sophistication),
                stages: scenario.stages.iter().map(|s| format!("{:?}", s)).collect(),
                start_date: scenario.start_date.format("%Y-%m-%d").to_string(),
                end_date: scenario.end_date.format("%Y-%m-%d").to_string(),
                account_count: scenario.involved_accounts.len(),
                detectability: scenario.detectability,
            },
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_narrative_generation() {
        let mut generator = NarrativeGenerator::new(12345);

        let scenario = AmlScenario::new(
            "TEST-001",
            AmlTypology::Structuring,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        )
        .with_sophistication(Sophistication::Standard);

        let narrative = generator.generate(&scenario);

        assert!(!narrative.storyline.is_empty());
        assert!(!narrative.evidence_points.is_empty());
        assert!(!narrative.violated_expectations.is_empty());
        assert!(!narrative.red_flags.is_empty());
    }

    #[test]
    fn test_recommendation() {
        let generator = NarrativeGenerator::new(12345);

        // Professional sophistication -> SAR
        let scenario = AmlScenario::new(
            "TEST-001",
            AmlTypology::Structuring,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        )
        .with_sophistication(Sophistication::Professional);

        let rec = generator.recommend_action(&scenario);
        assert_eq!(rec, CaseRecommendation::FileSar);

        // Money mule -> Close account
        let scenario = AmlScenario::new(
            "TEST-002",
            AmlTypology::MoneyMule,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        )
        .with_sophistication(Sophistication::Basic);

        let rec = generator.recommend_action(&scenario);
        assert_eq!(rec, CaseRecommendation::CloseAccount);
    }
}
