//! Industry-specific evaluation benchmarks.
//!
//! Provides evaluation benchmarks for industry-specific fraud patterns:
//! - Manufacturing: Yield manipulation, labor fraud, inventory schemes
//! - Retail: Sweethearting, skimming, refund fraud
//! - Healthcare: Upcoding, unbundling, kickbacks
//! - Technology: Revenue manipulation, capitalization fraud
//! - Financial Services: Loan fraud, trading manipulation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{
    BaselineModelType, BaselineResult, BenchmarkBuilder, BenchmarkSuite, BenchmarkTaskType,
    MetricType,
};

/// Industry-specific benchmark analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustryBenchmarkAnalysis {
    /// Industry sector.
    pub industry: String,
    /// Industry-specific anomaly count.
    pub industry_anomaly_count: usize,
    /// Industry-specific anomaly rate.
    pub industry_anomaly_rate: f64,
    /// Transaction type distribution.
    pub transaction_type_distribution: HashMap<String, usize>,
    /// Terminology coverage score (0.0-1.0).
    pub terminology_coverage: f64,
    /// Regulatory framework alignment score.
    pub regulatory_alignment: f64,
    /// Issues found.
    pub issues: Vec<String>,
}

impl Default for IndustryBenchmarkAnalysis {
    fn default() -> Self {
        Self {
            industry: String::new(),
            industry_anomaly_count: 0,
            industry_anomaly_rate: 0.0,
            transaction_type_distribution: HashMap::new(),
            terminology_coverage: 1.0,
            regulatory_alignment: 1.0,
            issues: Vec::new(),
        }
    }
}

// =============================================================================
// Manufacturing Benchmarks
// =============================================================================

/// Manufacturing-Fraud-5K: Manufacturing-specific fraud detection.
pub fn manufacturing_fraud_5k() -> BenchmarkSuite {
    let mut class_dist = HashMap::new();
    class_dist.insert("normal".to_string(), 4500);
    class_dist.insert("yield_manipulation".to_string(), 150);
    class_dist.insert("labor_misallocation".to_string(), 120);
    class_dist.insert("phantom_production".to_string(), 50);
    class_dist.insert("standard_cost_manipulation".to_string(), 80);
    class_dist.insert("inventory_fraud".to_string(), 60);
    class_dist.insert("scrap_fraud".to_string(), 40);

    BenchmarkBuilder::new("manufacturing-fraud-5k", "Manufacturing-Fraud-5K")
        .description("5K manufacturing transactions with industry-specific fraud patterns. Tests detection of yield manipulation, labor fraud, and inventory schemes.")
        .task_type(BenchmarkTaskType::FraudClassification)
        .dataset_size(5000, 500)
        .class_distribution(class_dist)
        .split_ratios(0.7, 0.15, 0.15, true)
        .primary_metric(MetricType::MacroF1)
        .metrics(vec![
            MetricType::AucRoc,
            MetricType::AucPr,
            MetricType::MacroF1,
            MetricType::Recall,
            MetricType::Precision,
        ])
        .seed(11111)
        .time_span_days(365)
        .num_companies(2)
        .add_baseline(BaselineResult {
            model_name: "RuleBased-MFG".to_string(),
            model_type: BaselineModelType::RuleBased,
            metrics: [
                ("auc_roc".to_string(), 0.68),
                ("macro_f1".to_string(), 0.35),
            ].into_iter().collect(),
            training_time_seconds: Some(0.0),
            inference_time_ms: Some(0.5),
            notes: Some("Manufacturing variance analysis rules".to_string()),
        })
        .add_baseline(BaselineResult {
            model_name: "XGBoost-MFG".to_string(),
            model_type: BaselineModelType::XgBoost,
            metrics: [
                ("auc_roc".to_string(), 0.85),
                ("macro_f1".to_string(), 0.58),
            ].into_iter().collect(),
            training_time_seconds: Some(4.0),
            inference_time_ms: Some(0.05),
            notes: Some("With BOM/routing features".to_string()),
        })
        .metadata("industry", "manufacturing")
        .metadata("transaction_types", "production_order,material_issue,labor_posting,variance")
        .metadata("difficulty", "medium")
        .build()
}

