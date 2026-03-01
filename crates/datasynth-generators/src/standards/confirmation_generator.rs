//! ISA 505 External Confirmation generator.
//!
//! Produces [`ExternalConfirmation`] instances for audit engagements, covering
//! accounts receivable, accounts payable, bank, and legal confirmations with
//! realistic response statuses, reconciliation details, and alternative
//! procedures for non-responses.

use chrono::NaiveDate;
use datasynth_core::utils::seeded_rng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use uuid::Uuid;

use datasynth_standards::audit::confirmation::{
    AlternativeProcedureConclusion, AlternativeProcedureReason, AlternativeProcedures,
    ConfirmationConclusion, ConfirmationForm, ConfirmationReconciliation, ConfirmationResponse,
    ConfirmationResponseStatus, ConfirmationType, ExternalConfirmation, ReconcilingItem,
    ReconcilingItemType, ResponseReliability,
};

/// Configuration for the confirmation generator.
#[derive(Debug, Clone)]
pub struct ConfirmationGeneratorConfig {
    /// Total number of confirmations to generate.
    pub confirmation_count: usize,
    /// Positive response rate (0.0..1.0).
    pub positive_response_rate: f64,
    /// Exception rate (disagreements among responses).
    pub exception_rate: f64,
    /// Non-response rate.
    pub non_response_rate: f64,
    /// Type distribution: [AR, AP, Bank, Legal] (rest is Other).
    pub type_weights: [f64; 4],
}

impl Default for ConfirmationGeneratorConfig {
    fn default() -> Self {
        Self {
            confirmation_count: 50,
            positive_response_rate: 0.85,
            exception_rate: 0.10,
            non_response_rate: 0.10,
            type_weights: [0.40, 0.30, 0.20, 0.10],
        }
    }
}

/// Generates [`ExternalConfirmation`] instances for a given audit engagement.
pub struct ConfirmationGenerator {
    rng: ChaCha8Rng,
    config: ConfirmationGeneratorConfig,
    confirmation_counter: usize,
}

/// Discriminator added to the seed so this generator's RNG stream does not
/// overlap with other generators that may share the same base seed.
const SEED_DISCRIMINATOR: u64 = 0xAE_0E;

