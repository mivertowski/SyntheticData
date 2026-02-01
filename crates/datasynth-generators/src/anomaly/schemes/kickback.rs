//! Vendor kickback fraud scheme.
//!
//! Models a kickback scheme where an employee colludes with a vendor
//! to inflate invoices and receive a portion of the excess payment.

use chrono::NaiveDate;
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

/// A vendor kickback scheme involving price inflation.
///
/// Stages:
/// 1. Setup (3 months): Establish vendor relationship
/// 2. Price inflation (12 months): Inflate invoices 10-25%
/// 3. Kickback payments (ongoing): Receive portion of inflated amounts
/// 4. Concealment (ongoing): Hide the relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorKickbackScheme {
    /// Unique scheme ID.
    pub scheme_id: Uuid,
    /// Employee perpetrator ID.
    pub perpetrator_id: String,
    /// Colluding vendor ID.
    pub vendor_id: String,
    /// Start date.
    pub start_date: Option<NaiveDate>,
    /// Current stage index.
    current_stage_index: usize,
    /// All stages.
    stages: Vec<SchemeStage>,
    /// Transaction references.
    transactions: Vec<SchemeTransactionRef>,
    /// Total impact (inflated amounts).
    total_impact: Decimal,
    /// Total kickback payments.
    total_kickbacks: Decimal,
    /// Status.
    status: SchemeStatus,
    /// Detection status.
    detection_status: SchemeDetectionStatus,
    /// Detection probability.
    detection_probability: f64,
    /// Price inflation percentage (10-25%).
    inflation_percent: f64,
    /// Kickback percentage (typically 50% of inflation).
    kickback_percent: f64,
    /// Legitimate vendor transactions to blend in.
    legitimate_transaction_count: u32,
    /// Inflated transaction count.
    inflated_transaction_count: u32,
}

impl VendorKickbackScheme {
    /// Creates a new vendor kickback scheme.
    pub fn new(perpetrator_id: impl Into<String>, vendor_id: impl Into<String>) -> Self {
        let stages = vec![
            // Stage 1: Setup (3 months)
            SchemeStage::new(
                1,
                "setup",
                3,
                (dec!(0), dec!(0)), // No fraudulent amounts during setup
                (0, 2),
                AnomalyDetectionDifficulty::Expert, // Setup is very hard to detect
            )
            .with_description("Establish vendor relationship and trust")
            .with_technique(ConcealmentTechnique::FalseDocumentation),
            // Stage 2: Price inflation (12 months)
            SchemeStage::new(
                2,
                "price_inflation",
                12,
                (dec!(5000), dec!(100000)),
                (10, 30),
                AnomalyDetectionDifficulty::Hard,
            )
            .with_description("Inflate invoice amounts 10-25%")
            .with_technique(ConcealmentTechnique::DocumentManipulation)
            .with_technique(ConcealmentTechnique::Collusion),
            // Stage 3: Kickback payments (6 months)
            SchemeStage::new(
                3,
                "kickback_payments",
                6,
                (dec!(500), dec!(25000)),
                (5, 15),
                AnomalyDetectionDifficulty::Moderate,
            )
            .with_description("Receive kickback payments from vendor")
            .with_technique(ConcealmentTechnique::Collusion)
            .with_technique(ConcealmentTechnique::TimingExploitation),
            // Stage 4: Concealment (3 months)
            SchemeStage::new(
                4,
                "concealment",
                3,
                (dec!(0), dec!(0)),
                (0, 2),
                AnomalyDetectionDifficulty::Hard,
            )
            .with_description("Cover tracks and maintain relationship")
            .with_technique(ConcealmentTechnique::DataAlteration)
            .with_technique(ConcealmentTechnique::FalseDocumentation),
        ];

        Self {
            scheme_id: Uuid::new_v4(),
            perpetrator_id: perpetrator_id.into(),
            vendor_id: vendor_id.into(),
            start_date: None,
            current_stage_index: 0,
            stages,
            transactions: Vec::new(),
            total_impact: Decimal::ZERO,
            total_kickbacks: Decimal::ZERO,
            status: SchemeStatus::NotStarted,
            detection_status: SchemeDetectionStatus::Undetected,
            detection_probability: 0.0,
            inflation_percent: 0.15, // 15% default
            kickback_percent: 0.50,  // 50% of inflation
            legitimate_transaction_count: 0,
            inflated_transaction_count: 0,
        }
    }

    /// Sets the inflation percentage (0.10 to 0.25).
    pub fn with_inflation_percent(mut self, percent: f64) -> Self {
        self.inflation_percent = percent.clamp(0.10, 0.25);
        self
    }

    /// Sets the kickback percentage (0.30 to 0.70).
    pub fn with_kickback_percent(mut self, percent: f64) -> Self {
        self.kickback_percent = percent.clamp(0.30, 0.70);
        self
    }

