//! Security headers middleware.
//!
//! Injects security-related response headers on all responses.

use axum::{
    body::Body,
    http::{Request, Response},
    middleware::Next,
};

/// Security headers middleware.
///
/// Adds the following headers to all responses:
/// - `X-Content-Type-Options: nosniff`
/// - `X-Frame-Options: DENY`
/// - `X-XSS-Protection: 0` (modern best practice - rely on CSP instead)
/// - `Referrer-Policy: strict-origin-when-cross-origin`
/// - `Content-Security-Policy: default-src 'none'; frame-ancestors 'none'`
/// - `Cache-Control: no-store` (API responses should not be cached)
pub async fn security_headers_middleware(request: Request<Body>, next: Next) -> Response<Body> {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    headers.insert("x-content-type-options", "nosniff".parse().unwrap());
    headers.insert("x-frame-options", "DENY".parse().unwrap());
    headers.insert("x-xss-protection", "0".parse().unwrap());
    headers.insert(
        "referrer-policy",
        "strict-origin-when-cross-origin".parse().unwrap(),
    );
    headers.insert(
        "content-security-policy",
        "default-src 'none'; frame-ancestors 'none'"
            .parse()
            .unwrap(),
    );
    headers.insert("cache-control", "no-store".parse().unwrap());

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

    fn test_router() -> Router {
        Router::new()
            .route("/test", get(ok_handler))
            .layer(axum::middleware::from_fn(security_headers_middleware))
    }

    #[tokio::test]
    async fn test_security_headers_present() {
        let router = test_router();
        let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

        let response = router.oneshot(request).await.unwrap();
        let headers = response.headers();

        assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
        assert_eq!(headers.get("x-frame-options").unwrap(), "DENY");
        assert_eq!(headers.get("x-xss-protection").unwrap(), "0");
        assert_eq!(
            headers.get("referrer-policy").unwrap(),
            "strict-origin-when-cross-origin"
        );
        assert_eq!(
            headers.get("content-security-policy").unwrap(),
            "default-src 'none'; frame-ancestors 'none'"
        );
        assert_eq!(headers.get("cache-control").unwrap(), "no-store");
    }
}
