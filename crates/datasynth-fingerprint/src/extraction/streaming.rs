//! Streaming statistics computation using online algorithms.
//!
//! This module provides algorithms for computing statistics incrementally
//! without loading all data into memory, enabling extraction from large datasets.

use std::collections::HashMap;

/// Streaming statistics accumulator for numeric data.
///
/// Uses Welford's online algorithm for numerically stable computation
/// of mean and variance in a single pass.
#[derive(Debug, Clone)]
pub struct StreamingNumericStats {
    /// Number of values seen.
    count: u64,
    /// Running mean.
    mean: f64,
    /// Running M2 (sum of squares of differences from the mean).
    m2: f64,
    /// Minimum value seen.
    min: f64,
    /// Maximum value seen.
    max: f64,
    /// Sum of values (for total computation).
    sum: f64,
    /// Count of zero values.
    zero_count: u64,
    /// Count of negative values.
    negative_count: u64,
    /// Benford first digit counts.
    benford_counts: [u64; 9],
    /// Reservoir sample for percentile estimation.
    reservoir: Vec<f64>,
    /// Maximum reservoir size.
    reservoir_capacity: usize,
    /// Total items seen (for reservoir sampling).
    total_seen: u64,
}

impl StreamingNumericStats {
    /// Create a new streaming stats accumulator.
    pub fn new(reservoir_capacity: usize) -> Self {
        Self {
            count: 0,
            mean: 0.0,
            m2: 0.0,
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
            sum: 0.0,
            zero_count: 0,
            negative_count: 0,
            benford_counts: [0; 9],
            reservoir: Vec::with_capacity(reservoir_capacity),
            reservoir_capacity,
            total_seen: 0,
        }
    }

    /// Add a value to the accumulator.
    pub fn add(&mut self, value: f64) {
        self.count += 1;
        self.total_seen += 1;
        self.sum += value;

        // Update min/max
        if value < self.min {
            self.min = value;
        }
        if value > self.max {
            self.max = value;
        }

        // Welford's algorithm for online mean and variance
        let delta = value - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = value - self.mean;
        self.m2 += delta * delta2;

        // Count zeros and negatives
        if value == 0.0 {
            self.zero_count += 1;
        }
        if value < 0.0 {
            self.negative_count += 1;
        }

        // Benford's law first digit
        let abs_value = value.abs();
        if abs_value > 0.0 {
            let s = format!("{:.15}", abs_value);
            for c in s.chars() {
                if c.is_ascii_digit() && c != '0' {
                    if let Some(digit) = c.to_digit(10) {
                        if (1..=9).contains(&digit) {
                            self.benford_counts[(digit - 1) as usize] += 1;
                        }
                    }
                    break;
                }
            }
        }

        // Reservoir sampling for percentiles
        if self.reservoir.len() < self.reservoir_capacity {
            self.reservoir.push(value);
        } else {
            // Random replacement with probability k/n
            let j = rand::random::<u64>() % self.total_seen;
            if j < self.reservoir_capacity as u64 {
                self.reservoir[j as usize] = value;
            }
        }
    }

    /// Add a batch of values.
    pub fn add_batch(&mut self, values: &[f64]) {
        for &value in values {
            self.add(value);
        }
    }

    /// Get the count.
    pub fn count(&self) -> u64 {
        self.count
    }

    /// Get the mean.
    pub fn mean(&self) -> f64 {
        self.mean
    }

    /// Get the variance.
    pub fn variance(&self) -> f64 {
        if self.count < 2 {
            0.0
        } else {
            self.m2 / self.count as f64
        }
    }

    /// Get the standard deviation.
    pub fn std_dev(&self) -> f64 {
        self.variance().sqrt()
    }

    /// Get the minimum value.
    pub fn min(&self) -> f64 {
        if self.min.is_infinite() {
            0.0
        } else {
            self.min
        }
    }

    /// Get the maximum value.
    pub fn max(&self) -> f64 {
        if self.max.is_infinite() {
            0.0
        } else {
            self.max
        }
    }

