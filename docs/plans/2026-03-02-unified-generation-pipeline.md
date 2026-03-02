# Unified Generation Pipeline — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the single-shot generation model with a stateful `GenerationSession` that generates periods incrementally, streams as it goes, supports fraud scenario packs, and produces enriched OCEL 2.0 events — per the design at `docs/plans/2026-03-02-unified-generation-pipeline-design.md`.

**Architecture:** A `GenerationSession` wraps the existing `EnhancedOrchestrator`, calling it per-period with adjusted configs (date range, opening balances, seed advancement). SessionState is serializable to `.dss` checkpoint files. The StreamPipeline uses a `PhaseSink` trait woven into generation phases. Fraud packs are YAML fragments merged into configs. OCEL enrichment adds lifecycle state machines and multi-object correlation events.

**Spec:** `docs/plans/2026-03-02-unified-generation-pipeline-design.md`

---

## Phase 1: Session Foundation (datasynth-core + datasynth-runtime)

### Task 1: FiscalPeriod and SessionState Models

**Files:**
- Create: `crates/datasynth-core/src/models/generation_session.rs`
- Modify: `crates/datasynth-core/src/models/mod.rs`

**Step 1:** Create `generation_session.rs` with core session types:

```rust
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single fiscal period within a multi-period generation session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiscalPeriod {
    pub index: usize,
    pub label: String,              // e.g. "FY2022"
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub months: u32,
}

impl FiscalPeriod {
    pub fn compute_periods(
        start_date: NaiveDate,
        total_months: u32,
        fiscal_year_months: u32,
    ) -> Vec<FiscalPeriod> {
        let mut periods = Vec::new();
        let mut current = start_date;
        let mut remaining = total_months;
        let mut index = 0;

        while remaining > 0 {
            let months = remaining.min(fiscal_year_months);
            let end = add_months(current, months).pred_opt().unwrap_or(current);
            let year = current.format("%Y").to_string();
            periods.push(FiscalPeriod {
                index,
                label: format!("FY{}", year),
                start_date: current,
                end_date: end,
                months,
            });
            current = add_months(current, months);
            remaining -= months;
            index += 1;
        }
        periods
    }
}

fn add_months(date: NaiveDate, months: u32) -> NaiveDate {
    let total_months = date.month0() as i32 + months as i32;
    let year = date.year() + total_months / 12;
    let month = (total_months % 12) as u32 + 1;
    NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(date)
}

/// Mutable state tracked across generation periods, serializable to `.dss`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub rng_seed: u64,
    pub period_cursor: usize,
    pub balance_state: BalanceState,
    pub document_id_state: DocumentIdState,
    pub entity_counts: EntityCounts,
    pub generation_log: Vec<PeriodLog>,
    pub config_hash: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BalanceState {
    pub gl_balances: HashMap<String, f64>,
    pub ar_total: f64,
    pub ap_total: f64,
    pub fa_net_book_value: f64,
    pub retained_earnings: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentIdState {
    pub next_po_number: u64,
    pub next_so_number: u64,
    pub next_je_number: u64,
    pub next_invoice_number: u64,
    pub next_payment_number: u64,
    pub next_gr_number: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntityCounts {
    pub vendors: usize,
    pub customers: usize,
    pub employees: usize,
    pub materials: usize,
    pub fixed_assets: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodLog {
    pub period_label: String,
    pub journal_entries: usize,
    pub documents: usize,
    pub anomalies: usize,
    pub duration_secs: f64,
}

/// Deterministic seed advancement: seed_n+1 = hash(seed_n, period_index)
pub fn advance_seed(seed: u64, period_index: usize) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    seed.hash(&mut hasher);
    period_index.hash(&mut hasher);
    hasher.finish()
}
```

**Step 2:** Update `crates/datasynth-core/src/models/mod.rs` — add module and re-export:
```rust
pub mod generation_session;
pub use generation_session::*;
```

**Step 3:** Add unit tests at bottom of `generation_session.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_periods_single_year() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let periods = FiscalPeriod::compute_periods(start, 12, 12);
        assert_eq!(periods.len(), 1);
        assert_eq!(periods[0].label, "FY2024");
        assert_eq!(periods[0].months, 12);
    }

    #[test]
    fn test_compute_periods_three_years() {
        let start = NaiveDate::from_ymd_opt(2022, 1, 1).unwrap();
        let periods = FiscalPeriod::compute_periods(start, 36, 12);
        assert_eq!(periods.len(), 3);
        assert_eq!(periods[0].label, "FY2022");
        assert_eq!(periods[1].label, "FY2023");
        assert_eq!(periods[2].label, "FY2024");
    }

    #[test]
    fn test_compute_periods_partial() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let periods = FiscalPeriod::compute_periods(start, 18, 12);
        assert_eq!(periods.len(), 2);
        assert_eq!(periods[0].months, 12);
        assert_eq!(periods[1].months, 6);
    }

    #[test]
    fn test_advance_seed_deterministic() {
        let s1 = advance_seed(42, 0);
        let s2 = advance_seed(42, 0);
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_advance_seed_differs_by_index() {
        let s1 = advance_seed(42, 0);
        let s2 = advance_seed(42, 1);
        assert_ne!(s1, s2);
    }

    #[test]
    fn test_session_state_serde_roundtrip() {
        let state = SessionState {
            rng_seed: 42,
            period_cursor: 1,
            balance_state: BalanceState::default(),
            document_id_state: DocumentIdState::default(),
            entity_counts: EntityCounts::default(),
            generation_log: vec![],
            config_hash: "abc123".to_string(),
        };
        let json = serde_json::to_string(&state).unwrap();
        let restored: SessionState = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.rng_seed, 42);
        assert_eq!(restored.period_cursor, 1);
    }

    #[test]
    fn test_balance_state_default() {
        let bs = BalanceState::default();
        assert_eq!(bs.gl_balances.len(), 0);
        assert_eq!(bs.ar_total, 0.0);
    }
}
```

**Step 4:** `cargo check -p datasynth-core && cargo test -p datasynth-core -- generation_session`

**Step 5:** Commit: `feat(core): add FiscalPeriod, SessionState, and seed advancement models`

---

### Task 2: SessionConfig Schema Extension

**Files:**
- Modify: `crates/datasynth-config/src/schema.rs`
- Modify: `crates/datasynth-config/src/validation.rs`

**Step 1:** Add session-related fields to `GlobalConfig` in `schema.rs` (after `memory_limit_mb` field around line 1189):
```rust
    /// Number of months per fiscal year for multi-period generation (default: same as period_months)
    #[serde(default)]
    pub fiscal_year_months: Option<u32>,
```

**Step 2:** Add `SessionSchemaConfig` after `GlobalConfig`:
```rust
/// Configuration for generation session behavior
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionSchemaConfig {
    /// Enable session-based generation with checkpointing
    #[serde(default)]
    pub enabled: bool,
    /// Path to save/load .dss checkpoint files
    #[serde(default)]
    pub checkpoint_path: Option<String>,
    /// Enable per-period output directories (FY2022/, FY2023/, etc.)
    #[serde(default = "default_true")]
    pub per_period_output: bool,
    /// Generate consolidated output across all periods
    #[serde(default = "default_true")]
    pub consolidated_output: bool,
}
```

**Step 3:** Add `session: SessionSchemaConfig` field to `GeneratorConfig` with `#[serde(default)]`.

**Step 4:** In `validation.rs`, add validation:
```rust
fn validate_session(config: &GeneratorConfig) -> Vec<ValidationWarning> {
    let mut warnings = Vec::new();
    if let Some(fy_months) = config.global.fiscal_year_months {
        if fy_months == 0 || fy_months > 120 {
            warnings.push(ValidationWarning::new(
                "global.fiscal_year_months",
                "Must be between 1 and 120",
            ));
        }
        if config.global.period_months < fy_months {
            warnings.push(ValidationWarning::new(
                "global.fiscal_year_months",
                "fiscal_year_months should not exceed period_months",
            ));
        }
    }
    warnings
}
```

Wire into `validate_config()`.

**Step 5:** Add tests:
```rust
#[test]
fn test_session_config_default_disabled() {
    let config: SessionSchemaConfig = serde_yaml::from_str("{}").unwrap();
    assert!(!config.enabled);
    assert!(config.per_period_output);
    assert!(config.consolidated_output);
}

#[test]
fn test_config_backward_compatible_without_session() {
    // Existing minimal config parses without session field
    let yaml = r#"
global:
  seed: 42
  industry: retail
  start_date: "2024-01-01"
  period_months: 12
companies: []
"#;
    let config: GeneratorConfig = serde_yaml::from_str(yaml).unwrap();
    assert!(!config.session.enabled);
}

#[test]
fn test_fiscal_year_months_parsed() {
    let yaml = r#"
global:
  seed: 42
  industry: retail
  start_date: "2024-01-01"
  period_months: 36
  fiscal_year_months: 12
companies: []
"#;
    let config: GeneratorConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.global.fiscal_year_months, Some(12));
}
```

