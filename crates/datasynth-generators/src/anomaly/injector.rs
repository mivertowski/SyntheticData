//! Main anomaly injection engine.
//!
//! The injector coordinates anomaly generation across all data types,
//! managing rates, patterns, clustering, and label generation.
//!
//! ## Enhanced Features (v0.3.0+)
//!
//! - **Multi-stage fraud schemes**: Embezzlement, revenue manipulation, kickbacks
//! - **Correlated injection**: Co-occurrence patterns and error cascades
//! - **Near-miss generation**: Suspicious but legitimate transactions
//! - **Detection difficulty classification**: Trivial to expert levels
//! - **Context-aware injection**: Entity-specific anomaly patterns

use chrono::NaiveDate;
use datasynth_core::utils::seeded_rng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use std::collections::HashMap;
use tracing::debug;

use datasynth_core::models::{
    AnomalyCausalReason, AnomalyDetectionDifficulty, AnomalyRateConfig, AnomalySummary,
    AnomalyType, ErrorType, FraudType, JournalEntry, LabeledAnomaly, NearMissLabel,
    RelationalAnomalyType,
};
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};

use super::context::{
    AccountContext, BehavioralBaseline, BehavioralBaselineConfig, EmployeeContext,
    EntityAwareInjector, VendorContext,
};
use super::correlation::{AnomalyCoOccurrence, TemporalClusterGenerator};
use super::difficulty::DifficultyCalculator;
use super::near_miss::{NearMissConfig, NearMissGenerator};
use super::patterns::{
    should_inject_anomaly, AnomalyPatternConfig, ClusterManager, EntityTargetingManager,
    TemporalPattern,
};
use super::scheme_advancer::{SchemeAdvancer, SchemeAdvancerConfig};
use super::schemes::{SchemeAction, SchemeContext};
use super::strategies::{DuplicationStrategy, StrategyCollection};
use super::types::AnomalyTypeSelector;

/// Configuration for the anomaly injector.
#[derive(Debug, Clone)]
pub struct AnomalyInjectorConfig {
    /// Rate configuration.
    pub rates: AnomalyRateConfig,
    /// Pattern configuration.
    pub patterns: AnomalyPatternConfig,
    /// Random seed for reproducibility.
    pub seed: u64,
    /// Whether to generate labels.
    pub generate_labels: bool,
    /// Whether to allow duplicate injection.
    pub allow_duplicates: bool,
    /// Maximum anomalies per document.
    pub max_anomalies_per_document: usize,
    /// Company codes to target (empty = all).
    pub target_companies: Vec<String>,
    /// Date range for injection.
    pub date_range: Option<(NaiveDate, NaiveDate)>,
    /// Enhanced features configuration.
    pub enhanced: EnhancedInjectionConfig,
}

/// Enhanced injection configuration for v0.3.0+ features.
#[derive(Debug, Clone, Default)]
pub struct EnhancedInjectionConfig {
    /// Enable multi-stage fraud scheme generation.
    pub multi_stage_schemes_enabled: bool,
    /// Probability of starting a new scheme per perpetrator per year.
    pub scheme_probability: f64,
    /// Enable correlated anomaly injection.
    pub correlated_injection_enabled: bool,
    /// Enable temporal clustering (period-end spikes).
    pub temporal_clustering_enabled: bool,
    /// Period-end anomaly rate multiplier.
    pub period_end_multiplier: f64,
    /// Enable near-miss generation.
    pub near_miss_enabled: bool,
    /// Proportion of anomalies that are near-misses.
    pub near_miss_proportion: f64,
    /// Approval thresholds for threshold-proximity near-misses.
    pub approval_thresholds: Vec<Decimal>,
    /// Enable detection difficulty classification.
    pub difficulty_classification_enabled: bool,
    /// Enable context-aware injection.
    pub context_aware_enabled: bool,
    /// Behavioral baseline configuration.
    pub behavioral_baseline_config: BehavioralBaselineConfig,
}

impl Default for AnomalyInjectorConfig {
    fn default() -> Self {
        Self {
            rates: AnomalyRateConfig::default(),
            patterns: AnomalyPatternConfig::default(),
            seed: 42,
            generate_labels: true,
            allow_duplicates: true,
            max_anomalies_per_document: 2,
            target_companies: Vec::new(),
            date_range: None,
            enhanced: EnhancedInjectionConfig::default(),
        }
    }
}

/// Result of an injection batch.
#[derive(Debug, Clone)]
pub struct InjectionBatchResult {
    /// Number of entries processed.
    pub entries_processed: usize,
    /// Number of anomalies injected.
    pub anomalies_injected: usize,
    /// Number of duplicates created.
    pub duplicates_created: usize,
    /// Labels generated.
    pub labels: Vec<LabeledAnomaly>,
    /// Summary of anomalies.
    pub summary: AnomalySummary,
    /// Entries that were modified (document numbers).
    pub modified_documents: Vec<String>,
    /// Near-miss labels (suspicious but legitimate transactions).
    pub near_miss_labels: Vec<NearMissLabel>,
    /// Multi-stage scheme actions generated.
    pub scheme_actions: Vec<SchemeAction>,
    /// Difficulty distribution summary.
    pub difficulty_distribution: HashMap<AnomalyDetectionDifficulty, usize>,
}

/// Main anomaly injection engine.
pub struct AnomalyInjector {
    config: AnomalyInjectorConfig,
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
    type_selector: AnomalyTypeSelector,
    strategies: StrategyCollection,
    cluster_manager: ClusterManager,
    // Constructed from config; will be consumed when entity-aware injection
    // patterns are integrated into the main inject loop.
    #[allow(dead_code)]
    entity_targeting: EntityTargetingManager,
    /// Tracking which documents already have anomalies.
    document_anomaly_counts: HashMap<String, usize>,
    /// All generated labels.
    labels: Vec<LabeledAnomaly>,
    /// Statistics.
    stats: InjectorStats,
    // Enhanced components (v0.3.0+)
    /// Multi-stage fraud scheme advancer.
    scheme_advancer: Option<SchemeAdvancer>,
    /// Near-miss generator.
    near_miss_generator: Option<NearMissGenerator>,
    /// Near-miss labels generated.
    near_miss_labels: Vec<NearMissLabel>,
    // Constructed when correlated_injection_enabled; pending integration.
    #[allow(dead_code)]
    co_occurrence_handler: Option<AnomalyCoOccurrence>,
    // Constructed when temporal_clustering_enabled; pending integration.
    #[allow(dead_code)]
    temporal_cluster_generator: Option<TemporalClusterGenerator>,
    /// Difficulty calculator.
    difficulty_calculator: Option<DifficultyCalculator>,
    /// Entity-aware injector.
    entity_aware_injector: Option<EntityAwareInjector>,
    /// Behavioral baseline tracker.
    behavioral_baseline: Option<BehavioralBaseline>,
    /// Scheme actions generated.
    scheme_actions: Vec<SchemeAction>,
    /// Difficulty distribution.
    difficulty_distribution: HashMap<AnomalyDetectionDifficulty, usize>,
    // Entity context lookup maps for risk-adjusted injection rates
    /// Vendor contexts keyed by vendor ID.
    vendor_contexts: HashMap<String, VendorContext>,
    /// Employee contexts keyed by employee ID.
    employee_contexts: HashMap<String, EmployeeContext>,
    /// Account contexts keyed by account code.
    account_contexts: HashMap<String, AccountContext>,
}

