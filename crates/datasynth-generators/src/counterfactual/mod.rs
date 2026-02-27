//! Counterfactual generation for what-if scenarios and paired examples.
//!
//! This module provides:
//! - Paired normal/anomaly example generation for ML training
//! - Controllable anomaly injection with specific parameters
//! - What-if scenario generation for testing and analysis
//!
//! Counterfactual generation is essential for:
//! - Training robust anomaly detection models
//! - Understanding the impact of specific changes
//! - Testing detection system sensitivity
//! - Generating balanced ML datasets

use chrono::{NaiveDateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};

use datasynth_core::models::{
    AnomalyCausalReason, AnomalyType, ErrorType, FraudType, InjectionStrategy, JournalEntry,
    JournalEntryLine, LabeledAnomaly, RelationalAnomalyType, StatisticalAnomalyType,
};

/// A counterfactual pair containing both the original and modified versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterfactualPair {
    /// Unique identifier for this pair.
    pub pair_id: String,

    /// The original (normal) journal entry.
    pub original: JournalEntry,

    /// The modified (anomalous) journal entry.
    pub modified: JournalEntry,

    /// The anomaly label for the modified entry.
    pub anomaly_label: LabeledAnomaly,

    /// Description of what changed.
    pub change_description: String,

    /// The injection strategy applied.
    pub injection_strategy: InjectionStrategy,

    /// Timestamp when the pair was generated.
    pub generated_at: NaiveDateTime,

    /// Additional metadata.
    pub metadata: HashMap<String, String>,
}

