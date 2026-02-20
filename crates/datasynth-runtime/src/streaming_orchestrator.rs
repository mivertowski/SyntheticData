//! Streaming orchestrator for real-time data generation.
//!
//! This orchestrator provides streaming capabilities with backpressure,
//! progress reporting, and control for real-time data generation.

use std::sync::Arc;
use std::thread;
use std::time::Instant;

use chrono::NaiveDate;
use tracing::{debug, info, warn};

use datasynth_config::schema::GeneratorConfig;
use datasynth_core::error::SynthResult;
use datasynth_core::models::{
    documents::{
        CustomerInvoice, Delivery, GoodsReceipt, Payment, PurchaseOrder, SalesOrder, VendorInvoice,
    },
    ChartOfAccounts, Customer, Employee, JournalEntry, Material, Vendor,
};
use datasynth_core::streaming::{stream_channel, StreamReceiver, StreamSender};
use datasynth_core::traits::{
    BackpressureStrategy, StreamConfig, StreamControl, StreamEvent, StreamProgress, StreamSummary,
};

/// Generated items that can be streamed.
#[derive(Debug, Clone)]
pub enum GeneratedItem {
    /// Chart of Accounts.
    ChartOfAccounts(Box<ChartOfAccounts>),
    /// A vendor.
    Vendor(Box<Vendor>),
    /// A customer.
    Customer(Box<Customer>),
    /// A material.
    Material(Box<Material>),
    /// An employee.
    Employee(Box<Employee>),
    /// A journal entry.
    JournalEntry(Box<JournalEntry>),
    /// A purchase order (P2P).
    PurchaseOrder(Box<PurchaseOrder>),
    /// A goods receipt (P2P).
    GoodsReceipt(Box<GoodsReceipt>),
    /// A vendor invoice (P2P).
    VendorInvoice(Box<VendorInvoice>),
    /// A payment (P2P/O2C).
    Payment(Box<Payment>),
    /// A sales order (O2C).
    SalesOrder(Box<SalesOrder>),
    /// A delivery (O2C).
    Delivery(Box<Delivery>),
    /// A customer invoice (O2C).
    CustomerInvoice(Box<CustomerInvoice>),
    /// Progress update.
    Progress(StreamProgress),
    /// Phase completion marker.
    PhaseComplete(String),
}

impl GeneratedItem {
    /// Returns the item type name.
    pub fn type_name(&self) -> &'static str {
        match self {
            GeneratedItem::ChartOfAccounts(_) => "chart_of_accounts",
            GeneratedItem::Vendor(_) => "vendor",
            GeneratedItem::Customer(_) => "customer",
            GeneratedItem::Material(_) => "material",
            GeneratedItem::Employee(_) => "employee",
            GeneratedItem::JournalEntry(_) => "journal_entry",
            GeneratedItem::PurchaseOrder(_) => "purchase_order",
            GeneratedItem::GoodsReceipt(_) => "goods_receipt",
            GeneratedItem::VendorInvoice(_) => "vendor_invoice",
            GeneratedItem::Payment(_) => "payment",
            GeneratedItem::SalesOrder(_) => "sales_order",
            GeneratedItem::Delivery(_) => "delivery",
            GeneratedItem::CustomerInvoice(_) => "customer_invoice",
            GeneratedItem::Progress(_) => "progress",
            GeneratedItem::PhaseComplete(_) => "phase_complete",
        }
    }
}

/// Phase of generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerationPhase {
    /// Chart of Accounts generation.
    ChartOfAccounts,
    /// Master data generation (vendors, customers, etc.).
    MasterData,
    /// Document flow generation (P2P, O2C).
    DocumentFlows,
    /// OCPM event log generation.
    OcpmEvents,
    /// Journal entry generation.
    JournalEntries,
    /// Anomaly injection.
    AnomalyInjection,
    /// Balance validation.
    BalanceValidation,
    /// Data quality injection.
    DataQuality,
    /// Complete.
    Complete,
}

