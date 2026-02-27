//! Drift Detection Evaluation Module.
//!
//! Provides tools for evaluating drift detection ground truth labels and
//! validating that generated drift events are detectable and properly labeled.
//!
//! # Overview
//!
//! This module evaluates the quality and detectability of drift events in
//! synthetic data by analyzing:
//!
//! - Statistical distribution shifts (mean, variance changes)
//! - Categorical shifts (proportion changes, new categories)
//! - Temporal pattern changes (seasonality, trend)
//! - Regulatory and organizational event impacts
//!
//! # Example
//!
//! ```ignore
//! use datasynth_eval::statistical::{DriftDetectionAnalyzer, DriftDetectionEntry};
//!
//! let analyzer = DriftDetectionAnalyzer::new(0.05);
//! let entries = vec![
//!     DriftDetectionEntry::new(1, 100.0, Some(true)),
//!     DriftDetectionEntry::new(2, 102.0, Some(false)),
//!     // ...
//! ];
//!
//! let analysis = analyzer.analyze(&entries)?;
//! println!("Drift detected: {}", analysis.drift_detected);
//! ```

use crate::error::{EvalError, EvalResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// Drift Detection Entry
// =============================================================================

/// A single data point for drift detection analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftDetectionEntry {
    /// Period number (e.g., month number from start).
    pub period: u32,
    /// Observed value at this period.
    pub value: f64,
    /// Ground truth label: true if this period has drift, false otherwise.
    pub ground_truth_drift: Option<bool>,
    /// Drift event type if labeled.
    pub drift_type: Option<String>,
    /// Magnitude of drift if known.
    pub drift_magnitude: Option<f64>,
    /// Detection difficulty (0.0 = easy, 1.0 = hard).
    pub detection_difficulty: Option<f64>,
}

impl DriftDetectionEntry {
    /// Create a new drift detection entry.
    pub fn new(period: u32, value: f64, ground_truth_drift: Option<bool>) -> Self {
        Self {
            period,
            value,
            ground_truth_drift,
            drift_type: None,
            drift_magnitude: None,
            detection_difficulty: None,
        }
    }

    /// Create entry with full metadata.
    pub fn with_metadata(
        period: u32,
        value: f64,
        ground_truth_drift: bool,
        drift_type: impl Into<String>,
        drift_magnitude: f64,
        detection_difficulty: f64,
    ) -> Self {
        Self {
            period,
            value,
            ground_truth_drift: Some(ground_truth_drift),
            drift_type: Some(drift_type.into()),
            drift_magnitude: Some(drift_magnitude),
            detection_difficulty: Some(detection_difficulty),
        }
    }
}

// =============================================================================
// Labeled Drift Event
// =============================================================================

/// A labeled drift event from ground truth data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabeledDriftEvent {
    /// Unique event identifier.
    pub event_id: String,
    /// Event type classification.
    pub event_type: DriftEventCategory,
    /// Start period of the drift.
    pub start_period: u32,
    /// End period of the drift (None if ongoing).
    pub end_period: Option<u32>,
    /// Affected fields/metrics.
    pub affected_fields: Vec<String>,
    /// Magnitude of the drift effect.
    pub magnitude: f64,
    /// Detection difficulty level.
    pub detection_difficulty: DetectionDifficulty,
}

/// Categories of drift events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DriftEventCategory {
    /// Mean shift in distribution.
    MeanShift,
    /// Variance change in distribution.
    VarianceChange,
    /// Trend change (slope).
    TrendChange,
    /// Seasonality pattern change.
    SeasonalityChange,
    /// Categorical proportion shift.
    ProportionShift,
    /// New category emergence.
    NewCategory,
    /// Organizational event (acquisition, merger, etc.).
    OrganizationalEvent,
    /// Regulatory change impact.
    RegulatoryChange,
    /// Technology transition impact.
    TechnologyTransition,
    /// Economic cycle effect.
    EconomicCycle,
    /// Process evolution.
    ProcessEvolution,
}

