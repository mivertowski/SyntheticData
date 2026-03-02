//! Diff engine for comparing baseline vs counterfactual output directories.

use crate::scenario_diff::*;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use thiserror::Error;

/// Errors from the diff engine.
#[derive(Debug, Error)]
pub enum DiffError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("CSV parse error: {0}")]
    CsvParse(String),
    #[error("mismatched schemas: baseline has {baseline} columns, counterfactual has {counterfactual} for file {file}")]
    MismatchedSchemas {
        file: String,
        baseline: usize,
        counterfactual: usize,
    },
}

/// Diff format options.
#[derive(Debug, Clone, PartialEq)]
pub enum DiffFormat {
    Summary,
    RecordLevel,
    Aggregate,
}

/// Configuration for diff computation.
#[derive(Debug, Clone)]
pub struct DiffConfig {
    pub formats: Vec<DiffFormat>,
    /// Files to compare (empty = all CSV files found in baseline directory).
    pub scope: Vec<String>,
    pub max_sample_changes: usize,
}

impl Default for DiffConfig {
    fn default() -> Self {
        Self {
            formats: vec![DiffFormat::Summary, DiffFormat::Aggregate],
            scope: vec![],
            max_sample_changes: 1000,
        }
    }
}

/// Engine for computing diffs between baseline and counterfactual outputs.
pub struct DiffEngine;

impl DiffEngine {
    /// Compute a diff between baseline and counterfactual directories.
    pub fn compute(
        baseline_path: &Path,
        counterfactual_path: &Path,
        config: &DiffConfig,
    ) -> Result<ScenarioDiff, DiffError> {
        let summary = if config.formats.contains(&DiffFormat::Summary) {
            Some(Self::compute_summary(baseline_path, counterfactual_path)?)
        } else {
            None
        };

        let record_level = if config.formats.contains(&DiffFormat::RecordLevel) {
            Some(Self::compute_record_level(
                baseline_path,
                counterfactual_path,
                &config.scope,
                config.max_sample_changes,
            )?)
        } else {
            None
        };

        let aggregate = if config.formats.contains(&DiffFormat::Aggregate) {
            Some(Self::compute_aggregate(baseline_path, counterfactual_path)?)
        } else {
            None
        };

        Ok(ScenarioDiff {
            summary,
            record_level,
            aggregate,
            intervention_trace: None, // populated separately by causal engine
        })
    }

    /// Compute impact summary from the two directories.
    fn compute_summary(
        baseline_path: &Path,
        counterfactual_path: &Path,
    ) -> Result<ImpactSummary, DiffError> {
        let mut kpi_impacts = Vec::new();

        // Compare journal_entries.csv if present
        let baseline_je = baseline_path.join("journal_entries.csv");
        let counter_je = counterfactual_path.join("journal_entries.csv");

        if baseline_je.exists() && counter_je.exists() {
            let baseline_stats = Self::csv_stats(&baseline_je)?;
            let counter_stats = Self::csv_stats(&counter_je)?;

            // Record count KPI
            let b_count = baseline_stats.record_count as f64;
            let c_count = counter_stats.record_count as f64;
            kpi_impacts.push(Self::make_kpi("total_transactions", b_count, c_count));

            // Total amount KPI (sum of first numeric column after ID)
            if let (Some(b_sum), Some(c_sum)) =
                (baseline_stats.numeric_sum, counter_stats.numeric_sum)
            {
                kpi_impacts.push(Self::make_kpi("total_amount", b_sum, c_sum));
            }
        }

        // Compare anomaly_labels.csv if present
        let baseline_al = baseline_path.join("anomaly_labels.csv");
        let counter_al = counterfactual_path.join("anomaly_labels.csv");
        let anomaly_impact = if baseline_al.exists() && counter_al.exists() {
            let b_stats = Self::csv_stats(&baseline_al)?;
            let c_stats = Self::csv_stats(&counter_al)?;
            let b_count = b_stats.record_count;
            let c_count = c_stats.record_count;
            let rate_change = if b_count > 0 {
                ((c_count as f64 - b_count as f64) / b_count as f64) * 100.0
            } else if c_count > 0 {
                100.0
            } else {
                0.0
            };

            // Parse anomaly types from both files
            let b_types = Self::extract_anomaly_types(&baseline_al)?;
            let c_types = Self::extract_anomaly_types(&counter_al)?;

            let new_types: Vec<String> = c_types.difference(&b_types).cloned().collect();
            let removed_types: Vec<String> = b_types.difference(&c_types).cloned().collect();

            Some(AnomalyImpact {
                baseline_count: b_count,
                counterfactual_count: c_count,
                new_types,
                removed_types,
                rate_change_pct: rate_change,
            })
        } else {
            None
        };

        // Compute financial statement impacts if trial_balance.csv exists
        let financial_statement_impacts =
            Self::compute_financial_impacts(baseline_path, counterfactual_path)?;

        Ok(ImpactSummary {
            scenario_name: String::new(),
            generation_timestamp: chrono::Utc::now().to_rfc3339(),
            interventions_applied: 0,
            kpi_impacts,
            financial_statement_impacts,
            anomaly_impact,
            control_impact: None,
        })
    }