**Step 6:** `cargo check -p datasynth-config && cargo test -p datasynth-config -- session`

**Step 7:** Commit: `feat(config): add SessionSchemaConfig and fiscal_year_months to GlobalConfig`

---

### Task 3: GenerationSession Runtime

**Files:**
- Create: `crates/datasynth-runtime/src/generation_session.rs`
- Modify: `crates/datasynth-runtime/src/lib.rs`

**Step 1:** Create `generation_session.rs` — the core `GenerationSession` struct:

```rust
use std::path::{Path, PathBuf};
use std::fs;
use datasynth_config::GeneratorConfig;
use datasynth_core::models::{
    FiscalPeriod, SessionState, BalanceState, DocumentIdState,
    EntityCounts, PeriodLog, advance_seed,
};
use datasynth_core::SynthError;
use crate::enhanced_orchestrator::{EnhancedOrchestrator, EnhancedGenerationResult, PhaseConfig};

type SynthResult<T> = Result<T, SynthError>;

/// Output mode for the generation session.
#[derive(Debug, Clone)]
pub enum OutputMode {
    /// Write all output to a directory
    Batch(PathBuf),
    /// Batch + per-period subdirectories
    MultiPeriod(PathBuf),
}

/// Result from generating a single period.
#[derive(Debug)]
pub struct PeriodResult {
    pub period: FiscalPeriod,
    pub output_path: PathBuf,
    pub journal_entry_count: usize,
    pub document_count: usize,
    pub anomaly_count: usize,
    pub duration_secs: f64,
}

/// Stateful generation session that calls EnhancedOrchestrator per-period.
pub struct GenerationSession {
    config: GeneratorConfig,
    state: SessionState,
    periods: Vec<FiscalPeriod>,
    output_mode: OutputMode,
    phase_config: PhaseConfig,
}

impl GenerationSession {
    /// Create a new session from config, computing fiscal periods.
    pub fn new(config: GeneratorConfig, output_path: PathBuf) -> SynthResult<Self> {
        let start_date = chrono::NaiveDate::parse_from_str(
            &config.global.start_date, "%Y-%m-%d"
        ).map_err(|e| SynthError::generation(format!("Invalid start_date: {}", e)))?;

        let total_months = config.global.period_months;
        let fy_months = config.global.fiscal_year_months.unwrap_or(total_months);

        let periods = FiscalPeriod::compute_periods(start_date, total_months, fy_months);

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

    /// Resume from a saved .dss checkpoint file.
    pub fn resume(path: &Path, config: GeneratorConfig) -> SynthResult<Self> {
        let data = fs::read_to_string(path)
            .map_err(|e| SynthError::generation(format!("Failed to read .dss: {}", e)))?;
        let state: SessionState = serde_json::from_str(&data)
            .map_err(|e| SynthError::generation(format!("Failed to parse .dss: {}", e)))?;

        // Validate config hash matches
        let current_hash = Self::compute_config_hash(&config);
        if state.config_hash != current_hash {
            return Err(SynthError::generation(
                "Config has changed since last checkpoint. Cannot resume with different config."
                    .to_string(),
            ));
        }

        let start_date = chrono::NaiveDate::parse_from_str(
            &config.global.start_date, "%Y-%m-%d"
        ).map_err(|e| SynthError::generation(format!("Invalid start_date: {}", e)))?;

        let total_months = config.global.period_months;
        let fy_months = config.global.fiscal_year_months.unwrap_or(total_months);
        let periods = FiscalPeriod::compute_periods(start_date, total_months, fy_months);

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

    /// Save session state to a .dss checkpoint file.
    pub fn save(&self, path: &Path) -> SynthResult<()> {
        let data = serde_json::to_string_pretty(&self.state)
            .map_err(|e| SynthError::generation(format!("Failed to serialize state: {}", e)))?;
        fs::write(path, data)
            .map_err(|e| SynthError::generation(format!("Failed to write .dss: {}", e)))?;
        Ok(())
    }

    /// Generate the next fiscal period, advancing the cursor.
    pub fn generate_next_period(&mut self) -> SynthResult<Option<PeriodResult>> {
        if self.state.period_cursor >= self.periods.len() {
            return Ok(None);
        }

        let period = self.periods[self.state.period_cursor].clone();
        let start = std::time::Instant::now();

        // Compute seed for this period
        let period_seed = advance_seed(self.state.rng_seed, period.index);

        // Build per-period config with adjusted date range and seed
        let mut period_config = self.config.clone();
        period_config.global.start_date = period.start_date.format("%Y-%m-%d").to_string();
        period_config.global.period_months = period.months;
        period_config.global.seed = Some(period_seed);

        // Determine output path for this period
        let output_path = match &self.output_mode {
            OutputMode::Batch(p) => p.clone(),
            OutputMode::MultiPeriod(p) => p.join(&period.label),
        };

        fs::create_dir_all(&output_path)
            .map_err(|e| SynthError::generation(format!("Failed to create output dir: {}", e)))?;

        // Run generation via EnhancedOrchestrator
        let mut orchestrator = EnhancedOrchestrator::new(
            period_config, self.phase_config.clone()
        )?;
        let orchestrator = orchestrator.with_output_path(&output_path);
        let result = orchestrator.generate()?;

        let duration = start.elapsed().as_secs_f64();

        let je_count = result.journal_entries.len();
        let doc_count = result.document_flows.purchase_orders.len()
            + result.document_flows.sales_orders.len();
        let anomaly_count = result.anomaly_labels.labels.len();

        // Log this period
        self.state.generation_log.push(PeriodLog {
            period_label: period.label.clone(),
            journal_entries: je_count,
            documents: doc_count,
            anomalies: anomaly_count,
            duration_secs: duration,
        });

        // Advance cursor and seed
        self.state.period_cursor += 1;

        let period_result = PeriodResult {
            period,
            output_path,
            journal_entry_count: je_count,
            document_count: doc_count,
            anomaly_count: anomaly_count,
            duration_secs: duration,
        };

        Ok(Some(period_result))
    }

    /// Generate all remaining periods.
    pub fn generate_all(&mut self) -> SynthResult<Vec<PeriodResult>> {
        let mut results = Vec::new();
        while let Some(result) = self.generate_next_period()? {
            results.push(result);
        }
        Ok(results)
    }

    /// Append N more months of data (incremental delta generation).
    pub fn generate_delta(&mut self, additional_months: u32) -> SynthResult<Vec<PeriodResult>> {
        let last_end = if let Some(last_period) = self.periods.last() {
            datasynth_core::models::generation_session::add_months(
                last_period.end_date, 1
            )
        } else {
            chrono::NaiveDate::parse_from_str(&self.config.global.start_date, "%Y-%m-%d")
                .map_err(|e| SynthError::generation(format!("Invalid start_date: {}", e)))?
        };

        let fy_months = self.config.global.fiscal_year_months
            .unwrap_or(self.config.global.period_months);
        let new_periods = FiscalPeriod::compute_periods(last_end, additional_months, fy_months);

        // Re-index new periods continuing from where we left off
        let base_index = self.periods.len();
        let new_periods: Vec<FiscalPeriod> = new_periods.into_iter().enumerate().map(|(i, mut p)| {
            p.index = base_index + i;
            p
        }).collect();

        self.periods.extend(new_periods);
        self.generate_all()
    }

    /// Get current session state for inspection.
    pub fn state(&self) -> &SessionState { &self.state }

    /// Get all planned fiscal periods.
    pub fn periods(&self) -> &[FiscalPeriod] { &self.periods }

    /// Get remaining period count.
    pub fn remaining_periods(&self) -> usize {
        self.periods.len().saturating_sub(self.state.period_cursor)
    }

    fn compute_config_hash(config: &GeneratorConfig) -> String {
        use std::hash::{Hash, Hasher};
        let json = serde_json::to_string(config).unwrap_or_default();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        json.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }
}
```

**Step 2:** Register in `crates/datasynth-runtime/src/lib.rs`:
```rust
pub mod generation_session;
pub use generation_session::*;
```

