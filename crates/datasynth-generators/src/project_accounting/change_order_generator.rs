//! Change order and milestone generator.
//!
//! Probabilistically injects change orders with cost/schedule/revenue impacts
//! and generates milestones with payment and completion tracking.
use chrono::NaiveDate;
use datasynth_config::schema::{ChangeOrderSchemaConfig, MilestoneSchemaConfig};
use datasynth_core::models::{
    ChangeOrder, ChangeOrderStatus, ChangeReason, MilestoneStatus, Project, ProjectMilestone,
};
use datasynth_core::utils::seeded_rng;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Generates [`ChangeOrder`] records for projects.
pub struct ChangeOrderGenerator {
    rng: ChaCha8Rng,
    config: ChangeOrderSchemaConfig,
    counter: u64,
}

impl ChangeOrderGenerator {
    /// Create a new change order generator.
    pub fn new(config: ChangeOrderSchemaConfig, seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config,
            counter: 0,
        }
    }

    /// Generate change orders for a set of projects.
    pub fn generate(
        &mut self,
        projects: &[Project],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Vec<ChangeOrder> {
        let mut change_orders = Vec::new();
        let period_days = (end_date - start_date).num_days().max(1);

        for project in projects {
            if !project.allows_postings() {
                continue;
            }

            // Check if this project gets change orders
            if self.rng.random::<f64>() >= self.config.probability {
                continue;
            }

            let co_count = self.rng.random_range(1..=self.config.max_per_project);

            for number in 1..=co_count {
                self.counter += 1;

                // Submit at a random point during the project
                let day_offset = self.rng.random_range(1..period_days);
                let submitted_date = start_date + chrono::Duration::days(day_offset);

                let reason = self.pick_reason();
                let description = self.description_for(reason);

                // Cost impact: 2-15% of project budget
                let impact_pct: f64 = self.rng.random_range(0.02..0.15);
                let cost_impact = (project.budget
                    * Decimal::from_f64_retain(impact_pct).unwrap_or(dec!(0.05)))
                .round_dp(2);

                // Estimated cost impact is usually close to contract impact
                let est_factor: f64 = self.rng.random_range(0.80..1.20);
                let estimated_cost_impact = (cost_impact
                    * Decimal::from_f64_retain(est_factor).unwrap_or(dec!(1)))
                .round_dp(2);

                // Schedule impact: 0-60 days
                let schedule_days = self.rng.random_range(0..60i32);

                let mut co = ChangeOrder::new(
                    format!("CO-{:06}", self.counter),
                    &project.project_id,
                    number,
                    submitted_date,
                    reason,
                    description,
                )
                .with_cost_impact(cost_impact, estimated_cost_impact)
                .with_schedule_impact(schedule_days);

                // Approve based on config rate
                if self.rng.random::<f64>() < self.config.approval_rate {
                    let approval_delay = self.rng.random_range(3..30);
                    let approved_date = submitted_date + chrono::Duration::days(approval_delay);
                    if approved_date <= end_date {
                        co = co.approve(approved_date);
                    }
                } else if self.rng.random::<f64>() < 0.7 {
                    co.status = ChangeOrderStatus::Rejected;
                } else {
                    co.status = ChangeOrderStatus::UnderReview;
                }

                change_orders.push(co);
            }
        }

        change_orders
    }

    fn pick_reason(&mut self) -> ChangeReason {
        let roll: f64 = self.rng.random::<f64>();
        if roll < 0.30 {
            ChangeReason::ScopeChange
        } else if roll < 0.50 {
            ChangeReason::UnforeseenConditions
        } else if roll < 0.65 {
            ChangeReason::DesignError
        } else if roll < 0.80 {
            ChangeReason::RegulatoryChange
        } else if roll < 0.92 {
            ChangeReason::ValueEngineering
        } else {
            ChangeReason::ScheduleAcceleration
        }
    }

    fn description_for(&self, reason: ChangeReason) -> String {
        match reason {
            ChangeReason::ScopeChange => {
                "Client-requested modification to deliverable scope".to_string()
            }
            ChangeReason::UnforeseenConditions => {
                "Unforeseen site conditions requiring additional work".to_string()
            }
            ChangeReason::DesignError => {
                "Design specification correction and remediation".to_string()
            }
            ChangeReason::RegulatoryChange => {
                "Regulatory compliance update requirement".to_string()
            }
            ChangeReason::ValueEngineering => {
                "Value engineering cost reduction opportunity".to_string()
            }
            ChangeReason::ScheduleAcceleration => {
                "Schedule acceleration to meet revised deadline".to_string()
            }
        }
    }
}

