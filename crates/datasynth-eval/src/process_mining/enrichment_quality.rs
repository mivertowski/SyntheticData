//! OCEL enrichment quality evaluator.
//!
//! Validates that OCEL 2.0 events are properly enriched with state transitions,
//! resource workload data, and correlation identifiers. Also checks workload
//! distribution balance via Gini coefficient.

use serde::{Deserialize, Serialize};

/// Input data for a single OCEL event's enrichment status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelEnrichmentData {
    pub event_id: String,
    pub has_from_state: bool,
    pub has_to_state: bool,
    pub has_resource_workload: bool,
    pub has_correlation_id: bool,
    pub from_state: Option<String>,
    pub to_state: Option<String>,
    pub resource_workload: Option<f64>,
}

/// Configurable thresholds for OCEL enrichment quality.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelEnrichmentThresholds {
    /// Minimum fraction of events with state transitions. Default: 0.80.
    pub min_state_coverage: f64,
    /// Minimum fraction of events with resource workload. Default: 0.70.
    pub min_workload_coverage: f64,
    /// Minimum fraction with correlation IDs. Default: 0.10.
    pub min_correlation_coverage: f64,
    /// Maximum Gini coefficient for workload distribution. Default: 0.50.
    pub max_workload_imbalance: f64,
}

impl Default for OcelEnrichmentThresholds {
    fn default() -> Self {
        Self {
            min_state_coverage: 0.80,
            min_workload_coverage: 0.70,
            min_correlation_coverage: 0.10,
            max_workload_imbalance: 0.50,
        }
    }
}

/// Result of OCEL enrichment quality analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelEnrichmentAnalysis {
    pub state_coverage: f64,
    pub workload_coverage: f64,
    pub correlation_coverage: f64,
    /// Gini coefficient of resource workloads (0 = perfectly equal, 1 = maximally unequal).
    pub workload_gini: f64,
    pub unique_states: usize,
    pub unique_correlations: usize,
    pub passes: bool,
    pub issues: Vec<String>,
}

/// Analyzer for OCEL enrichment quality metrics.
pub struct OcelEnrichmentAnalyzer {
    thresholds: OcelEnrichmentThresholds,
}

impl OcelEnrichmentAnalyzer {
    pub fn new(thresholds: OcelEnrichmentThresholds) -> Self {
        Self { thresholds }
    }

    pub fn with_defaults() -> Self {
        Self::new(OcelEnrichmentThresholds::default())
    }

