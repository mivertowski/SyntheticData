//! Engagement letter generator per ISA 210.
//!
//! Generates one engagement letter per audit engagement.  The scope is derived
//! from the number of entities involved: a single entity produces a
//! `StatutoryAudit` scope; multiple entities produce a `GroupAudit` scope.
//! Fees are calculated as a function of entity count and a complexity factor.

use chrono::{Duration, NaiveDate};
use datasynth_core::models::audit::engagement_letter::{
    EngagementLetter, EngagementScope, FeeArrangement,
};
use datasynth_core::utils::seeded_rng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use tracing::info;

/// Configuration for engagement letter generation.
#[derive(Debug, Clone)]
pub struct EngagementLetterGeneratorConfig {
    /// Base fee per entity (in the entity's currency)
    pub base_fee_per_entity: Decimal,
    /// Complexity factor range (min, max) multiplied by base fee
    pub complexity_factor_range: (f64, f64),
    /// Number of days after period-end when the report is due
    pub reporting_deadline_days: i64,
}

impl Default for EngagementLetterGeneratorConfig {
    fn default() -> Self {
        Self {
            base_fee_per_entity: Decimal::new(75_000, 0),
            complexity_factor_range: (0.8, 2.5),
            reporting_deadline_days: 90,
        }
    }
}

/// Generator for ISA 210 engagement letters.
pub struct EngagementLetterGenerator {
    rng: ChaCha8Rng,
    config: EngagementLetterGeneratorConfig,
}

