//! Statistical diffusion backend that generates data matching target distributions.
//!
//! Uses a Langevin-inspired reverse process to denoise samples toward
//! target means and standard deviations, with optional correlation structure
//! applied via Cholesky decomposition.

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, StandardNormal};

use super::backend::{DiffusionBackend, DiffusionConfig};
use super::schedule::NoiseSchedule;

/// A diffusion backend that generates samples matching target statistical properties.
///
/// The forward process adds Gaussian noise according to the noise schedule.
/// The reverse process uses Langevin-inspired updates to guide samples toward
/// the target distribution (means, standard deviations, correlations).
#[derive(Debug, Clone)]
pub struct StatisticalDiffusionBackend {
    /// Target means for each feature.
    means: Vec<f64>,
    /// Target standard deviations for each feature.
    stds: Vec<f64>,
    /// Optional correlation matrix (n_features x n_features).
    correlations: Option<Vec<Vec<f64>>>,
    /// Diffusion configuration.
    config: DiffusionConfig,
    /// Precomputed noise schedule.
    schedule: NoiseSchedule,
}

impl StatisticalDiffusionBackend {
    /// Create a new statistical diffusion backend.
    ///
    /// # Arguments
    /// * `means` - Target means for each feature dimension
    /// * `stds` - Target standard deviations for each feature dimension
    /// * `config` - Diffusion configuration (steps, schedule type, seed)
    pub fn new(means: Vec<f64>, stds: Vec<f64>, config: DiffusionConfig) -> Self {
        let schedule = config.build_schedule();
        Self {
            means,
            stds,
            correlations: None,
            config,
            schedule,
        }
    }

    /// Set the correlation matrix for multi-dimensional generation.
    ///
    /// The matrix should be symmetric positive-definite with ones on the diagonal.
    /// After denoising, Cholesky decomposition is used to impose this correlation
    /// structure on the generated samples.
    pub fn with_correlations(mut self, corr_matrix: Vec<Vec<f64>>) -> Self {
        self.correlations = Some(corr_matrix);
        self
    }

    /// Perform Cholesky decomposition of a symmetric positive-definite matrix.
    ///
    /// Returns the lower-triangular matrix L such that A = L * L^T.
    /// Returns `None` if the matrix is not positive-definite.
    fn cholesky_decomposition(matrix: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
        let n = matrix.len();
        if n == 0 {
            return Some(vec![]);
        }

        let mut l = vec![vec![0.0; n]; n];

        for i in 0..n {
            for j in 0..=i {
                let sum: f64 = l[i]
                    .iter()
                    .zip(l[j].iter())
                    .take(j)
                    .map(|(a, b)| a * b)
                    .sum();

                if i == j {
                    let diag = matrix[i][i] - sum;
                    if diag <= 0.0 {
                        return None;
                    }
                    l[i][j] = diag.sqrt();
                } else {
                    if l[j][j].abs() < 1e-15 {
                        return None;
                    }
                    l[i][j] = (matrix[i][j] - sum) / l[j][j];
                }
            }
        }

        Some(l)
    }

    /// Apply correlation structure to independent samples using Cholesky decomposition.
    ///
    /// Given independent standard normal samples, multiply by the Cholesky factor
    /// to produce correlated samples.
    fn apply_correlation(samples: &mut [Vec<f64>], cholesky_l: &[Vec<f64>]) {
        let n_features = cholesky_l.len();
        for row in samples.iter_mut() {
            let original: Vec<f64> = row.iter().copied().take(n_features).collect();
            for i in 0..n_features.min(row.len()) {
                let mut val = 0.0;
                for j in 0..=i {
                    if j < original.len() {
                        val += cholesky_l[i][j] * original[j];
                    }
                }
                row[i] = val;
            }
        }
    }
}

impl DiffusionBackend for StatisticalDiffusionBackend {
    fn name(&self) -> &str {
        "statistical"
    }

    /// Forward process: add noise at timestep t.
    ///
    /// x_t = sqrt(alpha_bar_t) * x_0 + sqrt(1 - alpha_bar_t) * noise
    fn forward(&self, x: &[Vec<f64>], t: usize) -> Vec<Vec<f64>> {
        let t_clamped = t.min(self.schedule.n_steps().saturating_sub(1));
        let sqrt_alpha_bar = self.schedule.sqrt_alpha_bars[t_clamped];
        let sqrt_one_minus_alpha_bar = self.schedule.sqrt_one_minus_alpha_bars[t_clamped];

        let n_features = x.first().map_or(0, |row| row.len());
        let noise =
            super::generate_noise(x.len(), n_features, self.config.seed.wrapping_add(t as u64));

        x.iter()
            .zip(noise.iter())
            .map(|(row, noise_row)| {
                row.iter()
                    .zip(noise_row.iter())
                    .map(|(&xi, &ni)| sqrt_alpha_bar * xi + sqrt_one_minus_alpha_bar * ni)
                    .collect()
            })
            .collect()
    }

