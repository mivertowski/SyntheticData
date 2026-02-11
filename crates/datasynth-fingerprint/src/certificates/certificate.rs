//! Synthetic data certificates for proving privacy guarantees.
//!
//! A [`SyntheticDataCertificate`] provides cryptographic attestation of
//! differential privacy parameters and data quality metrics for a synthetic
//! dataset. Certificates can be signed and verified using SHA256 HMAC-based
//! signing to ensure integrity and provenance.
//!
//! # Example
//!
//! ```
//! use datasynth_fingerprint::certificates::{
//!     CertificateBuilder, DpGuarantee, QualityMetrics,
//!     sign_certificate, verify_certificate,
//! };
//!
//! let mut cert = CertificateBuilder::new("DataSynth")
//!     .with_dp_guarantee(DpGuarantee {
//!         mechanism: "Laplace".to_string(),
//!         epsilon: 1.0,
//!         delta: None,
//!         composition_method: "naive".to_string(),
//!         total_queries: 50,
//!     })
//!     .with_quality_metrics(QualityMetrics {
//!         benford_mad: Some(0.012),
//!         correlation_preservation: Some(0.95),
//!         statistical_fidelity: Some(0.92),
//!         mia_auc: Some(0.52),
//!     })
//!     .with_config_hash("abc123def456")
//!     .with_seed(42)
//!     .build();
//!
//! sign_certificate(&mut cert, "my-secret-key");
//! assert!(verify_certificate(&cert, "my-secret-key"));
//! assert!(!verify_certificate(&cert, "wrong-key"));
//! ```

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Differential privacy guarantee parameters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DpGuarantee {
    /// The DP mechanism used (e.g., "Laplace", "Gaussian", "Exponential").
    pub mechanism: String,
    /// Privacy budget epsilon.
    pub epsilon: f64,
    /// Optional delta parameter for approximate DP.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<f64>,
    /// Composition method used (e.g., "naive", "renyi_dp", "zcdp").
    pub composition_method: String,
    /// Total number of queries/mechanisms applied.
    pub total_queries: u32,
}

impl Default for DpGuarantee {
    fn default() -> Self {
        Self {
            mechanism: "Laplace".to_string(),
            epsilon: 1.0,
            delta: None,
            composition_method: "naive".to_string(),
            total_queries: 0,
        }
    }
}

/// Quality metrics for the synthetic data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct QualityMetrics {
    /// Mean Absolute Deviation from Benford's Law first-digit distribution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub benford_mad: Option<f64>,
    /// How well correlations are preserved (0.0 = none, 1.0 = perfect).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_preservation: Option<f64>,
    /// Overall statistical fidelity score (0.0 = poor, 1.0 = perfect).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statistical_fidelity: Option<f64>,
    /// Membership Inference Attack AUC (closer to 0.5 = better privacy).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mia_auc: Option<f64>,
}

/// A certificate attesting to the privacy guarantees and quality of synthetic data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticDataCertificate {
    /// Unique certificate identifier.
    pub certificate_id: String,
    /// ISO 8601 timestamp of certificate generation.
    pub generation_timestamp: String,
    /// Version of the generator that produced the data.
    pub generator_version: String,
    /// Hash of the generation configuration.
    pub config_hash: String,
    /// RNG seed used for generation, if deterministic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    /// Differential privacy guarantee parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dp_guarantee: Option<DpGuarantee>,
    /// Quality metrics for the generated data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality_metrics: Option<QualityMetrics>,
    /// Hash of the source fingerprint file, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint_hash: Option<String>,
    /// Entity that issued this certificate.
    pub issuer: String,
    /// HMAC-SHA256 signature over the certificate content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

/// Builder for constructing [`SyntheticDataCertificate`] instances.
pub struct CertificateBuilder {
    issuer: String,
    dp_guarantee: Option<DpGuarantee>,
    quality_metrics: Option<QualityMetrics>,
    config_hash: Option<String>,
    seed: Option<u64>,
    fingerprint_hash: Option<String>,
    generator_version: Option<String>,
}

impl CertificateBuilder {
    /// Create a new builder with the given issuer name.
    pub fn new(issuer: impl Into<String>) -> Self {
        Self {
            issuer: issuer.into(),
            dp_guarantee: None,
            quality_metrics: None,
            config_hash: None,
            seed: None,
            fingerprint_hash: None,
            generator_version: None,
        }
    }

    /// Set the differential privacy guarantee.
    pub fn with_dp_guarantee(mut self, dp: DpGuarantee) -> Self {
        self.dp_guarantee = Some(dp);
        self
    }

