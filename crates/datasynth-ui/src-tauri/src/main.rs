//! Tauri application for synthetic data generator UI.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

/// Application state shared across Tauri commands.
pub struct AppState {
    pub server_url: RwLock<String>,
    pub client: reqwest::Client,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            server_url: RwLock::new("http://localhost:3000".to_string()),
            client: reqwest::Client::new(),
        }
    }
}

/// Health check response from the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub healthy: bool,
    pub version: String,
    pub uptime_seconds: u64,
}

/// Metrics response from the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsResponse {
    pub total_entries_generated: u64,
    pub total_anomalies_injected: u64,
    pub uptime_seconds: u64,
    pub session_entries: u64,
    pub session_entries_per_second: f64,
    pub active_streams: u32,
    pub total_stream_events: u64,
}

/// Configuration DTO.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub success: bool,
    pub message: String,
    pub config: Option<GenerationConfigDto>,
}

/// Generation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfigDto {
    pub industry: String,
    pub start_date: String,
    pub period_months: u32,
    pub seed: Option<u64>,
    pub coa_complexity: String,
    pub companies: Vec<CompanyConfigDto>,
    pub fraud_enabled: bool,
    pub fraud_rate: f32,
}

/// Company configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyConfigDto {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub country: String,
    pub annual_transaction_volume: u64,
    pub volume_weight: f32,
}

/// Bulk generation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkGenerateRequest {
    pub entry_count: Option<u64>,
    pub include_master_data: Option<bool>,
    pub inject_anomalies: Option<bool>,
}

/// Bulk generation response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkGenerateResponse {
    pub success: bool,
    pub entries_generated: u64,
    pub duration_ms: u64,
    pub anomaly_count: u64,
}

/// Stream control response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamResponse {
    pub success: bool,
    pub message: String,
}

// ===========================================================================
// Tauri Commands
// ===========================================================================

/// Set the server URL.
#[tauri::command]
async fn set_server_url(state: State<'_, Arc<AppState>>, url: String) -> Result<(), String> {
    let mut server_url = state.server_url.write().await;
    *server_url = url;
    Ok(())
}

/// Get the current server URL.
#[tauri::command]
async fn get_server_url(state: State<'_, Arc<AppState>>) -> Result<String, String> {
    let url = state.server_url.read().await.clone();
    Ok(url)
}

/// Check server health.
#[tauri::command]
async fn check_health(state: State<'_, Arc<AppState>>) -> Result<HealthResponse, String> {
    let url = state.server_url.read().await.clone();
    let response = state
        .client
        .get(format!("{url}/health"))
        .send()
        .await
        .map_err(|e| format!("Failed to connect: {e}"))?;

    response
        .json::<HealthResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {e}"))
}

/// Get server metrics.
#[tauri::command]
async fn get_metrics(state: State<'_, Arc<AppState>>) -> Result<MetricsResponse, String> {
    let url = state.server_url.read().await.clone();
    let response = state
        .client
        .get(format!("{url}/api/metrics"))
        .send()
        .await
        .map_err(|e| format!("Failed to connect: {e}"))?;

    response
        .json::<MetricsResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {e}"))
}

/// Get current configuration.
#[tauri::command]
async fn get_config(state: State<'_, Arc<AppState>>) -> Result<ConfigResponse, String> {
    let url = state.server_url.read().await.clone();
    let response = state
        .client
        .get(format!("{url}/api/config"))
        .send()
        .await
        .map_err(|e| format!("Failed to connect: {e}"))?;

    response
        .json::<ConfigResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {e}"))
}

/// Update configuration.
#[tauri::command]
async fn set_config(
    state: State<'_, Arc<AppState>>,
    config: GenerationConfigDto,
) -> Result<ConfigResponse, String> {
    let url = state.server_url.read().await.clone();
    let response = state
        .client
        .post(format!("{url}/api/config"))
        .json(&config)
        .send()
        .await
        .map_err(|e| format!("Failed to connect: {e}"))?;

    response
        .json::<ConfigResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {e}"))
}