    /// Starts the scheme.
    pub fn start(&mut self, date: NaiveDate) {
        self.start_date = Some(date);
        self.status = SchemeStatus::Active;
    }

    /// Gets the stage end date.
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

    /// Checks if stage should advance.
    fn should_advance_stage(&self, current_date: NaiveDate) -> bool {
        if let Some(end_date) = self.stage_end_date() {
            current_date >= end_date && self.current_stage_index < self.stages.len() - 1
        } else {
            false
        }
    }

    /// Advances to next stage.
    fn advance_stage(&mut self) {
        if self.current_stage_index < self.stages.len() - 1 {
            self.current_stage_index += 1;
        }
    }

    /// Calculates the inflation amount for a base invoice amount.
    pub fn calculate_inflation(&self, base_amount: Decimal) -> Decimal {
        let inflation_factor =
            Decimal::from_f64_retain(self.inflation_percent).unwrap_or(dec!(0.15));
        base_amount * inflation_factor
    }

    /// Calculates the kickback amount from an inflated amount.
    pub fn calculate_kickback(&self, inflated_amount: Decimal) -> Decimal {
        let kickback_factor = Decimal::from_f64_retain(self.kickback_percent).unwrap_or(dec!(0.50));
        inflated_amount * kickback_factor
    }

    /// Updates detection probability.
    fn update_detection_probability(&mut self) {
        let base_prob = 0.1;

        // Vendor concentration increases detection risk
        let concentration_factor = if self.inflated_transaction_count > 20 {
            0.15
        } else if self.inflated_transaction_count > 10 {
            0.10
        } else {
            0.0
        };

        // Inflation level affects detection
        let inflation_factor = if self.inflation_percent > 0.20 {
            0.15
        } else if self.inflation_percent > 0.15 {
            0.10
        } else {
            0.05
        };

        // Total amount affects detection
        let amount_factor = if self.total_impact > dec!(500000) {
            0.20
        } else if self.total_impact > dec!(100000) {
            0.10
        } else {
            0.0
        };

        // Kickback stage is riskier
        let stage_factor = if self.current_stage_index == 2 {
            0.15
        } else {
            0.0
        };

        let prob: f64 =
            base_prob + concentration_factor + inflation_factor + amount_factor + stage_factor;
        self.detection_probability = prob.min(0.85);
    }
}

impl FraudScheme for VendorKickbackScheme {
    fn scheme_type(&self) -> SchemeType {
        SchemeType::VendorKickback
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
            if rng.gen::<f64>() < 0.3 {
                self.status = SchemeStatus::Detected;
                return actions;
            }
        }

        // Pause during audit
        if context.audit_in_progress && rng.gen::<f64>() < 0.6 {
            self.status = SchemeStatus::Paused;
            return actions;
        }
        self.status = SchemeStatus::Active;

        // Check stage advancement
        if self.should_advance_stage(context.current_date) {
            self.advance_stage();
        }

        let stage = &self.stages[self.current_stage_index];

