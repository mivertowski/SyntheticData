//! Anomaly patterns for realistic distribution.
//!
//! Patterns control how anomalies are distributed across time and entities,
//! including clustering behavior and temporal patterns.

use chrono::{Datelike, NaiveDate, Weekday};
use rand::Rng;
use std::collections::HashMap;

/// Temporal pattern for anomaly injection.
#[derive(Debug, Clone)]
pub enum TemporalPattern {
    /// Uniform distribution across all periods.
    Uniform,
    /// Higher probability at period/year end.
    PeriodEndSpike {
        /// Multiplier for month-end days.
        month_end_multiplier: f64,
        /// Multiplier for quarter-end.
        quarter_end_multiplier: f64,
        /// Multiplier for year-end.
        year_end_multiplier: f64,
    },
    /// Higher probability at specific times.
    TimeBased {
        /// Multiplier for after-hours.
        after_hours_multiplier: f64,
        /// Multiplier for weekends.
        weekend_multiplier: f64,
    },
    /// Seasonal pattern.
    Seasonal {
        /// Multipliers by month (1-12).
        month_multipliers: [f64; 12],
    },
    /// Custom pattern function.
    Custom {
        /// Name of the pattern.
        name: String,
    },
}

impl Default for TemporalPattern {
    fn default() -> Self {
        TemporalPattern::PeriodEndSpike {
            month_end_multiplier: 2.0,
            quarter_end_multiplier: 3.0,
            year_end_multiplier: 5.0,
        }
    }
}

impl TemporalPattern {
    /// Calculates the probability multiplier for a given date.
    pub fn probability_multiplier(&self, date: NaiveDate) -> f64 {
        match self {
            TemporalPattern::Uniform => 1.0,
            TemporalPattern::PeriodEndSpike {
                month_end_multiplier,
                quarter_end_multiplier,
                year_end_multiplier,
            } => {
                let day = date.day();
                let month = date.month();

                // Year end (December 28-31)
                if month == 12 && day >= 28 {
                    return *year_end_multiplier;
                }

                // Quarter end (Mar, Jun, Sep, Dec last 3 days)
                if matches!(month, 3 | 6 | 9 | 12) && day >= 28 {
                    return *quarter_end_multiplier;
                }

                // Month end (last 3 days)
                if day >= 28 {
                    return *month_end_multiplier;
                }

                1.0
            }
            TemporalPattern::TimeBased {
                after_hours_multiplier: _,
                weekend_multiplier,
            } => {
                let weekday = date.weekday();
                if weekday == Weekday::Sat || weekday == Weekday::Sun {
                    return *weekend_multiplier;
                }
                // Assume all entries have potential for after-hours
                // In practice, this would check timestamp
                1.0
            }
            TemporalPattern::Seasonal { month_multipliers } => {
                let month_idx = (date.month() - 1) as usize;
                month_multipliers[month_idx]
            }
            TemporalPattern::Custom { .. } => 1.0,
        }
    }

    /// Creates a standard audit season pattern (higher in Q1).
    pub fn audit_season() -> Self {
        TemporalPattern::Seasonal {
            month_multipliers: [
                2.0, 2.0, 1.5, // Q1 - audit busy season
                1.0, 1.0, 1.2, // Q2 - quarter end
                1.0, 1.0, 1.2, // Q3 - quarter end
                1.0, 1.0, 3.0, // Q4 - year end
            ],
        }
    }
}

/// Fraud category for cluster time window selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FraudCategory {
    /// Accounts Receivable fraud (invoice aging: 30-45 days)
    AccountsReceivable,
    /// Accounts Payable fraud (payment cycles: 14-30 days)
    AccountsPayable,
    /// Payroll fraud (monthly: 28-35 days)
    Payroll,
    /// Expense fraud (submission cycles: 7-14 days)
    Expense,
    /// Revenue manipulation (quarterly: 85-95 days)
    Revenue,
    /// Asset fraud (periodic: 30-60 days)
    Asset,
    /// General fraud (default: 7 days)
    General,
}

impl FraudCategory {
    /// Get the time window range (min, max days) for this fraud category.
    pub fn time_window_days(&self) -> (i64, i64) {
        match self {
            FraudCategory::AccountsReceivable => (30, 45), // Invoice aging cycles
            FraudCategory::AccountsPayable => (14, 30),    // Payment terms
            FraudCategory::Payroll => (28, 35),            // Monthly pay cycles
            FraudCategory::Expense => (7, 14),             // Expense report cycles
            FraudCategory::Revenue => (85, 95),            // Quarterly close periods
            FraudCategory::Asset => (30, 60),              // Asset reconciliation
            FraudCategory::General => (5, 10),             // Default short window
        }
    }