    /// Compute record-level diffs for CSV files.
    fn compute_record_level(
        baseline_path: &Path,
        counterfactual_path: &Path,
        scope: &[String],
        max_samples: usize,
    ) -> Result<Vec<RecordLevelDiff>, DiffError> {
        let files = if scope.is_empty() {
            Self::find_csv_files(baseline_path)?
        } else {
            scope.to_vec()
        };

        let mut diffs = Vec::new();
        for file in &files {
            let b_path = baseline_path.join(file);
            let c_path = counterfactual_path.join(file);

            if !b_path.exists() || !c_path.exists() {
                continue;
            }

            let diff = Self::diff_csv_file(&b_path, &c_path, file, max_samples)?;
            diffs.push(diff);
        }
        Ok(diffs)
    }

    /// Compute aggregate comparison.
    fn compute_aggregate(
        baseline_path: &Path,
        counterfactual_path: &Path,
    ) -> Result<AggregateComparison, DiffError> {
        let files = Self::find_csv_files(baseline_path)?;
        let mut metrics = Vec::new();

        for file in &files {
            let b_path = baseline_path.join(file);
            let c_path = counterfactual_path.join(file);

            if !c_path.exists() {
                continue;
            }

            let b_stats = Self::csv_stats(&b_path)?;
            let c_stats = Self::csv_stats(&c_path)?;

            let b_count = b_stats.record_count as f64;
            let c_count = c_stats.record_count as f64;
            let change_pct = if b_count > 0.0 {
                ((c_count - b_count) / b_count) * 100.0
            } else {
                0.0
            };

            metrics.push(MetricComparison {
                metric_name: format!("{}_record_count", file.trim_end_matches(".csv")),
                baseline: b_count,
                counterfactual: c_count,
                change_pct,
            });
        }

        Ok(AggregateComparison {
            metrics,
            period_comparisons: vec![],
        })
    }

    /// Create a KpiImpact from baseline and counterfactual values.
    fn make_kpi(name: &str, baseline: f64, counterfactual: f64) -> KpiImpact {
        let abs = counterfactual - baseline;
        let pct = if baseline.abs() > f64::EPSILON {
            (abs / baseline) * 100.0
        } else {
            0.0
        };
        let direction = if abs > f64::EPSILON {
            ChangeDirection::Increase
        } else if abs < -f64::EPSILON {
            ChangeDirection::Decrease
        } else {
            ChangeDirection::Unchanged
        };
        KpiImpact {
            kpi_name: name.to_string(),
            baseline_value: baseline,
            counterfactual_value: counterfactual,
            absolute_change: abs,
            percent_change: pct,
            direction,
        }
    }

    /// Compute basic CSV statistics (record count, column count, first numeric column sum).
    fn csv_stats(path: &Path) -> Result<CsvStats, DiffError> {
        let content = std::fs::read_to_string(path)?;
        let mut lines = content.lines();
        let header = lines.next().unwrap_or("");
        let col_count = header.split(',').count();

        let mut record_count = 0;
        let mut numeric_sum: Option<f64> = None;

        for line in lines {
            if line.trim().is_empty() {
                continue;
            }
            record_count += 1;
            // Try to find a numeric column to sum (skip first column as ID)
            let fields: Vec<&str> = line.split(',').collect();
            for field in fields.iter().skip(1) {
                let trimmed = field.trim().trim_matches('"');
                if let Ok(val) = trimmed.parse::<f64>() {
                    *numeric_sum.get_or_insert(0.0) += val;
                    break;
                }
            }
        }

        Ok(CsvStats {
            record_count,
            _col_count: col_count,
            numeric_sum,
        })
    }

