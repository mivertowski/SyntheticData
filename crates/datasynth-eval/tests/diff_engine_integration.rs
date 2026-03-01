#![allow(clippy::unwrap_used)]

use datasynth_eval::diff_engine::{DiffConfig, DiffEngine, DiffFormat};
use datasynth_eval::scenario_diff::*;
use std::fs;
use tempfile::TempDir;

/// Test 1: Summary diff computes KPI changes from journal_entries.csv and anomaly_labels.csv.
///
/// Baseline: 5 journal entry records, 3 anomaly labels.
/// Counterfactual: 7 journal entry records (different amounts), 5 anomaly labels.
#[test]
fn test_summary_diff_computes_kpi_changes() {
    let baseline = TempDir::new().unwrap();
    let counterfactual = TempDir::new().unwrap();

    // Baseline: 5 records, amounts sum = 100+200+300+400+500 = 1500
    fs::write(
        baseline.path().join("journal_entries.csv"),
        "id,amount,description\n\
         JE-001,100.0,Salary expense\n\
         JE-002,200.0,Office supplies\n\
         JE-003,300.0,Rent payment\n\
         JE-004,400.0,Consulting fee\n\
         JE-005,500.0,Insurance premium\n",
    )
    .unwrap();

    // Counterfactual: 7 records, amounts sum = 150+250+300+450+500+600+350 = 2600
    fs::write(
        counterfactual.path().join("journal_entries.csv"),
        "id,amount,description\n\
         JE-001,150.0,Salary expense adjusted\n\
         JE-002,250.0,Office supplies adjusted\n\
         JE-003,300.0,Rent payment\n\
         JE-004,450.0,Consulting fee adjusted\n\
         JE-005,500.0,Insurance premium\n\
         JE-006,600.0,Marketing spend\n\
         JE-007,350.0,Travel reimbursement\n",
    )
    .unwrap();

    // Baseline anomaly labels: 3 records
    fs::write(
        baseline.path().join("anomaly_labels.csv"),
        "id,type,severity\n\
         A-001,DuplicateEntry,low\n\
         A-002,UnusualAmount,medium\n\
         A-003,LatePosting,low\n",
    )
    .unwrap();

    // Counterfactual anomaly labels: 5 records
    fs::write(
        counterfactual.path().join("anomaly_labels.csv"),
        "id,type,severity\n\
         A-001,DuplicateEntry,low\n\
         A-002,UnusualAmount,high\n\
         A-003,LatePosting,low\n\
         A-004,SplitTransaction,medium\n\
         A-005,ThresholdManipulation,high\n",
    )
    .unwrap();

    let config = DiffConfig {
        formats: vec![DiffFormat::Summary],
        ..Default::default()
    };

    let diff = DiffEngine::compute(baseline.path(), counterfactual.path(), &config).unwrap();
    let summary = diff.summary.unwrap();

    // Should have both total_transactions and total_amount KPIs
    assert!(
        summary.kpi_impacts.len() >= 2,
        "Expected at least 2 KPI impacts, got {}",
        summary.kpi_impacts.len()
    );

    // Check total_transactions KPI
    let tx_kpi = summary
        .kpi_impacts
        .iter()
        .find(|k| k.kpi_name == "total_transactions")
        .expect("Should have total_transactions KPI");
    assert_eq!(tx_kpi.baseline_value, 5.0);
    assert_eq!(tx_kpi.counterfactual_value, 7.0);
    assert_eq!(tx_kpi.absolute_change, 2.0);
    assert_eq!(tx_kpi.direction, ChangeDirection::Increase);

    // Check total_amount KPI
    let amount_kpi = summary
        .kpi_impacts
        .iter()
        .find(|k| k.kpi_name == "total_amount")
        .expect("Should have total_amount KPI");
    assert!(
        (amount_kpi.baseline_value - 1500.0).abs() < 0.01,
        "Baseline total_amount should be 1500.0, got {}",
        amount_kpi.baseline_value
    );
    assert!(
        (amount_kpi.counterfactual_value - 2600.0).abs() < 0.01,
        "Counterfactual total_amount should be 2600.0, got {}",
        amount_kpi.counterfactual_value
    );
    assert_eq!(amount_kpi.direction, ChangeDirection::Increase);

    // Check anomaly impact
    let anomaly = summary
        .anomaly_impact
        .as_ref()
        .expect("Should have anomaly impact");
    assert_eq!(anomaly.baseline_count, 3);
    assert_eq!(anomaly.counterfactual_count, 5);
    // rate_change_pct = ((5 - 3) / 3) * 100 ≈ 66.67%
    assert!(
        (anomaly.rate_change_pct - 66.666).abs() < 1.0,
        "Anomaly rate change should be ~66.67%, got {}",
        anomaly.rate_change_pct
    );
}

