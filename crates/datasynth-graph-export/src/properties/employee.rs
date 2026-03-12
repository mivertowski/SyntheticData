//! Property serializer for `Employee` entities (entity type code 360).
//!
//! Reads fields directly from the [`Employee`] model in `master_data.employees`.

use std::collections::HashMap;

use serde_json::Value;

use crate::traits::{PropertySerializer, SerializationContext};

/// Property serializer for employees.
///
/// Handles entity type `"employee"` (code 360). Looks up the employee
/// in `ctx.ds_result.master_data.employees` by matching `node_external_id`
/// to `employee.employee_id`.
pub struct EmployeePropertySerializer;

impl PropertySerializer for EmployeePropertySerializer {
    fn entity_type(&self) -> &'static str {
        "employee"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let emp = ctx
            .ds_result
            .master_data
            .employees
            .iter()
            .find(|e| e.employee_id == node_external_id)?;

        let mut props = HashMap::with_capacity(14);

        // Identity
        props.insert(
            "employeeId".into(),
            Value::String(emp.employee_id.clone()),
        );
        props.insert(
            "fullName".into(),
            Value::String(emp.display_name.clone()),
        );
        props.insert(
            "firstName".into(),
            Value::String(emp.first_name.clone()),
        );
        props.insert(
            "lastName".into(),
            Value::String(emp.last_name.clone()),
        );
        props.insert("email".into(), Value::String(emp.email.clone()));

        // Role / classification
        props.insert(
            "role".into(),
            Value::String(format!("{:?}", emp.persona)),
        );
        props.insert(
            "jobLevel".into(),
            Value::String(format!("{:?}", emp.job_level)),
        );
        props.insert(
            "jobTitle".into(),
            Value::String(emp.job_title.clone()),
        );

        // Organization
        if let Some(ref dept) = emp.department_id {
            props.insert("department".into(), Value::String(dept.clone()));
        }
        if let Some(ref cc) = emp.cost_center {
            props.insert("costCenter".into(), Value::String(cc.clone()));
        }
        props.insert(
            "companyCode".into(),
            Value::String(emp.company_code.clone()),
        );
        if let Some(ref mgr) = emp.manager_id {
            props.insert("managerId".into(), Value::String(mgr.clone()));
        }

        // Status
        props.insert(
            "isActive".into(),
            Value::Bool(matches!(
                emp.status,
                datasynth_core::models::EmployeeStatus::Active
            )),
        );
        props.insert(
            "status".into(),
            Value::String(format!("{:?}", emp.status)),
        );

        Some(props)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn entity_type_is_employee() {
        let s = EmployeePropertySerializer;
        assert_eq!(s.entity_type(), "employee");
    }
}
