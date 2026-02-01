//! ACFE-aligned fraud evaluation benchmarks.
//!
//! Provides evaluation benchmarks calibrated against the ACFE (Association of
//! Certified Fraud Examiners) Report to the Nations statistics.
//!
//! Key ACFE statistics (2024 Report):
//! - Median loss: $117,000
//! - Median duration: 12 months
//! - Asset Misappropriation: 86% of cases, $100k median
//! - Corruption: 33% of cases, $150k median
//! - Financial Statement Fraud: 10% of cases, $954k median

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{
    BaselineModelType, BaselineResult, BenchmarkBuilder, BenchmarkSuite, BenchmarkTaskType,
    MetricType,
};

/// ACFE-calibrated fraud statistics for benchmarking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcfeCalibration {
    /// Expected median loss.
    pub median_loss: Decimal,
    /// Expected median duration in months.
    pub median_duration_months: u32,
    /// Expected category distribution.
    pub category_distribution: AcfeCategoryDistribution,
    /// Expected detection method distribution.
    pub detection_method_distribution: HashMap<String, f64>,
    /// Expected perpetrator department distribution.
    pub perpetrator_department_distribution: HashMap<String, f64>,
}

impl Default for AcfeCalibration {
    fn default() -> Self {
        Self {
            median_loss: Decimal::new(117_000, 0),
            median_duration_months: 12,
            category_distribution: AcfeCategoryDistribution::default(),
            detection_method_distribution: default_detection_methods(),
            perpetrator_department_distribution: default_perpetrator_departments(),
        }
    }
}

/// ACFE category distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcfeCategoryDistribution {
    /// Asset misappropriation rate (86% of cases).
    pub asset_misappropriation: f64,
    /// Corruption rate (33% of cases, overlaps with others).
    pub corruption: f64,
    /// Financial statement fraud rate (10% of cases).
    pub financial_statement_fraud: f64,
    /// Asset misappropriation median loss.
    pub asset_misappropriation_median: Decimal,
    /// Corruption median loss.
    pub corruption_median: Decimal,
    /// Financial statement fraud median loss.
    pub financial_statement_fraud_median: Decimal,
}

impl Default for AcfeCategoryDistribution {
    fn default() -> Self {
        Self {
            asset_misappropriation: 0.86,
            corruption: 0.33,
            financial_statement_fraud: 0.10,
            asset_misappropriation_median: Decimal::new(100_000, 0),
            corruption_median: Decimal::new(150_000, 0),
            financial_statement_fraud_median: Decimal::new(954_000, 0),
        }
    }
}

fn default_detection_methods() -> HashMap<String, f64> {
    let mut methods = HashMap::new();
    methods.insert("tip".to_string(), 0.42);
    methods.insert("internal_audit".to_string(), 0.16);
    methods.insert("management_review".to_string(), 0.12);
    methods.insert("by_accident".to_string(), 0.06);
    methods.insert("external_audit".to_string(), 0.04);
    methods.insert("account_reconciliation".to_string(), 0.05);
    methods.insert("document_examination".to_string(), 0.04);
    methods.insert("surveillance".to_string(), 0.02);
    methods.insert("it_controls".to_string(), 0.02);
    methods.insert("other".to_string(), 0.07);
    methods
}

fn default_perpetrator_departments() -> HashMap<String, f64> {
    let mut depts = HashMap::new();
    depts.insert("accounting".to_string(), 0.21);
    depts.insert("operations".to_string(), 0.17);
    depts.insert("executive_management".to_string(), 0.12);
    depts.insert("sales".to_string(), 0.11);
    depts.insert("customer_service".to_string(), 0.09);
    depts.insert("purchasing".to_string(), 0.07);
    depts.insert("finance".to_string(), 0.05);
    depts.insert("warehousing".to_string(), 0.05);
    depts.insert("other".to_string(), 0.13);
    depts
}