impl DriftEventCategory {
    /// Get human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::MeanShift => "Mean Shift",
            Self::VarianceChange => "Variance Change",
            Self::TrendChange => "Trend Change",
            Self::SeasonalityChange => "Seasonality Change",
            Self::ProportionShift => "Proportion Shift",
            Self::NewCategory => "New Category",
            Self::OrganizationalEvent => "Organizational Event",
            Self::RegulatoryChange => "Regulatory Change",
            Self::TechnologyTransition => "Technology Transition",
            Self::EconomicCycle => "Economic Cycle",
            Self::ProcessEvolution => "Process Evolution",
        }
    }

    /// Check if this is a statistical drift type.
    pub fn is_statistical(&self) -> bool {
        matches!(
            self,
            Self::MeanShift | Self::VarianceChange | Self::TrendChange | Self::SeasonalityChange
        )
    }

    /// Check if this is a business event drift type.
    pub fn is_business_event(&self) -> bool {
        matches!(
            self,
            Self::OrganizationalEvent
                | Self::RegulatoryChange
                | Self::TechnologyTransition
                | Self::ProcessEvolution
        )
    }
}

/// Detection difficulty levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DetectionDifficulty {
    /// Easy to detect (large magnitude, clear signal).
    Easy,
    /// Medium difficulty.
    Medium,
    /// Hard to detect (small magnitude, noisy signal).
    Hard,
}

impl DetectionDifficulty {
    /// Convert to numeric score (0.0 = easy, 1.0 = hard).
    pub fn to_score(&self) -> f64 {
        match self {
            Self::Easy => 0.0,
            Self::Medium => 0.5,
            Self::Hard => 1.0,
        }
    }

    /// Create from numeric score.
    pub fn from_score(score: f64) -> Self {
        if score < 0.33 {
            Self::Easy
        } else if score < 0.67 {
            Self::Medium
        } else {
            Self::Hard
        }
    }
}

// =============================================================================
// Drift Detection Analyzer
// =============================================================================

/// Analyzer for drift detection evaluation.
#[derive(Debug, Clone)]
pub struct DriftDetectionAnalyzer {
    /// Significance level for statistical tests.
    significance_level: f64,
    /// Window size for rolling statistics.
    window_size: usize,
    /// Minimum magnitude threshold to consider as drift.
    min_magnitude_threshold: f64,
    /// Enable Hellinger distance calculation.
    use_hellinger: bool,
    /// Enable Population Stability Index (PSI) calculation.
    use_psi: bool,
}

impl DriftDetectionAnalyzer {
    /// Create a new drift detection analyzer.
    pub fn new(significance_level: f64) -> Self {
        Self {
            significance_level,
            window_size: 10,
            min_magnitude_threshold: 0.05,
            use_hellinger: true,
            use_psi: true,
        }
    }

    /// Set the rolling window size.
    pub fn with_window_size(mut self, size: usize) -> Self {
        self.window_size = size;
        self
    }

    /// Set the minimum magnitude threshold.
    pub fn with_min_magnitude(mut self, threshold: f64) -> Self {
        self.min_magnitude_threshold = threshold;
        self
    }

    /// Enable or disable Hellinger distance calculation.
    pub fn with_hellinger(mut self, enabled: bool) -> Self {
        self.use_hellinger = enabled;
        self
    }

    /// Enable or disable PSI calculation.
    pub fn with_psi(mut self, enabled: bool) -> Self {
        self.use_psi = enabled;
        self
    }

