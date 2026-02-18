//! Workforce ESG generator — derives diversity metrics, pay equity ratios,
//! safety incidents, and aggregate safety metrics from employee data.

use chrono::NaiveDate;
use datasynth_config::schema::SocialConfig;
use datasynth_core::models::{
    DiversityDimension, GovernanceMetric, IncidentType, OrganizationLevel, PayEquityMetric,
    SafetyIncident, SafetyMetric, WorkforceDiversityMetric,
};
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Generates workforce diversity, pay equity, and safety metrics.
pub struct WorkforceGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
    config: SocialConfig,
    counter: u64,
}

impl WorkforceGenerator {
    /// Create a new workforce generator.
    pub fn new(config: SocialConfig, seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::Esg),
            config,
            counter: 0,
        }
    }

    // ----- Diversity -----

    /// Generate workforce diversity metrics for a reporting period.
    ///
    /// Produces one record per (dimension × category × level) combination.
    pub fn generate_diversity(
        &mut self,
        entity_id: &str,
        total_headcount: u32,
        period: NaiveDate,
    ) -> Vec<WorkforceDiversityMetric> {
        if !self.config.diversity.enabled || total_headcount == 0 {
            return Vec::new();
        }

        let mut metrics = Vec::new();
        let levels = [
            OrganizationLevel::Corporate,
            OrganizationLevel::Executive,
            OrganizationLevel::Board,
        ];

        for dimension in &[
            DiversityDimension::Gender,
            DiversityDimension::Ethnicity,
            DiversityDimension::Age,
        ] {
            let categories = self.categories_for(*dimension);
            // Distribute headcount per level
            for level in &levels {
                let level_hc = match level {
                    OrganizationLevel::Corporate => total_headcount,
                    OrganizationLevel::Executive => (total_headcount / 50).max(5),
                    OrganizationLevel::Board => 11,
                    _ => total_headcount,
                };

                let shares = self.random_shares(categories.len());
                for (i, cat) in categories.iter().enumerate() {
                    self.counter += 1;
                    let headcount = (Decimal::from(level_hc)
                        * Decimal::from_f64_retain(shares[i]).unwrap_or(Decimal::ZERO))
                    .round_dp(0);
                    let hc = headcount.to_string().parse::<u32>().unwrap_or(0);

                    let percentage = if level_hc > 0 {
                        (Decimal::from(hc) / Decimal::from(level_hc)).round_dp(4)
                    } else {
                        Decimal::ZERO
                    };

                    metrics.push(WorkforceDiversityMetric {
                        id: format!("DV-{:06}", self.counter),
                        entity_id: entity_id.to_string(),
                        period,
                        dimension: *dimension,
                        level: *level,
                        category: cat.to_string(),
                        headcount: hc,
                        total_headcount: level_hc,
                        percentage,
                    });
                }
            }
        }

        metrics
    }

    fn categories_for(&self, dimension: DiversityDimension) -> Vec<&'static str> {
        match dimension {
            DiversityDimension::Gender => vec!["Male", "Female", "Non-Binary"],
            DiversityDimension::Ethnicity => {
                vec!["White", "Asian", "Hispanic", "Black", "Other"]
            }
            DiversityDimension::Age => {
                vec!["Under 30", "30-50", "Over 50"]
            }
            DiversityDimension::Disability => vec!["No Disability", "Has Disability"],
            DiversityDimension::VeteranStatus => vec!["Non-Veteran", "Veteran"],
        }
    }

    /// Generate random shares that sum to 1.0.
    fn random_shares(&mut self, count: usize) -> Vec<f64> {
        let mut raw: Vec<f64> = (0..count).map(|_| self.rng.gen::<f64>()).collect();
        let total: f64 = raw.iter().sum();
        if total > 0.0 {
            for v in &mut raw {
                *v /= total;
            }
        }
        raw
    }

    // ----- Pay Equity -----

    /// Generate pay equity metrics for common group comparisons.
    pub fn generate_pay_equity(
        &mut self,
        entity_id: &str,
        period: NaiveDate,
    ) -> Vec<PayEquityMetric> {
        if !self.config.pay_equity.enabled {
            return Vec::new();
        }

        let comparisons = [
            (DiversityDimension::Gender, "Male", "Female"),
            (DiversityDimension::Ethnicity, "White", "Asian"),
            (DiversityDimension::Ethnicity, "White", "Hispanic"),
            (DiversityDimension::Ethnicity, "White", "Black"),
        ];

        comparisons
            .iter()
            .map(|(dim, ref_group, cmp_group)| {
                self.counter += 1;
                let ref_salary: f64 = self.rng.gen_range(70_000.0..120_000.0);
                // Pay gap: comparison group earns 85-105% of reference
                let gap_factor: f64 = self.rng.gen_range(0.85..1.05);
                let cmp_salary = ref_salary * gap_factor;

                let ref_dec = Decimal::from_f64_retain(ref_salary)
                    .unwrap_or(dec!(90000))
                    .round_dp(2);
                let cmp_dec = Decimal::from_f64_retain(cmp_salary)
                    .unwrap_or(dec!(85000))
                    .round_dp(2);
                let ratio = if ref_dec.is_zero() {
                    dec!(1.00)
                } else {
                    (cmp_dec / ref_dec).round_dp(4)
                };

                PayEquityMetric {
                    id: format!("PE-{:06}", self.counter),
                    entity_id: entity_id.to_string(),
                    period,
                    dimension: *dim,
                    reference_group: ref_group.to_string(),
                    comparison_group: cmp_group.to_string(),
                    reference_median_salary: ref_dec,
                    comparison_median_salary: cmp_dec,
                    pay_gap_ratio: ratio,
                    sample_size: self.rng.gen_range(50..500),
                }
            })
            .collect()
    }

    // ----- Safety -----

    /// Generate safety incidents for a period.
    pub fn generate_safety_incidents(
        &mut self,
        entity_id: &str,
        facility_count: u32,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Vec<SafetyIncident> {
        if !self.config.safety.enabled {
            return Vec::new();
        }

        let total_incidents = self.config.safety.incident_count;
        let period_days = (end_date - start_date).num_days().max(1);

        (0..total_incidents)
            .map(|_| {
                self.counter += 1;
                let day_offset = self.rng.gen_range(0..period_days);
                let date = start_date + chrono::Duration::days(day_offset);
                let fac = self.rng.gen_range(1..=facility_count.max(1));

                let incident_type = self.pick_incident_type();
                let days_away = match incident_type {
                    IncidentType::Fatality => 0,
                    IncidentType::NearMiss | IncidentType::PropertyDamage => 0,
                    IncidentType::Injury => self.rng.gen_range(0..30u32),
                    IncidentType::Illness => self.rng.gen_range(1..15u32),
                };
                let is_recordable = !matches!(incident_type, IncidentType::NearMiss);

                let description = match incident_type {
                    IncidentType::Injury => "Workplace injury incident".to_string(),
                    IncidentType::Illness => "Occupational illness reported".to_string(),
                    IncidentType::NearMiss => "Near miss event documented".to_string(),
                    IncidentType::Fatality => "Fatal workplace incident".to_string(),
                    IncidentType::PropertyDamage => "Property damage incident".to_string(),
                };

                SafetyIncident {
                    id: format!("SI-{:06}", self.counter),
                    entity_id: entity_id.to_string(),
                    facility_id: format!("FAC-{:03}", fac),
                    date,
                    incident_type,
                    days_away,
                    is_recordable,
                    description,
                }
            })
            .collect()
    }

    fn pick_incident_type(&mut self) -> IncidentType {
        let roll: f64 = self.rng.gen::<f64>();
        if roll < 0.35 {
            IncidentType::NearMiss
        } else if roll < 0.65 {
            IncidentType::Injury
        } else if roll < 0.80 {
            IncidentType::Illness
        } else if roll < 0.95 {
            IncidentType::PropertyDamage
        } else {
            IncidentType::Fatality
        }
    }

    /// Compute aggregate safety metrics from incidents.
    pub fn compute_safety_metrics(
        &mut self,
        entity_id: &str,
        incidents: &[SafetyIncident],
        total_hours_worked: u64,
        period: NaiveDate,
    ) -> SafetyMetric {
        self.counter += 1;

        let recordable = incidents.iter().filter(|i| i.is_recordable).count() as u32;
        let lost_time = incidents.iter().filter(|i| i.days_away > 0).count() as u32;
        let days_away: u32 = incidents.iter().map(|i| i.days_away).sum();
        let near_misses = incidents
            .iter()
            .filter(|i| i.incident_type == IncidentType::NearMiss)
            .count() as u32;
        let fatalities = incidents
            .iter()
            .filter(|i| i.incident_type == IncidentType::Fatality)
            .count() as u32;

        let hours_dec = Decimal::from(total_hours_worked);
        let base = dec!(200000);

        let trir = if total_hours_worked > 0 {
            (Decimal::from(recordable) * base / hours_dec).round_dp(4)
        } else {
            Decimal::ZERO
        };
        let ltir = if total_hours_worked > 0 {
            (Decimal::from(lost_time) * base / hours_dec).round_dp(4)
        } else {
            Decimal::ZERO
        };
        let dart_rate = if total_hours_worked > 0 {
            (Decimal::from(days_away) * base / hours_dec).round_dp(4)
        } else {
            Decimal::ZERO
        };

        SafetyMetric {
            id: format!("SM-{:06}", self.counter),
            entity_id: entity_id.to_string(),
            period,
            total_hours_worked,
            recordable_incidents: recordable,
            lost_time_incidents: lost_time,
            days_away,
            near_misses,
            fatalities,
            trir,
            ltir,
            dart_rate,
        }
    }
}

