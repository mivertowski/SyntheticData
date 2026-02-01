//! Benchmark suite definitions for ML evaluation.
//!
//! Provides standardized benchmark datasets for:
//! - Anomaly detection (AnomalyBench-1K)
//! - Fraud detection (FraudDetect-10K)
//! - Data quality detection (DataQuality-100K)
//! - Entity matching (EntityMatch-5K)
//! - ACFE-calibrated fraud detection (ACFE-Calibrated-1K, ACFE-Collusion-5K)
//! - Industry-specific fraud detection (Manufacturing, Retail, Healthcare, Technology, Financial Services)
//!
//! Each benchmark defines:
//! - Dataset size and composition
//! - Ground truth labels
//! - Evaluation metrics
//! - Expected baseline performance

pub mod acfe;
pub mod industry;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use acfe::{
    acfe_calibrated_1k, acfe_collusion_5k, acfe_management_override_2k, all_acfe_benchmarks,
    AcfeAlignment, AcfeCalibration, AcfeCategoryDistribution,
};
pub use industry::{
    all_industry_benchmarks, financial_services_fraud_5k, get_industry_benchmark,
    healthcare_fraud_5k, manufacturing_fraud_5k, retail_fraud_10k, technology_fraud_3k,
    IndustryBenchmarkAnalysis,
};

/// A benchmark suite definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSuite {
    /// Unique identifier for the benchmark
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of the benchmark
    pub description: String,
    /// Version of the benchmark specification
    pub version: String,
    /// Task type being evaluated
    pub task_type: BenchmarkTaskType,
    /// Dataset specification
    pub dataset: DatasetSpec,
    /// Evaluation configuration
    pub evaluation: EvaluationSpec,
    /// Expected baseline results
    pub baselines: Vec<BaselineResult>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Types of benchmark tasks.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BenchmarkTaskType {
    /// Binary classification: anomaly vs normal
    AnomalyDetection,
    /// Multi-class classification: fraud type
    FraudClassification,
    /// Binary classification: data quality issue detection
    DataQualityDetection,
    /// Entity resolution: matching records
    EntityMatching,
    /// Multi-label classification: anomaly types
    MultiLabelAnomalyDetection,
    /// Regression: amount prediction
    AmountPrediction,
    /// Time series: anomaly detection over time
    TimeSeriesAnomalyDetection,
    /// Graph: fraud network detection
    GraphFraudDetection,
}

/// Dataset specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetSpec {
    /// Total number of records
    pub total_records: usize,
    /// Number of positive examples (anomalies/fraud/issues)
    pub positive_count: usize,
    /// Number of negative examples (normal/clean)
    pub negative_count: usize,
    /// Class distribution by label
    pub class_distribution: HashMap<String, usize>,
    /// Feature set description
    pub features: FeatureSet,
    /// Split ratios (train/val/test)
    pub split_ratios: SplitRatios,
    /// Seed for reproducibility
    pub seed: u64,
    /// Time span in days
    pub time_span_days: u32,
    /// Number of companies
    pub num_companies: usize,
}

/// Feature set for the benchmark.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSet {
    /// Number of numerical features
    pub numerical_count: usize,
    /// Number of categorical features
    pub categorical_count: usize,
    /// Number of temporal features
    pub temporal_count: usize,
    /// Number of text features
    pub text_count: usize,
    /// Feature names and descriptions
    pub feature_descriptions: HashMap<String, String>,
}

/// Train/validation/test split ratios.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitRatios {
    /// Training set ratio (0.0-1.0)
    pub train: f64,
    /// Validation set ratio (0.0-1.0)
    pub validation: f64,
    /// Test set ratio (0.0-1.0)
    pub test: f64,
    /// Temporal split (if true, test set is the latest data)
    pub temporal_split: bool,
}

impl Default for SplitRatios {
    fn default() -> Self {
        Self {
            train: 0.7,
            validation: 0.15,
            test: 0.15,
            temporal_split: false,
        }
    }
}

