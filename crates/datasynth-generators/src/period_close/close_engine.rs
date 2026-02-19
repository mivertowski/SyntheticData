//! Period close engine for orchestrating the close process.

use rust_decimal::Decimal;
use tracing::debug;

use datasynth_core::models::{
    CloseSchedule, CloseTask, CloseTaskResult, CloseTaskStatus, FiscalPeriod, JournalEntry,
    PeriodCloseRun, PeriodCloseStatus, PeriodStatus,
};

/// Configuration for the close engine.
#[derive(Debug, Clone)]
pub struct CloseEngineConfig {
    /// Whether to stop on first error.
    pub stop_on_error: bool,
    /// Whether to generate reversal entries for accruals.
    pub auto_reverse_accruals: bool,
    /// Whether to validate subledger reconciliation.
    pub require_reconciliation: bool,
    /// Tolerance for reconciliation differences.
    pub reconciliation_tolerance: Decimal,
}

impl Default for CloseEngineConfig {
    fn default() -> Self {
        Self {
            stop_on_error: false,
            auto_reverse_accruals: true,
            require_reconciliation: true,
            reconciliation_tolerance: Decimal::new(1, 2), // 0.01
        }
    }
}

/// Period close engine that orchestrates the close process.
pub struct CloseEngine {
    config: CloseEngineConfig,
    run_counter: u64,
}

impl CloseEngine {
    /// Creates a new close engine.
    pub fn new(config: CloseEngineConfig) -> Self {
        Self {
            config,
            run_counter: 0,
        }
    }

    /// Executes a period close for a company.
    pub fn execute_close(
        &mut self,
        company_code: &str,
        fiscal_period: FiscalPeriod,
        schedule: &CloseSchedule,
        context: &mut CloseContext,
    ) -> PeriodCloseRun {
        debug!(
            company_code,
            period = fiscal_period.period,
            year = fiscal_period.year,
            task_count = schedule.tasks.len(),
            "Executing period close"
        );
        self.run_counter += 1;
        let run_id = format!("CLOSE-{:08}", self.run_counter);

        let mut run = PeriodCloseRun::new(run_id, company_code.to_string(), fiscal_period.clone());
        run.status = PeriodCloseStatus::InProgress;
        run.started_at = Some(fiscal_period.end_date);

        // Execute tasks in sequence order
        let mut _current_sequence = 0u32;
        for scheduled_task in &schedule.tasks {
            // Skip year-end tasks if not year-end
            if scheduled_task.task.is_year_end_only() && !fiscal_period.is_year_end {
                let mut result = CloseTaskResult::new(
                    scheduled_task.task.clone(),
                    company_code.to_string(),
                    fiscal_period.clone(),
                );
                result.status = CloseTaskStatus::Skipped("Not year-end period".to_string());
                run.task_results.push(result);
                continue;
            }

            // Check dependencies
            let deps_met = scheduled_task.depends_on.iter().all(|dep| {
                run.task_results
                    .iter()
                    .any(|r| r.task == *dep && r.is_success())
            });

            if !deps_met {
                let mut result = CloseTaskResult::new(
                    scheduled_task.task.clone(),
                    company_code.to_string(),
                    fiscal_period.clone(),
                );
                result.status = CloseTaskStatus::Skipped("Dependencies not met".to_string());
                run.task_results.push(result);
                continue;
            }

            // Execute the task
            let result =
                self.execute_task(&scheduled_task.task, company_code, &fiscal_period, context);

            run.total_journal_entries += result.journal_entries_created;

            if let CloseTaskStatus::Failed(ref err) = result.status {
                run.errors
                    .push(format!("{}: {}", scheduled_task.task.name(), err));
                if self.config.stop_on_error {
                    run.task_results.push(result);
                    run.status = PeriodCloseStatus::Failed;
                    return run;
                }
            }

            run.task_results.push(result);
            _current_sequence = scheduled_task.sequence;
        }

        // Determine final status
        run.completed_at = Some(fiscal_period.end_date);
        if run.errors.is_empty() {
            run.status = PeriodCloseStatus::Completed;
        } else {
            run.status = PeriodCloseStatus::CompletedWithErrors;
        }

        run
    }