// =============================================================================
// Retail Benchmarks
// =============================================================================

/// Retail-Fraud-10K: Retail-specific fraud detection.
pub fn retail_fraud_10k() -> BenchmarkSuite {
    let mut class_dist = HashMap::new();
    class_dist.insert("normal".to_string(), 9000);
    class_dist.insert("sweethearting".to_string(), 300);
    class_dist.insert("skimming".to_string(), 150);
    class_dist.insert("refund_fraud".to_string(), 250);
    class_dist.insert("void_abuse".to_string(), 120);
    class_dist.insert("gift_card_fraud".to_string(), 80);
    class_dist.insert("employee_discount_abuse".to_string(), 60);
    class_dist.insert("vendor_kickback".to_string(), 40);

    BenchmarkBuilder::new("retail-fraud-10k", "Retail-Fraud-10K")
        .description("10K retail POS transactions with industry-specific fraud patterns. Tests detection of sweethearting, skimming, and refund schemes.")
        .task_type(BenchmarkTaskType::FraudClassification)
        .dataset_size(10000, 1000)
        .class_distribution(class_dist)
        .split_ratios(0.7, 0.15, 0.15, true)
        .primary_metric(MetricType::AucPr)
        .metrics(vec![
            MetricType::AucPr,
            MetricType::AucRoc,
            MetricType::MacroF1,
            MetricType::PrecisionAtK(100),
            MetricType::Recall,
        ])
        .seed(22222)
        .time_span_days(90)
        .num_companies(1)
        .add_baseline(BaselineResult {
            model_name: "RuleBased-Retail".to_string(),
            model_type: BaselineModelType::RuleBased,
            metrics: [
                ("auc_pr".to_string(), 0.42),
                ("auc_roc".to_string(), 0.72),
            ].into_iter().collect(),
            training_time_seconds: Some(0.0),
            inference_time_ms: Some(0.2),
            notes: Some("POS exception analysis rules".to_string()),
        })
        .add_baseline(BaselineResult {
            model_name: "RandomForest-Retail".to_string(),
            model_type: BaselineModelType::RandomForest,
            metrics: [
                ("auc_pr".to_string(), 0.58),
                ("auc_roc".to_string(), 0.84),
            ].into_iter().collect(),
            training_time_seconds: Some(3.0),
            inference_time_ms: Some(0.1),
            notes: Some("With cashier behavior features".to_string()),
        })
        .add_baseline(BaselineResult {
            model_name: "LightGBM-Retail".to_string(),
            model_type: BaselineModelType::LightGbm,
            metrics: [
                ("auc_pr".to_string(), 0.68),
                ("auc_roc".to_string(), 0.90),
            ].into_iter().collect(),
            training_time_seconds: Some(2.0),
            inference_time_ms: Some(0.05),
            notes: Some("Optimized with temporal features".to_string()),
        })
        .metadata("industry", "retail")
        .metadata("transaction_types", "pos_sale,return,void,discount,gift_card")
        .metadata("difficulty", "medium")
        .build()
}

// =============================================================================
// Healthcare Benchmarks
// =============================================================================

