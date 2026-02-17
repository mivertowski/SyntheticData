//! ESG Anomaly Injector.
//!
//! Injects labeled anomalies into ESG data for ML ground-truth generation.
//! Each injected anomaly produces an [`EsgAnomalyLabel`] that records the
//! anomaly type, severity, affected record, and original vs. anomalous values.

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use datasynth_core::models::{
    EmissionRecord, MaterialityAssessment, SafetyMetric, SupplierEsgAssessment,
    WorkforceDiversityMetric,
};

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Types of ESG anomalies that can be injected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EsgAnomalyType {
    /// Reported emissions significantly lower than expected from activity data.
    GreenwashingIndicator,
    /// Diversity metrics unchanged across multiple periods.
    DiversityStagnation,
    /// High-risk supplier with suppressed risk flags or inflated scores.
    SupplyChainRisk,
    /// Missing or implausible data in key ESG metrics.
    DataQualityGap,
    /// Material topic without required disclosure.
    MissingDisclosure,
    /// Climate scenario assumptions inconsistent with reported strategy.
    ScenarioInconsistency,
}

/// Severity of the ESG anomaly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EsgAnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

// ---------------------------------------------------------------------------
// Label
// ---------------------------------------------------------------------------

/// A labeled ESG anomaly for ground truth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsgAnomalyLabel {
    /// Unique anomaly label identifier.
    pub id: String,
    /// Type of the anomaly.
    pub anomaly_type: EsgAnomalyType,
    /// Severity of the anomaly.
    pub severity: EsgAnomalySeverity,
    /// Kind of record affected (e.g. "emission_record", "supplier_assessment").
    pub record_type: String,
    /// ID of the affected record.
    pub record_id: String,
    /// Human-readable description of the anomaly.
    pub description: String,
    /// Original value (before injection).
    pub original_value: Option<String>,
    /// Anomalous value (after injection).
    pub anomalous_value: Option<String>,
}

// ---------------------------------------------------------------------------
// Injector
// ---------------------------------------------------------------------------

/// Injects anomalies into ESG data and produces ground-truth labels.
pub struct EsgAnomalyInjector {
    rng: ChaCha8Rng,
    anomaly_rate: f64,
    counter: u64,
}

