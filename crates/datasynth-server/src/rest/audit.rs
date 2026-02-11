//! Structured audit logging for the REST API.
//!
//! Provides a trait-based audit logging system that records security-relevant
//! events (authentication, authorization, data access) in a structured JSON
//! format suitable for SIEM ingestion.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ===========================================================================
// Audit event types
// ===========================================================================

/// Outcome of an audited action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    /// The action completed successfully.
    Success,
    /// The action was denied (e.g., insufficient permissions).
    Denied,
    /// The action failed due to an internal error.
    Error,
}

/// A single audit event capturing who did what, when, and whether it succeeded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// When the event occurred (UTC).
    pub timestamp: DateTime<Utc>,
    /// Unique request identifier for correlation.
    pub request_id: String,
    /// Identity of the actor (user ID, API key hash prefix, or "anonymous").
    pub actor: String,
    /// The action that was attempted (e.g., "generate_data", "view_config").
    pub action: String,
    /// The resource that was acted upon (e.g., "/api/stream/start", "job:abc123").
    pub resource: String,
    /// Whether the action succeeded, was denied, or errored.
    pub outcome: AuditOutcome,
    /// Tenant identifier for multi-tenant deployments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    /// Source IP address of the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    /// User-Agent header value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

// ===========================================================================
// Audit logger trait and implementations
// ===========================================================================

/// Trait for audit event sinks.
///
/// Implementations may log to stdout, files, external services, etc.
pub trait AuditLogger: Send + Sync {
    /// Record an audit event.
    fn log_event(&self, event: &AuditEvent);
}

/// Logs audit events as JSON via the `tracing` crate.
///
/// Events are emitted at `INFO` level with a structured `audit_event` field,
/// making them easy to filter and forward in log aggregation pipelines.
pub struct JsonAuditLogger;

impl AuditLogger for JsonAuditLogger {
    fn log_event(&self, event: &AuditEvent) {
        // Serialize to a JSON string; fall back to debug format on failure.
        match serde_json::to_string(event) {
            Ok(json) => {
                tracing::info!(
                    audit_event = %json,
                    actor = %event.actor,
                    action = %event.action,
                    outcome = ?event.outcome,
                    "audit"
                );
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    actor = %event.actor,
                    action = %event.action,
                    "Failed to serialize audit event"
                );
            }
        }
    }
}

/// A no-op logger that silently discards events.
///
/// Used when audit logging is disabled to avoid runtime overhead.
pub struct NoopAuditLogger;

impl AuditLogger for NoopAuditLogger {
    fn log_event(&self, _event: &AuditEvent) {
        // Intentionally empty.
    }
}

// ===========================================================================
// Configuration
// ===========================================================================

/// Configuration for the audit logging subsystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Whether audit logging is enabled.
    #[serde(default)]
    pub enabled: bool,

    /// Whether to log audit events to stdout (via tracing).
    #[serde(default = "default_true")]
    pub log_to_stdout: bool,
}

fn default_true() -> bool {
    true
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            log_to_stdout: true,
        }
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_json_audit_logger_does_not_panic() {
        let logger = JsonAuditLogger;
        let event = AuditEvent {
            timestamp: Utc::now(),
            request_id: "req-001".to_string(),
            actor: "user@example.com".to_string(),
            action: "generate_data".to_string(),
            resource: "/api/stream/start".to_string(),
            outcome: AuditOutcome::Success,
            tenant_id: Some("tenant-1".to_string()),
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: Some("datasynth-cli/0.5.0".to_string()),
        };
        // Should not panic even without a tracing subscriber installed.
        logger.log_event(&event);
    }

    #[test]
    fn test_noop_audit_logger() {
        let logger = NoopAuditLogger;
        let event = AuditEvent {
            timestamp: Utc::now(),
            request_id: "req-002".to_string(),
            actor: "anonymous".to_string(),
            action: "view_metrics".to_string(),
            resource: "/metrics".to_string(),
            outcome: AuditOutcome::Denied,
            tenant_id: None,
            ip_address: None,
            user_agent: None,
        };
        // Should be a no-op.
        logger.log_event(&event);
    }

    #[test]
    fn test_audit_event_serialization_roundtrip() {
        let event = AuditEvent {
            timestamp: Utc::now(),
            request_id: "req-003".to_string(),
            actor: "admin-key-ab12".to_string(),
            action: "manage_config".to_string(),
            resource: "/api/config".to_string(),
            outcome: AuditOutcome::Error,
            tenant_id: None,
            ip_address: Some("10.0.0.1".to_string()),
            user_agent: None,
        };

        let json = serde_json::to_string(&event).expect("should serialize");
        let deserialized: AuditEvent = serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(deserialized.request_id, "req-003");
        assert_eq!(deserialized.actor, "admin-key-ab12");
        assert_eq!(deserialized.action, "manage_config");
        assert_eq!(deserialized.outcome, AuditOutcome::Error);
        assert!(deserialized.tenant_id.is_none());
        assert_eq!(deserialized.ip_address, Some("10.0.0.1".to_string()));
        assert!(deserialized.user_agent.is_none());
    }

    #[test]
    fn test_audit_config_defaults() {
        let config = AuditConfig::default();
        assert!(!config.enabled);
        assert!(config.log_to_stdout);
    }

    #[test]
    fn test_audit_outcome_serialization() {
        assert_eq!(
            serde_json::to_string(&AuditOutcome::Success).unwrap(),
            "\"success\""
        );
        assert_eq!(
            serde_json::to_string(&AuditOutcome::Denied).unwrap(),
            "\"denied\""
        );
        assert_eq!(
            serde_json::to_string(&AuditOutcome::Error).unwrap(),
            "\"error\""
        );
    }
}
