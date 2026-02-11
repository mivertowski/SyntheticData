//! Department and organizational structure models.
//!
//! Provides department definitions with associated cost centers,
//! business processes, and typical user personas.

use crate::models::{BusinessProcess, UserPersona};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Department definition for organizational structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Department {
    /// Department code (e.g., "FIN", "AP", "AR")
    pub code: String,

    /// Department name (e.g., "Finance", "Accounts Payable")
    pub name: String,

    /// Parent department code (for hierarchy)
    pub parent_code: Option<String>,

    /// Associated cost center
    pub cost_center: String,

    /// Typical user personas in this department
    pub typical_personas: Vec<UserPersona>,

    /// Primary business processes handled
    pub primary_processes: Vec<BusinessProcess>,

    /// Standard headcount for this department
    pub standard_headcount: DepartmentHeadcount,

    /// Is this department active
    pub is_active: bool,
}

impl Department {
    /// Create a new department.
    pub fn new(code: &str, name: &str, cost_center: &str) -> Self {
        Self {
            code: code.to_string(),
            name: name.to_string(),
            parent_code: None,
            cost_center: cost_center.to_string(),
            typical_personas: Vec::new(),
            primary_processes: Vec::new(),
            standard_headcount: DepartmentHeadcount::default(),
            is_active: true,
        }
    }

    /// Set parent department.
    pub fn with_parent(mut self, parent_code: &str) -> Self {
        self.parent_code = Some(parent_code.to_string());
        self
    }

    /// Add typical personas.
    pub fn with_personas(mut self, personas: Vec<UserPersona>) -> Self {
        self.typical_personas = personas;
        self
    }

    /// Add primary business processes.
    pub fn with_processes(mut self, processes: Vec<BusinessProcess>) -> Self {
        self.primary_processes = processes;
        self
    }

    /// Set headcount.
    pub fn with_headcount(mut self, headcount: DepartmentHeadcount) -> Self {
        self.standard_headcount = headcount;
        self
    }

    /// Check if this department handles a specific business process.
    pub fn handles_process(&self, process: BusinessProcess) -> bool {
        self.primary_processes.contains(&process)
    }

    /// Check if a persona is typical for this department.
    pub fn is_typical_persona(&self, persona: UserPersona) -> bool {
        self.typical_personas.contains(&persona)
    }
}

/// Headcount configuration for a department.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartmentHeadcount {
    /// Number of junior accountants
    pub junior_accountant: usize,
    /// Number of senior accountants
    pub senior_accountant: usize,
    /// Number of controllers
    pub controller: usize,
    /// Number of managers
    pub manager: usize,
    /// Number of executives (usually 0 or 1)
    pub executive: usize,
    /// Number of automated systems/batch jobs
    pub automated_system: usize,
}

impl Default for DepartmentHeadcount {
    fn default() -> Self {
        Self {
            junior_accountant: 2,
            senior_accountant: 1,
            controller: 0,
            manager: 0,
            executive: 0,
            automated_system: 1,
        }
    }
}

impl DepartmentHeadcount {
    /// Create an empty headcount.
    pub fn empty() -> Self {
        Self {
            junior_accountant: 0,
            senior_accountant: 0,
            controller: 0,
            manager: 0,
            executive: 0,
            automated_system: 0,
        }
    }

    /// Total headcount.
    pub fn total(&self) -> usize {
        self.junior_accountant
            + self.senior_accountant
            + self.controller
            + self.manager
            + self.executive
            + self.automated_system
    }

    /// Apply a multiplier to all counts.
    pub fn scaled(&self, multiplier: f64) -> Self {
        Self {
            junior_accountant: (self.junior_accountant as f64 * multiplier).round() as usize,
            senior_accountant: (self.senior_accountant as f64 * multiplier).round() as usize,
            controller: (self.controller as f64 * multiplier).round() as usize,
            manager: (self.manager as f64 * multiplier).round() as usize,
            executive: (self.executive as f64 * multiplier).round() as usize,
            automated_system: (self.automated_system as f64 * multiplier).round() as usize,
        }
    }
}

/// Organization structure containing all departments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationStructure {
    /// Company code this structure belongs to
    pub company_code: String,

    /// All departments in the organization
    pub departments: Vec<Department>,

    /// Index by department code for quick lookup
    #[serde(skip)]
    department_index: HashMap<String, usize>,
}

impl OrganizationStructure {
    /// Create a new empty organization structure.
    pub fn new(company_code: &str) -> Self {
        Self {
            company_code: company_code.to_string(),
            departments: Vec::new(),
            department_index: HashMap::new(),
        }
    }

