//! Property serializer for `InternalControl` entities (entity type code 503).
//!
//! Reads fields directly from the [`InternalControl`] model — no re-derivation
//! from `maturity_level` or `risk_level` needed since Task 2 enriched the model
//! with pre-computed fields (`effectiveness`, `test_result`, `owner_name`, etc.).

use std::collections::HashMap;

use serde_json::Value;

use crate::traits::{PropertySerializer, SerializationContext};

/// Property serializer for internal controls.
///
/// Handles entity type `"internal_control"` (code 503). Looks up the control
/// in `ctx.ds_result.internal_controls` by matching `node_external_id` to
/// `control.control_id`.
pub struct ControlPropertySerializer;

impl PropertySerializer for ControlPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "internal_control"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let control = ctx
            .ds_result
            .internal_controls
            .iter()
            .find(|c| c.control_id == node_external_id)?;

        let mut props = HashMap::with_capacity(20);

        // Identity
        props.insert(
            "controlId".into(),
            Value::String(control.control_id.clone()),
        );
        props.insert("name".into(), Value::String(control.control_name.clone()));
        props.insert(
            "description".into(),
            Value::String(control.description.clone()),
        );

        // Classification
        props.insert(
            "controlType".into(),
            Value::String(control.control_type.to_string()),
        );
        props.insert(
            "category".into(),
            serde_json::to_value(&control.coso_component).unwrap_or(Value::Null),
        );
        props.insert("isKeyControl".into(), Value::Bool(control.is_key_control));
        props.insert(
            "automated".into(),
            Value::Bool(matches!(
                control.owner_role,
                datasynth_core::models::UserPersona::AutomatedSystem
            )),
        );

        // SOX / COSO
        props.insert(
            "soxAssertion".into(),
            Value::String(control.sox_assertion.to_string()),
        );
        props.insert(
            "cosoComponent".into(),
            serde_json::to_value(&control.coso_component).unwrap_or(Value::Null),
        );
        props.insert(
            "controlScope".into(),
            serde_json::to_value(&control.control_scope).unwrap_or(Value::Null),
        );
        props.insert("objective".into(), Value::String(control.objective.clone()));

        // Operational
        props.insert(
            "frequency".into(),
            Value::String(control.frequency.to_string()),
        );
        props.insert(
            "owner".into(),
            if control.owner_name.is_empty() {
                serde_json::to_value(&control.owner_role).unwrap_or(Value::Null)
            } else {
                Value::String(control.owner_name.clone())
            },
        );
        props.insert(
            "riskLevel".into(),
            Value::String(control.risk_level.to_string()),
        );

        // Test history + effectiveness (pre-computed by Task 2's derive_from_maturity)
        props.insert(
            "effectiveness".into(),
            Value::String(control.effectiveness.to_string()),
        );
        props.insert("testCount".into(), Value::Number(control.test_count.into()));
        props.insert(
            "testResult".into(),
            Value::String(control.test_result.to_string()),
        );
        if let Some(date) = control.last_tested_date {
            props.insert(
                "lastTestedDate".into(),
                Value::String(date.format("%Y-%m-%d").to_string()),
            );
        }

        // Risk linkage
        if !control.mitigates_risk_ids.is_empty() {
            props.insert(
                "linkedRiskIds".into(),
                Value::Array(
                    control
                        .mitigates_risk_ids
                        .iter()
                        .map(|id| Value::String(id.clone()))
                        .collect(),
                ),
            );
        }

        // Account class coverage
        if !control.covers_account_classes.is_empty() {
            props.insert(
                "coversAccountClasses".into(),
                Value::Array(
                    control
                        .covers_account_classes
                        .iter()
                        .map(|c| Value::String(c.clone()))
                        .collect(),
                ),
            );
        }

        Some(props)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn entity_type_is_internal_control() {
        let s = ControlPropertySerializer;
        assert_eq!(s.entity_type(), "internal_control");
    }
}
