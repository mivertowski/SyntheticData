//! REST API routes.

use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{State, WebSocketUpgrade},
    http::{header, Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::timeout::TimeoutLayer;
use tracing::info;

use crate::grpc::service::{ServerState, SynthService};
use crate::jobs::{JobQueue, JobRequest};
use datasynth_runtime::{EnhancedOrchestrator, PhaseConfig};

use super::websocket;

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    pub server_state: Arc<ServerState>,
    pub job_queue: Option<Arc<JobQueue>>,
}

/// Timeout configuration for the REST API.
#[derive(Clone, Debug)]
pub struct TimeoutConfig {
    /// Request timeout in seconds.
    pub request_timeout_secs: u64,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            // 5 minutes default - bulk generation can take a while
            request_timeout_secs: 300,
        }
    }
}

impl TimeoutConfig {
    /// Create a new timeout config.
    pub fn new(timeout_secs: u64) -> Self {
        Self {
            request_timeout_secs: timeout_secs,
        }
    }
}

/// CORS configuration for the REST API.
#[derive(Clone)]
pub struct CorsConfig {
    /// Allowed origins. If empty, only localhost is allowed.
    pub allowed_origins: Vec<String>,
    /// Allow any origin (development mode only - NOT recommended for production).
    pub allow_any_origin: bool,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec![
                "http://localhost:5173".to_string(), // Vite dev server
                "http://localhost:3000".to_string(), // Common dev server
                "http://127.0.0.1:5173".to_string(),
                "http://127.0.0.1:3000".to_string(),
                "tauri://localhost".to_string(), // Tauri app
            ],
            allow_any_origin: false,
        }
    }
}

/// Add API version header to responses.
async fn api_version_header(response: axum::response::Response) -> axum::response::Response {
    let (mut parts, body) = response.into_parts();
    parts.headers.insert(
        axum::http::HeaderName::from_static("x-api-version"),
        axum::http::HeaderValue::from_static("v1"),
    );
    axum::response::Response::from_parts(parts, body)
}

use super::auth::{auth_middleware, AuthConfig};
use super::rate_limit::RateLimitConfig;
use super::rate_limit_backend::{backend_rate_limit_middleware, RateLimitBackend};
use super::request_id::request_id_middleware;
use super::request_validation::request_validation_middleware;
use super::security_headers::security_headers_middleware;

/// Create the REST API router with default CORS settings.
pub fn create_router(service: SynthService) -> Router {
    create_router_with_cors(service, CorsConfig::default())
}

/// Create the REST API router with full configuration (CORS, auth, rate limiting, and timeout).
///
/// Uses in-memory rate limiting by default. For distributed rate limiting
/// with Redis, use [`create_router_full_with_backend`] instead.
pub fn create_router_full(
    service: SynthService,
    cors_config: CorsConfig,
    auth_config: AuthConfig,
    rate_limit_config: RateLimitConfig,
    timeout_config: TimeoutConfig,
) -> Router {
    let backend = RateLimitBackend::in_memory(rate_limit_config);
    create_router_full_with_backend(service, cors_config, auth_config, backend, timeout_config)
}