**Step 3:** Add tests at bottom of `generation_session.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_config() -> GeneratorConfig {
        let yaml = r#"
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
chart_of_accounts:
  complexity: small
"#;
        serde_yaml::from_str(yaml).unwrap()
    }

    #[test]
    fn test_session_new_single_period() {
        let config = minimal_config();
        let session = GenerationSession::new(config, PathBuf::from("/tmp/test")).unwrap();
        assert_eq!(session.periods().len(), 1);
        assert_eq!(session.remaining_periods(), 1);
    }

    #[test]
    fn test_session_new_multi_period() {
        let mut config = minimal_config();
        config.global.period_months = 36;
        config.global.fiscal_year_months = Some(12);
        let session = GenerationSession::new(config, PathBuf::from("/tmp/test")).unwrap();
        assert_eq!(session.periods().len(), 3);
    }

    #[test]
    fn test_session_save_and_resume() {
        let config = minimal_config();
        let session = GenerationSession::new(config.clone(), PathBuf::from("/tmp/test")).unwrap();

        let tmp = std::env::temp_dir().join("test_session.dss");
        session.save(&tmp).unwrap();

        let resumed = GenerationSession::resume(&tmp, config).unwrap();
        assert_eq!(resumed.state().period_cursor, 0);
        assert_eq!(resumed.state().rng_seed, 42);

        let _ = fs::remove_file(&tmp);
    }

    #[test]
    fn test_session_resume_config_mismatch() {
        let config = minimal_config();
        let session = GenerationSession::new(config.clone(), PathBuf::from("/tmp/test")).unwrap();

        let tmp = std::env::temp_dir().join("test_session_mismatch.dss");
        session.save(&tmp).unwrap();

        let mut different_config = config;
        different_config.global.seed = Some(999);
        let result = GenerationSession::resume(&tmp, different_config);
        assert!(result.is_err());

        let _ = fs::remove_file(&tmp);
    }

    #[test]
    fn test_session_remaining_periods() {
        let config = minimal_config();
        let session = GenerationSession::new(config, PathBuf::from("/tmp/test")).unwrap();
        assert_eq!(session.remaining_periods(), 1);
    }
}
```

**Step 4:** `cargo check -p datasynth-runtime && cargo test -p datasynth-runtime -- generation_session`

**Step 5:** Commit: `feat(runtime): add GenerationSession with multi-period loop and .dss checkpointing`

---

### Task 4: Per-Period Output Directory Support

**Files:**
- Modify: `crates/datasynth-cli/src/main.rs`

**Step 1:** In the `Commands::Generate` match arm (around line 314), add session-aware generation path. After the existing orchestrator setup, add logic that checks for multi-period:

```rust
// After loading config, before orchestrator creation:
let fy_months = config.global.fiscal_year_months;
let total_months = config.global.period_months;
let use_session = fy_months.is_some() && fy_months.unwrap() < total_months;

if use_session {
    use datasynth_runtime::generation_session::GenerationSession;
    let mut session = GenerationSession::new(config, output.clone())?;
    let results = session.generate_all()?;

    // Save checkpoint
    let dss_path = output.join("session.dss");
    session.save(&dss_path)?;

    println!("\nGeneration complete:");
    for r in &results {
        println!("  {} — {} JEs, {} docs, {:.1}s",
            r.period.label, r.journal_entry_count,
            r.document_count, r.duration_secs);
    }
    return Ok(());
}
// ... existing single-period path continues below
```

**Step 2:** Add `--append` and `--months` CLI flags to the Generate command:
```rust
/// Append incremental data to existing output
#[arg(long)]
append: bool,

/// Number of additional months for incremental generation
#[arg(long)]
months: Option<u32>,
```

**Step 3:** Add append handling before the session/single-period fork:
```rust
if append {
    let dss_path = output.join("session.dss");
    if !dss_path.exists() {
        eprintln!("Error: No session.dss found in output directory. Cannot append.");
        std::process::exit(1);
    }
    let additional = months.unwrap_or(12);
    let mut session = GenerationSession::resume(&dss_path, config)?;
    let results = session.generate_delta(additional)?;
    session.save(&dss_path)?;

    println!("\nIncremental generation complete ({} new months):", additional);
    for r in &results {
        println!("  {} — {} JEs, {:.1}s", r.period.label, r.journal_entry_count, r.duration_secs);
    }
    return Ok(());
}
```

**Step 4:** `cargo check -p datasynth-cli && cargo build -p datasynth-cli`

**Step 5:** Commit: `feat(cli): add multi-period session generation and --append/--months flags`

---

## Phase 2: Fraud Scenario Packs

### Task 5: Fraud Pack YAML Templates

**Files:**
- Create: `crates/datasynth-config/src/fraud_packs/mod.rs`
- Create: `crates/datasynth-config/src/fraud_packs/revenue_fraud.yaml`
- Create: `crates/datasynth-config/src/fraud_packs/payroll_ghost.yaml`
- Create: `crates/datasynth-config/src/fraud_packs/vendor_kickback.yaml`
- Create: `crates/datasynth-config/src/fraud_packs/management_override.yaml`
- Create: `crates/datasynth-config/src/fraud_packs/comprehensive.yaml`
- Modify: `crates/datasynth-config/src/lib.rs`

**Step 1:** Create the 5 fraud pack YAML files. Example `revenue_fraud.yaml`:
```yaml
fraud:
  enabled: true
  fraud_rate: 0.02
  fraud_type_distribution:
    revenue_manipulation: 0.40
    fictitious_transaction: 0.30
    expense_capitalization: 0.20
    timing_anomaly: 0.10
    suspense_account_abuse: 0.0
    split_transaction: 0.0
    unauthorized_access: 0.0
    duplicate_payment: 0.0
anomaly_injection:
  enabled: true
  rates:
    total_rate: 0.03
    fraud_rate: 0.02
    error_rate: 0.005
    process_rate: 0.005
  multi_stage_schemes:
    enabled: true
    revenue_manipulation:
      probability: 0.005
```

Create similar YAML fragments for the other 4 packs per design §7.1.

**Step 2:** Create `fraud_packs/mod.rs` with pack loading:
```rust
use serde_json::Value;
use std::collections::HashMap;

/// Built-in fraud scenario pack names.
pub const FRAUD_PACKS: &[&str] = &[
    "revenue_fraud",
    "payroll_ghost",
    "vendor_kickback",
    "management_override",
    "comprehensive",
];

/// Load a built-in fraud pack by name, returning it as a JSON Value for merging.
pub fn load_fraud_pack(name: &str) -> Option<Value> {
    let yaml_str = match name {
        "revenue_fraud" => include_str!("fraud_packs/revenue_fraud.yaml"),
        "payroll_ghost" => include_str!("fraud_packs/payroll_ghost.yaml"),
        "vendor_kickback" => include_str!("fraud_packs/vendor_kickback.yaml"),
        "management_override" => include_str!("fraud_packs/management_override.yaml"),
        "comprehensive" => include_str!("fraud_packs/comprehensive.yaml"),
        _ => return None,
    };
    serde_yaml::from_str(yaml_str).ok()
}

/// Merge a fraud pack overlay into a base config JSON value.
/// Pack values override base values. Objects are merged recursively.
pub fn merge_fraud_pack(base: &mut Value, overlay: &Value) {
    match (base, overlay) {
        (Value::Object(base_map), Value::Object(overlay_map)) => {
            for (key, overlay_val) in overlay_map {
                let entry = base_map.entry(key.clone()).or_insert(Value::Null);
                merge_fraud_pack(entry, overlay_val);
            }
        }
        (base, overlay) => {
            *base = overlay.clone();
        }
    }
}

/// Apply one or more fraud packs to a GeneratorConfig, returning the modified config.
pub fn apply_fraud_packs(
    config: &crate::GeneratorConfig,
    pack_names: &[String],
) -> Result<crate::GeneratorConfig, String> {
    let mut config_json = serde_json::to_value(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    for name in pack_names {
        let pack = load_fraud_pack(name)
            .ok_or_else(|| format!("Unknown fraud pack: '{}'. Available: {:?}", name, FRAUD_PACKS))?;
        merge_fraud_pack(&mut config_json, &pack);
    }

    // Strip nulls (same pattern as ConfigMutator)
    strip_nulls(&mut config_json);

    serde_json::from_value(config_json)
        .map_err(|e| format!("Failed to deserialize merged config: {}", e))
}

fn strip_nulls(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.retain(|_, v| !v.is_null());
            for v in map.values_mut() {
                strip_nulls(v);
            }
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                strip_nulls(v);
            }
        }
        _ => {}
    }
}
```

**Step 3:** Register in `crates/datasynth-config/src/lib.rs`:
```rust
pub mod fraud_packs;
```