/// Evaluation specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationSpec {
    /// Primary metric for ranking
    pub primary_metric: MetricType,
    /// All metrics to compute
    pub metrics: Vec<MetricType>,
    /// Threshold for binary classification
    pub classification_threshold: Option<f64>,
    /// Cross-validation folds (if applicable)
    pub cv_folds: Option<usize>,
    /// Cost matrix for cost-sensitive evaluation
    pub cost_matrix: Option<CostMatrix>,
}

/// Types of evaluation metrics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MetricType {
    /// Area Under ROC Curve
    AucRoc,
    /// Area Under Precision-Recall Curve
    AucPr,
    /// Precision at K
    PrecisionAtK(usize),
    /// Recall at K
    RecallAtK(usize),
    /// F1 Score
    F1Score,
    /// Precision
    Precision,
    /// Recall
    Recall,
    /// Accuracy
    Accuracy,
    /// Matthews Correlation Coefficient
    Mcc,
    /// Mean Average Precision
    Map,
    /// Normalized Discounted Cumulative Gain
    Ndcg,
    /// Mean Squared Error
    Mse,
    /// Mean Absolute Error
    Mae,
    /// R-squared
    R2,
    /// Log Loss
    LogLoss,
    /// Cohen's Kappa
    CohenKappa,
    /// Macro F1
    MacroF1,
    /// Weighted F1
    WeightedF1,
}

/// Cost matrix for cost-sensitive evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostMatrix {
    /// Cost of false positive
    pub false_positive_cost: f64,
    /// Cost of false negative
    pub false_negative_cost: f64,
    /// Cost of true positive (usually 0 or negative for reward)
    pub true_positive_cost: f64,
    /// Cost of true negative (usually 0)
    pub true_negative_cost: f64,
}

impl Default for CostMatrix {
    fn default() -> Self {
        Self {
            false_positive_cost: 1.0,
            false_negative_cost: 10.0, // Missing fraud is usually worse
            true_positive_cost: 0.0,
            true_negative_cost: 0.0,
        }
    }
}

/// Expected baseline result for a benchmark.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineResult {
    /// Model/algorithm name
    pub model_name: String,
    /// Model type
    pub model_type: BaselineModelType,
    /// Metric results
    pub metrics: HashMap<String, f64>,
    /// Training time in seconds
    pub training_time_seconds: Option<f64>,
    /// Inference time per sample in milliseconds
    pub inference_time_ms: Option<f64>,
    /// Notes about the baseline
    pub notes: Option<String>,
}

/// Types of baseline models.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BaselineModelType {
    /// Random baseline
    Random,
    /// Majority class baseline
    MajorityClass,
    /// Rule-based baseline
    RuleBased,
    /// Isolation Forest
    IsolationForest,
    /// One-Class SVM
    OneClassSvm,
    /// Local Outlier Factor
    Lof,
    /// Logistic Regression
    LogisticRegression,
    /// Random Forest
    RandomForest,
    /// XGBoost
    XgBoost,
    /// LightGBM
    LightGbm,
    /// Neural Network
    NeuralNetwork,
    /// Graph Neural Network
    Gnn,
    /// Autoencoder
    Autoencoder,
    /// Custom model
    Custom(String),
}

/// Leaderboard entry for benchmark results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    /// Rank on the leaderboard
    pub rank: usize,
    /// Submission name
    pub submission_name: String,
    /// Submitter name/organization
    pub submitter: String,
    /// Model description
    pub model_description: String,
    /// Primary metric score
    pub primary_score: f64,
    /// All metric scores
    pub all_scores: HashMap<String, f64>,
    /// Submission date
    pub submission_date: String,
    /// Is this a baseline entry
    pub is_baseline: bool,
}

/// Builder for creating benchmark suites.
pub struct BenchmarkBuilder {
    suite: BenchmarkSuite,
}