/// Create the REST API router with full configuration and a specific rate limiting backend.
///
/// This allows using either in-memory or Redis-backed rate limiting.
///
/// # Example (in-memory)
/// ```rust,ignore
/// let backend = RateLimitBackend::in_memory(rate_limit_config);
/// let router = create_router_full_with_backend(service, cors, auth, backend, timeout);
/// ```
///
/// # Example (Redis)
/// ```rust,ignore
/// let backend = RateLimitBackend::redis("redis://127.0.0.1:6379", rate_limit_config).await?;
/// let router = create_router_full_with_backend(service, cors, auth, backend, timeout);
/// ```
pub fn create_router_full_with_backend(
    service: SynthService,
    cors_config: CorsConfig,
    auth_config: AuthConfig,
    rate_limit_backend: RateLimitBackend,
    timeout_config: TimeoutConfig,
) -> Router {
    let server_state = service.state.clone();
    let state = AppState {
        server_state,
        job_queue: None,
    };

    let cors = if cors_config.allow_any_origin {
        CorsLayer::permissive()
    } else {
        let origins: Vec<_> = cors_config
            .allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();

        CorsLayer::new()
            .allow_origin(AllowOrigin::list(origins))
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::ACCEPT])
    };

    Router::new()
        // Health and metrics (exempt from auth and rate limiting by default)
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        .route("/live", get(liveness_check))
        .route("/api/metrics", get(get_metrics))
        .route("/metrics", get(prometheus_metrics))
        // Configuration
        .route("/api/config", get(get_config))
        .route("/api/config", post(set_config))
        .route("/api/config/reload", post(reload_config))
        // Generation
        .route("/api/generate/bulk", post(bulk_generate))
        .route("/api/stream/start", post(start_stream))
        .route("/api/stream/stop", post(stop_stream))
        .route("/api/stream/pause", post(pause_stream))
        .route("/api/stream/resume", post(resume_stream))
        .route("/api/stream/trigger/{pattern}", post(trigger_pattern))
        // Jobs
        .route("/api/jobs/submit", post(submit_job))
        .route("/api/jobs", get(list_jobs))
        .route("/api/jobs/{id}", get(get_job))
        .route("/api/jobs/{id}/cancel", post(cancel_job))
        // WebSocket
        .route("/ws/metrics", get(websocket_metrics))
        .route("/ws/events", get(websocket_events))
        // Middleware stack (outermost applied first, innermost last)
        // Order: Timeout -> RateLimit -> RequestValidation -> Auth -> RequestId -> CORS -> SecurityHeaders -> APIVersion -> Router
        .layer(axum::middleware::from_fn(security_headers_middleware))
        .layer(axum::middleware::map_response(api_version_header))
        .layer(cors)
        .layer(axum::middleware::from_fn(request_id_middleware))
        .layer(axum::middleware::from_fn(auth_middleware))
        .layer(axum::Extension(auth_config))
        .layer(axum::middleware::from_fn(request_validation_middleware))
        .layer(axum::middleware::from_fn(backend_rate_limit_middleware))
        .layer(axum::Extension(rate_limit_backend))
        .layer(TimeoutLayer::new(Duration::from_secs(
            timeout_config.request_timeout_secs,
        )))
        .with_state(state)
}

/// Create the REST API router with custom CORS and authentication settings.
pub fn create_router_with_auth(
    service: SynthService,
    cors_config: CorsConfig,
    auth_config: AuthConfig,
) -> Router {
    let server_state = service.state.clone();
    let state = AppState {
        server_state,
        job_queue: None,
    };

    let cors = if cors_config.allow_any_origin {
        CorsLayer::permissive()
    } else {
        let origins: Vec<_> = cors_config
            .allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();

        CorsLayer::new()
            .allow_origin(AllowOrigin::list(origins))
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::ACCEPT])
    };

    Router::new()
        // Health and metrics (exempt from auth by default)
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        .route("/live", get(liveness_check))
        .route("/api/metrics", get(get_metrics))
        .route("/metrics", get(prometheus_metrics))
        // Configuration
        .route("/api/config", get(get_config))
        .route("/api/config", post(set_config))
        .route("/api/config/reload", post(reload_config))
        // Generation
        .route("/api/generate/bulk", post(bulk_generate))
        .route("/api/stream/start", post(start_stream))
        .route("/api/stream/stop", post(stop_stream))
        .route("/api/stream/pause", post(pause_stream))
        .route("/api/stream/resume", post(resume_stream))
        .route("/api/stream/trigger/{pattern}", post(trigger_pattern))
        // Jobs
        .route("/api/jobs/submit", post(submit_job))
        .route("/api/jobs", get(list_jobs))
        .route("/api/jobs/{id}", get(get_job))
        .route("/api/jobs/{id}/cancel", post(cancel_job))
        // WebSocket
        .route("/ws/metrics", get(websocket_metrics))
        .route("/ws/events", get(websocket_events))
        .layer(axum::middleware::from_fn(auth_middleware))
        .layer(axum::Extension(auth_config))
        .layer(cors)
        .with_state(state)
}

/// Create the REST API router with custom CORS settings.
pub fn create_router_with_cors(service: SynthService, cors_config: CorsConfig) -> Router {
    let server_state = service.state.clone();
    let state = AppState {
        server_state,
        job_queue: None,
    };

    let cors = if cors_config.allow_any_origin {
        // Development mode - allow any origin (use with caution)
        CorsLayer::permissive()
    } else {
        // Production mode - restricted origins
        let origins: Vec<_> = cors_config
            .allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();

        CorsLayer::new()
            .allow_origin(AllowOrigin::list(origins))
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::ACCEPT])
    };

    Router::new()
        // Health and metrics
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        .route("/live", get(liveness_check))
        .route("/api/metrics", get(get_metrics))
        .route("/metrics", get(prometheus_metrics))
        // Configuration
        .route("/api/config", get(get_config))
        .route("/api/config", post(set_config))
        .route("/api/config/reload", post(reload_config))
        // Generation
        .route("/api/generate/bulk", post(bulk_generate))
        .route("/api/stream/start", post(start_stream))
        .route("/api/stream/stop", post(stop_stream))
        .route("/api/stream/pause", post(pause_stream))
        .route("/api/stream/resume", post(resume_stream))
        .route("/api/stream/trigger/{pattern}", post(trigger_pattern))
        // Jobs
        .route("/api/jobs/submit", post(submit_job))
        .route("/api/jobs", get(list_jobs))
        .route("/api/jobs/{id}", get(get_job))
        .route("/api/jobs/{id}/cancel", post(cancel_job))
        // WebSocket
        .route("/ws/metrics", get(websocket_metrics))
        .route("/ws/events", get(websocket_events))
        .layer(cors)
        .with_state(state)
}

