//! Digital signing and verification for DSF files.
//!
//! This module provides HMAC-based signing and verification for fingerprint files.
//! It uses HMAC-SHA256 to ensure integrity and authenticity of DSF files.

use chrono::Utc;
use hex;
use sha2::{Digest, Sha256};

use crate::models::Manifest;

use crate::error::{FingerprintError, FingerprintResult};
use crate::models::SignatureMetadata;

/// Algorithm identifier for HMAC-SHA256 signatures.
pub const ALGORITHM_HMAC_SHA256: &str = "HMAC-SHA256";

/// Signing key for DSF files.
#[derive(Clone)]
pub struct SigningKey {
    /// Key identifier.
    pub key_id: String,
    /// Secret key bytes.
    secret: Vec<u8>,
}

impl SigningKey {
    /// Create a signing key from a secret.
    ///
    /// The secret should be at least 32 bytes for security.
    pub fn new(key_id: impl Into<String>, secret: impl AsRef<[u8]>) -> Self {
        Self {
            key_id: key_id.into(),
            secret: secret.as_ref().to_vec(),
        }
    }

    /// Generate a random signing key.
    pub fn generate(key_id: impl Into<String>) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut secret = vec![0u8; 32];
        rng.fill(&mut secret[..]);

        Self {
            key_id: key_id.into(),
            secret,
        }
    }

    /// Create from a hex-encoded secret.
    pub fn from_hex(key_id: impl Into<String>, hex_secret: &str) -> FingerprintResult<Self> {
        let secret = hex::decode(hex_secret).map_err(|e| {
            FingerprintError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid hex key: {}", e),
            ))
        })?;

        Ok(Self {
            key_id: key_id.into(),
            secret,
        })
    }

    /// Export the secret as hex for storage.
    pub fn to_hex(&self) -> String {
        hex::encode(&self.secret)
    }

    /// Sign data and return signature metadata.
    pub fn sign(&self, data: &[u8]) -> SignatureMetadata {
        let signature = self.compute_signature(data);

        SignatureMetadata {
            algorithm: ALGORITHM_HMAC_SHA256.to_string(),
            key_id: self.key_id.clone(),
            signature: hex::encode(&signature),
            signed_at: Utc::now(),
        }
    }

    /// Verify a signature against data.
    pub fn verify(&self, data: &[u8], signature: &SignatureMetadata) -> FingerprintResult<bool> {
        // Check algorithm
        if signature.algorithm != ALGORITHM_HMAC_SHA256 {
            return Err(FingerprintError::InvalidFormat(format!(
                "Unsupported signature algorithm: {}",
                signature.algorithm
            )));
        }

        // Decode signature
        let expected_signature = hex::decode(&signature.signature).map_err(|e| {
            FingerprintError::InvalidFormat(format!("Invalid signature encoding: {}", e))
        })?;

        // Compute and compare
        let actual_signature = self.compute_signature(data);

        Ok(constant_time_compare(
            &actual_signature,
            &expected_signature,
        ))
    }

    /// Compute HMAC-SHA256 signature.
    fn compute_signature(&self, data: &[u8]) -> Vec<u8> {
        // HMAC-SHA256: H((K XOR opad) || H((K XOR ipad) || message))
        let block_size = 64;
        let mut key = self.secret.clone();

        // If key is longer than block size, hash it
        if key.len() > block_size {
            let mut hasher = Sha256::new();
            hasher.update(&key);
            key = hasher.finalize().to_vec();
        }

        // Pad key to block size
        key.resize(block_size, 0);

        // Create inner and outer padded keys
        let ipad: Vec<u8> = key.iter().map(|b| b ^ 0x36).collect();
        let opad: Vec<u8> = key.iter().map(|b| b ^ 0x5c).collect();

        // Inner hash
        let mut inner_hasher = Sha256::new();
        inner_hasher.update(&ipad);
        inner_hasher.update(data);
        let inner_hash = inner_hasher.finalize();

        // Outer hash
        let mut outer_hasher = Sha256::new();
        outer_hasher.update(&opad);
        outer_hasher.update(inner_hash);

        outer_hasher.finalize().to_vec()
    }
}

impl std::fmt::Debug for SigningKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SigningKey")
            .field("key_id", &self.key_id)
            .field("secret", &"[REDACTED]")
            .finish()
    }
}

/// Constant-time comparison to prevent timing attacks.
fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

/// Create canonical JSON representation of a manifest for signing.
///
/// This creates a deterministic JSON string by:
/// 1. Removing the signature field
/// 2. Sorting all HashMap keys
pub fn canonical_manifest_json(manifest: &Manifest) -> String {
    // Create a copy without signature
    let mut manifest = manifest.clone();
    manifest.signature = None;

    // Convert to serde_json::Value for sorting
    let value = serde_json::to_value(&manifest)
        .expect("Manifest serialization to serde_json::Value should not fail");

    // Recursively sort all objects
    let sorted = sort_json_value(value);

    serde_json::to_string(&sorted)
        .expect("Sorted JSON Value serialization to string should not fail")
}

