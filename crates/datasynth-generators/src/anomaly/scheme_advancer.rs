//! Scheme advancer for managing multiple fraud schemes.
//!
//! The SchemeAdvancer coordinates the lifecycle of multiple fraud schemes,
//! handling scheme creation, advancement, and completion.

use chrono::NaiveDate;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use datasynth_core::models::{SchemeDetectionStatus, SchemeType};

use super::schemes::{
    FraudScheme, GradualEmbezzlementScheme, RevenueManipulationScheme, SchemeAction, SchemeContext,
    SchemeStatus, VendorKickbackScheme,
};

/// Configuration for scheme generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemeAdvancerConfig {
    /// Probability of starting an embezzlement scheme per period.
    pub embezzlement_probability: f64,
    /// Probability of starting a revenue manipulation scheme per period.
    pub revenue_manipulation_probability: f64,
    /// Probability of starting a kickback scheme per period.
    pub kickback_probability: f64,
    /// Maximum number of concurrent schemes.
    pub max_concurrent_schemes: usize,
    /// Whether to allow the same perpetrator in multiple schemes.
    pub allow_repeat_perpetrators: bool,
    /// Random seed for reproducibility.
    pub seed: u64,
}

impl Default for SchemeAdvancerConfig {
    fn default() -> Self {
        Self {
            embezzlement_probability: 0.02,
            revenue_manipulation_probability: 0.01,
            kickback_probability: 0.01,
            max_concurrent_schemes: 5,
            allow_repeat_perpetrators: false,
            seed: 42,
        }
    }
}

/// Summary of a completed scheme.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedScheme {
    /// Scheme ID.
    pub scheme_id: Uuid,
    /// Scheme type.
    pub scheme_type: SchemeType,
    /// Perpetrator ID.
    pub perpetrator_id: String,
    /// Start date.
    pub start_date: Option<NaiveDate>,
    /// End date.
    pub end_date: NaiveDate,
    /// Final status.
    pub final_status: SchemeStatus,
    /// Detection status.
    pub detection_status: SchemeDetectionStatus,
    /// Total financial impact.
    pub total_impact: Decimal,
    /// Number of stages completed.
    pub stages_completed: u32,
    /// Total transactions.
    pub transaction_count: usize,
}

/// Label for an anomaly that's part of a multi-stage scheme.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiStageAnomalyLabel {
    /// Anomaly ID.
    pub anomaly_id: String,
    /// Scheme ID.
    pub scheme_id: Uuid,
    /// Scheme type.
    pub scheme_type: SchemeType,
    /// Stage number within scheme.
    pub stage_number: u32,
    /// Stage name.
    pub stage_name: String,
    /// Total stages in scheme.
    pub total_stages: u32,
    /// Perpetrator ID.
    pub perpetrator_id: String,
    /// Whether scheme was ultimately detected.
    pub scheme_detected: bool,
}

/// Manages the lifecycle of multiple fraud schemes.
pub struct SchemeAdvancer {
    config: SchemeAdvancerConfig,
    rng: ChaCha8Rng,
    /// Active schemes.
    active_schemes: Vec<Box<dyn FraudScheme>>,
    /// Completed schemes.
    completed_schemes: Vec<CompletedScheme>,
    /// Users who are currently perpetrators.
    active_perpetrators: Vec<String>,
    /// Vendors involved in active schemes.
    active_vendors: Vec<String>,
    /// Multi-stage labels generated.
    labels: Vec<MultiStageAnomalyLabel>,
}

impl SchemeAdvancer {
    /// Creates a new scheme advancer.
    pub fn new(config: SchemeAdvancerConfig) -> Self {
        let rng = ChaCha8Rng::seed_from_u64(config.seed);
        Self {
            config,
            rng,
            active_schemes: Vec::new(),
            completed_schemes: Vec::new(),
            active_perpetrators: Vec::new(),
            active_vendors: Vec::new(),
            labels: Vec::new(),
        }
    }

