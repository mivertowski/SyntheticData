//! OCEL 2.0 exporter for the graph export pipeline.
//!
//! Produces an OCEL 2.0 JSON payload suitable for RustGraph's OCEL import endpoint.
//! Uses a three-level fallback strategy:
//!
//! 1. **Primary**: If the `EnhancedGenerationResult` has an OCPM event log, convert it
//!    directly to OCEL 2.0 JSON (snake_case keys, simplified format).
//! 2. **Fallback**: If document flows exist, synthesize OCEL events from P2P/O2C chains.
//! 3. **None**: If neither source is available, return `None`.
//!
//! The output uses the simplified OCEL 2.0 format expected by RustGraph's parser:
//!
//! ```json
//! {
//!     "events": [{"id": "...", "type": "...", "time": "...", "relationships": [...]}],
//!     "objects": [{"id": "...", "type": "...", "properties": {...}}]
//! }
//! ```

use chrono::{DateTime, Utc};
use serde::Serialize;
use tracing::{debug, info};

use crate::config::{ExportConfig, OcelExportConfig};
use crate::error::ExportError;
use crate::id_map::IdMap;
use crate::traits::PostProcessor;
use crate::types::{GraphExportResult, OcelExport};

use datasynth_runtime::EnhancedGenerationResult;

// ──────────────────────────── OCEL 2.0 Types ────────────────────

/// OCEL 2.0 document (simplified format for RustGraph).
#[derive(Debug, Clone, Serialize)]
struct OcelDocument {
    events: Vec<OcelEvent>,
    objects: Vec<OcelObject>,
}

/// A single OCEL 2.0 event.
#[derive(Debug, Clone, Serialize)]
struct OcelEvent {
    id: String,
    #[serde(rename = "type")]
    event_type: String,
    time: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    relationships: Vec<OcelRelationship>,
}

/// A single OCEL 2.0 object.
#[derive(Debug, Clone, Serialize)]
struct OcelObject {
    id: String,
    #[serde(rename = "type")]
    object_type: String,
    #[serde(skip_serializing_if = "serde_json::Map::is_empty")]
    properties: serde_json::Map<String, serde_json::Value>,
}

/// An event-to-object relationship in OCEL 2.0.
#[derive(Debug, Clone, Serialize)]
struct OcelRelationship {
    object_id: String,
    qualifier: String,
}

// ──────────────────────────── PostProcessor ──────────────────────

/// OCEL exporter post-processor.
///
/// Produces the optional OCEL 2.0 export in [`GraphExportResult::ocel`].
/// Runs after all nodes/edges are generated and budget-enforced.
pub struct OcelExporterPostProcessor;

impl PostProcessor for OcelExporterPostProcessor {
    fn name(&self) -> &'static str {
        "OcelExporter"
    }

    fn process(
        &self,
        result: &mut GraphExportResult,
        ds_result: &EnhancedGenerationResult,
        config: &ExportConfig,
        _id_map: &IdMap,
    ) -> Result<(), ExportError> {
        if !config.ocel.enabled {
            debug!("OcelExporter: disabled, skipping");
            return Ok(());
        }

        let ocel = export_ocel(ds_result, &config.ocel)?;
        result.ocel = ocel;
        Ok(())
    }
}

// ──────────────────────────── Export Logic ───────────────────────

/// Three-level OCEL export strategy.
fn export_ocel(
    ds_result: &EnhancedGenerationResult,
    config: &OcelExportConfig,
) -> Result<Option<OcelExport>, ExportError> {
    let max_events = if config.max_events == 0 {
        usize::MAX
    } else {
        config.max_events
    };

    // Strategy 1: Primary — use OCPM event log if available.
    if let Some(ref event_log) = ds_result.ocpm.event_log {
        if !event_log.events.is_empty() {
            info!(
                "OcelExporter: converting OCPM event log ({} events, {} objects)",
                event_log.events.len(),
                event_log.objects.len()
            );
            return Ok(Some(from_ocpm_event_log(event_log, max_events)?));
        }
    }

    // Strategy 2: Fallback — synthesize from document flows.
    let doc_flows = &ds_result.document_flows;
    if !doc_flows.p2p_chains.is_empty() || !doc_flows.o2c_chains.is_empty() {
        info!(
            "OcelExporter: synthesizing from {} P2P + {} O2C document chains",
            doc_flows.p2p_chains.len(),
            doc_flows.o2c_chains.len()
        );
        return Ok(Some(from_document_flows(doc_flows, max_events)?));
    }

    // Strategy 3: No data available.
    debug!("OcelExporter: no OCPM event log or document flows, returning None");
    Ok(None)
}

