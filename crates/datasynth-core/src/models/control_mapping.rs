//! Control-to-entity mappings for Internal Controls System.
//!
//! Defines how controls map to GL accounts, business processes,
//! amount thresholds, and document types.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::chart_of_accounts::AccountSubType;
use super::journal_entry::BusinessProcess;

/// Comparison operator for threshold mappings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThresholdComparison {
    /// Amount must be greater than threshold
    GreaterThan,
    /// Amount must be greater than or equal to threshold
    GreaterThanOrEqual,
    /// Amount must be less than threshold
    LessThan,
    /// Amount must be less than or equal to threshold
    LessThanOrEqual,
    /// Amount must be between two thresholds
    Between,
}

/// Mapping between a control and GL accounts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlAccountMapping {
    /// Control ID
    pub control_id: String,
    /// Specific GL account numbers (if any)
    pub account_numbers: Vec<String>,
    /// Account sub-types this control applies to
    pub account_sub_types: Vec<AccountSubType>,
}

impl ControlAccountMapping {
    /// Create a new control-to-account mapping.
    pub fn new(control_id: impl Into<String>) -> Self {
        Self {
            control_id: control_id.into(),
            account_numbers: Vec::new(),
            account_sub_types: Vec::new(),
        }
    }

    /// Add specific account numbers.
    pub fn with_accounts(mut self, accounts: Vec<String>) -> Self {
        self.account_numbers = accounts;
        self
    }

    /// Add account sub-types.
    pub fn with_sub_types(mut self, sub_types: Vec<AccountSubType>) -> Self {
        self.account_sub_types = sub_types;
        self
    }

    /// Check if this mapping applies to a given account.
    pub fn applies_to_account(
        &self,
        account_number: &str,
        sub_type: Option<&AccountSubType>,
    ) -> bool {
        // Check specific account numbers first
        if !self.account_numbers.is_empty()
            && self.account_numbers.iter().any(|a| a == account_number)
        {
            return true;
        }

        // Then check sub-types
        if let Some(st) = sub_type {
            if self.account_sub_types.contains(st) {
                return true;
            }
        }

        // If no specific accounts or sub-types defined, mapping doesn't apply
        false
    }
}

/// Mapping between a control and business processes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlProcessMapping {
    /// Control ID
    pub control_id: String,
    /// Business processes this control applies to
    pub business_processes: Vec<BusinessProcess>,
}

impl ControlProcessMapping {
    /// Create a new control-to-process mapping.
    pub fn new(control_id: impl Into<String>, processes: Vec<BusinessProcess>) -> Self {
        Self {
            control_id: control_id.into(),
            business_processes: processes,
        }
    }

    /// Check if this mapping applies to a given process.
    pub fn applies_to_process(&self, process: &BusinessProcess) -> bool {
        self.business_processes.contains(process)
    }
}

/// Mapping between a control and amount thresholds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlThresholdMapping {
    /// Control ID
    pub control_id: String,
    /// Primary threshold amount
    pub amount_threshold: Decimal,
    /// Optional upper bound for 'between' comparison
    pub upper_threshold: Option<Decimal>,
    /// Comparison type
    pub comparison: ThresholdComparison,
}

impl ControlThresholdMapping {
    /// Create a new control-to-threshold mapping.
    pub fn new(
        control_id: impl Into<String>,
        threshold: Decimal,
        comparison: ThresholdComparison,
    ) -> Self {
        Self {
            control_id: control_id.into(),
            amount_threshold: threshold,
            upper_threshold: None,
            comparison,
        }
    }

    /// Create a 'between' threshold mapping.
    pub fn between(control_id: impl Into<String>, lower: Decimal, upper: Decimal) -> Self {
        Self {
            control_id: control_id.into(),
            amount_threshold: lower,
            upper_threshold: Some(upper),
            comparison: ThresholdComparison::Between,
        }
    }