impl BenchmarkBuilder {
    /// Create a new benchmark builder.
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            suite: BenchmarkSuite {
                id: id.to_string(),
                name: name.to_string(),
                description: String::new(),
                version: "1.0.0".to_string(),
                task_type: BenchmarkTaskType::AnomalyDetection,
                dataset: DatasetSpec {
                    total_records: 1000,
                    positive_count: 50,
                    negative_count: 950,
                    class_distribution: HashMap::new(),
                    features: FeatureSet {
                        numerical_count: 10,
                        categorical_count: 5,
                        temporal_count: 3,
                        text_count: 1,
                        feature_descriptions: HashMap::new(),
                    },
                    split_ratios: SplitRatios::default(),
                    seed: 42,
                    time_span_days: 365,
                    num_companies: 1,
                },
                evaluation: EvaluationSpec {
                    primary_metric: MetricType::AucRoc,
                    metrics: vec![MetricType::AucRoc, MetricType::AucPr, MetricType::F1Score],
                    classification_threshold: Some(0.5),
                    cv_folds: None,
                    cost_matrix: None,
                },
                baselines: Vec::new(),
                metadata: HashMap::new(),
            },
        }
    }

    /// Set the description.
    pub fn description(mut self, desc: &str) -> Self {
        self.suite.description = desc.to_string();
        self
    }

    /// Set the task type.
    pub fn task_type(mut self, task_type: BenchmarkTaskType) -> Self {
        self.suite.task_type = task_type;
        self
    }

    /// Set dataset size.
    pub fn dataset_size(mut self, total: usize, positive: usize) -> Self {
        self.suite.dataset.total_records = total;
        self.suite.dataset.positive_count = positive;
        self.suite.dataset.negative_count = total.saturating_sub(positive);
        self
    }

    /// Set class distribution.
    pub fn class_distribution(mut self, distribution: HashMap<String, usize>) -> Self {
        self.suite.dataset.class_distribution = distribution;
        self
    }

    /// Set split ratios.
    pub fn split_ratios(mut self, train: f64, val: f64, test: f64, temporal: bool) -> Self {
        self.suite.dataset.split_ratios = SplitRatios {
            train,
            validation: val,
            test,
            temporal_split: temporal,
        };
        self
    }

    /// Set primary metric.
    pub fn primary_metric(mut self, metric: MetricType) -> Self {
        self.suite.evaluation.primary_metric = metric;
        self
    }

    /// Set all metrics.
    pub fn metrics(mut self, metrics: Vec<MetricType>) -> Self {
        self.suite.evaluation.metrics = metrics;
        self
    }

    /// Add a baseline result.
    pub fn add_baseline(mut self, baseline: BaselineResult) -> Self {
        self.suite.baselines.push(baseline);
        self
    }

    /// Set the seed.
    pub fn seed(mut self, seed: u64) -> Self {
        self.suite.dataset.seed = seed;
        self
    }

    /// Set time span.
    pub fn time_span_days(mut self, days: u32) -> Self {
        self.suite.dataset.time_span_days = days;
        self
    }

    /// Set number of companies.
    pub fn num_companies(mut self, n: usize) -> Self {
        self.suite.dataset.num_companies = n;
        self
    }

    /// Add metadata.
    pub fn metadata(mut self, key: &str, value: &str) -> Self {
        self.suite
            .metadata
            .insert(key.to_string(), value.to_string());
        self
    }

    /// Build the benchmark suite.
    pub fn build(self) -> BenchmarkSuite {
        self.suite
    }
}

// ============================================================================
// Pre-defined Benchmark Suites
// ============================================================================

