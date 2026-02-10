//! Structured request logging middleware.
//!
//! Configures `tower_http::TraceLayer` for structured request/response spans
//! with request_id, method, path, status, and latency.

use axum::{body::Body, http::Request, middleware::Next, response::Response};
use tracing::{info_span, Instrument};

/// Request logging middleware that creates structured spans.
///
/// Logs method, path, status code, and duration for every request.
pub async fn request_logging_middleware(request: Request<Body>, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let start = std::time::Instant::now();

    // Extract request ID if present (set by request_id middleware)
    let request_id = request
        .extensions()
        .get::<crate::rest::request_id::RequestId>()
        .map(|r| r.0.clone())
        .unwrap_or_default();

    let span = info_span!(
        "http_request",
        method = %method,
        path = %path,
        request_id = %request_id,
    );

    let response = next.run(request).instrument(span).await;

    let duration_ms = start.elapsed().as_millis();
    let status = response.status().as_u16();

    tracing::info!(
        method = %method,
        path = %path,
        status = status,
        latency_ms = duration_ms,
        request_id = %request_id,
        "Request completed"
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::get, Router};
    use tower::ServiceExt;

    async fn ok_handler() -> &'static str {
        "ok"
    }

    #[tokio::test]
    async fn test_request_logging_passes_through() {
        let router = Router::new()
            .route("/test", get(ok_handler))
            .layer(axum::middleware::from_fn(request_logging_middleware));

        let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), 200);
    }
}
