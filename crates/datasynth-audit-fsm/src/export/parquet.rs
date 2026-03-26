//! Parquet export for audit events (PM4Py compatible).
//!
//! Feature-gated behind `parquet-export`. Writes audit events as an Apache
//! Parquet file with columns matching the CSV export, suitable for direct
//! consumption by PM4Py's `pm4py.read.read_parquet()`.

use std::path::Path;
use std::sync::Arc;

use arrow::array::{BooleanArray, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;

use crate::event::AuditEvent;

/// Build the Arrow schema matching the CSV export columns.
fn audit_event_schema() -> Schema {
    Schema::new(vec![
        Field::new("case_id", DataType::Utf8, false),
        Field::new("activity", DataType::Utf8, false),
        Field::new("timestamp", DataType::Utf8, false),
        Field::new("resource", DataType::Utf8, false),
        Field::new("procedure_id", DataType::Utf8, false),
        Field::new("phase_id", DataType::Utf8, false),
        Field::new("from_state", DataType::Utf8, true),
        Field::new("to_state", DataType::Utf8, true),
        Field::new("is_anomaly", DataType::Boolean, false),
        Field::new("anomaly_type", DataType::Utf8, true),
        Field::new("evidence_refs", DataType::Utf8, true),
        Field::new("standards_refs", DataType::Utf8, true),
    ])
}

/// Export audit events to a Parquet file.
///
/// The `engagement_id` is used as the `case_id` column for all events.
pub fn export_events_to_parquet(
    events: &[AuditEvent],
    path: &Path,
    engagement_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let schema = Arc::new(audit_event_schema());

    let case_ids: Vec<&str> = vec![engagement_id; events.len()];
    let activities: Vec<String> = events.iter().map(|e| e.command.clone()).collect();
    let timestamps: Vec<String> = events
        .iter()
        .map(|e| e.timestamp.format("%Y-%m-%dT%H:%M:%S").to_string())
        .collect();
    let resources: Vec<String> = events.iter().map(|e| e.actor_id.clone()).collect();
    let procedure_ids: Vec<String> = events.iter().map(|e| e.procedure_id.clone()).collect();
    let phase_ids: Vec<String> = events.iter().map(|e| e.phase_id.clone()).collect();
    let from_states: Vec<Option<String>> = events.iter().map(|e| e.from_state.clone()).collect();
    let to_states: Vec<Option<String>> = events.iter().map(|e| e.to_state.clone()).collect();
    let is_anomalies: Vec<bool> = events.iter().map(|e| e.is_anomaly).collect();
    let anomaly_types: Vec<Option<String>> = events
        .iter()
        .map(|e| e.anomaly_type.as_ref().map(|a| a.to_string()))
        .collect();
    let evidence: Vec<Option<String>> = events
        .iter()
        .map(|e| {
            if e.evidence_refs.is_empty() {
                None
            } else {
                Some(e.evidence_refs.join(";"))
            }
        })
        .collect();
    let standards: Vec<Option<String>> = events
        .iter()
        .map(|e| {
            if e.standards_refs.is_empty() {
                None
            } else {
                Some(e.standards_refs.join(";"))
            }
        })
        .collect();

    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(StringArray::from(case_ids)),
            Arc::new(StringArray::from(
                activities.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                timestamps.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                resources.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                procedure_ids.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                phase_ids.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                from_states.iter().map(|s| s.as_deref()).collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                to_states.iter().map(|s| s.as_deref()).collect::<Vec<_>>(),
            )),
            Arc::new(BooleanArray::from(is_anomalies)),
            Arc::new(StringArray::from(
                anomaly_types
                    .iter()
                    .map(|s| s.as_deref())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                evidence.iter().map(|s| s.as_deref()).collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                standards.iter().map(|s| s.as_deref()).collect::<Vec<_>>(),
            )),
        ],
    )?;

    let file = std::fs::File::create(path)?;
    let mut writer = ArrowWriter::try_new(file, schema, None)?;
    writer.write(&batch)?;
    writer.close()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::AuditEventBuilder;
    use chrono::NaiveDate;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn sample_events() -> Vec<AuditEvent> {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let ts = NaiveDate::from_ymd_opt(2025, 3, 15)
            .unwrap()
            .and_hms_opt(9, 0, 0)
            .unwrap();

        vec![AuditEventBuilder::transition()
            .procedure_id("accept_engagement")
            .phase_id("planning")
            .from_state("not_started")
            .to_state("in_progress")
            .command("start_acceptance")
            .event_type("state_transition")
            .actor_id("engagement_partner")
            .timestamp(ts)
            .build_with_rng(&mut rng)]
    }

    #[test]
    fn test_parquet_export_creates_file() {
        let events = sample_events();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("audit_events.parquet");

        export_events_to_parquet(&events, &path, "ENG-001").unwrap();

        assert!(path.exists());
        let metadata = std::fs::metadata(&path).unwrap();
        assert!(metadata.len() > 0);
    }
}