/// AnomalyBench-1K: 1000 transactions with known anomalies.
pub fn anomaly_bench_1k() -> BenchmarkSuite {
    let mut class_dist = HashMap::new();
    class_dist.insert("normal".to_string(), 950);
    class_dist.insert("anomaly".to_string(), 50);

    BenchmarkBuilder::new("anomaly-bench-1k", "AnomalyBench-1K")
        .description("1000 journal entry transactions with 5% anomaly rate. Balanced mix of fraud, error, and statistical anomalies.")
        .task_type(BenchmarkTaskType::AnomalyDetection)
        .dataset_size(1000, 50)
        .class_distribution(class_dist)
        .split_ratios(0.7, 0.15, 0.15, true)
        .primary_metric(MetricType::AucPr)
        .metrics(vec![
            MetricType::AucRoc,
            MetricType::AucPr,
            MetricType::F1Score,
            MetricType::PrecisionAtK(10),
            MetricType::PrecisionAtK(50),
            MetricType::Recall,
        ])
        .seed(42)
        .time_span_days(90)
        .num_companies(1)
        .add_baseline(BaselineResult {
            model_name: "Random".to_string(),
            model_type: BaselineModelType::Random,
            metrics: [
                ("auc_roc".to_string(), 0.50),
                ("auc_pr".to_string(), 0.05),
                ("f1".to_string(), 0.09),
            ].into_iter().collect(),
            training_time_seconds: Some(0.0),
            inference_time_ms: Some(0.01),
            notes: Some("Random baseline for reference".to_string()),
        })
        .add_baseline(BaselineResult {
            model_name: "IsolationForest".to_string(),
            model_type: BaselineModelType::IsolationForest,
            metrics: [
                ("auc_roc".to_string(), 0.78),
                ("auc_pr".to_string(), 0.42),
                ("f1".to_string(), 0.45),
            ].into_iter().collect(),
            training_time_seconds: Some(0.5),
            inference_time_ms: Some(0.1),
            notes: Some("Unsupervised baseline".to_string()),
        })
        .add_baseline(BaselineResult {
            model_name: "XGBoost".to_string(),
            model_type: BaselineModelType::XgBoost,
            metrics: [
                ("auc_roc".to_string(), 0.92),
                ("auc_pr".to_string(), 0.68),
                ("f1".to_string(), 0.72),
            ].into_iter().collect(),
            training_time_seconds: Some(2.0),
            inference_time_ms: Some(0.05),
            notes: Some("Supervised baseline with full labels".to_string()),
        })
        .metadata("domain", "accounting")
        .metadata("difficulty", "easy")
        .build()
}

/// FraudDetect-10K: 10K transactions for fraud detection.
pub fn fraud_detect_10k() -> BenchmarkSuite {
    let mut class_dist = HashMap::new();
    class_dist.insert("normal".to_string(), 9700);
    class_dist.insert("fictitious_transaction".to_string(), 80);
    class_dist.insert("duplicate_payment".to_string(), 60);
    class_dist.insert("round_tripping".to_string(), 40);
    class_dist.insert("threshold_manipulation".to_string(), 50);
    class_dist.insert("self_approval".to_string(), 30);
    class_dist.insert("other_fraud".to_string(), 40);

    BenchmarkBuilder::new("fraud-detect-10k", "FraudDetect-10K")
        .description("10K journal entries with multi-class fraud labels. Includes 6 fraud types with realistic class imbalance.")
        .task_type(BenchmarkTaskType::FraudClassification)
        .dataset_size(10000, 300)
        .class_distribution(class_dist)
        .split_ratios(0.7, 0.15, 0.15, true)
        .primary_metric(MetricType::MacroF1)
        .metrics(vec![
            MetricType::AucRoc,
            MetricType::MacroF1,
            MetricType::WeightedF1,
            MetricType::Recall,
            MetricType::Precision,
            MetricType::CohenKappa,
        ])
        .seed(12345)
        .time_span_days(365)
        .num_companies(3)
        .add_baseline(BaselineResult {
            model_name: "MajorityClass".to_string(),
            model_type: BaselineModelType::MajorityClass,
            metrics: [
                ("macro_f1".to_string(), 0.07),
                ("weighted_f1".to_string(), 0.94),
            ].into_iter().collect(),
            training_time_seconds: Some(0.0),
            inference_time_ms: Some(0.01),
            notes: Some("Predicts normal for all transactions".to_string()),
        })
        .add_baseline(BaselineResult {
            model_name: "RandomForest".to_string(),
            model_type: BaselineModelType::RandomForest,
            metrics: [
                ("macro_f1".to_string(), 0.58),
                ("weighted_f1".to_string(), 0.96),
                ("auc_roc".to_string(), 0.89),
            ].into_iter().collect(),
            training_time_seconds: Some(5.0),
            inference_time_ms: Some(0.2),
            notes: Some("Balanced class weights".to_string()),
        })
        .add_baseline(BaselineResult {
            model_name: "LightGBM".to_string(),
            model_type: BaselineModelType::LightGbm,
            metrics: [
                ("macro_f1".to_string(), 0.65),
                ("weighted_f1".to_string(), 0.97),
                ("auc_roc".to_string(), 0.93),
            ].into_iter().collect(),
            training_time_seconds: Some(3.0),
            inference_time_ms: Some(0.05),
            notes: Some("Optimized hyperparameters".to_string()),
        })
        .metadata("domain", "accounting")
        .metadata("difficulty", "medium")
        .build()
}