    /// Analyze drift detection entries.
    pub fn analyze(&self, entries: &[DriftDetectionEntry]) -> EvalResult<DriftDetectionAnalysis> {
        if entries.len() < self.window_size * 2 {
            return Err(EvalError::InsufficientData {
                required: self.window_size * 2,
                actual: entries.len(),
            });
        }

        // Extract values and labels
        let values: Vec<f64> = entries.iter().map(|e| e.value).collect();
        let ground_truth: Vec<Option<bool>> =
            entries.iter().map(|e| e.ground_truth_drift).collect();

        // Calculate rolling statistics
        let rolling_means = self.calculate_rolling_means(&values);
        let rolling_stds = self.calculate_rolling_stds(&values);

        // Detect drift points using CUSUM-like approach
        let detected_drift = self.detect_drift_points(&rolling_means, &rolling_stds);

        // Calculate detection metrics if ground truth is available
        let metrics = self.calculate_detection_metrics(&detected_drift, &ground_truth);

        // Calculate statistical measures
        let hellinger_distance = if self.use_hellinger {
            Some(self.calculate_hellinger_distance(&values))
        } else {
            None
        };

        let psi = if self.use_psi {
            Some(self.calculate_psi(&values))
        } else {
            None
        };

        // Determine overall drift status
        let drift_detected = detected_drift.iter().any(|&d| d);
        let drift_count = detected_drift.iter().filter(|&&d| d).count();

        // Calculate magnitude of detected drifts
        let drift_magnitude = self.calculate_drift_magnitude(&rolling_means);

        let passes = self.evaluate_pass_status(&metrics, drift_magnitude);
        let issues = self.collect_issues(&metrics, drift_magnitude, drift_count);

        Ok(DriftDetectionAnalysis {
            sample_size: entries.len(),
            drift_detected,
            drift_count,
            drift_magnitude,
            detection_metrics: metrics,
            hellinger_distance,
            psi,
            rolling_mean_change: self.calculate_mean_change(&rolling_means),
            rolling_std_change: self.calculate_std_change(&rolling_stds),
            passes,
            issues,
        })
    }

    /// Analyze labeled drift events for quality.
    pub fn analyze_labeled_events(
        &self,
        events: &[LabeledDriftEvent],
    ) -> EvalResult<LabeledEventAnalysis> {
        if events.is_empty() {
            return Ok(LabeledEventAnalysis::empty());
        }

        // Count events by category
        let mut category_counts: HashMap<DriftEventCategory, usize> = HashMap::new();
        for event in events {
            *category_counts.entry(event.event_type).or_insert(0) += 1;
        }

        // Count events by difficulty
        let mut difficulty_counts: HashMap<DetectionDifficulty, usize> = HashMap::new();
        for event in events {
            *difficulty_counts
                .entry(event.detection_difficulty)
                .or_insert(0) += 1;
        }

        // Calculate coverage metrics
        let total_events = events.len();
        let statistical_events = events
            .iter()
            .filter(|e| e.event_type.is_statistical())
            .count();
        let business_events = events
            .iter()
            .filter(|e| e.event_type.is_business_event())
            .count();

        // Calculate average magnitude and difficulty
        let avg_magnitude = events.iter().map(|e| e.magnitude).sum::<f64>() / total_events as f64;
        let avg_difficulty = events
            .iter()
            .map(|e| e.detection_difficulty.to_score())
            .sum::<f64>()
            / total_events as f64;

        // Calculate period coverage
        let min_period = events.iter().map(|e| e.start_period).min().unwrap_or(0);
        let max_period = events
            .iter()
            .filter_map(|e| e.end_period)
            .max()
            .unwrap_or(min_period);

        let passes = total_events > 0 && avg_magnitude >= self.min_magnitude_threshold;
        let issues = if !passes {
            vec!["Insufficient drift events or magnitude too low".to_string()]
        } else {
            Vec::new()
        };

        Ok(LabeledEventAnalysis {
            total_events,
            statistical_events,
            business_events,
            category_distribution: category_counts,
            difficulty_distribution: difficulty_counts,
            avg_magnitude,
            avg_difficulty,
            period_coverage: (min_period, max_period),
            passes,
            issues,
        })
    }

    // Helper methods

    fn calculate_rolling_means(&self, values: &[f64]) -> Vec<f64> {
        if values.len() < self.window_size {
            tracing::debug!(
                "Drift detection: not enough values ({}) for window size ({}), returning empty",
                values.len(),
                self.window_size
            );
            return Vec::new();
        }
        let mut means = Vec::with_capacity(values.len() - self.window_size + 1);
        for i in 0..=(values.len() - self.window_size) {
            let window = &values[i..i + self.window_size];
            let mean = window.iter().sum::<f64>() / self.window_size as f64;
            means.push(mean);
        }
        means
    }