impl GenerationPhase {
    /// Returns the phase name.
    pub fn name(&self) -> &'static str {
        match self {
            GenerationPhase::ChartOfAccounts => "chart_of_accounts",
            GenerationPhase::MasterData => "master_data",
            GenerationPhase::DocumentFlows => "document_flows",
            GenerationPhase::OcpmEvents => "ocpm_events",
            GenerationPhase::JournalEntries => "journal_entries",
            GenerationPhase::AnomalyInjection => "anomaly_injection",
            GenerationPhase::BalanceValidation => "balance_validation",
            GenerationPhase::DataQuality => "data_quality",
            GenerationPhase::Complete => "complete",
        }
    }
}

/// Configuration for streaming orchestration.
#[derive(Debug, Clone)]
pub struct StreamingOrchestratorConfig {
    /// Generator configuration.
    pub generator_config: GeneratorConfig,
    /// Stream configuration.
    pub stream_config: StreamConfig,
    /// Phases to execute.
    pub phases: Vec<GenerationPhase>,
}

impl StreamingOrchestratorConfig {
    /// Creates a new streaming orchestrator configuration.
    pub fn new(generator_config: GeneratorConfig) -> Self {
        Self {
            generator_config,
            stream_config: StreamConfig::default(),
            phases: vec![
                GenerationPhase::ChartOfAccounts,
                GenerationPhase::MasterData,
                GenerationPhase::DocumentFlows,
                GenerationPhase::JournalEntries,
            ],
        }
    }

    /// Creates a configuration with all phases enabled including OCPM.
    pub fn with_all_phases(generator_config: GeneratorConfig) -> Self {
        Self {
            generator_config,
            stream_config: StreamConfig::default(),
            phases: vec![
                GenerationPhase::ChartOfAccounts,
                GenerationPhase::MasterData,
                GenerationPhase::DocumentFlows,
                GenerationPhase::OcpmEvents,
                GenerationPhase::JournalEntries,
                GenerationPhase::AnomalyInjection,
                GenerationPhase::DataQuality,
            ],
        }
    }

    /// Sets the stream configuration.
    pub fn with_stream_config(mut self, config: StreamConfig) -> Self {
        self.stream_config = config;
        self
    }

    /// Sets the phases to execute.
    pub fn with_phases(mut self, phases: Vec<GenerationPhase>) -> Self {
        self.phases = phases;
        self
    }
}

/// Streaming orchestrator for real-time data generation.
pub struct StreamingOrchestrator {
    config: StreamingOrchestratorConfig,
}

impl StreamingOrchestrator {
    /// Creates a new streaming orchestrator.
    pub fn new(config: StreamingOrchestratorConfig) -> Self {
        Self { config }
    }

    /// Creates a streaming orchestrator from generator config with defaults.
    pub fn from_generator_config(config: GeneratorConfig) -> Self {
        Self::new(StreamingOrchestratorConfig::new(config))
    }

    /// Starts streaming generation.
    ///
    /// Returns a receiver for stream events and a control handle.
    pub fn stream(&self) -> SynthResult<(StreamReceiver<GeneratedItem>, Arc<StreamControl>)> {
        let (sender, receiver) = stream_channel(
            self.config.stream_config.buffer_size,
            self.config.stream_config.backpressure,
        );

        let control = Arc::new(StreamControl::new());
        let control_clone = Arc::clone(&control);

        let config = self.config.clone();

        // Spawn generation thread
        thread::spawn(move || {
            let result = Self::run_generation(config, sender, control_clone);
            if let Err(e) = result {
                warn!("Streaming generation error: {}", e);
            }
        });

        Ok((receiver, control))
    }