    /// Potentially starts a new scheme based on probabilities.
    pub fn maybe_start_scheme(&mut self, context: &SchemeContext) -> Option<Uuid> {
        // Check if we can add more schemes
        if self.active_schemes.len() >= self.config.max_concurrent_schemes {
            return None;
        }

        // Check available perpetrators
        let available_users: Vec<_> = if self.config.allow_repeat_perpetrators {
            context.available_users.clone()
        } else {
            context
                .available_users
                .iter()
                .filter(|u| !self.active_perpetrators.contains(u))
                .cloned()
                .collect()
        };

        if available_users.is_empty() {
            return None;
        }

        // Determine which scheme type to start (if any)
        let r = self.rng.gen::<f64>();
        let total_prob = self.config.embezzlement_probability
            + self.config.revenue_manipulation_probability
            + self.config.kickback_probability;

        if r > total_prob {
            return None;
        }

        let normalized_r = r / total_prob;
        let embezzlement_threshold = self.config.embezzlement_probability / total_prob;
        let revenue_threshold =
            embezzlement_threshold + self.config.revenue_manipulation_probability / total_prob;

        let user_idx = self.rng.gen_range(0..available_users.len());
        let perpetrator = available_users[user_idx].clone();

        let scheme: Box<dyn FraudScheme> = if normalized_r < embezzlement_threshold {
            // Start embezzlement scheme
            let scheme = GradualEmbezzlementScheme::new(&perpetrator)
                .with_accounts(context.available_accounts.clone());
            Box::new(scheme)
        } else if normalized_r < revenue_threshold {
            // Start revenue manipulation scheme
            let scheme = RevenueManipulationScheme::new(&perpetrator);
            Box::new(scheme)
        } else {
            // Start kickback scheme - need a vendor
            if context.available_counterparties.is_empty() {
                return None;
            }

            let available_vendors: Vec<_> = context
                .available_counterparties
                .iter()
                .filter(|v| !self.active_vendors.contains(v))
                .cloned()
                .collect();

            if available_vendors.is_empty() {
                return None;
            }

            let vendor_idx = self.rng.gen_range(0..available_vendors.len());
            let vendor = available_vendors[vendor_idx].clone();

            let inflation = 0.10 + self.rng.gen::<f64>() * 0.15; // 10-25%
            let scheme =
                VendorKickbackScheme::new(&perpetrator, &vendor).with_inflation_percent(inflation);

            self.active_vendors.push(vendor);
            Box::new(scheme)
        };

        let scheme_id = scheme.scheme_id();
        self.active_perpetrators.push(perpetrator);
        self.active_schemes.push(scheme);

        Some(scheme_id)
    }

    /// Advances all active schemes and returns actions to execute.
    pub fn advance_all(&mut self, context: &SchemeContext) -> Vec<SchemeAction> {
        let mut all_actions = Vec::new();
        let mut schemes_to_complete = Vec::new();

        for (idx, scheme) in self.active_schemes.iter_mut().enumerate() {
            // Create a local RNG for each scheme to ensure determinism
            let mut scheme_rng = ChaCha8Rng::seed_from_u64(
                self.config
                    .seed
                    .wrapping_add(scheme.scheme_id().as_u128() as u64),
            );

            let actions = scheme.advance(context, &mut scheme_rng);
            all_actions.extend(actions);

            // Check if scheme is done
            if matches!(
                scheme.status(),
                SchemeStatus::Completed | SchemeStatus::Terminated | SchemeStatus::Detected
            ) {
                schemes_to_complete.push(idx);
            }
        }

        // Complete finished schemes (iterate in reverse to maintain indices)
        for idx in schemes_to_complete.into_iter().rev() {
            let scheme = self.active_schemes.remove(idx);
            let completed = CompletedScheme {
                scheme_id: scheme.scheme_id(),
                scheme_type: scheme.scheme_type(),
                perpetrator_id: scheme.perpetrator_id().to_string(),
                start_date: scheme.start_date(),
                end_date: context.current_date,
                final_status: scheme.status(),
                detection_status: scheme.detection_status(),
                total_impact: scheme.total_impact(),
                stages_completed: scheme.current_stage_number(),
                transaction_count: scheme.transaction_refs().len(),
            };

            // Remove perpetrator from active list
            self.active_perpetrators
                .retain(|p| p != scheme.perpetrator_id());

            self.completed_schemes.push(completed);
        }

        all_actions
    }

