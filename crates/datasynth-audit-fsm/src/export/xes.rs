//! XES 2.0 XML export for audit events, targeting ProM/pm4py process mining tools.

use crate::event::AuditEvent;
use std::io::Write;
use std::path::Path;

/// Escape a string for safe inclusion in XML attribute values and text content.
fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn event_to_xes(event: &AuditEvent) -> String {
    let mut lines = Vec::new();
    lines.push("      <event>".to_string());
    lines.push(format!(
        "        <string key=\"concept:name\" value=\"{}\"/>",
        xml_escape(&event.command)
    ));
    lines.push(format!(
        "        <date key=\"time:timestamp\" value=\"{}+00:00\"/>",
        event.timestamp.format("%Y-%m-%dT%H:%M:%S")
    ));
    lines.push(format!(
        "        <string key=\"org:resource\" value=\"{}\"/>",
        xml_escape(&event.actor_id)
    ));
    lines.push("        <string key=\"lifecycle:transition\" value=\"complete\"/>".to_string());
    lines.push(format!(
        "        <string key=\"procedure_id\" value=\"{}\"/>",
        xml_escape(&event.procedure_id)
    ));
    lines.push(format!(
        "        <string key=\"phase_id\" value=\"{}\"/>",
        xml_escape(&event.phase_id)
    ));
    if let Some(ref from) = event.from_state {
        lines.push(format!(
            "        <string key=\"from_state\" value=\"{}\"/>",
            xml_escape(from)
        ));
    }
    if let Some(ref to) = event.to_state {
        lines.push(format!(
            "        <string key=\"to_state\" value=\"{}\"/>",
            xml_escape(to)
        ));
    }
    lines.push(format!(
        "        <boolean key=\"is_anomaly\" value=\"{}\"/>",
        event.is_anomaly
    ));
    if let Some(ref anomaly) = event.anomaly_type {
        lines.push(format!(
            "        <string key=\"anomaly_type\" value=\"{}\"/>",
            xml_escape(&anomaly.to_string())
        ));
    }
    if !event.evidence_refs.is_empty() {
        lines.push(format!(
            "        <string key=\"evidence_refs\" value=\"{}\"/>",
            xml_escape(&event.evidence_refs.join(";"))
        ));
    }
    if !event.standards_refs.is_empty() {
        lines.push(format!(
            "        <string key=\"standards_refs\" value=\"{}\"/>",
            xml_escape(&event.standards_refs.join(";"))
        ));
    }
    lines.push("      </event>".to_string());
    lines.join("\n")
}

/// Export audit events to an XES 2.0 XML string.
pub fn export_events_to_xes_string(events: &[AuditEvent]) -> String {
    let mut parts = vec![
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>".to_string(),
        "<log xes.version=\"2.0\" xes.features=\"\">".to_string(),
        "  <extension name=\"Concept\" prefix=\"concept\" uri=\"http://www.xes-standard.org/concept.xesext\"/>".to_string(),
        "  <extension name=\"Time\" prefix=\"time\" uri=\"http://www.xes-standard.org/time.xesext\"/>".to_string(),
        "  <extension name=\"Organizational\" prefix=\"org\" uri=\"http://www.xes-standard.org/org.xesext\"/>".to_string(),
        "  <extension name=\"Lifecycle\" prefix=\"lifecycle\" uri=\"http://www.xes-standard.org/lifecycle.xesext\"/>".to_string(),
        "  <global scope=\"event\">".to_string(),
        "    <string key=\"concept:name\" value=\"UNKNOWN\"/>".to_string(),
        "    <date key=\"time:timestamp\" value=\"1970-01-01T00:00:00+00:00\"/>".to_string(),
        "  </global>".to_string(),
        "  <classifier name=\"Activity\" keys=\"concept:name\"/>".to_string(),
        "  <trace>".to_string(),
        "    <string key=\"concept:name\" value=\"engagement_1\"/>".to_string(),
    ];

    for event in events {
        parts.push(event_to_xes(event));
    }

    parts.push("  </trace>".to_string());
    parts.push("</log>".to_string());

    parts.join("\n")
}

/// Export audit events to an XES 2.0 XML file.
pub fn export_events_to_xes(events: &[AuditEvent], path: &Path) -> std::io::Result<()> {
    let xes = export_events_to_xes_string(events);
    let mut file = std::fs::File::create(path)?;
    file.write_all(xes.as_bytes())?;
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
                .timestamp(ts2)
                .build_with_rng(&mut rng),
        ]
    }

    #[test]
    fn test_xes_export_valid_xml() {
        let events = sample_events();
        let xes = export_events_to_xes_string(&events);
        assert!(xes.starts_with("<?xml"));
        assert!(xes.contains("<log"));
        assert!(xes.contains("<trace>"));
        assert!(xes.contains("<event>"));
        assert!(xes.contains("</log>"));
        assert!(xes.contains("concept:name"));
        assert!(xes.contains("time:timestamp"));
        assert!(xes.contains("org:resource"));
        assert!(xes.contains("lifecycle:transition"));
    }

    #[test]
    fn test_xes_export_event_count() {
        let events = sample_events();
        let xes = export_events_to_xes_string(&events);
        let event_open_count = xes.matches("<event>").count();
        let event_close_count = xes.matches("</event>").count();
        assert_eq!(event_open_count, events.len());
        assert_eq!(event_close_count, events.len());
    }

    #[test]
    fn test_xes_export_to_file() {
        let events = sample_events();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("audit_events.xes");

        export_events_to_xes(&events, &path).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.starts_with("<?xml"));
        assert!(content.contains("<trace>"));
        assert!(content.contains("engagement_1"));
        assert!(content.contains("start_acceptance"));
        assert!(content.contains("2025-01-15T09:00:00+00:00"));
    }

    #[test]
    fn test_xes_xml_escape() {
        assert_eq!(xml_escape("hello"), "hello");
        assert_eq!(xml_escape("a & b"), "a &amp; b");
        assert_eq!(xml_escape("<tag>"), "&lt;tag&gt;");
        assert_eq!(xml_escape("he said \"hi\""), "he said &quot;hi&quot;");
    }
}
