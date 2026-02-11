//! Regulatory compliance module for EU AI Act, GDPR, and related frameworks.
//!
//! Provides content marking for synthetic data (Article 50), data governance
//! reporting (Article 10), and compliance metadata generation.

pub mod article10;
pub mod content_marking;

pub use article10::{DataGovernanceReport, ProcessingStep, QualityMeasure};
pub use content_marking::{ContentCredential, MarkingConfig, MarkingFormat, SyntheticContentMarker};
