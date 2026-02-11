//! JSON report generation.
//!
//! Generates machine-readable JSON reports for CI/CD integration.

use super::{EvaluationReport, ReportGenerator};
use crate::error::EvalResult;
use serde_json;

/// JSON report generator.
pub struct JsonReportGenerator {
    /// Whether to pretty-print the JSON.
    pretty: bool,
}

impl JsonReportGenerator {
    /// Create a new generator.
    pub fn new(pretty: bool) -> Self {
        Self { pretty }
    }
}

impl Default for JsonReportGenerator {
    fn default() -> Self {
        Self::new(true)
    }
}

impl ReportGenerator for JsonReportGenerator {
    fn generate(&self, report: &EvaluationReport) -> EvalResult<String> {
        let json = if self.pretty {
            serde_json::to_string_pretty(report)?
        } else {
            serde_json::to_string(report)?
        };
        Ok(json)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::report::ReportMetadata;
    use chrono::Utc;

    #[test]
    fn test_json_generation() {
        let metadata = ReportMetadata {
            generated_at: Utc::now(),
            version: "1.0.0".to_string(),
            data_source: "test".to_string(),
            thresholds_name: "default".to_string(),
            records_evaluated: 1000,
            duration_ms: 500,
        };

        let report = EvaluationReport::new(metadata, None, None, None, None);
        let generator = JsonReportGenerator::new(true);
        let json = generator.generate(&report).unwrap();

        assert!(json.contains("generated_at"));
        assert!(json.contains("passes"));
    }
}
