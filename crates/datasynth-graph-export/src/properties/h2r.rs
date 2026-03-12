//! Property serializers for Hire-to-Retire (H2R) entities.
//!
//! Covers: PayrollRun, TimeEntry, ExpenseReport.

use std::collections::HashMap;

use serde_json::Value;

use crate::traits::{PropertySerializer, SerializationContext};

// ──────────────────────────── Payroll Run ────────────────────────────

/// Property serializer for payroll runs (entity type code 600).
pub struct PayrollRunPropertySerializer;

impl PropertySerializer for PayrollRunPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "payroll_run"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let pr = ctx
            .ds_result
            .hr
            .payroll_runs
            .iter()
            .find(|p| p.payroll_id == node_external_id)?;

        let mut props = HashMap::with_capacity(10);

        props.insert(
            "payrollId".into(),
            Value::String(pr.payroll_id.clone()),
        );
        props.insert(
            "companyCode".into(),
            Value::String(pr.company_code.clone()),
        );
        props.insert(
            "payPeriodStart".into(),
            Value::String(pr.pay_period_start.format("%Y-%m-%d").to_string()),
        );
        props.insert(
            "payPeriodEnd".into(),
            Value::String(pr.pay_period_end.format("%Y-%m-%d").to_string()),
        );
        props.insert(
            "runDate".into(),
            Value::String(pr.run_date.format("%Y-%m-%d").to_string()),
        );
        props.insert(
            "status".into(),
            Value::String(format!("{:?}", pr.status)),
        );
        props.insert("totalGross".into(), serde_json::json!(pr.total_gross));
        props.insert("totalNet".into(), serde_json::json!(pr.total_net));
        props.insert(
            "totalDeductions".into(),
            serde_json::json!(pr.total_deductions),
        );
        props.insert(
            "employeeCount".into(),
            Value::Number(pr.employee_count.into()),
        );

        Some(props)
    }
}

// ──────────────────────────── Time Entry ─────────────────────────────

/// Property serializer for time entries (entity type code 601).
pub struct TimeEntryPropertySerializer;

impl PropertySerializer for TimeEntryPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "time_entry"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let te = ctx
            .ds_result
            .hr
            .time_entries
            .iter()
            .find(|t| t.entry_id == node_external_id)?;

        let mut props = HashMap::with_capacity(10);

        props.insert("entryId".into(), Value::String(te.entry_id.clone()));
        props.insert(
            "employeeId".into(),
            Value::String(te.employee_id.clone()),
        );
        props.insert(
            "date".into(),
            Value::String(te.date.format("%Y-%m-%d").to_string()),
        );
        props.insert(
            "hoursRegular".into(),
            serde_json::json!(te.hours_regular),
        );
        props.insert(
            "hoursOvertime".into(),
            serde_json::json!(te.hours_overtime),
        );
        props.insert("hoursPto".into(), serde_json::json!(te.hours_pto));
        props.insert("hoursSick".into(), serde_json::json!(te.hours_sick));
        props.insert(
            "status".into(),
            Value::String(format!("{:?}", te.approval_status)),
        );
        if let Some(ref project) = te.project_id {
            props.insert("projectId".into(), Value::String(project.clone()));
        }
        if let Some(ref cc) = te.cost_center {
            props.insert("costCenter".into(), Value::String(cc.clone()));
        }

        Some(props)
    }
}

// ──────────────────────────── Expense Report ────────────────────────

/// Property serializer for expense reports (entity type code 602).
pub struct ExpenseReportPropertySerializer;

impl PropertySerializer for ExpenseReportPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "expense_report"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let er = ctx
            .ds_result
            .hr
            .expense_reports
            .iter()
            .find(|e| e.report_id == node_external_id)?;

        let mut props = HashMap::with_capacity(10);

        props.insert("reportId".into(), Value::String(er.report_id.clone()));
        props.insert(
            "employeeId".into(),
            Value::String(er.employee_id.clone()),
        );
        props.insert(
            "submissionDate".into(),
            Value::String(er.submission_date.format("%Y-%m-%d").to_string()),
        );
        props.insert(
            "description".into(),
            Value::String(er.description.clone()),
        );
        props.insert(
            "status".into(),
            Value::String(format!("{:?}", er.status)),
        );
        props.insert("amount".into(), serde_json::json!(er.total_amount));
        props.insert("currency".into(), Value::String(er.currency.clone()));
        props.insert(
            "lineCount".into(),
            Value::Number(er.line_items.len().into()),
        );
        if let Some(ref approver) = er.approved_by {
            let name = ctx
                .employee_by_id
                .get(approver)
                .cloned()
                .unwrap_or_else(|| approver.clone());
            props.insert("approvedBy".into(), Value::String(name));
        }
        if let Some(ref cc) = er.cost_center {
            props.insert("costCenter".into(), Value::String(cc.clone()));
        }

        Some(props)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn entity_types_are_correct() {
        assert_eq!(PayrollRunPropertySerializer.entity_type(), "payroll_run");
        assert_eq!(TimeEntryPropertySerializer.entity_type(), "time_entry");
        assert_eq!(
            ExpenseReportPropertySerializer.entity_type(),
            "expense_report"
        );
    }
}