impl EngagementLetterGenerator {
    /// Create a new generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0x210),
            config: EngagementLetterGeneratorConfig::default(),
        }
    }

    /// Create a new generator with custom configuration.
    pub fn with_config(seed: u64, config: EngagementLetterGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0x210),
            config,
        }
    }

    /// Generate an engagement letter for the given engagement.
    ///
    /// # Arguments
    /// * `engagement_id` — ID of the parent engagement
    /// * `client_name` — Name of the client entity (used as addressee base)
    /// * `entity_count` — Number of entities in scope (>1 → GroupAudit)
    /// * `period_end_date` — Balance sheet date
    /// * `currency` — Currency code for the fee
    /// * `applicable_framework` — Accounting framework (e.g. "IFRS")
    /// * `engagement_date` — Date the engagement letter is issued
    pub fn generate(
        &mut self,
        engagement_id: &str,
        client_name: &str,
        entity_count: usize,
        period_end_date: NaiveDate,
        currency: &str,
        applicable_framework: &str,
        engagement_date: NaiveDate,
    ) -> EngagementLetter {
        info!(
            "Generating engagement letter for {} (engagement {})",
            client_name, engagement_id
        );
        let scope = if entity_count > 1 {
            EngagementScope::GroupAudit
        } else {
            EngagementScope::StatutoryAudit
        };

        let complexity_factor = self.rng.random_range(
            self.config.complexity_factor_range.0..=self.config.complexity_factor_range.1,
        );
        let fee = self.config.base_fee_per_entity
            * Decimal::from(entity_count.max(1) as u64)
            * Decimal::try_from(complexity_factor).unwrap_or(Decimal::ONE);

        let fee_arrangement = FeeArrangement::new("Fixed fee", fee, currency);

        let reporting_deadline =
            period_end_date + Duration::days(self.config.reporting_deadline_days);

        let addressee = format!("The Board of Directors, {}", client_name);

        let mut letter = EngagementLetter::new(
            engagement_id,
            addressee,
            engagement_date,
            scope,
            fee_arrangement,
            reporting_deadline,
            applicable_framework,
        );

        letter.responsibilities_auditor = self.auditor_responsibilities(scope);
        letter.responsibilities_management = self.management_responsibilities();
        letter.special_terms = self.special_terms(scope);

        info!(
            "Engagement letter generated for {} scope={:?}",
            client_name, scope
        );
        letter
    }

    /// Generate a batch of engagement letters for multiple companies.
    ///
    /// Each entry in `engagements` is a tuple of
    /// `(engagement_id, client_name, period_end_date, currency)`.
    /// The `entity_count` for each company is treated as 1 unless the
    /// caller passes the total group entity count for a group engagement.
    pub fn generate_batch(
        &mut self,
        engagements: &[(String, String, NaiveDate, String)],
        total_entity_count: usize,
        applicable_framework: &str,
    ) -> Vec<EngagementLetter> {
        engagements
            .iter()
            .map(|(eng_id, client_name, period_end, currency)| {
                // Use planning start date as letter date (90 days before period end)
                let letter_date = *period_end - Duration::days(90);
                self.generate(
                    eng_id,
                    client_name,
                    total_entity_count,
                    *period_end,
                    currency,
                    applicable_framework,
                    letter_date,
                )
            })
            .collect()
    }

    fn auditor_responsibilities(&self, scope: EngagementScope) -> Vec<String> {
        let mut responsibilities = vec![
            "Express an opinion on whether the financial statements give a true and fair view \
             in accordance with the applicable financial reporting framework."
                .to_string(),
            "Plan and perform the audit in accordance with International Standards on Auditing \
             (ISAs) to obtain reasonable assurance that the financial statements are free from \
             material misstatement."
                .to_string(),
            "Identify and assess risks of material misstatement, whether due to fraud or error, \
             and design and perform audit procedures responsive to those risks."
                .to_string(),
            "Evaluate the appropriateness of accounting policies used and the reasonableness of \
             accounting estimates made by management."
                .to_string(),
            "Report to those charged with governance any significant deficiencies in internal \
             control identified during the audit."
                .to_string(),
        ];

        if matches!(scope, EngagementScope::GroupAudit) {
            responsibilities.push(
                "Coordinate the group audit including communication with component auditors \
                 per ISA 600."
                    .to_string(),
            );
        }

        responsibilities
    }

    fn management_responsibilities(&self) -> Vec<String> {
        vec![
            "Prepare financial statements in accordance with the applicable financial reporting \
             framework."
                .to_string(),
            "Maintain such internal control as management determines is necessary to enable the \
             preparation of financial statements that are free from material misstatement."
                .to_string(),
            "Provide the auditor with access to all information relevant to the preparation of \
             the financial statements, including books and records, documentation, and other matters."
                .to_string(),
            "Provide the auditor with unrestricted access to persons within the entity from whom \
             the auditor determines it necessary to obtain audit evidence."
                .to_string(),
            "Provide the auditor with a letter of representation at the conclusion of the audit."
                .to_string(),
        ]
    }

    fn special_terms(&mut self, scope: EngagementScope) -> Vec<String> {
        let mut terms = Vec::new();

        if matches!(scope, EngagementScope::GroupAudit) {
            terms.push(
                "Component auditor reports must be submitted to the group auditor no later than \
                 45 days after the period-end date."
                    .to_string(),
            );
        }

        // Randomly add additional terms
        if self.rng.random::<f64>() < 0.40 {
            terms.push(
                "The auditor's fee is subject to an uplift of up to 15% should the scope be \
                 materially extended due to circumstances beyond the auditor's control."
                    .to_string(),
            );
        }

        terms
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn period_end() -> NaiveDate {
        NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()
    }

    #[test]
    fn test_single_entity_produces_statutory_scope() {
        let mut gen = EngagementLetterGenerator::new(42);
        let letter = gen.generate(
            "ENG-001",
            "Solo Corp",
            1,
            period_end(),
            "USD",
            "US GAAP",
            period_end() - Duration::days(90),
        );
        assert_eq!(letter.scope, EngagementScope::StatutoryAudit);
    }

    #[test]
    fn test_multi_entity_produces_group_scope() {
        let mut gen = EngagementLetterGenerator::new(42);
        let letter = gen.generate(
            "ENG-002",
            "Group Parent SA",
            5,
            period_end(),
            "EUR",
            "IFRS",
            period_end() - Duration::days(90),
        );
        assert_eq!(letter.scope, EngagementScope::GroupAudit);
    }

    #[test]
    fn test_fee_is_positive() {
        let mut gen = EngagementLetterGenerator::new(42);
        let letter = gen.generate(
            "ENG-001",
            "Test Corp",
            2,
            period_end(),
            "GBP",
            "IFRS",
            period_end() - Duration::days(90),
        );
        assert!(letter.fee_arrangement.amount > Decimal::ZERO);
    }

    #[test]
    fn test_reporting_deadline_after_period_end() {
        let mut gen = EngagementLetterGenerator::new(42);
        let letter = gen.generate(
            "ENG-001",
            "Test Corp",
            1,
            period_end(),
            "USD",
            "US GAAP",
            period_end() - Duration::days(90),
        );
        assert!(letter.reporting_deadline > period_end());
    }

    #[test]
    fn test_responsibilities_are_non_empty() {
        let mut gen = EngagementLetterGenerator::new(42);
        let letter = gen.generate(
            "ENG-001",
            "Test Corp",
            1,
            period_end(),
            "USD",
            "US GAAP",
            period_end() - Duration::days(90),
        );
        assert!(!letter.responsibilities_auditor.is_empty());
        assert!(!letter.responsibilities_management.is_empty());
    }
}
