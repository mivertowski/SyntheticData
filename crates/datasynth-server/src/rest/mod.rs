//! REST and WebSocket API implementation.

pub mod audit;
mod auth;
pub mod rbac;
mod rate_limit;
mod rate_limit_backend;
#[cfg(feature = "redis")]
mod redis_rate_limit;
pub mod request_id;
pub mod request_logging;
pub mod request_validation;
mod routes;
pub mod security_headers;
mod websocket;

pub use audit::{AuditConfig, AuditEvent, AuditLogger, AuditOutcome, JsonAuditLogger, NoopAuditLogger};
pub use auth::{auth_middleware, AuthConfig};
#[cfg(feature = "jwt")]
pub use auth::{JwtConfig, JwtValidator, TokenClaims};
pub use rbac::{Permission, RbacConfig, Role, RolePermissions};
pub use rate_limit::{rate_limit_middleware, RateLimitConfig, RateLimiter};
pub use rate_limit_backend::{backend_rate_limit_middleware, RateLimitBackend};
#[cfg(feature = "redis")]
pub use redis_rate_limit::RedisRateLimiter;
pub use routes::{
    create_router, create_router_full, create_router_full_with_backend, create_router_with_auth,
    create_router_with_cors, CorsConfig, TimeoutConfig,
};
pub use websocket::MetricsStream;
