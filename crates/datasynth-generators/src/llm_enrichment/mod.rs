//! LLM metadata enrichment for synthetic data generation.
//!
//! This module uses the `LlmProvider` trait from `datasynth-core` to generate
//! realistic vendor names, transaction descriptions, memo fields, and anomaly
//! explanations. Each enricher falls back to deterministic templates when the
//! LLM provider returns an error.

pub mod anomaly_explainer;
pub mod transaction_enricher;
pub mod vendor_enricher;

pub use anomaly_explainer::AnomalyLlmExplainer;
pub use transaction_enricher::TransactionLlmEnricher;
pub use vendor_enricher::VendorLlmEnricher;
