#![deny(clippy::unwrap_used)]
//! # synth-banking
//!
//! KYC/AML banking transaction generator for synthetic data.
//!
//! This crate provides comprehensive banking transaction simulation for:
//! - Compliance testing and model training
//! - AML/fraud detection system evaluation
//! - KYC process simulation
//! - Regulatory reporting testing
//!
//! ## Features
//!
//! - **Customer Generation**: Retail, business, and trust customers with realistic KYC profiles
//! - **Account Generation**: Multiple account types with proper feature sets
//! - **Transaction Engine**: Persona-based transaction generation with causal drivers
//! - **AML Typologies**: Structuring, funnel accounts, layering, mule networks, and more
//! - **Ground Truth Labels**: Multi-level labels for ML training
//! - **Spoofing Mode**: Adversarial transaction generation for robustness testing
//!
//! ## Architecture
//!
//! The crate follows a layered architecture:
//!
//! ```text
//! BankingOrchestrator (orchestration)
//!         ↓
//! Generators (customer, account, transaction, counterparty)
//!         ↓
//! Typologies (AML pattern injection)
//!         ↓
//! Labels (ground truth generation)
//!         ↓
//! Models (customer, account, transaction, KYC)
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use datasynth_banking::{BankingOrchestrator, BankingConfig};
//!
//! let config = BankingConfig::default();
//! let mut orchestrator = BankingOrchestrator::new(config, 12345);
//!
//! // Generate customers and accounts
//! let customers = orchestrator.generate_customers();
//! let accounts = orchestrator.generate_accounts(&customers);
//!
//! // Generate transaction stream
//! let transactions = orchestrator.generate_transactions(&accounts);
//! ```

pub mod generators;
pub mod labels;
pub mod models;
pub mod personas;
pub mod seed_offsets;
pub mod typologies;

mod config;
mod orchestrator;

/// Parse a start date string in YYYY-MM-DD format, logging a warning and
/// falling back to 2024-01-01 when the string is malformed.
pub(crate) fn parse_start_date(date_str: &str) -> chrono::NaiveDate {
    chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap_or_else(|e| {
        tracing::warn!(
            "Failed to parse start_date '{}': {}. Defaulting to 2024-01-01",
            date_str,
            e
        );
        chrono::NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid date")
    })
}

pub use config::*;
pub use orchestrator::*;

// Re-export key types for convenience
pub use datasynth_core::models::banking::{
    AmlTypology, BankAccountType, BankingCustomerType, Direction, LaunderingStage,
    MerchantCategoryCode, RiskTier, Sophistication, TransactionCategory, TransactionChannel,
};
pub use models::{BankAccount, BankTransaction, BankingCustomer, CounterpartyPool, KycProfile};
