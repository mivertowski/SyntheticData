//! Membership Inference Attack (MIA) testing.
//!
//! Evaluates privacy by checking whether an attacker can determine if a specific
//! record was part of the training (original) dataset by examining the synthetic data.
//!
//! A distance-based MIA computes nearest-neighbor distances from member (in-training)
//! and non-member (hold-out) records to the synthetic dataset. If these distances are
//! distinguishable, privacy is at risk.
//!
//! **Interpretation**: AUC-ROC ~0.5 means the attack is no better than random = good privacy.
//! AUC-ROC ~1.0 means the synthetic data leaks membership information = poor privacy.

use serde::{Deserialize, Serialize};

/// Configuration for Membership Inference Attack evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiaConfig {
    /// Maximum AUC-ROC threshold below which we consider privacy preserved.
    /// Default: 0.6 (anything below indicates near-random classification).
    pub auc_threshold: f64,
    /// Number of nearest neighbors to consider for distance calculation.
    /// Default: 1 (single nearest neighbor).
    pub k_neighbors: usize,
}

impl Default for MiaConfig {
    fn default() -> Self {
        Self {
            auc_threshold: 0.6,
            k_neighbors: 1,
        }
    }
}

/// Results from a Membership Inference Attack evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiaResults {
    /// Area Under the ROC Curve for the distance-based classifier.
    /// ~0.5 = good privacy, ~1.0 = poor privacy.
    pub auc_roc: f64,
    /// Accuracy of the best threshold classifier.
    pub accuracy: f64,
    /// Precision of the best threshold classifier.
    pub precision: f64,
    /// Recall of the best threshold classifier.
    pub recall: f64,
    /// Whether privacy passes the configured AUC threshold.
    pub passes: bool,
    /// Number of member (in-training) records evaluated.
    pub n_members: usize,
    /// Number of non-member (hold-out) records evaluated.
    pub n_non_members: usize,
    /// Configured AUC threshold.
    pub auc_threshold: f64,
}

/// Membership Inference Attack evaluator.
///
/// Uses a distance-based approach: compute nearest-neighbor distances from
/// member and non-member records to the synthetic dataset, then evaluate
/// how well these distances separate the two groups.
pub struct MembershipInferenceAttack {
    config: MiaConfig,
}

impl MembershipInferenceAttack {
    /// Create a new MIA evaluator with the given config.
    pub fn new(config: MiaConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(MiaConfig::default())
    }

    /// Run the membership inference attack.
    ///
    /// # Arguments
    /// * `members` - Records that were in the original dataset (each record is a Vec<f64> of features)
    /// * `non_members` - Records NOT in the original dataset (hold-out set)
    /// * `synthetic` - The generated synthetic dataset
    ///
    /// All records must have the same dimensionality.
    pub fn evaluate(
        &self,
        members: &[Vec<f64>],
        non_members: &[Vec<f64>],
        synthetic: &[Vec<f64>],
    ) -> MiaResults {
        if members.is_empty() || non_members.is_empty() || synthetic.is_empty() {
            return MiaResults {
                auc_roc: 0.5,
                accuracy: 0.5,
                precision: 0.0,
                recall: 0.0,
                passes: true,
                n_members: members.len(),
                n_non_members: non_members.len(),
                auc_threshold: self.config.auc_threshold,
            };
        }

        // Compute nearest-neighbor distances for members
        let member_distances: Vec<f64> = members
            .iter()
            .map(|record| self.knn_distance(record, synthetic))
            .collect();

        // Compute nearest-neighbor distances for non-members
        let non_member_distances: Vec<f64> = non_members
            .iter()
            .map(|record| self.knn_distance(record, synthetic))
            .collect();

        // Members should have SMALLER distances if privacy is poor
        // (synthetic data is closer to training data)
        // Label: member=1 (positive), non-member=0 (negative)
        // Score: negative distance (lower distance = higher score = more likely member)
        let mut scored: Vec<(f64, bool)> = Vec::with_capacity(members.len() + non_members.len());
        for d in &member_distances {
            scored.push((-d, true)); // negative distance as score
        }
        for d in &non_member_distances {
            scored.push((-d, false));
        }

        let auc_roc = compute_auc(&scored);
        let (accuracy, precision, recall) = compute_best_threshold_metrics(&scored);

        MiaResults {
            auc_roc,
            accuracy,
            precision,
            recall,
            passes: auc_roc <= self.config.auc_threshold,
            n_members: members.len(),
            n_non_members: non_members.len(),
            auc_threshold: self.config.auc_threshold,
        }
    }

