//! Multi-period generation session with checkpoint/resume support.
//!
//! [`GenerationSession`] wraps [`EnhancedOrchestrator`] and drives it through
//! a sequence of [`GenerationPeriod`]s, persisting state to `.dss` files so
//! that long runs can be resumed after interruption.

use std::fs;
use std::path::{Path, PathBuf};

use datasynth_config::GeneratorConfig;
use datasynth_core::models::generation_session::{
    add_months, advance_seed, BalanceState, DocumentIdState, EntityCounts, GenerationPeriod,
    PeriodLog, SessionState,
};
use datasynth_core::SynthError;

use crate::enhanced_orchestrator::{EnhancedOrchestrator, PhaseConfig};

type SynthResult<T> = Result<T, SynthError>;

/// Controls how period output directories are laid out.
#[derive(Debug, Clone)]
pub enum OutputMode {
    /// Single output directory (one period).
    Batch(PathBuf),
    /// One sub-directory per period under a root directory.
    MultiPeriod(PathBuf),
}

/// Summary of a single completed period generation.
#[derive(Debug)]
pub struct PeriodResult {
    /// The period that was generated.
    pub period: GenerationPeriod,
    /// Filesystem path where this period's output was written.
    pub output_path: PathBuf,
    /// Number of journal entries generated in this period.
    pub journal_entry_count: usize,
    /// Number of document flow records generated in this period.
    pub document_count: usize,
    /// Number of anomalies injected in this period.
    pub anomaly_count: usize,
    /// Wall-clock duration for generating this period (seconds).
    pub duration_secs: f64,
}

/// A multi-period generation session with checkpoint/resume support.
///
/// The session decomposes the total requested time span into fiscal-year-aligned
/// periods and generates each one sequentially, carrying forward balance and ID
/// state between periods.
#[derive(Debug)]
pub struct GenerationSession {
    config: GeneratorConfig,
    state: SessionState,
    periods: Vec<GenerationPeriod>,
    output_mode: OutputMode,
    phase_config: PhaseConfig,
}

impl GenerationSession {
    /// Create a new session from a config and output path.
    ///
    /// The total time span is decomposed into fiscal-year-aligned periods
    /// based on `config.global.fiscal_year_months` (defaults to `period_months`
    /// if not set, yielding a single period).
    pub fn new(config: GeneratorConfig, output_path: PathBuf) -> SynthResult<Self> {
        let start_date = chrono::NaiveDate::parse_from_str(&config.global.start_date, "%Y-%m-%d")
            .map_err(|e| SynthError::generation(format!("Invalid start_date: {e}")))?;

        let total_months = config.global.period_months;
        let fy_months = config.global.fiscal_year_months.unwrap_or(total_months);
        let periods = GenerationPeriod::compute_periods(start_date, total_months, fy_months);

        let output_mode = if periods.len() > 1 {
            OutputMode::MultiPeriod(output_path)
        } else {
            OutputMode::Batch(output_path)
        };

        let seed = config.global.seed.unwrap_or(42);
        let config_hash = Self::compute_config_hash(&config);

        let state = SessionState {
            rng_seed: seed,
            period_cursor: 0,
            balance_state: BalanceState::default(),
            document_id_state: DocumentIdState::default(),
            entity_counts: EntityCounts::default(),
            generation_log: Vec::new(),
            config_hash,
        };

        Ok(Self {
            config,
            state,
            periods,
            output_mode,
            phase_config: PhaseConfig::default(),
        })
    }

    /// Resume a session from a `.dss` checkpoint file.
    ///
    /// The config hash is verified against the checkpoint to ensure the config
    /// has not changed since the session was last saved.
    pub fn resume(path: &Path, config: GeneratorConfig) -> SynthResult<Self> {
        let data = fs::read_to_string(path)
            .map_err(|e| SynthError::generation(format!("Failed to read .dss: {e}")))?;
        let state: SessionState = serde_json::from_str(&data)
            .map_err(|e| SynthError::generation(format!("Failed to parse .dss: {e}")))?;

        let current_hash = Self::compute_config_hash(&config);
        if state.config_hash != current_hash {
            return Err(SynthError::generation(
                "Config has changed since last checkpoint. Cannot resume with different config."
                    .to_string(),
            ));
        }

        let start_date = chrono::NaiveDate::parse_from_str(&config.global.start_date, "%Y-%m-%d")
            .map_err(|e| SynthError::generation(format!("Invalid start_date: {e}")))?;

        let total_months = config.global.period_months;
        let fy_months = config.global.fiscal_year_months.unwrap_or(total_months);
        let periods = GenerationPeriod::compute_periods(start_date, total_months, fy_months);

        let output_dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        let output_mode = if periods.len() > 1 {
            OutputMode::MultiPeriod(output_dir)
        } else {
            OutputMode::Batch(output_dir)
        };

        Ok(Self {
            config,
            state,
            periods,
            output_mode,
            phase_config: PhaseConfig::default(),
        })
    }

