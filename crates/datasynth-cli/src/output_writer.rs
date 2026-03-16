//! Comprehensive output writer for all generated data.
//!
//! Writes all generated data from the EnhancedGenerationResult to files
//! in the output directory. Uses CSV for flat tabular data (journal entry
//! lines) and JSON for types with nested structures (Vecs, sub-structs).

use std::io::Write;
use std::path::Path;

use datasynth_core::documents::PaymentType;
use datasynth_runtime::enhanced_orchestrator::EnhancedGenerationResult;
use tracing::{info, warn};

/// Write a JSON file for any serializable slice. Skips empty slices.
///
/// Streams JSON directly to a buffered file writer instead of allocating
/// the entire JSON string in memory (Phase 3 I/O optimization).
fn write_json<T: serde::Serialize>(
    data: &[T],
    path: &Path,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if data.is_empty() {
        return Ok(());
    }
    let file = std::fs::File::create(path)?;
    let writer = std::io::BufWriter::with_capacity(256 * 1024, file);
    serde_json::to_writer_pretty(writer, data)?;
    info!(
        "  {} written: {} records -> {}",
        label,
        data.len(),
        path.display()
    );
    Ok(())
}

/// Write journal entry lines as a flat CSV file.
///
/// This extracts the key fields from both the header and each line item to
/// produce a single flat CSV that can be loaded directly into dataframes.
fn write_journal_entries_csv(
    result: &EnhancedGenerationResult,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if result.journal_entries.is_empty() {
        return Ok(());
    }

    let path = output_dir.join("journal_entries.csv");
    let file = std::fs::File::create(&path)?;
    let mut w = std::io::BufWriter::with_capacity(256 * 1024, file);

    // Write header
    writeln!(
        w,
        "document_id,company_code,fiscal_year,fiscal_period,posting_date,document_date,\
         document_type,currency,exchange_rate,reference,header_text,created_by,source,\
         business_process,ledger,is_fraud,is_anomaly,\
         line_number,gl_account,debit_amount,credit_amount,local_amount,\
         cost_center,profit_center,line_text,\
         auxiliary_account_number,auxiliary_account_label,lettrage,lettrage_date"
    )?;

    for je in &result.journal_entries {
        let h = &je.header;
        for line in &je.lines {
            let lettrage_date_str = line
                .lettrage_date
                .map(|d| d.to_string())
                .unwrap_or_default();
            writeln!(
                w,
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                h.document_id,
                csv_escape(&h.company_code),
                h.fiscal_year,
                h.fiscal_period,
                h.posting_date,
                h.document_date,
                csv_escape(&h.document_type),
                csv_escape(&h.currency),
                h.exchange_rate,
                csv_opt_str(&h.reference),
                csv_opt_str(&h.header_text),
                csv_escape(&h.created_by),
                h.source,
                h.business_process
                    .map(|bp| format!("{bp:?}"))
                    .unwrap_or_default(),
                csv_escape(&h.ledger),
                h.is_fraud,
                h.is_anomaly,
                line.line_number,
                csv_escape(&line.gl_account),
                line.debit_amount,
                line.credit_amount,
                line.local_amount,
                csv_opt_str(&line.cost_center),
                csv_opt_str(&line.profit_center),
                csv_opt_str(&line.line_text),
                csv_opt_str(&line.auxiliary_account_number),
                csv_opt_str(&line.auxiliary_account_label),
                csv_opt_str(&line.lettrage),
                lettrage_date_str,
            )?;
        }
    }

    w.flush()?;
    let total_lines: usize = result.journal_entries.iter().map(|je| je.lines.len()).sum();
    info!(
        "  Journal entries CSV written: {} entries, {} line items -> {}",
        result.journal_entries.len(),
        total_lines,
        path.display()
    );
    Ok(())
}

/// Escape a string for CSV output by quoting if it contains commas or quotes.
fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Format an Option<String> for CSV output (empty string for None).
fn csv_opt_str(opt: &Option<String>) -> String {
    match opt {
        Some(s) => csv_escape(s),
        None => String::new(),
    }
}

