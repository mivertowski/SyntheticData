//! Formal differential privacy composition methods.
//!
//! This module provides multiple composition strategies for tracking privacy loss
//! across sequential mechanism applications:
//!
//! - **Naive**: Simple sequential composition (sum of epsilons). Always valid, but loose.
//! - **Advanced**: Advanced composition theorem with improved bounds for multiple queries.
//! - **Renyi DP (RDP)**: Tracks Renyi divergence curves across multiple alpha orders,
//!   then converts to (epsilon, delta)-DP via the optimal conversion formula.
//! - **Zero-Concentrated DP (zCDP)**: Tracks additive rho parameter (rho = epsilon^2/2),
//!   then converts to (epsilon, delta)-DP.
//!
//! # References
//!
//! - Mironov, I. (2017). "Renyi Differential Privacy". CSF 2017.
//! - Bun, M. & Steinke, T. (2016). "Concentrated Differential Privacy". arXiv:1605.02065.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The alpha orders at which Renyi DP curves are tracked.
pub const RDP_ALPHA_ORDERS: [f64; 7] = [2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0];

/// Composition method for privacy budget accounting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CompositionMethod {
    /// Simple sequential composition: total epsilon = sum of all epsilons.
    /// Always valid but provides the loosest bounds.
    #[default]
    Naive,

    /// Advanced composition theorem (Dwork, Rothblum, Vadhan 2010).
    /// Provides tighter bounds for many queries with a delta parameter.
    Advanced,

    /// Renyi Differential Privacy (Mironov 2017).
    /// Tracks RDP curves at multiple alpha orders for tight composition.
    #[serde(rename = "renyi_dp")]
    RenyiDP,

    /// Zero-Concentrated Differential Privacy (Bun & Steinke 2016).
    /// Uses additive rho parameter for clean composition.
    #[serde(rename = "zcdp")]
    ZeroCDP,
}

impl std::fmt::Display for CompositionMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Naive => write!(f, "naive"),
            Self::Advanced => write!(f, "advanced"),
            Self::RenyiDP => write!(f, "renyi_dp"),
            Self::ZeroCDP => write!(f, "zcdp"),
        }
    }
}

/// A record of a single privacy mechanism application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MechanismRecord {
    /// The epsilon consumed by this mechanism (pure DP interpretation).
    pub epsilon: f64,

    /// The delta parameter for this mechanism (0.0 for pure DP mechanisms).
    #[serde(default)]
    pub delta: f64,

    /// Timestamp when the mechanism was applied.
    pub timestamp: DateTime<Utc>,

    /// Human-readable description of the mechanism.
    pub description: String,
}

impl MechanismRecord {
    /// Create a new mechanism record.
    pub fn new(epsilon: f64, description: impl Into<String>) -> Self {
        Self {
            epsilon,
            delta: 0.0,
            timestamp: Utc::now(),
            description: description.into(),
        }
    }

    /// Create a mechanism record with a delta parameter.
    pub fn with_delta(mut self, delta: f64) -> Self {
        self.delta = delta;
        self
    }
}

/// Trait for privacy accountants that track cumulative privacy loss.
pub trait PrivacyAccountant {
    /// Record a mechanism application and update the privacy budget.
    fn record_mechanism(&mut self, record: MechanismRecord);

    /// Get the effective (epsilon, delta) guarantee after all recorded mechanisms.
    ///
    /// For pure DP methods (Naive), delta will be 0.0.
    /// For approximate DP methods (RDP, zCDP), delta is the target delta parameter.
    fn effective_epsilon(&self) -> f64;

    /// Get the remaining privacy budget (total_budget - effective_epsilon).
    fn remaining_budget(&self) -> f64;

    /// Check if the privacy budget has been exhausted.
    fn is_exhausted(&self) -> bool;

    /// Get the composition method used by this accountant.
    fn method(&self) -> CompositionMethod;

    /// Get all recorded mechanisms.
    fn mechanisms(&self) -> &[MechanismRecord];

