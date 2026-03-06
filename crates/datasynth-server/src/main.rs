//! Synthetic Data gRPC + REST Server
//!
//! Starts both gRPC and REST servers for synthetic data generation.

use std::net::SocketAddr;
use std::panic;
use std::sync::Arc;

use clap::Parser;
use tokio::signal;
use tonic::transport::Server;
use tracing::{error, info};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use datasynth_server::grpc::service::{default_generator_config, ServerState, SynthService};
use datasynth_server::rest::{
    create_router_full_with_backend, AuthConfig, CorsConfig, RateLimitBackend, RateLimitConfig,
    TimeoutConfig,
};
use datasynth_server::SyntheticDataServiceServer;

#[derive(Parser, Debug)]
#[command(name = "synth-server")]
#[command(about = "Synthetic Data gRPC + REST Server", long_about = None)]
struct Args {
    /// Host address to bind to
    #[arg(short = 'H', long, default_value = "0.0.0.0")]
    host: String,

    /// gRPC port to listen on
    #[arg(short, long, default_value = "50051")]
    port: u16,

    /// REST API port to listen on
    #[arg(long, default_value = "3000")]
    rest_port: u16,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Number of worker threads (0 = automatic based on CPU cores)
    #[arg(short, long, default_value = "0")]
    worker_threads: usize,

    /// API keys for authentication (comma-separated)
    #[arg(long, env = "DATASYNTH_API_KEYS")]
    api_keys: Option<String>,

    /// Redis URL for distributed rate limiting (e.g., redis://127.0.0.1:6379).
    /// When provided, rate limiting state is shared across all server instances.
    /// Requires the `redis` feature to be enabled.
    #[arg(long, env = "DATASYNTH_REDIS_URL")]
    redis_url: Option<String>,

    /// TLS certificate file path (PEM format)
    #[cfg(feature = "tls")]
    #[arg(long, env = "DATASYNTH_TLS_CERT")]
    tls_cert: Option<String>,

    /// TLS private key file path (PEM format)
    #[cfg(feature = "tls")]
    #[arg(long, env = "DATASYNTH_TLS_KEY")]
    tls_key: Option<String>,

    /// JWT issuer URL for OIDC token validation (e.g., "https://auth.example.com/realms/main")
    #[cfg(feature = "jwt")]
    #[arg(long, env = "DATASYNTH_JWT_ISSUER")]
    jwt_issuer: Option<String>,

    /// JWT audience claim for token validation (e.g., "datasynth-api")
    #[cfg(feature = "jwt")]
    #[arg(long, env = "DATASYNTH_JWT_AUDIENCE")]
    jwt_audience: Option<String>,

    /// PEM-encoded RSA public key for JWT verification
    #[cfg(feature = "jwt")]
    #[arg(long, env = "DATASYNTH_JWT_PUBLIC_KEY")]
    jwt_public_key: Option<String>,

    /// Enable RBAC (role-based access control)
    #[arg(long, env = "DATASYNTH_RBAC_ENABLED", default_value = "false")]
    rbac_enabled: bool,

    /// Path to audit log file (stdout if not specified)
    #[arg(long, env = "DATASYNTH_AUDIT_LOG")]
    audit_log: Option<String>,
}

/// Setup panic hook to log panics before aborting.
fn setup_panic_hook() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        error!("Server panic: {}", panic_info);
        default_hook(panic_info);
    }));
}