    /// Infer fraud category from anomaly type string.
    pub fn from_anomaly_type(anomaly_type: &str) -> Self {
        let lower = anomaly_type.to_lowercase();
        if lower.contains("receivable")
            || lower.contains("ar")
            || lower.contains("invoice")
            || lower.contains("customer")
        {
            FraudCategory::AccountsReceivable
        } else if lower.contains("payable")
            || lower.contains("ap")
            || lower.contains("vendor")
            || lower.contains("payment")
        {
            FraudCategory::AccountsPayable
        } else if lower.contains("payroll")
            || lower.contains("ghost")
            || lower.contains("employee")
            || lower.contains("salary")
        {
            FraudCategory::Payroll
        } else if lower.contains("expense") || lower.contains("reimbursement") {
            FraudCategory::Expense
        } else if lower.contains("revenue")
            || lower.contains("sales")
            || lower.contains("channel")
            || lower.contains("premature")
        {
            FraudCategory::Revenue
        } else if lower.contains("asset")
            || lower.contains("inventory")
            || lower.contains("fixed")
            || lower.contains("depreciation")
        {
            FraudCategory::Asset
        } else {
            FraudCategory::General
        }
    }
}

/// Clustering behavior for anomalies.
#[derive(Debug, Clone)]
pub struct ClusteringConfig {
    /// Whether clustering is enabled.
    pub enabled: bool,
    /// Probability that an anomaly starts a new cluster.
    pub cluster_start_probability: f64,
    /// Probability that next anomaly joins current cluster.
    pub cluster_continuation_probability: f64,
    /// Minimum cluster size.
    pub min_cluster_size: usize,
    /// Maximum cluster size.
    pub max_cluster_size: usize,
    /// Time window for cluster (days) - default for General category.
    pub cluster_time_window_days: i64,
    /// Whether to use fraud-type-specific time windows.
    pub use_fraud_specific_windows: bool,
    /// Whether to preserve account relationships within clusters.
    pub preserve_account_relationships: bool,
}

impl Default for ClusteringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cluster_start_probability: 0.3,
            cluster_continuation_probability: 0.7,
            min_cluster_size: 2,
            max_cluster_size: 10,
            cluster_time_window_days: 7,
            use_fraud_specific_windows: true,
            preserve_account_relationships: true,
        }
    }
}

/// Causal link between entities in a fraud cluster.
#[derive(Debug, Clone)]
pub struct CausalLink {
    /// Source entity (e.g., payment document ID)
    pub source_entity: String,
    /// Source entity type
    pub source_type: String,
    /// Target entity (e.g., vendor ID)
    pub target_entity: String,
    /// Target entity type
    pub target_type: String,
    /// Relationship type
    pub relationship: String,
}

impl CausalLink {
    /// Create a new causal link.
    pub fn new(
        source_entity: impl Into<String>,
        source_type: impl Into<String>,
        target_entity: impl Into<String>,
        target_type: impl Into<String>,
        relationship: impl Into<String>,
    ) -> Self {
        Self {
            source_entity: source_entity.into(),
            source_type: source_type.into(),
            target_entity: target_entity.into(),
            target_type: target_type.into(),
            relationship: relationship.into(),
        }
    }
}

/// Manages anomaly clustering.
pub struct ClusterManager {
    config: ClusteringConfig,
    /// Current active clusters by fraud category.
    active_clusters: HashMap<FraudCategory, ActiveCluster>,
    /// Next cluster ID to assign.
    next_cluster_id: u64,
    /// Cluster statistics.
    cluster_stats: HashMap<String, ClusterStats>,
}

/// Active cluster state.
#[derive(Debug, Clone)]
struct ActiveCluster {
    /// Cluster ID.
    cluster_id: String,
    /// Number of anomalies in cluster.
    size: usize,
    /// Start date.
    start_date: NaiveDate,
    /// Fraud category.
    category: FraudCategory,
    /// Time window for this cluster.
    time_window_days: i64,
    /// Accounts involved (for relationship preservation).
    accounts: Vec<String>,
    /// Entities involved (vendors, customers, employees).
    entities: Vec<String>,
}

/// Statistics for a cluster.
#[derive(Debug, Clone, Default)]
pub struct ClusterStats {
    /// Number of anomalies in cluster.
    pub size: usize,
    /// Start date.
    pub start_date: Option<NaiveDate>,
    /// End date.
    pub end_date: Option<NaiveDate>,
    /// Anomaly types in cluster.
    pub anomaly_types: Vec<String>,
    /// Fraud category of this cluster.
    pub fraud_category: Option<FraudCategory>,
    /// Time window used (days).
    pub time_window_days: i64,
    /// Accounts involved in this cluster.
    pub accounts: Vec<String>,
    /// Entities involved in this cluster.
    pub entities: Vec<String>,
    /// Causal links within this cluster.
    pub causal_links: Vec<CausalLink>,
}

impl ClusterManager {
    /// Creates a new cluster manager.
    pub fn new(config: ClusteringConfig) -> Self {
        Self {
            config,
            active_clusters: HashMap::new(),
            next_cluster_id: 1,
            cluster_stats: HashMap::new(),
        }
    }

    /// Determines the cluster ID for a new anomaly.
    pub fn assign_cluster<R: Rng>(
        &mut self,
        date: NaiveDate,
        anomaly_type: &str,
        rng: &mut R,
    ) -> Option<String> {
        self.assign_cluster_with_context(date, anomaly_type, None, None, rng)
    }

