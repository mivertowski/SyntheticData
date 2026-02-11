//! Core traits for generators, output sinks, post-processors, and plugins.

mod generator;
pub mod plugin;
mod post_processor;
pub mod registry;
mod sink;
mod streaming;

pub use generator::*;
pub use plugin::{
    GeneratedRecord, GenerationContext, GeneratorPlugin, PluginInfo, PluginType, SinkPlugin,
    SinkSummary, TransformPlugin,
};
pub use post_processor::*;
pub use registry::PluginRegistry;
pub use sink::*;
pub use streaming::*;
