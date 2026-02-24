//! Export internal controls master data to CSV files.
//!
//! Exports control definitions, mappings, and SoD conflict pairs
//! as separate CSV files for use in BI/analytics systems.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use datasynth_core::error::SynthResult;
use datasynth_core::models::{
    ControlAccountMapping, ControlDocTypeMapping, ControlMappingRegistry, ControlProcessMapping,
    ControlThresholdMapping, InternalControl, SodConflictPair, SodRule,
};

/// Exporter for internal controls master data.
pub struct ControlExporter {
    output_dir: PathBuf,
}

impl ControlExporter {
    /// Create a new control exporter.
    pub fn new(output_dir: impl AsRef<Path>) -> Self {
        Self {
            output_dir: output_dir.as_ref().to_path_buf(),
        }
    }

    /// Export all control master data.
    ///
    /// Creates the following CSV files:
    /// - internal_controls.csv
    /// - control_account_mappings.csv
    /// - control_process_mappings.csv
    /// - control_threshold_mappings.csv
    /// - control_doctype_mappings.csv
    /// - sod_conflict_pairs.csv
    /// - sod_rules.csv
    /// - coso_control_mapping.csv
    pub fn export_all(
        &self,
        controls: &[InternalControl],
        registry: &ControlMappingRegistry,
        sod_conflicts: &[SodConflictPair],
        sod_rules: &[SodRule],
    ) -> SynthResult<ExportSummary> {
        std::fs::create_dir_all(&self.output_dir)?;

        let summary = ExportSummary {
            controls_count: self.export_controls(controls)?,
            account_mappings_count: self.export_account_mappings(&registry.account_mappings)?,
            process_mappings_count: self.export_process_mappings(&registry.process_mappings)?,
            threshold_mappings_count: self
                .export_threshold_mappings(&registry.threshold_mappings)?,
            doctype_mappings_count: self.export_doctype_mappings(&registry.doc_type_mappings)?,
            sod_conflicts_count: self.export_sod_conflicts(sod_conflicts)?,
            sod_rules_count: self.export_sod_rules(sod_rules)?,
            coso_mappings_count: self.export_coso_mapping(controls)?,
        };

        Ok(summary)
    }

    /// Export internal control definitions.
    pub fn export_controls(&self, controls: &[InternalControl]) -> SynthResult<usize> {
        let path = self.output_dir.join("internal_controls.csv");
        let file = File::create(&path)?;
        let mut writer = BufWriter::with_capacity(256 * 1024, file);

        // Header
        writeln!(
            writer,
            "control_id,control_name,control_type,objective,frequency,owner_role,\
             risk_level,is_key_control,sox_assertion,coso_component,coso_principles,control_scope,maturity_level"
        )?;

        for control in controls {
            // Format COSO principles as semicolon-separated list
            let principles: Vec<String> = control
                .coso_principles
                .iter()
                .map(|p| format!("{}", p))
                .collect();

            writeln!(
                writer,
                "{},{},{:?},{},{:?},{:?},{:?},{},{:?},{},{},{},{}",
                escape_csv(&control.control_id),
                escape_csv(&control.control_name),
                control.control_type,
                escape_csv(&control.objective),
                control.frequency,
                control.owner_role,
                control.risk_level,
                control.is_key_control,
                control.sox_assertion,
                escape_csv(&control.coso_component.to_string()),
                escape_csv(&principles.join(";")),
                escape_csv(&control.control_scope.to_string()),
                escape_csv(&control.maturity_level.to_string()),
            )?;
        }

        writer.flush()?;
        Ok(controls.len())
    }

    /// Export control-to-account mappings.
    pub fn export_account_mappings(
        &self,
        mappings: &[ControlAccountMapping],
    ) -> SynthResult<usize> {
        let path = self.output_dir.join("control_account_mappings.csv");
        let file = File::create(&path)?;
        let mut writer = BufWriter::with_capacity(256 * 1024, file);

        // Header
        writeln!(writer, "control_id,account_numbers,account_sub_types")?;

        for mapping in mappings {
            let account_numbers = mapping.account_numbers.join(";");
            let sub_types: Vec<String> = mapping
                .account_sub_types
                .iter()
                .map(|st| format!("{:?}", st))
                .collect();

            writeln!(
                writer,
                "{},{},{}",
                escape_csv(&mapping.control_id),
                escape_csv(&account_numbers),
                escape_csv(&sub_types.join(";"))
            )?;
        }

        writer.flush()?;
        Ok(mappings.len())
    }