impl CounterfactualPair {
    /// Create a new counterfactual pair.
    pub fn new(
        original: JournalEntry,
        modified: JournalEntry,
        anomaly_label: LabeledAnomaly,
        injection_strategy: InjectionStrategy,
        uuid_factory: &DeterministicUuidFactory,
    ) -> Self {
        let pair_id = uuid_factory.next().to_string();
        let change_description = injection_strategy.description();

        Self {
            pair_id,
            original,
            modified,
            anomaly_label,
            change_description,
            injection_strategy,
            generated_at: Utc::now().naive_utc(),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the pair.
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Specification for a counterfactual modification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CounterfactualSpec {
    /// Multiply amount by a factor.
    ScaleAmount {
        /// Multiplication factor.
        factor: f64,
    },

    /// Add a fixed amount.
    AddAmount {
        /// Amount to add (can be negative).
        delta: Decimal,
    },

    /// Set amount to a specific value.
    SetAmount {
        /// Target amount.
        target: Decimal,
    },

    /// Shift the posting date.
    ShiftDate {
        /// Days to shift (negative = earlier).
        days: i32,
    },

    /// Change the fiscal period.
    ChangePeriod {
        /// Target fiscal period.
        target_period: u8,
    },

    /// Change the account classification.
    ReclassifyAccount {
        /// New account number.
        new_account: String,
    },

    /// Add a line item.
    AddLineItem {
        /// Account for the new line.
        account: String,
        /// Amount for the new line.
        amount: Decimal,
        /// Is debit (true) or credit (false).
        is_debit: bool,
    },

    /// Remove a line item by index.
    RemoveLineItem {
        /// Index of line to remove.
        line_index: usize,
    },

    /// Split into multiple transactions.
    SplitTransaction {
        /// Number of splits.
        split_count: u32,
    },

    /// Create a round-tripping pattern.
    CreateRoundTrip {
        /// Intermediate entities.
        intermediaries: Vec<String>,
    },

    /// Mark as self-approved.
    SelfApprove,

    /// Inject a specific fraud type.
    InjectFraud {
        /// The fraud type to inject.
        fraud_type: FraudType,
    },

    /// Apply a custom transformation.
    Custom {
        /// Transformation name.
        name: String,
        /// Parameters.
        params: HashMap<String, String>,
    },
}

impl CounterfactualSpec {
    /// Get the anomaly type this spec would produce.
    pub fn to_anomaly_type(&self) -> AnomalyType {
        match self {
            CounterfactualSpec::ScaleAmount { factor } if *factor > 2.0 => {
                AnomalyType::Fraud(FraudType::RevenueManipulation)
            }
            CounterfactualSpec::ScaleAmount { .. } => {
                AnomalyType::Statistical(StatisticalAnomalyType::UnusuallyHighAmount)
            }
            CounterfactualSpec::AddAmount { .. } => {
                AnomalyType::Statistical(StatisticalAnomalyType::UnusuallyHighAmount)
            }
            CounterfactualSpec::SetAmount { .. } => {
                AnomalyType::Statistical(StatisticalAnomalyType::UnusuallyHighAmount)
            }
            CounterfactualSpec::ShiftDate { .. } => AnomalyType::Fraud(FraudType::TimingAnomaly),
            CounterfactualSpec::ChangePeriod { .. } => AnomalyType::Fraud(FraudType::TimingAnomaly),
            CounterfactualSpec::ReclassifyAccount { .. } => {
                AnomalyType::Error(ErrorType::MisclassifiedAccount)
            }
            CounterfactualSpec::AddLineItem { .. } => {
                AnomalyType::Fraud(FraudType::FictitiousEntry)
            }
            CounterfactualSpec::RemoveLineItem { .. } => {
                AnomalyType::Error(ErrorType::MissingField)
            }
            CounterfactualSpec::SplitTransaction { .. } => {
                AnomalyType::Fraud(FraudType::SplitTransaction)
            }
            CounterfactualSpec::CreateRoundTrip { .. } => {
                AnomalyType::Relational(RelationalAnomalyType::CircularTransaction)
            }
            CounterfactualSpec::SelfApprove => AnomalyType::Fraud(FraudType::SelfApproval),
            CounterfactualSpec::InjectFraud { fraud_type } => AnomalyType::Fraud(*fraud_type),
            CounterfactualSpec::Custom { .. } => AnomalyType::Custom("custom".to_string()),
        }
    }

    /// Get a description of this specification.
    pub fn description(&self) -> String {
        match self {
            CounterfactualSpec::ScaleAmount { factor } => {
                format!("Scale amount by {:.2}x", factor)
            }
            CounterfactualSpec::AddAmount { delta } => {
                format!("Add {} to amount", delta)
            }
            CounterfactualSpec::SetAmount { target } => {
                format!("Set amount to {}", target)
            }
            CounterfactualSpec::ShiftDate { days } => {
                if *days < 0 {
                    format!("Backdate by {} days", days.abs())
                } else {
                    format!("Forward-date by {} days", days)
                }
            }
            CounterfactualSpec::ChangePeriod { target_period } => {
                format!("Change to period {}", target_period)
            }
            CounterfactualSpec::ReclassifyAccount { new_account } => {
                format!("Reclassify to account {}", new_account)
            }
            CounterfactualSpec::AddLineItem {
                account,
                amount,
                is_debit,
            } => {
                format!(
                    "Add {} line for {} to account {}",
                    if *is_debit { "debit" } else { "credit" },
                    amount,
                    account
                )
            }
            CounterfactualSpec::RemoveLineItem { line_index } => {
                format!("Remove line item {}", line_index)
            }
            CounterfactualSpec::SplitTransaction { split_count } => {
                format!("Split into {} transactions", split_count)
            }
            CounterfactualSpec::CreateRoundTrip { intermediaries } => {
                format!(
                    "Create round-trip through {} entities",
                    intermediaries.len()
                )
            }
            CounterfactualSpec::SelfApprove => "Apply self-approval".to_string(),
            CounterfactualSpec::InjectFraud { fraud_type } => {
                format!("Inject {:?} fraud", fraud_type)
            }
            CounterfactualSpec::Custom { name, .. } => {
                format!("Apply custom transformation: {}", name)
            }
        }
    }
}

/// Generator for counterfactual pairs.
pub struct CounterfactualGenerator {
    /// Seed for reproducibility.
    seed: u64,
    /// Counter for generating unique IDs.
    counter: u64,
    /// Deterministic UUID factory for pair IDs.
    uuid_factory: DeterministicUuidFactory,
}

impl CounterfactualGenerator {
    /// Create a new counterfactual generator.
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            counter: 0,
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::Anomaly),
        }
    }

    /// Generate a counterfactual pair by applying a specification to an entry.
    pub fn generate(
        &mut self,
        original: &JournalEntry,
        spec: &CounterfactualSpec,
    ) -> CounterfactualPair {
        self.counter += 1;

        // Clone the original to create modified version
        let mut modified = original.clone();

        // Apply the specification to create the modified entry
        let injection_strategy = self.apply_spec(&mut modified, spec, original);

        // Create the anomaly label
        let anomaly_label =
            self.create_anomaly_label(&modified, spec, &injection_strategy, original);

        // Mark the modified entry as fraudulent if the anomaly type is fraud
        if let AnomalyType::Fraud(fraud_type) = spec.to_anomaly_type() {
            modified.header.is_fraud = true;
            modified.header.fraud_type = Some(fraud_type);
        }

        CounterfactualPair::new(
            original.clone(),
            modified,
            anomaly_label,
            injection_strategy,
            &self.uuid_factory,
        )
    }