/// Recursively sort all keys in a JSON value.
fn sort_json_value(value: serde_json::Value) -> serde_json::Value {
    use serde_json::Value;
    use std::collections::BTreeMap;

    match value {
        Value::Object(map) => {
            let sorted: BTreeMap<String, Value> = map
                .into_iter()
                .map(|(k, v)| (k, sort_json_value(v)))
                .collect();
            Value::Object(sorted.into_iter().collect())
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(sort_json_value).collect()),
        other => other,
    }
}

/// Signer that can sign DSF file contents.
#[derive(Debug, Clone)]
pub struct DsfSigner {
    key: SigningKey,
}

impl DsfSigner {
    /// Create a new signer with the given key.
    pub fn new(key: SigningKey) -> Self {
        Self { key }
    }

    /// Sign a manifest.
    ///
    /// Creates a canonical JSON representation and signs it.
    pub fn sign_manifest(&self, manifest: &Manifest) -> SignatureMetadata {
        let canonical = canonical_manifest_json(manifest);
        self.key.sign(canonical.as_bytes())
    }

    /// Sign raw data (legacy method).
    ///
    /// The data should be the canonical JSON representation of the manifest
    /// without the signature field.
    pub fn sign(&self, manifest_data: &[u8]) -> SignatureMetadata {
        self.key.sign(manifest_data)
    }

    /// Get the key ID.
    pub fn key_id(&self) -> &str {
        &self.key.key_id
    }
}

/// Verifier that can verify DSF file signatures.
#[derive(Debug, Clone)]
pub struct DsfVerifier {
    key: SigningKey,
}

impl DsfVerifier {
    /// Create a new verifier with the given key.
    pub fn new(key: SigningKey) -> Self {
        Self { key }
    }

    /// Verify a manifest's signature.
    ///
    /// Creates a canonical JSON representation and verifies against the signature.
    pub fn verify_manifest(&self, manifest: &Manifest) -> FingerprintResult<bool> {
        let signature = match &manifest.signature {
            Some(sig) => sig,
            None => return Ok(false),
        };

        // Check key ID matches
        if signature.key_id != self.key.key_id {
            return Ok(false);
        }

        let canonical = canonical_manifest_json(manifest);
        self.key.verify(canonical.as_bytes(), signature)
    }

    /// Verify the manifest signature (legacy raw data method).
    ///
    /// The data should be the canonical JSON representation of the manifest
    /// without the signature field.
    pub fn verify(
        &self,
        manifest_data: &[u8],
        signature: &SignatureMetadata,
    ) -> FingerprintResult<bool> {
        // Check key ID matches
        if signature.key_id != self.key.key_id {
            return Ok(false);
        }

        self.key.verify(manifest_data, signature)
    }

    /// Get the key ID.
    pub fn key_id(&self) -> &str {
        &self.key.key_id
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify() {
        let key = SigningKey::generate("test-key");
        let data = b"Hello, World!";

        let signature = key.sign(data);
        assert_eq!(signature.algorithm, ALGORITHM_HMAC_SHA256);
        assert_eq!(signature.key_id, "test-key");

        let result = key.verify(data, &signature).unwrap();
        assert!(result, "Signature should be valid");
    }

    #[test]
    fn test_verify_wrong_data() {
        let key = SigningKey::generate("test-key");
        let data = b"Hello, World!";
        let wrong_data = b"Hello, World";

        let signature = key.sign(data);
        let result = key.verify(wrong_data, &signature).unwrap();
        assert!(!result, "Signature should be invalid for wrong data");
    }

    #[test]
    fn test_key_from_hex() {
        let original = SigningKey::generate("test-key");
        let hex_secret = original.to_hex();

        let restored = SigningKey::from_hex("test-key", &hex_secret).unwrap();
        assert_eq!(restored.key_id, original.key_id);

        // Both keys should produce the same signature
        let data = b"test data";
        let sig1 = original.sign(data);
        let sig2 = restored.sign(data);
        assert_eq!(sig1.signature, sig2.signature);
    }

    #[test]
    fn test_signer_verifier() {
        let key = SigningKey::generate("my-key");
        let signer = DsfSigner::new(key.clone());
        let verifier = DsfVerifier::new(key);

        let manifest_data = b"{\"version\":\"1.0\",\"format\":\"dsf\"}";
        let signature = signer.sign(manifest_data);

        let is_valid = verifier.verify(manifest_data, &signature).unwrap();
        assert!(is_valid, "Signature should be valid");
    }
}
