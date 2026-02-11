use serde::{Deserialize, Serialize};

use super::schedule::NoiseSchedule;

/// Diffusion model backend trait.
pub trait DiffusionBackend: Send + Sync {
    /// Backend name.
    fn name(&self) -> &str;
    /// Forward process: add noise at timestep t.
    fn forward(&self, x: &[Vec<f64>], t: usize) -> Vec<Vec<f64>>;
    /// Reverse process: denoise at timestep t.
    fn reverse(&self, x_t: &[Vec<f64>], t: usize) -> Vec<Vec<f64>>;
    /// Generate n_samples with n_features from noise.
    fn generate(&self, n_samples: usize, n_features: usize, seed: u64) -> Vec<Vec<f64>>;
}

/// Diffusion configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffusionConfig {
    /// Number of diffusion steps.
    #[serde(default = "default_n_steps")]
    pub n_steps: usize,
    /// Noise schedule type.
    #[serde(default)]
    pub schedule: NoiseScheduleType,
    /// Random seed.
    #[serde(default)]
    pub seed: u64,
}

fn default_n_steps() -> usize {
    1000
}

impl Default for DiffusionConfig {
    fn default() -> Self {
        Self {
            n_steps: default_n_steps(),
            schedule: NoiseScheduleType::default(),
            seed: 0,
        }
    }
}

impl DiffusionConfig {
    /// Build a noise schedule from this configuration.
    pub fn build_schedule(&self) -> NoiseSchedule {
        NoiseSchedule::new(&self.schedule, self.n_steps)
    }
}

/// Noise schedule type.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoiseScheduleType {
    #[default]
    Linear,
    Cosine,
    Sigmoid,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_diffusion_config_default() {
        let config = DiffusionConfig::default();
        assert_eq!(config.n_steps, 1000);
        assert!(matches!(config.schedule, NoiseScheduleType::Linear));
    }

    #[test]
    fn test_diffusion_config_serde() {
        let config = DiffusionConfig {
            n_steps: 500,
            schedule: NoiseScheduleType::Cosine,
            seed: 42,
        };
        let json = serde_json::to_string(&config).unwrap();
        let parsed: DiffusionConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.n_steps, 500);
        assert_eq!(parsed.seed, 42);
    }

    #[test]
    fn test_build_schedule() {
        let config = DiffusionConfig {
            n_steps: 100,
            schedule: NoiseScheduleType::Cosine,
            seed: 0,
        };
        let schedule = config.build_schedule();
        assert_eq!(schedule.n_steps(), 100);
    }
}