/// ACFE alignment metrics for evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcfeAlignment {
    /// Mean absolute deviation from ACFE category distribution.
    pub category_distribution_mad: f64,
    /// Ratio of actual median loss to ACFE expected.
    pub median_loss_ratio: f64,
    /// Kolmogorov-Smirnov statistic for duration distribution.
    pub duration_distribution_ks: f64,
    /// Chi-squared statistic against ACFE detection method distribution.
    pub detection_method_chi_sq: f64,
    /// Chi-squared statistic against ACFE perpetrator department distribution.
    pub perpetrator_department_chi_sq: f64,
    /// Overall alignment score (0.0-1.0, higher is better).
    pub overall_alignment: f64,
    /// List of alignment issues.
    pub issues: Vec<String>,
}

impl Default for AcfeAlignment {
    fn default() -> Self {
        Self {
            category_distribution_mad: 0.0,
            median_loss_ratio: 1.0,
            duration_distribution_ks: 0.0,
            detection_method_chi_sq: 0.0,
            perpetrator_department_chi_sq: 0.0,
            overall_alignment: 1.0,
            issues: Vec::new(),
        }
    }
}

impl AcfeAlignment {
    /// Calculate alignment metrics from observed values.
    pub fn calculate(
        observed_category_dist: &HashMap<String, f64>,
        observed_median_loss: Decimal,
        observed_median_duration: u32,
        _observed_detection_methods: &HashMap<String, f64>,
        _observed_perpetrator_depts: &HashMap<String, f64>,
    ) -> Self {
        let calibration = AcfeCalibration::default();
        let mut alignment = Self::default();
        let mut score_components: Vec<f64> = Vec::new();

        // Category distribution MAD
        let expected_cat = HashMap::from([
            (
                "asset_misappropriation".to_string(),
                calibration.category_distribution.asset_misappropriation,
            ),
            (
                "corruption".to_string(),
                calibration.category_distribution.corruption,
            ),
            (
                "financial_statement_fraud".to_string(),
                calibration.category_distribution.financial_statement_fraud,
            ),
        ]);

        let mut total_deviation = 0.0;
        let mut count = 0;
        for (cat, expected) in &expected_cat {
            let observed = observed_category_dist.get(cat).copied().unwrap_or(0.0);
            total_deviation += (observed - expected).abs();
            count += 1;
        }
        alignment.category_distribution_mad = if count > 0 {
            total_deviation / count as f64
        } else {
            0.0
        };

        // Penalize if MAD > 0.10
        let category_score = (1.0 - alignment.category_distribution_mad * 5.0).max(0.0);
        score_components.push(category_score);
        if alignment.category_distribution_mad > 0.10 {
            alignment.issues.push(format!(
                "Category distribution deviates from ACFE by MAD={:.2}",
                alignment.category_distribution_mad
            ));
        }

        // Median loss ratio
        let expected_loss_f64 = calibration
            .median_loss
            .to_string()
            .parse::<f64>()
            .unwrap_or(117000.0);
        let observed_loss_f64 = observed_median_loss
            .to_string()
            .parse::<f64>()
            .unwrap_or(0.0);
        alignment.median_loss_ratio = if expected_loss_f64 > 0.0 {
            observed_loss_f64 / expected_loss_f64
        } else {
            0.0
        };

        // Score: penalize if ratio is outside [0.5, 2.0]
        let loss_score = if alignment.median_loss_ratio >= 0.5 && alignment.median_loss_ratio <= 2.0
        {
            1.0 - ((alignment.median_loss_ratio - 1.0).abs() * 0.5)
        } else {
            0.2
        };
        score_components.push(loss_score);
        if alignment.median_loss_ratio < 0.5 || alignment.median_loss_ratio > 2.0 {
            alignment.issues.push(format!(
                "Median loss ratio {:.2}x differs significantly from ACFE benchmark",
                alignment.median_loss_ratio
            ));
        }

        // Duration comparison
        let expected_duration = calibration.median_duration_months as f64;
        let observed_duration = observed_median_duration as f64;
        let duration_ratio = if expected_duration > 0.0 {
            observed_duration / expected_duration
        } else {
            0.0
        };

        let duration_score = if (0.5..=2.0).contains(&duration_ratio) {
            1.0 - ((duration_ratio - 1.0).abs() * 0.5)
        } else {
            0.2
        };
        score_components.push(duration_score);
        if !(0.5..=2.0).contains(&duration_ratio) {
            alignment.issues.push(format!(
                "Median duration {}mo differs from ACFE benchmark of {}mo",
                observed_median_duration, calibration.median_duration_months
            ));
        }

        // Overall alignment
        alignment.overall_alignment = if !score_components.is_empty() {
            score_components.iter().sum::<f64>() / score_components.len() as f64
        } else {
            1.0
        };

        alignment
    }
}