// ===========================================================================
// Request/Response types
// ===========================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub healthy: bool,
    pub version: String,
    pub uptime_seconds: u64,
}

/// Readiness check response for Kubernetes.
#[derive(Debug, Serialize, Deserialize)]
pub struct ReadinessResponse {
    pub ready: bool,
    pub message: String,
    pub checks: Vec<HealthCheck>,
}

/// Individual health check result.
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: String,
}

/// Liveness check response for Kubernetes.
#[derive(Debug, Serialize, Deserialize)]
pub struct LivenessResponse {
    pub alive: bool,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsResponse {
    pub total_entries_generated: u64,
    pub total_anomalies_injected: u64,
    pub uptime_seconds: u64,
    pub session_entries: u64,
    pub session_entries_per_second: f64,
    pub active_streams: u32,
    pub total_stream_events: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub success: bool,
    pub message: String,
    pub config: Option<GenerationConfigDto>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyConfigDto {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub country: String,
    pub annual_transaction_volume: u64,
    pub volume_weight: f32,
}

#[derive(Debug, Deserialize)]
pub struct BulkGenerateRequest {
    pub entry_count: Option<u64>,
    pub include_master_data: Option<bool>,
    pub inject_anomalies: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct BulkGenerateResponse {
    pub success: bool,
    pub entries_generated: u64,
    pub duration_ms: u64,
    pub anomaly_count: u64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields deserialized from request, reserved for future use
pub struct StreamRequest {
    pub events_per_second: Option<u32>,
    pub max_events: Option<u64>,
    pub inject_anomalies: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct StreamResponse {
    pub success: bool,
    pub message: String,
}

// ===========================================================================
// Handlers
// ===========================================================================

/// Health check endpoint - returns overall health status.
async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        healthy: true,
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: state.server_state.uptime_seconds(),
    })
}

/// Readiness probe - indicates the service is ready to accept traffic.
/// Use for Kubernetes readiness probes.
async fn readiness_check(
    State(state): State<AppState>,
) -> Result<Json<ReadinessResponse>, (StatusCode, Json<ReadinessResponse>)> {
    let mut checks = Vec::new();
    let mut any_fail = false;

    // Check if configuration is loaded and valid
    let config = state.server_state.config.read().await;
    let config_valid = !config.companies.is_empty();
    checks.push(HealthCheck {
        name: "config".to_string(),
        status: if config_valid { "ok" } else { "fail" }.to_string(),
    });
    if !config_valid {
        any_fail = true;
    }
    drop(config);

    // Check resource guard (memory)
    let resource_status = state.server_state.resource_status();
    let memory_status = if resource_status.degradation_level == "Emergency" {
        any_fail = true;
        "fail"
    } else if resource_status.degradation_level != "Normal" {
        "degraded"
    } else {
        "ok"
    };
    checks.push(HealthCheck {
        name: "memory".to_string(),
        status: memory_status.to_string(),
    });

    // Check disk (>100MB free)
    let disk_ok = resource_status.disk_available_mb > 100;
    checks.push(HealthCheck {
        name: "disk".to_string(),
        status: if disk_ok { "ok" } else { "fail" }.to_string(),
    });
    if !disk_ok {
        any_fail = true;
    }

    let response = ReadinessResponse {
        ready: !any_fail,
        message: if any_fail {
            "Service is not ready".to_string()
        } else {
            "Service is ready".to_string()
        },
        checks,
    };

    if any_fail {
        Err((StatusCode::SERVICE_UNAVAILABLE, Json(response)))
    } else {
        Ok(Json(response))
    }
}

/// Liveness probe - indicates the service is alive.
/// Use for Kubernetes liveness probes.
async fn liveness_check() -> Json<LivenessResponse> {
    Json(LivenessResponse {
        alive: true,
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

/// Prometheus-compatible metrics endpoint.
/// Returns metrics in Prometheus text exposition format.
async fn prometheus_metrics(State(state): State<AppState>) -> impl IntoResponse {
    use std::sync::atomic::Ordering;

    let uptime = state.server_state.uptime_seconds();
    let total_entries = state.server_state.total_entries.load(Ordering::Relaxed);
    let total_anomalies = state.server_state.total_anomalies.load(Ordering::Relaxed);
    let active_streams = state.server_state.active_streams.load(Ordering::Relaxed);
    let total_stream_events = state
        .server_state
        .total_stream_events
        .load(Ordering::Relaxed);

    let entries_per_second = if uptime > 0 {
        total_entries as f64 / uptime as f64
    } else {
        0.0
    };

    let metrics = format!(
        r#"# HELP synth_entries_generated_total Total number of journal entries generated
# TYPE synth_entries_generated_total counter
synth_entries_generated_total {}

# HELP synth_anomalies_injected_total Total number of anomalies injected
# TYPE synth_anomalies_injected_total counter
synth_anomalies_injected_total {}

# HELP synth_uptime_seconds Server uptime in seconds
# TYPE synth_uptime_seconds gauge
synth_uptime_seconds {}

# HELP synth_entries_per_second Rate of entry generation
# TYPE synth_entries_per_second gauge
synth_entries_per_second {:.2}

# HELP synth_active_streams Number of active streaming connections
# TYPE synth_active_streams gauge
synth_active_streams {}

# HELP synth_stream_events_total Total events sent through streams
# TYPE synth_stream_events_total counter
synth_stream_events_total {}

# HELP synth_info Server version information
# TYPE synth_info gauge
synth_info{{version="{}"}} 1
"#,
        total_entries,
        total_anomalies,
        uptime,
        entries_per_second,
        active_streams,
        total_stream_events,
        env!("CARGO_PKG_VERSION")
    );

    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        metrics,
    )
}

/// Get server metrics.
async fn get_metrics(State(state): State<AppState>) -> Json<MetricsResponse> {
    let uptime = state.server_state.uptime_seconds();
    let total_entries = state
        .server_state
        .total_entries
        .load(std::sync::atomic::Ordering::Relaxed);

    let entries_per_second = if uptime > 0 {
        total_entries as f64 / uptime as f64
    } else {
        0.0
    };

    Json(MetricsResponse {
        total_entries_generated: total_entries,
        total_anomalies_injected: state
            .server_state
            .total_anomalies
            .load(std::sync::atomic::Ordering::Relaxed),
        uptime_seconds: uptime,
        session_entries: total_entries,
        session_entries_per_second: entries_per_second,
        active_streams: state
            .server_state
            .active_streams
            .load(std::sync::atomic::Ordering::Relaxed) as u32,
        total_stream_events: state
            .server_state
            .total_stream_events
            .load(std::sync::atomic::Ordering::Relaxed),
    })
}

/// Get current configuration.
async fn get_config(State(state): State<AppState>) -> Json<ConfigResponse> {
    let config = state.server_state.config.read().await;

    Json(ConfigResponse {
        success: true,
        message: "Current configuration".to_string(),
        config: Some(GenerationConfigDto {
            industry: format!("{:?}", config.global.industry),
            start_date: config.global.start_date.clone(),
            period_months: config.global.period_months,
            seed: config.global.seed,
            coa_complexity: format!("{:?}", config.chart_of_accounts.complexity),
            companies: config
                .companies
                .iter()
                .map(|c| CompanyConfigDto {
                    code: c.code.clone(),
                    name: c.name.clone(),
                    currency: c.currency.clone(),
                    country: c.country.clone(),
                    annual_transaction_volume: c.annual_transaction_volume.count(),
                    volume_weight: c.volume_weight as f32,
                })
                .collect(),
            fraud_enabled: config.fraud.enabled,
            fraud_rate: config.fraud.fraud_rate as f32,
        }),
    })
}

/// Set configuration.
async fn set_config(
    State(state): State<AppState>,
    Json(new_config): Json<GenerationConfigDto>,
) -> Result<Json<ConfigResponse>, (StatusCode, Json<ConfigResponse>)> {
    use datasynth_config::schema::{CompanyConfig, TransactionVolume};
    use datasynth_core::models::{CoAComplexity, IndustrySector};

    info!(
        "Configuration update requested: industry={}, period_months={}",
        new_config.industry, new_config.period_months
    );

    // Parse industry from string
    let industry = match new_config.industry.to_lowercase().as_str() {
        "manufacturing" => IndustrySector::Manufacturing,
        "retail" => IndustrySector::Retail,
        "financial_services" | "financialservices" => IndustrySector::FinancialServices,
        "healthcare" => IndustrySector::Healthcare,
        "technology" => IndustrySector::Technology,
        "professional_services" | "professionalservices" => IndustrySector::ProfessionalServices,
        "energy" => IndustrySector::Energy,
        "transportation" => IndustrySector::Transportation,
        "real_estate" | "realestate" => IndustrySector::RealEstate,
        "telecommunications" => IndustrySector::Telecommunications,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConfigResponse {
                    success: false,
                    message: format!("Unknown industry: '{}'. Valid values: manufacturing, retail, financial_services, healthcare, technology, professional_services, energy, transportation, real_estate, telecommunications", new_config.industry),
                    config: None,
                }),
            ));
        }
    };