    /// Runs the generation process.
    fn run_generation(
        config: StreamingOrchestratorConfig,
        sender: StreamSender<GeneratedItem>,
        control: Arc<StreamControl>,
    ) -> SynthResult<()> {
        let start_time = Instant::now();
        let mut items_generated: u64 = 0;
        let mut phases_completed = Vec::new();

        // Track stats
        let progress_interval = config.stream_config.progress_interval;

        // Send initial progress
        let mut progress = StreamProgress::new("initializing");
        sender.send(StreamEvent::Progress(progress.clone()))?;

        for phase in &config.phases {
            if control.is_cancelled() {
                info!("Generation cancelled");
                break;
            }

            // Handle pause
            while control.is_paused() {
                std::thread::sleep(std::time::Duration::from_millis(100));
                if control.is_cancelled() {
                    break;
                }
            }

            progress.phase = phase.name().to_string();
            sender.send(StreamEvent::Progress(progress.clone()))?;

            match phase {
                GenerationPhase::ChartOfAccounts => {
                    let result =
                        Self::generate_coa_phase(&config.generator_config, &sender, &control)?;
                    items_generated += result;
                }
                GenerationPhase::MasterData => {
                    let result = Self::generate_master_data_phase(
                        &config.generator_config,
                        &sender,
                        &control,
                    )?;
                    items_generated += result;
                }
                GenerationPhase::DocumentFlows => {
                    let result = Self::generate_document_flows_phase(
                        &config.generator_config,
                        &sender,
                        &control,
                        progress_interval,
                        &mut progress,
                    )?;
                    items_generated += result;
                }
                GenerationPhase::OcpmEvents => {
                    // OCPM event generation is optional and requires document flows
                    debug!("OCPM events phase - skipping (documents should be generated via P2P/O2C generators)");
                }
                GenerationPhase::JournalEntries => {
                    let result = Self::generate_journal_entries_phase(
                        &config.generator_config,
                        &sender,
                        &control,
                        progress_interval,
                        &mut progress,
                    )?;
                    items_generated += result;
                }
                GenerationPhase::AnomalyInjection | GenerationPhase::DataQuality => {
                    // These phases modify existing data; not directly generating new items
                    debug!(
                        "Phase {:?} operates on existing data (not streaming new items)",
                        phase
                    );
                }
                GenerationPhase::BalanceValidation | GenerationPhase::Complete => {
                    // Validation and completion phases don't generate items
                    debug!("Phase {:?} is a validation/completion phase", phase);
                }
            }

            // Send phase completion
            sender.send(StreamEvent::Data(GeneratedItem::PhaseComplete(
                phase.name().to_string(),
            )))?;
            phases_completed.push(phase.name().to_string());

            // Update progress
            progress.items_generated = items_generated;
            progress.elapsed_ms = start_time.elapsed().as_millis() as u64;
            if progress.elapsed_ms > 0 {
                progress.items_per_second =
                    (items_generated as f64) / (progress.elapsed_ms as f64 / 1000.0);
            }
            sender.send(StreamEvent::Progress(progress.clone()))?;
        }

        // Send completion
        let stats = sender.stats();
        let summary = StreamSummary {
            total_items: items_generated,
            total_time_ms: start_time.elapsed().as_millis() as u64,
            avg_items_per_second: if start_time.elapsed().as_millis() > 0 {
                (items_generated as f64) / (start_time.elapsed().as_millis() as f64 / 1000.0)
            } else {
                0.0
            },
            error_count: 0,
            dropped_count: stats.items_dropped,
            peak_memory_mb: None,
            phases_completed,
        };

        sender.send(StreamEvent::Complete(summary))?;
        sender.close();

        Ok(())
    }

