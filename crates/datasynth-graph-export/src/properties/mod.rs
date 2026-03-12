//! Property serializers for converting domain model fields into property maps.
//!
//! Each serializer handles a specific entity type (identified by `entity_type()`)
//! and converts the domain model's strongly-typed fields into
//! `HashMap<String, serde_json::Value>` for export.
//!
//! ## Implemented Serializers
//!
//! - [`ControlPropertySerializer`](control::ControlPropertySerializer) — `InternalControl` (code 503)
//! - [`RiskPropertySerializer`](risk::RiskPropertySerializer) — `RiskAssessment` (code 364)

pub mod control;
pub mod risk;

use crate::traits::PropertySerializer;

/// Returns all built-in property serializers.
///
/// Used by [`GraphExportPipeline::standard()`](crate::pipeline::GraphExportPipeline::standard)
/// to register the default set of serializers.
pub fn all_serializers() -> Vec<Box<dyn PropertySerializer>> {
    vec![
        Box::new(control::ControlPropertySerializer),
        Box::new(risk::RiskPropertySerializer),
    ]
}
