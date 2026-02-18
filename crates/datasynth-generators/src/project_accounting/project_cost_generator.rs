//! Project cost generator (linking pattern).
//!
//! Probabilistically links existing source documents (time entries, expense reports,
//! purchase orders, vendor invoices) to project WBS elements, creating
//! [`ProjectCostLine`] records based on configurable allocation rates.

use chrono::NaiveDate;
use datasynth_config::schema::CostAllocationConfig;
use datasynth_core::models::{CostCategory, CostSourceType, ProjectCostLine, ProjectPool};
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;

/// A minimal source document reference for linking.
#[derive(Debug, Clone)]
pub struct SourceDocument {
    /// Document ID
    pub id: String,
    /// Entity (company code) that created the document
    pub entity_id: String,
    /// Date of the document
    pub date: NaiveDate,
    /// Amount on the document
    pub amount: Decimal,
    /// Source type
    pub source_type: CostSourceType,
    /// Hours (for time entries)
    pub hours: Option<Decimal>,
}

/// Generates [`ProjectCostLine`] records by linking source documents to projects.
pub struct ProjectCostGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
    config: CostAllocationConfig,
    counter: u64,
}

impl ProjectCostGenerator {
    /// Create a new project cost generator.
    pub fn new(config: CostAllocationConfig, seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::ProjectAccounting),
            config,
            counter: 0,
        }
    }

    /// Link source documents to projects, creating cost lines.
    ///
    /// Each document is probabilistically assigned to a project based on
    /// the allocation rate for its source type. The assigned WBS element
    /// is chosen randomly from the project's active elements.
    pub fn link_documents(
        &mut self,
        pool: &ProjectPool,
        documents: &[SourceDocument],
    ) -> Vec<ProjectCostLine> {
        let mut cost_lines = Vec::new();

        for doc in documents {
            let rate = self.rate_for(doc.source_type);
            if self.rng.gen::<f64>() >= rate {
                continue;
            }

            // Pick a random active project
            let project = match pool.random_active_project(&mut self.rng) {
                Some(p) => p,
                None => continue,
            };

            // Pick a random active WBS element
            let active_wbs = project.active_wbs_elements();
            if active_wbs.is_empty() {
                continue;
            }
            let wbs = active_wbs[self.rng.gen_range(0..active_wbs.len())];

            self.counter += 1;
            let cost_line_id = format!("PCL-{:06}", self.counter);
            let category = self.category_for(doc.source_type);

            let mut line = ProjectCostLine::new(
                cost_line_id,
                &project.project_id,
                &wbs.wbs_id,
                &doc.entity_id,
                doc.date,
                category,
                doc.source_type,
                &doc.id,
                doc.amount,
                "USD",
            );

            if let Some(hours) = doc.hours {
                line = line.with_hours(hours);
            }

            cost_lines.push(line);
        }

        cost_lines
    }

    /// Get the allocation rate for a source type.
    fn rate_for(&self, source_type: CostSourceType) -> f64 {
        match source_type {
            CostSourceType::TimeEntry => self.config.time_entry_project_rate,
            CostSourceType::ExpenseReport => self.config.expense_project_rate,
            CostSourceType::PurchaseOrder => self.config.purchase_order_project_rate,
            CostSourceType::VendorInvoice => self.config.vendor_invoice_project_rate,
            CostSourceType::JournalEntry => 0.0, // JEs aren't linked by default
        }
    }

    /// Map source types to cost categories.
    fn category_for(&self, source_type: CostSourceType) -> CostCategory {
        match source_type {
            CostSourceType::TimeEntry => CostCategory::Labor,
            CostSourceType::ExpenseReport => CostCategory::Travel,
            CostSourceType::PurchaseOrder => CostCategory::Material,
            CostSourceType::VendorInvoice => CostCategory::Subcontractor,
            CostSourceType::JournalEntry => CostCategory::Overhead,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::{Project, ProjectType};
    use rust_decimal_macros::dec;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    fn test_pool() -> ProjectPool {
        let mut pool = ProjectPool::new();
        for i in 0..5 {
            let mut project = Project::new(
                &format!("PRJ-{:03}", i + 1),
                &format!("Test Project {}", i + 1),
                ProjectType::Customer,
            )
            .with_budget(Decimal::from(1_000_000))
            .with_company("TEST");

            project.add_wbs_element(
                datasynth_core::models::WbsElement::new(
                    &format!("PRJ-{:03}.01", i + 1),
                    &format!("PRJ-{:03}", i + 1),
                    "Phase 1",
                )
                .with_budget(Decimal::from(500_000)),
            );
            project.add_wbs_element(
                datasynth_core::models::WbsElement::new(
                    &format!("PRJ-{:03}.02", i + 1),
                    &format!("PRJ-{:03}", i + 1),
                    "Phase 2",
                )
                .with_budget(Decimal::from(500_000)),
            );

            pool.add_project(project);
        }
        pool
    }

    fn test_time_entries(count: usize) -> Vec<SourceDocument> {
        (0..count)
            .map(|i| SourceDocument {
                id: format!("TE-{:04}", i + 1),
                entity_id: "TEST".to_string(),
                date: d("2024-03-15"),
                amount: dec!(750),
                source_type: CostSourceType::TimeEntry,
                hours: Some(dec!(8)),
            })
            .collect()
    }

    #[test]
    fn test_project_cost_linking() {
        let pool = test_pool();
        let time_entries = test_time_entries(100);
        let config = CostAllocationConfig {
            time_entry_project_rate: 0.60,
            ..Default::default()
        };

        let mut gen = ProjectCostGenerator::new(config, 42);
        let cost_lines = gen.link_documents(&pool, &time_entries);

        // ~60% of 100 time entries should be linked (with variance)
        let linked_count = cost_lines.len();
        assert!(
            linked_count >= 40 && linked_count <= 80,
            "Expected ~60 linked, got {}",
            linked_count
        );

        // All linked entries should reference valid projects and WBS elements
        for line in &cost_lines {
            assert!(
                pool.projects
                    .iter()
                    .any(|p| p.project_id == line.project_id),
                "Cost line should reference a valid project"
            );
            assert_eq!(line.cost_category, CostCategory::Labor);
            assert_eq!(line.source_type, CostSourceType::TimeEntry);
            assert!(line.hours.is_some());
        }
    }

    #[test]
    fn test_zero_rate_links_nothing() {
        let pool = test_pool();
        let docs = test_time_entries(50);
        let config = CostAllocationConfig {
            time_entry_project_rate: 0.0,
            expense_project_rate: 0.0,
            purchase_order_project_rate: 0.0,
            vendor_invoice_project_rate: 0.0,
        };

        let mut gen = ProjectCostGenerator::new(config, 42);
        let cost_lines = gen.link_documents(&pool, &docs);
        assert!(cost_lines.is_empty(), "Zero rate should produce no links");
    }

    #[test]
    fn test_full_rate_links_everything() {
        let pool = test_pool();
        let docs = test_time_entries(50);
        let config = CostAllocationConfig {
            time_entry_project_rate: 1.0,
            ..Default::default()
        };

        let mut gen = ProjectCostGenerator::new(config, 42);
        let cost_lines = gen.link_documents(&pool, &docs);
        assert_eq!(cost_lines.len(), 50, "100% rate should link all documents");
    }

    #[test]
    fn test_expense_linking() {
        let pool = test_pool();
        let expenses: Vec<SourceDocument> = (0..50)
            .map(|i| SourceDocument {
                id: format!("EXP-{:04}", i + 1),
                entity_id: "TEST".to_string(),
                date: d("2024-03-15"),
                amount: dec!(350),
                source_type: CostSourceType::ExpenseReport,
                hours: None,
            })
            .collect();

        let config = CostAllocationConfig {
            expense_project_rate: 1.0,
            ..Default::default()
        };

        let mut gen = ProjectCostGenerator::new(config, 42);
        let cost_lines = gen.link_documents(&pool, &expenses);

        assert_eq!(cost_lines.len(), 50);
        for line in &cost_lines {
            assert_eq!(line.cost_category, CostCategory::Travel);
            assert_eq!(line.source_type, CostSourceType::ExpenseReport);
            assert!(line.hours.is_none());
        }
    }

    #[test]
    fn test_deterministic_linking() {
        let pool = test_pool();
        let docs = test_time_entries(100);
        let config = CostAllocationConfig::default();

        let mut gen1 = ProjectCostGenerator::new(config.clone(), 42);
        let lines1 = gen1.link_documents(&pool, &docs);

        let mut gen2 = ProjectCostGenerator::new(config, 42);
        let lines2 = gen2.link_documents(&pool, &docs);

        assert_eq!(lines1.len(), lines2.len());
        for (l1, l2) in lines1.iter().zip(lines2.iter()) {
            assert_eq!(l1.project_id, l2.project_id);
            assert_eq!(l1.wbs_id, l2.wbs_id);
            assert_eq!(l1.amount, l2.amount);
        }
    }
}
