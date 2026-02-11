//! Reader for .dsf fingerprint files.

use std::io::{Read, Seek};
use std::path::Path;

use sha2::{Digest, Sha256};
use zip::ZipArchive;

use crate::error::{FingerprintError, FingerprintResult};
use crate::models::{
    AnomalyFingerprint, CorrelationFingerprint, Fingerprint, IntegrityFingerprint, Manifest,
    PrivacyAudit, RulesFingerprint, SchemaFingerprint, StatisticsFingerprint, FINGERPRINT_VERSION,
};

use super::file_names;
use super::signing::DsfVerifier;

/// Options for reading fingerprint files.
#[derive(Debug, Clone, Default)]
pub struct ReadOptions {
    /// Whether to verify checksums.
    pub verify_checksums: bool,
    /// Whether to allow version mismatches (with warning).
    pub allow_version_mismatch: bool,
}

/// Reader for .dsf fingerprint files.
pub struct FingerprintReader {
    options: ReadOptions,
}

impl FingerprintReader {
    /// Create a new fingerprint reader with default options.
    pub fn new() -> Self {
        Self {
            options: ReadOptions {
                verify_checksums: true,
                allow_version_mismatch: false,
            },
        }
    }

    /// Create a new fingerprint reader with custom options.
    pub fn with_options(options: ReadOptions) -> Self {
        Self { options }
    }

    /// Read a fingerprint from a file.
    pub fn read_from_file(&self, path: &Path) -> FingerprintResult<Fingerprint> {
        let file = std::fs::File::open(path)?;
        self.read(file)
    }

    /// Read a fingerprint from a file and verify its signature.
    ///
    /// Returns an error if:
    /// - The file has no signature
    /// - The signature verification fails
    /// - Any other read error occurs
    pub fn read_from_file_verified(
        &self,
        path: &Path,
        verifier: &DsfVerifier,
    ) -> FingerprintResult<Fingerprint> {
        let file = std::fs::File::open(path)?;
        self.read_verified(file, verifier)
    }

    /// Read a fingerprint and verify its signature.
    pub fn read_verified<R: Read + Seek>(
        &self,
        reader: R,
        verifier: &DsfVerifier,
    ) -> FingerprintResult<Fingerprint> {
        let mut archive = ZipArchive::new(reader)?;

        // Read manifest with signature
        let manifest: Manifest = {
            let mut file = archive.by_name(file_names::MANIFEST)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            serde_json::from_str(&contents)?
        };

        // Verify signature using canonical JSON
        let is_valid = verifier.verify_manifest(&manifest)?;
        if !is_valid {
            if manifest.signature.is_none() {
                return Err(FingerprintError::InvalidFormat(
                    "DSF file is not signed".to_string(),
                ));
            } else {
                return Err(FingerprintError::InvalidFormat(
                    "Signature verification failed".to_string(),
                ));
            }
        }

        // Check version compatibility
        if manifest.version != FINGERPRINT_VERSION
            && !self.options.allow_version_mismatch
            && !is_compatible_version(&manifest.version, FINGERPRINT_VERSION)
        {
            return Err(FingerprintError::UnsupportedVersion(manifest.version));
        }

        // Read components (same as regular read)
        let schema: SchemaFingerprint =
            self.read_yaml_component(&mut archive, file_names::SCHEMA, &manifest.checksums)?;

        let statistics: StatisticsFingerprint =
            self.read_yaml_component(&mut archive, file_names::STATISTICS, &manifest.checksums)?;

        let privacy_audit: PrivacyAudit =
            self.read_json_component(&mut archive, file_names::PRIVACY_AUDIT, &manifest.checksums)?;

        let correlations: Option<CorrelationFingerprint> = self.try_read_yaml_component(
            &mut archive,
            file_names::CORRELATIONS,
            &manifest.checksums,
        )?;

        let integrity: Option<IntegrityFingerprint> =
            self.try_read_yaml_component(&mut archive, file_names::INTEGRITY, &manifest.checksums)?;

        let rules: Option<RulesFingerprint> =
            self.try_read_yaml_component(&mut archive, file_names::RULES, &manifest.checksums)?;

        let anomalies: Option<AnomalyFingerprint> =
            self.try_read_yaml_component(&mut archive, file_names::ANOMALIES, &manifest.checksums)?;

        Ok(Fingerprint {
            manifest,
            schema,
            statistics,
            correlations,
            integrity,
            rules,
            anomalies,
            privacy_audit,
        })
    }

    /// Check if a fingerprint file is signed.
    pub fn is_signed(&self, path: &Path) -> FingerprintResult<bool> {
        let file = std::fs::File::open(path)?;
        let mut archive = ZipArchive::new(file)?;

        let manifest: Manifest = {
            let mut file = archive.by_name(file_names::MANIFEST)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            serde_json::from_str(&contents)?
        };

        Ok(manifest.signature.is_some())
    }

