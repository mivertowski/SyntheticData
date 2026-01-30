//! gRPC service implementation.

use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Utc;
use futures::Stream;
use prost_types::Timestamp;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::{error, info, warn};

use datasynth_config::schema::{
    ChartOfAccountsConfig, CompanyConfig, GeneratorConfig, GlobalConfig, OutputConfig,
    TransactionVolume,
};
use datasynth_core::models::{CoAComplexity, IndustrySector, JournalEntry};
use datasynth_core::{DegradationLevel, ResourceGuard, ResourceGuardBuilder};
use datasynth_runtime::{EnhancedOrchestrator, PhaseConfig};

use super::synth::*;

/// Server state for tracking metrics and configuration.
pub struct ServerState {
    /// Current configuration
    pub config: RwLock<GeneratorConfig>,
    /// Server start time
    start_time: Instant,
    /// Total entries generated
    pub total_entries: AtomicU64,
    /// Total anomalies injected
    pub total_anomalies: AtomicU64,
    /// Active streams count
    pub active_streams: AtomicU64,
    /// Total stream events
    pub total_stream_events: AtomicU64,
    /// Stream control flag
    pub stream_paused: AtomicBool,
    /// Stream stop flag
    pub stream_stopped: AtomicBool,
    /// Triggered pattern name (if any) - will be applied to next generated entries
    pub triggered_pattern: RwLock<Option<String>>,
    /// Resource guard for memory and disk monitoring
    pub resource_guard: Arc<ResourceGuard>,
    /// Maximum concurrent generations allowed
    max_concurrent_generations: AtomicU64,
}

impl ServerState {
    pub fn new(config: GeneratorConfig) -> Self {
        // Build resource guard from config
        let memory_limit = config.global.memory_limit_mb;
        let resource_guard = if memory_limit > 0 {
            ResourceGuardBuilder::new()
                .memory_limit(memory_limit)
                .conservative()
                .build()
        } else {
            // Default: 2GB limit for server
            ResourceGuardBuilder::new()
                .memory_limit(2048)
                .conservative()
                .build()
        };

        Self {
            config: RwLock::new(config),
            start_time: Instant::now(),
            total_entries: AtomicU64::new(0),
            total_anomalies: AtomicU64::new(0),
            active_streams: AtomicU64::new(0),
            total_stream_events: AtomicU64::new(0),
            stream_paused: AtomicBool::new(false),
            stream_stopped: AtomicBool::new(false),
            triggered_pattern: RwLock::new(None),
            resource_guard: Arc::new(resource_guard),
            max_concurrent_generations: AtomicU64::new(4),
        }
    }

    /// Create with custom resource limits.
    pub fn with_resource_limits(config: GeneratorConfig, memory_limit_mb: usize) -> Self {
        let resource_guard = ResourceGuardBuilder::new()
            .memory_limit(memory_limit_mb)
            .conservative()
            .build();

        Self {
            config: RwLock::new(config),
            start_time: Instant::now(),
            total_entries: AtomicU64::new(0),
            total_anomalies: AtomicU64::new(0),
            active_streams: AtomicU64::new(0),
            total_stream_events: AtomicU64::new(0),
            stream_paused: AtomicBool::new(false),
            stream_stopped: AtomicBool::new(false),
            triggered_pattern: RwLock::new(None),
            resource_guard: Arc::new(resource_guard),
            max_concurrent_generations: AtomicU64::new(4),
        }
    }

    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Check if resources are available for a new generation.
    #[allow(clippy::result_large_err)] // tonic::Status is the idiomatic error type for gRPC
    pub fn check_resources(&self) -> Result<DegradationLevel, Status> {
        // Check if too many concurrent generations
        let active = self.active_streams.load(Ordering::Relaxed);
        let max = self.max_concurrent_generations.load(Ordering::Relaxed);
        if active >= max {
            return Err(Status::resource_exhausted(format!(
                "Too many concurrent generations ({}/{}). Try again later.",
                active, max
            )));
        }

        // Check memory and other resources
        match self.resource_guard.check() {
            Ok(level) => {
                if level == DegradationLevel::Emergency {
                    Err(Status::resource_exhausted(
                        "Server resources critically low. Generation not possible.",
                    ))
                } else if level == DegradationLevel::Minimal {
                    warn!("Resources constrained, generation may be limited");
                    Ok(level)
                } else {
                    Ok(level)
                }
            }
            Err(e) => Err(Status::resource_exhausted(format!(
                "Resource check failed: {}",
                e
            ))),
        }
    }