    /// Generate multiple counterfactual pairs from a single original.
    pub fn generate_batch(
        &mut self,
        original: &JournalEntry,
        specs: &[CounterfactualSpec],
    ) -> Vec<CounterfactualPair> {
        specs
            .iter()
            .map(|spec| self.generate(original, spec))
            .collect()
    }

    /// Apply a specification to a journal entry.
    fn apply_spec(
        &self,
        entry: &mut JournalEntry,
        spec: &CounterfactualSpec,
        original: &JournalEntry,
    ) -> InjectionStrategy {
        match spec {
            CounterfactualSpec::ScaleAmount { factor } => {
                let original_total = original.total_debit();
                for line in &mut entry.lines {
                    if line.debit_amount > Decimal::ZERO {
                        let new_amount = Decimal::from_f64_retain(
                            line.debit_amount.to_f64().unwrap_or(0.0) * factor,
                        )
                        .unwrap_or(line.debit_amount);
                        line.debit_amount = new_amount;
                        line.local_amount = new_amount;
                    }
                    if line.credit_amount > Decimal::ZERO {
                        let new_amount = Decimal::from_f64_retain(
                            line.credit_amount.to_f64().unwrap_or(0.0) * factor,
                        )
                        .unwrap_or(line.credit_amount);
                        line.credit_amount = new_amount;
                        line.local_amount = -new_amount;
                    }
                }
                InjectionStrategy::AmountManipulation {
                    original: original_total,
                    factor: *factor,
                }
            }
            CounterfactualSpec::AddAmount { delta } => {
                // Add delta to first debit line and first credit line to keep balanced
                if !entry.lines.is_empty() {
                    let original_amount = entry.lines[0].debit_amount;
                    if entry.lines[0].debit_amount > Decimal::ZERO {
                        entry.lines[0].debit_amount += delta;
                        entry.lines[0].local_amount += delta;
                    }
                    // Find first credit line and add to it
                    for line in entry.lines.iter_mut().skip(1) {
                        if line.credit_amount > Decimal::ZERO {
                            line.credit_amount += delta;
                            line.local_amount -= delta;
                            break;
                        }
                    }
                    InjectionStrategy::AmountManipulation {
                        original: original_amount,
                        factor: (original_amount + delta).to_f64().unwrap_or(1.0)
                            / original_amount.to_f64().unwrap_or(1.0),
                    }
                } else {
                    InjectionStrategy::Custom {
                        name: "AddAmount".to_string(),
                        parameters: HashMap::new(),
                    }
                }
            }
            CounterfactualSpec::SetAmount { target } => {
                let original_total = original.total_debit();
                if !entry.lines.is_empty() {
                    // Set first debit line
                    if entry.lines[0].debit_amount > Decimal::ZERO {
                        entry.lines[0].debit_amount = *target;
                        entry.lines[0].local_amount = *target;
                    }
                    // Find first credit line and set it
                    for line in entry.lines.iter_mut().skip(1) {
                        if line.credit_amount > Decimal::ZERO {
                            line.credit_amount = *target;
                            line.local_amount = -*target;
                            break;
                        }
                    }
                }
                InjectionStrategy::AmountManipulation {
                    original: original_total,
                    factor: target.to_f64().unwrap_or(1.0) / original_total.to_f64().unwrap_or(1.0),
                }
            }
            CounterfactualSpec::ShiftDate { days } => {
                let original_date = entry.header.posting_date;
                entry.header.posting_date = if *days >= 0 {
                    entry.header.posting_date + chrono::Duration::days(*days as i64)
                } else {
                    entry.header.posting_date - chrono::Duration::days(days.abs() as i64)
                };
                InjectionStrategy::DateShift {
                    days_shifted: *days,
                    original_date,
                }
            }
            CounterfactualSpec::ChangePeriod { target_period } => {
                entry.header.fiscal_period = *target_period;
                InjectionStrategy::TimingManipulation {
                    timing_type: "PeriodChange".to_string(),
                    original_time: None,
                }
            }
            CounterfactualSpec::ReclassifyAccount { new_account } => {
                let old_account = if !entry.lines.is_empty() {
                    let old = entry.lines[0].gl_account.clone();
                    entry.lines[0].gl_account = new_account.clone();
                    entry.lines[0].account_code = new_account.clone();
                    old
                } else {
                    String::new()
                };
                InjectionStrategy::AccountMisclassification {
                    correct_account: old_account,
                    incorrect_account: new_account.clone(),
                }
            }
            CounterfactualSpec::SelfApprove => {
                let user_id = entry.header.created_by.clone();
                entry.header.sod_violation = true;
                InjectionStrategy::SelfApproval { user_id }
            }
            CounterfactualSpec::SplitTransaction { split_count } => {
                let original_amount = original.total_debit();
                InjectionStrategy::SplitTransaction {
                    original_amount,
                    split_count: *split_count,
                    split_doc_ids: vec![entry.header.document_id.to_string()],
                }
            }
            CounterfactualSpec::CreateRoundTrip { intermediaries } => {
                InjectionStrategy::CircularFlow {
                    entity_chain: intermediaries.clone(),
                }
            }
            CounterfactualSpec::AddLineItem {
                account,
                amount,
                is_debit,
            } => {
                let next_line_number =
                    entry.lines.iter().map(|l| l.line_number).max().unwrap_or(0) + 1;
                let new_line = if *is_debit {
                    JournalEntryLine::debit(
                        entry.header.document_id,
                        next_line_number,
                        account.clone(),
                        *amount,
                    )
                } else {
                    JournalEntryLine::credit(
                        entry.header.document_id,
                        next_line_number,
                        account.clone(),
                        *amount,
                    )
                };
                entry.lines.push(new_line);
                InjectionStrategy::Custom {
                    name: "AddLineItem".to_string(),
                    parameters: HashMap::from([
                        ("account".to_string(), account.clone()),
                        ("amount".to_string(), amount.to_string()),
                        ("is_debit".to_string(), is_debit.to_string()),
                    ]),
                }
            }
            CounterfactualSpec::RemoveLineItem { line_index } => {
                let removed_account = if *line_index < entry.lines.len() {
                    let removed = entry.lines.remove(*line_index);
                    removed.gl_account
                } else {
                    String::from("(index out of bounds)")
                };
                InjectionStrategy::Custom {
                    name: "RemoveLineItem".to_string(),
                    parameters: HashMap::from([
                        ("line_index".to_string(), line_index.to_string()),
                        ("removed_account".to_string(), removed_account),
                    ]),
                }
            }
            _ => InjectionStrategy::Custom {
                name: spec.description(),
                parameters: HashMap::new(),
            },
        }
    }

