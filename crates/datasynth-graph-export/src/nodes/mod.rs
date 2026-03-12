//! Node synthesizers for creating entity types that HypergraphBuilder doesn't produce.
//!
//! Each synthesizer reads from specific fields of [`EnhancedGenerationResult`] and produces
//! [`ExportNode`]s with assigned IDs via the pipeline's [`IdMap`].
//!
//! ## Synthesizer Domains
//!
//! | Module              | Entity Codes | Source Data                |
//! |---------------------|-------------|----------------------------|
//! | `aml_alert`         | 505         | Banking suspicious txns    |
//! | `kyc_profile`       | 504         | Banking customers          |
//! | `red_flag`          | 510         | Fraud red flags            |
//! | `collusion_ring`    | 511         | Fraud collusion rings      |
//! | `tax`               | 410–415     | Tax snapshot               |
//! | `treasury`          | 420–427     | Treasury snapshot          |
//! | `esg`               | 430–442     | ESG snapshot               |
//! | `project`           | 450–455     | Project accounting snapshot|
//! | `intercompany`      | 460–462     | IC matched pairs/elims     |
//! | `subledger_recon`   | 463         | GL-to-subledger recon      |
//! | `compliance`        | 480–483     | Compliance regulations     |
//! | `temporal_events`   | 470–473     | Process/org/disruption     |
//! | `ocel_events`       | 400         | OCPM event log             |
//! | `audit_procedures`  | 366–379     | Audit procedure entities   |

pub mod aml_alert;
pub mod audit_procedures;
pub mod collusion_ring;
pub mod compliance;
pub mod esg;
pub mod intercompany;
pub mod kyc_profile;
pub mod ocel_events;
pub mod project;
pub mod red_flag;
pub mod subledger_recon;
pub mod tax;
pub mod temporal_events;
pub mod treasury;

use crate::traits::NodeSynthesizer;

/// Returns all built-in node synthesizers.
///
/// Used by [`GraphExportPipeline::standard()`](crate::pipeline::GraphExportPipeline::standard)
/// to register the default set of node synthesizers.
pub fn all_synthesizers() -> Vec<Box<dyn NodeSynthesizer>> {
    vec![
        // Banking / AML
        Box::new(aml_alert::AmlAlertNodeSynthesizer),
        Box::new(kyc_profile::KycProfileNodeSynthesizer),
        // Fraud
        Box::new(red_flag::RedFlagNodeSynthesizer),
        Box::new(collusion_ring::CollusionRingNodeSynthesizer),
        // Tax
        Box::new(tax::TaxNodeSynthesizer),
        // Treasury / Cash Management
        Box::new(treasury::TreasuryNodeSynthesizer),
        // ESG
        Box::new(esg::EsgNodeSynthesizer),
        // Project Accounting
        Box::new(project::ProjectNodeSynthesizer),
        // Intercompany
        Box::new(intercompany::IntercompanyNodeSynthesizer),
        // Subledger Reconciliation
        Box::new(subledger_recon::SubledgerReconNodeSynthesizer),
        // Compliance & Regulatory
        Box::new(compliance::ComplianceNodeSynthesizer),
        // Temporal Events
        Box::new(temporal_events::TemporalEventsNodeSynthesizer),
        // OCEL Events
        Box::new(ocel_events::OcelEventsNodeSynthesizer),
        // Audit Procedures (ISA 505/330/530/520/610/550)
        Box::new(audit_procedures::AuditProcedureNodeSynthesizer),
    ]
}

/// Convert a [`GraphPropertyValue`] map to a [`serde_json::Value`] map.
///
/// Used by synthesizers that leverage `ToNodeProperties` trait implementations
/// from `datasynth-core`.
pub(crate) fn graph_props_to_json(
    props: std::collections::HashMap<String, datasynth_core::models::graph_properties::GraphPropertyValue>,
) -> std::collections::HashMap<String, serde_json::Value> {
    props
        .into_iter()
        .map(|(k, v)| {
            let json_val = match v {
                datasynth_core::models::graph_properties::GraphPropertyValue::String(s) => {
                    serde_json::Value::String(s)
                }
                datasynth_core::models::graph_properties::GraphPropertyValue::Int(i) => {
                    serde_json::json!(i)
                }
                datasynth_core::models::graph_properties::GraphPropertyValue::Float(f) => {
                    serde_json::json!(f)
                }
                datasynth_core::models::graph_properties::GraphPropertyValue::Decimal(d) => {
                    serde_json::Value::String(d.to_string())
                }
                datasynth_core::models::graph_properties::GraphPropertyValue::Bool(b) => {
                    serde_json::Value::Bool(b)
                }
                datasynth_core::models::graph_properties::GraphPropertyValue::Date(d) => {
                    serde_json::Value::String(format!("{d}T00:00:00Z"))
                }
                datasynth_core::models::graph_properties::GraphPropertyValue::StringList(v) => {
                    serde_json::Value::Array(
                        v.into_iter().map(serde_json::Value::String).collect(),
                    )
                }
            };
            (k, json_val)
        })
        .collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn all_synthesizers_have_unique_names() {
        let synths = all_synthesizers();
        let mut seen: HashSet<String> = HashSet::new();
        for s in &synths {
            let name = s.name().to_string();
            assert!(
                seen.insert(name.clone()),
                "Duplicate synthesizer name: {name}"
            );
        }
    }

    #[test]
    fn all_synthesizers_count() {
        let synths = all_synthesizers();
        assert_eq!(synths.len(), 14);
    }

    #[test]
    fn graph_props_to_json_converts_all_variants() {
        use datasynth_core::models::graph_properties::GraphPropertyValue;
        use std::collections::HashMap;

        let mut props = HashMap::new();
        props.insert("s".into(), GraphPropertyValue::String("hello".into()));
        props.insert("i".into(), GraphPropertyValue::Int(42));
        props.insert("f".into(), GraphPropertyValue::Float(3.14));
        props.insert(
            "d".into(),
            GraphPropertyValue::Decimal(rust_decimal::Decimal::new(1234, 2)),
        );
        props.insert("b".into(), GraphPropertyValue::Bool(true));
        props.insert(
            "dt".into(),
            GraphPropertyValue::Date(chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()),
        );
        props.insert(
            "sl".into(),
            GraphPropertyValue::StringList(vec!["a".into(), "b".into()]),
        );

        let json = graph_props_to_json(props);
        assert_eq!(json.get("s").unwrap(), &serde_json::json!("hello"));
        assert_eq!(json.get("i").unwrap(), &serde_json::json!(42));
        assert_eq!(json.get("b").unwrap(), &serde_json::json!(true));
        assert_eq!(json.get("d").unwrap(), &serde_json::json!("12.34"));
        assert_eq!(
            json.get("dt").unwrap(),
            &serde_json::json!("2024-01-15T00:00:00Z")
        );
        assert!(json.get("sl").unwrap().is_array());
    }
}