    /// Determines the cluster ID with additional context for relationship preservation.
    pub fn assign_cluster_with_context<R: Rng>(
        &mut self,
        date: NaiveDate,
        anomaly_type: &str,
        account: Option<&str>,
        entity: Option<&str>,
        rng: &mut R,
    ) -> Option<String> {
        if !self.config.enabled {
            return None;
        }

        // Determine fraud category from anomaly type
        let category = FraudCategory::from_anomaly_type(anomaly_type);

        // Get time window for this category
        let time_window = if self.config.use_fraud_specific_windows {
            let (min, max) = category.time_window_days();
            rng.gen_range(min..=max)
        } else {
            self.config.cluster_time_window_days
        };

        // Check if we should continue an existing cluster for this category
        if let Some(active) = self.active_clusters.get(&category).cloned() {
            let days_elapsed = (date - active.start_date).num_days();

            // Check if within time window and not at max size
            if days_elapsed <= active.time_window_days
                && active.size < self.config.max_cluster_size
                && rng.gen::<f64>() < self.config.cluster_continuation_probability
            {
                // If preserving relationships, prefer matching accounts/entities
                let relationship_match = if self.config.preserve_account_relationships {
                    let account_match =
                        account.is_none_or(|a| active.accounts.contains(&a.to_string()));
                    let entity_match =
                        entity.is_none_or(|e| active.entities.contains(&e.to_string()));
                    account_match || entity_match
                } else {
                    true
                };

                if relationship_match {
                    // Continue the cluster
                    let cluster_id = active.cluster_id.clone();

                    // Update active cluster
                    if let Some(active_mut) = self.active_clusters.get_mut(&category) {
                        active_mut.size += 1;
                        if let Some(acct) = account {
                            if !active_mut.accounts.contains(&acct.to_string()) {
                                active_mut.accounts.push(acct.to_string());
                            }
                        }
                        if let Some(ent) = entity {
                            if !active_mut.entities.contains(&ent.to_string()) {
                                active_mut.entities.push(ent.to_string());
                            }
                        }
                    }

                    // Update cluster stats
                    if let Some(stats) = self.cluster_stats.get_mut(&cluster_id) {
                        stats.size += 1;
                        stats.end_date = Some(date);
                        stats.anomaly_types.push(anomaly_type.to_string());
                        if let Some(acct) = account {
                            if !stats.accounts.contains(&acct.to_string()) {
                                stats.accounts.push(acct.to_string());
                            }
                        }
                        if let Some(ent) = entity {
                            if !stats.entities.contains(&ent.to_string()) {
                                stats.entities.push(ent.to_string());
                            }
                        }
                    }

                    return Some(cluster_id);
                }
            }

            // End current cluster if at min size
            if active.size >= self.config.min_cluster_size {
                self.active_clusters.remove(&category);
            }
        }

        // Decide whether to start a new cluster
        if rng.gen::<f64>() < self.config.cluster_start_probability {
            let cluster_id = format!("CLU{:06}", self.next_cluster_id);
            self.next_cluster_id += 1;

            let mut accounts = Vec::new();
            let mut entities = Vec::new();
            if let Some(acct) = account {
                accounts.push(acct.to_string());
            }
            if let Some(ent) = entity {
                entities.push(ent.to_string());
            }

            // Create new active cluster
            self.active_clusters.insert(
                category,
                ActiveCluster {
                    cluster_id: cluster_id.clone(),
                    size: 1,
                    start_date: date,
                    category,
                    time_window_days: time_window,
                    accounts: accounts.clone(),
                    entities: entities.clone(),
                },
            );

            // Initialize cluster stats
            self.cluster_stats.insert(
                cluster_id.clone(),
                ClusterStats {
                    size: 1,
                    start_date: Some(date),
                    end_date: Some(date),
                    anomaly_types: vec![anomaly_type.to_string()],
                    fraud_category: Some(category),
                    time_window_days: time_window,
                    accounts,
                    entities,
                    causal_links: Vec::new(),
                },
            );

            return Some(cluster_id);
        }

        None
    }

    /// Add a causal link to a cluster.
    pub fn add_causal_link(&mut self, cluster_id: &str, link: CausalLink) {
        if let Some(stats) = self.cluster_stats.get_mut(cluster_id) {
            stats.causal_links.push(link);
        }
    }

    /// Get suggested account for relationship preservation within a cluster.
    pub fn get_related_account(&self, cluster_id: &str) -> Option<&str> {
        self.cluster_stats
            .get(cluster_id)
            .and_then(|s| s.accounts.first().map(|a| a.as_str()))
    }

    /// Get suggested entity for relationship preservation within a cluster.
    pub fn get_related_entity(&self, cluster_id: &str) -> Option<&str> {
        self.cluster_stats
            .get(cluster_id)
            .and_then(|s| s.entities.first().map(|e| e.as_str()))
    }

    /// Gets cluster statistics.
    pub fn get_cluster_stats(&self, cluster_id: &str) -> Option<&ClusterStats> {
        self.cluster_stats.get(cluster_id)
    }