/// Generates [`GovernanceMetric`] records.
pub struct GovernanceGenerator {
    rng: ChaCha8Rng,
    counter: u64,
    board_size: u32,
    independence_target: f64,
}

impl GovernanceGenerator {
    /// Create a new governance generator.
    pub fn new(seed: u64, board_size: u32, independence_target: f64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            counter: 0,
            board_size: board_size.max(3),
            independence_target,
        }
    }

    /// Generate a governance metric for a period.
    pub fn generate(&mut self, entity_id: &str, period: NaiveDate) -> GovernanceMetric {
        self.counter += 1;

        // Independent directors: aim near target with some noise
        let ind_frac: f64 = self.rng.gen_range(
            (self.independence_target - 0.10).max(0.0)..(self.independence_target + 0.10).min(1.0),
        );
        let independent = (self.board_size as f64 * ind_frac).round() as u32;
        let independent = independent.min(self.board_size);

        // Female directors: 20-40% range
        let fem_frac: f64 = self.rng.gen_range(0.20..0.40);
        let female = (self.board_size as f64 * fem_frac).round() as u32;
        let female = female.min(self.board_size);

        let independence_ratio = if self.board_size > 0 {
            (Decimal::from(independent) / Decimal::from(self.board_size)).round_dp(4)
        } else {
            Decimal::ZERO
        };
        let gender_ratio = if self.board_size > 0 {
            (Decimal::from(female) / Decimal::from(self.board_size)).round_dp(4)
        } else {
            Decimal::ZERO
        };

        let ethics_pct: f64 = self.rng.gen_range(0.85..0.99);
        let whistleblower: u32 = self.rng.gen_range(0..5);
        let anti_corruption: u32 = if self.rng.gen::<f64>() < 0.10 { 1 } else { 0 };

        GovernanceMetric {
            id: format!("GV-{:06}", self.counter),
            entity_id: entity_id.to_string(),
            period,
            board_size: self.board_size,
            independent_directors: independent,
            female_directors: female,
            board_independence_ratio: independence_ratio,
            board_gender_diversity_ratio: gender_ratio,
            ethics_training_completion_pct: (ethics_pct * 100.0).round() / 100.0,
            whistleblower_reports: whistleblower,
            anti_corruption_violations: anti_corruption,
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
    fn test_diversity_percentages_sum_to_one() {
        let config = SocialConfig::default();
        let mut gen = WorkforceGenerator::new(config, 42);
        let metrics = gen.generate_diversity("C001", 1000, d("2025-01-01"));

        assert!(!metrics.is_empty());

        // Group by (dimension, level) and check percentages sum ≈ 1.0
        let mut groups: std::collections::HashMap<
            (String, String),
            Vec<&WorkforceDiversityMetric>,
        > = std::collections::HashMap::new();
        for m in &metrics {
            let key = (format!("{:?}", m.dimension), format!("{:?}", m.level));
            groups.entry(key).or_default().push(m);
        }

        for (key, group) in &groups {
            let total_hc: u32 = group.iter().map(|m| m.headcount).sum();
            let expected = group[0].total_headcount;
            // Allow rounding tolerance of ±1
            assert!(
                total_hc.abs_diff(expected) <= 1,
                "Group {:?}: headcount sum {} != expected {}",
                key,
                total_hc,
                expected
            );
        }
    }

    #[test]
    fn test_pay_equity_ratios() {
        let config = SocialConfig::default();
        let mut gen = WorkforceGenerator::new(config, 42);
        let metrics = gen.generate_pay_equity("C001", d("2025-01-01"));

        assert_eq!(metrics.len(), 4, "Should have 4 comparison pairs");
        for m in &metrics {
            assert!(m.pay_gap_ratio > dec!(0.80) && m.pay_gap_ratio < dec!(1.10));
            assert!(m.reference_median_salary > Decimal::ZERO);
            assert!(m.comparison_median_salary > Decimal::ZERO);
            assert!(m.sample_size > 0);
        }
    }

    #[test]
    fn test_safety_incidents() {
        let config = SocialConfig {
            safety: datasynth_config::schema::SafetySchemaConfig {
                enabled: true,
                target_trir: 2.5,
                incident_count: 30,
            },
            ..Default::default()
        };
        let mut gen = WorkforceGenerator::new(config, 42);
        let incidents = gen.generate_safety_incidents("C001", 3, d("2025-01-01"), d("2025-12-31"));

        assert_eq!(incidents.len(), 30);

        let recordable = incidents.iter().filter(|i| i.is_recordable).count();
        let near_miss = incidents
            .iter()
            .filter(|i| i.incident_type == IncidentType::NearMiss)
            .count();
        // Near misses are not recordable
        assert_eq!(recordable + near_miss, 30);
    }

    #[test]
    fn test_safety_metric_trir_computation() {
        let config = SocialConfig::default();
        let mut gen = WorkforceGenerator::new(config, 42);

        let incidents = vec![
            SafetyIncident {
                id: "SI-001".into(),
                entity_id: "C001".into(),
                facility_id: "FAC-001".into(),
                date: d("2025-03-15"),
                incident_type: IncidentType::Injury,
                days_away: 5,
                is_recordable: true,
                description: "Test".into(),
            },
            SafetyIncident {
                id: "SI-002".into(),
                entity_id: "C001".into(),
                facility_id: "FAC-001".into(),
                date: d("2025-06-20"),
                incident_type: IncidentType::NearMiss,
                days_away: 0,
                is_recordable: false,
                description: "Test".into(),
            },
        ];

        let metric = gen.compute_safety_metrics("C001", &incidents, 500_000, d("2025-01-01"));

        assert_eq!(metric.recordable_incidents, 1);
        assert_eq!(metric.near_misses, 1);
        assert_eq!(metric.lost_time_incidents, 1);
        assert_eq!(metric.days_away, 5);
        // TRIR = 1 × 200,000 / 500,000 = 0.4
        assert_eq!(metric.trir, dec!(0.4000));
        assert_eq!(metric.computed_trir(), dec!(0.4000));
    }

    #[test]
    fn test_governance_generation() {
        let mut gen = GovernanceGenerator::new(42, 11, 0.67);
        let metric = gen.generate("C001", d("2025-01-01"));

        assert_eq!(metric.board_size, 11);
        assert!(metric.independent_directors <= 11);
        assert!(metric.female_directors <= 11);
        assert!(metric.board_independence_ratio > Decimal::ZERO);
        assert!(metric.ethics_training_completion_pct >= 0.85);
    }

    #[test]
    fn test_disabled_diversity() {
        let mut config = SocialConfig::default();
        config.diversity.enabled = false;
        let mut gen = WorkforceGenerator::new(config, 42);
        let metrics = gen.generate_diversity("C001", 1000, d("2025-01-01"));
        assert!(metrics.is_empty());
    }
}