/// Injection statistics tracking.
#[derive(Debug, Clone, Default)]
pub struct InjectorStats {
    /// Total number of entries processed.
    pub total_processed: usize,
    /// Total number of anomalies injected.
    pub total_injected: usize,
    /// Anomalies injected by category (e.g., "Fraud", "Error").
    pub by_category: HashMap<String, usize>,
    /// Anomalies injected by specific type name.
    pub by_type: HashMap<String, usize>,
    /// Anomalies injected by company code.
    pub by_company: HashMap<String, usize>,
    /// Entries skipped due to rate check.
    pub skipped_rate: usize,
    /// Entries skipped due to date range filter.
    pub skipped_date: usize,
    /// Entries skipped due to company filter.
    pub skipped_company: usize,
    /// Entries skipped due to max-anomalies-per-document limit.
    pub skipped_max_per_doc: usize,
}

impl AnomalyInjector {
    /// Creates a new anomaly injector.
    pub fn new(config: AnomalyInjectorConfig) -> Self {
        let mut rng = seeded_rng(config.seed, 0);
        let cluster_manager = ClusterManager::new(config.patterns.clustering.clone());
        let entity_targeting =
            EntityTargetingManager::new(config.patterns.entity_targeting.clone());

        // Initialize enhanced components based on configuration
        let scheme_advancer = if config.enhanced.multi_stage_schemes_enabled {
            let scheme_config = SchemeAdvancerConfig {
                embezzlement_probability: config.enhanced.scheme_probability,
                revenue_manipulation_probability: config.enhanced.scheme_probability * 0.5,
                kickback_probability: config.enhanced.scheme_probability * 0.5,
                seed: rng.random(),
                ..Default::default()
            };
            Some(SchemeAdvancer::new(scheme_config))
        } else {
            None
        };

        let near_miss_generator = if config.enhanced.near_miss_enabled {
            let near_miss_config = NearMissConfig {
                proportion: config.enhanced.near_miss_proportion,
                seed: rng.random(),
                ..Default::default()
            };
            Some(NearMissGenerator::new(near_miss_config))
        } else {
            None
        };

        let co_occurrence_handler = if config.enhanced.correlated_injection_enabled {
            Some(AnomalyCoOccurrence::new())
        } else {
            None
        };

        let temporal_cluster_generator = if config.enhanced.temporal_clustering_enabled {
            Some(TemporalClusterGenerator::new())
        } else {
            None
        };

        let difficulty_calculator = if config.enhanced.difficulty_classification_enabled {
            Some(DifficultyCalculator::new())
        } else {
            None
        };

        let entity_aware_injector = if config.enhanced.context_aware_enabled {
            Some(EntityAwareInjector::default())
        } else {
            None
        };

        let behavioral_baseline = if config.enhanced.context_aware_enabled
            && config.enhanced.behavioral_baseline_config.enabled
        {
            Some(BehavioralBaseline::new(
                config.enhanced.behavioral_baseline_config.clone(),
            ))
        } else {
            None
        };

        let uuid_factory = DeterministicUuidFactory::new(config.seed, GeneratorType::Anomaly);

        Self {
            config,
            rng,
            uuid_factory,
            type_selector: AnomalyTypeSelector::new(),
            strategies: StrategyCollection::default(),
            cluster_manager,
            entity_targeting,
            document_anomaly_counts: HashMap::new(),
            labels: Vec::new(),
            stats: InjectorStats::default(),
            scheme_advancer,
            near_miss_generator,
            near_miss_labels: Vec::new(),
            co_occurrence_handler,
            temporal_cluster_generator,
            difficulty_calculator,
            entity_aware_injector,
            behavioral_baseline,
            scheme_actions: Vec::new(),
            difficulty_distribution: HashMap::new(),
            vendor_contexts: HashMap::new(),
            employee_contexts: HashMap::new(),
            account_contexts: HashMap::new(),
        }
    }

