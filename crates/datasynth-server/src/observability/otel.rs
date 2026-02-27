//! OpenTelemetry initialization (feature-gated behind `otel`).
//!
//! Sets up:
//! - OTLP trace export to a configurable endpoint (default: `http://localhost:4317`)
//! - Prometheus metric bridge for `/metrics` scraping

#[cfg(feature = "otel")]
use opentelemetry::global;
#[cfg(feature = "otel")]
use opentelemetry::trace::TracerProvider as _;
#[cfg(feature = "otel")]
use opentelemetry_otlp::WithExportConfig;
#[cfg(feature = "otel")]
use opentelemetry_sdk::trace::SdkTracerProvider;

/// Initialize OpenTelemetry tracing and metrics.
///
/// Returns a tracing layer that can be composed with the subscriber registry.
///
/// # Environment variables
///
/// - `OTEL_EXPORTER_OTLP_ENDPOINT`: OTLP endpoint (default: `http://localhost:4317`)
/// - `OTEL_SERVICE_NAME`: Service name (default: `datasynth-server`)
#[cfg(feature = "otel")]
pub fn init_otel_layer() -> Result<
    tracing_opentelemetry::OpenTelemetryLayer<
        tracing_subscriber::Registry,
        opentelemetry_sdk::trace::SdkTracer,
    >,
    Box<dyn std::error::Error>,
> {
    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&endpoint)
        .build()?;

    let tracer_provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .build();

    let tracer = tracer_provider.tracer("datasynth-server");
    global::set_tracer_provider(tracer_provider);

    Ok(tracing_opentelemetry::layer().with_tracer(tracer))
}

/// Initialize the Prometheus metrics exporter.
///
/// Returns a `PrometheusExporter` whose `registry()` can be used
/// to render the `/metrics` endpoint.
#[cfg(feature = "otel")]
pub fn init_prometheus_exporter(
) -> Result<opentelemetry_prometheus::PrometheusExporter, Box<dyn std::error::Error>> {
    let exporter = opentelemetry_prometheus::exporter()
        .with_registry(prometheus::default_registry().clone())
        .build()?;
    Ok(exporter)
}

/// Render Prometheus metrics from the default registry.
#[cfg(feature = "otel")]
pub fn render_prometheus_metrics() -> String {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let metric_families = prometheus::default_registry().gather();
    let mut buf = Vec::new();
    if let Err(e) = encoder.encode(&metric_families, &mut buf) {
        tracing::error!("Failed to encode Prometheus metrics: {}", e);
        return String::from("# Error encoding metrics\n");
    }
    String::from_utf8(buf).unwrap_or_else(|e| {
        tracing::error!("Prometheus metrics buffer is not valid UTF-8: {}", e);
        String::from("# Error: invalid UTF-8 in metrics\n")
    })
}

/// Shutdown OpenTelemetry providers gracefully.
#[cfg(feature = "otel")]
pub fn shutdown_otel() {
    // In OTel SDK 0.31+, shutdown is handled by dropping the provider
    // or calling provider.shutdown() on the stored instance.
    // The global provider is shut down when it is dropped.
    drop(global::tracer_provider());
}