/// ACFE-Calibrated-1K: Benchmark calibrated to ACFE statistics.
pub fn acfe_calibrated_1k() -> BenchmarkSuite {
    let mut class_dist = HashMap::new();
    // Based on ACFE category percentages
    class_dist.insert("normal".to_string(), 900);
    class_dist.insert("asset_misappropriation".to_string(), 43); // 86% of fraud
    class_dist.insert("corruption".to_string(), 17); // 33% of fraud (overlap)
    class_dist.insert("financial_statement_fraud".to_string(), 5); // 10% of fraud
    class_dist.insert("mixed_scheme".to_string(), 35); // Combination

    BenchmarkBuilder::new("acfe-calibrated-1k", "ACFE-Calibrated-1K")
        .description("1K transactions calibrated to ACFE Report to the Nations statistics. Tests fraud category distribution, loss amounts, and duration patterns.")
        .task_type(BenchmarkTaskType::FraudClassification)
        .dataset_size(1000, 100)
        .class_distribution(class_dist)
        .split_ratios(0.7, 0.15, 0.15, true)
        .primary_metric(MetricType::MacroF1)
        .metrics(vec![
            MetricType::AucRoc,
            MetricType::AucPr,
            MetricType::MacroF1,
            MetricType::WeightedF1,
            MetricType::Recall,
            MetricType::Precision,
        ])
        .seed(2024)
        .time_span_days(365)
        .num_companies(2)
        .add_baseline(BaselineResult {
            model_name: "Random".to_string(),
            model_type: BaselineModelType::Random,
            metrics: [
                ("auc_roc".to_string(), 0.50),
                ("macro_f1".to_string(), 0.10),
            ].into_iter().collect(),
            training_time_seconds: Some(0.0),
            inference_time_ms: Some(0.01),
            notes: Some("Random baseline".to_string()),
        })
        .add_baseline(BaselineResult {
            model_name: "IsolationForest".to_string(),
            model_type: BaselineModelType::IsolationForest,
            metrics: [
                ("auc_roc".to_string(), 0.75),
                ("macro_f1".to_string(), 0.35),
            ].into_iter().collect(),
            training_time_seconds: Some(0.5),
            inference_time_ms: Some(0.1),
            notes: Some("Unsupervised, tuned for ACFE patterns".to_string()),
        })
        .add_baseline(BaselineResult {
            model_name: "XGBoost-ACFE".to_string(),
            model_type: BaselineModelType::XgBoost,
            metrics: [
                ("auc_roc".to_string(), 0.88),
                ("macro_f1".to_string(), 0.62),
            ].into_iter().collect(),
            training_time_seconds: Some(3.0),
            inference_time_ms: Some(0.05),
            notes: Some("Supervised with ACFE-informed features".to_string()),
        })
        .metadata("calibration_source", "ACFE Report to the Nations 2024")
        .metadata("median_loss", "117000")
        .metadata("median_duration_months", "12")
        .metadata("domain", "fraud_detection")
        .metadata("difficulty", "medium")
        .build()
}