    /// Generates Chart of Accounts phase.
    fn generate_coa_phase(
        config: &GeneratorConfig,
        sender: &StreamSender<GeneratedItem>,
        control: &Arc<StreamControl>,
    ) -> SynthResult<u64> {
        use datasynth_generators::ChartOfAccountsGenerator;

        if control.is_cancelled() {
            return Ok(0);
        }

        info!("Generating Chart of Accounts");
        let seed = config.global.seed.unwrap_or(42);
        let complexity = config.chart_of_accounts.complexity;
        let industry = config.global.industry;
        let use_french_pcg = config.accounting_standards.enabled
            && matches!(
                config.accounting_standards.framework,
                Some(datasynth_config::schema::AccountingFrameworkConfig::FrenchGaap)
            );

        let mut coa_gen = ChartOfAccountsGenerator::new(complexity, industry, seed)
            .with_french_pcg(use_french_pcg);
        let coa = coa_gen.generate();

        let account_count = coa.account_count() as u64;
        sender.send(StreamEvent::Data(GeneratedItem::ChartOfAccounts(Box::new(
            coa,
        ))))?;

        Ok(account_count)
    }

    /// Generates master data phase.
    fn generate_master_data_phase(
        config: &GeneratorConfig,
        sender: &StreamSender<GeneratedItem>,
        control: &Arc<StreamControl>,
    ) -> SynthResult<u64> {
        use datasynth_generators::{CustomerGenerator, EmployeeGenerator, VendorGenerator};

        let mut count: u64 = 0;
        let seed = config.global.seed.unwrap_or(42);
        let md_config = &config.master_data;
        let effective_date = NaiveDate::parse_from_str(&config.global.start_date, "%Y-%m-%d")
            .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid default date"));

        let company_code = config
            .companies
            .first()
            .map(|c| c.code.as_str())
            .unwrap_or("1000");

        // Generate vendors
        if control.is_cancelled() {
            return Ok(count);
        }

        info!("Generating vendors");
        let mut vendor_gen = VendorGenerator::new(seed);
        for _ in 0..md_config.vendors.count {
            if control.is_cancelled() {
                break;
            }
            let vendor = vendor_gen.generate_vendor(company_code, effective_date);
            sender.send(StreamEvent::Data(GeneratedItem::Vendor(Box::new(vendor))))?;
            count += 1;
        }

        // Generate customers
        if control.is_cancelled() {
            return Ok(count);
        }

        info!("Generating customers");
        let mut customer_gen = CustomerGenerator::new(seed + 1);
        for _ in 0..md_config.customers.count {
            if control.is_cancelled() {
                break;
            }
            let customer = customer_gen.generate_customer(company_code, effective_date);
            sender.send(StreamEvent::Data(GeneratedItem::Customer(Box::new(
                customer,
            ))))?;
            count += 1;
        }

        // Generate employees
        if control.is_cancelled() {
            return Ok(count);
        }

        info!("Generating employees");
        let mut employee_gen = EmployeeGenerator::new(seed + 4);
        // Use first department from config
        let dept = datasynth_generators::DepartmentDefinition {
            code: "1000".to_string(),
            name: "General".to_string(),
            cost_center: "CC1000".to_string(),
            headcount: 10,
            system_roles: vec![],
            transaction_codes: vec![],
        };
        for _ in 0..md_config.employees.count {
            if control.is_cancelled() {
                break;
            }
            let employee = employee_gen.generate_employee(company_code, &dept, effective_date);
            sender.send(StreamEvent::Data(GeneratedItem::Employee(Box::new(
                employee,
            ))))?;
            count += 1;
        }

        Ok(count)
    }