/// Write all generated data to the output directory.
///
/// This function exports every non-empty dataset from the generation result.
/// Journal entries are written as a flat CSV file (one row per line item)
/// and as a nested JSON file. Other data is written as JSON files since
/// many model types contain nested structures.
pub fn write_all_output(
    result: &EnhancedGenerationResult,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(output_dir)?;
    info!("Writing comprehensive output to: {}", output_dir.display());

    // ========================================================================
    // Journal Entries (flat CSV + nested JSON)
    // ========================================================================
    if !result.journal_entries.is_empty() {
        // Write flat CSV with one row per line item (header fields repeated)
        if let Err(e) = write_journal_entries_csv(result, output_dir) {
            warn!("Failed to write journal_entries.csv: {}", e);
        }

        // Also write full journal entries as JSON for consumers that need the nested structure
        write_json(
            &result.journal_entries,
            &output_dir.join("journal_entries.json"),
            "Journal entries (JSON)",
        )?;
    }

    // ========================================================================
    // Master Data
    // ========================================================================
    let md_dir = output_dir.join("master_data");
    if !result.master_data.vendors.is_empty()
        || !result.master_data.customers.is_empty()
        || !result.master_data.materials.is_empty()
        || !result.master_data.assets.is_empty()
        || !result.master_data.employees.is_empty()
    {
        std::fs::create_dir_all(&md_dir)?;
        info!("Writing master data...");

        write_json_safe(
            &result.master_data.vendors,
            &md_dir.join("vendors.json"),
            "Vendors",
        );
        write_json_safe(
            &result.master_data.customers,
            &md_dir.join("customers.json"),
            "Customers",
        );
        write_json_safe(
            &result.master_data.materials,
            &md_dir.join("materials.json"),
            "Materials",
        );
        write_json_safe(
            &result.master_data.assets,
            &md_dir.join("fixed_assets.json"),
            "Fixed assets",
        );
        write_json_safe(
            &result.master_data.employees,
            &md_dir.join("employees.json"),
            "Employees",
        );
    }

    // ========================================================================
    // Document Flows
    // ========================================================================
    let df_dir = output_dir.join("document_flows");
    if !result.document_flows.purchase_orders.is_empty()
        || !result.document_flows.sales_orders.is_empty()
    {
        std::fs::create_dir_all(&df_dir)?;
        info!("Writing document flows...");

        write_json_safe(
            &result.document_flows.purchase_orders,
            &df_dir.join("purchase_orders.json"),
            "Purchase orders",
        );
        write_json_safe(
            &result.document_flows.goods_receipts,
            &df_dir.join("goods_receipts.json"),
            "Goods receipts",
        );
        write_json_safe(
            &result.document_flows.vendor_invoices,
            &df_dir.join("vendor_invoices.json"),
            "Vendor invoices",
        );
        write_json_safe(
            &result.document_flows.payments,
            &df_dir.join("payments.json"),
            "Payments",
        );
        let customer_receipts: Vec<_> = result
            .document_flows
            .payments
            .iter()
            .filter(|p| p.payment_type == PaymentType::ArReceipt)
            .collect();
        write_json_safe(
            &customer_receipts,
            &df_dir.join("customer_receipts.json"),
            "Customer receipts",
        );
        write_json_safe(
            &result.document_flows.sales_orders,
            &df_dir.join("sales_orders.json"),
            "Sales orders",
        );
        write_json_safe(
            &result.document_flows.deliveries,
            &df_dir.join("deliveries.json"),
            "Deliveries",
        );
        write_json_safe(
            &result.document_flows.customer_invoices,
            &df_dir.join("customer_invoices.json"),
            "Customer invoices",
        );

        // Note: P2P/O2C chain types do not implement Serialize, so we log
        // their counts instead. The individual documents above capture all data.
        if !result.document_flows.p2p_chains.is_empty() {
            info!(
                "  P2P chains: {} (data exported via individual document files)",
                result.document_flows.p2p_chains.len()
            );
        }
        if !result.document_flows.o2c_chains.is_empty() {
            info!(
                "  O2C chains: {} (data exported via individual document files)",
                result.document_flows.o2c_chains.len()
            );
        }
    }

    // ========================================================================
    // Subledger
    // ========================================================================
    let sl_dir = output_dir.join("subledger");
    if !result.subledger.ap_invoices.is_empty()
        || !result.subledger.ar_invoices.is_empty()
        || !result.subledger.fa_records.is_empty()
        || !result.subledger.inventory_positions.is_empty()
    {
        std::fs::create_dir_all(&sl_dir)?;
        info!("Writing subledger data...");

        write_json_safe(
            &result.subledger.ap_invoices,
            &sl_dir.join("ap_invoices.json"),
            "AP invoices",
        );
        write_json_safe(
            &result.subledger.ar_invoices,
            &sl_dir.join("ar_invoices.json"),
            "AR invoices",
        );
        write_json_safe(
            &result.subledger.fa_records,
            &sl_dir.join("fa_records.json"),
            "FA records",
        );
        write_json_safe(
            &result.subledger.inventory_positions,
            &sl_dir.join("inventory_positions.json"),
            "Inventory positions",
        );
        write_json_safe(
            &result.subledger.inventory_movements,
            &sl_dir.join("inventory_movements.json"),
            "Inventory movements",
        );
        write_json_safe(
            &result.subledger.ar_aging_reports,
            &sl_dir.join("ar_aging.json"),
            "AR aging reports",
        );
        write_json_safe(
            &result.subledger.ap_aging_reports,
            &sl_dir.join("ap_aging.json"),
            "AP aging reports",
        );
    }

    // ========================================================================
    // Audit
    // ========================================================================
    let audit_dir = output_dir.join("audit");
    if !result.audit.engagements.is_empty() {
        std::fs::create_dir_all(&audit_dir)?;
        info!("Writing audit data...");

        write_json_safe(
            &result.audit.engagements,
            &audit_dir.join("audit_engagements.json"),
            "Audit engagements",
        );
        write_json_safe(
            &result.audit.workpapers,
            &audit_dir.join("audit_workpapers.json"),
            "Audit workpapers",
        );
        write_json_safe(
            &result.audit.evidence,
            &audit_dir.join("audit_evidence.json"),
            "Audit evidence",
        );
        write_json_safe(
            &result.audit.risk_assessments,
            &audit_dir.join("audit_risk_assessments.json"),
            "Audit risk assessments",
        );
        write_json_safe(
            &result.audit.findings,
            &audit_dir.join("audit_findings.json"),
            "Audit findings",
        );
        write_json_safe(
            &result.audit.judgments,
            &audit_dir.join("audit_judgments.json"),
            "Audit judgments",
        );
        write_json_safe(
            &result.audit.confirmations,
            &audit_dir.join("audit_confirmations.json"),
            "Audit confirmations",
        );
        write_json_safe(
            &result.audit.confirmation_responses,
            &audit_dir.join("audit_confirmation_responses.json"),
            "Audit confirmation responses",
        );
        write_json_safe(
            &result.audit.procedure_steps,
            &audit_dir.join("audit_procedure_steps.json"),
            "Audit procedure steps",
        );
        write_json_safe(
            &result.audit.samples,
            &audit_dir.join("audit_samples.json"),
            "Audit samples",
        );
        write_json_safe(
            &result.audit.analytical_results,
            &audit_dir.join("audit_analytical_results.json"),
            "Audit analytical results",
        );
        write_json_safe(
            &result.audit.ia_functions,
            &audit_dir.join("audit_ia_functions.json"),
            "Audit IA functions",
        );
        write_json_safe(
            &result.audit.ia_reports,
            &audit_dir.join("audit_ia_reports.json"),
            "Audit IA reports",
        );
        write_json_safe(
            &result.audit.related_parties,
            &audit_dir.join("audit_related_parties.json"),
            "Audit related parties",
        );
        write_json_safe(
            &result.audit.related_party_transactions,
            &audit_dir.join("audit_related_party_transactions.json"),
            "Audit related party transactions",
        );
        // ISA 600: Group audit artefacts
        if !result.audit.component_auditors.is_empty() {
            write_json_safe(
                &result.audit.component_auditors,
                &audit_dir.join("component_auditors.json"),
                "Component auditors (ISA 600)",
            );
            if let Some(plan) = &result.audit.group_audit_plan {
                write_json_single_safe(
                    plan,
                    &audit_dir.join("group_audit_plan.json"),
                    "Group audit plan (ISA 600)",
                );
            }
            write_json_safe(
                &result.audit.component_instructions,
                &audit_dir.join("component_instructions.json"),
                "Component instructions (ISA 600)",
            );
            write_json_safe(
                &result.audit.component_reports,
                &audit_dir.join("component_reports.json"),
                "Component auditor reports (ISA 600)",
            );
        }
    }

    // ========================================================================
    // Banking (JSON - keep existing format for backward compat)
    // ========================================================================
    let banking_dir = output_dir.join("banking");
    if !result.banking.customers.is_empty() {
        std::fs::create_dir_all(&banking_dir)?;
        info!("Writing banking data...");

        write_json_safe(
            &result.banking.customers,
            &banking_dir.join("banking_customers.json"),
            "Banking customers",
        );
        write_json_safe(
            &result.banking.accounts,
            &banking_dir.join("banking_accounts.json"),
            "Banking accounts",
        );
        write_json_safe(
            &result.banking.transactions,
            &banking_dir.join("banking_transactions.json"),
            "Banking transactions",
        );
        write_json_safe(
            &result.banking.transaction_labels,
            &banking_dir.join("aml_transaction_labels.json"),
            "AML transaction labels",
        );
        write_json_safe(
            &result.banking.customer_labels,
            &banking_dir.join("aml_customer_labels.json"),
            "AML customer labels",
        );
        write_json_safe(
            &result.banking.account_labels,
            &banking_dir.join("aml_account_labels.json"),
            "AML account labels",
        );
        write_json_safe(
            &result.banking.relationship_labels,
            &banking_dir.join("aml_relationship_labels.json"),
            "AML relationship labels",
        );
        write_json_safe(
            &result.banking.narratives,
            &banking_dir.join("aml_narratives.json"),
            "AML narratives",
        );
    }

    // ========================================================================
    // Sourcing (S2C)
    // ========================================================================
    let s2c_dir = output_dir.join("sourcing");
    if !result.sourcing.spend_analyses.is_empty() || !result.sourcing.sourcing_projects.is_empty() {
        std::fs::create_dir_all(&s2c_dir)?;
        info!("Writing sourcing (S2C) data...");

        write_json_safe(
            &result.sourcing.spend_analyses,
            &s2c_dir.join("spend_analyses.json"),
            "Spend analyses",
        );
        write_json_safe(
            &result.sourcing.sourcing_projects,
            &s2c_dir.join("sourcing_projects.json"),
            "Sourcing projects",
        );
        write_json_safe(
            &result.sourcing.qualifications,
            &s2c_dir.join("supplier_qualifications.json"),
            "Supplier qualifications",
        );
        write_json_safe(
            &result.sourcing.rfx_events,
            &s2c_dir.join("rfx_events.json"),
            "RFx events",
        );
        write_json_safe(
            &result.sourcing.bids,
            &s2c_dir.join("supplier_bids.json"),
            "Supplier bids",
        );
        write_json_safe(
            &result.sourcing.bid_evaluations,
            &s2c_dir.join("bid_evaluations.json"),
            "Bid evaluations",
        );
        write_json_safe(
            &result.sourcing.contracts,
            &s2c_dir.join("procurement_contracts.json"),
            "Procurement contracts",
        );
        write_json_safe(
            &result.sourcing.catalog_items,
            &s2c_dir.join("catalog_items.json"),
            "Catalog items",
        );
        write_json_safe(
            &result.sourcing.scorecards,
            &s2c_dir.join("supplier_scorecards.json"),
            "Supplier scorecards",
        );
    }

    // ========================================================================
    // Intercompany
    // ========================================================================
    let ic_dir = output_dir.join("intercompany");
    if result.intercompany.group_structure.is_some()
        || !result.intercompany.matched_pairs.is_empty()
    {
        std::fs::create_dir_all(&ic_dir)?;
        info!("Writing intercompany data...");

        // Always write group structure when present (independent of IC transactions).
        if let Some(gs) = &result.intercompany.group_structure {
            write_json_single_safe(
                gs,
                &ic_dir.join("group_structure.json"),
                "Group structure",
            );
        }

        write_json_safe(
            &result.intercompany.matched_pairs,
            &ic_dir.join("ic_matched_pairs.json"),
            "IC matched pairs",
        );
        write_json_safe(
            &result.intercompany.seller_journal_entries,
            &ic_dir.join("ic_seller_journal_entries.json"),
            "IC seller journal entries",
        );
        write_json_safe(
            &result.intercompany.buyer_journal_entries,
            &ic_dir.join("ic_buyer_journal_entries.json"),
            "IC buyer journal entries",
        );
        write_json_safe(
            &result.intercompany.elimination_entries,
            &ic_dir.join("ic_elimination_entries.json"),
            "IC elimination entries",
        );
    }

    // ========================================================================
    // Financial Reporting
    // ========================================================================
    let fin_dir = output_dir.join("financial_reporting");
    if !result.financial_reporting.financial_statements.is_empty()
        || !result.financial_reporting.bank_reconciliations.is_empty()
        || !result.financial_reporting.consolidated_statements.is_empty()
    {
        std::fs::create_dir_all(&fin_dir)?;
        info!("Writing financial reporting data...");

        // Legacy flat file (all standalone statements combined)
        write_json_safe(
            &result.financial_reporting.financial_statements,
            &fin_dir.join("financial_statements.json"),
            "Financial statements",
        );

        // Per-entity standalone statements
        if !result.financial_reporting.standalone_statements.is_empty() {
            let standalone_dir = fin_dir.join("standalone");
            std::fs::create_dir_all(&standalone_dir)?;
            for (entity_code, stmts) in &result.financial_reporting.standalone_statements {
                let file_name = format!("{}_financial_statements.json", entity_code);
                write_json_safe(
                    stmts,
                    &standalone_dir.join(&file_name),
                    &format!("Standalone statements for {}", entity_code),
                );
            }
        }

        // Consolidated statements + schedule
        if !result.financial_reporting.consolidated_statements.is_empty()
            || !result.financial_reporting.consolidation_schedules.is_empty()
        {
            let consolidated_dir = fin_dir.join("consolidated");
            std::fs::create_dir_all(&consolidated_dir)?;
            write_json_safe(
                &result.financial_reporting.consolidated_statements,
                &consolidated_dir.join("consolidated_financial_statements.json"),
                "Consolidated financial statements",
            );
            write_json_safe(
                &result.financial_reporting.consolidation_schedules,
                &consolidated_dir.join("consolidation_schedule.json"),
                "Consolidation schedule",
            );
        }

        write_json_safe(
            &result.financial_reporting.bank_reconciliations,
            &fin_dir.join("bank_reconciliations.json"),
            "Bank reconciliations",
        );
    }

    // ========================================================================
    // Period-Close Trial Balances
    // ========================================================================
    if !result.financial_reporting.trial_balances.is_empty() {
        let pc_dir = output_dir.join("period_close");
        std::fs::create_dir_all(&pc_dir)?;
        info!(
            "Writing {} period-close trial balances...",
            result.financial_reporting.trial_balances.len()
        );
        write_json_safe(
            &result.financial_reporting.trial_balances,
            &pc_dir.join("trial_balances.json"),
            "Period-close trial balances",
        );
    }

    // ========================================================================
    // Balance: Opening Balances + GL-Subledger Reconciliation
    // ========================================================================
    if !result.opening_balances.is_empty() || !result.subledger_reconciliation.is_empty() {
        let balance_dir = output_dir.join("balance");
        std::fs::create_dir_all(&balance_dir)?;
        info!("Writing balance data...");

        write_json_safe(
            &result.opening_balances,
            &balance_dir.join("opening_balances.json"),
            "Opening balances",
        );
        write_json_safe(
            &result.subledger_reconciliation,
            &balance_dir.join("subledger_reconciliation.json"),
            "Subledger reconciliation",
        );
    }

    // ========================================================================
    // HR (Payroll, Time Entries, Expense Reports, Benefit Enrollments)
    // ========================================================================
    let hr_dir = output_dir.join("hr");
    if !result.hr.payroll_runs.is_empty()
        || !result.hr.time_entries.is_empty()
        || !result.hr.expense_reports.is_empty()
        || !result.hr.benefit_enrollments.is_empty()
    {
        std::fs::create_dir_all(&hr_dir)?;
        info!("Writing HR data...");

        write_json_safe(
            &result.hr.payroll_runs,
            &hr_dir.join("payroll_runs.json"),
            "Payroll runs",
        );
        write_json_safe(
            &result.hr.payroll_line_items,
            &hr_dir.join("payroll_line_items.json"),
            "Payroll line items",
        );
        write_json_safe(
            &result.hr.time_entries,
            &hr_dir.join("time_entries.json"),
            "Time entries",
        );
        write_json_safe(
            &result.hr.expense_reports,
            &hr_dir.join("expense_reports.json"),
            "Expense reports",
        );
        write_json_safe(
            &result.hr.benefit_enrollments,
            &hr_dir.join("benefit_enrollments.json"),
            "Benefit enrollments",
        );
    }

    // ========================================================================
    // Manufacturing
    // ========================================================================
    let mfg_dir = output_dir.join("manufacturing");
    if !result.manufacturing.production_orders.is_empty()
        || !result.manufacturing.quality_inspections.is_empty()
        || !result.manufacturing.cycle_counts.is_empty()
        || !result.manufacturing.bom_components.is_empty()
        || !result.manufacturing.inventory_movements.is_empty()
    {
        std::fs::create_dir_all(&mfg_dir)?;
        info!("Writing manufacturing data...");

        write_json_safe(
            &result.manufacturing.production_orders,
            &mfg_dir.join("production_orders.json"),
            "Production orders",
        );
        write_json_safe(
            &result.manufacturing.quality_inspections,
            &mfg_dir.join("quality_inspections.json"),
            "Quality inspections",
        );
        write_json_safe(
            &result.manufacturing.cycle_counts,
            &mfg_dir.join("cycle_counts.json"),
            "Cycle counts",
        );
        write_json_safe(
            &result.manufacturing.bom_components,
            &mfg_dir.join("bom_components.json"),
            "BOM components",
        );
        write_json_safe(
            &result.manufacturing.inventory_movements,
            &mfg_dir.join("inventory_movements.json"),
            "Inventory movements",
        );
    }

    // ========================================================================
    // Sales, KPIs, Budgets
    // ========================================================================
    let sales_dir = output_dir.join("sales_kpi_budgets");
    if !result.sales_kpi_budgets.sales_quotes.is_empty()
        || !result.sales_kpi_budgets.kpis.is_empty()
        || !result.sales_kpi_budgets.budgets.is_empty()
    {
        std::fs::create_dir_all(&sales_dir)?;
        info!("Writing sales, KPI, and budget data...");

        write_json_safe(
            &result.sales_kpi_budgets.sales_quotes,
            &sales_dir.join("sales_quotes.json"),
            "Sales quotes",
        );
        write_json_safe(
            &result.sales_kpi_budgets.kpis,
            &sales_dir.join("management_kpis.json"),
            "Management KPIs",
        );
        write_json_safe(
            &result.sales_kpi_budgets.budgets,
            &sales_dir.join("budgets.json"),
            "Budgets",
        );
    }

    // ========================================================================
    // Tax
    // ========================================================================
    let tax_dir = output_dir.join("tax");
    if !result.tax.jurisdictions.is_empty()
        || !result.tax.codes.is_empty()
        || !result.tax.tax_provisions.is_empty()
    {
        std::fs::create_dir_all(&tax_dir)?;
        info!("Writing tax data...");

        write_json_safe(
            &result.tax.jurisdictions,
            &tax_dir.join("tax_jurisdictions.json"),
            "Tax jurisdictions",
        );
        write_json_safe(
            &result.tax.codes,
            &tax_dir.join("tax_codes.json"),
            "Tax codes",
        );
        write_json_safe(
            &result.tax.tax_provisions,
            &tax_dir.join("tax_provisions.json"),
            "Tax provisions",
        );
        write_json_safe(
            &result.tax.tax_lines,
            &tax_dir.join("tax_lines.json"),
            "Tax lines",
        );
        write_json_safe(
            &result.tax.tax_returns,
            &tax_dir.join("tax_returns.json"),
            "Tax returns",
        );
        write_json_safe(
            &result.tax.withholding_records,
            &tax_dir.join("withholding_records.json"),
            "Withholding tax records",
        );
        if !result.tax.tax_anomaly_labels.is_empty() {
            write_json_safe(
                &result.tax.tax_anomaly_labels,
                &tax_dir.join("tax_anomaly_labels.json"),
                "Tax anomaly labels",
            );
        }
        // Deferred tax engine output (IAS 12 / ASC 740)
        if !result.tax.deferred_tax.temporary_differences.is_empty() {
            write_json_safe(
                &result.tax.deferred_tax.temporary_differences,
                &tax_dir.join("temporary_differences.json"),
                "Temporary differences",
            );
            write_json_safe(
                &result.tax.deferred_tax.etr_reconciliations,
                &tax_dir.join("etr_reconciliation.json"),
                "ETR reconciliation",
            );
            write_json_safe(
                &result.tax.deferred_tax.rollforwards,
                &tax_dir.join("deferred_tax_rollforward.json"),
                "Deferred tax rollforward",
            );
            write_json_safe(
                &result.tax.deferred_tax.journal_entries,
                &tax_dir.join("deferred_tax_journal_entries.json"),
                "Deferred tax journal entries",
            );
        }
    }

    // ========================================================================
    // ESG
    // ========================================================================
    let esg_dir = output_dir.join("esg");
    if !result.esg.emissions.is_empty()
        || !result.esg.energy.is_empty()
        || !result.esg.diversity.is_empty()
        || !result.esg.governance.is_empty()
    {
        std::fs::create_dir_all(&esg_dir)?;
        info!("Writing ESG data...");

        write_json_safe(
            &result.esg.emissions,
            &esg_dir.join("emission_records.json"),
            "Emission records",
        );
        write_json_safe(
            &result.esg.energy,
            &esg_dir.join("energy_consumption.json"),
            "Energy consumption",
        );
        write_json_safe(
            &result.esg.water,
            &esg_dir.join("water_usage.json"),
            "Water usage",
        );
        write_json_safe(
            &result.esg.waste,
            &esg_dir.join("waste_records.json"),
            "Waste records",
        );
        write_json_safe(
            &result.esg.diversity,
            &esg_dir.join("workforce_diversity.json"),
            "Workforce diversity",
        );
        write_json_safe(
            &result.esg.pay_equity,
            &esg_dir.join("pay_equity.json"),
            "Pay equity",
        );
        write_json_safe(
            &result.esg.safety_incidents,
            &esg_dir.join("safety_incidents.json"),
            "Safety incidents",
        );
        write_json_safe(
            &result.esg.safety_metrics,
            &esg_dir.join("safety_metrics.json"),
            "Safety metrics",
        );
        write_json_safe(
            &result.esg.governance,
            &esg_dir.join("governance_metrics.json"),
            "Governance metrics",
        );
        write_json_safe(
            &result.esg.supplier_assessments,
            &esg_dir.join("supplier_esg_assessments.json"),
            "Supplier ESG assessments",
        );
        write_json_safe(
            &result.esg.materiality,
            &esg_dir.join("materiality_assessments.json"),
            "Materiality assessments",
        );
        write_json_safe(
            &result.esg.disclosures,
            &esg_dir.join("esg_disclosures.json"),
            "ESG disclosures",
        );
        write_json_safe(
            &result.esg.climate_scenarios,
            &esg_dir.join("climate_scenarios.json"),
            "Climate scenarios",
        );
        write_json_safe(
            &result.esg.anomaly_labels,
            &esg_dir.join("esg_anomaly_labels.json"),
            "ESG anomaly labels",
        );
    }

    // ========================================================================
    // Process Mining (OCPM)
    // ========================================================================
    if let Some(ref event_log) = result.ocpm.event_log {
        if !event_log.events.is_empty() || !event_log.objects.is_empty() {
            let pm_dir = output_dir.join("process_mining");
            std::fs::create_dir_all(&pm_dir)?;
            info!("Writing process mining (OCPM) data...");

            // Write the full OCEL 2.0 event log
            match serde_json::to_string_pretty(event_log) {
                Ok(json) => {
                    if let Err(e) = std::fs::write(pm_dir.join("event_log.json"), json) {
                        warn!("Failed to write OCPM event log: {}", e);
                    } else {
                        info!(
                            "  Event log written: {} events, {} objects",
                            result.ocpm.event_count, result.ocpm.object_count
                        );
                    }
                }
                Err(e) => warn!("Failed to serialize OCPM event log: {}", e),
            }

            // Write events separately for easy consumption
            if !event_log.events.is_empty() {
                match serde_json::to_string_pretty(&event_log.events) {
                    Ok(json) => {
                        if let Err(e) = std::fs::write(pm_dir.join("events.json"), json) {
                            warn!("Failed to write OCPM events: {}", e);
                        } else {
                            info!("  Events written: {} records", event_log.events.len());
                        }
                    }
                    Err(e) => warn!("Failed to serialize OCPM events: {}", e),
                }
            }

            // Write objects separately for easy consumption
            if !event_log.objects.is_empty() {
                let objects: Vec<&_> = event_log.objects.iter().collect();
                match serde_json::to_string_pretty(&objects) {
                    Ok(json) => {
                        if let Err(e) = std::fs::write(pm_dir.join("objects.json"), json) {
                            warn!("Failed to write OCPM objects: {}", e);
                        } else {
                            info!("  Objects written: {} records", event_log.objects.len());
                        }
                    }
                    Err(e) => warn!("Failed to serialize OCPM objects: {}", e),
                }
            }

            // Write process variants if any were computed
            if !event_log.variants.is_empty() {
                let variants: Vec<&_> = event_log.variants.values().collect();
                match serde_json::to_string_pretty(&variants) {
                    Ok(json) => {
                        if let Err(e) = std::fs::write(pm_dir.join("process_variants.json"), json) {
                            warn!("Failed to write process variants: {}", e);
                        } else {
                            info!(
                                "  Process variants written: {} variants",
                                event_log.variants.len()
                            );
                        }
                    }
                    Err(e) => warn!("Failed to serialize process variants: {}", e),
                }
            }
        }
    }

    // ========================================================================
    // Chart of Accounts
    // ========================================================================
    match serde_json::to_string_pretty(&result.chart_of_accounts) {
        Ok(json) => {
            if let Err(e) = std::fs::write(output_dir.join("chart_of_accounts.json"), json) {
                warn!("Failed to write chart of accounts: {}", e);
            } else {
                info!("  Chart of accounts written");
            }
        }
        Err(e) => warn!("Failed to serialize chart of accounts: {}", e),
    }

    // ========================================================================
    // Balance Validation Summary
    // ========================================================================
    if result.balance_validation.validated {
        match serde_json::to_string_pretty(&BalanceValidationSummary::from(
            &result.balance_validation,
        )) {
            Ok(json) => {
                if let Err(e) = std::fs::write(output_dir.join("balance_validation.json"), json) {
                    warn!("Failed to write balance validation: {}", e);
                } else {
                    info!("  Balance validation summary written");
                }
            }
            Err(e) => warn!("Failed to serialize balance validation: {}", e),
        }
    }

    // ========================================================================
    // Data Quality Statistics (now serializable directly via Serialize derives)
    // ========================================================================
    {
        match serde_json::to_string_pretty(&result.data_quality_stats) {
            Ok(json) => {
                if let Err(e) = std::fs::write(output_dir.join("data_quality_stats.json"), json) {
                    warn!("Failed to write data quality stats: {}", e);
                } else {
                    info!("  Data quality stats written (full detail)");
                }
            }
            Err(e) => warn!("Failed to serialize data quality stats: {}", e),
        }
    }

    // ========================================================================
    // Internal Controls
    // ========================================================================
    if !result.internal_controls.is_empty() {
        let ctrl_dir = output_dir.join("internal_controls");
        std::fs::create_dir_all(&ctrl_dir)?;
        info!("Writing internal controls data...");

        write_json_safe(
            &result.internal_controls,
            &ctrl_dir.join("internal_controls.json"),
            "Internal controls",
        );
    }

    // ========================================================================
    // Accounting Standards
    // ========================================================================
    if !result.accounting_standards.contracts.is_empty()
        || !result.accounting_standards.impairment_tests.is_empty()
    {
        let acct_dir = output_dir.join("accounting_standards");
        std::fs::create_dir_all(&acct_dir)?;
        info!("Writing accounting standards data...");

        write_json_safe(
            &result.accounting_standards.contracts,
            &acct_dir.join("customer_contracts.json"),
            "Customer contracts",
        );
        write_json_safe(
            &result.accounting_standards.impairment_tests,
            &acct_dir.join("impairment_tests.json"),
            "Impairment tests",
        );
    }

    // ========================================================================
    // Quality Gate Results
    // ========================================================================
    if let Some(ref gate_result) = result.gate_result {
        match serde_json::to_string_pretty(gate_result) {
            Ok(json) => {
                if let Err(e) = std::fs::write(output_dir.join("quality_gate_result.json"), json) {
                    warn!("Failed to write quality gate result: {}", e);
                } else {
                    info!(
                        "  Quality gate result written (passed={})",
                        gate_result.passed
                    );
                }
            }
            Err(e) => warn!("Failed to serialize quality gate result: {}", e),
        }
    }

    // ========================================================================
    // Treasury
    // ========================================================================
    if !result.treasury.debt_instruments.is_empty()
        || !result.treasury.cash_positions.is_empty()
        || !result.treasury.hedging_instruments.is_empty()
    {
        let treasury_dir = output_dir.join("treasury");
        std::fs::create_dir_all(&treasury_dir)?;
        info!("Writing treasury data...");

        write_json_safe(
            &result.treasury.debt_instruments,
            &treasury_dir.join("debt_instruments.json"),
            "Debt instruments",
        );
        write_json_safe(
            &result.treasury.hedging_instruments,
            &treasury_dir.join("hedging_instruments.json"),
            "Hedging instruments",
        );
        write_json_safe(
            &result.treasury.hedge_relationships,
            &treasury_dir.join("hedge_relationships.json"),
            "Hedge relationships",
        );
        write_json_safe(
            &result.treasury.cash_positions,
            &treasury_dir.join("cash_positions.json"),
            "Cash positions",
        );
        write_json_safe(
            &result.treasury.cash_forecasts,
            &treasury_dir.join("cash_forecasts.json"),
            "Cash forecasts",
        );
        write_json_safe(
            &result.treasury.cash_pools,
            &treasury_dir.join("cash_pools.json"),
            "Cash pools",
        );
        write_json_safe(
            &result.treasury.cash_pool_sweeps,
            &treasury_dir.join("cash_pool_sweeps.json"),
            "Cash pool sweeps",
        );
        write_json_safe(
            &result.treasury.bank_guarantees,
            &treasury_dir.join("bank_guarantees.json"),
            "Bank guarantees",
        );
        write_json_safe(
            &result.treasury.netting_runs,
            &treasury_dir.join("netting_runs.json"),
            "Netting runs",
        );
        if !result.treasury.treasury_anomaly_labels.is_empty() {
            write_json_safe(
                &result.treasury.treasury_anomaly_labels,
                &treasury_dir.join("treasury_anomaly_labels.json"),
                "Treasury anomaly labels",
            );
        }
    }

    // ========================================================================
    // Project Accounting
    // ========================================================================
    if !result.project_accounting.projects.is_empty() {
        let pa_dir = output_dir.join("project_accounting");
        std::fs::create_dir_all(&pa_dir)?;
        info!("Writing project accounting data...");

        write_json_safe(
            &result.project_accounting.projects,
            &pa_dir.join("projects.json"),
            "Projects",
        );
        write_json_safe(
            &result.project_accounting.cost_lines,
            &pa_dir.join("cost_lines.json"),
            "Project cost lines",
        );
        write_json_safe(
            &result.project_accounting.revenue_records,
            &pa_dir.join("revenue_records.json"),
            "Project revenue records",
        );
        write_json_safe(
            &result.project_accounting.earned_value_metrics,
            &pa_dir.join("earned_value_metrics.json"),
            "Earned value metrics",
        );
        write_json_safe(
            &result.project_accounting.change_orders,
            &pa_dir.join("change_orders.json"),
            "Change orders",
        );
        write_json_safe(
            &result.project_accounting.milestones,
            &pa_dir.join("milestones.json"),
            "Project milestones",
        );
    }

    // ========================================================================
    // Evolution Events (Process Evolution + Organizational Events)
    // ========================================================================
    if !result.process_evolution.is_empty()
        || !result.organizational_events.is_empty()
        || !result.disruption_events.is_empty()
    {
        let events_dir = output_dir.join("events");
        std::fs::create_dir_all(&events_dir)?;
        info!("Writing evolution events...");

        write_json_safe(
            &result.process_evolution,
            &events_dir.join("process_evolution_events.json"),
            "Process evolution events",
        );
        write_json_safe(
            &result.organizational_events,
            &events_dir.join("organizational_events.json"),
            "Organizational events",
        );
        write_json_safe(
            &result.disruption_events,
            &events_dir.join("disruption_events.json"),
            "Disruption events",
        );
    }

    // ========================================================================
    // ML Training: Counterfactual Pairs
    // ========================================================================
    if !result.counterfactual_pairs.is_empty() {
        let ml_dir = output_dir.join("ml_training");
        std::fs::create_dir_all(&ml_dir)?;
        info!("Writing ML training data...");

        write_json_safe(
            &result.counterfactual_pairs,
            &ml_dir.join("counterfactual_pairs.json"),
            "Counterfactual pairs",
        );
    }

    // ========================================================================
    // Fraud Red-Flag Indicators
    // ========================================================================
    if !result.red_flags.is_empty() {
        let labels_dir = output_dir.join("labels");
        std::fs::create_dir_all(&labels_dir)?;
        info!("Writing fraud red-flag indicators...");

        write_json_safe(
            &result.red_flags,
            &labels_dir.join("fraud_red_flags.json"),
            "Fraud red flags",
        );
    }

    // ========================================================================
    // Collusion Rings
    // ========================================================================
    if !result.collusion_rings.is_empty() {
        let labels_dir = output_dir.join("labels");
        std::fs::create_dir_all(&labels_dir)?;
        info!("Writing collusion rings...");

        write_json_safe(
            &result.collusion_rings,
            &labels_dir.join("collusion_rings.json"),
            "Collusion rings",
        );
    }

    // ========================================================================
    // Temporal Vendor Version Chains
    // ========================================================================
    if !result.temporal_vendor_chains.is_empty() {
        let temporal_dir = output_dir.join("temporal");
        std::fs::create_dir_all(&temporal_dir)?;
        info!("Writing temporal vendor version chains...");

        write_json_safe(
            &result.temporal_vendor_chains,
            &temporal_dir.join("vendor_version_chains.json"),
            "Vendor version chains",
        );
    }

    // ========================================================================
    // Entity Relationship Graph + Cross-Process Links
    // ========================================================================
    if result.entity_relationship_graph.is_some() || !result.cross_process_links.is_empty() {
        let rel_dir = output_dir.join("relationships");
        std::fs::create_dir_all(&rel_dir)?;
        info!("Writing entity relationship data...");

        if let Some(ref graph) = result.entity_relationship_graph {
            match serde_json::to_string_pretty(graph) {
                Ok(json) => {
                    let path = rel_dir.join("entity_relationship_graph.json");
                    if let Err(e) = std::fs::write(&path, json) {
                        warn!("Failed to write entity relationship graph: {}", e);
                    } else {
                        info!(
                            "  Entity relationship graph written: {} nodes, {} edges -> {}",
                            graph.nodes.len(),
                            graph.edges.len(),
                            path.display()
                        );
                    }
                }
                Err(e) => warn!("Failed to serialize entity relationship graph: {}", e),
            }
        }

        write_json_safe(
            &result.cross_process_links,
            &rel_dir.join("cross_process_links.json"),
            "Cross-process links",
        );
    }

    // ========================================================================
    // Industry-Specific Data
    // ========================================================================
    if let Some(ref industry_output) = result.industry_output {
        if !industry_output.gl_accounts.is_empty() {
            let industry_dir = output_dir.join("industry");
            std::fs::create_dir_all(&industry_dir).ok();
            info!("Writing industry-specific data...");
            match serde_json::to_string_pretty(industry_output) {
                Ok(json) => {
                    if let Err(e) = std::fs::write(industry_dir.join("industry_data.json"), json) {
                        warn!("Failed to write industry data: {}", e);
                    } else {
                        info!(
                            "  Industry data written: {} GL accounts for {}",
                            industry_output.gl_accounts.len(),
                            industry_output.industry
                        );
                    }
                }
                Err(e) => warn!("Failed to serialize industry data: {}", e),
            }
        }
    }

    // ========================================================================
    // Graph Export Summary
    // ========================================================================
    if result.graph_export.exported {
        let graph_dir = output_dir.join("graph_export");
        std::fs::create_dir_all(&graph_dir).ok();
        match serde_json::to_string_pretty(&result.graph_export) {
            Ok(json) => {
                if let Err(e) = std::fs::write(graph_dir.join("graph_export_summary.json"), json) {
                    warn!("Failed to write graph export summary: {}", e);
                } else {
                    info!("  Graph export summary written");
                }
            }
            Err(e) => warn!("Failed to serialize graph export summary: {}", e),
        }
    }

    // ========================================================================
    // Compliance Regulations
    // ========================================================================
    let cr = &result.compliance_regulations;
    let has_compliance_data = !cr.standard_records.is_empty()
        || !cr.audit_procedures.is_empty()
        || !cr.findings.is_empty()
        || !cr.filings.is_empty();
    if has_compliance_data {
        let cr_dir = output_dir.join("compliance_regulations");
        std::fs::create_dir_all(&cr_dir)?;
        info!("Writing compliance regulations data...");

        write_json_safe(
            &cr.standard_records,
            &cr_dir.join("compliance_standards.json"),
            "Compliance standards",
        );
        write_json_safe(
            &cr.cross_reference_records,
            &cr_dir.join("cross_references.json"),
            "Cross-references",
        );
        write_json_safe(
            &cr.jurisdiction_records,
            &cr_dir.join("jurisdiction_profiles.json"),
            "Jurisdiction profiles",
        );
        write_json_safe(
            &cr.audit_procedures,
            &cr_dir.join("audit_procedures.json"),
            "Audit procedures",
        );
        write_json_safe(
            &cr.findings,
            &cr_dir.join("compliance_findings.json"),
            "Compliance findings",
        );
        write_json_safe(
            &cr.filings,
            &cr_dir.join("regulatory_filings.json"),
            "Regulatory filings",
        );

        if let Some(ref graph) = cr.compliance_graph {
            match serde_json::to_string_pretty(graph) {
                Ok(json) => {
                    if let Err(e) = std::fs::write(cr_dir.join("compliance_graph.json"), json) {
                        warn!("Failed to write compliance graph: {}", e);
                    } else {
                        info!(
                            "  Compliance graph written: {} nodes, {} edges",
                            graph.nodes.len(),
                            graph.edges.len()
                        );
                    }
                }
                Err(e) => warn!("Failed to serialize compliance graph: {}", e),
            }
        }
    }

    // ========================================================================
    // Generation Statistics
    // ========================================================================
    match serde_json::to_string_pretty(&result.statistics) {
        Ok(json) => {
            if let Err(e) = std::fs::write(output_dir.join("generation_statistics.json"), json) {
                warn!("Failed to write generation statistics: {}", e);
            } else {
                info!("  Generation statistics written");
            }
        }
        Err(e) => warn!("Failed to serialize generation statistics: {}", e),
    }

    info!("Output writing complete.");
    Ok(())
}