    /// Processes a batch of journal entries, potentially injecting anomalies.
    pub fn process_entries(&mut self, entries: &mut [JournalEntry]) -> InjectionBatchResult {
        debug!(
            entry_count = entries.len(),
            total_rate = self.config.rates.total_rate,
            seed = self.config.seed,
            "Injecting anomalies into journal entries"
        );

        let mut modified_documents = Vec::new();
        let mut duplicates = Vec::new();

        for entry in entries.iter_mut() {
            self.stats.total_processed += 1;

            // Update behavioral baseline if enabled
            if let Some(ref mut baseline) = self.behavioral_baseline {
                use super::context::Observation;
                // Record the observation for baseline building
                let entity_id = entry.header.created_by.clone();
                let observation =
                    Observation::new(entry.posting_date()).with_amount(entry.total_debit());
                baseline.record_observation(&entity_id, observation);
            }

            // Check if we should process this entry
            if !self.should_process(entry) {
                continue;
            }

            // Calculate effective rate (temporal clustering is applied later per-type)
            let base_rate = self.config.rates.total_rate;

            // Calculate entity-aware rate adjustment using context lookup maps
            let effective_rate = if let Some(ref injector) = self.entity_aware_injector {
                let employee_id = &entry.header.created_by;
                let first_account = entry
                    .lines
                    .first()
                    .map(|l| l.gl_account.as_str())
                    .unwrap_or("");
                // Look up vendor from the entry's reference field (vendor ID convention)
                let vendor_ref = entry.header.reference.as_deref().unwrap_or("");

                let vendor_ctx = self.vendor_contexts.get(vendor_ref);
                let employee_ctx = self.employee_contexts.get(employee_id);
                let account_ctx = self.account_contexts.get(first_account);

                let multiplier =
                    injector.get_rate_multiplier(vendor_ctx, employee_ctx, account_ctx);
                (base_rate * multiplier).min(1.0)
            } else {
                // No entity-aware injector: fall back to context maps alone
                self.calculate_context_rate_multiplier(entry) * base_rate
            };

            // Determine if we inject an anomaly
            if should_inject_anomaly(
                effective_rate,
                entry.posting_date(),
                &self.config.patterns.temporal_pattern,
                &mut self.rng,
            ) {
                // Check if this should be a near-miss instead
                if let Some(ref mut near_miss_gen) = self.near_miss_generator {
                    // Record the transaction for near-duplicate detection
                    let account = entry
                        .lines
                        .first()
                        .map(|l| l.gl_account.clone())
                        .unwrap_or_default();
                    near_miss_gen.record_transaction(
                        entry.document_number().clone(),
                        entry.posting_date(),
                        entry.total_debit(),
                        &account,
                        None,
                    );

                    // Check if this could be a near-miss
                    if let Some(near_miss_label) = near_miss_gen.check_near_miss(
                        entry.document_number().clone(),
                        entry.posting_date(),
                        entry.total_debit(),
                        &account,
                        None,
                        &self.config.enhanced.approval_thresholds,
                    ) {
                        self.near_miss_labels.push(near_miss_label);
                        continue; // Skip actual anomaly injection
                    }
                }

                // Select anomaly category based on rates
                let anomaly_type = self.select_anomaly_category();

                // Apply the anomaly
                if let Some(mut label) = self.inject_anomaly(entry, anomaly_type) {
                    // Calculate detection difficulty if enabled
                    if let Some(ref calculator) = self.difficulty_calculator {
                        let difficulty = calculator.calculate(&label);

                        // Store difficulty in metadata
                        label = label
                            .with_metadata("detection_difficulty", &format!("{:?}", difficulty));
                        label = label.with_metadata(
                            "difficulty_score",
                            &difficulty.difficulty_score().to_string(),
                        );

                        // Update difficulty distribution
                        *self.difficulty_distribution.entry(difficulty).or_insert(0) += 1;
                    }

                    modified_documents.push(entry.document_number().clone());
                    self.labels.push(label);
                    self.stats.total_injected += 1;
                }

                // Check for duplicate injection
                if self.config.allow_duplicates
                    && matches!(
                        self.labels.last().map(|l| &l.anomaly_type),
                        Some(AnomalyType::Error(ErrorType::DuplicateEntry))
                            | Some(AnomalyType::Fraud(FraudType::DuplicatePayment))
                    )
                {
                    let dup_strategy = DuplicationStrategy::default();
                    let duplicate =
                        dup_strategy.duplicate(entry, &mut self.rng, &self.uuid_factory);
                    duplicates.push(duplicate);
                }
            }
        }

        // Count duplicates
        let duplicates_created = duplicates.len();

        // Build summary
        let summary = AnomalySummary::from_anomalies(&self.labels);

        InjectionBatchResult {
            entries_processed: self.stats.total_processed,
            anomalies_injected: self.stats.total_injected,
            duplicates_created,
            labels: self.labels.clone(),
            summary,
            modified_documents,
            near_miss_labels: self.near_miss_labels.clone(),
            scheme_actions: self.scheme_actions.clone(),
            difficulty_distribution: self.difficulty_distribution.clone(),
        }
    }

    /// Checks if an entry should be processed.
    fn should_process(&mut self, entry: &JournalEntry) -> bool {
        // Check company filter
        if !self.config.target_companies.is_empty()
            && !self
                .config
                .target_companies
                .iter()
                .any(|c| c == entry.company_code())
        {
            self.stats.skipped_company += 1;
            return false;
        }

        // Check date range
        if let Some((start, end)) = self.config.date_range {
            if entry.posting_date() < start || entry.posting_date() > end {
                self.stats.skipped_date += 1;
                return false;
            }
        }

        // Check max anomalies per document
        let current_count = self
            .document_anomaly_counts
            .get(&entry.document_number())
            .copied()
            .unwrap_or(0);
        if current_count >= self.config.max_anomalies_per_document {
            self.stats.skipped_max_per_doc += 1;
            return false;
        }

        true
    }

    /// Selects an anomaly category based on configured rates.
    fn select_anomaly_category(&mut self) -> AnomalyType {
        let r = self.rng.random::<f64>();
        let rates = &self.config.rates;

        let mut cumulative = 0.0;

        cumulative += rates.fraud_rate;
        if r < cumulative {
            return self.type_selector.select_fraud(&mut self.rng);
        }

        cumulative += rates.error_rate;
        if r < cumulative {
            return self.type_selector.select_error(&mut self.rng);
        }

        cumulative += rates.process_issue_rate;
        if r < cumulative {
            return self.type_selector.select_process_issue(&mut self.rng);
        }

        cumulative += rates.statistical_rate;
        if r < cumulative {
            return self.type_selector.select_statistical(&mut self.rng);
        }

        self.type_selector.select_relational(&mut self.rng)
    }

