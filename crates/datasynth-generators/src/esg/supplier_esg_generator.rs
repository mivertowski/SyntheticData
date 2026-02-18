//! Supplier ESG assessment generator — derives ESG scores for vendors
//! correlated with quality and flags high-risk suppliers by country/industry.

use chrono::NaiveDate;
use datasynth_config::schema::SupplyChainEsgConfig;
use datasynth_core::models::{AssessmentMethod, EsgRiskFlag, SupplierEsgAssessment};
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Input representing an existing vendor to assess.
#[derive(Debug, Clone)]
pub struct VendorInput {
    pub vendor_id: String,
    pub country: String,
    pub industry: String,
    /// Optional quality score (0-100) to correlate ESG with.
    pub quality_score: Option<f64>,
}

/// Generates [`SupplierEsgAssessment`] records for vendors.
pub struct SupplierEsgGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
    config: SupplyChainEsgConfig,
    counter: u64,
}

impl SupplierEsgGenerator {
    /// Create a new supplier ESG generator.
    pub fn new(config: SupplyChainEsgConfig, seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::Esg),
            config,
            counter: 0,
        }
    }

    /// Generate ESG assessments for a set of vendors.
    ///
    /// Only assesses `assessment_coverage` fraction of vendors.
    pub fn generate(
        &mut self,
        entity_id: &str,
        vendors: &[VendorInput],
        assessment_date: NaiveDate,
    ) -> Vec<SupplierEsgAssessment> {
        if !self.config.enabled {
            return Vec::new();
        }

        // Determine which vendors to assess (must collect first to avoid borrow conflict)
        let assessed_indices: Vec<usize> = vendors
            .iter()
            .enumerate()
            .filter(|_| self.rng.gen::<f64>() < self.config.assessment_coverage)
            .map(|(i, _)| i)
            .collect();

        let mut assessments = Vec::with_capacity(assessed_indices.len());
        for idx in assessed_indices {
            let vendor = &vendors[idx];
            self.counter += 1;

            // Base scores: 40-90, correlated with quality if available
            let base = vendor.quality_score.unwrap_or(65.0);
            let env_score = self.score_with_noise(base, &vendor.country, &vendor.industry, "env");
            let soc_score = self.score_with_noise(base, &vendor.country, &vendor.industry, "soc");
            let gov_score = self.score_with_noise(base, &vendor.country, &vendor.industry, "gov");

            let env_dec = Decimal::from_f64_retain(env_score)
                .unwrap_or(dec!(50))
                .round_dp(2);
            let soc_dec = Decimal::from_f64_retain(soc_score)
                .unwrap_or(dec!(50))
                .round_dp(2);
            let gov_dec = Decimal::from_f64_retain(gov_score)
                .unwrap_or(dec!(50))
                .round_dp(2);
            let overall = ((env_dec + soc_dec + gov_dec) / dec!(3)).round_dp(2);

            let is_high_risk_country = self
                .config
                .high_risk_countries
                .iter()
                .any(|c| c == &vendor.country);

            let risk_flag = self.determine_risk(overall, is_high_risk_country);
            let corrective_actions = match risk_flag {
                EsgRiskFlag::Critical => self.rng.gen_range(3..8u32),
                EsgRiskFlag::High => self.rng.gen_range(1..5u32),
                EsgRiskFlag::Medium => self.rng.gen_range(0..3u32),
                EsgRiskFlag::Low => 0,
            };

            let method = self.pick_method();

            assessments.push(SupplierEsgAssessment {
                id: format!("SA-{:06}", self.counter),
                entity_id: entity_id.to_string(),
                vendor_id: vendor.vendor_id.clone(),
                assessment_date,
                method,
                environmental_score: env_dec,
                social_score: soc_dec,
                governance_score: gov_dec,
                overall_score: overall,
                risk_flag,
                corrective_actions_required: corrective_actions,
            });
        }

        assessments
    }

    /// Generate a score with noise, adjusting for country risk and industry.
    fn score_with_noise(
        &mut self,
        base_quality: f64,
        country: &str,
        industry: &str,
        _dimension: &str,
    ) -> f64 {
        let mut score = base_quality + self.rng.gen_range(-15.0..15.0);

        // Country risk adjustment
        if self.config.high_risk_countries.iter().any(|c| c == country) {
            score -= self.rng.gen_range(5.0..20.0);
        }

        // Industry adjustment: manufacturing and mining tend lower on environmental
        match industry {
            "manufacturing" | "mining" | "chemicals" => score -= self.rng.gen_range(0.0..10.0),
            "technology" | "professional_services" => score += self.rng.gen_range(0.0..5.0),
            _ => {}
        }

        score.clamp(0.0, 100.0)
    }

    fn determine_risk(&self, overall: Decimal, is_high_risk_country: bool) -> EsgRiskFlag {
        if overall < dec!(30) || (is_high_risk_country && overall < dec!(45)) {
            EsgRiskFlag::Critical
        } else if overall < dec!(50) || (is_high_risk_country && overall < dec!(60)) {
            EsgRiskFlag::High
        } else if overall < dec!(70) {
            EsgRiskFlag::Medium
        } else {
            EsgRiskFlag::Low
        }
    }

    fn pick_method(&mut self) -> AssessmentMethod {
        let roll: f64 = self.rng.gen::<f64>();
        if roll < 0.40 {
            AssessmentMethod::SelfAssessment
        } else if roll < 0.65 {
            AssessmentMethod::DocumentReview
        } else if roll < 0.85 {
            AssessmentMethod::ThirdPartyAudit
        } else {
            AssessmentMethod::OnSiteAssessment
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

    fn test_vendors() -> Vec<VendorInput> {
        vec![
            VendorInput {
                vendor_id: "V-001".into(),
                country: "US".into(),
                industry: "technology".into(),
                quality_score: Some(85.0),
            },
            VendorInput {
                vendor_id: "V-002".into(),
                country: "CN".into(),
                industry: "manufacturing".into(),
                quality_score: Some(60.0),
            },
            VendorInput {
                vendor_id: "V-003".into(),
                country: "DE".into(),
                industry: "professional_services".into(),
                quality_score: Some(90.0),
            },
            VendorInput {
                vendor_id: "V-004".into(),
                country: "BD".into(),
                industry: "manufacturing".into(),
                quality_score: Some(45.0),
            },
            VendorInput {
                vendor_id: "V-005".into(),
                country: "US".into(),
                industry: "mining".into(),
                quality_score: None,
            },
        ]
    }

    #[test]
    fn test_assessment_coverage() {
        let config = SupplyChainEsgConfig {
            enabled: true,
            assessment_coverage: 1.0, // Assess all
            high_risk_countries: vec!["CN".into(), "BD".into()],
        };
        let vendors = test_vendors();
        let mut gen = SupplierEsgGenerator::new(config, 42);
        let assessments = gen.generate("C001", &vendors, d("2025-06-01"));

        assert_eq!(
            assessments.len(),
            5,
            "100% coverage should assess all vendors"
        );
    }

    #[test]
    fn test_scores_in_range() {
        let config = SupplyChainEsgConfig::default();
        let vendors = test_vendors();
        let mut gen = SupplierEsgGenerator::new(config, 42);
        let assessments = gen.generate("C001", &vendors, d("2025-06-01"));

        for a in &assessments {
            assert!(a.environmental_score >= Decimal::ZERO && a.environmental_score <= dec!(100));
            assert!(a.social_score >= Decimal::ZERO && a.social_score <= dec!(100));
            assert!(a.governance_score >= Decimal::ZERO && a.governance_score <= dec!(100));
            assert!(a.overall_score >= Decimal::ZERO && a.overall_score <= dec!(100));
        }
    }

    #[test]
    fn test_high_risk_country_flagging() {
        let config = SupplyChainEsgConfig {
            enabled: true,
            assessment_coverage: 1.0,
            high_risk_countries: vec!["CN".into(), "BD".into()],
        };
        let vendors = vec![VendorInput {
            vendor_id: "V-LOW".into(),
            country: "BD".into(),
            industry: "manufacturing".into(),
            quality_score: Some(40.0),
        }];
        let mut gen = SupplierEsgGenerator::new(config, 42);
        let assessments = gen.generate("C001", &vendors, d("2025-06-01"));

        assert_eq!(assessments.len(), 1);
        // Low quality + high risk country → should be High or Critical
        assert!(
            matches!(
                assessments[0].risk_flag,
                EsgRiskFlag::High | EsgRiskFlag::Critical
            ),
            "Low quality + high risk country should flag high/critical, got {:?}",
            assessments[0].risk_flag
        );
    }

    #[test]
    fn test_corrective_actions_by_risk() {
        let config = SupplyChainEsgConfig {
            enabled: true,
            assessment_coverage: 1.0,
            high_risk_countries: vec!["CN".into(), "BD".into()],
        };
        let vendors = test_vendors();
        let mut gen = SupplierEsgGenerator::new(config, 42);
        let assessments = gen.generate("C001", &vendors, d("2025-06-01"));

        for a in &assessments {
            match a.risk_flag {
                EsgRiskFlag::Low => assert_eq!(a.corrective_actions_required, 0),
                EsgRiskFlag::Critical => assert!(a.corrective_actions_required >= 3),
                _ => {} // Medium and High have variable counts
            }
        }
    }

    #[test]
    fn test_disabled_produces_nothing() {
        let mut config = SupplyChainEsgConfig::default();
        config.enabled = false;
        let vendors = test_vendors();
        let mut gen = SupplierEsgGenerator::new(config, 42);
        let assessments = gen.generate("C001", &vendors, d("2025-06-01"));
        assert!(assessments.is_empty());
    }
}