/// Start bulk generation.
#[tauri::command]
async fn bulk_generate(
    state: State<'_, Arc<AppState>>,
    request: BulkGenerateRequest,
) -> Result<BulkGenerateResponse, String> {
    let url = state.server_url.read().await.clone();
    let response = state
        .client
        .post(format!("{url}/api/generate/bulk"))
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Failed to connect: {e}"))?;

    response
        .json::<BulkGenerateResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {e}"))
}

/// Start streaming.
#[tauri::command]
async fn start_stream(state: State<'_, Arc<AppState>>) -> Result<StreamResponse, String> {
    let url = state.server_url.read().await.clone();
    let response = state
        .client
        .post(format!("{url}/api/stream/start"))
        .json(&serde_json::json!({}))
        .send()
        .await
        .map_err(|e| format!("Failed to connect: {e}"))?;

    response
        .json::<StreamResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {e}"))
}

/// Stop streaming.
#[tauri::command]
async fn stop_stream(state: State<'_, Arc<AppState>>) -> Result<StreamResponse, String> {
    let url = state.server_url.read().await.clone();
    let response = state
        .client
        .post(format!("{url}/api/stream/stop"))
        .send()
        .await
        .map_err(|e| format!("Failed to connect: {e}"))?;

    response
        .json::<StreamResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {e}"))
}

/// Pause streaming.
#[tauri::command]
async fn pause_stream(state: State<'_, Arc<AppState>>) -> Result<StreamResponse, String> {
    let url = state.server_url.read().await.clone();
    let response = state
        .client
        .post(format!("{url}/api/stream/pause"))
        .send()
        .await
        .map_err(|e| format!("Failed to connect: {e}"))?;

    response
        .json::<StreamResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {e}"))
}

/// Resume streaming.
#[tauri::command]
async fn resume_stream(state: State<'_, Arc<AppState>>) -> Result<StreamResponse, String> {
    let url = state.server_url.read().await.clone();
    let response = state
        .client
        .post(format!("{url}/api/stream/resume"))
        .send()
        .await
        .map_err(|e| format!("Failed to connect: {e}"))?;

    response
        .json::<StreamResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {e}"))
}

/// Trigger a specific pattern.
#[tauri::command]
async fn trigger_pattern(
    state: State<'_, Arc<AppState>>,
    pattern: String,
) -> Result<StreamResponse, String> {
    let url = state.server_url.read().await.clone();
    let response = state
        .client
        .post(format!("{url}/api/stream/trigger/{pattern}"))
        .send()
        .await
        .map_err(|e| format!("Failed to connect: {e}"))?;

    response
        .json::<StreamResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {e}"))
}

