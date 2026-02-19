//! Gradual embezzlement fraud scheme.
//!
//! Models a classic embezzlement pattern where a perpetrator starts with small
//! test amounts and gradually escalates over time.

use chrono::NaiveDate;
use rand::Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};

use datasynth_core::models::{
    AnomalyDetectionDifficulty, ConcealmentTechnique, SchemeDetectionStatus, SchemeType,
};

use super::scheme::{
    FraudScheme, SchemeAction, SchemeActionType, SchemeContext, SchemeStage, SchemeStatus,
    SchemeTransactionRef,
};

/// A gradual embezzlement scheme that evolves through stages.
///
/// Stages:
/// 1. Testing (2 months): Small amounts to test detection
/// 2. Escalation (6 months): Gradually increasing amounts
/// 3. Acceleration (3 months): Larger, more frequent thefts
/// 4. Desperation (1 month): Large amounts before exit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradualEmbezzlementScheme {
    /// Unique scheme ID.
    pub scheme_id: Uuid,
    /// Perpetrator user ID.
    pub perpetrator_id: String,
    /// Start date of the scheme.
    pub start_date: Option<NaiveDate>,
    /// Current stage index (0-indexed).
    current_stage_index: usize,
    /// All stages.
    stages: Vec<SchemeStage>,
    /// Transaction references.
    transactions: Vec<SchemeTransactionRef>,
    /// Total impact so far.
    total_impact: Decimal,
    /// Current status.
    status: SchemeStatus,
    /// Detection status.
    detection_status: SchemeDetectionStatus,
    /// Cumulative detection probability.
    detection_probability: f64,
    /// Preferred accounts for embezzlement.
    preferred_accounts: Vec<String>,
    /// Number of transactions in current stage.
    stage_transaction_count: u32,
    /// Days since last transaction.
    days_since_last_transaction: u32,
}

impl GradualEmbezzlementScheme {
    /// Creates a new gradual embezzlement scheme.
    pub fn new(perpetrator_id: impl Into<String>) -> Self {
        let stages = vec![
            // Stage 1: Testing (2 months, $100-500, hard to detect)
            SchemeStage::new(
                1,
                "testing",
                2,
                (dec!(100), dec!(500)),
                (2, 4),
                AnomalyDetectionDifficulty::Hard,
            )
            .with_description("Initial testing phase with small amounts")
            .with_technique(ConcealmentTechnique::TimingExploitation),
            // Stage 2: Escalation (6 months, $500-2000, moderate)
            SchemeStage::new(
                2,
                "escalation",
                6,
                (dec!(500), dec!(2000)),
                (4, 8),
                AnomalyDetectionDifficulty::Moderate,
            )
            .with_description("Gradual increase in amounts as confidence grows")
            .with_technique(ConcealmentTechnique::AccountMisclassification),
            // Stage 3: Acceleration (3 months, $2000-10000, easy)
            SchemeStage::new(
                3,
                "acceleration",
                3,
                (dec!(2000), dec!(10000)),
                (6, 12),
                AnomalyDetectionDifficulty::Easy,
            )
            .with_description("Accelerated theft with larger amounts")
            .with_technique(ConcealmentTechnique::DocumentManipulation)
            .with_technique(ConcealmentTechnique::ApprovalCircumvention),
            // Stage 4: Desperation (1 month, $10000-50000, trivial)
            SchemeStage::new(
                4,
                "desperation",
                1,
                (dec!(10000), dec!(50000)),
                (2, 5),
                AnomalyDetectionDifficulty::Trivial,
            )
            .with_description("Final desperate phase before exit or detection")
            .with_technique(ConcealmentTechnique::FalseDocumentation),
        ];

        let uuid_factory = DeterministicUuidFactory::new(0, GeneratorType::Anomaly);

        Self {
            scheme_id: uuid_factory.next(),
            perpetrator_id: perpetrator_id.into(),
            start_date: None,
            current_stage_index: 0,
            stages,
            transactions: Vec::new(),
            total_impact: Decimal::ZERO,
            status: SchemeStatus::NotStarted,
            detection_status: SchemeDetectionStatus::Undetected,
            detection_probability: 0.0,
            preferred_accounts: Vec::new(),
            stage_transaction_count: 0,
            days_since_last_transaction: 0,
        }
    }

    /// Sets preferred accounts for embezzlement.
    pub fn with_accounts(mut self, accounts: Vec<String>) -> Self {
        self.preferred_accounts = accounts;
        self
    }

    /// Starts the scheme on a given date.
    pub fn start(&mut self, date: NaiveDate) {
        self.start_date = Some(date);
        self.status = SchemeStatus::Active;
    }