    /// Check if this mapping applies to a given amount.
    pub fn applies_to_amount(&self, amount: Decimal) -> bool {
        match self.comparison {
            ThresholdComparison::GreaterThan => amount > self.amount_threshold,
            ThresholdComparison::GreaterThanOrEqual => amount >= self.amount_threshold,
            ThresholdComparison::LessThan => amount < self.amount_threshold,
            ThresholdComparison::LessThanOrEqual => amount <= self.amount_threshold,
            ThresholdComparison::Between => {
                if let Some(upper) = self.upper_threshold {
                    amount >= self.amount_threshold && amount <= upper
                } else {
                    amount >= self.amount_threshold
                }
            }
        }
    }
}

/// Mapping between a control and document types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlDocTypeMapping {
    /// Control ID
    pub control_id: String,
    /// Document types this control applies to (SAP document types)
    pub document_types: Vec<String>,
}

impl ControlDocTypeMapping {
    /// Create a new control-to-document type mapping.
    pub fn new(control_id: impl Into<String>, doc_types: Vec<String>) -> Self {
        Self {
            control_id: control_id.into(),
            document_types: doc_types,
        }
    }

    /// Check if this mapping applies to a given document type.
    pub fn applies_to_doc_type(&self, doc_type: &str) -> bool {
        self.document_types.iter().any(|dt| dt == doc_type)
    }
}

/// Master registry of all control mappings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ControlMappingRegistry {
    /// Control-to-account mappings
    pub account_mappings: Vec<ControlAccountMapping>,
    /// Control-to-process mappings
    pub process_mappings: Vec<ControlProcessMapping>,
    /// Control-to-threshold mappings
    pub threshold_mappings: Vec<ControlThresholdMapping>,
    /// Control-to-document type mappings
    pub doc_type_mappings: Vec<ControlDocTypeMapping>,
}