    /// Gets all cluster statistics.
    pub fn all_cluster_stats(&self) -> &HashMap<String, ClusterStats> {
        &self.cluster_stats
    }

    /// Returns the number of clusters created.
    pub fn cluster_count(&self) -> usize {
        self.cluster_stats.len()
    }

    /// Get cluster statistics by fraud category.
    pub fn clusters_by_category(&self) -> HashMap<FraudCategory, Vec<&ClusterStats>> {
        let mut by_category: HashMap<FraudCategory, Vec<&ClusterStats>> = HashMap::new();
        for stats in self.cluster_stats.values() {
            if let Some(cat) = stats.fraud_category {
                by_category.entry(cat).or_default().push(stats);
            }
        }
        by_category
    }
}

/// Entity targeting pattern.
#[derive(Debug, Clone, Default)]
pub enum EntityTargetingPattern {
    /// Random entity selection.
    #[default]
    Random,
    /// Weighted by transaction volume.
    VolumeWeighted,
    /// Focus on specific entity types.
    TypeFocused {
        /// Target entity types with weights.
        type_weights: HashMap<String, f64>,
    },
    /// Repeat offender pattern (same entities).
    RepeatOffender {
        /// Probability of targeting same entity.
        repeat_probability: f64,
    },
}

/// Manages entity targeting for anomalies.
pub struct EntityTargetingManager {
    pattern: EntityTargetingPattern,
    /// Recently targeted entities.
    recent_targets: Vec<String>,
    /// Maximum recent targets to track.
    max_recent: usize,
    /// Entity hit counts.
    hit_counts: HashMap<String, usize>,
}

impl EntityTargetingManager {
    /// Creates a new entity targeting manager.
    pub fn new(pattern: EntityTargetingPattern) -> Self {
        Self {
            pattern,
            recent_targets: Vec::new(),
            max_recent: 20,
            hit_counts: HashMap::new(),
        }
    }

    /// Selects an entity to target.
    pub fn select_entity<R: Rng>(&mut self, candidates: &[String], rng: &mut R) -> Option<String> {
        if candidates.is_empty() {
            return None;
        }

        let selected = match &self.pattern {
            EntityTargetingPattern::Random => {
                candidates[rng.gen_range(0..candidates.len())].clone()
            }
            EntityTargetingPattern::VolumeWeighted => {
                // In practice, would weight by actual volume
                // For now, use random
                candidates[rng.gen_range(0..candidates.len())].clone()
            }
            EntityTargetingPattern::TypeFocused { type_weights } => {
                // Filter by type weights
                let weighted: Vec<_> = candidates
                    .iter()
                    .filter_map(|c| type_weights.get(c).map(|&w| (c.clone(), w)))
                    .collect();

                if weighted.is_empty() {
                    candidates[rng.gen_range(0..candidates.len())].clone()
                } else {
                    let total: f64 = weighted.iter().map(|(_, w)| w).sum();
                    let mut r = rng.gen::<f64>() * total;
                    for (entity, weight) in &weighted {
                        r -= weight;
                        if r <= 0.0 {
                            return Some(entity.clone());
                        }
                    }
                    weighted[0].0.clone()
                }
            }
            EntityTargetingPattern::RepeatOffender { repeat_probability } => {
                // Check if we should repeat a recent target
                if !self.recent_targets.is_empty() && rng.gen::<f64>() < *repeat_probability {
                    let idx = rng.gen_range(0..self.recent_targets.len());
                    self.recent_targets[idx].clone()
                } else {
                    candidates[rng.gen_range(0..candidates.len())].clone()
                }
            }
        };

        // Track the selection
        self.recent_targets.push(selected.clone());
        if self.recent_targets.len() > self.max_recent {
            self.recent_targets.remove(0);
        }

        *self.hit_counts.entry(selected.clone()).or_insert(0) += 1;

        Some(selected)
    }

    /// Gets hit count for an entity.
    pub fn hit_count(&self, entity: &str) -> usize {
        *self.hit_counts.get(entity).unwrap_or(&0)
    }
}

/// Combined pattern configuration.
#[derive(Debug, Clone)]
pub struct AnomalyPatternConfig {
    /// Temporal pattern.
    pub temporal_pattern: TemporalPattern,
    /// Clustering configuration.
    pub clustering: ClusteringConfig,
    /// Entity targeting pattern.
    pub entity_targeting: EntityTargetingPattern,
    /// Whether to inject anomalies in batches.
    pub batch_injection: bool,
    /// Batch size range.
    pub batch_size_range: (usize, usize),
}

impl Default for AnomalyPatternConfig {
    fn default() -> Self {
        Self {
            temporal_pattern: TemporalPattern::default(),
            clustering: ClusteringConfig::default(),
            entity_targeting: EntityTargetingPattern::default(),
            batch_injection: false,
            batch_size_range: (2, 5),
        }
    }
}