    /// Records a label for a scheme action.
    pub fn record_label(&mut self, anomaly_id: impl Into<String>, action: &SchemeAction) {
        if let Some(scheme) = self
            .active_schemes
            .iter()
            .find(|s| s.scheme_id() == action.scheme_id)
        {
            let label = MultiStageAnomalyLabel {
                anomaly_id: anomaly_id.into(),
                scheme_id: scheme.scheme_id(),
                scheme_type: scheme.scheme_type(),
                stage_number: action.stage,
                stage_name: scheme.current_stage().name.clone(),
                total_stages: scheme.stages().len() as u32,
                perpetrator_id: scheme.perpetrator_id().to_string(),
                scheme_detected: scheme.detection_status() != SchemeDetectionStatus::Undetected,
            };
            self.labels.push(label);
        }
    }

    /// Returns all generated labels.
    pub fn get_labels(&self) -> &[MultiStageAnomalyLabel] {
        &self.labels
    }

    /// Returns completed schemes.
    pub fn get_completed_schemes(&self) -> &[CompletedScheme] {
        &self.completed_schemes
    }

    /// Returns the number of active schemes.
    pub fn active_scheme_count(&self) -> usize {
        self.active_schemes.len()
    }

    /// Returns the number of completed schemes.
    pub fn completed_scheme_count(&self) -> usize {
        self.completed_schemes.len()
    }

    /// Returns active schemes summary.
    pub fn active_schemes_summary(&self) -> Vec<(Uuid, SchemeType, SchemeStatus)> {
        self.active_schemes
            .iter()
            .map(|s| (s.scheme_id(), s.scheme_type(), s.status()))
            .collect()
    }

    /// Gets a specific scheme by ID.
    pub fn get_scheme(&self, scheme_id: Uuid) -> Option<&dyn FraudScheme> {
        self.active_schemes
            .iter()
            .find(|s| s.scheme_id() == scheme_id)
            .map(|s| s.as_ref())
    }

    /// Resets the advancer state.
    pub fn reset(&mut self) {
        self.active_schemes.clear();
        self.completed_schemes.clear();
        self.active_perpetrators.clear();
        self.active_vendors.clear();
        self.labels.clear();
        self.rng = ChaCha8Rng::seed_from_u64(self.config.seed);
    }

    /// Returns statistics about schemes.
    pub fn get_statistics(&self) -> SchemeStatistics {
        let total_impact: Decimal = self
            .completed_schemes
            .iter()
            .map(|s| s.total_impact)
            .sum::<Decimal>()
            + self
                .active_schemes
                .iter()
                .map(|s| s.total_impact())
                .sum::<Decimal>();

        let detected_count = self
            .completed_schemes
            .iter()
            .filter(|s| s.detection_status != SchemeDetectionStatus::Undetected)
            .count();

        let by_type = |t: SchemeType| {
            self.completed_schemes
                .iter()
                .filter(|s| s.scheme_type == t)
                .count()
                + self
                    .active_schemes
                    .iter()
                    .filter(|s| s.scheme_type() == t)
                    .count()
        };

        SchemeStatistics {
            total_schemes: self.active_schemes.len() + self.completed_schemes.len(),
            active_schemes: self.active_schemes.len(),
            completed_schemes: self.completed_schemes.len(),
            detected_schemes: detected_count,
            total_impact,
            embezzlement_count: by_type(SchemeType::GradualEmbezzlement),
            revenue_manipulation_count: by_type(SchemeType::RevenueManipulation),
            kickback_count: by_type(SchemeType::VendorKickback),
        }
    }
}

