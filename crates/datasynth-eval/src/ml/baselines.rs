//! ML Baseline Tasks and Metrics
//!
//! Defines standard ML tasks for benchmarking synthetic data quality:
//! - Anomaly Detection (isolation forest, autoencoder baselines)
//! - Entity Matching (duplicate detection)
//! - Link Prediction (graph-based fraud detection)
//! - Time Series Forecasting (amount and volume prediction)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ML task type for benchmarking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MLTaskType {
    /// Anomaly detection (fraud, errors, outliers).
    AnomalyDetection,
    /// Entity matching (duplicate detection, record linkage).
    EntityMatching,
    /// Link prediction (graph-based fraud, relationship discovery).
    LinkPrediction,
    /// Time series forecasting (amount, volume prediction).
    TimeSeriesForecasting,
}

impl std::fmt::Display for MLTaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MLTaskType::AnomalyDetection => write!(f, "Anomaly Detection"),
            MLTaskType::EntityMatching => write!(f, "Entity Matching"),
            MLTaskType::LinkPrediction => write!(f, "Link Prediction"),
            MLTaskType::TimeSeriesForecasting => write!(f, "Time Series Forecasting"),
        }
    }
}

/// Baseline algorithm for a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BaselineAlgorithm {
    // Anomaly Detection
    IsolationForest,
    LocalOutlierFactor,
    OneClassSVM,
    Autoencoder,

    // Entity Matching
    ExactMatch,
    JaccardSimilarity,
    LevenshteinDistance,
    TFIDFCosine,

    // Link Prediction
    CommonNeighbors,
    AdamicAdar,
    ResourceAllocation,
    GraphNeuralNetwork,

    // Time Series
    ARIMA,
    ExponentialSmoothing,
    Prophet,
    LSTM,
}

impl std::fmt::Display for BaselineAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BaselineAlgorithm::IsolationForest => write!(f, "Isolation Forest"),
            BaselineAlgorithm::LocalOutlierFactor => write!(f, "Local Outlier Factor"),
            BaselineAlgorithm::OneClassSVM => write!(f, "One-Class SVM"),
            BaselineAlgorithm::Autoencoder => write!(f, "Autoencoder"),
            BaselineAlgorithm::ExactMatch => write!(f, "Exact Match"),
            BaselineAlgorithm::JaccardSimilarity => write!(f, "Jaccard Similarity"),
            BaselineAlgorithm::LevenshteinDistance => write!(f, "Levenshtein Distance"),
            BaselineAlgorithm::TFIDFCosine => write!(f, "TF-IDF Cosine"),
            BaselineAlgorithm::CommonNeighbors => write!(f, "Common Neighbors"),
            BaselineAlgorithm::AdamicAdar => write!(f, "Adamic-Adar"),
            BaselineAlgorithm::ResourceAllocation => write!(f, "Resource Allocation"),
            BaselineAlgorithm::GraphNeuralNetwork => write!(f, "Graph Neural Network"),
            BaselineAlgorithm::ARIMA => write!(f, "ARIMA"),
            BaselineAlgorithm::ExponentialSmoothing => write!(f, "Exponential Smoothing"),
            BaselineAlgorithm::Prophet => write!(f, "Prophet"),
            BaselineAlgorithm::LSTM => write!(f, "LSTM"),
        }
    }
}

/// Classification metrics for binary/multiclass tasks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClassificationMetrics {
    /// Accuracy (correct predictions / total).
    pub accuracy: f64,
    /// Precision (true positives / predicted positives).
    pub precision: f64,
    /// Recall (true positives / actual positives).
    pub recall: f64,
    /// F1 score (harmonic mean of precision and recall).
    pub f1_score: f64,
    /// Area under ROC curve.
    pub auc_roc: f64,
    /// Area under precision-recall curve.
    pub auc_pr: f64,
    /// Matthews correlation coefficient.
    pub mcc: f64,
}