    // Parse CoA complexity from string
    let complexity = match new_config.coa_complexity.to_lowercase().as_str() {
        "small" => CoAComplexity::Small,
        "medium" => CoAComplexity::Medium,
        "large" => CoAComplexity::Large,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ConfigResponse {
                    success: false,
                    message: format!(
                        "Unknown CoA complexity: '{}'. Valid values: small, medium, large",
                        new_config.coa_complexity
                    ),
                    config: None,
                }),
            ));
        }
    };

    // Convert CompanyConfigDto to CompanyConfig
    let companies: Vec<CompanyConfig> = new_config
        .companies
        .iter()
        .map(|c| CompanyConfig {
            code: c.code.clone(),
            name: c.name.clone(),
            currency: c.currency.clone(),
            country: c.country.clone(),
            fiscal_year_variant: "K4".to_string(),
            annual_transaction_volume: TransactionVolume::Custom(c.annual_transaction_volume),
            volume_weight: c.volume_weight as f64,
        })
        .collect();

    // Update the configuration
    let mut config = state.server_state.config.write().await;
    config.global.industry = industry;
    config.global.start_date = new_config.start_date.clone();
    config.global.period_months = new_config.period_months;
    config.global.seed = new_config.seed;
    config.chart_of_accounts.complexity = complexity;
    config.fraud.enabled = new_config.fraud_enabled;
    config.fraud.fraud_rate = new_config.fraud_rate as f64;

    // Only update companies if provided
    if !companies.is_empty() {
        config.companies = companies;
    }

    info!("Configuration updated successfully");

    Ok(Json(ConfigResponse {
        success: true,
        message: "Configuration updated and applied".to_string(),
        config: Some(new_config),
    }))
}

