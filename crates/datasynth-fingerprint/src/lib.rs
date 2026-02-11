// Allow some clippy lints that are common in numerical/matrix code
#![allow(clippy::needless_range_loop)]
#![allow(clippy::explicit_counter_loop)]
#![deny(clippy::unwrap_used)]

//! DataSynth Fingerprint - Privacy-preserving synthetic data fingerprinting.
//!
//! This crate provides functionality for:
//! - **Extracting** statistical fingerprints from real data
//! - **Applying privacy** mechanisms (differential privacy, k-anonymity)
//! - **Storing** fingerprints in `.dsf` files
//! - **Synthesizing** generator configurations from fingerprints
//! - **Evaluating** fidelity of generated data
//!
//! # Overview
//!
//! A fingerprint captures the statistical properties of a dataset without storing
//! any individual records, enabling privacy-preserving synthetic data generation.
//!
//! ```text
//! Real Data → Extract → .dsf File → Generate → Synthetic Data → Evaluate
//! ```
//!
//! # Quick Start
//!
//! ## Basic Extraction and Storage
//!
//! ```ignore
//! use datasynth_fingerprint::{
//!     extraction::{FingerprintExtractor, ExtractionConfig},
//!     io::{FingerprintReader, FingerprintWriter},
//!     models::PrivacyLevel,
//! };
//! use std::path::Path;
//!
//! // Extract fingerprint from CSV data with standard privacy
//! let extractor = FingerprintExtractor::new(PrivacyLevel::Standard);
//! let fingerprint = extractor.extract_from_csv(Path::new("data.csv"))?;
//!
//! // Write to .dsf file
//! let writer = FingerprintWriter::new();
//! writer.write_to_file(&fingerprint, Path::new("output.dsf"))?;
//!
//! // Read back from .dsf file
//! let reader = FingerprintReader::new();
//! let loaded = reader.read_from_file(Path::new("output.dsf"))?;
//!
//! // Check privacy audit
//! println!("Epsilon spent: {}", loaded.epsilon_spent());
//! ```
//!
//! ## Signed Fingerprints
//!
//! ```ignore
//! use datasynth_fingerprint::io::{SigningKey, DsfSigner, DsfVerifier};
//!
//! // Generate a signing key
//! let key = SigningKey::generate("my-org-key");
//!
//! // Sign when writing
//! let signer = DsfSigner::new(key.clone());
//! writer.write_to_file_signed(&fingerprint, Path::new("signed.dsf"), &signer)?;
//!
//! // Verify when reading
//! let verifier = DsfVerifier::new(key);
//! let verified = reader.read_from_file_verified(Path::new("signed.dsf"), &verifier)?;
//! ```
//!
//! ## Streaming Extraction for Large Files
//!
//! ```ignore
//! use datasynth_fingerprint::extraction::{FingerprintExtractor, ExtractionConfig};
//!
//! // Configure for streaming (memory-efficient for large files)
//! let config = ExtractionConfig {
//!     streaming: true,
//!     stream_batch_size: 100_000,
//!     ..ExtractionConfig::default()
//! };
//!
//! let extractor = FingerprintExtractor::with_config(config);
//! let fingerprint = extractor.extract_streaming_csv(Path::new("large_data.csv"))?;
//! ```
//!
//! ## Config Synthesis
//!
//! ```ignore
//! use datasynth_fingerprint::synthesis::{ConfigSynthesizer, SynthesisOptions};
//!
//! let options = SynthesisOptions {
//!     scale: 2.0,              // Generate 2x original row count
//!     seed: Some(42),          // Reproducible generation
//!     preserve_correlations: true,
//!     inject_anomalies: true,
//! };
//!
//! let synthesizer = ConfigSynthesizer::with_options(options);
//! let result = synthesizer.synthesize_full(&fingerprint, 42)?;
//!
//! // result.config_patch - configuration values for generators
//! // result.copula_generators - for preserving correlations
//! ```
//!
//! ## Fidelity Evaluation
//!
//! ```ignore
//! use datasynth_fingerprint::evaluation::FidelityEvaluator;
//!
//! let evaluator = FidelityEvaluator::new();
//! let report = evaluator.evaluate(&original_fingerprint, &synthetic_fingerprint)?;
//!
//! println!("Overall fidelity: {:.2}", report.overall_score);
//! println!("Statistical fidelity: {:.2}", report.statistical_fidelity);
//! println!("Correlation fidelity: {:.2}", report.correlation_fidelity);
//! ```
//!
//! # DSF File Format
//!
//! A `.dsf` (DataSynth Fingerprint) file is a ZIP archive containing:
//!
//! | File | Format | Description |
//! |------|--------|-------------|
//! | `manifest.json` | JSON | Version, checksums, privacy config, optional signature |
//! | `schema.yaml` | YAML | Tables, columns, types, relationships |
//! | `statistics.yaml` | YAML | Distributions, percentiles, Benford analysis |
//! | `correlations.yaml` | YAML | Correlation matrices, copulas (optional) |
//! | `integrity.yaml` | YAML | FK relationships, cardinality (optional) |
//! | `rules.yaml` | YAML | Balance constraints, approval thresholds (optional) |
//! | `anomalies.yaml` | YAML | Anomaly rates, type distribution (optional) |
//! | `privacy_audit.json` | JSON | Privacy decisions, epsilon spent |
//!
//! # Privacy Levels
//!
//! The crate supports four privacy levels with different tradeoffs:
//!
//! | Level | Epsilon | K | Description |
//! |-------|---------|---|-------------|
//! | [`PrivacyLevel::Minimal`] | 5.0 | 3 | Low privacy, high utility |
//! | [`PrivacyLevel::Standard`] | 1.0 | 5 | Balanced (default) |
//! | [`PrivacyLevel::High`] | 0.5 | 10 | Higher privacy for sensitive data |
//! | [`PrivacyLevel::Maximum`] | 0.1 | 20 | Maximum privacy, reduced utility |
//!
//! # Privacy Mechanisms
//!
//! The fingerprinting process applies multiple privacy mechanisms:
//!
//! - **Differential Privacy**: Laplace noise calibrated to the sensitivity of each statistic,
//!   with configurable epsilon budget. Privacy is enforced through composition tracking.
//!
//! - **K-Anonymity**: Categorical values appearing fewer than k times are suppressed to
//!   prevent re-identification of rare values.
//!
//! - **Outlier Handling**: Extreme values are winsorized at configurable percentiles to
//!   prevent leakage of unusual records.
//!
//! - **Privacy Audit Trail**: Every privacy decision (noise addition, suppression,
//!   generalization) is logged in the fingerprint's `privacy_audit` field.
//!
//! # Supported Data Sources
//!
//! | Source | Method | Notes |
//! |--------|--------|-------|
//! | CSV | `extract_from_csv()` | Auto-infers column types |
//! | Parquet | `extract_from_parquet()` | Preserves type information |
//! | JSON/JSONL | `extract_from_json()` | Array or newline-delimited |
//! | Directory | `extract_from_directory()` | Multi-table fingerprints |
//! | Memory | `DataSource::Memory` | For in-memory data |
//!
//! # Module Overview
//!
//! ## [`models`] - Data Structures
//!
//! Core data structures for fingerprints:
//! - [`Fingerprint`] - Root structure containing all components
//! - [`SchemaFingerprint`] - Table schemas, column types, relationships
//! - [`StatisticsFingerprint`] - Distribution parameters, percentiles
//! - [`CorrelationFingerprint`] - Correlation matrices, Gaussian copulas
//! - [`PrivacyAudit`] - Privacy action tracking
//!
//! ## [`io`] - File I/O
//!
//! Reading and writing `.dsf` files:
//! - [`FingerprintWriter`] - Write fingerprints to `.dsf` files
//! - [`FingerprintReader`] - Read fingerprints from `.dsf` files
//! - [`FingerprintValidator`] - Validate `.dsf` file integrity
//! - [`SigningKey`], [`DsfSigner`], [`DsfVerifier`] - Digital signatures
//!
//! ## [`extraction`] - Data Extraction
//!
//! Extract fingerprints from data sources:
//! - [`FingerprintExtractor`] - Main extraction coordinator
//! - [`DataSource`] - Input data source types
//! - [`ExtractionConfig`] - Extraction settings
//! - Streaming extraction for large files
//!
//! ## [`privacy`] - Privacy Mechanisms
//!
//! Privacy-preserving transformations:
//! - Laplace noise for differential privacy
//! - K-anonymity suppression
//! - Privacy budget tracking
//!
//! ## [`synthesis`] - Config Synthesis
//!
//! Convert fingerprints to generator configurations:
//! - [`ConfigSynthesizer`] - Synthesis coordinator
//! - [`ConfigPatch`] - Configuration values to apply
//! - Gaussian copula generation for correlations
//!
//! ## [`evaluation`] - Fidelity Evaluation
//!
//! Evaluate synthetic data quality:
//! - [`FidelityEvaluator`] - Comparison engine
//! - Statistical, correlation, and schema metrics
//!
//! # CLI Integration
//!
//! The fingerprint crate integrates with the `datasynth-data` CLI:
//!
//! ```bash
//! # Extract fingerprint from data
//! datasynth-data fingerprint extract \
//!     --input ./real_data/ \
//!     --output ./fingerprint.dsf \
//!     --privacy-level standard
//!
//! # Validate fingerprint file
//! datasynth-data fingerprint validate ./fingerprint.dsf
//!
//! # Generate from fingerprint
//! datasynth-data generate \
//!     --fingerprint ./fingerprint.dsf \
//!     --output ./synthetic/ \
//!     --scale 1.0
//!
//! # Evaluate fidelity
//! datasynth-data fingerprint evaluate \
//!     --fingerprint ./fingerprint.dsf \
//!     --synthetic ./synthetic/
//! ```
//!
//! [`Fingerprint`]: models::Fingerprint
//! [`SchemaFingerprint`]: models::SchemaFingerprint
//! [`StatisticsFingerprint`]: models::StatisticsFingerprint
//! [`CorrelationFingerprint`]: models::CorrelationFingerprint
//! [`PrivacyAudit`]: models::PrivacyAudit
//! [`FingerprintWriter`]: io::FingerprintWriter
//! [`FingerprintReader`]: io::FingerprintReader
//! [`FingerprintValidator`]: io::FingerprintValidator
//! [`SigningKey`]: io::SigningKey
//! [`DsfSigner`]: io::DsfSigner
//! [`DsfVerifier`]: io::DsfVerifier
//! [`FingerprintExtractor`]: extraction::FingerprintExtractor
//! [`DataSource`]: extraction::DataSource
//! [`ExtractionConfig`]: extraction::ExtractionConfig
//! [`ConfigSynthesizer`]: synthesis::ConfigSynthesizer
//! [`ConfigPatch`]: synthesis::ConfigPatch
//! [`FidelityEvaluator`]: evaluation::FidelityEvaluator
//! [`PrivacyLevel::Minimal`]: models::PrivacyLevel::Minimal
//! [`PrivacyLevel::Standard`]: models::PrivacyLevel::Standard
//! [`PrivacyLevel::High`]: models::PrivacyLevel::High
//! [`PrivacyLevel::Maximum`]: models::PrivacyLevel::Maximum

pub mod error;
pub mod evaluation;
pub mod extraction;
pub mod io;
pub mod models;
pub mod privacy;
pub mod synthesis;

// Re-export commonly used types
pub use error::{FingerprintError, FingerprintResult};
pub use io::{FingerprintReader, FingerprintValidator, FingerprintWriter};
pub use models::{Fingerprint, Manifest, PrivacyLevel, PrivacyMetadata, SchemaFingerprint};
