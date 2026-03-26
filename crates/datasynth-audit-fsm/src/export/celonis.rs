//! Celonis IBC-format export for audit events.
//!
//! Produces a CSV with Celonis-specific column naming (`_CASE_KEY`, `_ACTIVITY`,
//! `_EVENTTIME`, etc.) and a metadata JSON sidecar for direct Celonis import.

use crate::event::AuditEvent;
use std::io::Write;
use std::path::Path;

/// Celonis IBC CSV header using standard Celonis column naming.
const CELONIS_HEADER: &str =
    "_CASE_KEY,_ACTIVITY,_EVENTTIME,_RESOURCE,PROCEDURE_ID,PHASE_ID,FROM_STATE,TO_STATE,IS_ANOMALY,ANOMALY_TYPE";

/// Escape a field value for CSV output.
fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        let escaped = value.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        value.to_string()
    }
}

fn event_to_celonis_row(event: &AuditEvent, case_key: &str) -> String {
    let case_key = csv_escape(case_key);
    let activity = csv_escape(&event.command);
    let eventtime = event.timestamp.format("%Y-%m-%dT%H:%M:%S").to_string();
    let resource = csv_escape(&event.actor_id);
    let procedure_id = csv_escape(&event.procedure_id);
    let phase_id = csv_escape(&event.phase_id);
    let from_state = event
        .from_state
        .as_deref()
        .map(csv_escape)
        .unwrap_or_default();
    let to_state = event
        .to_state
        .as_deref()
        .map(csv_escape)
        .unwrap_or_default();
    let is_anomaly = event.is_anomaly;
    let anomaly_type = event
        .anomaly_type
        .as_ref()
        .map(|a| a.to_string())
        .unwrap_or_default();

    format!(
        "{case_key},{activity},{eventtime},{resource},{procedure_id},{phase_id},{from_state},{to_state},{is_anomaly},{anomaly_type}"
    )
}

/// Celonis metadata sidecar describing the event log schema.
fn celonis_metadata(engagement_id: &str, event_count: usize) -> String {
    serde_json::json!({
        "format": "celonis_ibc",
        "version": "1.0",
        "engagement_id": engagement_id,
        "event_count": event_count,
        "columns": {
            "_CASE_KEY": { "type": "string", "role": "case_id" },
            "_ACTIVITY": { "type": "string", "role": "activity" },
            "_EVENTTIME": { "type": "datetime", "role": "timestamp", "format": "ISO8601" },
            "_RESOURCE": { "type": "string", "role": "resource" },
            "PROCEDURE_ID": { "type": "string" },
            "PHASE_ID": { "type": "string" },
            "FROM_STATE": { "type": "string" },
            "TO_STATE": { "type": "string" },
            "IS_ANOMALY": { "type": "boolean" },
            "ANOMALY_TYPE": { "type": "string" }
        }
    })
    .to_string()
}

/// Export audit events to a Celonis IBC CSV string.
pub fn export_events_to_celonis_string(events: &[AuditEvent], engagement_id: &str) -> String {
    let mut lines = Vec::with_capacity(events.len() + 1);
    lines.push(CELONIS_HEADER.to_string());
    for event in events {
        lines.push(event_to_celonis_row(event, engagement_id));
    }
    lines.join("\n")
}

/// Export audit events to Celonis IBC format: a CSV file and a metadata JSON sidecar.
///
/// Writes two files:
/// - `{dir}/celonis_events.csv` — the event log in Celonis column format
/// - `{dir}/celonis_metadata.json` — schema/metadata sidecar
pub fn export_events_to_celonis(
    events: &[AuditEvent],
    dir: &Path,
    engagement_id: &str,
) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;

    // CSV
    let csv = export_events_to_celonis_string(events, engagement_id);
    let mut csv_file = std::fs::File::create(dir.join("celonis_events.csv"))?;
    csv_file.write_all(csv.as_bytes())?;

    // Metadata sidecar
    let meta = celonis_metadata(engagement_id, events.len());
    let mut meta_file = std::fs::File::create(dir.join("celonis_metadata.json"))?;
    meta_file.write_all(meta.as_bytes())?;

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
    fn test_celonis_csv_has_ibc_header() {
        let events = sample_events();
        let csv = export_events_to_celonis_string(&events, "ENG-001");
        let first_line = csv.lines().next().unwrap();
        assert!(first_line.contains("_CASE_KEY"));
        assert!(first_line.contains("_ACTIVITY"));
        assert!(first_line.contains("_EVENTTIME"));
    }

    #[test]
    fn test_celonis_csv_row_count() {
        let events = sample_events();
        let csv = export_events_to_celonis_string(&events, "ENG-001");
        assert_eq!(csv.lines().count(), events.len() + 1);
    }

    #[test]
    fn test_celonis_export_to_dir() {
        let events = sample_events();
        let dir = tempfile::tempdir().unwrap();
        export_events_to_celonis(&events, dir.path(), "ENG-001").unwrap();

        assert!(dir.path().join("celonis_events.csv").exists());
        assert!(dir.path().join("celonis_metadata.json").exists());

        let meta = std::fs::read_to_string(dir.path().join("celonis_metadata.json")).unwrap();
        assert!(meta.contains("celonis_ibc"));
        assert!(meta.contains("_CASE_KEY"));
    }
}
