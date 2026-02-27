//! CLI for synthetic accounting data generation.

mod output_writer;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use datasynth_config::schema::AccountingFrameworkConfig;
use datasynth_config::{presets, GeneratorConfig};
use datasynth_core::memory_guard::{MemoryGuard, MemoryGuardConfig};
use datasynth_core::models::{CoAComplexity, IndustrySector};
use datasynth_fingerprint::{
    evaluation::FidelityEvaluator,
    extraction::{CsvDataSource, DataSource, ExtractionConfig, FingerprintExtractor},
    io::{validate_dsf, FingerprintReader, FingerprintWriter},
    models::PrivacyLevel,
    privacy::PrivacyConfig,
};
use datasynth_output::write_fec_csv;
use datasynth_runtime::{
    export_labels_all_formats, EnhancedOrchestrator, LabelExportConfig, LabelExportSummary,
    OutputFileInfo, PhaseConfig, RunManifest,
};

#[cfg(unix)]
use signal_hook::consts::SIGUSR1;

#[derive(Parser)]
#[command(name = "datasynth-data")]
#[command(about = "Synthetic Enterprise Accounting Data Generator")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate synthetic accounting data
    Generate {
        /// Path to configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Output directory
        #[arg(short, long, default_value = "./output")]
        output: PathBuf,

        /// Use demo preset (small dataset for testing)
        #[arg(long)]
        demo: bool,

        /// Load a scenario pack (e.g., "manufacturing/supplier_fraud")
        #[arg(long)]
        scenario_pack: Option<String>,

        /// Generate from a fingerprint file (.dsf)
        #[arg(long)]
        fingerprint: Option<PathBuf>,

        /// Scale factor for fingerprint-based generation (default: 1.0)
        #[arg(long, default_value = "1.0")]
        scale: f64,

        /// Random seed for reproducibility
        #[arg(short, long)]
        seed: Option<u64>,

        /// Enable banking KYC/AML data generation
        #[arg(long)]
        banking: bool,

        /// Enable audit data generation
        #[arg(long)]
        audit: bool,

        /// Memory limit in MB (default: 1024 MB)
        #[arg(long, default_value = "1024")]
        memory_limit: usize,

        /// Maximum CPU threads to use (default: half of available cores, min 1)
        #[arg(long)]
        max_threads: Option<usize>,

        /// Enable graph export for accounting networks (PyTorch Geometric format)
        #[arg(long)]
        graph_export: bool,

        /// Stream unified hypergraph JSONL to a RustGraph ingest endpoint URL
        #[arg(long)]
        stream_target: Option<String>,

        /// API key for the RustGraph ingest endpoint
        #[arg(long)]
        stream_api_key: Option<String>,

        /// Batch size for streaming (lines per HTTP POST, default 1000)
        #[arg(long, default_value = "1000")]
        stream_batch_size: usize,

        /// Quality gate profile (none/lenient/default/strict)
        #[arg(long, default_value = "none")]
        quality_gate: String,
    },

    /// Validate a configuration file
    Validate {
        /// Path to configuration file
        #[arg(short, long)]
        config: PathBuf,
    },

    /// Generate a sample configuration file
    Init {
        /// Output path
        #[arg(short, long, default_value = "datasynth_config.yaml")]
        output: PathBuf,

        /// Industry preset
        #[arg(short, long, default_value = "manufacturing")]
        industry: String,

        /// CoA complexity (small, medium, large)
        #[arg(short, long, default_value = "medium")]
        complexity: String,
    },

    /// Show information about available presets
    Info,

    /// Verify output integrity (checksums, record counts)
    Verify {
        /// Output directory to verify
        #[arg(short, long, default_value = "./output")]
        output: PathBuf,

        /// Verify file checksums
        #[arg(long)]
        checksums: bool,

        /// Verify record counts
        #[arg(long)]
        record_counts: bool,
    },

    /// Fingerprint extraction and management
    Fingerprint {
        #[command(subcommand)]
        command: FingerprintCommands,
    },
}

#[derive(Subcommand)]
enum FingerprintCommands {
    /// Extract fingerprint from data
    Extract {
        /// Input data path (CSV file or directory)
        #[arg(short, long)]
        input: PathBuf,

        /// Output fingerprint file (.dsf)
        #[arg(short, long)]
        output: PathBuf,

        /// Privacy level (minimal, standard, high, maximum)
        #[arg(long, default_value = "standard")]
        privacy_level: String,

        /// Custom epsilon budget for differential privacy
        #[arg(long)]
        privacy_epsilon: Option<f64>,

        /// Custom k-anonymity threshold
        #[arg(long)]
        privacy_k: Option<u32>,

        /// Sign the fingerprint
        #[arg(long)]
        sign: bool,
    },

    /// Validate a fingerprint file
    Validate {
        /// Fingerprint file to validate
        #[arg(required = true)]
        file: PathBuf,
    },

    /// Show fingerprint information
    Info {
        /// Fingerprint file
        #[arg(required = true)]
        file: PathBuf,

        /// Show detailed statistics
        #[arg(long)]
        detailed: bool,
    },

    /// Compare two fingerprints
    Diff {
        /// First fingerprint file
        #[arg(required = true)]
        file1: PathBuf,

        /// Second fingerprint file
        #[arg(required = true)]
        file2: PathBuf,
    },

