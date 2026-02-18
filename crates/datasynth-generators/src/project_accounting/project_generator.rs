//! Project and WBS hierarchy generator.
//!
//! Creates [`Project`] records with [`WbsElement`] hierarchies based on
//! [`ProjectAccountingConfig`] settings, distributing project types according
//! to configured weights.

use chrono::NaiveDate;
use datasynth_config::schema::{ProjectAccountingConfig, WbsSchemaConfig};
use datasynth_core::models::{Project, ProjectPool, ProjectStatus, ProjectType, WbsElement};
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;

/// Generates [`Project`] records with WBS hierarchies.
pub struct ProjectGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
    config: ProjectAccountingConfig,
}

impl ProjectGenerator {
    /// Create a new project generator.
    pub fn new(config: ProjectAccountingConfig, seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::ProjectAccounting),
            config,
        }
    }

    /// Generate a pool of projects with WBS hierarchies.
    pub fn generate(
        &mut self,
        company_code: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> ProjectPool {
        let count = self.config.project_count as usize;
        let mut pool = ProjectPool::new();

        for i in 0..count {
            let project_type = self.pick_project_type();
            let project_id = format!("PRJ-{:04}", i + 1);
            let budget = self.generate_budget(project_type);

            let mut project = Project::new(
                &project_id,
                &self.project_name(project_type, i),
                project_type,
            )
            .with_budget(budget)
            .with_company(company_code);

            project.start_date = Some(start_date.to_string());
            project.end_date = Some(end_date.to_string());
            project.status = self.pick_status();
            project.description = self.project_description(project_type);
            project.responsible_cost_center = format!("{:04}", self.rng.gen_range(1000..9999u32));

            // Generate WBS hierarchy
            let wbs_elements = self.generate_wbs(&project_id, budget, &self.config.wbs.clone());
            for wbs in wbs_elements {
                project.add_wbs_element(wbs);
            }

            pool.add_project(project);
        }

        pool
    }

    /// Pick a project type based on distribution weights.
    fn pick_project_type(&mut self) -> ProjectType {
        let dist = &self.config.project_types;
        let total = dist.capital
            + dist.internal
            + dist.customer
            + dist.r_and_d
            + dist.maintenance
            + dist.technology;

        let roll: f64 = self.rng.gen::<f64>() * total;
        let mut cumulative = 0.0;

        let types = [
            (dist.capital, ProjectType::Capital),
            (dist.internal, ProjectType::Internal),
            (dist.customer, ProjectType::Customer),
            (dist.r_and_d, ProjectType::RandD),
            (dist.maintenance, ProjectType::Maintenance),
            (dist.technology, ProjectType::Technology),
        ];

        for (weight, pt) in &types {
            cumulative += weight;
            if roll < cumulative {
                return *pt;
            }
        }

        ProjectType::Internal
    }

    /// Pick a project status (most should be Active).
    fn pick_status(&mut self) -> ProjectStatus {
        let roll: f64 = self.rng.gen::<f64>();
        if roll < 0.05 {
            ProjectStatus::Planned
        } else if roll < 0.80 {
            ProjectStatus::Active
        } else if roll < 0.90 {
            ProjectStatus::Closing
        } else if roll < 0.95 {
            ProjectStatus::Completed
        } else if roll < 0.98 {
            ProjectStatus::OnHold
        } else {
            ProjectStatus::Cancelled
        }
    }

    /// Generate a realistic budget based on project type.
    fn generate_budget(&mut self, project_type: ProjectType) -> Decimal {
        let (lo, hi) = match project_type {
            ProjectType::Capital => (500_000.0, 10_000_000.0),
            ProjectType::Internal => (50_000.0, 500_000.0),
            ProjectType::Customer => (100_000.0, 5_000_000.0),
            ProjectType::RandD => (200_000.0, 3_000_000.0),
            ProjectType::Maintenance => (25_000.0, 300_000.0),
            ProjectType::Technology => (100_000.0, 2_000_000.0),
        };
        let amount = self.rng.gen_range(lo..hi);
        Decimal::from_f64_retain(amount)
            .unwrap_or(Decimal::from(500_000))
            .round_dp(2)
    }

    /// Generate WBS elements for a project.
    fn generate_wbs(
        &mut self,
        project_id: &str,
        total_budget: Decimal,
        wbs_config: &WbsSchemaConfig,
    ) -> Vec<WbsElement> {
        let mut elements = Vec::new();
        let top_count = self
            .rng
            .gen_range(wbs_config.min_elements_per_level..=wbs_config.max_elements_per_level);

        let mut remaining_budget = total_budget;

        for i in 0..top_count {
            let wbs_id = format!("{}.{:02}", project_id, i + 1);
            let phase_name = self.phase_name(i);

            // Distribute budget: give equal shares, last element gets remainder
            let budget = if i == top_count - 1 {
                remaining_budget
            } else {
                let share = (total_budget / Decimal::from(top_count)).round_dp(2);
                remaining_budget -= share;
                share
            };

            let element = WbsElement::new(&wbs_id, project_id, &phase_name).with_budget(budget);
            elements.push(element);

            // Generate sub-levels if depth > 1
            if wbs_config.max_depth > 1 {
                let sub_count = self.rng.gen_range(
                    wbs_config.min_elements_per_level.min(3)
                        ..=wbs_config.max_elements_per_level.min(4),
                );
                let mut sub_remaining = budget;

                for j in 0..sub_count {
                    let sub_wbs_id = format!("{}.{:03}", wbs_id, j + 1);
                    let sub_name = format!("{} - Task {}", phase_name, j + 1);

                    let sub_budget = if j == sub_count - 1 {
                        sub_remaining
                    } else {
                        let share = (budget / Decimal::from(sub_count)).round_dp(2);
                        sub_remaining -= share;
                        share
                    };

                    let sub_element = WbsElement::new(&sub_wbs_id, project_id, &sub_name)
                        .with_parent(&wbs_id, 2)
                        .with_budget(sub_budget);
                    elements.push(sub_element);
                }
            }
        }

        elements
    }

    fn phase_name(&self, index: u32) -> String {
        let phases = [
            "Planning & Design",
            "Procurement",
            "Implementation",
            "Testing & Validation",
            "Deployment",
            "Closeout",
        ];
        phases
            .get(index as usize)
            .unwrap_or(&"Additional Work")
            .to_string()
    }

    fn project_name(&self, project_type: ProjectType, index: usize) -> String {
        let names: &[&str] = match project_type {
            ProjectType::Capital => &[
                "Data Center Expansion",
                "Manufacturing Line Upgrade",
                "Office Renovation",
                "Fleet Replacement",
                "Warehouse Automation",
                "Plant Equipment Overhaul",
                "New Facility Construction",
            ],
            ProjectType::Internal => &[
                "Process Improvement Initiative",
                "Employee Training Program",
                "Quality Certification",
                "Lean Six Sigma Rollout",
                "Culture Transformation",
                "Knowledge Management System",
            ],
            ProjectType::Customer => &[
                "Enterprise ERP Implementation",
                "Custom Software Build",
                "Infrastructure Deployment",
                "System Integration",
                "Data Migration Project",
                "Cloud Platform Build",
            ],
            ProjectType::RandD => &[
                "Next-Gen Product Research",
                "AI/ML Capability Study",
                "Materials Science Investigation",
                "Prototype Development",
                "Emerging Tech Evaluation",
                "Patent Portfolio Expansion",
            ],
            ProjectType::Maintenance => &[
                "Annual Equipment Maintenance",
                "HVAC System Overhaul",
                "Network Infrastructure Refresh",
                "Building Repairs",
                "Software Licensing Renewal",
                "Safety Compliance Update",
            ],
            ProjectType::Technology => &[
                "ERP System Implementation",
                "Cloud Migration",
                "Cybersecurity Enhancement",
                "Digital Transformation",
                "IT Infrastructure Upgrade",
                "Enterprise Data Platform",
            ],
        };
        let name = names[index % names.len()];
        if index < names.len() {
            name.to_string()
        } else {
            format!("{} Phase {}", name, index / names.len() + 1)
        }
    }

    fn project_description(&mut self, project_type: ProjectType) -> String {
        match project_type {
            ProjectType::Capital => {
                "Capital expenditure project for asset acquisition or improvement.".to_string()
            }
            ProjectType::Internal => "Internal project for operational improvement.".to_string(),
            ProjectType::Customer => {
                "Customer-facing project with contracted deliverables.".to_string()
            }
            ProjectType::RandD => "Research and development initiative.".to_string(),
            ProjectType::Maintenance => "Maintenance and sustainment activities.".to_string(),
            ProjectType::Technology => "Technology infrastructure or platform project.".to_string(),
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
    fn test_generate_projects_default_config() {
        let mut config = ProjectAccountingConfig::default();
        config.enabled = true;
        config.project_count = 10;

        let mut gen = ProjectGenerator::new(config, 42);
        let pool = gen.generate("TEST", d("2024-01-01"), d("2024-12-31"));

        assert_eq!(pool.projects.len(), 10);
        for project in &pool.projects {
            assert!(
                !project.wbs_elements.is_empty(),
                "Each project should have WBS elements"
            );
            assert!(project.budget > Decimal::ZERO, "Budget should be positive");
            assert_eq!(project.company_code, "TEST");
        }
    }

    #[test]
    fn test_project_type_distribution() {
        let mut config = ProjectAccountingConfig::default();
        config.enabled = true;
        config.project_count = 100;

        let mut gen = ProjectGenerator::new(config, 42);
        let pool = gen.generate("TEST", d("2024-01-01"), d("2024-12-31"));

        let customer_count = pool
            .projects
            .iter()
            .filter(|p| p.project_type == ProjectType::Customer)
            .count();

        // With 0.30 weight for customer, expect roughly 30 out of 100
        assert!(
            customer_count >= 15 && customer_count <= 50,
            "Expected ~30 customer projects, got {}",
            customer_count
        );
    }

    #[test]
    fn test_wbs_hierarchy_depth() {
        let mut config = ProjectAccountingConfig::default();
        config.enabled = true;
        config.project_count = 5;
        config.wbs.max_depth = 2;

        let mut gen = ProjectGenerator::new(config, 42);
        let pool = gen.generate("TEST", d("2024-01-01"), d("2024-12-31"));

        for project in &pool.projects {
            let has_children = project.wbs_elements.iter().any(|w| w.parent_wbs.is_some());
            assert!(
                has_children,
                "WBS should have child elements when max_depth > 1"
            );
        }
    }

    #[test]
    fn test_deterministic_generation() {
        let config = ProjectAccountingConfig::default();
        let mut gen1 = ProjectGenerator::new(config.clone(), 42);
        let pool1 = gen1.generate("TEST", d("2024-01-01"), d("2024-12-31"));

        let mut gen2 = ProjectGenerator::new(config, 42);
        let pool2 = gen2.generate("TEST", d("2024-01-01"), d("2024-12-31"));

        assert_eq!(pool1.projects.len(), pool2.projects.len());
        for (p1, p2) in pool1.projects.iter().zip(pool2.projects.iter()) {
            assert_eq!(p1.project_id, p2.project_id);
            assert_eq!(p1.project_type, p2.project_type);
            assert_eq!(p1.budget, p2.budget);
            assert_eq!(p1.wbs_elements.len(), p2.wbs_elements.len());
        }
    }
}