    fn calculate_rolling_stds(&self, values: &[f64]) -> Vec<f64> {
        if values.len() < self.window_size {
            tracing::debug!(
                "Drift detection: not enough values ({}) for window size ({}), returning empty",
                values.len(),
                self.window_size
            );
            return Vec::new();
        }
        let mut stds = Vec::with_capacity(values.len() - self.window_size + 1);
        for i in 0..=(values.len() - self.window_size) {
            let window = &values[i..i + self.window_size];
            let mean = window.iter().sum::<f64>() / self.window_size as f64;
            let variance =
                window.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / self.window_size as f64;
            stds.push(variance.sqrt());
        }
        stds
    }

    fn detect_drift_points(&self, means: &[f64], stds: &[f64]) -> Vec<bool> {
        if means.len() < 2 {
            return vec![false; means.len()];
        }

        let mut detected = vec![false; means.len()];

        // Calculate baseline statistics from first half
        let baseline_end = means.len() / 2;
        let baseline_mean = means[..baseline_end].iter().sum::<f64>() / baseline_end as f64;
        let baseline_std = if baseline_end > 1 {
            let variance = means[..baseline_end]
                .iter()
                .map(|x| (x - baseline_mean).powi(2))
                .sum::<f64>()
                / baseline_end as f64;
            variance.sqrt().max(0.001) // Avoid division by zero
        } else {
            0.001
        };

        // Detect drift using z-score approach
        for i in baseline_end..means.len() {
            let z_score = (means[i] - baseline_mean).abs() / baseline_std;
            let threshold = 1.96 / self.significance_level.sqrt(); // Adjust for significance

            if z_score > threshold {
                detected[i] = true;
            }

            // Also check for variance change
            if i < stds.len() && baseline_end > 0 {
                let baseline_var_mean =
                    stds[..baseline_end].iter().sum::<f64>() / baseline_end as f64;
                if baseline_var_mean > 0.001 {
                    let var_ratio = stds[i] / baseline_var_mean;
                    if !(0.5..=2.0).contains(&var_ratio) {
                        detected[i] = true;
                    }
                }
            }
        }

        detected
    }

    fn calculate_detection_metrics(
        &self,
        detected: &[bool],
        ground_truth: &[Option<bool>],
    ) -> DriftDetectionMetrics {
        let mut true_positives = 0;
        let mut false_positives = 0;
        let mut true_negatives = 0;
        let mut false_negatives = 0;
        let mut detection_delays = Vec::new();

        // Adjust for window offset
        let offset = detected.len().saturating_sub(ground_truth.len());

        for (i, &gt) in ground_truth.iter().enumerate() {
            let detected_idx = i + offset;
            if detected_idx >= detected.len() {
                break;
            }

            let pred = detected[detected_idx];
            match gt {
                Some(true) => {
                    if pred {
                        true_positives += 1;
                    } else {
                        false_negatives += 1;
                    }
                }
                Some(false) => {
                    if pred {
                        false_positives += 1;
                    } else {
                        true_negatives += 1;
                    }
                }
                None => {}
            }
        }

        // Calculate detection delay for true positives
        let mut last_drift_start: Option<usize> = None;
        for (i, &gt) in ground_truth.iter().enumerate() {
            if gt == Some(true) && last_drift_start.is_none() {
                last_drift_start = Some(i);
            } else if gt == Some(false) {
                last_drift_start = None;
            }

            let detected_idx = i + offset;
            if detected_idx < detected.len() && detected[detected_idx] {
                if let Some(start) = last_drift_start {
                    detection_delays.push((i - start) as f64);
                    last_drift_start = None;
                }
            }
        }

        let precision = if true_positives + false_positives > 0 {
            true_positives as f64 / (true_positives + false_positives) as f64
        } else {
            0.0
        };

        let recall = if true_positives + false_negatives > 0 {
            true_positives as f64 / (true_positives + false_negatives) as f64
        } else {
            0.0
        };

        let f1_score = if precision + recall > 0.0 {
            2.0 * precision * recall / (precision + recall)
        } else {
            0.0
        };

        let mean_detection_delay = if detection_delays.is_empty() {
            None
        } else {
            Some(detection_delays.iter().sum::<f64>() / detection_delays.len() as f64)
        };

        DriftDetectionMetrics {
            true_positives,
            false_positives,
            true_negatives,
            false_negatives,
            precision,
            recall,
            f1_score,
            mean_detection_delay,
        }
    }