/// DataQuality-100K: 100K records for data quality detection.
pub fn data_quality_100k() -> BenchmarkSuite {
    let mut class_dist = HashMap::new();
    class_dist.insert("clean".to_string(), 90000);
    class_dist.insert("missing_value".to_string(), 3000);
    class_dist.insert("typo".to_string(), 2000);
    class_dist.insert("format_error".to_string(), 2000);
    class_dist.insert("duplicate".to_string(), 1500);
    class_dist.insert("encoding_issue".to_string(), 1000);
    class_dist.insert("truncation".to_string(), 500);

    BenchmarkBuilder::new("data-quality-100k", "DataQuality-100K")
        .description("100K records with various data quality issues. Tests detection of missing values, typos, format errors, duplicates, and encoding issues.")
        .task_type(BenchmarkTaskType::DataQualityDetection)
        .dataset_size(100000, 10000)
        .class_distribution(class_dist)
        .split_ratios(0.8, 0.1, 0.1, false)
        .primary_metric(MetricType::F1Score)
        .metrics(vec![
            MetricType::F1Score,
            MetricType::Precision,
            MetricType::Recall,
            MetricType::AucRoc,
            MetricType::MacroF1,
        ])
        .seed(99999)
        .time_span_days(730) // 2 years
        .num_companies(5)
        .add_baseline(BaselineResult {
            model_name: "RuleBased".to_string(),
            model_type: BaselineModelType::RuleBased,
            metrics: [
                ("f1".to_string(), 0.72),
                ("precision".to_string(), 0.85),
                ("recall".to_string(), 0.62),
            ].into_iter().collect(),
            training_time_seconds: Some(0.0),
            inference_time_ms: Some(0.5),
            notes: Some("Regex patterns and null checks".to_string()),
        })
        .add_baseline(BaselineResult {
            model_name: "LogisticRegression".to_string(),
            model_type: BaselineModelType::LogisticRegression,
            metrics: [
                ("f1".to_string(), 0.78),
                ("precision".to_string(), 0.80),
                ("recall".to_string(), 0.76),
            ].into_iter().collect(),
            training_time_seconds: Some(2.0),
            inference_time_ms: Some(0.02),
            notes: Some("Character n-gram features".to_string()),
        })
        .add_baseline(BaselineResult {
            model_name: "XGBoost".to_string(),
            model_type: BaselineModelType::XgBoost,
            metrics: [
                ("f1".to_string(), 0.88),
                ("precision".to_string(), 0.90),
                ("recall".to_string(), 0.86),
            ].into_iter().collect(),
            training_time_seconds: Some(15.0),
            inference_time_ms: Some(0.08),
            notes: Some("Mixed feature types".to_string()),
        })
        .metadata("domain", "data_quality")
        .metadata("difficulty", "medium")
        .build()
}

