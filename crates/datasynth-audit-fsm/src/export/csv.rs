//! CSV export for audit events, targeting Disco/Celonis/Minit process mining tools.

use crate::event::AuditEvent;
use std::io::Write;
use std::path::Path;

const HEADER: &str =
    "case_id,activity,timestamp,resource,procedure_id,phase_id,from_state,to_state,is_anomaly,anomaly_type,evidence_refs,standards_refs";

/// Escape a field value for CSV output.
///
/// If the value contains a comma, double-quote, or newline it is wrapped in
/// double-quotes with any internal double-quotes doubled.
fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        let escaped = value.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        value.to_string()
    }
}

fn event_to_csv_row(event: &AuditEvent, case_id: &str) -> String {
    let case_id = csv_escape(case_id);
    let activity = csv_escape(&event.command);
    let timestamp = event.timestamp.format("%Y-%m-%dT%H:%M:%S").to_string();
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
    let evidence_refs = csv_escape(&event.evidence_refs.join(";"));
    let standards_refs = csv_escape(&event.standards_refs.join(";"));

    format!(
        "{case_id},{activity},{timestamp},{resource},{procedure_id},{phase_id},{from_state},{to_state},{is_anomaly},{anomaly_type},{evidence_refs},{standards_refs}"
    )
}

/// Derive a case ID from the first event's procedure_id, or fall back to a
/// default when the event list is empty.
fn derive_case_id(events: &[AuditEvent]) -> String {
    events
        .first()
        .map(|e| {
            // Use the phase_id of the first event as a stable case identifier.
            if e.phase_id.is_empty() {
                "engagement_1".to_string()
            } else {
                format!("engagement_{}", e.phase_id)
            }
        })
        .unwrap_or_else(|| "engagement_1".to_string())
}

/// Export audit events to a CSV string suitable for Disco/Celonis/Minit.
///
/// The `engagement_id` parameter is used as the `case_id` column value.
/// If `None`, a case ID is derived from the first event.
pub fn export_events_to_csv_string_with_id(
    events: &[AuditEvent],
    engagement_id: Option<&str>,
) -> String {
    let case_id = engagement_id
        .map(|s| s.to_string())
        .unwrap_or_else(|| derive_case_id(events));
    let mut lines = Vec::with_capacity(events.len() + 1);
    lines.push(HEADER.to_string());
    for event in events {
        lines.push(event_to_csv_row(event, &case_id));
    }
    lines.join("\n")
}

/// Export audit events to a CSV string suitable for Disco/Celonis/Minit.
///
/// Uses `"engagement_1"` as the default case ID for backward compatibility.
pub fn export_events_to_csv_string(events: &[AuditEvent]) -> String {
    export_events_to_csv_string_with_id(events, Some("engagement_1"))
}

/// Export audit events to a CSV file.
///
/// Uses `"engagement_1"` as the default case ID for backward compatibility.
pub fn export_events_to_csv(events: &[AuditEvent], path: &Path) -> std::io::Result<()> {
    export_events_to_csv_with_id(events, path, Some("engagement_1"))
}

/// Export audit events to a CSV file with an explicit engagement ID.
pub fn export_events_to_csv_with_id(
    events: &[AuditEvent],
    path: &Path,
    engagement_id: Option<&str>,
) -> std::io::Result<()> {
    let csv = export_events_to_csv_string_with_id(events, engagement_id);
    let mut file = std::fs::File::create(path)?;
    file.write_all(csv.as_bytes())?;
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
        let ts1 = NaiveDate::from_ymd_opt(2025, 1, 15)
            .unwrap()
            .and_hms_opt(9, 0, 0)
            .unwrap();
        let ts2 = NaiveDate::from_ymd_opt(2025, 1, 15)
            .unwrap()
            .and_hms_opt(10, 30, 0)
            .unwrap();

        vec![
            AuditEventBuilder::transition()
                .procedure_id("accept_engagement")
                .phase_id("planning")
                .from_state("not_started")
                .to_state("in_progress")
                .command("start_acceptance")
                .event_type("state_transition")
                .actor_id("engagement_partner")
                .evidence_ref("EVD-001")
                .standard_ref("ISA-210")
                .timestamp(ts1)
                .build_with_rng(&mut rng),
            AuditEventBuilder::step()
                .procedure_id("risk_assessment")
                .phase_id("planning")
                .command("assess_risk")
                .event_type("procedure_step")
                .actor_id("audit_manager")
                .standard_ref("ISA-315")
                .standard_ref("ISA-330")
                .timestamp(ts2)
                .build_with_rng(&mut rng),
        ]
    }

    #[test]
    fn test_csv_export_has_header() {
        let events = sample_events();
        let csv = export_events_to_csv_string(&events);
        let first_line = csv.lines().next().unwrap();
        assert_eq!(first_line, HEADER);
        assert!(first_line.contains("case_id"));
        assert!(first_line.contains("activity"));
        assert!(first_line.contains("timestamp"));
        assert!(first_line.contains("evidence_refs"));
    }

    #[test]
    fn test_csv_export_row_count() {
        let events = sample_events();
        let csv = export_events_to_csv_string(&events);
        let line_count = csv.lines().count();
        // header + one row per event
        assert_eq!(line_count, events.len() + 1);
    }

    #[test]
    fn test_csv_export_roundtrip_to_file() {
        let events = sample_events();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("audit_events.csv");

        export_events_to_csv(&events, &path).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines[0], HEADER);
        assert_eq!(lines.len(), events.len() + 1);
        // Verify first data row contains expected values
        assert!(lines[1].contains("engagement_1"));
        assert!(lines[1].contains("start_acceptance"));
        assert!(lines[1].contains("engagement_partner"));
        assert!(lines[1].contains("2025-01-15T09:00:00"));
    }

    #[test]
    fn test_csv_escape_commas() {
        assert_eq!(csv_escape("hello,world"), "\"hello,world\"");
        assert_eq!(csv_escape("simple"), "simple");
        assert_eq!(csv_escape("has\"quote"), "\"has\"\"quote\"");
    }

    #[test]
    fn test_csv_semicolon_separated_refs() {
        let events = sample_events();
        let csv = export_events_to_csv_string(&events);
        let lines: Vec<&str> = csv.lines().collect();
        // Second event has two standards_refs joined by semicolon
        assert!(lines[2].contains("ISA-315;ISA-330"));
    }
}
