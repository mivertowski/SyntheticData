//! Control generator for applying Internal Controls System (ICS) to transactions.
//!
//! Implements control application, SOX relevance determination, and
//! Segregation of Duties (SoD) violation detection.

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;

use datasynth_core::models::{
    BusinessProcess, ChartOfAccounts, ControlMappingRegistry, ControlStatus, InternalControl,
    JournalEntry, RiskLevel, SodConflictPair, SodConflictType, SodViolation,
};

/// Configuration for the control generator.
#[derive(Debug, Clone)]
pub struct ControlGeneratorConfig {
    /// Rate at which controls result in exceptions (0.0 - 1.0).
    pub exception_rate: f64,
    /// Rate at which SoD violations occur (0.0 - 1.0).
    pub sod_violation_rate: f64,
    /// Whether to mark SOX-relevant transactions.
    pub enable_sox_marking: bool,
    /// Amount threshold above which transactions are SOX-relevant.
    pub sox_materiality_threshold: Decimal,
}

impl Default for ControlGeneratorConfig {
    fn default() -> Self {
        Self {
            exception_rate: 0.02,     // 2% exception rate
            sod_violation_rate: 0.01, // 1% SoD violation rate
            enable_sox_marking: true,
            sox_materiality_threshold: Decimal::from(10000),
        }
    }
}

/// Generator that applies internal controls to journal entries.
pub struct ControlGenerator {
    rng: ChaCha8Rng,
    seed: u64,
    config: ControlGeneratorConfig,
    registry: ControlMappingRegistry,
    controls: Vec<InternalControl>,
    sod_checker: SodChecker,
}

impl ControlGenerator {
    /// Create a new control generator with default configuration.
    pub fn new(seed: u64) -> Self {
        Self::with_config(seed, ControlGeneratorConfig::default())
    }

    /// Create a new control generator with custom configuration.
    pub fn with_config(seed: u64, config: ControlGeneratorConfig) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            seed,
            config: config.clone(),
            registry: ControlMappingRegistry::standard(),
            controls: InternalControl::standard_controls(),
            sod_checker: SodChecker::new(seed + 1, config.sod_violation_rate),
        }
    }

    /// Apply controls to a journal entry.
    ///
    /// This modifies the journal entry header to include:
    /// - Applicable control IDs
    /// - SOX relevance flag
    /// - Control status (effective, exception, not tested)
    /// - SoD violation flag and conflict type
    pub fn apply_controls(&mut self, entry: &mut JournalEntry, coa: &ChartOfAccounts) {
        // Determine applicable controls from all line items
        let mut all_control_ids = Vec::new();

        for line in &entry.lines {
            let amount = if line.debit_amount > Decimal::ZERO {
                line.debit_amount
            } else {
                line.credit_amount
            };

            // Get account sub-type from CoA
            let account_sub_type = coa.get_account(&line.gl_account).map(|acc| acc.sub_type);

            let control_ids = self.registry.get_applicable_controls(
                &line.gl_account,
                account_sub_type.as_ref(),
                entry.header.business_process.as_ref(),
                amount,
                Some(&entry.header.document_type),
            );

            all_control_ids.extend(control_ids);
        }

        // Deduplicate and sort control IDs
        all_control_ids.sort();
        all_control_ids.dedup();
        entry.header.control_ids = all_control_ids;

        // Determine SOX relevance
        entry.header.sox_relevant = self.determine_sox_relevance(entry);

        // Determine control status
        entry.header.control_status = self.determine_control_status(entry);

        // Check for SoD violations
        let (sod_violation, sod_conflict_type) = self.sod_checker.check_entry(entry);
        entry.header.sod_violation = sod_violation;
        entry.header.sod_conflict_type = sod_conflict_type;
    }

    /// Determine if a transaction is SOX-relevant.
    fn determine_sox_relevance(&self, entry: &JournalEntry) -> bool {
        if !self.config.enable_sox_marking {
            return false;
        }

        // SOX-relevant if:
        // 1. Amount exceeds materiality threshold
        let total_amount = entry.total_debit();
        if total_amount >= self.config.sox_materiality_threshold {
            return true;
        }

        // 2. Has key controls applied
        let has_key_control = entry.header.control_ids.iter().any(|cid| {
            self.controls
                .iter()
                .any(|c| c.control_id == *cid && c.is_key_control)
        });
        if has_key_control {
            return true;
        }

        // 3. Involves critical business processes
        if let Some(bp) = &entry.header.business_process {
            matches!(
                bp,
                BusinessProcess::R2R | BusinessProcess::P2P | BusinessProcess::O2C
            )
        } else {
            false
        }
    }

    /// Determine the control status for a transaction.
    fn determine_control_status(&mut self, entry: &JournalEntry) -> ControlStatus {
        // If no controls apply, mark as not tested
        if entry.header.control_ids.is_empty() {
            return ControlStatus::NotTested;
        }

        // Roll for exception based on exception rate
        if self.rng.gen::<f64>() < self.config.exception_rate {
            ControlStatus::Exception
        } else {
            ControlStatus::Effective
        }
    }

    /// Get the current control definitions.
    pub fn controls(&self) -> &[InternalControl] {
        &self.controls
    }

    /// Get the control mapping registry.
    pub fn registry(&self) -> &ControlMappingRegistry {
        &self.registry
    }

    /// Reset the generator to its initial state.
    pub fn reset(&mut self) {
        self.rng = ChaCha8Rng::seed_from_u64(self.seed);
        self.sod_checker.reset();
    }
}