    /// Injects an anomaly into an entry.
    fn inject_anomaly(
        &mut self,
        entry: &mut JournalEntry,
        anomaly_type: AnomalyType,
    ) -> Option<LabeledAnomaly> {
        // Check if strategy can be applied
        if !self.strategies.can_apply(entry, &anomaly_type) {
            return None;
        }

        // Apply the strategy
        let result = self
            .strategies
            .apply_strategy(entry, &anomaly_type, &mut self.rng);

        if !result.success {
            return None;
        }

        // Update document anomaly count
        *self
            .document_anomaly_counts
            .entry(entry.document_number().clone())
            .or_insert(0) += 1;

        // Update statistics
        let category = anomaly_type.category().to_string();
        let type_name = anomaly_type.type_name();

        *self.stats.by_category.entry(category).or_insert(0) += 1;
        *self.stats.by_type.entry(type_name.clone()).or_insert(0) += 1;
        *self
            .stats
            .by_company
            .entry(entry.company_code().to_string())
            .or_insert(0) += 1;

        // Generate label
        if self.config.generate_labels {
            let anomaly_id = format!("ANO{:08}", self.labels.len() + 1);

            // Update entry header with anomaly tracking fields
            entry.header.is_anomaly = true;
            entry.header.anomaly_id = Some(anomaly_id.clone());
            entry.header.anomaly_type = Some(type_name.clone());

            // Also set fraud flag if this is a fraud anomaly
            if matches!(anomaly_type, AnomalyType::Fraud(_)) {
                entry.header.is_fraud = true;
                if let AnomalyType::Fraud(ref ft) = anomaly_type {
                    entry.header.fraud_type = Some(*ft);
                }
            }

            let mut label = LabeledAnomaly::new(
                anomaly_id,
                anomaly_type.clone(),
                entry.document_number().clone(),
                "JE".to_string(),
                entry.company_code().to_string(),
                entry.posting_date(),
            )
            .with_description(&result.description)
            .with_injection_strategy(&type_name);

            // Add causal reason with injection context (provenance tracking)
            let causal_reason = AnomalyCausalReason::RandomRate {
                base_rate: self.config.rates.total_rate,
            };
            label = label.with_causal_reason(causal_reason);

            // Add entity context metadata if contexts are populated
            let context_multiplier = self.calculate_context_rate_multiplier(entry);
            if (context_multiplier - 1.0).abs() > f64::EPSILON {
                label = label.with_metadata(
                    "entity_context_multiplier",
                    &format!("{:.3}", context_multiplier),
                );
                label = label.with_metadata(
                    "effective_rate",
                    &format!(
                        "{:.6}",
                        (self.config.rates.total_rate * context_multiplier).min(1.0)
                    ),
                );
            }

            // Add monetary impact
            if let Some(impact) = result.monetary_impact {
                label = label.with_monetary_impact(impact);
            }

            // Add related entities
            for entity in &result.related_entities {
                label = label.with_related_entity(entity);
            }

            // Add metadata
            for (key, value) in &result.metadata {
                label = label.with_metadata(key, value);
            }

            // Assign cluster and update causal reason if in cluster
            if let Some(cluster_id) =
                self.cluster_manager
                    .assign_cluster(entry.posting_date(), &type_name, &mut self.rng)
            {
                label = label.with_cluster(&cluster_id);
                // Update causal reason to reflect cluster membership
                label = label.with_causal_reason(AnomalyCausalReason::ClusterMembership {
                    cluster_id: cluster_id.clone(),
                });
            }

            return Some(label);
        }

        None
    }

    /// Injects a specific anomaly type into an entry.
    pub fn inject_specific(
        &mut self,
        entry: &mut JournalEntry,
        anomaly_type: AnomalyType,
    ) -> Option<LabeledAnomaly> {
        self.inject_anomaly(entry, anomaly_type)
    }

    /// Creates a self-approval anomaly.
    pub fn create_self_approval(
        &mut self,
        entry: &mut JournalEntry,
        user_id: &str,
    ) -> Option<LabeledAnomaly> {
        let anomaly_type = AnomalyType::Fraud(FraudType::SelfApproval);

        let label = LabeledAnomaly::new(
            format!("ANO{:08}", self.labels.len() + 1),
            anomaly_type,
            entry.document_number().clone(),
            "JE".to_string(),
            entry.company_code().to_string(),
            entry.posting_date(),
        )
        .with_description(&format!("User {} approved their own transaction", user_id))
        .with_related_entity(user_id)
        .with_injection_strategy("ManualSelfApproval")
        .with_causal_reason(AnomalyCausalReason::EntityTargeting {
            target_type: "User".to_string(),
            target_id: user_id.to_string(),
        });

        // Set entry header anomaly tracking fields
        entry.header.is_anomaly = true;
        entry.header.is_fraud = true;
        entry.header.anomaly_id = Some(label.anomaly_id.clone());
        entry.header.anomaly_type = Some("SelfApproval".to_string());
        entry.header.fraud_type = Some(FraudType::SelfApproval);

        // Set approver = requester
        entry.header.created_by = user_id.to_string();

        self.labels.push(label.clone());
        Some(label)
    }

    /// Creates a segregation of duties violation.
    pub fn create_sod_violation(
        &mut self,
        entry: &mut JournalEntry,
        user_id: &str,
        conflicting_duties: (&str, &str),
    ) -> Option<LabeledAnomaly> {
        let anomaly_type = AnomalyType::Fraud(FraudType::SegregationOfDutiesViolation);

        let label = LabeledAnomaly::new(
            format!("ANO{:08}", self.labels.len() + 1),
            anomaly_type,
            entry.document_number().clone(),
            "JE".to_string(),
            entry.company_code().to_string(),
            entry.posting_date(),
        )
        .with_description(&format!(
            "User {} performed conflicting duties: {} and {}",
            user_id, conflicting_duties.0, conflicting_duties.1
        ))
        .with_related_entity(user_id)
        .with_metadata("duty1", conflicting_duties.0)
        .with_metadata("duty2", conflicting_duties.1)
        .with_injection_strategy("ManualSoDViolation")
        .with_causal_reason(AnomalyCausalReason::EntityTargeting {
            target_type: "User".to_string(),
            target_id: user_id.to_string(),
        });

        // Set entry header anomaly tracking fields
        entry.header.is_anomaly = true;
        entry.header.is_fraud = true;
        entry.header.anomaly_id = Some(label.anomaly_id.clone());
        entry.header.anomaly_type = Some("SegregationOfDutiesViolation".to_string());
        entry.header.fraud_type = Some(FraudType::SegregationOfDutiesViolation);

        self.labels.push(label.clone());
        Some(label)
    }

    /// Creates an intercompany mismatch anomaly.
    pub fn create_ic_mismatch(
        &mut self,
        entry: &mut JournalEntry,
        matching_company: &str,
        expected_amount: Decimal,
        actual_amount: Decimal,
    ) -> Option<LabeledAnomaly> {
        let anomaly_type = AnomalyType::Relational(RelationalAnomalyType::UnmatchedIntercompany);

        let label = LabeledAnomaly::new(
            format!("ANO{:08}", self.labels.len() + 1),
            anomaly_type,
            entry.document_number().clone(),
            "JE".to_string(),
            entry.company_code().to_string(),
            entry.posting_date(),
        )
        .with_description(&format!(
            "Intercompany mismatch with {}: expected {} but got {}",
            matching_company, expected_amount, actual_amount
        ))
        .with_related_entity(matching_company)
        .with_monetary_impact(actual_amount - expected_amount)
        .with_metadata("expected_amount", &expected_amount.to_string())
        .with_metadata("actual_amount", &actual_amount.to_string())
        .with_injection_strategy("ManualICMismatch")
        .with_causal_reason(AnomalyCausalReason::EntityTargeting {
            target_type: "Intercompany".to_string(),
            target_id: matching_company.to_string(),
        });

        // Set entry header anomaly tracking fields
        entry.header.is_anomaly = true;
        entry.header.anomaly_id = Some(label.anomaly_id.clone());
        entry.header.anomaly_type = Some("UnmatchedIntercompany".to_string());

        self.labels.push(label.clone());
        Some(label)
    }

    /// Returns all generated labels.
    pub fn get_labels(&self) -> &[LabeledAnomaly] {
        &self.labels
    }

