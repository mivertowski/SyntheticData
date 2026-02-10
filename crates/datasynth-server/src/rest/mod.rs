//! REST and WebSocket API implementation.

mod auth;
mod rate_limit;
pub mod request_id;
pub mod request_logging;
pub mod request_validation;
mod routes;
pub mod security_headers;
mod websocket;

pub use auth::{auth_middleware, AuthConfig};
pub use rate_limit::{rate_limit_middleware, RateLimitConfig, RateLimiter};
pub use routes::{
    create_router, create_router_full, create_router_with_auth, create_router_with_cors,
    CorsConfig, TimeoutConfig,
};
pub use websocket::MetricsStream;