/// Bulk generation endpoint.
async fn bulk_generate(
    State(state): State<AppState>,
    Json(req): Json<BulkGenerateRequest>,
) -> Result<Json<BulkGenerateResponse>, (StatusCode, String)> {
    // Validate entry_count bounds
    const MAX_ENTRY_COUNT: u64 = 1_000_000;
    if let Some(count) = req.entry_count {
        if count > MAX_ENTRY_COUNT {
            return Err((
                StatusCode::BAD_REQUEST,
                format!(
                    "entry_count ({}) exceeds maximum allowed value ({})",
                    count, MAX_ENTRY_COUNT
                ),
            ));
        }
    }

    let config = state.server_state.config.read().await.clone();
    let start_time = std::time::Instant::now();

    let phase_config = PhaseConfig {
        generate_master_data: req.include_master_data.unwrap_or(false),
        generate_document_flows: false,
        generate_journal_entries: true,
        inject_anomalies: req.inject_anomalies.unwrap_or(false),
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator = EnhancedOrchestrator::new(config, phase_config).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create orchestrator: {}", e),
        )
    })?;

    let result = orchestrator.generate().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Generation failed: {}", e),
        )
    })?;

    let duration_ms = start_time.elapsed().as_millis() as u64;
    let entries_count = result.journal_entries.len() as u64;
    let anomaly_count = result.anomaly_labels.labels.len() as u64;

    // Update metrics
    state
        .server_state
        .total_entries
        .fetch_add(entries_count, std::sync::atomic::Ordering::Relaxed);
    state
        .server_state
        .total_anomalies
        .fetch_add(anomaly_count, std::sync::atomic::Ordering::Relaxed);

    Ok(Json(BulkGenerateResponse {
        success: true,
        entries_generated: entries_count,
        duration_ms,
        anomaly_count,
    }))
}

/// Start streaming.
async fn start_stream(
    State(state): State<AppState>,
    Json(_req): Json<StreamRequest>,
) -> Json<StreamResponse> {
    state
        .server_state
        .stream_stopped
        .store(false, std::sync::atomic::Ordering::Relaxed);
    state
        .server_state
        .stream_paused
        .store(false, std::sync::atomic::Ordering::Relaxed);

    Json(StreamResponse {
        success: true,
        message: "Stream started".to_string(),
    })
}