    /// Read a fingerprint from any reader.
    pub fn read<R: Read + Seek>(&self, reader: R) -> FingerprintResult<Fingerprint> {
        let mut archive = ZipArchive::new(reader)?;

        // Read and parse manifest first
        let manifest: Manifest = {
            let mut file = archive.by_name(file_names::MANIFEST)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            serde_json::from_str(&contents)?
        };

        // Check version compatibility
        if manifest.version != FINGERPRINT_VERSION && !self.options.allow_version_mismatch {
            // For now, we only support exact version match
            // In the future, we could support backward-compatible reads
            if !is_compatible_version(&manifest.version, FINGERPRINT_VERSION) {
                return Err(FingerprintError::UnsupportedVersion(manifest.version));
            }
        }

        // Read schema (required)
        let schema: SchemaFingerprint =
            self.read_yaml_component(&mut archive, file_names::SCHEMA, &manifest.checksums)?;

        // Read statistics (required)
        let statistics: StatisticsFingerprint =
            self.read_yaml_component(&mut archive, file_names::STATISTICS, &manifest.checksums)?;

        // Read privacy audit (required)
        let privacy_audit: PrivacyAudit =
            self.read_json_component(&mut archive, file_names::PRIVACY_AUDIT, &manifest.checksums)?;

        // Read optional components
        let correlations: Option<CorrelationFingerprint> = self.try_read_yaml_component(
            &mut archive,
            file_names::CORRELATIONS,
            &manifest.checksums,
        )?;

        let integrity: Option<IntegrityFingerprint> =
            self.try_read_yaml_component(&mut archive, file_names::INTEGRITY, &manifest.checksums)?;

        let rules: Option<RulesFingerprint> =
            self.try_read_yaml_component(&mut archive, file_names::RULES, &manifest.checksums)?;

        let anomalies: Option<AnomalyFingerprint> =
            self.try_read_yaml_component(&mut archive, file_names::ANOMALIES, &manifest.checksums)?;

        Ok(Fingerprint {
            manifest,
            schema,
            statistics,
            correlations,
            integrity,
            rules,
            anomalies,
            privacy_audit,
        })
    }

    /// Read a required YAML component.
    fn read_yaml_component<R: Read + Seek, T: serde::de::DeserializeOwned>(
        &self,
        archive: &mut ZipArchive<R>,
        name: &str,
        checksums: &std::collections::HashMap<String, String>,
    ) -> FingerprintResult<T> {
        let mut file = archive
            .by_name(name)
            .map_err(|_| FingerprintError::MissingComponent(name.to_string()))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        // Verify checksum if enabled
        if self.options.verify_checksums {
            if let Some(expected) = checksums.get(name) {
                let actual = compute_checksum(contents.as_bytes());
                if &actual != expected {
                    return Err(FingerprintError::ChecksumMismatch {
                        file: name.to_string(),
                        expected: expected.clone(),
                        actual,
                    });
                }
            }
        }

        Ok(serde_yaml::from_str(&contents)?)
    }

    /// Read a required JSON component.
    fn read_json_component<R: Read + Seek, T: serde::de::DeserializeOwned>(
        &self,
        archive: &mut ZipArchive<R>,
        name: &str,
        checksums: &std::collections::HashMap<String, String>,
    ) -> FingerprintResult<T> {
        let mut file = archive
            .by_name(name)
            .map_err(|_| FingerprintError::MissingComponent(name.to_string()))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        // Verify checksum if enabled
        if self.options.verify_checksums {
            if let Some(expected) = checksums.get(name) {
                let actual = compute_checksum(contents.as_bytes());
                if &actual != expected {
                    return Err(FingerprintError::ChecksumMismatch {
                        file: name.to_string(),
                        expected: expected.clone(),
                        actual,
                    });
                }
            }
        }

        Ok(serde_json::from_str(&contents)?)
    }

    /// Try to read an optional YAML component.
    fn try_read_yaml_component<R: Read + Seek, T: serde::de::DeserializeOwned>(
        &self,
        archive: &mut ZipArchive<R>,
        name: &str,
        checksums: &std::collections::HashMap<String, String>,
    ) -> FingerprintResult<Option<T>> {
        match archive.by_name(name) {
            Ok(mut file) => {
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;

                // Verify checksum if enabled
                if self.options.verify_checksums {
                    if let Some(expected) = checksums.get(name) {
                        let actual = compute_checksum(contents.as_bytes());
                        if &actual != expected {
                            return Err(FingerprintError::ChecksumMismatch {
                                file: name.to_string(),
                                expected: expected.clone(),
                                actual,
                            });
                        }
                    }
                }

                Ok(Some(serde_yaml::from_str(&contents)?))
            }
            Err(zip::result::ZipError::FileNotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

impl Default for FingerprintReader {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if two versions are compatible.
fn is_compatible_version(file_version: &str, current_version: &str) -> bool {
    // Parse semver-like versions
    let file_parts: Vec<u32> = file_version
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    let current_parts: Vec<u32> = current_version
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    if file_parts.is_empty() || current_parts.is_empty() {
        return false;
    }

    // Major version must match
    if file_parts[0] != current_parts[0] {
        return false;
    }

    // Minor version of file must be <= current
    if file_parts.len() > 1 && current_parts.len() > 1 {
        return file_parts[1] <= current_parts[1];
    }

    true
}

/// Compute SHA-256 checksum of data.
fn compute_checksum(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_version_compatibility() {
        assert!(is_compatible_version("1.0.0", "1.0.0"));
        assert!(is_compatible_version("1.0.0", "1.1.0"));
        assert!(is_compatible_version("1.0.0", "1.0.1"));
        assert!(!is_compatible_version("2.0.0", "1.0.0"));
        assert!(!is_compatible_version("1.2.0", "1.1.0"));
    }
}
