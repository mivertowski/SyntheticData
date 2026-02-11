//! Request ID middleware.
//!
//! Generates or preserves a unique request ID for each request.

use axum::{body::Body, http::Request, middleware::Next, response::Response};
use uuid::Uuid;

const REQUEST_ID_HEADER: &str = "x-request-id";

/// Request ID middleware.
///
/// If the request already has an `X-Request-Id` header, it is preserved.
/// Otherwise, a new UUID v4 is generated and added to both the request
/// extension and the response headers.
pub async fn request_id_middleware(mut request: Request<Body>, next: Next) -> Response {
    let request_id = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Store in request extensions for logging
    request
        .extensions_mut()
        .insert(RequestId(request_id.clone()));

    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert(REQUEST_ID_HEADER, request_id.parse().unwrap());

    response
}

/// Request ID stored in request extensions.
#[derive(Clone, Debug)]
pub struct RequestId(pub String);

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use axum::{routing::get, Router};
    use tower::ServiceExt;

    async fn ok_handler() -> &'static str {
        "ok"
    }

    fn test_router() -> Router {
        Router::new()
            .route("/test", get(ok_handler))
            .layer(axum::middleware::from_fn(request_id_middleware))
    }

    #[tokio::test]
    async fn test_generates_request_id() {
        let router = test_router();
        let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert!(response.headers().get("x-request-id").is_some());
    }

    #[tokio::test]
    async fn test_preserves_client_request_id() {
        let router = test_router();
        let request = Request::builder()
            .uri("/test")
            .header("x-request-id", "client-123")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(
            response.headers().get("x-request-id").unwrap(),
            "client-123"
        );
    }
}
