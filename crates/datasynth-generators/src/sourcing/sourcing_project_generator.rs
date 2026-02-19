//! Sourcing project generator.
//!
//! Creates sourcing projects triggered by spend analysis or contract expiry.

use chrono::NaiveDate;
use datasynth_config::schema::SourcingConfig;
use datasynth_core::models::sourcing::{
    SourcingProject, SourcingProjectStatus, SourcingProjectType,
};
use datasynth_core::utils::seeded_rng;
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;

/// Generates sourcing projects.
pub struct SourcingProjectGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
    config: SourcingConfig,
}

impl SourcingProjectGenerator {
    /// Create a new sourcing project generator.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::SourcingProject),
            config: SourcingConfig::default(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(seed: u64, config: SourcingConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::SourcingProject),
            config,
        }
    }

    /// Generate sourcing projects for a given period.
    ///
    /// # Arguments
    /// * `company_code` - Company code
    /// * `categories` - Available spend categories (id, name, annual_spend)
    /// * `owner_ids` - Available buyer/sourcing manager IDs
    /// * `period_start` - Period start date
    /// * `period_months` - Number of months
    pub fn generate(
        &mut self,
        company_code: &str,
        categories: &[(String, String, Decimal)],
        owner_ids: &[String],
        period_start: NaiveDate,
        period_months: u32,
    ) -> Vec<SourcingProject> {
        tracing::debug!(
            company_code,
            categories = categories.len(),
            period_months,
            "Generating sourcing projects"
        );
        let mut projects = Vec::new();
        let years = (period_months as f64 / 12.0).ceil() as u32;
        let target_count = self.config.projects_per_year * years;

        for _ in 0..target_count {
            if categories.is_empty() || owner_ids.is_empty() {
                break;
            }

            let (cat_id, cat_name, annual_spend) =
                &categories[self.rng.gen_range(0..categories.len())];
            let owner_id = &owner_ids[self.rng.gen_range(0..owner_ids.len())];

            let project_type = if self.rng.gen_bool(0.4) {
                SourcingProjectType::Renewal
            } else if self.rng.gen_bool(0.15) {
                SourcingProjectType::Consolidation
            } else {
                SourcingProjectType::NewSourcing
            };

            let days_offset = self.rng.gen_range(0..period_months * 30);
            let start_date = period_start + chrono::Duration::days(days_offset as i64);
            let duration_months = self.config.project_duration_months;
            let target_end_date =
                start_date + chrono::Duration::days((duration_months * 30) as i64);

            let project_id = self.uuid_factory.next().to_string();
            let target_savings = self.rng.gen_range(0.03..=0.15);

            projects.push(SourcingProject {
                project_id,
                project_name: format!("{} - {} Sourcing", cat_name, company_code),
                company_code: company_code.to_string(),
                project_type,
                status: SourcingProjectStatus::Completed,
                category_id: cat_id.clone(),
                estimated_annual_spend: *annual_spend,
                target_savings_pct: target_savings,
                owner_id: owner_id.clone(),
                start_date,
                target_end_date,
                actual_end_date: Some(
                    target_end_date + chrono::Duration::days(self.rng.gen_range(-10..=20) as i64),
                ),
                spend_analysis_id: None,
                rfx_ids: Vec::new(),
                contract_id: None,
                actual_savings_pct: Some(target_savings * self.rng.gen_range(0.6..=1.2)),
            });
        }

        projects
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn test_categories() -> Vec<(String, String, Decimal)> {
        vec![
            (
                "CAT-001".to_string(),
                "Office Supplies".to_string(),
                Decimal::from(500_000),
            ),
            (
                "CAT-002".to_string(),
                "IT Equipment".to_string(),
                Decimal::from(1_200_000),
            ),
        ]
    }

    fn test_owner_ids() -> Vec<String> {
        vec!["BUYER-001".to_string(), "BUYER-002".to_string()]
    }

    #[test]
    fn test_basic_generation() {
        let mut gen = SourcingProjectGenerator::new(42);
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let results = gen.generate("C001", &test_categories(), &test_owner_ids(), start, 12);

        // Default is 10 projects per year
        assert_eq!(results.len(), 10);
        for project in &results {
            assert_eq!(project.company_code, "C001");
            assert!(!project.project_id.is_empty());
            assert!(!project.project_name.is_empty());
            assert!(!project.category_id.is_empty());
            assert!(!project.owner_id.is_empty());
            assert!(project.start_date >= start);
            assert!(project.target_end_date > project.start_date);
            assert!(project.estimated_annual_spend > Decimal::ZERO);
        }
    }

    #[test]
    fn test_deterministic() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let cats = test_categories();
        let owners = test_owner_ids();

        let mut gen1 = SourcingProjectGenerator::new(42);
        let mut gen2 = SourcingProjectGenerator::new(42);

        let r1 = gen1.generate("C001", &cats, &owners, start, 12);
        let r2 = gen2.generate("C001", &cats, &owners, start, 12);

        assert_eq!(r1.len(), r2.len());
        for (a, b) in r1.iter().zip(r2.iter()) {
            assert_eq!(a.project_id, b.project_id);
            assert_eq!(a.category_id, b.category_id);
            assert_eq!(a.start_date, b.start_date);
            assert_eq!(a.target_savings_pct, b.target_savings_pct);
        }
    }

    #[test]
    fn test_field_constraints() {
        let mut gen = SourcingProjectGenerator::new(99);
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let results = gen.generate("C001", &test_categories(), &test_owner_ids(), start, 12);

        for project in &results {
            // Target savings should be in 3-15% range
            assert!(project.target_savings_pct >= 0.03 && project.target_savings_pct <= 0.15);

            // Actual savings should be present (status is Completed)
            assert!(project.actual_savings_pct.is_some());

            // Actual end date should be present for completed projects
            assert!(project.actual_end_date.is_some());

            // Project type should be one of the valid variants
            matches!(
                project.project_type,
                SourcingProjectType::NewSourcing
                    | SourcingProjectType::Renewal
                    | SourcingProjectType::Consolidation
            );
        }
    }
}