/// Checker for Segregation of Duties (SoD) violations.
pub struct SodChecker {
    rng: ChaCha8Rng,
    seed: u64,
    violation_rate: f64,
    conflict_pairs: Vec<SodConflictPair>,
}

impl SodChecker {
    /// Create a new SoD checker.
    pub fn new(seed: u64, violation_rate: f64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            seed,
            violation_rate,
            conflict_pairs: SodConflictPair::standard_conflicts(),
        }
    }

    /// Check a journal entry for SoD violations.
    ///
    /// Returns a tuple of (has_violation, conflict_type).
    pub fn check_entry(&mut self, entry: &JournalEntry) -> (bool, Option<SodConflictType>) {
        // Roll for violation based on violation rate
        if self.rng.gen::<f64>() >= self.violation_rate {
            return (false, None);
        }

        // Select an appropriate conflict type based on transaction characteristics
        let conflict_type = self.select_conflict_type(entry);

        (true, Some(conflict_type))
    }

    /// Select a conflict type based on transaction characteristics.
    fn select_conflict_type(&mut self, entry: &JournalEntry) -> SodConflictType {
        // Map business process to likely conflict types
        let likely_conflicts: Vec<SodConflictType> = match entry.header.business_process {
            Some(BusinessProcess::P2P) => vec![
                SodConflictType::PaymentReleaser,
                SodConflictType::MasterDataMaintainer,
                SodConflictType::PreparerApprover,
            ],
            Some(BusinessProcess::O2C) => vec![
                SodConflictType::PreparerApprover,
                SodConflictType::RequesterApprover,
            ],
            Some(BusinessProcess::R2R) => vec![
                SodConflictType::PreparerApprover,
                SodConflictType::ReconcilerPoster,
                SodConflictType::JournalEntryPoster,
            ],
            Some(BusinessProcess::H2R) => vec![
                SodConflictType::RequesterApprover,
                SodConflictType::PreparerApprover,
            ],
            Some(BusinessProcess::A2R) => vec![SodConflictType::PreparerApprover],
            Some(BusinessProcess::Intercompany) => vec![
                SodConflictType::PreparerApprover,
                SodConflictType::ReconcilerPoster,
            ],
            Some(BusinessProcess::S2C) => vec![
                SodConflictType::RequesterApprover,
                SodConflictType::MasterDataMaintainer,
            ],
            Some(BusinessProcess::Mfg) => vec![
                SodConflictType::PreparerApprover,
                SodConflictType::RequesterApprover,
            ],
            Some(BusinessProcess::Bank) => vec![
                SodConflictType::PaymentReleaser,
                SodConflictType::PreparerApprover,
            ],
            Some(BusinessProcess::Audit) => vec![
                SodConflictType::PreparerApprover,
            ],
            Some(BusinessProcess::Treasury) | Some(BusinessProcess::Tax) => vec![
                SodConflictType::PreparerApprover,
                SodConflictType::PaymentReleaser,
            ],
            None => vec![
                SodConflictType::PreparerApprover,
                SodConflictType::SystemAccessConflict,
            ],
        };

        // Randomly select from likely conflicts
        likely_conflicts
            .choose(&mut self.rng)
            .copied()
            .unwrap_or(SodConflictType::PreparerApprover)
    }

    /// Create a SoD violation record from an entry.
    pub fn create_violation_record(
        &self,
        entry: &JournalEntry,
        conflict_type: SodConflictType,
    ) -> SodViolation {
        let description = match conflict_type {
            SodConflictType::PreparerApprover => {
                format!(
                    "User {} both prepared and approved journal entry {}",
                    entry.header.created_by, entry.header.document_id
                )
            }
            SodConflictType::RequesterApprover => {
                format!(
                    "User {} approved their own request in transaction {}",
                    entry.header.created_by, entry.header.document_id
                )
            }
            SodConflictType::ReconcilerPoster => {
                format!(
                    "User {} both reconciled and posted adjustments in {}",
                    entry.header.created_by, entry.header.document_id
                )
            }
            SodConflictType::MasterDataMaintainer => {
                format!(
                    "User {} maintains master data and processed payment {}",
                    entry.header.created_by, entry.header.document_id
                )
            }
            SodConflictType::PaymentReleaser => {
                format!(
                    "User {} both created and released payment {}",
                    entry.header.created_by, entry.header.document_id
                )
            }
            SodConflictType::JournalEntryPoster => {
                format!(
                    "User {} posted to sensitive accounts without review in {}",
                    entry.header.created_by, entry.header.document_id
                )
            }
            SodConflictType::SystemAccessConflict => {
                format!(
                    "User {} has conflicting system access roles for {}",
                    entry.header.created_by, entry.header.document_id
                )
            }
        };

        // Determine severity based on conflict type and amount
        let severity = self.determine_violation_severity(entry, conflict_type);

        SodViolation::with_timestamp(
            conflict_type,
            &entry.header.created_by,
            description,
            severity,
            entry.header.created_at,
        )
    }

    /// Determine the severity of a violation.
    fn determine_violation_severity(
        &self,
        entry: &JournalEntry,
        conflict_type: SodConflictType,
    ) -> RiskLevel {
        let amount = entry.total_debit();

        // Base severity from conflict type
        let base_severity = match conflict_type {
            SodConflictType::PaymentReleaser | SodConflictType::RequesterApprover => {
                RiskLevel::Critical
            }
            SodConflictType::PreparerApprover | SodConflictType::MasterDataMaintainer => {
                RiskLevel::High
            }
            SodConflictType::ReconcilerPoster | SodConflictType::JournalEntryPoster => {
                RiskLevel::Medium
            }
            SodConflictType::SystemAccessConflict => RiskLevel::Low,
        };

        // Escalate based on amount
        if amount >= Decimal::from(100000) {
            match base_severity {
                RiskLevel::Low => RiskLevel::Medium,
                RiskLevel::Medium => RiskLevel::High,
                RiskLevel::High | RiskLevel::Critical => RiskLevel::Critical,
            }
        } else {
            base_severity
        }
    }

    /// Get the SoD conflict pairs.
    pub fn conflict_pairs(&self) -> &[SodConflictPair] {
        &self.conflict_pairs
    }

    /// Reset the checker to its initial state.
    pub fn reset(&mut self) {
        self.rng = ChaCha8Rng::seed_from_u64(self.seed);
    }
}