impl EsgAnomalyInjector {
    /// Create a new ESG anomaly injector.
    pub fn new(seed: u64, anomaly_rate: f64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            anomaly_rate: anomaly_rate.clamp(0.0, 1.0),
            counter: 0,
        }
    }

    /// Inject greenwashing indicators by reducing reported emissions.
    pub fn inject_greenwashing(
        &mut self,
        emissions: &mut [EmissionRecord],
    ) -> Vec<EsgAnomalyLabel> {
        let mut labels = Vec::new();

        for record in emissions.iter_mut() {
            if self.rng.gen::<f64>() >= self.anomaly_rate {
                continue;
            }

            self.counter += 1;
            let original = record.co2e_tonnes;

            // Reduce reported emissions by 30-60%
            let reduction: f64 = self.rng.gen_range(0.30..0.60);
            let reduction_dec = Decimal::from_f64_retain(reduction).unwrap_or(dec!(0.40));
            record.co2e_tonnes = (original * (dec!(1) - reduction_dec)).round_dp(4);

            labels.push(EsgAnomalyLabel {
                id: format!("EA-{:06}", self.counter),
                anomaly_type: EsgAnomalyType::GreenwashingIndicator,
                severity: if reduction > 0.50 {
                    EsgAnomalySeverity::Critical
                } else {
                    EsgAnomalySeverity::High
                },
                record_type: "emission_record".to_string(),
                record_id: record.id.clone(),
                description: format!(
                    "Reported emissions reduced by {:.0}% below activity-based estimate",
                    reduction * 100.0
                ),
                original_value: Some(original.to_string()),
                anomalous_value: Some(record.co2e_tonnes.to_string()),
            });
        }

        labels
    }

    /// Inject diversity stagnation by flattening diversity metrics.
    pub fn inject_diversity_stagnation(
        &mut self,
        metrics: &mut [WorkforceDiversityMetric],
    ) -> Vec<EsgAnomalyLabel> {
        let mut labels = Vec::new();

        for metric in metrics.iter_mut() {
            if self.rng.gen::<f64>() >= self.anomaly_rate {
                continue;
            }

            self.counter += 1;
            let original = metric.percentage;

            // Set a suspiciously round, unchanging percentage
            metric.percentage = dec!(0.5000);

            labels.push(EsgAnomalyLabel {
                id: format!("EA-{:06}", self.counter),
                anomaly_type: EsgAnomalyType::DiversityStagnation,
                severity: EsgAnomalySeverity::Medium,
                record_type: "workforce_diversity_metric".to_string(),
                record_id: metric.id.clone(),
                description:
                    "Diversity metric unchanged — potential stagnation or data fabrication"
                        .to_string(),
                original_value: Some(original.to_string()),
                anomalous_value: Some(metric.percentage.to_string()),
            });
        }

        labels
    }

    /// Inject supply chain risk by inflating supplier ESG scores.
    pub fn inject_supply_chain_risk(
        &mut self,
        assessments: &mut [SupplierEsgAssessment],
    ) -> Vec<EsgAnomalyLabel> {
        let mut labels = Vec::new();

        for assessment in assessments.iter_mut() {
            if self.rng.gen::<f64>() >= self.anomaly_rate {
                continue;
            }

            self.counter += 1;
            let original_overall = assessment.overall_score;

            // Inflate scores by 20-40 points
            let inflation: f64 = self.rng.gen_range(20.0..40.0);
            let bump = Decimal::from_f64_retain(inflation).unwrap_or(dec!(30));
            assessment.environmental_score = (assessment.environmental_score + bump).min(dec!(100));
            assessment.social_score = (assessment.social_score + bump).min(dec!(100));
            assessment.governance_score = (assessment.governance_score + bump).min(dec!(100));
            assessment.overall_score = ((assessment.environmental_score
                + assessment.social_score
                + assessment.governance_score)
                / dec!(3))
            .round_dp(2);

            // Suppress risk flag
            assessment.risk_flag = datasynth_core::models::EsgRiskFlag::Low;
            assessment.corrective_actions_required = 0;

            labels.push(EsgAnomalyLabel {
                id: format!("EA-{:06}", self.counter),
                anomaly_type: EsgAnomalyType::SupplyChainRisk,
                severity: EsgAnomalySeverity::High,
                record_type: "supplier_esg_assessment".to_string(),
                record_id: assessment.id.clone(),
                description: format!(
                    "Supplier ESG score inflated from {:.1} to {:.1} with risk flag suppressed",
                    original_overall, assessment.overall_score
                ),
                original_value: Some(original_overall.to_string()),
                anomalous_value: Some(assessment.overall_score.to_string()),
            });
        }

        labels
    }

    /// Inject data quality gaps by zeroing safety metrics.
    pub fn inject_data_quality_gaps(
        &mut self,
        safety_metrics: &mut [SafetyMetric],
    ) -> Vec<EsgAnomalyLabel> {
        let mut labels = Vec::new();

        for metric in safety_metrics.iter_mut() {
            if self.rng.gen::<f64>() >= self.anomaly_rate {
                continue;
            }

            self.counter += 1;
            let original_trir = metric.trir;

            // Set implausible zero values
            metric.recordable_incidents = 0;
            metric.lost_time_incidents = 0;
            metric.days_away = 0;
            metric.trir = Decimal::ZERO;
            metric.ltir = Decimal::ZERO;
            metric.dart_rate = Decimal::ZERO;

            labels.push(EsgAnomalyLabel {
                id: format!("EA-{:06}", self.counter),
                anomaly_type: EsgAnomalyType::DataQualityGap,
                severity: EsgAnomalySeverity::Medium,
                record_type: "safety_metric".to_string(),
                record_id: metric.id.clone(),
                description: "Safety metrics suspiciously zeroed — possible data gap".to_string(),
                original_value: Some(format!("TRIR={}", original_trir)),
                anomalous_value: Some("TRIR=0".to_string()),
            });
        }

        labels
    }

    /// Inject missing disclosures by marking material topics as non-material.
    pub fn inject_missing_disclosures(
        &mut self,
        materiality: &mut [MaterialityAssessment],
    ) -> Vec<EsgAnomalyLabel> {
        let mut labels = Vec::new();

        for assessment in materiality.iter_mut() {
            if !assessment.is_material || self.rng.gen::<f64>() >= self.anomaly_rate {
                continue;
            }

            self.counter += 1;
            let original_impact = assessment.impact_score;
            let original_financial = assessment.financial_score;

            // Suppress materiality
            assessment.impact_score = dec!(0.20);
            assessment.financial_score = dec!(0.20);
            assessment.combined_score = dec!(0.20);
            assessment.is_material = false;

            labels.push(EsgAnomalyLabel {
                id: format!("EA-{:06}", self.counter),
                anomaly_type: EsgAnomalyType::MissingDisclosure,
                severity: EsgAnomalySeverity::Critical,
                record_type: "materiality_assessment".to_string(),
                record_id: assessment.id.clone(),
                description: format!(
                    "Material topic '{}' suppressed to avoid disclosure",
                    assessment.topic
                ),
                original_value: Some(format!(
                    "impact={}, financial={}",
                    original_impact, original_financial
                )),
                anomalous_value: Some("impact=0.20, financial=0.20".to_string()),
            });
        }

        labels
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use datasynth_core::models::{EmissionScope, EstimationMethod};

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    fn sample_emissions() -> Vec<EmissionRecord> {
        (0..10)
            .map(|i| EmissionRecord {
                id: format!("EM-{:03}", i),
                entity_id: "C001".into(),
                scope: EmissionScope::Scope1,
                scope3_category: None,
                facility_id: Some("FAC-001".into()),
                period: d("2025-01-01"),
                activity_data: None,
                activity_unit: None,
                emission_factor: Some(dec!(0.181)),
                co2e_tonnes: dec!(100.0),
                estimation_method: EstimationMethod::ActivityBased,
                source: None,
            })
            .collect()
    }

    #[test]
    fn test_greenwashing_injection() {
        let mut emissions = sample_emissions();
        let mut injector = EsgAnomalyInjector::new(42, 1.0); // 100% rate for testing
        let labels = injector.inject_greenwashing(&mut emissions);

        assert_eq!(labels.len(), 10);
        for (em, label) in emissions.iter().zip(labels.iter()) {
            assert!(em.co2e_tonnes < dec!(100.0), "Emissions should be reduced");
            assert_eq!(label.anomaly_type, EsgAnomalyType::GreenwashingIndicator);
            assert!(label.original_value.is_some());
        }
    }

    #[test]
    fn test_supply_chain_risk_injection() {
        let mut assessments = vec![SupplierEsgAssessment {
            id: "SA-001".into(),
            entity_id: "C001".into(),
            vendor_id: "V-001".into(),
            assessment_date: d("2025-06-01"),
            method: datasynth_core::models::AssessmentMethod::SelfAssessment,
            environmental_score: dec!(35),
            social_score: dec!(40),
            governance_score: dec!(30),
            overall_score: dec!(35),
            risk_flag: datasynth_core::models::EsgRiskFlag::High,
            corrective_actions_required: 3,
        }];

        let mut injector = EsgAnomalyInjector::new(42, 1.0);
        let labels = injector.inject_supply_chain_risk(&mut assessments);

        assert_eq!(labels.len(), 1);
        assert!(
            assessments[0].overall_score > dec!(35),
            "Score should be inflated"
        );
        assert_eq!(
            assessments[0].risk_flag,
            datasynth_core::models::EsgRiskFlag::Low
        );
        assert_eq!(assessments[0].corrective_actions_required, 0);
    }

    #[test]
    fn test_data_quality_gap_injection() {
        let mut metrics = vec![SafetyMetric {
            id: "SM-001".into(),
            entity_id: "C001".into(),
            period: d("2025-01-01"),
            total_hours_worked: 500_000,
            recordable_incidents: 5,
            lost_time_incidents: 2,
            days_away: 15,
            near_misses: 8,
            fatalities: 0,
            trir: dec!(2.0),
            ltir: dec!(0.8),
            dart_rate: dec!(6.0),
        }];

        let mut injector = EsgAnomalyInjector::new(42, 1.0);
        let labels = injector.inject_data_quality_gaps(&mut metrics);

        assert_eq!(labels.len(), 1);
        assert_eq!(metrics[0].trir, Decimal::ZERO);
        assert_eq!(metrics[0].recordable_incidents, 0);
        assert_eq!(labels[0].anomaly_type, EsgAnomalyType::DataQualityGap);
    }

    #[test]
    fn test_missing_disclosure_injection() {
        let mut materiality = vec![
            MaterialityAssessment {
                id: "MA-001".into(),
                entity_id: "C001".into(),
                period: d("2025-01-01"),
                topic: "GHG Emissions".into(),
                impact_score: dec!(0.85),
                financial_score: dec!(0.75),
                combined_score: dec!(0.80),
                is_material: true,
            },
            MaterialityAssessment {
                id: "MA-002".into(),
                entity_id: "C001".into(),
                period: d("2025-01-01"),
                topic: "Board Composition".into(),
                impact_score: dec!(0.30),
                financial_score: dec!(0.25),
                combined_score: dec!(0.275),
                is_material: false,
            },
        ];

        let mut injector = EsgAnomalyInjector::new(42, 1.0);
        let labels = injector.inject_missing_disclosures(&mut materiality);

        // Only the material one should be affected
        assert_eq!(labels.len(), 1);
        assert!(!materiality[0].is_material, "Should now be non-material");
        assert!(!materiality[1].is_material, "Was already non-material");
    }

    #[test]
    fn test_zero_anomaly_rate() {
        let mut emissions = sample_emissions();
        let mut injector = EsgAnomalyInjector::new(42, 0.0);
        let labels = injector.inject_greenwashing(&mut emissions);

        assert!(labels.is_empty(), "Zero rate should produce no anomalies");
        assert!(
            emissions.iter().all(|e| e.co2e_tonnes == dec!(100.0)),
            "No emissions should be modified"
        );
    }
}
