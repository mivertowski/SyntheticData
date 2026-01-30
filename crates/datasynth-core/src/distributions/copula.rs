//! Copula implementations for modeling dependency structures.
//!
//! Copulas separate marginal distributions from dependency structure,
//! enabling generation of correlated random variables with different
//! tail dependency characteristics:
//!
//! - **Gaussian**: No tail dependence, symmetric
//! - **Clayton**: Lower tail dependence (joint failures)
//! - **Gumbel**: Upper tail dependence (joint successes)
//! - **Frank**: Symmetric, no tail dependence (like Gaussian)
//! - **Student-t**: Both tail dependencies (extreme co-movements)

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// Types of copula for dependency modeling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CopulaType {
    /// Gaussian copula - no tail dependence, symmetric
    #[default]
    Gaussian,
    /// Clayton copula - lower tail dependence (good for risk modeling)
    Clayton,
    /// Gumbel copula - upper tail dependence
    Gumbel,
    /// Frank copula - symmetric, no tail dependence
    Frank,
    /// Student-t copula - both tail dependencies
    StudentT,
}

/// Configuration for copula-based correlation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopulaConfig {
    /// Type of copula to use
    pub copula_type: CopulaType,
    /// Correlation parameter (interpretation depends on copula type)
    /// - Gaussian/Student-t: correlation coefficient (-1 to 1)
    /// - Clayton: theta > 0 (higher = stronger lower tail dependence)
    /// - Gumbel: theta >= 1 (higher = stronger upper tail dependence)
    /// - Frank: theta != 0 (|theta| higher = stronger dependence)
    pub theta: f64,
    /// Degrees of freedom for Student-t copula (only used if copula_type = StudentT)
    #[serde(default = "default_df")]
    pub degrees_of_freedom: f64,
}

fn default_df() -> f64 {
    4.0
}

impl Default for CopulaConfig {
    fn default() -> Self {
        Self {
            copula_type: CopulaType::Gaussian,
            theta: 0.5,
            degrees_of_freedom: 4.0,
        }
    }
}

impl CopulaConfig {
    /// Create a Gaussian copula configuration.
    pub fn gaussian(correlation: f64) -> Self {
        Self {
            copula_type: CopulaType::Gaussian,
            theta: correlation.clamp(-0.999, 0.999),
            degrees_of_freedom: 4.0,
        }
    }

    /// Create a Clayton copula configuration.
    pub fn clayton(theta: f64) -> Self {
        Self {
            copula_type: CopulaType::Clayton,
            theta: theta.max(0.001),
            degrees_of_freedom: 4.0,
        }
    }

    /// Create a Gumbel copula configuration.
    pub fn gumbel(theta: f64) -> Self {
        Self {
            copula_type: CopulaType::Gumbel,
            theta: theta.max(1.0),
            degrees_of_freedom: 4.0,
        }
    }

    /// Create a Frank copula configuration.
    pub fn frank(theta: f64) -> Self {
        Self {
            copula_type: CopulaType::Frank,
            theta: if theta.abs() < 0.001 { 0.001 } else { theta },
            degrees_of_freedom: 4.0,
        }
    }

