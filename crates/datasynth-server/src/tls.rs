//! Optional TLS configuration for the REST server.
//!
//! Enabled with the `tls` feature flag.

#[cfg(feature = "tls")]
use std::path::PathBuf;

/// TLS configuration.
#[cfg(feature = "tls")]
#[derive(Clone, Debug)]
pub struct TlsConfig {
    /// Path to the TLS certificate file (PEM format).
    pub cert_path: PathBuf,
    /// Path to the TLS private key file (PEM format).
    pub key_path: PathBuf,
}

#[cfg(feature = "tls")]
impl TlsConfig {
    /// Create a new TLS configuration.
    pub fn new(cert_path: impl Into<PathBuf>, key_path: impl Into<PathBuf>) -> Self {
        Self {
            cert_path: cert_path.into(),
            key_path: key_path.into(),
        }
    }

    /// Build a `RustlsConfig` from the certificate and key files.
    pub async fn build_rustls_config(
        &self,
    ) -> Result<axum_server::tls_rustls::RustlsConfig, std::io::Error> {
        axum_server::tls_rustls::RustlsConfig::from_pem_file(&self.cert_path, &self.key_path).await
    }
}