/// Generates [`ProjectMilestone`] records for projects.
pub struct MilestoneGenerator {
    rng: ChaCha8Rng,
    config: MilestoneSchemaConfig,
    counter: u64,
}

impl MilestoneGenerator {
    /// Create a new milestone generator.
    pub fn new(config: MilestoneSchemaConfig, seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config,
            counter: 0,
        }
    }

    /// Generate milestones for a set of projects.
    ///
    /// Distributes milestones evenly across the project duration,
    /// with payment milestones based on the configured rate.
    pub fn generate(
        &mut self,
        projects: &[Project],
        start_date: NaiveDate,
        end_date: NaiveDate,
        reference_date: NaiveDate,
    ) -> Vec<ProjectMilestone> {
        let mut milestones = Vec::new();

        for project in projects {
            let ms_count = self.config.avg_per_project.max(1);
            let period_days = (end_date - start_date).num_days().max(1);
            let interval = period_days / ms_count as i64;

            let milestone_names = [
                "Requirements Complete",
                "Design Approved",
                "Foundation Complete",
                "Structural Milestone",
                "Integration Testing",
                "User Acceptance",
                "Go-Live",
                "Project Closeout",
            ];

            for seq in 0..ms_count {
                self.counter += 1;

                let planned_date = start_date + chrono::Duration::days(interval * (seq as i64 + 1));
                let name = milestone_names
                    .get(seq as usize)
                    .unwrap_or(&"Additional Milestone");

                let mut ms = ProjectMilestone::new(
                    format!("MS-{:06}", self.counter),
                    &project.project_id,
                    *name,
                    planned_date,
                    seq + 1,
                );

                // Assign to first WBS element if available
                if let Some(wbs) = project.wbs_elements.first() {
                    ms = ms.with_wbs(&wbs.wbs_id);
                }

                // Payment milestone?
                if self.rng.random::<f64>() < self.config.payment_milestone_rate {
                    let payment_share = dec!(1) / Decimal::from(ms_count.max(1));
                    let payment = (project.budget * payment_share).round_dp(2);
                    ms = ms.with_payment(payment);
                }

                // EVM weight
                let weight = dec!(1) / Decimal::from(ms_count.max(1));
                ms = ms.with_weight(weight.round_dp(4));

                // Determine status based on reference date
                if planned_date <= reference_date {
                    if self.rng.random::<f64>() < 0.85 {
                        // Completed (possibly late)
                        ms.status = MilestoneStatus::Completed;
                        let variance_days: i64 = self.rng.random_range(-5..15);
                        ms.actual_date = Some(planned_date + chrono::Duration::days(variance_days));
                    } else {
                        ms.status = MilestoneStatus::Overdue;
                    }
                } else if planned_date <= reference_date + chrono::Duration::days(30) {
                    ms.status = MilestoneStatus::InProgress;
                }
                // Otherwise stays Pending

                milestones.push(ms);
            }
        }

        milestones
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::ProjectType;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    fn test_projects() -> Vec<Project> {
        (0..5)
            .map(|i| {
                Project::new(
                    &format!("PRJ-{:03}", i + 1),
                    &format!("Project {}", i + 1),
                    ProjectType::Customer,
                )
                .with_budget(dec!(1000000))
                .with_company("TEST")
            })
            .collect()
    }

    #[test]
    fn test_change_order_generation() {
        let projects = test_projects();
        let config = ChangeOrderSchemaConfig {
            enabled: true,
            probability: 1.0, // Force all projects to get change orders
            max_per_project: 2,
            approval_rate: 0.75,
        };

        let mut gen = ChangeOrderGenerator::new(config, 42);
        let cos = gen.generate(&projects, d("2024-01-01"), d("2024-12-31"));

        assert!(!cos.is_empty(), "Should generate change orders");

        for co in &cos {
            assert!(
                projects.iter().any(|p| p.project_id == co.project_id),
                "Change order should reference valid project"
            );
            assert!(
                co.cost_impact > Decimal::ZERO,
                "Cost impact should be positive"
            );
            assert!(
                co.schedule_impact_days >= 0,
                "Schedule impact should be non-negative"
            );
        }
    }

    #[test]
    fn test_change_order_approval_rate() {
        let projects = test_projects();
        let config = ChangeOrderSchemaConfig {
            enabled: true,
            probability: 1.0,
            max_per_project: 3,
            approval_rate: 1.0, // All approved (subject to date constraints)
        };

        let mut gen = ChangeOrderGenerator::new(config, 42);
        let cos = gen.generate(&projects, d("2024-01-01"), d("2024-12-31"));

        let approved = cos.iter().filter(|co| co.is_approved()).count();
        // Most should be approved; some submitted late may miss the window
        let approval_pct = approved as f64 / cos.len() as f64;
        assert!(
            approval_pct >= 0.70,
            "At 100% approval rate, most should be approved: {}/{} = {:.0}%",
            approved,
            cos.len(),
            approval_pct * 100.0
        );
    }

    #[test]
    fn test_change_order_zero_probability() {
        let projects = test_projects();
        let config = ChangeOrderSchemaConfig {
            enabled: true,
            probability: 0.0,
            max_per_project: 3,
            approval_rate: 0.75,
        };

        let mut gen = ChangeOrderGenerator::new(config, 42);
        let cos = gen.generate(&projects, d("2024-01-01"), d("2024-12-31"));

        assert!(
            cos.is_empty(),
            "Zero probability should produce no change orders"
        );
    }

    #[test]
    fn test_milestone_generation() {
        let projects = test_projects();
        let config = MilestoneSchemaConfig {
            enabled: true,
            avg_per_project: 4,
            payment_milestone_rate: 0.50,
        };

        let mut gen = MilestoneGenerator::new(config, 42);
        let milestones = gen.generate(&projects, d("2024-01-01"), d("2024-12-31"), d("2024-06-30"));

        assert_eq!(milestones.len(), 20, "5 projects * 4 milestones each");

        // Check that milestones are sequenced
        for project in &projects {
            let project_ms: Vec<_> = milestones
                .iter()
                .filter(|m| m.project_id == project.project_id)
                .collect();
            assert_eq!(project_ms.len(), 4);

            for (i, ms) in project_ms.iter().enumerate() {
                assert_eq!(ms.sequence, (i + 1) as u32);
            }
        }
    }

    #[test]
    fn test_milestone_status_progression() {
        let projects = vec![Project::new("PRJ-001", "Test", ProjectType::Customer)
            .with_budget(dec!(500000))
            .with_company("TEST")];
        let config = MilestoneSchemaConfig {
            enabled: true,
            avg_per_project: 4,
            payment_milestone_rate: 0.50,
        };

        let mut gen = MilestoneGenerator::new(config, 42);
        let milestones = gen.generate(
            &projects,
            d("2024-01-01"),
            d("2024-12-31"),
            d("2024-06-30"), // Reference: mid-year
        );

        // Early milestones should be completed or overdue
        let early_ms: Vec<_> = milestones
            .iter()
            .filter(|m| m.planned_date <= d("2024-06-30"))
            .collect();

        for ms in &early_ms {
            assert!(
                ms.status == MilestoneStatus::Completed || ms.status == MilestoneStatus::Overdue,
                "Past milestones should be completed or overdue, got {:?}",
                ms.status
            );
        }
    }

    #[test]
    fn test_milestone_payment_amounts() {
        let projects = vec![Project::new("PRJ-001", "Test", ProjectType::Customer)
            .with_budget(dec!(1000000))
            .with_company("TEST")];
        let config = MilestoneSchemaConfig {
            enabled: true,
            avg_per_project: 4,
            payment_milestone_rate: 1.0, // All are payment milestones
        };

        let mut gen = MilestoneGenerator::new(config, 42);
        let milestones = gen.generate(&projects, d("2024-01-01"), d("2024-12-31"), d("2024-01-01"));

        let total_payments: Decimal = milestones.iter().map(|m| m.payment_amount).sum();
        assert_eq!(
            total_payments,
            dec!(1000000),
            "Total payments should equal budget"
        );
    }

    #[test]
    fn test_deterministic_change_orders() {
        let projects = test_projects();
        let config = ChangeOrderSchemaConfig::default();

        let mut gen1 = ChangeOrderGenerator::new(config.clone(), 42);
        let cos1 = gen1.generate(&projects, d("2024-01-01"), d("2024-12-31"));

        let mut gen2 = ChangeOrderGenerator::new(config, 42);
        let cos2 = gen2.generate(&projects, d("2024-01-01"), d("2024-12-31"));

        assert_eq!(cos1.len(), cos2.len());
        for (a, b) in cos1.iter().zip(cos2.iter()) {
            assert_eq!(a.project_id, b.project_id);
            assert_eq!(a.cost_impact, b.cost_impact);
            assert_eq!(a.status, b.status);
        }
    }
}
