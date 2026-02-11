//! Temporal attribute generator implementation.
//!
//! Provides generation of temporal attributes for entities, supporting
//! bi-temporal data models.

use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, Utc};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use datasynth_core::models::{BiTemporal, TemporalChangeType, TemporalVersionChain};

/// Configuration for temporal attribute generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalAttributeConfig {
    /// Enable temporal attribute generation.
    pub enabled: bool,
    /// Valid time configuration.
    pub valid_time: ValidTimeConfig,
    /// Transaction time configuration.
    pub transaction_time: TransactionTimeConfig,
    /// Generate version chains for entities.
    pub generate_version_chains: bool,
    /// Average number of versions per entity.
    pub avg_versions_per_entity: f64,
}

impl Default for TemporalAttributeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            valid_time: ValidTimeConfig::default(),
            transaction_time: TransactionTimeConfig::default(),
            generate_version_chains: false,
            avg_versions_per_entity: 1.5,
        }
    }
}

/// Configuration for valid time (business time) generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidTimeConfig {
    /// Probability that valid_to is set (entity has ended validity).
    pub closed_probability: f64,
    /// Average validity duration in days.
    pub avg_validity_days: u32,
    /// Standard deviation of validity duration in days.
    pub validity_stddev_days: u32,
}

impl Default for ValidTimeConfig {
    fn default() -> Self {
        Self {
            closed_probability: 0.1,
            avg_validity_days: 365,
            validity_stddev_days: 90,
        }
    }
}

/// Configuration for transaction time (system time) generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionTimeConfig {
    /// Average recording delay in seconds (0 = immediate).
    pub avg_recording_delay_seconds: u32,
    /// Allow backdating (recording time before valid time).
    pub allow_backdating: bool,
    /// Probability of backdating if allowed.
    pub backdating_probability: f64,
    /// Maximum backdate days.
    pub max_backdate_days: u32,
}

impl Default for TransactionTimeConfig {
    fn default() -> Self {
        Self {
            avg_recording_delay_seconds: 0,
            allow_backdating: false,
            backdating_probability: 0.01,
            max_backdate_days: 30,
        }
    }
}

/// Generator for temporal attributes.
pub struct TemporalAttributeGenerator {
    /// Configuration.
    config: TemporalAttributeConfig,
    /// Random number generator.
    rng: ChaCha8Rng,
    /// Base date for generation.
    base_date: NaiveDate,
    /// Generation count.
    count: u64,
}

impl TemporalAttributeGenerator {
    /// Creates a new temporal attribute generator.
    pub fn new(config: TemporalAttributeConfig, seed: u64, base_date: NaiveDate) -> Self {
        Self {
            config,
            rng: ChaCha8Rng::seed_from_u64(seed),
            base_date,
            count: 0,
        }
    }

    /// Creates a generator with default configuration.
    pub fn with_defaults(seed: u64, base_date: NaiveDate) -> Self {
        Self::new(TemporalAttributeConfig::default(), seed, base_date)
    }

    /// Wraps an entity with temporal attributes.
    pub fn generate_temporal<T: Clone>(&mut self, entity: T) -> BiTemporal<T> {
        self.count += 1;

        let (valid_from, valid_to) = self.generate_valid_time();
        let transaction_time = self.generate_transaction_time(valid_from);

        let recorded_by = format!("system_{}", self.rng.gen_range(1..=100));
        let mut temporal = BiTemporal::new(entity)
            .with_valid_time(valid_from, valid_to)
            .with_recorded_at(transaction_time)
            .with_recorded_by(&recorded_by)
            .with_change_type(TemporalChangeType::Original);

        // Optionally add a change reason
        if self.rng.gen_bool(0.2) {
            temporal = temporal.with_change_reason("Initial creation");
        }

        temporal
    }

    /// Generates a version chain for an entity.
    pub fn generate_version_chain<T: Clone>(
        &mut self,
        entity: T,
        id: Uuid,
    ) -> TemporalVersionChain<T> {
        // Determine number of versions
        let num_versions = if self.config.generate_version_chains {
            let base_versions = self.config.avg_versions_per_entity;
            // Poisson-like distribution
            let lambda = base_versions;
            let mut count = 0;
            let mut p = 1.0;
            let l = (-lambda).exp();
            loop {
                count += 1;
                p *= self.rng.gen::<f64>();
                if p <= l {
                    break;
                }
            }
            count.max(1)
        } else {
            1
        };

        // Generate initial version
        let initial_temporal = self.generate_temporal(entity.clone());
        let mut chain = TemporalVersionChain::new(id, initial_temporal);

        // Generate subsequent versions
        let current_entity = entity;
        for i in 1..num_versions {
            // Each version is a correction or adjustment
            let change_type = if i == num_versions - 1 && self.rng.gen_bool(0.1) {
                TemporalChangeType::Reversal
            } else if self.rng.gen_bool(0.3) {
                TemporalChangeType::Correction
            } else {
                TemporalChangeType::Adjustment
            };

            let version = self.generate_version(current_entity.clone(), change_type);
            chain.add_version(version);
        }

        chain
    }

