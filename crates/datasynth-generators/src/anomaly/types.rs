//! Anomaly type definitions and configurations for injection.

use rand::Rng;
use rust_decimal::Decimal;

use datasynth_core::models::{
    AnomalyType, ErrorType, FraudType, ProcessIssueType, RelationalAnomalyType,
    StatisticalAnomalyType,
};

/// Configuration for fraud type injection.
#[derive(Debug, Clone)]
pub struct FraudTypeConfig {
    /// Type of fraud.
    pub fraud_type: FraudType,
    /// Relative weight for selection.
    pub weight: f64,
    /// Minimum amount for this fraud type.
    pub min_amount: Option<Decimal>,
    /// Maximum amount for this fraud type.
    pub max_amount: Option<Decimal>,
    /// Whether this requires specific conditions.
    pub requires_conditions: bool,
    /// Description template.
    pub description_template: String,
}

impl FraudTypeConfig {
    /// Creates default configurations for all fraud types.
    pub fn all_defaults() -> Vec<Self> {
        vec![
            Self {
                fraud_type: FraudType::FictitiousEntry,
                weight: 1.0,
                min_amount: Some(Decimal::new(10000, 0)),
                max_amount: Some(Decimal::new(500000, 0)),
                requires_conditions: false,
                description_template: "Fictitious journal entry with no supporting documentation"
                    .to_string(),
            },
            Self {
                fraud_type: FraudType::RoundDollarManipulation,
                weight: 2.0,
                min_amount: Some(Decimal::new(1000, 0)),
                max_amount: Some(Decimal::new(100000, 0)),
                requires_conditions: false,
                description_template:
                    "Suspicious round-dollar amount suggesting manual manipulation".to_string(),
            },
            Self {
                fraud_type: FraudType::JustBelowThreshold,
                weight: 2.5,
                min_amount: None,
                max_amount: None,
                requires_conditions: true,
                description_template: "Transaction amount just below approval threshold of {}"
                    .to_string(),
            },
            Self {
                fraud_type: FraudType::SelfApproval,
                weight: 1.5,
                min_amount: None,
                max_amount: None,
                requires_conditions: true,
                description_template: "User {} approved their own transaction".to_string(),
            },
            Self {
                fraud_type: FraudType::ExceededApprovalLimit,
                weight: 1.5,
                min_amount: None,
                max_amount: None,
                requires_conditions: true,
                description_template: "Approval exceeds user's limit of {}".to_string(),
            },
            Self {
                fraud_type: FraudType::SegregationOfDutiesViolation,
                weight: 1.0,
                min_amount: None,
                max_amount: None,
                requires_conditions: true,
                description_template: "User performed conflicting duties: {} and {}".to_string(),
            },
            Self {
                fraud_type: FraudType::DuplicatePayment,
                weight: 2.0,
                min_amount: Some(Decimal::new(5000, 0)),
                max_amount: Some(Decimal::new(200000, 0)),
                requires_conditions: false,
                description_template: "Duplicate payment to vendor {} for invoice {}".to_string(),
            },
            Self {
                fraud_type: FraudType::FictitiousVendor,
                weight: 0.5,
                min_amount: Some(Decimal::new(25000, 0)),
                max_amount: Some(Decimal::new(1000000, 0)),
                requires_conditions: false,
                description_template: "Payment to potentially fictitious vendor {}".to_string(),
            },
            Self {
                fraud_type: FraudType::RevenueManipulation,
                weight: 0.5,
                min_amount: Some(Decimal::new(100000, 0)),
                max_amount: Some(Decimal::new(5000000, 0)),
                requires_conditions: false,
                description_template: "Premature or fraudulent revenue recognition".to_string(),
            },
            Self {
                fraud_type: FraudType::ImproperCapitalization,
                weight: 1.0,
                min_amount: Some(Decimal::new(10000, 0)),
                max_amount: Some(Decimal::new(500000, 0)),
                requires_conditions: false,
                description_template: "Expense improperly capitalized as asset".to_string(),
            },
        ]
    }