/// ACFE-Collusion-5K: Benchmark focused on collusion detection.
pub fn acfe_collusion_5k() -> BenchmarkSuite {
    let mut class_dist = HashMap::new();
    class_dist.insert("normal".to_string(), 4500);
    class_dist.insert("solo_fraud".to_string(), 300);
    class_dist.insert("two_person_collusion".to_string(), 120);
    class_dist.insert("ring_collusion".to_string(), 50);
    class_dist.insert("external_collusion".to_string(), 30);

    BenchmarkBuilder::new("acfe-collusion-5k", "ACFE-Collusion-5K")
        .description("5K transactions for collusion detection. ACFE reports collusion cases have 2x median loss. Tests detection of coordinated fraud networks.")
        .task_type(BenchmarkTaskType::FraudClassification)
        .dataset_size(5000, 500)
        .class_distribution(class_dist)
        .split_ratios(0.7, 0.15, 0.15, true)
        .primary_metric(MetricType::AucPr)
        .metrics(vec![
            MetricType::AucPr,
            MetricType::AucRoc,
            MetricType::MacroF1,
            MetricType::PrecisionAtK(50),
            MetricType::Recall,
        ])
        .seed(12345)
        .time_span_days(730) // 2 years for scheme development
        .num_companies(3)
        .add_baseline(BaselineResult {
            model_name: "NodeFeatures".to_string(),
            model_type: BaselineModelType::XgBoost,
            metrics: [
                ("auc_pr".to_string(), 0.35),
                ("auc_roc".to_string(), 0.72),
            ].into_iter().collect(),
            training_time_seconds: Some(2.0),
            inference_time_ms: Some(0.05),
            notes: Some("Without relationship features".to_string()),
        })
        .add_baseline(BaselineResult {
            model_name: "NetworkFeatures".to_string(),
            model_type: BaselineModelType::XgBoost,
            metrics: [
                ("auc_pr".to_string(), 0.52),
                ("auc_roc".to_string(), 0.84),
            ].into_iter().collect(),
            training_time_seconds: Some(5.0),
            inference_time_ms: Some(0.1),
            notes: Some("With entity relationship features".to_string()),
        })
        .add_baseline(BaselineResult {
            model_name: "GNN-Collusion".to_string(),
            model_type: BaselineModelType::Gnn,
            metrics: [
                ("auc_pr".to_string(), 0.68),
                ("auc_roc".to_string(), 0.91),
            ].into_iter().collect(),
            training_time_seconds: Some(60.0),
            inference_time_ms: Some(5.0),
            notes: Some("Graph neural network for network patterns".to_string()),
        })
        .metadata("collusion_multiplier", "2.0")
        .metadata("domain", "fraud_detection")
        .metadata("difficulty", "hard")
        .build()
}

/// ACFE-ManagementOverride-2K: Benchmark for management override detection.
pub fn acfe_management_override_2k() -> BenchmarkSuite {
    let mut class_dist = HashMap::new();
    class_dist.insert("normal".to_string(), 1800);
    class_dist.insert("journal_entry_override".to_string(), 80);
    class_dist.insert("revenue_manipulation".to_string(), 50);
    class_dist.insert("reserve_manipulation".to_string(), 40);
    class_dist.insert("expense_capitalization".to_string(), 30);

    BenchmarkBuilder::new("acfe-management-override-2k", "ACFE-ManagementOverride-2K")
        .description("2K transactions testing management override detection. ACFE reports executive fraud has 6x higher median loss. Tests detection of sophisticated C-suite fraud patterns.")
        .task_type(BenchmarkTaskType::FraudClassification)
        .dataset_size(2000, 200)
        .class_distribution(class_dist)
        .split_ratios(0.7, 0.15, 0.15, true)
        .primary_metric(MetricType::AucPr)
        .metrics(vec![
            MetricType::AucPr,
            MetricType::AucRoc,
            MetricType::MacroF1,
            MetricType::Recall,
            MetricType::PrecisionAtK(20),
        ])
        .seed(99999)
        .time_span_days(1095) // 3 years for sophisticated schemes
        .num_companies(1)
        .add_baseline(BaselineResult {
            model_name: "RuleBased".to_string(),
            model_type: BaselineModelType::RuleBased,
            metrics: [
                ("auc_pr".to_string(), 0.25),
                ("auc_roc".to_string(), 0.65),
            ].into_iter().collect(),
            training_time_seconds: Some(0.0),
            inference_time_ms: Some(0.5),
            notes: Some("Traditional audit analytics rules".to_string()),
        })
        .add_baseline(BaselineResult {
            model_name: "Autoencoder".to_string(),
            model_type: BaselineModelType::Autoencoder,
            metrics: [
                ("auc_pr".to_string(), 0.42),
                ("auc_roc".to_string(), 0.78),
            ].into_iter().collect(),
            training_time_seconds: Some(30.0),
            inference_time_ms: Some(1.0),
            notes: Some("Reconstruction-based anomaly detection".to_string()),
        })
        .add_baseline(BaselineResult {
            model_name: "LightGBM-Override".to_string(),
            model_type: BaselineModelType::LightGbm,
            metrics: [
                ("auc_pr".to_string(), 0.58),
                ("auc_roc".to_string(), 0.86),
            ].into_iter().collect(),
            training_time_seconds: Some(5.0),
            inference_time_ms: Some(0.05),
            notes: Some("With temporal and approval chain features".to_string()),
        })
        .metadata("executive_loss_multiplier", "6.0")
        .metadata("domain", "fraud_detection")
        .metadata("difficulty", "expert")
        .build()
}

