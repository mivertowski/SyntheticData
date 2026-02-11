//! Project and WBS (Work Breakdown Structure) models.
//!
//! Provides project master data for capital projects, internal projects,
//! and associated WBS elements for cost tracking.

use rand::seq::SliceRandom;
use rand::Rng;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProjectType {
    /// Capital expenditure project (assets)
    #[default]
    Capital,
    /// Internal project (expensed)
    Internal,
    /// Research and development
    RandD,
    /// Customer project (billable)
    Customer,
    /// Maintenance project
    Maintenance,
    /// IT/Technology project
    Technology,
}

impl ProjectType {
    /// Check if this project type typically capitalizes costs.
    pub fn is_capitalizable(&self) -> bool {
        matches!(self, Self::Capital | Self::RandD)
    }

    /// Get typical account type for this project.
    pub fn typical_account_prefix(&self) -> &'static str {
        match self {
            Self::Capital => "1",     // Assets
            Self::Internal => "5",    // Expenses
            Self::RandD => "1",       // Assets (capitalized) or "5" (expensed)
            Self::Customer => "4",    // Revenue
            Self::Maintenance => "5", // Expenses
            Self::Technology => "1",  // Assets (often capitalized)
        }
    }
}

/// Status of a project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    /// Project is planned but not started
    #[default]
    Planned,
    /// Project is active
    Active,
    /// Project is on hold
    OnHold,
    /// Project is complete
    Completed,
    /// Project was cancelled
    Cancelled,
    /// Project is in closing phase
    Closing,
}

impl ProjectStatus {
    /// Check if project can receive postings.
    pub fn allows_postings(&self) -> bool {
        matches!(self, Self::Active | Self::Closing)
    }
}

/// WBS (Work Breakdown Structure) element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WbsElement {
    /// WBS element ID (e.g., "P-001.01.001")
    pub wbs_id: String,

    /// Parent project ID
    pub project_id: String,

    /// Description
    pub description: String,

    /// Level in the hierarchy (1 = top level)
    pub level: u8,

    /// Parent WBS element ID (None for top-level)
    pub parent_wbs: Option<String>,

    /// Budget amount
    pub budget: Decimal,

    /// Actual costs to date
    pub actual_costs: Decimal,

    /// Is this element active for postings
    pub is_active: bool,

    /// Responsible cost center
    pub responsible_cost_center: Option<String>,
}

impl WbsElement {
    /// Create a new WBS element.
    pub fn new(wbs_id: &str, project_id: &str, description: &str) -> Self {
        Self {
            wbs_id: wbs_id.to_string(),
            project_id: project_id.to_string(),
            description: description.to_string(),
            level: 1,
            parent_wbs: None,
            budget: Decimal::ZERO,
            actual_costs: Decimal::ZERO,
            is_active: true,
            responsible_cost_center: None,
        }
    }

    /// Set the level and parent.
    pub fn with_parent(mut self, parent_wbs: &str, level: u8) -> Self {
        self.parent_wbs = Some(parent_wbs.to_string());
        self.level = level;
        self
    }

    /// Set the budget.
    pub fn with_budget(mut self, budget: Decimal) -> Self {
        self.budget = budget;
        self
    }

    /// Calculate remaining budget.
    pub fn remaining_budget(&self) -> Decimal {
        self.budget - self.actual_costs
    }

    /// Check if over budget.
    pub fn is_over_budget(&self) -> bool {
        self.actual_costs > self.budget
    }
}

/// Project master data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Project ID (e.g., "P-001234")
    pub project_id: String,

    /// Project name
    pub name: String,

    /// Project description
    pub description: String,

    /// Type of project
    pub project_type: ProjectType,

    /// Current status
    pub status: ProjectStatus,

    /// Total budget
    pub budget: Decimal,

    /// Responsible cost center
    pub responsible_cost_center: String,

    /// WBS elements
    pub wbs_elements: Vec<WbsElement>,

    /// Company code
    pub company_code: String,

    /// Start date (YYYY-MM-DD)
    pub start_date: Option<String>,

    /// End date (YYYY-MM-DD)
    pub end_date: Option<String>,
}

impl Project {
    /// Create a new project.
    pub fn new(project_id: &str, name: &str, project_type: ProjectType) -> Self {
        Self {
            project_id: project_id.to_string(),
            name: name.to_string(),
            description: String::new(),
            project_type,
            status: ProjectStatus::Active,
            budget: Decimal::ZERO,
            responsible_cost_center: "1000".to_string(),
            wbs_elements: Vec::new(),
            company_code: "1000".to_string(),
            start_date: None,
            end_date: None,
        }
    }