impl ClassificationMetrics {
    /// Create metrics from confusion matrix values.
    pub fn from_confusion(tp: u64, tn: u64, fp: u64, fn_: u64) -> Self {
        let total = (tp + tn + fp + fn_) as f64;
        let accuracy = if total > 0.0 {
            (tp + tn) as f64 / total
        } else {
            0.0
        };

        let precision = if tp + fp > 0 {
            tp as f64 / (tp + fp) as f64
        } else {
            0.0
        };
        let recall = if tp + fn_ > 0 {
            tp as f64 / (tp + fn_) as f64
        } else {
            0.0
        };
        let f1_score = if precision + recall > 0.0 {
            2.0 * precision * recall / (precision + recall)
        } else {
            0.0
        };

        // Matthews Correlation Coefficient
        let mcc_num = (tp * tn) as f64 - (fp * fn_) as f64;
        let mcc_denom =
            ((tp + fp) as f64 * (tp + fn_) as f64 * (tn + fp) as f64 * (tn + fn_) as f64).sqrt();
        let mcc = if mcc_denom > 0.0 {
            mcc_num / mcc_denom
        } else {
            0.0
        };

        Self {
            accuracy,
            precision,
            recall,
            f1_score,
            auc_roc: 0.0, // Requires probability scores
            auc_pr: 0.0,  // Requires probability scores
            mcc,
        }
    }
}

/// Regression metrics for continuous prediction tasks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RegressionMetrics {
    /// Mean Absolute Error.
    pub mae: f64,
    /// Mean Squared Error.
    pub mse: f64,
    /// Root Mean Squared Error.
    pub rmse: f64,
    /// Mean Absolute Percentage Error.
    pub mape: f64,
    /// R-squared (coefficient of determination).
    pub r2: f64,
}

impl RegressionMetrics {
    /// Create metrics from predictions and actuals.
    pub fn from_predictions(predictions: &[f64], actuals: &[f64]) -> Self {
        if predictions.len() != actuals.len() || predictions.is_empty() {
            return Self::default();
        }

        let n = predictions.len() as f64;

        // Calculate errors
        let errors: Vec<f64> = predictions
            .iter()
            .zip(actuals.iter())
            .map(|(p, a)| p - a)
            .collect();

        let mae = errors.iter().map(|e| e.abs()).sum::<f64>() / n;
        let mse = errors.iter().map(|e| e * e).sum::<f64>() / n;
        let rmse = mse.sqrt();

        // MAPE (avoid division by zero)
        let mape = predictions
            .iter()
            .zip(actuals.iter())
            .filter(|(_, a)| a.abs() > 1e-10)
            .map(|(p, a)| ((p - a) / a).abs())
            .sum::<f64>()
            / n
            * 100.0;

        // R-squared
        let actual_mean = actuals.iter().sum::<f64>() / n;
        let ss_tot: f64 = actuals.iter().map(|a| (a - actual_mean).powi(2)).sum();
        let ss_res: f64 = errors.iter().map(|e| e * e).sum();
        let r2 = if ss_tot > 0.0 {
            1.0 - (ss_res / ss_tot)
        } else {
            0.0
        };

        Self {
            mae,
            mse,
            rmse,
            mape,
            r2,
        }
    }
}

/// Ranking metrics for link prediction and recommendation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RankingMetrics {
    /// Mean Reciprocal Rank.
    pub mrr: f64,
    /// Hits@1 (top-1 accuracy).
    pub hits_at_1: f64,
    /// Hits@10.
    pub hits_at_10: f64,
    /// Hits@100.
    pub hits_at_100: f64,
    /// Normalized Discounted Cumulative Gain.
    pub ndcg: f64,
}

/// ML baseline task definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineTask {
    /// Task identifier.
    pub id: String,
    /// Task type.
    pub task_type: MLTaskType,
    /// Human-readable description.
    pub description: String,
    /// Required data fields.
    pub required_fields: Vec<String>,
    /// Target field (label).
    pub target_field: String,
    /// Recommended baseline algorithms.
    pub recommended_algorithms: Vec<BaselineAlgorithm>,
    /// Expected baseline performance (for reference).
    pub expected_metrics: ExpectedMetrics,
}

