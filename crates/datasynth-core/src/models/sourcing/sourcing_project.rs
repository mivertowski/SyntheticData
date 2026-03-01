//! Sourcing project models for strategic procurement initiatives.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::super::graph_properties::{GraphPropertyValue, ToNodeProperties};

/// Type of sourcing project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SourcingProjectType {
    /// New category sourcing
    #[default]
    NewSourcing,
    /// Contract renewal/renegotiation
    Renewal,
    /// Supplier consolidation
    Consolidation,
    /// Emergency sourcing
    Emergency,
    /// Strategic partnership
    StrategicPartnership,
}

/// Status of a sourcing project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SourcingProjectStatus {
    /// Project created, not yet started
    #[default]
    Draft,
    /// Spend analysis in progress
    SpendAnalysis,
    /// Supplier qualification phase
    Qualification,
    /// RFx in progress
    RfxActive,
    /// Evaluating bids
    Evaluation,
    /// Contract negotiation
    Negotiation,
    /// Contract awarded
    Awarded,
    /// Project completed
    Completed,
    /// Project cancelled
    Cancelled,
}

/// A sourcing project tracking a procurement initiative.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourcingProject {
    /// Unique project identifier
    pub project_id: String,
    /// Project name
    pub project_name: String,
    /// Company code
    pub company_code: String,
    /// Project type
    pub project_type: SourcingProjectType,
    /// Current status
    pub status: SourcingProjectStatus,
    /// Spend category being sourced
    pub category_id: String,
    /// Estimated annual spend
    #[serde(with = "rust_decimal::serde::str")]
    pub estimated_annual_spend: Decimal,
    /// Target savings percentage
    pub target_savings_pct: f64,
    /// Project owner (buyer/sourcing manager)
    pub owner_id: String,
    /// Start date
    pub start_date: NaiveDate,
    /// Target completion date
    pub target_end_date: NaiveDate,
    /// Actual completion date
    pub actual_end_date: Option<NaiveDate>,
    /// Related spend analysis ID
    pub spend_analysis_id: Option<String>,
    /// Related RFx event IDs
    pub rfx_ids: Vec<String>,
    /// Awarded contract ID
    pub contract_id: Option<String>,
    /// Actual savings achieved
    pub actual_savings_pct: Option<f64>,
}

impl ToNodeProperties for SourcingProject {
    fn node_type_name(&self) -> &'static str {
        "sourcing_project"
    }
    fn node_type_code(&self) -> u16 {
        320
    }
    fn to_node_properties(&self) -> HashMap<String, GraphPropertyValue> {
        let mut p = HashMap::new();
        p.insert(
            "projectId".into(),
            GraphPropertyValue::String(self.project_id.clone()),
        );
        p.insert(
            "projectName".into(),
            GraphPropertyValue::String(self.project_name.clone()),
        );
        p.insert(
            "entityCode".into(),
            GraphPropertyValue::String(self.company_code.clone()),
        );
        p.insert(
            "projectType".into(),
            GraphPropertyValue::String(format!("{:?}", self.project_type)),
        );
        p.insert(
            "status".into(),
            GraphPropertyValue::String(format!("{:?}", self.status)),
        );
        p.insert(
            "estimatedValue".into(),
            GraphPropertyValue::Decimal(self.estimated_annual_spend),
        );
        p.insert(
            "targetSavingsPct".into(),
            GraphPropertyValue::Float(self.target_savings_pct),
        );
        p.insert(
            "owner".into(),
            GraphPropertyValue::String(self.owner_id.clone()),
        );
        p.insert(
            "startDate".into(),
            GraphPropertyValue::Date(self.start_date),
        );
        p.insert(
            "targetEndDate".into(),
            GraphPropertyValue::Date(self.target_end_date),
        );
        p.insert(
            "bidCount".into(),
            GraphPropertyValue::Int(self.rfx_ids.len() as i64),
        );
        p.insert(
            "isComplete".into(),
            GraphPropertyValue::Bool(matches!(
                self.status,
                SourcingProjectStatus::Completed | SourcingProjectStatus::Awarded
            )),
        );
        p
    }
}