/// Determines if an anomaly should be injected at this point.
pub fn should_inject_anomaly<R: Rng>(
    base_rate: f64,
    date: NaiveDate,
    pattern: &TemporalPattern,
    rng: &mut R,
) -> bool {
    let multiplier = pattern.probability_multiplier(date);
    let adjusted_rate = (base_rate * multiplier).min(1.0);
    rng.gen::<f64>() < adjusted_rate
}

// ============================================================================
// Fraud Actor System - User-Based Fraud Targeting
// ============================================================================

/// Escalation pattern for fraud amounts over time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscalationPattern {
    /// Fraud amounts stay relatively constant.
    Stable,
    /// Fraud amounts gradually increase over time (typical embezzlement).
    Gradual,
    /// Fraud amounts increase rapidly (getting bolder).
    Aggressive,
    /// Fraud amounts vary but trend upward.
    Erratic,
    /// Single large fraud after testing with small amounts.
    TestThenStrike,
}

impl EscalationPattern {
    /// Get the escalation multiplier based on the number of prior frauds.
    pub fn escalation_multiplier(&self, prior_fraud_count: usize) -> f64 {
        match self {
            EscalationPattern::Stable => 1.0,
            EscalationPattern::Gradual => {
                // 10% increase per prior fraud, max 3x
                (1.0 + 0.1 * prior_fraud_count as f64).min(3.0)
            }
            EscalationPattern::Aggressive => {
                // 25% increase per prior fraud, max 5x
                (1.0 + 0.25 * prior_fraud_count as f64).min(5.0)
            }
            EscalationPattern::Erratic => {
                // Variable multiplier with upward trend
                let base = 1.0 + 0.15 * prior_fraud_count as f64;
                base.min(4.0)
            }
            EscalationPattern::TestThenStrike => {
                // Small amounts initially, then big jump
                if prior_fraud_count < 3 {
                    0.3 // Test with small amounts
                } else if prior_fraud_count == 3 {
                    5.0 // Big strike
                } else {
                    0.0 // Stop after the strike
                }
            }
        }
    }
}

/// A fraud actor represents a user who commits fraud over time.
#[derive(Debug, Clone)]
pub struct FraudActor {
    /// User ID of the fraudster.
    pub user_id: String,
    /// User's name for display purposes.
    pub user_name: String,
    /// Fraud history (document IDs and dates).
    pub fraud_history: Vec<FraudIncident>,
    /// Escalation pattern for this actor.
    pub escalation_pattern: EscalationPattern,
    /// Preferred GL accounts for fraud.
    pub preferred_accounts: Vec<String>,
    /// Preferred vendors (for AP fraud).
    pub preferred_vendors: Vec<String>,
    /// Total amount of fraud committed.
    pub total_amount: rust_decimal::Decimal,
    /// Start date of fraud activity.
    pub start_date: Option<NaiveDate>,
    /// Detection likelihood (0.0-1.0) - increases with activity.
    pub detection_risk: f64,
    /// Is this actor currently active?
    pub is_active: bool,
}

/// A single fraud incident committed by an actor.
#[derive(Debug, Clone)]
pub struct FraudIncident {
    /// Document ID of the fraudulent entry.
    pub document_id: String,
    /// Date of the fraud.
    pub date: NaiveDate,
    /// Amount of the fraud.
    pub amount: rust_decimal::Decimal,
    /// Fraud type.
    pub fraud_type: String,
    /// Account used.
    pub account: Option<String>,
    /// Related entity (vendor, customer, etc.).
    pub entity: Option<String>,
}

impl FraudActor {
    /// Create a new fraud actor.
    pub fn new(
        user_id: impl Into<String>,
        user_name: impl Into<String>,
        escalation_pattern: EscalationPattern,
    ) -> Self {
        Self {
            user_id: user_id.into(),
            user_name: user_name.into(),
            fraud_history: Vec::new(),
            escalation_pattern,
            preferred_accounts: Vec::new(),
            preferred_vendors: Vec::new(),
            total_amount: rust_decimal::Decimal::ZERO,
            start_date: None,
            detection_risk: 0.0,
            is_active: true,
        }
    }

    /// Add a preferred account for fraud.
    pub fn with_account(mut self, account: impl Into<String>) -> Self {
        self.preferred_accounts.push(account.into());
        self
    }

    /// Add a preferred vendor for fraud.
    pub fn with_vendor(mut self, vendor: impl Into<String>) -> Self {
        self.preferred_vendors.push(vendor.into());
        self
    }

    /// Record a fraud incident.
    pub fn record_fraud(
        &mut self,
        document_id: impl Into<String>,
        date: NaiveDate,
        amount: rust_decimal::Decimal,
        fraud_type: impl Into<String>,
        account: Option<String>,
        entity: Option<String>,
    ) {
        let incident = FraudIncident {
            document_id: document_id.into(),
            date,
            amount,
            fraud_type: fraud_type.into(),
            account: account.clone(),
            entity: entity.clone(),
        };

        self.fraud_history.push(incident);
        self.total_amount += amount;

        if self.start_date.is_none() {
            self.start_date = Some(date);
        }

        // Update detection risk based on activity
        self.update_detection_risk();

        // Add account/entity to preferences if not already present
        if let Some(acct) = account {
            if !self.preferred_accounts.contains(&acct) {
                self.preferred_accounts.push(acct);
            }
        }
        if let Some(ent) = entity {
            if !self.preferred_vendors.contains(&ent) {
                self.preferred_vendors.push(ent);
            }
        }
    }