    /// Generates a new version of an entity.
    fn generate_version<T: Clone>(
        &mut self,
        entity: T,
        change_type: TemporalChangeType,
    ) -> BiTemporal<T> {
        let (valid_from, valid_to) = self.generate_valid_time();
        let transaction_time = self.generate_transaction_time(valid_from);

        let reason: Option<&str> = match change_type {
            TemporalChangeType::Correction => Some("Data correction"),
            TemporalChangeType::Adjustment => Some("Adjustment per policy"),
            TemporalChangeType::Reversal => Some("Reversed entry"),
            _ => None,
        };

        let recorded_by = format!("user_{}", self.rng.gen_range(1..=50));
        let mut temporal = BiTemporal::new(entity)
            .with_valid_time(valid_from, valid_to)
            .with_recorded_at(transaction_time)
            .with_recorded_by(&recorded_by)
            .with_change_type(change_type);

        if let Some(r) = reason {
            temporal = temporal.with_change_reason(r);
        }

        temporal
    }

    /// Generates valid time (business time) attributes.
    pub fn generate_valid_time(&mut self) -> (NaiveDateTime, Option<NaiveDateTime>) {
        // Generate valid_from within a reasonable range from base_date
        let days_offset = self.rng.gen_range(-365..=365);
        let valid_from_date = self.base_date + Duration::days(days_offset as i64);
        let valid_from = valid_from_date
            .and_hms_opt(
                self.rng.gen_range(0..24),
                self.rng.gen_range(0..60),
                self.rng.gen_range(0..60),
            )
            .expect("valid h/m/s ranges");

        // Determine if validity is closed
        let valid_to = if self.rng.gen_bool(self.config.valid_time.closed_probability) {
            // Generate validity duration
            let avg_days = self.config.valid_time.avg_validity_days as f64;
            let stddev_days = self.config.valid_time.validity_stddev_days as f64;

            // Normal distribution for duration
            let duration_days = (avg_days + self.rng.gen::<f64>() * stddev_days * 2.0 - stddev_days)
                .max(1.0) as i64;

            Some(valid_from + Duration::days(duration_days))
        } else {
            None
        };

        (valid_from, valid_to)
    }

    /// Generates transaction time (system time) based on valid time.
    pub fn generate_transaction_time(&mut self, valid_from: NaiveDateTime) -> DateTime<Utc> {
        let base_time = DateTime::<Utc>::from_naive_utc_and_offset(valid_from, Utc);

        // Add recording delay
        let delay_secs = if self.config.transaction_time.avg_recording_delay_seconds > 0 {
            let avg = self.config.transaction_time.avg_recording_delay_seconds as f64;
            // Exponential distribution for delay
            let delay = -avg * self.rng.gen::<f64>().ln();
            delay as i64
        } else {
            0
        };

        let recorded_at = base_time + Duration::seconds(delay_secs);

        // Handle backdating
        if self.config.transaction_time.allow_backdating
            && self
                .rng
                .gen_bool(self.config.transaction_time.backdating_probability)
        {
            let backdate_days = self
                .rng
                .gen_range(1..=self.config.transaction_time.max_backdate_days)
                as i64;
            recorded_at - Duration::days(backdate_days)
        } else {
            recorded_at
        }
    }

    /// Returns the number of entities processed.
    pub fn count(&self) -> u64 {
        self.count
    }

    /// Resets the generator.
    pub fn reset(&mut self, seed: u64) {
        self.rng = ChaCha8Rng::seed_from_u64(seed);
        self.count = 0;
    }

    /// Returns the configuration.
    pub fn config(&self) -> &TemporalAttributeConfig {
        &self.config
    }
}

/// Builder for temporal attribute configuration.
pub struct TemporalAttributeConfigBuilder {
    config: TemporalAttributeConfig,
}

impl TemporalAttributeConfigBuilder {
    /// Creates a new builder with default values.
    pub fn new() -> Self {
        Self {
            config: TemporalAttributeConfig::default(),
        }
    }

    /// Sets whether temporal attributes are enabled.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    /// Sets the probability of closed validity.
    pub fn closed_probability(mut self, prob: f64) -> Self {
        self.config.valid_time.closed_probability = prob.clamp(0.0, 1.0);
        self
    }

    /// Sets the average validity duration in days.
    pub fn avg_validity_days(mut self, days: u32) -> Self {
        self.config.valid_time.avg_validity_days = days;
        self
    }