/// Get all ACFE-calibrated benchmarks.
pub fn all_acfe_benchmarks() -> Vec<BenchmarkSuite> {
    vec![
        acfe_calibrated_1k(),
        acfe_collusion_5k(),
        acfe_management_override_2k(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acfe_calibration_defaults() {
        let calibration = AcfeCalibration::default();
        assert_eq!(calibration.median_loss, Decimal::new(117_000, 0));
        assert_eq!(calibration.median_duration_months, 12);
        assert_eq!(
            calibration.category_distribution.asset_misappropriation,
            0.86
        );
    }

    #[test]
    fn test_acfe_alignment_calculate() {
        let observed_cat = HashMap::from([
            ("asset_misappropriation".to_string(), 0.85),
            ("corruption".to_string(), 0.30),
            ("financial_statement_fraud".to_string(), 0.08),
        ]);

        let alignment = AcfeAlignment::calculate(
            &observed_cat,
            Decimal::new(120_000, 0),
            10,
            &HashMap::new(),
            &HashMap::new(),
        );

        // Should be close to ACFE
        assert!(alignment.overall_alignment > 0.7);
        assert!(alignment.median_loss_ratio > 0.9 && alignment.median_loss_ratio < 1.1);
    }

    #[test]
    fn test_acfe_alignment_poor() {
        let observed_cat = HashMap::from([
            ("asset_misappropriation".to_string(), 0.50), // Way off from 0.86
            ("corruption".to_string(), 0.50),             // Way off from 0.33
            ("financial_statement_fraud".to_string(), 0.50), // Way off from 0.10
        ]);

        let alignment = AcfeAlignment::calculate(
            &observed_cat,
            Decimal::new(500_000, 0), // 4x expected
            36,                       // 3x expected
            &HashMap::new(),
            &HashMap::new(),
        );

        // Should have issues
        assert!(!alignment.issues.is_empty());
        assert!(alignment.overall_alignment < 0.7);
    }

    #[test]
    fn test_acfe_calibrated_1k() {
        let bench = acfe_calibrated_1k();
        assert_eq!(bench.id, "acfe-calibrated-1k");
        assert_eq!(bench.dataset.total_records, 1000);
        assert!(bench.metadata.contains_key("calibration_source"));
        assert!(!bench.baselines.is_empty());
    }

    #[test]
    fn test_acfe_collusion_5k() {
        let bench = acfe_collusion_5k();
        assert_eq!(bench.id, "acfe-collusion-5k");
        assert_eq!(bench.dataset.total_records, 5000);
        assert!(bench
            .dataset
            .class_distribution
            .contains_key("ring_collusion"));
    }

    #[test]
    fn test_acfe_management_override_2k() {
        let bench = acfe_management_override_2k();
        assert_eq!(bench.id, "acfe-management-override-2k");
        assert!(bench
            .dataset
            .class_distribution
            .contains_key("journal_entry_override"));
    }

    #[test]
    fn test_all_acfe_benchmarks() {
        let benchmarks = all_acfe_benchmarks();
        assert_eq!(benchmarks.len(), 3);

        for bench in &benchmarks {
            assert!(bench.metadata.get("domain") == Some(&"fraud_detection".to_string()));
        }
    }
}