/// Statistics about fraud schemes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemeStatistics {
    /// Total number of schemes (active + completed).
    pub total_schemes: usize,
    /// Number of active schemes.
    pub active_schemes: usize,
    /// Number of completed schemes.
    pub completed_schemes: usize,
    /// Number of detected schemes.
    pub detected_schemes: usize,
    /// Total financial impact.
    pub total_impact: Decimal,
    /// Number of embezzlement schemes.
    pub embezzlement_count: usize,
    /// Number of revenue manipulation schemes.
    pub revenue_manipulation_count: usize,
    /// Number of kickback schemes.
    pub kickback_count: usize,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_scheme_advancer_creation() {
        let advancer = SchemeAdvancer::new(SchemeAdvancerConfig::default());
        assert_eq!(advancer.active_scheme_count(), 0);
        assert_eq!(advancer.completed_scheme_count(), 0);
    }

    #[test]
    fn test_scheme_advancer_start_scheme() {
        let mut advancer = SchemeAdvancer::new(SchemeAdvancerConfig {
            embezzlement_probability: 1.0, // Always start
            ..Default::default()
        });

        let context = SchemeContext::new(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(), "1000")
            .with_users(vec!["USER001".to_string(), "USER002".to_string()])
            .with_accounts(vec!["5000".to_string()]);

        let scheme_id = advancer.maybe_start_scheme(&context);
        assert!(scheme_id.is_some());
        assert_eq!(advancer.active_scheme_count(), 1);
    }

    #[test]
    fn test_scheme_advancer_max_concurrent() {
        let mut advancer = SchemeAdvancer::new(SchemeAdvancerConfig {
            embezzlement_probability: 1.0,
            max_concurrent_schemes: 2,
            ..Default::default()
        });

        let context = SchemeContext::new(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(), "1000")
            .with_users(vec![
                "USER001".to_string(),
                "USER002".to_string(),
                "USER003".to_string(),
            ])
            .with_accounts(vec!["5000".to_string()]);

        // Start schemes up to max
        advancer.maybe_start_scheme(&context);
        advancer.maybe_start_scheme(&context);
        let third = advancer.maybe_start_scheme(&context);

        assert_eq!(advancer.active_scheme_count(), 2);
        assert!(third.is_none()); // Should not start third due to max
    }

    #[test]
    fn test_scheme_advancer_advance_all() {
        let mut advancer = SchemeAdvancer::new(SchemeAdvancerConfig {
            embezzlement_probability: 1.0,
            ..Default::default()
        });

        let context = SchemeContext::new(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(), "1000")
            .with_users(vec!["USER001".to_string()])
            .with_accounts(vec!["5000".to_string()]);

        advancer.maybe_start_scheme(&context);

        // Advance for several days
        for day in 0..30 {
            let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap() + chrono::Duration::days(day);
            let mut ctx = context.clone();
            ctx.current_date = date;

            let _actions = advancer.advance_all(&ctx);
        }

        assert_eq!(advancer.active_scheme_count(), 1);
    }

    #[test]
    fn test_scheme_advancer_statistics() {
        let mut advancer = SchemeAdvancer::new(SchemeAdvancerConfig {
            embezzlement_probability: 1.0,
            ..Default::default()
        });

        let context = SchemeContext::new(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(), "1000")
            .with_users(vec!["USER001".to_string()])
            .with_accounts(vec!["5000".to_string()]);

        advancer.maybe_start_scheme(&context);

        let stats = advancer.get_statistics();
        assert_eq!(stats.total_schemes, 1);
        assert_eq!(stats.active_schemes, 1);
        assert_eq!(stats.embezzlement_count, 1);
    }
}