    /// Compute the k-nearest-neighbor distance from a record to a dataset.
    fn knn_distance(&self, record: &[f64], dataset: &[Vec<f64>]) -> f64 {
        let mut distances: Vec<f64> = dataset
            .iter()
            .map(|other| euclidean_distance(record, other))
            .collect();

        distances.sort_by(|a, b| a.total_cmp(b));

        let k = self.config.k_neighbors.min(distances.len());
        if k == 0 {
            return f64::MAX;
        }

        // Average of k nearest distances
        distances[..k].iter().sum::<f64>() / k as f64
    }
}

/// Compute Euclidean distance between two feature vectors.
fn euclidean_distance(a: &[f64], b: &[f64]) -> f64 {
    let len = a.len().min(b.len());
    let sum: f64 = (0..len).map(|i| (a[i] - b[i]).powi(2)).sum();
    sum.sqrt()
}

/// Compute AUC-ROC using the trapezoidal rule.
///
/// Expects `scored` as (score, is_positive) pairs where higher score = more likely positive.
fn compute_auc(scored: &[(f64, bool)]) -> f64 {
    if scored.is_empty() {
        return 0.5;
    }

    let mut sorted = scored.to_vec();
    sorted.sort_by(|a, b| b.0.total_cmp(&a.0)); // descending by score

    let total_pos = sorted.iter().filter(|s| s.1).count() as f64;
    let total_neg = sorted.iter().filter(|s| !s.1).count() as f64;

    if total_pos == 0.0 || total_neg == 0.0 {
        return 0.5;
    }

    let mut auc = 0.0;
    let mut tp = 0.0;
    let mut fp = 0.0;
    let mut prev_fpr = 0.0;
    let mut prev_tpr = 0.0;

    for &(_, is_pos) in &sorted {
        if is_pos {
            tp += 1.0;
        } else {
            fp += 1.0;
        }

        let tpr = tp / total_pos;
        let fpr = fp / total_neg;

        // Trapezoidal rule
        auc += (fpr - prev_fpr) * (tpr + prev_tpr) / 2.0;

        prev_fpr = fpr;
        prev_tpr = tpr;
    }

    auc
}