    /// Set the quality metrics.
    pub fn with_quality_metrics(mut self, metrics: QualityMetrics) -> Self {
        self.quality_metrics = Some(metrics);
        self
    }

    /// Set the configuration hash.
    pub fn with_config_hash(mut self, hash: impl Into<String>) -> Self {
        self.config_hash = Some(hash.into());
        self
    }

    /// Set the RNG seed.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Set the fingerprint hash.
    pub fn with_fingerprint_hash(mut self, hash: impl Into<String>) -> Self {
        self.fingerprint_hash = Some(hash.into());
        self
    }

    /// Set the generator version string.
    pub fn with_generator_version(mut self, version: impl Into<String>) -> Self {
        self.generator_version = Some(version.into());
        self
    }

    /// Build the certificate.
    ///
    /// Generates a unique certificate ID (UUID v4) and timestamps the
    /// certificate with the current UTC time.
    pub fn build(self) -> SyntheticDataCertificate {
        SyntheticDataCertificate {
            certificate_id: uuid::Uuid::new_v4().to_string(),
            generation_timestamp: chrono::Utc::now().to_rfc3339(),
            generator_version: self
                .generator_version
                .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string()),
            config_hash: self.config_hash.unwrap_or_default(),
            seed: self.seed,
            dp_guarantee: self.dp_guarantee,
            quality_metrics: self.quality_metrics,
            fingerprint_hash: self.fingerprint_hash,
            issuer: self.issuer,
            signature: None,
        }
    }
}

/// Compute the signable content of a certificate.
///
/// This creates a canonical representation by serializing all fields
/// except the signature to JSON, then hashing with the key material.
fn signable_content(certificate: &SyntheticDataCertificate) -> String {
    // Create a copy without signature for canonical content
    let mut cert_copy = certificate.clone();
    cert_copy.signature = None;
    // serde_json::to_string is deterministic for structs (field order is declaration order)
    serde_json::to_string(&cert_copy).unwrap_or_default()
}

/// Sign a certificate using HMAC-SHA256 with the given key material.
///
/// The signature is stored in the certificate's `signature` field as a
/// hex-encoded string. The signature covers all other fields of the
/// certificate.
pub fn sign_certificate(certificate: &mut SyntheticDataCertificate, key_material: &str) {
    let content = signable_content(certificate);
    let signature = hmac_sha256(content.as_bytes(), key_material.as_bytes());
    certificate.signature = Some(hex::encode(signature));
}

/// Verify a certificate's signature against the given key material.
///
/// Returns `true` if the certificate has a valid signature that matches
/// the key material. Returns `false` if the certificate has no signature
/// or if the signature does not match.
pub fn verify_certificate(certificate: &SyntheticDataCertificate, key_material: &str) -> bool {
    let stored_sig = match &certificate.signature {
        Some(s) => s.clone(),
        None => return false,
    };

    let content = signable_content(certificate);
    let expected = hmac_sha256(content.as_bytes(), key_material.as_bytes());
    let expected_hex = hex::encode(expected);

    // Constant-time comparison
    if stored_sig.len() != expected_hex.len() {
        return false;
    }
    let mut diff = 0u8;
    for (a, b) in stored_sig.bytes().zip(expected_hex.bytes()) {
        diff |= a ^ b;
    }
    diff == 0
}