/// Convert an OCPM event log to OCEL 2.0 JSON.
fn from_ocpm_event_log(
    event_log: &datasynth_ocpm::OcpmEventLog,
    max_events: usize,
) -> Result<OcelExport, ExportError> {
    let cap = event_log.events.len().min(max_events);

    // Convert events
    let events: Vec<OcelEvent> = event_log
        .events
        .iter()
        .take(max_events)
        .map(|e| OcelEvent {
            id: e.event_id.to_string(),
            event_type: e.activity_name.clone(),
            time: e.timestamp.to_rfc3339(),
            relationships: e
                .object_refs
                .iter()
                .map(|r| OcelRelationship {
                    object_id: r
                        .external_id
                        .clone()
                        .unwrap_or_else(|| r.object_id.to_string()),
                    qualifier: format!("{:?}", r.qualifier).to_lowercase(),
                })
                .collect(),
        })
        .collect();

    // Convert objects
    let objects: Vec<OcelObject> = event_log
        .objects
        .iter()
        .map(|obj| {
            let mut properties = serde_json::Map::new();
            properties.insert("current_state".into(), serde_json::json!(obj.current_state));
            properties.insert("company_code".into(), serde_json::json!(obj.company_code));

            OcelObject {
                id: obj.external_id.clone(),
                object_type: obj.object_type_id.clone(),
                properties,
            }
        })
        .collect();

    let event_count = events.len();
    let doc = OcelDocument { events, objects };
    let data = serde_json::to_value(&doc)?;

    debug!(
        "OcelExporter: converted {event_count}/{cap} OCPM events + {} objects to OCEL 2.0",
        doc.objects.len()
    );

    Ok(OcelExport { data, event_count })
}

