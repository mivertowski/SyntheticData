//! Hire-to-Retire (H2R) edge synthesizer.
//!
//! Produces edges linking time entries, expense reports, and payroll
//! to employees.
//!
//! ## Edge Types Produced
//!
//! | Code | Name              | Direction                       |
//! |------|-------------------|---------------------------------|
//! |  90  | TIME_EMPLOYEE     | time_entry -> employee          |
//! |  91  | EXPENSE_EMPLOYEE  | expense_report -> employee      |
//! |  92  | EXPENSE_APPROVER  | expense_report -> approving_emp |
//! |  93  | PAYROLL_EMPLOYEE  | payroll_line -> employee        |

use std::collections::HashMap;

use tracing::debug;

use crate::error::ExportError;
use crate::traits::EdgeSynthesisContext;
use crate::types::ExportEdge;

/// Edge type codes produced by this synthesizer.
const TIME_EMPLOYEE: u32 = 90;
const EXPENSE_EMPLOYEE: u32 = 91;
const EXPENSE_APPROVER: u32 = 92;
const PAYROLL_EMPLOYEE: u32 = 93;

/// Synthesizes HR/people edges linking time, expense, and payroll to employees.
pub struct H2REdgeSynthesizer;

impl crate::traits::EdgeSynthesizer for H2REdgeSynthesizer {
    fn name(&self) -> &'static str {
        "h2r"
    }

    fn synthesize(
        &self,
        ctx: &mut EdgeSynthesisContext<'_>,
    ) -> Result<Vec<ExportEdge>, ExportError> {
        let mut edges = Vec::new();

        edges.extend(self.synthesize_time_employee(ctx));
        edges.extend(self.synthesize_expense_employee(ctx));
        edges.extend(self.synthesize_expense_approver(ctx));
        edges.extend(self.synthesize_payroll_employee(ctx));

        debug!("H2REdgeSynthesizer produced {} total edges", edges.len());
        Ok(edges)
    }
}

impl H2REdgeSynthesizer {
    /// TIME_EMPLOYEE (code 90): time_entry -> employee.
    fn synthesize_time_employee(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        let time_entries = &ctx.ds_result.hr.time_entries;
        let mut edges = Vec::new();

        for te in time_entries {
            let Some(te_id) = ctx.id_map.get(&te.entry_id) else {
                continue;
            };
            let Some(emp_id) = ctx.id_map.get(&te.employee_id) else {
                continue;
            };

            edges.push(ExportEdge {
                source: te_id,
                target: emp_id,
                edge_type: TIME_EMPLOYEE,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        debug!("TIME_EMPLOYEE: {} edges", edges.len());
        edges
    }

    /// EXPENSE_EMPLOYEE (code 91): expense_report -> employee.
    fn synthesize_expense_employee(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        let expense_reports = &ctx.ds_result.hr.expense_reports;
        let mut edges = Vec::new();

        for er in expense_reports {
            let Some(er_id) = ctx.id_map.get(&er.report_id) else {
                continue;
            };
            let Some(emp_id) = ctx.id_map.get(&er.employee_id) else {
                continue;
            };

            edges.push(ExportEdge {
                source: er_id,
                target: emp_id,
                edge_type: EXPENSE_EMPLOYEE,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        debug!("EXPENSE_EMPLOYEE: {} edges", edges.len());
        edges
    }

    /// EXPENSE_APPROVER (code 92): expense_report -> approving_employee.
    fn synthesize_expense_approver(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        if !ctx.config.edge_synthesis.people_edges {
            return Vec::new();
        }

        let expense_reports = &ctx.ds_result.hr.expense_reports;
        let mut edges = Vec::new();

        for er in expense_reports {
            let Some(ref approver_ref) = er.approved_by else {
                continue;
            };
            let Some(er_id) = ctx.id_map.get(&er.report_id) else {
                continue;
            };
            let Some(approver_id) = ctx.id_map.get(approver_ref) else {
                continue;
            };

            edges.push(ExportEdge {
                source: er_id,
                target: approver_id,
                edge_type: EXPENSE_APPROVER,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        debug!("EXPENSE_APPROVER: {} edges", edges.len());
        edges
    }

    /// PAYROLL_EMPLOYEE (code 93): payroll_line -> employee.
    ///
    /// Uses PayrollLineItem which has an employee_id FK. Each line item
    /// represents one employee's pay in a payroll run.
    fn synthesize_payroll_employee(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        let payroll_lines = &ctx.ds_result.hr.payroll_line_items;
        let mut edges = Vec::new();

        for pli in payroll_lines {
            let Some(pli_id) = ctx.id_map.get(&pli.line_id) else {
                // Payroll lines may not be exported as individual nodes;
                // fall back to linking the payroll run to the employee.
                let Some(pr_id) = ctx.id_map.get(&pli.payroll_id) else {
                    continue;
                };
                let Some(emp_id) = ctx.id_map.get(&pli.employee_id) else {
                    continue;
                };
                edges.push(ExportEdge {
                    source: pr_id,
                    target: emp_id,
                    edge_type: PAYROLL_EMPLOYEE,
                    weight: 1.0,
                    properties: HashMap::new(),
                });
                continue;
            };
            let Some(emp_id) = ctx.id_map.get(&pli.employee_id) else {
                continue;
            };

            edges.push(ExportEdge {
                source: pli_id,
                target: emp_id,
                edge_type: PAYROLL_EMPLOYEE,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        debug!("PAYROLL_EMPLOYEE: {} edges", edges.len());
        edges
    }
}