    /// Generates journal entries phase.
    ///
    /// Note: This is a simplified version that generates basic journal entries.
    /// For full-featured generation with all options, use EnhancedOrchestrator.
    fn generate_journal_entries_phase(
        config: &GeneratorConfig,
        sender: &StreamSender<GeneratedItem>,
        control: &Arc<StreamControl>,
        progress_interval: u64,
        progress: &mut StreamProgress,
    ) -> SynthResult<u64> {
        use datasynth_generators::{ChartOfAccountsGenerator, JournalEntryGenerator};
        use std::sync::Arc;

        let mut count: u64 = 0;
        let seed = config.global.seed.unwrap_or(42);

        // Calculate total entries to generate based on volume weights
        let default_monthly = 500;
        let total_entries: usize = config
            .companies
            .iter()
            .map(|c| {
                let monthly = (c.volume_weight * default_monthly as f64) as usize;
                monthly.max(100) * config.global.period_months as usize
            })
            .sum();

        progress.items_remaining = Some(total_entries as u64);
        info!("Generating {} journal entries", total_entries);

        // Generate a shared CoA for all companies
        let complexity = config.chart_of_accounts.complexity;
        let industry = config.global.industry;
        let mut coa_gen = ChartOfAccountsGenerator::new(complexity, industry, seed);
        let coa = Arc::new(coa_gen.generate());

        // Parse start date
        let start_date = NaiveDate::parse_from_str(&config.global.start_date, "%Y-%m-%d")
            .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid default date"));
        let end_date =
            start_date + chrono::Duration::days((config.global.period_months as i64) * 30);

        // Create JE generator from config
        let mut je_gen = JournalEntryGenerator::from_generator_config(
            config,
            Arc::clone(&coa),
            start_date,
            end_date,
            seed,
        );

        for _ in 0..total_entries {
            if control.is_cancelled() {
                break;
            }

            // Handle pause
            while control.is_paused() {
                std::thread::sleep(std::time::Duration::from_millis(100));
                if control.is_cancelled() {
                    break;
                }
            }

            let je = je_gen.generate();
            sender.send(StreamEvent::Data(GeneratedItem::JournalEntry(Box::new(je))))?;
            count += 1;

            // Send progress updates
            if count.is_multiple_of(progress_interval) {
                progress.items_generated = count;
                progress.items_remaining = Some(total_entries as u64 - count);
                sender.send(StreamEvent::Progress(progress.clone()))?;
            }
        }

        Ok(count)
    }