**Step 4:** Add tests:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_all_packs() {
        for name in FRAUD_PACKS {
            let pack = load_fraud_pack(name);
            assert!(pack.is_some(), "Failed to load pack: {}", name);
        }
    }

    #[test]
    fn test_load_unknown_pack_returns_none() {
        assert!(load_fraud_pack("nonexistent").is_none());
    }

    #[test]
    fn test_merge_fraud_pack_overwrites() {
        let mut base = serde_json::json!({"fraud": {"enabled": false, "fraud_rate": 0.01}});
        let overlay = serde_json::json!({"fraud": {"enabled": true, "fraud_rate": 0.05}});
        merge_fraud_pack(&mut base, &overlay);
        assert_eq!(base["fraud"]["enabled"], true);
        assert_eq!(base["fraud"]["fraud_rate"], 0.05);
    }

    #[test]
    fn test_merge_preserves_non_overlapping() {
        let mut base = serde_json::json!({"fraud": {"enabled": false}, "other": "keep"});
        let overlay = serde_json::json!({"fraud": {"enabled": true}});
        merge_fraud_pack(&mut base, &overlay);
        assert_eq!(base["other"], "keep");
    }
}
```

**Step 5:** `cargo check -p datasynth-config && cargo test -p datasynth-config -- fraud_pack`

**Step 6:** Commit: `feat(config): add 5 built-in fraud scenario packs with merge logic`

---

### Task 6: Fraud Pack CLI Integration

**Files:**
- Modify: `crates/datasynth-cli/src/main.rs`

**Step 1:** Add CLI flags to `Commands::Generate`:
```rust
/// Apply a fraud scenario pack (can be repeated: --fraud-scenario revenue_fraud --fraud-scenario payroll_ghost)
#[arg(long, action = clap::ArgAction::Append)]
fraud_scenario: Vec<String>,

/// Override fraud rate when using fraud scenarios
#[arg(long)]
fraud_rate: Option<f64>,
```

**Step 2:** After config loading but before orchestrator creation, add fraud pack merging:
```rust
if !fraud_scenario.is_empty() {
    config = datasynth_config::fraud_packs::apply_fraud_packs(&config, &fraud_scenario)
        .map_err(|e| SynthError::generation(e))?;
    println!("Applied fraud packs: {:?}", fraud_scenario);
}

if let Some(rate) = fraud_rate {
    config.fraud.enabled = true;
    config.fraud.fraud_rate = rate;
    config.anomaly_injection.enabled = true;
    config.anomaly_injection.rates.fraud_rate = rate;
}
```

**Step 3:** `cargo check -p datasynth-cli && cargo build -p datasynth-cli`

**Step 4:** Commit: `feat(cli): add --fraud-scenario and --fraud-rate flags`

---

## Phase 3: Streaming Pipeline

### Task 7: PhaseSink Trait and StreamPipeline

**Files:**
- Create: `crates/datasynth-runtime/src/stream_pipeline.rs`
- Modify: `crates/datasynth-runtime/src/lib.rs`

**Step 1:** Create `stream_pipeline.rs` with the PhaseSink trait and StreamPipeline:

```rust
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use serde::Serialize;

/// Trait for sinks that receive typed items during generation phases.
pub trait PhaseSink: Send + Sync {
    /// Emit a single item (serialized as JSON).
    fn emit(&self, phase: &str, item_type: &str, item: &dyn erased_serde::Serialize) -> Result<(), StreamError>;
    /// Signal that a phase has completed.
    fn phase_complete(&self, phase: &str) -> Result<(), StreamError>;
    /// Flush buffered items.
    fn flush(&self) -> Result<(), StreamError>;
    /// Get stats.
    fn stats(&self) -> StreamStats;
}

#[derive(Debug, Clone, Default)]
pub struct StreamStats {
    pub items_emitted: u64,
    pub bytes_sent: u64,
    pub errors: u64,
    pub phases_completed: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum StreamError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Backpressure: buffer full")]
    BackpressureFull,
}

/// Stream target configuration.
#[derive(Debug, Clone)]
pub enum StreamTarget {
    /// Stream to an HTTP endpoint as JSONL.
    Http { url: String, api_key: Option<String>, batch_size: usize },
    /// Write JSONL to a file.
    File { path: PathBuf },
    /// No streaming — null sink.
    None,
}

/// Backpressure strategy when sink can't keep up.
#[derive(Debug, Clone, Default)]
pub enum BackpressureStrategy {
    #[default]
    Block,
    DropOldest,
    Buffer { max_items: usize },
}

/// The streaming pipeline that wraps a target and manages buffering.
pub struct StreamPipeline {
    target: StreamTarget,
    stats: Arc<Mutex<StreamStats>>,
    writer: Mutex<Option<Box<dyn std::io::Write + Send>>>,
}

impl StreamPipeline {
    pub fn new(target: StreamTarget) -> Result<Self, StreamError> {
        let writer: Option<Box<dyn std::io::Write + Send>> = match &target {
            StreamTarget::File { path } => {
                let file = std::fs::File::create(path)?;
                Some(Box::new(std::io::BufWriter::new(file)))
            }
            StreamTarget::Http { .. } => None, // HTTP batching handled separately
            StreamTarget::None => None,
        };

        Ok(Self {
            target,
            stats: Arc::new(Mutex::new(StreamStats::default())),
            writer: Mutex::new(writer),
        })
    }

    pub fn none() -> Self {
        Self {
            target: StreamTarget::None,
            stats: Arc::new(Mutex::new(StreamStats::default())),
            writer: Mutex::new(None),
        }
    }

    pub fn is_active(&self) -> bool {
        !matches!(self.target, StreamTarget::None)
    }
}

impl PhaseSink for StreamPipeline {
    fn emit(&self, phase: &str, item_type: &str, item: &dyn erased_serde::Serialize) -> Result<(), StreamError> {
        if !self.is_active() {
            return Ok(());
        }

        let json = serde_json::to_string(&JsonlEnvelope { phase, item_type, data: item })
            .map_err(|e| StreamError::Serialization(e.to_string()))?;
        let bytes = json.len() as u64 + 1; // +1 for newline

        if let Ok(mut writer_guard) = self.writer.lock() {
            if let Some(writer) = writer_guard.as_mut() {
                use std::io::Write;
                writeln!(writer, "{}", json)?;
            }
        }

        if let Ok(mut stats) = self.stats.lock() {
            stats.items_emitted += 1;
            stats.bytes_sent += bytes;
        }

        Ok(())
    }

    fn phase_complete(&self, _phase: &str) -> Result<(), StreamError> {
        if let Ok(mut stats) = self.stats.lock() {
            stats.phases_completed += 1;
        }
        self.flush()
    }

    fn flush(&self) -> Result<(), StreamError> {
        if let Ok(mut writer_guard) = self.writer.lock() {
            if let Some(writer) = writer_guard.as_mut() {
                use std::io::Write;
                writer.flush()?;
            }
        }
        Ok(())
    }

    fn stats(&self) -> StreamStats {
        self.stats.lock().map(|s| s.clone()).unwrap_or_default()
    }
}

/// Internal JSONL envelope for streaming.
#[derive(Serialize)]
struct JsonlEnvelope<'a> {
    phase: &'a str,
    item_type: &'a str,
    data: &'a dyn erased_serde::Serialize,
}
```

**Step 2:** Add `erased-serde` dependency to `crates/datasynth-runtime/Cargo.toml`:
```toml
erased-serde = "0.4"
```

**Step 3:** Register in `lib.rs`:
```rust
pub mod stream_pipeline;
pub use stream_pipeline::*;
```

**Step 4:** Add tests:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_none_pipeline_is_inactive() {
        let pipeline = StreamPipeline::none();
        assert!(!pipeline.is_active());
    }

    #[test]
    fn test_file_pipeline_writes_jsonl() {
        let tmp = std::env::temp_dir().join("test_stream.jsonl");
        let pipeline = StreamPipeline::new(StreamTarget::File { path: tmp.clone() }).unwrap();
        assert!(pipeline.is_active());

        let item = serde_json::json!({"id": "test-001", "amount": 100.0});
        pipeline.emit("journal_entries", "JournalEntry", &item).unwrap();
        pipeline.flush().unwrap();

        let content = std::fs::read_to_string(&tmp).unwrap();
        assert!(content.contains("test-001"));
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_stats_increment() {
        let tmp = std::env::temp_dir().join("test_stream_stats.jsonl");
        let pipeline = StreamPipeline::new(StreamTarget::File { path: tmp.clone() }).unwrap();

        let item = serde_json::json!({"id": 1});
        pipeline.emit("phase1", "Item", &item).unwrap();
        pipeline.emit("phase1", "Item", &item).unwrap();
        pipeline.phase_complete("phase1").unwrap();

        let stats = pipeline.stats();
        assert_eq!(stats.items_emitted, 2);
        assert_eq!(stats.phases_completed, 1);
        let _ = std::fs::remove_file(&tmp);
    }
}
```

**Step 5:** `cargo check -p datasynth-runtime && cargo test -p datasynth-runtime -- stream_pipeline`

**Step 6:** Commit: `feat(runtime): add PhaseSink trait and StreamPipeline for phase-aware streaming`

---

### Task 8: Stream CLI Integration

**Files:**
- Modify: `crates/datasynth-cli/src/main.rs`

**Step 1:** Add `--stream-file` flag to `Commands::Generate`:
```rust
/// Stream output to a JSONL file during generation
#[arg(long)]
stream_file: Option<PathBuf>,
```