    /// Create an anomaly label for the modified entry.
    fn create_anomaly_label(
        &self,
        modified: &JournalEntry,
        spec: &CounterfactualSpec,
        strategy: &InjectionStrategy,
        original: &JournalEntry,
    ) -> LabeledAnomaly {
        let anomaly_id = format!("CF-{}-{}", self.seed, self.counter);
        let anomaly_type = spec.to_anomaly_type();

        LabeledAnomaly {
            anomaly_id,
            anomaly_type: anomaly_type.clone(),
            document_id: modified.header.document_id.to_string(),
            document_type: "JournalEntry".to_string(),
            company_code: modified.header.company_code.clone(),
            anomaly_date: modified.header.posting_date,
            detection_timestamp: Utc::now().naive_utc(),
            confidence: 1.0, // Counterfactuals are known anomalies
            severity: anomaly_type.severity(),
            description: spec.description(),
            related_entities: vec![original.header.document_id.to_string()],
            monetary_impact: Some(modified.total_debit()),
            metadata: HashMap::new(),
            is_injected: true,
            injection_strategy: Some(strategy.description()),
            cluster_id: None,
            original_document_hash: Some(format!("{:x}", hash_entry(original))),
            causal_reason: Some(AnomalyCausalReason::MLTrainingBalance {
                target_class: "counterfactual".to_string(),
            }),
            structured_strategy: Some(strategy.clone()),
            parent_anomaly_id: None,
            child_anomaly_ids: vec![],
            scenario_id: None,
            run_id: None,
            generation_seed: Some(self.seed),
        }
    }
}