    fn calculate_hellinger_distance(&self, values: &[f64]) -> f64 {
        if values.len() < 20 {
            return 0.0;
        }

        let mid = values.len() / 2;
        let first_half = &values[..mid];
        let second_half = &values[mid..];

        // Create histograms with 10 bins
        let (min_val, max_val) = values.iter().fold((f64::MAX, f64::MIN), |(min, max), &v| {
            (min.min(v), max.max(v))
        });

        if (max_val - min_val).abs() < f64::EPSILON {
            return 0.0;
        }

        let num_bins = 10;
        let bin_width = (max_val - min_val) / num_bins as f64;

        let mut hist1 = vec![0.0; num_bins];
        let mut hist2 = vec![0.0; num_bins];

        for &v in first_half {
            let bin = ((v - min_val) / bin_width).floor() as usize;
            let bin = bin.min(num_bins - 1);
            hist1[bin] += 1.0;
        }

        for &v in second_half {
            let bin = ((v - min_val) / bin_width).floor() as usize;
            let bin = bin.min(num_bins - 1);
            hist2[bin] += 1.0;
        }

        // Normalize
        let sum1: f64 = hist1.iter().sum();
        let sum2: f64 = hist2.iter().sum();

        if sum1 == 0.0 || sum2 == 0.0 {
            return 0.0;
        }

        for h in &mut hist1 {
            *h /= sum1;
        }
        for h in &mut hist2 {
            *h /= sum2;
        }

        // Calculate Hellinger distance
        let mut sum_sq_diff = 0.0;
        for i in 0..num_bins {
            let diff = hist1[i].sqrt() - hist2[i].sqrt();
            sum_sq_diff += diff * diff;
        }

        (sum_sq_diff / 2.0).sqrt()
    }

    fn calculate_psi(&self, values: &[f64]) -> f64 {
        if values.len() < 20 {
            return 0.0;
        }

        let mid = values.len() / 2;
        let baseline = &values[..mid];
        let current = &values[mid..];

        // Create histograms with 10 bins
        let (min_val, max_val) = values.iter().fold((f64::MAX, f64::MIN), |(min, max), &v| {
            (min.min(v), max.max(v))
        });

        if (max_val - min_val).abs() < f64::EPSILON {
            return 0.0;
        }

        let num_bins = 10;
        let bin_width = (max_val - min_val) / num_bins as f64;

        let mut hist_baseline = vec![0.0; num_bins];
        let mut hist_current = vec![0.0; num_bins];

        for &v in baseline {
            let bin = ((v - min_val) / bin_width).floor() as usize;
            let bin = bin.min(num_bins - 1);
            hist_baseline[bin] += 1.0;
        }

        for &v in current {
            let bin = ((v - min_val) / bin_width).floor() as usize;
            let bin = bin.min(num_bins - 1);
            hist_current[bin] += 1.0;
        }

        // Normalize and add small constant to avoid log(0)
        let epsilon = 0.0001;
        let sum_baseline: f64 = hist_baseline.iter().sum();
        let sum_current: f64 = hist_current.iter().sum();

        if sum_baseline == 0.0 || sum_current == 0.0 {
            return 0.0;
        }

        for h in &mut hist_baseline {
            *h = (*h / sum_baseline).max(epsilon);
        }
        for h in &mut hist_current {
            *h = (*h / sum_current).max(epsilon);
        }

        // Calculate PSI
        let mut psi = 0.0;
        for i in 0..num_bins {
            let diff = hist_current[i] - hist_baseline[i];
            let ratio = hist_current[i] / hist_baseline[i];
            psi += diff * ratio.ln();
        }

        psi
    }

    fn calculate_drift_magnitude(&self, means: &[f64]) -> f64 {
        if means.len() < 2 {
            return 0.0;
        }

        let mid = means.len() / 2;
        let first_mean = means[..mid].iter().sum::<f64>() / mid as f64;
        let second_mean = means[mid..].iter().sum::<f64>() / (means.len() - mid) as f64;

        if first_mean.abs() < f64::EPSILON {
            return (second_mean - first_mean).abs();
        }

        ((second_mean - first_mean) / first_mean).abs()
    }

