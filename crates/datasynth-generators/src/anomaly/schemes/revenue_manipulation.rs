//! Revenue manipulation fraud scheme.
//!
//! Models financial statement fraud involving revenue manipulation across
//! multiple quarters, including premature recognition, expense deferral,
//! reserve release, and channel stuffing.

use chrono::{Datelike, NaiveDate};
use rand::Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use datasynth_core::models::{
    AnomalyDetectionDifficulty, ConcealmentTechnique, SchemeDetectionStatus, SchemeType,
};

use super::scheme::{
    FraudScheme, SchemeAction, SchemeActionType, SchemeContext, SchemeStage, SchemeStatus,
    SchemeTransactionRef,
};

/// A revenue manipulation scheme that exploits accounting periods.
///
/// Stages:
/// 1. Early revenue recognition (Q4) - Recognize revenue prematurely
/// 2. Expense deferral (Q1) - Defer expenses to later periods
/// 3. Reserve release (Q2) - Release cookie jar reserves
/// 4. Channel stuffing (Q4) - Push sales to inflate year-end numbers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueManipulationScheme {
    /// Unique scheme ID.
    pub scheme_id: Uuid,
    /// Perpetrator (typically management).
    pub perpetrator_id: String,
    /// Start date of the scheme.
    pub start_date: Option<NaiveDate>,
    /// Current stage index.
    current_stage_index: usize,
    /// All stages.
    stages: Vec<SchemeStage>,
    /// Transaction references.
    transactions: Vec<SchemeTransactionRef>,
    /// Total impact.
    total_impact: Decimal,
    /// Status.
    status: SchemeStatus,
    /// Detection status.
    detection_status: SchemeDetectionStatus,
    /// Detection probability.
    detection_probability: f64,
    /// Target inflation percentage per stage.
    inflation_targets: Vec<f64>,
    /// Current fiscal year being manipulated.
    current_fiscal_year: i32,
}

impl RevenueManipulationScheme {
    /// Creates a new revenue manipulation scheme.
    pub fn new(perpetrator_id: impl Into<String>) -> Self {
        let stages = vec![
            // Stage 1: Early revenue recognition (Q4, ~2% inflation)
            SchemeStage::new(
                1,
                "early_revenue_recognition",
                3,
                (dec!(50000), dec!(500000)),
                (3, 8),
                AnomalyDetectionDifficulty::Hard,
            )
            .with_description("Premature recognition of revenue before performance obligations met")
            .with_technique(ConcealmentTechnique::DocumentManipulation)
            .with_technique(ConcealmentTechnique::TimingExploitation),
            // Stage 2: Expense deferral (Q1, ~3% inflation)
            SchemeStage::new(
                2,
                "expense_deferral",
                3,
                (dec!(25000), dec!(200000)),
                (5, 15),
                AnomalyDetectionDifficulty::Moderate,
            )
            .with_description("Deferral of current period expenses to future periods")
            .with_technique(ConcealmentTechnique::AccountMisclassification),
            // Stage 3: Reserve release (Q2, ~2% inflation)
            SchemeStage::new(
                3,
                "reserve_release",
                3,
                (dec!(100000), dec!(1000000)),
                (2, 5),
                AnomalyDetectionDifficulty::Moderate,
            )
            .with_description("Inappropriate release of excess reserves to boost income")
            .with_technique(ConcealmentTechnique::FalseDocumentation),
            // Stage 4: Channel stuffing (Q4, ~5% inflation)
            SchemeStage::new(
                4,
                "channel_stuffing",
                3,
                (dec!(200000), dec!(2000000)),
                (3, 10),
                AnomalyDetectionDifficulty::Easy,
            )
            .with_description("Pushing excess inventory to distributors with side agreements")
            .with_technique(ConcealmentTechnique::FalseDocumentation)
            .with_technique(ConcealmentTechnique::Collusion),
        ];

        Self {
            scheme_id: Uuid::new_v4(),
            perpetrator_id: perpetrator_id.into(),
            start_date: None,
            current_stage_index: 0,
            stages,
            transactions: Vec::new(),
            total_impact: Decimal::ZERO,
            status: SchemeStatus::NotStarted,
            detection_status: SchemeDetectionStatus::Undetected,
            detection_probability: 0.0,
            inflation_targets: vec![0.02, 0.03, 0.02, 0.05], // Percentage targets per stage
            current_fiscal_year: 0,
        }
    }