/// Stop streaming.
async fn stop_stream(State(state): State<AppState>) -> Json<StreamResponse> {
    state
        .server_state
        .stream_stopped
        .store(true, std::sync::atomic::Ordering::Relaxed);

    Json(StreamResponse {
        success: true,
        message: "Stream stopped".to_string(),
    })
}

/// Pause streaming.
async fn pause_stream(State(state): State<AppState>) -> Json<StreamResponse> {
    state
        .server_state
        .stream_paused
        .store(true, std::sync::atomic::Ordering::Relaxed);

    Json(StreamResponse {
        success: true,
        message: "Stream paused".to_string(),
    })
}

/// Resume streaming.
async fn resume_stream(State(state): State<AppState>) -> Json<StreamResponse> {
    state
        .server_state
        .stream_paused
        .store(false, std::sync::atomic::Ordering::Relaxed);

    Json(StreamResponse {
        success: true,
        message: "Stream resumed".to_string(),
    })
}

/// Trigger a specific pattern.
///
/// Valid patterns: year_end_spike, period_end_spike, holiday_cluster,
/// fraud_cluster, error_cluster, uniform, or custom:* patterns.
async fn trigger_pattern(
    State(state): State<AppState>,
    axum::extract::Path(pattern): axum::extract::Path<String>,
) -> Json<StreamResponse> {
    info!("Pattern trigger requested: {}", pattern);

    // Validate pattern name
    let valid_patterns = [
        "year_end_spike",
        "period_end_spike",
        "holiday_cluster",
        "fraud_cluster",
        "error_cluster",
        "uniform",
    ];

    let is_valid = valid_patterns.contains(&pattern.as_str()) || pattern.starts_with("custom:");

    if !is_valid {
        return Json(StreamResponse {
            success: false,
            message: format!(
                "Unknown pattern '{}'. Valid patterns: {:?}, or use 'custom:name' for custom patterns",
                pattern, valid_patterns
            ),
        });
    }

    // Store the pattern for the stream generator to pick up
    match state.server_state.triggered_pattern.try_write() {
        Ok(mut triggered) => {
            *triggered = Some(pattern.clone());
            Json(StreamResponse {
                success: true,
                message: format!("Pattern '{}' will be applied to upcoming entries", pattern),
            })
        }
        Err(_) => Json(StreamResponse {
            success: false,
            message: "Failed to acquire lock for pattern trigger".to_string(),
        }),
    }
}

/// WebSocket endpoint for metrics stream.
async fn websocket_metrics(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket::handle_metrics_socket(socket, state))
}

/// WebSocket endpoint for event stream.
async fn websocket_events(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket::handle_events_socket(socket, state))
}

// ===========================================================================
// Job Queue Handlers
// ===========================================================================

/// Submit a new async generation job.
async fn submit_job(
    State(state): State<AppState>,
    Json(request): Json<JobRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let queue = state.job_queue.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "Job queue not enabled"})),
        )
    })?;

    let job_id = queue.submit(request).await;
    info!("Job submitted: {}", job_id);

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "id": job_id.to_string(),
            "status": "queued"
        })),
    ))
}

/// Get status of a specific job.
async fn get_job(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let queue = state.job_queue.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "Job queue not enabled"})),
        )
    })?;

    match queue.get(&id).await {
        Some(entry) => Ok(Json(serde_json::json!({
            "id": entry.id,
            "status": format!("{:?}", entry.status).to_lowercase(),
            "submitted_at": entry.submitted_at.to_rfc3339(),
            "started_at": entry.started_at.map(|t| t.to_rfc3339()),
            "completed_at": entry.completed_at.map(|t| t.to_rfc3339()),
            "result": entry.result,
        }))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Job not found"})),
        )),
    }
}

/// List all jobs.
async fn list_jobs(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let queue = state.job_queue.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "Job queue not enabled"})),
        )
    })?;

    let summaries: Vec<_> = queue
        .list()
        .await
        .into_iter()
        .map(|s| {
            serde_json::json!({
                "id": s.id,
                "status": format!("{:?}", s.status).to_lowercase(),
                "submitted_at": s.submitted_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(serde_json::json!({ "jobs": summaries })))
}

/// Cancel a queued job.
async fn cancel_job(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let queue = state.job_queue.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "Job queue not enabled"})),
        )
    })?;

    if queue.cancel(&id).await {
        Ok(Json(serde_json::json!({"id": id, "status": "cancelled"})))
    } else {
        Err((
            StatusCode::CONFLICT,
            Json(
                serde_json::json!({"error": "Job cannot be cancelled (not in queued state or not found)"}),
            ),
        ))
    }
}

