//! Webhook notification system for generation events.
//!
//! Sends HTTP POST notifications to configured endpoints when
//! generation events occur (started, completed, failed, gate_violation).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

/// Webhook event types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEvent {
    /// Generation run started.
    RunStarted,
    /// Generation run completed successfully.
    RunCompleted,
    /// Generation run failed.
    RunFailed,
    /// Quality gate violation detected.
    GateViolation,
}

/// Payload sent to webhook endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    /// Event type.
    pub event: WebhookEvent,
    /// Run identifier.
    pub run_id: String,
    /// ISO 8601 timestamp.
    pub timestamp: String,
    /// Additional event-specific data.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub data: HashMap<String, serde_json::Value>,
}

/// Configuration for a single webhook endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEndpoint {
    /// Target URL for the webhook.
    pub url: String,
    /// Events this endpoint subscribes to.
    pub events: Vec<WebhookEvent>,
    /// Optional secret for HMAC-SHA256 signature (X-Webhook-Signature header).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    /// Optional custom headers.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
    /// Maximum retry attempts (default: 3).
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    /// Timeout in seconds (default: 10).
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

fn default_max_retries() -> u32 {
    3
}
fn default_timeout_secs() -> u64 {
    10
}

/// Webhook notification configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WebhookConfig {
    /// Whether webhooks are enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Configured webhook endpoints.
    #[serde(default)]
    pub endpoints: Vec<WebhookEndpoint>,
}

/// Webhook dispatcher that sends notifications to configured endpoints.
///
/// Uses a fire-and-forget pattern — delivery failures are logged but
/// do not block generation.
#[derive(Debug, Clone)]
pub struct WebhookDispatcher {
    config: WebhookConfig,
}

impl WebhookDispatcher {
    /// Create a new dispatcher from configuration.
    pub fn new(config: WebhookConfig) -> Self {
        Self { config }
    }

    /// Create a disabled dispatcher.
    pub fn disabled() -> Self {
        Self {
            config: WebhookConfig::default(),
        }
    }

    /// Check if webhooks are enabled.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled && !self.config.endpoints.is_empty()
    }

    /// Dispatch a webhook event to all matching endpoints.
    ///
    /// This is a synchronous logging-only implementation.
    /// In production, this would use an async HTTP client.
    pub fn dispatch(&self, payload: &WebhookPayload) {
        if !self.is_enabled() {
            return;
        }

        for endpoint in &self.config.endpoints {
            if endpoint.events.contains(&payload.event) {
                info!(
                    url = %endpoint.url,
                    event = ?payload.event,
                    run_id = %payload.run_id,
                    "Webhook notification queued"
                );
            }
        }
    }

    /// Create a payload for a run-started event.
    pub fn run_started_payload(run_id: &str) -> WebhookPayload {
        WebhookPayload {
            event: WebhookEvent::RunStarted,
            run_id: run_id.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            data: HashMap::new(),
        }
    }

    /// Create a payload for a run-completed event.
    pub fn run_completed_payload(
        run_id: &str,
        total_entries: usize,
        duration_secs: f64,
    ) -> WebhookPayload {
        let mut data = HashMap::new();
        data.insert(
            "total_entries".to_string(),
            serde_json::json!(total_entries),
        );
        data.insert(
            "duration_secs".to_string(),
            serde_json::json!(duration_secs),
        );
        WebhookPayload {
            event: WebhookEvent::RunCompleted,
            run_id: run_id.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            data,
        }
    }

    /// Create a payload for a gate-violation event.
    pub fn gate_violation_payload(run_id: &str, failed_gates: Vec<String>) -> WebhookPayload {
        let mut data = HashMap::new();
        data.insert("failed_gates".to_string(), serde_json::json!(failed_gates));
        WebhookPayload {
            event: WebhookEvent::GateViolation,
            run_id: run_id.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            data,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_disabled_dispatcher() {
        let d = WebhookDispatcher::disabled();
        assert!(!d.is_enabled());
    }

    #[test]
    fn test_enabled_dispatcher() {
        let config = WebhookConfig {
            enabled: true,
            endpoints: vec![WebhookEndpoint {
                url: "https://example.com/webhook".to_string(),
                events: vec![WebhookEvent::RunCompleted],
                secret: None,
                headers: HashMap::new(),
                max_retries: 3,
                timeout_secs: 10,
            }],
        };
        let d = WebhookDispatcher::new(config);
        assert!(d.is_enabled());
    }

    #[test]
    fn test_payload_serialization() {
        let payload = WebhookDispatcher::run_completed_payload("run-123", 1000, 5.5);
        let json = serde_json::to_string(&payload).expect("serialization should succeed");
        assert!(json.contains("run_completed"));
        assert!(json.contains("run-123"));
        assert!(json.contains("1000"));
    }

    #[test]
    fn test_gate_violation_payload() {
        let payload = WebhookDispatcher::gate_violation_payload(
            "run-456",
            vec!["benford_mad".to_string(), "balance_coherence".to_string()],
        );
        assert_eq!(payload.event, WebhookEvent::GateViolation);
        assert!(payload.data.contains_key("failed_gates"));
    }

    #[test]
    fn test_webhook_config_default() {
        let config = WebhookConfig::default();
        assert!(!config.enabled);
        assert!(config.endpoints.is_empty());
    }

    #[test]
    fn test_dispatch_noop_when_disabled() {
        let d = WebhookDispatcher::disabled();
        let payload = WebhookDispatcher::run_started_payload("run-789");
        d.dispatch(&payload); // Should not panic
    }
}