    /// Get current resource status for monitoring.
    pub fn resource_status(&self) -> ResourceStatus {
        let stats = self.resource_guard.stats();
        ResourceStatus {
            memory_usage_mb: stats.memory.resident_bytes / (1024 * 1024),
            memory_peak_mb: stats.memory.peak_resident_bytes / (1024 * 1024),
            disk_available_mb: stats.disk.available_bytes / (1024 * 1024),
            degradation_level: stats.degradation_level.name().to_string(),
            active_generations: self.active_streams.load(Ordering::Relaxed),
        }
    }
}

/// Resource status for monitoring endpoints.
#[derive(Debug, Clone)]
pub struct ResourceStatus {
    pub memory_usage_mb: u64,
    pub memory_peak_mb: u64,
    pub disk_available_mb: u64,
    pub degradation_level: String,
    pub active_generations: u64,
}

/// Main gRPC service implementation.
pub struct SynthService {
    pub state: Arc<ServerState>,
}

impl SynthService {
    pub fn new(config: GeneratorConfig) -> Self {
        Self {
            state: Arc::new(ServerState::new(config)),
        }
    }

    pub fn with_state(state: Arc<ServerState>) -> Self {
        Self { state }
    }

    /// Convert a GenerationConfig proto to GeneratorConfig.
    async fn proto_to_config(
        &self,
        proto: Option<GenerationConfig>,
    ) -> Result<GeneratorConfig, Status> {
        match proto {
            Some(p) => {
                let industry = match p.industry.to_lowercase().as_str() {
                    "manufacturing" => IndustrySector::Manufacturing,
                    "retail" => IndustrySector::Retail,
                    "financial_services" | "financial" => IndustrySector::FinancialServices,
                    "healthcare" => IndustrySector::Healthcare,
                    "technology" => IndustrySector::Technology,
                    _ => IndustrySector::Manufacturing,
                };

                let complexity = match p.coa_complexity.to_lowercase().as_str() {
                    "small" => CoAComplexity::Small,
                    "medium" => CoAComplexity::Medium,
                    "large" => CoAComplexity::Large,
                    _ => CoAComplexity::Small,
                };

                let companies: Vec<CompanyConfig> = if p.companies.is_empty() {
                    vec![CompanyConfig {
                        code: "1000".to_string(),
                        name: "Default Company".to_string(),
                        currency: "USD".to_string(),
                        country: "US".to_string(),
                        annual_transaction_volume: TransactionVolume::TenK,
                        volume_weight: 1.0,
                        fiscal_year_variant: "K4".to_string(),
                    }]
                } else {
                    p.companies
                        .into_iter()
                        .map(|c| CompanyConfig {
                            code: c.code,
                            name: c.name,
                            currency: c.currency,
                            country: c.country,
                            annual_transaction_volume: TransactionVolume::Custom(
                                c.annual_transaction_volume,
                            ),
                            volume_weight: c.volume_weight as f64,
                            fiscal_year_variant: "K4".to_string(),
                        })
                        .collect()
                };

                let mut config = GeneratorConfig {
                    global: GlobalConfig {
                        seed: if p.seed > 0 { Some(p.seed) } else { None },
                        industry,
                        start_date: if p.start_date.is_empty() {
                            "2024-01-01".to_string()
                        } else {
                            p.start_date
                        },
                        period_months: if p.period_months == 0 {
                            12
                        } else {
                            p.period_months
                        },
                        group_currency: "USD".to_string(),
                        parallel: true,
                        worker_threads: 0,
                        memory_limit_mb: 0,
                    },
                    companies,
                    chart_of_accounts: ChartOfAccountsConfig {
                        complexity,
                        industry_specific: true,
                        custom_accounts: None,
                        min_hierarchy_depth: 2,
                        max_hierarchy_depth: 5,
                    },
                    ..default_generator_config()
                };

                // Enable fraud if requested
                if p.fraud_enabled {
                    config.fraud.enabled = true;
                    config.fraud.fraud_rate = p.fraud_rate as f64;
                }

                Ok(config)
            }
            None => {
                // Use current server config
                let config = self.state.config.read().await;
                Ok(config.clone())
            }
        }
    }

    /// Convert a JournalEntry to proto format.
    fn journal_entry_to_proto(entry: &JournalEntry) -> JournalEntryProto {
        JournalEntryProto {
            document_id: entry.header.document_id.to_string(),
            company_code: entry.header.company_code.clone(),
            fiscal_year: entry.header.fiscal_year as u32,
            fiscal_period: entry.header.fiscal_period as u32,
            posting_date: entry.header.posting_date.to_string(),
            document_date: entry.header.document_date.to_string(),
            created_at: entry.header.created_at.to_rfc3339(),
            source: format!("{:?}", entry.header.source),
            business_process: entry.header.business_process.map(|bp| format!("{:?}", bp)),
            lines: entry
                .lines
                .iter()
                .map(|line| {
                    let amount = if line.is_debit() {
                        line.debit_amount
                    } else {
                        line.credit_amount
                    };
                    JournalLineProto {
                        line_number: line.line_number,
                        account_number: line.gl_account.clone(),
                        account_name: line.account_description.clone().unwrap_or_default(),
                        amount: amount.to_string(),
                        is_debit: line.is_debit(),
                        cost_center: line.cost_center.clone(),
                        profit_center: line.profit_center.clone(),
                        vendor_id: None,
                        customer_id: None,
                        material_id: None,
                        text: None,
                    }
                })
                .collect(),
            is_anomaly: entry.header.is_fraud,
            anomaly_type: entry.header.fraud_type.map(|ft| format!("{:?}", ft)),
        }
    }