    /// Reverse process: denoise at timestep t using Langevin-inspired updates.
    ///
    /// x_{t-1} = x_t - step_size * (x_t - mu) / sigma^2 + noise_scale * noise
    fn reverse(&self, x_t: &[Vec<f64>], t: usize) -> Vec<Vec<f64>> {
        let t_clamped = t.min(self.schedule.n_steps().saturating_sub(1));
        let beta_t = self.schedule.betas[t_clamped];

        // Step size derived from the noise schedule beta
        let step_size = beta_t;
        // Noise scale decreases as we approach t=0
        let noise_scale = if t_clamped > 0 { beta_t.sqrt() } else { 0.0 };

        let n_features = x_t.first().map_or(0, |row| row.len());
        let noise = super::generate_noise(
            x_t.len(),
            n_features,
            self.config
                .seed
                .wrapping_add(t as u64)
                .wrapping_add(1_000_000),
        );

        x_t.iter()
            .zip(noise.iter())
            .map(|(row, noise_row)| {
                row.iter()
                    .enumerate()
                    .map(|(j, &x_val)| {
                        let mu = if j < self.means.len() {
                            self.means[j]
                        } else {
                            0.0
                        };
                        let sigma = if j < self.stds.len() {
                            self.stds[j].max(1e-8)
                        } else {
                            1.0
                        };
                        let n = if j < noise_row.len() {
                            noise_row[j]
                        } else {
                            0.0
                        };

                        // Langevin-inspired drift toward target distribution
                        let drift = step_size * (x_val - mu) / (sigma * sigma);
                        x_val - drift + noise_scale * n
                    })
                    .collect()
            })
            .collect()
    }