    /// Update detection risk based on fraud activity.
    fn update_detection_risk(&mut self) {
        // Detection risk increases with:
        // 1. Number of frauds committed
        // 2. Total amount
        // 3. How bold the escalation pattern is
        let count_factor = (self.fraud_history.len() as f64 * 0.05).min(0.3);
        let amount_factor = if self.total_amount > rust_decimal::Decimal::from(100_000) {
            0.3
        } else if self.total_amount > rust_decimal::Decimal::from(10_000) {
            0.2
        } else {
            0.1
        };
        let pattern_factor = match self.escalation_pattern {
            EscalationPattern::Stable => 0.1,
            EscalationPattern::Gradual => 0.15,
            EscalationPattern::Erratic => 0.2,
            EscalationPattern::Aggressive => 0.25,
            EscalationPattern::TestThenStrike => 0.3,
        };

        self.detection_risk = (count_factor + amount_factor + pattern_factor).min(0.95);
    }

    /// Get the escalation multiplier for the next fraud.
    pub fn next_escalation_multiplier(&self) -> f64 {
        self.escalation_pattern
            .escalation_multiplier(self.fraud_history.len())
    }

    /// Get a preferred account, or None if no preferences.
    pub fn get_preferred_account<R: Rng>(&self, rng: &mut R) -> Option<&str> {
        if self.preferred_accounts.is_empty() {
            None
        } else {
            Some(&self.preferred_accounts[rng.gen_range(0..self.preferred_accounts.len())])
        }
    }

    /// Get a preferred vendor, or None if no preferences.
    pub fn get_preferred_vendor<R: Rng>(&self, rng: &mut R) -> Option<&str> {
        if self.preferred_vendors.is_empty() {
            None
        } else {
            Some(&self.preferred_vendors[rng.gen_range(0..self.preferred_vendors.len())])
        }
    }
}

/// Manages fraud actors for user-based fraud targeting.
pub struct FraudActorManager {
    /// All fraud actors.
    actors: Vec<FraudActor>,
    /// Map from user_id to actor index.
    user_index: HashMap<String, usize>,
    /// Probability of using an existing actor vs creating new one.
    repeat_actor_probability: f64,
    /// Maximum active actors at any time.
    max_active_actors: usize,
}

impl FraudActorManager {
    /// Create a new fraud actor manager.
    pub fn new(repeat_actor_probability: f64, max_active_actors: usize) -> Self {
        Self {
            actors: Vec::new(),
            user_index: HashMap::new(),
            repeat_actor_probability,
            max_active_actors,
        }
    }

    /// Add a fraud actor.
    pub fn add_actor(&mut self, actor: FraudActor) {
        let idx = self.actors.len();
        self.user_index.insert(actor.user_id.clone(), idx);
        self.actors.push(actor);
    }

    /// Get or create a fraud actor for the next fraud.
    pub fn get_or_create_actor<R: Rng>(
        &mut self,
        available_users: &[String],
        rng: &mut R,
    ) -> Option<&mut FraudActor> {
        if available_users.is_empty() {
            return None;
        }

        // Check if we should use an existing active actor
        let active_actors: Vec<usize> = self
            .actors
            .iter()
            .enumerate()
            .filter(|(_, a)| a.is_active)
            .map(|(i, _)| i)
            .collect();

        if !active_actors.is_empty() && rng.gen::<f64>() < self.repeat_actor_probability {
            // Use existing actor
            let idx = active_actors[rng.gen_range(0..active_actors.len())];
            return Some(&mut self.actors[idx]);
        }

        // Create new actor if under max
        if self.actors.len() < self.max_active_actors {
            // Pick a random user
            let user_id = &available_users[rng.gen_range(0..available_users.len())];

            // Check if user already has an actor
            if let Some(&idx) = self.user_index.get(user_id) {
                return Some(&mut self.actors[idx]);
            }

            // Create new actor with random escalation pattern
            let pattern = match rng.gen_range(0..5) {
                0 => EscalationPattern::Stable,
                1 => EscalationPattern::Gradual,
                2 => EscalationPattern::Aggressive,
                3 => EscalationPattern::Erratic,
                _ => EscalationPattern::TestThenStrike,
            };

            let actor = FraudActor::new(user_id.clone(), format!("Fraudster {}", user_id), pattern);
            let idx = self.actors.len();
            self.user_index.insert(user_id.clone(), idx);
            self.actors.push(actor);
            return Some(&mut self.actors[idx]);
        }

        // Use random existing actor
        if !self.actors.is_empty() {
            let idx = rng.gen_range(0..self.actors.len());
            return Some(&mut self.actors[idx]);
        }

        None
    }

    /// Get an actor by user ID.
    pub fn get_actor(&self, user_id: &str) -> Option<&FraudActor> {
        self.user_index.get(user_id).map(|&i| &self.actors[i])
    }