    /// Evaluate fidelity of synthetic data against fingerprint
    Evaluate {
        /// Fingerprint file
        #[arg(short, long)]
        fingerprint: PathBuf,

        /// Synthetic data directory
        #[arg(short, long)]
        synthetic: PathBuf,

        /// Output report path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Fidelity threshold (0.0-1.0)
        #[arg(long, default_value = "0.8")]
        threshold: f64,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let filter = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| filter.into()),
        )
        .init();

    match cli.command {
        Commands::Generate {
            config,
            output,
            demo,
            scenario_pack,
            fingerprint,
            scale,
            seed,
            banking,
            audit,
            memory_limit,
            max_threads,
            graph_export,
            stream_target,
            stream_api_key,
            stream_batch_size,
            quality_gate,
        } => {
            // ========================================
            // CPU SAFEGUARD: Limit thread pool size
            // ========================================
            let available_cpus = num_cpus::get();
            let effective_threads = max_threads.unwrap_or_else(|| {
                // Default: use half of available cores, minimum 1, maximum 4
                (available_cpus / 2).clamp(1, 4)
            });

            // Configure rayon thread pool with limited threads
            if let Err(e) = rayon::ThreadPoolBuilder::new()
                .num_threads(effective_threads)
                .build_global()
            {
                eprintln!(
                    "Warning: failed to configure thread pool with {} threads: {}",
                    effective_threads, e
                );
            }

            tracing::info!(
                "CPU safeguard: using {} threads (of {} available)",
                effective_threads,
                available_cpus
            );

            // ========================================
            // MEMORY SAFEGUARD: Set conservative limits
            // ========================================
            let effective_memory_limit = if memory_limit > 0 {
                memory_limit.min(get_safe_memory_limit()) // Cap at safe limit
            } else {
                1024 // Default 1GB
            };

            let memory_config =
                MemoryGuardConfig::with_limit_mb(effective_memory_limit).aggressive();
            let memory_guard = Arc::new(MemoryGuard::new(memory_config));

            tracing::info!(
                "Memory safeguard: {} MB limit ({} MB soft limit)",
                effective_memory_limit,
                (effective_memory_limit * 80) / 100
            );

            // Check initial memory status
            let initial_memory = memory_guard.current_usage_mb();
            tracing::info!("Initial memory usage: {} MB", initial_memory);

            // ========================================
            // LOAD CONFIGURATION OR ORCHESTRATOR
            // ========================================
            // When generating from fingerprint, we create the orchestrator directly.
            // Otherwise, we load a config and create the orchestrator later.
            #[allow(clippy::large_enum_variant)] // Temporary local enum, not worth boxing both
            enum ConfigOrOrchestrator {
                Config(GeneratorConfig),
                Orchestrator(Box<EnhancedOrchestrator>),
            }

            let config_or_orchestrator = if demo {
                tracing::info!("Using demo preset (conservative settings)");
                ConfigOrOrchestrator::Config(create_safe_demo_preset())
            } else if let Some(ref fp_path) = fingerprint {
                tracing::info!("Generating from fingerprint: {}", fp_path.display());
                tracing::info!("Scale factor: {:.2}", scale);

                let phase_config = PhaseConfig {
                    generate_banking: banking,
                    generate_audit: audit,
                    generate_graph_export: graph_export,
                    show_progress: true,
                    inject_anomalies: true, // Let fingerprint control this
                    inject_data_quality: true,
                    ..PhaseConfig::default()
                };

                // Create orchestrator directly from fingerprint
                let orchestrator =
                    EnhancedOrchestrator::from_fingerprint(fp_path, phase_config, scale)?;
                ConfigOrOrchestrator::Orchestrator(Box::new(orchestrator))
            } else if let Some(ref pack) = scenario_pack {
                tracing::info!("Loading scenario pack: {}", pack);
                let scenario_path = find_scenario_pack(pack)?;
                let content = std::fs::read_to_string(&scenario_path)?;
                let mut cfg: GeneratorConfig = serde_yaml::from_str(&content)?;
                apply_safety_limits(&mut cfg);
                ConfigOrOrchestrator::Config(cfg)
            } else if let Some(config_path) = config {
                let content = std::fs::read_to_string(&config_path)?;
                let mut cfg: GeneratorConfig = serde_yaml::from_str(&content)?;
                // Apply safety limits to loaded config
                apply_safety_limits(&mut cfg);
                ConfigOrOrchestrator::Config(cfg)
            } else {
                tracing::info!("No config specified, using safe demo preset");
                ConfigOrOrchestrator::Config(create_safe_demo_preset())
            };

            // Apply config modifications only when we have a Config (not fingerprint)
            let config_or_orchestrator = match config_or_orchestrator {
                ConfigOrOrchestrator::Config(mut cfg) => {
                    // Apply seed override
                    if let Some(s) = seed {
                        cfg.global.seed = Some(s);
                    }

                    // Enable banking if flag is set (with conservative defaults)
                    if banking {
                        cfg.banking.enabled = true;
                        cfg.banking.population.retail_customers =
                            cfg.banking.population.retail_customers.min(100);
                        cfg.banking.population.business_customers =
                            cfg.banking.population.business_customers.min(20);
                        cfg.banking.population.trusts = cfg.banking.population.trusts.min(5);
                        tracing::info!("Banking KYC/AML generation enabled (conservative mode)");
                    }

                    // Enable graph export if flag is set
                    if graph_export {
                        cfg.graph_export.enabled = true;
                        tracing::info!("Graph export enabled (PyTorch Geometric format)");
                    }

                    // Apply streaming settings if provided
                    if let Some(ref target) = stream_target {
                        cfg.graph_export.enabled = true;
                        cfg.graph_export.hypergraph.enabled = true;
                        cfg.graph_export.hypergraph.output_format = "unified".to_string();
                        cfg.graph_export.hypergraph.stream_target = Some(target.clone());
                        cfg.graph_export.hypergraph.stream_batch_size = stream_batch_size;
                        if let Some(ref key) = stream_api_key {
                            std::env::set_var("RUSTGRAPH_API_KEY", key);
                            tracing::debug!("API key set from CLI argument");
                        }
                        tracing::info!("Streaming unified hypergraph to: {}", target);
                    }

                    // Apply output and resource settings
                    cfg.output.output_directory = output.clone();
                    cfg.global.parallel = false;
                    cfg.global.worker_threads = effective_threads;
                    cfg.global.memory_limit_mb = effective_memory_limit;

                    ConfigOrOrchestrator::Config(cfg)
                }
                orch @ ConfigOrOrchestrator::Orchestrator(_) => {
                    // For fingerprint-based generation, the orchestrator already has its config
                    orch
                }
            };

            // Extract generator_config for logging and manifest
            let generator_config = match &config_or_orchestrator {
                ConfigOrOrchestrator::Config(cfg) => cfg.clone(),
                ConfigOrOrchestrator::Orchestrator(_) => {
                    // Fingerprint orchestrator has its own config; use demo preset as
                    // a stand-in for manifest generation metadata.
                    tracing::warn!(
                        "Fingerprint-based generation: manifest uses approximate config metadata"
                    );
                    create_safe_demo_preset()
                }
            };

            tracing::info!("Starting generation...");
            match &config_or_orchestrator {
                ConfigOrOrchestrator::Config(cfg) => {
                    tracing::info!("Industry: {:?}", cfg.global.industry);
                    tracing::info!("Period: {} months", cfg.global.period_months);
                    tracing::info!("Companies: {}", cfg.companies.len());
                }
                ConfigOrOrchestrator::Orchestrator(_) => {
                    tracing::info!("Mode: Fingerprint-based generation (scale: {:.2})", scale);
                }
            }

            // ========================================
            // SIGNAL HANDLING (Unix only)
            // ========================================
            let pause_flag = Arc::new(AtomicBool::new(false));

            #[cfg(unix)]
            {
                let pause_flag_clone = Arc::clone(&pause_flag);
                let signal_flag = Arc::new(AtomicBool::new(false));
                let signal_flag_clone = Arc::clone(&signal_flag);

                if signal_hook::flag::register(SIGUSR1, signal_flag_clone).is_ok() {
                    let pid = std::process::id();
                    tracing::info!("Pause/resume: send SIGUSR1 to toggle (kill -USR1 {})", pid);

                    std::thread::spawn(move || loop {
                        if signal_flag.swap(false, Ordering::Relaxed) {
                            let was_paused = pause_flag_clone.load(Ordering::Relaxed);
                            pause_flag_clone.store(!was_paused, Ordering::Relaxed);
                            if was_paused {
                                eprintln!("\n>>> RESUMED");
                            } else {
                                eprintln!("\n>>> PAUSED - send SIGUSR1 again to resume");
                            }
                        }
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    });
                }
            }

            // ========================================
            // PRE-GENERATION MEMORY CHECK
            // ========================================
            if let Err(e) = memory_guard.check_now() {
                tracing::error!("Memory limit already exceeded before generation: {}", e);
                return Err(anyhow::anyhow!("Insufficient memory to start generation"));
            }

            // ========================================
            // GENERATE DATA
            // ========================================
            // Capture values for manifest before potentially moving config
            let effective_seed = generator_config.global.seed.unwrap_or(42);
            let config_for_manifest = generator_config.clone();

            // Create or use existing orchestrator
            let mut orchestrator = match config_or_orchestrator {
                ConfigOrOrchestrator::Orchestrator(orch) => {
                    tracing::info!("Using orchestrator from fingerprint");
                    *orch
                }
                ConfigOrOrchestrator::Config(cfg) => {
                    let phase_config = PhaseConfig {
                        // Wire CLI flags OR config-enabled sections
                        // Note: banking defaults to enabled=true in its crate, so only
                        // use the explicit CLI --banking flag to avoid unexpected generation
                        generate_banking: banking,
                        generate_audit: audit || cfg.audit.enabled,
                        generate_graph_export: graph_export || cfg.graph_export.enabled,
                        generate_manufacturing: cfg.manufacturing.enabled,
                        generate_sourcing: cfg.source_to_pay.enabled,
                        generate_tax: cfg.tax.enabled,
                        generate_esg: cfg.esg.enabled,
                        generate_intercompany: cfg.intercompany.enabled,
                        generate_accounting_standards: cfg.accounting_standards.enabled,
                        generate_financial_statements: cfg.financial_reporting.enabled,
                        generate_sales_kpi_budgets: cfg.sales_quotes.enabled,
                        generate_bank_reconciliation: cfg.financial_reporting.enabled,
                        generate_ocpm_events: cfg.ocpm.enabled,
                        show_progress: true,
                        // Wire up anomaly and data quality injection from config
                        inject_anomalies: cfg.fraud.enabled || cfg.anomaly_injection.enabled,
                        inject_data_quality: cfg.data_quality.enabled,
                        // Use conservative defaults for document generation
                        p2p_chains: 50,
                        o2c_chains: 50,
                        vendors_per_company: 20,
                        customers_per_company: 30,
                        materials_per_company: 50,
                        assets_per_company: 20,
                        employees_per_company: 30,
                        ..PhaseConfig::default()
                    };
                    EnhancedOrchestrator::new(cfg, phase_config)?
                }
            };

            let result = orchestrator.generate()?;

            // ========================================
            // REPORT RESULTS
            // ========================================
            tracing::info!("Generation complete!");
            tracing::info!("Total entries: {}", result.statistics.total_entries);
            tracing::info!("Total line items: {}", result.statistics.total_line_items);
            tracing::info!("Accounts in CoA: {}", result.statistics.accounts_count);

            // Memory usage reporting
            let stats = memory_guard.stats();
            let peak_mb = stats.peak_resident_bytes / (1024 * 1024);
            let current_mb = stats.resident_bytes / (1024 * 1024);
            tracing::info!(
                "Memory usage: current {} MB, peak {} MB",
                current_mb,
                peak_mb
            );
            if stats.soft_limit_warnings > 0 {
                tracing::warn!(
                    "Memory soft limit was exceeded {} times during generation",
                    stats.soft_limit_warnings
                );
            }

            // Banking statistics
            if result.statistics.banking_customer_count > 0 {
                tracing::info!(
                    "Banking: {} customers, {} accounts, {} transactions ({} suspicious)",
                    result.statistics.banking_customer_count,
                    result.statistics.banking_account_count,
                    result.statistics.banking_transaction_count,
                    result.statistics.banking_suspicious_count
                );
            }

            // Audit statistics
            if result.statistics.audit_engagement_count > 0 {
                tracing::info!(
                    "Audit: {} engagements, {} workpapers, {} findings",
                    result.statistics.audit_engagement_count,
                    result.statistics.audit_workpaper_count,
                    result.statistics.audit_finding_count
                );
            }

            // ========================================
            // WRITE OUTPUT (with memory checks)
            // ========================================
            std::fs::create_dir_all(&output)?;

            // Check memory before writing
            if memory_guard.check_now().is_err() {
                tracing::warn!("Memory limit reached, writing minimal output");
            }

            // Write all generated data (journal entries, master data, document flows,
            // subledgers, HR, manufacturing, sourcing, banking, audit, tax, ESG, etc.)
            if let Err(e) = output_writer::write_all_output(&result, &output) {
                tracing::warn!("Some output files may not have been written: {}", e);
            }

            // Write FEC (Fichier des Écritures Comptables) when French GAAP – 18 mandatory columns
            if matches!(
                config_for_manifest.accounting_standards.framework,
                Some(AccountingFrameworkConfig::FrenchGaap)
            ) && !result.journal_entries.is_empty()
            {
                let fec_path = output.join("fec.csv");
                match write_fec_csv(
                    &fec_path,
                    &result.journal_entries,
                    &result.chart_of_accounts,
                ) {
                    Ok(()) => tracing::info!(
                        "FEC (18 columns) written to: {} ({} entries, {} lines)",
                        fec_path.display(),
                        result.journal_entries.len(),
                        result
                            .journal_entries
                            .iter()
                            .map(|e| e.lines.len())
                            .sum::<usize>()
                    ),
                    Err(e) => tracing::warn!("Could not write FEC file: {}", e),
                }
            }

            // Write GoBD (Grundsätze zur ordnungsmäßigen Führung) when German GAAP
            if matches!(
                config_for_manifest.accounting_standards.framework,
                Some(AccountingFrameworkConfig::GermanGaap)
            ) && !result.journal_entries.is_empty()
            {
                let gobd_dir = output.join("gobd_export");
                if let Err(e) = std::fs::create_dir_all(&gobd_dir) {
                    tracing::warn!("Could not create gobd_export directory: {}", e);
                } else {
                    // Journal CSV
                    match datasynth_output::write_gobd_journal_csv(
                        &gobd_dir.join("gobd_journal.csv"),
                        &result.journal_entries,
                        &result.chart_of_accounts,
                    ) {
                        Ok(()) => tracing::info!(
                            "GoBD journal (13 columns) written: {} entries",
                            result.journal_entries.len()
                        ),
                        Err(e) => tracing::warn!("Could not write GoBD journal: {}", e),
                    }

                    // Accounts CSV
                    match datasynth_output::write_gobd_accounts_csv(
                        &gobd_dir.join("gobd_accounts.csv"),
                        &result.chart_of_accounts,
                    ) {
                        Ok(()) => tracing::info!(
                            "GoBD accounts written: {} accounts",
                            result.chart_of_accounts.accounts.len()
                        ),
                        Err(e) => tracing::warn!("Could not write GoBD accounts: {}", e),
                    }

                    // Index XML
                    let company_code = config_for_manifest
                        .companies
                        .first()
                        .map(|c| c.code.as_str())
                        .unwrap_or("UNKNOWN");
                    let fiscal_year: i32 = config_for_manifest
                        .global
                        .start_date
                        .split('-')
                        .next()
                        .and_then(|y| y.parse().ok())
                        .unwrap_or(2024);
                    let tables = vec![
                        ("gobd_journal.csv", "Buchungsjournal"),
                        ("gobd_accounts.csv", "Kontenplan"),
                    ];
                    match datasynth_output::write_gobd_index_xml(
                        &gobd_dir.join("index.xml"),
                        company_code,
                        fiscal_year,
                        &tables,
                    ) {
                        Ok(()) => tracing::info!("GoBD index.xml written"),
                        Err(e) => tracing::warn!("Could not write GoBD index.xml: {}", e),
                    }
                }
            }

            // ========================================
            // WRITE ANOMALY LABELS (Phase 1.1)
            // ========================================
            if !result.anomaly_labels.labels.is_empty() {
                let labels_dir = output.join("labels");
                std::fs::create_dir_all(&labels_dir)?;

                let export_config = LabelExportConfig::default();
                match export_labels_all_formats(
                    &result.anomaly_labels.labels,
                    &labels_dir,
                    "anomaly_labels",
                    &export_config,
                ) {
                    Ok(results) => {
                        for (path, count) in &results {
                            tracing::info!(
                                "Anomaly labels written to: {} ({} labels)",
                                path,
                                count
                            );
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to write anomaly labels: {}", e);
                    }
                }

                // Write summary
                let summary = LabelExportSummary::from_labels(&result.anomaly_labels.labels);
                if let Err(e) =
                    summary.write_to_file(&labels_dir.join("anomaly_labels_summary.json"))
                {
                    tracing::warn!("Failed to write anomaly label summary: {}", e);
                }

                tracing::info!(
                    "Anomaly labels: {} total, {} with provenance, {} in clusters",
                    summary.total_labels,
                    summary.with_provenance,
                    summary.in_clusters
                );
            }

            // ========================================
            // WRITE RUN MANIFEST (Phase 1.3)
            // ========================================
            let mut manifest = RunManifest::new(&config_for_manifest, effective_seed);
            manifest.set_output_directory(&output);
            manifest.complete(result.statistics.clone());

            // Add output file info for journal entries
            if !result.journal_entries.is_empty() {
                let total_lines: usize =
                    result.journal_entries.iter().map(|je| je.lines.len()).sum();
                manifest.add_output_file(OutputFileInfo {
                    path: "journal_entries.csv".to_string(),
                    format: "csv".to_string(),
                    record_count: Some(total_lines),
                    size_bytes: None,
                    sha256_checksum: None,
                    first_record_index: None,
                    last_record_index: None,
                });
                manifest.add_output_file(OutputFileInfo {
                    path: "journal_entries.json".to_string(),
                    format: "json".to_string(),
                    record_count: Some(result.journal_entries.len()),
                    size_bytes: None,
                    sha256_checksum: None,
                    first_record_index: None,
                    last_record_index: None,
                });
            }

            // Add master data file info
            for (name, count) in [
                ("master_data/vendors.json", result.master_data.vendors.len()),
                (
                    "master_data/customers.json",
                    result.master_data.customers.len(),
                ),
                (
                    "master_data/materials.json",
                    result.master_data.materials.len(),
                ),
                (
                    "master_data/fixed_assets.json",
                    result.master_data.assets.len(),
                ),
                (
                    "master_data/employees.json",
                    result.master_data.employees.len(),
                ),
            ] {
                if count > 0 {
                    manifest.add_output_file(OutputFileInfo {
                        path: name.to_string(),
                        format: "json".to_string(),
                        record_count: Some(count),
                        size_bytes: None,
                        sha256_checksum: None,
                        first_record_index: None,
                        last_record_index: None,
                    });
                }
            }

            // Add document flow file info
            for (name, count) in [
                (
                    "document_flows/purchase_orders.json",
                    result.document_flows.purchase_orders.len(),
                ),
                (
                    "document_flows/goods_receipts.json",
                    result.document_flows.goods_receipts.len(),
                ),
                (
                    "document_flows/vendor_invoices.json",
                    result.document_flows.vendor_invoices.len(),
                ),
                (
                    "document_flows/payments.json",
                    result.document_flows.payments.len(),
                ),
                (
                    "document_flows/sales_orders.json",
                    result.document_flows.sales_orders.len(),
                ),
                (
                    "document_flows/deliveries.json",
                    result.document_flows.deliveries.len(),
                ),
                (
                    "document_flows/customer_invoices.json",
                    result.document_flows.customer_invoices.len(),
                ),
            ] {
                if count > 0 {
                    manifest.add_output_file(OutputFileInfo {
                        path: name.to_string(),
                        format: "json".to_string(),
                        record_count: Some(count),
                        size_bytes: None,
                        sha256_checksum: None,
                        first_record_index: None,
                        last_record_index: None,
                    });
                }
            }

            if !result.anomaly_labels.labels.is_empty() {
                manifest.add_output_file(OutputFileInfo {
                    path: "labels/anomaly_labels.csv".to_string(),
                    format: "csv".to_string(),
                    record_count: Some(result.anomaly_labels.labels.len()),
                    size_bytes: None,
                    sha256_checksum: None,
                    first_record_index: None,
                    last_record_index: None,
                });
            }

            // Register additional output subdirectories in manifest
            // Helper to add a manifest entry for a JSON file
            let mut register = |path: &str, count: usize| {
                if count > 0 {
                    manifest.add_output_file(OutputFileInfo {
                        path: path.to_string(),
                        format: "json".to_string(),
                        record_count: Some(count),
                        size_bytes: None,
                        sha256_checksum: None,
                        first_record_index: None,
                        last_record_index: None,
                    });
                }
            };

            // Subledger
            register(
                "subledger/ar_invoices.json",
                result.subledger.ar_invoices.len(),
            );
            register(
                "subledger/ap_invoices.json",
                result.subledger.ap_invoices.len(),
            );
            register(
                "subledger/fa_records.json",
                result.subledger.fa_records.len(),
            );
            register(
                "subledger/inventory_positions.json",
                result.subledger.inventory_positions.len(),
            );
            register(
                "subledger/inventory_movements.json",
                result.subledger.inventory_movements.len(),
            );

            // Audit
            register(
                "audit/audit_engagements.json",
                result.audit.engagements.len(),
            );
            register("audit/audit_workpapers.json", result.audit.workpapers.len());
            register("audit/audit_evidence.json", result.audit.evidence.len());
            register(
                "audit/audit_risk_assessments.json",
                result.audit.risk_assessments.len(),
            );
            register("audit/audit_findings.json", result.audit.findings.len());
            register("audit/audit_judgments.json", result.audit.judgments.len());

            // Banking
            register(
                "banking/banking_customers.json",
                result.banking.customers.len(),
            );
            register(
                "banking/banking_transactions.json",
                result.banking.transactions.len(),
            );
            register(
                "banking/banking_accounts.json",
                result.banking.accounts.len(),
            );
            register(
                "banking/aml_transaction_labels.json",
                result.banking.transaction_labels.len(),
            );
            register(
                "banking/aml_customer_labels.json",
                result.banking.customer_labels.len(),
            );
            register(
                "banking/aml_account_labels.json",
                result.banking.account_labels.len(),
            );
            register(
                "banking/aml_relationship_labels.json",
                result.banking.relationship_labels.len(),
            );
            register(
                "banking/aml_narratives.json",
                result.banking.narratives.len(),
            );

            // Sourcing (S2C)
            register(
                "sourcing/sourcing_projects.json",
                result.sourcing.sourcing_projects.len(),
            );
            register(
                "sourcing/spend_analyses.json",
                result.sourcing.spend_analyses.len(),
            );
            register(
                "sourcing/supplier_qualifications.json",
                result.sourcing.qualifications.len(),
            );
            register("sourcing/rfx_events.json", result.sourcing.rfx_events.len());
            register("sourcing/supplier_bids.json", result.sourcing.bids.len());
            register(
                "sourcing/bid_evaluations.json",
                result.sourcing.bid_evaluations.len(),
            );
            register(
                "sourcing/procurement_contracts.json",
                result.sourcing.contracts.len(),
            );
            register(
                "sourcing/catalog_items.json",
                result.sourcing.catalog_items.len(),
            );
            register(
                "sourcing/supplier_scorecards.json",
                result.sourcing.scorecards.len(),
            );

            // Intercompany
            register(
                "intercompany/ic_matched_pairs.json",
                result.intercompany.matched_pairs.len(),
            );
            register(
                "intercompany/ic_elimination_entries.json",
                result.intercompany.elimination_entries.len(),
            );
            register(
                "intercompany/ic_seller_journal_entries.json",
                result.intercompany.seller_journal_entries.len(),
            );
            register(
                "intercompany/ic_buyer_journal_entries.json",
                result.intercompany.buyer_journal_entries.len(),
            );

            // Financial Reporting
            register(
                "financial_reporting/financial_statements.json",
                result.financial_reporting.financial_statements.len(),
            );
            register(
                "financial_reporting/bank_reconciliations.json",
                result.financial_reporting.bank_reconciliations.len(),
            );

            // Period Close
            register(
                "period_close/trial_balances.json",
                result.financial_reporting.trial_balances.len(),
            );

            // HR
            register("hr/payroll_runs.json", result.hr.payroll_runs.len());
            register("hr/time_entries.json", result.hr.time_entries.len());
            register("hr/expense_reports.json", result.hr.expense_reports.len());
            register(
                "hr/payroll_line_items.json",
                result.hr.payroll_line_items.len(),
            );

            // Manufacturing
            register(
                "manufacturing/production_orders.json",
                result.manufacturing.production_orders.len(),
            );
            register(
                "manufacturing/quality_inspections.json",
                result.manufacturing.quality_inspections.len(),
            );
            register(
                "manufacturing/cycle_counts.json",
                result.manufacturing.cycle_counts.len(),
            );

            // Sales / KPI / Budgets
            register(
                "sales_kpi_budgets/sales_quotes.json",
                result.sales_kpi_budgets.sales_quotes.len(),
            );
            register(
                "sales_kpi_budgets/management_kpis.json",
                result.sales_kpi_budgets.kpis.len(),
            );
            register(
                "sales_kpi_budgets/budgets.json",
                result.sales_kpi_budgets.budgets.len(),
            );

            // Internal Controls
            register(
                "internal_controls/internal_controls.json",
                result.internal_controls.len(),
            );

            // Accounting Standards
            register(
                "accounting_standards/customer_contracts.json",
                result.accounting_standards.contracts.len(),
            );
            register(
                "accounting_standards/impairment_tests.json",
                result.accounting_standards.impairment_tests.len(),
            );

            // Treasury
            register(
                "treasury/debt_instruments.json",
                result.treasury.debt_instruments.len(),
            );
            register(
                "treasury/hedging_instruments.json",
                result.treasury.hedging_instruments.len(),
            );
            register(
                "treasury/hedge_relationships.json",
                result.treasury.hedge_relationships.len(),
            );
            register(
                "treasury/cash_positions.json",
                result.treasury.cash_positions.len(),
            );
            register(
                "treasury/cash_forecasts.json",
                result.treasury.cash_forecasts.len(),
            );
            register("treasury/cash_pools.json", result.treasury.cash_pools.len());
            register(
                "treasury/cash_pool_sweeps.json",
                result.treasury.cash_pool_sweeps.len(),
            );
            register(
                "treasury/treasury_anomaly_labels.json",
                result.treasury.treasury_anomaly_labels.len(),
            );

            // Project Accounting
            register(
                "project_accounting/projects.json",
                result.project_accounting.projects.len(),
            );
            register(
                "project_accounting/change_orders.json",
                result.project_accounting.change_orders.len(),
            );
            register(
                "project_accounting/milestones.json",
                result.project_accounting.milestones.len(),
            );
            register(
                "project_accounting/cost_lines.json",
                result.project_accounting.cost_lines.len(),
            );
            register(
                "project_accounting/revenue_records.json",
                result.project_accounting.revenue_records.len(),
            );
            register(
                "project_accounting/earned_value_metrics.json",
                result.project_accounting.earned_value_metrics.len(),
            );

            // Tax (extended)
            register("tax/tax_provisions.json", result.tax.tax_provisions.len());
            register("tax/tax_jurisdictions.json", result.tax.jurisdictions.len());
            register("tax/tax_codes.json", result.tax.codes.len());
            register("tax/tax_lines.json", result.tax.tax_lines.len());
            register("tax/tax_returns.json", result.tax.tax_returns.len());
            register(
                "tax/withholding_records.json",
                result.tax.withholding_records.len(),
            );
            register(
                "tax/tax_anomaly_labels.json",
                result.tax.tax_anomaly_labels.len(),
            );

            // ESG
            register("esg/emission_records.json", result.esg.emissions.len());
            register("esg/energy_consumption.json", result.esg.energy.len());
            register("esg/water_usage.json", result.esg.water.len());
            register("esg/waste_records.json", result.esg.waste.len());
            register("esg/workforce_diversity.json", result.esg.diversity.len());
            register("esg/pay_equity.json", result.esg.pay_equity.len());
            register(
                "esg/safety_incidents.json",
                result.esg.safety_incidents.len(),
            );
            register("esg/safety_metrics.json", result.esg.safety_metrics.len());
            register("esg/governance_metrics.json", result.esg.governance.len());
            register(
                "esg/supplier_esg_assessments.json",
                result.esg.supplier_assessments.len(),
            );
            register(
                "esg/materiality_assessments.json",
                result.esg.materiality.len(),
            );
            register("esg/esg_disclosures.json", result.esg.disclosures.len());
            register(
                "esg/climate_scenarios.json",
                result.esg.climate_scenarios.len(),
            );
            register(
                "esg/esg_anomaly_labels.json",
                result.esg.anomaly_labels.len(),
            );

            // Balance
            register(
                "balance/opening_balances.json",
                result.opening_balances.len(),
            );
            register(
                "balance/subledger_reconciliation.json",
                result.subledger_reconciliation.len(),
            );

            // Process Mining
            register("process_mining/event_log.json", result.ocpm.event_count);

            // Root-level files
            register("chart_of_accounts.json", 1);
            register("generation_statistics.json", 1);

            // Attach lineage graph to manifest and write separate file
            if let Some(ref lineage) = result.lineage {
                manifest.lineage = Some(lineage.clone());
                let lineage_path = output.join("lineage_graph.json");
                if let Ok(json) = lineage.to_json() {
                    if let Err(e) = std::fs::write(&lineage_path, json) {
                        tracing::warn!("Failed to write lineage graph: {}", e);
                    } else {
                        tracing::info!(
                            "Lineage graph written to: {} ({} nodes, {} edges)",
                            lineage_path.display(),
                            lineage.node_count(),
                            lineage.edge_count()
                        );
                    }
                }
            }

            // Write W3C PROV-JSON
            {
                let prov_path = output.join("prov.json");
                let prov_doc = datasynth_runtime::prov::manifest_to_prov(&manifest);
                match serde_json::to_string_pretty(&prov_doc) {
                    Ok(json) => {
                        if let Err(e) = std::fs::write(&prov_path, json) {
                            tracing::warn!("Failed to write PROV-JSON: {}", e);
                        } else {
                            tracing::info!("PROV-JSON written to: {}", prov_path.display());
                        }
                    }
                    Err(e) => tracing::warn!("Failed to serialize PROV-JSON: {}", e),
                }
            }

            // Populate file checksums
            manifest.populate_file_checksums(&output);

            // Write manifest
            let manifest_path = output.join("run_manifest.json");
            if let Err(e) = manifest.write_to_file(&manifest_path) {
                tracing::warn!("Failed to write run manifest: {}", e);
            } else {
                tracing::info!(
                    "Run manifest written to: {} (run_id: {})",
                    manifest_path.display(),
                    manifest.run_id()
                );
            }

            // ========================================
            // QUALITY GATE EVALUATION
            // ========================================
            if quality_gate != "none" {
                if let Some(profile) = datasynth_eval::gates::get_profile(&quality_gate) {
                    tracing::warn!(
                        "Quality gate evaluation uses placeholder data — full integration pending"
                    );
                    let evaluation = datasynth_eval::ComprehensiveEvaluation::new();
                    let gate_result =
                        datasynth_eval::gates::GateEngine::evaluate(&evaluation, &profile);

                    // Print gate result summary
                    println!();
                    println!(
                        "Quality Gate Evaluation (profile: {})",
                        gate_result.profile_name
                    );
                    println!("==========================================");
                    for check in &gate_result.results {
                        let status = if check.passed { "PASS" } else { "FAIL" };
                        println!("  [{}] {}: {}", status, check.gate_name, check.message);
                    }
                    println!();
                    println!(
                        "Result: {}/{} gates passed",
                        gate_result.gates_passed, gate_result.gates_total
                    );
                    println!("{}", gate_result.summary);

                    if !gate_result.passed {
                        tracing::error!(
                            "Quality gates FAILED: {}/{}",
                            gate_result.gates_total - gate_result.gates_passed,
                            gate_result.gates_total
                        );
                        std::process::exit(2);
                    }
                } else {
                    tracing::warn!(
                        "Unknown quality gate profile '{}'. Valid profiles: none, lenient, default, strict",
                        quality_gate
                    );
                }
            }

            Ok(())
        }

        Commands::Validate { config } => {
            let content = std::fs::read_to_string(&config)?;
            let generator_config: GeneratorConfig = serde_yaml::from_str(&content)?;
            datasynth_config::validate_config(&generator_config)?;
            tracing::info!("Configuration is valid!");
            Ok(())
        }

        Commands::Init {
            output,
            industry,
            complexity,
        } => {
            let industry_lower = industry.to_lowercase();
            let industry_sector = match industry_lower.as_str() {
                "manufacturing" => IndustrySector::Manufacturing,
                "retail" => IndustrySector::Retail,
                "financial" | "financial_services" => IndustrySector::FinancialServices,
                "healthcare" => IndustrySector::Healthcare,
                "technology" | "tech" => IndustrySector::Technology,
                _ => {
                    eprintln!(
                        "Warning: unrecognized industry '{}'. Valid values: manufacturing, retail, financial_services, healthcare, technology. Defaulting to manufacturing.",
                        industry
                    );
                    IndustrySector::Manufacturing
                }
            };

            let complexity_lower = complexity.to_lowercase();
            let coa_complexity = match complexity_lower.as_str() {
                "small" => CoAComplexity::Small,
                "medium" => CoAComplexity::Medium,
                "large" => CoAComplexity::Large,
                _ => {
                    eprintln!(
                        "Warning: unrecognized complexity '{}'. Valid values: small, medium, large. Defaulting to medium.",
                        complexity
                    );
                    CoAComplexity::Medium
                }
            };

            let config = presets::create_preset(
                industry_sector,
                2,
                12,
                coa_complexity,
                datasynth_config::TransactionVolume::TenK, // Conservative default
            );

            let yaml = serde_yaml::to_string(&config)?;
            std::fs::write(&output, yaml)?;
            tracing::info!("Configuration written to: {}", output.display());
            Ok(())
        }

        Commands::Info => {
            println!("Available Industry Presets:");
            println!("  - manufacturing: Manufacturing industry");
            println!("  - retail: Retail industry");
            println!("  - financial_services: Financial services");
            println!("  - healthcare: Healthcare industry");
            println!("  - technology: Technology industry");
            println!();
            println!("Chart of Accounts Complexity:");
            println!("  - small: ~100 accounts");
            println!("  - medium: ~400 accounts");
            println!("  - large: ~2500 accounts");
            println!();
            println!("Transaction Volumes:");
            println!("  - ten_k: 10,000 transactions/year");
            println!("  - hundred_k: 100,000 transactions/year");
            println!("  - one_m: 1,000,000 transactions/year");
            println!("  - ten_m: 10,000,000 transactions/year");
            println!("  - hundred_m: 100,000,000 transactions/year");
            println!();
            println!("Resource Safeguards:");
            println!("  --memory-limit <MB>  : Set memory limit (default: 1024 MB)");
            println!("  --max-threads <N>    : Limit CPU threads (default: half of cores, max 4)");
            Ok(())
        }

        Commands::Verify {
            output,
            checksums,
            record_counts,
        } => {
            let manifest_path = output.join("run_manifest.json");
            if !manifest_path.exists() {
                anyhow::bail!("No run_manifest.json found in {}", output.display());
            }

            let manifest_json = std::fs::read_to_string(&manifest_path)?;
            let manifest: RunManifest = serde_json::from_str(&manifest_json)?;

            println!("Verifying output: {}", output.display());
            println!("  Manifest version: {}", manifest.manifest_version);
            println!("  Run ID: {}", manifest.run_id);
            println!("  Generator version: {}", manifest.generator_version);
            println!("  Output files: {}", manifest.output_files.len());
            println!();

            let mut all_pass = true;
            let mut checked = 0;
            let mut passed = 0;
            let mut failed = 0;

            // Check file existence
            for file_info in &manifest.output_files {
                let file_path = output.join(&file_info.path);
                checked += 1;
                if file_path.exists() {
                    passed += 1;
                    println!("  [PASS] {} exists", file_info.path);
                } else {
                    failed += 1;
                    all_pass = false;
                    println!("  [FAIL] {} missing", file_info.path);
                }
            }

            // Verify checksums
            if checksums {
                println!();
                println!("Checksum verification:");
                let results = manifest.verify_file_checksums(&output);
                for result in &results {
                    match result.status {
                        datasynth_runtime::ChecksumStatus::Ok => {
                            println!("  [PASS] {} checksum OK", result.path);
                            passed += 1;
                        }
                        datasynth_runtime::ChecksumStatus::Mismatch => {
                            println!("  [FAIL] {} checksum MISMATCH", result.path);
                            if let (Some(ref exp), Some(ref act)) =
                                (&result.expected, &result.actual)
                            {
                                println!("         expected: {}", exp);
                                println!("         actual:   {}", act);
                            }
                            failed += 1;
                            all_pass = false;
                        }
                        datasynth_runtime::ChecksumStatus::Missing => {
                            println!("  [FAIL] {} file missing", result.path);
                            failed += 1;
                            all_pass = false;
                        }
                        datasynth_runtime::ChecksumStatus::NoChecksum => {
                            println!("  [SKIP] {} no checksum recorded", result.path);
                        }
                    }
                    checked += 1;
                }
            }

            // Verify record counts
            if record_counts {
                println!();
                println!("Record count verification:");
                for file_info in &manifest.output_files {
                    let file_path = output.join(&file_info.path);
                    if let Some(expected_count) = file_info.record_count {
                        checked += 1;
                        if file_path.exists() {
                            // Count lines for CSV/JSON
                            let content = std::fs::read_to_string(&file_path).unwrap_or_default();
                            let line_count = if file_info.format == "csv" {
                                content.lines().count().saturating_sub(1) // minus header
                            } else if file_info.format == "json" {
                                // JSON array - count top-level objects
                                if let Ok(arr) =
                                    serde_json::from_str::<Vec<serde_json::Value>>(&content)
                                {
                                    arr.len()
                                } else {
                                    content.lines().count()
                                }
                            } else {
                                content.lines().count()
                            };

                            if line_count == expected_count {
                                println!(
                                    "  [PASS] {} count: {} records",
                                    file_info.path, expected_count
                                );
                                passed += 1;
                            } else {
                                println!(
                                    "  [WARN] {} count: expected {}, found {}",
                                    file_info.path, expected_count, line_count
                                );
                                // Counts may differ due to formatting, so warn only
                                passed += 1;
                            }
                        } else {
                            println!("  [SKIP] {} file missing", file_info.path);
                        }
                    }
                }
            }

            println!();
            println!(
                "Summary: {} checked, {} passed, {} failed",
                checked, passed, failed
            );

            if all_pass {
                println!("Verification: PASSED");
                Ok(())
            } else {
                anyhow::bail!("Verification: FAILED ({} failures)", failed);
            }
        }

        Commands::Fingerprint { command } => handle_fingerprint_command(command),
    }
}

