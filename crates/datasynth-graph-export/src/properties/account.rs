//! Property serializer for `GLAccount` entities (entity type code 301).
//!
//! Reads fields directly from the [`GLAccount`] model in `chart_of_accounts.accounts`.

use std::collections::HashMap;

use serde_json::Value;

use crate::traits::{PropertySerializer, SerializationContext};

/// Property serializer for GL accounts.
///
/// Handles entity type `"gl_account"` (code 301). Looks up the account
/// in `ctx.ds_result.chart_of_accounts.accounts` by matching `node_external_id`
/// to `account.account_number`.
pub struct AccountPropertySerializer;

impl PropertySerializer for AccountPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "gl_account"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let account = ctx
            .ds_result
            .chart_of_accounts
            .accounts
            .iter()
            .find(|a| a.account_number == node_external_id)?;

        let mut props = HashMap::with_capacity(14);

        // Identity
        props.insert(
            "accountNumber".into(),
            Value::String(account.account_number.clone()),
        );
        props.insert(
            "accountName".into(),
            Value::String(account.short_description.clone()),
        );
        props.insert(
            "description".into(),
            Value::String(account.long_description.clone()),
        );

        // Classification
        props.insert(
            "accountType".into(),
            Value::String(format!("{:?}", account.account_type)),
        );
        props.insert(
            "subType".into(),
            Value::String(format!("{:?}", account.sub_type)),
        );
        props.insert(
            "accountClass".into(),
            Value::String(account.account_class.clone()),
        );
        props.insert(
            "accountGroup".into(),
            Value::String(account.account_group.clone()),
        );

        // Balance
        props.insert(
            "normalBalance".into(),
            Value::String(if account.normal_debit_balance {
                "Debit".into()
            } else {
                "Credit".into()
            }),
        );

        // Opening balance from context
        if let Some(&balance) = ctx.opening_balances.get(node_external_id) {
            props.insert("openingBalance".into(), serde_json::json!(balance));
        }

        // Flags
        props.insert(
            "isActive".into(),
            Value::Bool(account.is_postable && !account.is_blocked),
        );
        props.insert(
            "isControlAccount".into(),
            Value::Bool(account.is_control_account),
        );
        props.insert(
            "isSuspenseAccount".into(),
            Value::Bool(account.is_suspense_account),
        );
        props.insert(
            "hierarchyLevel".into(),
            Value::Number(account.hierarchy_level.into()),
        );

        if let Some(ref parent) = account.parent_account {
            props.insert("parentAccount".into(), Value::String(parent.clone()));
        }

        Some(props)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn entity_type_is_gl_account() {
        let s = AccountPropertySerializer;
        assert_eq!(s.entity_type(), "gl_account");
    }
}
