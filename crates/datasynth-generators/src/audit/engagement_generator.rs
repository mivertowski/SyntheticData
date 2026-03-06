//! Audit engagement generator.
//!
//! Generates complete audit engagements including risk assessments,
//! workpapers, evidence, findings, and professional judgments.

use chrono::{Duration, NaiveDate};
use datasynth_core::utils::seeded_rng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;

use datasynth_core::models::audit::{
    AuditEngagement, EngagementPhase, EngagementStatus, EngagementType, RiskLevel,
};

/// Configuration for audit engagement generation.
#[derive(Debug, Clone)]
pub struct AuditEngagementConfig {
    /// Default engagement type
    pub default_engagement_type: EngagementType,
    /// Materiality percentage range (min, max)
    pub materiality_percentage_range: (f64, f64),
    /// Performance materiality factor (e.g., 0.50-0.75)
    pub performance_materiality_factor_range: (f64, f64),
    /// Clearly trivial factor (e.g., 0.03-0.05)
    pub clearly_trivial_factor_range: (f64, f64),
    /// Planning phase duration in days (min, max)
    pub planning_duration_range: (u32, u32),
    /// Fieldwork phase duration in days (min, max)
    pub fieldwork_duration_range: (u32, u32),
    /// Completion phase duration in days (min, max)
    pub completion_duration_range: (u32, u32),
    /// Team size range (min, max)
    pub team_size_range: (u32, u32),
    /// Probability of high fraud risk
    pub high_fraud_risk_probability: f64,
    /// Probability of significant risks
    pub significant_risk_probability: f64,
}

impl Default for AuditEngagementConfig {
    fn default() -> Self {
        Self {
            default_engagement_type: EngagementType::AnnualAudit,
            materiality_percentage_range: (0.003, 0.010), // 0.3% to 1% of base
            performance_materiality_factor_range: (0.50, 0.75),
            clearly_trivial_factor_range: (0.03, 0.05),
            planning_duration_range: (14, 30),
            fieldwork_duration_range: (30, 60),
            completion_duration_range: (14, 21),
            team_size_range: (3, 8),
            high_fraud_risk_probability: 0.15,
            significant_risk_probability: 0.30,
        }
    }
}

/// Generator for audit engagements and related data.
pub struct AuditEngagementGenerator {
    /// Random number generator
    rng: ChaCha8Rng,
    /// Configuration
    config: AuditEngagementConfig,
    /// Counter for engagement references
    engagement_counter: u32,
}