    /// Export control-to-process mappings.
    pub fn export_process_mappings(
        &self,
        mappings: &[ControlProcessMapping],
    ) -> SynthResult<usize> {
        let path = self.output_dir.join("control_process_mappings.csv");
        let file = File::create(&path)?;
        let mut writer = BufWriter::with_capacity(256 * 1024, file);

        // Header
        writeln!(writer, "control_id,business_processes")?;

        for mapping in mappings {
            let processes: Vec<String> = mapping
                .business_processes
                .iter()
                .map(|bp| format!("{:?}", bp))
                .collect();

            writeln!(
                writer,
                "{},{}",
                escape_csv(&mapping.control_id),
                escape_csv(&processes.join(";"))
            )?;
        }

        writer.flush()?;
        Ok(mappings.len())
    }

    /// Export control-to-threshold mappings.
    pub fn export_threshold_mappings(
        &self,
        mappings: &[ControlThresholdMapping],
    ) -> SynthResult<usize> {
        let path = self.output_dir.join("control_threshold_mappings.csv");
        let file = File::create(&path)?;
        let mut writer = BufWriter::with_capacity(256 * 1024, file);

        // Header
        writeln!(
            writer,
            "control_id,amount_threshold,upper_threshold,comparison"
        )?;

        for mapping in mappings {
            writeln!(
                writer,
                "{},{},{},{:?}",
                escape_csv(&mapping.control_id),
                mapping.amount_threshold,
                mapping
                    .upper_threshold
                    .map(|t| t.to_string())
                    .unwrap_or_default(),
                mapping.comparison
            )?;
        }

        writer.flush()?;
        Ok(mappings.len())
    }

    /// Export control-to-document type mappings.
    pub fn export_doctype_mappings(
        &self,
        mappings: &[ControlDocTypeMapping],
    ) -> SynthResult<usize> {
        let path = self.output_dir.join("control_doctype_mappings.csv");
        let file = File::create(&path)?;
        let mut writer = BufWriter::with_capacity(256 * 1024, file);

        // Header
        writeln!(writer, "control_id,document_types")?;

        for mapping in mappings {
            writeln!(
                writer,
                "{},{}",
                escape_csv(&mapping.control_id),
                escape_csv(&mapping.document_types.join(";"))
            )?;
        }

        writer.flush()?;
        Ok(mappings.len())
    }

    /// Export SoD conflict pairs.
    pub fn export_sod_conflicts(&self, conflicts: &[SodConflictPair]) -> SynthResult<usize> {
        let path = self.output_dir.join("sod_conflict_pairs.csv");
        let file = File::create(&path)?;
        let mut writer = BufWriter::with_capacity(256 * 1024, file);

        // Header
        writeln!(writer, "conflict_type,role_a,role_b,description,severity")?;

        for conflict in conflicts {
            writeln!(
                writer,
                "{:?},{:?},{:?},{},{:?}",
                conflict.conflict_type,
                conflict.role_a,
                conflict.role_b,
                escape_csv(&conflict.description),
                conflict.severity
            )?;
        }

        writer.flush()?;
        Ok(conflicts.len())
    }

    /// Export SoD rules.
    pub fn export_sod_rules(&self, rules: &[SodRule]) -> SynthResult<usize> {
        let path = self.output_dir.join("sod_rules.csv");
        let file = File::create(&path)?;
        let mut writer = BufWriter::with_capacity(256 * 1024, file);

        // Header
        writeln!(
            writer,
            "rule_id,name,conflict_type,description,is_active,risk_level"
        )?;

        for rule in rules {
            writeln!(
                writer,
                "{},{},{:?},{},{},{:?}",
                escape_csv(&rule.rule_id),
                escape_csv(&rule.name),
                rule.conflict_type,
                escape_csv(&rule.description),
                rule.is_active,
                rule.risk_level
            )?;
        }

        writer.flush()?;
        Ok(rules.len())
    }

    /// Export COSO control mapping.
    ///
    /// Creates a detailed mapping of controls to COSO components and principles.
    /// Each row represents one principle mapped to a control.
    pub fn export_coso_mapping(&self, controls: &[InternalControl]) -> SynthResult<usize> {
        let path = self.output_dir.join("coso_control_mapping.csv");
        let file = File::create(&path)?;
        let mut writer = BufWriter::with_capacity(256 * 1024, file);

        // Header
        writeln!(
            writer,
            "control_id,coso_component,principle_number,principle_name,control_scope"
        )?;

        let mut row_count = 0;
        for control in controls {
            for principle in &control.coso_principles {
                writeln!(
                    writer,
                    "{},{},{},{},{}",
                    escape_csv(&control.control_id),
                    escape_csv(&control.coso_component.to_string()),
                    principle.principle_number(),
                    escape_csv(&principle.to_string()),
                    escape_csv(&control.control_scope.to_string()),
                )?;
                row_count += 1;
            }
        }

        writer.flush()?;
        Ok(row_count)
    }

