//! Domain gap evaluation.
//!
//! Measures distribution divergence between synthetic and reference data
//! using PSI (Population Stability Index), KS statistic, and MMD
//! (Maximum Mean Discrepancy).

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};

/// A pair of value distributions to compare.
#[derive(Debug, Clone)]
pub struct DistributionSample {
    /// Name of this distribution.
    pub name: String,
    /// Values from the synthetic dataset.
    pub synthetic_values: Vec<f64>,
    /// Values from the reference (real) dataset.
    pub reference_values: Vec<f64>,
}

/// Thresholds for domain gap analysis.
#[derive(Debug, Clone)]
pub struct DomainGapThresholds {
    /// Maximum acceptable domain gap score.
    pub max_domain_gap: f64,
}

impl Default for DomainGapThresholds {
    fn default() -> Self {
        Self {
            max_domain_gap: 0.25,
        }
    }
}

/// Detail for a single distribution comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainGapDetail {
    /// Name of the distribution.
    pub name: String,
    /// Population Stability Index.
    pub psi: f64,
    /// Kolmogorov-Smirnov statistic.
    pub ks_statistic: f64,
    /// Maximum Mean Discrepancy (Gaussian kernel).
    pub mmd: f64,
}

/// Results of domain gap analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainGapAnalysis {
    /// Overall domain gap score (0 = identical, 1 = very different).
    pub domain_gap_score: f64,
    /// Per-distribution comparison details.
    pub per_distribution: Vec<DomainGapDetail>,
    /// Total number of distributions compared.
    pub total_distributions: usize,
    /// Whether the analysis passes all thresholds.
    pub passes: bool,
    /// Issues found during analysis.
    pub issues: Vec<String>,
}

/// Analyzer for domain gap between synthetic and reference distributions.
pub struct DomainGapAnalyzer {
    thresholds: DomainGapThresholds,
}

impl DomainGapAnalyzer {
    /// Create a new analyzer with default thresholds.
    pub fn new() -> Self {
        Self {
            thresholds: DomainGapThresholds::default(),
        }
    }

    /// Create an analyzer with custom thresholds.
    pub fn with_thresholds(thresholds: DomainGapThresholds) -> Self {
        Self { thresholds }
    }

    /// Analyze domain gap across distribution samples.
    pub fn analyze(&self, samples: &[DistributionSample]) -> EvalResult<DomainGapAnalysis> {
        let mut issues = Vec::new();
        let total_distributions = samples.len();

        if samples.is_empty() {
            return Ok(DomainGapAnalysis {
                domain_gap_score: 0.0,
                per_distribution: Vec::new(),
                total_distributions: 0,
                passes: true,
                issues: vec!["No distributions provided".to_string()],
            });
        }

        let mut details = Vec::new();
        let mut gap_sum = 0.0;

        for sample in samples {
            if sample.synthetic_values.is_empty() || sample.reference_values.is_empty() {
                details.push(DomainGapDetail {
                    name: sample.name.clone(),
                    psi: 0.0,
                    ks_statistic: 0.0,
                    mmd: 0.0,
                });
                continue;
            }

            let psi = self.compute_psi(&sample.synthetic_values, &sample.reference_values);
            let ks = self.compute_ks(&sample.synthetic_values, &sample.reference_values);
            let mmd = self.compute_mmd(&sample.synthetic_values, &sample.reference_values);

            // Normalize each metric to [0, 1] and average for composite gap
            let psi_norm = (psi / 0.5).clamp(0.0, 1.0); // PSI > 0.25 is major shift
            let ks_norm = ks.clamp(0.0, 1.0);
            let mmd_norm = mmd.clamp(0.0, 1.0);

            let gap = (psi_norm + ks_norm + mmd_norm) / 3.0;
            gap_sum += gap;

            details.push(DomainGapDetail {
                name: sample.name.clone(),
                psi,
                ks_statistic: ks,
                mmd,
            });
        }

        let domain_gap_score = if total_distributions > 0 {
            (gap_sum / total_distributions as f64).clamp(0.0, 1.0)
        } else {
            0.0
        };

        if domain_gap_score > self.thresholds.max_domain_gap {
            issues.push(format!(
                "Domain gap score {:.4} > {:.4} (threshold)",
                domain_gap_score, self.thresholds.max_domain_gap
            ));
        }

        let passes = issues.is_empty();

        Ok(DomainGapAnalysis {
            domain_gap_score,
            per_distribution: details,
            total_distributions,
            passes,
            issues,
        })
    }

    /// Compute Population Stability Index.
    ///
    /// Bins both distributions into equal-width buckets and computes
    /// PSI = sum((p_i - q_i) * ln(p_i / q_i)).
    fn compute_psi(&self, synthetic: &[f64], reference: &[f64]) -> f64 {
        let num_bins = 10;
        let epsilon = 1e-6;

        // Find global min/max
        let all_min = synthetic
            .iter()
            .chain(reference.iter())
            .cloned()
            .fold(f64::INFINITY, f64::min);
        let all_max = synthetic
            .iter()
            .chain(reference.iter())
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);

        if (all_max - all_min).abs() < 1e-12 {
            return 0.0;
        }

        let bin_width = (all_max - all_min) / num_bins as f64;

        let bin_index = |val: f64| -> usize {
            let idx = ((val - all_min) / bin_width) as usize;
            idx.min(num_bins - 1)
        };

        let mut syn_counts = vec![0usize; num_bins];
        let mut ref_counts = vec![0usize; num_bins];

        for &v in synthetic {
            syn_counts[bin_index(v)] += 1;
        }
        for &v in reference {
            ref_counts[bin_index(v)] += 1;
        }

