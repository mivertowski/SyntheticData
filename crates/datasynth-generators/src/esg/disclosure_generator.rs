//! ESG disclosure and materiality generator — maps calculated metrics
//! to framework-specific standard IDs (GRI, ESRS, SASB, TCFD, ISSB)
//! and performs double-materiality assessments.

use chrono::NaiveDate;
use datasynth_config::schema::{ClimateScenarioConfig, EsgReportingConfig};
use datasynth_core::models::{
    AssuranceLevel, ClimateScenario, EsgDisclosure, EsgFramework, MaterialityAssessment,
    ScenarioType, TimeHorizon,
};
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Standard ESG topics with framework-specific disclosure IDs.
struct DisclosureTopic {
    topic: &'static str,
    gri_id: &'static str,
    esrs_id: &'static str,
}

const DISCLOSURE_TOPICS: &[DisclosureTopic] = &[
    DisclosureTopic { topic: "GHG Emissions - Scope 1", gri_id: "GRI 305-1", esrs_id: "ESRS E1-6" },
    DisclosureTopic { topic: "GHG Emissions - Scope 2", gri_id: "GRI 305-2", esrs_id: "ESRS E1-6" },
    DisclosureTopic { topic: "GHG Emissions - Scope 3", gri_id: "GRI 305-3", esrs_id: "ESRS E1-6" },
    DisclosureTopic { topic: "Energy Consumption", gri_id: "GRI 302-1", esrs_id: "ESRS E1-5" },
    DisclosureTopic { topic: "Water Withdrawal", gri_id: "GRI 303-3", esrs_id: "ESRS E3-4" },
    DisclosureTopic { topic: "Waste Generation", gri_id: "GRI 306-3", esrs_id: "ESRS E5-5" },
    DisclosureTopic { topic: "Workforce Diversity", gri_id: "GRI 405-1", esrs_id: "ESRS S1-12" },
    DisclosureTopic { topic: "Pay Equity", gri_id: "GRI 405-2", esrs_id: "ESRS S1-16" },
    DisclosureTopic { topic: "Occupational Safety", gri_id: "GRI 403-9", esrs_id: "ESRS S1-14" },
    DisclosureTopic { topic: "Board Composition", gri_id: "GRI 405-1", esrs_id: "ESRS G1-1" },
    DisclosureTopic { topic: "Anti-Corruption", gri_id: "GRI 205-3", esrs_id: "ESRS G1-4" },
    DisclosureTopic { topic: "Supply Chain Assessment", gri_id: "GRI 308-1", esrs_id: "ESRS S2-1" },
];

/// Generates [`EsgDisclosure`] and [`MaterialityAssessment`] records.
pub struct DisclosureGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
    config: EsgReportingConfig,
    climate_config: ClimateScenarioConfig,
    counter: u64,
}