    /// Persist the current session state to a `.dss` file.
    pub fn save(&self, path: &Path) -> SynthResult<()> {
        let data = serde_json::to_string_pretty(&self.state)
            .map_err(|e| SynthError::generation(format!("Failed to serialize state: {e}")))?;
        fs::write(path, data)
            .map_err(|e| SynthError::generation(format!("Failed to write .dss: {e}")))?;
        Ok(())
    }

    /// Generate the next period in the sequence.
    ///
    /// Returns `Ok(None)` if all periods have been generated.
    pub fn generate_next_period(&mut self) -> SynthResult<Option<PeriodResult>> {
        if self.state.period_cursor >= self.periods.len() {
            return Ok(None);
        }

        let period = self.periods[self.state.period_cursor].clone();
        let start = std::time::Instant::now();

        let period_seed = advance_seed(self.state.rng_seed, period.index);

        let mut period_config = self.config.clone();
        period_config.global.start_date = period.start_date.format("%Y-%m-%d").to_string();
        period_config.global.period_months = period.months;
        period_config.global.seed = Some(period_seed);

        let output_path = match &self.output_mode {
            OutputMode::Batch(p) => p.clone(),
            OutputMode::MultiPeriod(p) => p.join(&period.label),
        };

        fs::create_dir_all(&output_path)
            .map_err(|e| SynthError::generation(format!("Failed to create output dir: {e}")))?;

        let orchestrator = EnhancedOrchestrator::new(period_config, self.phase_config.clone())?;
        let mut orchestrator = orchestrator.with_output_path(&output_path);
        let result = orchestrator.generate()?;

        let duration = start.elapsed().as_secs_f64();

        // Count journal entries from the result vec
        let je_count = result.journal_entries.len();

        // Count documents from the document_flows snapshot
        let doc_count = result.document_flows.purchase_orders.len()
            + result.document_flows.sales_orders.len()
            + result.document_flows.goods_receipts.len()
            + result.document_flows.vendor_invoices.len()
            + result.document_flows.customer_invoices.len()
            + result.document_flows.deliveries.len()
            + result.document_flows.payments.len();

        // Count anomalies from anomaly_labels
        let anomaly_count = result.anomaly_labels.labels.len();

        self.state.generation_log.push(PeriodLog {
            period_label: period.label.clone(),
            journal_entries: je_count,
            documents: doc_count,
            anomalies: anomaly_count,
            duration_secs: duration,
        });

        self.state.period_cursor += 1;

        Ok(Some(PeriodResult {
            period,
            output_path,
            journal_entry_count: je_count,
            document_count: doc_count,
            anomaly_count,
            duration_secs: duration,
        }))
    }

    /// Generate all remaining periods in the sequence.
    pub fn generate_all(&mut self) -> SynthResult<Vec<PeriodResult>> {
        let mut results = Vec::new();
        while let Some(result) = self.generate_next_period()? {
            results.push(result);
        }
        Ok(results)
    }

    /// Extend the session with additional months and generate them.
    pub fn generate_delta(&mut self, additional_months: u32) -> SynthResult<Vec<PeriodResult>> {
        let last_end = if let Some(last_period) = self.periods.last() {
            add_months(last_period.end_date, 1)
        } else {
            chrono::NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
                .map_err(|e| SynthError::generation(format!("Invalid start_date: {e}")))?
        };

        let fy_months = self
            .config
            .global
            .fiscal_year_months
            .unwrap_or(self.config.global.period_months);
        let new_periods = GenerationPeriod::compute_periods(last_end, additional_months, fy_months);

        let base_index = self.periods.len();
        let new_periods: Vec<GenerationPeriod> = new_periods
            .into_iter()
            .enumerate()
            .map(|(i, mut p)| {
                p.index = base_index + i;
                p
            })
            .collect();

        self.periods.extend(new_periods);
        self.generate_all()
    }

