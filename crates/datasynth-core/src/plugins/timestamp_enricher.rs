//! Example `TransformPlugin` that enriches records with generation metadata.
//!
//! Adds a UTC timestamp and plugin version to every record that passes through.

use crate::error::SynthError;
use crate::traits::plugin::{GeneratedRecord, TransformPlugin};
use chrono::Utc;

/// A transform plugin that adds `_generated_at` (ISO 8601 UTC) and
/// `_plugin_version` fields to each record.
pub struct TimestampEnricher;

impl TimestampEnricher {
    /// Create a new `TimestampEnricher`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for TimestampEnricher {
    fn default() -> Self {
        Self::new()
    }
}

impl TransformPlugin for TimestampEnricher {
    fn name(&self) -> &str {
        "timestamp_enricher"
    }

    fn transform(&self, records: Vec<GeneratedRecord>) -> Result<Vec<GeneratedRecord>, SynthError> {
        let now = Utc::now().to_rfc3339();
        let version = serde_json::Value::String("1.0.0".to_string());

        let enriched = records
            .into_iter()
            .map(|mut record| {
                record
                    .fields
                    .insert("_generated_at".to_string(), serde_json::json!(now));
                record
                    .fields
                    .insert("_plugin_version".to_string(), version.clone());
                record
            })
            .collect();

        Ok(enriched)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_enricher_adds_fields() {
        let enricher = TimestampEnricher::new();

        let records = vec![
            GeneratedRecord::new("invoice").with_field("id", serde_json::json!("INV001")),
            GeneratedRecord::new("invoice").with_field("id", serde_json::json!("INV002")),
        ];

        let result = enricher
            .transform(records)
            .expect("transform should succeed");
        assert_eq!(result.len(), 2);

        for record in &result {
            assert!(
                record.fields.contains_key("_generated_at"),
                "should have _generated_at field"
            );
            assert_eq!(
                record.get_str("_plugin_version"),
                Some("1.0.0"),
                "should have _plugin_version = 1.0.0"
            );
            // Original field should be preserved.
            assert!(
                record.get_str("id").is_some(),
                "original id field should be preserved"
            );
        }
    }

    #[test]
    fn test_timestamp_enricher_empty_input() {
        let enricher = TimestampEnricher::new();
        let result = enricher
            .transform(vec![])
            .expect("transform should succeed");
        assert!(result.is_empty());
    }

    #[test]
    fn test_timestamp_enricher_preserves_existing_fields() {
        let enricher = TimestampEnricher::new();

        let records = vec![GeneratedRecord::new("order")
            .with_field("customer", serde_json::json!("CUST001"))
            .with_field("amount", serde_json::json!(1500.0))];

        let result = enricher
            .transform(records)
            .expect("transform should succeed");
        let record = &result[0];

        assert_eq!(record.get_str("customer"), Some("CUST001"));
        assert_eq!(record.get("amount").and_then(|v| v.as_f64()), Some(1500.0));
        assert!(record.fields.contains_key("_generated_at"));
        assert_eq!(record.get_str("_plugin_version"), Some("1.0.0"));
    }
}