/// Expected performance metrics for a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedMetrics {
    /// Minimum acceptable F1/R2/MRR depending on task type.
    pub min_acceptable: f64,
    /// Good performance threshold.
    pub good_threshold: f64,
    /// Excellent performance threshold.
    pub excellent_threshold: f64,
    /// Primary metric name.
    pub primary_metric: String,
}

/// Baseline task results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineResult {
    /// Task that was evaluated.
    pub task: BaselineTask,
    /// Algorithm used.
    pub algorithm: BaselineAlgorithm,
    /// Classification metrics (if applicable).
    pub classification_metrics: Option<ClassificationMetrics>,
    /// Regression metrics (if applicable).
    pub regression_metrics: Option<RegressionMetrics>,
    /// Ranking metrics (if applicable).
    pub ranking_metrics: Option<RankingMetrics>,
    /// Training time in seconds.
    pub training_time_secs: f64,
    /// Inference time per sample in milliseconds.
    pub inference_time_ms: f64,
    /// Number of samples used for training.
    pub train_samples: usize,
    /// Number of samples used for testing.
    pub test_samples: usize,
    /// Performance grade (excellent, good, acceptable, poor).
    pub grade: PerformanceGrade,
}

/// Performance grade for baseline results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PerformanceGrade {
    Excellent,
    Good,
    Acceptable,
    Poor,
}

impl std::fmt::Display for PerformanceGrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PerformanceGrade::Excellent => write!(f, "Excellent"),
            PerformanceGrade::Good => write!(f, "Good"),
            PerformanceGrade::Acceptable => write!(f, "Acceptable"),
            PerformanceGrade::Poor => write!(f, "Poor"),
        }
    }
}

/// Collection of baseline results for all tasks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BaselineEvaluation {
    /// Results for each task.
    pub results: Vec<BaselineResult>,
    /// Summary statistics.
    pub summary: BaselineSummary,
}

/// Summary of baseline evaluation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BaselineSummary {
    /// Number of tasks evaluated.
    pub tasks_evaluated: usize,
    /// Tasks meeting minimum threshold.
    pub tasks_passing: usize,
    /// Tasks with good performance.
    pub tasks_good: usize,
    /// Tasks with excellent performance.
    pub tasks_excellent: usize,
    /// Average primary metric across all tasks.
    pub average_primary_metric: f64,
    /// Best performing algorithm per task.
    pub best_algorithms: HashMap<String, BaselineAlgorithm>,
}