/// Simple hash function for journal entries (for provenance tracking).
fn hash_entry(entry: &JournalEntry) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    entry.header.document_id.hash(&mut hasher);
    entry.header.company_code.hash(&mut hasher);
    entry.header.posting_date.hash(&mut hasher);
    entry.lines.len().hash(&mut hasher);
    hasher.finish()
}

/// Configuration for batch counterfactual generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterfactualConfig {
    /// Seed for reproducibility.
    pub seed: u64,
    /// Number of counterfactual variants per original.
    pub variants_per_original: usize,
    /// Specifications to apply (randomly selected).
    pub specifications: Vec<CounterfactualSpec>,
    /// Whether to include the original in output.
    pub include_originals: bool,
}

impl Default for CounterfactualConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            variants_per_original: 3,
            specifications: vec![
                CounterfactualSpec::ScaleAmount { factor: 1.5 },
                CounterfactualSpec::ScaleAmount { factor: 2.0 },
                CounterfactualSpec::ScaleAmount { factor: 0.5 },
                CounterfactualSpec::ShiftDate { days: -7 },
                CounterfactualSpec::ShiftDate { days: 30 },
                CounterfactualSpec::SelfApprove,
            ],
            include_originals: true,
        }
    }
}

// Re-export Decimal for use in specs
use rust_decimal::prelude::*;

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use datasynth_core::models::{JournalEntryHeader, JournalEntryLine};

    fn create_test_entry() -> JournalEntry {
        let header = JournalEntryHeader::new(
            "TEST".to_string(),
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
        );
        let mut entry = JournalEntry::new(header);

        entry.add_line(JournalEntryLine::debit(
            entry.header.document_id,
            1,
            "1100".to_string(),
            Decimal::new(10000, 2), // 100.00
        ));
        entry.add_line(JournalEntryLine::credit(
            entry.header.document_id,
            2,
            "2000".to_string(),
            Decimal::new(10000, 2), // 100.00
        ));

        entry
    }

    #[test]
    fn test_counterfactual_generator_scale_amount() {
        let mut generator = CounterfactualGenerator::new(42);
        let original = create_test_entry();
        let spec = CounterfactualSpec::ScaleAmount { factor: 2.0 };

        let pair = generator.generate(&original, &spec);

        assert_eq!(pair.original.total_debit(), Decimal::new(10000, 2));
        assert_eq!(pair.modified.total_debit(), Decimal::new(20000, 2));
        // ScaleAmount with factor <= 2.0 is statistical anomaly, not fraud
        assert!(!pair.modified.header.is_fraud);
    }

    #[test]
    fn test_counterfactual_generator_shift_date() {
        let mut generator = CounterfactualGenerator::new(42);
        let original = create_test_entry();
        let spec = CounterfactualSpec::ShiftDate { days: -7 };

        let pair = generator.generate(&original, &spec);

        let expected_date = NaiveDate::from_ymd_opt(2024, 6, 8).unwrap();
        assert_eq!(pair.modified.header.posting_date, expected_date);
    }

    #[test]
    fn test_counterfactual_spec_to_anomaly_type() {
        let spec = CounterfactualSpec::SelfApprove;
        let anomaly_type = spec.to_anomaly_type();

        // SelfApprove is classified as Fraud (FraudType::SelfApproval)
        assert!(matches!(
            anomaly_type,
            AnomalyType::Fraud(FraudType::SelfApproval)
        ));
    }

    #[test]
    fn test_counterfactual_batch_generation() {
        let mut generator = CounterfactualGenerator::new(42);
        let original = create_test_entry();
        let specs = vec![
            CounterfactualSpec::ScaleAmount { factor: 1.5 },
            CounterfactualSpec::ShiftDate { days: -3 },
            CounterfactualSpec::SelfApprove,
        ];

        let pairs = generator.generate_batch(&original, &specs);

        assert_eq!(pairs.len(), 3);
        // Only fraud types (ShiftDate, SelfApprove) set is_fraud = true
        // ScaleAmount with factor <= 2.0 is statistical, not fraud
        assert!(!pairs[0].modified.header.is_fraud); // ScaleAmount -> Statistical
        assert!(pairs[1].modified.header.is_fraud); // ShiftDate -> TimingAnomaly (Fraud)
        assert!(pairs[2].modified.header.is_fraud); // SelfApprove -> SelfApproval (Fraud)
    }
}