    /// Returns the anomaly summary.
    pub fn get_summary(&self) -> AnomalySummary {
        AnomalySummary::from_anomalies(&self.labels)
    }

    /// Returns injection statistics.
    pub fn get_stats(&self) -> &InjectorStats {
        &self.stats
    }

    /// Clears all labels and resets statistics.
    pub fn reset(&mut self) {
        self.labels.clear();
        self.document_anomaly_counts.clear();
        self.stats = InjectorStats::default();
        self.cluster_manager = ClusterManager::new(self.config.patterns.clustering.clone());

        // Reset enhanced components
        self.near_miss_labels.clear();
        self.scheme_actions.clear();
        self.difficulty_distribution.clear();

        if let Some(ref mut baseline) = self.behavioral_baseline {
            *baseline =
                BehavioralBaseline::new(self.config.enhanced.behavioral_baseline_config.clone());
        }
    }

    /// Returns the number of clusters created.
    pub fn cluster_count(&self) -> usize {
        self.cluster_manager.cluster_count()
    }

    // =========================================================================
    // Entity Context API
    // =========================================================================

    /// Sets entity contexts for risk-adjusted anomaly injection.
    ///
    /// When entity contexts are provided, the injector adjusts anomaly injection
    /// rates based on entity risk factors. Entries involving high-risk vendors,
    /// new employees, or sensitive accounts will have higher effective injection
    /// rates.
    ///
    /// Pass empty HashMaps to clear previously set contexts.
    pub fn set_entity_contexts(
        &mut self,
        vendors: HashMap<String, VendorContext>,
        employees: HashMap<String, EmployeeContext>,
        accounts: HashMap<String, AccountContext>,
    ) {
        self.vendor_contexts = vendors;
        self.employee_contexts = employees;
        self.account_contexts = accounts;
    }

    /// Returns a reference to the vendor context map.
    pub fn vendor_contexts(&self) -> &HashMap<String, VendorContext> {
        &self.vendor_contexts
    }

    /// Returns a reference to the employee context map.
    pub fn employee_contexts(&self) -> &HashMap<String, EmployeeContext> {
        &self.employee_contexts
    }

    /// Returns a reference to the account context map.
    pub fn account_contexts(&self) -> &HashMap<String, AccountContext> {
        &self.account_contexts
    }

    /// Calculates a rate multiplier from the entity context maps alone (no
    /// `EntityAwareInjector` needed). This provides a lightweight fallback
    /// when context-aware injection is not fully enabled but context maps
    /// have been populated.
    ///
    /// The multiplier is the product of individual entity risk factors found
    /// in the context maps for the given journal entry. If no contexts match,
    /// returns 1.0 (no adjustment).
    fn calculate_context_rate_multiplier(&self, entry: &JournalEntry) -> f64 {
        if self.vendor_contexts.is_empty()
            && self.employee_contexts.is_empty()
            && self.account_contexts.is_empty()
        {
            return 1.0;
        }

        let mut multiplier = 1.0;

        // Vendor lookup via reference field
        if let Some(ref vendor_ref) = entry.header.reference {
            if let Some(ctx) = self.vendor_contexts.get(vendor_ref) {
                // New vendors get a 2.0x multiplier, dormant reactivations get 1.5x
                if ctx.is_new {
                    multiplier *= 2.0;
                }
                if ctx.is_dormant_reactivation {
                    multiplier *= 1.5;
                }
            }
        }

        // Employee lookup via created_by
        if let Some(ctx) = self.employee_contexts.get(&entry.header.created_by) {
            if ctx.is_new {
                multiplier *= 1.5;
            }
            if ctx.is_volume_fatigued {
                multiplier *= 1.3;
            }
            if ctx.is_overtime {
                multiplier *= 1.2;
            }
        }

        // Account lookup via first line's GL account
        if let Some(first_line) = entry.lines.first() {
            if let Some(ctx) = self.account_contexts.get(&first_line.gl_account) {
                if ctx.is_high_risk {
                    multiplier *= 2.0;
                }
            }
        }

        multiplier
    }

    // =========================================================================
    // Enhanced Features API (v0.3.0+)
    // =========================================================================

    /// Advances all active fraud schemes by one time step.
    ///
    /// Call this method once per simulated day to generate scheme actions.
    /// Returns the scheme actions generated for this date.
    pub fn advance_schemes(&mut self, date: NaiveDate, company_code: &str) -> Vec<SchemeAction> {
        if let Some(ref mut advancer) = self.scheme_advancer {
            let context = SchemeContext::new(date, company_code);
            let actions = advancer.advance_all(&context);
            self.scheme_actions.extend(actions.clone());
            actions
        } else {
            Vec::new()
        }
    }

    /// Potentially starts a new fraud scheme based on probabilities.
    ///
    /// Call this method periodically (e.g., once per period) to allow new
    /// schemes to start based on configured probabilities.
    /// Returns the scheme ID if a scheme was started.
    pub fn maybe_start_scheme(
        &mut self,
        date: NaiveDate,
        company_code: &str,
        available_users: Vec<String>,
        available_accounts: Vec<String>,
        available_counterparties: Vec<String>,
    ) -> Option<uuid::Uuid> {
        if let Some(ref mut advancer) = self.scheme_advancer {
            let mut context = SchemeContext::new(date, company_code);
            context.available_users = available_users;
            context.available_accounts = available_accounts;
            context.available_counterparties = available_counterparties;

            advancer.maybe_start_scheme(&context)
        } else {
            None
        }
    }

    /// Returns all near-miss labels generated.
    pub fn get_near_miss_labels(&self) -> &[NearMissLabel] {
        &self.near_miss_labels
    }

    /// Returns all scheme actions generated.
    pub fn get_scheme_actions(&self) -> &[SchemeAction] {
        &self.scheme_actions
    }

    /// Returns the detection difficulty distribution.
    pub fn get_difficulty_distribution(&self) -> &HashMap<AnomalyDetectionDifficulty, usize> {
        &self.difficulty_distribution
    }

    /// Checks for behavioral deviations for an entity with an observation.
    pub fn check_behavioral_deviations(
        &self,
        entity_id: &str,
        observation: &super::context::Observation,
    ) -> Vec<super::context::BehavioralDeviation> {
        if let Some(ref baseline) = self.behavioral_baseline {
            baseline.check_deviation(entity_id, observation)
        } else {
            Vec::new()
        }
    }

    /// Gets the baseline for an entity.
    pub fn get_entity_baseline(&self, entity_id: &str) -> Option<&super::context::EntityBaseline> {
        if let Some(ref baseline) = self.behavioral_baseline {
            baseline.get_baseline(entity_id)
        } else {
            None
        }
    }