**Step 2:** Wire the StreamPipeline into the GenerationSession flow (in the session-aware branch):
```rust
// After creating session but before generate_all:
let stream_pipeline = if let Some(stream_path) = &stream_file {
    Arc::new(datasynth_runtime::stream_pipeline::StreamPipeline::new(
        datasynth_runtime::stream_pipeline::StreamTarget::File { path: stream_path.clone() }
    ).map_err(|e| SynthError::generation(format!("Stream error: {}", e)))?)
} else {
    Arc::new(datasynth_runtime::stream_pipeline::StreamPipeline::none())
};
```

**Step 3:** `cargo check -p datasynth-cli && cargo build -p datasynth-cli`

**Step 4:** Commit: `feat(cli): add --stream-file flag for JSONL streaming output`

---

## Phase 4: OCEL 2.0 Enrichment

### Task 9: Lifecycle State Machine Types

**Files:**
- Create: `crates/datasynth-ocpm/src/models/lifecycle_state_machine.rs`
- Modify: `crates/datasynth-ocpm/src/models/mod.rs`

**Step 1:** Create `lifecycle_state_machine.rs` with state machine types per design §8.1:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A formal lifecycle state machine for an OCEL 2.0 object type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleStateMachine {
    pub object_type: String,
    pub initial_state: String,
    pub terminal_states: Vec<String>,
    pub transitions: Vec<StateTransition>,
}

/// A single transition between states with probability and timing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    pub from_state: String,
    pub to_state: String,
    pub probability: f64,
    pub min_lag_hours: f64,
    pub max_lag_hours: f64,
    pub activity_name: String,
}

impl LifecycleStateMachine {
    /// Get all valid transitions from a given state.
    pub fn transitions_from(&self, state: &str) -> Vec<&StateTransition> {
        self.transitions.iter().filter(|t| t.from_state == state).collect()
    }

    /// Check if a state is terminal.
    pub fn is_terminal(&self, state: &str) -> bool {
        self.terminal_states.contains(&state.to_string())
    }

    /// Validate that transition probabilities from each state sum to ~1.0.
    pub fn validate(&self) -> Result<(), String> {
        let mut state_probs: HashMap<String, f64> = HashMap::new();
        for t in &self.transitions {
            *state_probs.entry(t.from_state.clone()).or_default() += t.probability;
        }
        for (state, prob) in &state_probs {
            if (*prob - 1.0).abs() > 0.05 {
                return Err(format!(
                    "State '{}' transitions sum to {:.2}, expected ~1.0", state, prob
                ));
            }
        }
        Ok(())
    }
}

/// Built-in state machines for standard object types.
pub fn purchase_order_state_machine() -> LifecycleStateMachine {
    LifecycleStateMachine {
        object_type: "PurchaseOrder".to_string(),
        initial_state: "Draft".to_string(),
        terminal_states: vec!["Closed".to_string(), "Cancelled".to_string()],
        transitions: vec![
            StateTransition {
                from_state: "Draft".into(), to_state: "Submitted".into(),
                probability: 0.95, min_lag_hours: 2.0, max_lag_hours: 8.0,
                activity_name: "Submit Purchase Order".into(),
            },
            StateTransition {
                from_state: "Draft".into(), to_state: "Cancelled".into(),
                probability: 0.05, min_lag_hours: 1.0, max_lag_hours: 24.0,
                activity_name: "Cancel Purchase Order".into(),
            },
            StateTransition {
                from_state: "Submitted".into(), to_state: "Approved".into(),
                probability: 0.90, min_lag_hours: 4.0, max_lag_hours: 48.0,
                activity_name: "Approve Purchase Order".into(),
            },
            StateTransition {
                from_state: "Submitted".into(), to_state: "Rejected".into(),
                probability: 0.10, min_lag_hours: 2.0, max_lag_hours: 24.0,
                activity_name: "Reject Purchase Order".into(),
            },
            StateTransition {
                from_state: "Approved".into(), to_state: "Released".into(),
                probability: 1.0, min_lag_hours: 1.0, max_lag_hours: 4.0,
                activity_name: "Release Purchase Order".into(),
            },
            StateTransition {
                from_state: "Released".into(), to_state: "PartiallyReceived".into(),
                probability: 0.30, min_lag_hours: 72.0, max_lag_hours: 336.0,
                activity_name: "Partially Receive Goods".into(),
            },
            StateTransition {
                from_state: "Released".into(), to_state: "FullyReceived".into(),
                probability: 0.70, min_lag_hours: 120.0, max_lag_hours: 504.0,
                activity_name: "Receive Goods".into(),
            },
            StateTransition {
                from_state: "PartiallyReceived".into(), to_state: "FullyReceived".into(),
                probability: 1.0, min_lag_hours: 72.0, max_lag_hours: 336.0,
                activity_name: "Receive Remaining Goods".into(),
            },
            StateTransition {
                from_state: "FullyReceived".into(), to_state: "Closed".into(),
                probability: 1.0, min_lag_hours: 24.0, max_lag_hours: 120.0,
                activity_name: "Close Purchase Order".into(),
            },
        ],
    }
}

pub fn sales_order_state_machine() -> LifecycleStateMachine {
    LifecycleStateMachine {
        object_type: "SalesOrder".to_string(),
        initial_state: "Created".to_string(),
        terminal_states: vec!["Closed".to_string(), "Cancelled".to_string()],
        transitions: vec![
            StateTransition {
                from_state: "Created".into(), to_state: "Confirmed".into(),
                probability: 0.92, min_lag_hours: 1.0, max_lag_hours: 24.0,
                activity_name: "Confirm Sales Order".into(),
            },
            StateTransition {
                from_state: "Created".into(), to_state: "Cancelled".into(),
                probability: 0.08, min_lag_hours: 1.0, max_lag_hours: 48.0,
                activity_name: "Cancel Sales Order".into(),
            },
            StateTransition {
                from_state: "Confirmed".into(), to_state: "Shipped".into(),
                probability: 0.95, min_lag_hours: 24.0, max_lag_hours: 168.0,
                activity_name: "Ship Order".into(),
            },
            StateTransition {
                from_state: "Confirmed".into(), to_state: "Cancelled".into(),
                probability: 0.05, min_lag_hours: 2.0, max_lag_hours: 72.0,
                activity_name: "Cancel Sales Order".into(),
            },
            StateTransition {
                from_state: "Shipped".into(), to_state: "Delivered".into(),
                probability: 0.98, min_lag_hours: 24.0, max_lag_hours: 240.0,
                activity_name: "Deliver Order".into(),
            },
            StateTransition {
                from_state: "Shipped".into(), to_state: "Returned".into(),
                probability: 0.02, min_lag_hours: 48.0, max_lag_hours: 336.0,
                activity_name: "Return Shipment".into(),
            },
            StateTransition {
                from_state: "Delivered".into(), to_state: "Invoiced".into(),
                probability: 1.0, min_lag_hours: 1.0, max_lag_hours: 48.0,
                activity_name: "Invoice Customer".into(),
            },
            StateTransition {
                from_state: "Invoiced".into(), to_state: "Closed".into(),
                probability: 1.0, min_lag_hours: 24.0, max_lag_hours: 720.0,
                activity_name: "Receive Payment".into(),
            },
        ],
    }
}

pub fn vendor_invoice_state_machine() -> LifecycleStateMachine {
    LifecycleStateMachine {
        object_type: "VendorInvoice".to_string(),
        initial_state: "Received".to_string(),
        terminal_states: vec!["Paid".to_string(), "Cancelled".to_string()],
        transitions: vec![
            StateTransition {
                from_state: "Received".into(), to_state: "Registered".into(),
                probability: 0.95, min_lag_hours: 2.0, max_lag_hours: 48.0,
                activity_name: "Register Invoice".into(),
            },
            StateTransition {
                from_state: "Received".into(), to_state: "Cancelled".into(),
                probability: 0.05, min_lag_hours: 1.0, max_lag_hours: 24.0,
                activity_name: "Reject Invoice".into(),
            },
            StateTransition {
                from_state: "Registered".into(), to_state: "Matched".into(),
                probability: 0.85, min_lag_hours: 4.0, max_lag_hours: 72.0,
                activity_name: "Three-Way Match".into(),
            },
            StateTransition {
                from_state: "Registered".into(), to_state: "Blocked".into(),
                probability: 0.15, min_lag_hours: 4.0, max_lag_hours: 72.0,
                activity_name: "Block Invoice".into(),
            },
            StateTransition {
                from_state: "Blocked".into(), to_state: "Matched".into(),
                probability: 0.80, min_lag_hours: 24.0, max_lag_hours: 168.0,
                activity_name: "Resolve Block".into(),
            },
            StateTransition {
                from_state: "Blocked".into(), to_state: "Cancelled".into(),
                probability: 0.20, min_lag_hours: 24.0, max_lag_hours: 336.0,
                activity_name: "Cancel Invoice".into(),
            },
            StateTransition {
                from_state: "Matched".into(), to_state: "Approved".into(),
                probability: 1.0, min_lag_hours: 1.0, max_lag_hours: 24.0,
                activity_name: "Approve for Payment".into(),
            },
            StateTransition {
                from_state: "Approved".into(), to_state: "Paid".into(),
                probability: 1.0, min_lag_hours: 24.0, max_lag_hours: 720.0,
                activity_name: "Execute Payment".into(),
            },
        ],
    }
}

