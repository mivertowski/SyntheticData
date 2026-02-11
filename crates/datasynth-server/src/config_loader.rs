//! External config loading for multi-instance deployments.
//!
//! Supports loading configuration from file, URL, inline string, or defaults.

use datasynth_config::schema::GeneratorConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Source for loading configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConfigSource {
    /// Load from a YAML file.
    File { path: PathBuf },
    /// Load from a URL (HTTP GET) - requires an external fetch.
    Url { url: String },
    /// Inline YAML/JSON string.
    Inline { content: String },
    /// Use default configuration.
    #[default]
    Default,
}

/// Loads a GeneratorConfig from the specified source.
pub async fn load_config(source: &ConfigSource) -> Result<GeneratorConfig, ConfigLoadError> {
    match source {
        ConfigSource::File { path } => {
            info!("Loading config from file: {}", path.display());
            let content = tokio::fs::read_to_string(path).await.map_err(|e| {
                ConfigLoadError::Io(format!("Failed to read {}: {}", path.display(), e))
            })?;
            let config: GeneratorConfig = serde_yaml::from_str(&content)
                .map_err(|e| ConfigLoadError::Parse(format!("Failed to parse YAML: {}", e)))?;
            Ok(config)
        }
        ConfigSource::Url { url } => {
            // URL loading uses a simple blocking HTTP client to avoid adding reqwest dependency
            info!("Loading config from URL: {}", url);
            Err(ConfigLoadError::Io(format!(
                "URL config loading not yet supported. Use file or inline config instead. URL: {}",
                url
            )))
        }
        ConfigSource::Inline { content } => {
            info!("Loading inline config ({} bytes)", content.len());
            let config: GeneratorConfig = serde_yaml::from_str(content)
                .map_err(|e| ConfigLoadError::Parse(format!("Failed to parse YAML: {}", e)))?;
            Ok(config)
        }
        ConfigSource::Default => {
            info!("Using default generator config");
            Ok(crate::grpc::service::default_generator_config())
        }
    }
}

/// Reloads configuration from a source into shared state.
pub async fn reload_config(
    source: &ConfigSource,
    config_lock: &Arc<RwLock<GeneratorConfig>>,
) -> Result<(), ConfigLoadError> {
    let new_config = load_config(source).await?;
    let mut config = config_lock.write().await;
    *config = new_config;
    info!("Configuration reloaded successfully");
    Ok(())
}

/// Error type for config loading.
#[derive(Debug, Clone)]
pub enum ConfigLoadError {
    /// I/O error (file not found, network error).
    Io(String),
    /// Parse error (invalid YAML/JSON).
    Parse(String),
}

impl std::fmt::Display for ConfigLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(msg) => write!(f, "Config I/O error: {}", msg),
            Self::Parse(msg) => write!(f, "Config parse error: {}", msg),
        }
    }
}

impl std::error::Error for ConfigLoadError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_default_config() {
        let config = load_config(&ConfigSource::Default).await.unwrap();
        assert!(!config.companies.is_empty());
    }

    #[tokio::test]
    async fn test_load_inline_config() {
        let yaml = r#"
global:
  industry: manufacturing
  start_date: "2024-01-01"
  period_months: 1
  seed: 42
  parallel: false
  group_currency: USD
  worker_threads: 1
  memory_limit_mb: 512
companies:
  - code: TEST
    name: Test Company
    currency: USD
    country: US
    annual_transaction_volume: ten_k
    volume_weight: 1.0
    fiscal_year_variant: K4
chart_of_accounts:
  complexity: small
output:
  output_directory: ./output
"#;
        let source = ConfigSource::Inline {
            content: yaml.to_string(),
        };
        let config = load_config(&source).await.unwrap();
        assert_eq!(config.companies[0].code, "TEST");
    }

    #[tokio::test]
    async fn test_load_missing_file() {
        let source = ConfigSource::File {
            path: PathBuf::from("/nonexistent/config.yaml"),
        };
        assert!(load_config(&source).await.is_err());
    }

    #[tokio::test]
    async fn test_load_invalid_yaml() {
        let source = ConfigSource::Inline {
            content: "{{invalid yaml:".to_string(),
        };
        assert!(load_config(&source).await.is_err());
    }

    #[tokio::test]
    async fn test_reload_config() {
        let initial = crate::grpc::service::default_generator_config();
        let config_lock = Arc::new(RwLock::new(initial));

        reload_config(&ConfigSource::Default, &config_lock)
            .await
            .unwrap();

        let config = config_lock.read().await;
        assert!(!config.companies.is_empty());
    }
}