    /// Returns the number of active schemes.
    pub fn active_scheme_count(&self) -> usize {
        if let Some(ref advancer) = self.scheme_advancer {
            advancer.active_scheme_count()
        } else {
            0
        }
    }

    /// Returns whether enhanced features are enabled.
    pub fn has_enhanced_features(&self) -> bool {
        self.scheme_advancer.is_some()
            || self.near_miss_generator.is_some()
            || self.difficulty_calculator.is_some()
            || self.entity_aware_injector.is_some()
    }
}

/// Builder for AnomalyInjectorConfig.
pub struct AnomalyInjectorConfigBuilder {
    config: AnomalyInjectorConfig,
}

impl AnomalyInjectorConfigBuilder {
    /// Creates a new builder with default configuration.
    pub fn new() -> Self {
        Self {
            config: AnomalyInjectorConfig::default(),
        }
    }

    /// Sets the total anomaly rate.
    pub fn with_total_rate(mut self, rate: f64) -> Self {
        self.config.rates.total_rate = rate;
        self
    }

    /// Sets the fraud rate (proportion of anomalies).
    pub fn with_fraud_rate(mut self, rate: f64) -> Self {
        self.config.rates.fraud_rate = rate;
        self
    }

    /// Sets the error rate (proportion of anomalies).
    pub fn with_error_rate(mut self, rate: f64) -> Self {
        self.config.rates.error_rate = rate;
        self
    }

    /// Sets the random seed.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.config.seed = seed;
        self
    }

    /// Sets the temporal pattern.
    pub fn with_temporal_pattern(mut self, pattern: TemporalPattern) -> Self {
        self.config.patterns.temporal_pattern = pattern;
        self
    }

    /// Enables or disables label generation.
    pub fn with_labels(mut self, generate: bool) -> Self {
        self.config.generate_labels = generate;
        self
    }

    /// Sets target companies.
    pub fn with_target_companies(mut self, companies: Vec<String>) -> Self {
        self.config.target_companies = companies;
        self
    }

    /// Sets the date range.
    pub fn with_date_range(mut self, start: NaiveDate, end: NaiveDate) -> Self {
        self.config.date_range = Some((start, end));
        self
    }

    // =========================================================================
    // Enhanced Features Configuration (v0.3.0+)
    // =========================================================================

    /// Enables multi-stage fraud scheme generation.
    pub fn with_multi_stage_schemes(mut self, enabled: bool, probability: f64) -> Self {
        self.config.enhanced.multi_stage_schemes_enabled = enabled;
        self.config.enhanced.scheme_probability = probability;
        self
    }

    /// Enables near-miss generation.
    pub fn with_near_misses(mut self, enabled: bool, proportion: f64) -> Self {
        self.config.enhanced.near_miss_enabled = enabled;
        self.config.enhanced.near_miss_proportion = proportion;
        self
    }

    /// Sets approval thresholds for threshold-proximity near-misses.
    pub fn with_approval_thresholds(mut self, thresholds: Vec<Decimal>) -> Self {
        self.config.enhanced.approval_thresholds = thresholds;
        self
    }

    /// Enables correlated anomaly injection.
    pub fn with_correlated_injection(mut self, enabled: bool) -> Self {
        self.config.enhanced.correlated_injection_enabled = enabled;
        self
    }

    /// Enables temporal clustering (period-end spikes).
    pub fn with_temporal_clustering(mut self, enabled: bool, multiplier: f64) -> Self {
        self.config.enhanced.temporal_clustering_enabled = enabled;
        self.config.enhanced.period_end_multiplier = multiplier;
        self
    }

    /// Enables detection difficulty classification.
    pub fn with_difficulty_classification(mut self, enabled: bool) -> Self {
        self.config.enhanced.difficulty_classification_enabled = enabled;
        self
    }

    /// Enables context-aware injection.
    pub fn with_context_aware_injection(mut self, enabled: bool) -> Self {
        self.config.enhanced.context_aware_enabled = enabled;
        self
    }

    /// Sets behavioral baseline configuration.
    pub fn with_behavioral_baseline(mut self, config: BehavioralBaselineConfig) -> Self {
        self.config.enhanced.behavioral_baseline_config = config;
        self
    }

    /// Enables all enhanced features with default settings.
    pub fn with_all_enhanced_features(mut self) -> Self {
        self.config.enhanced.multi_stage_schemes_enabled = true;
        self.config.enhanced.scheme_probability = 0.02;
        self.config.enhanced.correlated_injection_enabled = true;
        self.config.enhanced.temporal_clustering_enabled = true;
        self.config.enhanced.period_end_multiplier = 2.5;
        self.config.enhanced.near_miss_enabled = true;
        self.config.enhanced.near_miss_proportion = 0.30;
        self.config.enhanced.difficulty_classification_enabled = true;
        self.config.enhanced.context_aware_enabled = true;
        self.config.enhanced.behavioral_baseline_config.enabled = true;
        self
    }

    /// Builds the configuration.
    pub fn build(self) -> AnomalyInjectorConfig {
        self.config
    }
}