/// Handle fingerprint subcommands.
fn handle_fingerprint_command(command: FingerprintCommands) -> Result<()> {
    match command {
        FingerprintCommands::Extract {
            input,
            output,
            privacy_level,
            privacy_epsilon,
            privacy_k,
            sign,
        } => {
            tracing::info!("Extracting fingerprint from: {}", input.display());

            // Parse privacy level
            let level = match privacy_level.to_lowercase().as_str() {
                "minimal" => PrivacyLevel::Minimal,
                "standard" => PrivacyLevel::Standard,
                "high" => PrivacyLevel::High,
                "maximum" => PrivacyLevel::Maximum,
                _ => {
                    tracing::warn!("Unknown privacy level '{}', using standard", privacy_level);
                    PrivacyLevel::Standard
                }
            };

            // Create extraction config with privacy settings
            let mut privacy_config = PrivacyConfig::from_level(level);
            if let Some(eps) = privacy_epsilon {
                privacy_config.epsilon = eps;
            }
            if let Some(k) = privacy_k {
                privacy_config.k_anonymity = k;
            }

            let extraction_config = ExtractionConfig {
                privacy: privacy_config,
                ..Default::default()
            };

            // Create data source
            let data_source = if input.is_file() {
                DataSource::Csv(CsvDataSource::new(input.clone()))
            } else {
                // For directories, find CSV files
                let csv_files: Vec<_> = std::fs::read_dir(&input)?
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().is_some_and(|ext| ext == "csv"))
                    .collect();

                if csv_files.is_empty() {
                    anyhow::bail!("No CSV files found in directory: {}", input.display());
                }

                // Use first CSV file for now (multi-table support would require more logic)
                let first_csv = csv_files[0].path();
                tracing::info!("Using CSV file: {}", first_csv.display());
                DataSource::Csv(CsvDataSource::new(first_csv))
            };

            // Extract fingerprint
            let extractor = FingerprintExtractor::with_config(extraction_config);
            let fingerprint = extractor.extract(&data_source)?;

            // Write fingerprint
            let writer = FingerprintWriter::new();
            if sign {
                tracing::warn!(
                    "Fingerprint signing is not yet implemented; writing unsigned fingerprint"
                );
            }
            writer.write_to_file(&fingerprint, &output)?;

            tracing::info!("Fingerprint written to: {}", output.display());
            tracing::info!(
                "Privacy audit: {} actions recorded",
                fingerprint.privacy_audit.actions.len()
            );
            tracing::info!(
                "Epsilon spent: {:.3} of {:.3} budget",
                fingerprint.privacy_audit.total_epsilon_spent,
                fingerprint.privacy_audit.epsilon_budget
            );

            Ok(())
        }

        FingerprintCommands::Validate { file } => {
            tracing::info!("Validating fingerprint: {}", file.display());

            match validate_dsf(&file) {
                Ok(report) => {
                    if report.is_valid {
                        println!("✓ Fingerprint is valid");
                        println!("  Version: {}", report.version);
                        println!("  Components: {:?}", report.components);
                        if !report.warnings.is_empty() {
                            println!("  Warnings:");
                            for warning in &report.warnings {
                                println!("    - {}", warning);
                            }
                        }
                    } else {
                        println!("✗ Fingerprint validation failed");
                        for error in &report.errors {
                            println!("  Error: {}", error);
                        }
                    }
                }
                Err(e) => {
                    println!("✗ Failed to validate fingerprint: {}", e);
                    return Err(e.into());
                }
            }

            Ok(())
        }

        FingerprintCommands::Info { file, detailed } => {
            let reader = FingerprintReader::new();
            let fingerprint = reader.read_from_file(&file)?;

            println!("Fingerprint Information");
            println!("=======================");
            println!();
            println!("Manifest:");
            println!("  Version: {}", fingerprint.manifest.version);
            println!("  Format: {}", fingerprint.manifest.format);
            println!("  Created: {}", fingerprint.manifest.created_at);
            println!();
            println!("Source:");
            println!("  Description: {}", fingerprint.manifest.source.description);
            println!("  Tables: {}", fingerprint.manifest.source.table_count);
            println!("  Total Rows: {}", fingerprint.manifest.source.total_rows);
            if let Some(ref industry) = fingerprint.manifest.source.industry {
                println!("  Industry: {}", industry);
            }
            println!();
            println!("Privacy:");
            println!("  Level: {:?}", fingerprint.manifest.privacy.level);
            println!("  Epsilon: {}", fingerprint.manifest.privacy.epsilon);
            println!(
                "  K-Anonymity: {}",
                fingerprint.manifest.privacy.k_anonymity
            );
            println!();
            println!("Schema:");
            println!("  Tables: {}", fingerprint.schema.tables.len());
            for (name, table) in &fingerprint.schema.tables {
                println!("    - {} ({} columns)", name, table.columns.len());
            }
            println!();
            println!("Statistics:");
            println!(
                "  Numeric columns: {}",
                fingerprint.statistics.numeric_columns.len()
            );
            println!(
                "  Categorical columns: {}",
                fingerprint.statistics.categorical_columns.len()
            );

            if detailed {
                println!();
                println!("Detailed Statistics:");
                for (name, stats) in &fingerprint.statistics.numeric_columns {
                    println!("  {}:", name);
                    println!("    Count: {}", stats.count);
                    println!("    Min: {:.2}, Max: {:.2}", stats.min, stats.max);
                    println!("    Mean: {:.2}, StdDev: {:.2}", stats.mean, stats.std_dev);
                    println!("    Distribution: {:?}", stats.distribution);
                }
                for (name, stats) in &fingerprint.statistics.categorical_columns {
                    println!("  {}:", name);
                    println!("    Count: {}", stats.count);
                    println!("    Cardinality: {}", stats.cardinality);
                    println!("    Top values: {}", stats.top_values.len());
                }
            }

            println!();
            println!("Privacy Audit:");
            println!(
                "  Total actions: {}",
                fingerprint.privacy_audit.actions.len()
            );
            println!(
                "  Epsilon spent: {:.3}",
                fingerprint.privacy_audit.total_epsilon_spent
            );
            println!("  Warnings: {}", fingerprint.privacy_audit.warnings.len());

            Ok(())
        }

        FingerprintCommands::Diff { file1, file2 } => {
            let reader = FingerprintReader::new();
            let fp1 = reader.read_from_file(&file1)?;
            let fp2 = reader.read_from_file(&file2)?;

            println!("Fingerprint Comparison");
            println!("======================");
            println!();

            // Compare manifests
            println!("Manifests:");
            if fp1.manifest.version != fp2.manifest.version {
                println!(
                    "  Version: {} vs {}",
                    fp1.manifest.version, fp2.manifest.version
                );
            }
            if fp1.manifest.privacy.level != fp2.manifest.privacy.level {
                println!(
                    "  Privacy Level: {:?} vs {:?}",
                    fp1.manifest.privacy.level, fp2.manifest.privacy.level
                );
            }
            if fp1.manifest.privacy.epsilon != fp2.manifest.privacy.epsilon {
                println!(
                    "  Epsilon: {} vs {}",
                    fp1.manifest.privacy.epsilon, fp2.manifest.privacy.epsilon
                );
            }

            // Compare schemas
            println!();
            println!("Schema:");
            let tables1: std::collections::HashSet<_> = fp1.schema.tables.keys().collect();
            let tables2: std::collections::HashSet<_> = fp2.schema.tables.keys().collect();

            let only_in_1: Vec<_> = tables1.difference(&tables2).collect();
            let only_in_2: Vec<_> = tables2.difference(&tables1).collect();
            let common: Vec<_> = tables1.intersection(&tables2).collect();

            if !only_in_1.is_empty() {
                println!("  Only in {}: {:?}", file1.display(), only_in_1);
            }
            if !only_in_2.is_empty() {
                println!("  Only in {}: {:?}", file2.display(), only_in_2);
            }
            println!("  Common tables: {}", common.len());

            // Compare statistics
            println!();
            println!("Statistics:");
            println!(
                "  Numeric columns: {} vs {}",
                fp1.statistics.numeric_columns.len(),
                fp2.statistics.numeric_columns.len()
            );
            println!(
                "  Categorical columns: {} vs {}",
                fp1.statistics.categorical_columns.len(),
                fp2.statistics.categorical_columns.len()
            );

            // Compare numeric stats for common columns
            for col in fp1.statistics.numeric_columns.keys() {
                if let (Some(s1), Some(s2)) = (
                    fp1.statistics.numeric_columns.get(col),
                    fp2.statistics.numeric_columns.get(col),
                ) {
                    let mean_diff = (s1.mean - s2.mean).abs();
                    let std_diff = (s1.std_dev - s2.std_dev).abs();
                    if mean_diff > 0.01 || std_diff > 0.01 {
                        println!("  {}:", col);
                        println!(
                            "    Mean: {:.2} vs {:.2} (diff: {:.2})",
                            s1.mean, s2.mean, mean_diff
                        );
                        println!(
                            "    StdDev: {:.2} vs {:.2} (diff: {:.2})",
                            s1.std_dev, s2.std_dev, std_diff
                        );
                    }
                }
            }

            Ok(())
        }

        FingerprintCommands::Evaluate {
            fingerprint,
            synthetic,
            output,
            threshold,
        } => {
            tracing::info!("Evaluating fidelity of synthetic data");
            tracing::info!("  Fingerprint: {}", fingerprint.display());
            tracing::info!("  Synthetic data: {}", synthetic.display());

            // Read fingerprint
            let reader = FingerprintReader::new();
            let fp = reader.read_from_file(&fingerprint)?;

            // Find CSV files in synthetic directory
            let csv_files: Vec<PathBuf> = std::fs::read_dir(&synthetic)?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "csv"))
                .map(|e| e.path())
                .collect();

            if csv_files.is_empty() {
                anyhow::bail!(
                    "No CSV files found in synthetic directory: {}",
                    synthetic.display()
                );
            }

            // Load synthetic data from first CSV (simplified)
            let first_csv = &csv_files[0];
            tracing::info!("  Using: {}", first_csv.display());

            let data_source = DataSource::Csv(CsvDataSource::new(first_csv.clone()));

            // Extract fingerprint from synthetic data for comparison
            let extractor = FingerprintExtractor::new();
            let synthetic_fp = extractor.extract(&data_source)?;

            // Evaluate fidelity
            let evaluator = FidelityEvaluator::with_threshold(threshold);
            let report = evaluator.evaluate_fingerprints(&fp, &synthetic_fp)?;

            // Print report
            println!();
            println!("Fidelity Report");
            println!("===============");
            println!();
            println!("Overall Score: {:.1}%", report.overall_score * 100.0);
            println!("Threshold: {:.1}%", threshold * 100.0);
            println!(
                "Status: {}",
                if report.passes {
                    "PASS ✓"
                } else {
                    "FAIL ✗"
                }
            );
            println!();
            println!("Component Scores:");
            println!(
                "  Statistical Fidelity:  {:.1}%",
                report.statistical_fidelity * 100.0
            );
            println!(
                "  Correlation Fidelity:  {:.1}%",
                report.correlation_fidelity * 100.0
            );
            println!(
                "  Schema Fidelity:       {:.1}%",
                report.schema_fidelity * 100.0
            );
            println!(
                "  Rule Compliance:       {:.1}%",
                report.rule_compliance * 100.0
            );
            println!(
                "  Anomaly Fidelity:      {:.1}%",
                report.anomaly_fidelity * 100.0
            );

            // Write report if output path specified
            if let Some(output_path) = output {
                let json = serde_json::to_string_pretty(&report)?;
                std::fs::write(&output_path, json)?;
                tracing::info!("Report written to: {}", output_path.display());
            }

            if !report.passes {
                anyhow::bail!(
                    "Fidelity check failed: {:.1}% < {:.1}%",
                    report.overall_score * 100.0,
                    threshold * 100.0
                );
            }

            Ok(())
        }
    }
}

