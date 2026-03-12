//! Property serializer for `RiskAssessment` entities (entity type code 364).
//!
//! Reads fields directly from the [`RiskAssessment`] model — continuous scores
//! (`inherent_impact`, `inherent_likelihood`, etc.) are pre-computed by Task 1's
//! `recompute_continuous_scores()`.

use std::collections::HashMap;

use serde_json::Value;

use crate::traits::{PropertySerializer, SerializationContext};

/// Property serializer for risk assessments.
///
/// Handles entity type `"risk_assessment"` (code 364). Looks up the risk
/// in `ctx.ds_result.audit.risk_assessments` by matching `node_external_id`
/// to `risk.risk_ref`.
pub struct RiskPropertySerializer;

impl PropertySerializer for RiskPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "risk_assessment"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let risk = ctx
            .ds_result
            .audit
            .risk_assessments
            .iter()
            .find(|r| r.risk_ref == node_external_id)?;

        let mut props = HashMap::with_capacity(20);

        // Identity
        props.insert("riskRef".into(), Value::String(risk.risk_ref.clone()));
        props.insert("name".into(), Value::String(risk.risk_name.clone()));
        props.insert(
            "description".into(),
            Value::String(risk.description.clone()),
        );

        // Classification
        props.insert(
            "category".into(),
            serde_json::to_value(&risk.risk_category).unwrap_or(Value::Null),
        );
        props.insert(
            "accountOrProcess".into(),
            Value::String(risk.account_or_process.clone()),
        );
        if let Some(ref assertion) = risk.assertion {
            props.insert(
                "assertion".into(),
                serde_json::to_value(assertion).unwrap_or(Value::Null),
            );
        }

        // Lifecycle status
        props.insert(
            "status".into(),
            serde_json::to_value(&risk.status).unwrap_or(Value::Null),
        );

        // Risk scores (continuous, pre-computed by Task 1)
        props.insert(
            "inherentImpact".into(),
            serde_json::json!(risk.inherent_impact),
        );
        props.insert(
            "inherentLikelihood".into(),
            serde_json::json!(risk.inherent_likelihood),
        );
        props.insert(
            "residualImpact".into(),
            serde_json::json!(risk.residual_impact),
        );
        props.insert(
            "residualLikelihood".into(),
            serde_json::json!(risk.residual_likelihood),
        );
        props.insert("riskScore".into(), serde_json::json!(risk.risk_score));

        // Significance
        props.insert(
            "isSignificant".into(),
            Value::Bool(risk.is_significant_risk),
        );
        if let Some(ref rationale) = risk.significant_risk_rationale {
            props.insert(
                "significantRiskRationale".into(),
                Value::String(rationale.clone()),
            );
        }

        // Risk levels (enum)
        props.insert(
            "inherentRisk".into(),
            serde_json::to_value(&risk.inherent_risk).unwrap_or(Value::Null),
        );
        props.insert(
            "controlRisk".into(),
            serde_json::to_value(&risk.control_risk).unwrap_or(Value::Null),
        );
        props.insert(
            "riskOfMaterialMisstatement".into(),
            serde_json::to_value(&risk.risk_of_material_misstatement).unwrap_or(Value::Null),
        );

        // Owner / assessor
        if !risk.assessed_by.is_empty() {
            // Try to resolve the employee name from the context lookup
            let owner = ctx
                .employee_by_id
                .get(&risk.assessed_by)
                .cloned()
                .unwrap_or_else(|| risk.assessed_by.clone());
            props.insert("owner".into(), Value::String(owner));
        }

        // Control linkage counts
        props.insert(
            "mitigatingControlCount".into(),
            Value::Number(risk.mitigating_control_count.into()),
        );
        props.insert(
            "effectiveControlCount".into(),
            Value::Number(risk.effective_control_count.into()),
        );

        // Fraud risk factors
        if !risk.fraud_risk_factors.is_empty() {
            let factors: Vec<Value> = risk
                .fraud_risk_factors
                .iter()
                .map(|f| {
                    serde_json::json!({
                        "factorType": serde_json::to_value(&f.factor_type).unwrap_or(Value::Null),
                        "indicator": f.indicator,
                        "score": f.score,
                    })
                })
                .collect();
            props.insert("fraudRiskFactors".into(), Value::Array(factors));
        }

        // Related controls
        if !risk.related_controls.is_empty() {
            props.insert(
                "relatedControls".into(),
                Value::Array(
                    risk.related_controls
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
    fn entity_type_is_risk_assessment() {
        let s = RiskPropertySerializer;
        assert_eq!(s.entity_type(), "risk_assessment");
    }
}