/// Find the best threshold and compute accuracy, precision, recall.
fn compute_best_threshold_metrics(scored: &[(f64, bool)]) -> (f64, f64, f64) {
    if scored.is_empty() {
        return (0.5, 0.0, 0.0);
    }

    let mut sorted = scored.to_vec();
    sorted.sort_by(|a, b| b.0.total_cmp(&a.0));

    let total = sorted.len() as f64;
    let total_pos = sorted.iter().filter(|s| s.1).count() as f64;

    let mut best_accuracy = 0.0;
    let mut best_precision = 0.0;
    let mut best_recall = 0.0;
    let mut tp = 0.0;
    for (i, &(_, is_pos)) in sorted.iter().enumerate() {
        if is_pos {
            tp += 1.0;
        }

        let predicted_pos = (i + 1) as f64;
        let fn_count = total_pos - tp;
        let tn = total - predicted_pos - fn_count;

        let accuracy = (tp + tn) / total;
        let precision = if predicted_pos > 0.0 {
            tp / predicted_pos
        } else {
            0.0
        };
        let recall = if total_pos > 0.0 { tp / total_pos } else { 0.0 };

        if accuracy > best_accuracy {
            best_accuracy = accuracy;
            best_precision = precision;
            best_recall = recall;
        }
    }

    (best_accuracy, best_precision, best_recall)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_copy_high_auc() {
        // If synthetic data IS the original data, MIA should detect membership easily
        let members: Vec<Vec<f64>> = (0..50)
            .map(|i| vec![i as f64, (i * 2) as f64, (i * 3) as f64])
            .collect();
        let non_members: Vec<Vec<f64>> = (100..150)
            .map(|i| vec![i as f64, (i * 2) as f64, (i * 3) as f64])
            .collect();
        // Synthetic = exact copy of members
        let synthetic = members.clone();

        let attack = MembershipInferenceAttack::with_defaults();
        let results = attack.evaluate(&members, &non_members, &synthetic);

        // AUC should be high (close to 1.0) — privacy is poor
        assert!(
            results.auc_roc > 0.8,
            "Expected high AUC for exact copies, got {}",
            results.auc_roc
        );
        assert!(
            !results.passes,
            "Should NOT pass privacy check for exact copies"
        );
    }

    #[test]
    fn test_random_data_low_auc() {
        // If synthetic data is random, MIA should be near random chance
        let members: Vec<Vec<f64>> = (0..50).map(|i| vec![i as f64, (i * 2) as f64]).collect();
        let non_members: Vec<Vec<f64>> =
            (50..100).map(|i| vec![i as f64, (i * 2) as f64]).collect();
        // Synthetic = different range entirely
        let synthetic: Vec<Vec<f64>> = (200..300).map(|i| vec![i as f64, (i * 2) as f64]).collect();

        let attack = MembershipInferenceAttack::with_defaults();
        let results = attack.evaluate(&members, &non_members, &synthetic);

        // AUC should be near 0.5 (random) — good privacy
        assert!(
            results.auc_roc < 0.7,
            "Expected low AUC for unrelated data, got {}",
            results.auc_roc
        );
    }

    #[test]
    fn test_empty_inputs() {
        let attack = MembershipInferenceAttack::with_defaults();

        let results = attack.evaluate(&[], &[], &[]);
        assert!(results.passes);
        assert!((results.auc_roc - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_euclidean_distance() {
        let a = vec![0.0, 0.0, 0.0];
        let b = vec![3.0, 4.0, 0.0];
        let dist = euclidean_distance(&a, &b);
        assert!((dist - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_compute_auc_perfect() {
        // Perfect classifier: all positives scored higher than all negatives
        let scored: Vec<(f64, bool)> = vec![
            (1.0, true),
            (0.9, true),
            (0.8, true),
            (0.3, false),
            (0.2, false),
            (0.1, false),
        ];
        let auc = compute_auc(&scored);
        assert!(
            (auc - 1.0).abs() < 1e-10,
            "Perfect AUC should be 1.0, got {}",
            auc
        );
    }

    #[test]
    fn test_compute_auc_random() {
        // Random classifier: interleaved positives and negatives
        let scored: Vec<(f64, bool)> = vec![
            (0.6, true),
            (0.5, false),
            (0.4, true),
            (0.3, false),
            (0.2, true),
            (0.1, false),
        ];
        let auc = compute_auc(&scored);
        assert!(
            (auc - 0.5).abs() < 0.2,
            "Near-random AUC should be around 0.5, got {}",
            auc
        );
    }

    #[test]
    fn test_mia_config_serde() {
        let config = MiaConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: MiaConfig = serde_json::from_str(&json).unwrap();
        assert!((parsed.auc_threshold - 0.6).abs() < 1e-10);
        assert_eq!(parsed.k_neighbors, 1);
    }

    #[test]
    fn test_mia_results_serde() {
        let results = MiaResults {
            auc_roc: 0.55,
            accuracy: 0.52,
            precision: 0.51,
            recall: 0.53,
            passes: true,
            n_members: 100,
            n_non_members: 100,
            auc_threshold: 0.6,
        };
        let json = serde_json::to_string(&results).unwrap();
        let parsed: MiaResults = serde_json::from_str(&json).unwrap();
        assert!((parsed.auc_roc - 0.55).abs() < 1e-10);
        assert!(parsed.passes);
    }
}
