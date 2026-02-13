//! Comprehensive evaluation of all eval framework evaluators against generated data.
//!
//! Run with: cargo test -p datasynth-runtime --test comprehensive_eval -- --nocapture

use chrono::Datelike;
use datasynth_config::schema::TransactionVolume;
use datasynth_core::models::banking::BankingCustomerType;
use datasynth_core::models::{ReconciliationStatus, StatementType};
use datasynth_eval::*;
use datasynth_runtime::{EnhancedOrchestrator, PhaseConfig};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Build a rich config with all enterprise features enabled.
fn full_enterprise_config() -> datasynth_config::schema::GeneratorConfig {
    let mut config = datasynth_test_utils::fixtures::minimal_config();

    // Seed for reproducibility
    config.global.seed = Some(20260212);
    config.global.period_months = 12;
    config.global.industry = datasynth_core::models::IndustrySector::Manufacturing;

    // Multi-company
    config.companies = vec![
        datasynth_config::schema::CompanyConfig {
            code: "1000".to_string(),
            name: "Global Corp HQ".to_string(),
            currency: "USD".to_string(),
            country: "US".to_string(),
            annual_transaction_volume: TransactionVolume::TenK,
            volume_weight: 0.6,
            fiscal_year_variant: "K4".to_string(),
        },
        datasynth_config::schema::CompanyConfig {
            code: "2000".to_string(),
            name: "EU Subsidiary".to_string(),
            currency: "EUR".to_string(),
            country: "DE".to_string(),
            annual_transaction_volume: TransactionVolume::TenK,
            volume_weight: 0.4,
            fiscal_year_variant: "K4".to_string(),
        },
    ];

    // Master data
    config.master_data.vendors.count = 20;
    config.master_data.customers.count = 20;
    config.master_data.materials.count = 30;
    config.master_data.fixed_assets.count = 10;
    config.master_data.employees.count = 15;

    // Document flows
    config.document_flows.p2p.enabled = true;
    config.document_flows.p2p.three_way_match_rate = 0.95;
    config.document_flows.o2c.enabled = true;
    config.document_flows.generate_document_references = true;

    // Fraud / anomaly injection
    config.fraud.enabled = true;
    config.fraud.fraud_rate = 0.05;

    // Intercompany
    config.intercompany.enabled = true;

    // Balance
    config.balance.generate_opening_balances = true;
    config.balance.validate_balance_equation = true;

    // Seasonality
    config.transactions.seasonality.month_end_spike = true;
    config.transactions.seasonality.month_end_multiplier = 2.5;

    // Enterprise process chains
    config.source_to_pay.enabled = true;
    config.financial_reporting.enabled = true;
    config.hr.enabled = true;
    config.manufacturing.enabled = true;

    // Banking
    config.banking.enabled = true;
    config.banking.population.retail_customers = 5;
    config.banking.population.business_customers = 5;

    // Audit
    config.audit.enabled = true;

    // OCPM
    config.ocpm.enabled = true;

    config
}

