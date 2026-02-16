//! Treasury Anomaly Injector.
//!
//! Injects labeled anomalies into treasury data (cash positions, forecasts,
//! hedge relationships, debt covenants) for ML ground-truth generation.
//! Each injected anomaly produces a [`TreasuryAnomalyLabel`] that records the
//! anomaly type, severity, affected document, and original vs. anomalous values.

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use datasynth_core::models::{CashPosition, DebtCovenant, HedgeRelationship};

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Types of treasury anomalies that can be injected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TreasuryAnomalyType {
    /// Actual cash flow deviates significantly from forecast.
    CashForecastMiss,
    /// Covenant headroom trending toward zero (potential breach).
    CovenantBreachRisk,
    /// Hedge effectiveness ratio falls outside 80-125% corridor.
    HedgeIneffectiveness,
    /// Unusually large or unexpected cash movement.
    UnusualCashMovement,
    /// Available cash drops below minimum balance policy.
    LiquidityCrisis,
    /// Excessive hedging exposure to a single counterparty.
    CounterpartyConcentration,
}

/// Severity of the anomaly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TreasuryAnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

// ---------------------------------------------------------------------------
// Label
// ---------------------------------------------------------------------------

/// A labeled treasury anomaly for ground truth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryAnomalyLabel {
    /// Unique anomaly label identifier.
    pub id: String,
    /// Type of the anomaly.
    pub anomaly_type: TreasuryAnomalyType,
    /// Severity of the anomaly.
    pub severity: TreasuryAnomalySeverity,
    /// Kind of document affected: `"cash_position"`, `"hedge_relationship"`,
    /// `"debt_covenant"`, `"cash_forecast"`.
    pub document_type: String,
    /// ID of the affected record.
    pub document_id: String,
    /// Human-readable description of the anomaly.
    pub description: String,
    /// What the correct value should be (if applicable).
    pub original_value: Option<String>,
    /// What was injected (if applicable).
    pub anomalous_value: Option<String>,
}

// ---------------------------------------------------------------------------
// Injector
// ---------------------------------------------------------------------------

/// Injects treasury anomalies into generated data.
pub struct TreasuryAnomalyInjector {
    rng: ChaCha8Rng,
    anomaly_rate: f64,
    counter: u64,
}

