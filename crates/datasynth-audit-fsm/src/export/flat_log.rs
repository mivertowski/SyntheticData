//! Flat JSON event trail exporter.

use crate::event::AuditEvent;
use std::io::Write;
use std::path::Path;

/// Export audit events to a JSON string (pretty-printed).
pub fn export_events_to_json(events: &[AuditEvent]) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(events)
}

/// Export audit events to a JSON file.
pub fn export_events_to_file(events: &[AuditEvent], path: &Path) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(events)
        .map_err(std::io::Error::other)?;
    let mut file = std::fs::File::create(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_to_json_string() {
        use rand::SeedableRng;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(42);
        let events = vec![
            crate::event::AuditEventBuilder::transition()
                .procedure_id("test_proc")
                .phase_id("planning")
                .from_state("not_started")
                .to_state("in_progress")
                .command("start")
                .event_type("TestStarted")
                .actor_id("manager")
                .timestamp(
                    chrono::NaiveDate::from_ymd_opt(2025, 3, 1)
                        .unwrap()
                        .and_hms_opt(9, 0, 0)
                        .unwrap(),
                )
                .build_with_rng(&mut rng),
        ];
        let json = export_events_to_json(&events).unwrap();
        assert!(json.contains("TestStarted"));
        assert!(json.contains("test_proc"));
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 1);
    }
}