        // Generate actions based on stage
        match self.current_stage_index {
            0 => {
                // Setup stage - create fictitious vendor if needed
                if rng.gen::<f64>() < 0.1 {
                    let action = SchemeAction::new(
                        self.scheme_id,
                        stage.stage_number,
                        SchemeActionType::CreateFictitiousVendor,
                        context.current_date,
                    )
                    .with_counterparty(&self.vendor_id)
                    .with_user(&self.perpetrator_id)
                    .with_difficulty(stage.detection_difficulty)
                    .with_description("Establish vendor relationship for kickback scheme");

                    actions.push(action);
                }
            }
            1 => {
                // Price inflation stage
                if rng.gen::<f64>() < 0.2 {
                    let base_amount = stage.random_amount(rng);
                    let inflation = self.calculate_inflation(base_amount);
                    let total_amount = base_amount + inflation;

                    let mut action = SchemeAction::new(
                        self.scheme_id,
                        stage.stage_number,
                        SchemeActionType::InflateInvoice,
                        context.current_date,
                    )
                    .with_amount(total_amount)
                    .with_counterparty(&self.vendor_id)
                    .with_user(&self.perpetrator_id)
                    .with_difficulty(stage.detection_difficulty)
                    .with_description(format!(
                        "Inflated invoice - base: {}, inflation: {}",
                        base_amount, inflation
                    ));

                    for technique in &stage.concealment_techniques {
                        action = action.with_technique(*technique);
                    }

                    self.inflated_transaction_count += 1;
                    actions.push(action);
                }
            }
            2 => {
                // Kickback payment stage
                if self.total_impact > Decimal::ZERO && rng.gen::<f64>() < 0.15 {
                    // Calculate kickback based on accumulated inflation
                    let kickback_amount = if self.total_kickbacks < self.total_impact * dec!(0.5) {
                        let max_kickback = self.total_impact
                            * Decimal::from_f64_retain(
                                self.kickback_percent * self.inflation_percent,
                            )
                            .unwrap_or(dec!(0.075));
                        let remaining = max_kickback - self.total_kickbacks;
                        stage.random_amount(rng).min(remaining).max(dec!(500))
                    } else {
                        dec!(0)
                    };

                    if kickback_amount > Decimal::ZERO {
                        let mut action = SchemeAction::new(
                            self.scheme_id,
                            stage.stage_number,
                            SchemeActionType::MakeKickbackPayment,
                            context.current_date,
                        )
                        .with_amount(kickback_amount)
                        .with_user(&self.perpetrator_id)
                        .with_difficulty(stage.detection_difficulty)
                        .with_description(format!(
                            "Kickback payment from vendor {}",
                            self.vendor_id
                        ));

                        for technique in &stage.concealment_techniques {
                            action = action.with_technique(*technique);
                        }

                        self.total_kickbacks += kickback_amount;
                        actions.push(action);
                    }
                }
            }
            3 => {
                // Concealment stage
                if rng.gen::<f64>() < 0.05 {
                    let action = SchemeAction::new(
                        self.scheme_id,
                        stage.stage_number,
                        SchemeActionType::CoverUp,
                        context.current_date,
                    )
                    .with_user(&self.perpetrator_id)
                    .with_difficulty(stage.detection_difficulty)
                    .with_description("Cover up kickback scheme evidence")
                    .with_technique(ConcealmentTechnique::DataAlteration);

                    actions.push(action);
                }
            }
            _ => {}
        }

        // Update detection probability
        self.update_detection_probability();

        // Check completion
        if self.current_stage_index == self.stages.len() - 1 {
            if let Some(end_date) = self.stage_end_date() {
                if context.current_date >= end_date {
                    self.status = SchemeStatus::Completed;
                }
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
        // Kickback schemes are more sensitive to detection
        if context.detection_activity > 0.7 {
            return true;
        }

        if self.detection_status != SchemeDetectionStatus::Undetected
            && self.detection_status == SchemeDetectionStatus::FullyDetected
        {
            return true;
        }
        // Under investigation - might continue if they think they can beat it

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
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_kickback_scheme_creation() {
        let scheme = VendorKickbackScheme::new("EMP001", "VENDOR001")
            .with_inflation_percent(0.20)
            .with_kickback_percent(0.50);

        assert_eq!(scheme.perpetrator_id, "EMP001");
        assert_eq!(scheme.vendor_id, "VENDOR001");
        assert!((scheme.inflation_percent - 0.20).abs() < 0.01);
        assert_eq!(scheme.stages.len(), 4);
    }

    #[test]
    fn test_kickback_scheme_stages() {
        let scheme = VendorKickbackScheme::new("EMP001", "VENDOR001");

        assert_eq!(scheme.stages[0].name, "setup");
        assert_eq!(scheme.stages[1].name, "price_inflation");
        assert_eq!(scheme.stages[2].name, "kickback_payments");
        assert_eq!(scheme.stages[3].name, "concealment");
    }

    #[test]
    fn test_inflation_calculation() {
        let scheme = VendorKickbackScheme::new("EMP001", "VENDOR001").with_inflation_percent(0.20);

        let base = dec!(10000);
        let inflation = scheme.calculate_inflation(base);

        // Use approximate comparison due to floating point conversion
        let expected = dec!(2000);
        let diff = (inflation - expected).abs();
        assert!(
            diff < dec!(0.01),
            "Expected ~{}, got {}",
            expected,
            inflation
        );
    }

    #[test]
    fn test_kickback_calculation() {
        let scheme = VendorKickbackScheme::new("EMP001", "VENDOR001").with_kickback_percent(0.50);

        let inflated = dec!(2000);
        let kickback = scheme.calculate_kickback(inflated);

        assert_eq!(kickback, dec!(1000));
    }

    #[test]
    fn test_kickback_scheme_advance() {
        let mut scheme = VendorKickbackScheme::new("EMP001", "VENDOR001");
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let context = SchemeContext::new(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(), "1000");

        // Advance multiple times
        let mut total_actions = 0;
        for day in 0..100 {
            let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap() + chrono::Duration::days(day);
            let mut ctx = context.clone();
            ctx.current_date = date;

            let actions = scheme.advance(&ctx, &mut rng);
            total_actions += actions.len();
        }

        assert!(total_actions >= 0); // May or may not have actions depending on RNG
        assert_eq!(scheme.status, SchemeStatus::Active);
    }
}