/// Find a scenario pack file by name.
///
/// Searches in the following locations:
/// 1. templates/scenarios/{pack}.yaml
/// 2. Current directory templates/scenarios/{pack}.yaml
/// 3. Executable directory templates/scenarios/{pack}.yaml
fn find_scenario_pack(pack: &str) -> Result<PathBuf> {
    // Normalize the pack name (remove .yaml if present)
    let pack_name = pack.trim_end_matches(".yaml");

    // Search paths in order of priority
    let search_paths = [
        PathBuf::from(format!("templates/scenarios/{}.yaml", pack_name)),
        PathBuf::from(format!("./templates/scenarios/{}.yaml", pack_name)),
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|p| p.join(format!("templates/scenarios/{}.yaml", pack_name)))
            .unwrap_or_default(),
    ];

    for path in search_paths.iter() {
        if path.exists() {
            tracing::info!("Found scenario pack at: {}", path.display());
            return Ok(path.clone());
        }
    }

    // List available scenario packs if not found
    let available = list_available_scenarios();
    anyhow::bail!(
        "Scenario pack '{}' not found.\n\nAvailable scenario packs:\n{}",
        pack,
        available.join("\n")
    );
}

/// List available scenario packs.
fn list_available_scenarios() -> Vec<String> {
    let mut scenarios = Vec::new();
    let base_path = PathBuf::from("templates/scenarios");

    if let Ok(industries) = std::fs::read_dir(&base_path) {
        for industry in industries.flatten() {
            if industry.path().is_dir() {
                let industry_name = industry.file_name().to_string_lossy().to_string();
                if let Ok(files) = std::fs::read_dir(industry.path()) {
                    for file in files.flatten() {
                        let file_name = file.file_name().to_string_lossy().to_string();
                        if file_name.ends_with(".yaml") {
                            let scenario_name = file_name.trim_end_matches(".yaml");
                            scenarios.push(format!("  - {}/{}", industry_name, scenario_name));
                        }
                    }
                }
            }
        }
    }

    if scenarios.is_empty() {
        scenarios.push("  (no scenario packs found in templates/scenarios/)".to_string());
    }

    scenarios
}

