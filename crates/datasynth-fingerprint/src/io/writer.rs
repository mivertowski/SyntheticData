//! Writer for .dsf fingerprint files.

use std::io::{Seek, Write};
use std::path::Path;

use sha2::{Digest, Sha256};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::error::FingerprintResult;
use crate::models::Fingerprint;

use super::file_names;
use super::signing::DsfSigner;

/// Options for writing fingerprint files.
#[derive(Debug, Clone)]
pub struct WriteOptions {
    /// Compression level (0-9, 0 = no compression).
    pub compression_level: u32,
    /// Whether to pretty-print JSON/YAML.
    pub pretty: bool,
}

impl Default for WriteOptions {
    fn default() -> Self {
        Self {
            compression_level: 6,
            pretty: true,
        }
    }
}

/// Writer for .dsf fingerprint files.
pub struct FingerprintWriter {
    options: WriteOptions,
}

impl FingerprintWriter {
    /// Create a new fingerprint writer with default options.
    pub fn new() -> Self {
        Self {
            options: WriteOptions::default(),
        }
    }

    /// Create a new fingerprint writer with custom options.
    pub fn with_options(options: WriteOptions) -> Self {
        Self { options }
    }

    /// Write a fingerprint to a file.
    pub fn write_to_file(&self, fingerprint: &Fingerprint, path: &Path) -> FingerprintResult<()> {
        let file = std::fs::File::create(path)?;
        self.write(fingerprint, file)
    }

    /// Write a fingerprint to a file with digital signature.
    ///
    /// The signature is computed over the manifest content (excluding the signature field)
    /// and included in the manifest.
    pub fn write_to_file_signed(
        &self,
        fingerprint: &Fingerprint,
        path: &Path,
        signer: &DsfSigner,
    ) -> FingerprintResult<()> {
        let file = std::fs::File::create(path)?;
        self.write_signed(fingerprint, file, signer)
    }

    /// Write a fingerprint with digital signature to any writer.
    pub fn write_signed<W: Write + Seek>(
        &self,
        fingerprint: &Fingerprint,
        writer: W,
        signer: &DsfSigner,
    ) -> FingerprintResult<()> {
        let mut zip = ZipWriter::new(writer);
        let options = SimpleFileOptions::default().compression_method(
            if self.options.compression_level > 0 {
                zip::CompressionMethod::Deflated
            } else {
                zip::CompressionMethod::Stored
            },
        );

        // Track checksums
        let mut checksums = std::collections::HashMap::new();

        // Write all components and collect checksums (same as regular write)
        let schema_yaml = serde_yaml::to_string(&fingerprint.schema)?;
        checksums.insert(
            file_names::SCHEMA.to_string(),
            compute_checksum(schema_yaml.as_bytes()),
        );
        zip.start_file(file_names::SCHEMA, options)?;
        zip.write_all(schema_yaml.as_bytes())?;

        let stats_yaml = serde_yaml::to_string(&fingerprint.statistics)?;
        checksums.insert(
            file_names::STATISTICS.to_string(),
            compute_checksum(stats_yaml.as_bytes()),
        );
        zip.start_file(file_names::STATISTICS, options)?;
        zip.write_all(stats_yaml.as_bytes())?;

        if let Some(ref correlations) = fingerprint.correlations {
            let yaml = serde_yaml::to_string(correlations)?;
            checksums.insert(
                file_names::CORRELATIONS.to_string(),
                compute_checksum(yaml.as_bytes()),
            );
            zip.start_file(file_names::CORRELATIONS, options)?;
            zip.write_all(yaml.as_bytes())?;
        }

        if let Some(ref integrity) = fingerprint.integrity {
            let yaml = serde_yaml::to_string(integrity)?;
            checksums.insert(
                file_names::INTEGRITY.to_string(),
                compute_checksum(yaml.as_bytes()),
            );
            zip.start_file(file_names::INTEGRITY, options)?;
            zip.write_all(yaml.as_bytes())?;
        }

        if let Some(ref rules) = fingerprint.rules {
            let yaml = serde_yaml::to_string(rules)?;
            checksums.insert(
                file_names::RULES.to_string(),
                compute_checksum(yaml.as_bytes()),
            );
            zip.start_file(file_names::RULES, options)?;
            zip.write_all(yaml.as_bytes())?;
        }

        if let Some(ref anomalies) = fingerprint.anomalies {
            let yaml = serde_yaml::to_string(anomalies)?;
            checksums.insert(
                file_names::ANOMALIES.to_string(),
                compute_checksum(yaml.as_bytes()),
            );
            zip.start_file(file_names::ANOMALIES, options)?;
            zip.write_all(yaml.as_bytes())?;
        }

        let audit_json = if self.options.pretty {
            serde_json::to_string_pretty(&fingerprint.privacy_audit)?
        } else {
            serde_json::to_string(&fingerprint.privacy_audit)?
        };
        checksums.insert(
            file_names::PRIVACY_AUDIT.to_string(),
            compute_checksum(audit_json.as_bytes()),
        );
        zip.start_file(file_names::PRIVACY_AUDIT, options)?;
        zip.write_all(audit_json.as_bytes())?;

        // Create manifest with checksums but WITHOUT signature
        let mut manifest = fingerprint.manifest.clone();
        manifest.checksums = checksums;
        manifest.signature = None;

        // Sign the manifest using canonical JSON
        let signature = signer.sign_manifest(&manifest);

        // Add signature to manifest
        manifest.signature = Some(signature);

        // Write final manifest with signature
        let manifest_json = if self.options.pretty {
            serde_json::to_string_pretty(&manifest)?
        } else {
            serde_json::to_string(&manifest)?
        };
        zip.start_file(file_names::MANIFEST, options)?;
        zip.write_all(manifest_json.as_bytes())?;

        zip.finish()?;
        Ok(())
    }