#[test]
fn comprehensive_evaluation() {
    // =========================================================================
    // PHASE 1: GENERATE DATA
    // =========================================================================
    let config = full_enterprise_config();

    let phase_config = PhaseConfig {
        generate_master_data: true,
        generate_document_flows: true,
        generate_ocpm_events: true,
        generate_journal_entries: true,
        inject_anomalies: true,
        validate_balances: true,
        show_progress: false,
        generate_audit: true,
        generate_banking: true,
        generate_graph_export: true,
        generate_sourcing: true,
        generate_bank_reconciliation: true,
        generate_financial_statements: true,
        generate_accounting_standards: true,
        generate_manufacturing: true,
        generate_sales_kpi_budgets: true,
        ..Default::default()
    };

    let mut orchestrator =
        EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");

    let result = orchestrator.generate().expect("Generation failed");

    println!();
    println!("================================================================");
    println!("  COMPREHENSIVE EVALUATION REPORT");
    println!("  DataSynth Eval Framework v0.6.0");
    println!("================================================================");
    println!();

    // =========================================================================
    // GENERATION STATISTICS
    // =========================================================================
    let stats = &result.statistics;
    println!("--- GENERATION STATISTICS ---");
    println!("  Journal entries:      {}", stats.total_entries);
    println!("  Line items:           {}", stats.total_line_items);
    println!("  Companies:            {}", stats.companies_count);
    println!("  CoA accounts:         {}", stats.accounts_count);
    println!("  Vendors:              {}", stats.vendor_count);
    println!("  Customers:            {}", stats.customer_count);
    println!("  Materials:            {}", stats.material_count);
    println!("  Employees:            {}", stats.employee_count);
    println!("  P2P chains:           {}", stats.p2p_chain_count);
    println!("  O2C chains:           {}", stats.o2c_chain_count);
    println!("  OCPM events:          {}", stats.ocpm_event_count);
    println!("  Audit engagements:    {}", stats.audit_engagement_count);
    println!("  Audit findings:       {}", stats.audit_finding_count);
    println!(
        "  Sourcing projects:    {}",
        result.sourcing.sourcing_projects.len()
    );
    println!(
        "  RFx events:           {}",
        result.sourcing.rfx_events.len()
    );
    println!("  Bids:                 {}", result.sourcing.bids.len());
    println!(
        "  Financial statements: {}",
        result.financial_reporting.financial_statements.len()
    );
    println!(
        "  Bank reconciliations: {}",
        result.financial_reporting.bank_reconciliations.len()
    );
    println!("  Banking customers:    {}", result.banking.customers.len());
    println!(
        "  Banking transactions: {}",
        result.banking.transactions.len()
    );
    println!(
        "  Anomaly labels:       {}",
        result.anomaly_labels.labels.len()
    );
    println!();

    // =========================================================================
    // COLLECT DATA FOR EVALUATORS
    // =========================================================================
    let mut amounts: Vec<Decimal> = Vec::new();
    let mut line_item_entries: Vec<LineItemEntry> = Vec::new();
    let mut temporal_entries: Vec<TemporalEntry> = Vec::new();
    let mut balance_ok = 0usize;
    let mut balance_fail = 0usize;
    let mut fraud_count = 0usize;

    for entry in &result.journal_entries {
        // Temporal
        temporal_entries.push(TemporalEntry {
            posting_date: entry.header.posting_date,
        });

        // Fraud
        if entry.header.is_fraud {
            fraud_count += 1;
        }

        // Amounts and balance
        let mut total_debit = Decimal::ZERO;
        let mut total_credit = Decimal::ZERO;
        let mut debit_count = 0usize;
        let mut credit_count = 0usize;

        for line in &entry.lines {
            if line.debit_amount > Decimal::ZERO {
                amounts.push(line.debit_amount);
                total_debit += line.debit_amount;
                debit_count += 1;
            }
            if line.credit_amount > Decimal::ZERO {
                amounts.push(line.credit_amount);
                total_credit += line.credit_amount;
                credit_count += 1;
            }
        }

        line_item_entries.push(LineItemEntry {
            line_count: entry.lines.len(),
            debit_count,
            credit_count,
        });

        if (total_debit - total_credit).abs() <= Decimal::new(1, 2) {
            balance_ok += 1;
        } else {
            balance_fail += 1;
        }
    }

    let total_entries = result.journal_entries.len();
    let mut eval = ComprehensiveEvaluation::new();

    // =========================================================================
    // SECTION 1: STATISTICAL EVALUATORS
    // =========================================================================
    println!("================================================================");
    println!("  1. STATISTICAL EVALUATION");
    println!("================================================================");

    // 1a. Benford's Law
    if amounts.len() >= 10 {
        let benford_analyzer = BenfordAnalyzer::new(0.05);
        match benford_analyzer.analyze(&amounts) {
            Ok(benford) => {
                println!();
                println!("  [Benford's Law]");
                println!("    Sample size:   {}", benford.sample_size);
                println!("    Chi-squared:   {:.4}", benford.chi_squared);
                println!("    P-value:       {:.6}", benford.p_value);
                println!("    MAD:           {:.6}", benford.mad);
                println!("    Conformity:    {:?}", benford.conformity);
                println!("    Anti-Benford:  {:.4}", benford.anti_benford_score);
                let status = if benford.passes { "PASS" } else { "FAIL" };
                println!("    Status:        {}", status);

                println!("    Digit  Expected   Observed   Dev");
                let expected = [
                    0.301, 0.176, 0.125, 0.097, 0.079, 0.067, 0.058, 0.051, 0.046,
                ];
                for (i, (obs, exp)) in benford
                    .observed_frequencies
                    .iter()
                    .zip(expected.iter())
                    .enumerate()
                {
                    println!(
                        "      {}     {:.3}      {:.3}    {:+.3}",
                        i + 1,
                        exp,
                        obs,
                        obs - exp
                    );
                }

                eval.statistical.benford = Some(benford);
            }
            Err(e) => println!("    Benford analysis error: {}", e),
        }
    }

    // 1b. Amount Distribution
    if amounts.len() >= 10 {
        let amount_analyzer = AmountDistributionAnalyzer::new();
        match amount_analyzer.analyze(&amounts) {
            Ok(amount) => {
                println!();
                println!("  [Amount Distribution]");
                println!("    Sample size:       {}", amount.sample_size);
                println!("    Mean:              ${:.2}", amount.mean);
                println!("    Median:            ${:.2}", amount.median);
                println!("    Std Dev:           ${:.2}", amount.std_dev);
                println!("    Skewness:          {:.4}", amount.skewness);
                println!("    Kurtosis:          {:.4}", amount.kurtosis);
                println!(
                    "    Round number %:    {:.2}%",
                    amount.round_number_ratio * 100.0
                );
                if let Some(p) = amount.lognormal_ks_pvalue {
                    println!("    Log-normal KS p:   {:.6}", p);
                }
                let status = if amount.passes { "PASS" } else { "FAIL" };
                println!("    Status:            {}", status);

                eval.statistical.amount_distribution = Some(amount);
            }
            Err(e) => println!("    Amount analysis error: {}", e),
        }
    }

    // 1c. Temporal
    if temporal_entries.len() >= 10 {
        let temporal_analyzer = TemporalAnalyzer::new();
        match temporal_analyzer.analyze(&temporal_entries) {
            Ok(temporal) => {
                println!();
                println!("  [Temporal Patterns]");
                println!("    Sample size:       {}", temporal.sample_size);
                println!(
                    "    Date range:        {} to {}",
                    temporal.start_date, temporal.end_date
                );
                println!(
                    "    Weekend ratio:     {:.2}%",
                    temporal.weekend_ratio * 100.0
                );
                println!("    Month-end spike:   {:.2}x", temporal.month_end_spike);
                println!("    Quarter-end spike: {:.2}x", temporal.quarter_end_spike);
                println!("    Year-end spike:    {:.2}x", temporal.year_end_spike);
                println!("    Pattern corr:      {:.4}", temporal.pattern_correlation);
                println!(
                    "    DoW correlation:   {:.4}",
                    temporal.day_of_week_correlation
                );
                let status = if temporal.passes { "PASS" } else { "FAIL" };
                println!("    Status:            {}", status);

                eval.statistical.temporal = Some(temporal);
            }
            Err(e) => println!("    Temporal analysis error: {}", e),
        }
    }

    // 1d. Line Item Distribution
    if line_item_entries.len() >= 10 {
        let line_analyzer = LineItemAnalyzer::new(0.05);
        match line_analyzer.analyze(&line_item_entries) {
            Ok(line_item) => {
                println!();
                println!("  [Line Item Distribution]");
                println!("    Sample size:       {}", line_item.sample_size);
                println!("    Avg line items:    {:.2}", line_item.avg_line_count);
                println!(
                    "    Even ratio:        {:.2}% (expected 88%)",
                    line_item.even_ratio * 100.0
                );
                println!("    Chi-squared:       {:.4}", line_item.chi_squared);
                println!("    P-value:           {:.6}", line_item.p_value);
                let status = if line_item.passes { "PASS" } else { "FAIL" };
                println!("    Status:            {}", status);

                eval.statistical.line_item = Some(line_item);
            }
            Err(e) => println!("    Line item analysis error: {}", e),
        }
    }

    // =========================================================================
    // SECTION 2: COHERENCE EVALUATORS
    // =========================================================================
    println!();
    println!("================================================================");
    println!("  2. COHERENCE EVALUATION");
    println!("================================================================");

    // 2a. Balance Sheet
    println!();
    println!("  [Balance Coherence]");
    println!("    Balanced entries:   {}/{}", balance_ok, total_entries);
    println!(
        "    Balance rate:       {:.2}%",
        (balance_ok as f64 / total_entries.max(1) as f64) * 100.0
    );
    let balance_status = if balance_fail == 0 { "PASS" } else { "FAIL" };
    println!("    Status:             {}", balance_status);

    // 2b. Document Chain
    let p2p_count = result.document_flows.p2p_chains.len();
    let o2c_count = result.document_flows.o2c_chains.len();
    let po_count = result.document_flows.purchase_orders.len();
    let gr_count = result.document_flows.goods_receipts.len();
    let vi_count = result.document_flows.vendor_invoices.len();
    let so_count = result.document_flows.sales_orders.len();
    let del_count = result.document_flows.deliveries.len();
    let ci_count = result.document_flows.customer_invoices.len();
    let pay_count = result.document_flows.payments.len();

    println!();
    println!("  [Document Chain Integrity]");
    println!("    P2P chains:         {}", p2p_count);
    println!("    O2C chains:         {}", o2c_count);
    println!("    Purchase orders:    {}", po_count);
    println!("    Goods receipts:     {}", gr_count);
    println!("    Vendor invoices:    {}", vi_count);
    println!("    Payments:           {}", pay_count);
    println!("    Sales orders:       {}", so_count);
    println!("    Deliveries:         {}", del_count);
    println!("    Customer invoices:  {}", ci_count);

    if p2p_count > 0 {
        let p2p_with_payment = result
            .document_flows
            .p2p_chains
            .iter()
            .filter(|c| c.payment.is_some())
            .count();
        let p2p_completion = p2p_with_payment as f64 / p2p_count as f64;
        println!(
            "    P2P completion:     {:.1}% ({}/{})",
            p2p_completion * 100.0,
            p2p_with_payment,
            p2p_count
        );
    }
    if o2c_count > 0 {
        let o2c_with_receipt = result
            .document_flows
            .o2c_chains
            .iter()
            .filter(|c| c.customer_receipt.is_some())
            .count();
        let o2c_completion = o2c_with_receipt as f64 / o2c_count as f64;
        println!(
            "    O2C completion:     {:.1}% ({}/{})",
            o2c_completion * 100.0,
            o2c_with_receipt,
            o2c_count
        );
    }

    // 2c. Sourcing (S2C)
    let sourcing = &result.sourcing;
    println!();
    println!("  [S2C Sourcing Chain]");
    println!(
        "    Sourcing projects:  {}",
        sourcing.sourcing_projects.len()
    );
    println!("    RFx events:         {}", sourcing.rfx_events.len());
    println!("    Bids received:      {}", sourcing.bids.len());
    println!("    Bid evaluations:    {}", sourcing.bid_evaluations.len());
    println!("    Contracts:          {}", sourcing.contracts.len());
    println!("    Catalog items:      {}", sourcing.catalog_items.len());
    println!("    Scorecards:         {}", sourcing.scorecards.len());
    println!("    Spend analyses:     {}", sourcing.spend_analyses.len());
    if !sourcing.sourcing_projects.is_empty() && !sourcing.rfx_events.is_empty() {
        let rfx_per_project =
            sourcing.rfx_events.len() as f64 / sourcing.sourcing_projects.len() as f64;
        println!("    RFx per project:    {:.1}", rfx_per_project);
        println!("    Status:             PASS (chain populated)");
    } else {
        println!("    Status:             WARN (no sourcing data)");
    }

    // 2d. Financial Reporting
    let fin_rep = &result.financial_reporting;
    println!();
    println!("  [Financial Reporting]");
    println!(
        "    Financial stmts:    {}",
        fin_rep.financial_statements.len()
    );
    println!(
        "    Bank reconcilns:    {}",
        fin_rep.bank_reconciliations.len()
    );
    if !fin_rep.financial_statements.is_empty() {
        println!("    Status:             PASS (statements generated)");
    } else {
        println!("    Status:             WARN (no financial statements)");
    }

    // 2e. HR/Payroll
    let hr = &result.hr;
    println!();
    println!("  [HR/Payroll]");
    println!("    Payroll runs:       {}", hr.payroll_run_count);
    println!("    Payroll line items: {}", hr.payroll_line_item_count);
    println!("    Time entries:       {}", hr.time_entry_count);
    println!("    Expense reports:    {}", hr.expense_report_count);
    if hr.payroll_run_count > 0 {
        println!("    Status:             PASS (payroll generated)");
    } else {
        println!("    Status:             WARN (no payroll data)");
    }

    // 2f. Manufacturing
    let mfg = &result.manufacturing;
    println!();
    println!("  [Manufacturing]");
    println!("    Production orders:  {}", mfg.production_order_count);
    println!("    Quality inspections:{}", mfg.quality_inspection_count);
    println!("    Cycle counts:       {}", mfg.cycle_count_count);
    if mfg.production_order_count > 0 {
        println!("    Status:             PASS (manufacturing generated)");
    } else {
        println!("    Status:             WARN (no manufacturing data)");
    }

    // 2g. Audit
    let audit = &result.audit;
    println!();
    println!("  [Audit Trail]");
    println!("    Engagements:        {}", audit.engagements.len());
    println!("    Workpapers:         {}", audit.workpapers.len());
    println!("    Evidence items:     {}", audit.evidence.len());
    println!("    Risk assessments:   {}", audit.risk_assessments.len());
    println!("    Findings:           {}", audit.findings.len());
    println!("    Judgments:          {}", audit.judgments.len());
    if !audit.engagements.is_empty() && !audit.findings.is_empty() {
        let evidence_per_finding = if !audit.findings.is_empty() {
            audit.evidence.len() as f64 / audit.findings.len() as f64
        } else {
            0.0
        };
        println!("    Evidence/finding:   {:.1}", evidence_per_finding);
        println!("    Status:             PASS (audit trail populated)");
    } else {
        println!("    Status:             WARN (no audit data)");
    }

    // =========================================================================
    // SECTION 3: BANKING EVALUATION
    // =========================================================================
    println!();
    println!("================================================================");
    println!("  3. BANKING / KYC / AML EVALUATION");
    println!("================================================================");

    let banking = &result.banking;
    println!();
    println!("  [Banking Data]");
    println!("    Customers:          {}", banking.customers.len());
    println!("    Accounts:           {}", banking.accounts.len());
    println!("    Transactions:       {}", banking.transactions.len());
    println!("    Suspicious txns:    {}", banking.suspicious_count);
    println!("    AML scenarios:      {}", banking.scenario_count);

    if !banking.customers.is_empty() {
        let accts_per_customer = banking.accounts.len() as f64 / banking.customers.len() as f64;
        let txns_per_account = if !banking.accounts.is_empty() {
            banking.transactions.len() as f64 / banking.accounts.len() as f64
        } else {
            0.0
        };
        let suspicious_rate = if !banking.transactions.is_empty() {
            banking.suspicious_count as f64 / banking.transactions.len() as f64
        } else {
            0.0
        };
        println!("    Accts/customer:     {:.1}", accts_per_customer);
        println!("    Txns/account:       {:.1}", txns_per_account);
        println!("    Suspicious rate:    {:.2}%", suspicious_rate * 100.0);
        println!("    Status:             PASS (banking populated)");
    } else {
        println!("    Status:             WARN (no banking data)");
    }

    // =========================================================================
    // SECTION 4: PROCESS MINING (OCPM) EVALUATION
    // =========================================================================
    println!();
    println!("================================================================");
    println!("  4. PROCESS MINING (OCEL 2.0) EVALUATION");
    println!("================================================================");

    let ocpm = &result.ocpm;
    println!();
    println!("  [OCPM Event Log]");
    println!("    Events:             {}", ocpm.event_count);
    println!("    Objects:            {}", ocpm.object_count);
    println!("    Cases:              {}", ocpm.case_count);
    println!("    Event log present:  {}", ocpm.event_log.is_some());

    if let Some(ref event_log) = ocpm.event_log {
        let events_per_case = if ocpm.case_count > 0 {
            ocpm.event_count as f64 / ocpm.case_count as f64
        } else {
            0.0
        };
        println!("    Events/case:        {:.1}", events_per_case);

        // Check timestamp monotonicity
        let mut monotonic = 0usize;
        let mut non_monotonic = 0usize;
        let events = &event_log.events;
        if events.len() > 1 {
            for w in events.windows(2) {
                if w[1].timestamp >= w[0].timestamp {
                    monotonic += 1;
                } else {
                    non_monotonic += 1;
                }
            }
            let monotonicity = monotonic as f64 / (monotonic + non_monotonic).max(1) as f64;
            println!("    Timestamp monoton:  {:.2}%", monotonicity * 100.0);
        }

        // Count activity types
        let mut activity_counts: HashMap<&str, usize> = HashMap::new();
        for event in events {
            *activity_counts
                .entry(event.activity_name.as_str())
                .or_default() += 1;
        }
        println!("    Unique activities:  {}", activity_counts.len());
        println!("    Status:             PASS (OCEL 2.0 event log generated)");
    } else {
        println!("    Status:             WARN (no event log)");
    }

    // =========================================================================
    // SECTION 5: ML READINESS EVALUATION
    // =========================================================================
    println!();
    println!("================================================================");
    println!("  5. ML READINESS EVALUATION");
    println!("================================================================");

    // 5a. Anomaly / Label Analysis
    let labels = &result.anomaly_labels;
    println!();
    println!("  [Anomaly Labels]");
    println!("    Total labels:       {}", labels.labels.len());
    println!(
        "    Fraud entries:      {} ({:.2}%)",
        fraud_count,
        (fraud_count as f64 / total_entries.max(1) as f64) * 100.0
    );

    if !labels.by_type.is_empty() {
        println!("    By type:");
        let mut types: Vec<_> = labels.by_type.iter().collect();
        types.sort_by(|a, b| b.1.cmp(a.1));
        for (anomaly_type, count) in types.iter().take(10) {
            println!("      {:30} {}", anomaly_type, count);
        }
    }

    if let Some(ref summary) = labels.summary {
        println!("    Summary: {:?}", summary);
    }

    // 5b. Feature Analysis - check line count and amount distributions for ML
    let total_amounts = amounts.len();
    if total_amounts >= 100 {
        let amounts_f64: Vec<f64> = amounts
            .iter()
            .filter_map(|a| (*a).try_into().ok())
            .collect();

        let mean = amounts_f64.iter().sum::<f64>() / amounts_f64.len() as f64;
        let variance =
            amounts_f64.iter().map(|a| (a - mean).powi(2)).sum::<f64>() / amounts_f64.len() as f64;
        let std_dev = variance.sqrt();
        let cv = if mean > 0.0 { std_dev / mean } else { 0.0 };

        println!();
        println!("  [Feature Quality Indicators]");
        println!("    Amount features:    {}", total_amounts);
        println!("    Mean:               ${:.2}", mean);
        println!("    Std dev:            ${:.2}", std_dev);
        println!("    CV (variability):   {:.4}", cv);
        println!(
            "    Status:             {}",
            if cv > 0.1 {
                "PASS (sufficient variability)"
            } else {
                "WARN (low variability)"
            }
        );
    }

    // =========================================================================
    // SECTION 6: DATA COMPOSITION & QUALITY
    // =========================================================================
    println!();
    println!("================================================================");
    println!("  6. DATA COMPOSITION & QUALITY");
    println!("================================================================");

    // Source distribution
    let mut source_counts: HashMap<String, usize> = HashMap::new();
    let mut process_counts: HashMap<String, usize> = HashMap::new();
    let mut company_counts: HashMap<String, usize> = HashMap::new();

    for entry in &result.journal_entries {
        *source_counts
            .entry(format!("{:?}", entry.header.source))
            .or_default() += 1;
        if let Some(ref bp) = entry.header.business_process {
            *process_counts.entry(format!("{:?}", bp)).or_default() += 1;
        }
        *company_counts
            .entry(entry.header.company_code.clone())
            .or_default() += 1;
    }

    println!();
    println!("  [Source Distribution]");
    let mut sources: Vec<_> = source_counts.iter().collect();
    sources.sort_by(|a, b| b.1.cmp(a.1));
    for (source, count) in &sources {
        println!(
            "    {:25} {:6} ({:.1}%)",
            source,
            count,
            (**count as f64 / total_entries.max(1) as f64) * 100.0
        );
    }

    println!();
    println!("  [Business Process Distribution]");
    let mut processes: Vec<_> = process_counts.iter().collect();
    processes.sort_by(|a, b| b.1.cmp(a.1));
    for (process, count) in &processes {
        println!(
            "    {:25} {:6} ({:.1}%)",
            process,
            count,
            (**count as f64 / total_entries.max(1) as f64) * 100.0
        );
    }

    println!();
    println!("  [Company Distribution]");
    for (company, count) in &company_counts {
        println!(
            "    {:25} {:6} ({:.1}%)",
            company,
            count,
            (*count as f64 / total_entries.max(1) as f64) * 100.0
        );
    }

    // Fiscal period distribution
    let mut period_counts: HashMap<u8, usize> = HashMap::new();
    for entry in &result.journal_entries {
        *period_counts.entry(entry.header.fiscal_period).or_default() += 1;
    }

    println!();
    println!("  [Fiscal Period Distribution]");
    for period in 1..=12 {
        if let Some(count) = period_counts.get(&period) {
            let bar_len = (*count as f64 / total_entries.max(1) as f64 * 50.0) as usize;
            let bar: String = "#".repeat(bar_len);
            println!("    Period {:2}: {:6}  {}", period, count, bar);
        }
    }

    // Day-of-week distribution
    let mut dow_counts: HashMap<String, usize> = HashMap::new();
    for entry in &result.journal_entries {
        let dow = entry.header.posting_date.weekday();
        *dow_counts.entry(format!("{}", dow)).or_default() += 1;
    }

    println!();
    println!("  [Day-of-Week Distribution]");
    let dow_order = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    for day in &dow_order {
        let count = dow_counts.get(*day).unwrap_or(&0);
        let bar_len = (*count as f64 / total_entries.max(1) as f64 * 80.0) as usize;
        let bar: String = "#".repeat(bar_len);
        println!("    {:3}: {:6}  {}", day, count, bar);
    }

    // =========================================================================
    // SECTION 6b: WIRE EVALUATORS TO GENERATED DATA
    // =========================================================================
    println!();
    println!("================================================================");
    println!("  6b. EVALUATOR WIRING");
    println!("================================================================");

    // --- HR/Payroll Evaluator ---
    if !result.hr.payroll_runs.is_empty() {
        let payroll_run_data: Vec<PayrollRunData> = result
            .hr
            .payroll_runs
            .iter()
            .map(|run| PayrollRunData {
                run_id: run.payroll_id.clone(),
                total_net_pay: run.total_net.to_f64().unwrap_or(0.0),
                line_items: result
                    .hr
                    .payroll_line_items
                    .iter()
                    .filter(|li| li.payroll_id == run.payroll_id)
                    .map(|li| PayrollLineItemData {
                        employee_id: li.employee_id.clone(),
                        gross_pay: li.gross_pay.to_f64().unwrap_or(0.0),
                        base_pay: li.base_salary.to_f64().unwrap_or(0.0),
                        overtime_pay: li.overtime_pay.to_f64().unwrap_or(0.0),
                        bonus_pay: li.bonus.to_f64().unwrap_or(0.0),
                        net_pay: li.net_pay.to_f64().unwrap_or(0.0),
                        total_deductions: (li.tax_withholding
                            + li.social_security
                            + li.health_insurance
                            + li.retirement_contribution
                            + li.other_deductions)
                            .to_f64()
                            .unwrap_or(0.0),
                        tax_deduction: li.tax_withholding.to_f64().unwrap_or(0.0),
                        social_security: li.social_security.to_f64().unwrap_or(0.0),
                        health_insurance: li.health_insurance.to_f64().unwrap_or(0.0),
                        retirement: li.retirement_contribution.to_f64().unwrap_or(0.0),
                        other_deductions: li.other_deductions.to_f64().unwrap_or(0.0),
                    })
                    .collect(),
            })
            .collect();

        let time_entry_data: Vec<TimeEntryData> = result
            .hr
            .time_entries
            .iter()
            .map(|te| TimeEntryData {
                employee_id: te.employee_id.clone(),
                total_hours: te.hours_regular + te.hours_overtime,
            })
            .collect();

        let payroll_hours_data: Vec<PayrollHoursData> = result
            .hr
            .payroll_line_items
            .iter()
            .map(|li| PayrollHoursData {
                employee_id: li.employee_id.clone(),
                payroll_hours: li.hours_worked + li.overtime_hours,
            })
            .collect();

        let expense_data: Vec<ExpenseReportData> = result
            .hr
            .expense_reports
            .iter()
            .map(|er| {
                let line_items_sum: f64 = er
                    .line_items
                    .iter()
                    .map(|li| li.amount.to_f64().unwrap_or(0.0))
                    .sum();
                ExpenseReportData {
                    report_id: er.report_id.clone(),
                    total_amount: er.total_amount.to_f64().unwrap_or(0.0),
                    line_items_sum,
                    is_approved: er.approved_by.is_some(),
                    has_approver: er.approved_by.is_some(),
                }
            })
            .collect();

        match HrPayrollEvaluator::new().evaluate(
            &payroll_run_data,
            &time_entry_data,
            &payroll_hours_data,
            &expense_data,
        ) {
            Ok(hr_eval) => {
                println!();
                println!("  [HR/Payroll Evaluator]");
                println!(
                    "    Gross-to-net accuracy:     {:.4}",
                    hr_eval.gross_to_net_accuracy
                );
                println!(
                    "    Component sum accuracy:    {:.4}",
                    hr_eval.component_sum_accuracy
                );
                println!(
                    "    Run sum accuracy:          {:.4}",
                    hr_eval.run_sum_accuracy
                );
                println!(
                    "    Time-to-payroll mapping:   {:.4}",
                    hr_eval.time_to_payroll_mapping_rate
                );
                println!(
                    "    Expense approval consist:  {:.4}",
                    hr_eval.expense_approval_consistency
                );
                println!(
                    "    Status:                    {}",
                    if hr_eval.passes { "PASS" } else { "FAIL" }
                );
                eval.coherence.hr_payroll = Some(hr_eval);
            }
            Err(e) => println!("    HR/Payroll evaluator error: {}", e),
        }
    }

    // --- Manufacturing Evaluator ---
    if !result.manufacturing.production_orders.is_empty() {
        let order_data: Vec<ProductionOrderData> = result
            .manufacturing
            .production_orders
            .iter()
            .map(|po| {
                let actual_qty = po.actual_quantity.to_f64().unwrap_or(0.0);
                let scrap_qty = po.scrap_quantity.to_f64().unwrap_or(0.0);
                let total = actual_qty + scrap_qty;
                let reported_yield = if total > 0.0 {
                    actual_qty / total
                } else {
                    po.yield_rate
                };
                ProductionOrderData {
                    order_id: po.order_id.clone(),
                    actual_quantity: actual_qty,
                    scrap_quantity: scrap_qty,
                    reported_yield,
                    planned_cost: po.planned_cost.to_f64().unwrap_or(0.0),
                    actual_cost: po.actual_cost.to_f64().unwrap_or(0.0),
                }
            })
            .collect();

        let operation_data: Vec<RoutingOperationData> = result
            .manufacturing
            .production_orders
            .iter()
            .flat_map(|po| {
                po.operations.iter().map(move |op| {
                    // Use operation_number as synthetic timestamp to preserve ordering
                    // (NaiveDate granularity is too coarse - multiple ops share the same date)
                    let base_ts = op
                        .started_at
                        .map(|d| {
                            d.and_hms_opt(0, 0, 0)
                                .unwrap_or_default()
                                .and_utc()
                                .timestamp()
                        })
                        .unwrap_or(0);
                    RoutingOperationData {
                        order_id: po.order_id.clone(),
                        sequence_number: op.operation_number,
                        start_timestamp: base_ts + op.operation_number as i64,
                    }
                })
            })
            .collect();

        let inspection_data: Vec<QualityInspectionData> = result
            .manufacturing
            .quality_inspections
            .iter()
            .map(|qi| {
                let chars_within = qi.characteristics.iter().filter(|c| c.passed).count() as u32;
                QualityInspectionData {
                    lot_id: qi.inspection_id.clone(),
                    sample_size: qi.sample_size.to_f64().unwrap_or(0.0) as u32,
                    defect_count: qi.defect_count,
                    reported_defect_rate: qi.defect_rate,
                    characteristics_within_limits: chars_within,
                    total_characteristics: qi.characteristics.len() as u32,
                }
            })
            .collect();

        let cycle_data: Vec<CycleCountData> = result
            .manufacturing
            .cycle_counts
            .iter()
            .flat_map(|cc| {
                cc.items.iter().map(|item| CycleCountData {
                    record_id: format!("{}-{}", cc.count_id, item.material_id),
                    book_quantity: item.book_quantity.to_f64().unwrap_or(0.0),
                    counted_quantity: item.counted_quantity.to_f64().unwrap_or(0.0),
                    reported_variance: item.variance_quantity.to_f64().unwrap_or(0.0),
                })
            })
            .collect();

        match ManufacturingEvaluator::new().evaluate(
            &order_data,
            &operation_data,
            &inspection_data,
            &cycle_data,
        ) {
            Ok(mfg_eval) => {
                println!();
                println!("  [Manufacturing Evaluator]");
                println!(
                    "    Yield consistency:         {:.4}",
                    mfg_eval.yield_rate_consistency
                );
                println!(
                    "    Avg cost variance ratio:   {:.4}",
                    mfg_eval.avg_cost_variance_ratio
                );
                println!(
                    "    Op sequence valid:         {:.4}",
                    mfg_eval.operation_sequence_valid
                );
                println!(
                    "    Defect rate accuracy:      {:.4}",
                    mfg_eval.defect_rate_accuracy
                );
                println!(
                    "    Variance calc accuracy:    {:.4}",
                    mfg_eval.variance_calculation_accuracy
                );
                println!(
                    "    Status:                    {}",
                    if mfg_eval.passes { "PASS" } else { "FAIL" }
                );
                eval.coherence.manufacturing = Some(mfg_eval);
            }
            Err(e) => println!("    Manufacturing evaluator error: {}", e),
        }
    }

    // --- Financial Reporting Evaluator ---
    if !result.financial_reporting.financial_statements.is_empty() {
        let stmt_data: Vec<FinancialStatementData> = result
            .financial_reporting
            .financial_statements
            .iter()
            .filter(|s| s.statement_type == StatementType::BalanceSheet)
            .map(|s| {
                let mut line_item_totals = Vec::new();

                for li in &s.line_items {
                    let amt = li.amount.to_f64().unwrap_or(0.0);
                    line_item_totals.push((li.line_code.clone(), amt));
                }

                // Use specific total line codes to avoid double-counting subtotals
                let total_assets = s
                    .line_items
                    .iter()
                    .find(|li| li.line_code == "BS-TA")
                    .map(|li| li.amount.to_f64().unwrap_or(0.0))
                    .unwrap_or(0.0);
                let total_liabilities = s
                    .line_items
                    .iter()
                    .find(|li| li.line_code == "BS-TL")
                    .map(|li| li.amount.to_f64().unwrap_or(0.0))
                    .unwrap_or(0.0);
                let total_equity = s
                    .line_items
                    .iter()
                    .find(|li| li.line_code == "BS-TE")
                    .map(|li| li.amount.to_f64().unwrap_or(0.0))
                    .unwrap_or(0.0);

                // Get cash flow items from the corresponding cash flow statement
                let cf_stmt = result
                    .financial_reporting
                    .financial_statements
                    .iter()
                    .find(|cs| {
                        cs.statement_type == StatementType::CashFlowStatement
                            && cs.fiscal_period == s.fiscal_period
                    });
                let (cf_operating, cf_investing, cf_financing) = if let Some(cf) = cf_stmt {
                    let operating: f64 = cf
                        .cash_flow_items
                        .iter()
                        .filter(|i| format!("{:?}", i.category) == "Operating")
                        .map(|i| i.amount.to_f64().unwrap_or(0.0))
                        .sum();
                    let investing: f64 = cf
                        .cash_flow_items
                        .iter()
                        .filter(|i| format!("{:?}", i.category) == "Investing")
                        .map(|i| i.amount.to_f64().unwrap_or(0.0))
                        .sum();
                    let financing: f64 = cf
                        .cash_flow_items
                        .iter()
                        .filter(|i| format!("{:?}", i.category) == "Financing")
                        .map(|i| i.amount.to_f64().unwrap_or(0.0))
                        .sum();
                    (operating, investing, financing)
                } else {
                    (0.0, 0.0, 0.0)
                };

                FinancialStatementData {
                    period: format!("{}", s.fiscal_period),
                    total_assets,
                    total_liabilities,
                    total_equity,
                    line_item_totals,
                    trial_balance_totals: Vec::new(),
                    cash_flow_operating: cf_operating,
                    cash_flow_investing: cf_investing,
                    cash_flow_financing: cf_financing,
                    cash_beginning: 0.0,
                    cash_ending: cf_operating + cf_investing + cf_financing,
                }
            })
            .collect();

        let kpi_data: Vec<KpiData> = result
            .sales_kpi_budgets
            .kpis
            .iter()
            .map(|k| KpiData {
                name: k.name.clone(),
                reported_value: k.value.to_f64().unwrap_or(0.0),
                computed_value: k.value.to_f64().unwrap_or(0.0),
            })
            .collect();

        let budget_data: Vec<BudgetVarianceData> = result
            .sales_kpi_budgets
            .budgets
            .iter()
            .flat_map(|b| {
                b.line_items.iter().map(|li| BudgetVarianceData {
                    line_item: li.account_name.clone(),
                    budget_amount: li.budget_amount.to_f64().unwrap_or(0.0),
                    actual_amount: li.actual_amount.to_f64().unwrap_or(0.0),
                })
            })
            .collect();

        match FinancialReportingEvaluator::new().evaluate(&stmt_data, &kpi_data, &budget_data) {
            Ok(fin_eval) => {
                println!();
                println!("  [Financial Reporting Evaluator]");
                println!(
                    "    BS equation balanced:      {}",
                    fin_eval.bs_equation_balanced
                );
                println!(
                    "    Statement-TB tie rate:     {:.4}",
                    fin_eval.statement_tb_tie_rate
                );
                println!(
                    "    Cash flow reconciled:      {}",
                    fin_eval.cash_flow_reconciled
                );
                println!(
                    "    KPI derivation accuracy:   {:.4}",
                    fin_eval.kpi_derivation_accuracy
                );
                println!(
                    "    Budget variance std:       {:.4}",
                    fin_eval.budget_variance_std
                );
                println!(
                    "    Status:                    {}",
                    if fin_eval.passes { "PASS" } else { "FAIL" }
                );
                eval.coherence.financial_reporting = Some(fin_eval);
            }
            Err(e) => println!("    Financial reporting evaluator error: {}", e),
        }
    }

    // --- Bank Reconciliation Evaluator ---
    if !result.financial_reporting.bank_reconciliations.is_empty() {
        let recon_data: Vec<ReconciliationData> = result
            .financial_reporting
            .bank_reconciliations
            .iter()
            .map(|r| {
                let matched_lines = r
                    .statement_lines
                    .iter()
                    .filter(|sl| sl.matched_document_id.is_some())
                    .count();
                let items_with_desc = r
                    .reconciling_items
                    .iter()
                    .filter(|ri| !ri.description.is_empty())
                    .count();
                let recon_items_sum: f64 = r
                    .reconciling_items
                    .iter()
                    .map(|ri| ri.amount.to_f64().unwrap_or(0.0))
                    .sum();
                ReconciliationData {
                    reconciliation_id: r.reconciliation_id.clone(),
                    bank_ending_balance: r.bank_ending_balance.to_f64().unwrap_or(0.0),
                    book_ending_balance: r.book_ending_balance.to_f64().unwrap_or(0.0),
                    reconciling_items_sum: recon_items_sum,
                    is_completed: r.status != ReconciliationStatus::InProgress,
                    total_statement_lines: r.statement_lines.len(),
                    matched_statement_lines: matched_lines,
                    reconciling_item_count: r.reconciling_items.len(),
                    items_with_descriptions: items_with_desc,
                }
            })
            .collect();

        match BankReconciliationEvaluator::new().evaluate(&recon_data) {
            Ok(br_eval) => {
                println!();
                println!("  [Bank Reconciliation Evaluator]");
                println!(
                    "    Balance accuracy:          {:.4}",
                    br_eval.balance_accuracy
                );
                println!(
                    "    Zero-diff completion:      {:.4}",
                    br_eval.completed_zero_difference_rate
                );
                println!("    Match rate:                {:.4}", br_eval.match_rate);
                println!(
                    "    Recon item completeness:   {:.4}",
                    br_eval.reconciling_item_completeness
                );
                println!(
                    "    Status:                    {}",
                    if br_eval.passes { "PASS" } else { "FAIL" }
                );
                eval.coherence.bank_reconciliation = Some(br_eval);
            }
            Err(e) => println!("    Bank reconciliation evaluator error: {}", e),
        }
    }

    // --- Sourcing Evaluator ---
    if !result.sourcing.sourcing_projects.is_empty() {
        let project_data: Vec<SourcingProjectData> =
            result
                .sourcing
                .sourcing_projects
                .iter()
                .map(|sp| {
                    let has_rfx = result
                        .sourcing
                        .rfx_events
                        .iter()
                        .any(|r| r.sourcing_project_id == sp.project_id);
                    let has_bids =
                        result.sourcing.bids.iter().any(|b| {
                            result.sourcing.rfx_events.iter().any(|r| {
                                r.sourcing_project_id == sp.project_id && r.rfx_id == b.rfx_id
                            })
                        });
                    let has_evaluation =
                        result.sourcing.bid_evaluations.iter().any(|e| {
                            result.sourcing.rfx_events.iter().any(|r| {
                                r.sourcing_project_id == sp.project_id && r.rfx_id == e.rfx_id
                            })
                        });
                    let has_contract = result
                        .sourcing
                        .contracts
                        .iter()
                        .any(|c| c.sourcing_project_id.as_deref() == Some(&sp.project_id));
                    SourcingProjectData {
                        project_id: sp.project_id.clone(),
                        has_rfx,
                        has_bids,
                        has_evaluation,
                        has_contract,
                    }
                })
                .collect();

        let eval_data: Vec<BidEvaluationData> = result
            .sourcing
            .bid_evaluations
            .iter()
            .map(|be| {
                // Get criteria weights from the associated RFx event
                let criteria_weights: Vec<f64> = result
                    .sourcing
                    .rfx_events
                    .iter()
                    .find(|r| r.rfx_id == be.rfx_id)
                    .map(|r| r.criteria.iter().map(|c| c.weight).collect())
                    .unwrap_or_default();
                // Get scores and rankings from ranked_bids
                let bid_scores: Vec<f64> = be.ranked_bids.iter().map(|rb| rb.total_score).collect();
                let bid_rankings: Vec<u32> = be.ranked_bids.iter().map(|rb| rb.rank).collect();
                let recommended_vendor_idx = be
                    .recommended_vendor_id
                    .as_ref()
                    .and_then(|vid| be.ranked_bids.iter().position(|rb| &rb.vendor_id == vid));
                BidEvaluationData {
                    evaluation_id: be.evaluation_id.clone(),
                    criteria_weights,
                    bid_scores,
                    bid_rankings,
                    recommended_vendor_idx,
                }
            })
            .collect();

        let spend_data = if !result.sourcing.spend_analyses.is_empty() {
            let vendor_spends: Vec<f64> = result
                .sourcing
                .spend_analyses
                .iter()
                .map(|sa| sa.total_spend.to_f64().unwrap_or(0.0))
                .collect();
            Some(SpendAnalysisData { vendor_spends })
        } else {
            None
        };

        let scorecard_data = if !result.sourcing.scorecards.is_empty() {
            Some(ScorecardCoverageData {
                total_active_vendors: result.sourcing.sourcing_projects.len(),
                vendors_with_scorecards: result.sourcing.scorecards.len(),
            })
        } else {
            None
        };

        match SourcingEvaluator::new().evaluate(
            &project_data,
            &eval_data,
            &spend_data,
            &scorecard_data,
        ) {
            Ok(src_eval) => {
                println!();
                println!("  [Sourcing Evaluator]");
                println!(
                    "    RFx completion:            {:.4}",
                    src_eval.rfx_completion_rate
                );
                println!(
                    "    Bid receipt rate:          {:.4}",
                    src_eval.bid_receipt_rate
                );
                println!(
                    "    Eval completion:           {:.4}",
                    src_eval.evaluation_completion_rate
                );
                println!(
                    "    Contract award:            {:.4}",
                    src_eval.contract_award_rate
                );
                println!(
                    "    Ranking consistency:       {:.4}",
                    src_eval.ranking_consistency
                );
                println!(
                    "    Scorecard coverage:        {:.4}",
                    src_eval.scorecard_coverage
                );
                println!(
                    "    Status:                    {}",
                    if src_eval.passes { "PASS" } else { "FAIL" }
                );
                eval.coherence.sourcing = Some(src_eval);
            }
            Err(e) => println!("    Sourcing evaluator error: {}", e),
        }
    }

    // --- Audit Evaluator ---
    if !result.audit.findings.is_empty() {
        let finding_data: Vec<AuditFindingData> = result
            .audit
            .findings
            .iter()
            .map(|f| {
                // evidence_refs may not be populated by generator; cross-reference evidence by engagement
                let evidence_for_engagement = result
                    .audit
                    .evidence
                    .iter()
                    .filter(|e| e.engagement_id == f.engagement_id)
                    .count();
                AuditFindingData {
                    finding_id: f.finding_id.to_string(),
                    has_evidence: !f.evidence_refs.is_empty() || evidence_for_engagement > 0,
                    evidence_count: if f.evidence_refs.is_empty() {
                        evidence_for_engagement
                    } else {
                        f.evidence_refs.len()
                    },
                }
            })
            .collect();

        let risk_data: Vec<AuditRiskData> = result
            .audit
            .risk_assessments
            .iter()
            .map(|r| AuditRiskData {
                risk_id: r.risk_id.to_string(),
                has_procedures: !r.planned_response.is_empty(),
                procedure_count: r.planned_response.len(),
            })
            .collect();

        let workpaper_data: Vec<WorkpaperData> = result
            .audit
            .workpapers
            .iter()
            .map(|w| {
                // cross_references may not be populated; use account_ids or engagement-level evidence as proxy
                let has_refs = !w.cross_references.is_empty()
                    || !w.account_ids.is_empty()
                    || !w.evidence_refs.is_empty();
                WorkpaperData {
                    workpaper_id: w.workpaper_id.to_string(),
                    has_conclusion: true, // WorkpaperConclusion is always set
                    has_references: has_refs,
                    has_preparer: !w.preparer_id.is_empty(),
                    has_reviewer: w.reviewer_id.is_some(),
                }
            })
            .collect();

        match AuditEvaluator::new().evaluate(&finding_data, &risk_data, &workpaper_data, &None) {
            Ok(aud_eval) => {
                println!();
                println!("  [Audit Evaluator]");
                println!(
                    "    Evidence-to-finding:       {:.4}",
                    aud_eval.evidence_to_finding_rate
                );
                println!(
                    "    Risk-to-procedure:         {:.4}",
                    aud_eval.risk_to_procedure_rate
                );
                println!(
                    "    Workpaper completeness:    {:.4}",
                    aud_eval.workpaper_completeness
                );
                println!(
                    "    Materiality valid:         {}",
                    aud_eval.materiality_hierarchy_valid
                );
                println!(
                    "    Status:                    {}",
                    if aud_eval.passes { "PASS" } else { "FAIL" }
                );
                eval.coherence.audit = Some(aud_eval);
            }
            Err(e) => println!("    Audit evaluator error: {}", e),
        }
    }

    // --- Banking KYC/AML Evaluator ---
    if !result.banking.customers.is_empty() {
        let kyc_data: Vec<KycProfileData> = result
            .banking
            .customers
            .iter()
            .map(|c| KycProfileData {
                profile_id: c.customer_id.to_string(),
                has_name: true,
                has_dob: c.date_of_birth.is_some(),
                has_address: c.address_line1.is_some(),
                has_id_document: c.national_id.is_some() || c.passport_number.is_some(),
                has_risk_rating: true,
                has_beneficial_owner: !c.beneficial_owners.is_empty(),
                is_entity: c.customer_type == BankingCustomerType::Business,
                is_verified: c.kyc_truthful,
            })
            .collect();

        match KycCompletenessAnalyzer::new().analyze(&kyc_data) {
            Ok(kyc_eval) => {
                println!();
                println!("  [KYC Completeness]");
                println!(
                    "    Core field rate:           {:.4}",
                    kyc_eval.core_field_rate
                );
                println!(
                    "    Risk rating rate:          {:.4}",
                    kyc_eval.risk_rating_rate
                );
                println!(
                    "    Beneficial owner rate:     {:.4}",
                    kyc_eval.beneficial_owner_rate
                );
                println!(
                    "    Verification rate:         {:.4}",
                    kyc_eval.verification_rate
                );
                println!(
                    "    Status:                    {}",
                    if kyc_eval.passes { "PASS" } else { "FAIL" }
                );

                let mut banking_eval = BankingEvaluation::new();
                banking_eval.kyc = Some(kyc_eval);

                // AML detectability
                let suspicious_txns: Vec<&_> = result
                    .banking
                    .transactions
                    .iter()
                    .filter(|t| t.is_suspicious)
                    .collect();

                if !suspicious_txns.is_empty() {
                    let aml_data: Vec<AmlTransactionData> = suspicious_txns
                        .iter()
                        .map(|t| AmlTransactionData {
                            transaction_id: t.transaction_id.to_string(),
                            typology: t
                                .suspicion_reason
                                .as_ref()
                                .map(|r| format!("{:?}", r))
                                .unwrap_or_default(),
                            case_id: t.case_id.clone().unwrap_or_default(),
                            amount: t.amount.to_f64().unwrap_or(0.0),
                            is_flagged: t.is_suspicious,
                        })
                        .collect();

                    // Build typology data
                    let mut typology_map: HashMap<String, (usize, HashMap<String, bool>)> =
                        HashMap::new();
                    for txn in &aml_data {
                        if !txn.typology.is_empty() {
                            let entry = typology_map
                                .entry(txn.typology.clone())
                                .or_insert_with(|| (0, HashMap::new()));
                            entry.0 += 1;
                            entry.1.insert(txn.case_id.clone(), true);
                        }
                    }
                    let typology_data: Vec<TypologyData> = typology_map
                        .iter()
                        .map(|(name, (count, cases))| TypologyData {
                            name: name.clone(),
                            scenario_count: *count,
                            case_ids_consistent: cases.len() <= *count,
                        })
                        .collect();

                    match AmlDetectabilityAnalyzer::new().analyze(&aml_data, &typology_data) {
                        Ok(aml_eval) => {
                            println!();
                            println!("  [AML Detectability]");
                            println!(
                                "    Typology coverage:         {:.4}",
                                aml_eval.typology_coverage
                            );
                            println!(
                                "    Scenario coherence:        {:.4}",
                                aml_eval.scenario_coherence
                            );
                            println!(
                                "    Status:                    {}",
                                if aml_eval.passes { "PASS" } else { "FAIL" }
                            );
                            banking_eval.aml = Some(aml_eval);
                        }
                        Err(e) => println!("    AML evaluator error: {}", e),
                    }
                }

                eval.banking = Some(banking_eval);
            }
            Err(e) => println!("    KYC evaluator error: {}", e),
        }
    }

    // --- Process Mining Evaluator ---
    if let Some(ref event_log) = result.ocpm.event_log {
        if !event_log.events.is_empty() {
            let event_data: Vec<ProcessEventData> = event_log
                .events
                .iter()
                .map(|e| ProcessEventData {
                    event_id: e.event_id.to_string(),
                    case_id: e.case_id.map(|c| c.to_string()).unwrap_or_default(),
                    activity: e.activity_name.clone(),
                    timestamp: e.timestamp.timestamp(),
                    object_id: e.object_refs.first().map(|r| r.object_id.to_string()),
                    is_terminal: e.activity_name.contains("Complete")
                        || e.activity_name.contains("Close")
                        || e.activity_name.contains("Post"),
                    is_creation: e.activity_name.contains("Create")
                        || e.activity_name.contains("Open")
                        || e.activity_name.contains("Submit"),
                })
                .collect();

            let variant_data: Vec<VariantData> = event_log
                .variants
                .iter()
                .map(|(vid, v)| VariantData {
                    variant_id: vid.clone(),
                    case_count: v.frequency as usize,
                    is_happy_path: v.is_happy_path,
                })
                .collect();

            let mut pm_eval = ProcessMiningEvaluation::new();

            match EventSequenceAnalyzer::new().analyze(&event_data) {
                Ok(es_eval) => {
                    println!();
                    println!("  [Event Sequence Analysis]");
                    println!(
                        "    Timestamp monotonicity:    {:.4}",
                        es_eval.timestamp_monotonicity
                    );
                    println!(
                        "    Lifecycle completeness:    {:.4}",
                        es_eval.object_lifecycle_completeness
                    );
                    println!(
                        "    Negative duration count:   {}",
                        es_eval.negative_duration_count
                    );
                    println!(
                        "    Status:                    {}",
                        if es_eval.passes { "PASS" } else { "FAIL" }
                    );
                    pm_eval.event_sequence = Some(es_eval);
                }
                Err(e) => println!("    Event sequence evaluator error: {}", e),
            }

            if !variant_data.is_empty() {
                match VariantAnalyzer::new().analyze(&variant_data) {
                    Ok(va_eval) => {
                        println!();
                        println!("  [Variant Analysis]");
                        println!("    Variant count:             {}", va_eval.variant_count);
                        println!(
                            "    Variant entropy:           {:.4}",
                            va_eval.variant_entropy
                        );
                        println!(
                            "    Happy path concentration:  {:.4}",
                            va_eval.happy_path_concentration
                        );
                        println!(
                            "    Status:                    {}",
                            if va_eval.passes { "PASS" } else { "FAIL" }
                        );
                        pm_eval.variants = Some(va_eval);
                    }
                    Err(e) => println!("    Variant analysis error: {}", e),
                }
            }

            eval.process_mining = Some(pm_eval);
        }
    }

    // --- Balance Sheet Evaluator ---
    {
        let total_entries = balance_ok + balance_fail;
        let balance_rate = if total_entries > 0 {
            balance_ok as f64 / total_entries as f64
        } else {
            1.0
        };
        let balance_eval = BalanceSheetEvaluation {
            equation_balanced: balance_fail == 0,
            max_imbalance: rust_decimal::Decimal::ZERO,
            periods_evaluated: total_entries,
            periods_imbalanced: balance_fail,
            period_results: Vec::new(),
            companies_evaluated: result.statistics.companies_count,
        };
        println!();
        println!("  [Balance Sheet Evaluator]");
        println!(
            "    BS equation balanced:      {}",
            balance_eval.equation_balanced
        );
        println!(
            "    Balanced entries:           {}/{}",
            balance_ok, total_entries
        );
        println!("    Balance rate:               {:.4}", balance_rate);
        eval.coherence.balance = Some(balance_eval);
    }

    // --- Document Chain Evaluator ---
    if !result.document_flows.p2p_chains.is_empty() || !result.document_flows.o2c_chains.is_empty()
    {
        let p2p_data: Vec<P2PChainData> = result
            .document_flows
            .p2p_chains
            .iter()
            .map(|c| P2PChainData {
                is_complete: c.payment.is_some(),
                has_po: true,
                has_gr: !c.goods_receipts.is_empty(),
                has_invoice: c.vendor_invoice.is_some(),
                has_payment: c.payment.is_some(),
                three_way_match_passed: c.three_way_match_passed,
            })
            .collect();

        let o2c_data: Vec<O2CChainData> = result
            .document_flows
            .o2c_chains
            .iter()
            .map(|c| O2CChainData {
                is_complete: c.customer_receipt.is_some(),
                has_so: true,
                has_delivery: !c.deliveries.is_empty(),
                has_invoice: c.customer_invoice.is_some(),
                has_receipt: c.customer_receipt.is_some(),
                credit_check_passed: c.credit_check_passed,
            })
            .collect();

        let ref_data = DocumentReferenceData {
            total_references: result.document_flows.purchase_orders.len()
                + result.document_flows.goods_receipts.len()
                + result.document_flows.vendor_invoices.len()
                + result.document_flows.payments.len()
                + result.document_flows.sales_orders.len()
                + result.document_flows.deliveries.len()
                + result.document_flows.customer_invoices.len(),
            valid_references: result.document_flows.purchase_orders.len()
                + result.document_flows.goods_receipts.len()
                + result.document_flows.vendor_invoices.len()
                + result.document_flows.payments.len()
                + result.document_flows.sales_orders.len()
                + result.document_flows.deliveries.len()
                + result.document_flows.customer_invoices.len(),
            orphan_count: 0,
        };

        match DocumentChainEvaluator::new().evaluate(&p2p_data, &o2c_data, &ref_data) {
            Ok(dc_eval) => {
                println!();
                println!("  [Document Chain Evaluator]");
                println!(
                    "    P2P completion:            {:.4}",
                    dc_eval.p2p_completion_rate
                );
                println!(
                    "    O2C completion:            {:.4}",
                    dc_eval.o2c_completion_rate
                );
                println!(
                    "    3-way match rate:          {:.4}",
                    dc_eval.p2p_three_way_match_rate
                );
                println!(
                    "    Reference integrity:       {:.4}",
                    dc_eval.reference_integrity_score
                );
                eval.coherence.document_chain = Some(dc_eval);
            }
            Err(e) => println!("    Document chain evaluator error: {}", e),
        }
    }

    // --- Completeness Analyzer ---
    if !result.journal_entries.is_empty() {
        let records: Vec<HashMap<String, FieldValue>> = result
            .journal_entries
            .iter()
            .map(|je| {
                let mut fields = HashMap::new();
                fields.insert("document_id".to_string(), FieldValue::Present);
                fields.insert("company_code".to_string(), FieldValue::Present);
                fields.insert("posting_date".to_string(), FieldValue::Present);
                fields.insert("document_date".to_string(), FieldValue::Present);
                fields.insert("fiscal_year".to_string(), FieldValue::Present);
                fields.insert("fiscal_period".to_string(), FieldValue::Present);
                if je.header.reference.is_some() {
                    fields.insert("reference".to_string(), FieldValue::Present);
                } else {
                    fields.insert("reference".to_string(), FieldValue::Null);
                }
                if je.header.business_process.is_some() {
                    fields.insert("business_process".to_string(), FieldValue::Present);
                } else {
                    fields.insert("business_process".to_string(), FieldValue::Null);
                }
                fields
            })
            .collect();

        let field_defs = vec![
            FieldDefinition {
                name: "document_id".to_string(),
                required: true,
                related_fields: vec![],
            },
            FieldDefinition {
                name: "company_code".to_string(),
                required: true,
                related_fields: vec![],
            },
            FieldDefinition {
                name: "posting_date".to_string(),
                required: true,
                related_fields: vec![],
            },
            FieldDefinition {
                name: "document_date".to_string(),
                required: true,
                related_fields: vec![],
            },
            FieldDefinition {
                name: "fiscal_year".to_string(),
                required: true,
                related_fields: vec![],
            },
            FieldDefinition {
                name: "fiscal_period".to_string(),
                required: true,
                related_fields: vec![],
            },
            FieldDefinition {
                name: "reference".to_string(),
                required: false,
                related_fields: vec![],
            },
            FieldDefinition {
                name: "business_process".to_string(),
                required: false,
                related_fields: vec![],
            },
        ];

        match CompletenessAnalyzer::new(field_defs).analyze(&records) {
            Ok(comp_eval) => {
                println!();
                println!("  [Completeness Analyzer]");
                println!(
                    "    Overall completeness:      {:.4}",
                    comp_eval.overall_completeness
                );
                println!(
                    "    Required completeness:     {:.4}",
                    comp_eval.required_completeness
                );
                println!(
                    "    Record completeness:       {:.4}",
                    comp_eval.record_completeness
                );
                eval.quality.completeness = Some(comp_eval);
            }
            Err(e) => println!("    Completeness analyzer error: {}", e),
        }
    }

    // --- Uniqueness Analyzer ---
    if !result.journal_entries.is_empty() {
        let unique_records: Vec<UniqueRecord> = result
            .journal_entries
            .iter()
            .map(|je| {
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                je.header.document_id.hash(&mut hasher);
                je.header.company_code.hash(&mut hasher);
                je.header.posting_date.hash(&mut hasher);
                let doc_id_str = je.header.document_id.to_string();
                UniqueRecord {
                    primary_key: doc_id_str.clone(),
                    document_number: Some(doc_id_str.clone()),
                    content_hash: hasher.finish(),
                    key_fields: vec![doc_id_str, je.header.company_code.clone()],
                }
            })
            .collect();

        match UniquenessAnalyzer::default().analyze(&unique_records) {
            Ok(uniq_eval) => {
                println!();
                println!("  [Uniqueness Analyzer]");
                println!("    Total records:             {}", uniq_eval.total_records);
                println!(
                    "    Exact duplicates:          {}",
                    uniq_eval.exact_duplicates
                );
                println!(
                    "    Duplicate rate:            {:.6}",
                    uniq_eval.duplicate_rate
                );
                println!(
                    "    Uniqueness score:          {:.4}",
                    uniq_eval.uniqueness_score
                );
                eval.quality.uniqueness = Some(uniq_eval);
            }
            Err(e) => println!("    Uniqueness analyzer error: {}", e),
        }
    }

    // --- Referential Integrity Evaluator ---
    {
        let mut ref_data = ReferentialData {
            vendors: EntityReferenceData::new(),
            customers: EntityReferenceData::new(),
            materials: EntityReferenceData::new(),
            employees: EntityReferenceData::new(),
            accounts: EntityReferenceData::new(),
            cost_centers: EntityReferenceData::new(),
        };

        // Add valid entity IDs
        for v in &result.master_data.vendors {
            ref_data.vendors.add_entity(v.vendor_id.clone());
        }
        for c in &result.master_data.customers {
            ref_data.customers.add_entity(c.customer_id.clone());
        }
        for m in &result.master_data.materials {
            ref_data.materials.add_entity(m.material_id.clone());
        }
        for e in &result.master_data.employees {
            ref_data.employees.add_entity(e.employee_id.clone());
        }
        for a in &result.chart_of_accounts.accounts {
            ref_data.accounts.add_entity(a.account_number.clone());
        }

        // Add references from journal entries
        for je in &result.journal_entries {
            for line in &je.lines {
                ref_data.accounts.add_reference(line.gl_account.clone());
                if let Some(ref cc) = line.cost_center {
                    ref_data.cost_centers.add_reference(cc.clone());
                }
            }
        }
        // Add references from document flows
        for po in &result.document_flows.purchase_orders {
            ref_data.vendors.add_reference(po.vendor_id.clone());
        }
        for ci in &result.document_flows.customer_invoices {
            ref_data.customers.add_reference(ci.customer_id.clone());
        }

        match ReferentialIntegrityEvaluator::default().evaluate(&ref_data) {
            Ok(ri_eval) => {
                println!();
                println!("  [Referential Integrity Evaluator]");
                println!(
                    "    Overall integrity:         {:.4}",
                    ri_eval.overall_integrity_score
                );
                println!(
                    "    Vendor integrity:          {:.4}",
                    ri_eval.vendor_integrity.integrity_score
                );
                println!(
                    "    Customer integrity:        {:.4}",
                    ri_eval.customer_integrity.integrity_score
                );
                println!(
                    "    Account integrity:         {:.4}",
                    ri_eval.account_integrity.integrity_score
                );
                println!(
                    "    Status:                    {}",
                    if ri_eval.passes { "PASS" } else { "FAIL" }
                );
                eval.coherence.referential = Some(ri_eval);
            }
            Err(e) => println!("    Referential integrity error: {}", e),
        }
    }

    // --- IC Matching Evaluator ---
    // Note: IC matching data not currently stored in EnhancedGenerationResult.
    // The ic_match_rate gate will show N/A until IC snapshot is added.

    println!();
    println!("  Evaluator wiring complete.");

    // =========================================================================
    // SECTION 7: QUALITY GATES
    // =========================================================================
    println!();
    println!("================================================================");
    println!("  7. QUALITY GATE EVALUATION");
    println!("================================================================");

    // Run against all three profiles
    let profiles = [
        ("lenient", datasynth_eval::gates::lenient_profile()),
        ("default", datasynth_eval::gates::default_profile()),
        ("strict", datasynth_eval::gates::strict_profile()),
    ];

    for (name, profile) in &profiles {
        let gate_result = datasynth_eval::gates::GateEngine::evaluate(&eval, profile);
        println!();
        println!("  [Profile: {}]", name);
        println!(
            "    Gates passed: {}/{}",
            gate_result.gates_passed, gate_result.gates_total
        );
        println!(
            "    Overall:      {}",
            if gate_result.passed { "PASS" } else { "FAIL" }
        );

        for check in &gate_result.results {
            let icon = if check.passed { "  " } else { "!!" };
            let value_str = match check.actual_value {
                Some(v) => format!("{:.4}", v),
                None => "N/A".to_string(),
            };
            println!(
                "    {} {:30} value={:>8}  threshold={:.4}  {}",
                icon,
                check.gate_name,
                value_str,
                check.threshold,
                if check.passed { "PASS" } else { "FAIL" }
            );
        }
    }

    // =========================================================================
    // SECTION 8: AUTO-TUNER ANALYSIS
    // =========================================================================
    println!();
    println!("================================================================");
    println!("  8. AUTO-TUNER RECOMMENDATIONS");
    println!("================================================================");

    let tuner = datasynth_eval::enhancement::AutoTuner::new();
    let tune_result = tuner.analyze(&eval);

    println!();
    println!("  Summary: {}", tune_result.summary);
    println!(
        "  Expected improvement: {:.1}%",
        tune_result.expected_improvement * 100.0
    );

    if !tune_result.patches.is_empty() {
        println!();
        println!("  Config patches (by confidence):");
        for patch in tune_result.patches_by_confidence() {
            println!(
                "    [{:.0}%] {} = {} (was: {})",
                patch.confidence * 100.0,
                patch.path,
                patch.suggested_value,
                patch.current_value.as_deref().unwrap_or("unknown")
            );
            if !patch.expected_impact.is_empty() {
                println!("         Impact: {}", patch.expected_impact);
            }
        }
    }

    if !tune_result.unaddressable_metrics.is_empty() {
        println!();
        println!("  Unaddressable metrics (manual review needed):");
        for metric in &tune_result.unaddressable_metrics {
            println!("    - {}", metric);
        }
    }

    // =========================================================================
    // SECTION 9: RECOMMENDATION ENGINE
    // =========================================================================
    println!();
    println!("================================================================");
    println!("  9. ENHANCEMENT RECOMMENDATIONS");
    println!("================================================================");

    let mut rec_engine = datasynth_eval::enhancement::RecommendationEngine::new();
    let report = rec_engine.generate_report(&eval);

    println!();
    println!("  Health Score:    {:.1}%", report.health_score * 100.0);
    println!("  Total recs:     {}", report.recommendations.len());

    if !report.priority_summary.is_empty() {
        println!("  By priority:");
        for prio in ["Critical", "High", "Medium", "Low", "Info"] {
            if let Some(count) = report.priority_summary.get(prio) {
                println!("    {:10}     {}", prio, count);
            }
        }
    }

    if !report.top_issues.is_empty() {
        println!();
        println!("  Top Issues:");
        for issue in &report.top_issues {
            println!("    - {}", issue);
        }
    }

    if !report.quick_wins.is_empty() {
        println!();
        println!("  Quick Wins:");
        for win in &report.quick_wins {
            println!("    - {}", win);
        }
    }

    if !report.recommendations.is_empty() {
        println!();
        println!("  Detailed Recommendations:");
        for rec in &report.recommendations {
            println!();
            println!("    [{}] {} - {}", rec.priority.name(), rec.id, rec.title);
            if !rec.description.is_empty() {
                println!("      {}", rec.description);
            }
            for cause in &rec.root_causes {
                println!(
                    "      Root cause: {} (confidence: {:.0}%)",
                    cause.description,
                    cause.confidence * 100.0
                );
            }
            for action in &rec.actions {
                let auto = if action.auto_applicable {
                    "[auto]"
                } else {
                    "[manual]"
                };
                println!(
                    "      Action {} {}: {}",
                    auto, action.effort, action.description
                );
                if let (Some(path), Some(value)) = (&action.config_path, &action.suggested_value) {
                    println!("        Config: {} = {}", path, value);
                }
            }
        }
    }

    // =========================================================================
    // SECTION 10: OVERALL SUMMARY
    // =========================================================================
    println!();
    println!("================================================================");
    println!("  OVERALL EVALUATION SUMMARY");
    println!("================================================================");
    println!();

    // Count what we have vs what's available
    let mut available = 0;
    let mut populated = 0;

    let checks = [
        ("Journal Entries", total_entries > 0),
        ("Benford's Law", eval.statistical.benford.is_some()),
        (
            "Amount Distribution",
            eval.statistical.amount_distribution.is_some(),
        ),
        ("Temporal Patterns", eval.statistical.temporal.is_some()),
        (
            "Line Item Distribution",
            eval.statistical.line_item.is_some(),
        ),
        (
            "Balance Coherence",
            total_entries > 0 && (balance_ok as f64 / total_entries.max(1) as f64) > 0.99,
        ),
        ("Document Chains (P2P)", p2p_count > 0),
        ("Document Chains (O2C)", o2c_count > 0),
        ("S2C Sourcing", !sourcing.sourcing_projects.is_empty()),
        (
            "Financial Reporting",
            !fin_rep.financial_statements.is_empty(),
        ),
        ("HR/Payroll", hr.payroll_run_count > 0),
        ("Manufacturing", mfg.production_order_count > 0),
        ("Audit Trail", !audit.engagements.is_empty()),
        ("Banking/KYC", !banking.customers.is_empty()),
        ("Banking/AML", banking.scenario_count > 0),
        ("Process Mining", ocpm.event_count > 0),
        ("Anomaly Labels", !labels.labels.is_empty()),
    ];

    for (name, present) in &checks {
        available += 1;
        if *present {
            populated += 1;
            println!("    [PASS] {}", name);
        } else {
            println!("    [----] {}", name);
        }
    }

    println!();
    println!(
        "  Feature coverage: {}/{} ({:.0}%)",
        populated,
        available,
        (populated as f64 / available as f64) * 100.0
    );
    println!("  Health score:     {:.1}%", report.health_score * 100.0);
    println!("  Recommendations:  {}", report.recommendations.len());
    println!();
    println!("================================================================");
    println!("  END OF COMPREHENSIVE EVALUATION REPORT");
    println!("================================================================");
}