    /// Convert current config to proto format.
    fn config_to_proto(config: &GeneratorConfig) -> GenerationConfig {
        GenerationConfig {
            industry: format!("{:?}", config.global.industry),
            start_date: config.global.start_date.clone(),
            period_months: config.global.period_months,
            seed: config.global.seed.unwrap_or(0),
            coa_complexity: format!("{:?}", config.chart_of_accounts.complexity),
            companies: config
                .companies
                .iter()
                .map(|c| CompanyConfigProto {
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
            generate_master_data: true,
            generate_document_flows: true,
        }
    }
}

#[tonic::async_trait]
impl synthetic_data_service_server::SyntheticDataService for SynthService {
    /// Bulk generation - generates all data at once and returns.
    async fn bulk_generate(
        &self,
        request: Request<BulkGenerateRequest>,
    ) -> Result<Response<BulkGenerateResponse>, Status> {
        let req = request.into_inner();

        // Validate entry_count bounds
        const MAX_ENTRY_COUNT: u64 = 1_000_000;
        if req.entry_count > MAX_ENTRY_COUNT {
            return Err(Status::invalid_argument(format!(
                "entry_count ({}) exceeds maximum allowed value ({})",
                req.entry_count, MAX_ENTRY_COUNT
            )));
        }

        // Check resources before starting generation
        let degradation_level = self.state.check_resources()?;
        if degradation_level != DegradationLevel::Normal {
            warn!(
                "Starting bulk generation under resource pressure (level: {:?})",
                degradation_level
            );
        }

        info!("Bulk generate request: {} entries", req.entry_count);

        let config = self.proto_to_config(req.config).await?;
        let start_time = Instant::now();

        // Create orchestrator with appropriate phase config
        let phase_config = PhaseConfig {
            generate_master_data: req.include_master_data,
            generate_document_flows: false,
            generate_journal_entries: true,
            inject_anomalies: req.inject_anomalies,
            show_progress: false,
            ..Default::default()
        };

        let mut orchestrator = EnhancedOrchestrator::new(config, phase_config)
            .map_err(|e| Status::internal(format!("Failed to create orchestrator: {}", e)))?;

        let result = orchestrator
            .generate()
            .map_err(|e| Status::internal(format!("Generation failed: {}", e)))?;

        let duration_ms = start_time.elapsed().as_millis() as u64;

        // Update metrics
        let entries_count = result.journal_entries.len() as u64;
        self.state
            .total_entries
            .fetch_add(entries_count, Ordering::Relaxed);

        let anomaly_count = result.anomaly_labels.labels.len() as u64;
        self.state
            .total_anomalies
            .fetch_add(anomaly_count, Ordering::Relaxed);

        // Convert to proto
        let journal_entries: Vec<JournalEntryProto> = result
            .journal_entries
            .iter()
            .map(Self::journal_entry_to_proto)
            .collect();

        let anomaly_labels: Vec<AnomalyLabelProto> = result
            .anomaly_labels
            .labels
            .iter()
            .map(|a| AnomalyLabelProto {
                anomaly_id: a.anomaly_id.clone(),
                document_id: a.document_id.clone(),
                anomaly_type: format!("{:?}", a.anomaly_type),
                anomaly_category: a.document_type.clone(),
                description: a.description.clone(),
                severity_score: a.severity as f32,
            })
            .collect();

        // Compute stats
        let mut total_debit = rust_decimal::Decimal::ZERO;
        let mut total_credit = rust_decimal::Decimal::ZERO;
        let mut total_lines = 0u64;
        let mut entries_by_company = std::collections::HashMap::new();
        let mut entries_by_source = std::collections::HashMap::new();

        for entry in &result.journal_entries {
            *entries_by_company
                .entry(entry.header.company_code.clone())
                .or_insert(0u64) += 1;
            *entries_by_source
                .entry(format!("{:?}", entry.header.source))
                .or_insert(0u64) += 1;

            for line in &entry.lines {
                total_lines += 1;
                total_debit += line.debit_amount;
                total_credit += line.credit_amount;
            }
        }

        let stats = GenerationStats {
            total_entries: entries_count,
            total_lines,
            total_debit_amount: total_debit.to_string(),
            total_credit_amount: total_credit.to_string(),
            anomaly_count,
            entries_by_company,
            entries_by_source,
        };

        info!(
            "Bulk generation complete: {} entries in {}ms",
            entries_count, duration_ms
        );

        Ok(Response::new(BulkGenerateResponse {
            entries_generated: entries_count,
            duration_ms,
            journal_entries,
            anomaly_labels,
            stats: Some(stats),
        }))
    }

    type StreamDataStream = Pin<Box<dyn Stream<Item = Result<DataEvent, Status>> + Send + 'static>>;

    /// Streaming generation - continuously generates data events.
    async fn stream_data(
        &self,
        request: Request<StreamDataRequest>,
    ) -> Result<Response<Self::StreamDataStream>, Status> {
        let req = request.into_inner();

        // Validate events_per_second bounds
        const MIN_EVENTS_PER_SECOND: u32 = 1;
        const MAX_EVENTS_PER_SECOND: u32 = 10_000;
        if req.events_per_second < MIN_EVENTS_PER_SECOND {
            return Err(Status::invalid_argument(format!(
                "events_per_second ({}) must be at least {}",
                req.events_per_second, MIN_EVENTS_PER_SECOND
            )));
        }
        if req.events_per_second > MAX_EVENTS_PER_SECOND {
            return Err(Status::invalid_argument(format!(
                "events_per_second ({}) exceeds maximum allowed value ({})",
                req.events_per_second, MAX_EVENTS_PER_SECOND
            )));
        }

        // Validate max_events if specified
        const MAX_STREAM_EVENTS: u64 = 10_000_000;
        if req.max_events > MAX_STREAM_EVENTS {
            return Err(Status::invalid_argument(format!(
                "max_events ({}) exceeds maximum allowed value ({})",
                req.max_events, MAX_STREAM_EVENTS
            )));
        }

        // Check resources before starting stream
        let degradation_level = self.state.check_resources()?;
        if degradation_level != DegradationLevel::Normal {
            warn!(
                "Starting stream under resource pressure (level: {:?})",
                degradation_level
            );
        }

        info!(
            "Stream data request: {} events/sec, max {}",
            req.events_per_second, req.max_events
        );

        let config = self.proto_to_config(req.config).await?;
        let state = self.state.clone();

        // Increment active streams
        state.active_streams.fetch_add(1, Ordering::Relaxed);

        // Reset control flags
        state.stream_paused.store(false, Ordering::Relaxed);
        state.stream_stopped.store(false, Ordering::Relaxed);

        let (tx, rx) = mpsc::channel(100);

        // Spawn background task to generate and stream data
        let events_per_second = req.events_per_second;
        let max_events = req.max_events;
        let inject_anomalies = req.inject_anomalies;

        tokio::spawn(async move {
            let phase_config = PhaseConfig {
                generate_master_data: false,
                generate_document_flows: false,
                generate_journal_entries: true,
                inject_anomalies,
                show_progress: false,
                ..Default::default()
            };

            let mut sequence = 0u64;
            let delay = if events_per_second > 0 {
                Duration::from_micros(1_000_000 / events_per_second as u64)
            } else {
                Duration::from_millis(1)
            };

            loop {
                // Check stop flag
                if state.stream_stopped.load(Ordering::Relaxed) {
                    info!("Stream stopped by control command");
                    break;
                }

                // Check pause flag
                while state.stream_paused.load(Ordering::Relaxed) {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    if state.stream_stopped.load(Ordering::Relaxed) {
                        break;
                    }
                }

                // Check max events
                if max_events > 0 && sequence >= max_events {
                    info!("Stream reached max events: {}", max_events);
                    break;
                }

                // Generate a batch
                let mut orchestrator =
                    match EnhancedOrchestrator::new(config.clone(), phase_config.clone()) {
                        Ok(o) => o,
                        Err(e) => {
                            error!("Failed to create orchestrator: {}", e);
                            break;
                        }
                    };

                let result = match orchestrator.generate() {
                    Ok(r) => r,
                    Err(e) => {
                        error!("Generation failed: {}", e);
                        break;
                    }
                };

                // Stream each entry
                for entry in result.journal_entries {
                    sequence += 1;
                    state.total_stream_events.fetch_add(1, Ordering::Relaxed);
                    state.total_entries.fetch_add(1, Ordering::Relaxed);

                    let timestamp = Timestamp {
                        seconds: Utc::now().timestamp(),
                        nanos: 0,
                    };

                    let event = DataEvent {
                        sequence,
                        timestamp: Some(timestamp),
                        event: Some(data_event::Event::JournalEntry(
                            SynthService::journal_entry_to_proto(&entry),
                        )),
                    };

                    if tx.send(Ok(event)).await.is_err() {
                        info!("Stream receiver dropped");
                        break;
                    }

                    // Rate limiting
                    tokio::time::sleep(delay).await;

                    // Check max events
                    if max_events > 0 && sequence >= max_events {
                        break;
                    }
                }
            }

            // Decrement active streams
            state.active_streams.fetch_sub(1, Ordering::Relaxed);
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }

    /// Control commands for streaming.
    async fn control(
        &self,
        request: Request<ControlCommand>,
    ) -> Result<Response<ControlResponse>, Status> {
        let cmd = request.into_inner();
        let action = ControlAction::try_from(cmd.action).unwrap_or(ControlAction::Unspecified);

        info!("Control command: {:?}", action);

        let (success, message, status) = match action {
            ControlAction::Pause => {
                self.state.stream_paused.store(true, Ordering::Relaxed);
                (true, "Stream paused".to_string(), StreamStatus::Paused)
            }
            ControlAction::Resume => {
                self.state.stream_paused.store(false, Ordering::Relaxed);
                (true, "Stream resumed".to_string(), StreamStatus::Running)
            }
            ControlAction::Stop => {
                self.state.stream_stopped.store(true, Ordering::Relaxed);
                (true, "Stream stopped".to_string(), StreamStatus::Stopped)
            }
            ControlAction::TriggerPattern => {
                let pattern = cmd.pattern_name.unwrap_or_default();
                if pattern.is_empty() {
                    (
                        false,
                        "Pattern name is required for TriggerPattern action".to_string(),
                        StreamStatus::Running,
                    )
                } else {
                    // Valid patterns: year_end_spike, period_end_spike, holiday_cluster,
                    // fraud_cluster, error_cluster, or any custom pattern name
                    let valid_patterns = [
                        "year_end_spike",
                        "period_end_spike",
                        "holiday_cluster",
                        "fraud_cluster",
                        "error_cluster",
                        "uniform",
                    ];
                    let is_valid = valid_patterns.contains(&pattern.as_str())
                        || pattern.starts_with("custom:");

                    if is_valid {
                        // Store the pattern for the stream generator to pick up
                        if let Ok(mut triggered) = self.state.triggered_pattern.try_write() {
                            *triggered = Some(pattern.clone());
                        }
                        info!("Pattern trigger activated: {}", pattern);
                        (
                            true,
                            format!("Pattern '{}' will be applied to upcoming entries", pattern),
                            StreamStatus::Running,
                        )
                    } else {
                        (
                            false,
                            format!(
                                "Unknown pattern '{}'. Valid patterns: {:?}",
                                pattern, valid_patterns
                            ),
                            StreamStatus::Running,
                        )
                    }
                }
            }
            ControlAction::Unspecified => (
                false,
                "Unknown control action".to_string(),
                StreamStatus::Unspecified,
            ),
        };

        Ok(Response::new(ControlResponse {
            success,
            message,
            current_status: status as i32,
        }))
    }

    /// Get current configuration.
    async fn get_config(&self, _request: Request<()>) -> Result<Response<ConfigResponse>, Status> {
        let config = self.state.config.read().await;
        let proto_config = Self::config_to_proto(&config);

        Ok(Response::new(ConfigResponse {
            success: true,
            message: "Current configuration retrieved".to_string(),
            current_config: Some(proto_config),
        }))
    }

    /// Set configuration.
    async fn set_config(
        &self,
        request: Request<ConfigRequest>,
    ) -> Result<Response<ConfigResponse>, Status> {
        let req = request.into_inner();

        if let Some(proto_config) = req.config {
            let new_config = self.proto_to_config(Some(proto_config)).await?;

            let mut config = self.state.config.write().await;
            *config = new_config.clone();

            info!("Configuration updated");

            Ok(Response::new(ConfigResponse {
                success: true,
                message: "Configuration updated".to_string(),
                current_config: Some(Self::config_to_proto(&new_config)),
            }))
        } else {
            Err(Status::invalid_argument("No configuration provided"))
        }
    }

    /// Get server metrics.
    async fn get_metrics(
        &self,
        _request: Request<()>,
    ) -> Result<Response<MetricsResponse>, Status> {
        let uptime = self.state.uptime_seconds();
        let total_entries = self.state.total_entries.load(Ordering::Relaxed);

        let entries_per_second = if uptime > 0 {
            total_entries as f64 / uptime as f64
        } else {
            0.0
        };

        Ok(Response::new(MetricsResponse {
            total_entries_generated: total_entries,
            total_anomalies_injected: self.state.total_anomalies.load(Ordering::Relaxed),
            uptime_seconds: uptime,
            session_entries: total_entries,
            session_entries_per_second: entries_per_second,
            active_streams: self.state.active_streams.load(Ordering::Relaxed) as u32,
            total_stream_events: self.state.total_stream_events.load(Ordering::Relaxed),
        }))
    }

    /// Health check.
    async fn health_check(
        &self,
        _request: Request<()>,
    ) -> Result<Response<HealthResponse>, Status> {
        Ok(Response::new(HealthResponse {
            healthy: true,
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: self.state.uptime_seconds(),
        }))
    }
}

/// Create a default GeneratorConfig.
pub fn default_generator_config() -> GeneratorConfig {
    GeneratorConfig {
        global: GlobalConfig {
            seed: None,
            industry: IndustrySector::Manufacturing,
            start_date: "2024-01-01".to_string(),
            period_months: 12,
            group_currency: "USD".to_string(),
            parallel: true,
            worker_threads: 0,
            memory_limit_mb: 0,
        },
        companies: vec![CompanyConfig {
            code: "1000".to_string(),
            name: "Default Company".to_string(),
            currency: "USD".to_string(),
            country: "US".to_string(),
            annual_transaction_volume: TransactionVolume::TenK,
            volume_weight: 1.0,
            fiscal_year_variant: "K4".to_string(),
        }],
        chart_of_accounts: ChartOfAccountsConfig {
            complexity: CoAComplexity::Small,
            industry_specific: true,
            custom_accounts: None,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 5,
        },
        transactions: Default::default(),
        output: OutputConfig::default(),
        fraud: Default::default(),
        internal_controls: Default::default(),
        business_processes: Default::default(),
        user_personas: Default::default(),
        templates: Default::default(),
        approval: Default::default(),
        departments: Default::default(),
        master_data: Default::default(),
        document_flows: Default::default(),
        intercompany: Default::default(),
        balance: Default::default(),
        ocpm: Default::default(),
        audit: Default::default(),
        banking: Default::default(),
        data_quality: Default::default(),
        scenario: Default::default(),
        temporal: Default::default(),
        graph_export: Default::default(),
        streaming: Default::default(),
        rate_limit: Default::default(),
        temporal_attributes: Default::default(),
        relationships: Default::default(),
        accounting_standards: Default::default(),
        audit_standards: Default::default(),
        distributions: Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grpc::synth::synthetic_data_service_server::SyntheticDataService;

    // ==========================================================================
    // Service Creation Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_service_creation() {
        let config = default_generator_config();
        let service = SynthService::new(config);
        // Service should start with zero or very small uptime (test completes quickly)
        assert!(service.state.uptime_seconds() < 60);
    }

    #[tokio::test]
    async fn test_service_with_state() {
        let config = default_generator_config();
        let state = Arc::new(ServerState::new(config));
        let service = SynthService::with_state(Arc::clone(&state));

        // Should share the same state
        state.total_entries.store(100, Ordering::Relaxed);
        assert_eq!(service.state.total_entries.load(Ordering::Relaxed), 100);
    }

    // ==========================================================================
    // Health Check Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_health_check() {
        let config = default_generator_config();
        let service = SynthService::new(config);

        let response = service.health_check(Request::new(())).await.unwrap();
        let health = response.into_inner();

        assert!(health.healthy);
        assert!(!health.version.is_empty());
    }

    #[tokio::test]
    async fn test_health_check_returns_version() {
        let config = default_generator_config();
        let service = SynthService::new(config);

        let response = service.health_check(Request::new(())).await.unwrap();
        let health = response.into_inner();

        assert_eq!(health.version, env!("CARGO_PKG_VERSION"));
    }

    // ==========================================================================
    // Configuration Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_get_config() {
        let config = default_generator_config();
        let service = SynthService::new(config);

        let response = service.get_config(Request::new(())).await.unwrap();
        let config_response = response.into_inner();

        assert!(config_response.success);
        assert!(config_response.current_config.is_some());
    }

    #[tokio::test]
    async fn test_get_config_returns_industry() {
        let config = default_generator_config();
        let service = SynthService::new(config);

        let response = service.get_config(Request::new(())).await.unwrap();
        let config_response = response.into_inner();
        let current = config_response.current_config.unwrap();

        assert_eq!(current.industry, "Manufacturing");
    }

    #[tokio::test]
    async fn test_set_config() {
        let config = default_generator_config();
        let service = SynthService::new(config);

        let new_config = GenerationConfig {
            industry: "retail".to_string(),
            start_date: "2024-06-01".to_string(),
            period_months: 6,
            seed: 42,
            coa_complexity: "medium".to_string(),
            companies: vec![],
            fraud_enabled: true,
            fraud_rate: 0.05,
            generate_master_data: false,
            generate_document_flows: false,
        };

        let response = service
            .set_config(Request::new(ConfigRequest {
                config: Some(new_config),
            }))
            .await
            .unwrap();
        let config_response = response.into_inner();

        assert!(config_response.success);
    }

    #[tokio::test]
    async fn test_set_config_without_config_fails() {
        let config = default_generator_config();
        let service = SynthService::new(config);

        let result = service
            .set_config(Request::new(ConfigRequest { config: None }))
            .await;

        assert!(result.is_err());
    }

    // ==========================================================================
    // Metrics Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_get_metrics_initial() {
        let config = default_generator_config();
        let service = SynthService::new(config);

        let response = service.get_metrics(Request::new(())).await.unwrap();
        let metrics = response.into_inner();

        assert_eq!(metrics.total_entries_generated, 0);
        assert_eq!(metrics.total_anomalies_injected, 0);
        assert_eq!(metrics.active_streams, 0);
    }

    #[tokio::test]
    async fn test_get_metrics_after_updates() {
        let config = default_generator_config();
        let service = SynthService::new(config);

        // Simulate some activity
        service.state.total_entries.store(1000, Ordering::Relaxed);
        service.state.total_anomalies.store(20, Ordering::Relaxed);
        service.state.active_streams.store(2, Ordering::Relaxed);

        let response = service.get_metrics(Request::new(())).await.unwrap();
        let metrics = response.into_inner();

        assert_eq!(metrics.total_entries_generated, 1000);
        assert_eq!(metrics.total_anomalies_injected, 20);
        assert_eq!(metrics.active_streams, 2);
    }

    // ==========================================================================
    // Control Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_control_pause() {
        let config = default_generator_config();
        let service = SynthService::new(config);

        let response = service
            .control(Request::new(ControlCommand {
                action: ControlAction::Pause as i32,
                pattern_name: None,
            }))
            .await
            .unwrap();
        let control_response = response.into_inner();

        assert!(control_response.success);
        assert!(service.state.stream_paused.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn test_control_resume() {
        let config = default_generator_config();
        let service = SynthService::new(config);

        // First pause
        service.state.stream_paused.store(true, Ordering::Relaxed);

        let response = service
            .control(Request::new(ControlCommand {
                action: ControlAction::Resume as i32,
                pattern_name: None,
            }))
            .await
            .unwrap();
        let control_response = response.into_inner();

        assert!(control_response.success);
        assert!(!service.state.stream_paused.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn test_control_stop() {
        let config = default_generator_config();
        let service = SynthService::new(config);

        let response = service
            .control(Request::new(ControlCommand {
                action: ControlAction::Stop as i32,
                pattern_name: None,
            }))
            .await
            .unwrap();
        let control_response = response.into_inner();

        assert!(control_response.success);
        assert!(service.state.stream_stopped.load(Ordering::Relaxed));
    }

    // ==========================================================================
    // ServerState Tests
    // ==========================================================================

    #[test]
    fn test_server_state_creation() {
        let config = default_generator_config();
        let state = ServerState::new(config);

        assert_eq!(state.total_entries.load(Ordering::Relaxed), 0);
        assert_eq!(state.total_anomalies.load(Ordering::Relaxed), 0);
        assert_eq!(state.active_streams.load(Ordering::Relaxed), 0);
        assert!(!state.stream_paused.load(Ordering::Relaxed));
        assert!(!state.stream_stopped.load(Ordering::Relaxed));
    }

    #[test]
    fn test_server_state_uptime() {
        let config = default_generator_config();
        let state = ServerState::new(config);

        // Uptime should be small since we just created the state
        assert!(state.uptime_seconds() < 60);
    }

    // ==========================================================================
    // Proto Conversion Tests
    // ==========================================================================

    #[test]
    fn test_default_generator_config() {
        let config = default_generator_config();

        assert_eq!(config.global.industry, IndustrySector::Manufacturing);
        assert_eq!(config.global.period_months, 12);
        assert!(!config.companies.is_empty());
        assert_eq!(config.companies[0].code, "1000");
    }

    #[test]
    fn test_config_to_proto() {
        let config = default_generator_config();
        let proto = SynthService::config_to_proto(&config);

        assert_eq!(proto.industry, "Manufacturing");
        assert_eq!(proto.period_months, 12);
        assert!(!proto.companies.is_empty());
    }

    #[tokio::test]
    async fn test_proto_to_config_with_none() {
        let config = default_generator_config();
        let service = SynthService::new(config.clone());

        let result = service.proto_to_config(None).await.unwrap();

        // Should return current config when None is passed
        assert_eq!(result.global.industry, config.global.industry);
    }

    #[tokio::test]
    async fn test_proto_to_config_with_retail() {
        let config = default_generator_config();
        let service = SynthService::new(config);

        let proto = GenerationConfig {
            industry: "retail".to_string(),
            start_date: "2024-01-01".to_string(),
            period_months: 6,
            seed: 0,
            coa_complexity: "large".to_string(),
            companies: vec![],
            fraud_enabled: false,
            fraud_rate: 0.0,
            generate_master_data: false,
            generate_document_flows: false,
        };

        let result = service.proto_to_config(Some(proto)).await.unwrap();

        assert_eq!(result.global.industry, IndustrySector::Retail);
        assert_eq!(result.chart_of_accounts.complexity, CoAComplexity::Large);
    }

    #[tokio::test]
    async fn test_proto_to_config_with_healthcare() {
        let config = default_generator_config();
        let service = SynthService::new(config);

        let proto = GenerationConfig {
            industry: "healthcare".to_string(),
            start_date: "2024-01-01".to_string(),
            period_months: 12,
            seed: 42,
            coa_complexity: "small".to_string(),
            companies: vec![],
            fraud_enabled: true,
            fraud_rate: 0.1,
            generate_master_data: true,
            generate_document_flows: true,
        };

        let result = service.proto_to_config(Some(proto)).await.unwrap();

        assert_eq!(result.global.industry, IndustrySector::Healthcare);
        assert_eq!(result.global.seed, Some(42));
        assert!(result.fraud.enabled);
        assert!((result.fraud.fraud_rate - 0.1).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_proto_to_config_with_companies() {
        let config = default_generator_config();
        let service = SynthService::new(config);

        let proto = GenerationConfig {
            industry: "technology".to_string(),
            start_date: "2024-01-01".to_string(),
            period_months: 12,
            seed: 0,
            coa_complexity: "medium".to_string(),
            companies: vec![
                CompanyConfigProto {
                    code: "1000".to_string(),
                    name: "Parent Corp".to_string(),
                    currency: "USD".to_string(),
                    country: "US".to_string(),
                    annual_transaction_volume: 100000,
                    volume_weight: 1.0,
                },
                CompanyConfigProto {
                    code: "2000".to_string(),
                    name: "EU Sub".to_string(),
                    currency: "EUR".to_string(),
                    country: "DE".to_string(),
                    annual_transaction_volume: 50000,
                    volume_weight: 0.5,
                },
            ],
            fraud_enabled: false,
            fraud_rate: 0.0,
            generate_master_data: false,
            generate_document_flows: false,
        };

        let result = service.proto_to_config(Some(proto)).await.unwrap();

        assert_eq!(result.companies.len(), 2);
        assert_eq!(result.companies[0].code, "1000");
        assert_eq!(result.companies[1].currency, "EUR");
    }

    // ==========================================================================
    // Input Validation Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_bulk_generate_entry_count_validation() {
        let config = default_generator_config();
        let service = SynthService::new(config);

        let request = BulkGenerateRequest {
            entry_count: 2_000_000, // Exceeds MAX_ENTRY_COUNT
            include_master_data: false,
            inject_anomalies: false,
            output_format: 0,
            config: None,
        };

        let result = service.bulk_generate(Request::new(request)).await;
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.message().contains("exceeds maximum allowed value"));
    }

    #[tokio::test]
    async fn test_stream_data_events_per_second_too_low() {
        let config = default_generator_config();
        let service = SynthService::new(config);

        let request = StreamDataRequest {
            events_per_second: 0, // Below MIN_EVENTS_PER_SECOND
            max_events: 100,
            inject_anomalies: false,
            anomaly_rate: 0.0,
            config: None,
        };

        let result = service.stream_data(Request::new(request)).await;
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.message().contains("must be at least"));
    }

    #[tokio::test]
    async fn test_stream_data_events_per_second_too_high() {
        let config = default_generator_config();
        let service = SynthService::new(config);

        let request = StreamDataRequest {
            events_per_second: 20_000, // Exceeds MAX_EVENTS_PER_SECOND
            max_events: 100,
            inject_anomalies: false,
            anomaly_rate: 0.0,
            config: None,
        };

        let result = service.stream_data(Request::new(request)).await;
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.message().contains("exceeds maximum allowed value"));
    }

    #[tokio::test]
    async fn test_stream_data_max_events_too_high() {
        let config = default_generator_config();
        let service = SynthService::new(config);

        let request = StreamDataRequest {
            events_per_second: 100,
            max_events: 100_000_000, // Exceeds MAX_STREAM_EVENTS
            inject_anomalies: false,
            anomaly_rate: 0.0,
            config: None,
        };

        let result = service.stream_data(Request::new(request)).await;
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.message().contains("max_events"));
    }

    #[tokio::test]
    async fn test_stream_data_valid_request() {
        let config = default_generator_config();
        let service = SynthService::new(config);

        let request = StreamDataRequest {
            events_per_second: 10,
            max_events: 5,
            inject_anomalies: false,
            anomaly_rate: 0.0,
            config: None,
        };

        // This should succeed - we can't easily test the stream output,
        // but we verify the request is accepted
        let result = service.stream_data(Request::new(request)).await;
        assert!(result.is_ok());
    }
}
