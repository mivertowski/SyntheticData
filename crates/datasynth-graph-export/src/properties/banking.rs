//! Property serializers for Banking KYC/AML entities.
//!
//! Covers: BankingCustomer, BankAccount, BankTransaction.

use std::collections::HashMap;

use serde_json::Value;

use crate::traits::{PropertySerializer, SerializationContext};

// ──────────────────────────── Banking Customer ──────────────────────

/// Property serializer for banking customers (entity type code 400).
pub struct BankingCustomerPropertySerializer;

impl PropertySerializer for BankingCustomerPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "banking_customer"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let cust = ctx
            .ds_result
            .banking
            .customers
            .iter()
            .find(|c| c.customer_id.to_string() == node_external_id)?;

        let mut props = HashMap::with_capacity(10);

        props.insert(
            "customerId".into(),
            Value::String(cust.customer_id.to_string()),
        );
        props.insert(
            "customerType".into(),
            serde_json::to_value(&cust.customer_type).unwrap_or(Value::Null),
        );
        props.insert(
            "name".into(),
            Value::String(cust.name.display_name().to_string()),
        );
        props.insert(
            "residenceCountry".into(),
            Value::String(cust.residence_country.clone()),
        );
        props.insert(
            "onboardingDate".into(),
            Value::String(cust.onboarding_date.format("%Y-%m-%d").to_string()),
        );
        props.insert(
            "riskTier".into(),
            serde_json::to_value(&cust.risk_tier).unwrap_or(Value::Null),
        );
        props.insert(
            "accountCount".into(),
            Value::Number(cust.account_ids.len().into()),
        );
        props.insert(
            "status".into(),
            serde_json::to_value(&cust.status).unwrap_or(Value::Null),
        );

        Some(props)
    }
}

// ──────────────────────────── Bank Account ──────────────────────────

/// Property serializer for bank accounts (entity type code 401).
pub struct BankAccountPropertySerializer;

impl PropertySerializer for BankAccountPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "bank_account"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let acct = ctx
            .ds_result
            .banking
            .accounts
            .iter()
            .find(|a| a.account_id.to_string() == node_external_id)?;

        let mut props = HashMap::with_capacity(10);

        props.insert(
            "accountId".into(),
            Value::String(acct.account_id.to_string()),
        );
        props.insert(
            "accountNumber".into(),
            Value::String(acct.account_number.clone()),
        );
        props.insert(
            "accountType".into(),
            serde_json::to_value(&acct.account_type).unwrap_or(Value::Null),
        );
        props.insert(
            "primaryOwnerId".into(),
            Value::String(acct.primary_owner_id.to_string()),
        );
        props.insert(
            "status".into(),
            serde_json::to_value(&acct.status).unwrap_or(Value::Null),
        );
        props.insert("currency".into(), Value::String(acct.currency.clone()));
        props.insert(
            "currentBalance".into(),
            serde_json::json!(acct.current_balance),
        );
        props.insert(
            "availableBalance".into(),
            serde_json::json!(acct.available_balance),
        );
        props.insert(
            "openingDate".into(),
            Value::String(acct.opening_date.format("%Y-%m-%d").to_string()),
        );
        if let Some(ref iban) = acct.iban {
            props.insert("iban".into(), Value::String(iban.clone()));
        }

        Some(props)
    }
}

// ──────────────────────────── Bank Transaction ──────────────────────

/// Property serializer for bank transactions (entity type code 402).
///
/// Note: Banking transactions can number in the millions, so this serializer
/// does a linear scan. For large datasets, consider pre-building an index.
pub struct BankTransactionPropertySerializer;

impl PropertySerializer for BankTransactionPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "bank_transaction"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let txn = ctx
            .ds_result
            .banking
            .transactions
            .iter()
            .find(|t| t.transaction_id.to_string() == node_external_id)?;

        let mut props = HashMap::with_capacity(12);

        props.insert(
            "transactionId".into(),
            Value::String(txn.transaction_id.to_string()),
        );
        props.insert(
            "accountId".into(),
            Value::String(txn.account_id.to_string()),
        );
        props.insert("amount".into(), serde_json::json!(txn.amount));
        props.insert("currency".into(), Value::String(txn.currency.clone()));
        props.insert(
            "direction".into(),
            serde_json::to_value(&txn.direction).unwrap_or(Value::Null),
        );
        props.insert(
            "channel".into(),
            serde_json::to_value(&txn.channel).unwrap_or(Value::Null),
        );
        props.insert(
            "category".into(),
            serde_json::to_value(&txn.category).unwrap_or(Value::Null),
        );
        props.insert("reference".into(), Value::String(txn.reference.clone()));
        props.insert(
            "status".into(),
            serde_json::to_value(&txn.status).unwrap_or(Value::Null),
        );
        props.insert(
            "isAnomalous".into(),
            Value::Bool(txn.is_suspicious),
        );
        props.insert(
            "isSuspicious".into(),
            Value::Bool(txn.is_suspicious),
        );
        props.insert(
            "timestamp".into(),
            Value::String(txn.timestamp_booked.to_rfc3339()),
        );

        Some(props)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn entity_types_are_correct() {
        assert_eq!(
            BankingCustomerPropertySerializer.entity_type(),
            "banking_customer"
        );
        assert_eq!(
            BankAccountPropertySerializer.entity_type(),
            "bank_account"
        );
        assert_eq!(
            BankTransactionPropertySerializer.entity_type(),
            "bank_transaction"
        );
    }
}