    /// Export standard control master data.
    ///
    /// This is a convenience method that exports standard controls,
    /// mappings, and SoD definitions.
    pub fn export_standard(&self) -> SynthResult<ExportSummary> {
        let controls = InternalControl::standard_controls();
        let registry = ControlMappingRegistry::standard();
        let sod_conflicts = SodConflictPair::standard_conflicts();
        let sod_rules = SodRule::standard_rules();

        self.export_all(&controls, &registry, &sod_conflicts, &sod_rules)
    }
}

/// Summary of exported control data.
#[derive(Debug, Default)]
pub struct ExportSummary {
    /// Number of control definitions exported.
    pub controls_count: usize,
    /// Number of account mappings exported.
    pub account_mappings_count: usize,
    /// Number of process mappings exported.
    pub process_mappings_count: usize,
    /// Number of threshold mappings exported.
    pub threshold_mappings_count: usize,
    /// Number of document type mappings exported.
    pub doctype_mappings_count: usize,
    /// Number of SoD conflict pairs exported.
    pub sod_conflicts_count: usize,
    /// Number of SoD rules exported.
    pub sod_rules_count: usize,
    /// Number of COSO control-principle mappings exported.
    pub coso_mappings_count: usize,
}

impl ExportSummary {
    /// Get the total number of records exported.
    pub fn total(&self) -> usize {
        self.controls_count
            + self.account_mappings_count
            + self.process_mappings_count
            + self.threshold_mappings_count
            + self.doctype_mappings_count
            + self.sod_conflicts_count
            + self.sod_rules_count
            + self.coso_mappings_count
    }
}

/// Escape a string for CSV output.
fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_export_standard() {
        let temp_dir = TempDir::new().unwrap();
        let exporter = ControlExporter::new(temp_dir.path());

        let summary = exporter.export_standard().unwrap();

        assert!(summary.controls_count > 0);
        assert!(summary.account_mappings_count > 0);
        assert!(summary.process_mappings_count > 0);
        assert!(summary.sod_conflicts_count > 0);
        assert!(summary.sod_rules_count > 0);
        assert!(summary.coso_mappings_count > 0);

        // Verify files were created
        assert!(temp_dir.path().join("internal_controls.csv").exists());
        assert!(temp_dir
            .path()
            .join("control_account_mappings.csv")
            .exists());
        assert!(temp_dir
            .path()
            .join("control_process_mappings.csv")
            .exists());
        assert!(temp_dir.path().join("sod_conflict_pairs.csv").exists());
        assert!(temp_dir.path().join("sod_rules.csv").exists());
        assert!(temp_dir.path().join("coso_control_mapping.csv").exists());
    }

    #[test]
    fn test_escape_csv() {
        assert_eq!(escape_csv("hello"), "hello");
        assert_eq!(escape_csv("hello,world"), "\"hello,world\"");
        assert_eq!(escape_csv("hello\"world"), "\"hello\"\"world\"");
        assert_eq!(escape_csv("hello\nworld"), "\"hello\nworld\"");
    }

    #[test]
    fn test_export_controls() {
        let temp_dir = TempDir::new().unwrap();
        let exporter = ControlExporter::new(temp_dir.path());

        let controls = InternalControl::standard_controls();
        let count = exporter.export_controls(&controls).unwrap();

        assert_eq!(count, controls.len());

        // Read the file and verify content
        let content =
            std::fs::read_to_string(temp_dir.path().join("internal_controls.csv")).unwrap();
        assert!(content.contains("control_id"));
        assert!(content.contains("C001")); // Cash control
    }

    #[test]
    fn test_export_sod_conflicts() {
        let temp_dir = TempDir::new().unwrap();
        let exporter = ControlExporter::new(temp_dir.path());

        let conflicts = SodConflictPair::standard_conflicts();
        let count = exporter.export_sod_conflicts(&conflicts).unwrap();

        assert_eq!(count, conflicts.len());

        let content =
            std::fs::read_to_string(temp_dir.path().join("sod_conflict_pairs.csv")).unwrap();
        assert!(content.contains("PreparerApprover"));
    }
}