impl AuditEngagementGenerator {
    /// Create a new generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config: AuditEngagementConfig::default(),
            engagement_counter: 0,
        }
    }

    /// Create a new generator with custom configuration.
    pub fn with_config(seed: u64, config: AuditEngagementConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config,
            engagement_counter: 0,
        }
    }

    /// Generate an audit engagement for a company.
    pub fn generate_engagement(
        &mut self,
        client_entity_id: &str,
        client_name: &str,
        fiscal_year: u16,
        period_end_date: NaiveDate,
        total_revenue: Decimal,
        engagement_type: Option<EngagementType>,
    ) -> AuditEngagement {
        self.engagement_counter += 1;

        let eng_type = engagement_type.unwrap_or(self.config.default_engagement_type);

        // Calculate materiality
        let materiality_pct = self.rng.random_range(
            self.config.materiality_percentage_range.0..=self.config.materiality_percentage_range.1,
        );
        let materiality = total_revenue * Decimal::try_from(materiality_pct).unwrap_or_default();

        let perf_mat_factor = self.rng.random_range(
            self.config.performance_materiality_factor_range.0
                ..=self.config.performance_materiality_factor_range.1,
        );

        let trivial_factor = self.rng.random_range(
            self.config.clearly_trivial_factor_range.0..=self.config.clearly_trivial_factor_range.1,
        );

        // Generate timeline
        let timeline = self.generate_timeline(period_end_date);

        // Generate team
        let (partner_id, partner_name, manager_id, manager_name, team_members) =
            self.generate_team();

        // Determine risk levels
        let (overall_risk, fraud_risk, significant_count) = self.generate_risk_profile();

        let mut engagement = AuditEngagement::new(
            client_entity_id,
            client_name,
            eng_type,
            fiscal_year,
            period_end_date,
        );

        engagement.engagement_ref = format!("AUD-{}-{:04}", fiscal_year, self.engagement_counter);

        engagement = engagement.with_materiality(
            materiality,
            perf_mat_factor,
            trivial_factor,
            "Total Revenue",
            materiality_pct,
        );

        engagement = engagement.with_timeline(
            timeline.planning_start,
            timeline.planning_end,
            timeline.fieldwork_start,
            timeline.fieldwork_end,
            timeline.completion_start,
            timeline.report_date,
        );

        engagement = engagement.with_team(
            &partner_id,
            &partner_name,
            &manager_id,
            &manager_name,
            team_members,
        );

        engagement.overall_audit_risk = overall_risk;
        engagement.fraud_risk_level = fraud_risk;
        engagement.significant_risk_count = significant_count;

        // Set initial status
        engagement.status = EngagementStatus::Planning;
        engagement.current_phase = EngagementPhase::Planning;

        engagement
    }

    /// Generate an engagement timeline based on period end date.
    fn generate_timeline(&mut self, period_end_date: NaiveDate) -> EngagementTimeline {
        // Planning typically starts 3-4 months before year end
        let planning_duration = self.rng.random_range(
            self.config.planning_duration_range.0..=self.config.planning_duration_range.1,
        );
        let fieldwork_duration = self.rng.random_range(
            self.config.fieldwork_duration_range.0..=self.config.fieldwork_duration_range.1,
        );
        let completion_duration = self.rng.random_range(
            self.config.completion_duration_range.0..=self.config.completion_duration_range.1,
        );

        // Planning starts ~90 days before period end
        let planning_start = period_end_date - Duration::days(90);
        let planning_end = planning_start + Duration::days(planning_duration as i64);

        // Fieldwork starts after period end
        let fieldwork_start = period_end_date + Duration::days(5);
        let fieldwork_end = fieldwork_start + Duration::days(fieldwork_duration as i64);

        // Completion follows fieldwork
        let completion_start = fieldwork_end + Duration::days(1);
        let report_date = completion_start + Duration::days(completion_duration as i64);

        EngagementTimeline {
            planning_start,
            planning_end,
            fieldwork_start,
            fieldwork_end,
            completion_start,
            report_date,
        }
    }

    /// Generate engagement team.
    fn generate_team(&mut self) -> (String, String, String, String, Vec<String>) {
        let team_size = self
            .rng
            .random_range(self.config.team_size_range.0..=self.config.team_size_range.1)
            as usize;

        // Partner
        let partner_num = self.rng.random_range(1..=20);
        let partner_id = format!("PARTNER{partner_num:03}");
        let partner_name = self.generate_auditor_name(partner_num);

        // Manager
        let manager_num = self.rng.random_range(1..=50);
        let manager_id = format!("MANAGER{manager_num:03}");
        let manager_name = self.generate_auditor_name(manager_num + 100);

        // Team members (seniors and staff)
        let mut team_members = Vec::with_capacity(team_size);
        for i in 0..team_size {
            let member_num = self.rng.random_range(1..=200);
            if i < team_size / 2 {
                team_members.push(format!("SENIOR{member_num:03}"));
            } else {
                team_members.push(format!("STAFF{member_num:03}"));
            }
        }

        (
            partner_id,
            partner_name,
            manager_id,
            manager_name,
            team_members,
        )
    }

    /// Generate a plausible auditor name.
    fn generate_auditor_name(&mut self, seed: u32) -> String {
        let first_names = [
            "Michael",
            "Sarah",
            "David",
            "Jennifer",
            "Robert",
            "Emily",
            "James",
            "Amanda",
            "William",
            "Jessica",
            "John",
            "Ashley",
            "Daniel",
            "Nicole",
            "Christopher",
            "Michelle",
        ];
        let last_names = [
            "Smith", "Johnson", "Williams", "Brown", "Jones", "Davis", "Miller", "Wilson", "Moore",
            "Taylor", "Anderson", "Thomas", "Jackson", "White", "Harris", "Martin",
        ];

        let first_idx = (seed as usize) % first_names.len();
        let last_idx = ((seed as usize) / first_names.len()) % last_names.len();

        format!("{} {}", first_names[first_idx], last_names[last_idx])
    }

    /// Generate risk profile for the engagement.
    fn generate_risk_profile(&mut self) -> (RiskLevel, RiskLevel, u32) {
        // Determine fraud risk
        let fraud_risk = if self.rng.random::<f64>() < self.config.high_fraud_risk_probability {
            RiskLevel::High
        } else if self.rng.random::<f64>() < 0.40 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        // Determine significant risk count (typically 2-8)
        let significant_count =
            if self.rng.random::<f64>() < self.config.significant_risk_probability {
                self.rng.random_range(2..=8)
            } else {
                self.rng.random_range(0..=2)
            };

        // Overall risk is influenced by both
        let overall_risk = if fraud_risk == RiskLevel::High || significant_count > 5 {
            RiskLevel::High
        } else if fraud_risk == RiskLevel::Medium || significant_count > 2 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        (overall_risk, fraud_risk, significant_count)
    }

    /// Generate multiple engagements for a batch of companies.
    pub fn generate_engagements_batch(
        &mut self,
        companies: &[CompanyInfo],
        fiscal_year: u16,
    ) -> Vec<AuditEngagement> {
        companies
            .iter()
            .map(|company| {
                self.generate_engagement(
                    &company.entity_id,
                    &company.name,
                    fiscal_year,
                    company.period_end_date,
                    company.total_revenue,
                    company.engagement_type,
                )
            })
            .collect()
    }

    /// Advance an engagement to the next phase based on current date.
    pub fn advance_engagement_phase(
        &mut self,
        engagement: &mut AuditEngagement,
        current_date: NaiveDate,
    ) {
        // Determine what phase we should be in based on dates
        let new_phase = if current_date < engagement.planning_end {
            EngagementPhase::Planning
        } else if current_date < engagement.fieldwork_start {
            EngagementPhase::RiskAssessment
        } else if current_date < engagement.fieldwork_end {
            // During fieldwork, alternate between control testing and substantive
            let days_into_fieldwork = (current_date - engagement.fieldwork_start).num_days();
            let fieldwork_duration =
                (engagement.fieldwork_end - engagement.fieldwork_start).num_days();

            if days_into_fieldwork < fieldwork_duration / 3 {
                EngagementPhase::ControlTesting
            } else {
                EngagementPhase::SubstantiveTesting
            }
        } else if current_date < engagement.report_date {
            EngagementPhase::Completion
        } else {
            EngagementPhase::Reporting
        };

        if new_phase != engagement.current_phase {
            engagement.current_phase = new_phase;

            // Update status based on phase
            engagement.status = match new_phase {
                EngagementPhase::Planning | EngagementPhase::RiskAssessment => {
                    EngagementStatus::Planning
                }
                EngagementPhase::ControlTesting | EngagementPhase::SubstantiveTesting => {
                    EngagementStatus::InProgress
                }
                EngagementPhase::Completion => EngagementStatus::UnderReview,
                EngagementPhase::Reporting => EngagementStatus::PendingSignOff,
            };
        }
    }
}