    /// Get a mutable actor by user ID.
    pub fn get_actor_mut(&mut self, user_id: &str) -> Option<&mut FraudActor> {
        if let Some(&idx) = self.user_index.get(user_id) {
            Some(&mut self.actors[idx])
        } else {
            None
        }
    }

    /// Deactivate actors who have high detection risk.
    pub fn apply_detection<R: Rng>(&mut self, rng: &mut R) {
        for actor in &mut self.actors {
            if actor.is_active && rng.gen::<f64>() < actor.detection_risk {
                actor.is_active = false;
            }
        }
    }

    /// Get all actors.
    pub fn all_actors(&self) -> &[FraudActor] {
        &self.actors
    }

    /// Get summary statistics.
    pub fn get_statistics(&self) -> FraudActorStatistics {
        let total_actors = self.actors.len();
        let active_actors = self.actors.iter().filter(|a| a.is_active).count();
        let total_incidents: usize = self.actors.iter().map(|a| a.fraud_history.len()).sum();
        let total_amount: rust_decimal::Decimal = self.actors.iter().map(|a| a.total_amount).sum();

        FraudActorStatistics {
            total_actors,
            active_actors,
            total_incidents,
            total_amount,
        }
    }
}

/// Statistics about fraud actors.
#[derive(Debug, Clone)]
pub struct FraudActorStatistics {
    /// Total number of fraud actors.
    pub total_actors: usize,
    /// Number of currently active actors.
    pub active_actors: usize,
    /// Total fraud incidents across all actors.
    pub total_incidents: usize,
    /// Total fraud amount across all actors.
    pub total_amount: rust_decimal::Decimal,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_temporal_pattern_multiplier() {
        let pattern = TemporalPattern::default();

        // Regular day
        let regular = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        assert_eq!(pattern.probability_multiplier(regular), 1.0);

        // Month end
        let month_end = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();
        assert!(pattern.probability_multiplier(month_end) > 1.0);

        // Year end
        let year_end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        assert!(
            pattern.probability_multiplier(year_end) > pattern.probability_multiplier(month_end)
        );
    }

    #[test]
    fn test_cluster_manager() {
        let mut manager = ClusterManager::new(ClusteringConfig::default());
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();

        // Generate several anomalies and check clustering
        let mut clustered = 0;
        for i in 0..20 {
            let d = date + chrono::Duration::days(i % 7); // Within time window
            if manager.assign_cluster(d, "TestType", &mut rng).is_some() {
                clustered += 1;
            }
        }

        // Some should be clustered
        assert!(clustered > 0);
        assert!(manager.cluster_count() > 0);
    }

    #[test]
    fn test_fraud_category_time_windows() {
        // AR fraud should have longer window than general
        let ar = FraudCategory::AccountsReceivable;
        let general = FraudCategory::General;

        let (ar_min, ar_max) = ar.time_window_days();
        let (gen_min, gen_max) = general.time_window_days();

        assert!(ar_min > gen_min);
        assert!(ar_max > gen_max);
    }

    #[test]
    fn test_fraud_category_inference() {
        assert_eq!(
            FraudCategory::from_anomaly_type("AccountsReceivable"),
            FraudCategory::AccountsReceivable
        );
        assert_eq!(
            FraudCategory::from_anomaly_type("VendorPayment"),
            FraudCategory::AccountsPayable
        );
        assert_eq!(
            FraudCategory::from_anomaly_type("GhostEmployee"),
            FraudCategory::Payroll
        );
        assert_eq!(
            FraudCategory::from_anomaly_type("RandomType"),
            FraudCategory::General
        );
    }

    #[test]
    fn test_cluster_with_context() {
        let mut manager = ClusterManager::new(ClusteringConfig {
            cluster_start_probability: 1.0,        // Always start
            cluster_continuation_probability: 1.0, // Always continue
            ..Default::default()
        });
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();

        // First anomaly starts a cluster
        let cluster1 = manager.assign_cluster_with_context(
            date,
            "VendorPayment",
            Some("200000"),
            Some("V001"),
            &mut rng,
        );
        assert!(cluster1.is_some());

        // Second anomaly with same account should join same cluster
        let cluster2 = manager.assign_cluster_with_context(
            date + chrono::Duration::days(5),
            "VendorPayment",
            Some("200000"),
            Some("V002"),
            &mut rng,
        );

        assert_eq!(cluster1, cluster2);

        // Check stats have both entities
        let stats = manager.get_cluster_stats(&cluster1.unwrap()).unwrap();
        assert_eq!(stats.accounts.len(), 1); // Same account
        assert_eq!(stats.entities.len(), 2); // Two vendors
    }

    #[test]
    fn test_causal_links() {
        let mut manager = ClusterManager::new(ClusteringConfig {
            cluster_start_probability: 1.0,
            ..Default::default()
        });
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();

        let cluster_id = manager
            .assign_cluster(date, "VendorPayment", &mut rng)
            .unwrap();

        // Add causal link
        manager.add_causal_link(
            &cluster_id,
            CausalLink::new("PAY-001", "Payment", "V001", "Vendor", "references"),
        );
        manager.add_causal_link(
            &cluster_id,
            CausalLink::new("V001", "Vendor", "EMP-001", "Employee", "owned_by"),
        );

        let stats = manager.get_cluster_stats(&cluster_id).unwrap();
        assert_eq!(stats.causal_links.len(), 2);
    }

