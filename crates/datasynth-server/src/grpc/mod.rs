//! gRPC service implementation for synthetic data generation.

pub mod auth_interceptor;
pub mod service;

// Include the generated protobuf code
#[allow(clippy::all)]
#[allow(warnings)]
pub mod synth {
    include!("synth.rs");
}

pub use service::SynthService;
pub use synth::synthetic_data_service_server::SyntheticDataServiceServer;
// Re-export proto types for testing
pub use synth::{
    synthetic_data_service_server::SyntheticDataService, BulkGenerateRequest, BulkGenerateResponse,
    ConfigRequest, ConfigResponse, ControlAction, ControlCommand, ControlResponse,
    GenerationConfig, HealthResponse, MetricsResponse, StreamDataRequest,
};