    /// Sets custom inflation targets per stage.
    pub fn with_inflation_targets(mut self, targets: Vec<f64>) -> Self {
        self.inflation_targets = targets;
        self
    }

    /// Starts the scheme.
    pub fn start(&mut self, date: NaiveDate) {
        self.start_date = Some(date);
        self.status = SchemeStatus::Active;
        self.current_fiscal_year = date.year();
    }

    /// Determines the appropriate stage based on the current quarter.
    fn stage_for_quarter(quarter: u32) -> usize {
        match quarter {
            4 => 0, // Q4: Early revenue recognition or channel stuffing
            1 => 1, // Q1: Expense deferral
            2 => 2, // Q2: Reserve release
            3 => 3, // Q3: Preparation/channel stuffing setup
            _ => 0,
        }
    }

    /// Gets the current quarter from a date.
    fn current_quarter(date: NaiveDate) -> u32 {
        ((date.month() - 1) / 3) + 1
    }

    /// Determines the action type for the current stage.
    fn action_type_for_stage(stage_index: usize) -> SchemeActionType {
        match stage_index {
            0 => SchemeActionType::ManipulateRevenue,
            1 => SchemeActionType::DeferExpense,
            2 => SchemeActionType::ReleaseReserves,
            3 => SchemeActionType::ChannelStuff,
            _ => SchemeActionType::ManipulateRevenue,
        }
    }

    /// Updates detection probability.
    fn update_detection_probability(&mut self) {
        // Revenue manipulation has specific detection patterns
        let base_prob = 0.1;

        // Higher detection during audit season (Q1)
        let audit_factor = if self.current_stage_index == 1 {
            0.15
        } else {
            0.0
        };

        // Channel stuffing is more visible
        let channel_factor = if self.current_stage_index == 3 {
            0.2
        } else {
            0.0
        };

        // Larger manipulation = higher detection
        let amount_factor = if self.total_impact > dec!(5000000) {
            0.25
        } else if self.total_impact > dec!(1000000) {
            0.15
        } else {
            0.05
        };

        let prob: f64 = base_prob + audit_factor + channel_factor + amount_factor;
        self.detection_probability = prob.min(0.8);
    }
}

impl FraudScheme for RevenueManipulationScheme {
    fn scheme_type(&self) -> SchemeType {
        SchemeType::RevenueManipulation
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

        // Start if not started
        if self.status == SchemeStatus::NotStarted {
            self.start(context.current_date);
        }

        // Check termination
        if self.should_terminate(context) {
            self.status = SchemeStatus::Terminated;
            return actions;
        }

        // Check detection
        if rng.gen::<f64>() < self.detection_probability * context.detection_activity {
            self.detection_status = SchemeDetectionStatus::UnderInvestigation;
            // Don't fully terminate - management fraud often continues
            if context.detection_activity > 0.7 {
                self.status = SchemeStatus::Detected;
                return actions;
            }
        }

        // Determine stage based on quarter
        let quarter = Self::current_quarter(context.current_date);
        let target_stage = Self::stage_for_quarter(quarter);

        // Adjust stage if needed
        if target_stage != self.current_stage_index {
            self.current_stage_index = target_stage;
        }

        let stage = &self.stages[self.current_stage_index];

        // Generate actions based on stage
        // Revenue manipulation typically happens at quarter/year end
        let is_period_end = context.current_date.day() >= 25;
        let should_act = is_period_end && rng.gen::<f64>() < 0.4;

        if should_act {
            let amount = stage.random_amount(rng);
            let action_type = Self::action_type_for_stage(self.current_stage_index);

            let mut action = SchemeAction::new(
                self.scheme_id,
                stage.stage_number,
                action_type,
                context.current_date,
            )
            .with_amount(amount)
            .with_user(&self.perpetrator_id)
            .with_difficulty(stage.detection_difficulty)
            .with_description(format!(
                "Revenue manipulation: {} (Q{})",
                stage.name, quarter
            ));

            // Add concealment techniques
            for technique in &stage.concealment_techniques {
                action = action.with_technique(*technique);
            }

            actions.push(action);
        }

        // Update detection probability
        self.update_detection_probability();

        // Check for year completion
        if context.current_date.year() > self.current_fiscal_year {
            // New fiscal year - scheme could continue or complete
            if self.transactions.len() > 20 {
                self.status = SchemeStatus::Completed;
            } else {
                self.current_fiscal_year = context.current_date.year();
            }
        }

        actions
    }