    /// Sets the average recording delay in seconds.
    pub fn avg_recording_delay(mut self, seconds: u32) -> Self {
        self.config.transaction_time.avg_recording_delay_seconds = seconds;
        self
    }

    /// Enables backdating with the given probability.
    pub fn allow_backdating(mut self, prob: f64) -> Self {
        self.config.transaction_time.allow_backdating = true;
        self.config.transaction_time.backdating_probability = prob.clamp(0.0, 1.0);
        self
    }

    /// Enables version chain generation.
    pub fn with_version_chains(mut self, avg_versions: f64) -> Self {
        self.config.generate_version_chains = true;
        self.config.avg_versions_per_entity = avg_versions.max(1.0);
        self
    }

    /// Builds the configuration.
    pub fn build(self) -> TemporalAttributeConfig {
        self.config
    }
}

impl Default for TemporalAttributeConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_temporal() {
        let base_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let mut generator = TemporalAttributeGenerator::with_defaults(42, base_date);

        let entity = "test_entity";
        let temporal = generator.generate_temporal(entity.to_string());

        assert_eq!(temporal.data, "test_entity");
        assert!(temporal.recorded_at > DateTime::<Utc>::MIN_UTC);
        assert_eq!(temporal.change_type, TemporalChangeType::Original);
    }

    #[test]
    fn test_generate_valid_time() {
        let base_date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let config = TemporalAttributeConfig {
            valid_time: ValidTimeConfig {
                closed_probability: 0.5, // 50% chance of closed
                avg_validity_days: 30,
                validity_stddev_days: 10,
            },
            ..Default::default()
        };
        let mut generator = TemporalAttributeGenerator::new(config, 42, base_date);

        let mut has_closed = false;
        let mut has_open = false;

        for _ in 0..100 {
            let (valid_from, valid_to) = generator.generate_valid_time();
            assert!(valid_from.date() >= base_date - Duration::days(365));

            if valid_to.is_some() {
                has_closed = true;
                assert!(valid_to.unwrap() > valid_from);
            } else {
                has_open = true;
            }
        }

        // With 50% probability, should have both
        assert!(has_closed);
        assert!(has_open);
    }

    #[test]
    fn test_generate_transaction_time() {
        let base_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let config = TemporalAttributeConfig {
            transaction_time: TransactionTimeConfig {
                avg_recording_delay_seconds: 3600, // 1 hour average delay
                allow_backdating: false,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut generator = TemporalAttributeGenerator::new(config, 42, base_date);

        let valid_from = DateTime::from_timestamp(1704067200, 0).unwrap().naive_utc();
        let transaction_time = generator.generate_transaction_time(valid_from);

        // Transaction time should be >= valid_from when backdating is disabled
        let valid_from_utc = DateTime::<Utc>::from_naive_utc_and_offset(valid_from, Utc);
        assert!(transaction_time >= valid_from_utc);
    }

    #[test]
    fn test_generate_version_chain() {
        let base_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let config = TemporalAttributeConfig {
            generate_version_chains: true,
            avg_versions_per_entity: 3.0,
            ..Default::default()
        };
        let mut generator = TemporalAttributeGenerator::new(config, 42, base_date);

        let entity = "test_entity";
        let chain = generator.generate_version_chain(entity.to_string(), Uuid::new_v4());

        assert!(!chain.all_versions().is_empty());
        // Should have at least 1 version
        assert!(!chain.all_versions().is_empty());
    }

    #[test]
    fn test_config_builder() {
        let config = TemporalAttributeConfigBuilder::new()
            .enabled(true)
            .closed_probability(0.3)
            .avg_validity_days(180)
            .avg_recording_delay(60)
            .allow_backdating(0.05)
            .with_version_chains(2.5)
            .build();

        assert!(config.enabled);
        assert_eq!(config.valid_time.closed_probability, 0.3);
        assert_eq!(config.valid_time.avg_validity_days, 180);
        assert_eq!(config.transaction_time.avg_recording_delay_seconds, 60);
        assert!(config.transaction_time.allow_backdating);
        assert_eq!(config.transaction_time.backdating_probability, 0.05);
        assert!(config.generate_version_chains);
        assert_eq!(config.avg_versions_per_entity, 2.5);
    }

    #[test]
    fn test_generator_count() {
        let base_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let mut generator = TemporalAttributeGenerator::with_defaults(42, base_date);

        assert_eq!(generator.count(), 0);

        for _ in 0..5 {
            generator.generate_temporal("entity".to_string());
        }

        assert_eq!(generator.count(), 5);

        generator.reset(42);
        assert_eq!(generator.count(), 0);
    }
}