/// Test 2: Record-level diff identifies added, removed, modified, and unchanged records.
///
/// Baseline has records: R1, R2, R3, R4
/// Counterfactual has records: R1, R2 (modified), R3, R5 (added)
/// So: R1 + R3 = unchanged (2), R2 = modified (1), R4 = removed (1), R5 = added (1)
#[test]
fn test_record_level_identifies_changes() {
    let baseline = TempDir::new().unwrap();
    let counterfactual = TempDir::new().unwrap();

    // Baseline: 4 records
    fs::write(
        baseline.path().join("journal_entries.csv"),
        "id,amount,account\n\
         R1,100.0,4000\n\
         R2,200.0,5000\n\
         R3,300.0,6000\n\
         R4,400.0,7000\n",
    )
    .unwrap();

    // Counterfactual: R1 same, R2 modified (amount changed), R3 same, R4 removed, R5 added
    fs::write(
        counterfactual.path().join("journal_entries.csv"),
        "id,amount,account\n\
         R1,100.0,4000\n\
         R2,250.0,5000\n\
         R3,300.0,6000\n\
         R5,500.0,8000\n",
    )
    .unwrap();

    let config = DiffConfig {
        formats: vec![DiffFormat::RecordLevel],
        scope: vec!["journal_entries.csv".to_string()],
        max_sample_changes: 100,
    };

    let diff = DiffEngine::compute(baseline.path(), counterfactual.path(), &config).unwrap();
    let records = diff.record_level.unwrap();
    assert_eq!(records.len(), 1, "Should have one file diff");

    let file_diff = &records[0];
    assert_eq!(file_diff.file_name, "journal_entries.csv");
    assert_eq!(file_diff.records_unchanged, 2, "R1 and R3 are unchanged");
    assert_eq!(file_diff.records_modified, 1, "R2 is modified");
    assert_eq!(file_diff.records_added, 1, "R5 is added");
    assert_eq!(file_diff.records_removed, 1, "R4 is removed");

    // Check that sample_changes contains a Modified entry for R2 with the "amount" field
    let modified_change = file_diff
        .sample_changes
        .iter()
        .find(|c| c.change_type == RecordChangeType::Modified)
        .expect("Should have a Modified change");
    assert_eq!(modified_change.record_id, "R2");
    assert!(
        modified_change
            .field_changes
            .iter()
            .any(|f| f.field_name == "amount"),
        "Modified record should show 'amount' field change"
    );
    let amount_change = modified_change
        .field_changes
        .iter()
        .find(|f| f.field_name == "amount")
        .unwrap();
    assert_eq!(amount_change.baseline_value, "200.0");
    assert_eq!(amount_change.counterfactual_value, "250.0");

    // Check Added entry for R5
    let added_change = file_diff
        .sample_changes
        .iter()
        .find(|c| c.change_type == RecordChangeType::Added)
        .expect("Should have an Added change");
    assert_eq!(added_change.record_id, "R5");

    // Check Removed entry for R4
    let removed_change = file_diff
        .sample_changes
        .iter()
        .find(|c| c.change_type == RecordChangeType::Removed)
        .expect("Should have a Removed change");
    assert_eq!(removed_change.record_id, "R4");
}

/// Test 3: Aggregate comparison computes metrics across multiple CSV files.
///
/// Baseline: data.csv (3 records), other.csv (5 records)
/// Counterfactual: data.csv (6 records), other.csv (4 records)
#[test]
fn test_aggregate_computes_metrics() {
    let baseline = TempDir::new().unwrap();
    let counterfactual = TempDir::new().unwrap();

    // Baseline data.csv: 3 records
    fs::write(
        baseline.path().join("data.csv"),
        "id,value\n\
         D1,10\n\
         D2,20\n\
         D3,30\n",
    )
    .unwrap();

    // Baseline other.csv: 5 records
    fs::write(
        baseline.path().join("other.csv"),
        "id,label\n\
         O1,alpha\n\
         O2,beta\n\
         O3,gamma\n\
         O4,delta\n\
         O5,epsilon\n",
    )
    .unwrap();

    // Counterfactual data.csv: 6 records (doubled)
    fs::write(
        counterfactual.path().join("data.csv"),
        "id,value\n\
         D1,10\n\
         D2,20\n\
         D3,30\n\
         D4,40\n\
         D5,50\n\
         D6,60\n",
    )
    .unwrap();

    // Counterfactual other.csv: 4 records (one fewer)
    fs::write(
        counterfactual.path().join("other.csv"),
        "id,label\n\
         O1,alpha\n\
         O2,beta\n\
         O3,gamma\n\
         O4,delta\n",
    )
    .unwrap();

    let config = DiffConfig {
        formats: vec![DiffFormat::Aggregate],
        ..Default::default()
    };

    let diff = DiffEngine::compute(baseline.path(), counterfactual.path(), &config).unwrap();
    let agg = diff.aggregate.unwrap();

    assert_eq!(
        agg.metrics.len(),
        2,
        "Should have 2 metric entries (one per CSV file)"
    );

    // Metrics are sorted by file name, so data.csv comes before other.csv
    let data_metric = agg
        .metrics
        .iter()
        .find(|m| m.metric_name == "data_record_count")
        .expect("Should have data_record_count metric");
    assert_eq!(data_metric.baseline, 3.0);
    assert_eq!(data_metric.counterfactual, 6.0);
    // change_pct = ((6-3)/3) * 100 = 100%
    assert!(
        (data_metric.change_pct - 100.0).abs() < 0.01,
        "data change_pct should be 100%, got {}",
        data_metric.change_pct
    );

    let other_metric = agg
        .metrics
        .iter()
        .find(|m| m.metric_name == "other_record_count")
        .expect("Should have other_record_count metric");
    assert_eq!(other_metric.baseline, 5.0);
    assert_eq!(other_metric.counterfactual, 4.0);
    // change_pct = ((4-5)/5) * 100 = -20%
    assert!(
        (other_metric.change_pct - (-20.0)).abs() < 0.01,
        "other change_pct should be -20%, got {}",
        other_metric.change_pct
    );
}