// ===========================================================================
// Config Reload Handler
// ===========================================================================

/// Reload configuration from the configured source.
async fn reload_config(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Reload from default config source
    let new_config = crate::grpc::service::default_generator_config();
    let mut config = state.server_state.config.write().await;
    *config = new_config;
    info!("Configuration reloaded via REST API");

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Configuration reloaded"
    })))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // ==========================================================================
    // Response Serialization Tests
    // ==========================================================================

    #[test]
    fn test_health_response_serialization() {
        let response = HealthResponse {
            healthy: true,
            version: "0.1.0".to_string(),
            uptime_seconds: 100,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("healthy"));
        assert!(json.contains("version"));
        assert!(json.contains("uptime_seconds"));
    }

    #[test]
    fn test_health_response_deserialization() {
        let json = r#"{"healthy":true,"version":"0.1.0","uptime_seconds":100}"#;
        let response: HealthResponse = serde_json::from_str(json).unwrap();
        assert!(response.healthy);
        assert_eq!(response.version, "0.1.0");
        assert_eq!(response.uptime_seconds, 100);
    }

    #[test]
    fn test_metrics_response_serialization() {
        let response = MetricsResponse {
            total_entries_generated: 1000,
            total_anomalies_injected: 10,
            uptime_seconds: 60,
            session_entries: 1000,
            session_entries_per_second: 16.67,
            active_streams: 1,
            total_stream_events: 500,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("total_entries_generated"));
        assert!(json.contains("session_entries_per_second"));
    }

    #[test]
    fn test_metrics_response_deserialization() {
        let json = r#"{
            "total_entries_generated": 5000,
            "total_anomalies_injected": 50,
            "uptime_seconds": 300,
            "session_entries": 5000,
            "session_entries_per_second": 16.67,
            "active_streams": 2,
            "total_stream_events": 10000
        }"#;
        let response: MetricsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.total_entries_generated, 5000);
        assert_eq!(response.active_streams, 2);
    }

    #[test]
    fn test_config_response_serialization() {
        let response = ConfigResponse {
            success: true,
            message: "Configuration loaded".to_string(),
            config: Some(GenerationConfigDto {
                industry: "manufacturing".to_string(),
                start_date: "2024-01-01".to_string(),
                period_months: 12,
                seed: Some(42),
                coa_complexity: "medium".to_string(),
                companies: vec![],
                fraud_enabled: false,
                fraud_rate: 0.0,
            }),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("success"));
        assert!(json.contains("config"));
    }

    #[test]
    fn test_config_response_without_config() {
        let response = ConfigResponse {
            success: false,
            message: "No configuration available".to_string(),
            config: None,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("null") || json.contains("config\":null"));
    }

    #[test]
    fn test_generation_config_dto_roundtrip() {
        let original = GenerationConfigDto {
            industry: "retail".to_string(),
            start_date: "2024-06-01".to_string(),
            period_months: 6,
            seed: Some(12345),
            coa_complexity: "large".to_string(),
            companies: vec![CompanyConfigDto {
                code: "1000".to_string(),
                name: "Test Corp".to_string(),
                currency: "USD".to_string(),
                country: "US".to_string(),
                annual_transaction_volume: 100000,
                volume_weight: 1.0,
            }],
            fraud_enabled: true,
            fraud_rate: 0.05,
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: GenerationConfigDto = serde_json::from_str(&json).unwrap();

        assert_eq!(original.industry, deserialized.industry);
        assert_eq!(original.seed, deserialized.seed);
        assert_eq!(original.companies.len(), deserialized.companies.len());
    }

    #[test]
    fn test_company_config_dto_serialization() {
        let company = CompanyConfigDto {
            code: "2000".to_string(),
            name: "European Subsidiary".to_string(),
            currency: "EUR".to_string(),
            country: "DE".to_string(),
            annual_transaction_volume: 50000,
            volume_weight: 0.5,
        };
        let json = serde_json::to_string(&company).unwrap();
        assert!(json.contains("2000"));
        assert!(json.contains("EUR"));
        assert!(json.contains("DE"));
    }

    #[test]
    fn test_bulk_generate_request_deserialization() {
        let json = r#"{
            "entry_count": 5000,
            "include_master_data": true,
            "inject_anomalies": true
        }"#;
        let request: BulkGenerateRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.entry_count, Some(5000));
        assert_eq!(request.include_master_data, Some(true));
        assert_eq!(request.inject_anomalies, Some(true));
    }

    #[test]
    fn test_bulk_generate_request_with_defaults() {
        let json = r#"{}"#;
        let request: BulkGenerateRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.entry_count, None);
        assert_eq!(request.include_master_data, None);
        assert_eq!(request.inject_anomalies, None);
    }

    #[test]
    fn test_bulk_generate_response_serialization() {
        let response = BulkGenerateResponse {
            success: true,
            entries_generated: 1000,
            duration_ms: 250,
            anomaly_count: 20,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("entries_generated"));
        assert!(json.contains("1000"));
        assert!(json.contains("duration_ms"));
    }

    #[test]
    fn test_stream_response_serialization() {
        let response = StreamResponse {
            success: true,
            message: "Stream started successfully".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("success"));
        assert!(json.contains("Stream started"));
    }

    #[test]
    fn test_stream_response_failure() {
        let response = StreamResponse {
            success: false,
            message: "Stream failed to start".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("false"));
        assert!(json.contains("failed"));
    }

    // ==========================================================================
    // CORS Configuration Tests
    // ==========================================================================

    #[test]
    fn test_cors_config_default() {
        let config = CorsConfig::default();
        assert!(!config.allow_any_origin);
        assert!(!config.allowed_origins.is_empty());
        assert!(config
            .allowed_origins
            .contains(&"http://localhost:5173".to_string()));
        assert!(config
            .allowed_origins
            .contains(&"tauri://localhost".to_string()));
    }

    #[test]
    fn test_cors_config_custom_origins() {
        let config = CorsConfig {
            allowed_origins: vec![
                "https://example.com".to_string(),
                "https://app.example.com".to_string(),
            ],
            allow_any_origin: false,
        };
        assert_eq!(config.allowed_origins.len(), 2);
        assert!(config
            .allowed_origins
            .contains(&"https://example.com".to_string()));
    }

    #[test]
    fn test_cors_config_permissive() {
        let config = CorsConfig {
            allowed_origins: vec![],
            allow_any_origin: true,
        };
        assert!(config.allow_any_origin);
    }

    // ==========================================================================
    // Request Validation Tests (edge cases)
    // ==========================================================================

    #[test]
    fn test_bulk_generate_request_partial() {
        let json = r#"{"entry_count": 100}"#;
        let request: BulkGenerateRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.entry_count, Some(100));
        assert!(request.include_master_data.is_none());
    }

    #[test]
    fn test_generation_config_no_seed() {
        let config = GenerationConfigDto {
            industry: "technology".to_string(),
            start_date: "2024-01-01".to_string(),
            period_months: 3,
            seed: None,
            coa_complexity: "small".to_string(),
            companies: vec![],
            fraud_enabled: false,
            fraud_rate: 0.0,
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("seed"));
    }

    #[test]
    fn test_generation_config_multiple_companies() {
        let config = GenerationConfigDto {
            industry: "manufacturing".to_string(),
            start_date: "2024-01-01".to_string(),
            period_months: 12,
            seed: Some(42),
            coa_complexity: "large".to_string(),
            companies: vec![
                CompanyConfigDto {
                    code: "1000".to_string(),
                    name: "Headquarters".to_string(),
                    currency: "USD".to_string(),
                    country: "US".to_string(),
                    annual_transaction_volume: 100000,
                    volume_weight: 1.0,
                },
                CompanyConfigDto {
                    code: "2000".to_string(),
                    name: "European Sub".to_string(),
                    currency: "EUR".to_string(),
                    country: "DE".to_string(),
                    annual_transaction_volume: 50000,
                    volume_weight: 0.5,
                },
                CompanyConfigDto {
                    code: "3000".to_string(),
                    name: "APAC Sub".to_string(),
                    currency: "JPY".to_string(),
                    country: "JP".to_string(),
                    annual_transaction_volume: 30000,
                    volume_weight: 0.3,
                },
            ],
            fraud_enabled: true,
            fraud_rate: 0.02,
        };
        assert_eq!(config.companies.len(), 3);
    }

    // ==========================================================================
    // Metrics Calculation Tests
    // ==========================================================================

    #[test]
    fn test_metrics_entries_per_second_calculation() {
        // Test that we can represent the expected calculation
        let total_entries: u64 = 1000;
        let uptime: u64 = 60;
        let eps = if uptime > 0 {
            total_entries as f64 / uptime as f64
        } else {
            0.0
        };
        assert!((eps - 16.67).abs() < 0.1);
    }

    #[test]
    fn test_metrics_entries_per_second_zero_uptime() {
        let total_entries: u64 = 1000;
        let uptime: u64 = 0;
        let eps = if uptime > 0 {
            total_entries as f64 / uptime as f64
        } else {
            0.0
        };
        assert_eq!(eps, 0.0);
    }
}