    /// Executes a single close task.
    fn execute_task(
        &self,
        task: &CloseTask,
        company_code: &str,
        fiscal_period: &FiscalPeriod,
        context: &mut CloseContext,
    ) -> CloseTaskResult {
        let mut result = CloseTaskResult::new(
            task.clone(),
            company_code.to_string(),
            fiscal_period.clone(),
        );
        result.status = CloseTaskStatus::InProgress;
        result.started_at = Some(fiscal_period.end_date);

        // Delegate to appropriate handler
        match task {
            CloseTask::RunDepreciation => {
                if let Some(handler) = &context.depreciation_handler {
                    let (entries, total) = handler(company_code, fiscal_period);
                    result.journal_entries_created = entries.len() as u32;
                    result.total_amount = total;
                    context.journal_entries.extend(entries);
                    result.status = CloseTaskStatus::Completed;
                } else {
                    result.status = CloseTaskStatus::Skipped("No depreciation handler".to_string());
                }
            }
            CloseTask::PostAccruedExpenses | CloseTask::PostAccruedRevenue => {
                if let Some(handler) = &context.accrual_handler {
                    let (entries, total) = handler(company_code, fiscal_period, task);
                    result.journal_entries_created = entries.len() as u32;
                    result.total_amount = total;
                    context.journal_entries.extend(entries);
                    result.status = CloseTaskStatus::Completed;
                } else {
                    result.status = CloseTaskStatus::Skipped("No accrual handler".to_string());
                }
            }
            CloseTask::PostPrepaidAmortization => {
                if let Some(handler) = &context.prepaid_handler {
                    let (entries, total) = handler(company_code, fiscal_period);
                    result.journal_entries_created = entries.len() as u32;
                    result.total_amount = total;
                    context.journal_entries.extend(entries);
                    result.status = CloseTaskStatus::Completed;
                } else {
                    result.status = CloseTaskStatus::Skipped("No prepaid handler".to_string());
                }
            }
            CloseTask::ReconcileArToGl
            | CloseTask::ReconcileApToGl
            | CloseTask::ReconcileFaToGl
            | CloseTask::ReconcileInventoryToGl => {
                if let Some(handler) = &context.reconciliation_handler {
                    match handler(company_code, fiscal_period, task) {
                        Ok(diff) => {
                            if diff.abs() <= self.config.reconciliation_tolerance {
                                result.status = CloseTaskStatus::Completed;
                            } else if self.config.require_reconciliation {
                                result.status = CloseTaskStatus::Failed(format!(
                                    "Reconciliation difference: {}",
                                    diff
                                ));
                            } else {
                                result.status =
                                    CloseTaskStatus::CompletedWithWarnings(vec![format!(
                                        "Reconciliation difference: {}",
                                        diff
                                    )]);
                            }
                            result.total_amount = diff;
                        }
                        Err(e) => {
                            result.status = CloseTaskStatus::Failed(e);
                        }
                    }
                } else {
                    result.status =
                        CloseTaskStatus::Skipped("No reconciliation handler".to_string());
                }
            }
            CloseTask::RevalueForeignCurrency => {
                if let Some(handler) = &context.fx_revaluation_handler {
                    let (entries, total) = handler(company_code, fiscal_period);
                    result.journal_entries_created = entries.len() as u32;
                    result.total_amount = total;
                    context.journal_entries.extend(entries);
                    result.status = CloseTaskStatus::Completed;
                } else {
                    result.status =
                        CloseTaskStatus::Skipped("No FX revaluation handler".to_string());
                }
            }
            CloseTask::AllocateCorporateOverhead => {
                if let Some(handler) = &context.overhead_handler {
                    let (entries, total) = handler(company_code, fiscal_period);
                    result.journal_entries_created = entries.len() as u32;
                    result.total_amount = total;
                    context.journal_entries.extend(entries);
                    result.status = CloseTaskStatus::Completed;
                } else {
                    result.status = CloseTaskStatus::Skipped("No overhead handler".to_string());
                }
            }
            CloseTask::PostIntercompanySettlements => {
                if let Some(handler) = &context.ic_settlement_handler {
                    let (entries, total) = handler(company_code, fiscal_period);
                    result.journal_entries_created = entries.len() as u32;
                    result.total_amount = total;
                    context.journal_entries.extend(entries);
                    result.status = CloseTaskStatus::Completed;
                } else {
                    result.status =
                        CloseTaskStatus::Skipped("No IC settlement handler".to_string());
                }
            }
            CloseTask::TranslateForeignSubsidiaries => {
                if let Some(handler) = &context.translation_handler {
                    let (entries, total) = handler(company_code, fiscal_period);
                    result.journal_entries_created = entries.len() as u32;
                    result.total_amount = total;
                    context.journal_entries.extend(entries);
                    result.status = CloseTaskStatus::Completed;
                } else {
                    result.status = CloseTaskStatus::Skipped("No translation handler".to_string());
                }
            }
            CloseTask::EliminateIntercompany => {
                if let Some(handler) = &context.elimination_handler {
                    let (entries, total) = handler(company_code, fiscal_period);
                    result.journal_entries_created = entries.len() as u32;
                    result.total_amount = total;
                    context.journal_entries.extend(entries);
                    result.status = CloseTaskStatus::Completed;
                } else {
                    result.status = CloseTaskStatus::Skipped("No elimination handler".to_string());
                }
            }
            CloseTask::CalculateTaxProvision => {
                if let Some(handler) = &context.tax_provision_handler {
                    let (entries, total) = handler(company_code, fiscal_period);
                    result.journal_entries_created = entries.len() as u32;
                    result.total_amount = total;
                    context.journal_entries.extend(entries);
                    result.status = CloseTaskStatus::Completed;
                } else {
                    result.status =
                        CloseTaskStatus::Skipped("No tax provision handler".to_string());
                }
            }
            CloseTask::CloseIncomeStatement => {
                if let Some(handler) = &context.income_close_handler {
                    let (entries, total) = handler(company_code, fiscal_period);
                    result.journal_entries_created = entries.len() as u32;
                    result.total_amount = total;
                    context.journal_entries.extend(entries);
                    result.status = CloseTaskStatus::Completed;
                } else {
                    result.status = CloseTaskStatus::Skipped("No income close handler".to_string());
                }
            }
            CloseTask::PostRetainedEarningsRollforward => {
                if let Some(handler) = &context.re_rollforward_handler {
                    let (entries, total) = handler(company_code, fiscal_period);
                    result.journal_entries_created = entries.len() as u32;
                    result.total_amount = total;
                    context.journal_entries.extend(entries);
                    result.status = CloseTaskStatus::Completed;
                } else {
                    result.status =
                        CloseTaskStatus::Skipped("No RE rollforward handler".to_string());
                }
            }
            CloseTask::GenerateTrialBalance | CloseTask::GenerateFinancialStatements => {
                // These are reporting tasks, not JE generators
                result.status = CloseTaskStatus::Completed;
                result.notes.push("Report generation completed".to_string());
            }
            CloseTask::PostInventoryRevaluation => {
                if let Some(handler) = &context.inventory_reval_handler {
                    let (entries, total) = handler(company_code, fiscal_period);
                    result.journal_entries_created = entries.len() as u32;
                    result.total_amount = total;
                    context.journal_entries.extend(entries);
                    result.status = CloseTaskStatus::Completed;
                } else {
                    result.status =
                        CloseTaskStatus::Skipped("No inventory reval handler".to_string());
                }
            }
            CloseTask::Custom(name) => {
                if let Some(handler) = context.custom_handlers.get(name) {
                    let (entries, total) = handler(company_code, fiscal_period);
                    result.journal_entries_created = entries.len() as u32;
                    result.total_amount = total;
                    context.journal_entries.extend(entries);
                    result.status = CloseTaskStatus::Completed;
                } else {
                    // Return error status instead of silently skipping
                    result.status = CloseTaskStatus::Failed(format!(
                        "Custom close task '{}' has no registered handler. \
                         Register a handler via CloseContext.custom_handlers.insert(\"{}\",...)",
                        name, name
                    ));
                }
            }
        }

        result.completed_at = Some(fiscal_period.end_date);
        result
    }