/// EntityMatch-5K: 5K records for entity matching.
pub fn entity_match_5k() -> BenchmarkSuite {
    let mut class_dist = HashMap::new();
    class_dist.insert("match".to_string(), 2000);
    class_dist.insert("non_match".to_string(), 3000);

    BenchmarkBuilder::new("entity-match-5k", "EntityMatch-5K")
        .description("5K vendor/customer record pairs for entity matching. Includes name variations, typos, and abbreviations.")
        .task_type(BenchmarkTaskType::EntityMatching)
        .dataset_size(5000, 2000)
        .class_distribution(class_dist)
        .split_ratios(0.7, 0.15, 0.15, false)
        .primary_metric(MetricType::F1Score)
        .metrics(vec![
            MetricType::F1Score,
            MetricType::Precision,
            MetricType::Recall,
            MetricType::AucRoc,
        ])
        .seed(54321)
        .time_span_days(365)
        .num_companies(2)
        .add_baseline(BaselineResult {
            model_name: "ExactMatch".to_string(),
            model_type: BaselineModelType::RuleBased,
            metrics: [
                ("f1".to_string(), 0.35),
                ("precision".to_string(), 1.0),
                ("recall".to_string(), 0.21),
            ].into_iter().collect(),
            training_time_seconds: Some(0.0),
            inference_time_ms: Some(0.1),
            notes: Some("Exact string matching only".to_string()),
        })
        .add_baseline(BaselineResult {
            model_name: "FuzzyMatch".to_string(),
            model_type: BaselineModelType::RuleBased,
            metrics: [
                ("f1".to_string(), 0.68),
                ("precision".to_string(), 0.72),
                ("recall".to_string(), 0.65),
            ].into_iter().collect(),
            training_time_seconds: Some(0.0),
            inference_time_ms: Some(2.0),
            notes: Some("Levenshtein distance threshold".to_string()),
        })
        .add_baseline(BaselineResult {
            model_name: "Magellan".to_string(),
            model_type: BaselineModelType::RandomForest,
            metrics: [
                ("f1".to_string(), 0.89),
                ("precision".to_string(), 0.91),
                ("recall".to_string(), 0.87),
            ].into_iter().collect(),
            training_time_seconds: Some(10.0),
            inference_time_ms: Some(5.0),
            notes: Some("Feature-based entity matcher".to_string()),
        })
        .metadata("domain", "master_data")
        .metadata("difficulty", "hard")
        .build()
}