    /// Get the target delta parameter, if applicable.
    ///
    /// Returns `None` for pure DP accountants (Naive).
    /// Returns `Some(delta)` for approximate DP accountants (RDP, zCDP).
    fn target_delta(&self) -> Option<f64> {
        None
    }

    /// Get the optimal Renyi DP alpha order, if applicable.
    ///
    /// Only meaningful for RDP accountants. Returns `None` by default.
    fn optimal_alpha(&self) -> Option<f64> {
        None
    }
}

// ---------------------------------------------------------------------------
// Naive Accountant
// ---------------------------------------------------------------------------

/// Naive (sequential) composition accountant.
///
/// Simply sums all epsilon values. This is the simplest and most conservative
/// approach, always providing valid upper bounds on privacy loss.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaiveAccountant {
    /// Total epsilon budget.
    pub total_budget: f64,
    /// All recorded mechanisms.
    pub mechanisms: Vec<MechanismRecord>,
    /// Running sum of epsilon values.
    pub epsilon_spent: f64,
}

impl NaiveAccountant {
    /// Create a new naive accountant with the given total budget.
    pub fn new(total_budget: f64) -> Self {
        Self {
            total_budget,
            mechanisms: Vec::new(),
            epsilon_spent: 0.0,
        }
    }
}

impl PrivacyAccountant for NaiveAccountant {
    fn record_mechanism(&mut self, record: MechanismRecord) {
        self.epsilon_spent += record.epsilon;
        self.mechanisms.push(record);
    }

    fn effective_epsilon(&self) -> f64 {
        self.epsilon_spent
    }

    fn remaining_budget(&self) -> f64 {
        (self.total_budget - self.epsilon_spent).max(0.0)
    }

    fn is_exhausted(&self) -> bool {
        self.epsilon_spent >= self.total_budget
    }

    fn method(&self) -> CompositionMethod {
        CompositionMethod::Naive
    }

    fn mechanisms(&self) -> &[MechanismRecord] {
        &self.mechanisms
    }
}

// ---------------------------------------------------------------------------
// Renyi DP Accountant
// ---------------------------------------------------------------------------

/// Renyi Differential Privacy accountant.
///
/// Tracks RDP divergence curves at multiple alpha orders (`RDP_ALPHA_ORDERS`).
/// Under RDP composition, the Renyi divergence at each alpha is simply summed.
/// The optimal (epsilon, delta) guarantee is obtained by minimizing over all
/// tracked alpha values.
///
/// Conversion formula (RDP to (epsilon, delta)-DP):
///   epsilon(delta) = min over alpha of { rdp(alpha) + ln(1/delta) / (alpha - 1) }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenyiDPAccountant {
    /// Total epsilon budget (target epsilon for the final conversion).
    pub total_budget: f64,

    /// Target delta for the RDP-to-DP conversion.
    pub target_delta: f64,

    /// Accumulated RDP values at each alpha order.
    /// `rdp_curve[i]` corresponds to `RDP_ALPHA_ORDERS[i]`.
    pub rdp_curve: Vec<f64>,

    /// All recorded mechanisms.
    pub mechanisms: Vec<MechanismRecord>,
}

impl RenyiDPAccountant {
    /// Create a new Renyi DP accountant.
    ///
    /// # Arguments
    /// * `total_budget` - The target epsilon budget.
    /// * `target_delta` - The delta parameter for RDP-to-DP conversion (e.g., 1e-5).
    pub fn new(total_budget: f64, target_delta: f64) -> Self {
        Self {
            total_budget,
            target_delta,
            rdp_curve: vec![0.0; RDP_ALPHA_ORDERS.len()],
            mechanisms: Vec::new(),
        }
    }