    /// Selects a fraud type based on weights.
    pub fn select_weighted<'a, R: Rng>(configs: &'a [Self], rng: &mut R) -> &'a Self {
        let total_weight: f64 = configs.iter().map(|c| c.weight).sum();
        let mut random_weight = rng.gen::<f64>() * total_weight;

        for config in configs {
            random_weight -= config.weight;
            if random_weight <= 0.0 {
                return config;
            }
        }

        &configs[0]
    }
}

/// Configuration for error type injection.
#[derive(Debug, Clone)]
pub struct ErrorTypeConfig {
    /// Type of error.
    pub error_type: ErrorType,
    /// Relative weight for selection.
    pub weight: f64,
    /// Whether this error can be auto-detected.
    pub auto_detectable: bool,
    /// Description template.
    pub description_template: String,
}

impl ErrorTypeConfig {
    /// Creates default configurations for all error types.
    pub fn all_defaults() -> Vec<Self> {
        vec![
            Self {
                error_type: ErrorType::DuplicateEntry,
                weight: 2.0,
                auto_detectable: true,
                description_template: "Duplicate entry of document {}".to_string(),
            },
            Self {
                error_type: ErrorType::ReversedAmount,
                weight: 1.5,
                auto_detectable: false,
                description_template: "Debit and credit amounts appear reversed".to_string(),
            },
            Self {
                error_type: ErrorType::TransposedDigits,
                weight: 2.5,
                auto_detectable: false,
                description_template: "Digits transposed in amount: {} vs expected {}".to_string(),
            },
            Self {
                error_type: ErrorType::DecimalError,
                weight: 1.5,
                auto_detectable: false,
                description_template: "Decimal place error: {} should be {}".to_string(),
            },
            Self {
                error_type: ErrorType::MissingField,
                weight: 3.0,
                auto_detectable: true,
                description_template: "Missing required field: {}".to_string(),
            },
            Self {
                error_type: ErrorType::InvalidAccount,
                weight: 1.0,
                auto_detectable: true,
                description_template: "Invalid account code: {}".to_string(),
            },
            Self {
                error_type: ErrorType::WrongPeriod,
                weight: 2.0,
                auto_detectable: false,
                description_template: "Entry posted to wrong period: {} vs {}".to_string(),
            },
            Self {
                error_type: ErrorType::BackdatedEntry,
                weight: 1.5,
                auto_detectable: true,
                description_template: "Entry backdated by {} days".to_string(),
            },
            Self {
                error_type: ErrorType::FutureDatedEntry,
                weight: 0.5,
                auto_detectable: true,
                description_template: "Entry future-dated by {} days".to_string(),
            },
            Self {
                error_type: ErrorType::MisclassifiedAccount,
                weight: 2.0,
                auto_detectable: false,
                description_template: "Account {} misclassified, should be {}".to_string(),
            },
            Self {
                error_type: ErrorType::WrongCostCenter,
                weight: 2.5,
                auto_detectable: false,
                description_template: "Wrong cost center: {} vs {}".to_string(),
            },
            Self {
                error_type: ErrorType::UnbalancedEntry,
                weight: 0.5,
                auto_detectable: true,
                description_template: "Journal entry out of balance by {}".to_string(),
            },
            Self {
                error_type: ErrorType::RoundingError,
                weight: 3.0,
                auto_detectable: false,
                description_template: "Rounding discrepancy of {}".to_string(),
            },
            Self {
                error_type: ErrorType::CurrencyError,
                weight: 1.0,
                auto_detectable: false,
                description_template: "Currency conversion error: rate {} vs {}".to_string(),
            },
        ]
    }

    /// Selects an error type based on weights.
    pub fn select_weighted<'a, R: Rng>(configs: &'a [Self], rng: &mut R) -> &'a Self {
        let total_weight: f64 = configs.iter().map(|c| c.weight).sum();
        let mut random_weight = rng.gen::<f64>() * total_weight;

        for config in configs {
            random_weight -= config.weight;
            if random_weight <= 0.0 {
                return config;
            }
        }

        &configs[0]
    }
}

/// Configuration for process issue injection.
#[derive(Debug, Clone)]
pub struct ProcessIssueConfig {
    /// Type of process issue.
    pub issue_type: ProcessIssueType,
    /// Relative weight for selection.
    pub weight: f64,
    /// Description template.
    pub description_template: String,
}