    /// Create a Student-t copula configuration.
    pub fn student_t(correlation: f64, df: f64) -> Self {
        Self {
            copula_type: CopulaType::StudentT,
            theta: correlation.clamp(-0.999, 0.999),
            degrees_of_freedom: df.max(2.0),
        }
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<(), String> {
        match self.copula_type {
            CopulaType::Gaussian | CopulaType::StudentT => {
                if self.theta < -1.0 || self.theta > 1.0 {
                    return Err(format!(
                        "Correlation must be in [-1, 1], got {}",
                        self.theta
                    ));
                }
            }
            CopulaType::Clayton => {
                if self.theta <= 0.0 {
                    return Err(format!("Clayton theta must be > 0, got {}", self.theta));
                }
            }
            CopulaType::Gumbel => {
                if self.theta < 1.0 {
                    return Err(format!("Gumbel theta must be >= 1, got {}", self.theta));
                }
            }
            CopulaType::Frank => {
                if self.theta.abs() < 0.0001 {
                    return Err("Frank theta must be non-zero".to_string());
                }
            }
        }

        if self.copula_type == CopulaType::StudentT && self.degrees_of_freedom <= 0.0 {
            return Err("Degrees of freedom must be positive".to_string());
        }

        Ok(())
    }

    /// Get the implied Kendall's tau for this copula configuration.
    pub fn kendalls_tau(&self) -> f64 {
        match self.copula_type {
            CopulaType::Gaussian | CopulaType::StudentT => {
                // tau = (2/pi) * arcsin(rho)
                2.0 * self.theta.asin() / std::f64::consts::PI
            }
            CopulaType::Clayton => {
                // tau = theta / (theta + 2)
                self.theta / (self.theta + 2.0)
            }
            CopulaType::Gumbel => {
                // tau = 1 - 1/theta
                1.0 - 1.0 / self.theta
            }
            CopulaType::Frank => {
                // tau = 1 - 4/theta * (1 - D_1(theta)) where D_1 is Debye function
                // Approximation for |theta| not too large
                let abs_theta = self.theta.abs();
                if abs_theta < 10.0 {
                    1.0 - 4.0 / self.theta + 4.0 / self.theta.powi(2) * debye_1(abs_theta)
                } else {
                    // For large |theta|, tau approaches sign(theta)
                    self.theta.signum() * (1.0 - 4.0 / abs_theta)
                }
            }
        }
    }

    /// Get the lower tail dependence coefficient.
    pub fn lower_tail_dependence(&self) -> f64 {
        match self.copula_type {
            CopulaType::Gaussian | CopulaType::Frank => 0.0,
            CopulaType::Clayton => 2.0_f64.powf(-1.0 / self.theta),
            CopulaType::Gumbel => 0.0,
            CopulaType::StudentT => {
                // lambda_L = 2 * t_{df+1}(-sqrt((df+1)(1-rho)/(1+rho)))
                // Approximation for moderate df
                let nu = self.degrees_of_freedom;
                let rho = self.theta;
                let arg = ((nu + 1.0) * (1.0 - rho) / (1.0 + rho)).sqrt();
                2.0 * student_t_cdf(-arg, nu + 1.0)
            }
        }
    }

    /// Get the upper tail dependence coefficient.
    pub fn upper_tail_dependence(&self) -> f64 {
        match self.copula_type {
            CopulaType::Gaussian | CopulaType::Frank => 0.0,
            CopulaType::Clayton => 0.0,
            CopulaType::Gumbel => 2.0 - 2.0_f64.powf(1.0 / self.theta),
            CopulaType::StudentT => self.lower_tail_dependence(), // Symmetric
        }
    }
}

/// Bivariate copula sampler.
pub struct BivariateCopulaSampler {
    rng: ChaCha8Rng,
    config: CopulaConfig,
}

impl BivariateCopulaSampler {
    /// Create a new bivariate copula sampler.
    pub fn new(seed: u64, config: CopulaConfig) -> Result<Self, String> {
        config.validate()?;
        Ok(Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            config,
        })
    }

    /// Sample a pair of correlated uniform values (u, v) in [0, 1]^2.
    pub fn sample(&mut self) -> (f64, f64) {
        match self.config.copula_type {
            CopulaType::Gaussian => self.sample_gaussian(),
            CopulaType::Clayton => self.sample_clayton(),
            CopulaType::Gumbel => self.sample_gumbel(),
            CopulaType::Frank => self.sample_frank(),
            CopulaType::StudentT => self.sample_student_t(),
        }
    }

    /// Sample from Gaussian copula.
    fn sample_gaussian(&mut self) -> (f64, f64) {
        let rho = self.config.theta;

        // Generate independent standard normals
        let z1 = self.sample_standard_normal();
        let z2 = self.sample_standard_normal();

        // Correlate them
        let x1 = z1;
        let x2 = rho * z1 + (1.0 - rho.powi(2)).sqrt() * z2;

        // Transform to uniform via normal CDF
        (standard_normal_cdf(x1), standard_normal_cdf(x2))
    }