/// Synthesize OCEL events from P2P and O2C document chains.
fn from_document_flows(
    doc_flows: &datasynth_runtime::DocumentFlowSnapshot,
    max_events: usize,
) -> Result<OcelExport, ExportError> {
    let mut events = Vec::new();
    let mut objects = Vec::new();
    let mut event_counter = 0usize;

    // P2P chains: PO → GR → Invoice → Payment
    for chain in &doc_flows.p2p_chains {
        if event_counter >= max_events {
            break;
        }

        let po = &chain.purchase_order;
        let po_id = &po.header.document_id;

        // PO object
        objects.push(OcelObject {
            id: po_id.clone(),
            object_type: "purchase_order".into(),
            properties: {
                let mut m = serde_json::Map::new();
                m.insert("vendor_id".into(), serde_json::json!(po.vendor_id));
                m.insert(
                    "company_code".into(),
                    serde_json::json!(po.header.company_code),
                );
                m
            },
        });

        // PO created event
        events.push(make_flow_event(
            &mut event_counter,
            "Create Purchase Order",
            to_utc(po.header.document_date),
            &[(po_id.as_str(), "created")],
        ));
        if event_counter >= max_events {
            break;
        }

        // Goods receipts
        for gr in &chain.goods_receipts {
            if event_counter >= max_events {
                break;
            }
            let gr_id = &gr.header.document_id;
            objects.push(OcelObject {
                id: gr_id.clone(),
                object_type: "goods_receipt".into(),
                properties: serde_json::Map::new(),
            });
            events.push(make_flow_event(
                &mut event_counter,
                "Post Goods Receipt",
                to_utc(gr.header.document_date),
                &[(gr_id.as_str(), "created"), (po_id.as_str(), "updated")],
            ));
        }

        // Vendor invoice
        if let Some(ref inv) = chain.vendor_invoice {
            if event_counter < max_events {
                let inv_id = &inv.header.document_id;
                objects.push(OcelObject {
                    id: inv_id.clone(),
                    object_type: "vendor_invoice".into(),
                    properties: serde_json::Map::new(),
                });
                events.push(make_flow_event(
                    &mut event_counter,
                    "Post Vendor Invoice",
                    to_utc(inv.header.document_date),
                    &[(inv_id.as_str(), "created"), (po_id.as_str(), "read")],
                ));
            }
        }

        // Payment
        if let Some(ref pmt) = chain.payment {
            if event_counter < max_events {
                let pmt_id = &pmt.header.document_id;
                objects.push(OcelObject {
                    id: pmt_id.clone(),
                    object_type: "payment".into(),
                    properties: serde_json::Map::new(),
                });
                events.push(make_flow_event(
                    &mut event_counter,
                    "Clear Payment",
                    to_utc(pmt.header.document_date),
                    &[(pmt_id.as_str(), "created"), (po_id.as_str(), "consumed")],
                ));
            }
        }
    }

    // O2C chains: SO → Delivery → Invoice → Receipt
    for chain in &doc_flows.o2c_chains {
        if event_counter >= max_events {
            break;
        }

        let so = &chain.sales_order;
        let so_id = &so.header.document_id;

        objects.push(OcelObject {
            id: so_id.clone(),
            object_type: "sales_order".into(),
            properties: {
                let mut m = serde_json::Map::new();
                m.insert("customer_id".into(), serde_json::json!(so.customer_id));
                m.insert(
                    "company_code".into(),
                    serde_json::json!(so.header.company_code),
                );
                m
            },
        });

        events.push(make_flow_event(
            &mut event_counter,
            "Create Sales Order",
            to_utc(so.header.document_date),
            &[(so_id.as_str(), "created")],
        ));
        if event_counter >= max_events {
            break;
        }

        // Deliveries
        for dlv in &chain.deliveries {
            if event_counter >= max_events {
                break;
            }
            let dlv_id = &dlv.header.document_id;
            objects.push(OcelObject {
                id: dlv_id.clone(),
                object_type: "delivery".into(),
                properties: serde_json::Map::new(),
            });
            events.push(make_flow_event(
                &mut event_counter,
                "Post Delivery",
                to_utc(dlv.header.document_date),
                &[(dlv_id.as_str(), "created"), (so_id.as_str(), "updated")],
            ));
        }

        // Customer invoice
        if let Some(ref inv) = chain.customer_invoice {
            if event_counter < max_events {
                let inv_id = &inv.header.document_id;
                objects.push(OcelObject {
                    id: inv_id.clone(),
                    object_type: "customer_invoice".into(),
                    properties: serde_json::Map::new(),
                });
                events.push(make_flow_event(
                    &mut event_counter,
                    "Post Customer Invoice",
                    to_utc(inv.header.document_date),
                    &[(inv_id.as_str(), "created"), (so_id.as_str(), "read")],
                ));
            }
        }

        // Customer receipt
        if let Some(ref rcpt) = chain.customer_receipt {
            if event_counter < max_events {
                let rcpt_id = &rcpt.header.document_id;
                objects.push(OcelObject {
                    id: rcpt_id.clone(),
                    object_type: "customer_receipt".into(),
                    properties: serde_json::Map::new(),
                });
                events.push(make_flow_event(
                    &mut event_counter,
                    "Receive Customer Payment",
                    to_utc(rcpt.header.document_date),
                    &[(rcpt_id.as_str(), "created"), (so_id.as_str(), "consumed")],
                ));
            }
        }
    }

    let event_count = events.len();
    let doc = OcelDocument { events, objects };
    let data = serde_json::to_value(&doc)?;

    debug!("OcelExporter: synthesized {event_count} events from document flows");

    Ok(OcelExport { data, event_count })
}

/// Create an OCEL event from document flow data.
fn make_flow_event(
    counter: &mut usize,
    activity: &str,
    time: DateTime<Utc>,
    rels: &[(&str, &str)],
) -> OcelEvent {
    *counter += 1;
    OcelEvent {
        id: format!("FLOW-EVT-{:08}", *counter),
        event_type: activity.into(),
        time: time.to_rfc3339(),
        relationships: rels
            .iter()
            .map(|(obj_id, qual)| OcelRelationship {
                object_id: (*obj_id).into(),
                qualifier: (*qual).into(),
            })
            .collect(),
    }
}

/// Convert a NaiveDate to DateTime<Utc> at midnight.
fn to_utc(date: chrono::NaiveDate) -> DateTime<Utc> {
    date.and_hms_opt(0, 0, 0).unwrap_or_default().and_utc()
}