impl ProcessIssueConfig {
    /// Creates default configurations for all process issue types.
    pub fn all_defaults() -> Vec<Self> {
        vec![
            Self {
                issue_type: ProcessIssueType::SkippedApproval,
                weight: 1.5,
                description_template: "Required approval level skipped".to_string(),
            },
            Self {
                issue_type: ProcessIssueType::LateApproval,
                weight: 2.5,
                description_template: "Approval received {} days after posting".to_string(),
            },
            Self {
                issue_type: ProcessIssueType::MissingDocumentation,
                weight: 3.0,
                description_template: "Missing supporting documentation for {}".to_string(),
            },
            Self {
                issue_type: ProcessIssueType::LatePosting,
                weight: 3.5,
                description_template: "Posted {} days after transaction date".to_string(),
            },
            Self {
                issue_type: ProcessIssueType::AfterHoursPosting,
                weight: 2.0,
                description_template: "Posted at {} outside business hours".to_string(),
            },
            Self {
                issue_type: ProcessIssueType::WeekendPosting,
                weight: 1.5,
                description_template: "Posted on weekend: {}".to_string(),
            },
            Self {
                issue_type: ProcessIssueType::RushedPeriodEnd,
                weight: 2.0,
                description_template: "Rushed posting in final {} hours of period".to_string(),
            },
            Self {
                issue_type: ProcessIssueType::ManualOverride,
                weight: 1.0,
                description_template: "Manual override of control: {}".to_string(),
            },
            Self {
                issue_type: ProcessIssueType::VagueDescription,
                weight: 4.0,
                description_template: "Vague or non-descriptive text: '{}'".to_string(),
            },
            Self {
                issue_type: ProcessIssueType::IncompleteApprovalChain,
                weight: 1.5,
                description_template: "Approval chain incomplete: missing {}".to_string(),
            },
        ]
    }

    /// Selects a process issue type based on weights.
    pub fn select_weighted<'a, R: Rng>(configs: &'a [Self], rng: &mut R) -> &'a Self {
        let total_weight: f64 = configs.iter().map(|c| c.weight).sum();
        let mut random_weight = rng.gen::<f64>() * total_weight;

        for config in configs {
            random_weight -= config.weight;
            if random_weight <= 0.0 {
                return config;
            }
        }

        &configs[0]
    }
}

/// Configuration for statistical anomaly injection.
#[derive(Debug, Clone)]
pub struct StatisticalAnomalyConfig {
    /// Type of statistical anomaly.
    pub anomaly_type: StatisticalAnomalyType,
    /// Relative weight for selection.
    pub weight: f64,
    /// Multiplier for amount anomalies.
    pub amount_multiplier: Option<f64>,
    /// Description template.
    pub description_template: String,
}

impl StatisticalAnomalyConfig {
    /// Creates default configurations for all statistical anomaly types.
    pub fn all_defaults() -> Vec<Self> {
        vec![
            Self {
                anomaly_type: StatisticalAnomalyType::UnusuallyHighAmount,
                weight: 2.0,
                amount_multiplier: Some(5.0),
                description_template: "Amount {} is {} standard deviations above mean".to_string(),
            },
            Self {
                anomaly_type: StatisticalAnomalyType::UnusuallyLowAmount,
                weight: 1.5,
                amount_multiplier: Some(0.1),
                description_template: "Amount {} is unusually low for this account".to_string(),
            },
            Self {
                anomaly_type: StatisticalAnomalyType::BenfordViolation,
                weight: 2.5,
                amount_multiplier: None,
                description_template:
                    "First digit {} violates Benford's Law (expected probability: {})".to_string(),
            },
            Self {
                anomaly_type: StatisticalAnomalyType::ExactDuplicateAmount,
                weight: 2.0,
                amount_multiplier: None,
                description_template: "Exact duplicate amount {} found in {} transactions"
                    .to_string(),
            },
            Self {
                anomaly_type: StatisticalAnomalyType::RepeatingAmount,
                weight: 1.5,
                amount_multiplier: None,
                description_template: "Repeating amount pattern: {} appears {} times".to_string(),
            },
            Self {
                anomaly_type: StatisticalAnomalyType::UnusualFrequency,
                weight: 2.0,
                amount_multiplier: None,
                description_template: "Unusual transaction frequency: {} vs expected {}"
                    .to_string(),
            },
            Self {
                anomaly_type: StatisticalAnomalyType::TransactionBurst,
                weight: 1.5,
                amount_multiplier: None,
                description_template: "Burst of {} transactions in {} minute window".to_string(),
            },
            Self {
                anomaly_type: StatisticalAnomalyType::UnusualTiming,
                weight: 3.0,
                amount_multiplier: None,
                description_template: "Transaction at unusual time: {}".to_string(),
            },
            Self {
                anomaly_type: StatisticalAnomalyType::TrendBreak,
                weight: 1.0,
                amount_multiplier: None,
                description_template: "Break in historical trend for account {}".to_string(),
            },
            Self {
                anomaly_type: StatisticalAnomalyType::StatisticalOutlier,
                weight: 2.0,
                amount_multiplier: Some(3.0),
                description_template: "Statistical outlier: z-score of {}".to_string(),
            },
        ]
    }