/// Compute HMAC-SHA256.
fn hmac_sha256(data: &[u8], key: &[u8]) -> Vec<u8> {
    let block_size = 64;
    let mut key_padded = key.to_vec();

    // If key is longer than block size, hash it
    if key_padded.len() > block_size {
        let mut hasher = Sha256::new();
        hasher.update(&key_padded);
        key_padded = hasher.finalize().to_vec();
    }

    // Pad key to block size
    key_padded.resize(block_size, 0);

    // Inner padding
    let ipad: Vec<u8> = key_padded.iter().map(|b| b ^ 0x36).collect();
    // Outer padding
    let opad: Vec<u8> = key_padded.iter().map(|b| b ^ 0x5c).collect();

    // Inner hash: H(ipad || data)
    let mut inner_hasher = Sha256::new();
    inner_hasher.update(&ipad);
    inner_hasher.update(data);
    let inner_hash = inner_hasher.finalize();

    // Outer hash: H(opad || inner_hash)
    let mut outer_hasher = Sha256::new();
    outer_hasher.update(&opad);
    outer_hasher.update(inner_hash);
    outer_hasher.finalize().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_creates_valid_certificate() {
        let cert = CertificateBuilder::new("TestOrg")
            .with_dp_guarantee(DpGuarantee {
                mechanism: "Laplace".to_string(),
                epsilon: 1.0,
                delta: Some(1e-5),
                composition_method: "naive".to_string(),
                total_queries: 100,
            })
            .with_quality_metrics(QualityMetrics {
                benford_mad: Some(0.01),
                correlation_preservation: Some(0.95),
                statistical_fidelity: Some(0.90),
                mia_auc: Some(0.52),
            })
            .with_config_hash("deadbeef")
            .with_seed(42)
            .build();

        assert!(!cert.certificate_id.is_empty());
        assert!(!cert.generation_timestamp.is_empty());
        assert_eq!(cert.issuer, "TestOrg");
        assert_eq!(cert.config_hash, "deadbeef");
        assert_eq!(cert.seed, Some(42));
        assert!(cert.dp_guarantee.is_some());
        assert!(cert.quality_metrics.is_some());
        assert!(cert.signature.is_none()); // Not signed yet

        let dp = cert.dp_guarantee.as_ref().expect("dp_guarantee present");
        assert_eq!(dp.mechanism, "Laplace");
        assert!((dp.epsilon - 1.0).abs() < 1e-10);
        assert_eq!(dp.total_queries, 100);
    }

    #[test]
    fn test_serde_roundtrip() {
        let cert = CertificateBuilder::new("TestOrg")
            .with_dp_guarantee(DpGuarantee::default())
            .with_quality_metrics(QualityMetrics {
                benford_mad: Some(0.015),
                ..QualityMetrics::default()
            })
            .with_seed(123)
            .build();

        let json = serde_json::to_string(&cert).expect("serialize");
        let deserialized: SyntheticDataCertificate =
            serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.certificate_id, cert.certificate_id);
        assert_eq!(deserialized.issuer, cert.issuer);
        assert_eq!(deserialized.seed, cert.seed);
        assert_eq!(deserialized.dp_guarantee, cert.dp_guarantee);
    }

    #[test]
    fn test_sign_and_verify_passes() {
        let mut cert = CertificateBuilder::new("TestOrg")
            .with_dp_guarantee(DpGuarantee {
                mechanism: "Gaussian".to_string(),
                epsilon: 0.5,
                delta: Some(1e-6),
                composition_method: "renyi_dp".to_string(),
                total_queries: 200,
            })
            .with_config_hash("cafebabe")
            .build();

        sign_certificate(&mut cert, "super-secret-key-2024");

        assert!(cert.signature.is_some());
        assert!(verify_certificate(&cert, "super-secret-key-2024"));
    }

    #[test]
    fn test_wrong_key_fails_verification() {
        let mut cert = CertificateBuilder::new("TestOrg")
            .with_dp_guarantee(DpGuarantee::default())
            .build();

        sign_certificate(&mut cert, "correct-key");

        assert!(!verify_certificate(&cert, "wrong-key"));
    }

    #[test]
    fn test_unsigned_certificate_fails_verification() {
        let cert = CertificateBuilder::new("TestOrg")
            .with_dp_guarantee(DpGuarantee::default())
            .build();

        // No signature set
        assert!(!verify_certificate(&cert, "any-key"));
    }

    #[test]
    fn test_tampered_certificate_fails_verification() {
        let mut cert = CertificateBuilder::new("TestOrg")
            .with_dp_guarantee(DpGuarantee {
                mechanism: "Laplace".to_string(),
                epsilon: 1.0,
                delta: None,
                composition_method: "naive".to_string(),
                total_queries: 50,
            })
            .build();

        sign_certificate(&mut cert, "secret");
        assert!(verify_certificate(&cert, "secret"));

        // Tamper with the certificate
        cert.issuer = "EvilOrg".to_string();
        assert!(!verify_certificate(&cert, "secret"));
    }

    #[test]
    fn test_builder_defaults() {
        let cert = CertificateBuilder::new("MinimalOrg").build();

        assert_eq!(cert.issuer, "MinimalOrg");
        assert!(cert.dp_guarantee.is_none());
        assert!(cert.quality_metrics.is_none());
        assert!(cert.seed.is_none());
        assert!(cert.fingerprint_hash.is_none());
        assert!(cert.signature.is_none());
        assert!(cert.config_hash.is_empty());
    }

    #[test]
    fn test_fingerprint_hash_in_certificate() {
        let cert = CertificateBuilder::new("Org")
            .with_fingerprint_hash("sha256:abcdef0123456789")
            .build();

        assert_eq!(
            cert.fingerprint_hash,
            Some("sha256:abcdef0123456789".to_string())
        );
    }
}