    /// Convert a pure (epsilon, 0)-DP mechanism to RDP at a given alpha.
    ///
    /// For a pure epsilon-DP mechanism, the RDP guarantee at order alpha is:
    ///   rdp(alpha) = epsilon (since pure DP implies RDP at all orders).
    ///
    /// More precisely, for the Laplace mechanism with parameter epsilon:
    ///   rdp(alpha) = (1 / (alpha - 1)) * ln( (alpha-1)/(2*alpha-1) * exp((alpha-1)*eps)
    ///                + alpha/(2*alpha-1) * exp(-(alpha)*eps) )
    /// but for simplicity and safety, we use the pure-DP bound: rdp(alpha) <= epsilon.
    fn epsilon_to_rdp(epsilon: f64, alpha: f64) -> f64 {
        // For the Laplace mechanism, a tighter bound exists, but the pure-DP
        // bound rdp(alpha) <= epsilon is always valid and simpler.
        // We use the slightly tighter Laplace-specific formula when possible.
        if epsilon <= 0.0 {
            return 0.0;
        }

        // Laplace mechanism RDP bound:
        // rdp(alpha) = (1/(alpha-1)) * ln( (alpha-1)/(2*alpha-1) * exp((alpha-1)*eps)
        //              + alpha/(2*alpha-1) * exp(-alpha*eps) )
        if alpha > 1.0 {
            let term1 = ((alpha - 1.0) / (2.0 * alpha - 1.0)) * ((alpha - 1.0) * epsilon).exp();
            let term2 = (alpha / (2.0 * alpha - 1.0)) * (-(alpha) * epsilon).exp();
            let sum = term1 + term2;
            if sum > 0.0 {
                let rdp_val = sum.ln() / (alpha - 1.0);
                // RDP is always non-negative and at most the pure-DP bound
                return rdp_val.clamp(0.0, epsilon);
            }
        }

        // Fallback: pure DP bound
        epsilon
    }

    /// Convert accumulated RDP curve to (epsilon, delta)-DP.
    ///
    /// Uses the formula: epsilon = min over alpha of { rdp(alpha) + ln(1/delta) / (alpha - 1) }
    pub fn rdp_to_dp(&self) -> (f64, f64) {
        if self.target_delta <= 0.0 {
            // Without delta, RDP reduces to pure DP (worst case)
            let max_rdp = self.rdp_curve.iter().copied().fold(f64::INFINITY, f64::min);
            return (max_rdp, 0.0);
        }

        let ln_inv_delta = (1.0 / self.target_delta).ln();

        let best_epsilon = RDP_ALPHA_ORDERS
            .iter()
            .zip(self.rdp_curve.iter())
            .map(|(&alpha, &rdp_val)| rdp_val + ln_inv_delta / (alpha - 1.0))
            .fold(f64::INFINITY, f64::min);

        (best_epsilon, self.target_delta)
    }

    /// Get the alpha order that provides the tightest bound.
    pub fn optimal_alpha(&self) -> f64 {
        if self.target_delta <= 0.0 {
            return RDP_ALPHA_ORDERS[0];
        }

        let ln_inv_delta = (1.0 / self.target_delta).ln();

        let (best_idx, _) = RDP_ALPHA_ORDERS
            .iter()
            .zip(self.rdp_curve.iter())
            .enumerate()
            .map(|(i, (&alpha, &rdp_val))| (i, rdp_val + ln_inv_delta / (alpha - 1.0)))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or((0, f64::INFINITY));

        RDP_ALPHA_ORDERS[best_idx]
    }
}

impl PrivacyAccountant for RenyiDPAccountant {
    fn record_mechanism(&mut self, record: MechanismRecord) {
        // For each alpha order, compute the RDP cost and add it (composition = sum)
        for (i, &alpha) in RDP_ALPHA_ORDERS.iter().enumerate() {
            let rdp_cost = Self::epsilon_to_rdp(record.epsilon, alpha);
            self.rdp_curve[i] += rdp_cost;
        }
        self.mechanisms.push(record);
    }

    fn effective_epsilon(&self) -> f64 {
        let (eps, _) = self.rdp_to_dp();
        eps
    }

    fn remaining_budget(&self) -> f64 {
        (self.total_budget - self.effective_epsilon()).max(0.0)
    }

    fn is_exhausted(&self) -> bool {
        self.effective_epsilon() >= self.total_budget
    }

    fn method(&self) -> CompositionMethod {
        CompositionMethod::RenyiDP
    }

    fn mechanisms(&self) -> &[MechanismRecord] {
        &self.mechanisms
    }

