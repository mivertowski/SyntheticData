//! XES 2.0 export functionality for process mining tools.
//!
//! XES (eXtensible Event Stream) is the IEEE standard format for event logs,
//! widely supported by tools like ProM, Celonis, Disco, and pm4py.
//!
//! This module exports OCPM event logs to XES 2.0 XML format.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::models::{EventLifecycle, OcpmEvent, OcpmEventLog};

/// XES 2.0 exporter for process mining tools.
///
/// Exports OCPM event logs to the XES XML format, which is the standard
/// format for process mining applications like ProM, Celonis, and Disco.
///
/// # Features
///
/// - XES 2.0 compliant output
/// - Configurable lifecycle inclusion
/// - Resource attribute support
/// - Case grouping by process instance
///
/// # Example
///
/// ```ignore
/// use datasynth_ocpm::export::XesExporter;
///
/// let exporter = XesExporter::new()
///     .with_lifecycle(true)
///     .with_resources(true);
///
/// exporter.export_to_file(&event_log, "output.xes")?;
/// ```
#[derive(Debug, Clone)]
pub struct XesExporter {
    /// Include lifecycle transition attributes
    pub include_lifecycle: bool,
    /// Include resource attributes
    pub include_resources: bool,
    /// Include custom attributes from events
    pub include_custom_attributes: bool,
    /// Pretty print XML output
    pub pretty_print: bool,
    /// Indent string for pretty printing
    indent: String,
}

impl Default for XesExporter {
    fn default() -> Self {
        Self {
            include_lifecycle: true,
            include_resources: true,
            include_custom_attributes: true,
            pretty_print: true,
            indent: "  ".to_string(),
        }
    }
}

impl XesExporter {
    /// Create a new XES exporter with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether to include lifecycle transition attributes.
    pub fn with_lifecycle(mut self, include: bool) -> Self {
        self.include_lifecycle = include;
        self
    }

    /// Set whether to include resource attributes.
    pub fn with_resources(mut self, include: bool) -> Self {
        self.include_resources = include;
        self
    }

    /// Set whether to include custom attributes.
    pub fn with_custom_attributes(mut self, include: bool) -> Self {
        self.include_custom_attributes = include;
        self
    }

    /// Set whether to pretty print the output.
    pub fn with_pretty_print(mut self, pretty: bool) -> Self {
        self.pretty_print = pretty;
        self
    }