/// Get all built-in state machines.
pub fn all_state_machines() -> Vec<LifecycleStateMachine> {
    vec![
        purchase_order_state_machine(),
        sales_order_state_machine(),
        vendor_invoice_state_machine(),
    ]
}
```

**Step 2:** Update `crates/datasynth-ocpm/src/models/mod.rs`:
```rust
pub mod lifecycle_state_machine;
pub use lifecycle_state_machine::*;
```

**Step 3:** Add tests:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_po_state_machine_validates() {
        let sm = purchase_order_state_machine();
        sm.validate().unwrap();
    }

    #[test]
    fn test_so_state_machine_validates() {
        let sm = sales_order_state_machine();
        sm.validate().unwrap();
    }

    #[test]
    fn test_vi_state_machine_validates() {
        let sm = vendor_invoice_state_machine();
        sm.validate().unwrap();
    }

    #[test]
    fn test_transitions_from_draft() {
        let sm = purchase_order_state_machine();
        let transitions = sm.transitions_from("Draft");
        assert_eq!(transitions.len(), 2);
        let prob_sum: f64 = transitions.iter().map(|t| t.probability).sum();
        assert!((prob_sum - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_terminal_states() {
        let sm = purchase_order_state_machine();
        assert!(sm.is_terminal("Closed"));
        assert!(sm.is_terminal("Cancelled"));
        assert!(!sm.is_terminal("Draft"));
    }

    #[test]
    fn test_all_state_machines_count() {
        let machines = all_state_machines();
        assert_eq!(machines.len(), 3);
    }

    #[test]
    fn test_serde_roundtrip() {
        let sm = purchase_order_state_machine();
        let json = serde_json::to_string(&sm).unwrap();
        let restored: LifecycleStateMachine = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.object_type, "PurchaseOrder");
        assert_eq!(restored.transitions.len(), sm.transitions.len());
    }
}
```

**Step 4:** `cargo check -p datasynth-ocpm && cargo test -p datasynth-ocpm -- lifecycle_state_machine`

**Step 5:** Commit: `feat(ocpm): add lifecycle state machines for PO, SO, and VendorInvoice (OCEL 2.0 enrichment)`

---

### Task 10: Multi-Object Correlation Events

**Files:**
- Create: `crates/datasynth-ocpm/src/models/correlation_event.rs`
- Modify: `crates/datasynth-ocpm/src/models/mod.rs`

**Step 1:** Create `correlation_event.rs` with multi-object event types per design §8.2:

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use super::{EventObjectRef, ObjectQualifier};

/// Multi-object correlation event types per OCEL 2.0 enrichment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CorrelationEventType {
    ThreeWayMatch,
    PaymentAllocation,
    IntercompanyElimination,
    BankReconciliation,
    GoodsIssue,
}

/// A correlation event that explicitly references 2+ objects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationEvent {
    pub event_id: Uuid,
    pub correlation_type: CorrelationEventType,
    pub correlation_id: String,
    pub timestamp: DateTime<Utc>,
    pub object_refs: Vec<EventObjectRef>,
    pub resource_id: String,
    pub attributes: std::collections::HashMap<String, serde_json::Value>,
    pub company_code: String,
}

impl CorrelationEvent {
    /// Create a three-way match event linking PO, GR, and Invoice.
    pub fn three_way_match(
        po_id: Uuid,
        gr_id: Uuid,
        invoice_id: Uuid,
        timestamp: DateTime<Utc>,
        resource_id: &str,
        company_code: &str,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            correlation_type: CorrelationEventType::ThreeWayMatch,
            correlation_id: format!("3WAY-{}", &po_id.to_string()[..8]),
            timestamp,
            object_refs: vec![
                EventObjectRef::read(po_id, "PurchaseOrder"),
                EventObjectRef::read(gr_id, "GoodsReceipt"),
                EventObjectRef::updated(invoice_id, "VendorInvoice"),
            ],
            resource_id: resource_id.to_string(),
            attributes: std::collections::HashMap::new(),
            company_code: company_code.to_string(),
        }
    }

    /// Create a payment allocation event linking payment to one or more invoices.
    pub fn payment_allocation(
        payment_id: Uuid,
        invoice_ids: &[Uuid],
        timestamp: DateTime<Utc>,
        resource_id: &str,
        company_code: &str,
    ) -> Self {
        let mut refs = vec![
            EventObjectRef::created(payment_id, "Payment"),
        ];
        for inv_id in invoice_ids {
            refs.push(EventObjectRef::updated(*inv_id, "VendorInvoice"));
        }
        Self {
            event_id: Uuid::new_v4(),
            correlation_type: CorrelationEventType::PaymentAllocation,
            correlation_id: format!("PAY-ALLOC-{}", &payment_id.to_string()[..8]),
            timestamp,
            object_refs: refs,
            resource_id: resource_id.to_string(),
            attributes: std::collections::HashMap::new(),
            company_code: company_code.to_string(),
        }
    }

    /// Create a bank reconciliation event linking bank statement to JE.
    pub fn bank_reconciliation(
        statement_line_id: Uuid,
        je_id: Uuid,
        timestamp: DateTime<Utc>,
        resource_id: &str,
        company_code: &str,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            correlation_type: CorrelationEventType::BankReconciliation,
            correlation_id: format!("RECON-{}", &statement_line_id.to_string()[..8]),
            timestamp,
            object_refs: vec![
                EventObjectRef::read(statement_line_id, "BankStatementLine"),
                EventObjectRef::updated(je_id, "JournalEntry"),
            ],
            resource_id: resource_id.to_string(),
            attributes: std::collections::HashMap::new(),
            company_code: company_code.to_string(),
        }
    }

    /// Add an attribute to this correlation event.
    pub fn with_attribute(mut self, key: &str, value: serde_json::Value) -> Self {
        self.attributes.insert(key.to_string(), value);
        self
    }
}
```

**Step 2:** Update `models/mod.rs`:
```rust
pub mod correlation_event;
pub use correlation_event::*;
```

**Step 3:** Add tests:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_three_way_match_has_3_objects() {
        let event = CorrelationEvent::three_way_match(
            Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(),
            Utc::now(), "AP-CLERK-001", "C001",
        );
        assert_eq!(event.object_refs.len(), 3);
        assert!(event.correlation_id.starts_with("3WAY-"));
    }

    #[test]
    fn test_payment_allocation_multi_invoice() {
        let invoices = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
        let event = CorrelationEvent::payment_allocation(
            Uuid::new_v4(), &invoices, Utc::now(), "AP-CLERK-002", "C001",
        );
        assert_eq!(event.object_refs.len(), 4); // 1 payment + 3 invoices
    }

    #[test]
    fn test_bank_reconciliation() {
        let event = CorrelationEvent::bank_reconciliation(
            Uuid::new_v4(), Uuid::new_v4(), Utc::now(), "ACCOUNTANT-001", "C001",
        );
        assert_eq!(event.object_refs.len(), 2);
        assert!(event.correlation_id.starts_with("RECON-"));
    }

    #[test]
    fn test_serde_roundtrip() {
        let event = CorrelationEvent::three_way_match(
            Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(),
            Utc::now(), "CLERK-001", "C001",
        );
        let json = serde_json::to_string(&event).unwrap();
        let restored: CorrelationEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.object_refs.len(), 3);
    }
}
```

**Step 4:** `cargo check -p datasynth-ocpm && cargo test -p datasynth-ocpm -- correlation_event`

**Step 5:** Commit: `feat(ocpm): add multi-object correlation events (ThreeWayMatch, PaymentAllocation, BankReconciliation)`

---

### Task 11: Resource Workload Modeling

**Files:**
- Create: `crates/datasynth-ocpm/src/models/resource_pool.rs`
- Modify: `crates/datasynth-ocpm/src/models/mod.rs`