    /// Set the budget.
    pub fn with_budget(mut self, budget: Decimal) -> Self {
        self.budget = budget;
        self
    }

    /// Set the company code.
    pub fn with_company(mut self, company_code: &str) -> Self {
        self.company_code = company_code.to_string();
        self
    }

    /// Add a WBS element.
    pub fn add_wbs_element(&mut self, element: WbsElement) {
        self.wbs_elements.push(element);
    }

    /// Get active WBS elements.
    pub fn active_wbs_elements(&self) -> Vec<&WbsElement> {
        self.wbs_elements.iter().filter(|w| w.is_active).collect()
    }

    /// Check if project allows postings.
    pub fn allows_postings(&self) -> bool {
        self.status.allows_postings()
    }

    /// Get total actual costs across all WBS elements.
    pub fn total_actual_costs(&self) -> Decimal {
        self.wbs_elements.iter().map(|w| w.actual_costs).sum()
    }

    /// Check if project is over budget.
    pub fn is_over_budget(&self) -> bool {
        self.total_actual_costs() > self.budget
    }
}

/// Pool of projects for transaction generation.
#[derive(Debug, Clone, Default)]
pub struct ProjectPool {
    /// All projects
    pub projects: Vec<Project>,
    /// Index by project type
    type_index: HashMap<ProjectType, Vec<usize>>,
}

impl ProjectPool {
    /// Create a new empty project pool.
    pub fn new() -> Self {
        Self {
            projects: Vec::new(),
            type_index: HashMap::new(),
        }
    }

    /// Add a project to the pool.
    pub fn add_project(&mut self, project: Project) {
        let idx = self.projects.len();
        let project_type = project.project_type;
        self.projects.push(project);
        self.type_index.entry(project_type).or_default().push(idx);
    }

    /// Get a random active project.
    pub fn random_active_project(&self, rng: &mut impl Rng) -> Option<&Project> {
        let active: Vec<_> = self
            .projects
            .iter()
            .filter(|p| p.allows_postings())
            .collect();
        active.choose(rng).copied()
    }

    /// Get a random project of a specific type.
    pub fn random_project_of_type(
        &self,
        project_type: ProjectType,
        rng: &mut impl Rng,
    ) -> Option<&Project> {
        self.type_index
            .get(&project_type)
            .and_then(|indices| indices.choose(rng))
            .map(|&idx| &self.projects[idx])
            .filter(|p| p.allows_postings())
    }

    /// Rebuild the type index.
    pub fn rebuild_index(&mut self) {
        self.type_index.clear();
        for (idx, project) in self.projects.iter().enumerate() {
            self.type_index
                .entry(project.project_type)
                .or_default()
                .push(idx);
        }
    }

