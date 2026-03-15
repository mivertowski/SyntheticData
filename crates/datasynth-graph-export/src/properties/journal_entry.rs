//! Property serializer for `JournalEntry` entities (entity type code 300).
//!
//! Reads fields from the [`JournalEntry`] model's header and computes totals
//! from line items.

use std::collections::HashMap;

use serde_json::Value;

use crate::traits::{PropertySerializer, SerializationContext};

/// Property serializer for journal entries.
///
/// Handles entity type `"journal_entry"` (code 300). Looks up the entry
/// in `ctx.ds_result.journal_entries` by matching `node_external_id` to
/// `entry.header.document_id` (UUID string).
pub struct JournalEntryPropertySerializer;

impl PropertySerializer for JournalEntryPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "journal_entry"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let entry = ctx
            .ds_result
            .journal_entries
            .iter()
            .find(|je| je.header.document_id.to_string() == node_external_id)?;

        let mut props = HashMap::with_capacity(16);

        // Identity
        props.insert(
            "documentId".into(),
            Value::String(entry.header.document_id.to_string()),
        );
        props.insert(
            "postingDate".into(),
            Value::String(entry.header.posting_date.format("%Y-%m-%d").to_string()),
        );
        props.insert(
            "companyCode".into(),
            Value::String(entry.header.company_code.clone()),
        );
        props.insert(
            "currency".into(),
            Value::String(entry.header.currency.clone()),
        );

        // Totals computed from lines
        let total_debit = entry.total_debit();
        let total_credit = entry.total_credit();
        props.insert("totalDebit".into(), serde_json::json!(total_debit));
        props.insert("totalCredit".into(), serde_json::json!(total_credit));
        props.insert("lineCount".into(), Value::Number(entry.lines.len().into()));

        // Reference and description
        if let Some(ref reference) = entry.header.reference {
            props.insert("reference".into(), Value::String(reference.clone()));
        }
        if let Some(ref text) = entry.header.header_text {
            props.insert("headerText".into(), Value::String(text.clone()));
        }

        // Source document
        if let Some(ref doc_ref) = entry.header.source_document {
            props.insert(
                "sourceDocument".into(),
                Value::String(format!("{doc_ref:?}")),
            );
        }

        // Approval
        if let Some(ref approver) = entry.header.approved_by {
            let name = ctx
                .employee_by_id
                .get(approver)
                .cloned()
                .unwrap_or_else(|| approver.clone());
            props.insert("approvedBy".into(), Value::String(name));
        }

        // SOD / compliance flags
        props.insert(
            "sodViolation".into(),
            Value::Bool(entry.header.sod_violation),
        );
        props.insert("soxRelevant".into(), Value::Bool(entry.header.sox_relevant));
        props.insert("isAnomaly".into(), Value::Bool(entry.header.is_anomaly));
        props.insert("isFraud".into(), Value::Bool(entry.header.is_fraud));

        // Document type
        props.insert(
            "documentType".into(),
            Value::String(entry.header.document_type.clone()),
        );
        props.insert(
            "source".into(),
            Value::String(format!("{:?}", entry.header.source)),
        );

        Some(props)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn entity_type_is_journal_entry() {
        let s = JournalEntryPropertySerializer;
        assert_eq!(s.entity_type(), "journal_entry");
    }
}