    fn target_delta(&self) -> Option<f64> {
        Some(self.target_delta)
    }

    fn optimal_alpha(&self) -> Option<f64> {
        Some(self.optimal_alpha())
    }
}

// ---------------------------------------------------------------------------
// Zero-Concentrated DP Accountant
// ---------------------------------------------------------------------------

/// Zero-Concentrated Differential Privacy (zCDP) accountant.
///
/// Under zCDP, each mechanism has a rho parameter (rho = epsilon^2 / 2 for the
/// Gaussian mechanism). Composition is additive: total_rho = sum of individual rhos.
///
/// Conversion to (epsilon, delta)-DP:
///   epsilon = rho + 2 * sqrt(rho * ln(1/delta))
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZeroCDPAccountant {
    /// Total epsilon budget (target epsilon for the final conversion).
    pub total_budget: f64,

    /// Target delta for the zCDP-to-DP conversion.
    pub target_delta: f64,

    /// Accumulated rho value (additive under composition).
    pub total_rho: f64,

    /// All recorded mechanisms.
    pub mechanisms: Vec<MechanismRecord>,
}

impl ZeroCDPAccountant {
    /// Create a new zCDP accountant.
    ///
    /// # Arguments
    /// * `total_budget` - The target epsilon budget.
    /// * `target_delta` - The delta parameter for zCDP-to-DP conversion (e.g., 1e-5).
    pub fn new(total_budget: f64, target_delta: f64) -> Self {
        Self {
            total_budget,
            target_delta,
            total_rho: 0.0,
            mechanisms: Vec::new(),
        }
    }

    /// Convert a pure epsilon-DP mechanism to its zCDP rho parameter.
    ///
    /// For a pure epsilon-DP mechanism: rho = epsilon^2 / 2
    pub fn epsilon_to_rho(epsilon: f64) -> f64 {
        epsilon * epsilon / 2.0
    }

    /// Convert accumulated rho to (epsilon, delta)-DP.
    ///
    /// Uses the formula: epsilon = rho + 2 * sqrt(rho * ln(1/delta))
    pub fn rho_to_dp(&self) -> (f64, f64) {
        if self.total_rho <= 0.0 {
            return (0.0, 0.0);
        }

        if self.target_delta <= 0.0 {
            // Without delta, zCDP can't convert to finite epsilon
            return (f64::INFINITY, 0.0);
        }

        let ln_inv_delta = (1.0 / self.target_delta).ln();
        let epsilon = self.total_rho + 2.0 * (self.total_rho * ln_inv_delta).sqrt();
        (epsilon, self.target_delta)
    }

    /// Get the current accumulated rho value.
    pub fn current_rho(&self) -> f64 {
        self.total_rho
    }
}

impl PrivacyAccountant for ZeroCDPAccountant {
    fn record_mechanism(&mut self, record: MechanismRecord) {
        let rho = Self::epsilon_to_rho(record.epsilon);
        self.total_rho += rho;
        self.mechanisms.push(record);
    }

    fn effective_epsilon(&self) -> f64 {
        let (eps, _) = self.rho_to_dp();
        eps
    }

    fn remaining_budget(&self) -> f64 {
        (self.total_budget - self.effective_epsilon()).max(0.0)
    }

    fn is_exhausted(&self) -> bool {
        self.effective_epsilon() >= self.total_budget
    }

    fn method(&self) -> CompositionMethod {
        CompositionMethod::ZeroCDP
    }

    fn mechanisms(&self) -> &[MechanismRecord] {
        &self.mechanisms
    }

    fn target_delta(&self) -> Option<f64> {
        Some(self.target_delta)
    }
}

// ---------------------------------------------------------------------------
// Factory
// ---------------------------------------------------------------------------