    /// Export an OCPM event log to an XES file.
    pub fn export_to_file<P: AsRef<Path>>(
        &self,
        log: &OcpmEventLog,
        path: P,
    ) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        self.write_xes(log, &mut writer)?;
        writer.flush()?;
        Ok(())
    }

    /// Export an OCPM event log to a string.
    pub fn export_to_string(&self, log: &OcpmEventLog) -> std::io::Result<String> {
        let mut buffer = Vec::new();
        self.write_xes(log, &mut buffer)?;
        String::from_utf8(buffer)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Write XES content to a writer.
    fn write_xes<W: Write>(&self, log: &OcpmEventLog, writer: &mut W) -> std::io::Result<()> {
        // XML declaration
        writeln!(writer, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;

        // XES log element with namespaces
        self.write_log_start(writer)?;

        // Extensions
        self.write_extensions(writer)?;

        // Global trace attributes
        self.write_global_trace_attrs(writer)?;

        // Global event attributes
        self.write_global_event_attrs(writer)?;

        // Classifiers
        self.write_classifiers(writer)?;

        // Traces (grouped by case_id)
        self.write_traces(log, writer)?;

        // Close log element
        writeln!(writer, "</log>")?;

        Ok(())
    }

    fn write_log_start<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writeln!(
            writer,
            r#"<log xes.version="2.0" xes.features="nested-attributes" xmlns="http://www.xes-standard.org/">"#
        )
    }

    fn write_extensions<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let i = if self.pretty_print { &self.indent } else { "" };

        writeln!(
            writer,
            r#"{i}<extension name="Concept" prefix="concept" uri="http://www.xes-standard.org/concept.xesext"/>"#
        )?;
        writeln!(
            writer,
            r#"{i}<extension name="Time" prefix="time" uri="http://www.xes-standard.org/time.xesext"/>"#
        )?;

        if self.include_lifecycle {
            writeln!(
                writer,
                r#"{i}<extension name="Lifecycle" prefix="lifecycle" uri="http://www.xes-standard.org/lifecycle.xesext"/>"#
            )?;
        }

        if self.include_resources {
            writeln!(
                writer,
                r#"{i}<extension name="Organizational" prefix="org" uri="http://www.xes-standard.org/org.xesext"/>"#
            )?;
        }

        Ok(())
    }

    fn write_global_trace_attrs<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let i = if self.pretty_print { &self.indent } else { "" };

        writeln!(writer, r#"{i}<global scope="trace">"#)?;
        writeln!(
            writer,
            r#"{i}{i}<string key="concept:name" value="UNKNOWN"/>"#
        )?;
        writeln!(writer, r#"{i}</global>"#)?;

        Ok(())
    }

    fn write_global_event_attrs<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let i = if self.pretty_print { &self.indent } else { "" };

        writeln!(writer, r#"{i}<global scope="event">"#)?;
        writeln!(
            writer,
            r#"{i}{i}<string key="concept:name" value="UNKNOWN"/>"#
        )?;
        writeln!(
            writer,
            r#"{i}{i}<date key="time:timestamp" value="1970-01-01T00:00:00.000+00:00"/>"#
        )?;

        if self.include_lifecycle {
            writeln!(
                writer,
                r#"{i}{i}<string key="lifecycle:transition" value="complete"/>"#
            )?;
        }

        writeln!(writer, r#"{i}</global>"#)?;

        Ok(())
    }

    fn write_classifiers<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let i = if self.pretty_print { &self.indent } else { "" };

        writeln!(
            writer,
            r#"{i}<classifier name="Activity" keys="concept:name"/>"#
        )?;

        if self.include_resources {
            writeln!(
                writer,
                r#"{i}<classifier name="Resource" keys="org:resource"/>"#
            )?;
        }

        Ok(())
    }

    fn write_traces<W: Write>(&self, log: &OcpmEventLog, writer: &mut W) -> std::io::Result<()> {
        use std::collections::HashMap;

        let i = if self.pretty_print { &self.indent } else { "" };

        // Group events by case_id
        let mut cases: HashMap<String, Vec<&OcpmEvent>> = HashMap::new();

        for event in &log.events {
            let case_key = event
                .case_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "UNKNOWN".to_string());

            cases.entry(case_key).or_default().push(event);
        }

        // Sort events within each case by timestamp
        for events in cases.values_mut() {
            events.sort_by_key(|e| e.timestamp);
        }

        // Write traces
        for (case_id, events) in &cases {
            writeln!(writer, r#"{i}<trace>"#)?;

            // Trace name (case_id)
            writeln!(
                writer,
                r#"{i}{i}<string key="concept:name" value="{}"/>"#,
                escape_xml(case_id)
            )?;

            // Write events
            for event in events {
                self.write_event(event, writer)?;
            }

            writeln!(writer, r#"{i}</trace>"#)?;
        }

        Ok(())
    }

    fn write_event<W: Write>(&self, event: &OcpmEvent, writer: &mut W) -> std::io::Result<()> {
        let i = if self.pretty_print { &self.indent } else { "" };
        let ii = format!("{i}{i}");
        let iii = format!("{ii}{i}");

        writeln!(writer, r#"{ii}<event>"#)?;

        // Activity name
        writeln!(
            writer,
            r#"{iii}<string key="concept:name" value="{}"/>"#,
            escape_xml(&event.activity_name)
        )?;

        // Timestamp
        writeln!(
            writer,
            r#"{iii}<date key="time:timestamp" value="{}"/>"#,
            event.timestamp.format("%Y-%m-%dT%H:%M:%S%.3f%:z")
        )?;

        // Lifecycle transition
        if self.include_lifecycle {
            let lifecycle = match event.lifecycle {
                EventLifecycle::Start => "start",
                EventLifecycle::Complete => "complete",
                EventLifecycle::Abort => "abort",
                EventLifecycle::Suspend => "suspend",
                EventLifecycle::Resume => "resume",
                EventLifecycle::Atomic => "complete", // Atomic maps to complete in XES
            };
            writeln!(
                writer,
                r#"{iii}<string key="lifecycle:transition" value="{lifecycle}"/>"#
            )?;
        }

        // Resource
        if self.include_resources && !event.resource_id.is_empty() {
            writeln!(
                writer,
                r#"{iii}<string key="org:resource" value="{}"/>"#,
                escape_xml(&event.resource_id)
            )?;
        }

        // Company code as organizational group
        if self.include_resources && !event.company_code.is_empty() {
            writeln!(
                writer,
                r#"{iii}<string key="org:group" value="{}"/>"#,
                escape_xml(&event.company_code)
            )?;
        }

        // Activity ID
        writeln!(
            writer,
            r#"{iii}<string key="activity:id" value="{}"/>"#,
            escape_xml(&event.activity_id)
        )?;

        // Document reference
        if let Some(ref doc_ref) = event.document_ref {
            writeln!(
                writer,
                r#"{iii}<string key="document:ref" value="{}"/>"#,
                escape_xml(doc_ref)
            )?;
        }

        // Anomaly flag
        if event.is_anomaly {
            writeln!(writer, r#"{iii}<boolean key="is:anomaly" value="true"/>"#)?;
            if let Some(ref anomaly_type) = event.anomaly_type {
                writeln!(
                    writer,
                    r#"{iii}<string key="anomaly:type" value="{}"/>"#,
                    escape_xml(anomaly_type)
                )?;
            }
        }

        // Custom attributes
        if self.include_custom_attributes {
            for (key, value) in &event.attributes {
                let escaped_key = escape_xml(key);
                match value {
                    crate::models::ObjectAttributeValue::String(s) => {
                        writeln!(
                            writer,
                            r#"{iii}<string key="{escaped_key}" value="{}"/>"#,
                            escape_xml(s)
                        )?;
                    }
                    crate::models::ObjectAttributeValue::Integer(i) => {
                        writeln!(writer, r#"{iii}<int key="{escaped_key}" value="{i}"/>"#)?;
                    }
                    crate::models::ObjectAttributeValue::Decimal(d) => {
                        writeln!(
                            writer,
                            r#"{iii}<float key="{escaped_key}" value="{}"/>"#,
                            d.to_string().parse::<f64>().unwrap_or(0.0)
                        )?;
                    }
                    crate::models::ObjectAttributeValue::Boolean(b) => {
                        writeln!(writer, r#"{iii}<boolean key="{escaped_key}" value="{b}"/>"#)?;
                    }
                    crate::models::ObjectAttributeValue::Date(date) => {
                        writeln!(
                            writer,
                            r#"{iii}<date key="{escaped_key}" value="{}T00:00:00.000+00:00"/>"#,
                            date
                        )?;
                    }
                    crate::models::ObjectAttributeValue::DateTime(dt) => {
                        writeln!(
                            writer,
                            r#"{iii}<date key="{escaped_key}" value="{}"/>"#,
                            dt.format("%Y-%m-%dT%H:%M:%S%.3f%:z")
                        )?;
                    }
                    crate::models::ObjectAttributeValue::Reference(id) => {
                        writeln!(writer, r#"{iii}<id key="{escaped_key}" value="{}"/>"#, id)?;
                    }
                    crate::models::ObjectAttributeValue::Null => {
                        // Skip null values in XES
                    }
                }
            }
        }

        writeln!(writer, r#"{ii}</event>"#)?;

        Ok(())
    }
}

/// Escape special characters for XML.
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_log() -> OcpmEventLog {
        let mut log = OcpmEventLog::new().with_standard_types();

        let case_id = Uuid::new_v4();

        let event1 = OcpmEvent::new(
            "create_po",
            "Create Purchase Order",
            Utc::now(),
            "user001",
            "1000",
        )
        .with_case(case_id);

        let event2 = OcpmEvent::new(
            "approve_po",
            "Approve Purchase Order",
            Utc::now(),
            "user002",
            "1000",
        )
        .with_case(case_id);

        log.events.push(event1);
        log.events.push(event2);

        log
    }

    #[test]
    fn test_xes_exporter_creation() {
        let exporter = XesExporter::new()
            .with_lifecycle(true)
            .with_resources(true)
            .with_pretty_print(false);

        assert!(exporter.include_lifecycle);
        assert!(exporter.include_resources);
        assert!(!exporter.pretty_print);
    }

    #[test]
    fn test_xes_export_to_string() {
        let log = create_test_log();
        let exporter = XesExporter::new();

        let xes = exporter.export_to_string(&log).unwrap();

        // Verify basic XES structure
        assert!(xes.contains(r#"<?xml version="1.0""#));
        assert!(xes.contains(r#"xes.version="2.0""#));
        assert!(xes.contains("<log"));
        assert!(xes.contains("</log>"));
        assert!(xes.contains("<trace>"));
        assert!(xes.contains("<event>"));
        assert!(xes.contains("concept:name"));
        assert!(xes.contains("time:timestamp"));
    }

    #[test]
    fn test_xes_contains_lifecycle() {
        let log = create_test_log();

        // With lifecycle
        let exporter = XesExporter::new().with_lifecycle(true);
        let xes = exporter.export_to_string(&log).unwrap();
        assert!(xes.contains("lifecycle:transition"));

        // Without lifecycle
        let exporter = XesExporter::new().with_lifecycle(false);
        let xes = exporter.export_to_string(&log).unwrap();
        assert!(!xes.contains("lifecycle:transition"));
    }

    #[test]
    fn test_xes_contains_resources() {
        let log = create_test_log();

        // With resources
        let exporter = XesExporter::new().with_resources(true);
        let xes = exporter.export_to_string(&log).unwrap();
        assert!(xes.contains("org:resource"));

        // Without resources
        let exporter = XesExporter::new().with_resources(false);
        let xes = exporter.export_to_string(&log).unwrap();
        assert!(!xes.contains("org:resource"));
    }

    #[test]
    fn test_xml_escaping() {
        assert_eq!(escape_xml("hello"), "hello");
        assert_eq!(escape_xml("<test>"), "&lt;test&gt;");
        assert_eq!(escape_xml("a & b"), "a &amp; b");
        assert_eq!(escape_xml(r#"say "hi""#), "say &quot;hi&quot;");
    }
}