/// Write JSON with error handling - logs a warning on failure but does not abort.
fn write_json_safe<T: serde::Serialize>(data: &[T], path: &Path, label: &str) {
    if let Err(e) = write_json(data, path, label) {
        warn!("Failed to write {}: {}", label, e);
    }
}

/// Write a single serializable value as a JSON file.
fn write_json_single<T: serde::Serialize>(
    data: &T,
    path: &Path,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::create(path)?;
    let writer = std::io::BufWriter::with_capacity(256 * 1024, file);
    serde_json::to_writer_pretty(writer, data)?;
    info!("  {} written -> {}", label, path.display());
    Ok(())
}

/// Write a single serializable value as a JSON file, logging a warning on failure.
fn write_json_single_safe<T: serde::Serialize>(data: &T, path: &Path, label: &str) {
    if let Err(e) = write_json_single(data, path, label) {
        warn!("Failed to write {}: {}", label, e);
    }
}

/// Serializable summary of balance validation (avoids serializing the full
/// `BalanceValidationResult` which has non-Serialize validation error types).
#[derive(serde::Serialize)]
struct BalanceValidationSummary {
    validated: bool,
    is_balanced: bool,
    entries_processed: u64,
    total_debits: String,
    total_credits: String,
    accounts_tracked: usize,
    companies_tracked: usize,
    has_unbalanced_entries: bool,
    validation_error_count: usize,
}

impl BalanceValidationSummary {
    fn from(v: &datasynth_runtime::enhanced_orchestrator::BalanceValidationResult) -> Self {
        Self {
            validated: v.validated,
            is_balanced: v.is_balanced,
            entries_processed: v.entries_processed,
            total_debits: v.total_debits.to_string(),
            total_credits: v.total_credits.to_string(),
            accounts_tracked: v.accounts_tracked,
            companies_tracked: v.companies_tracked,
            has_unbalanced_entries: v.has_unbalanced_entries,
            validation_error_count: v.validation_errors.len(),
        }
    }
}
