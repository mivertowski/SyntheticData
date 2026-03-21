//! Cost center hierarchy generator.
//!
//! Generates a two-level cost center hierarchy (departments → sub-departments)
//! per company: typically 5 level-1 department nodes and 2-4 sub-department
//! nodes per department, resulting in 10-25 cost centers per company.

use datasynth_core::models::{CostCenter, CostCenterCategory};
use datasynth_core::utils::seeded_rng;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use tracing::debug;

/// Seed discriminator for cost center generator (avoids UUID collisions).
const SEED_DISCRIMINATOR: u64 = 0x4343_434e; // "CCCN"

/// Template for generating a department and its sub-departments.
struct DeptTemplate {
    code: &'static str,
    name: &'static str,
    category: CostCenterCategory,
    sub_depts: &'static [(&'static str, &'static str)],
}

const DEPT_TEMPLATES: &[DeptTemplate] = &[
    DeptTemplate {
        code: "FIN",
        name: "Finance",
        category: CostCenterCategory::Administration,
        sub_depts: &[
            ("AP", "Accounts Payable"),
            ("AR", "Accounts Receivable"),
            ("GL", "General Ledger"),
            ("TAX", "Tax"),
        ],
    },
    DeptTemplate {
        code: "PROD",
        name: "Production",
        category: CostCenterCategory::Production,
        sub_depts: &[
            ("ASSY", "Assembly"),
            ("QC", "Quality Control"),
            ("MAINT", "Maintenance"),
        ],
    },
    DeptTemplate {
        code: "SALES",
        name: "Sales & Marketing",
        category: CostCenterCategory::Sales,
        sub_depts: &[
            ("DOM", "Domestic Sales"),
            ("INTL", "International Sales"),
            ("MKT", "Marketing"),
        ],
    },
    DeptTemplate {
        code: "RD",
        name: "Research & Development",
        category: CostCenterCategory::RAndD,
        sub_depts: &[("RSCH", "Research"), ("DEV", "Development")],
    },
    DeptTemplate {
        code: "CORP",
        name: "Corporate",
        category: CostCenterCategory::Corporate,
        sub_depts: &[
            ("EXEC", "Executive"),
            ("HR", "Human Resources"),
            ("IT", "Information Technology"),
            ("LEGAL", "Legal"),
        ],
    },
];

/// Generator for cost center hierarchies.
pub struct CostCenterGenerator {
    rng: ChaCha8Rng,
}

impl CostCenterGenerator {
    /// Create a new cost center generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, SEED_DISCRIMINATOR),
        }
    }

    /// Generate all cost centers for a single company.
    ///
    /// Produces level-1 department nodes and level-2 sub-department nodes in a
    /// 2-level hierarchy.  Each department gets all of its configured
    /// sub-departments, and each node has a 20 % chance of being assigned a
    /// responsible person from `employee_ids` (if provided).
    pub fn generate_for_company(
        &mut self,
        company_code: &str,
        employee_ids: &[String],
    ) -> Vec<CostCenter> {
        let mut cost_centers: Vec<CostCenter> = Vec::with_capacity(25);

        for tmpl in DEPT_TEMPLATES {
            let dept_id = format!("CC-{}-{}", company_code, tmpl.code);

            // Level-1: department node
            let mut dept = CostCenter::department(
                dept_id.clone(),
                format!("{} — {}", company_code, tmpl.name),
                company_code,
                tmpl.category,
            );
            dept.responsible_person = self.pick_employee(employee_ids);
            cost_centers.push(dept);

            // Level-2: sub-department nodes
            for (sub_code, sub_name) in tmpl.sub_depts {
                let sub_id = format!("CC-{}-{}-{}", company_code, tmpl.code, sub_code);
                let mut sub = CostCenter::sub_department(
                    sub_id,
                    format!("{} / {}", tmpl.name, sub_name),
                    dept_id.clone(),
                    company_code,
                    tmpl.category,
                );
                sub.responsible_person = self.pick_employee(employee_ids);
                cost_centers.push(sub);
            }
        }

        debug!(
            company_code,
            count = cost_centers.len(),
            "Generated cost centers"
        );
        cost_centers
    }

    /// Randomly pick an employee ID (20 % chance of assignment).
    fn pick_employee(&mut self, employee_ids: &[String]) -> Option<String> {
        if employee_ids.is_empty() || self.rng.random::<f64>() > 0.20 {
            return None;
        }
        let idx = self.rng.random_range(0..employee_ids.len());
        Some(employee_ids[idx].clone())
    }
}