/// Extension trait for applying controls to journal entries.
pub trait ControlApplicationExt {
    /// Apply controls using the given generator.
    fn apply_controls(&mut self, generator: &mut ControlGenerator, coa: &ChartOfAccounts);
}

impl ControlApplicationExt for JournalEntry {
    fn apply_controls(&mut self, generator: &mut ControlGenerator, coa: &ChartOfAccounts) {
        generator.apply_controls(self, coa);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use datasynth_core::models::{JournalEntryHeader, JournalEntryLine};
    use uuid::Uuid;

    fn create_test_entry() -> JournalEntry {
        let mut header = JournalEntryHeader::new(
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
        );
        header.business_process = Some(BusinessProcess::R2R);
        header.created_by = "USER001".to_string();

        let mut entry = JournalEntry::new(header);
        entry.add_line(JournalEntryLine::debit(
            Uuid::new_v4(),
            1,
            "100000".to_string(),
            Decimal::from(50000),
        ));
        entry.add_line(JournalEntryLine::credit(
            Uuid::new_v4(),
            2,
            "200000".to_string(),
            Decimal::from(50000),
        ));

        entry
    }

    fn create_test_coa() -> ChartOfAccounts {
        ChartOfAccounts::new(
            "TEST".to_string(),
            "Test CoA".to_string(),
            "US".to_string(),
            datasynth_core::IndustrySector::Manufacturing,
            datasynth_core::CoAComplexity::Small,
        )
    }

    #[test]
    fn test_control_generator_creation() {
        let gen = ControlGenerator::new(42);
        assert!(!gen.controls().is_empty());
    }

    #[test]
    fn test_apply_controls() {
        let mut gen = ControlGenerator::new(42);
        let mut entry = create_test_entry();
        let coa = create_test_coa();

        gen.apply_controls(&mut entry, &coa);

        // After applying controls, entry should have control metadata
        assert!(matches!(
            entry.header.control_status,
            ControlStatus::Effective | ControlStatus::Exception | ControlStatus::NotTested
        ));
    }

    #[test]
    fn test_sox_relevance_high_amount() {
        let config = ControlGeneratorConfig {
            sox_materiality_threshold: Decimal::from(10000),
            ..Default::default()
        };
        let mut gen = ControlGenerator::with_config(42, config);
        let mut entry = create_test_entry();
        let coa = create_test_coa();

        gen.apply_controls(&mut entry, &coa);

        // Entry with 50,000 amount should be SOX-relevant
        assert!(entry.header.sox_relevant);
    }

    #[test]
    fn test_sod_checker() {
        let mut checker = SodChecker::new(42, 1.0); // 100% violation rate for testing
        let entry = create_test_entry();

        let (has_violation, conflict_type) = checker.check_entry(&entry);

        assert!(has_violation);
        assert!(conflict_type.is_some());
    }

    #[test]
    fn test_sod_violation_record() {
        let checker = SodChecker::new(42, 1.0);
        let entry = create_test_entry();

        let violation = checker.create_violation_record(&entry, SodConflictType::PreparerApprover);

        assert_eq!(violation.actor_id, "USER001");
        assert_eq!(violation.conflict_type, SodConflictType::PreparerApprover);
    }

    #[test]
    fn test_deterministic_generation() {
        let mut gen1 = ControlGenerator::new(42);
        let mut gen2 = ControlGenerator::new(42);

        let mut entry1 = create_test_entry();
        let mut entry2 = create_test_entry();
        let coa = create_test_coa();

        gen1.apply_controls(&mut entry1, &coa);
        gen2.apply_controls(&mut entry2, &coa);

        assert_eq!(entry1.header.control_status, entry2.header.control_status);
        assert_eq!(entry1.header.sod_violation, entry2.header.sod_violation);
    }

    #[test]
    fn test_reset() {
        let mut gen = ControlGenerator::new(42);
        let coa = create_test_coa();

        // Generate some entries
        for _ in 0..10 {
            let mut entry = create_test_entry();
            gen.apply_controls(&mut entry, &coa);
        }

        // Reset
        gen.reset();

        // Generate again - should produce same results
        let mut entry1 = create_test_entry();
        gen.apply_controls(&mut entry1, &coa);

        gen.reset();

        let mut entry2 = create_test_entry();
        gen.apply_controls(&mut entry2, &coa);

        assert_eq!(entry1.header.control_status, entry2.header.control_status);
    }
}