    /// Find all CSV files in a directory, sorted by name.
    fn find_csv_files(dir: &Path) -> Result<Vec<String>, DiffError> {
        let mut files = Vec::new();
        if dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("csv") {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        files.push(name.to_string());
                    }
                }
            }
        }
        files.sort();
        Ok(files)
    }

    /// Diff a single CSV file between baseline and counterfactual directories.
    fn diff_csv_file(
        baseline: &Path,
        counterfactual: &Path,
        file_name: &str,
        max_samples: usize,
    ) -> Result<RecordLevelDiff, DiffError> {
        let b_content = std::fs::read_to_string(baseline)?;
        let c_content = std::fs::read_to_string(counterfactual)?;

        let b_records = Self::parse_csv_records(&b_content);
        let c_records = Self::parse_csv_records(&c_content);

        let b_ids: HashSet<&str> = b_records.keys().copied().collect();
        let c_ids: HashSet<&str> = c_records.keys().copied().collect();

        let added: Vec<&str> = c_ids.difference(&b_ids).copied().collect();
        let removed: Vec<&str> = b_ids.difference(&c_ids).copied().collect();
        let common: Vec<&str> = b_ids.intersection(&c_ids).copied().collect();

        let mut modified_count = 0;
        let mut unchanged_count = 0;
        let mut sample_changes = Vec::new();

        // Get header for field names
        let header: Vec<&str> = b_content.lines().next().unwrap_or("").split(',').collect();

        for id in &common {
            let b_line = b_records[id];
            let c_line = c_records[id];
            if b_line == c_line {
                unchanged_count += 1;
            } else {
                modified_count += 1;
                if sample_changes.len() < max_samples {
                    let b_fields: Vec<&str> = b_line.split(',').collect();
                    let c_fields: Vec<&str> = c_line.split(',').collect();
                    let mut field_changes = Vec::new();
                    for (i, (bf, cf)) in b_fields.iter().zip(c_fields.iter()).enumerate() {
                        if bf != cf {
                            field_changes.push(FieldChange {
                                field_name: header.get(i).unwrap_or(&"unknown").to_string(),
                                baseline_value: bf.to_string(),
                                counterfactual_value: cf.to_string(),
                            });
                        }
                    }
                    sample_changes.push(RecordChange {
                        record_id: id.to_string(),
                        change_type: RecordChangeType::Modified,
                        field_changes,
                    });
                }
            }
        }

        // Add samples for added records
        for id in added
            .iter()
            .take(max_samples.saturating_sub(sample_changes.len()))
        {
            sample_changes.push(RecordChange {
                record_id: id.to_string(),
                change_type: RecordChangeType::Added,
                field_changes: vec![],
            });
        }

        // Add samples for removed records
        for id in removed
            .iter()
            .take(max_samples.saturating_sub(sample_changes.len()))
        {
            sample_changes.push(RecordChange {
                record_id: id.to_string(),
                change_type: RecordChangeType::Removed,
                field_changes: vec![],
            });
        }

        Ok(RecordLevelDiff {
            file_name: file_name.to_string(),
            records_added: added.len(),
            records_removed: removed.len(),
            records_modified: modified_count,
            records_unchanged: unchanged_count,
            sample_changes,
        })
    }

    /// Extract unique anomaly type values from an anomaly_labels CSV.
    /// Looks for a column named "anomaly_type" or "type" in the header.
    fn extract_anomaly_types(path: &Path) -> Result<HashSet<String>, DiffError> {
        let content = std::fs::read_to_string(path)?;
        let mut lines = content.lines();
        let header = lines.next().unwrap_or("");
        let columns: Vec<&str> = header.split(',').collect();

        // Find the type column index
        let type_col = columns
            .iter()
            .position(|c| {
                let trimmed = c.trim().trim_matches('"').to_lowercase();
                trimmed == "anomaly_type" || trimmed == "type"
            })
            .unwrap_or(1); // Default to second column if not found

        let mut types = HashSet::new();
        for line in lines {
            if line.trim().is_empty() {
                continue;
            }
            let fields: Vec<&str> = line.split(',').collect();
            if let Some(field) = fields.get(type_col) {
                let val = field.trim().trim_matches('"').to_string();
                if !val.is_empty() {
                    types.insert(val);
                }
            }
        }
        Ok(types)
    }

    /// Compute financial statement impacts by comparing trial_balance.csv
    /// or balance_sheet.csv between baseline and counterfactual.
    fn compute_financial_impacts(
        baseline_path: &Path,
        counterfactual_path: &Path,
    ) -> Result<Option<FinancialStatementImpact>, DiffError> {
        // Try trial_balance.csv first, then balance_sheet.csv
        let file_candidates = ["trial_balance.csv", "balance_sheet.csv"];
        let mut b_file = None;
        let mut c_file = None;

        for candidate in &file_candidates {
            let bp = baseline_path.join(candidate);
            let cp = counterfactual_path.join(candidate);
            if bp.exists() && cp.exists() {
                b_file = Some(bp);
                c_file = Some(cp);
                break;
            }
        }

        let (b_path, c_path) = match (b_file, c_file) {
            (Some(b), Some(c)) => (b, c),
            _ => return Ok(None),
        };

        let b_items = Self::parse_financial_line_items(&b_path)?;
        let c_items = Self::parse_financial_line_items(&c_path)?;

        let pct_change = |key: &str| -> f64 {
            let b_val = b_items.get(key).copied().unwrap_or(0.0);
            let c_val = c_items.get(key).copied().unwrap_or(0.0);
            if b_val.abs() > f64::EPSILON {
                ((c_val - b_val) / b_val) * 100.0
            } else {
                0.0
            }
        };

        // Collect top changed line items
        let mut line_item_impacts: Vec<LineItemImpact> = b_items
            .keys()
            .chain(c_items.keys())
            .collect::<HashSet<_>>()
            .into_iter()
            .filter_map(|key| {
                let b_val = b_items.get(key).copied().unwrap_or(0.0);
                let c_val = c_items.get(key).copied().unwrap_or(0.0);
                let change = if b_val.abs() > f64::EPSILON {
                    ((c_val - b_val) / b_val) * 100.0
                } else {
                    0.0
                };
                if change.abs() > f64::EPSILON {
                    Some(LineItemImpact {
                        line_item: key.clone(),
                        baseline: b_val,
                        counterfactual: c_val,
                        change_pct: change,
                    })
                } else {
                    None
                }
            })
            .collect();

        // Sort by absolute change percentage, descending
        line_item_impacts.sort_by(|a, b| {
            b.change_pct
                .abs()
                .partial_cmp(&a.change_pct.abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        line_item_impacts.truncate(10);

        Ok(Some(FinancialStatementImpact {
            revenue_change_pct: pct_change("revenue"),
            cogs_change_pct: pct_change("cogs"),
            margin_change_pct: pct_change("gross_margin"),
            net_income_change_pct: pct_change("net_income"),
            total_assets_change_pct: pct_change("total_assets"),
            total_liabilities_change_pct: pct_change("total_liabilities"),
            cash_flow_change_pct: pct_change("cash_flow"),
            top_changed_line_items: line_item_impacts,
        }))
    }

    /// Parse a financial CSV into a map of line item name → value.
    /// Expects columns like: account/line_item, amount/balance/value.
    fn parse_financial_line_items(path: &Path) -> Result<HashMap<String, f64>, DiffError> {
        let content = std::fs::read_to_string(path)?;
        let mut lines = content.lines();
        let header = lines.next().unwrap_or("");
        let columns: Vec<&str> = header.split(',').collect();

        // Find name and value column indices
        let name_col = columns
            .iter()
            .position(|c| {
                let t = c.trim().trim_matches('"').to_lowercase();
                t == "account" || t == "line_item" || t == "item" || t == "name"
            })
            .unwrap_or(0);

        let value_col = columns
            .iter()
            .position(|c| {
                let t = c.trim().trim_matches('"').to_lowercase();
                t == "amount" || t == "balance" || t == "value" || t == "total"
            })
            .unwrap_or(1);

        let mut items = HashMap::new();
        for line in lines {
            if line.trim().is_empty() {
                continue;
            }
            let fields: Vec<&str> = line.split(',').collect();
            if let (Some(name), Some(val_str)) = (fields.get(name_col), fields.get(value_col)) {
                let name = name.trim().trim_matches('"').to_lowercase();
                let val = val_str.trim().trim_matches('"').parse::<f64>().unwrap_or(0.0);
                items.insert(name, val);
            }
        }
        Ok(items)
    }

    /// Parse CSV content into a map of (first-column value) -> (full line).
    fn parse_csv_records(content: &str) -> HashMap<&str, &str> {
        let mut records = HashMap::new();
        for (i, line) in content.lines().enumerate() {
            if i == 0 || line.trim().is_empty() {
                continue; // skip header
            }
            let id = line.split(',').next().unwrap_or("");
            records.insert(id, line);
        }
        records
    }
}

/// Internal statistics for a CSV file.
struct CsvStats {
    record_count: usize,
    _col_count: usize,
    numeric_sum: Option<f64>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_csv(dir: &Path, name: &str, content: &str) {
        fs::write(dir.join(name), content).unwrap();
    }

    #[test]
    fn test_diff_identical_dirs() {
        let baseline = TempDir::new().unwrap();
        let counter = TempDir::new().unwrap();

        let csv = "id,amount,desc\n1,100.0,test\n2,200.0,test2\n";
        write_csv(baseline.path(), "data.csv", csv);
        write_csv(counter.path(), "data.csv", csv);

        let config = DiffConfig {
            formats: vec![
                DiffFormat::Summary,
                DiffFormat::RecordLevel,
                DiffFormat::Aggregate,
            ],
            ..Default::default()
        };

        let diff = DiffEngine::compute(baseline.path(), counter.path(), &config).unwrap();

        // Record level should show no changes
        let records = diff.record_level.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].records_modified, 0);
        assert_eq!(records[0].records_added, 0);
        assert_eq!(records[0].records_removed, 0);
        assert_eq!(records[0].records_unchanged, 2);
    }

    #[test]
    fn test_diff_record_added() {
        let baseline = TempDir::new().unwrap();
        let counter = TempDir::new().unwrap();

        write_csv(baseline.path(), "data.csv", "id,amount\n1,100.0\n");
        write_csv(counter.path(), "data.csv", "id,amount\n1,100.0\n2,200.0\n");

        let config = DiffConfig {
            formats: vec![DiffFormat::RecordLevel],
            ..Default::default()
        };

        let diff = DiffEngine::compute(baseline.path(), counter.path(), &config).unwrap();
        let records = diff.record_level.unwrap();
        assert_eq!(records[0].records_added, 1);
        assert_eq!(records[0].records_unchanged, 1);
    }

    #[test]
    fn test_diff_field_changed() {
        let baseline = TempDir::new().unwrap();
        let counter = TempDir::new().unwrap();

        write_csv(baseline.path(), "data.csv", "id,amount\n1,100.0\n2,200.0\n");
        write_csv(counter.path(), "data.csv", "id,amount\n1,150.0\n2,200.0\n");

        let config = DiffConfig {
            formats: vec![DiffFormat::RecordLevel],
            ..Default::default()
        };

        let diff = DiffEngine::compute(baseline.path(), counter.path(), &config).unwrap();
        let records = diff.record_level.unwrap();
        assert_eq!(records[0].records_modified, 1);
        assert_eq!(records[0].records_unchanged, 1);
        assert_eq!(records[0].sample_changes.len(), 1);
        assert_eq!(
            records[0].sample_changes[0].field_changes[0].field_name,
            "amount"
        );
    }

    #[test]
    fn test_diff_summary_kpis() {
        let baseline = TempDir::new().unwrap();
        let counter = TempDir::new().unwrap();

        write_csv(
            baseline.path(),
            "journal_entries.csv",
            "id,amount\n1,100.0\n2,200.0\n",
        );
        write_csv(
            counter.path(),
            "journal_entries.csv",
            "id,amount\n1,150.0\n2,200.0\n3,50.0\n",
        );

        let config = DiffConfig {
            formats: vec![DiffFormat::Summary],
            ..Default::default()
        };

        let diff = DiffEngine::compute(baseline.path(), counter.path(), &config).unwrap();
        let summary = diff.summary.unwrap();
        assert_eq!(summary.kpi_impacts.len(), 2); // transaction count + total_amount

        let tx_kpi = summary
            .kpi_impacts
            .iter()
            .find(|k| k.kpi_name == "total_transactions")
            .unwrap();
        assert_eq!(tx_kpi.baseline_value, 2.0);
        assert_eq!(tx_kpi.counterfactual_value, 3.0);
        assert_eq!(tx_kpi.direction, ChangeDirection::Increase);
    }

    #[test]
    fn test_diff_anomaly_types_new_and_removed() {
        let baseline = TempDir::new().unwrap();
        let counter = TempDir::new().unwrap();

        write_csv(
            baseline.path(),
            "anomaly_labels.csv",
            "id,anomaly_type,severity\n1,FictitiousTransaction,high\n2,DuplicateEntry,medium\n",
        );
        write_csv(
            counter.path(),
            "anomaly_labels.csv",
            "id,anomaly_type,severity\n1,DuplicateEntry,medium\n2,SplitTransaction,high\n3,BenfordViolation,low\n",
        );

        let config = DiffConfig {
            formats: vec![DiffFormat::Summary],
            ..Default::default()
        };

        let diff = DiffEngine::compute(baseline.path(), counter.path(), &config).unwrap();
        let summary = diff.summary.unwrap();
        let anomaly = summary.anomaly_impact.unwrap();

        assert_eq!(anomaly.baseline_count, 2);
        assert_eq!(anomaly.counterfactual_count, 3);
        assert!(anomaly.new_types.contains(&"SplitTransaction".to_string()));
        assert!(anomaly.new_types.contains(&"BenfordViolation".to_string()));
        assert!(anomaly
            .removed_types
            .contains(&"FictitiousTransaction".to_string()));
        assert!(!anomaly.new_types.contains(&"DuplicateEntry".to_string()));
    }

    #[test]
    fn test_diff_financial_statement_impacts() {
        let baseline = TempDir::new().unwrap();
        let counter = TempDir::new().unwrap();

        write_csv(
            baseline.path(),
            "trial_balance.csv",
            "account,amount\nrevenue,1000000.0\ncogs,600000.0\ntotal_assets,5000000.0\n",
        );
        write_csv(
            counter.path(),
            "trial_balance.csv",
            "account,amount\nrevenue,850000.0\ncogs,550000.0\ntotal_assets,4800000.0\n",
        );

        let config = DiffConfig {
            formats: vec![DiffFormat::Summary],
            ..Default::default()
        };

        let diff = DiffEngine::compute(baseline.path(), counter.path(), &config).unwrap();
        let summary = diff.summary.unwrap();
        let fi = summary.financial_statement_impacts.unwrap();

        assert!(fi.revenue_change_pct < 0.0); // Revenue decreased
        assert!(fi.total_assets_change_pct < 0.0); // Assets decreased
        assert!(!fi.top_changed_line_items.is_empty());
    }

    #[test]
    fn test_diff_aggregate() {
        let baseline = TempDir::new().unwrap();
        let counter = TempDir::new().unwrap();

        write_csv(baseline.path(), "data.csv", "id,val\n1,10\n2,20\n");
        write_csv(counter.path(), "data.csv", "id,val\n1,10\n2,20\n3,30\n");

        let config = DiffConfig {
            formats: vec![DiffFormat::Aggregate],
            ..Default::default()
        };

        let diff = DiffEngine::compute(baseline.path(), counter.path(), &config).unwrap();
        let agg = diff.aggregate.unwrap();
        assert_eq!(agg.metrics.len(), 1);
        assert_eq!(agg.metrics[0].baseline, 2.0);
        assert_eq!(agg.metrics[0].counterfactual, 3.0);
    }
}