/// Timeline for an engagement.
#[derive(Debug, Clone)]
struct EngagementTimeline {
    planning_start: NaiveDate,
    planning_end: NaiveDate,
    fieldwork_start: NaiveDate,
    fieldwork_end: NaiveDate,
    completion_start: NaiveDate,
    report_date: NaiveDate,
}

/// Information about a company for engagement generation.
#[derive(Debug, Clone)]
pub struct CompanyInfo {
    /// Entity ID
    pub entity_id: String,
    /// Company name
    pub name: String,
    /// Period end date
    pub period_end_date: NaiveDate,
    /// Total revenue for materiality calculation
    pub total_revenue: Decimal,
    /// Optional specific engagement type
    pub engagement_type: Option<EngagementType>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_engagement_generation() {
        let mut generator = AuditEngagementGenerator::new(42);
        let period_end = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
        let revenue = Decimal::new(100_000_000, 0); // $100M

        let engagement = generator.generate_engagement(
            "ENTITY001",
            "Test Company Inc.",
            2025,
            period_end,
            revenue,
            None,
        );

        assert_eq!(engagement.fiscal_year, 2025);
        assert_eq!(engagement.engagement_type, EngagementType::AnnualAudit);
        assert!(engagement.materiality > Decimal::ZERO);
        assert!(engagement.performance_materiality <= engagement.materiality);
        assert!(!engagement.engagement_partner_id.is_empty());
        assert!(!engagement.team_member_ids.is_empty());
    }