/// Get predefined baseline tasks for synthetic accounting data.
pub fn get_accounting_baseline_tasks() -> Vec<BaselineTask> {
    vec![
        // Anomaly Detection Tasks
        BaselineTask {
            id: "anomaly_fraud_detection".to_string(),
            task_type: MLTaskType::AnomalyDetection,
            description:
                "Detect fraudulent journal entries based on amount, timing, and user patterns"
                    .to_string(),
            required_fields: vec![
                "amount".to_string(),
                "posting_date".to_string(),
                "created_by".to_string(),
                "account_number".to_string(),
                "is_fraud".to_string(),
            ],
            target_field: "is_fraud".to_string(),
            recommended_algorithms: vec![
                BaselineAlgorithm::IsolationForest,
                BaselineAlgorithm::Autoencoder,
                BaselineAlgorithm::LocalOutlierFactor,
            ],
            expected_metrics: ExpectedMetrics {
                min_acceptable: 0.60,
                good_threshold: 0.75,
                excellent_threshold: 0.90,
                primary_metric: "f1_score".to_string(),
            },
        },
        BaselineTask {
            id: "anomaly_error_detection".to_string(),
            task_type: MLTaskType::AnomalyDetection,
            description: "Detect data entry errors and anomalies in journal entries".to_string(),
            required_fields: vec![
                "amount".to_string(),
                "account_number".to_string(),
                "is_anomaly".to_string(),
            ],
            target_field: "is_anomaly".to_string(),
            recommended_algorithms: vec![
                BaselineAlgorithm::IsolationForest,
                BaselineAlgorithm::OneClassSVM,
            ],
            expected_metrics: ExpectedMetrics {
                min_acceptable: 0.50,
                good_threshold: 0.70,
                excellent_threshold: 0.85,
                primary_metric: "f1_score".to_string(),
            },
        },
        // Entity Matching Tasks
        BaselineTask {
            id: "entity_vendor_matching".to_string(),
            task_type: MLTaskType::EntityMatching,
            description: "Match duplicate or similar vendor records".to_string(),
            required_fields: vec![
                "vendor_name".to_string(),
                "vendor_address".to_string(),
                "tax_id".to_string(),
            ],
            target_field: "is_duplicate".to_string(),
            recommended_algorithms: vec![
                BaselineAlgorithm::TFIDFCosine,
                BaselineAlgorithm::LevenshteinDistance,
                BaselineAlgorithm::JaccardSimilarity,
            ],
            expected_metrics: ExpectedMetrics {
                min_acceptable: 0.80,
                good_threshold: 0.90,
                excellent_threshold: 0.95,
                primary_metric: "f1_score".to_string(),
            },
        },
        BaselineTask {
            id: "entity_customer_matching".to_string(),
            task_type: MLTaskType::EntityMatching,
            description: "Match duplicate or similar customer records".to_string(),
            required_fields: vec![
                "customer_name".to_string(),
                "customer_address".to_string(),
                "customer_email".to_string(),
            ],
            target_field: "is_duplicate".to_string(),
            recommended_algorithms: vec![
                BaselineAlgorithm::TFIDFCosine,
                BaselineAlgorithm::LevenshteinDistance,
            ],
            expected_metrics: ExpectedMetrics {
                min_acceptable: 0.80,
                good_threshold: 0.90,
                excellent_threshold: 0.95,
                primary_metric: "f1_score".to_string(),
            },
        },
        // Link Prediction Tasks
        BaselineTask {
            id: "link_fraud_network".to_string(),
            task_type: MLTaskType::LinkPrediction,
            description: "Predict fraudulent transaction links in entity graph".to_string(),
            required_fields: vec![
                "source_entity".to_string(),
                "target_entity".to_string(),
                "transaction_amount".to_string(),
                "is_suspicious".to_string(),
            ],
            target_field: "is_suspicious".to_string(),
            recommended_algorithms: vec![
                BaselineAlgorithm::GraphNeuralNetwork,
                BaselineAlgorithm::AdamicAdar,
                BaselineAlgorithm::CommonNeighbors,
            ],
            expected_metrics: ExpectedMetrics {
                min_acceptable: 0.10,
                good_threshold: 0.25,
                excellent_threshold: 0.40,
                primary_metric: "mrr".to_string(),
            },
        },
        BaselineTask {
            id: "link_intercompany".to_string(),
            task_type: MLTaskType::LinkPrediction,
            description: "Predict intercompany transaction relationships".to_string(),
            required_fields: vec![
                "company_from".to_string(),
                "company_to".to_string(),
                "transaction_type".to_string(),
            ],
            target_field: "has_relationship".to_string(),
            recommended_algorithms: vec![
                BaselineAlgorithm::CommonNeighbors,
                BaselineAlgorithm::ResourceAllocation,
            ],
            expected_metrics: ExpectedMetrics {
                min_acceptable: 0.20,
                good_threshold: 0.35,
                excellent_threshold: 0.50,
                primary_metric: "mrr".to_string(),
            },
        },
        // Time Series Forecasting Tasks
        BaselineTask {
            id: "forecast_transaction_volume".to_string(),
            task_type: MLTaskType::TimeSeriesForecasting,
            description: "Forecast daily transaction volume".to_string(),
            required_fields: vec!["date".to_string(), "transaction_count".to_string()],
            target_field: "transaction_count".to_string(),
            recommended_algorithms: vec![
                BaselineAlgorithm::Prophet,
                BaselineAlgorithm::ARIMA,
                BaselineAlgorithm::ExponentialSmoothing,
            ],
            expected_metrics: ExpectedMetrics {
                min_acceptable: 0.70,
                good_threshold: 0.85,
                excellent_threshold: 0.95,
                primary_metric: "r2".to_string(),
            },
        },
        BaselineTask {
            id: "forecast_transaction_amount".to_string(),
            task_type: MLTaskType::TimeSeriesForecasting,
            description: "Forecast daily transaction amounts".to_string(),
            required_fields: vec!["date".to_string(), "total_amount".to_string()],
            target_field: "total_amount".to_string(),
            recommended_algorithms: vec![
                BaselineAlgorithm::LSTM,
                BaselineAlgorithm::Prophet,
                BaselineAlgorithm::ARIMA,
            ],
            expected_metrics: ExpectedMetrics {
                min_acceptable: 0.60,
                good_threshold: 0.80,
                excellent_threshold: 0.90,
                primary_metric: "r2".to_string(),
            },
        },
    ]
}