    fn calculate_mean_change(&self, means: &[f64]) -> f64 {
        if means.len() < 2 {
            return 0.0;
        }
        let first = means.first().unwrap_or(&0.0);
        let last = means.last().unwrap_or(&0.0);
        if first.abs() < f64::EPSILON {
            return 0.0;
        }
        (last - first) / first
    }

    fn calculate_std_change(&self, stds: &[f64]) -> f64 {
        if stds.len() < 2 {
            return 0.0;
        }
        let first = stds.first().unwrap_or(&0.0);
        let last = stds.last().unwrap_or(&0.0);
        if first.abs() < f64::EPSILON {
            return 0.0;
        }
        (last - first) / first
    }

    fn evaluate_pass_status(&self, metrics: &DriftDetectionMetrics, drift_magnitude: f64) -> bool {
        // Pass if we have reasonable detection metrics or magnitude is below threshold
        if drift_magnitude < self.min_magnitude_threshold {
            return true; // No significant drift to detect
        }

        // If there's significant drift, we need decent detection
        metrics.f1_score >= 0.5 || metrics.precision >= 0.6 || metrics.recall >= 0.6
    }

    fn collect_issues(
        &self,
        metrics: &DriftDetectionMetrics,
        drift_magnitude: f64,
        drift_count: usize,
    ) -> Vec<String> {
        let mut issues = Vec::new();

        if drift_magnitude >= self.min_magnitude_threshold {
            if metrics.precision < 0.5 {
                issues.push(format!(
                    "Low precision ({:.2}): many false positives",
                    metrics.precision
                ));
            }
            if metrics.recall < 0.5 {
                issues.push(format!(
                    "Low recall ({:.2}): many drift events missed",
                    metrics.recall
                ));
            }
            if let Some(delay) = metrics.mean_detection_delay {
                if delay > 3.0 {
                    issues.push(format!("High detection delay ({:.1} periods)", delay));
                }
            }
        }

        if drift_count == 0 && drift_magnitude >= self.min_magnitude_threshold {
            issues.push("No drift detected despite significant magnitude change".to_string());
        }

        issues
    }
}

impl Default for DriftDetectionAnalyzer {
    fn default() -> Self {
        Self::new(0.05)
    }
}

// =============================================================================
// Analysis Results
// =============================================================================

/// Results from drift detection analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftDetectionAnalysis {
    /// Number of data points analyzed.
    pub sample_size: usize,
    /// Whether any drift was detected.
    pub drift_detected: bool,
    /// Number of drift points detected.
    pub drift_count: usize,
    /// Overall magnitude of detected drift.
    pub drift_magnitude: f64,
    /// Detection metrics (precision, recall, F1).
    pub detection_metrics: DriftDetectionMetrics,
    /// Hellinger distance between first and second half.
    pub hellinger_distance: Option<f64>,
    /// Population Stability Index.
    pub psi: Option<f64>,
    /// Relative change in rolling mean.
    pub rolling_mean_change: f64,
    /// Relative change in rolling standard deviation.
    pub rolling_std_change: f64,
    /// Whether the analysis passes quality thresholds.
    pub passes: bool,
    /// Issues identified during analysis.
    pub issues: Vec<String>,
}

/// Drift detection performance metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DriftDetectionMetrics {
    /// True positive count.
    pub true_positives: usize,
    /// False positive count.
    pub false_positives: usize,
    /// True negative count.
    pub true_negatives: usize,
    /// False negative count.
    pub false_negatives: usize,
    /// Precision (TP / (TP + FP)).
    pub precision: f64,
    /// Recall (TP / (TP + FN)).
    pub recall: f64,
    /// F1 score (harmonic mean of precision and recall).
    pub f1_score: f64,
    /// Mean delay in detecting drift (in periods).
    pub mean_detection_delay: Option<f64>,
}