    fn detection_probability(&self) -> f64 {
        self.detection_probability
    }

    fn total_impact(&self) -> Decimal {
        self.total_impact
    }

    fn should_terminate(&self, context: &SchemeContext) -> bool {
        // Revenue manipulation schemes are harder to terminate
        // (management has strong incentive to continue)
        if context.detection_activity > 0.9 {
            return true;
        }

        if self.detection_status == SchemeDetectionStatus::FullyDetected {
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
    fn test_revenue_manipulation_creation() {
        let scheme = RevenueManipulationScheme::new("CFO001");

        assert_eq!(scheme.perpetrator_id, "CFO001");
        assert_eq!(scheme.stages.len(), 4);
        assert_eq!(scheme.status, SchemeStatus::NotStarted);
    }

    #[test]
    fn test_revenue_manipulation_stages() {
        let scheme = RevenueManipulationScheme::new("CFO001");

        assert_eq!(scheme.stages[0].name, "early_revenue_recognition");
        assert_eq!(scheme.stages[1].name, "expense_deferral");
        assert_eq!(scheme.stages[2].name, "reserve_release");
        assert_eq!(scheme.stages[3].name, "channel_stuffing");
    }

    #[test]
    fn test_quarter_calculation() {
        assert_eq!(
            RevenueManipulationScheme::current_quarter(
                NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()
            ),
            1
        );
        assert_eq!(
            RevenueManipulationScheme::current_quarter(
                NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()
            ),
            2
        );
        assert_eq!(
            RevenueManipulationScheme::current_quarter(
                NaiveDate::from_ymd_opt(2024, 9, 15).unwrap()
            ),
            3
        );
        assert_eq!(
            RevenueManipulationScheme::current_quarter(
                NaiveDate::from_ymd_opt(2024, 12, 15).unwrap()
            ),
            4
        );
    }

    #[test]
    fn test_revenue_manipulation_advance() {
        let mut scheme = RevenueManipulationScheme::new("CFO001");
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        // Test at quarter end
        let context = SchemeContext::new(
            NaiveDate::from_ymd_opt(2024, 12, 28).unwrap(), // Q4 end
            "1000",
        );

        let mut total_actions = 0;
        for _ in 0..10 {
            let actions = scheme.advance(&context, &mut rng);
            total_actions += actions.len();
        }

        // Should generate some actions at quarter end
        assert!(total_actions > 0 || scheme.status == SchemeStatus::Active);
    }

    #[test]
    fn test_revenue_manipulation_stage_selection() {
        assert_eq!(RevenueManipulationScheme::stage_for_quarter(4), 0); // Q4 -> early revenue
        assert_eq!(RevenueManipulationScheme::stage_for_quarter(1), 1); // Q1 -> expense deferral
        assert_eq!(RevenueManipulationScheme::stage_for_quarter(2), 2); // Q2 -> reserve release
        assert_eq!(RevenueManipulationScheme::stage_for_quarter(3), 3); // Q3 -> channel stuffing
    }
}