/// Healthcare-Fraud-5K: Healthcare revenue cycle fraud detection.
pub fn healthcare_fraud_5k() -> BenchmarkSuite {
    let mut class_dist = HashMap::new();
    class_dist.insert("normal".to_string(), 4500);
    class_dist.insert("upcoding".to_string(), 150);
    class_dist.insert("unbundling".to_string(), 100);
    class_dist.insert("phantom_billing".to_string(), 50);
    class_dist.insert("duplicate_billing".to_string(), 80);
    class_dist.insert("kickback".to_string(), 40);
    class_dist.insert("medical_necessity_abuse".to_string(), 60);
    class_dist.insert("dme_fraud".to_string(), 20);

    BenchmarkBuilder::new("healthcare-fraud-5k", "Healthcare-Fraud-5K")
        .description("5K healthcare revenue cycle transactions with industry-specific fraud patterns. Tests detection of upcoding, unbundling, and kickbacks under HIPAA/Stark/FCA compliance.")
        .task_type(BenchmarkTaskType::FraudClassification)
        .dataset_size(5000, 500)
        .class_distribution(class_dist)
        .split_ratios(0.7, 0.15, 0.15, true)
        .primary_metric(MetricType::AucPr)
        .metrics(vec![
            MetricType::AucPr,
            MetricType::AucRoc,
            MetricType::MacroF1,
            MetricType::Recall,
            MetricType::PrecisionAtK(50),
        ])
        .seed(33333)
        .time_span_days(365)
        .num_companies(1)
        .add_baseline(BaselineResult {
            model_name: "NCCI-Edits".to_string(),
            model_type: BaselineModelType::RuleBased,
            metrics: [
                ("auc_pr".to_string(), 0.35),
                ("auc_roc".to_string(), 0.65),
            ].into_iter().collect(),
            training_time_seconds: Some(0.0),
            inference_time_ms: Some(1.0),
            notes: Some("CMS NCCI edit-based detection".to_string()),
        })
        .add_baseline(BaselineResult {
            model_name: "ClaimAnalytics".to_string(),
            model_type: BaselineModelType::RandomForest,
            metrics: [
                ("auc_pr".to_string(), 0.52),
                ("auc_roc".to_string(), 0.80),
            ].into_iter().collect(),
            training_time_seconds: Some(5.0),
            inference_time_ms: Some(0.2),
            notes: Some("With ICD-10/CPT coding features".to_string()),
        })
        .add_baseline(BaselineResult {
            model_name: "DeepClaim".to_string(),
            model_type: BaselineModelType::NeuralNetwork,
            metrics: [
                ("auc_pr".to_string(), 0.65),
                ("auc_roc".to_string(), 0.88),
            ].into_iter().collect(),
            training_time_seconds: Some(30.0),
            inference_time_ms: Some(2.0),
            notes: Some("Embedding-based claim analysis".to_string()),
        })
        .metadata("industry", "healthcare")
        .metadata("regulatory_framework", "hipaa,stark,anti_kickback,fca")
        .metadata("coding_systems", "icd10,cpt,drg,hcpcs")
        .metadata("difficulty", "hard")
        .build()
}

// =============================================================================
// Technology Benchmarks
// =============================================================================

/// Technology-Fraud-3K: Technology revenue recognition fraud.
pub fn technology_fraud_3k() -> BenchmarkSuite {
    let mut class_dist = HashMap::new();
    class_dist.insert("normal".to_string(), 2700);
    class_dist.insert("premature_revenue".to_string(), 100);
    class_dist.insert("side_letter_abuse".to_string(), 60);
    class_dist.insert("channel_stuffing".to_string(), 50);
    class_dist.insert("improper_capitalization".to_string(), 50);
    class_dist.insert("useful_life_manipulation".to_string(), 40);

    BenchmarkBuilder::new("technology-fraud-3k", "Technology-Fraud-3K")
        .description("3K technology sector transactions with SaaS/license revenue and R&D fraud patterns. Tests detection of ASC 606 violations and improper capitalization.")
        .task_type(BenchmarkTaskType::FraudClassification)
        .dataset_size(3000, 300)
        .class_distribution(class_dist)
        .split_ratios(0.7, 0.15, 0.15, true)
        .primary_metric(MetricType::AucPr)
        .metrics(vec![
            MetricType::AucPr,
            MetricType::AucRoc,
            MetricType::MacroF1,
            MetricType::Recall,
        ])
        .seed(44444)
        .time_span_days(730) // 2 years for revenue recognition
        .num_companies(2)
        .add_baseline(BaselineResult {
            model_name: "RevenueRules".to_string(),
            model_type: BaselineModelType::RuleBased,
            metrics: [
                ("auc_pr".to_string(), 0.30),
                ("auc_roc".to_string(), 0.62),
            ].into_iter().collect(),
            training_time_seconds: Some(0.0),
            inference_time_ms: Some(0.5),
            notes: Some("ASC 606 compliance rules".to_string()),
        })
        .add_baseline(BaselineResult {
            model_name: "ContractML".to_string(),
            model_type: BaselineModelType::XgBoost,
            metrics: [
                ("auc_pr".to_string(), 0.48),
                ("auc_roc".to_string(), 0.78),
            ].into_iter().collect(),
            training_time_seconds: Some(3.0),
            inference_time_ms: Some(0.1),
            notes: Some("With contract/performance obligation features".to_string()),
        })
        .add_baseline(BaselineResult {
            model_name: "TemporalLGBM".to_string(),
            model_type: BaselineModelType::LightGbm,
            metrics: [
                ("auc_pr".to_string(), 0.58),
                ("auc_roc".to_string(), 0.85),
            ].into_iter().collect(),
            training_time_seconds: Some(4.0),
            inference_time_ms: Some(0.08),
            notes: Some("With temporal revenue patterns".to_string()),
        })
        .metadata("industry", "technology")
        .metadata("revenue_standards", "asc_606,asc_985")
        .metadata("difficulty", "hard")
        .build()
}