    /// Sample from Clayton copula.
    fn sample_clayton(&mut self) -> (f64, f64) {
        let theta = self.config.theta;

        // Use conditional method
        let u: f64 = self.rng.gen();
        let t: f64 = self.rng.gen();

        // v = ((u^(-theta) - 1) * t^(-theta/(theta+1)) + 1)^(-1/theta)
        let v = (u.powf(-theta) * (t.powf(-theta / (theta + 1.0)) - 1.0) + 1.0).powf(-1.0 / theta);

        (u, v.clamp(0.0, 1.0))
    }

    /// Sample from Gumbel copula.
    fn sample_gumbel(&mut self) -> (f64, f64) {
        let theta = self.config.theta;

        // Use Marshall-Olkin method with stable distribution
        // Simplified implementation using conditional method

        let u: f64 = self.rng.gen();
        let t: f64 = self.rng.gen();

        // Approximate using Gumbel stable variate
        let s = sample_positive_stable(&mut self.rng, 1.0 / theta);
        let e1 = sample_exponential(&mut self.rng, 1.0);
        let e2 = sample_exponential(&mut self.rng, 1.0);

        let v1 = (-e1 / s).exp().powf(1.0 / theta);
        let v2 = (-e2 / s).exp().powf(1.0 / theta);

        let c_u = v1 / (v1 + v2);
        let c_v = v2 / (v1 + v2);

        // Map to [0,1]
        let u_out = (-((-u.ln()).powf(theta) + (-c_u.ln()).powf(theta)).powf(1.0 / theta)).exp();
        let v_out = (-((-t.ln()).powf(theta) + (-c_v.ln()).powf(theta)).powf(1.0 / theta)).exp();

        (u_out.clamp(0.0001, 0.9999), v_out.clamp(0.0001, 0.9999))
    }

    /// Sample from Frank copula.
    fn sample_frank(&mut self) -> (f64, f64) {
        let theta = self.config.theta;

        let u: f64 = self.rng.gen();
        let t: f64 = self.rng.gen();

        // Conditional distribution inversion
        let v = -((1.0 - t)
            / (t * (-theta).exp() + (1.0 - t) * (1.0 - u * (1.0 - (-theta).exp())).recip()))
        .ln()
            / theta;

        (u, v.clamp(0.0, 1.0))
    }

    /// Sample from Student-t copula.
    fn sample_student_t(&mut self) -> (f64, f64) {
        let rho = self.config.theta;
        let nu = self.config.degrees_of_freedom;

        // Generate correlated chi-squared variate
        let chi2 = sample_chi_squared(&mut self.rng, nu);
        let scale = (nu / chi2).sqrt();

        // Generate correlated normals
        let z1 = self.sample_standard_normal();
        let z2 = self.sample_standard_normal();

        let x1 = z1 * scale;
        let x2 = (rho * z1 + (1.0 - rho.powi(2)).sqrt() * z2) * scale;

        // Transform to uniform via Student-t CDF
        (student_t_cdf(x1, nu), student_t_cdf(x2, nu))
    }

    /// Sample from standard normal using Box-Muller.
    fn sample_standard_normal(&mut self) -> f64 {
        let u1: f64 = self.rng.gen();
        let u2: f64 = self.rng.gen();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }

    /// Sample multiple pairs.
    pub fn sample_n(&mut self, n: usize) -> Vec<(f64, f64)> {
        (0..n).map(|_| self.sample()).collect()
    }

    /// Reset the sampler with a new seed.
    pub fn reset(&mut self, seed: u64) {
        self.rng = ChaCha8Rng::seed_from_u64(seed);
    }

    /// Get the configuration.
    pub fn config(&self) -> &CopulaConfig {
        &self.config
    }
}