impl Default for AnomalyInjectorConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use datasynth_core::models::{JournalEntryLine, StatisticalAnomalyType};
    use rust_decimal_macros::dec;

    fn create_test_entry(doc_num: &str) -> JournalEntry {
        let mut entry = JournalEntry::new_simple(
            doc_num.to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            "Test Entry".to_string(),
        );

        entry.add_line(JournalEntryLine {
            line_number: 1,
            gl_account: "5000".to_string(),
            debit_amount: dec!(1000),
            ..Default::default()
        });

        entry.add_line(JournalEntryLine {
            line_number: 2,
            gl_account: "1000".to_string(),
            credit_amount: dec!(1000),
            ..Default::default()
        });

        entry
    }

    #[test]
    fn test_anomaly_injector_basic() {
        let config = AnomalyInjectorConfigBuilder::new()
            .with_total_rate(0.5) // High rate for testing
            .with_seed(42)
            .build();

        let mut injector = AnomalyInjector::new(config);

        let mut entries: Vec<_> = (0..100)
            .map(|i| create_test_entry(&format!("JE{:04}", i)))
            .collect();

        let result = injector.process_entries(&mut entries);

        // With 50% rate, we should have some anomalies
        assert!(result.anomalies_injected > 0);
        assert!(!result.labels.is_empty());
        assert_eq!(result.labels.len(), result.anomalies_injected);
    }

    #[test]
    fn test_specific_injection() {
        let config = AnomalyInjectorConfig::default();
        let mut injector = AnomalyInjector::new(config);

        let mut entry = create_test_entry("JE001");
        let anomaly_type = AnomalyType::Statistical(StatisticalAnomalyType::UnusuallyHighAmount);

        let label = injector.inject_specific(&mut entry, anomaly_type);

        assert!(label.is_some());
        let label = label.unwrap();
        // document_id is the UUID string from the journal entry header
        assert!(!label.document_id.is_empty());
        assert_eq!(label.document_id, entry.document_number());
    }

    #[test]
    fn test_self_approval_injection() {
        let config = AnomalyInjectorConfig::default();
        let mut injector = AnomalyInjector::new(config);

        let mut entry = create_test_entry("JE001");
        let label = injector.create_self_approval(&mut entry, "USER001");

        assert!(label.is_some());
        let label = label.unwrap();
        assert!(matches!(
            label.anomaly_type,
            AnomalyType::Fraud(FraudType::SelfApproval)
        ));
        assert!(label.related_entities.contains(&"USER001".to_string()));
    }

    #[test]
    fn test_company_filtering() {
        let config = AnomalyInjectorConfigBuilder::new()
            .with_total_rate(1.0) // Inject all
            .with_target_companies(vec!["2000".to_string()])
            .build();

        let mut injector = AnomalyInjector::new(config);

        let mut entries = vec![
            create_test_entry("JE001"), // company 1000
            create_test_entry("JE002"), // company 1000
        ];

        let result = injector.process_entries(&mut entries);

        // No anomalies because entries are in company 1000, not 2000
        assert_eq!(result.anomalies_injected, 0);
    }

    // =========================================================================
    // Entity Context Tests
    // =========================================================================

    /// Helper to create a test entry with specific vendor reference and employee.
    fn create_test_entry_with_context(
        doc_num: &str,
        vendor_ref: Option<&str>,
        employee_id: &str,
        gl_account: &str,
    ) -> JournalEntry {
        let mut entry = JournalEntry::new_simple(
            doc_num.to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            "Test Entry".to_string(),
        );

        entry.header.reference = vendor_ref.map(|v| v.to_string());
        entry.header.created_by = employee_id.to_string();

        entry.add_line(JournalEntryLine {
            line_number: 1,
            gl_account: gl_account.to_string(),
            debit_amount: dec!(1000),
            ..Default::default()
        });

        entry.add_line(JournalEntryLine {
            line_number: 2,
            gl_account: "1000".to_string(),
            credit_amount: dec!(1000),
            ..Default::default()
        });

        entry
    }

    #[test]
    fn test_set_entity_contexts() {
        let config = AnomalyInjectorConfig::default();
        let mut injector = AnomalyInjector::new(config);

        // Initially empty
        assert!(injector.vendor_contexts().is_empty());
        assert!(injector.employee_contexts().is_empty());
        assert!(injector.account_contexts().is_empty());

        // Set contexts
        let mut vendors = HashMap::new();
        vendors.insert(
            "V001".to_string(),
            VendorContext {
                vendor_id: "V001".to_string(),
                is_new: true,
                ..Default::default()
            },
        );

        let mut employees = HashMap::new();
        employees.insert(
            "EMP001".to_string(),
            EmployeeContext {
                employee_id: "EMP001".to_string(),
                is_new: true,
                ..Default::default()
            },
        );

        let mut accounts = HashMap::new();
        accounts.insert(
            "8100".to_string(),
            AccountContext {
                account_code: "8100".to_string(),
                is_high_risk: true,
                ..Default::default()
            },
        );

        injector.set_entity_contexts(vendors, employees, accounts);

        assert_eq!(injector.vendor_contexts().len(), 1);
        assert_eq!(injector.employee_contexts().len(), 1);
        assert_eq!(injector.account_contexts().len(), 1);
        assert!(injector.vendor_contexts().contains_key("V001"));
        assert!(injector.employee_contexts().contains_key("EMP001"));
        assert!(injector.account_contexts().contains_key("8100"));
    }

    #[test]
    fn test_default_behavior_no_contexts() {
        // Without any entity contexts, the base rate is used unchanged.
        let config = AnomalyInjectorConfigBuilder::new()
            .with_total_rate(0.5)
            .with_seed(42)
            .build();

        let mut injector = AnomalyInjector::new(config);

        let mut entries: Vec<_> = (0..200)
            .map(|i| create_test_entry(&format!("JE{:04}", i)))
            .collect();

        let result = injector.process_entries(&mut entries);

        // With 50% base rate and no context, expect roughly 50% injection
        // Allow wide margin for randomness
        assert!(result.anomalies_injected > 0);
        let rate = result.anomalies_injected as f64 / result.entries_processed as f64;
        assert!(
            rate > 0.2 && rate < 0.8,
            "Expected ~50% rate, got {:.2}%",
            rate * 100.0
        );
    }

    #[test]
    fn test_entity_context_increases_injection_rate() {
        // With high-risk entity contexts, the effective rate should be higher
        // than the base rate, leading to more anomalies being injected.
        let base_rate = 0.10; // Low base rate

        // Run without contexts
        let config_no_ctx = AnomalyInjectorConfigBuilder::new()
            .with_total_rate(base_rate)
            .with_seed(123)
            .build();

        let mut injector_no_ctx = AnomalyInjector::new(config_no_ctx);

        let mut entries_no_ctx: Vec<_> = (0..500)
            .map(|i| {
                create_test_entry_with_context(
                    &format!("JE{:04}", i),
                    Some("V001"),
                    "EMP001",
                    "8100",
                )
            })
            .collect();

        let result_no_ctx = injector_no_ctx.process_entries(&mut entries_no_ctx);

        // Run with high-risk contexts (same seed for comparable randomness)
        let config_ctx = AnomalyInjectorConfigBuilder::new()
            .with_total_rate(base_rate)
            .with_seed(123)
            .build();

        let mut injector_ctx = AnomalyInjector::new(config_ctx);

        // Set up high-risk contexts
        let mut vendors = HashMap::new();
        vendors.insert(
            "V001".to_string(),
            VendorContext {
                vendor_id: "V001".to_string(),
                is_new: true,                  // 2.0x multiplier
                is_dormant_reactivation: true, // 1.5x multiplier
                ..Default::default()
            },
        );

        let mut employees = HashMap::new();
        employees.insert(
            "EMP001".to_string(),
            EmployeeContext {
                employee_id: "EMP001".to_string(),
                is_new: true, // 1.5x multiplier
                ..Default::default()
            },
        );

        let mut accounts = HashMap::new();
        accounts.insert(
            "8100".to_string(),
            AccountContext {
                account_code: "8100".to_string(),
                is_high_risk: true, // 2.0x multiplier
                ..Default::default()
            },
        );

        injector_ctx.set_entity_contexts(vendors, employees, accounts);

        let mut entries_ctx: Vec<_> = (0..500)
            .map(|i| {
                create_test_entry_with_context(
                    &format!("JE{:04}", i),
                    Some("V001"),
                    "EMP001",
                    "8100",
                )
            })
            .collect();

        let result_ctx = injector_ctx.process_entries(&mut entries_ctx);

        // The context-enhanced run should inject more anomalies
        assert!(
            result_ctx.anomalies_injected > result_no_ctx.anomalies_injected,
            "Expected more anomalies with high-risk contexts: {} (with ctx) vs {} (without ctx)",
            result_ctx.anomalies_injected,
            result_no_ctx.anomalies_injected,
        );
    }

    #[test]
    fn test_risk_score_multiplication() {
        // Verify the calculate_context_rate_multiplier produces correct values.
        let config = AnomalyInjectorConfig::default();
        let mut injector = AnomalyInjector::new(config);

        // No contexts: multiplier should be 1.0
        let entry_plain = create_test_entry_with_context("JE001", None, "USER1", "5000");
        assert!(
            (injector.calculate_context_rate_multiplier(&entry_plain) - 1.0).abs() < f64::EPSILON,
        );

        // Set up a new vendor (2.0x) + high-risk account (2.0x) = 4.0x
        let mut vendors = HashMap::new();
        vendors.insert(
            "V_RISKY".to_string(),
            VendorContext {
                vendor_id: "V_RISKY".to_string(),
                is_new: true,
                ..Default::default()
            },
        );

        let mut accounts = HashMap::new();
        accounts.insert(
            "9000".to_string(),
            AccountContext {
                account_code: "9000".to_string(),
                is_high_risk: true,
                ..Default::default()
            },
        );

        injector.set_entity_contexts(vendors, HashMap::new(), accounts);

        let entry_risky = create_test_entry_with_context("JE002", Some("V_RISKY"), "USER1", "9000");
        let multiplier = injector.calculate_context_rate_multiplier(&entry_risky);
        // new vendor = 2.0x, high-risk account = 2.0x => 4.0x
        assert!(
            (multiplier - 4.0).abs() < f64::EPSILON,
            "Expected 4.0x multiplier, got {}",
            multiplier,
        );

        // Entry with only vendor context match (no account match)
        let entry_vendor_only =
            create_test_entry_with_context("JE003", Some("V_RISKY"), "USER1", "5000");
        let multiplier_vendor = injector.calculate_context_rate_multiplier(&entry_vendor_only);
        assert!(
            (multiplier_vendor - 2.0).abs() < f64::EPSILON,
            "Expected 2.0x multiplier (vendor only), got {}",
            multiplier_vendor,
        );

        // Entry with no matching contexts
        let entry_no_match =
            create_test_entry_with_context("JE004", Some("V_SAFE"), "USER1", "5000");
        let multiplier_none = injector.calculate_context_rate_multiplier(&entry_no_match);
        assert!(
            (multiplier_none - 1.0).abs() < f64::EPSILON,
            "Expected 1.0x multiplier (no match), got {}",
            multiplier_none,
        );
    }

    #[test]
    fn test_employee_context_multiplier() {
        let config = AnomalyInjectorConfig::default();
        let mut injector = AnomalyInjector::new(config);

        let mut employees = HashMap::new();
        employees.insert(
            "EMP_NEW".to_string(),
            EmployeeContext {
                employee_id: "EMP_NEW".to_string(),
                is_new: true,             // 1.5x
                is_volume_fatigued: true, // 1.3x
                is_overtime: true,        // 1.2x
                ..Default::default()
            },
        );

        injector.set_entity_contexts(HashMap::new(), employees, HashMap::new());

        let entry = create_test_entry_with_context("JE001", None, "EMP_NEW", "5000");
        let multiplier = injector.calculate_context_rate_multiplier(&entry);

        // 1.5 * 1.3 * 1.2 = 2.34
        let expected = 1.5 * 1.3 * 1.2;
        assert!(
            (multiplier - expected).abs() < 0.01,
            "Expected {:.3}x multiplier, got {:.3}",
            expected,
            multiplier,
        );
    }

    #[test]
    fn test_entity_contexts_persist_across_reset() {
        let config = AnomalyInjectorConfig::default();
        let mut injector = AnomalyInjector::new(config);

        let mut vendors = HashMap::new();
        vendors.insert(
            "V001".to_string(),
            VendorContext {
                vendor_id: "V001".to_string(),
                is_new: true,
                ..Default::default()
            },
        );

        injector.set_entity_contexts(vendors, HashMap::new(), HashMap::new());
        assert_eq!(injector.vendor_contexts().len(), 1);

        // Reset clears labels and stats but not entity contexts
        injector.reset();
        assert_eq!(injector.vendor_contexts().len(), 1);
    }

    #[test]
    fn test_set_empty_contexts_clears() {
        let config = AnomalyInjectorConfig::default();
        let mut injector = AnomalyInjector::new(config);

        let mut vendors = HashMap::new();
        vendors.insert(
            "V001".to_string(),
            VendorContext {
                vendor_id: "V001".to_string(),
                ..Default::default()
            },
        );

        injector.set_entity_contexts(vendors, HashMap::new(), HashMap::new());
        assert_eq!(injector.vendor_contexts().len(), 1);

        // Setting empty maps clears
        injector.set_entity_contexts(HashMap::new(), HashMap::new(), HashMap::new());
        assert!(injector.vendor_contexts().is_empty());
    }

    #[test]
    fn test_dormant_vendor_multiplier() {
        let config = AnomalyInjectorConfig::default();
        let mut injector = AnomalyInjector::new(config);

        let mut vendors = HashMap::new();
        vendors.insert(
            "V_DORMANT".to_string(),
            VendorContext {
                vendor_id: "V_DORMANT".to_string(),
                is_dormant_reactivation: true, // 1.5x
                ..Default::default()
            },
        );

        injector.set_entity_contexts(vendors, HashMap::new(), HashMap::new());

        let entry = create_test_entry_with_context("JE001", Some("V_DORMANT"), "USER1", "5000");
        let multiplier = injector.calculate_context_rate_multiplier(&entry);
        assert!(
            (multiplier - 1.5).abs() < f64::EPSILON,
            "Expected 1.5x multiplier for dormant vendor, got {}",
            multiplier,
        );
    }
}