impl DisclosureGenerator {
    /// Create a new disclosure generator.
    pub fn new(seed: u64, config: EsgReportingConfig, climate_config: ClimateScenarioConfig) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::Esg),
            config,
            climate_config,
            counter: 0,
        }
    }

    // ----- Materiality Assessment -----

    /// Perform double-materiality assessment for all standard topics.
    pub fn generate_materiality(
        &mut self,
        entity_id: &str,
        period: NaiveDate,
    ) -> Vec<MaterialityAssessment> {
        if !self.config.materiality_assessment {
            return Vec::new();
        }

        DISCLOSURE_TOPICS
            .iter()
            .map(|dt| {
                self.counter += 1;

                let impact_score = self.random_score();
                let financial_score = self.random_score();
                let combined = ((impact_score + financial_score) / dec!(2)).round_dp(2);

                let impact_threshold =
                    Decimal::from_f64_retain(self.config.impact_threshold).unwrap_or(dec!(0.6));
                let financial_threshold =
                    Decimal::from_f64_retain(self.config.financial_threshold).unwrap_or(dec!(0.6));

                let is_material =
                    impact_score >= impact_threshold || financial_score >= financial_threshold;

                MaterialityAssessment {
                    id: format!("MA-{:06}", self.counter),
                    entity_id: entity_id.to_string(),
                    period,
                    topic: dt.topic.to_string(),
                    impact_score,
                    financial_score,
                    combined_score: combined,
                    is_material,
                }
            })
            .collect()
    }

    // ----- Disclosures -----

    /// Generate disclosures for material topics under configured frameworks.
    pub fn generate_disclosures(
        &mut self,
        entity_id: &str,
        materiality: &[MaterialityAssessment],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Vec<EsgDisclosure> {
        if !self.config.enabled {
            return Vec::new();
        }

        let material_topics: Vec<&str> = materiality
            .iter()
            .filter(|m| m.is_material)
            .map(|m| m.topic.as_str())
            .collect();

        let frameworks = self.parse_frameworks();
        let mut disclosures = Vec::new();

        for framework in &frameworks {
            for dt in DISCLOSURE_TOPICS {
                // Only disclose material topics
                if !material_topics.contains(&dt.topic) {
                    continue;
                }

                self.counter += 1;

                let standard_id = match framework {
                    EsgFramework::Gri => dt.gri_id,
                    EsgFramework::Esrs => dt.esrs_id,
                    _ => dt.gri_id, // Fallback
                };

                let (metric_value, metric_unit) = self.metric_for_topic(dt.topic);

                let assurance_level = if self.rng.gen::<f64>() < 0.30 {
                    AssuranceLevel::Reasonable
                } else if self.rng.gen::<f64>() < 0.60 {
                    AssuranceLevel::Limited
                } else {
                    AssuranceLevel::None
                };

                disclosures.push(EsgDisclosure {
                    id: format!("ED-{:06}", self.counter),
                    entity_id: entity_id.to_string(),
                    reporting_period_start: start_date,
                    reporting_period_end: end_date,
                    framework: *framework,
                    assurance_level,
                    disclosure_topic: format!("{} ({})", dt.topic, standard_id),
                    metric_value,
                    metric_unit,
                    is_assured: !matches!(assurance_level, AssuranceLevel::None),
                });
            }
        }

        disclosures
    }

    // ----- Climate Scenarios -----

    /// Generate climate scenario analysis records.
    pub fn generate_climate_scenarios(
        &mut self,
        entity_id: &str,
    ) -> Vec<ClimateScenario> {
        if !self.climate_config.enabled {
            return Vec::new();
        }

        let scenarios = [
            (ScenarioType::WellBelow2C, "Paris-aligned net zero by 2050", dec!(1.5)),
            (ScenarioType::Orderly, "Orderly transition with moderate carbon pricing", dec!(2.0)),
            (ScenarioType::Disorderly, "Delayed policy action with abrupt transition", dec!(2.5)),
            (ScenarioType::HotHouse, "Business as usual with severe physical risks", dec!(4.0)),
        ];

        let horizons = [
            (TimeHorizon::Short, 5),
            (TimeHorizon::Medium, 10),
            (TimeHorizon::Long, 30),
        ];

        let mut records = Vec::new();

        for (scenario_type, description, temp_rise) in &scenarios {
            for (horizon, _years) in &horizons {
                self.counter += 1;

                // Transition risk: higher for orderly/well-below-2C in short-medium term
                let transition_risk = match (scenario_type, horizon) {
                    (ScenarioType::WellBelow2C, TimeHorizon::Short) => self.random_impact(0.3, 0.7),
                    (ScenarioType::WellBelow2C, _) => self.random_impact(0.2, 0.5),
                    (ScenarioType::Orderly, _) => self.random_impact(0.15, 0.4),
                    (ScenarioType::Disorderly, TimeHorizon::Medium) => self.random_impact(0.4, 0.8),
                    (ScenarioType::HotHouse, _) => self.random_impact(0.05, 0.2),
                    _ => self.random_impact(0.1, 0.5),
                };

                // Physical risk: higher for hot house, especially long term
                let physical_risk = match (scenario_type, horizon) {
                    (ScenarioType::HotHouse, TimeHorizon::Long) => self.random_impact(0.5, 0.9),
                    (ScenarioType::HotHouse, _) => self.random_impact(0.3, 0.6),
                    (ScenarioType::Disorderly, TimeHorizon::Long) => self.random_impact(0.2, 0.5),
                    (ScenarioType::WellBelow2C, _) => self.random_impact(0.05, 0.15),
                    _ => self.random_impact(0.1, 0.3),
                };

                // Financial impact = weighted combo
                let financial = ((transition_risk * dec!(0.6) + physical_risk * dec!(0.4))
                    * dec!(100))
                .round_dp(2);

                records.push(ClimateScenario {
                    id: format!("CS-{:06}", self.counter),
                    entity_id: entity_id.to_string(),
                    scenario_type: *scenario_type,
                    time_horizon: *horizon,
                    description: description.to_string(),
                    temperature_rise_c: *temp_rise,
                    transition_risk_impact: transition_risk,
                    physical_risk_impact: physical_risk,
                    financial_impact: financial,
                });
            }
        }

        records
    }

    fn parse_frameworks(&self) -> Vec<EsgFramework> {
        self.config
            .frameworks
            .iter()
            .filter_map(|f| match f.to_uppercase().as_str() {
                "GRI" => Some(EsgFramework::Gri),
                "ESRS" => Some(EsgFramework::Esrs),
                "SASB" => Some(EsgFramework::Sasb),
                "TCFD" => Some(EsgFramework::Tcfd),
                "ISSB" => Some(EsgFramework::Issb),
                _ => None,
            })
            .collect()
    }

    fn random_score(&mut self) -> Decimal {
        let v: f64 = self.rng.gen_range(0.2..0.95);
        Decimal::from_f64_retain(v).unwrap_or(dec!(0.5)).round_dp(2)
    }

    fn random_impact(&mut self, min: f64, max: f64) -> Decimal {
        let v: f64 = self.rng.gen_range(min..max);
        Decimal::from_f64_retain(v).unwrap_or(dec!(0.3)).round_dp(4)
    }

    fn metric_for_topic(&mut self, topic: &str) -> (String, String) {
        match topic {
            "GHG Emissions - Scope 1" | "GHG Emissions - Scope 2" | "GHG Emissions - Scope 3" => {
                let val: f64 = self.rng.gen_range(100.0..50000.0);
                (format!("{:.1}", val), "tonnes CO2e".to_string())
            }
            "Energy Consumption" => {
                let val: f64 = self.rng.gen_range(1_000_000.0..50_000_000.0);
                (format!("{:.0}", val), "kWh".to_string())
            }
            "Water Withdrawal" => {
                let val: f64 = self.rng.gen_range(10_000.0..500_000.0);
                (format!("{:.0}", val), "m3".to_string())
            }
            "Waste Generation" => {
                let val: f64 = self.rng.gen_range(100.0..10_000.0);
                (format!("{:.1}", val), "tonnes".to_string())
            }
            "Workforce Diversity" => {
                let val: f64 = self.rng.gen_range(30.0..55.0);
                (format!("{:.1}%", val), "percent female".to_string())
            }
            "Pay Equity" => {
                let val: f64 = self.rng.gen_range(0.85..1.05);
                (format!("{:.3}", val), "ratio".to_string())
            }
            "Occupational Safety" => {
                let val: f64 = self.rng.gen_range(0.5..5.0);
                (format!("{:.2}", val), "TRIR".to_string())
            }
            "Board Composition" => {
                let val: f64 = self.rng.gen_range(0.50..0.80);
                (format!("{:.1}%", val * 100.0), "percent independent".to_string())
            }
            "Anti-Corruption" => {
                let val: u32 = self.rng.gen_range(0..3);
                (val.to_string(), "violations".to_string())
            }
            "Supply Chain Assessment" => {
                let val: f64 = self.rng.gen_range(60.0..95.0);
                (format!("{:.1}%", val), "percent assessed".to_string())
            }
            _ => ("N/A".to_string(), "N/A".to_string()),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn test_materiality_assessment() {
        let config = EsgReportingConfig::default();
        let climate = ClimateScenarioConfig::default();
        let mut gen = DisclosureGenerator::new(42, config, climate);

        let assessments = gen.generate_materiality("C001", d("2025-01-01"));

        assert_eq!(assessments.len(), DISCLOSURE_TOPICS.len());
        // Some should be material, some not (with default 0.6 threshold)
        let material = assessments.iter().filter(|a| a.is_material).count();
        assert!(
            material > 0 && material < assessments.len(),
            "Expected mix of material/non-material, got {}/{}",
            material,
            assessments.len()
        );
    }

    #[test]
    fn test_all_material_topics_have_disclosures() {
        let config = EsgReportingConfig::default();
        let climate = ClimateScenarioConfig::default();
        let mut gen = DisclosureGenerator::new(42, config, climate);

        let materiality = gen.generate_materiality("C001", d("2025-01-01"));
        let disclosures = gen.generate_disclosures(
            "C001",
            &materiality,
            d("2025-01-01"),
            d("2025-12-31"),
        );

        let material_topics: Vec<_> = materiality
            .iter()
            .filter(|m| m.is_material)
            .map(|m| m.topic.as_str())
            .collect();

        // Each material topic should have at least one disclosure per framework
        for topic in &material_topics {
            let has_disclosure = disclosures
                .iter()
                .any(|d| d.disclosure_topic.contains(topic));
            assert!(
                has_disclosure,
                "Material topic '{}' should have a disclosure",
                topic
            );
        }
    }

    #[test]
    fn test_framework_ids_are_valid() {
        let config = EsgReportingConfig::default();
        let climate = ClimateScenarioConfig::default();
        let mut gen = DisclosureGenerator::new(42, config, climate);

        let materiality = gen.generate_materiality("C001", d("2025-01-01"));
        let disclosures = gen.generate_disclosures(
            "C001",
            &materiality,
            d("2025-01-01"),
            d("2025-12-31"),
        );

        for d in &disclosures {
            // Should contain a framework-specific ID like "GRI 305-1" or "ESRS E1-6"
            assert!(
                d.disclosure_topic.contains("GRI") || d.disclosure_topic.contains("ESRS"),
                "Disclosure topic should contain framework ID: {}",
                d.disclosure_topic
            );
        }
    }

    #[test]
    fn test_climate_scenarios() {
        let config = EsgReportingConfig::default();
        let climate = ClimateScenarioConfig {
            enabled: true,
            scenarios: vec![
                "net_zero_2050".into(),
                "stated_policies".into(),
                "current_trajectory".into(),
            ],
            time_horizons: vec![5, 10, 30],
        };
        let mut gen = DisclosureGenerator::new(42, config, climate);
        let scenarios = gen.generate_climate_scenarios("C001");

        // 4 scenario types × 3 horizons = 12
        assert_eq!(scenarios.len(), 12);

        // Hot house should have highest physical risk in long term
        let hot_house_long: Vec<_> = scenarios
            .iter()
            .filter(|s| {
                s.scenario_type == ScenarioType::HotHouse && s.time_horizon == TimeHorizon::Long
            })
            .collect();
        assert_eq!(hot_house_long.len(), 1);
        assert!(hot_house_long[0].physical_risk_impact > dec!(0.4));
    }

    #[test]
    fn test_climate_disabled() {
        let config = EsgReportingConfig::default();
        let climate = ClimateScenarioConfig {
            enabled: false,
            ..Default::default()
        };
        let mut gen = DisclosureGenerator::new(42, config, climate);
        let scenarios = gen.generate_climate_scenarios("C001");
        assert!(scenarios.is_empty());
    }

    #[test]
    fn test_disclosure_assurance_levels() {
        let config = EsgReportingConfig::default();
        let climate = ClimateScenarioConfig::default();
        let mut gen = DisclosureGenerator::new(42, config, climate);

        let materiality = gen.generate_materiality("C001", d("2025-01-01"));
        let disclosures = gen.generate_disclosures(
            "C001",
            &materiality,
            d("2025-01-01"),
            d("2025-12-31"),
        );

        // Check is_assured matches assurance_level
        for d in &disclosures {
            assert_eq!(
                d.is_assured,
                !matches!(d.assurance_level, AssuranceLevel::None),
                "is_assured should match assurance_level"
            );
        }
    }
}