/// Cholesky decomposition for multivariate normal generation.
pub fn cholesky_decompose(matrix: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
    let n = matrix.len();
    let mut l = vec![vec![0.0; n]; n];

    for i in 0..n {
        for j in 0..=i {
            let sum: f64 = (0..j).map(|k| l[i][k] * l[j][k]).sum();

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
pub fn standard_normal_cdf(x: f64) -> f64 {
    0.5 * (1.0 + erf(x / std::f64::consts::SQRT_2))
}

/// Standard normal quantile function (inverse CDF) approximation.
pub fn standard_normal_quantile(p: f64) -> f64 {
    // Rational approximation (Abramowitz and Stegun)
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }

    let p_low = 0.02425;
    let p_high = 1.0 - p_low;

    if p < p_low {
        // Lower tail
        let q = (-2.0 * p.ln()).sqrt();
        let c = [2.515517, 0.802853, 0.010328];
        let d = [1.432788, 0.189269, 0.001308];
        -(c[0] + c[1] * q + c[2] * q.powi(2))
            / (1.0 + d[0] * q + d[1] * q.powi(2) + d[2] * q.powi(3))
            + q
    } else if p <= p_high {
        // Central region
        let q = p - 0.5;
        let r = q * q;
        let a = [
            2.50662823884,
            -18.61500062529,
            41.39119773534,
            -25.44106049637,
        ];
        let b = [
            -8.47351093090,
            23.08336743743,
            -21.06224101826,
            3.13082909833,
        ];
        q * (a[0] + a[1] * r + a[2] * r.powi(2) + a[3] * r.powi(3))
            / (1.0 + b[0] * r + b[1] * r.powi(2) + b[2] * r.powi(3) + b[3] * r.powi(4))
    } else {
        // Upper tail
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        let c = [2.515517, 0.802853, 0.010328];
        let d = [1.432788, 0.189269, 0.001308];
        (c[0] + c[1] * q + c[2] * q.powi(2))
            / (1.0 + d[0] * q + d[1] * q.powi(2) + d[2] * q.powi(3))
            - q
    }
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

/// Student-t CDF approximation.
fn student_t_cdf(x: f64, df: f64) -> f64 {
    // Use normal approximation for large df
    if df > 30.0 {
        return standard_normal_cdf(x);
    }

    // Incomplete beta function approach
    let t2 = x * x;
    let prob = 0.5 * incomplete_beta(df / 2.0, 0.5, df / (df + t2));

    if x > 0.0 {
        1.0 - prob
    } else {
        prob
    }
}

/// Simplified incomplete beta function approximation.
fn incomplete_beta(a: f64, b: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }

    // Use continued fraction approximation
    let lbeta = ln_gamma(a) + ln_gamma(b) - ln_gamma(a + b);
    let front = (x.powf(a) * (1.0 - x).powf(b)) / lbeta.exp();

    // Lentz's algorithm for continued fraction
    let mut c: f64 = 1.0;
    let mut d: f64 = 1.0 / (1.0 - (a + b) * x / (a + 1.0)).max(1e-30);
    let mut h = d;

    for m in 1..100 {
        let m = m as f64;
        let d1 = m * (b - m) * x / ((a + 2.0 * m - 1.0) * (a + 2.0 * m));
        let d2 = -(a + m) * (a + b + m) * x / ((a + 2.0 * m) * (a + 2.0 * m + 1.0));

        d = 1.0 / (1.0 + d1 * d).max(1e-30);
        c = 1.0 + d1 / c.max(1e-30);
        h *= c * d;

        d = 1.0 / (1.0 + d2 * d).max(1e-30);
        c = 1.0 + d2 / c.max(1e-30);
        h *= c * d;

        if ((c * d) - 1.0).abs() < 1e-8 {
            break;
        }
    }

    front * h / a
}

/// Log gamma function approximation (Stirling).
fn ln_gamma(x: f64) -> f64 {
    if x <= 0.0 {
        return f64::INFINITY;
    }
    0.5 * (2.0 * std::f64::consts::PI / x).ln() + x * ((x + 1.0 / (12.0 * x)).ln() - 1.0)
}

/// Debye function D_1(x) for Frank copula.
fn debye_1(x: f64) -> f64 {
    if x.abs() < 0.01 {
        return 1.0 - x / 4.0 + x.powi(2) / 36.0;
    }

    // Numerical integration
    let n = 100;
    let h = x / n as f64;
    let mut sum = 0.0;

    for i in 1..n {
        let t = i as f64 * h;
        sum += t / (t.exp() - 1.0);
    }

    (sum + 0.5 * (h / (h.exp() - 1.0) + x / (x.exp() - 1.0))) * h / x
}

/// Sample from exponential distribution.
fn sample_exponential(rng: &mut ChaCha8Rng, lambda: f64) -> f64 {
    let u: f64 = rng.gen();
    -u.ln() / lambda
}

/// Sample from chi-squared distribution using sum of squared normals.
fn sample_chi_squared(rng: &mut ChaCha8Rng, df: f64) -> f64 {
    let n = df.floor() as usize;
    let mut sum = 0.0;
    for _ in 0..n {
        let u1: f64 = rng.gen();
        let u2: f64 = rng.gen();
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        sum += z * z;
    }
    sum
}

/// Sample from positive stable distribution (for Gumbel copula).
fn sample_positive_stable(rng: &mut ChaCha8Rng, alpha: f64) -> f64 {
    if (alpha - 1.0).abs() < 0.001 {
        return 1.0;
    }

    let u: f64 = rng.gen::<f64>() * std::f64::consts::PI - std::f64::consts::PI / 2.0;
    let e = sample_exponential(rng, 1.0);

    let b = (std::f64::consts::PI * alpha / 2.0).tan();
    let s = (1.0 + b * b).powf(1.0 / (2.0 * alpha));

    let term1 = (alpha * u).sin();
    let term2 = (u.cos()).powf(1.0 / alpha);
    let term3 = ((1.0 - alpha) * u).cos() / e;

    s * term1 / term2 * term3.powf((1.0 - alpha) / alpha)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copula_validation() {
        let gaussian = CopulaConfig::gaussian(0.5);
        assert!(gaussian.validate().is_ok());

        // Note: gaussian() constructor clamps to valid range, so create invalid config directly
        let invalid_gaussian = CopulaConfig {
            copula_type: CopulaType::Gaussian,
            theta: 1.5, // Invalid: > 1.0
            degrees_of_freedom: 4.0,
        };
        assert!(invalid_gaussian.validate().is_err());

        let clayton = CopulaConfig::clayton(2.0);
        assert!(clayton.validate().is_ok());

        // Note: clayton() constructor clamps to valid range, so create invalid config directly
        let invalid_clayton = CopulaConfig {
            copula_type: CopulaType::Clayton,
            theta: -1.0, // Invalid: must be > 0
            degrees_of_freedom: 4.0,
        };
        assert!(invalid_clayton.validate().is_err());

        let gumbel = CopulaConfig::gumbel(2.0);
        assert!(gumbel.validate().is_ok());

        // Note: gumbel() constructor clamps to valid range, so create invalid config directly
        let invalid_gumbel = CopulaConfig {
            copula_type: CopulaType::Gumbel,
            theta: 0.5, // Invalid: must be >= 1
            degrees_of_freedom: 4.0,
        };
        assert!(invalid_gumbel.validate().is_err());
    }

    #[test]
    fn test_gaussian_copula_sampling() {
        let config = CopulaConfig::gaussian(0.7);
        let mut sampler = BivariateCopulaSampler::new(42, config).unwrap();

        let samples = sampler.sample_n(1000);
        assert_eq!(samples.len(), 1000);

        // All samples should be in [0, 1]^2
        assert!(samples
            .iter()
            .all(|(u, v)| *u >= 0.0 && *u <= 1.0 && *v >= 0.0 && *v <= 1.0));

        // Verify positive correlation
        let mean_u: f64 = samples.iter().map(|(u, _)| u).sum::<f64>() / 1000.0;
        let mean_v: f64 = samples.iter().map(|(_, v)| v).sum::<f64>() / 1000.0;
        let covariance: f64 = samples
            .iter()
            .map(|(u, v)| (u - mean_u) * (v - mean_v))
            .sum::<f64>()
            / 1000.0;

        assert!(covariance > 0.0); // Positive correlation expected
    }

    #[test]
    fn test_copula_determinism() {
        let config = CopulaConfig::gaussian(0.5);

        let mut sampler1 = BivariateCopulaSampler::new(42, config.clone()).unwrap();
        let mut sampler2 = BivariateCopulaSampler::new(42, config).unwrap();

        for _ in 0..100 {
            assert_eq!(sampler1.sample(), sampler2.sample());
        }
    }

    #[test]
    fn test_kendalls_tau() {
        // Gaussian: tau = (2/pi) * arcsin(rho)
        let gaussian = CopulaConfig::gaussian(0.5);
        let tau = gaussian.kendalls_tau();
        let expected = 2.0 * (0.5_f64).asin() / std::f64::consts::PI;
        assert!((tau - expected).abs() < 0.001);

        // Clayton: tau = theta / (theta + 2)
        let clayton = CopulaConfig::clayton(2.0);
        let tau = clayton.kendalls_tau();
        assert!((tau - 0.5).abs() < 0.001);

        // Gumbel: tau = 1 - 1/theta
        let gumbel = CopulaConfig::gumbel(2.0);
        let tau = gumbel.kendalls_tau();
        assert!((tau - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_tail_dependence() {
        // Gaussian has no tail dependence
        let gaussian = CopulaConfig::gaussian(0.7);
        assert_eq!(gaussian.lower_tail_dependence(), 0.0);
        assert_eq!(gaussian.upper_tail_dependence(), 0.0);

        // Clayton has lower tail dependence
        let clayton = CopulaConfig::clayton(2.0);
        assert!(clayton.lower_tail_dependence() > 0.0);
        assert_eq!(clayton.upper_tail_dependence(), 0.0);

        // Gumbel has upper tail dependence
        let gumbel = CopulaConfig::gumbel(2.0);
        assert_eq!(gumbel.lower_tail_dependence(), 0.0);
        assert!(gumbel.upper_tail_dependence() > 0.0);
    }

    #[test]
    fn test_cholesky_decomposition() {
        let matrix = vec![vec![1.0, 0.5], vec![0.5, 1.0]];
        let l = cholesky_decompose(&matrix).unwrap();

        // Verify L * L^T = A
        let reconstructed_00 = l[0][0] * l[0][0];
        let reconstructed_01 = l[0][0] * l[1][0];
        let reconstructed_11 = l[1][0] * l[1][0] + l[1][1] * l[1][1];

        assert!((reconstructed_00 - 1.0).abs() < 0.001);
        assert!((reconstructed_01 - 0.5).abs() < 0.001);
        assert!((reconstructed_11 - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_standard_normal_cdf() {
        assert!((standard_normal_cdf(0.0) - 0.5).abs() < 0.001);
        assert!(standard_normal_cdf(-3.0) < 0.01);
        assert!(standard_normal_cdf(3.0) > 0.99);
    }

    #[test]
    fn test_clayton_copula() {
        let config = CopulaConfig::clayton(2.0);
        let mut sampler = BivariateCopulaSampler::new(42, config).unwrap();

        let samples = sampler.sample_n(1000);
        assert!(samples
            .iter()
            .all(|(u, v)| *u >= 0.0 && *u <= 1.0 && *v >= 0.0 && *v <= 1.0));
    }

    #[test]
    fn test_frank_copula() {
        let config = CopulaConfig::frank(5.0);
        let mut sampler = BivariateCopulaSampler::new(42, config).unwrap();

        let samples = sampler.sample_n(1000);
        assert!(samples
            .iter()
            .all(|(u, v)| *u >= 0.0 && *u <= 1.0 && *v >= 0.0 && *v <= 1.0));
    }
}