**Step 1:** Create `resource_pool.rs` with resource pool types per design §8.3:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A pool of resources (e.g., AP Clerks) with workload tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePool {
    pub pool_id: String,
    pub pool_name: String,
    pub resources: Vec<PoolResource>,
    pub assignment_strategy: AssignmentStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolResource {
    pub resource_id: String,
    pub name: String,
    pub max_concurrent: usize,
    pub current_workload: f64,
    pub total_assigned: u64,
    pub skills: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum AssignmentStrategy {
    #[default]
    RoundRobin,
    LeastBusy,
    SkillBased,
}

impl ResourcePool {
    /// Create a pool with N resources using a naming pattern.
    pub fn new(pool_id: &str, pool_name: &str, count: usize, prefix: &str) -> Self {
        let resources = (1..=count).map(|i| {
            PoolResource {
                resource_id: format!("{}-{:03}", prefix, i),
                name: format!("{} {}", pool_name, i),
                max_concurrent: 10,
                current_workload: 0.0,
                total_assigned: 0,
                skills: vec![],
            }
        }).collect();

        Self {
            pool_id: pool_id.to_string(),
            pool_name: pool_name.to_string(),
            resources,
            assignment_strategy: AssignmentStrategy::default(),
        }
    }

    /// Assign work to the next available resource based on strategy.
    pub fn assign(&mut self) -> Option<&str> {
        if self.resources.is_empty() {
            return None;
        }

        let idx = match self.assignment_strategy {
            AssignmentStrategy::RoundRobin => {
                let min_assigned = self.resources.iter().map(|r| r.total_assigned).min().unwrap_or(0);
                self.resources.iter().position(|r| r.total_assigned == min_assigned).unwrap_or(0)
            }
            AssignmentStrategy::LeastBusy => {
                self.resources.iter()
                    .enumerate()
                    .min_by(|a, b| a.1.current_workload.partial_cmp(&b.1.current_workload).unwrap())
                    .map(|(i, _)| i)
                    .unwrap_or(0)
            }
            AssignmentStrategy::SkillBased => 0, // Fallback to first available
        };

        self.resources[idx].total_assigned += 1;
        self.resources[idx].current_workload += 0.1;
        Some(&self.resources[idx].resource_id)
    }

    /// Release workload from a resource.
    pub fn release(&mut self, resource_id: &str) {
        if let Some(r) = self.resources.iter_mut().find(|r| r.resource_id == resource_id) {
            r.current_workload = (r.current_workload - 0.1).max(0.0);
        }
    }
}

/// Standard resource pools for a typical enterprise.
pub fn default_resource_pools() -> Vec<ResourcePool> {
    vec![
        ResourcePool::new("ap-pool", "AP Clerk", 5, "AP-CLERK"),
        ResourcePool::new("ar-pool", "AR Clerk", 3, "AR-CLERK"),
        ResourcePool::new("gl-pool", "GL Accountant", 2, "GL-ACCT"),
        ResourcePool::new("approver-pool", "Manager", 4, "MGR"),
        ResourcePool::new("warehouse-pool", "Warehouse Staff", 6, "WH-STAFF"),
    ]
}
```

**Step 2:** Update `models/mod.rs`:
```rust
pub mod resource_pool;
pub use resource_pool::*;
```

**Step 3:** Add tests:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_creation() {
        let pool = ResourcePool::new("test", "Tester", 3, "TST");
        assert_eq!(pool.resources.len(), 3);
        assert_eq!(pool.resources[0].resource_id, "TST-001");
        assert_eq!(pool.resources[2].resource_id, "TST-003");
    }

    #[test]
    fn test_round_robin_assignment() {
        let mut pool = ResourcePool::new("test", "Tester", 3, "TST");
        let r1 = pool.assign().unwrap().to_string();
        let r2 = pool.assign().unwrap().to_string();
        let r3 = pool.assign().unwrap().to_string();
        // After 3 assignments, should cycle through all 3
        assert_eq!(r1, "TST-001");
        assert_eq!(r2, "TST-002");
        assert_eq!(r3, "TST-003");
    }

    #[test]
    fn test_release_reduces_workload() {
        let mut pool = ResourcePool::new("test", "Tester", 1, "TST");
        pool.assign();
        assert!(pool.resources[0].current_workload > 0.0);
        pool.release("TST-001");
        assert_eq!(pool.resources[0].current_workload, 0.0);
    }

    #[test]
    fn test_default_pools() {
        let pools = default_resource_pools();
        assert_eq!(pools.len(), 5);
    }

    #[test]
    fn test_serde_roundtrip() {
        let pool = ResourcePool::new("ap", "AP Clerk", 2, "AP");
        let json = serde_json::to_string(&pool).unwrap();
        let restored: ResourcePool = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.resources.len(), 2);
    }
}
```

**Step 4:** `cargo check -p datasynth-ocpm && cargo test -p datasynth-ocpm -- resource_pool`

**Step 5:** Commit: `feat(ocpm): add ResourcePool with assignment strategies for workload modeling`

---

### Task 12: Enriched OcpmEvent Fields

**Files:**
- Modify: `crates/datasynth-ocpm/src/models/event.rs`

**Step 1:** Add lifecycle state machine fields to `OcpmEvent`:
```rust
    /// Source state for lifecycle transition (e.g., "Submitted")
    pub from_state: Option<String>,
    /// Target state for lifecycle transition (e.g., "Approved")
    pub to_state: Option<String>,
    /// Resource workload at time of event (0.0 to 1.0)
    pub resource_workload: Option<f64>,
    /// Correlation ID linking related multi-object events
    pub correlation_id: Option<String>,
```

**Step 2:** Add builder methods:
```rust
    pub fn with_state_transition(mut self, from: &str, to: &str) -> Self {
        self.from_state = Some(from.to_string());
        self.to_state = Some(to.to_string());
        self
    }

    pub fn with_resource_workload(mut self, workload: f64) -> Self {
        self.resource_workload = Some(workload);
        self
    }

    pub fn with_correlation_id(mut self, id: &str) -> Self {
        self.correlation_id = Some(id.to_string());
        self
    }
```

**Step 3:** Update the `new()` constructor to initialize new fields to `None`.

**Step 4:** Add tests:
```rust
#[test]
fn test_enriched_event_state_transition() {
    let event = OcpmEvent::new("ACT-001", "Submit PO", Utc::now(), "USER-001", "C001")
        .with_state_transition("Draft", "Submitted")
        .with_resource_workload(0.72)
        .with_correlation_id("3WAY-MATCH-0042");
    assert_eq!(event.from_state.as_deref(), Some("Draft"));
    assert_eq!(event.to_state.as_deref(), Some("Submitted"));
    assert_eq!(event.resource_workload, Some(0.72));
    assert_eq!(event.correlation_id.as_deref(), Some("3WAY-MATCH-0042"));
}
```

**Step 5:** `cargo check -p datasynth-ocpm && cargo test -p datasynth-ocpm -- event`

**Step 6:** Commit: `feat(ocpm): enrich OcpmEvent with from_state, to_state, resource_workload, correlation_id`

---

## Phase 5: Integration Tests

### Task 13: Session Integration Tests

**Files:**
- Create: `crates/datasynth-runtime/tests/generation_session_integration.rs`

**Step 1:** Create integration tests:
```rust
use std::path::PathBuf;
use datasynth_config::GeneratorConfig;
use datasynth_runtime::generation_session::GenerationSession;

fn minimal_config() -> GeneratorConfig {
    let yaml = include_str!("../../datasynth-config/src/presets/retail_small.yaml");
    serde_yaml::from_str(yaml).unwrap_or_else(|_| {
        // Fallback minimal config
        serde_yaml::from_str(r#"
global:
  seed: 42
  industry: retail
  start_date: "2024-01-01"
  period_months: 3
companies:
  - code: "C001"
    name: "Test Corp"
    currency: "USD"
    country: "US"
chart_of_accounts:
  complexity: small
"#).unwrap()
    })
}

#[test]
fn test_session_single_period_generates_output() {
    let tmp = tempfile::tempdir().unwrap();
    let config = minimal_config();
    let mut session = GenerationSession::new(config, tmp.path().to_path_buf()).unwrap();
    let results = session.generate_all().unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].journal_entry_count > 0);
}

#[test]
fn test_session_checkpoint_roundtrip() {
    let tmp = tempfile::tempdir().unwrap();
    let config = minimal_config();
    let session = GenerationSession::new(config.clone(), tmp.path().to_path_buf()).unwrap();

    let dss_path = tmp.path().join("session.dss");
    session.save(&dss_path).unwrap();
    assert!(dss_path.exists());

    let resumed = GenerationSession::resume(&dss_path, config).unwrap();
    assert_eq!(resumed.remaining_periods(), 1);
}

#[test]
fn test_session_multi_period() {
    let tmp = tempfile::tempdir().unwrap();
    let mut config = minimal_config();
    config.global.period_months = 6;
    config.global.fiscal_year_months = Some(3);

    let mut session = GenerationSession::new(config, tmp.path().to_path_buf()).unwrap();
    assert_eq!(session.periods().len(), 2);

    let results = session.generate_all().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(session.remaining_periods(), 0);
}

#[test]
fn test_session_seed_determinism() {
    let tmp1 = tempfile::tempdir().unwrap();
    let tmp2 = tempfile::tempdir().unwrap();
    let config = minimal_config();

    let mut s1 = GenerationSession::new(config.clone(), tmp1.path().to_path_buf()).unwrap();
    let mut s2 = GenerationSession::new(config, tmp2.path().to_path_buf()).unwrap();

    let r1 = s1.generate_all().unwrap();
    let r2 = s2.generate_all().unwrap();

    assert_eq!(r1.len(), r2.len());
    assert_eq!(r1[0].journal_entry_count, r2[0].journal_entry_count);
}
```

**Step 2:** `cargo test -p datasynth-runtime --test generation_session_integration`

**Step 3:** Commit: `test(runtime): add GenerationSession integration tests`

---

### Task 14: OCEL Enrichment Integration Tests

**Files:**
- Create: `crates/datasynth-ocpm/tests/ocel_enrichment_integration.rs`

**Step 1:** Create integration tests for lifecycle + correlation + resource pool:
```rust
use datasynth_ocpm::models::*;
use chrono::Utc;
use uuid::Uuid;

#[test]
fn test_po_lifecycle_full_path() {
    let sm = purchase_order_state_machine();
    sm.validate().unwrap();

    // Verify a full happy path: Draft → Submitted → Approved → Released → FullyReceived → Closed
    let mut current = "Draft";
    let path = ["Submitted", "Approved", "Released", "FullyReceived", "Closed"];
    for expected_next in &path {
        let transitions = sm.transitions_from(current);
        assert!(!transitions.is_empty(), "No transitions from {}", current);
        let matching = transitions.iter().find(|t| t.to_state == *expected_next);
        assert!(matching.is_some(), "No transition from {} to {}", current, expected_next);
        current = expected_next;
    }
    assert!(sm.is_terminal(current));
}

#[test]
fn test_correlation_event_three_way_match_integration() {
    let po_id = Uuid::new_v4();
    let gr_id = Uuid::new_v4();
    let invoice_id = Uuid::new_v4();

    let event = CorrelationEvent::three_way_match(
        po_id, gr_id, invoice_id, Utc::now(), "AP-CLERK-001", "C001"
    ).with_attribute("match_tolerance", serde_json::json!(0.01));

    assert_eq!(event.object_refs.len(), 3);
    assert!(event.attributes.contains_key("match_tolerance"));

    // Verify JSON roundtrip
    let json = serde_json::to_string_pretty(&event).unwrap();
    let restored: CorrelationEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.object_refs.len(), 3);
}

#[test]
fn test_resource_pool_workload_balancing() {
    let mut pool = ResourcePool::new("ap", "AP Clerk", 3, "AP-CLERK");

    // Assign 9 tasks — should distribute evenly with round-robin
    let mut assignments: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    for _ in 0..9 {
        let r = pool.assign().unwrap().to_string();
        *assignments.entry(r).or_default() += 1;
    }

    // Each resource should have exactly 3 tasks
    for (_resource, count) in &assignments {
        assert_eq!(*count, 3);
    }
}

#[test]
fn test_enriched_event_full_attributes() {
    let event = OcpmEvent::new("ACT-APPROVE", "Approve PO", Utc::now(), "MGR-001", "C001")
        .with_state_transition("Submitted", "Approved")
        .with_resource_workload(0.65)
        .with_correlation_id("BATCH-2024-001")
        .with_lifecycle(EventLifecycle::Complete);

    assert_eq!(event.from_state.as_deref(), Some("Submitted"));
    assert_eq!(event.to_state.as_deref(), Some("Approved"));
    assert_eq!(event.resource_workload, Some(0.65));
    assert!(event.lifecycle.is_completion());
}
```

**Step 2:** `cargo test -p datasynth-ocpm --test ocel_enrichment_integration`

**Step 3:** Commit: `test(ocpm): add OCEL 2.0 enrichment integration tests`

---

### Task 15: Fraud Pack Integration Tests

**Files:**
- Create: `crates/datasynth-config/tests/fraud_packs_integration.rs`

**Step 1:** Create integration tests:
```rust
use datasynth_config::fraud_packs::*;
use datasynth_config::GeneratorConfig;

fn base_config() -> GeneratorConfig {
    serde_yaml::from_str(r#"
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
chart_of_accounts:
  complexity: small
"#).unwrap()
}

#[test]
fn test_apply_revenue_fraud_pack() {
    let config = base_config();
    let result = apply_fraud_packs(&config, &["revenue_fraud".to_string()]).unwrap();
    assert!(result.fraud.enabled);
    assert!(result.fraud.fraud_rate > 0.0);
    assert!(result.fraud.fraud_type_distribution.revenue_manipulation > 0.0);
}

#[test]
fn test_apply_multiple_packs_merge() {
    let config = base_config();
    let result = apply_fraud_packs(
        &config,
        &["revenue_fraud".to_string(), "payroll_ghost".to_string()],
    ).unwrap();
    assert!(result.fraud.enabled);
}

#[test]
fn test_apply_unknown_pack_error() {
    let config = base_config();
    let result = apply_fraud_packs(&config, &["nonexistent".to_string()]);
    assert!(result.is_err());
}

#[test]
fn test_all_packs_produce_valid_configs() {
    let config = base_config();
    for pack_name in FRAUD_PACKS {
        let result = apply_fraud_packs(&config, &[pack_name.to_string()]);
        assert!(result.is_ok(), "Pack '{}' produced invalid config: {:?}", pack_name, result.err());
    }
}

#[test]
fn test_pack_preserves_non_fraud_config() {
    let config = base_config();
    let result = apply_fraud_packs(&config, &["revenue_fraud".to_string()]).unwrap();
    assert_eq!(result.global.seed, Some(42));
    assert_eq!(result.global.period_months, 12);
    assert_eq!(result.companies.len(), 1);
}
```

**Step 2:** `cargo test -p datasynth-config --test fraud_packs_integration`

**Step 3:** Commit: `test(config): add fraud pack integration tests`

---

## Phase 6: Final Verification

### Task 16: Full Workspace Verification

**Step 1:** `cargo fmt --all`
**Step 2:** `cargo clippy --workspace --exclude datasynth-ui` — fix any warnings
**Step 3:** `cargo test --workspace --exclude datasynth-ui --exclude datasynth-server` — all tests pass
**Step 4:** `cargo build --release -p datasynth-cli`
**Step 5:** Verify multi-period generation works end-to-end:
```bash
./target/release/datasynth-data generate --demo --output /tmp/test_multiperiod
```
**Step 6:** Commit any final formatting/clippy fixes.

---

### Task 17: Version Bump and CHANGELOG

**Step 1:** Bump version from `0.10.0` to `0.11.0` in root `Cargo.toml` (both `version` and all `workspace.dependencies` internal crate versions).
**Step 2:** Bump Python wrapper from `1.6.0` to `1.7.0` in `python/pyproject.toml`.
**Step 3:** Add CHANGELOG.md entry for v0.11.0 documenting:
- GenerationSession with multi-period generation and .dss checkpointing
- Incremental delta generation (--append --months)
- 5 built-in fraud scenario packs (--fraud-scenario)
- PhaseSink trait and StreamPipeline for phase-aware streaming
- OCEL 2.0 lifecycle state machines (PO, SO, VendorInvoice)
- Multi-object correlation events (ThreeWayMatch, PaymentAllocation, BankReconciliation)
- Resource pool workload modeling
- Enriched OcpmEvent with from_state, to_state, resource_workload, correlation_id
**Step 4:** Add python/CHANGELOG.md entry for 1.7.0.
**Step 5:** `cargo check --workspace`
**Step 6:** Commit: `chore: bump version to v0.11.0 (Rust) / v1.7.0 (Python)`

---

## Task Dependency Graph

```
Phase 1 (Session Foundation):
  Task 1 (FiscalPeriod/SessionState) ─→ Task 2 (SessionConfig) ─→ Task 3 (GenerationSession) ─→ Task 4 (CLI per-period)

Phase 2 (Fraud Packs):
  Task 5 (YAML templates + merge) ─→ Task 6 (CLI integration)

Phase 3 (Streaming):
  Task 7 (PhaseSink + StreamPipeline) ─→ Task 8 (CLI integration)

Phase 4 (OCEL Enrichment):
  Task 9 (Lifecycle state machines) ─┐
  Task 10 (Correlation events)  ─────┼─→ Task 12 (Enriched OcpmEvent fields)
  Task 11 (Resource pools) ──────────┘

Phase 5 (Integration Tests):
  Task 13 (Session tests) ─── depends on Tasks 1-4
  Task 14 (OCEL tests) ────── depends on Tasks 9-12
  Task 15 (Fraud tests) ───── depends on Tasks 5-6

Phase 6 (Final):
  Task 16 (Verification) ──── depends on all above
  Task 17 (Version bump) ──── depends on Task 16
```

**Recommended batching:**
- Batch 1: Tasks 1-2 (Core models + config, sequential)
- Batch 2: Tasks 3-4 + Tasks 5-6 + Tasks 7-8 + Tasks 9-11 (parallel tracks: Session, Fraud, Streaming, OCEL)
- Batch 3: Task 12 + Tasks 13-15 (Enriched events + integration tests)
- Batch 4: Tasks 16-17 (Final verification + version bump)
