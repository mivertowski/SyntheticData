//! Export project accounting data to CSV files.
//!
//! Exports projects, WBS elements, project cost lines, revenue records,
//! milestones, change orders, retainage, and earned value metrics.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use datasynth_core::error::SynthResult;
use datasynth_core::models::{
    ChangeOrder, EarnedValueMetric, Project, ProjectCostLine, ProjectMilestone, ProjectRevenue,
    Retainage,
};

// ---------------------------------------------------------------------------
// Export summary
// ---------------------------------------------------------------------------

/// Summary of exported project accounting data.
#[derive(Debug, Default)]
pub struct ProjectAccountingExportSummary {
    pub projects_count: usize,
    pub wbs_elements_count: usize,
    pub cost_lines_count: usize,
    pub revenue_count: usize,
    pub milestones_count: usize,
    pub change_orders_count: usize,
    pub retainage_count: usize,
    pub earned_value_count: usize,
}

impl ProjectAccountingExportSummary {
    /// Total number of rows exported across all files.
    pub fn total(&self) -> usize {
        self.projects_count
            + self.wbs_elements_count
            + self.cost_lines_count
            + self.revenue_count
            + self.milestones_count
            + self.change_orders_count
            + self.retainage_count
            + self.earned_value_count
    }
}

// ---------------------------------------------------------------------------
// Exporter
// ---------------------------------------------------------------------------

/// Exporter for project accounting data.
pub struct ProjectAccountingExporter {
    output_dir: PathBuf,
}

impl ProjectAccountingExporter {
    /// Create a new project accounting exporter writing to the given directory.
    pub fn new(output_dir: impl AsRef<Path>) -> Self {
        Self {
            output_dir: output_dir.as_ref().to_path_buf(),
        }
    }

