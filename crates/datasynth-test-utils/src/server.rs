//! Test server utilities for integration testing.

use std::net::SocketAddr;
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;

use thiserror::Error;
use tokio::time::timeout;

/// Error type for test server operations.
#[derive(Debug, Error)]
pub enum TestServerError {
    #[error("Server startup timeout")]
    StartupTimeout,

    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),

    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

/// Global port counter for unique test ports.
static PORT_COUNTER: AtomicU16 = AtomicU16::new(50100);

/// Get a unique port for testing.
pub fn get_test_port() -> u16 {
    PORT_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Configuration for a test server.
#[derive(Debug, Clone)]
pub struct TestServerConfig {
    /// Host address.
    pub host: String,
    /// Port to listen on.
    pub port: u16,
    /// Startup timeout in seconds.
    pub startup_timeout_secs: u64,
    /// Health check interval in milliseconds.
    pub health_check_interval_ms: u64,
}

impl Default for TestServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: get_test_port(),
            startup_timeout_secs: 10,
            health_check_interval_ms: 100,
        }
    }
}

impl TestServerConfig {
    /// Get the address as a SocketAddr.
    pub fn addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("Invalid address")
    }

    /// Get the base URL for REST API.
    pub fn rest_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }

    /// Get the gRPC address.
    pub fn grpc_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }

    /// Get the WebSocket URL.
    pub fn ws_url(&self, path: &str) -> String {
        format!("ws://{}:{}{}", self.host, self.port, path)
    }
}

/// Wait for a server to become healthy.
pub async fn wait_for_health(
    base_url: &str,
    timeout_secs: u64,
    interval_ms: u64,
) -> Result<(), TestServerError> {
    let client = reqwest::Client::new();
    let health_url = format!("{}/health", base_url);

    let result = timeout(Duration::from_secs(timeout_secs), async {
        loop {
            match client.get(&health_url).send().await {
                Ok(response) if response.status().is_success() => {
                    return Ok(());
                }
                Ok(response) => {
                    // Server responded but not healthy
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    tracing::debug!("Health check returned {}: {}", status, body);
                }
                Err(e) => {
                    tracing::debug!("Health check failed: {}", e);
                }
            }
            tokio::time::sleep(Duration::from_millis(interval_ms)).await;
        }
    })
    .await;

    match result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(TestServerError::StartupTimeout),
    }
}

/// Check if a server is healthy.
pub async fn is_healthy(base_url: &str) -> bool {
    let client = reqwest::Client::new();
    let health_url = format!("{}/health", base_url);

    match client.get(&health_url).send().await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

/// HTTP client wrapper for testing REST APIs.
pub struct TestHttpClient {
    client: reqwest::Client,
    base_url: String,
}

impl TestHttpClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            base_url: base_url.to_string(),
        }
    }

    /// GET request.
    pub async fn get(&self, path: &str) -> Result<reqwest::Response, TestServerError> {
        let url = format!("{}{}", self.base_url, path);
        self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| TestServerError::RequestFailed(e.to_string()))
    }

    /// GET request returning JSON.
    pub async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T, TestServerError> {
        let response = self.get(path).await?;
        response
            .json()
            .await
            .map_err(|e| TestServerError::InvalidResponse(e.to_string()))
    }

    /// POST request with JSON body.
    pub async fn post<T: serde::Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<reqwest::Response, TestServerError> {
        let url = format!("{}{}", self.base_url, path);
        self.client
            .post(&url)
            .json(body)
            .send()
            .await
            .map_err(|e| TestServerError::RequestFailed(e.to_string()))
    }

    /// POST request returning JSON.
    pub async fn post_json<T: serde::Serialize, R: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<R, TestServerError> {
        let response = self.post(path, body).await?;
        response
            .json()
            .await
            .map_err(|e| TestServerError::InvalidResponse(e.to_string()))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_get_test_port_unique() {
        let port1 = get_test_port();
        let port2 = get_test_port();
        let port3 = get_test_port();

        assert_ne!(port1, port2);
        assert_ne!(port2, port3);
        assert_ne!(port1, port3);
    }

    #[test]
    fn test_server_config_default() {
        let config = TestServerConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert!(config.port > 50000);
        assert_eq!(config.startup_timeout_secs, 10);
    }

    #[test]
    #[allow(clippy::field_reassign_with_default)]
    fn test_server_config_urls() {
        let mut config = TestServerConfig::default();
        config.port = 3000;

        assert_eq!(config.rest_url(), "http://127.0.0.1:3000");
        assert_eq!(config.grpc_url(), "http://127.0.0.1:3000");
        assert_eq!(config.ws_url("/ws/events"), "ws://127.0.0.1:3000/ws/events");
    }
}
