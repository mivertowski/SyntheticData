//! Named seed offsets for deterministic sub-generator isolation.
//!
//! Each sub-generator in the banking module uses a distinct seed offset
//! so that generators produce independent, non-overlapping random sequences
//! even when initialized from the same base seed.  Using named constants
//! instead of bare numeric literals makes the purpose of each offset clear
//! and prevents accidental collisions when adding new generators.

// -- Generators ---------------------------------------------------------------

/// Seed offset for [`AccountGenerator`](crate::generators::AccountGenerator).
pub const ACCOUNT_GENERATOR_SEED_OFFSET: u64 = 1000;

/// Seed offset for [`TransactionGenerator`](crate::generators::TransactionGenerator).
pub const TRANSACTION_GENERATOR_SEED_OFFSET: u64 = 2000;

/// Seed offset for [`CounterpartyGenerator`](crate::generators::CounterpartyGenerator).
pub const COUNTERPARTY_GENERATOR_SEED_OFFSET: u64 = 3000;

/// Seed offset for [`KycGenerator`](crate::generators::KycGenerator).
pub const KYC_GENERATOR_SEED_OFFSET: u64 = 4000;

// -- Typologies ---------------------------------------------------------------

/// Seed offset for [`TypologyInjector`](crate::typologies::TypologyInjector).
pub const TYPOLOGY_INJECTOR_SEED_OFFSET: u64 = 5000;

/// Seed offset for [`StructuringInjector`](crate::typologies::StructuringInjector).
pub const STRUCTURING_INJECTOR_SEED_OFFSET: u64 = 6000;

/// Seed offset for [`FunnelInjector`](crate::typologies::FunnelInjector).
pub const FUNNEL_INJECTOR_SEED_OFFSET: u64 = 6100;

/// Seed offset for [`LayeringInjector`](crate::typologies::LayeringInjector).
pub const LAYERING_INJECTOR_SEED_OFFSET: u64 = 6200;

/// Seed offset for [`MuleInjector`](crate::typologies::MuleInjector).
pub const MULE_INJECTOR_SEED_OFFSET: u64 = 6300;

/// Seed offset for [`SpoofingEngine`](crate::typologies::SpoofingEngine).
pub const SPOOFING_ENGINE_SEED_OFFSET: u64 = 6400;

// -- Labels -------------------------------------------------------------------

/// Seed offset for [`NarrativeGenerator`](crate::labels::NarrativeGenerator).
pub const NARRATIVE_GENERATOR_SEED_OFFSET: u64 = 7000;

// -- Extended typologies ------------------------------------------------------

/// Seed offset for [`RoundTrippingInjector`](crate::typologies::RoundTrippingInjector).
pub const ROUND_TRIPPING_INJECTOR_SEED_OFFSET: u64 = 7100;

/// Seed offset for [`FraudInjector`](crate::typologies::FraudInjector).
pub const FRAUD_INJECTOR_SEED_OFFSET: u64 = 7200;
