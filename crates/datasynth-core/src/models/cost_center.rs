//! Cost center hierarchy model for organizational cost accounting.
//!
//! Represents the two-level cost center hierarchy (departments → sub-departments)
//! used in SAP-style management accounting (CO module).

use serde::{Deserialize, Serialize};

/// Category classification for cost centers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CostCenterCategory {
    /// General and administrative functions (HR, Finance, Legal, IT)
    #[default]
    Administration,
    /// Direct production or manufacturing cost centers
    Production,
    /// Sales and marketing cost centers
    Sales,
    /// Research and development cost centers
    RAndD,
    /// Group / holding company level cost centers
    Corporate,
}

impl std::fmt::Display for CostCenterCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Administration => write!(f, "Administration"),
            Self::Production => write!(f, "Production"),
            Self::Sales => write!(f, "Sales"),
            Self::RAndD => write!(f, "R&D"),
            Self::Corporate => write!(f, "Corporate"),
        }
    }
}

/// A cost center node in the organizational cost hierarchy.
///
/// Cost centers are arranged in a two-level tree:
/// - **Level 1** (parent): Represents a department (e.g., Finance, Production).
///   These have `parent_id == None`.
/// - **Level 2** (child): Represents a sub-department or functional unit
///   (e.g., Accounts Payable within Finance).  These have `parent_id == Some(...)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostCenter {
    /// Unique cost center identifier (e.g., "CC-C001-FIN")
    pub id: String,

    /// Human-readable name (e.g., "Finance Department")
    pub name: String,

    /// Parent cost center ID for level-2 nodes; `None` for level-1 department nodes.
    pub parent_id: Option<String>,

    /// Company code this cost center belongs to.
    pub company_code: String,

    /// Employee ID of the person responsible for this cost center.
    pub responsible_person: Option<String>,

    /// Functional category of this cost center.
    pub category: CostCenterCategory,

    /// Hierarchy level (1 = department, 2 = sub-department).
    pub level: u8,

    /// Whether this cost center is currently active.
    pub is_active: bool,
}

impl CostCenter {
    /// Create a new level-1 (department) cost center.
    pub fn department(
        id: impl Into<String>,
        name: impl Into<String>,
        company_code: impl Into<String>,
        category: CostCenterCategory,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            parent_id: None,
            company_code: company_code.into(),
            responsible_person: None,
            category,
            level: 1,
            is_active: true,
        }
    }

    /// Create a new level-2 (sub-department) cost center.
    pub fn sub_department(
        id: impl Into<String>,
        name: impl Into<String>,
        parent_id: impl Into<String>,
        company_code: impl Into<String>,
        category: CostCenterCategory,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            parent_id: Some(parent_id.into()),
            company_code: company_code.into(),
            responsible_person: None,
            category,
            level: 2,
            is_active: true,
        }
    }
}
