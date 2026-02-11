use super::backend::NoiseScheduleType;

/// Precomputed noise schedule parameters.
///
/// For T timesteps, provides:
/// - `betas`: noise level at each step
/// - `alphas`: 1 - beta_t
/// - `alpha_bars`: cumulative product of alphas (signal retention)
#[derive(Debug, Clone)]
pub struct NoiseSchedule {
    pub betas: Vec<f64>,
    pub alphas: Vec<f64>,
    pub alpha_bars: Vec<f64>,
    pub sqrt_alpha_bars: Vec<f64>,
    pub sqrt_one_minus_alpha_bars: Vec<f64>,
}

impl NoiseSchedule {
    /// Create a noise schedule of the given type and length.
    pub fn new(schedule_type: &NoiseScheduleType, n_steps: usize) -> Self {
        let betas = match schedule_type {
            NoiseScheduleType::Linear => Self::linear_schedule(n_steps),
            NoiseScheduleType::Cosine => Self::cosine_schedule(n_steps),
            NoiseScheduleType::Sigmoid => Self::sigmoid_schedule(n_steps),
        };
        Self::from_betas(betas)
    }

    /// Build schedule from a vector of betas.
    pub fn from_betas(betas: Vec<f64>) -> Self {
        let alphas: Vec<f64> = betas.iter().map(|b| 1.0 - b).collect();

        let mut alpha_bars = Vec::with_capacity(alphas.len());
        let mut cumulative = 1.0;
        for &a in &alphas {
            cumulative *= a;
            alpha_bars.push(cumulative);
        }

        let sqrt_alpha_bars: Vec<f64> = alpha_bars.iter().map(|a| a.sqrt()).collect();
        let sqrt_one_minus_alpha_bars: Vec<f64> =
            alpha_bars.iter().map(|a| (1.0 - a).sqrt()).collect();

        Self {
            betas,
            alphas,
            alpha_bars,
            sqrt_alpha_bars,
            sqrt_one_minus_alpha_bars,
        }
    }

    /// Linear noise schedule: beta linearly interpolated from beta_start to beta_end.
    fn linear_schedule(n_steps: usize) -> Vec<f64> {
        let beta_start = 0.0001;
        let beta_end = 0.02;
        (0..n_steps)
            .map(|i| {
                beta_start + (beta_end - beta_start) * (i as f64) / ((n_steps - 1).max(1) as f64)
            })
            .collect()
    }

    /// Cosine noise schedule: alpha_bar_t = cos^2((t/T + s) / (1+s) * pi/2).
    fn cosine_schedule(n_steps: usize) -> Vec<f64> {
        let s = 0.008;
        let mut alpha_bars = Vec::with_capacity(n_steps + 1);
        for i in 0..=n_steps {
            let t = i as f64 / n_steps as f64;
            let val = ((t + s) / (1.0 + s) * std::f64::consts::FRAC_PI_2)
                .cos()
                .powi(2);
            alpha_bars.push(val);
        }

        let mut betas = Vec::with_capacity(n_steps);
        for i in 1..=n_steps {
            let beta = 1.0 - alpha_bars[i] / alpha_bars[i - 1];
            betas.push(beta.clamp(0.0001, 0.999));
        }
        betas
    }

    /// Sigmoid noise schedule: beta interpolated via sigmoid curve.
    fn sigmoid_schedule(n_steps: usize) -> Vec<f64> {
        let beta_start = 0.0001;
        let beta_end = 0.02;
        let range_start = -6.0;
        let range_end = 6.0;

        (0..n_steps)
            .map(|i| {
                let t = range_start
                    + (range_end - range_start) * (i as f64) / ((n_steps - 1).max(1) as f64);
                let sigmoid = 1.0 / (1.0 + (-t).exp());
                beta_start + (beta_end - beta_start) * sigmoid
            })
            .collect()
    }

    /// Number of timesteps.
    pub fn n_steps(&self) -> usize {
        self.betas.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_schedule_monotonic_betas() {
        let schedule = NoiseSchedule::new(&NoiseScheduleType::Linear, 100);
        for i in 1..schedule.betas.len() {
            assert!(
                schedule.betas[i] >= schedule.betas[i - 1],
                "Linear betas should be monotonically increasing"
            );
        }
    }

    #[test]
    fn test_cosine_schedule_alpha_bar_decreasing() {
        let schedule = NoiseSchedule::new(&NoiseScheduleType::Cosine, 100);
        // Alpha bars should decrease (signal retention decreases over time)
        assert!(
            schedule.alpha_bars[0] > 0.9,
            "First alpha_bar should be near 1.0"
        );
        assert!(
            schedule.alpha_bars.last().copied().unwrap_or(1.0) < 0.1,
            "Last alpha_bar should be near 0.0"
        );
        for i in 1..schedule.alpha_bars.len() {
            assert!(
                schedule.alpha_bars[i] <= schedule.alpha_bars[i - 1],
                "Alpha bars should be monotonically decreasing"
            );
        }
    }

    #[test]
    fn test_sigmoid_schedule_bounded() {
        let schedule = NoiseSchedule::new(&NoiseScheduleType::Sigmoid, 100);
        for &beta in &schedule.betas {
            assert!(
                beta >= 0.0001 && beta <= 0.02,
                "Sigmoid betas should be within [0.0001, 0.02], got {}",
                beta
            );
        }
    }

    #[test]
    fn test_schedule_lengths() {
        for n in [10, 100, 1000] {
            let schedule = NoiseSchedule::new(&NoiseScheduleType::Linear, n);
            assert_eq!(schedule.betas.len(), n);
            assert_eq!(schedule.alphas.len(), n);
            assert_eq!(schedule.alpha_bars.len(), n);
            assert_eq!(schedule.sqrt_alpha_bars.len(), n);
            assert_eq!(schedule.sqrt_one_minus_alpha_bars.len(), n);
        }
    }

    #[test]
    fn test_alpha_bar_product_correctness() {
        let schedule = NoiseSchedule::new(&NoiseScheduleType::Linear, 10);
        // Verify alpha_bar[i] = product of alphas[0..=i]
        let mut product = 1.0;
        for i in 0..schedule.alphas.len() {
            product *= schedule.alphas[i];
            assert!(
                (schedule.alpha_bars[i] - product).abs() < 1e-10,
                "Alpha bar mismatch at step {}",
                i
            );
        }
    }

    #[test]
    fn test_sqrt_consistency() {
        let schedule = NoiseSchedule::new(&NoiseScheduleType::Linear, 50);
        for i in 0..schedule.alpha_bars.len() {
            assert!((schedule.sqrt_alpha_bars[i] - schedule.alpha_bars[i].sqrt()).abs() < 1e-10);
            assert!(
                (schedule.sqrt_one_minus_alpha_bars[i] - (1.0 - schedule.alpha_bars[i]).sqrt())
                    .abs()
                    < 1e-10
            );
        }
    }
}