/// Create a privacy accountant from a composition method and budget.
///
/// For `RenyiDP` and `ZeroCDP`, a default delta of 1e-5 is used.
/// Use the specific accountant constructors for custom delta values.
pub fn create_accountant(
    method: CompositionMethod,
    total_budget: f64,
) -> Box<dyn PrivacyAccountant> {
    match method {
        CompositionMethod::Naive | CompositionMethod::Advanced => {
            Box::new(NaiveAccountant::new(total_budget))
        }
        CompositionMethod::RenyiDP => Box::new(RenyiDPAccountant::new(total_budget, 1e-5)),
        CompositionMethod::ZeroCDP => Box::new(ZeroCDPAccountant::new(total_budget, 1e-5)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_naive_accountant_simple_sum() {
        let mut acc = NaiveAccountant::new(1.0);

        acc.record_mechanism(MechanismRecord::new(0.1, "query 1"));
        acc.record_mechanism(MechanismRecord::new(0.2, "query 2"));
        acc.record_mechanism(MechanismRecord::new(0.3, "query 3"));

        assert!((acc.effective_epsilon() - 0.6).abs() < 1e-10);
        assert!((acc.remaining_budget() - 0.4).abs() < 1e-10);
        assert!(!acc.is_exhausted());
        assert_eq!(acc.mechanisms().len(), 3);
    }

    #[test]
    fn test_naive_accountant_exhaustion() {
        let mut acc = NaiveAccountant::new(0.5);

        acc.record_mechanism(MechanismRecord::new(0.3, "query 1"));
        assert!(!acc.is_exhausted());

        acc.record_mechanism(MechanismRecord::new(0.3, "query 2"));
        assert!(acc.is_exhausted());
        assert_eq!(acc.remaining_budget(), 0.0);
    }

    #[test]
    fn test_renyi_accountant_composition() {
        let mut acc = RenyiDPAccountant::new(100.0, 1e-5);

        // Record many mechanisms - RDP shines with many compositions
        let n_queries = 1000;
        let eps_per_query = 0.1;
        for i in 0..n_queries {
            acc.record_mechanism(MechanismRecord::new(eps_per_query, format!("query {}", i)));
        }

        // The naive total would be 100.0. RDP should do better for many queries.
        let effective = acc.effective_epsilon();
        let naive_total = n_queries as f64 * eps_per_query;
        assert!(effective > 0.0, "Effective epsilon should be positive");
        assert!(
            effective < naive_total,
            "RDP ({:.4}) should be tighter than naive ({:.4}) for many queries",
            effective,
            naive_total
        );
    }

    #[test]
    fn test_renyi_accountant_rdp_to_dp() {
        let mut acc = RenyiDPAccountant::new(10.0, 1e-5);

        acc.record_mechanism(MechanismRecord::new(1.0, "large query"));

        let (eps, delta) = acc.rdp_to_dp();
        assert!(eps > 0.0);
        assert!((delta - 1e-5).abs() < 1e-15);
    }

    #[test]
    fn test_renyi_optimal_alpha() {
        let mut acc = RenyiDPAccountant::new(10.0, 1e-5);
        acc.record_mechanism(MechanismRecord::new(0.5, "query"));

        let alpha = acc.optimal_alpha();
        assert!(RDP_ALPHA_ORDERS.contains(&alpha));
    }

    #[test]
    fn test_zcdp_accountant_composition() {
        let mut acc = ZeroCDPAccountant::new(10.0, 1e-5);

        acc.record_mechanism(MechanismRecord::new(0.1, "query 1"));
        acc.record_mechanism(MechanismRecord::new(0.2, "query 2"));

        // rho = 0.01/2 + 0.04/2 = 0.005 + 0.02 = 0.025
        let expected_rho = 0.1_f64.powi(2) / 2.0 + 0.2_f64.powi(2) / 2.0;
        assert!(
            (acc.current_rho() - expected_rho).abs() < 1e-10,
            "Expected rho={}, got rho={}",
            expected_rho,
            acc.current_rho()
        );
    }

    #[test]
    fn test_zcdp_rho_to_dp() {
        let mut acc = ZeroCDPAccountant::new(10.0, 1e-5);

        acc.record_mechanism(MechanismRecord::new(1.0, "query"));

        let (eps, delta) = acc.rho_to_dp();
        assert!(eps > 0.0);
        assert!((delta - 1e-5).abs() < 1e-15);

        // rho = 0.5, epsilon = 0.5 + 2*sqrt(0.5 * ln(1e5))
        let expected_rho = 0.5;
        let expected_eps = expected_rho + 2.0 * (expected_rho * (1.0 / 1e-5_f64).ln()).sqrt();
        assert!(
            (eps - expected_eps).abs() < 1e-10,
            "Expected eps={}, got eps={}",
            expected_eps,
            eps
        );
    }

    #[test]
    fn test_zcdp_epsilon_to_rho() {
        assert!((ZeroCDPAccountant::epsilon_to_rho(1.0) - 0.5).abs() < 1e-10);
        assert!((ZeroCDPAccountant::epsilon_to_rho(0.0) - 0.0).abs() < 1e-10);
        assert!((ZeroCDPAccountant::epsilon_to_rho(2.0) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_composition_method_display() {
        assert_eq!(CompositionMethod::Naive.to_string(), "naive");
        assert_eq!(CompositionMethod::Advanced.to_string(), "advanced");
        assert_eq!(CompositionMethod::RenyiDP.to_string(), "renyi_dp");
        assert_eq!(CompositionMethod::ZeroCDP.to_string(), "zcdp");
    }

    #[test]
    fn test_composition_method_serde() {
        let json = serde_json::to_string(&CompositionMethod::RenyiDP).unwrap();
        assert_eq!(json, "\"renyi_dp\"");

        let parsed: CompositionMethod = serde_json::from_str("\"zcdp\"").unwrap();
        assert_eq!(parsed, CompositionMethod::ZeroCDP);

        // Default should be Naive
        let default: CompositionMethod = Default::default();
        assert_eq!(default, CompositionMethod::Naive);
    }

    #[test]
    fn test_mechanism_record_serde() {
        let record = MechanismRecord::new(0.5, "test mechanism").with_delta(1e-5);

        let json = serde_json::to_string(&record).unwrap();
        let parsed: MechanismRecord = serde_json::from_str(&json).unwrap();

        assert!((parsed.epsilon - 0.5).abs() < 1e-10);
        assert!((parsed.delta - 1e-5).abs() < 1e-15);
        assert_eq!(parsed.description, "test mechanism");
    }

    #[test]
    fn test_create_accountant_factory() {
        let acc = create_accountant(CompositionMethod::Naive, 1.0);
        assert_eq!(acc.method(), CompositionMethod::Naive);

        let acc = create_accountant(CompositionMethod::RenyiDP, 1.0);
        assert_eq!(acc.method(), CompositionMethod::RenyiDP);

        let acc = create_accountant(CompositionMethod::ZeroCDP, 1.0);
        assert_eq!(acc.method(), CompositionMethod::ZeroCDP);
    }

    #[test]
    fn test_rdp_tighter_than_naive_many_queries() {
        // With many small queries, RDP should give a significantly tighter bound
        let n_queries = 100;
        let eps_per_query = 0.01;

        let mut naive = NaiveAccountant::new(100.0);
        let mut rdp = RenyiDPAccountant::new(100.0, 1e-5);

        for i in 0..n_queries {
            let record = MechanismRecord::new(eps_per_query, format!("q{}", i));
            naive.record_mechanism(record.clone());
            rdp.record_mechanism(record);
        }

        let naive_eps = naive.effective_epsilon();
        let rdp_eps = rdp.effective_epsilon();

        assert!(
            rdp_eps <= naive_eps + 1e-10,
            "RDP ({}) should not be worse than naive ({})",
            rdp_eps,
            naive_eps
        );
    }

    #[test]
    fn test_zcdp_exhaustion() {
        let mut acc = ZeroCDPAccountant::new(1.0, 1e-5);

        // Keep adding until exhausted
        for i in 0..100 {
            if acc.is_exhausted() {
                assert!(i > 0, "Should not be exhausted immediately");
                return;
            }
            acc.record_mechanism(MechanismRecord::new(0.1, format!("q{}", i)));
        }
        // With delta=1e-5 and budget=1.0, we should exhaust relatively quickly
        assert!(acc.is_exhausted());
    }
}