    /// Get the zero rate.
    pub fn zero_rate(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.zero_count as f64 / self.count as f64
        }
    }

    /// Get the negative rate.
    pub fn negative_rate(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.negative_count as f64 / self.count as f64
        }
    }

    /// Get the Benford first digit distribution.
    pub fn benford_distribution(&self) -> [f64; 9] {
        let total: u64 = self.benford_counts.iter().sum();
        if total == 0 {
            return [0.0; 9];
        }
        let mut dist = [0.0; 9];
        for i in 0..9 {
            dist[i] = self.benford_counts[i] as f64 / total as f64;
        }
        dist
    }

    /// Estimate percentiles from the reservoir sample.
    pub fn percentiles(&self) -> crate::models::Percentiles {
        let mut sorted = self.reservoir.clone();
        sorted.sort_by(|a, b| a.total_cmp(b));

        fn percentile(sorted: &[f64], p: f64) -> f64 {
            if sorted.is_empty() {
                return 0.0;
            }
            let idx = (p / 100.0 * (sorted.len() - 1) as f64).round() as usize;
            sorted[idx.min(sorted.len() - 1)]
        }

        crate::models::Percentiles {
            p1: percentile(&sorted, 1.0),
            p5: percentile(&sorted, 5.0),
            p10: percentile(&sorted, 10.0),
            p25: percentile(&sorted, 25.0),
            p50: percentile(&sorted, 50.0),
            p75: percentile(&sorted, 75.0),
            p90: percentile(&sorted, 90.0),
            p95: percentile(&sorted, 95.0),
            p99: percentile(&sorted, 99.0),
        }
    }
}

/// Streaming statistics accumulator for categorical data.
#[derive(Debug, Clone)]
pub struct StreamingCategoricalStats {
    /// Total count.
    count: u64,
    /// Frequency counts (capped at max_categories for memory).
    frequencies: HashMap<String, u64>,
    /// Maximum number of categories to track.
    max_categories: usize,
    /// Count of values in categories not tracked.
    other_count: u64,
}

impl StreamingCategoricalStats {
    /// Create a new streaming categorical stats accumulator.
    pub fn new(max_categories: usize) -> Self {
        Self {
            count: 0,
            frequencies: HashMap::new(),
            max_categories,
            other_count: 0,
        }
    }

    /// Add a value to the accumulator.
    pub fn add(&mut self, value: String) {
        if value.is_empty() {
            return;
        }

        self.count += 1;

        if let Some(count) = self.frequencies.get_mut(&value) {
            *count += 1;
        } else if self.frequencies.len() < self.max_categories {
            self.frequencies.insert(value, 1);
        } else {
            // Check if this value should replace a less frequent one
            // (This implements a lossy counting approach)
            self.other_count += 1;

            // Periodically prune low-frequency items
            if self.other_count > self.max_categories as u64 {
                self.prune_low_frequency();
            }
        }
    }

    /// Prune low-frequency items to make room for new ones.
    fn prune_low_frequency(&mut self) {
        let threshold = self.other_count / self.max_categories as u64;
        self.frequencies.retain(|_, &mut count| count > threshold);
        self.other_count = 0;
    }

    /// Get the count.
    pub fn count(&self) -> u64 {
        self.count
    }

    /// Get the cardinality (number of unique categories tracked).
    pub fn cardinality(&self) -> u64 {
        self.frequencies.len() as u64
    }

    /// Get the top values by frequency.
    pub fn top_values(&self, limit: usize) -> Vec<(String, u64)> {
        let mut values: Vec<_> = self.frequencies.iter().collect();
        values.sort_by(|a, b| b.1.cmp(a.1));
        values
            .into_iter()
            .take(limit)
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }

    /// Compute entropy.
    pub fn entropy(&self) -> f64 {
        let total = self.count as f64;
        if total == 0.0 {
            return 0.0;
        }

        let mut entropy = 0.0;
        for &count in self.frequencies.values() {
            if count > 0 {
                let p = count as f64 / total;
                entropy -= p * p.ln();
            }
        }
        entropy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_numeric_stats() {
        let mut stats = StreamingNumericStats::new(1000);

        // Add some values
        for i in 1..=100 {
            stats.add(i as f64);
        }

        assert_eq!(stats.count(), 100);
        assert!((stats.mean() - 50.5).abs() < 0.001);
        assert_eq!(stats.min(), 1.0);
        assert_eq!(stats.max(), 100.0);
    }

    #[test]
    fn test_streaming_categorical_stats() {
        let mut stats = StreamingCategoricalStats::new(100);

        // Add some values
        for _ in 0..50 {
            stats.add("A".to_string());
        }
        for _ in 0..30 {
            stats.add("B".to_string());
        }
        for _ in 0..20 {
            stats.add("C".to_string());
        }

        assert_eq!(stats.count(), 100);
        assert_eq!(stats.cardinality(), 3);

        let top = stats.top_values(3);
        assert_eq!(top[0].0, "A");
        assert_eq!(top[0].1, 50);
    }
}