    /// Gets the current stage end date.
    fn stage_end_date(&self) -> Option<NaiveDate> {
        self.start_date.map(|start| {
            let months_elapsed: u32 = self.stages[..self.current_stage_index]
                .iter()
                .map(|s| s.duration_months)
                .sum();
            let stage_months = self.stages[self.current_stage_index].duration_months;
            start + chrono::Months::new(months_elapsed + stage_months)
        })
    }

    /// Checks if it's time to advance to the next stage.
    fn should_advance_stage(&self, current_date: NaiveDate) -> bool {
        if let Some(end_date) = self.stage_end_date() {
            current_date >= end_date && self.current_stage_index < self.stages.len() - 1
        } else {
            false
        }
    }

    /// Advances to the next stage.
    fn advance_stage(&mut self) {
        if self.current_stage_index < self.stages.len() - 1 {
            self.current_stage_index += 1;
            self.stage_transaction_count = 0;
        }
    }

    /// Updates detection probability based on activity.
    fn update_detection_probability(&mut self) {
        let stage = &self.stages[self.current_stage_index];

        // Base probability from detection difficulty
        let base_prob = 1.0 - stage.detection_difficulty.expected_detection_rate();

        // Increase with total impact
        let impact_factor = if self.total_impact > dec!(100000) {
            0.3
        } else if self.total_impact > dec!(50000) {
            0.2
        } else if self.total_impact > dec!(10000) {
            0.1
        } else {
            0.0
        };

        // Increase with transaction count
        let count_factor = (self.transactions.len() as f64 * 0.02).min(0.2);

        // Later stages are riskier
        let stage_factor = (self.current_stage_index as f64 * 0.1).min(0.3);

        self.detection_probability =
            (base_prob + impact_factor + count_factor + stage_factor).min(0.95);
    }

    /// Selects an account for the transaction.
    fn select_account<R: Rng + ?Sized>(
        &self,
        context: &SchemeContext,
        rng: &mut R,
    ) -> Option<String> {
        // Prefer established accounts
        if !self.preferred_accounts.is_empty() && rng.gen::<f64>() < 0.8 {
            let idx = rng.gen_range(0..self.preferred_accounts.len());
            return Some(self.preferred_accounts[idx].clone());
        }

        // Fall back to available accounts
        if !context.available_accounts.is_empty() {
            let idx = rng.gen_range(0..context.available_accounts.len());
            return Some(context.available_accounts[idx].clone());
        }

        None
    }
}

impl FraudScheme for GradualEmbezzlementScheme {
    fn scheme_type(&self) -> SchemeType {
        SchemeType::GradualEmbezzlement
    }

    fn scheme_id(&self) -> Uuid {
        self.scheme_id
    }

    fn current_stage(&self) -> &SchemeStage {
        &self.stages[self.current_stage_index]
    }

    fn current_stage_number(&self) -> u32 {
        self.stages[self.current_stage_index].stage_number
    }

    fn stages(&self) -> &[SchemeStage] {
        &self.stages
    }

    fn status(&self) -> SchemeStatus {
        self.status
    }

    fn detection_status(&self) -> SchemeDetectionStatus {
        self.detection_status
    }

    fn advance(
        &mut self,
        context: &SchemeContext,
        rng: &mut dyn rand::RngCore,
    ) -> Vec<SchemeAction> {
        let mut actions = Vec::new();

        // Start scheme if not started
        if self.status == SchemeStatus::NotStarted {
            self.start(context.current_date);
        }

        // Check if scheme should terminate
        if self.should_terminate(context) {
            self.status = SchemeStatus::Terminated;
            return actions;
        }

        // Check if detected
        if rng.gen::<f64>() < self.detection_probability * context.detection_activity {
            self.detection_status = SchemeDetectionStatus::PartiallyDetected;
            self.status = SchemeStatus::Detected;
            return actions;
        }

        // Check for stage advancement
        if self.should_advance_stage(context.current_date) {
            self.advance_stage();
        }

        // Pause if audit in progress
        if context.audit_in_progress && rng.gen::<f64>() < 0.8 {
            self.status = SchemeStatus::Paused;
            return actions;
        }
        self.status = SchemeStatus::Active;

        let stage = &self.stages[self.current_stage_index];

        // Determine if we should generate a transaction this period
        let target_count = stage.random_transaction_count(rng);
        let should_transact = self.stage_transaction_count < target_count
            && self.days_since_last_transaction >= 3 // At least 3 days between transactions
            && rng.gen::<f64>() < 0.3; // Random chance

        if should_transact {
            let amount = stage.random_amount(rng);
            let account = self.select_account(context, rng);

            let mut action = SchemeAction::new(
                self.scheme_id,
                stage.stage_number,
                SchemeActionType::CreateFraudulentEntry,
                context.current_date,
            )
            .with_amount(amount)
            .with_user(&self.perpetrator_id)
            .with_difficulty(stage.detection_difficulty)
            .with_description(format!(
                "Embezzlement stage {} - {}",
                stage.stage_number, stage.name
            ));

            if let Some(acct) = account {
                action = action.with_account(acct);
            }

            // Add concealment techniques from stage
            for technique in &stage.concealment_techniques {
                action = action.with_technique(*technique);
            }

            self.stage_transaction_count += 1;
            self.days_since_last_transaction = 0;

            actions.push(action);
        } else {
            self.days_since_last_transaction += 1;
        }

        // Check if scheme is complete (all stages done)
        if self.current_stage_index == self.stages.len() - 1 {
            if let Some(end_date) = self.stage_end_date() {
                if context.current_date >= end_date {
                    self.status = SchemeStatus::Completed;
                }
            }
        }

        // Update detection probability
        self.update_detection_probability();

        actions
    }