    /// Read-only access to the session state.
    pub fn state(&self) -> &SessionState {
        &self.state
    }

    /// Read-only access to the period list.
    pub fn periods(&self) -> &[GenerationPeriod] {
        &self.periods
    }

    /// Number of periods that have not yet been generated.
    pub fn remaining_periods(&self) -> usize {
        self.periods.len().saturating_sub(self.state.period_cursor)
    }

    /// Compute a hash of the config for drift detection.
    fn compute_config_hash(config: &GeneratorConfig) -> String {
        use std::hash::{Hash, Hasher};
        let json = serde_json::to_string(config).unwrap_or_default();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        json.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn minimal_config() -> GeneratorConfig {
        serde_yaml::from_str(
            r#"
global:
  seed: 42
  industry: retail
  start_date: "2024-01-01"
  period_months: 12
companies:
  - code: "C001"
    name: "Test Corp"
    currency: "USD"
    country: "US"
    annual_transaction_volume: ten_k
chart_of_accounts:
  complexity: small
output:
  output_directory: "./output"
"#,
        )
        .expect("minimal config should parse")
    }

    #[test]
    fn test_session_new_single_period() {
        let config = minimal_config();
        let session =
            GenerationSession::new(config, PathBuf::from("/tmp/test_session_single")).unwrap();
        assert_eq!(session.periods().len(), 1);
        assert_eq!(session.remaining_periods(), 1);
    }

    #[test]
    fn test_session_new_multi_period() {
        let mut config = minimal_config();
        config.global.period_months = 36;
        config.global.fiscal_year_months = Some(12);
        let session =
            GenerationSession::new(config, PathBuf::from("/tmp/test_session_multi")).unwrap();
        assert_eq!(session.periods().len(), 3);
        assert_eq!(session.remaining_periods(), 3);
    }

    #[test]
    fn test_session_save_and_resume() {
        let config = minimal_config();
        let session =
            GenerationSession::new(config.clone(), PathBuf::from("/tmp/test_session_save"))
                .unwrap();
        let tmp = std::env::temp_dir().join("test_gen_session.dss");
        session.save(&tmp).unwrap();
        let resumed = GenerationSession::resume(&tmp, config).unwrap();
        assert_eq!(resumed.state().period_cursor, 0);
        assert_eq!(resumed.state().rng_seed, 42);
        let _ = fs::remove_file(&tmp);
    }

    #[test]
    fn test_session_resume_config_mismatch() {
        let config = minimal_config();
        let session =
            GenerationSession::new(config.clone(), PathBuf::from("/tmp/test_session_mismatch"))
                .unwrap();
        let tmp = std::env::temp_dir().join("test_gen_session_mismatch.dss");
        session.save(&tmp).unwrap();
        let mut different = config;
        different.global.seed = Some(999);
        let result = GenerationSession::resume(&tmp, different);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Config has changed"),
            "Expected config drift error, got: {}",
            err_msg
        );
        let _ = fs::remove_file(&tmp);
    }

    #[test]
    fn test_session_remaining_periods() {
        let config = minimal_config();
        let session =
            GenerationSession::new(config, PathBuf::from("/tmp/test_session_remaining")).unwrap();
        assert_eq!(session.remaining_periods(), 1);
    }

    #[test]
    fn test_session_config_hash_deterministic() {
        let config = minimal_config();
        let h1 = GenerationSession::compute_config_hash(&config);
        let h2 = GenerationSession::compute_config_hash(&config);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_session_config_hash_changes_on_mutation() {
        let config = minimal_config();
        let h1 = GenerationSession::compute_config_hash(&config);
        let mut modified = config;
        modified.global.seed = Some(999);
        let h2 = GenerationSession::compute_config_hash(&modified);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_session_output_mode_batch_for_single_period() {
        let config = minimal_config();
        let session =
            GenerationSession::new(config, PathBuf::from("/tmp/test_batch_mode")).unwrap();
        assert!(matches!(session.output_mode, OutputMode::Batch(_)));
    }

    #[test]
    fn test_session_output_mode_multi_for_multiple_periods() {
        let mut config = minimal_config();
        config.global.period_months = 24;
        config.global.fiscal_year_months = Some(12);
        let session =
            GenerationSession::new(config, PathBuf::from("/tmp/test_multi_mode")).unwrap();
        assert!(matches!(session.output_mode, OutputMode::MultiPeriod(_)));
    }
}