impl ControlMappingRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a registry with standard mappings.
    pub fn standard() -> Self {
        let mut registry = Self::new();

        // Cash controls (C001) - apply to cash accounts
        registry
            .account_mappings
            .push(ControlAccountMapping::new("C001").with_sub_types(vec![AccountSubType::Cash]));

        // Large transaction approval (C002) - threshold based
        registry
            .threshold_mappings
            .push(ControlThresholdMapping::new(
                "C002",
                Decimal::from(10000),
                ThresholdComparison::GreaterThanOrEqual,
            ));

        // P2P controls (C010, C011) - apply to P2P process
        registry.process_mappings.push(ControlProcessMapping::new(
            "C010",
            vec![BusinessProcess::P2P],
        ));
        registry.process_mappings.push(ControlProcessMapping::new(
            "C011",
            vec![BusinessProcess::P2P],
        ));
        registry.account_mappings.push(
            ControlAccountMapping::new("C010")
                .with_sub_types(vec![AccountSubType::AccountsPayable]),
        );

        // O2C controls (C020, C021) - apply to O2C process
        registry.process_mappings.push(ControlProcessMapping::new(
            "C020",
            vec![BusinessProcess::O2C],
        ));
        registry.process_mappings.push(ControlProcessMapping::new(
            "C021",
            vec![BusinessProcess::O2C],
        ));
        registry
            .account_mappings
            .push(ControlAccountMapping::new("C020").with_sub_types(vec![
                AccountSubType::ProductRevenue,
                AccountSubType::ServiceRevenue,
            ]));
        registry.account_mappings.push(
            ControlAccountMapping::new("C021")
                .with_sub_types(vec![AccountSubType::AccountsReceivable]),
        );

        // GL controls (C030, C031, C032) - apply to R2R process
        registry.process_mappings.push(ControlProcessMapping::new(
            "C030",
            vec![BusinessProcess::R2R],
        ));
        registry.process_mappings.push(ControlProcessMapping::new(
            "C031",
            vec![BusinessProcess::R2R],
        ));
        registry.process_mappings.push(ControlProcessMapping::new(
            "C032",
            vec![BusinessProcess::R2R],
        ));
        // Manual JE review applies to document type SA
        registry
            .doc_type_mappings
            .push(ControlDocTypeMapping::new("C031", vec!["SA".to_string()]));

        // Payroll controls (C040) - apply to H2R process
        registry.process_mappings.push(ControlProcessMapping::new(
            "C040",
            vec![BusinessProcess::H2R],
        ));

        // Fixed asset controls (C050) - apply to A2R process
        registry.process_mappings.push(ControlProcessMapping::new(
            "C050",
            vec![BusinessProcess::A2R],
        ));
        registry
            .account_mappings
            .push(ControlAccountMapping::new("C050").with_sub_types(vec![
                AccountSubType::FixedAssets,
                AccountSubType::AccumulatedDepreciation,
            ]));

        // Intercompany controls (C060) - apply to Intercompany process
        registry.process_mappings.push(ControlProcessMapping::new(
            "C060",
            vec![BusinessProcess::Intercompany],
        ));

        registry
    }

    /// Get all control IDs that apply to a transaction.
    pub fn get_applicable_controls(
        &self,
        account_number: &str,
        account_sub_type: Option<&AccountSubType>,
        process: Option<&BusinessProcess>,
        amount: Decimal,
        doc_type: Option<&str>,
    ) -> Vec<String> {
        let mut control_ids = HashSet::new();

        // Check account mappings
        for mapping in &self.account_mappings {
            if mapping.applies_to_account(account_number, account_sub_type) {
                control_ids.insert(mapping.control_id.clone());
            }
        }

        // Check process mappings
        if let Some(bp) = process {
            for mapping in &self.process_mappings {
                if mapping.applies_to_process(bp) {
                    control_ids.insert(mapping.control_id.clone());
                }
            }
        }

        // Check threshold mappings
        for mapping in &self.threshold_mappings {
            if mapping.applies_to_amount(amount) {
                control_ids.insert(mapping.control_id.clone());
            }
        }

        // Check document type mappings
        if let Some(dt) = doc_type {
            for mapping in &self.doc_type_mappings {
                if mapping.applies_to_doc_type(dt) {
                    control_ids.insert(mapping.control_id.clone());
                }
            }
        }

        let mut result: Vec<_> = control_ids.into_iter().collect();
        result.sort();
        result
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_threshold_mapping() {
        let mapping = ControlThresholdMapping::new(
            "C002",
            Decimal::from(10000),
            ThresholdComparison::GreaterThanOrEqual,
        );

        assert!(mapping.applies_to_amount(Decimal::from(10000)));
        assert!(mapping.applies_to_amount(Decimal::from(50000)));
        assert!(!mapping.applies_to_amount(Decimal::from(9999)));
    }

    #[test]
    fn test_between_threshold() {
        let mapping =
            ControlThresholdMapping::between("TEST", Decimal::from(1000), Decimal::from(10000));

        assert!(mapping.applies_to_amount(Decimal::from(5000)));
        assert!(mapping.applies_to_amount(Decimal::from(1000)));
        assert!(mapping.applies_to_amount(Decimal::from(10000)));
        assert!(!mapping.applies_to_amount(Decimal::from(999)));
        assert!(!mapping.applies_to_amount(Decimal::from(10001)));
    }

    #[test]
    fn test_account_mapping() {
        let mapping = ControlAccountMapping::new("C001").with_sub_types(vec![AccountSubType::Cash]);

        assert!(mapping.applies_to_account("100000", Some(&AccountSubType::Cash)));
        assert!(!mapping.applies_to_account("200000", Some(&AccountSubType::AccountsPayable)));
    }

    #[test]
    fn test_standard_registry() {
        let registry = ControlMappingRegistry::standard();

        // Test that standard mappings exist
        assert!(!registry.account_mappings.is_empty());
        assert!(!registry.process_mappings.is_empty());
        assert!(!registry.threshold_mappings.is_empty());

        // Test getting applicable controls for a large cash transaction
        let controls = registry.get_applicable_controls(
            "100000",
            Some(&AccountSubType::Cash),
            Some(&BusinessProcess::R2R),
            Decimal::from(50000),
            Some("SA"),
        );

        // Should include cash control and large transaction control
        assert!(controls.contains(&"C001".to_string()));
        assert!(controls.contains(&"C002".to_string()));
    }
}