impl ConfirmationGenerator {
    /// Create a new generator with the given seed and default config.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, SEED_DISCRIMINATOR),
            config: ConfirmationGeneratorConfig::default(),
            confirmation_counter: 0,
        }
    }

    /// Create a new generator with the given seed and custom config.
    pub fn with_config(seed: u64, config: ConfirmationGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, SEED_DISCRIMINATOR),
            config,
            confirmation_counter: 0,
        }
    }

    /// Generate confirmations for an audit engagement.
    ///
    /// Returns `config.confirmation_count` confirmations with realistic
    /// response statuses, reconciliation details, and alternative procedures.
    pub fn generate_confirmations(
        &mut self,
        engagement_id: Uuid,
        base_date: NaiveDate,
    ) -> Vec<ExternalConfirmation> {
        let count = self.config.confirmation_count;
        let mut confirmations = Vec::with_capacity(count);

        for _ in 0..count {
            self.confirmation_counter += 1;
            let confirmation = self.build_confirmation(engagement_id, base_date);
            confirmations.push(confirmation);
        }

        confirmations
    }

    /// Pick a confirmation type from the configured weights.
    fn pick_confirmation_type(&mut self) -> ConfirmationType {
        let weights = &self.config.type_weights;
        let total: f64 = weights.iter().sum();
        let mut r: f64 = self.rng.random_range(0.0..total);

        for (i, &w) in weights.iter().enumerate() {
            r -= w;
            if r <= 0.0 {
                return match i {
                    0 => ConfirmationType::AccountsReceivable,
                    1 => ConfirmationType::AccountsPayable,
                    2 => ConfirmationType::Bank,
                    _ => ConfirmationType::Legal,
                };
            }
        }
        ConfirmationType::AccountsReceivable
    }

    /// Generate a confirmee name based on confirmation type.
    fn generate_confirmee_name(&mut self, conf_type: ConfirmationType) -> String {
        let n = self.confirmation_counter;
        match conf_type {
            ConfirmationType::AccountsReceivable => format!("Customer-{}", n),
            ConfirmationType::AccountsPayable => format!("Vendor-{}", n),
            ConfirmationType::Bank => {
                let cities = ["New York", "London", "Chicago", "Dallas", "Boston"];
                let idx = self.rng.random_range(0..cities.len());
                format!("Bank of {}", cities[idx])
            }
            ConfirmationType::Legal => format!("Law Office {}", n),
            _ => format!("Confirmee-{}", n),
        }
    }

    /// Generate client amount based on confirmation type.
    fn generate_client_amount(&mut self, conf_type: ConfirmationType) -> Decimal {
        match conf_type {
            ConfirmationType::AccountsReceivable | ConfirmationType::AccountsPayable => {
                Decimal::from(self.rng.random_range(1000..500_000_i64))
            }
            ConfirmationType::Bank => Decimal::from(self.rng.random_range(50_000..5_000_000_i64)),
            ConfirmationType::Legal => Decimal::from(self.rng.random_range(500..100_000_i64)),
            _ => Decimal::from(self.rng.random_range(1000..500_000_i64)),
        }
    }

    /// Generate an item description based on confirmation type.
    fn generate_item_description(&self, conf_type: ConfirmationType) -> String {
        match conf_type {
            ConfirmationType::AccountsReceivable => "Trade receivable balance".to_string(),
            ConfirmationType::AccountsPayable => "Trade payable balance".to_string(),
            ConfirmationType::Bank => "Bank account balance".to_string(),
            ConfirmationType::Legal => "Legal matters and contingencies".to_string(),
            _ => "Account balance".to_string(),
        }
    }

    /// Build a single confirmation with all its details.
    fn build_confirmation(
        &mut self,
        engagement_id: Uuid,
        base_date: NaiveDate,
    ) -> ExternalConfirmation {
        let conf_type = self.pick_confirmation_type();
        let confirmee_name = self.generate_confirmee_name(conf_type);
        let client_amount = self.generate_client_amount(conf_type);
        let item_description = self.generate_item_description(conf_type);

        let days_offset = self.rng.random_range(0..14_i64);
        let date_sent = base_date + chrono::Duration::days(days_offset);

        let mut confirmation = ExternalConfirmation::new(
            engagement_id,
            conf_type,
            &confirmee_name,
            &item_description,
            client_amount,
            "USD",
        );

        confirmation.confirmation_form = ConfirmationForm::Positive;
        confirmation.date_sent = date_sent;
        confirmation.prepared_by = format!("Audit Staff {}", self.confirmation_counter);
        confirmation.workpaper_reference =
            Some(format!("WP-CONF-{:04}", self.confirmation_counter));

        // Determine response status
        let roll: f64 = self.rng.random_range(0.0..1.0);

        if roll < self.config.non_response_rate {
            // No response
            self.apply_no_response(&mut confirmation, date_sent);
        } else {
            // Got some response; distribute among agrees/disagrees/partial
            let remaining_roll: f64 = self.rng.random_range(0.0..1.0);

            if remaining_roll < self.config.positive_response_rate {
                self.apply_received_agrees(&mut confirmation, date_sent, client_amount);
            } else if remaining_roll
                < self.config.positive_response_rate + self.config.exception_rate
            {
                self.apply_received_disagrees(&mut confirmation, date_sent, client_amount);
            } else {
                self.apply_received_partial(&mut confirmation, date_sent, client_amount);
            }
        }

        // Set follow-up date for pending/no-response
        if matches!(
            confirmation.response_status,
            ConfirmationResponseStatus::Pending | ConfirmationResponseStatus::NoResponse
        ) {
            confirmation.follow_up_date = Some(date_sent + chrono::Duration::days(14));
        }

        confirmation
    }

    /// Apply ReceivedAgrees status to a confirmation.
    fn apply_received_agrees(
        &mut self,
        confirmation: &mut ExternalConfirmation,
        date_sent: NaiveDate,
        client_amount: Decimal,
    ) {
        let response_days = self.rng.random_range(7..30_i64);
        let date_received = date_sent + chrono::Duration::days(response_days);

        let mut response = ConfirmationResponse::new(date_received, client_amount, true);
        response.respondent_name = format!("{} - Authorized Signer", confirmation.confirmee_name);
        response.appears_authentic = true;
        response.reliability_assessment = ResponseReliability::Reliable;

        confirmation.response_status = ConfirmationResponseStatus::ReceivedAgrees;
        confirmation.response = Some(response);
        confirmation.conclusion = ConfirmationConclusion::Confirmed;
    }

    /// Apply ReceivedDisagrees status to a confirmation.
    fn apply_received_disagrees(
        &mut self,
        confirmation: &mut ExternalConfirmation,
        date_sent: NaiveDate,
        client_amount: Decimal,
    ) {
        let response_days = self.rng.random_range(7..30_i64);
        let date_received = date_sent + chrono::Duration::days(response_days);

        // Confirmed amount differs by 0.90..1.10 factor
        let factor: f64 = self.rng.random_range(0.90..1.10);
        let factor_decimal = Decimal::from_f64_retain(factor).unwrap_or(Decimal::ONE);
        let confirmed_amount = client_amount * factor_decimal;

        let mut response = ConfirmationResponse::new(date_received, confirmed_amount, false);
        response.respondent_name = format!("{} - Authorized Signer", confirmation.confirmee_name);
        response.appears_authentic = true;
        response.reliability_assessment = ResponseReliability::Reliable;

        confirmation.response_status = ConfirmationResponseStatus::ReceivedDisagrees;
        confirmation.response = Some(response);

        // Create reconciliation
        let mut reconciliation = ConfirmationReconciliation::new(client_amount, confirmed_amount);

        // Add a reconciling item
        let item_type = if self.rng.random_bool(0.5) {
            ReconcilingItemType::CashInTransit
        } else {
            ReconcilingItemType::CutoffAdjustment
        };

        let difference = client_amount - confirmed_amount;
        let item = ReconcilingItem {
            description: match item_type {
                ReconcilingItemType::CashInTransit => "Payment in transit".to_string(),
                ReconcilingItemType::CutoffAdjustment => "Cutoff timing difference".to_string(),
                _ => "Other reconciling item".to_string(),
            },
            amount: difference,
            item_type,
            evidence: "Examined supporting documentation".to_string(),
        };
        reconciliation.add_reconciling_item(item);

        confirmation.reconciliation = Some(reconciliation);

        // Conclusion: 80% ExceptionResolved, 20% PotentialMisstatement
        if self.rng.random_bool(0.80) {
            confirmation.conclusion = ConfirmationConclusion::ExceptionResolved;
        } else {
            confirmation.conclusion = ConfirmationConclusion::PotentialMisstatement;
        }
    }

    /// Apply NoResponse status with alternative procedures.
    fn apply_no_response(&mut self, confirmation: &mut ExternalConfirmation, date_sent: NaiveDate) {
        confirmation.response_status = ConfirmationResponseStatus::NoResponse;
        confirmation.follow_up_date = Some(date_sent + chrono::Duration::days(14));

        let mut alt_procedures = AlternativeProcedures::new(AlternativeProcedureReason::NoResponse);
        alt_procedures
            .evidence_obtained
            .push("Reviewed subsequent transactions".to_string());
        alt_procedures
            .evidence_obtained
            .push("Examined supporting documentation".to_string());

        // Conclusion: 90% satisfactory, 10% insufficient
        if self.rng.random_bool(0.90) {
            alt_procedures.conclusion = AlternativeProcedureConclusion::SufficientEvidence;
            confirmation.conclusion = ConfirmationConclusion::AlternativesSatisfactory;
        } else {
            alt_procedures.conclusion = AlternativeProcedureConclusion::InsufficientEvidence;
            confirmation.conclusion = ConfirmationConclusion::InsufficientEvidence;
        }

        confirmation.alternative_procedures = Some(alt_procedures);
    }

    /// Apply ReceivedPartial status to a confirmation.
    fn apply_received_partial(
        &mut self,
        confirmation: &mut ExternalConfirmation,
        date_sent: NaiveDate,
        client_amount: Decimal,
    ) {
        let response_days = self.rng.random_range(7..30_i64);
        let date_received = date_sent + chrono::Duration::days(response_days);

        // Partial amount: 50-90% of client amount
        let partial_factor: f64 = self.rng.random_range(0.50..0.90);
        let partial_decimal =
            Decimal::from_f64_retain(partial_factor).unwrap_or(Decimal::new(70, 2));
        let partial_amount = client_amount * partial_decimal;

        let mut response = ConfirmationResponse::new(date_received, partial_amount, false);
        response.respondent_name = format!("{} - Authorized Signer", confirmation.confirmee_name);
        response.appears_authentic = true;
        response.reliability_assessment = ResponseReliability::Reliable;
        response.comments = "Partial information provided".to_string();

        confirmation.response_status = ConfirmationResponseStatus::ReceivedPartial;
        confirmation.response = Some(response);
        confirmation.conclusion = ConfirmationConclusion::ExceptionResolved;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_generation() {
        let engagement_id = Uuid::nil();
        let base_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        let mut gen1 = ConfirmationGenerator::new(42);
        let mut gen2 = ConfirmationGenerator::new(42);

        let results1 = gen1.generate_confirmations(engagement_id, base_date);
        let results2 = gen2.generate_confirmations(engagement_id, base_date);

        assert_eq!(results1.len(), results2.len());
        for (c1, c2) in results1.iter().zip(results2.iter()) {
            assert_eq!(c1.confirmee_name, c2.confirmee_name);
            assert_eq!(c1.client_amount, c2.client_amount);
            assert_eq!(c1.response_status, c2.response_status);
            assert_eq!(c1.date_sent, c2.date_sent);
            assert_eq!(c1.prepared_by, c2.prepared_by);
        }
    }

    #[test]
    fn test_confirmation_count() {
        let engagement_id = Uuid::nil();
        let base_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        let config = ConfirmationGeneratorConfig {
            confirmation_count: 25,
            ..Default::default()
        };
        let mut gen = ConfirmationGenerator::with_config(42, config);
        let results = gen.generate_confirmations(engagement_id, base_date);

        assert_eq!(results.len(), 25);
    }

    #[test]
    fn test_type_distribution() {
        let engagement_id = Uuid::nil();
        let base_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        let config = ConfirmationGeneratorConfig {
            confirmation_count: 200,
            ..Default::default()
        };
        let mut gen = ConfirmationGenerator::with_config(42, config);
        let results = gen.generate_confirmations(engagement_id, base_date);

        let ar_count = results
            .iter()
            .filter(|c| c.confirmation_type == ConfirmationType::AccountsReceivable)
            .count();
        let ap_count = results
            .iter()
            .filter(|c| c.confirmation_type == ConfirmationType::AccountsPayable)
            .count();
        let bank_count = results
            .iter()
            .filter(|c| c.confirmation_type == ConfirmationType::Bank)
            .count();
        let legal_count = results
            .iter()
            .filter(|c| c.confirmation_type == ConfirmationType::Legal)
            .count();

        // With weights [0.40, 0.30, 0.20, 0.10], AR > AP > Bank > Legal
        assert!(
            ar_count > ap_count,
            "AR ({}) should exceed AP ({})",
            ar_count,
            ap_count
        );
        assert!(
            ap_count > bank_count,
            "AP ({}) should exceed Bank ({})",
            ap_count,
            bank_count
        );
        assert!(
            bank_count > legal_count,
            "Bank ({}) should exceed Legal ({})",
            bank_count,
            legal_count
        );
    }

    #[test]
    fn test_positive_response_rate() {
        let engagement_id = Uuid::nil();
        let base_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        let config = ConfirmationGeneratorConfig {
            confirmation_count: 100,
            positive_response_rate: 1.0,
            exception_rate: 0.0,
            non_response_rate: 0.0,
            type_weights: [1.0, 0.0, 0.0, 0.0],
        };
        let mut gen = ConfirmationGenerator::with_config(42, config);
        let results = gen.generate_confirmations(engagement_id, base_date);

        for c in &results {
            assert_eq!(
                c.response_status,
                ConfirmationResponseStatus::ReceivedAgrees,
                "All should be ReceivedAgrees when positive_response_rate=1.0"
            );
        }
    }

    #[test]
    fn test_non_response_generates_alternatives() {
        let engagement_id = Uuid::nil();
        let base_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        let config = ConfirmationGeneratorConfig {
            confirmation_count: 100,
            positive_response_rate: 0.0,
            exception_rate: 0.0,
            non_response_rate: 1.0,
            type_weights: [1.0, 0.0, 0.0, 0.0],
        };
        let mut gen = ConfirmationGenerator::with_config(42, config);
        let results = gen.generate_confirmations(engagement_id, base_date);

        for c in &results {
            assert_eq!(c.response_status, ConfirmationResponseStatus::NoResponse);
            assert!(
                c.alternative_procedures.is_some(),
                "NoResponse confirmations must have alternative_procedures"
            );
        }
    }

    #[test]
    fn test_disagreements_have_reconciliation() {
        let engagement_id = Uuid::nil();
        let base_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        // Force all to be disagrees: non_response_rate=0, positive_response_rate=0,
        // exception_rate=1.0 so the "remaining" roll always lands in disagree range.
        let config = ConfirmationGeneratorConfig {
            confirmation_count: 50,
            positive_response_rate: 0.0,
            exception_rate: 1.0,
            non_response_rate: 0.0,
            type_weights: [1.0, 0.0, 0.0, 0.0],
        };
        let mut gen = ConfirmationGenerator::with_config(42, config);
        let results = gen.generate_confirmations(engagement_id, base_date);

        let disagrees: Vec<_> = results
            .iter()
            .filter(|c| c.response_status == ConfirmationResponseStatus::ReceivedDisagrees)
            .collect();

        assert!(!disagrees.is_empty(), "Should have some disagreements");

        for c in &disagrees {
            assert!(
                c.reconciliation.is_some(),
                "ReceivedDisagrees confirmations must have reconciliation"
            );
        }
    }

    #[test]
    fn test_all_confirmations_have_prepared_by() {
        let engagement_id = Uuid::nil();
        let base_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        let mut gen = ConfirmationGenerator::new(42);
        let results = gen.generate_confirmations(engagement_id, base_date);

        for c in &results {
            assert!(
                !c.prepared_by.is_empty(),
                "All confirmations must have non-empty prepared_by"
            );
        }
    }

    #[test]
    fn test_zero_non_response_rate() {
        let engagement_id = Uuid::nil();
        let base_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        let config = ConfirmationGeneratorConfig {
            confirmation_count: 100,
            positive_response_rate: 0.85,
            exception_rate: 0.10,
            non_response_rate: 0.0,
            type_weights: [0.40, 0.30, 0.20, 0.10],
        };
        let mut gen = ConfirmationGenerator::with_config(42, config);
        let results = gen.generate_confirmations(engagement_id, base_date);

        let no_responses = results
            .iter()
            .filter(|c| c.response_status == ConfirmationResponseStatus::NoResponse)
            .count();

        assert_eq!(
            no_responses, 0,
            "With non_response_rate=0.0, there should be no NoResponse statuses"
        );
    }
}