    /// Selects a statistical anomaly type based on weights.
    pub fn select_weighted<'a, R: Rng>(configs: &'a [Self], rng: &mut R) -> &'a Self {
        let total_weight: f64 = configs.iter().map(|c| c.weight).sum();
        let mut random_weight = rng.gen::<f64>() * total_weight;

        for config in configs {
            random_weight -= config.weight;
            if random_weight <= 0.0 {
                return config;
            }
        }

        &configs[0]
    }
}

/// Configuration for relational anomaly injection.
#[derive(Debug, Clone)]
pub struct RelationalAnomalyConfig {
    /// Type of relational anomaly.
    pub anomaly_type: RelationalAnomalyType,
    /// Relative weight for selection.
    pub weight: f64,
    /// Description template.
    pub description_template: String,
}

impl RelationalAnomalyConfig {
    /// Creates default configurations for all relational anomaly types.
    pub fn all_defaults() -> Vec<Self> {
        vec![
            Self {
                anomaly_type: RelationalAnomalyType::CircularTransaction,
                weight: 1.0,
                description_template: "Circular transaction pattern: {} -> {} -> {}".to_string(),
            },
            Self {
                anomaly_type: RelationalAnomalyType::UnusualAccountPair,
                weight: 2.5,
                description_template: "Unusual account combination: {} with {}".to_string(),
            },
            Self {
                anomaly_type: RelationalAnomalyType::NewCounterparty,
                weight: 3.0,
                description_template: "First transaction with new counterparty {}".to_string(),
            },
            Self {
                anomaly_type: RelationalAnomalyType::DormantAccountActivity,
                weight: 2.0,
                description_template: "Activity on dormant account {} after {} days".to_string(),
            },
            Self {
                anomaly_type: RelationalAnomalyType::UnmatchedIntercompany,
                weight: 1.5,
                description_template: "Unmatched intercompany transaction: {} vs {}".to_string(),
            },
            Self {
                anomaly_type: RelationalAnomalyType::CircularIntercompany,
                weight: 0.5,
                description_template: "Circular intercompany flow detected".to_string(),
            },
            Self {
                anomaly_type: RelationalAnomalyType::TransferPricingAnomaly,
                weight: 1.0,
                description_template: "Transfer price deviation: {} vs arm's length {}".to_string(),
            },
            Self {
                anomaly_type: RelationalAnomalyType::MissingRelationship,
                weight: 2.0,
                description_template: "Expected relationship missing between {} and {}".to_string(),
            },
            Self {
                anomaly_type: RelationalAnomalyType::CentralityAnomaly,
                weight: 1.0,
                description_template: "Node {} has unusual centrality score: {}".to_string(),
            },
        ]
    }

    /// Selects a relational anomaly type based on weights.
    pub fn select_weighted<'a, R: Rng>(configs: &'a [Self], rng: &mut R) -> &'a Self {
        let total_weight: f64 = configs.iter().map(|c| c.weight).sum();
        let mut random_weight = rng.gen::<f64>() * total_weight;

        for config in configs {
            random_weight -= config.weight;
            if random_weight <= 0.0 {
                return config;
            }
        }

        &configs[0]
    }
}