    /// Generate n_samples with n_features by starting from pure noise and
    /// iteratively denoising using the reverse process.
    ///
    /// At each reverse step t, the sample is updated via:
    ///   x_{t-1} = (1 - blend) * x_t + blend * (mu + sigma * z) + noise
    /// where blend is derived from the schedule's signal-to-noise progression,
    /// z is standard normal for stochastic variation, and noise decreases to
    /// zero at t=0.
    fn generate(&self, n_samples: usize, n_features: usize, seed: u64) -> Vec<Vec<f64>> {
        if n_samples == 0 || n_features == 0 {
            return vec![];
        }

        let mut rng = ChaCha8Rng::seed_from_u64(seed);

        // Start from pure standard normal noise
        let normal = StandardNormal;
        let mut samples: Vec<Vec<f64>> = (0..n_samples)
            .map(|_| (0..n_features).map(|_| normal.sample(&mut rng)).collect())
            .collect();

        // Reverse process: denoise from t = T-1 down to 0
        // We use the schedule's alpha_bar to progressively blend toward the
        // target distribution. At each step, the blend factor increases as
        // more signal is recovered.
        let n_steps = self.schedule.n_steps();
        for t in (0..n_steps).rev() {
            let beta_t = self.schedule.betas[t];
            let alpha_t = self.schedule.alphas[t];
            let alpha_bar_t = self.schedule.alpha_bars[t];

            // Previous alpha_bar (at t-1); for t=0 this is 1.0 (fully denoised)
            let alpha_bar_prev = if t > 0 {
                self.schedule.alpha_bars[t - 1]
            } else {
                1.0
            };

            // Blend factor: how much to move toward the target at this step
            // As t decreases, alpha_bar increases, so we progressively reveal signal
            let blend = (alpha_bar_prev - alpha_bar_t).max(0.0) / (1.0 - alpha_bar_t).max(1e-12);
            let blend = blend.clamp(0.0, 1.0);

            let noise_scale = if t > 0 { beta_t.sqrt() * 0.5 } else { 0.0 };

            for row in samples.iter_mut() {
                for (j, x_val) in row.iter_mut().enumerate().take(n_features) {
                    let mu = if j < self.means.len() {
                        self.means[j]
                    } else {
                        0.0
                    };
                    let sigma = if j < self.stds.len() {
                        self.stds[j].max(1e-8)
                    } else {
                        1.0
                    };

                    // Target sample: draw from target distribution
                    let z: f64 = normal.sample(&mut rng);
                    let target_val = mu + sigma * z;

                    // Blend current noisy sample toward target
                    let denoised = (1.0 - blend) * *x_val + blend * target_val;

                    // Add small stochastic noise (diminishes to zero at t=0)
                    let noise_val: f64 = if t > 0 { normal.sample(&mut rng) } else { 0.0 };

                    // Update using DDPM-style posterior with Langevin correction
                    let correction = beta_t / (2.0 * alpha_t.max(1e-12)) * (*x_val - mu)
                        / (sigma * sigma).max(1e-12);
                    *x_val = denoised - correction + noise_scale * noise_val;
                }
            }
        }

        // Apply correlation structure via Cholesky decomposition if provided
        if let Some(ref corr_matrix) = self.correlations {
            if let Some(cholesky_l) = Self::cholesky_decomposition(corr_matrix) {
                // First standardize the samples (subtract mean, divide by std)
                let mut standardized: Vec<Vec<f64>> = samples
                    .iter()
                    .map(|row| {
                        row.iter()
                            .enumerate()
                            .map(|(j, &val)| {
                                let mu = if j < self.means.len() {
                                    self.means[j]
                                } else {
                                    0.0
                                };
                                let sigma = if j < self.stds.len() {
                                    self.stds[j].max(1e-8)
                                } else {
                                    1.0
                                };
                                (val - mu) / sigma
                            })
                            .collect()
                    })
                    .collect();

                // Apply correlation
                Self::apply_correlation(&mut standardized, &cholesky_l);

                // Denormalize back to target scale
                samples = standardized
                    .iter()
                    .map(|row| {
                        row.iter()
                            .enumerate()
                            .map(|(j, &val)| {
                                let mu = if j < self.means.len() {
                                    self.means[j]
                                } else {
                                    0.0
                                };
                                let sigma = if j < self.stds.len() {
                                    self.stds[j].max(1e-8)
                                } else {
                                    1.0
                                };
                                val * sigma + mu
                            })
                            .collect()
                    })
                    .collect();
            }
        }

        // Clip to reasonable ranges: mean +/- 4 * std
        for row in samples.iter_mut() {
            for (j, val) in row.iter_mut().enumerate() {
                let mu = if j < self.means.len() {
                    self.means[j]
                } else {
                    0.0
                };
                let sigma = if j < self.stds.len() {
                    self.stds[j]
                } else {
                    1.0
                };
                let lo = mu - 4.0 * sigma;
                let hi = mu + 4.0 * sigma;
                *val = val.clamp(lo, hi);
            }
        }

        samples
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config(n_steps: usize, seed: u64) -> DiffusionConfig {
        DiffusionConfig {
            n_steps,
            schedule: super::super::NoiseScheduleType::Linear,
            seed,
        }
    }

    #[test]
    fn test_output_dimensions() {
        let means = vec![100.0, 200.0, 300.0];
        let stds = vec![10.0, 20.0, 30.0];
        let backend = StatisticalDiffusionBackend::new(means, stds, make_config(50, 42));

        let samples = backend.generate(500, 3, 42);
        assert_eq!(samples.len(), 500);
        for row in &samples {
            assert_eq!(row.len(), 3);
        }
    }

    #[test]
    fn test_deterministic_with_same_seed() {
        let means = vec![50.0, 100.0];
        let stds = vec![5.0, 10.0];
        let backend = StatisticalDiffusionBackend::new(means, stds, make_config(50, 99));

        let samples1 = backend.generate(100, 2, 123);
        let samples2 = backend.generate(100, 2, 123);

        for (row1, row2) in samples1.iter().zip(samples2.iter()) {
            for (&v1, &v2) in row1.iter().zip(row2.iter()) {
                assert!(
                    (v1 - v2).abs() < 1e-12,
                    "Determinism failed: {} vs {}",
                    v1,
                    v2
                );
            }
        }
    }

    #[test]
    fn test_mean_within_tolerance() {
        let target_means = vec![100.0, 0.0, -50.0];
        let target_stds = vec![10.0, 5.0, 20.0];
        let backend = StatisticalDiffusionBackend::new(
            target_means.clone(),
            target_stds.clone(),
            make_config(100, 42),
        );

        let samples = backend.generate(5000, 3, 42);

        // Compute sample means
        for feat in 0..3 {
            let sample_mean: f64 =
                samples.iter().map(|r| r[feat]).sum::<f64>() / samples.len() as f64;
            let tolerance = target_stds[feat]; // within 1 std of target
            assert!(
                (sample_mean - target_means[feat]).abs() < tolerance,
                "Feature {} mean {} is more than 1 std ({}) from target {}",
                feat,
                sample_mean,
                tolerance,
                target_means[feat]
            );
        }
    }

    #[test]
    fn test_forward_adds_noise() {
        let means = vec![100.0, 200.0];
        let stds = vec![10.0, 20.0];
        let backend = StatisticalDiffusionBackend::new(means, stds, make_config(100, 42));

        let original = vec![vec![100.0, 200.0]; 100];

        // At early timestep, noise is small
        let noised_early = backend.forward(&original, 5);
        let dist_early: f64 = noised_early
            .iter()
            .zip(original.iter())
            .map(|(n, o)| {
                n.iter()
                    .zip(o.iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>()
            })
            .sum::<f64>()
            .sqrt();

        // At late timestep, noise is large
        let noised_late = backend.forward(&original, 90);
        let dist_late: f64 = noised_late
            .iter()
            .zip(original.iter())
            .map(|(n, o)| {
                n.iter()
                    .zip(o.iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>()
            })
            .sum::<f64>()
            .sqrt();

        assert!(
            dist_late > dist_early,
            "Later timestep should add more noise: early={}, late={}",
            dist_early,
            dist_late
        );
    }

    #[test]
    fn test_correlation_structure_preserved() {
        let means = vec![0.0, 0.0];
        let stds = vec![1.0, 1.0];
        // Strong positive correlation
        let corr = vec![vec![1.0, 0.9], vec![0.9, 1.0]];

        let backend = StatisticalDiffusionBackend::new(means, stds, make_config(100, 42))
            .with_correlations(corr);

        let samples = backend.generate(5000, 2, 42);

        // Compute sample correlation
        let n = samples.len() as f64;
        let mean0: f64 = samples.iter().map(|r| r[0]).sum::<f64>() / n;
        let mean1: f64 = samples.iter().map(|r| r[1]).sum::<f64>() / n;
        let std0: f64 = (samples.iter().map(|r| (r[0] - mean0).powi(2)).sum::<f64>() / n).sqrt();
        let std1: f64 = (samples.iter().map(|r| (r[1] - mean1).powi(2)).sum::<f64>() / n).sqrt();
        let cov01: f64 = samples
            .iter()
            .map(|r| (r[0] - mean0) * (r[1] - mean1))
            .sum::<f64>()
            / n;

        let sample_corr = if std0 > 1e-8 && std1 > 1e-8 {
            cov01 / (std0 * std1)
        } else {
            0.0
        };

        // Correlation should be positive and reasonably close to 0.9
        assert!(
            sample_corr > 0.5,
            "Expected positive correlation (target 0.9), got {}",
            sample_corr
        );
    }

    #[test]
    fn test_cholesky_identity() {
        let identity = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let l = StatisticalDiffusionBackend::cholesky_decomposition(&identity);
        assert!(l.is_some());
        let l = l.unwrap();
        assert!((l[0][0] - 1.0).abs() < 1e-10);
        assert!((l[1][1] - 1.0).abs() < 1e-10);
        assert!(l[0][1].abs() < 1e-10);
        assert!(l[1][0].abs() < 1e-10);
    }

    #[test]
    fn test_cholesky_non_positive_definite() {
        // Not positive definite
        let matrix = vec![vec![1.0, 2.0], vec![2.0, 1.0]];
        let l = StatisticalDiffusionBackend::cholesky_decomposition(&matrix);
        assert!(l.is_none());
    }

    #[test]
    fn test_generate_empty() {
        let backend = StatisticalDiffusionBackend::new(vec![], vec![], make_config(10, 0));
        let samples = backend.generate(0, 0, 0);
        assert!(samples.is_empty());
    }

    #[test]
    fn test_values_clipped_to_range() {
        let means = vec![0.0];
        let stds = vec![1.0];
        let backend = StatisticalDiffusionBackend::new(means, stds, make_config(50, 42));

        let samples = backend.generate(1000, 1, 42);
        for row in &samples {
            assert!(
                row[0] >= -4.0 && row[0] <= 4.0,
                "Value {} out of clipping range [-4, 4]",
                row[0]
            );
        }
    }
}