    /// Export projects to `projects.csv`.
    pub fn export_projects(&self, data: &[Project]) -> SynthResult<usize> {
        let path = self.output_dir.join("projects.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::new(file);

        writeln!(
            w,
            "project_id,name,description,project_type,status,budget,company_code,responsible_cost_center,start_date,end_date,wbs_count"
        )?;

        for p in data {
            writeln!(
                w,
                "{},{},{},{:?},{:?},{},{},{},{},{},{}",
                esc(&p.project_id),
                esc(&p.name),
                esc(&p.description),
                p.project_type,
                p.status,
                p.budget,
                esc(&p.company_code),
                esc(&p.responsible_cost_center),
                p.start_date.as_deref().unwrap_or(""),
                p.end_date.as_deref().unwrap_or(""),
                p.wbs_elements.len(),
            )?;
        }

        w.flush()?;
        Ok(data.len())
    }

    /// Export WBS elements to `wbs_elements.csv`.
    pub fn export_wbs_elements(&self, data: &[Project]) -> SynthResult<usize> {
        let path = self.output_dir.join("wbs_elements.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::new(file);

        writeln!(
            w,
            "wbs_id,project_id,description,level,parent_wbs,budget,actual_costs,is_active,responsible_cost_center"
        )?;

        let mut count = 0;
        for project in data {
            for wbs in &project.wbs_elements {
                writeln!(
                    w,
                    "{},{},{},{},{},{},{},{},{}",
                    esc(&wbs.wbs_id),
                    esc(&wbs.project_id),
                    esc(&wbs.description),
                    wbs.level,
                    wbs.parent_wbs.as_deref().unwrap_or(""),
                    wbs.budget,
                    wbs.actual_costs,
                    wbs.is_active,
                    wbs.responsible_cost_center.as_deref().unwrap_or(""),
                )?;
                count += 1;
            }
        }

        w.flush()?;
        Ok(count)
    }

    /// Export project cost lines to `project_cost_lines.csv`.
    pub fn export_cost_lines(&self, data: &[ProjectCostLine]) -> SynthResult<usize> {
        let path = self.output_dir.join("project_cost_lines.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::new(file);

        writeln!(
            w,
            "id,project_id,wbs_id,entity_id,posting_date,cost_category,source_type,source_document_id,amount,currency,hours,description"
        )?;

        for cl in data {
            writeln!(
                w,
                "{},{},{},{},{},{:?},{:?},{},{},{},{},{}",
                esc(&cl.id),
                esc(&cl.project_id),
                esc(&cl.wbs_id),
                esc(&cl.entity_id),
                cl.posting_date,
                cl.cost_category,
                cl.source_type,
                esc(&cl.source_document_id),
                cl.amount,
                esc(&cl.currency),
                cl.hours.map_or(String::new(), |h| h.to_string()),
                esc(&cl.description),
            )?;
        }

        w.flush()?;
        Ok(data.len())
    }

    /// Export project revenue to `project_revenue.csv`.
    pub fn export_revenue(&self, data: &[ProjectRevenue]) -> SynthResult<usize> {
        let path = self.output_dir.join("project_revenue.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::new(file);

        writeln!(
            w,
            "id,project_id,entity_id,period_start,period_end,contract_value,estimated_total_cost,costs_to_date,completion_pct,method,measure,cumulative_revenue,period_revenue,billed_to_date,unbilled_revenue,gross_margin_pct"
        )?;

        for r in data {
            writeln!(
                w,
                "{},{},{},{},{},{},{},{},{},{:?},{:?},{},{},{},{},{}",
                esc(&r.id),
                esc(&r.project_id),
                esc(&r.entity_id),
                r.period_start,
                r.period_end,
                r.contract_value,
                r.estimated_total_cost,
                r.costs_to_date,
                r.completion_pct,
                r.method,
                r.measure,
                r.cumulative_revenue,
                r.period_revenue,
                r.billed_to_date,
                r.unbilled_revenue,
                r.gross_margin_pct,
            )?;
        }

        w.flush()?;
        Ok(data.len())
    }

    /// Export project milestones to `project_milestones.csv`.
    pub fn export_milestones(&self, data: &[ProjectMilestone]) -> SynthResult<usize> {
        let path = self.output_dir.join("project_milestones.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::new(file);

        writeln!(
            w,
            "id,project_id,wbs_id,name,planned_date,actual_date,status,payment_amount,weight,sequence"
        )?;

        for m in data {
            writeln!(
                w,
                "{},{},{},{},{},{},{:?},{},{},{}",
                esc(&m.id),
                esc(&m.project_id),
                m.wbs_id.as_deref().unwrap_or(""),
                esc(&m.name),
                m.planned_date,
                m.actual_date.map_or(String::new(), |d| d.to_string()),
                m.status,
                m.payment_amount,
                m.weight,
                m.sequence,
            )?;
        }

        w.flush()?;
        Ok(data.len())
    }

    /// Export change orders to `change_orders.csv`.
    pub fn export_change_orders(&self, data: &[ChangeOrder]) -> SynthResult<usize> {
        let path = self.output_dir.join("change_orders.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::new(file);

        writeln!(
            w,
            "id,project_id,number,submitted_date,approved_date,status,reason,description,cost_impact,estimated_cost_impact,schedule_impact_days"
        )?;

        for co in data {
            writeln!(
                w,
                "{},{},{},{},{},{:?},{:?},{},{},{},{}",
                esc(&co.id),
                esc(&co.project_id),
                co.number,
                co.submitted_date,
                co.approved_date.map_or(String::new(), |d| d.to_string()),
                co.status,
                co.reason,
                esc(&co.description),
                co.cost_impact,
                co.estimated_cost_impact,
                co.schedule_impact_days,
            )?;
        }

        w.flush()?;
        Ok(data.len())
    }

    /// Export retainage records to `retainage.csv`.
    pub fn export_retainage(&self, data: &[Retainage]) -> SynthResult<usize> {
        let path = self.output_dir.join("retainage.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::new(file);

        writeln!(
            w,
            "id,project_id,entity_id,vendor_id,retainage_pct,total_held,released_amount,status,inception_date,last_release_date"
        )?;

        for r in data {
            writeln!(
                w,
                "{},{},{},{},{},{},{},{:?},{},{}",
                esc(&r.id),
                esc(&r.project_id),
                esc(&r.entity_id),
                esc(&r.vendor_id),
                r.retainage_pct,
                r.total_held,
                r.released_amount,
                r.status,
                r.inception_date,
                r.last_release_date.map_or(String::new(), |d| d.to_string()),
            )?;
        }

        w.flush()?;
        Ok(data.len())
    }

    /// Export earned value metrics to `earned_value_metrics.csv`.
    pub fn export_earned_value(&self, data: &[EarnedValueMetric]) -> SynthResult<usize> {
        let path = self.output_dir.join("earned_value_metrics.csv");
        let file = File::create(&path)?;
        let mut w = BufWriter::new(file);

        writeln!(
            w,
            "id,project_id,measurement_date,bac,planned_value,earned_value,actual_cost,schedule_variance,cost_variance,spi,cpi,eac,etc,tcpi"
        )?;

        for e in data {
            writeln!(
                w,
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                esc(&e.id),
                esc(&e.project_id),
                e.measurement_date,
                e.bac,
                e.planned_value,
                e.earned_value,
                e.actual_cost,
                e.schedule_variance,
                e.cost_variance,
                e.spi,
                e.cpi,
                e.eac,
                e.etc,
                e.tcpi,
            )?;
        }

        w.flush()?;
        Ok(data.len())
    }

    /// Export all project accounting data and return a summary.
    #[allow(clippy::too_many_arguments)]
    pub fn export_all(
        &self,
        projects: &[Project],
        cost_lines: &[ProjectCostLine],
        revenues: &[ProjectRevenue],
        milestones: &[ProjectMilestone],
        change_orders: &[ChangeOrder],
        retainage: &[Retainage],
        earned_value: &[EarnedValueMetric],
    ) -> SynthResult<ProjectAccountingExportSummary> {
        std::fs::create_dir_all(&self.output_dir)?;

        let summary = ProjectAccountingExportSummary {
            projects_count: self.export_projects(projects)?,
            wbs_elements_count: self.export_wbs_elements(projects)?,
            cost_lines_count: self.export_cost_lines(cost_lines)?,
            revenue_count: self.export_revenue(revenues)?,
            milestones_count: self.export_milestones(milestones)?,
            change_orders_count: self.export_change_orders(change_orders)?,
            retainage_count: self.export_retainage(retainage)?,
            earned_value_count: self.export_earned_value(earned_value)?,
        };
        Ok(summary)
    }
}

// ---------------------------------------------------------------------------
// CSV helper
// ---------------------------------------------------------------------------

fn esc(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use datasynth_core::models::{CostCategory, CostSourceType, ProjectType, WbsElement};
    use rust_decimal_macros::dec;
    use std::fs;
    use tempfile::TempDir;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    fn test_project() -> Project {
        let mut project = Project::new("PRJ-001", "Test Project", ProjectType::Customer)
            .with_budget(dec!(1000000))
            .with_company("TEST");
        project.start_date = Some("2024-01-01".to_string());
        project.end_date = Some("2024-12-31".to_string());
        project.add_wbs_element(
            WbsElement::new("PRJ-001.01", "PRJ-001", "Phase 1").with_budget(dec!(600000)),
        );
        project.add_wbs_element(
            WbsElement::new("PRJ-001.02", "PRJ-001", "Phase 2").with_budget(dec!(400000)),
        );
        project
    }

    #[test]
    fn test_export_projects() {
        let tmp = TempDir::new().unwrap();
        let exporter = ProjectAccountingExporter::new(tmp.path());
        let projects = vec![test_project()];

        let count = exporter.export_projects(&projects).unwrap();
        assert_eq!(count, 1);

        let content = fs::read_to_string(tmp.path().join("projects.csv")).unwrap();
        assert!(content.contains("project_id,name"));
        assert!(content.contains("PRJ-001"));
    }

    #[test]
    fn test_export_wbs_elements() {
        let tmp = TempDir::new().unwrap();
        let exporter = ProjectAccountingExporter::new(tmp.path());
        let projects = vec![test_project()];

        let count = exporter.export_wbs_elements(&projects).unwrap();
        assert_eq!(count, 2); // Two WBS elements

        let content = fs::read_to_string(tmp.path().join("wbs_elements.csv")).unwrap();
        assert!(content.contains("PRJ-001.01"));
        assert!(content.contains("PRJ-001.02"));
    }

    #[test]
    fn test_export_cost_lines() {
        let tmp = TempDir::new().unwrap();
        let exporter = ProjectAccountingExporter::new(tmp.path());

        let lines = vec![ProjectCostLine::new(
            "PCL-001",
            "PRJ-001",
            "PRJ-001.01",
            "TEST",
            d("2024-03-15"),
            CostCategory::Labor,
            CostSourceType::TimeEntry,
            "TE-001",
            dec!(1500),
            "USD",
        )];

        let count = exporter.export_cost_lines(&lines).unwrap();
        assert_eq!(count, 1);

        let content = fs::read_to_string(tmp.path().join("project_cost_lines.csv")).unwrap();
        assert!(content.contains("PCL-001"));
        assert!(content.contains("1500"));
    }

    #[test]
    fn test_export_earned_value() {
        let tmp = TempDir::new().unwrap();
        let exporter = ProjectAccountingExporter::new(tmp.path());

        let metrics = vec![EarnedValueMetric::compute(
            "EVM-001",
            "PRJ-001",
            d("2024-06-30"),
            dec!(1000000),
            dec!(500000),
            dec!(400000),
            dec!(450000),
        )];

        let count = exporter.export_earned_value(&metrics).unwrap();
        assert_eq!(count, 1);

        let content = fs::read_to_string(tmp.path().join("earned_value_metrics.csv")).unwrap();
        assert!(content.contains("EVM-001"));
        assert!(content.contains("spi"));
    }

    #[test]
    fn test_export_all() {
        let tmp = TempDir::new().unwrap();
        let exporter = ProjectAccountingExporter::new(tmp.path());

        let projects = vec![test_project()];
        let cost_lines = vec![ProjectCostLine::new(
            "PCL-001",
            "PRJ-001",
            "PRJ-001.01",
            "TEST",
            d("2024-03-15"),
            CostCategory::Labor,
            CostSourceType::TimeEntry,
            "TE-001",
            dec!(1500),
            "USD",
        )];
        let revenues = vec![];
        let milestones = vec![ProjectMilestone::new(
            "MS-001",
            "PRJ-001",
            "Kickoff",
            d("2024-02-01"),
            1,
        )];
        let change_orders = vec![];
        let retainage = vec![];
        let evm = vec![EarnedValueMetric::compute(
            "EVM-001",
            "PRJ-001",
            d("2024-06-30"),
            dec!(1000000),
            dec!(500000),
            dec!(400000),
            dec!(450000),
        )];

        let summary = exporter
            .export_all(
                &projects,
                &cost_lines,
                &revenues,
                &milestones,
                &change_orders,
                &retainage,
                &evm,
            )
            .unwrap();

        assert_eq!(summary.projects_count, 1);
        assert_eq!(summary.wbs_elements_count, 2);
        assert_eq!(summary.cost_lines_count, 1);
        assert_eq!(summary.milestones_count, 1);
        assert_eq!(summary.earned_value_count, 1);
        assert!(summary.total() > 0);

        // Verify all files exist
        assert!(tmp.path().join("projects.csv").exists());
        assert!(tmp.path().join("wbs_elements.csv").exists());
        assert!(tmp.path().join("project_cost_lines.csv").exists());
        assert!(tmp.path().join("project_revenue.csv").exists());
        assert!(tmp.path().join("project_milestones.csv").exists());
        assert!(tmp.path().join("change_orders.csv").exists());
        assert!(tmp.path().join("retainage.csv").exists());
        assert!(tmp.path().join("earned_value_metrics.csv").exists());
    }
}
