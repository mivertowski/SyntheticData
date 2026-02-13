//! Embedding readiness evaluation.
//!
//! Validates prerequisites for representation learning by checking effective
//! dimensionality (via eigendecomposition), contrastive learning viability
//! (minimum class counts), and feature overlap between classes.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Input for embedding readiness analysis.
#[derive(Debug, Clone)]
pub struct EmbeddingInput {
    /// Feature matrix: rows are samples, columns are features.
    pub feature_matrix: Vec<Vec<f64>>,
    /// Class labels for each sample.
    pub labels: Vec<String>,
}

/// Thresholds for embedding readiness analysis.
#[derive(Debug, Clone)]
pub struct EmbeddingReadinessThresholds {
    /// Minimum embedding readiness score.
    pub min_embedding_readiness: f64,
}

impl Default for EmbeddingReadinessThresholds {
    fn default() -> Self {
        Self {
            min_embedding_readiness: 0.50,
        }
    }
}

/// Results of embedding readiness analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingReadinessAnalysis {
    /// Overall embedding readiness score (0.0-1.0).
    pub embedding_readiness_score: f64,
    /// Number of dimensions needed to capture 95% of variance.
    pub effective_dimensionality: usize,
    /// Total number of feature dimensions.
    pub total_dimensions: usize,
    /// Whether contrastive learning is viable (min class count >= 2).
    pub contrastive_learning_viable: bool,
    /// Minimum number of samples in any class.
    pub min_class_count: usize,
    /// Whether the analysis passes all thresholds.
    pub passes: bool,
    /// Issues found during analysis.
    pub issues: Vec<String>,
}

/// Analyzer for embedding readiness.
pub struct EmbeddingReadinessAnalyzer {
    thresholds: EmbeddingReadinessThresholds,
}

impl EmbeddingReadinessAnalyzer {
    /// Create a new analyzer with default thresholds.
    pub fn new() -> Self {
        Self {
            thresholds: EmbeddingReadinessThresholds::default(),
        }
    }

    /// Create an analyzer with custom thresholds.
    pub fn with_thresholds(thresholds: EmbeddingReadinessThresholds) -> Self {
        Self { thresholds }
    }