    /// Generates document flows phase (P2P and O2C).
    ///
    /// Creates complete document chains:
    /// - P2P: PurchaseOrder → GoodsReceipt → VendorInvoice → Payment
    /// - O2C: SalesOrder → Delivery → CustomerInvoice
    fn generate_document_flows_phase(
        config: &GeneratorConfig,
        sender: &StreamSender<GeneratedItem>,
        control: &Arc<StreamControl>,
        progress_interval: u64,
        progress: &mut StreamProgress,
    ) -> SynthResult<u64> {
        use chrono::Datelike;
        use datasynth_generators::{
            CustomerGenerator, MaterialGenerator, O2CGenerator, P2PGenerator, VendorGenerator,
        };

        let mut count: u64 = 0;
        let seed = config.global.seed.unwrap_or(42);
        let df_config = &config.document_flows;
        let md_config = &config.master_data;

        // Parse dates
        let start_date = NaiveDate::parse_from_str(&config.global.start_date, "%Y-%m-%d")
            .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid default date"));

        let company_code = config
            .companies
            .first()
            .map(|c| c.code.as_str())
            .unwrap_or("1000");

        // Use master data config counts for generating reference data
        let vendor_count = md_config.vendors.count.min(100);
        let customer_count = md_config.customers.count.min(100);
        let material_count = md_config.materials.count.min(50);

        // Generate some master data for document flows
        let mut vendor_gen = VendorGenerator::new(seed);
        let mut customer_gen = CustomerGenerator::new(seed + 1);
        let mut material_gen = MaterialGenerator::new(seed + 2);

        let vendors: Vec<_> = (0..vendor_count)
            .map(|_| vendor_gen.generate_vendor(company_code, start_date))
            .collect();

        let customers: Vec<_> = (0..customer_count)
            .map(|_| customer_gen.generate_customer(company_code, start_date))
            .collect();

        let materials: Vec<_> = (0..material_count)
            .map(|_| material_gen.generate_material(company_code, start_date))
            .collect();

        // Determine number of chains based on transaction volume
        // Use period months as a multiplier for document chains
        let base_chains = (config.global.period_months as usize * 50).max(100);

        // P2P Generation
        if df_config.p2p.enabled && !vendors.is_empty() && !materials.is_empty() {
            info!("Generating P2P document flows");
            let mut p2p_gen = P2PGenerator::new(seed + 100);

            let chains_to_generate = base_chains.min(1000);
            progress.items_remaining = Some(chains_to_generate as u64);

            for i in 0..chains_to_generate {
                if control.is_cancelled() {
                    break;
                }

                // Handle pause
                while control.is_paused() {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    if control.is_cancelled() {
                        break;
                    }
                }

                let vendor = &vendors[i % vendors.len()];
                let material_refs: Vec<&datasynth_core::models::Material> =
                    vec![&materials[i % materials.len()]];

                // Calculate posting date within the period
                let days_offset = (i as i64 % (config.global.period_months as i64 * 30)).max(0);
                let po_date = start_date + chrono::Duration::days(days_offset);
                let fiscal_year = po_date.year() as u16;
                let fiscal_period = po_date.month() as u8;

                let chain = p2p_gen.generate_chain(
                    company_code,
                    vendor,
                    &material_refs,
                    po_date,
                    fiscal_year,
                    fiscal_period,
                    "SYSTEM",
                );

                // Send each document in the chain
                sender.send(StreamEvent::Data(GeneratedItem::PurchaseOrder(Box::new(
                    chain.purchase_order,
                ))))?;
                count += 1;

                for gr in chain.goods_receipts {
                    sender.send(StreamEvent::Data(GeneratedItem::GoodsReceipt(Box::new(gr))))?;
                    count += 1;
                }

                if let Some(vi) = chain.vendor_invoice {
                    sender.send(StreamEvent::Data(GeneratedItem::VendorInvoice(Box::new(
                        vi,
                    ))))?;
                    count += 1;
                }

                if let Some(payment) = chain.payment {
                    sender.send(StreamEvent::Data(GeneratedItem::Payment(Box::new(payment))))?;
                    count += 1;
                }

                if count.is_multiple_of(progress_interval) {
                    progress.items_generated = count;
                    sender.send(StreamEvent::Progress(progress.clone()))?;
                }
            }
        }

        // O2C Generation
        if df_config.o2c.enabled && !customers.is_empty() && !materials.is_empty() {
            info!("Generating O2C document flows");
            let mut o2c_gen = O2CGenerator::new(seed + 200);

            let chains_to_generate = base_chains.min(1000);

            for i in 0..chains_to_generate {
                if control.is_cancelled() {
                    break;
                }

                while control.is_paused() {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    if control.is_cancelled() {
                        break;
                    }
                }

                let customer = &customers[i % customers.len()];
                let material_refs: Vec<&datasynth_core::models::Material> =
                    vec![&materials[i % materials.len()]];

                let days_offset = (i as i64 % (config.global.period_months as i64 * 30)).max(0);
                let so_date = start_date + chrono::Duration::days(days_offset);
                let fiscal_year = so_date.year() as u16;
                let fiscal_period = so_date.month() as u8;

                let chain = o2c_gen.generate_chain(
                    company_code,
                    customer,
                    &material_refs,
                    so_date,
                    fiscal_year,
                    fiscal_period,
                    "SYSTEM",
                );

                sender.send(StreamEvent::Data(GeneratedItem::SalesOrder(Box::new(
                    chain.sales_order,
                ))))?;
                count += 1;

                for delivery in chain.deliveries {
                    sender.send(StreamEvent::Data(GeneratedItem::Delivery(Box::new(
                        delivery,
                    ))))?;
                    count += 1;
                }

                if let Some(ci) = chain.customer_invoice {
                    sender.send(StreamEvent::Data(GeneratedItem::CustomerInvoice(Box::new(
                        ci,
                    ))))?;
                    count += 1;
                }

                if count.is_multiple_of(progress_interval) {
                    progress.items_generated = count;
                    sender.send(StreamEvent::Progress(progress.clone()))?;
                }
            }
        }

        Ok(count)
    }

