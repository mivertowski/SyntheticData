//! Built-in plugin examples demonstrating the Plugin SDK.
//!
//! These plugins serve as reference implementations for custom generators,
//! sinks, and transforms.

mod csv_echo;
mod timestamp_enricher;

pub use csv_echo::CsvEchoSink;
pub use timestamp_enricher::TimestampEnricher;
