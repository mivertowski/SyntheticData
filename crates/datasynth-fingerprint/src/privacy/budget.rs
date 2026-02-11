//! Global privacy budget management across extraction runs.
//!
//! This module provides:
//! - **PrivacyBudgetManager**: Persistent budget tracking across multiple extraction runs
//!   with JSON file persistence.
//! - **BudgetRunRecord**: Per-run accounting of epsilon spent and mechanisms applied.
//! - **RunBudgetGuard**: RAII guard that auto-rolls back uncommitted budget usage on drop.
//!
//! # Usage
//!
//! ```ignore
//! use datasynth_fingerprint::privacy::budget::PrivacyBudgetManager;
//!
//! let mut manager = PrivacyBudgetManager::new(10.0);
//!
//! // Start a guarded run - will roll back if not committed
//! let mut guard = manager.start_run("run-001", "Initial extraction");
//! guard.record_epsilon(0.5, "Laplace noise on amounts");
//! guard.record_epsilon(0.3, "Laplace noise on counts");
//! guard.commit(); // Locks in the budget spend
//!
//! // If guard is dropped without commit, the run is rolled back
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::composition::MechanismRecord;
use crate::error::FingerprintResult;

/// Global privacy budget manager that tracks budget across multiple extraction runs.
///
/// Supports optional JSON persistence so budget state survives across process invocations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyBudgetManager {
    /// Total global epsilon budget.
    pub total_budget: f64,

    /// Epsilon consumed across all committed runs.
    pub total_spent: f64,

    /// Records of all committed runs.
    pub runs: Vec<BudgetRunRecord>,

    /// Timestamp when this manager was created.
    pub created_at: DateTime<Utc>,

    /// Timestamp of last modification.
    pub updated_at: DateTime<Utc>,
}

impl PrivacyBudgetManager {
    /// Create a new budget manager with the given total budget.
    pub fn new(total_budget: f64) -> Self {
        let now = Utc::now();
        Self {
            total_budget,
            total_spent: 0.0,
            runs: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Load a budget manager from a JSON file.
    pub fn load(path: &Path) -> FingerprintResult<Self> {
        let content = std::fs::read_to_string(path)?;
        let manager: Self = serde_json::from_str(&content)?;
        Ok(manager)
    }

    /// Save the budget manager to a JSON file.
    pub fn save(&self, path: &Path) -> FingerprintResult<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Load from file if it exists, or create a new manager.
    pub fn load_or_create(path: &Path, total_budget: f64) -> FingerprintResult<Self> {
        if path.exists() {
            Self::load(path)
        } else {
            Ok(Self::new(total_budget))
        }
    }

    /// Get the remaining epsilon budget.
    pub fn remaining_budget(&self) -> f64 {
        (self.total_budget - self.total_spent).max(0.0)
    }

    /// Check if the budget is exhausted.
    pub fn is_exhausted(&self) -> bool {
        self.total_spent >= self.total_budget
    }

    /// Check if there is enough budget for a given epsilon spend.
    pub fn can_spend(&self, epsilon: f64) -> bool {
        self.total_spent + epsilon <= self.total_budget
    }

    /// Start a new guarded run.
    ///
    /// Returns a `RunBudgetGuard` that will automatically roll back the budget
    /// if dropped without calling `commit()`.
    pub fn start_run(
        &mut self,
        run_id: impl Into<String>,
        description: impl Into<String>,
    ) -> RunBudgetGuard<'_> {
        let record = BudgetRunRecord {
            run_id: run_id.into(),
            description: description.into(),
            timestamp: Utc::now(),
            epsilon_spent: 0.0,
            mechanisms: Vec::new(),
            committed: false,
        };
        RunBudgetGuard {
            manager: self,
            record,
            committed: false,
        }
    }

    /// Directly commit a run record (without using a guard).
    pub fn commit_run(&mut self, mut record: BudgetRunRecord) {
        record.committed = true;
        self.total_spent += record.epsilon_spent;
        self.updated_at = Utc::now();
        self.runs.push(record);
    }

    /// Get the number of committed runs.
    pub fn run_count(&self) -> usize {
        self.runs.len()
    }
}

/// Record of a single extraction run's budget consumption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetRunRecord {
    /// Unique identifier for this run.
    pub run_id: String,