    /// Returns the orchestrator configuration stats.
    pub fn stats(&self) -> StreamingOrchestratorStats {
        StreamingOrchestratorStats {
            phases: self.config.phases.len(),
            buffer_size: self.config.stream_config.buffer_size,
            backpressure: self.config.stream_config.backpressure,
        }
    }
}

/// Statistics for the streaming orchestrator.
#[derive(Debug, Clone)]
pub struct StreamingOrchestratorStats {
    /// Number of phases configured.
    pub phases: usize,
    /// Buffer size.
    pub buffer_size: usize,
    /// Backpressure strategy.
    pub backpressure: BackpressureStrategy,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_config::presets::create_preset;
    use datasynth_config::schema::TransactionVolume;
    use datasynth_core::models::{CoAComplexity, IndustrySector};

    fn create_test_config() -> GeneratorConfig {
        create_preset(
            IndustrySector::Retail,
            2,
            3,
            CoAComplexity::Small,
            TransactionVolume::TenK,
        )
    }

    #[test]
    fn test_streaming_orchestrator_creation() {
        let config = create_test_config();
        let orchestrator = StreamingOrchestrator::from_generator_config(config);
        let stats = orchestrator.stats();

        assert!(stats.phases > 0);
        assert!(stats.buffer_size > 0);
    }

    #[test]
    fn test_streaming_generation() {
        let mut config = create_test_config();
        // Reduce volume for testing
        config.master_data.vendors.count = 5;
        config.master_data.customers.count = 5;
        config.master_data.employees.count = 5;
        config.global.period_months = 1;

        let streaming_config = StreamingOrchestratorConfig::new(config)
            .with_phases(vec![
                GenerationPhase::ChartOfAccounts,
                GenerationPhase::MasterData,
            ])
            .with_stream_config(StreamConfig {
                buffer_size: 100,
                progress_interval: 10,
                ..Default::default()
            });

        let orchestrator = StreamingOrchestrator::new(streaming_config);
        let (receiver, _control) = orchestrator.stream().unwrap();

        let mut items_count = 0;
        let mut has_coa = false;
        let mut has_completion = false;

        for event in receiver {
            match event {
                StreamEvent::Data(item) => {
                    items_count += 1;
                    if matches!(item, GeneratedItem::ChartOfAccounts(_)) {
                        has_coa = true;
                    }
                }
                StreamEvent::Complete(_) => {
                    has_completion = true;
                    break;
                }
                _ => {}
            }
        }

        assert!(items_count > 0);
        assert!(has_coa);
        assert!(has_completion);
    }

    #[test]
    fn test_stream_cancellation() {
        let mut config = create_test_config();
        config.global.period_months = 12; // Longer generation

        let streaming_config = StreamingOrchestratorConfig::new(config)
            .with_phases(vec![GenerationPhase::JournalEntries]);

        let orchestrator = StreamingOrchestrator::new(streaming_config);
        let (receiver, control) = orchestrator.stream().unwrap();

        // Cancel after receiving some items
        let mut items_count = 0;
        for event in receiver {
            if let StreamEvent::Data(_) = event {
                items_count += 1;
                if items_count >= 10 {
                    control.cancel();
                    break;
                }
            }
        }

        assert!(control.is_cancelled());
    }

    #[test]
    fn test_generated_item_type_name() {
        use datasynth_core::models::{CoAComplexity, IndustrySector};

        let coa = GeneratedItem::ChartOfAccounts(Box::new(ChartOfAccounts::new(
            "TEST_COA".to_string(),
            "Test Chart of Accounts".to_string(),
            "US".to_string(),
            IndustrySector::Manufacturing,
            CoAComplexity::Small,
        )));
        assert_eq!(coa.type_name(), "chart_of_accounts");

        let progress = GeneratedItem::Progress(StreamProgress::new("test"));
        assert_eq!(progress.type_name(), "progress");
    }
}