/// Combined anomaly type selector.
pub struct AnomalyTypeSelector {
    fraud_configs: Vec<FraudTypeConfig>,
    error_configs: Vec<ErrorTypeConfig>,
    process_configs: Vec<ProcessIssueConfig>,
    statistical_configs: Vec<StatisticalAnomalyConfig>,
    relational_configs: Vec<RelationalAnomalyConfig>,
}

impl Default for AnomalyTypeSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl AnomalyTypeSelector {
    /// Creates a new selector with default configurations.
    pub fn new() -> Self {
        Self {
            fraud_configs: FraudTypeConfig::all_defaults(),
            error_configs: ErrorTypeConfig::all_defaults(),
            process_configs: ProcessIssueConfig::all_defaults(),
            statistical_configs: StatisticalAnomalyConfig::all_defaults(),
            relational_configs: RelationalAnomalyConfig::all_defaults(),
        }
    }

    /// Selects a fraud anomaly type.
    pub fn select_fraud<R: Rng>(&self, rng: &mut R) -> AnomalyType {
        let config = FraudTypeConfig::select_weighted(&self.fraud_configs, rng);
        AnomalyType::Fraud(config.fraud_type)
    }

    /// Selects an error anomaly type.
    pub fn select_error<R: Rng>(&self, rng: &mut R) -> AnomalyType {
        let config = ErrorTypeConfig::select_weighted(&self.error_configs, rng);
        AnomalyType::Error(config.error_type)
    }

    /// Selects a process issue anomaly type.
    pub fn select_process_issue<R: Rng>(&self, rng: &mut R) -> AnomalyType {
        let config = ProcessIssueConfig::select_weighted(&self.process_configs, rng);
        AnomalyType::ProcessIssue(config.issue_type)
    }

    /// Selects a statistical anomaly type.
    pub fn select_statistical<R: Rng>(&self, rng: &mut R) -> AnomalyType {
        let config = StatisticalAnomalyConfig::select_weighted(&self.statistical_configs, rng);
        AnomalyType::Statistical(config.anomaly_type)
    }

    /// Selects a relational anomaly type.
    pub fn select_relational<R: Rng>(&self, rng: &mut R) -> AnomalyType {
        let config = RelationalAnomalyConfig::select_weighted(&self.relational_configs, rng);
        AnomalyType::Relational(config.anomaly_type)
    }

    /// Gets the fraud config for a specific type.
    pub fn get_fraud_config(&self, fraud_type: FraudType) -> Option<&FraudTypeConfig> {
        self.fraud_configs
            .iter()
            .find(|c| c.fraud_type == fraud_type)
    }

    /// Gets the error config for a specific type.
    pub fn get_error_config(&self, error_type: ErrorType) -> Option<&ErrorTypeConfig> {
        self.error_configs
            .iter()
            .find(|c| c.error_type == error_type)
    }

    /// Gets the statistical config for a specific type.
    pub fn get_statistical_config(
        &self,
        anomaly_type: StatisticalAnomalyType,
    ) -> Option<&StatisticalAnomalyConfig> {
        self.statistical_configs
            .iter()
            .find(|c| c.anomaly_type == anomaly_type)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_fraud_type_selection() {
        let configs = FraudTypeConfig::all_defaults();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        // Select multiple times to ensure variety
        let mut selected = std::collections::HashSet::new();
        for _ in 0..100 {
            let config = FraudTypeConfig::select_weighted(&configs, &mut rng);
            selected.insert(format!("{:?}", config.fraud_type));
        }

        assert!(selected.len() > 3); // Should select multiple types
    }

    #[test]
    fn test_anomaly_type_selector() {
        let selector = AnomalyTypeSelector::new();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let fraud = selector.select_fraud(&mut rng);
        assert!(matches!(fraud, AnomalyType::Fraud(_)));

        let error = selector.select_error(&mut rng);
        assert!(matches!(error, AnomalyType::Error(_)));
    }
}