    #[test]
    fn test_batch_generation() {
        let mut generator = AuditEngagementGenerator::new(42);

        let companies = vec![
            CompanyInfo {
                entity_id: "ENTITY001".into(),
                name: "Company A".into(),
                period_end_date: NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
                total_revenue: Decimal::new(50_000_000, 0),
                engagement_type: None,
            },
            CompanyInfo {
                entity_id: "ENTITY002".into(),
                name: "Company B".into(),
                period_end_date: NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
                total_revenue: Decimal::new(75_000_000, 0),
                engagement_type: Some(EngagementType::IntegratedAudit),
            },
        ];

        let engagements = generator.generate_engagements_batch(&companies, 2025);

        assert_eq!(engagements.len(), 2);
        assert_eq!(
            engagements[1].engagement_type,
            EngagementType::IntegratedAudit
        );
    }

    #[test]
    fn test_phase_advancement() {
        let mut generator = AuditEngagementGenerator::new(42);
        let period_end = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();

        let mut engagement = generator.generate_engagement(
            "ENTITY001",
            "Test Company",
            2025,
            period_end,
            Decimal::new(100_000_000, 0),
            None,
        );

        // During planning phase (planning starts ~90 days before period_end)
        // Use a date close to planning_start to ensure we're in planning
        generator.advance_engagement_phase(&mut engagement, period_end - Duration::days(85));
        assert_eq!(engagement.current_phase, EngagementPhase::Planning);

        // Between planning and fieldwork should be risk assessment
        generator.advance_engagement_phase(&mut engagement, period_end - Duration::days(30));
        assert_eq!(engagement.current_phase, EngagementPhase::RiskAssessment);

        // After fieldwork start
        generator.advance_engagement_phase(&mut engagement, period_end + Duration::days(10));
        assert!(matches!(
            engagement.current_phase,
            EngagementPhase::ControlTesting | EngagementPhase::SubstantiveTesting
        ));
    }

    #[test]
    fn test_materiality_calculation() {
        let mut generator = AuditEngagementGenerator::new(42);
        let revenue = Decimal::new(100_000_000, 0); // $100M

        let engagement = generator.generate_engagement(
            "ENTITY001",
            "Test Company",
            2025,
            NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
            revenue,
            None,
        );

        // Materiality should be between 0.3% and 1% of revenue
        let min_materiality = revenue * Decimal::try_from(0.003).unwrap();
        let max_materiality = revenue * Decimal::try_from(0.010).unwrap();

        assert!(engagement.materiality >= min_materiality);
        assert!(engagement.materiality <= max_materiality);

        // Performance materiality should be 50-75% of materiality
        assert!(
            engagement.performance_materiality
                >= engagement.materiality * Decimal::try_from(0.50).unwrap()
        );
        assert!(
            engagement.performance_materiality
                <= engagement.materiality * Decimal::try_from(0.75).unwrap()
        );
    }
}
