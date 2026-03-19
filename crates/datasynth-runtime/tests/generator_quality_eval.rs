//! Comprehensive generator output quality evaluation.
//!
//! This test runs the full pipeline with all generators enabled and inspects
//! every output domain for quality issues:
//! - Non-empty outputs when enabled
//! - Referential integrity (cross-references between domains)
//! - Amount/date/ID validity
//! - Structural correctness
//! - Cross-generator consistency

#[allow(clippy::unwrap_used)]
mod eval {
    use chrono::NaiveDate;
    use datasynth_config::schema::TransactionVolume;
    use datasynth_runtime::{EnhancedOrchestrator, PhaseConfig};
    use datasynth_test_utils::fixtures::minimal_config;
    use rust_decimal::Decimal;
    use std::collections::HashSet;

    /// Build a config with everything enabled.
    fn full_config() -> datasynth_config::schema::GeneratorConfig {
        let mut config = minimal_config();
        config.global.seed = Some(42);
        config.global.period_months = 1;
        config.companies[0].annual_transaction_volume = TransactionVolume::Custom(500);

        // Add a second company for intercompany
        config
            .companies
            .push(datasynth_config::schema::CompanyConfig {
                code: "SUB1".to_string(),
                name: "Subsidiary One".to_string(),
                currency: "EUR".to_string(),
                functional_currency: None,
                country: "DE".to_string(),
                annual_transaction_volume: TransactionVolume::Custom(500),
                volume_weight: 0.4,
                fiscal_year_variant: "K4".to_string(),
            });

        // Enable fraud
        config.fraud.enabled = true;
        config.fraud.fraud_rate = 0.05;
        config.fraud.clustering_enabled = true;

        // Enable HR
        config.hr.enabled = true;
        config.hr.payroll.enabled = true;
        config.hr.time_attendance.enabled = true;
        config.hr.expenses.enabled = true;

        // Enable manufacturing
        config.manufacturing.enabled = true;

        // Enable tax
        config.tax.enabled = true;

        // Enable ESG
        config.esg.enabled = true;

        // Enable treasury
        config.treasury.enabled = true;
        config.treasury.cash_positioning.enabled = true;
        config.treasury.cash_forecasting.enabled = true;
        config.treasury.cash_pooling.enabled = true;

        // Enable project accounting
        config.project_accounting.enabled = true;

        // Enable financial reporting
        config.financial_reporting.enabled = true;
        config.financial_reporting.management_kpis.enabled = true;
        config.financial_reporting.budgets.enabled = true;

        // Enable sales quotes
        config.sales_quotes.enabled = true;

        // Enable accounting standards
        config.accounting_standards.enabled = true;
        config.accounting_standards.revenue_recognition.enabled = true;
        config.accounting_standards.impairment.enabled = true;

        // Enable temporal attributes
        config.temporal_attributes.enabled = true;
        config.temporal_attributes.generate_version_chains = true;

        // Enable relationships
        config.relationship_strength.enabled = true;
        config.cross_process_links.enabled = true;
        config.cross_process_links.inventory_p2p_o2c = true;
        config.cross_process_links.payment_bank_reconciliation = true;
        config.cross_process_links.intercompany_bilateral = true;

        // Enable organizational events
        config.organizational_events.enabled = true;

        // Enable industry specific
        config.industry_specific.enabled = true;

        // Enable intercompany
        config.intercompany.enabled = true;
        config.intercompany.generate_eliminations = true;

        // Enable audit
        config.audit.enabled = true;

        // Enable banking
        config.banking.enabled = true;

        // Enable internal controls
        config.internal_controls.enabled = true;
        config.internal_controls.coso_enabled = true;

        // Enable anomaly injection
        config.anomaly_injection.enabled = true;

        // Enable data quality
        config.data_quality.enabled = true;

        config
    }

    fn all_phases() -> PhaseConfig {
        PhaseConfig {
            generate_master_data: true,
            generate_document_flows: true,
            generate_ocpm_events: true,
            generate_journal_entries: true,
            inject_anomalies: true,
            inject_data_quality: true,
            validate_balances: true,
            show_progress: false,
            generate_audit: true,
            generate_banking: true,
            generate_graph_export: false, // skip file I/O
            generate_sourcing: true,
            generate_bank_reconciliation: true,
            generate_financial_statements: true,
            generate_accounting_standards: true,
            generate_manufacturing: true,
            generate_sales_kpi_budgets: true,
            generate_tax: true,
            generate_esg: true,
            generate_intercompany: true,
            generate_evolution_events: true,
            generate_counterfactuals: true,
            vendors_per_company: 5,
            customers_per_company: 5,
            materials_per_company: 3,
            assets_per_company: 3,
            employees_per_company: 5,
            p2p_chains: 4,
            o2c_chains: 4,
            audit_engagements: 2,
            workpapers_per_engagement: 3,
            evidence_per_workpaper: 2,
            risks_per_engagement: 3,
            findings_per_engagement: 2,
            judgments_per_engagement: 2,
            ..Default::default()
        }
    }

    // ========================================================================
    // Finding tracker
    // ========================================================================

    #[derive(Debug)]
    enum Severity {
        Error,
        Warning,
        Info,
    }

    struct Finding {
        domain: String,
        severity: Severity,
        message: String,
    }

    struct QualityReport {
        findings: Vec<Finding>,
    }

    impl QualityReport {
        fn new() -> Self {
            Self {
                findings: Vec::new(),
            }
        }

        fn error(&mut self, domain: &str, msg: impl Into<String>) {
            self.findings.push(Finding {
                domain: domain.to_string(),
                severity: Severity::Error,
                message: msg.into(),
            });
        }

        fn warn(&mut self, domain: &str, msg: impl Into<String>) {
            self.findings.push(Finding {
                domain: domain.to_string(),
                severity: Severity::Warning,
                message: msg.into(),
            });
        }

        fn info(&mut self, domain: &str, msg: impl Into<String>) {
            self.findings.push(Finding {
                domain: domain.to_string(),
                severity: Severity::Info,
                message: msg.into(),
            });
        }

        fn assert_non_empty<T>(&mut self, domain: &str, name: &str, items: &[T]) {
            if items.is_empty() {
                self.error(domain, format!("{} is EMPTY (expected non-empty)", name));
            } else {
                self.info(domain, format!("{}: {} items", name, items.len()));
            }
        }

        fn print_report(&self) {
            eprintln!("\n{}", "=".repeat(80));
            eprintln!("  GENERATOR QUALITY EVALUATION REPORT");
            eprintln!("{}\n", "=".repeat(80));

            let errors: Vec<_> = self
                .findings
                .iter()
                .filter(|f| matches!(f.severity, Severity::Error))
                .collect();
            let warnings: Vec<_> = self
                .findings
                .iter()
                .filter(|f| matches!(f.severity, Severity::Warning))
                .collect();
            let infos: Vec<_> = self
                .findings
                .iter()
                .filter(|f| matches!(f.severity, Severity::Info))
                .collect();

            if !errors.is_empty() {
                eprintln!("--- ERRORS ({}) ---", errors.len());
                for f in &errors {
                    eprintln!("  [{}] {}", f.domain, f.message);
                }
                eprintln!();
            }

            if !warnings.is_empty() {
                eprintln!("--- WARNINGS ({}) ---", warnings.len());
                for f in &warnings {
                    eprintln!("  [{}] {}", f.domain, f.message);
                }
                eprintln!();
            }

            eprintln!("--- INFO ({}) ---", infos.len());
            for f in &infos {
                eprintln!("  [{}] {}", f.domain, f.message);
            }

            eprintln!("\n--- SUMMARY ---");
            eprintln!(
                "  Errors: {}  Warnings: {}  Info: {}",
                errors.len(),
                warnings.len(),
                infos.len()
            );

            if !errors.is_empty() {
                eprintln!("\n  *** QUALITY ISSUES FOUND ***");
            } else if !warnings.is_empty() {
                eprintln!("\n  Minor warnings found, but no critical issues.");
            } else {
                eprintln!("\n  All generators producing quality output.");
            }
        }

        fn error_count(&self) -> usize {
            self.findings
                .iter()
                .filter(|f| matches!(f.severity, Severity::Error))
                .count()
        }
    }

    // ========================================================================
    // Helper: date range
    // ========================================================================

    fn expected_date_range() -> (NaiveDate, NaiveDate) {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        (start, end)
    }

    fn date_in_range(date: NaiveDate, margin_days: i64) -> bool {
        let (start, end) = expected_date_range();
        let start = start - chrono::Duration::days(margin_days);
        let end = end + chrono::Duration::days(margin_days);
        date >= start && date <= end
    }