    /// Add a department to the structure.
    pub fn add_department(&mut self, department: Department) {
        let idx = self.departments.len();
        self.department_index.insert(department.code.clone(), idx);
        self.departments.push(department);
    }

    /// Get a department by code.
    pub fn get_department(&self, code: &str) -> Option<&Department> {
        self.department_index
            .get(code)
            .map(|&idx| &self.departments[idx])
    }

    /// Get departments that handle a specific business process.
    pub fn get_departments_for_process(&self, process: BusinessProcess) -> Vec<&Department> {
        self.departments
            .iter()
            .filter(|d| d.handles_process(process))
            .collect()
    }

    /// Get departments with a specific persona type.
    pub fn get_departments_for_persona(&self, persona: UserPersona) -> Vec<&Department> {
        self.departments
            .iter()
            .filter(|d| d.is_typical_persona(persona))
            .collect()
    }

    /// Rebuild the index (call after deserialization).
    pub fn rebuild_index(&mut self) {
        self.department_index.clear();
        for (idx, dept) in self.departments.iter().enumerate() {
            self.department_index.insert(dept.code.clone(), idx);
        }
    }

    /// Get total headcount across all departments.
    pub fn total_headcount(&self) -> usize {
        self.departments
            .iter()
            .map(|d| d.standard_headcount.total())
            .sum()
    }

    /// Generate a standard organization structure.
    pub fn standard(company_code: &str) -> Self {
        let mut org = Self::new(company_code);

        // Finance department (parent)
        org.add_department(
            Department::new("FIN", "Finance", "1000")
                .with_personas(vec![
                    UserPersona::Controller,
                    UserPersona::SeniorAccountant,
                    UserPersona::Manager,
                    UserPersona::Executive,
                ])
                .with_processes(vec![BusinessProcess::R2R])
                .with_headcount(DepartmentHeadcount {
                    junior_accountant: 0,
                    senior_accountant: 2,
                    controller: 2,
                    manager: 1,
                    executive: 1,
                    automated_system: 2,
                }),
        );

        // Accounts Payable
        org.add_department(
            Department::new("AP", "Accounts Payable", "1100")
                .with_parent("FIN")
                .with_personas(vec![
                    UserPersona::JuniorAccountant,
                    UserPersona::SeniorAccountant,
                ])
                .with_processes(vec![BusinessProcess::P2P])
                .with_headcount(DepartmentHeadcount {
                    junior_accountant: 5,
                    senior_accountant: 2,
                    controller: 0,
                    manager: 1,
                    executive: 0,
                    automated_system: 5,
                }),
        );

        // Accounts Receivable
        org.add_department(
            Department::new("AR", "Accounts Receivable", "1200")
                .with_parent("FIN")
                .with_personas(vec![
                    UserPersona::JuniorAccountant,
                    UserPersona::SeniorAccountant,
                ])
                .with_processes(vec![BusinessProcess::O2C])
                .with_headcount(DepartmentHeadcount {
                    junior_accountant: 4,
                    senior_accountant: 2,
                    controller: 0,
                    manager: 1,
                    executive: 0,
                    automated_system: 5,
                }),
        );

        // General Ledger
        org.add_department(
            Department::new("GL", "General Ledger", "1300")
                .with_parent("FIN")
                .with_personas(vec![UserPersona::SeniorAccountant, UserPersona::Controller])
                .with_processes(vec![BusinessProcess::R2R])
                .with_headcount(DepartmentHeadcount {
                    junior_accountant: 2,
                    senior_accountant: 3,
                    controller: 1,
                    manager: 0,
                    executive: 0,
                    automated_system: 3,
                }),
        );

        // Payroll / HR Accounting
        org.add_department(
            Department::new("HR", "Human Resources", "2000")
                .with_personas(vec![
                    UserPersona::JuniorAccountant,
                    UserPersona::SeniorAccountant,
                ])
                .with_processes(vec![BusinessProcess::H2R])
                .with_headcount(DepartmentHeadcount {
                    junior_accountant: 2,
                    senior_accountant: 1,
                    controller: 0,
                    manager: 1,
                    executive: 0,
                    automated_system: 2,
                }),
        );

        // Fixed Assets
        org.add_department(
            Department::new("FA", "Fixed Assets", "1400")
                .with_parent("FIN")
                .with_personas(vec![
                    UserPersona::JuniorAccountant,
                    UserPersona::SeniorAccountant,
                ])
                .with_processes(vec![BusinessProcess::A2R])
                .with_headcount(DepartmentHeadcount {
                    junior_accountant: 1,
                    senior_accountant: 1,
                    controller: 0,
                    manager: 0,
                    executive: 0,
                    automated_system: 2,
                }),
        );

        // Treasury
        org.add_department(
            Department::new("TRE", "Treasury", "1500")
                .with_parent("FIN")
                .with_personas(vec![
                    UserPersona::SeniorAccountant,
                    UserPersona::Controller,
                    UserPersona::Manager,
                ])
                .with_processes(vec![BusinessProcess::Treasury])
                .with_headcount(DepartmentHeadcount {
                    junior_accountant: 0,
                    senior_accountant: 2,
                    controller: 1,
                    manager: 1,
                    executive: 0,
                    automated_system: 2,
                }),
        );

        // Tax
        org.add_department(
            Department::new("TAX", "Tax", "1600")
                .with_parent("FIN")
                .with_personas(vec![UserPersona::SeniorAccountant, UserPersona::Controller])
                .with_processes(vec![BusinessProcess::Tax])
                .with_headcount(DepartmentHeadcount {
                    junior_accountant: 1,
                    senior_accountant: 2,
                    controller: 1,
                    manager: 0,
                    executive: 0,
                    automated_system: 1,
                }),
        );

        // Procurement
        org.add_department(
            Department::new("PROC", "Procurement", "3000")
                .with_personas(vec![UserPersona::SeniorAccountant, UserPersona::Manager])
                .with_processes(vec![BusinessProcess::P2P])
                .with_headcount(DepartmentHeadcount {
                    junior_accountant: 2,
                    senior_accountant: 2,
                    controller: 0,
                    manager: 1,
                    executive: 0,
                    automated_system: 3,
                }),
        );

        // IT (batch jobs)
        org.add_department(
            Department::new("IT", "Information Technology", "4000")
                .with_personas(vec![UserPersona::AutomatedSystem])
                .with_processes(vec![BusinessProcess::R2R])
                .with_headcount(DepartmentHeadcount {
                    junior_accountant: 0,
                    senior_accountant: 0,
                    controller: 0,
                    manager: 0,
                    executive: 0,
                    automated_system: 10,
                }),
        );

        org
    }