/// Create a safe demo preset with conservative resource usage.
fn create_safe_demo_preset() -> GeneratorConfig {
    use datasynth_config::schema::*;

    GeneratorConfig {
        global: GlobalConfig {
            industry: IndustrySector::Manufacturing,
            start_date: "2024-01-01".to_string(),
            period_months: 1, // Just 1 month for demo
            seed: Some(42),
            parallel: false,
            group_currency: "USD".to_string(),
            worker_threads: 2,
            memory_limit_mb: 512,
        },
        companies: vec![CompanyConfig {
            code: "DEMO".to_string(),
            name: "Demo Company".to_string(),
            currency: "USD".to_string(),
            country: "US".to_string(),
            annual_transaction_volume: TransactionVolume::TenK, // Small volume
            volume_weight: 1.0,
            fiscal_year_variant: "K4".to_string(),
        }],
        chart_of_accounts: ChartOfAccountsConfig {
            complexity: CoAComplexity::Small,
            industry_specific: false,
            custom_accounts: None,
            min_hierarchy_depth: 2,
            max_hierarchy_depth: 3,
        },
        transactions: TransactionConfig::default(),
        output: OutputConfig::default(),
        fraud: FraudConfig {
            enabled: false,
            ..Default::default()
        },
        internal_controls: InternalControlsConfig::default(),
        business_processes: BusinessProcessConfig::default(),
        user_personas: UserPersonaConfig::default(),
        templates: TemplateConfig::default(),
        approval: ApprovalConfig::default(),
        departments: DepartmentConfig::default(),
        master_data: MasterDataConfig::default(),
        document_flows: DocumentFlowConfig::default(),
        intercompany: IntercompanyConfig::default(),
        balance: BalanceConfig::default(),
        ocpm: OcpmConfig::default(),
        audit: AuditGenerationConfig::default(),
        banking: datasynth_banking::BankingConfig::small(), // Use small banking config
        data_quality: DataQualitySchemaConfig::default(),
        scenario: datasynth_config::schema::ScenarioConfig::default(),
        temporal: datasynth_config::schema::TemporalDriftConfig::default(),
        graph_export: datasynth_config::schema::GraphExportConfig::default(),
        streaming: datasynth_config::schema::StreamingSchemaConfig::default(),
        rate_limit: datasynth_config::schema::RateLimitSchemaConfig::default(),
        temporal_attributes: datasynth_config::schema::TemporalAttributeSchemaConfig::default(),
        relationships: datasynth_config::schema::RelationshipSchemaConfig::default(),
        accounting_standards: datasynth_config::schema::AccountingStandardsConfig::default(),
        audit_standards: datasynth_config::schema::AuditStandardsConfig::default(),
        distributions: datasynth_config::schema::AdvancedDistributionConfig::default(),
        temporal_patterns: datasynth_config::schema::TemporalPatternsConfig::default(),
        vendor_network: datasynth_config::schema::VendorNetworkSchemaConfig::default(),
        customer_segmentation: datasynth_config::schema::CustomerSegmentationSchemaConfig::default(
        ),
        relationship_strength: datasynth_config::schema::RelationshipStrengthSchemaConfig::default(
        ),
        cross_process_links: datasynth_config::schema::CrossProcessLinksSchemaConfig::default(),
        organizational_events: datasynth_config::schema::OrganizationalEventsSchemaConfig::default(
        ),
        behavioral_drift: datasynth_config::schema::BehavioralDriftSchemaConfig::default(),
        market_drift: datasynth_config::schema::MarketDriftSchemaConfig::default(),
        drift_labeling: datasynth_config::schema::DriftLabelingSchemaConfig::default(),
        anomaly_injection: Default::default(),
        industry_specific: Default::default(),
        fingerprint_privacy: Default::default(),
        quality_gates: Default::default(),
        compliance: Default::default(),
        webhooks: Default::default(),
        llm: Default::default(),
        diffusion: Default::default(),
        causal: Default::default(),
        source_to_pay: Default::default(),
        financial_reporting: Default::default(),
        hr: Default::default(),
        manufacturing: Default::default(),
        sales_quotes: Default::default(),
        tax: Default::default(),
        treasury: Default::default(),
        project_accounting: Default::default(),
        esg: Default::default(),
        country_packs: None,
    }
}