    /// Write a fingerprint to any writer.
    pub fn write<W: Write + Seek>(
        &self,
        fingerprint: &Fingerprint,
        writer: W,
    ) -> FingerprintResult<()> {
        let mut zip = ZipWriter::new(writer);
        let options = SimpleFileOptions::default().compression_method(
            if self.options.compression_level > 0 {
                zip::CompressionMethod::Deflated
            } else {
                zip::CompressionMethod::Stored
            },
        );

        // Track checksums
        let mut checksums = std::collections::HashMap::new();

        // Write manifest (we'll update it with checksums at the end)
        // For now, create a mutable copy
        let mut manifest = fingerprint.manifest.clone();

        // Write schema
        // Note: serde_yaml always produces human-readable output, so pretty option has no effect
        let schema_yaml = serde_yaml::to_string(&fingerprint.schema)?;
        checksums.insert(
            file_names::SCHEMA.to_string(),
            compute_checksum(schema_yaml.as_bytes()),
        );
        zip.start_file(file_names::SCHEMA, options)?;
        zip.write_all(schema_yaml.as_bytes())?;

        // Write statistics
        let stats_yaml = serde_yaml::to_string(&fingerprint.statistics)?;
        checksums.insert(
            file_names::STATISTICS.to_string(),
            compute_checksum(stats_yaml.as_bytes()),
        );
        zip.start_file(file_names::STATISTICS, options)?;
        zip.write_all(stats_yaml.as_bytes())?;

        // Write optional components
        if let Some(ref correlations) = fingerprint.correlations {
            let yaml = serde_yaml::to_string(correlations)?;
            checksums.insert(
                file_names::CORRELATIONS.to_string(),
                compute_checksum(yaml.as_bytes()),
            );
            zip.start_file(file_names::CORRELATIONS, options)?;
            zip.write_all(yaml.as_bytes())?;
        }

        if let Some(ref integrity) = fingerprint.integrity {
            let yaml = serde_yaml::to_string(integrity)?;
            checksums.insert(
                file_names::INTEGRITY.to_string(),
                compute_checksum(yaml.as_bytes()),
            );
            zip.start_file(file_names::INTEGRITY, options)?;
            zip.write_all(yaml.as_bytes())?;
        }

        if let Some(ref rules) = fingerprint.rules {
            let yaml = serde_yaml::to_string(rules)?;
            checksums.insert(
                file_names::RULES.to_string(),
                compute_checksum(yaml.as_bytes()),
            );
            zip.start_file(file_names::RULES, options)?;
            zip.write_all(yaml.as_bytes())?;
        }

        if let Some(ref anomalies) = fingerprint.anomalies {
            let yaml = serde_yaml::to_string(anomalies)?;
            checksums.insert(
                file_names::ANOMALIES.to_string(),
                compute_checksum(yaml.as_bytes()),
            );
            zip.start_file(file_names::ANOMALIES, options)?;
            zip.write_all(yaml.as_bytes())?;
        }

        // Write privacy audit
        let audit_json = if self.options.pretty {
            serde_json::to_string_pretty(&fingerprint.privacy_audit)?
        } else {
            serde_json::to_string(&fingerprint.privacy_audit)?
        };
        checksums.insert(
            file_names::PRIVACY_AUDIT.to_string(),
            compute_checksum(audit_json.as_bytes()),
        );
        zip.start_file(file_names::PRIVACY_AUDIT, options)?;
        zip.write_all(audit_json.as_bytes())?;

        // Update manifest with checksums and write it
        manifest.checksums = checksums;
        let manifest_json = if self.options.pretty {
            serde_json::to_string_pretty(&manifest)?
        } else {
            serde_json::to_string(&manifest)?
        };
        zip.start_file(file_names::MANIFEST, options)?;
        zip.write_all(manifest_json.as_bytes())?;

        zip.finish()?;
        Ok(())
    }
}

impl Default for FingerprintWriter {
    fn default() -> Self {
        Self::new()
    }
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
    use crate::models::{
        Manifest, PrivacyAudit, PrivacyLevel, PrivacyMetadata, SchemaFingerprint, SourceMetadata,
        StatisticsFingerprint,
    };
    use std::io::Cursor;

    #[test]
    fn test_write_fingerprint() {
        let source = SourceMetadata::new("Test source", vec!["test_table".to_string()], 100);
        let privacy = PrivacyMetadata::from_level(PrivacyLevel::Standard);
        let manifest = Manifest::new(source, privacy);
        let schema = SchemaFingerprint::new();
        let statistics = StatisticsFingerprint::new();
        let privacy_audit = PrivacyAudit::new(1.0, 5);

        let fingerprint = Fingerprint::new(manifest, schema, statistics, privacy_audit);

        let mut buffer = Cursor::new(Vec::new());
        let writer = FingerprintWriter::new();
        writer.write(&fingerprint, &mut buffer).unwrap();

        // Verify the buffer is not empty and starts with ZIP magic bytes
        let data = buffer.into_inner();
        assert!(!data.is_empty());
        assert_eq!(&data[0..2], b"PK"); // ZIP magic bytes
    }
}