    /// Validates that a period can be closed.
    pub fn validate_close_readiness(
        &self,
        company_code: &str,
        fiscal_period: &FiscalPeriod,
        context: &CloseContext,
    ) -> CloseReadinessResult {
        let mut result = CloseReadinessResult {
            company_code: company_code.to_string(),
            fiscal_period: fiscal_period.clone(),
            is_ready: true,
            blockers: Vec::new(),
            warnings: Vec::new(),
        };

        // Check period status
        if fiscal_period.status == PeriodStatus::Closed {
            result.is_ready = false;
            result.blockers.push("Period is already closed".to_string());
        }

        if fiscal_period.status == PeriodStatus::Locked {
            result.is_ready = false;
            result
                .blockers
                .push("Period is locked for audit".to_string());
        }

        // Check for required handlers
        if context.depreciation_handler.is_none() {
            result
                .warnings
                .push("No depreciation handler configured".to_string());
        }

        if context.accrual_handler.is_none() {
            result
                .warnings
                .push("No accrual handler configured".to_string());
        }

        if self.config.require_reconciliation && context.reconciliation_handler.is_none() {
            result.is_ready = false;
            result
                .blockers
                .push("Reconciliation required but no handler configured".to_string());
        }

        result
    }
}

/// Context for close execution containing handlers and state.
#[derive(Default)]
pub struct CloseContext {
    /// Journal entries generated during close.
    pub journal_entries: Vec<JournalEntry>,
    /// Handler for depreciation.
    pub depreciation_handler:
        Option<Box<dyn Fn(&str, &FiscalPeriod) -> (Vec<JournalEntry>, Decimal)>>,
    /// Handler for accruals.
    pub accrual_handler:
        Option<Box<dyn Fn(&str, &FiscalPeriod, &CloseTask) -> (Vec<JournalEntry>, Decimal)>>,
    /// Handler for prepaid amortization.
    pub prepaid_handler: Option<Box<dyn Fn(&str, &FiscalPeriod) -> (Vec<JournalEntry>, Decimal)>>,
    /// Handler for reconciliation.
    pub reconciliation_handler:
        Option<Box<dyn Fn(&str, &FiscalPeriod, &CloseTask) -> Result<Decimal, String>>>,
    /// Handler for FX revaluation.
    pub fx_revaluation_handler:
        Option<Box<dyn Fn(&str, &FiscalPeriod) -> (Vec<JournalEntry>, Decimal)>>,
    /// Handler for overhead allocation.
    pub overhead_handler: Option<Box<dyn Fn(&str, &FiscalPeriod) -> (Vec<JournalEntry>, Decimal)>>,
    /// Handler for IC settlements.
    pub ic_settlement_handler:
        Option<Box<dyn Fn(&str, &FiscalPeriod) -> (Vec<JournalEntry>, Decimal)>>,
    /// Handler for currency translation.
    pub translation_handler:
        Option<Box<dyn Fn(&str, &FiscalPeriod) -> (Vec<JournalEntry>, Decimal)>>,
    /// Handler for IC elimination.
    pub elimination_handler:
        Option<Box<dyn Fn(&str, &FiscalPeriod) -> (Vec<JournalEntry>, Decimal)>>,
    /// Handler for tax provision.
    pub tax_provision_handler:
        Option<Box<dyn Fn(&str, &FiscalPeriod) -> (Vec<JournalEntry>, Decimal)>>,
    /// Handler for income statement close.
    pub income_close_handler:
        Option<Box<dyn Fn(&str, &FiscalPeriod) -> (Vec<JournalEntry>, Decimal)>>,
    /// Handler for retained earnings rollforward.
    pub re_rollforward_handler:
        Option<Box<dyn Fn(&str, &FiscalPeriod) -> (Vec<JournalEntry>, Decimal)>>,
    /// Handler for inventory revaluation.
    pub inventory_reval_handler:
        Option<Box<dyn Fn(&str, &FiscalPeriod) -> (Vec<JournalEntry>, Decimal)>>,
    /// Handlers for custom close tasks, keyed by task name.
    pub custom_handlers: std::collections::HashMap<
        String,
        Box<dyn Fn(&str, &FiscalPeriod) -> (Vec<JournalEntry>, Decimal)>,
    >,
}

/// Result of close readiness validation.
#[derive(Debug, Clone)]
pub struct CloseReadinessResult {
    /// Company code.
    pub company_code: String,
    /// Fiscal period.
    pub fiscal_period: FiscalPeriod,
    /// Whether the period is ready to close.
    pub is_ready: bool,
    /// Blocking issues that prevent close.
    pub blockers: Vec<String>,
    /// Non-blocking warnings.
    pub warnings: Vec<String>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_close_engine_creation() {
        let engine = CloseEngine::new(CloseEngineConfig::default());
        assert!(!engine.config.stop_on_error);
    }

    #[test]
    fn test_close_readiness() {
        let engine = CloseEngine::new(CloseEngineConfig::default());
        let period = FiscalPeriod::monthly(2024, 1);
        let context = CloseContext::default();

        let result = engine.validate_close_readiness("1000", &period, &context);
        // Without reconciliation handler, should not be ready
        assert!(!result.is_ready);
    }
}