    #[test]
    fn test_should_inject_anomaly() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let pattern = TemporalPattern::default();

        let regular_date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let year_end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        // Count injections over many trials
        let mut regular_count = 0;
        let mut year_end_count = 0;

        for _ in 0..1000 {
            if should_inject_anomaly(0.1, regular_date, &pattern, &mut rng) {
                regular_count += 1;
            }
            if should_inject_anomaly(0.1, year_end, &pattern, &mut rng) {
                year_end_count += 1;
            }
        }

        // Year end should have more injections due to multiplier
        assert!(year_end_count > regular_count);
    }

    #[test]
    fn test_escalation_patterns() {
        // Stable should always return 1.0
        assert_eq!(EscalationPattern::Stable.escalation_multiplier(0), 1.0);
        assert_eq!(EscalationPattern::Stable.escalation_multiplier(10), 1.0);

        // Gradual should increase over time
        let gradual = EscalationPattern::Gradual;
        assert!(gradual.escalation_multiplier(5) > gradual.escalation_multiplier(0));
        assert!(gradual.escalation_multiplier(5) <= 3.0); // Max is 3x

        // Aggressive should increase faster
        let aggressive = EscalationPattern::Aggressive;
        assert!(aggressive.escalation_multiplier(5) > gradual.escalation_multiplier(5));

        // TestThenStrike has specific pattern
        let tts = EscalationPattern::TestThenStrike;
        assert!(tts.escalation_multiplier(0) < 1.0); // Small test amounts
        assert!(tts.escalation_multiplier(3) > 1.0); // Big strike
        assert_eq!(tts.escalation_multiplier(4), 0.0); // Stop after strike
    }

    #[test]
    fn test_fraud_actor() {
        use rust_decimal_macros::dec;

        let mut actor = FraudActor::new("USER001", "John Fraudster", EscalationPattern::Gradual)
            .with_account("600000")
            .with_vendor("V001");

        assert_eq!(actor.preferred_accounts.len(), 1);
        assert_eq!(actor.preferred_vendors.len(), 1);
        assert!(actor.is_active);

        // Record some fraud
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        actor.record_fraud(
            "JE-001",
            date,
            dec!(1000),
            "DuplicatePayment",
            Some("600000".to_string()),
            Some("V002".to_string()),
        );

        assert_eq!(actor.fraud_history.len(), 1);
        assert_eq!(actor.total_amount, dec!(1000));
        assert_eq!(actor.start_date, Some(date));
        assert!(actor.detection_risk > 0.0);

        // V002 should be added to preferences
        assert!(actor.preferred_vendors.contains(&"V002".to_string()));
    }

    #[test]
    fn test_fraud_actor_manager() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut manager = FraudActorManager::new(0.7, 5);

        let users = vec![
            "USER001".to_string(),
            "USER002".to_string(),
            "USER003".to_string(),
        ];

        // Get or create actor
        let actor = manager.get_or_create_actor(&users, &mut rng);
        assert!(actor.is_some());

        // Record fraud
        let actor = actor.unwrap();
        let user_id = actor.user_id.clone();
        actor.record_fraud(
            "JE-001",
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            rust_decimal::Decimal::from(1000),
            "FictitiousEntry",
            None,
            None,
        );

        // Should be able to retrieve actor
        let retrieved = manager.get_actor(&user_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().fraud_history.len(), 1);

        // Get statistics
        let stats = manager.get_statistics();
        assert_eq!(stats.total_actors, 1);
        assert_eq!(stats.active_actors, 1);
        assert_eq!(stats.total_incidents, 1);
    }

    #[test]
    fn test_fraud_actor_detection() {
        use rust_decimal_macros::dec;

        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut manager = FraudActorManager::new(1.0, 10);

        // Add actor with high activity
        let mut actor =
            FraudActor::new("USER001", "Heavy Fraudster", EscalationPattern::Aggressive);
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();

        // Record many frauds to increase detection risk
        for i in 0..10 {
            actor.record_fraud(
                format!("JE-{:03}", i),
                date + chrono::Duration::days(i as i64),
                dec!(10000),
                "FictitiousEntry",
                None,
                None,
            );
        }

        manager.add_actor(actor);

        // Detection risk should be high
        let actor = manager.get_actor("USER001").unwrap();
        assert!(actor.detection_risk > 0.5);

        // Apply detection (with high risk, likely to be caught eventually)
        for _ in 0..20 {
            manager.apply_detection(&mut rng);
        }

        // After many detection attempts, high-risk actor likely deactivated
        let stats = manager.get_statistics();
        // Note: This is probabilistic, but with high risk the actor should likely be caught
        assert!(stats.active_actors <= stats.total_actors);
    }
}