    /// Human-readable description of the run.
    pub description: String,

    /// Timestamp when the run started.
    pub timestamp: DateTime<Utc>,

    /// Total epsilon spent in this run.
    pub epsilon_spent: f64,

    /// Individual mechanism records within this run.
    pub mechanisms: Vec<MechanismRecord>,

    /// Whether this run was committed (false = rolled back).
    pub committed: bool,
}

impl BudgetRunRecord {
    /// Create a new run record.
    pub fn new(run_id: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            run_id: run_id.into(),
            description: description.into(),
            timestamp: Utc::now(),
            epsilon_spent: 0.0,
            mechanisms: Vec::new(),
            committed: false,
        }
    }

    /// Record epsilon spent for a mechanism.
    pub fn record_epsilon(&mut self, epsilon: f64, description: impl Into<String>) {
        self.epsilon_spent += epsilon;
        self.mechanisms
            .push(MechanismRecord::new(epsilon, description));
    }
}

/// RAII guard for run budget that auto-rolls back on drop unless committed.
///
/// If the guard is dropped without calling `commit()`, the run's epsilon
/// is not added to the manager's total spent.
pub struct RunBudgetGuard<'a> {
    manager: &'a mut PrivacyBudgetManager,
    record: BudgetRunRecord,
    committed: bool,
}

impl<'a> RunBudgetGuard<'a> {
    /// Record epsilon spent for a mechanism in this run.
    pub fn record_epsilon(&mut self, epsilon: f64, description: impl Into<String>) {
        self.record.record_epsilon(epsilon, description);
    }

    /// Get the epsilon spent so far in this run.
    pub fn epsilon_spent(&self) -> f64 {
        self.record.epsilon_spent
    }

    /// Check if the manager has enough remaining budget for additional spend.
    pub fn can_spend(&self, epsilon: f64) -> bool {
        self.manager.total_spent + self.record.epsilon_spent + epsilon <= self.manager.total_budget
    }

    /// Get the run ID.
    pub fn run_id(&self) -> &str {
        &self.record.run_id
    }

    /// Commit this run, locking in the epsilon spend.
    pub fn commit(mut self) {
        self.committed = true;
        let mut record = self.record.clone();
        record.committed = true;
        self.manager.total_spent += record.epsilon_spent;
        self.manager.updated_at = Utc::now();
        self.manager.runs.push(record);
    }
}

