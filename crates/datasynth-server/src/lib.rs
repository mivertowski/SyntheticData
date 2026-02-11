//! # synth-server
//!
//! gRPC and REST server for synthetic data generation.
//!
//! This crate provides a server that exposes the synthetic data generation
//! capabilities via gRPC and REST APIs for integration with other systems.
//!
//! ## Features
//!
//! - **Bulk Generation**: Generate large batches of data synchronously
//! - **Streaming**: Continuous real-time data generation with configurable throughput
//! - **Control**: Pause, resume, and stop generation streams
//! - **Configuration**: Dynamic configuration updates via API
//! - **Metrics**: Real-time generation statistics and health monitoring
//! - **WebSocket**: Real-time metrics and event streaming
//!
//! ## gRPC Usage
//!
//! ```rust,ignore
//! use datasynth_server::{SynthService, SyntheticDataServiceServer};
//! use tonic::transport::Server;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let addr = "[::1]:50051".parse()?;
//!     let service = SynthService::new(default_generator_config());
//!
//!     Server::builder()
//!         .add_service(SyntheticDataServiceServer::new(service))
//!         .serve(addr)
//!         .await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## REST Usage
//!
//! ```rust,ignore
//! use datasynth_server::{rest, SynthService, grpc::service::default_generator_config};
//! use axum::Router;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let service = SynthService::new(default_generator_config());
//!     let router = rest::create_router(service);
//!
//!     let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
//!     axum::serve(listener, router).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod config_loader;
pub mod grpc;
pub mod jobs;
pub mod observability;
pub mod rest;
pub mod tls;

pub use grpc::{SynthService, SyntheticDataServiceServer};