impl TreasuryAnomalyInjector {
    /// Creates a new treasury anomaly injector.
    pub fn new(seed: u64, anomaly_rate: f64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            anomaly_rate: anomaly_rate.clamp(0.0, 1.0),
            counter: 0,
        }
    }

    /// Inject anomalies into cash positions. Modifies positions in-place and returns labels.
    ///
    /// Anomaly types:
    /// - **UnusualCashMovement** (50%): Inject a large unexpected outflow.
    /// - **LiquidityCrisis** (50%): Reduce available balance below minimum policy.
    pub fn inject_into_cash_positions(
        &mut self,
        positions: &mut [CashPosition],
        minimum_balance: Decimal,
    ) -> Vec<TreasuryAnomalyLabel> {
        let mut labels = Vec::new();

        for pos in positions.iter_mut() {
            if !self.should_inject() {
                continue;
            }

            let roll: f64 = self.rng.gen();
            if roll < 0.50 {
                labels.push(self.inject_unusual_cash_movement(pos));
            } else {
                labels.push(self.inject_liquidity_crisis(pos, minimum_balance));
            }
        }

        labels
    }

    /// Inject anomalies into hedge relationships. Modifies relationships in-place
    /// and returns labels.
    ///
    /// Anomaly type: **HedgeIneffectiveness** — push effectiveness ratio outside
    /// the 80-125% corridor.
    pub fn inject_into_hedge_relationships(
        &mut self,
        relationships: &mut [HedgeRelationship],
    ) -> Vec<TreasuryAnomalyLabel> {
        let mut labels = Vec::new();

        for rel in relationships.iter_mut() {
            if !self.should_inject() {
                continue;
            }
            labels.push(self.inject_hedge_ineffectiveness(rel));
        }

        labels
    }

    /// Inject anomalies into debt covenants. Modifies covenants in-place and returns labels.
    ///
    /// Anomaly type: **CovenantBreachRisk** — push actual value toward or past threshold.
    pub fn inject_into_debt_covenants(
        &mut self,
        covenants: &mut [DebtCovenant],
    ) -> Vec<TreasuryAnomalyLabel> {
        let mut labels = Vec::new();

        for cov in covenants.iter_mut() {
            if !self.should_inject() {
                continue;
            }
            labels.push(self.inject_covenant_breach_risk(cov));
        }

        labels
    }

    // -----------------------------------------------------------------------
    // Private injection methods
    // -----------------------------------------------------------------------

    fn inject_unusual_cash_movement(&mut self, pos: &mut CashPosition) -> TreasuryAnomalyLabel {
        let original_outflows = pos.outflows;
        // Inject a large unexpected outflow (50-200% of current closing balance)
        let spike_pct = Decimal::try_from(self.rng.gen_range(0.50f64..2.00f64))
            .unwrap_or(dec!(1.0));
        let spike = (pos.closing_balance.abs() * spike_pct).round_dp(2);
        pos.outflows += spike;
        let new_closing = (pos.opening_balance + pos.inflows - pos.outflows).round_dp(2);
        pos.closing_balance = new_closing;
        pos.available_balance = new_closing.max(Decimal::ZERO);

        self.counter += 1;
        TreasuryAnomalyLabel {
            id: format!("TANOM-{:06}", self.counter),
            anomaly_type: TreasuryAnomalyType::UnusualCashMovement,
            severity: if spike > pos.opening_balance {
                TreasuryAnomalySeverity::Critical
            } else {
                TreasuryAnomalySeverity::High
            },
            document_type: "cash_position".to_string(),
            document_id: pos.id.clone(),
            description: format!(
                "Unusual cash outflow of {} on {}",
                spike, pos.date
            ),
            original_value: Some(original_outflows.to_string()),
            anomalous_value: Some(pos.outflows.to_string()),
        }
    }

    fn inject_liquidity_crisis(
        &mut self,
        pos: &mut CashPosition,
        minimum_balance: Decimal,
    ) -> TreasuryAnomalyLabel {
        let original_available = pos.available_balance;
        // Drop available balance below the minimum policy
        let target_pct = Decimal::try_from(self.rng.gen_range(0.10f64..0.80f64))
            .unwrap_or(dec!(0.50));
        pos.available_balance = (minimum_balance * target_pct).round_dp(2);

        self.counter += 1;
        TreasuryAnomalyLabel {
            id: format!("TANOM-{:06}", self.counter),
            anomaly_type: TreasuryAnomalyType::LiquidityCrisis,
            severity: if pos.available_balance < minimum_balance * dec!(0.25) {
                TreasuryAnomalySeverity::Critical
            } else {
                TreasuryAnomalySeverity::Medium
            },
            document_type: "cash_position".to_string(),
            document_id: pos.id.clone(),
            description: format!(
                "Available balance {} below minimum policy {} on {}",
                pos.available_balance, minimum_balance, pos.date
            ),
            original_value: Some(original_available.to_string()),
            anomalous_value: Some(pos.available_balance.to_string()),
        }
    }

    fn inject_hedge_ineffectiveness(
        &mut self,
        rel: &mut HedgeRelationship,
    ) -> TreasuryAnomalyLabel {
        let original_ratio = rel.effectiveness_ratio;
        // Push ratio outside the 80-125% corridor
        let new_ratio = if self.rng.gen_bool(0.5) {
            // Below 80%
            Decimal::try_from(self.rng.gen_range(0.50f64..0.79f64))
                .unwrap_or(dec!(0.65))
        } else {
            // Above 125%
            Decimal::try_from(self.rng.gen_range(1.26f64..1.60f64))
                .unwrap_or(dec!(1.40))
        };
        rel.effectiveness_ratio = new_ratio.round_dp(4);
        rel.update_effectiveness();

        self.counter += 1;
        TreasuryAnomalyLabel {
            id: format!("TANOM-{:06}", self.counter),
            anomaly_type: TreasuryAnomalyType::HedgeIneffectiveness,
            severity: TreasuryAnomalySeverity::High,
            document_type: "hedge_relationship".to_string(),
            document_id: rel.id.clone(),
            description: format!(
                "Hedge effectiveness ratio {} outside 80-125% corridor",
                rel.effectiveness_ratio
            ),
            original_value: Some(original_ratio.to_string()),
            anomalous_value: Some(rel.effectiveness_ratio.to_string()),
        }
    }

    fn inject_covenant_breach_risk(&mut self, cov: &mut DebtCovenant) -> TreasuryAnomalyLabel {
        let original_value = cov.actual_value;
        // Push actual value past the threshold
        let breach_factor = Decimal::try_from(self.rng.gen_range(1.05f64..1.25f64))
            .unwrap_or(dec!(1.10));
        cov.actual_value = (cov.threshold * breach_factor).round_dp(2);
        cov.update_compliance();

        self.counter += 1;
        TreasuryAnomalyLabel {
            id: format!("TANOM-{:06}", self.counter),
            anomaly_type: TreasuryAnomalyType::CovenantBreachRisk,
            severity: if cov.headroom.abs() > dec!(1.0) {
                TreasuryAnomalySeverity::Critical
            } else {
                TreasuryAnomalySeverity::High
            },
            document_type: "debt_covenant".to_string(),
            document_id: cov.id.clone(),
            description: format!(
                "Covenant {:?} actual value {} vs threshold {} (headroom: {})",
                cov.covenant_type, cov.actual_value, cov.threshold, cov.headroom
            ),
            original_value: Some(original_value.to_string()),
            anomalous_value: Some(cov.actual_value.to_string()),
        }
    }

    fn should_inject(&mut self) -> bool {
        self.rng.gen_bool(self.anomaly_rate)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use datasynth_core::models::{
        CashPosition, CovenantType, DebtCovenant, EffectivenessMethod, Frequency,
        HedgeRelationship, HedgeType, HedgedItemType,
    };

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn test_inject_unusual_cash_movement() {
        let mut injector = TreasuryAnomalyInjector::new(42, 1.0); // 100% rate for testing
        let mut positions = vec![CashPosition::new(
            "CP-001",
            "C001",
            "BA-001",
            "USD",
            d("2025-01-15"),
            dec!(100000),
            dec!(5000),
            dec!(2000),
        )];

        let labels = injector.inject_into_cash_positions(&mut positions, dec!(50000));

        assert_eq!(labels.len(), 1);
        assert!(
            labels[0].anomaly_type == TreasuryAnomalyType::UnusualCashMovement
                || labels[0].anomaly_type == TreasuryAnomalyType::LiquidityCrisis
        );
        assert!(labels[0].original_value.is_some());
        assert!(labels[0].anomalous_value.is_some());
    }

    #[test]
    fn test_inject_hedge_ineffectiveness() {
        let mut injector = TreasuryAnomalyInjector::new(42, 1.0);
        let mut relationships = vec![HedgeRelationship::new(
            "HR-001",
            HedgedItemType::ForecastedTransaction,
            "EUR receivables",
            "HI-001",
            HedgeType::CashFlowHedge,
            d("2025-01-01"),
            EffectivenessMethod::Regression,
            dec!(0.95), // starts effective
        )];

        let labels = injector.inject_into_hedge_relationships(&mut relationships);

        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0].anomaly_type, TreasuryAnomalyType::HedgeIneffectiveness);
        // Relationship should now be marked as ineffective
        assert!(!relationships[0].is_effective);
    }

    #[test]
    fn test_inject_covenant_breach() {
        let mut injector = TreasuryAnomalyInjector::new(42, 1.0);
        let mut covenants = vec![DebtCovenant::new(
            "COV-001",
            CovenantType::DebtToEbitda,
            dec!(3.5),
            Frequency::Quarterly,
            dec!(2.5), // starts compliant
            d("2025-03-31"),
        )];

        let labels = injector.inject_into_debt_covenants(&mut covenants);

        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0].anomaly_type, TreasuryAnomalyType::CovenantBreachRisk);
        // For DebtToEbitda (max covenant), injected value should exceed threshold
        // The breach_factor pushes actual_value = threshold * 1.05..1.25
        // So it will be above threshold, making it non-compliant
        assert!(!covenants[0].is_compliant);
        assert!(covenants[0].headroom < Decimal::ZERO);
    }

    #[test]
    fn test_no_injection_at_zero_rate() {
        let mut injector = TreasuryAnomalyInjector::new(42, 0.0);
        let mut positions = vec![CashPosition::new(
            "CP-001",
            "C001",
            "BA-001",
            "USD",
            d("2025-01-15"),
            dec!(100000),
            dec!(5000),
            dec!(2000),
        )];

        let labels = injector.inject_into_cash_positions(&mut positions, dec!(50000));
        assert!(labels.is_empty());
    }

    #[test]
    fn test_anomaly_label_serde_roundtrip() {
        let label = TreasuryAnomalyLabel {
            id: "TANOM-001".to_string(),
            anomaly_type: TreasuryAnomalyType::CashForecastMiss,
            severity: TreasuryAnomalySeverity::Medium,
            document_type: "cash_forecast".to_string(),
            document_id: "CF-001".to_string(),
            description: "Forecast missed by 25%".to_string(),
            original_value: Some("100000".to_string()),
            anomalous_value: Some("75000".to_string()),
        };

        let json = serde_json::to_string(&label).unwrap();
        let deserialized: TreasuryAnomalyLabel = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.anomaly_type, TreasuryAnomalyType::CashForecastMiss);
        assert_eq!(deserialized.document_id, "CF-001");
    }
}
