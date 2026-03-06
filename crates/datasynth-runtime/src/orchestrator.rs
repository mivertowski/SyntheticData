//! Generation orchestrator for coordinating data generation.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use datasynth_config::schema::GeneratorConfig;
use datasynth_core::error::{SynthError, SynthResult};
use datasynth_core::models::*;
use datasynth_core::traits::{Generator, ParallelGenerator};
use datasynth_generators::{ChartOfAccountsGenerator, JournalEntryGenerator};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

/// Result of a generation run.
pub struct GenerationResult {
    /// Generated chart of accounts
    pub chart_of_accounts: ChartOfAccounts,
    /// Generated journal entries
    pub journal_entries: Vec<JournalEntry>,
    /// Statistics about the generation
    pub statistics: GenerationStatistics,
}

/// Statistics about a generation run.
#[derive(Debug, Clone)]
pub struct GenerationStatistics {
    /// Total journal entries generated
    pub total_entries: u64,
    /// Total line items generated
    pub total_line_items: u64,
    /// Number of accounts in CoA
    pub accounts_count: usize,
    /// Number of companies
    pub companies_count: usize,
    /// Period in months
    pub period_months: u32,
}

/// Main orchestrator for generation.
pub struct GenerationOrchestrator {
    config: GeneratorConfig,
    coa: Option<Arc<ChartOfAccounts>>,
    /// Optional pause flag for external control (e.g., signal handlers).
    pause_flag: Option<Arc<AtomicBool>>,
}

impl GenerationOrchestrator {
    /// Create a new orchestrator.
    pub fn new(config: GeneratorConfig) -> SynthResult<Self> {
        // Validate config
        datasynth_config::validate_config(&config)?;

        Ok(Self {
            config,
            coa: None,
            pause_flag: None,
        })
    }

    /// Set a pause flag that can be controlled externally (e.g., by a signal handler).
    /// When the flag is true, generation will pause until it becomes false.
    pub fn with_pause_flag(mut self, flag: Arc<AtomicBool>) -> Self {
        self.pause_flag = Some(flag);
        self
    }

    /// Check if generation is currently paused.
    fn is_paused(&self) -> bool {
        self.pause_flag
            .as_ref()
            .map(|f| f.load(Ordering::Relaxed))
            .unwrap_or(false)
    }

    /// Wait while paused, checking periodically.
    fn wait_while_paused(&self, pb: &ProgressBar) {
        let was_paused = self.is_paused();
        if was_paused {
            pb.set_message("PAUSED - send SIGUSR1 to resume");
        }

        while self.is_paused() {
            std::thread::sleep(Duration::from_millis(100));
        }

        if was_paused {
            pb.set_message("");
        }
    }

    /// Generate the chart of accounts.
    pub fn generate_coa(&mut self) -> SynthResult<Arc<ChartOfAccounts>> {
        let seed = self.config.global.seed.unwrap_or_else(rand::random);
        let mut gen = ChartOfAccountsGenerator::new(
            self.config.chart_of_accounts.complexity,
            self.config.global.industry,
            seed,
        );

        let coa = Arc::new(gen.generate());
        self.coa = Some(Arc::clone(&coa));
        Ok(coa)
    }

    /// Calculate total transactions to generate.
    pub fn calculate_total_transactions(&self) -> u64 {
        let months = self.config.global.period_months as f64;

        self.config
            .companies
            .iter()
            .map(|c| {
                let annual = c.annual_transaction_volume.count() as f64;
                let weighted = annual * c.volume_weight;
                (weighted * months / 12.0) as u64
            })
            .sum()
    }

    /// Run the generation.
    pub fn generate(&mut self) -> SynthResult<GenerationResult> {
        // Generate CoA if not already done
        let coa = match &self.coa {
            Some(c) => Arc::clone(c),
            None => self.generate_coa()?,
        };

        let total = self.calculate_total_transactions();
        let seed = self.config.global.seed.unwrap_or_else(rand::random);

        // Parse dates
        let start_date =
            chrono::NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
                .map_err(|e| SynthError::config(format!("Invalid start_date: {e}")))?;

        let end_date = start_date + chrono::Months::new(self.config.global.period_months);

        // Create progress bar
        let pb = ProgressBar::new(total);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({per_sec})")
                .expect("Progress bar template is a compile-time constant and should always be valid")
                .progress_chars("#>-"),
        );

        // Get company codes
        let company_codes: Vec<String> = self
            .config
            .companies
            .iter()
            .map(|c| c.code.clone())
            .collect();

        // Generate entries with fraud config
        let mut generator = JournalEntryGenerator::new_with_params(
            self.config.transactions.clone(),
            Arc::clone(&coa),
            company_codes,
            start_date,
            end_date,
            seed,
        )
        .with_fraud_config(self.config.fraud.clone());

        // Parallel generation: split across available cores for large datasets
        let num_threads = num_cpus::get().max(1).min(total as usize).max(1);

        let entries = if total >= 10_000 && num_threads > 1 {
            let sub_generators = generator.split(num_threads);
            let entries_per_thread = total as usize / num_threads;
            let remainder = total as usize % num_threads;

            let batches: Vec<Vec<JournalEntry>> = sub_generators
                .into_par_iter()
                .enumerate()
                .map(|(i, mut gen)| {
                    let count = entries_per_thread + if i < remainder { 1 } else { 0 };
                    gen.generate_batch(count)
                })
                .collect();

            let entries = JournalEntryGenerator::merge_results(batches);
            pb.inc(total);
            entries
        } else {
            let mut entries = Vec::with_capacity(total as usize);
            for _ in 0..total {
                self.wait_while_paused(&pb);
                let entry = generator.generate();
                entries.push(entry);
                pb.inc(1);
            }
            entries
        };

        let total_lines: u64 = entries.iter().map(|e| e.line_count() as u64).sum();

        pb.finish_with_message("Generation complete");

        Ok(GenerationResult {
            chart_of_accounts: (*coa).clone(),
            journal_entries: entries,
            statistics: GenerationStatistics {
                total_entries: total,
                total_line_items: total_lines,
                accounts_count: coa.account_count(),
                companies_count: self.config.companies.len(),
                period_months: self.config.global.period_months,
            },
        })
    }
}
