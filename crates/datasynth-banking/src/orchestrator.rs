//! Banking data generation orchestrator.

use std::path::Path;

use crate::config::BankingConfig;
use crate::generators::{
    AccountGenerator, CounterpartyGenerator, CustomerGenerator, KycGenerator, TransactionGenerator,
};
use crate::labels::{
    AccountLabel, CustomerLabel, EntityLabelExtractor, ExportedNarrative, NarrativeGenerator,
    RelationshipLabel, RelationshipLabelExtractor, TransactionLabel, TransactionLabelExtractor,
};
use crate::models::{AmlScenario, BankAccount, BankTransaction, BankingCustomer, CounterpartyPool};
use crate::typologies::TypologyInjector;

/// Banking data generation orchestrator.
///
/// Coordinates the generation of:
/// - Customers with KYC profiles
/// - Accounts for customers
/// - Transactions based on personas
/// - AML typology injection
/// - Ground truth labels
pub struct BankingOrchestrator {
    config: BankingConfig,
    seed: u64,
}

/// Generated banking data result.
#[derive(Debug)]
pub struct BankingData {
    /// Generated customers
    pub customers: Vec<BankingCustomer>,
    /// Generated accounts
    pub accounts: Vec<BankAccount>,
    /// Generated transactions
    pub transactions: Vec<BankTransaction>,
    /// Counterparty pool
    pub counterparties: CounterpartyPool,
    /// AML scenarios
    pub scenarios: Vec<AmlScenario>,
    /// Transaction labels
    pub transaction_labels: Vec<TransactionLabel>,
    /// Customer labels
    pub customer_labels: Vec<CustomerLabel>,
    /// Account labels
    pub account_labels: Vec<AccountLabel>,
    /// Relationship labels
    pub relationship_labels: Vec<RelationshipLabel>,
    /// Case narratives
    pub narratives: Vec<ExportedNarrative>,
    /// Generation statistics
    pub stats: GenerationStats,
}

/// Generation statistics.
#[derive(Debug, Clone, Default)]
pub struct GenerationStats {
    /// Total customers generated
    pub customer_count: usize,
    /// Total accounts generated
    pub account_count: usize,
    /// Total transactions generated
    pub transaction_count: usize,
    /// Suspicious transaction count
    pub suspicious_count: usize,
    /// Suspicious rate
    pub suspicious_rate: f64,
    /// Spoofed transaction count
    pub spoofed_count: usize,
    /// Spoofed rate
    pub spoofed_rate: f64,
    /// AML scenario count
    pub scenario_count: usize,
    /// Generation duration in milliseconds
    pub duration_ms: u64,
}

impl BankingOrchestrator {
    /// Create a new banking orchestrator.
    pub fn new(config: BankingConfig, seed: u64) -> Self {
        Self { config, seed }
    }

    /// Generate all banking data.
    pub fn generate(&self) -> BankingData {
        let start = std::time::Instant::now();

        // Phase 1: Generate counterparty pool
        let mut counterparty_gen = CounterpartyGenerator::new(self.seed);
        let counterparties = counterparty_gen.generate_pool(&self.config);

        // Phase 2: Generate customers with KYC profiles
        let mut customer_gen = CustomerGenerator::new(self.config.clone(), self.seed);
        let mut customers = customer_gen.generate_all();

        // Phase 3: Generate KYC profiles
        let mut kyc_gen = KycGenerator::new(self.seed);
        for customer in &mut customers {
            let profile = kyc_gen.generate_profile(customer, &self.config);
            customer.kyc_profile = profile;
        }

        // Phase 4: Generate accounts for customers
        let mut account_gen = AccountGenerator::new(self.config.clone(), self.seed);
        let mut accounts = account_gen.generate_for_customers(&mut customers);

        // Phase 5: Generate transactions
        let mut txn_gen = TransactionGenerator::new(self.config.clone(), self.seed);
        let mut transactions = txn_gen.generate_all(&customers, &mut accounts);

        // Phase 6: Inject AML typologies
        let mut typology_injector = TypologyInjector::new(self.config.clone(), self.seed);
        typology_injector.inject(&mut customers, &mut accounts, &mut transactions);
        let scenarios: Vec<AmlScenario> = typology_injector.get_scenarios().to_vec();

        // Phase 7: Generate narratives
        let mut narrative_gen = NarrativeGenerator::new(self.seed);
        let narratives: Vec<ExportedNarrative> = scenarios
            .iter()
            .map(|s| {
                let narrative = narrative_gen.generate(s);
                ExportedNarrative::from_scenario(s, &narrative)
            })
            .collect();

        // Phase 8: Extract labels
        let transaction_labels = TransactionLabelExtractor::extract_with_features(&transactions);
        let customer_labels = EntityLabelExtractor::extract_customers(&customers);
        let account_labels = EntityLabelExtractor::extract_accounts(&accounts);
        let relationship_labels = RelationshipLabelExtractor::extract_from_customers(&customers);

        // Compute statistics
        let suspicious_count = transactions.iter().filter(|t| t.is_suspicious).count();
        let spoofed_count = transactions.iter().filter(|t| t.is_spoofed).count();

        let stats = GenerationStats {
            customer_count: customers.len(),
            account_count: accounts.len(),
            transaction_count: transactions.len(),
            suspicious_count,
            suspicious_rate: suspicious_count as f64 / transactions.len().max(1) as f64,
            spoofed_count,
            spoofed_rate: spoofed_count as f64 / transactions.len().max(1) as f64,
            scenario_count: scenarios.len(),
            duration_ms: start.elapsed().as_millis() as u64,
        };

        BankingData {
            customers,
            accounts,
            transactions,
            counterparties,
            scenarios,
            transaction_labels,
            customer_labels,
            account_labels,
            relationship_labels,
            narratives,
            stats,
        }
    }