    fn detection_probability(&self) -> f64 {
        self.detection_probability
    }

    fn total_impact(&self) -> Decimal {
        self.total_impact
    }

    fn should_terminate(&self, context: &SchemeContext) -> bool {
        // Terminate if detection activity is very high
        if context.detection_activity > 0.8 {
            return true;
        }

        // Terminate if detection probability is too high
        if self.detection_probability > 0.9 {
            return true;
        }

        // Terminate if already detected
        if self.detection_status != SchemeDetectionStatus::Undetected {
            return true;
        }

        false
    }

    fn perpetrator_id(&self) -> &str {
        &self.perpetrator_id
    }

    fn start_date(&self) -> Option<NaiveDate> {
        self.start_date
    }

    fn transaction_refs(&self) -> &[SchemeTransactionRef] {
        &self.transactions
    }

    fn record_transaction(&mut self, transaction: SchemeTransactionRef) {
        self.total_impact += transaction.amount;
        self.transactions.push(transaction);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_embezzlement_scheme_creation() {
        let scheme = GradualEmbezzlementScheme::new("USER001")
            .with_accounts(vec!["5000".to_string(), "6000".to_string()]);

        assert_eq!(scheme.perpetrator_id, "USER001");
        assert_eq!(scheme.stages.len(), 4);
        assert_eq!(scheme.status, SchemeStatus::NotStarted);
        assert_eq!(scheme.preferred_accounts.len(), 2);
    }

    #[test]
    fn test_embezzlement_scheme_stages() {
        let scheme = GradualEmbezzlementScheme::new("USER001");

        // Check stage progression
        assert_eq!(scheme.stages[0].name, "testing");
        assert_eq!(scheme.stages[0].duration_months, 2);
        assert_eq!(
            scheme.stages[0].detection_difficulty,
            AnomalyDetectionDifficulty::Hard
        );

        assert_eq!(scheme.stages[3].name, "desperation");
        assert_eq!(
            scheme.stages[3].detection_difficulty,
            AnomalyDetectionDifficulty::Trivial
        );
    }

    #[test]
    fn test_embezzlement_scheme_advance() {
        let mut scheme =
            GradualEmbezzlementScheme::new("USER001").with_accounts(vec!["5000".to_string()]);

        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let context = SchemeContext::new(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(), "1000")
            .with_accounts(vec!["5000".to_string(), "6000".to_string()])
            .with_users(vec!["USER001".to_string()]);

        // Advance multiple times
        let mut total_actions = 0;
        for day in 0..30 {
            let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap() + chrono::Duration::days(day);
            let mut ctx = context.clone();
            ctx.current_date = date;

            let actions = scheme.advance(&ctx, &mut rng);
            total_actions += actions.len();
        }

        // Should have generated some actions
        assert!(total_actions > 0);
        assert_eq!(scheme.status, SchemeStatus::Active);
    }

    #[test]
    fn test_embezzlement_scheme_pauses_during_audit() {
        let mut scheme = GradualEmbezzlementScheme::new("USER001");

        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let context = SchemeContext::new(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(), "1000")
            .with_audit(true);

        // Start the scheme
        scheme.start(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

        // Advance during audit - should often pause
        let mut pause_count = 0;
        for _ in 0..10 {
            scheme.advance(&context, &mut rng);
            if scheme.status == SchemeStatus::Paused {
                pause_count += 1;
            }
        }

        // Should have paused at least some times
        assert!(pause_count > 0);
    }

    #[test]
    fn test_embezzlement_scheme_terminates_on_high_detection() {
        let mut scheme = GradualEmbezzlementScheme::new("USER001");
        scheme.start(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

        let context = SchemeContext::new(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(), "1000")
            .with_detection_activity(0.9);

        assert!(scheme.should_terminate(&context));
    }
}