    pub fn analyze(&self, events: &[OcelEnrichmentData]) -> OcelEnrichmentAnalysis {
        let mut issues = Vec::new();
        let total = events.len();

        if total == 0 {
            return OcelEnrichmentAnalysis {
                state_coverage: 0.0,
                workload_coverage: 0.0,
                correlation_coverage: 0.0,
                workload_gini: 0.0,
                unique_states: 0,
                unique_correlations: 0,
                passes: false,
                issues: vec!["No events provided".into()],
            };
        }

        // State coverage
        let with_states = events
            .iter()
            .filter(|e| e.has_from_state && e.has_to_state)
            .count();
        let state_coverage = with_states as f64 / total as f64;

        // Workload coverage
        let with_workload = events.iter().filter(|e| e.has_resource_workload).count();
        let workload_coverage = with_workload as f64 / total as f64;

        // Correlation coverage
        let with_correlation = events.iter().filter(|e| e.has_correlation_id).count();
        let correlation_coverage = with_correlation as f64 / total as f64;

        // Unique states
        let mut unique_states_set = std::collections::HashSet::new();
        for e in events {
            if let Some(ref s) = e.from_state {
                unique_states_set.insert(s.clone());
            }
            if let Some(ref s) = e.to_state {
                unique_states_set.insert(s.clone());
            }
        }
        let unique_states = unique_states_set.len();

        // Unique correlations
        let unique_correlations = {
            let mut set = std::collections::HashSet::new();
            for e in events {
                if e.has_correlation_id {
                    set.insert(e.event_id.clone()); // Use event_id as proxy
                }
            }
            set.len()
        };

        // Gini coefficient of workloads
        let workload_gini = {
            let mut workloads: Vec<f64> =
                events.iter().filter_map(|e| e.resource_workload).collect();
            if workloads.len() < 2 {
                0.0
            } else {
                workloads.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let n = workloads.len() as f64;
                let sum: f64 = workloads.iter().sum();
                if sum == 0.0 {
                    0.0
                } else {
                    let mut numerator = 0.0;
                    for (i, &w) in workloads.iter().enumerate() {
                        numerator += (2.0 * (i as f64 + 1.0) - n - 1.0) * w;
                    }
                    numerator / (n * sum)
                }
            }
        };

        // Check thresholds
        if state_coverage < self.thresholds.min_state_coverage {
            issues.push(format!(
                "State coverage {:.2} < threshold {:.2}",
                state_coverage, self.thresholds.min_state_coverage
            ));
        }
        if workload_coverage < self.thresholds.min_workload_coverage {
            issues.push(format!(
                "Workload coverage {:.2} < threshold {:.2}",
                workload_coverage, self.thresholds.min_workload_coverage
            ));
        }
        if correlation_coverage < self.thresholds.min_correlation_coverage {
            issues.push(format!(
                "Correlation coverage {:.2} < threshold {:.2}",
                correlation_coverage, self.thresholds.min_correlation_coverage
            ));
        }
        if workload_gini > self.thresholds.max_workload_imbalance {
            issues.push(format!(
                "Workload Gini {:.3} > threshold {:.3}",
                workload_gini, self.thresholds.max_workload_imbalance
            ));
        }

        let passes = state_coverage >= self.thresholds.min_state_coverage
            && workload_coverage >= self.thresholds.min_workload_coverage
            && correlation_coverage >= self.thresholds.min_correlation_coverage
            && workload_gini <= self.thresholds.max_workload_imbalance;

        OcelEnrichmentAnalysis {
            state_coverage,
            workload_coverage,
            correlation_coverage,
            workload_gini,
            unique_states,
            unique_correlations,
            passes,
            issues,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_fully_enriched_events(count: usize) -> Vec<OcelEnrichmentData> {
        (0..count)
            .map(|i| OcelEnrichmentData {
                event_id: format!("EVT-{:04}", i),
                has_from_state: true,
                has_to_state: true,
                has_resource_workload: true,
                has_correlation_id: i % 5 == 0, // 20% have correlations
                from_state: Some(format!("state_{}", i % 3)),
                to_state: Some(format!("state_{}", (i + 1) % 3)),
                resource_workload: Some(1.0 + (i % 4) as f64),
            })
            .collect()
    }

    #[test]
    fn test_fully_enriched_passes() {
        let analyzer = OcelEnrichmentAnalyzer::with_defaults();
        let events = make_fully_enriched_events(100);
        let result = analyzer.analyze(&events);
        assert!(result.passes, "issues: {:?}", result.issues);
        assert_eq!(result.state_coverage, 1.0);
        assert_eq!(result.workload_coverage, 1.0);
        assert!(result.correlation_coverage >= 0.10);
        assert!(result.unique_states > 0);
    }

    #[test]
    fn test_no_states_fails_coverage() {
        let analyzer = OcelEnrichmentAnalyzer::with_defaults();
        let events: Vec<OcelEnrichmentData> = (0..50)
            .map(|i| OcelEnrichmentData {
                event_id: format!("EVT-{}", i),
                has_from_state: false,
                has_to_state: false,
                has_resource_workload: true,
                has_correlation_id: true,
                from_state: None,
                to_state: None,
                resource_workload: Some(1.0),
            })
            .collect();
        let result = analyzer.analyze(&events);
        assert!(!result.passes);
        assert_eq!(result.state_coverage, 0.0);
        assert!(result.issues.iter().any(|i| i.contains("State coverage")));
    }

    #[test]
    fn test_no_workload_fails() {
        let analyzer = OcelEnrichmentAnalyzer::with_defaults();
        let events: Vec<OcelEnrichmentData> = (0..50)
            .map(|i| OcelEnrichmentData {
                event_id: format!("EVT-{}", i),
                has_from_state: true,
                has_to_state: true,
                has_resource_workload: false,
                has_correlation_id: true,
                from_state: Some("A".into()),
                to_state: Some("B".into()),
                resource_workload: None,
            })
            .collect();
        let result = analyzer.analyze(&events);
        assert!(!result.passes);
        assert!(result
            .issues
            .iter()
            .any(|i| i.contains("Workload coverage")));
    }

    #[test]
    fn test_no_correlations_fails() {
        let analyzer = OcelEnrichmentAnalyzer::with_defaults();
        let events: Vec<OcelEnrichmentData> = (0..50)
            .map(|i| OcelEnrichmentData {
                event_id: format!("EVT-{}", i),
                has_from_state: true,
                has_to_state: true,
                has_resource_workload: true,
                has_correlation_id: false,
                from_state: Some("A".into()),
                to_state: Some("B".into()),
                resource_workload: Some(1.0),
            })
            .collect();
        let result = analyzer.analyze(&events);
        assert!(!result.passes);
        assert!(result
            .issues
            .iter()
            .any(|i| i.contains("Correlation coverage")));
    }

    #[test]
    fn test_empty_events_fails() {
        let analyzer = OcelEnrichmentAnalyzer::with_defaults();
        let result = analyzer.analyze(&[]);
        assert!(!result.passes);
        assert_eq!(result.unique_states, 0);
    }

    #[test]
    fn test_workload_gini_balanced() {
        let analyzer = OcelEnrichmentAnalyzer::with_defaults();
        // All same workload = Gini of 0
        let events: Vec<OcelEnrichmentData> = (0..20)
            .map(|i| OcelEnrichmentData {
                event_id: format!("EVT-{}", i),
                has_from_state: true,
                has_to_state: true,
                has_resource_workload: true,
                has_correlation_id: true,
                from_state: Some("A".into()),
                to_state: Some("B".into()),
                resource_workload: Some(5.0),
            })
            .collect();
        let result = analyzer.analyze(&events);
        assert!(result.workload_gini < 0.01);
    }
}