impl<'a> Drop for RunBudgetGuard<'a> {
    fn drop(&mut self) {
        if !self.committed {
            // Run was not committed - budget is automatically rolled back
            // (nothing was added to manager.total_spent)
            tracing::debug!(
                run_id = %self.record.run_id,
                epsilon = %self.record.epsilon_spent,
                "Privacy budget run rolled back (not committed)"
            );
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_budget_manager_basic() {
        let manager = PrivacyBudgetManager::new(10.0);
        assert_eq!(manager.total_budget, 10.0);
        assert_eq!(manager.total_spent, 0.0);
        assert_eq!(manager.remaining_budget(), 10.0);
        assert!(!manager.is_exhausted());
        assert!(manager.can_spend(5.0));
        assert!(!manager.can_spend(11.0));
    }

    #[test]
    fn test_budget_manager_commit_run() {
        let mut manager = PrivacyBudgetManager::new(10.0);

        let mut record = BudgetRunRecord::new("run-1", "First run");
        record.record_epsilon(0.5, "noise on amounts");
        record.record_epsilon(0.3, "noise on counts");
        manager.commit_run(record);

        assert!((manager.total_spent - 0.8).abs() < 1e-10);
        assert!((manager.remaining_budget() - 9.2).abs() < 1e-10);
        assert_eq!(manager.run_count(), 1);
    }

    #[test]
    fn test_budget_guard_commit() {
        let mut manager = PrivacyBudgetManager::new(10.0);

        {
            let mut guard = manager.start_run("run-1", "Test run");
            guard.record_epsilon(0.5, "noise");
            guard.record_epsilon(0.3, "more noise");
            assert!((guard.epsilon_spent() - 0.8).abs() < 1e-10);
            guard.commit();
        }

        assert!((manager.total_spent - 0.8).abs() < 1e-10);
        assert_eq!(manager.run_count(), 1);
    }

    #[test]
    fn test_budget_guard_rollback_on_drop() {
        let mut manager = PrivacyBudgetManager::new(10.0);

        {
            let mut guard = manager.start_run("run-1", "Will be rolled back");
            guard.record_epsilon(5.0, "big noise");
            // Dropped without commit
        }

        assert!((manager.total_spent - 0.0).abs() < 1e-10);
        assert_eq!(manager.run_count(), 0);
    }

    #[test]
    fn test_budget_guard_can_spend() {
        let mut manager = PrivacyBudgetManager::new(1.0);

        let mut guard = manager.start_run("run-1", "Check budget");
        guard.record_epsilon(0.5, "noise");
        assert!(guard.can_spend(0.4));
        assert!(!guard.can_spend(0.6));

        guard.commit();
    }

    #[test]
    fn test_budget_manager_persistence() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();

        // Create and save
        {
            let mut manager = PrivacyBudgetManager::new(10.0);
            let mut record = BudgetRunRecord::new("run-1", "Persisted run");
            record.record_epsilon(0.5, "noise");
            manager.commit_run(record);
            manager.save(path).unwrap();
        }

        // Load and verify
        {
            let manager = PrivacyBudgetManager::load(path).unwrap();
            assert_eq!(manager.total_budget, 10.0);
            assert!((manager.total_spent - 0.5).abs() < 1e-10);
            assert_eq!(manager.run_count(), 1);
            assert_eq!(manager.runs[0].run_id, "run-1");
            assert!(manager.runs[0].committed);
        }
    }

    #[test]
    fn test_budget_manager_load_or_create() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();

        // File doesn't exist yet (but tempfile creates it, so we remove it)
        std::fs::remove_file(path).unwrap();
        let manager = PrivacyBudgetManager::load_or_create(path, 5.0).unwrap();
        assert_eq!(manager.total_budget, 5.0);
        assert_eq!(manager.total_spent, 0.0);

        // Save and reload
        let mut manager = manager;
        let mut record = BudgetRunRecord::new("run-1", "test");
        record.record_epsilon(1.0, "mechanism");
        manager.commit_run(record);
        manager.save(path).unwrap();

        let loaded = PrivacyBudgetManager::load_or_create(path, 5.0).unwrap();
        assert!((loaded.total_spent - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_budget_exhaustion() {
        let mut manager = PrivacyBudgetManager::new(1.0);

        let mut record = BudgetRunRecord::new("run-1", "Use most budget");
        record.record_epsilon(0.9, "big query");
        manager.commit_run(record);

        assert!(!manager.is_exhausted());
        assert!(manager.can_spend(0.1));
        assert!(!manager.can_spend(0.2));

        let mut record2 = BudgetRunRecord::new("run-2", "Use remaining");
        record2.record_epsilon(0.1, "small query");
        manager.commit_run(record2);

        assert!(manager.is_exhausted());
        assert_eq!(manager.remaining_budget(), 0.0);
    }

    #[test]
    fn test_run_record_serde() {
        let mut record = BudgetRunRecord::new("run-1", "Test run");
        record.record_epsilon(0.5, "noise mechanism");
        record.committed = true;

        let json = serde_json::to_string(&record).unwrap();
        let parsed: BudgetRunRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.run_id, "run-1");
        assert_eq!(parsed.description, "Test run");
        assert!((parsed.epsilon_spent - 0.5).abs() < 1e-10);
        assert_eq!(parsed.mechanisms.len(), 1);
        assert!(parsed.committed);
    }
}