    /// Generate a standard project pool.
    pub fn standard(company_code: &str) -> Self {
        let mut pool = Self::new();

        // Capital projects
        let capital_projects = [
            (
                "PRJ-CAP-001",
                "Data Center Expansion",
                Decimal::from(5000000),
            ),
            (
                "PRJ-CAP-002",
                "Manufacturing Line Upgrade",
                Decimal::from(2500000),
            ),
            (
                "PRJ-CAP-003",
                "Office Building Renovation",
                Decimal::from(1500000),
            ),
            (
                "PRJ-CAP-004",
                "Fleet Vehicle Replacement",
                Decimal::from(800000),
            ),
            (
                "PRJ-CAP-005",
                "Warehouse Automation",
                Decimal::from(3000000),
            ),
        ];

        for (id, name, budget) in capital_projects {
            let mut project = Project::new(id, name, ProjectType::Capital)
                .with_budget(budget)
                .with_company(company_code);

            // Add WBS elements
            project.add_wbs_element(
                WbsElement::new(&format!("{}.01", id), id, "Planning & Design").with_budget(
                    budget * Decimal::from_f64_retain(0.1).expect("valid decimal fraction"),
                ),
            );
            project.add_wbs_element(
                WbsElement::new(&format!("{}.02", id), id, "Procurement").with_budget(
                    budget * Decimal::from_f64_retain(0.4).expect("valid decimal fraction"),
                ),
            );
            project.add_wbs_element(
                WbsElement::new(&format!("{}.03", id), id, "Implementation").with_budget(
                    budget * Decimal::from_f64_retain(0.4).expect("valid decimal fraction"),
                ),
            );
            project.add_wbs_element(
                WbsElement::new(&format!("{}.04", id), id, "Testing & Validation").with_budget(
                    budget * Decimal::from_f64_retain(0.1).expect("valid decimal fraction"),
                ),
            );

            pool.add_project(project);
        }

        // Internal projects
        let internal_projects = [
            (
                "PRJ-INT-001",
                "Process Improvement Initiative",
                Decimal::from(250000),
            ),
            (
                "PRJ-INT-002",
                "Employee Training Program",
                Decimal::from(150000),
            ),
            (
                "PRJ-INT-003",
                "Quality Certification",
                Decimal::from(100000),
            ),
        ];

        for (id, name, budget) in internal_projects {
            let mut project = Project::new(id, name, ProjectType::Internal)
                .with_budget(budget)
                .with_company(company_code);

            project.add_wbs_element(
                WbsElement::new(&format!("{}.01", id), id, "Phase 1").with_budget(
                    budget * Decimal::from_f64_retain(0.5).expect("valid decimal fraction"),
                ),
            );
            project.add_wbs_element(
                WbsElement::new(&format!("{}.02", id), id, "Phase 2").with_budget(
                    budget * Decimal::from_f64_retain(0.5).expect("valid decimal fraction"),
                ),
            );

            pool.add_project(project);
        }

        // Technology projects
        let tech_projects = [
            (
                "PRJ-IT-001",
                "ERP System Implementation",
                Decimal::from(2000000),
            ),
            ("PRJ-IT-002", "Cloud Migration", Decimal::from(1000000)),
            (
                "PRJ-IT-003",
                "Cybersecurity Enhancement",
                Decimal::from(500000),
            ),
        ];

        for (id, name, budget) in tech_projects {
            let mut project = Project::new(id, name, ProjectType::Technology)
                .with_budget(budget)
                .with_company(company_code);

            project.add_wbs_element(
                WbsElement::new(&format!("{}.01", id), id, "Assessment").with_budget(
                    budget * Decimal::from_f64_retain(0.15).expect("valid decimal fraction"),
                ),
            );
            project.add_wbs_element(
                WbsElement::new(&format!("{}.02", id), id, "Development").with_budget(
                    budget * Decimal::from_f64_retain(0.50).expect("valid decimal fraction"),
                ),
            );
            project.add_wbs_element(
                WbsElement::new(&format!("{}.03", id), id, "Deployment").with_budget(
                    budget * Decimal::from_f64_retain(0.25).expect("valid decimal fraction"),
                ),
            );
            project.add_wbs_element(
                WbsElement::new(&format!("{}.04", id), id, "Support").with_budget(
                    budget * Decimal::from_f64_retain(0.10).expect("valid decimal fraction"),
                ),
            );

            pool.add_project(project);
        }

        pool
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_project_creation() {
        let project = Project::new("P-001", "Test Project", ProjectType::Capital)
            .with_budget(Decimal::from(1000000));

        assert_eq!(project.project_id, "P-001");
        assert!(project.allows_postings());
        assert!(project.project_type.is_capitalizable());
    }

    #[test]
    fn test_wbs_element() {
        let wbs =
            WbsElement::new("P-001.01", "P-001", "Phase 1").with_budget(Decimal::from(100000));

        assert_eq!(wbs.remaining_budget(), Decimal::from(100000));
        assert!(!wbs.is_over_budget());
    }

    #[test]
    fn test_project_pool() {
        let pool = ProjectPool::standard("1000");

        assert!(!pool.projects.is_empty());

        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let project = pool.random_active_project(&mut rng);
        assert!(project.is_some());

        let cap_project = pool.random_project_of_type(ProjectType::Capital, &mut rng);
        assert!(cap_project.is_some());
    }

    #[test]
    fn test_project_budget_tracking() {
        let mut project =
            Project::new("P-001", "Test", ProjectType::Capital).with_budget(Decimal::from(100000));

        let mut wbs =
            WbsElement::new("P-001.01", "P-001", "Phase 1").with_budget(Decimal::from(100000));
        wbs.actual_costs = Decimal::from(50000);
        project.add_wbs_element(wbs);

        assert_eq!(project.total_actual_costs(), Decimal::from(50000));
        assert!(!project.is_over_budget());
    }
}