    /// Analyze embedding readiness.
    pub fn analyze(&self, input: &EmbeddingInput) -> EvalResult<EmbeddingReadinessAnalysis> {
        let mut issues = Vec::new();

        if input.feature_matrix.is_empty() {
            return Ok(EmbeddingReadinessAnalysis {
                embedding_readiness_score: 0.0,
                effective_dimensionality: 0,
                total_dimensions: 0,
                contrastive_learning_viable: false,
                min_class_count: 0,
                passes: true,
                issues: vec!["No samples provided".to_string()],
            });
        }

        let total_dimensions = input.feature_matrix.first().map(|r| r.len()).unwrap_or(0);

        if total_dimensions == 0 {
            return Ok(EmbeddingReadinessAnalysis {
                embedding_readiness_score: 0.0,
                effective_dimensionality: 0,
                total_dimensions: 0,
                contrastive_learning_viable: false,
                min_class_count: 0,
                passes: false,
                issues: vec!["Zero-dimensional features".to_string()],
            });
        }

        // Compute effective dimensionality
        let effective_dimensionality =
            self.compute_effective_dimensionality(&input.feature_matrix, total_dimensions);

        // Check contrastive learning viability
        let mut class_counts: HashMap<&str, usize> = HashMap::new();
        for label in &input.labels {
            *class_counts.entry(label.as_str()).or_insert(0) += 1;
        }

        let min_class_count = class_counts.values().copied().min().unwrap_or(0);
        let num_classes = class_counts.len();
        let contrastive_learning_viable = min_class_count >= 2 && num_classes >= 2;

        if !contrastive_learning_viable {
            issues.push(format!(
                "Contrastive learning not viable: {} classes, min count = {}",
                num_classes, min_class_count
            ));
        }

        // Composite readiness score
        let dim_ratio = if total_dimensions > 0 {
            effective_dimensionality as f64 / total_dimensions as f64
        } else {
            0.0
        };
        // Lower effective dimensionality ratio = better (more compressible)
        let dim_score = (1.0 - dim_ratio).clamp(0.0, 1.0);
        let contrastive_score = if contrastive_learning_viable {
            1.0
        } else {
            0.0
        };
        let class_balance_score = if num_classes >= 2 && min_class_count > 0 {
            let max_count = class_counts.values().copied().max().unwrap_or(1);
            (min_class_count as f64 / max_count as f64).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let embedding_readiness_score =
            (dim_score * 0.4 + contrastive_score * 0.3 + class_balance_score * 0.3).clamp(0.0, 1.0);

        if embedding_readiness_score < self.thresholds.min_embedding_readiness {
            issues.push(format!(
                "Embedding readiness score {:.4} < {:.4} (threshold)",
                embedding_readiness_score, self.thresholds.min_embedding_readiness
            ));
        }

        let passes = issues.is_empty();

        Ok(EmbeddingReadinessAnalysis {
            embedding_readiness_score,
            effective_dimensionality,
            total_dimensions,
            contrastive_learning_viable,
            min_class_count,
            passes,
            issues,
        })
    }

    /// Compute effective dimensionality using power iteration on the covariance matrix.
    ///
    /// Finds the top eigenvalues and counts how many are needed to reach 95%
    /// of total variance.
    fn compute_effective_dimensionality(
        &self,
        feature_matrix: &[Vec<f64>],
        total_dims: usize,
    ) -> usize {
        let n = feature_matrix.len();
        if n < 2 || total_dims == 0 {
            return total_dims;
        }

        // Compute column means
        let mut means = vec![0.0; total_dims];
        for row in feature_matrix {
            for (j, &val) in row.iter().enumerate().take(total_dims) {
                means[j] += val;
            }
        }
        for m in &mut means {
            *m /= n as f64;
        }

        // Compute covariance matrix (total_dims x total_dims)
        let dim = total_dims.min(50); // Cap for computational feasibility
        let mut cov = vec![vec![0.0; dim]; dim];

        for row in feature_matrix {
            for i in 0..dim {
                let di = if i < row.len() {
                    row[i] - means[i]
                } else {
                    0.0
                };
                for j in i..dim {
                    let dj = if j < row.len() {
                        row[j] - means[j]
                    } else {
                        0.0
                    };
                    cov[i][j] += di * dj;
                }
            }
        }

        // Symmetrize and normalize
        #[allow(clippy::needless_range_loop)]
        for i in 0..dim {
            for j in i..dim {
                cov[i][j] /= (n - 1) as f64;
                cov[j][i] = cov[i][j];
            }
        }

        // Extract eigenvalues via repeated power iteration with deflation
        let max_eigenvalues = dim;
        let mut eigenvalues = Vec::new();
        let mut work_cov = cov.clone();

        for _ in 0..max_eigenvalues {
            let (eigenvalue, eigenvector) = self.power_iteration(&work_cov, dim);
            if eigenvalue.abs() < 1e-12 {
                break;
            }
            eigenvalues.push(eigenvalue);

            // Deflate: A = A - lambda * v * v^T
            for i in 0..dim {
                for j in 0..dim {
                    work_cov[i][j] -= eigenvalue * eigenvector[i] * eigenvector[j];
                }
            }
        }

        if eigenvalues.is_empty() {
            return total_dims;
        }

        // Count dimensions for 95% of total variance
        let total_variance: f64 = eigenvalues.iter().filter(|&&v| v > 0.0).sum();
        if total_variance < 1e-12 {
            return total_dims;
        }

        let target = 0.95 * total_variance;
        let mut cumulative = 0.0;
        let mut effective = 0;

        for &ev in &eigenvalues {
            if ev <= 0.0 {
                continue;
            }
            cumulative += ev;
            effective += 1;
            if cumulative >= target {
                break;
            }
        }

        effective.max(1)
    }

    /// Power iteration to find the largest eigenvalue and corresponding eigenvector.
    fn power_iteration(&self, matrix: &[Vec<f64>], dim: usize) -> (f64, Vec<f64>) {
        let max_iter = 100;
        let tolerance = 1e-10;

        // Initialize with a non-zero vector
        let mut v = vec![1.0 / (dim as f64).sqrt(); dim];
        let mut eigenvalue = 0.0;

        for _ in 0..max_iter {
            // w = A * v
            let mut w = vec![0.0; dim];
            for i in 0..dim {
                for j in 0..dim {
                    w[i] += matrix[i][j] * v[j];
                }
            }

            // Compute eigenvalue as v^T * w
            let new_eigenvalue: f64 = v.iter().zip(w.iter()).map(|(vi, wi)| vi * wi).sum();

            // Normalize w
            let norm: f64 = w.iter().map(|x| x * x).sum::<f64>().sqrt();
            if norm < 1e-15 {
                break;
            }
            for x in &mut w {
                *x /= norm;
            }

            // Check convergence
            if (new_eigenvalue - eigenvalue).abs() < tolerance {
                eigenvalue = new_eigenvalue;
                v = w;
                break;
            }

            eigenvalue = new_eigenvalue;
            v = w;
        }

        (eigenvalue, v)
    }
}

impl Default for EmbeddingReadinessAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_embedding_input() {
        let input = EmbeddingInput {
            feature_matrix: vec![
                vec![1.0, 0.0, 0.0, 0.5],
                vec![0.9, 0.1, 0.0, 0.6],
                vec![0.0, 1.0, 0.0, 0.2],
                vec![0.1, 0.9, 0.1, 0.3],
                vec![0.0, 0.0, 1.0, 0.8],
                vec![0.0, 0.1, 0.9, 0.7],
            ],
            labels: vec![
                "A".into(),
                "A".into(),
                "B".into(),
                "B".into(),
                "C".into(),
                "C".into(),
            ],
        };

        let analyzer = EmbeddingReadinessAnalyzer::new();
        let result = analyzer.analyze(&input).unwrap();

        assert_eq!(result.total_dimensions, 4);
        assert!(result.effective_dimensionality > 0);
        assert!(result.effective_dimensionality <= 4);
        assert!(result.contrastive_learning_viable);
        assert_eq!(result.min_class_count, 2);
        assert!(result.embedding_readiness_score > 0.0);
    }

    #[test]
    fn test_invalid_single_class() {
        let input = EmbeddingInput {
            feature_matrix: vec![vec![1.0, 2.0], vec![3.0, 4.0], vec![5.0, 6.0]],
            labels: vec!["A".into(), "A".into(), "A".into()],
        };

        let analyzer = EmbeddingReadinessAnalyzer::new();
        let result = analyzer.analyze(&input).unwrap();

        assert!(!result.contrastive_learning_viable);
        assert!(!result.passes);
    }

    #[test]
    fn test_empty_input() {
        let input = EmbeddingInput {
            feature_matrix: Vec::new(),
            labels: Vec::new(),
        };

        let analyzer = EmbeddingReadinessAnalyzer::new();
        let result = analyzer.analyze(&input).unwrap();

        assert_eq!(result.total_dimensions, 0);
        assert_eq!(result.effective_dimensionality, 0);
        assert!(!result.contrastive_learning_viable);
    }
}
