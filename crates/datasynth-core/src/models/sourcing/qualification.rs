//! Supplier qualification models for vendor evaluation and certification.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Status of supplier qualification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum QualificationStatus {
    /// Qualification not started
    #[default]
    Pending,
    /// Qualification in progress
    InProgress,
    /// Supplier qualified (passed)
    Qualified,
    /// Conditionally qualified (with restrictions)
    ConditionallyQualified,
    /// Supplier disqualified (failed)
    Disqualified,
    /// Qualification expired
    Expired,
}

/// A criterion used for qualification scoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualificationCriterion {
    /// Criterion name (e.g., "Financial Stability", "Quality Management")
    pub name: String,
    /// Weight in total score (0.0 to 1.0)
    pub weight: f64,
    /// Minimum passing score (0.0 to 100.0)
    pub min_score: f64,
    /// Whether this criterion is mandatory (failing = disqualified)
    pub is_mandatory: bool,
}

/// Score for a single qualification criterion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualificationScore {
    /// Criterion name
    pub criterion_name: String,
    /// Score achieved (0.0 to 100.0)
    pub score: f64,
    /// Whether the criterion was passed
    pub passed: bool,
    /// Evaluator comments
    pub comments: Option<String>,
}

/// A supplier certification record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierCertification {
    /// Certification ID
    pub certification_id: String,
    /// Vendor ID
    pub vendor_id: String,
    /// Certification type (e.g., "ISO 9001", "ISO 14001", "SOC 2")
    pub certification_type: String,
    /// Issuing body
    pub issuing_body: String,
    /// Issue date
    pub issue_date: NaiveDate,
    /// Expiry date
    pub expiry_date: NaiveDate,
    /// Is the certification currently valid
    pub is_valid: bool,
}

/// Supplier qualification record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierQualification {
    /// Unique qualification ID
    pub qualification_id: String,
    /// Vendor ID being qualified
    pub vendor_id: String,
    /// Sourcing project ID (if applicable)
    pub sourcing_project_id: Option<String>,
    /// Company code
    pub company_code: String,
    /// Qualification status
    pub status: QualificationStatus,
    /// Qualification start date
    pub start_date: NaiveDate,
    /// Qualification completion date
    pub completion_date: Option<NaiveDate>,
    /// Validity period end date
    pub valid_until: Option<NaiveDate>,
    /// Individual criterion scores
    pub scores: Vec<QualificationScore>,
    /// Overall weighted score
    pub overall_score: f64,
    /// Evaluator ID
    pub evaluator_id: String,
    /// Certifications provided
    pub certifications: Vec<String>,
    /// Conditions or restrictions (if conditionally qualified)
    pub conditions: Option<String>,
}