/// Configuration for baseline evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineConfig {
    /// Which task types to evaluate.
    pub task_types: Vec<MLTaskType>,
    /// Train/test split ratio (e.g., 0.8 for 80% train).
    pub train_ratio: f64,
    /// Random seed for reproducibility.
    pub seed: u64,
    /// Whether to run all algorithms or just the primary one.
    pub run_all_algorithms: bool,
    /// Maximum training time per algorithm in seconds.
    pub max_training_time_secs: u64,
}

impl Default for BaselineConfig {
    fn default() -> Self {
        Self {
            task_types: vec![
                MLTaskType::AnomalyDetection,
                MLTaskType::EntityMatching,
                MLTaskType::LinkPrediction,
                MLTaskType::TimeSeriesForecasting,
            ],
            train_ratio: 0.8,
            seed: 42,
            run_all_algorithms: false,
            max_training_time_secs: 300,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_classification_metrics_from_confusion() {
        // Perfect classifier
        let metrics = ClassificationMetrics::from_confusion(100, 100, 0, 0);
        assert!((metrics.accuracy - 1.0).abs() < 0.001);
        assert!((metrics.precision - 1.0).abs() < 0.001);
        assert!((metrics.recall - 1.0).abs() < 0.001);
        assert!((metrics.f1_score - 1.0).abs() < 0.001);

        // Random classifier (50/50)
        let metrics = ClassificationMetrics::from_confusion(50, 50, 50, 50);
        assert!((metrics.accuracy - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_regression_metrics_from_predictions() {
        let predictions = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let actuals = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        let metrics = RegressionMetrics::from_predictions(&predictions, &actuals);
        assert!((metrics.mae).abs() < 0.001);
        assert!((metrics.mse).abs() < 0.001);
        assert!((metrics.r2 - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_get_accounting_baseline_tasks() {
        let tasks = get_accounting_baseline_tasks();
        assert!(!tasks.is_empty());

        // Check we have all task types
        let has_anomaly = tasks
            .iter()
            .any(|t| t.task_type == MLTaskType::AnomalyDetection);
        let has_entity = tasks
            .iter()
            .any(|t| t.task_type == MLTaskType::EntityMatching);
        let has_link = tasks
            .iter()
            .any(|t| t.task_type == MLTaskType::LinkPrediction);
        let has_ts = tasks
            .iter()
            .any(|t| t.task_type == MLTaskType::TimeSeriesForecasting);

        assert!(has_anomaly, "Should have anomaly detection tasks");
        assert!(has_entity, "Should have entity matching tasks");
        assert!(has_link, "Should have link prediction tasks");
        assert!(has_ts, "Should have time series tasks");
    }

    #[test]
    fn test_baseline_config_default() {
        let config = BaselineConfig::default();
        assert_eq!(config.train_ratio, 0.8);
        assert_eq!(config.task_types.len(), 4);
    }
}
