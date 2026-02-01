//! Healthcare industry transaction generation.
//!
//! Provides healthcare-specific:
//! - Revenue cycle transactions (charges, claims, payments)
//! - Clinical transactions (procedures, diagnoses)
//! - Master data (ICD-10, CPT, DRG codes, payers)
//! - Anomalies (upcoding, unbundling, kickbacks)

mod anomalies;
mod settings;
mod transactions;

pub use anomalies::HealthcareAnomaly;
pub use settings::HealthcareSettings;
pub use transactions::{
    Adjustment, Charge, ClinicalTransaction, CodingSystem, HealthcareTransaction,
    HealthcareTransactionGenerator, PayerType, RevenueCycleTransaction,
};
