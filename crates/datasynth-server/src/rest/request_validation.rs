//! Request validation middleware.
//!
//! Enforces Content-Type for mutation requests (POST/PUT/PATCH).

use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

/// Request validation middleware.
///
/// - POST, PUT, PATCH requests must include `Content-Type: application/json`
/// - GET, DELETE, OPTIONS, HEAD bypass this check
pub async fn request_validation_middleware(request: Request<Body>, next: Next) -> Response {
    let method = request.method().clone();

    // Only validate mutation methods
    if matches!(method, Method::POST | Method::PUT | Method::PATCH) {
        // Allow empty bodies (some POST endpoints don't need a body)
        let has_body = request
            .headers()
            .get(header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0)
            > 0;

        if has_body {
            let content_type = request
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");

            if !content_type.starts_with("application/json") {
                return (
                    StatusCode::UNSUPPORTED_MEDIA_TYPE,
                    "Content-Type must be application/json",
                )
                    .into_response();
            }
        }
    }

    next.run(request).await
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use axum::{routing::post, Router};
    use tower::ServiceExt;

    async fn ok_handler() -> &'static str {
        "ok"
    }

    fn test_router() -> Router {
        Router::new()
            .route("/test", post(ok_handler))
            .layer(axum::middleware::from_fn(request_validation_middleware))
    }

    #[tokio::test]
    async fn test_post_with_json_content_type() {
        let router = test_router();
        let request = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::CONTENT_LENGTH, "2")
            .body(Body::from("{}"))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_post_with_wrong_content_type() {
        let router = test_router();
        let request = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header(header::CONTENT_TYPE, "text/plain")
            .header(header::CONTENT_LENGTH, "5")
            .body(Body::from("hello"))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn test_post_without_body_passes() {
        let router = test_router();
        let request = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