    /// Write generated data to output directory.
    pub fn write_output(&self, data: &BankingData, output_dir: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(output_dir)?;

        // Write customers
        self.write_csv(&data.customers, &output_dir.join("banking_customers.csv"))?;

        // Write accounts
        self.write_csv(&data.accounts, &output_dir.join("banking_accounts.csv"))?;

        // Write transactions
        self.write_csv(
            &data.transactions,
            &output_dir.join("banking_transactions.csv"),
        )?;

        // Write labels
        self.write_csv(
            &data.transaction_labels,
            &output_dir.join("transaction_labels.csv"),
        )?;
        self.write_csv(
            &data.customer_labels,
            &output_dir.join("customer_labels.csv"),
        )?;
        self.write_csv(&data.account_labels, &output_dir.join("account_labels.csv"))?;
        self.write_csv(
            &data.relationship_labels,
            &output_dir.join("relationship_labels.csv"),
        )?;

        // Write narratives as JSON
        self.write_json(&data.narratives, &output_dir.join("case_narratives.json"))?;

        // Write counterparties
        self.write_csv(
            &data.counterparties.merchants,
            &output_dir.join("merchants.csv"),
        )?;
        self.write_csv(
            &data.counterparties.employers,
            &output_dir.join("employers.csv"),
        )?;

        Ok(())
    }

    /// Write data to CSV file.
    fn write_csv<T: serde::Serialize>(&self, data: &[T], path: &Path) -> std::io::Result<()> {
        let mut writer = csv::Writer::from_path(path)?;
        for item in data {
            writer.serialize(item)?;
        }
        writer.flush()?;
        Ok(())
    }

    /// Write data to JSON file.
    fn write_json<T: serde::Serialize>(&self, data: &T, path: &Path) -> std::io::Result<()> {
        let file = std::fs::File::create(path)?;
        serde_json::to_writer_pretty(file, data)?;
        Ok(())
    }
}

/// Builder for BankingOrchestrator.
pub struct BankingOrchestratorBuilder {
    config: Option<BankingConfig>,
    seed: u64,
}

impl Default for BankingOrchestratorBuilder {
    fn default() -> Self {
        Self {
            config: None,
            seed: 42,
        }
    }
}

impl BankingOrchestratorBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the configuration.
    pub fn config(mut self, config: BankingConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Set the random seed.
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Build the orchestrator.
    pub fn build(self) -> BankingOrchestrator {
        BankingOrchestrator::new(self.config.unwrap_or_default(), self.seed)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestrator_generation() {
        let config = BankingConfig::small();
        let orchestrator = BankingOrchestrator::new(config, 12345);

        let data = orchestrator.generate();

        assert!(!data.customers.is_empty());
        assert!(!data.accounts.is_empty());
        assert!(!data.transactions.is_empty());
        assert!(!data.transaction_labels.is_empty());
        assert!(!data.customer_labels.is_empty());

        // Stats should be populated
        assert!(data.stats.customer_count > 0);
        assert!(data.stats.transaction_count > 0);
    }

    #[test]
    fn test_builder() {
        let orchestrator = BankingOrchestratorBuilder::new()
            .config(BankingConfig::small())
            .seed(12345)
            .build();

        let data = orchestrator.generate();
        assert!(!data.customers.is_empty());
    }
}