// =============================================================================
// Financial Services Benchmarks
// =============================================================================

/// FinancialServices-Fraud-5K: Financial services fraud detection.
pub fn financial_services_fraud_5k() -> BenchmarkSuite {
    let mut class_dist = HashMap::new();
    class_dist.insert("normal".to_string(), 4500);
    class_dist.insert("loan_fraud".to_string(), 150);
    class_dist.insert("trading_fraud".to_string(), 100);
    class_dist.insert("account_manipulation".to_string(), 80);
    class_dist.insert("insurance_fraud".to_string(), 100);
    class_dist.insert("fee_fraud".to_string(), 70);

    BenchmarkBuilder::new("financial-services-fraud-5k", "FinancialServices-Fraud-5K")
        .description("5K financial services transactions with banking, insurance, and investment fraud patterns. Tests detection under regulatory frameworks (Basel, Solvency, SEC).")
        .task_type(BenchmarkTaskType::FraudClassification)
        .dataset_size(5000, 500)
        .class_distribution(class_dist)
        .split_ratios(0.7, 0.15, 0.15, true)
        .primary_metric(MetricType::AucPr)
        .metrics(vec![
            MetricType::AucPr,
            MetricType::AucRoc,
            MetricType::MacroF1,
            MetricType::Recall,
            MetricType::PrecisionAtK(50),
        ])
        .seed(55555)
        .time_span_days(365)
        .num_companies(1)
        .add_baseline(BaselineResult {
            model_name: "ComplianceRules".to_string(),
            model_type: BaselineModelType::RuleBased,
            metrics: [
                ("auc_pr".to_string(), 0.38),
                ("auc_roc".to_string(), 0.70),
            ].into_iter().collect(),
            training_time_seconds: Some(0.0),
            inference_time_ms: Some(0.5),
            notes: Some("Regulatory compliance rules".to_string()),
        })
        .add_baseline(BaselineResult {
            model_name: "FraudNet".to_string(),
            model_type: BaselineModelType::RandomForest,
            metrics: [
                ("auc_pr".to_string(), 0.55),
                ("auc_roc".to_string(), 0.83),
            ].into_iter().collect(),
            training_time_seconds: Some(5.0),
            inference_time_ms: Some(0.15),
            notes: Some("With account behavior features".to_string()),
        })
        .add_baseline(BaselineResult {
            model_name: "DeepFraud".to_string(),
            model_type: BaselineModelType::NeuralNetwork,
            metrics: [
                ("auc_pr".to_string(), 0.68),
                ("auc_roc".to_string(), 0.91),
            ].into_iter().collect(),
            training_time_seconds: Some(45.0),
            inference_time_ms: Some(3.0),
            notes: Some("LSTM-based sequence model".to_string()),
        })
        .metadata("industry", "financial_services")
        .metadata("regulatory_framework", "basel,sec,finra")
        .metadata("difficulty", "hard")
        .build()
}