    /// Generate a minimal organization structure for small companies.
    pub fn minimal(company_code: &str) -> Self {
        let mut org = Self::new(company_code);

        org.add_department(
            Department::new("FIN", "Finance", "1000")
                .with_personas(vec![
                    UserPersona::JuniorAccountant,
                    UserPersona::SeniorAccountant,
                    UserPersona::Controller,
                    UserPersona::Manager,
                ])
                .with_processes(vec![
                    BusinessProcess::O2C,
                    BusinessProcess::P2P,
                    BusinessProcess::R2R,
                    BusinessProcess::H2R,
                    BusinessProcess::A2R,
                ])
                .with_headcount(DepartmentHeadcount {
                    junior_accountant: 3,
                    senior_accountant: 2,
                    controller: 1,
                    manager: 1,
                    executive: 0,
                    automated_system: 5,
                }),
        );

        org
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_department_creation() {
        let dept = Department::new("FIN", "Finance", "1000")
            .with_personas(vec![UserPersona::Controller])
            .with_processes(vec![BusinessProcess::R2R]);

        assert_eq!(dept.code, "FIN");
        assert_eq!(dept.name, "Finance");
        assert!(dept.handles_process(BusinessProcess::R2R));
        assert!(!dept.handles_process(BusinessProcess::P2P));
        assert!(dept.is_typical_persona(UserPersona::Controller));
    }

    #[test]
    fn test_standard_organization() {
        let org = OrganizationStructure::standard("1000");

        assert!(!org.departments.is_empty());
        assert!(org.get_department("FIN").is_some());
        assert!(org.get_department("AP").is_some());
        assert!(org.get_department("AR").is_some());

        // Check process mapping
        let p2p_depts = org.get_departments_for_process(BusinessProcess::P2P);
        assert!(!p2p_depts.is_empty());

        // Check total headcount
        assert!(org.total_headcount() > 0);
    }

    #[test]
    fn test_headcount_scaling() {
        let headcount = DepartmentHeadcount {
            junior_accountant: 10,
            senior_accountant: 5,
            controller: 2,
            manager: 1,
            executive: 0,
            automated_system: 3,
        };

        let scaled = headcount.scaled(0.5);
        assert_eq!(scaled.junior_accountant, 5);
        assert_eq!(scaled.senior_accountant, 3); // 2.5 rounds to 3
        assert_eq!(scaled.controller, 1);
    }

    #[test]
    fn test_minimal_organization() {
        let org = OrganizationStructure::minimal("1000");

        assert_eq!(org.departments.len(), 1);
        let fin = org.get_department("FIN").unwrap();
        assert!(fin.handles_process(BusinessProcess::O2C));
        assert!(fin.handles_process(BusinessProcess::P2P));
    }
}
