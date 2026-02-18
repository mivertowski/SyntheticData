//! Supplier qualification generator.

use chrono::NaiveDate;
use datasynth_config::schema::QualificationConfig;
use datasynth_core::models::sourcing::{
    QualificationScore, QualificationStatus, SupplierCertification, SupplierQualification,
};
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

/// Generates supplier qualification records.
pub struct QualificationGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
    config: QualificationConfig,
}

impl QualificationGenerator {
    /// Create a new qualification generator.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::SupplierQualification),
            config: QualificationConfig::default(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(seed: u64, config: QualificationConfig) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::SupplierQualification),
            config,
        }
    }

    /// Generate qualifications for vendors in a sourcing project.
    pub fn generate(
        &mut self,
        company_code: &str,
        vendor_ids: &[String],
        sourcing_project_id: Option<&str>,
        evaluator_id: &str,
        qualification_date: NaiveDate,
    ) -> Vec<SupplierQualification> {
        let mut qualifications = Vec::new();

        for vendor_id in vendor_ids {
            let criteria = vec![
                ("Financial Stability", self.config.financial_weight, 60.0),
                ("Quality Management", self.config.quality_weight, 65.0),
                ("Delivery Performance", self.config.delivery_weight, 60.0),
                ("Compliance", self.config.compliance_weight, 70.0),
            ];

            let mut scores = Vec::new();
            let mut weighted_total = 0.0;
            let mut all_mandatory_passed = true;

            for (name, weight, min_score) in &criteria {
                let score = self.rng.gen_range(40.0..=100.0);
                let passed = score >= *min_score;
                if !passed {
                    all_mandatory_passed = false;
                }
                weighted_total += score * weight;
                scores.push(QualificationScore {
                    criterion_name: name.to_string(),
                    score,
                    passed,
                    comments: None,
                });
            }

            let status = if !all_mandatory_passed {
                QualificationStatus::Disqualified
            } else if weighted_total >= 75.0 {
                QualificationStatus::Qualified
            } else if weighted_total >= 60.0 {
                QualificationStatus::ConditionallyQualified
            } else {
                QualificationStatus::Disqualified
            };

            let valid_until = if matches!(
                status,
                QualificationStatus::Qualified | QualificationStatus::ConditionallyQualified
            ) {
                Some(qualification_date + chrono::Duration::days(self.config.validity_days as i64))
            } else {
                None
            };

            qualifications.push(SupplierQualification {
                qualification_id: self.uuid_factory.next().to_string(),
                vendor_id: vendor_id.clone(),
                sourcing_project_id: sourcing_project_id.map(|s| s.to_string()),
                company_code: company_code.to_string(),
                status,
                start_date: qualification_date - chrono::Duration::days(14),
                completion_date: Some(qualification_date),
                valid_until,
                scores,
                overall_score: weighted_total,
                evaluator_id: evaluator_id.to_string(),
                certifications: Vec::new(),
                conditions: if matches!(status, QualificationStatus::ConditionallyQualified) {
                    Some("Improvement plan required within 90 days".to_string())
                } else {
                    None
                },
            });
        }

        qualifications
    }

    /// Generate certifications for a vendor.
    pub fn generate_certifications(
        &mut self,
        vendor_id: &str,
        base_date: NaiveDate,
    ) -> Vec<SupplierCertification> {
        let cert_types = [
            ("ISO 9001", "TUV Rheinland"),
            ("ISO 14001", "Bureau Veritas"),
            ("SOC 2 Type II", "Deloitte"),
            ("ISO 27001", "BSI Group"),
        ];

        let count = self.rng.gen_range(0..=3);
        let mut certs = Vec::new();

        for &(cert_type, issuer) in cert_types.iter().take(count) {
            let issue_date = base_date - chrono::Duration::days(self.rng.gen_range(30..=730));
            let expiry_date = issue_date + chrono::Duration::days(365 * 3);

            certs.push(SupplierCertification {
                certification_id: self.uuid_factory.next().to_string(),
                vendor_id: vendor_id.to_string(),
                certification_type: cert_type.to_string(),
                issuing_body: issuer.to_string(),
                issue_date,
                expiry_date,
                is_valid: expiry_date >= base_date,
            });
        }

        certs
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn test_vendor_ids() -> Vec<String> {
        vec!["V001".to_string(), "V002".to_string(), "V003".to_string()]
    }

    #[test]
    fn test_basic_generation() {
        let mut gen = QualificationGenerator::new(42);
        let date = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let results = gen.generate("C001", &test_vendor_ids(), Some("SP-001"), "EVAL-01", date);

        assert_eq!(results.len(), 3);
        for qual in &results {
            assert_eq!(qual.company_code, "C001");
            assert!(!qual.qualification_id.is_empty());
            assert!(!qual.vendor_id.is_empty());
            assert_eq!(qual.evaluator_id, "EVAL-01");
            assert_eq!(qual.sourcing_project_id.as_deref(), Some("SP-001"));
            assert!(!qual.scores.is_empty());
            assert_eq!(qual.scores.len(), 4); // 4 criteria
            assert!(qual.overall_score > 0.0);
        }
    }

    #[test]
    fn test_deterministic() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let vendors = test_vendor_ids();

        let mut gen1 = QualificationGenerator::new(42);
        let mut gen2 = QualificationGenerator::new(42);

        let r1 = gen1.generate("C001", &vendors, Some("SP-001"), "EVAL-01", date);
        let r2 = gen2.generate("C001", &vendors, Some("SP-001"), "EVAL-01", date);

        assert_eq!(r1.len(), r2.len());
        for (a, b) in r1.iter().zip(r2.iter()) {
            assert_eq!(a.qualification_id, b.qualification_id);
            assert_eq!(a.vendor_id, b.vendor_id);
            assert_eq!(a.overall_score, b.overall_score);
            assert_eq!(a.status, b.status);
        }
    }

    #[test]
    fn test_field_constraints() {
        let mut gen = QualificationGenerator::new(99);
        let date = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let results = gen.generate("C001", &test_vendor_ids(), None, "EVAL-01", date);

        for qual in &results {
            // All scores should be between 40 and 100
            for score in &qual.scores {
                assert!(score.score >= 40.0 && score.score <= 100.0);
                assert!(!score.criterion_name.is_empty());
            }

            // Qualified or conditionally qualified should have valid_until
            match qual.status {
                QualificationStatus::Qualified | QualificationStatus::ConditionallyQualified => {
                    assert!(qual.valid_until.is_some());
                }
                QualificationStatus::Disqualified => {
                    assert!(qual.valid_until.is_none());
                }
                _ => {}
            }

            // Start date should be 14 days before completion
            assert_eq!(qual.start_date, date - chrono::Duration::days(14));
            assert_eq!(qual.completion_date, Some(date));
        }
    }

    #[test]
    fn test_generate_certifications() {
        let mut gen = QualificationGenerator::new(42);
        let date = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let certs = gen.generate_certifications("V001", date);

        // Could be 0-3 certs
        assert!(certs.len() <= 3);
        for cert in &certs {
            assert_eq!(cert.vendor_id, "V001");
            assert!(!cert.certification_id.is_empty());
            assert!(!cert.certification_type.is_empty());
            assert!(!cert.issuing_body.is_empty());
            assert!(cert.expiry_date > cert.issue_date);
        }
    }
}