        let syn_total = synthetic.len() as f64;
        let ref_total = reference.len() as f64;

        let mut psi = 0.0;
        for i in 0..num_bins {
            let p = (syn_counts[i] as f64 / syn_total) + epsilon;
            let q = (ref_counts[i] as f64 / ref_total) + epsilon;
            psi += (p - q) * (p / q).ln();
        }

        psi.max(0.0)
    }

    /// Compute Kolmogorov-Smirnov statistic: max|F_synthetic - F_reference|.
    fn compute_ks(&self, synthetic: &[f64], reference: &[f64]) -> f64 {
        let mut syn_sorted = synthetic.to_vec();
        let mut ref_sorted = reference.to_vec();
        syn_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        ref_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let syn_n = syn_sorted.len() as f64;
        let ref_n = ref_sorted.len() as f64;

        let mut max_diff = 0.0_f64;
        let mut i = 0usize;
        let mut j = 0usize;

        while i < syn_sorted.len() && j < ref_sorted.len() {
            let syn_cdf = (i + 1) as f64 / syn_n;
            let ref_cdf = (j + 1) as f64 / ref_n;

            if syn_sorted[i] <= ref_sorted[j] {
                let diff = (syn_cdf - (j as f64 / ref_n)).abs();
                if diff > max_diff {
                    max_diff = diff;
                }
                i += 1;
            } else {
                let diff = ((i as f64 / syn_n) - ref_cdf).abs();
                if diff > max_diff {
                    max_diff = diff;
                }
                j += 1;
            }
        }

        // Handle remaining elements
        while i < syn_sorted.len() {
            let syn_cdf = (i + 1) as f64 / syn_n;
            let diff = (syn_cdf - 1.0).abs();
            if diff > max_diff {
                max_diff = diff;
            }
            i += 1;
        }
        while j < ref_sorted.len() {
            let ref_cdf = (j + 1) as f64 / ref_n;
            let diff = (1.0 - ref_cdf).abs();
            if diff > max_diff {
                max_diff = diff;
            }
            j += 1;
        }

        max_diff
    }

    /// Compute Maximum Mean Discrepancy with Gaussian kernel.
    ///
    /// Subsamples both distributions to min(1000, n) for efficiency.
    fn compute_mmd(&self, synthetic: &[f64], reference: &[f64]) -> f64 {
        let max_samples = 1000;
        let syn_sub = subsample(synthetic, max_samples);
        let ref_sub = subsample(reference, max_samples);

        if syn_sub.is_empty() || ref_sub.is_empty() {
            return 0.0;
        }

        // Estimate bandwidth using median heuristic
        let sigma = self.median_bandwidth(&syn_sub, &ref_sub);
        if sigma < 1e-12 {
            return 0.0;
        }

        let gamma = -1.0 / (2.0 * sigma * sigma);

        let k_xx = self.mean_kernel(&syn_sub, &syn_sub, gamma);
        let k_yy = self.mean_kernel(&ref_sub, &ref_sub, gamma);
        let k_xy = self.mean_kernel(&syn_sub, &ref_sub, gamma);

        (k_xx + k_yy - 2.0 * k_xy).max(0.0).sqrt()
    }

    /// Compute mean Gaussian kernel value between two sets.
    fn mean_kernel(&self, x: &[f64], y: &[f64], gamma: f64) -> f64 {
        let mut sum = 0.0;
        for &xi in x {
            for &yi in y {
                let diff = xi - yi;
                sum += (gamma * diff * diff).exp();
            }
        }
        sum / (x.len() as f64 * y.len() as f64)
    }

    /// Estimate kernel bandwidth using the median heuristic.
    fn median_bandwidth(&self, x: &[f64], y: &[f64]) -> f64 {
        let mut dists = Vec::new();
        let step_x = if x.len() > 50 { x.len() / 50 } else { 1 };
        let step_y = if y.len() > 50 { y.len() / 50 } else { 1 };

        let mut ix = 0;
        while ix < x.len() {
            let mut iy = 0;
            while iy < y.len() {
                dists.push((x[ix] - y[iy]).abs());
                iy += step_y;
            }
            ix += step_x;
        }

        if dists.is_empty() {
            return 1.0;
        }

        dists.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        dists[dists.len() / 2].max(1e-6)
    }
}

/// Subsample a slice to at most `max` elements using stride.
fn subsample(data: &[f64], max: usize) -> Vec<f64> {
    if data.len() <= max {
        return data.to_vec();
    }
    let step = data.len() / max;
    data.iter().step_by(step).copied().take(max).collect()
}

impl Default for DomainGapAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_distributions() {
        let samples = vec![DistributionSample {
            name: "amount".to_string(),
            synthetic_values: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
            reference_values: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
        }];

        let analyzer = DomainGapAnalyzer::new();
        let result = analyzer.analyze(&samples).unwrap();

        assert!(result.domain_gap_score < 0.25);
        assert!(result.passes);
    }

    #[test]
    fn test_divergent_distributions() {
        let samples = vec![DistributionSample {
            name: "amount".to_string(),
            synthetic_values: vec![1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0, 5.5],
            reference_values: vec![50.0, 55.0, 60.0, 65.0, 70.0, 75.0, 80.0, 85.0, 90.0, 95.0],
        }];

        let analyzer = DomainGapAnalyzer::new();
        let result = analyzer.analyze(&samples).unwrap();

        assert!(result.domain_gap_score > 0.25);
        assert!(!result.passes);
    }

    #[test]
    fn test_empty_samples() {
        let analyzer = DomainGapAnalyzer::new();
        let result = analyzer.analyze(&[]).unwrap();

        assert_eq!(result.total_distributions, 0);
        assert_eq!(result.domain_gap_score, 0.0);
    }
}