/// Get all industry-specific benchmarks.
pub fn all_industry_benchmarks() -> Vec<BenchmarkSuite> {
    vec![
        manufacturing_fraud_5k(),
        retail_fraud_10k(),
        healthcare_fraud_5k(),
        technology_fraud_3k(),
        financial_services_fraud_5k(),
    ]
}

/// Get benchmarks for a specific industry.
pub fn get_industry_benchmark(industry: &str) -> Option<BenchmarkSuite> {
    match industry.to_lowercase().as_str() {
        "manufacturing" => Some(manufacturing_fraud_5k()),
        "retail" => Some(retail_fraud_10k()),
        "healthcare" => Some(healthcare_fraud_5k()),
        "technology" => Some(technology_fraud_3k()),
        "financial_services" | "financialservices" => Some(financial_services_fraud_5k()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manufacturing_fraud_5k() {
        let bench = manufacturing_fraud_5k();
        assert_eq!(bench.id, "manufacturing-fraud-5k");
        assert_eq!(bench.dataset.total_records, 5000);
        assert!(bench
            .dataset
            .class_distribution
            .contains_key("yield_manipulation"));
        assert_eq!(
            bench.metadata.get("industry"),
            Some(&"manufacturing".to_string())
        );
    }

    #[test]
    fn test_retail_fraud_10k() {
        let bench = retail_fraud_10k();
        assert_eq!(bench.id, "retail-fraud-10k");
        assert_eq!(bench.dataset.total_records, 10000);
        assert!(bench
            .dataset
            .class_distribution
            .contains_key("sweethearting"));
        assert!(bench.dataset.class_distribution.contains_key("skimming"));
    }

    #[test]
    fn test_healthcare_fraud_5k() {
        let bench = healthcare_fraud_5k();
        assert_eq!(bench.id, "healthcare-fraud-5k");
        assert!(bench.dataset.class_distribution.contains_key("upcoding"));
        assert!(bench.dataset.class_distribution.contains_key("unbundling"));
        assert!(bench.metadata.get("regulatory_framework").is_some());
    }

    #[test]
    fn test_technology_fraud_3k() {
        let bench = technology_fraud_3k();
        assert_eq!(bench.id, "technology-fraud-3k");
        assert!(bench
            .dataset
            .class_distribution
            .contains_key("premature_revenue"));
        assert!(bench
            .dataset
            .class_distribution
            .contains_key("channel_stuffing"));
    }

    #[test]
    fn test_financial_services_fraud_5k() {
        let bench = financial_services_fraud_5k();
        assert_eq!(bench.id, "financial-services-fraud-5k");
        assert!(bench.dataset.class_distribution.contains_key("loan_fraud"));
        assert!(bench
            .dataset
            .class_distribution
            .contains_key("trading_fraud"));
    }

    #[test]
    fn test_all_industry_benchmarks() {
        let benchmarks = all_industry_benchmarks();
        assert_eq!(benchmarks.len(), 5);

        // All should have industry metadata
        for bench in &benchmarks {
            assert!(bench.metadata.contains_key("industry"));
        }
    }

    #[test]
    fn test_get_industry_benchmark() {
        assert!(get_industry_benchmark("manufacturing").is_some());
        assert!(get_industry_benchmark("retail").is_some());
        assert!(get_industry_benchmark("healthcare").is_some());
        assert!(get_industry_benchmark("technology").is_some());
        assert!(get_industry_benchmark("financial_services").is_some());
        assert!(get_industry_benchmark("unknown").is_none());
    }

    #[test]
    fn test_industry_benchmark_analysis_default() {
        let analysis = IndustryBenchmarkAnalysis::default();
        assert!(analysis.industry.is_empty());
        assert_eq!(analysis.terminology_coverage, 1.0);
        assert!(analysis.issues.is_empty());
    }
}