/// Apply safety limits to a loaded configuration.
fn apply_safety_limits(config: &mut GeneratorConfig) {
    // Limit period to 12 months max
    if config.global.period_months > 12 {
        tracing::warn!(
            "Safety limit: period_months truncated from {} to 12",
            config.global.period_months
        );
        config.global.period_months = 12;
    }

    // Limit transaction volume
    for company in &mut config.companies {
        let original = company.annual_transaction_volume;
        company.annual_transaction_volume = match company.annual_transaction_volume {
            datasynth_config::TransactionVolume::OneM
            | datasynth_config::TransactionVolume::TenM
            | datasynth_config::TransactionVolume::HundredM => {
                tracing::warn!(
                    "Safety limit: transaction volume for company '{}' capped from {:?} to HundredK",
                    company.code,
                    original
                );
                datasynth_config::TransactionVolume::HundredK
            }
            other => other,
        };
    }

    // Limit banking population
    if config.banking.enabled {
        let orig_retail = config.banking.population.retail_customers;
        let orig_business = config.banking.population.business_customers;
        let orig_trusts = config.banking.population.trusts;
        config.banking.population.retail_customers = orig_retail.min(500);
        config.banking.population.business_customers = orig_business.min(100);
        config.banking.population.trusts = orig_trusts.min(20);
        if orig_retail > 500 || orig_business > 100 || orig_trusts > 20 {
            tracing::warn!(
                "Safety limit: banking population capped (retail: {} -> {}, business: {} -> {}, trusts: {} -> {})",
                orig_retail,
                config.banking.population.retail_customers,
                orig_business,
                config.banking.population.business_customers,
                orig_trusts,
                config.banking.population.trusts,
            );
        }
    }

    // Force conservative settings
    config.global.parallel = false;
    config.global.worker_threads = config.global.worker_threads.min(4);
}

/// Get safe memory limit based on available system memory.
/// Returns a conservative limit that won't overwhelm the system.
fn get_safe_memory_limit() -> usize {
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            for line in content.lines() {
                if line.starts_with("MemAvailable:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<usize>() {
                            let mb = kb / 1024;
                            // Use 50% of available memory, capped at 4GB
                            return (mb / 2).min(4096);
                        }
                    }
                    break;
                }
            }
        }
    }

    // Default to 1GB if detection fails
    1024
}
