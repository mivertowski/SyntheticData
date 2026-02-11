//! Gaussian copula for preserving correlations.

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use crate::models::{CorrelationMatrix, EmpiricalCdf, GaussianCopula};

/// Generator using Gaussian copula for correlated samples.
#[derive(Debug)]
pub struct CopulaGenerator {
    /// Correlation matrix (Cholesky decomposed).
    cholesky: Vec<Vec<f64>>,
    /// Marginal CDFs.
    marginals: Vec<EmpiricalCdf>,
    /// RNG.
    rng: ChaCha8Rng,
    /// Dimension.
    dim: usize,
}

impl CopulaGenerator {
    /// Create from a Gaussian copula specification.
    pub fn from_copula(copula: &GaussianCopula, seed: u64) -> Option<Self> {
        let dim = copula.columns.len();
        if dim == 0 {
            return None;
        }

        // Reconstruct full correlation matrix
        let mut corr_matrix = vec![vec![0.0; dim]; dim];
        let mut idx = 0;
        for i in 0..dim {
            corr_matrix[i][i] = 1.0;
            for j in (i + 1)..dim {
                if idx < copula.correlation_matrix.len() {
                    corr_matrix[i][j] = copula.correlation_matrix[idx];
                    corr_matrix[j][i] = copula.correlation_matrix[idx];
                    idx += 1;
                }
            }
        }

        let cholesky = cholesky_decompose(&corr_matrix)?;

        Some(Self {
            cholesky,
            marginals: copula.marginal_cdfs.clone(),
            rng: ChaCha8Rng::seed_from_u64(seed),
            dim,
        })
    }

    /// Create from a correlation matrix.
    pub fn from_correlation_matrix(matrix: &CorrelationMatrix, seed: u64) -> Option<Self> {
        let dim = matrix.columns.len();
        if dim == 0 {
            return None;
        }

        let full_matrix = matrix.to_full_matrix();
        let cholesky = cholesky_decompose(&full_matrix)?;

        Some(Self {
            cholesky,
            marginals: Vec::new(), // No marginals, will output uniform
            rng: ChaCha8Rng::seed_from_u64(seed),
            dim,
        })
    }

    /// Set marginal CDFs.
    pub fn with_marginals(mut self, marginals: Vec<EmpiricalCdf>) -> Self {
        self.marginals = marginals;
        self
    }

    /// Generate one sample of correlated uniform values.
    pub fn sample_uniform(&mut self) -> Vec<f64> {
        // Generate independent standard normal samples
        let z: Vec<f64> = (0..self.dim)
            .map(|_| self.sample_standard_normal())
            .collect();

        // Transform through Cholesky to get correlated normals
        let mut y = vec![0.0; self.dim];
        for i in 0..self.dim {
            for j in 0..=i {
                y[i] += self.cholesky[i][j] * z[j];
            }
        }

        // Transform to uniform through normal CDF
        y.iter().map(|&v| standard_normal_cdf(v)).collect()
    }

    /// Generate one sample, transforming through marginals.
    pub fn sample(&mut self) -> Vec<f64> {
        let u = self.sample_uniform();

        if self.marginals.is_empty() || self.marginals.len() != self.dim {
            return u;
        }

        // Transform uniform to target distribution through inverse CDF
        u.iter()
            .enumerate()
            .map(|(i, &p)| self.marginals[i].quantile(p))
            .collect()
    }

    /// Generate multiple samples.
    pub fn sample_n(&mut self, n: usize) -> Vec<Vec<f64>> {
        (0..n).map(|_| self.sample()).collect()
    }

    /// Sample from standard normal using Box-Muller.
    fn sample_standard_normal(&mut self) -> f64 {
        let u1: f64 = self.rng.gen();
        let u2: f64 = self.rng.gen();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
}

/// Cholesky decomposition of a positive-definite matrix.
fn cholesky_decompose(matrix: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
    let n = matrix.len();
    let mut l = vec![vec![0.0; n]; n];

    for i in 0..n {
        for j in 0..=i {
            let mut sum = 0.0;
            for k in 0..j {
                sum += l[i][k] * l[j][k];
            }

            if i == j {
                let diag = matrix[i][i] - sum;
                if diag <= 0.0 {
                    // Matrix not positive definite, apply small regularization
                    l[i][j] = (diag + 0.001).sqrt();
                } else {
                    l[i][j] = diag.sqrt();
                }
            } else {
                if l[j][j].abs() < 1e-10 {
                    return None;
                }
                l[i][j] = (matrix[i][j] - sum) / l[j][j];
            }
        }
    }

    Some(l)
}

/// Standard normal CDF approximation.
fn standard_normal_cdf(x: f64) -> f64 {
    0.5 * (1.0 + erf(x / std::f64::consts::SQRT_2))
}

/// Error function approximation.
fn erf(x: f64) -> f64 {
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();

    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

    sign * y
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_cholesky() {
        // Identity matrix should decompose to itself
        let identity = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let l = cholesky_decompose(&identity).unwrap();
        assert!((l[0][0] - 1.0).abs() < 0.001);
        assert!((l[1][1] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_copula_samples_in_range() {
        let corr = vec![vec![1.0, 0.5], vec![0.5, 1.0]];
        let cholesky = cholesky_decompose(&corr).unwrap();

        let mut gen = CopulaGenerator {
            cholesky,
            marginals: Vec::new(),
            rng: ChaCha8Rng::seed_from_u64(42),
            dim: 2,
        };

        for _ in 0..100 {
            let sample = gen.sample_uniform();
            assert!(sample[0] >= 0.0 && sample[0] <= 1.0);
            assert!(sample[1] >= 0.0 && sample[1] <= 1.0);
        }
    }
}