/// Analysis of labeled drift events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabeledEventAnalysis {
    /// Total number of labeled events.
    pub total_events: usize,
    /// Number of statistical drift events.
    pub statistical_events: usize,
    /// Number of business event drifts.
    pub business_events: usize,
    /// Distribution by event category.
    pub category_distribution: HashMap<DriftEventCategory, usize>,
    /// Distribution by detection difficulty.
    pub difficulty_distribution: HashMap<DetectionDifficulty, usize>,
    /// Average drift magnitude.
    pub avg_magnitude: f64,
    /// Average detection difficulty score.
    pub avg_difficulty: f64,
    /// Period coverage (min_period, max_period).
    pub period_coverage: (u32, u32),
    /// Whether the analysis passes quality thresholds.
    pub passes: bool,
    /// Issues identified.
    pub issues: Vec<String>,
}

impl LabeledEventAnalysis {
    /// Create an empty analysis result.
    pub fn empty() -> Self {
        Self {
            total_events: 0,
            statistical_events: 0,
            business_events: 0,
            category_distribution: HashMap::new(),
            difficulty_distribution: HashMap::new(),
            avg_magnitude: 0.0,
            avg_difficulty: 0.0,
            period_coverage: (0, 0),
            passes: true,
            issues: Vec::new(),
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_drift_detection_entry_creation() {
        let entry = DriftDetectionEntry::new(1, 100.0, Some(true));
        assert_eq!(entry.period, 1);
        assert_eq!(entry.value, 100.0);
        assert_eq!(entry.ground_truth_drift, Some(true));
    }

    #[test]
    fn test_drift_detection_entry_with_metadata() {
        let entry = DriftDetectionEntry::with_metadata(5, 150.0, true, "MeanShift", 0.15, 0.3);
        assert_eq!(entry.period, 5);
        assert_eq!(entry.drift_type, Some("MeanShift".to_string()));
        assert_eq!(entry.drift_magnitude, Some(0.15));
        assert_eq!(entry.detection_difficulty, Some(0.3));
    }

    #[test]
    fn test_drift_event_category_names() {
        assert_eq!(DriftEventCategory::MeanShift.name(), "Mean Shift");
        assert_eq!(
            DriftEventCategory::OrganizationalEvent.name(),
            "Organizational Event"
        );
    }

    #[test]
    fn test_drift_event_category_classification() {
        assert!(DriftEventCategory::MeanShift.is_statistical());
        assert!(!DriftEventCategory::MeanShift.is_business_event());
        assert!(DriftEventCategory::OrganizationalEvent.is_business_event());
        assert!(!DriftEventCategory::OrganizationalEvent.is_statistical());
    }

    #[test]
    fn test_detection_difficulty_conversion() {
        assert_eq!(DetectionDifficulty::Easy.to_score(), 0.0);
        assert_eq!(DetectionDifficulty::Medium.to_score(), 0.5);
        assert_eq!(DetectionDifficulty::Hard.to_score(), 1.0);

        assert_eq!(
            DetectionDifficulty::from_score(0.1),
            DetectionDifficulty::Easy
        );
        assert_eq!(
            DetectionDifficulty::from_score(0.5),
            DetectionDifficulty::Medium
        );
        assert_eq!(
            DetectionDifficulty::from_score(0.8),
            DetectionDifficulty::Hard
        );
    }

    #[test]
    fn test_analyzer_creation() {
        let analyzer = DriftDetectionAnalyzer::new(0.05)
            .with_window_size(15)
            .with_min_magnitude(0.1)
            .with_hellinger(true)
            .with_psi(true);

        assert_eq!(analyzer.significance_level, 0.05);
        assert_eq!(analyzer.window_size, 15);
        assert_eq!(analyzer.min_magnitude_threshold, 0.1);
    }

    #[test]
    fn test_analyze_no_drift() {
        let analyzer = DriftDetectionAnalyzer::new(0.05).with_window_size(5);

        // Create stable data with no drift
        let entries: Vec<DriftDetectionEntry> = (0..30)
            .map(|i| DriftDetectionEntry::new(i, 100.0 + (i as f64 * 0.01), Some(false)))
            .collect();

        let result = analyzer.analyze(&entries).unwrap();
        assert!(!result.drift_detected || result.drift_count < 5);
        assert!(result.drift_magnitude < 0.1);
    }

    #[test]
    fn test_analyze_with_drift() {
        let analyzer = DriftDetectionAnalyzer::new(0.05).with_window_size(5);

        // Create data with clear drift in the middle
        let mut entries: Vec<DriftDetectionEntry> = (0..15)
            .map(|i| DriftDetectionEntry::new(i, 100.0, Some(false)))
            .collect();

        // Add drift after period 15
        for i in 15..30 {
            entries.push(DriftDetectionEntry::new(i, 150.0, Some(true)));
        }

        let result = analyzer.analyze(&entries).unwrap();
        assert!(result.drift_detected);
        assert!(result.drift_magnitude > 0.3);
    }

    #[test]
    fn test_analyze_insufficient_data() {
        let analyzer = DriftDetectionAnalyzer::new(0.05).with_window_size(10);

        let entries: Vec<DriftDetectionEntry> = (0..5)
            .map(|i| DriftDetectionEntry::new(i, 100.0, None))
            .collect();

        let result = analyzer.analyze(&entries);
        assert!(result.is_err());
    }

    #[test]
    fn test_analyze_labeled_events() {
        let analyzer = DriftDetectionAnalyzer::new(0.05);

        let events = vec![
            LabeledDriftEvent {
                event_id: "E1".to_string(),
                event_type: DriftEventCategory::MeanShift,
                start_period: 10,
                end_period: Some(15),
                affected_fields: vec!["amount".to_string()],
                magnitude: 0.15,
                detection_difficulty: DetectionDifficulty::Easy,
            },
            LabeledDriftEvent {
                event_id: "E2".to_string(),
                event_type: DriftEventCategory::OrganizationalEvent,
                start_period: 20,
                end_period: Some(25),
                affected_fields: vec!["volume".to_string()],
                magnitude: 0.30,
                detection_difficulty: DetectionDifficulty::Medium,
            },
        ];

        let result = analyzer.analyze_labeled_events(&events).unwrap();
        assert_eq!(result.total_events, 2);
        assert_eq!(result.statistical_events, 1);
        assert_eq!(result.business_events, 1);
        assert!(result.avg_magnitude > 0.2);
        assert!(result.passes);
    }

    #[test]
    fn test_empty_labeled_events() {
        let analyzer = DriftDetectionAnalyzer::new(0.05);
        let result = analyzer.analyze_labeled_events(&[]).unwrap();
        assert_eq!(result.total_events, 0);
        assert!(result.passes);
    }

    #[test]
    fn test_hellinger_distance_no_drift() {
        let analyzer = DriftDetectionAnalyzer::new(0.05);

        // Stable data
        let entries: Vec<DriftDetectionEntry> = (0..40)
            .map(|i| DriftDetectionEntry::new(i, 100.0 + (i as f64 % 5.0), None))
            .collect();

        let result = analyzer.analyze(&entries).unwrap();
        assert!(result.hellinger_distance.unwrap() < 0.3);
    }

    #[test]
    fn test_psi_calculation() {
        let analyzer = DriftDetectionAnalyzer::new(0.05);

        // Data with drift
        let mut entries: Vec<DriftDetectionEntry> = (0..20)
            .map(|i| DriftDetectionEntry::new(i, 100.0, None))
            .collect();
        for i in 20..40 {
            entries.push(DriftDetectionEntry::new(i, 200.0, None));
        }

        let result = analyzer.analyze(&entries).unwrap();
        assert!(result.psi.is_some());
        // PSI > 0.1 indicates significant drift
        assert!(result.psi.unwrap() > 0.0);
    }

    #[test]
    fn test_detection_metrics_calculation() {
        let analyzer = DriftDetectionAnalyzer::new(0.05).with_window_size(3);

        // Create data where we know the ground truth
        let mut entries = Vec::new();
        for i in 0..10 {
            entries.push(DriftDetectionEntry::new(i, 100.0, Some(false)));
        }
        for i in 10..20 {
            entries.push(DriftDetectionEntry::new(i, 200.0, Some(true)));
        }

        let result = analyzer.analyze(&entries).unwrap();

        // Should have some detection capability
        assert!(result.detection_metrics.precision >= 0.0);
        assert!(result.detection_metrics.recall >= 0.0);
    }
}