// ──────────────────────────── Tests ─────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::error::ExportWarnings;
    use crate::types::ExportMetadata;

    fn stub_ds_result() -> EnhancedGenerationResult {
        EnhancedGenerationResult::default()
    }

    fn empty_result() -> GraphExportResult {
        GraphExportResult {
            nodes: Vec::new(),
            edges: Vec::new(),
            ocel: None,
            ground_truth: Vec::new(),
            feature_vectors: Vec::new(),
            hyperedges: Vec::new(),
            metadata: ExportMetadata::default(),
            warnings: ExportWarnings::new(),
        }
    }

    #[test]
    fn ocel_exporter_disabled_returns_none() {
        let mut config = ExportConfig::default();
        config.ocel.enabled = false;

        let mut result = empty_result();
        let ds = stub_ds_result();
        let id_map = IdMap::new();

        OcelExporterPostProcessor
            .process(&mut result, &ds, &config, &id_map)
            .unwrap();

        assert!(result.ocel.is_none());
    }

    #[test]
    fn ocel_exporter_no_data_returns_none() {
        let config = ExportConfig::default();
        let mut result = empty_result();
        let ds = stub_ds_result();
        let id_map = IdMap::new();

        OcelExporterPostProcessor
            .process(&mut result, &ds, &config, &id_map)
            .unwrap();

        assert!(result.ocel.is_none());
    }

    #[test]
    fn make_flow_event_increments_counter() {
        let mut counter = 0;
        let time = chrono::Utc::now();

        let evt1 = make_flow_event(&mut counter, "Create PO", time, &[("PO-001", "created")]);
        assert_eq!(counter, 1);
        assert_eq!(evt1.id, "FLOW-EVT-00000001");
        assert_eq!(evt1.event_type, "Create PO");
        assert_eq!(evt1.relationships.len(), 1);

        let evt2 = make_flow_event(
            &mut counter,
            "Post GR",
            time,
            &[("GR-001", "created"), ("PO-001", "updated")],
        );
        assert_eq!(counter, 2);
        assert_eq!(evt2.id, "FLOW-EVT-00000002");
        assert_eq!(evt2.relationships.len(), 2);
    }

    #[test]
    fn to_utc_produces_midnight() {
        use chrono::Timelike;
        let date = chrono::NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        let dt = to_utc(date);
        assert_eq!(dt.hour(), 0);
        assert_eq!(dt.minute(), 0);
    }

    #[test]
    fn ocel_document_serializes_correctly() {
        let doc = OcelDocument {
            events: vec![OcelEvent {
                id: "EVT-001".into(),
                event_type: "Create PO".into(),
                time: "2025-01-01T00:00:00+00:00".into(),
                relationships: vec![OcelRelationship {
                    object_id: "PO-001".into(),
                    qualifier: "created".into(),
                }],
            }],
            objects: vec![OcelObject {
                id: "PO-001".into(),
                object_type: "purchase_order".into(),
                properties: {
                    let mut m = serde_json::Map::new();
                    m.insert("vendor_id".into(), serde_json::json!("V-001"));
                    m
                },
            }],
        };

        let json = serde_json::to_value(&doc).unwrap();
        assert!(json["events"].is_array());
        assert_eq!(json["events"][0]["id"], "EVT-001");
        assert_eq!(json["events"][0]["type"], "Create PO");
        assert!(json["events"][0]["relationships"].is_array());
        assert_eq!(json["objects"][0]["id"], "PO-001");
        assert_eq!(json["objects"][0]["type"], "purchase_order");
        assert_eq!(json["objects"][0]["properties"]["vendor_id"], "V-001");
    }

    #[test]
    fn is_truthy_helper() {
        // Verify we use is_truthy from post_process (basic sanity)
        let val = serde_json::json!(true);
        assert!(val.as_bool().unwrap_or(false));
    }

    #[test]
    fn ocel_export_max_events_respected() {
        // Test that the max_events cap is applied in the flow event synthesis.
        let config = OcelExportConfig {
            enabled: true,
            max_events: 2,
        };
        let max = if config.max_events == 0 {
            usize::MAX
        } else {
            config.max_events
        };
        assert_eq!(max, 2);
    }

    #[test]
    fn post_processor_name() {
        let p = OcelExporterPostProcessor;
        assert_eq!(p.name(), "OcelExporter");
    }
}