/// GraphFraud-10K: 10K transactions with network structure.
pub fn graph_fraud_10k() -> BenchmarkSuite {
    let mut class_dist = HashMap::new();
    class_dist.insert("normal".to_string(), 9500);
    class_dist.insert("fraud_network".to_string(), 500);

    BenchmarkBuilder::new("graph-fraud-10k", "GraphFraud-10K")
        .description("10K transactions with entity graph structure. Fraud detection using network features and GNN models.")
        .task_type(BenchmarkTaskType::GraphFraudDetection)
        .dataset_size(10000, 500)
        .class_distribution(class_dist)
        .split_ratios(0.7, 0.15, 0.15, true)
        .primary_metric(MetricType::AucPr)
        .metrics(vec![
            MetricType::AucPr,
            MetricType::AucRoc,
            MetricType::F1Score,
            MetricType::PrecisionAtK(100),
        ])
        .seed(77777)
        .time_span_days(365)
        .num_companies(4)
        .add_baseline(BaselineResult {
            model_name: "NodeFeatures".to_string(),
            model_type: BaselineModelType::XgBoost,
            metrics: [
                ("auc_pr".to_string(), 0.45),
                ("auc_roc".to_string(), 0.78),
            ].into_iter().collect(),
            training_time_seconds: Some(5.0),
            inference_time_ms: Some(0.1),
            notes: Some("XGBoost on node features only".to_string()),
        })
        .add_baseline(BaselineResult {
            model_name: "GraphSAGE".to_string(),
            model_type: BaselineModelType::Gnn,
            metrics: [
                ("auc_pr".to_string(), 0.62),
                ("auc_roc".to_string(), 0.88),
            ].into_iter().collect(),
            training_time_seconds: Some(60.0),
            inference_time_ms: Some(5.0),
            notes: Some("2-layer GraphSAGE".to_string()),
        })
        .add_baseline(BaselineResult {
            model_name: "GAT".to_string(),
            model_type: BaselineModelType::Gnn,
            metrics: [
                ("auc_pr".to_string(), 0.68),
                ("auc_roc".to_string(), 0.91),
            ].into_iter().collect(),
            training_time_seconds: Some(90.0),
            inference_time_ms: Some(8.0),
            notes: Some("Graph Attention Network".to_string()),
        })
        .metadata("domain", "graph_analytics")
        .metadata("difficulty", "hard")
        .build()
}

/// Get all available benchmark suites.
pub fn all_benchmarks() -> Vec<BenchmarkSuite> {
    let mut benchmarks = vec![
        anomaly_bench_1k(),
        fraud_detect_10k(),
        data_quality_100k(),
        entity_match_5k(),
        graph_fraud_10k(),
    ];

    // Add ACFE-calibrated benchmarks
    benchmarks.extend(all_acfe_benchmarks());

    // Add industry-specific benchmarks
    benchmarks.extend(all_industry_benchmarks());

    benchmarks
}

/// Get a benchmark by ID.
pub fn get_benchmark(id: &str) -> Option<BenchmarkSuite> {
    all_benchmarks().into_iter().find(|b| b.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anomaly_bench_1k() {
        let bench = anomaly_bench_1k();
        assert_eq!(bench.id, "anomaly-bench-1k");
        assert_eq!(bench.dataset.total_records, 1000);
        assert_eq!(bench.dataset.positive_count, 50);
        assert_eq!(bench.baselines.len(), 3);
    }

    #[test]
    fn test_fraud_detect_10k() {
        let bench = fraud_detect_10k();
        assert_eq!(bench.id, "fraud-detect-10k");
        assert_eq!(bench.dataset.total_records, 10000);
        assert_eq!(bench.task_type, BenchmarkTaskType::FraudClassification);
    }

    #[test]
    fn test_data_quality_100k() {
        let bench = data_quality_100k();
        assert_eq!(bench.id, "data-quality-100k");
        assert_eq!(bench.dataset.total_records, 100000);
        assert!(bench.dataset.class_distribution.len() > 5);
    }

    #[test]
    fn test_all_benchmarks() {
        let benchmarks = all_benchmarks();
        // 5 original + 3 ACFE + 5 industry = 13
        assert_eq!(benchmarks.len(), 13);

        // Verify all have baselines
        for bench in &benchmarks {
            assert!(
                !bench.baselines.is_empty(),
                "Benchmark {} has no baselines",
                bench.id
            );
        }
    }

    #[test]
    fn test_get_benchmark() {
        assert!(get_benchmark("fraud-detect-10k").is_some());
        assert!(get_benchmark("nonexistent").is_none());
    }

    #[test]
    fn test_builder() {
        let bench = BenchmarkBuilder::new("custom", "Custom Benchmark")
            .description("A custom test benchmark")
            .task_type(BenchmarkTaskType::AnomalyDetection)
            .dataset_size(500, 25)
            .seed(123)
            .build();

        assert_eq!(bench.id, "custom");
        assert_eq!(bench.dataset.total_records, 500);
        assert_eq!(bench.dataset.seed, 123);
    }
}