/// Wait for shutdown signal (Ctrl+C or SIGTERM).
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, initiating graceful shutdown...");
        }
        _ = terminate => {
            info!("Received SIGTERM, initiating graceful shutdown...");
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Setup panic hook for crash logging
    setup_panic_hook();

    // Build tokio runtime with configured worker threads
    let mut runtime_builder = tokio::runtime::Builder::new_multi_thread();
    runtime_builder.enable_all();

    if args.worker_threads > 0 {
        runtime_builder.worker_threads(args.worker_threads);
        eprintln!("Using {} worker threads", args.worker_threads);
    } else {
        // 0 means automatic - tokio defaults to num_cpus
        let num_cpus = std::thread::available_parallelism()
            .map(std::num::NonZero::get)
            .unwrap_or(4);
        eprintln!("Using {num_cpus} worker threads (auto-detected)");
    }

    let runtime = runtime_builder.build()?;

    runtime.block_on(async {
        // Initialize structured logging
        let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            if args.verbose {
                EnvFilter::new("debug")
            } else {
                EnvFilter::new("info")
            }
        });

        let fmt_layer = fmt::layer()
            .json()
            .with_target(true)
            .with_thread_ids(true)
            .with_file(false)
            .with_line_number(false);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .init();

        // Parse address
        let grpc_addr: SocketAddr = format!("{}:{}", args.host, args.port)
            .parse()
            .expect("Invalid gRPC address");

        let rest_addr: SocketAddr = format!("{}:{}", args.host, args.rest_port)
            .parse()
            .expect("Invalid REST address");

        // Create shared state
        let config = default_generator_config();
        let state = Arc::new(ServerState::new(config));

        // Create gRPC service
        let grpc_service = SynthService::with_state(Arc::clone(&state));

        // Create REST service with shared state
        let rest_service = SynthService::with_state(Arc::clone(&state));

        // Configure auth
        #[allow(unused_mut)]
        let mut auth_config = if let Some(keys_str) = &args.api_keys {
            let keys: Vec<String> = keys_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if keys.is_empty() {
                AuthConfig::default()
            } else {
                info!("API key authentication enabled with {} key(s)", keys.len());
                AuthConfig::with_api_keys(keys)
            }
        } else {
            AuthConfig::default()
        };

        // Configure JWT authentication (if feature enabled and args provided)
        #[cfg(feature = "jwt")]
        {
            use datasynth_server::rest::JwtConfig;

            if let (Some(issuer), Some(audience)) = (&args.jwt_issuer, &args.jwt_audience) {
                let mut jwt_config = JwtConfig::new(issuer.clone(), audience.clone());
                if let Some(ref key_pem) = args.jwt_public_key {
                    jwt_config = jwt_config.with_public_key(key_pem.clone());
                }
                auth_config = auth_config
                    .with_jwt(jwt_config)
                    .expect("Failed to configure JWT validation");
                info!(
                    "JWT authentication enabled (issuer: {}, audience: {})",
                    issuer, audience
                );
            }
        }

        // Configure rate limiting backend
        let rate_limit_config = RateLimitConfig::default();
        let rate_limit_backend = {
            #[cfg(feature = "redis")]
            {
                if let Some(ref redis_url) = args.redis_url {
                    match RateLimitBackend::redis(redis_url, rate_limit_config.clone()).await {
                        Ok(backend) => {
                            info!(
                                "Using Redis-backed distributed rate limiting ({})",
                                redis_url
                            );
                            backend
                        }
                        Err(e) => {
                            error!(
                                "Failed to connect to Redis at {}: {}. Falling back to in-memory rate limiting.",
                                redis_url, e
                            );
                            RateLimitBackend::in_memory(rate_limit_config)
                        }
                    }
                } else {
                    info!("Using in-memory rate limiting (single instance)");
                    RateLimitBackend::in_memory(rate_limit_config)
                }
            }
            #[cfg(not(feature = "redis"))]
            {
                if args.redis_url.is_some() {
                    error!(
                        "--redis-url was provided but the `redis` feature is not enabled. \
                         Rebuild with `cargo build --features redis` to enable Redis rate limiting. \
                         Falling back to in-memory rate limiting."
                    );
                }
                info!("Using in-memory rate limiting (single instance)");
                RateLimitBackend::in_memory(rate_limit_config)
            }
        };

        let router = create_router_full_with_backend(
            rest_service,
            CorsConfig::default(),
            auth_config,
            rate_limit_backend,
            TimeoutConfig::default(),
        );

        info!(
            "Starting Synthetic Data Server - gRPC on {}, REST on {}",
            grpc_addr, rest_addr
        );

        // Start both servers concurrently
        let grpc_handle = tokio::spawn(async move {
            Server::builder()
                .add_service(SyntheticDataServiceServer::new(grpc_service))
                .serve_with_shutdown(grpc_addr, shutdown_signal())
                .await
                .expect("gRPC server failed");
        });

        let rest_handle = tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind(rest_addr)
                .await
                .expect("Failed to bind REST listener");
            axum::serve(listener, router)
                .with_graceful_shutdown(shutdown_signal())
                .await
                .expect("REST server failed");
        });

        // Wait for either server to finish (shutdown)
        tokio::select! {
            _ = grpc_handle => {
                info!("gRPC server shutdown complete");
            }
            _ = rest_handle => {
                info!("REST server shutdown complete");
            }
        }

        info!("Server shutdown complete");
    });

    Ok(())
}