/// Test 4: Full diff with all formats ("all") populates summary, record_level, and aggregate.
#[test]
fn test_full_diff_all_formats() {
    let baseline = TempDir::new().unwrap();
    let counterfactual = TempDir::new().unwrap();

    // Write journal_entries.csv for summary KPI computation
    fs::write(
        baseline.path().join("journal_entries.csv"),
        "id,amount\n\
         JE-001,100.0\n\
         JE-002,200.0\n\
         JE-003,300.0\n",
    )
    .unwrap();

    fs::write(
        counterfactual.path().join("journal_entries.csv"),
        "id,amount\n\
         JE-001,100.0\n\
         JE-002,250.0\n\
         JE-003,300.0\n\
         JE-004,400.0\n",
    )
    .unwrap();

    // Write anomaly_labels.csv for anomaly impact
    fs::write(
        baseline.path().join("anomaly_labels.csv"),
        "id,type\n\
         A-001,DuplicateEntry\n",
    )
    .unwrap();

    fs::write(
        counterfactual.path().join("anomaly_labels.csv"),
        "id,type\n\
         A-001,DuplicateEntry\n\
         A-002,UnusualAmount\n",
    )
    .unwrap();

    // Request all three diff formats
    let config = DiffConfig {
        formats: vec![DiffFormat::Summary, DiffFormat::RecordLevel, DiffFormat::Aggregate],
        scope: vec![],
        max_sample_changes: 100,
    };

    let diff = DiffEngine::compute(baseline.path(), counterfactual.path(), &config).unwrap();

    // Verify summary is populated
    let summary = diff.summary.as_ref().expect("summary should be populated");
    assert!(
        !summary.kpi_impacts.is_empty(),
        "summary should have KPI impacts"
    );
    assert!(
        summary.anomaly_impact.is_some(),
        "summary should have anomaly impact"
    );

    // Verify record_level is populated
    let record_level = diff
        .record_level
        .as_ref()
        .expect("record_level should be populated");
    // Should have diffs for both CSV files found in baseline
    assert!(
        !record_level.is_empty(),
        "record_level should have at least one file diff"
    );
    // The journal_entries.csv diff should show: 2 unchanged, 1 modified, 1 added
    let je_diff = record_level
        .iter()
        .find(|r| r.file_name == "journal_entries.csv")
        .expect("Should have journal_entries.csv diff");
    assert_eq!(je_diff.records_unchanged, 2);
    assert_eq!(je_diff.records_modified, 1);
    assert_eq!(je_diff.records_added, 1);
    assert_eq!(je_diff.records_removed, 0);

    // Verify aggregate is populated
    let aggregate = diff
        .aggregate
        .as_ref()
        .expect("aggregate should be populated");
    assert!(
        !aggregate.metrics.is_empty(),
        "aggregate should have metric entries"
    );
    // Should have metrics for both anomaly_labels and journal_entries
    let metric_names: Vec<&str> = aggregate
        .metrics
        .iter()
        .map(|m| m.metric_name.as_str())
        .collect();
    assert!(
        metric_names.contains(&"journal_entries_record_count"),
        "Should have journal_entries metric, got: {:?}",
        metric_names
    );
    assert!(
        metric_names.contains(&"anomaly_labels_record_count"),
        "Should have anomaly_labels metric, got: {:?}",
        metric_names
    );

    // intervention_trace should be None (populated separately by causal engine)
    assert!(diff.intervention_trace.is_none());
}