    // ========================================================================
    // Main evaluation test
    // ========================================================================

    #[test]
    #[ignore] // Long-running (~minutes): run with `cargo test -- --ignored`
    fn evaluate_all_generators() {
        let config = full_config();
        let phase_config = all_phases();
        let mut orchestrator =
            EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");
        let result = orchestrator.generate().expect("Generation failed");
        let mut report = QualityReport::new();

        // ================================================================
        // 1. MASTER DATA
        // ================================================================
        report.assert_non_empty("master_data", "vendors", &result.master_data.vendors);
        report.assert_non_empty("master_data", "customers", &result.master_data.customers);
        report.assert_non_empty("master_data", "materials", &result.master_data.materials);
        report.assert_non_empty("master_data", "assets", &result.master_data.assets);
        report.assert_non_empty("master_data", "employees", &result.master_data.employees);

        // Check vendor IDs are unique
        let vendor_ids: HashSet<_> = result
            .master_data
            .vendors
            .iter()
            .map(|v| &v.vendor_id)
            .collect();
        if vendor_ids.len() != result.master_data.vendors.len() {
            report.error(
                "master_data",
                format!(
                    "Duplicate vendor IDs: {} unique out of {}",
                    vendor_ids.len(),
                    result.master_data.vendors.len()
                ),
            );
        }

        // Check customer IDs are unique
        let customer_ids: HashSet<_> = result
            .master_data
            .customers
            .iter()
            .map(|c| &c.customer_id)
            .collect();
        if customer_ids.len() != result.master_data.customers.len() {
            report.error(
                "master_data",
                format!(
                    "Duplicate customer IDs: {} unique out of {}",
                    customer_ids.len(),
                    result.master_data.customers.len()
                ),
            );
        }

        // Check material IDs are unique
        let material_ids: HashSet<_> = result
            .master_data
            .materials
            .iter()
            .map(|m| &m.material_id)
            .collect();
        if material_ids.len() != result.master_data.materials.len() {
            report.error(
                "master_data",
                format!(
                    "Duplicate material IDs: {} unique out of {}",
                    material_ids.len(),
                    result.master_data.materials.len()
                ),
            );
        }

        // Check vendor amounts: typical_amount_range min < max
        for v in &result.master_data.vendors {
            if v.typical_amount_range.0 >= v.typical_amount_range.1 {
                report.error(
                    "master_data",
                    format!(
                        "Vendor {} has invalid amount range: {:?}",
                        v.vendor_id, v.typical_amount_range
                    ),
                );
            }
        }

        // Check employee IDs unique
        let employee_ids: HashSet<_> = result
            .master_data
            .employees
            .iter()
            .map(|e| &e.employee_id)
            .collect();
        if employee_ids.len() != result.master_data.employees.len() {
            report.error(
                "master_data",
                format!(
                    "Duplicate employee IDs: {} unique out of {}",
                    employee_ids.len(),
                    result.master_data.employees.len()
                ),
            );
        }

        // ================================================================
        // 2. DOCUMENT FLOWS
        // ================================================================
        report.assert_non_empty("doc_flows", "p2p_chains", &result.document_flows.p2p_chains);
        report.assert_non_empty("doc_flows", "o2c_chains", &result.document_flows.o2c_chains);

        // P2P: Check PO vendor_id references valid vendor
        let mut po_vendor_missing = 0;
        let mut po_amount_zero = 0;
        let mut po_date_out_of_range = 0;
        for chain in &result.document_flows.p2p_chains {
            let po = &chain.purchase_order;
            if !vendor_ids.contains(&po.vendor_id) {
                po_vendor_missing += 1;
            }
            if po.total_net_amount <= Decimal::ZERO {
                po_amount_zero += 1;
            }
            if !date_in_range(po.header.document_date, 30) {
                po_date_out_of_range += 1;
            }
            // GR should exist for each PO
            if chain.goods_receipts.is_empty() {
                report.warn(
                    "doc_flows",
                    format!(
                        "P2P chain PO {} has no goods receipts",
                        po.header.document_id
                    ),
                );
            }
        }
        if po_vendor_missing > 0 {
            report.error(
                "doc_flows",
                format!(
                    "{} POs reference vendor IDs not in master data",
                    po_vendor_missing
                ),
            );
        }
        if po_amount_zero > 0 {
            report.error(
                "doc_flows",
                format!(
                    "{} POs have zero or negative total_net_amount",
                    po_amount_zero
                ),
            );
        }
        if po_date_out_of_range > 0 {
            report.warn(
                "doc_flows",
                format!(
                    "{} POs have dates outside expected range",
                    po_date_out_of_range
                ),
            );
        }

        // O2C: Check SO customer_id references valid customer
        let mut so_customer_missing = 0;
        let mut so_amount_zero = 0;
        for chain in &result.document_flows.o2c_chains {
            let so = &chain.sales_order;
            if !customer_ids.contains(&so.customer_id) {
                so_customer_missing += 1;
            }
            if so.total_net_amount <= Decimal::ZERO {
                so_amount_zero += 1;
            }
        }
        if so_customer_missing > 0 {
            report.error(
                "doc_flows",
                format!(
                    "{} SOs reference customer IDs not in master data",
                    so_customer_missing
                ),
            );
        }
        if so_amount_zero > 0 {
            report.error(
                "doc_flows",
                format!(
                    "{} SOs have zero or negative total_net_amount",
                    so_amount_zero
                ),
            );
        }

        // ================================================================
        // 3. JOURNAL ENTRIES
        // ================================================================
        report.assert_non_empty(
            "journal_entries",
            "journal_entries",
            &result.journal_entries,
        );

        let mut unbalanced_normal = 0;
        let mut unbalanced_anomaly = 0;
        let mut zero_amount = 0;
        let mut je_date_out_of_range = 0;
        for je in &result.journal_entries {
            let total_debit: Decimal = je.lines.iter().map(|l| l.debit_amount).sum();
            let total_credit: Decimal = je.lines.iter().map(|l| l.credit_amount).sum();
            if total_debit != total_credit {
                if je.header.is_anomaly {
                    unbalanced_anomaly += 1;
                } else {
                    unbalanced_normal += 1;
                }
            }
            if total_debit == Decimal::ZERO && total_credit == Decimal::ZERO {
                zero_amount += 1;
            }
            if !date_in_range(je.header.posting_date, 60) {
                je_date_out_of_range += 1;
            }
        }
        if unbalanced_normal > 0 {
            report.error(
                "journal_entries",
                format!("{} non-anomaly JEs are UNBALANCED", unbalanced_normal),
            );
        }
        if unbalanced_anomaly > 0 {
            // ReversedAmount anomaly injection intentionally creates unbalanced JEs
            report.info(
                "journal_entries",
                format!(
                    "{} anomaly JEs are unbalanced (expected from ReversedAmount)",
                    unbalanced_anomaly
                ),
            );
        }
        if zero_amount > 0 {
            report.warn(
                "journal_entries",
                format!("{} JEs have zero total amounts", zero_amount),
            );
        }
        if je_date_out_of_range > 0 {
            report.warn(
                "journal_entries",
                format!(
                    "{}/{} JEs have dates outside expected range",
                    je_date_out_of_range,
                    result.journal_entries.len()
                ),
            );
        }
        report.info(
            "journal_entries",
            format!("total JEs: {}", result.journal_entries.len()),
        );

        // ================================================================
        // 4. SUBLEDGER
        // ================================================================
        report.assert_non_empty("subledger", "ap_invoices", &result.subledger.ap_invoices);
        report.assert_non_empty("subledger", "ar_invoices", &result.subledger.ar_invoices);

        // AP invoices should reference valid vendors
        let mut ap_vendor_missing = 0;
        for inv in &result.subledger.ap_invoices {
            if !vendor_ids.contains(&inv.vendor_id) {
                ap_vendor_missing += 1;
            }
        }
        if ap_vendor_missing > 0 {
            report.error(
                "subledger",
                format!(
                    "{} AP invoices reference vendor IDs not in master data",
                    ap_vendor_missing
                ),
            );
        }

        // AR invoices should reference valid customers
        let mut ar_customer_missing = 0;
        for inv in &result.subledger.ar_invoices {
            if !customer_ids.contains(&inv.customer_id) {
                ar_customer_missing += 1;
            }
        }
        if ar_customer_missing > 0 {
            report.error(
                "subledger",
                format!(
                    "{} AR invoices reference customer IDs not in master data",
                    ar_customer_missing
                ),
            );
        }

        // ================================================================
        // 5. HR DATA
        // ================================================================
        report.assert_non_empty("hr", "payroll_runs", &result.hr.payroll_runs);
        report.assert_non_empty("hr", "payroll_line_items", &result.hr.payroll_line_items);
        report.assert_non_empty("hr", "time_entries", &result.hr.time_entries);
        report.assert_non_empty("hr", "expense_reports", &result.hr.expense_reports);
        report.assert_non_empty("hr", "benefit_enrollments", &result.hr.benefit_enrollments);

        // Payroll: net_pay should be <= gross_pay
        let mut payroll_net_gt_gross = 0;
        for item in &result.hr.payroll_line_items {
            if item.net_pay > item.gross_pay {
                payroll_net_gt_gross += 1;
            }
        }
        if payroll_net_gt_gross > 0 {
            report.error(
                "hr",
                format!(
                    "{} payroll line items have net_pay > gross_pay",
                    payroll_net_gt_gross
                ),
            );
        }

        // Payroll employee_ids should reference valid employees
        let mut payroll_emp_missing = 0;
        for item in &result.hr.payroll_line_items {
            if !employee_ids.contains(&item.employee_id) {
                payroll_emp_missing += 1;
            }
        }
        if payroll_emp_missing > 0 {
            report.error(
                "hr",
                format!(
                    "{} payroll items reference employee IDs not in master data",
                    payroll_emp_missing
                ),
            );
        }

        // Time entries should reference valid employees
        let mut time_emp_missing = 0;
        for entry in &result.hr.time_entries {
            if !employee_ids.contains(&entry.employee_id) {
                time_emp_missing += 1;
            }
        }
        if time_emp_missing > 0 {
            report.error(
                "hr",
                format!(
                    "{} time entries reference employee IDs not in master data",
                    time_emp_missing
                ),
            );
        }

        // Expense reports should reference valid employees
        let mut expense_emp_missing = 0;
        for er in &result.hr.expense_reports {
            if !employee_ids.contains(&er.employee_id) {
                expense_emp_missing += 1;
            }
        }
        if expense_emp_missing > 0 {
            report.error(
                "hr",
                format!(
                    "{} expense reports reference employee IDs not in master data",
                    expense_emp_missing
                ),
            );
        }

        // Expense report totals should be positive
        let mut expense_zero_total = 0;
        for er in &result.hr.expense_reports {
            if er.total_amount <= Decimal::ZERO {
                expense_zero_total += 1;
            }
        }
        if expense_zero_total > 0 {
            report.warn(
                "hr",
                format!(
                    "{} expense reports have zero/negative total",
                    expense_zero_total
                ),
            );
        }

        // ================================================================
        // 6. MANUFACTURING
        // ================================================================
        report.assert_non_empty(
            "manufacturing",
            "production_orders",
            &result.manufacturing.production_orders,
        );
        report.assert_non_empty(
            "manufacturing",
            "quality_inspections",
            &result.manufacturing.quality_inspections,
        );
        report.assert_non_empty(
            "manufacturing",
            "cycle_counts",
            &result.manufacturing.cycle_counts,
        );
        report.assert_non_empty(
            "manufacturing",
            "bom_components",
            &result.manufacturing.bom_components,
        );
        report.assert_non_empty(
            "manufacturing",
            "inventory_movements",
            &result.manufacturing.inventory_movements,
        );

        // Production orders should reference valid materials
        let mut prod_mat_missing = 0;
        for po in &result.manufacturing.production_orders {
            if !material_ids.contains(&po.material_id) {
                prod_mat_missing += 1;
            }
        }
        if prod_mat_missing > 0 {
            report.error(
                "manufacturing",
                format!(
                    "{} production orders reference material IDs not in master data",
                    prod_mat_missing
                ),
            );
        }

        // Production order yield_rate should be between 0 and ~1.02
        // (slight overproduction up to ~2% is valid in manufacturing)
        for po in &result.manufacturing.production_orders {
            if po.yield_rate < 0.0 || po.yield_rate > 1.05 {
                report.error(
                    "manufacturing",
                    format!(
                        "Production order {} has invalid yield_rate: {}",
                        po.order_id, po.yield_rate
                    ),
                );
            }
        }

        // ================================================================
        // 7. SOURCING
        // ================================================================
        report.assert_non_empty(
            "sourcing",
            "sourcing_projects",
            &result.sourcing.sourcing_projects,
        );
        report.assert_non_empty("sourcing", "rfx_events", &result.sourcing.rfx_events);
        report.assert_non_empty("sourcing", "bids", &result.sourcing.bids);
        report.assert_non_empty("sourcing", "contracts", &result.sourcing.contracts);
        report.assert_non_empty("sourcing", "scorecards", &result.sourcing.scorecards);

        // ================================================================
        // 8. FINANCIAL REPORTING
        // ================================================================
        report.assert_non_empty(
            "fin_reporting",
            "financial_statements",
            &result.financial_reporting.financial_statements,
        );
        report.assert_non_empty(
            "fin_reporting",
            "trial_balances",
            &result.financial_reporting.trial_balances,
        );
        report.assert_non_empty(
            "fin_reporting",
            "bank_reconciliations",
            &result.financial_reporting.bank_reconciliations,
        );

        // Financial statements should have line items
        // CashFlowStatement uses cash_flow_items instead of line_items
        for stmt in &result.financial_reporting.financial_statements {
            use datasynth_core::models::StatementType;
            let has_content = match stmt.statement_type {
                StatementType::CashFlowStatement => !stmt.cash_flow_items.is_empty(),
                _ => !stmt.line_items.is_empty(),
            };
            if !has_content {
                report.warn(
                    "fin_reporting",
                    format!(
                        "Financial statement {} ({:?}) has no line items",
                        stmt.statement_id, stmt.statement_type
                    ),
                );
            }
        }

        // Trial balances: total debits should equal total credits
        for tb in &result.financial_reporting.trial_balances {
            let total_debit: Decimal = tb.entries.iter().map(|e| e.debit_balance).sum();
            let total_credit: Decimal = tb.entries.iter().map(|e| e.credit_balance).sum();
            if total_debit != total_credit {
                report.warn(
                    "fin_reporting",
                    format!(
                        "Trial balance FY{}/P{} is unbalanced: debits={}, credits={}",
                        tb.fiscal_year, tb.fiscal_period, total_debit, total_credit
                    ),
                );
            }
        }

        // ================================================================
        // 9. SALES, KPIS, BUDGETS
        // ================================================================
        report.assert_non_empty(
            "sales_kpi",
            "sales_quotes",
            &result.sales_kpi_budgets.sales_quotes,
        );
        report.assert_non_empty("sales_kpi", "kpis", &result.sales_kpi_budgets.kpis);
        report.assert_non_empty("sales_kpi", "budgets", &result.sales_kpi_budgets.budgets);

        // Sales quotes should reference valid customers
        let mut quote_cust_missing = 0;
        for q in &result.sales_kpi_budgets.sales_quotes {
            if !customer_ids.contains(&q.customer_id) {
                quote_cust_missing += 1;
            }
        }
        if quote_cust_missing > 0 {
            report.error(
                "sales_kpi",
                format!(
                    "{} sales quotes reference customer IDs not in master data",
                    quote_cust_missing
                ),
            );
        }

        // Budget variance = actual_amount - budget_amount for each line item
        let mut budget_variance_mismatch = 0;
        for budget in &result.sales_kpi_budgets.budgets {
            for line in &budget.line_items {
                let expected_variance = line.actual_amount - line.budget_amount;
                if (line.variance - expected_variance).abs() > Decimal::new(1, 2) {
                    budget_variance_mismatch += 1;
                }
            }
        }
        if budget_variance_mismatch > 0 {
            report.error(
                "sales_kpi",
                format!(
                    "{} budget line items have variance != actual - budget",
                    budget_variance_mismatch
                ),
            );
        }

        // ================================================================
        // 10. TAX
        // ================================================================
        report.assert_non_empty("tax", "jurisdictions", &result.tax.jurisdictions);
        report.assert_non_empty("tax", "codes", &result.tax.codes);
        // tax_lines depend on tax transactions being generated (config-dependent)
        if result.tax.tax_lines.is_empty() {
            report.info(
                "tax",
                "tax_lines empty (requires tax transaction generation config)",
            );
        } else {
            report.info(
                "tax",
                format!("tax_lines: {} items", result.tax.tax_lines.len()),
            );
        }

        // Tax codes should reference valid jurisdictions
        let jurisdiction_ids: HashSet<_> = result.tax.jurisdictions.iter().map(|j| &j.id).collect();
        let mut tax_code_jur_missing = 0;
        for tc in &result.tax.codes {
            if !jurisdiction_ids.contains(&tc.jurisdiction_id) {
                tax_code_jur_missing += 1;
            }
        }
        if tax_code_jur_missing > 0 {
            report.error(
                "tax",
                format!(
                    "{} tax codes reference jurisdiction IDs not in tax jurisdictions",
                    tax_code_jur_missing
                ),
            );
        }

        // Tax rates should be in [0, 1]
        for tc in &result.tax.codes {
            if tc.rate < Decimal::ZERO || tc.rate > Decimal::ONE {
                report.error(
                    "tax",
                    format!("Tax code {} has rate outside [0, 1]: {}", tc.code, tc.rate),
                );
            }
        }

        // ================================================================
        // 11. ESG
        // ================================================================
        report.assert_non_empty("esg", "emissions", &result.esg.emissions);
        report.assert_non_empty("esg", "energy", &result.esg.energy);
        report.assert_non_empty("esg", "governance", &result.esg.governance);

        // Emissions should have positive co2e_tonnes
        let mut neg_emissions = 0;
        for e in &result.esg.emissions {
            if e.co2e_tonnes < Decimal::ZERO {
                neg_emissions += 1;
            }
        }
        if neg_emissions > 0 {
            report.error(
                "esg",
                format!(
                    "{} emission records have negative co2e_tonnes",
                    neg_emissions
                ),
            );
        }

        // ================================================================
        // 12. TREASURY
        // ================================================================
        report.assert_non_empty(
            "treasury",
            "cash_positions",
            &result.treasury.cash_positions,
        );
        // debt_instruments and hedging_instruments require specific treasury sub-configs
        if result.treasury.debt_instruments.is_empty() {
            report.info(
                "treasury",
                "debt_instruments empty (requires debt_management config)",
            );
        } else {
            report.info(
                "treasury",
                format!(
                    "debt_instruments: {} items",
                    result.treasury.debt_instruments.len()
                ),
            );
        }
        if result.treasury.hedging_instruments.is_empty() {
            report.info(
                "treasury",
                "hedging_instruments empty (requires hedging config)",
            );
        } else {
            report.info(
                "treasury",
                format!(
                    "hedging_instruments: {} items",
                    result.treasury.hedging_instruments.len()
                ),
            );
        }

        // ================================================================
        // 13. PROJECT ACCOUNTING
        // ================================================================
        report.assert_non_empty("project", "projects", &result.project_accounting.projects);
        report.assert_non_empty(
            "project",
            "cost_lines",
            &result.project_accounting.cost_lines,
        );

        // Project budget should be positive
        for p in &result.project_accounting.projects {
            if p.budget <= Decimal::ZERO {
                report.warn(
                    "project",
                    format!(
                        "Project {} has zero/negative budget: {}",
                        p.project_id, p.budget
                    ),
                );
            }
        }

        // ================================================================
        // 14. INTERCOMPANY
        // ================================================================
        report.assert_non_empty(
            "intercompany",
            "matched_pairs",
            &result.intercompany.matched_pairs,
        );
        report.assert_non_empty(
            "intercompany",
            "seller_journal_entries",
            &result.intercompany.seller_journal_entries,
        );
        report.assert_non_empty(
            "intercompany",
            "buyer_journal_entries",
            &result.intercompany.buyer_journal_entries,
        );

        // IC match rate should be > 0
        if result.intercompany.match_rate <= 0.0 {
            report.warn(
                "intercompany",
                format!("Match rate is {}", result.intercompany.match_rate),
            );
        }

        // IC seller JEs should be balanced
        let mut ic_seller_unbalanced = 0;
        for je in &result.intercompany.seller_journal_entries {
            let dr: Decimal = je.lines.iter().map(|l| l.debit_amount).sum();
            let cr: Decimal = je.lines.iter().map(|l| l.credit_amount).sum();
            if dr != cr {
                ic_seller_unbalanced += 1;
            }
        }
        if ic_seller_unbalanced > 0 {
            report.error(
                "intercompany",
                format!("{} IC seller JEs are UNBALANCED", ic_seller_unbalanced),
            );
        }

        // IC buyer JEs should be balanced
        let mut ic_buyer_unbalanced = 0;
        for je in &result.intercompany.buyer_journal_entries {
            let dr: Decimal = je.lines.iter().map(|l| l.debit_amount).sum();
            let cr: Decimal = je.lines.iter().map(|l| l.credit_amount).sum();
            if dr != cr {
                ic_buyer_unbalanced += 1;
            }
        }
        if ic_buyer_unbalanced > 0 {
            report.error(
                "intercompany",
                format!("{} IC buyer JEs are UNBALANCED", ic_buyer_unbalanced),
            );
        }

        // IC matched pairs: amounts should be positive
        let mut ic_zero_amount = 0;
        for pair in &result.intercompany.matched_pairs {
            if pair.amount <= Decimal::ZERO {
                ic_zero_amount += 1;
            }
        }
        if ic_zero_amount > 0 {
            report.error(
                "intercompany",
                format!(
                    "{} IC matched pairs have zero/negative amount",
                    ic_zero_amount
                ),
            );
        }

        // ================================================================
        // 15. AUDIT
        // ================================================================
        report.assert_non_empty("audit", "engagements", &result.audit.engagements);
        report.assert_non_empty("audit", "workpapers", &result.audit.workpapers);
        report.assert_non_empty("audit", "evidence", &result.audit.evidence);
        report.assert_non_empty("audit", "risk_assessments", &result.audit.risk_assessments);

        // Audit engagement dates should be ordered: planning <= fieldwork_start <= fieldwork_end
        for eng in &result.audit.engagements {
            if eng.planning_start > eng.fieldwork_start {
                report.error(
                    "audit",
                    format!(
                        "Engagement {} has planning_start ({}) > fieldwork_start ({})",
                        eng.engagement_ref, eng.planning_start, eng.fieldwork_start
                    ),
                );
            }
            if eng.fieldwork_start > eng.fieldwork_end {
                report.error(
                    "audit",
                    format!(
                        "Engagement {} has fieldwork_start ({}) > fieldwork_end ({})",
                        eng.engagement_ref, eng.fieldwork_start, eng.fieldwork_end
                    ),
                );
            }
        }

        // Materiality should be positive and > performance_materiality > clearly_trivial
        for eng in &result.audit.engagements {
            if eng.materiality <= Decimal::ZERO {
                report.error(
                    "audit",
                    format!(
                        "Engagement {} has non-positive materiality",
                        eng.engagement_ref
                    ),
                );
            }
            if eng.performance_materiality >= eng.materiality {
                report.error(
                    "audit",
                    format!(
                        "Engagement {} has performance_materiality ({}) >= materiality ({})",
                        eng.engagement_ref, eng.performance_materiality, eng.materiality
                    ),
                );
            }
            if eng.clearly_trivial >= eng.performance_materiality {
                report.error(
                    "audit",
                    format!(
                        "Engagement {} has clearly_trivial ({}) >= performance_materiality ({})",
                        eng.engagement_ref, eng.clearly_trivial, eng.performance_materiality
                    ),
                );
            }
        }

        // ================================================================
        // 16. BANKING
        // ================================================================
        report.assert_non_empty("banking", "customers", &result.banking.customers);
        report.assert_non_empty("banking", "accounts", &result.banking.accounts);
        report.assert_non_empty("banking", "transactions", &result.banking.transactions);

        // ================================================================
        // 17. ACCOUNTING STANDARDS
        // ================================================================
        report.assert_non_empty(
            "acct_standards",
            "contracts",
            &result.accounting_standards.contracts,
        );
        report.assert_non_empty(
            "acct_standards",
            "impairment_tests",
            &result.accounting_standards.impairment_tests,
        );

        // Contracts should reference valid customers
        let mut contract_cust_missing = 0;
        for c in &result.accounting_standards.contracts {
            if !customer_ids.contains(&c.customer_id) {
                contract_cust_missing += 1;
            }
        }
        if contract_cust_missing > 0 {
            report.error(
                "acct_standards",
                format!(
                    "{} contracts reference customer IDs not in master data",
                    contract_cust_missing
                ),
            );
        }

        // ================================================================
        // 18. INTERNAL CONTROLS
        // ================================================================
        report.assert_non_empty("controls", "internal_controls", &result.internal_controls);

        // Controls should have unique IDs
        let control_ids: HashSet<_> = result
            .internal_controls
            .iter()
            .map(|c| &c.control_id)
            .collect();
        if control_ids.len() != result.internal_controls.len() {
            report.error(
                "controls",
                format!(
                    "Duplicate control IDs: {} unique out of {}",
                    control_ids.len(),
                    result.internal_controls.len()
                ),
            );
        }

        // ================================================================
        // 19. ANOMALY LABELS
        // ================================================================
        report.assert_non_empty("anomalies", "labels", &result.anomaly_labels.labels);

        // Anomaly confidence scores should be in [0, 1]
        let mut bad_confidence = 0;
        for label in &result.anomaly_labels.labels {
            if label.confidence < 0.0 || label.confidence > 1.0 {
                bad_confidence += 1;
            }
        }
        if bad_confidence > 0 {
            report.error(
                "anomalies",
                format!(
                    "{} anomaly labels have confidence outside [0, 1]",
                    bad_confidence
                ),
            );
        }

        // ================================================================
        // 20. BALANCE VALIDATION
        // ================================================================
        if !result.balance_validation.validated {
            report.error("balance", "Balance validation did not run");
        } else if !result.balance_validation.is_balanced {
            // Anomaly injection (ReversedAmount) intentionally creates unbalanced JEs,
            // which causes overall balance validation to fail. This is expected.
            report.warn(
                "balance",
                format!(
                    "Unbalanced (expected with anomaly injection): debits={}, credits={}, errors={}",
                    result.balance_validation.total_debits,
                    result.balance_validation.total_credits,
                    result.balance_validation.validation_errors.len()
                ),
            );
        } else {
            report.info(
                "balance",
                format!(
                    "Balanced: debits=credits={}, {} accounts, {} companies",
                    result.balance_validation.total_debits,
                    result.balance_validation.accounts_tracked,
                    result.balance_validation.companies_tracked
                ),
            );
        }

        // ================================================================
        // 21. OPENING BALANCES
        // ================================================================
        // Opening balances require balance.generate_opening_balances config
        if result.opening_balances.is_empty() {
            report.info(
                "opening_balances",
                "opening_balances empty (requires balance config)",
            );
        } else {
            report.info(
                "opening_balances",
                format!("opening_balances: {} items", result.opening_balances.len()),
            );
        }

        // ================================================================
        // 22. PROCESS EVOLUTION & DISRUPTION EVENTS
        // ================================================================
        report.assert_non_empty("evolution", "process_evolution", &result.process_evolution);
        report.assert_non_empty(
            "evolution",
            "organizational_events",
            &result.organizational_events,
        );
        report.assert_non_empty("evolution", "disruption_events", &result.disruption_events);

        // Events should have dates in range
        for evt in &result.process_evolution {
            if !date_in_range(evt.effective_date, 30) {
                report.warn(
                    "evolution",
                    format!(
                        "Process evolution event {} has out-of-range date: {}",
                        evt.event_id, evt.effective_date
                    ),
                );
            }
        }

        // ================================================================
        // 23. COUNTERFACTUAL PAIRS
        // ================================================================
        report.assert_non_empty(
            "counterfactual",
            "counterfactual_pairs",
            &result.counterfactual_pairs,
        );

        // Each pair should differ in either lines or header fields
        let mut identical_pairs = 0;
        for pair in &result.counterfactual_pairs {
            let lines_same =
                format!("{:?}", pair.original.lines) == format!("{:?}", pair.modified.lines);
            let header_same = pair.original.header.posting_date
                == pair.modified.header.posting_date
                && pair.original.header.sod_violation == pair.modified.header.sod_violation
                && pair.original.header.document_date == pair.modified.header.document_date;
            if lines_same && header_same {
                identical_pairs += 1;
            }
        }
        if identical_pairs > 0 {
            report.error(
                "counterfactual",
                format!(
                    "{}/{} pairs have truly identical original and modified (lines + headers)",
                    identical_pairs,
                    result.counterfactual_pairs.len()
                ),
            );
        }

        // ================================================================
        // 24. RED FLAGS
        // ================================================================
        report.assert_non_empty("red_flags", "red_flags", &result.red_flags);

        // Red flag confidence should be in [0, 1]
        let mut rf_bad_conf = 0;
        for rf in &result.red_flags {
            if rf.confidence < 0.0 || rf.confidence > 1.0 {
                rf_bad_conf += 1;
            }
        }
        if rf_bad_conf > 0 {
            report.error(
                "red_flags",
                format!("{} red flags have confidence outside [0, 1]", rf_bad_conf),
            );
        }

        // Mix of fraudulent and non-fraudulent flags
        let fraud_flags = result
            .red_flags
            .iter()
            .filter(|rf| rf.is_fraudulent)
            .count();
        let non_fraud_flags = result
            .red_flags
            .iter()
            .filter(|rf| !rf.is_fraudulent)
            .count();
        report.info(
            "red_flags",
            format!(
                "fraudulent: {}, non-fraudulent: {}",
                fraud_flags, non_fraud_flags
            ),
        );

        // ================================================================
        // 25. COLLUSION RINGS
        // ================================================================
        report.assert_non_empty("collusion", "collusion_rings", &result.collusion_rings);

        // Each ring should have members and transactions
        for ring in &result.collusion_rings {
            if ring.members.is_empty() {
                report.error("collusion", format!("Ring {} has no members", ring.ring_id));
            }
            if ring.total_stolen == Decimal::ZERO {
                report.warn(
                    "collusion",
                    format!("Ring {} has total_stolen=0", ring.ring_id),
                );
            }
        }

        // ================================================================
        // 26. TEMPORAL VERSION CHAINS
        // ================================================================
        report.assert_non_empty(
            "temporal",
            "temporal_vendor_chains",
            &result.temporal_vendor_chains,
        );

        // Chains should have multiple versions
        let mut single_version_chains = 0;
        let mut total_versions = 0;
        for chain in &result.temporal_vendor_chains {
            let version_count = chain.versions.len();
            total_versions += version_count;
            if version_count <= 1 {
                single_version_chains += 1;
            }
        }
        if !result.temporal_vendor_chains.is_empty() {
            let avg_versions = total_versions as f64 / result.temporal_vendor_chains.len() as f64;
            report.info(
                "temporal",
                format!(
                    "chains: {}, avg versions: {:.1}, single-version: {}",
                    result.temporal_vendor_chains.len(),
                    avg_versions,
                    single_version_chains
                ),
            );
            if avg_versions < 1.5 {
                report.warn(
                    "temporal",
                    format!(
                        "Average versions ({:.1}) is low, expected >= 1.5",
                        avg_versions
                    ),
                );
            }
        }

        // ================================================================
        // 27. ENTITY RELATIONSHIP GRAPH
        // ================================================================
        if let Some(ref graph) = result.entity_relationship_graph {
            report.info(
                "entity_graph",
                format!("nodes: {}, edges: {}", graph.nodes.len(), graph.edges.len()),
            );

            // Edges should have strength in (0, 1]
            let mut bad_strength = 0;
            let mut default_strength = 0;
            let unique_strengths: HashSet<String> = graph
                .edges
                .iter()
                .map(|e| format!("{:.3}", e.strength))
                .collect();
            for edge in &graph.edges {
                if edge.strength <= 0.0 || edge.strength > 1.0 {
                    bad_strength += 1;
                }
                if (edge.strength - 0.5).abs() < 0.001 {
                    default_strength += 1;
                }
            }
            if bad_strength > 0 {
                report.error(
                    "entity_graph",
                    format!("{} edges have strength outside (0, 1]", bad_strength),
                );
            }
            if default_strength > 0 && default_strength == graph.edges.len() {
                report.error(
                    "entity_graph",
                    "All edge strengths are default 0.500 - no computed strengths",
                );
            }
            report.info(
                "entity_graph",
                format!("unique strength values: {}", unique_strengths.len()),
            );
        } else {
            report.error(
                "entity_graph",
                "entity_relationship_graph is None (expected Some)",
            );
        }

        // ================================================================
        // 28. CROSS-PROCESS LINKS
        // ================================================================
        report.assert_non_empty(
            "cross_process",
            "cross_process_links",
            &result.cross_process_links,
        );

        // ================================================================
        // 29. INDUSTRY OUTPUT
        // ================================================================
        if let Some(ref industry) = result.industry_output {
            report.info(
                "industry",
                format!(
                    "industry: {}, gl_accounts: {}",
                    industry.industry,
                    industry.gl_accounts.len()
                ),
            );
            if industry.gl_accounts.is_empty() {
                report.warn("industry", "No industry GL accounts generated");
            }
        } else {
            report.error("industry", "industry_output is None (expected Some)");
        }

        // ================================================================
        // 30. SUBLEDGER RECONCILIATION
        // ================================================================
        report.assert_non_empty(
            "subledger_recon",
            "subledger_reconciliation",
            &result.subledger_reconciliation,
        );

        // ================================================================
        // 31. SUBLEDGER DEEP VALIDATION (Tier 1)
        // ================================================================

        // FA records
        if !result.subledger.fa_records.is_empty() {
            report.info(
                "subledger",
                format!("fa_records: {} items", result.subledger.fa_records.len()),
            );

            let mut fa_neg_cost = 0;
            let mut fa_neg_nbv = 0;
            let mut fa_date_order = 0;
            for fa in &result.subledger.fa_records {
                if fa.acquisition_cost < Decimal::ZERO {
                    fa_neg_cost += 1;
                }
                if fa.net_book_value < Decimal::ZERO {
                    fa_neg_nbv += 1;
                }
                if fa.capitalization_date < fa.acquisition_date {
                    fa_date_order += 1;
                }
            }
            if fa_neg_cost > 0 {
                report.error(
                    "subledger",
                    format!("{} FA records have negative acquisition_cost", fa_neg_cost),
                );
            }
            if fa_neg_nbv > 0 {
                report.error(
                    "subledger",
                    format!("{} FA records have negative net_book_value", fa_neg_nbv),
                );
            }
            if fa_date_order > 0 {
                report.warn(
                    "subledger",
                    format!(
                        "{} FA records have capitalization_date < acquisition_date",
                        fa_date_order
                    ),
                );
            }
        } else {
            report.info("subledger", "fa_records empty (depends on FA generation)");
        }

        // Inventory positions
        if !result.subledger.inventory_positions.is_empty() {
            report.info(
                "subledger",
                format!(
                    "inventory_positions: {} items",
                    result.subledger.inventory_positions.len()
                ),
            );

            let mut inv_neg_qty = 0;
            let mut inv_mat_missing = 0;
            for pos in &result.subledger.inventory_positions {
                if pos.quantity_on_hand < Decimal::ZERO {
                    inv_neg_qty += 1;
                }
                if !material_ids.contains(&pos.material_id) {
                    inv_mat_missing += 1;
                }
            }
            if inv_neg_qty > 0 {
                report.error(
                    "subledger",
                    format!(
                        "{} inventory positions have negative quantity_on_hand",
                        inv_neg_qty
                    ),
                );
            }
            if inv_mat_missing > 0 {
                report.error(
                    "subledger",
                    format!(
                        "{} inventory positions reference material IDs not in master data",
                        inv_mat_missing
                    ),
                );
            }
        } else {
            report.info(
                "subledger",
                "inventory_positions empty (depends on inventory generation)",
            );
        }

        // Inventory movements (subledger)
        if !result.subledger.inventory_movements.is_empty() {
            report.info(
                "subledger",
                format!(
                    "inventory_movements: {} items",
                    result.subledger.inventory_movements.len()
                ),
            );
        }

        // ================================================================
        // 32. SOURCING CHAIN INTEGRITY (Tier 1)
        // ================================================================

        // Bids should reference valid rfx_ids
        let rfx_ids: HashSet<_> = result
            .sourcing
            .rfx_events
            .iter()
            .map(|r| &r.rfx_id)
            .collect();
        let mut bid_rfx_missing = 0;
        let mut bid_vendor_missing = 0;
        let mut bid_neg_amount = 0;
        for bid in &result.sourcing.bids {
            if !rfx_ids.contains(&bid.rfx_id) {
                bid_rfx_missing += 1;
            }
            if !vendor_ids.contains(&bid.vendor_id) {
                bid_vendor_missing += 1;
            }
            if bid.total_amount <= Decimal::ZERO {
                bid_neg_amount += 1;
            }
        }
        if bid_rfx_missing > 0 {
            report.error(
                "sourcing",
                format!(
                    "{} bids reference rfx_ids not in rfx_events",
                    bid_rfx_missing
                ),
            );
        }
        if bid_vendor_missing > 0 {
            report.error(
                "sourcing",
                format!(
                    "{} bids reference vendor_ids not in master data",
                    bid_vendor_missing
                ),
            );
        }
        if bid_neg_amount > 0 {
            report.error(
                "sourcing",
                format!("{} bids have zero/negative total_amount", bid_neg_amount),
            );
        }

        // Contracts should reference valid vendor_ids
        let mut contract_vendor_missing = 0;
        for c in &result.sourcing.contracts {
            if !vendor_ids.contains(&c.vendor_id) {
                contract_vendor_missing += 1;
            }
        }
        if contract_vendor_missing > 0 {
            report.error(
                "sourcing",
                format!(
                    "{} contracts reference vendor_ids not in master data",
                    contract_vendor_missing
                ),
            );
        }

        // ================================================================
        // 33. HR DEEP CHECKS (Tier 2)
        // ================================================================

        // Expense line items: amounts should be positive
        let mut expense_neg_items = 0;
        let mut expense_total_items = 0;
        for er in &result.hr.expense_reports {
            for item in er.line_items.iter() {
                expense_total_items += 1;
                if item.amount <= Decimal::ZERO {
                    expense_neg_items += 1;
                }
            }
        }
        if expense_neg_items > 0 {
            report.error(
                "hr",
                format!(
                    "{}/{} expense line items have zero/negative amount",
                    expense_neg_items, expense_total_items
                ),
            );
        }

        // Benefit enrollments: employee references and contributions
        let mut benefit_emp_missing = 0;
        let mut benefit_neg_contrib = 0;
        for be in &result.hr.benefit_enrollments {
            if !employee_ids.contains(&be.employee_id) {
                benefit_emp_missing += 1;
            }
            if be.employee_contribution < Decimal::ZERO || be.employer_contribution < Decimal::ZERO
            {
                benefit_neg_contrib += 1;
            }
        }
        if benefit_emp_missing > 0 {
            report.error(
                "hr",
                format!(
                    "{} benefit enrollments reference employee IDs not in master data",
                    benefit_emp_missing
                ),
            );
        }
        if benefit_neg_contrib > 0 {
            report.error(
                "hr",
                format!(
                    "{} benefit enrollments have negative contributions",
                    benefit_neg_contrib
                ),
            );
        }

        // Time entries: total hours should be > 0
        let mut time_zero_hours = 0;
        let mut time_neg_hours = 0;
        for te in &result.hr.time_entries {
            let total = te.hours_regular + te.hours_overtime + te.hours_pto + te.hours_sick;
            if total == 0.0 {
                time_zero_hours += 1;
            }
            if te.hours_regular < 0.0
                || te.hours_overtime < 0.0
                || te.hours_pto < 0.0
                || te.hours_sick < 0.0
            {
                time_neg_hours += 1;
            }
        }
        if time_neg_hours > 0 {
            report.error(
                "hr",
                format!("{} time entries have negative hour values", time_neg_hours),
            );
        }
        if time_zero_hours > 0 {
            report.warn(
                "hr",
                format!("{} time entries have zero total hours", time_zero_hours),
            );
        }

        // ================================================================
        // 34. MANUFACTURING DEEP CHECKS (Tier 2)
        // ================================================================

        // Production order date ordering
        let mut mfg_date_order = 0;
        let mut mfg_actual_date_order = 0;
        let mut mfg_neg_cost = 0;
        let mut mfg_neg_scrap = 0;
        for po in &result.manufacturing.production_orders {
            if po.planned_start > po.planned_end {
                mfg_date_order += 1;
            }
            if let (Some(actual_start), Some(actual_end)) = (po.actual_start, po.actual_end) {
                if actual_start > actual_end {
                    mfg_actual_date_order += 1;
                }
            }
            if po.planned_cost < Decimal::ZERO || po.actual_cost < Decimal::ZERO {
                mfg_neg_cost += 1;
            }
            if po.scrap_quantity < Decimal::ZERO {
                mfg_neg_scrap += 1;
            }
        }
        if mfg_date_order > 0 {
            report.error(
                "manufacturing",
                format!(
                    "{} production orders have planned_start > planned_end",
                    mfg_date_order
                ),
            );
        }
        if mfg_actual_date_order > 0 {
            report.error(
                "manufacturing",
                format!(
                    "{} production orders have actual_start > actual_end",
                    mfg_actual_date_order
                ),
            );
        }
        if mfg_neg_cost > 0 {
            report.error(
                "manufacturing",
                format!(
                    "{} production orders have negative planned/actual cost",
                    mfg_neg_cost
                ),
            );
        }
        if mfg_neg_scrap > 0 {
            report.error(
                "manufacturing",
                format!(
                    "{} production orders have negative scrap_quantity",
                    mfg_neg_scrap
                ),
            );
        }

        // BOM components should reference valid materials
        let mut bom_mat_missing = 0;
        for bom in &result.manufacturing.bom_components {
            if !material_ids.contains(&bom.component_material_id) {
                bom_mat_missing += 1;
            }
        }
        if bom_mat_missing > 0 {
            report.error(
                "manufacturing",
                format!(
                    "{} BOM components reference material IDs not in master data",
                    bom_mat_missing
                ),
            );
        }

        // ================================================================
        // 35. PROJECT ACCOUNTING DEEP CHECKS (Tier 2)
        // ================================================================

        // Cost lines should reference valid project IDs
        let project_ids: HashSet<_> = result
            .project_accounting
            .projects
            .iter()
            .map(|p| &p.project_id)
            .collect();
        let mut cost_proj_missing = 0;
        let mut cost_neg_amount = 0;
        for cl in &result.project_accounting.cost_lines {
            if !project_ids.contains(&cl.project_id) {
                cost_proj_missing += 1;
            }
            if cl.amount < Decimal::ZERO {
                cost_neg_amount += 1;
            }
        }
        if cost_proj_missing > 0 {
            report.error(
                "project",
                format!(
                    "{} cost lines reference project IDs not in projects",
                    cost_proj_missing
                ),
            );
        }
        if cost_neg_amount > 0 {
            report.error(
                "project",
                format!("{} cost lines have negative amount", cost_neg_amount),
            );
        }

        // Revenue records
        if !result.project_accounting.revenue_records.is_empty() {
            report.info(
                "project",
                format!(
                    "revenue_records: {} items",
                    result.project_accounting.revenue_records.len()
                ),
            );
        } else {
            report.info(
                "project",
                "revenue_records empty (depends on project config)",
            );
        }

        // Earned value metrics
        if !result.project_accounting.earned_value_metrics.is_empty() {
            report.info(
                "project",
                format!(
                    "earned_value_metrics: {} items",
                    result.project_accounting.earned_value_metrics.len()
                ),
            );

            // EVM: SV = EV - PV, CV = EV - AC
            let mut evm_sv_mismatch = 0;
            let mut evm_cv_mismatch = 0;
            let tolerance = Decimal::new(1, 2); // 0.01
            for evm in &result.project_accounting.earned_value_metrics {
                let expected_sv = evm.earned_value - evm.planned_value;
                if (evm.schedule_variance - expected_sv).abs() > tolerance {
                    evm_sv_mismatch += 1;
                }
                let expected_cv = evm.earned_value - evm.actual_cost;
                if (evm.cost_variance - expected_cv).abs() > tolerance {
                    evm_cv_mismatch += 1;
                }
            }
            if evm_sv_mismatch > 0 {
                report.error(
                    "project",
                    format!(
                        "{} EVM records have schedule_variance != earned_value - planned_value",
                        evm_sv_mismatch
                    ),
                );
            }
            if evm_cv_mismatch > 0 {
                report.error(
                    "project",
                    format!(
                        "{} EVM records have cost_variance != earned_value - actual_cost",
                        evm_cv_mismatch
                    ),
                );
            }
        } else {
            report.info(
                "project",
                "earned_value_metrics empty (depends on project config)",
            );
        }

        // Change orders
        if !result.project_accounting.change_orders.is_empty() {
            report.info(
                "project",
                format!(
                    "change_orders: {} items",
                    result.project_accounting.change_orders.len()
                ),
            );
        } else {
            report.info("project", "change_orders empty (depends on project config)");
        }

        // Milestones
        if !result.project_accounting.milestones.is_empty() {
            report.info(
                "project",
                format!(
                    "milestones: {} items",
                    result.project_accounting.milestones.len()
                ),
            );
        } else {
            report.info("project", "milestones empty (depends on project config)");
        }

        // ================================================================
        // 36. ESG CONTENT CHECKS (Tier 2)
        // ================================================================

        // Energy consumption should be non-negative
        let mut energy_neg = 0;
        for e in &result.esg.energy {
            if e.consumption_kwh < Decimal::ZERO {
                energy_neg += 1;
            }
        }
        if energy_neg > 0 {
            report.error(
                "esg",
                format!(
                    "{} energy records have negative consumption_kwh",
                    energy_neg
                ),
            );
        }

        // Water/waste if present
        if !result.esg.water.is_empty() {
            report.info("esg", format!("water: {} items", result.esg.water.len()));
        }
        if !result.esg.waste.is_empty() {
            report.info("esg", format!("waste: {} items", result.esg.waste.len()));
        }
        if !result.esg.diversity.is_empty() {
            report.info(
                "esg",
                format!("diversity: {} items", result.esg.diversity.len()),
            );
        }
        if !result.esg.safety_incidents.is_empty() {
            report.info(
                "esg",
                format!(
                    "safety_incidents: {} items",
                    result.esg.safety_incidents.len()
                ),
            );
        }
        if !result.esg.supplier_assessments.is_empty() {
            report.info(
                "esg",
                format!(
                    "supplier_assessments: {} items",
                    result.esg.supplier_assessments.len()
                ),
            );
        }

        // ================================================================
        // 37. TREASURY CONTENT CHECKS (Tier 2)
        // ================================================================

        // Cash positions: closing_balance should equal opening + inflows - outflows
        let mut cash_balance_mismatch = 0;
        let tolerance = Decimal::new(1, 2); // 0.01
        for cp in &result.treasury.cash_positions {
            let expected = cp.opening_balance + cp.inflows - cp.outflows;
            if (cp.closing_balance - expected).abs() > tolerance {
                cash_balance_mismatch += 1;
            }
        }
        if cash_balance_mismatch > 0 {
            report.error(
                "treasury",
                format!(
                    "{} cash positions have closing_balance != opening + inflows - outflows",
                    cash_balance_mismatch
                ),
            );
        }

        // Cash forecasts
        if !result.treasury.cash_forecasts.is_empty() {
            report.info(
                "treasury",
                format!(
                    "cash_forecasts: {} items",
                    result.treasury.cash_forecasts.len()
                ),
            );
        }

        // Cash pools
        if !result.treasury.cash_pools.is_empty() {
            report.info(
                "treasury",
                format!("cash_pools: {} items", result.treasury.cash_pools.len()),
            );
        }

        // ================================================================
        // 38. SNAPSHOT COUNT CONSISTENCY (Tier 1)
        // ================================================================

        // Manufacturing snapshot counts should match Vec lengths
        if result.manufacturing.production_order_count
            != result.manufacturing.production_orders.len()
        {
            report.error(
                "snapshot_counts",
                format!(
                    "manufacturing.production_order_count ({}) != production_orders.len() ({})",
                    result.manufacturing.production_order_count,
                    result.manufacturing.production_orders.len()
                ),
            );
        }
        if result.manufacturing.quality_inspection_count
            != result.manufacturing.quality_inspections.len()
        {
            report.error(
                "snapshot_counts",
                format!(
                    "manufacturing.quality_inspection_count ({}) != quality_inspections.len() ({})",
                    result.manufacturing.quality_inspection_count,
                    result.manufacturing.quality_inspections.len()
                ),
            );
        }
        if result.manufacturing.cycle_count_count != result.manufacturing.cycle_counts.len() {
            report.error(
                "snapshot_counts",
                format!(
                    "manufacturing.cycle_count_count ({}) != cycle_counts.len() ({})",
                    result.manufacturing.cycle_count_count,
                    result.manufacturing.cycle_counts.len()
                ),
            );
        }
        if result.manufacturing.bom_component_count != result.manufacturing.bom_components.len() {
            report.error(
                "snapshot_counts",
                format!(
                    "manufacturing.bom_component_count ({}) != bom_components.len() ({})",
                    result.manufacturing.bom_component_count,
                    result.manufacturing.bom_components.len()
                ),
            );
        }
        if result.manufacturing.inventory_movement_count
            != result.manufacturing.inventory_movements.len()
        {
            report.error(
                "snapshot_counts",
                format!(
                    "manufacturing.inventory_movement_count ({}) != inventory_movements.len() ({})",
                    result.manufacturing.inventory_movement_count,
                    result.manufacturing.inventory_movements.len()
                ),
            );
        }

        // HR snapshot counts
        if result.hr.payroll_run_count != result.hr.payroll_runs.len() {
            report.error(
                "snapshot_counts",
                format!(
                    "hr.payroll_run_count ({}) != payroll_runs.len() ({})",
                    result.hr.payroll_run_count,
                    result.hr.payroll_runs.len()
                ),
            );
        }
        if result.hr.payroll_line_item_count != result.hr.payroll_line_items.len() {
            report.error(
                "snapshot_counts",
                format!(
                    "hr.payroll_line_item_count ({}) != payroll_line_items.len() ({})",
                    result.hr.payroll_line_item_count,
                    result.hr.payroll_line_items.len()
                ),
            );
        }
        if result.hr.time_entry_count != result.hr.time_entries.len() {
            report.error(
                "snapshot_counts",
                format!(
                    "hr.time_entry_count ({}) != time_entries.len() ({})",
                    result.hr.time_entry_count,
                    result.hr.time_entries.len()
                ),
            );
        }
        if result.hr.expense_report_count != result.hr.expense_reports.len() {
            report.error(
                "snapshot_counts",
                format!(
                    "hr.expense_report_count ({}) != expense_reports.len() ({})",
                    result.hr.expense_report_count,
                    result.hr.expense_reports.len()
                ),
            );
        }
        if result.hr.benefit_enrollment_count != result.hr.benefit_enrollments.len() {
            report.error(
                "snapshot_counts",
                format!(
                    "hr.benefit_enrollment_count ({}) != benefit_enrollments.len() ({})",
                    result.hr.benefit_enrollment_count,
                    result.hr.benefit_enrollments.len()
                ),
            );
        }

        // IC snapshot counts
        if result.intercompany.matched_pair_count != result.intercompany.matched_pairs.len() {
            report.error(
                "snapshot_counts",
                format!(
                    "intercompany.matched_pair_count ({}) != matched_pairs.len() ({})",
                    result.intercompany.matched_pair_count,
                    result.intercompany.matched_pairs.len()
                ),
            );
        }
        if result.intercompany.elimination_entry_count
            != result.intercompany.elimination_entries.len()
        {
            report.error(
                "snapshot_counts",
                format!(
                    "intercompany.elimination_entry_count ({}) != elimination_entries.len() ({})",
                    result.intercompany.elimination_entry_count,
                    result.intercompany.elimination_entries.len()
                ),
            );
        }

        // ================================================================
        // 39. EXPANDED STATISTICS CONSISTENCY (Tier 1)
        // ================================================================
        let stats = &result.statistics;

        // Stat counts should match actual data counts
        if stats.vendor_count != result.master_data.vendors.len() {
            report.error(
                "statistics",
                format!(
                    "vendor_count stat ({}) != actual vendors ({})",
                    stats.vendor_count,
                    result.master_data.vendors.len()
                ),
            );
        }
        if stats.customer_count != result.master_data.customers.len() {
            report.error(
                "statistics",
                format!(
                    "customer_count stat ({}) != actual customers ({})",
                    stats.customer_count,
                    result.master_data.customers.len()
                ),
            );
        }
        if stats.p2p_chain_count != result.document_flows.p2p_chains.len() {
            report.error(
                "statistics",
                format!(
                    "p2p_chain_count stat ({}) != actual chains ({})",
                    stats.p2p_chain_count,
                    result.document_flows.p2p_chains.len()
                ),
            );
        }
        if stats.o2c_chain_count != result.document_flows.o2c_chains.len() {
            report.error(
                "statistics",
                format!(
                    "o2c_chain_count stat ({}) != actual chains ({})",
                    stats.o2c_chain_count,
                    result.document_flows.o2c_chains.len()
                ),
            );
        }
        if stats.total_entries as usize != result.journal_entries.len() {
            report.error(
                "statistics",
                format!(
                    "total_entries stat ({}) != actual JEs ({})",
                    stats.total_entries,
                    result.journal_entries.len()
                ),
            );
        }
        if stats.payroll_run_count != result.hr.payroll_runs.len() {
            report.error(
                "statistics",
                format!(
                    "payroll_run_count stat ({}) != actual ({})",
                    stats.payroll_run_count,
                    result.hr.payroll_runs.len()
                ),
            );
        }
        if stats.production_order_count != result.manufacturing.production_orders.len() {
            report.error(
                "statistics",
                format!(
                    "production_order_count stat ({}) != actual ({})",
                    stats.production_order_count,
                    result.manufacturing.production_orders.len()
                ),
            );
        }
        if stats.tax_jurisdiction_count != result.tax.jurisdictions.len() {
            report.error(
                "statistics",
                format!(
                    "tax_jurisdiction_count stat ({}) != actual ({})",
                    stats.tax_jurisdiction_count,
                    result.tax.jurisdictions.len()
                ),
            );
        }
        if stats.financial_statement_count != result.financial_reporting.financial_statements.len()
        {
            report.error(
                "statistics",
                format!(
                    "financial_statement_count stat ({}) != actual ({})",
                    stats.financial_statement_count,
                    result.financial_reporting.financial_statements.len()
                ),
            );
        }
        if stats.project_count != result.project_accounting.projects.len() {
            report.error(
                "statistics",
                format!(
                    "project_count stat ({}) != actual ({})",
                    stats.project_count,
                    result.project_accounting.projects.len()
                ),
            );
        }

        // Additional stat checks
        if stats.material_count != result.master_data.materials.len() {
            report.error(
                "statistics",
                format!(
                    "material_count stat ({}) != actual ({})",
                    stats.material_count,
                    result.master_data.materials.len()
                ),
            );
        }
        if stats.asset_count != result.master_data.assets.len() {
            report.error(
                "statistics",
                format!(
                    "asset_count stat ({}) != actual ({})",
                    stats.asset_count,
                    result.master_data.assets.len()
                ),
            );
        }
        if stats.employee_count != result.master_data.employees.len() {
            report.error(
                "statistics",
                format!(
                    "employee_count stat ({}) != actual ({})",
                    stats.employee_count,
                    result.master_data.employees.len()
                ),
            );
        }
        if stats.ap_invoice_count != result.subledger.ap_invoices.len() {
            report.error(
                "statistics",
                format!(
                    "ap_invoice_count stat ({}) != actual ({})",
                    stats.ap_invoice_count,
                    result.subledger.ap_invoices.len()
                ),
            );
        }
        if stats.ar_invoice_count != result.subledger.ar_invoices.len() {
            report.error(
                "statistics",
                format!(
                    "ar_invoice_count stat ({}) != actual ({})",
                    stats.ar_invoice_count,
                    result.subledger.ar_invoices.len()
                ),
            );
        }
        if stats.sourcing_project_count != result.sourcing.sourcing_projects.len() {
            report.error(
                "statistics",
                format!(
                    "sourcing_project_count stat ({}) != actual ({})",
                    stats.sourcing_project_count,
                    result.sourcing.sourcing_projects.len()
                ),
            );
        }
        if stats.bid_count != result.sourcing.bids.len() {
            report.error(
                "statistics",
                format!(
                    "bid_count stat ({}) != actual ({})",
                    stats.bid_count,
                    result.sourcing.bids.len()
                ),
            );
        }
        if stats.contract_count != result.sourcing.contracts.len() {
            report.error(
                "statistics",
                format!(
                    "contract_count stat ({}) != actual ({})",
                    stats.contract_count,
                    result.sourcing.contracts.len()
                ),
            );
        }
        if stats.time_entry_count != result.hr.time_entries.len() {
            report.error(
                "statistics",
                format!(
                    "time_entry_count stat ({}) != actual ({})",
                    stats.time_entry_count,
                    result.hr.time_entries.len()
                ),
            );
        }
        if stats.expense_report_count != result.hr.expense_reports.len() {
            report.error(
                "statistics",
                format!(
                    "expense_report_count stat ({}) != actual ({})",
                    stats.expense_report_count,
                    result.hr.expense_reports.len()
                ),
            );
        }
        if stats.benefit_enrollment_count != result.hr.benefit_enrollments.len() {
            report.error(
                "statistics",
                format!(
                    "benefit_enrollment_count stat ({}) != actual ({})",
                    stats.benefit_enrollment_count,
                    result.hr.benefit_enrollments.len()
                ),
            );
        }
        if stats.esg_emission_count != result.esg.emissions.len() {
            report.error(
                "statistics",
                format!(
                    "esg_emission_count stat ({}) != actual ({})",
                    stats.esg_emission_count,
                    result.esg.emissions.len()
                ),
            );
        }
        if stats.cash_position_count != result.treasury.cash_positions.len() {
            report.error(
                "statistics",
                format!(
                    "cash_position_count stat ({}) != actual ({})",
                    stats.cash_position_count,
                    result.treasury.cash_positions.len()
                ),
            );
        }
        // companies_count and accounts_count
        if stats.companies_count != 2 {
            report.error(
                "statistics",
                format!(
                    "companies_count stat ({}) != expected (2)",
                    stats.companies_count
                ),
            );
        }

        // ================================================================
        // PRINT REPORT AND ASSERT
        // ================================================================
        report.print_report();

        // Fail the test if there are any errors
        assert_eq!(
            report.error_count(),
            0,
            "Found {} quality errors - see report above",
            report.error_count()
        );
    }
}