fn main() {
    let app_state = Arc::new(AppState::default());

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            set_server_url,
            get_server_url,
            check_health,
            get_metrics,
            get_config,
            set_config,
            bulk_generate,
            start_stream,
            stop_stream,
            pause_stream,
            resume_stream,
            trigger_pattern,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        let url = state.server_url.blocking_read();
        assert_eq!(*url, "http://localhost:3000");
    }

    #[test]
    fn test_health_response_serialization() {
        let response = HealthResponse {
            healthy: true,
            version: "1.0.0".to_string(),
            uptime_seconds: 3600,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"healthy\":true"));
        assert!(json.contains("\"version\":\"1.0.0\""));
        assert!(json.contains("\"uptime_seconds\":3600"));
    }

    #[test]
    fn test_health_response_deserialization() {
        let json = r#"{"healthy":true,"version":"1.0.0","uptime_seconds":3600}"#;
        let response: HealthResponse = serde_json::from_str(json).unwrap();

        assert!(response.healthy);
        assert_eq!(response.version, "1.0.0");
        assert_eq!(response.uptime_seconds, 3600);
    }

    #[test]
    fn test_metrics_response_serialization() {
        let response = MetricsResponse {
            total_entries_generated: 1000,
            total_anomalies_injected: 50,
            uptime_seconds: 120,
            session_entries: 500,
            session_entries_per_second: 4.5,
            active_streams: 2,
            total_stream_events: 100,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"total_entries_generated\":1000"));
        assert!(json.contains("\"session_entries_per_second\":4.5"));
    }

    #[test]
    fn test_metrics_response_deserialization() {
        let json = r#"{
            "total_entries_generated": 1000,
            "total_anomalies_injected": 50,
            "uptime_seconds": 120,
            "session_entries": 500,
            "session_entries_per_second": 4.5,
            "active_streams": 2,
            "total_stream_events": 100
        }"#;

        let response: MetricsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.total_entries_generated, 1000);
        assert_eq!(response.session_entries_per_second, 4.5);
    }

    #[test]
    fn test_bulk_generate_request_serialization() {
        let request = BulkGenerateRequest {
            entry_count: Some(1000),
            include_master_data: Some(false),
            inject_anomalies: Some(true),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"entry_count\":1000"));
        assert!(json.contains("\"inject_anomalies\":true"));
    }

    #[test]
    fn test_bulk_generate_request_optional_fields() {
        let json = r#"{"entry_count":null,"include_master_data":null,"inject_anomalies":null}"#;
        let request: BulkGenerateRequest = serde_json::from_str(json).unwrap();

        assert!(request.entry_count.is_none());
        assert!(request.include_master_data.is_none());
        assert!(request.inject_anomalies.is_none());
    }

    #[test]
    fn test_bulk_generate_response_serialization() {
        let response = BulkGenerateResponse {
            success: true,
            entries_generated: 1000,
            duration_ms: 500,
            anomaly_count: 10,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"entries_generated\":1000"));
    }

    #[test]
    fn test_stream_response_serialization() {
        let response = StreamResponse {
            success: true,
            message: "Stream started".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"message\":\"Stream started\""));
    }

    #[test]
    fn test_generation_config_dto_serialization() {
        let config = GenerationConfigDto {
            industry: "manufacturing".to_string(),
            start_date: "2024-01-01".to_string(),
            period_months: 12,
            seed: Some(42),
            coa_complexity: "medium".to_string(),
            companies: vec![],
            fraud_enabled: false,
            fraud_rate: 0.0,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"industry\":\"manufacturing\""));
        assert!(json.contains("\"period_months\":12"));
    }

    #[test]
    fn test_company_config_dto_serialization() {
        let company = CompanyConfigDto {
            code: "1000".to_string(),
            name: "Test Company".to_string(),
            currency: "USD".to_string(),
            country: "US".to_string(),
            annual_transaction_volume: 100000,
            volume_weight: 1.0,
        };

        let json = serde_json::to_string(&company).unwrap();
        assert!(json.contains("\"code\":\"1000\""));
        assert!(json.contains("\"volume_weight\":1.0"));
    }

    #[test]
    fn test_config_response_with_config() {
        let config = GenerationConfigDto {
            industry: "retail".to_string(),
            start_date: "2024-01-01".to_string(),
            period_months: 6,
            seed: None,
            coa_complexity: "small".to_string(),
            companies: vec![],
            fraud_enabled: true,
            fraud_rate: 0.05,
        };

        let response = ConfigResponse {
            success: true,
            message: "Config loaded".to_string(),
            config: Some(config),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"industry\":\"retail\""));
    }

    #[test]
    fn test_config_response_without_config() {
        let response = ConfigResponse {
            success: false,
            message: "Config not found".to_string(),
            config: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":false"));
        assert!(json.contains("\"config\":null"));
    }

    #[test]
    fn test_server_config_urls() {
        // Test different URL formats work
        let urls = vec![
            "http://localhost:3000",
            "http://127.0.0.1:3000",
            "https://example.com",
            "http://server.local:8080",
        ];

        for url in urls {
            let state = AppState::default();
            *state.server_url.blocking_write() = url.to_string();
            let stored = state.server_url.blocking_read();
            assert_eq!(*stored, url);
        }
    }
}
