//! Main AML typology injector.

use chrono::NaiveDate;
use datasynth_core::models::banking::{AmlTypology, LaunderingStage, Sophistication};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use crate::config::BankingConfig;
use crate::models::{
    AmlScenario, BankAccount, BankTransaction, BankingCustomer, CaseNarrative, CaseRecommendation,
};

use super::{FunnelInjector, LayeringInjector, MuleInjector, SpoofingEngine, StructuringInjector};
use crate::seed_offsets::TYPOLOGY_INJECTOR_SEED_OFFSET;

/// Main AML typology injector.
pub struct TypologyInjector {
    config: BankingConfig,
    rng: ChaCha8Rng,
    structuring_injector: StructuringInjector,
    funnel_injector: FunnelInjector,
    layering_injector: LayeringInjector,
    mule_injector: MuleInjector,
    spoofing_engine: SpoofingEngine,
    scenarios: Vec<AmlScenario>,
    scenario_counter: u32,
}

impl TypologyInjector {
    /// Create a new typology injector.
    pub fn new(config: BankingConfig, seed: u64) -> Self {
        Self {
            config: config.clone(),
            rng: ChaCha8Rng::seed_from_u64(seed.wrapping_add(TYPOLOGY_INJECTOR_SEED_OFFSET)),
            structuring_injector: StructuringInjector::new(seed),
            funnel_injector: FunnelInjector::new(seed),
            layering_injector: LayeringInjector::new(seed),
            mule_injector: MuleInjector::new(seed),
            spoofing_engine: SpoofingEngine::new(config.spoofing.clone(), seed),
            scenarios: Vec::new(),
            scenario_counter: 0,
        }
    }

    /// Inject AML patterns into transactions.
    pub fn inject(
        &mut self,
        customers: &mut [BankingCustomer],
        accounts: &mut [BankAccount],
        transactions: &mut Vec<BankTransaction>,
    ) {
        if !self.config.typologies.suspicious_rate.is_finite()
            || self.config.typologies.suspicious_rate <= 0.0
        {
            return;
        }

        let total_customers = customers.len();

        // Calculate number of suspicious customers
        let suspicious_count =
            (total_customers as f64 * self.config.typologies.suspicious_rate) as usize;

        // Select random customers for suspicious activity
        let mut customer_indices: Vec<usize> = (0..total_customers).collect();
        customer_indices.shuffle(&mut self.rng);
        let suspicious_indices: Vec<usize> = customer_indices
            .into_iter()
            .take(suspicious_count)
            .collect();

        // Inject different typologies
        for &idx in &suspicious_indices {
            let typology = self.select_typology();
            let sophistication = self.select_sophistication();

            match typology {
                AmlTypology::Structuring | AmlTypology::Smurfing => {
                    self.inject_structuring(idx, customers, accounts, transactions, sophistication);
                }
                AmlTypology::FunnelAccount => {
                    self.inject_funnel(idx, customers, accounts, transactions, sophistication);
                }
                AmlTypology::Layering => {
                    self.inject_layering(idx, customers, accounts, transactions, sophistication);
                }
                AmlTypology::MoneyMule => {
                    self.inject_mule(idx, customers, accounts, transactions, sophistication);
                }
                _ => {
                    // Generic suspicious marking
                    self.inject_generic(idx, customers, accounts, transactions, typology);
                }
            }
        }

        // Apply spoofing to sophisticated typologies
        if self.config.spoofing.enabled {
            self.apply_spoofing(customers, transactions);
        }
    }

    /// Select a typology based on configured rates.
    fn select_typology(&mut self) -> AmlTypology {
        let rates = [
            (
                AmlTypology::Structuring,
                self.config.typologies.structuring_rate,
            ),
            (
                AmlTypology::FunnelAccount,
                self.config.typologies.funnel_rate,
            ),
            (AmlTypology::Layering, self.config.typologies.layering_rate),
            (AmlTypology::MoneyMule, self.config.typologies.mule_rate),
            (
                AmlTypology::AccountTakeover,
                self.config.typologies.fraud_rate * 0.3,
            ),
            (
                AmlTypology::FirstPartyFraud,
                self.config.typologies.fraud_rate * 0.3,
            ),
            (
                AmlTypology::AuthorizedPushPayment,
                self.config.typologies.fraud_rate * 0.4,
            ),
        ];

        let total: f64 = rates.iter().map(|(_, r)| r).sum();
        if total <= 0.0 {
            return AmlTypology::Structuring;
        }

        let roll: f64 = self.rng.gen::<f64>() * total;
        let mut cumulative = 0.0;

        for (typology, rate) in rates {
            cumulative += rate;
            if roll < cumulative {
                return typology;
            }
        }

        AmlTypology::Structuring
    }

    /// Select sophistication level.
    fn select_sophistication(&mut self) -> Sophistication {
        let dist = &self.config.typologies.sophistication;
        let total = dist.basic + dist.standard + dist.professional + dist.advanced;
        let roll: f64 = self.rng.gen::<f64>() * total;

        let mut cumulative = 0.0;
        cumulative += dist.basic;
        if roll < cumulative {
            return Sophistication::Basic;
        }
        cumulative += dist.standard;
        if roll < cumulative {
            return Sophistication::Standard;
        }
        cumulative += dist.professional;
        if roll < cumulative {
            return Sophistication::Professional;
        }
        Sophistication::Advanced
    }

    /// Inject structuring pattern.
    fn inject_structuring(
        &mut self,
        customer_idx: usize,
        customers: &mut [BankingCustomer],
        accounts: &mut [BankAccount],
        transactions: &mut Vec<BankTransaction>,
        sophistication: Sophistication,
    ) {
        let customer = &mut customers[customer_idx];
        if customer.account_ids.is_empty() {
            return;
        }

        let account_id = customer.account_ids[0];
        let account = accounts.iter_mut().find(|a| a.account_id == account_id);

        if let Some(account) = account {
            let scenario_id = self.next_scenario_id();
            let start_date =
                NaiveDate::parse_from_str(&self.config.population.start_date, "%Y-%m-%d")
                    .unwrap_or_else(|_| {
                        NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid default date")
                    });
            let end_date = start_date + chrono::Months::new(1);

            let mut scenario =
                AmlScenario::new(&scenario_id, AmlTypology::Structuring, start_date, end_date)
                    .with_sophistication(sophistication);

            // Generate structuring transactions
            let structuring_txns = self.structuring_injector.generate(
                customer,
                account,
                start_date,
                end_date,
                sophistication,
            );

            for txn in structuring_txns {
                scenario.add_transaction(txn.transaction_id, txn.amount);
                transactions.push(txn);
            }

            scenario.add_customer(customer.customer_id);
            scenario.add_account(account.account_id);
            scenario.add_stage(LaunderingStage::Placement);

            // Generate narrative
            scenario.narrative = CaseNarrative::new(
                &format!(
                    "Customer {} conducted multiple cash deposits just below reporting threshold over a short period.",
                    customer.name.display_name()
                ),
            )
            .with_recommendation(CaseRecommendation::FileSar);

            // Mark customer
            customer.is_mule = false;
            account.case_id = Some(scenario_id.clone());

            self.scenarios.push(scenario);
        }
    }

    /// Inject funnel account pattern.
    fn inject_funnel(
        &mut self,
        customer_idx: usize,
        customers: &mut [BankingCustomer],
        accounts: &mut [BankAccount],
        transactions: &mut Vec<BankTransaction>,
        sophistication: Sophistication,
    ) {
        let customer = &mut customers[customer_idx];
        if customer.account_ids.is_empty() {
            return;
        }

        let account_id = customer.account_ids[0];
        if let Some(account) = accounts.iter_mut().find(|a| a.account_id == account_id) {
            let scenario_id = self.next_scenario_id();
            let start_date =
                NaiveDate::parse_from_str(&self.config.population.start_date, "%Y-%m-%d")
                    .unwrap_or_else(|_| {
                        NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid default date")
                    });
            let end_date = start_date + chrono::Months::new(2);

            let mut scenario = AmlScenario::new(
                &scenario_id,
                AmlTypology::FunnelAccount,
                start_date,
                end_date,
            )
            .with_sophistication(sophistication);

            // Generate funnel transactions
            let funnel_txns = self.funnel_injector.generate(
                customer,
                account,
                start_date,
                end_date,
                sophistication,
            );

            for txn in funnel_txns {
                scenario.add_transaction(txn.transaction_id, txn.amount);
                transactions.push(txn);
            }

            scenario.add_customer(customer.customer_id);
            scenario.add_account(account.account_id);
            scenario.add_stage(LaunderingStage::Layering);
            account.is_funnel_account = true;
            account.case_id = Some(scenario_id.clone());

            scenario.narrative = CaseNarrative::new(
                "Account shows funnel pattern with many inbound transfers rapidly consolidated and moved out.",
            )
            .with_recommendation(CaseRecommendation::FileSar);

            self.scenarios.push(scenario);
        }
    }

    /// Inject layering pattern.
    fn inject_layering(
        &mut self,
        customer_idx: usize,
        customers: &mut [BankingCustomer],
        accounts: &mut [BankAccount],
        transactions: &mut Vec<BankTransaction>,
        sophistication: Sophistication,
    ) {
        let customer = &mut customers[customer_idx];
        if customer.account_ids.is_empty() {
            return;
        }

        let account_id = customer.account_ids[0];
        if let Some(account) = accounts.iter_mut().find(|a| a.account_id == account_id) {
            let scenario_id = self.next_scenario_id();
            let start_date =
                NaiveDate::parse_from_str(&self.config.population.start_date, "%Y-%m-%d")
                    .unwrap_or_else(|_| {
                        NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid default date")
                    });
            let end_date = start_date + chrono::Months::new(1);

            let mut scenario =
                AmlScenario::new(&scenario_id, AmlTypology::Layering, start_date, end_date)
                    .with_sophistication(sophistication);

            let layering_txns = self.layering_injector.generate(
                customer,
                account,
                start_date,
                end_date,
                sophistication,
            );

            for txn in layering_txns {
                scenario.add_transaction(txn.transaction_id, txn.amount);
                transactions.push(txn);
            }

            scenario.add_customer(customer.customer_id);
            scenario.add_account(account.account_id);
            scenario.add_stage(LaunderingStage::Layering);
            account.case_id = Some(scenario_id.clone());

            scenario.narrative = CaseNarrative::new(
                "Complex layering pattern with rapid multi-hop transfers designed to obscure fund trail.",
            )
            .with_recommendation(CaseRecommendation::FileSar);

            self.scenarios.push(scenario);
        }
    }

    /// Inject mule pattern.
    fn inject_mule(
        &mut self,
        customer_idx: usize,
        customers: &mut [BankingCustomer],
        accounts: &mut [BankAccount],
        transactions: &mut Vec<BankTransaction>,
        sophistication: Sophistication,
    ) {
        let customer = &mut customers[customer_idx];
        if customer.account_ids.is_empty() {
            return;
        }

        let account_id = customer.account_ids[0];
        if let Some(account) = accounts.iter_mut().find(|a| a.account_id == account_id) {
            let scenario_id = self.next_scenario_id();
            let start_date =
                NaiveDate::parse_from_str(&self.config.population.start_date, "%Y-%m-%d")
                    .unwrap_or_else(|_| {
                        NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid default date")
                    });
            let end_date = start_date + chrono::Months::new(1);

            let mut scenario =
                AmlScenario::new(&scenario_id, AmlTypology::MoneyMule, start_date, end_date)
                    .with_sophistication(sophistication);

            let mule_txns = self.mule_injector.generate(
                customer,
                account,
                start_date,
                end_date,
                sophistication,
            );

            for txn in mule_txns {
                scenario.add_transaction(txn.transaction_id, txn.amount);
                transactions.push(txn);
            }

            scenario.add_customer(customer.customer_id);
            scenario.add_account(account.account_id);
            customer.is_mule = true;
            account.is_mule_account = true;
            account.case_id = Some(scenario_id.clone());

            scenario.narrative = CaseNarrative::new(
                "Account shows classic money mule pattern: inbound transfers followed by rapid cash withdrawals or wire transfers.",
            )
            .with_recommendation(CaseRecommendation::CloseAccount);

            self.scenarios.push(scenario);
        }
    }

    /// Inject generic suspicious pattern.
    fn inject_generic(
        &mut self,
        customer_idx: usize,
        customers: &mut [BankingCustomer],
        accounts: &mut [BankAccount],
        transactions: &mut [BankTransaction],
        typology: AmlTypology,
    ) {
        let customer = &mut customers[customer_idx];
        if customer.account_ids.is_empty() {
            return;
        }

        // Mark random transactions as suspicious
        let account_id = customer.account_ids[0];
        let scenario_id = self.next_scenario_id();

        for txn in transactions.iter_mut() {
            if txn.account_id == account_id && self.rng.gen::<f64>() < 0.1 {
                txn.is_suspicious = true;
                txn.suspicion_reason = Some(typology);
                txn.case_id = Some(scenario_id.clone());
            }
        }

        if let Some(account) = accounts.iter_mut().find(|a| a.account_id == account_id) {
            account.case_id = Some(scenario_id);
        }
    }

    /// Apply spoofing to sophisticated transactions.
    fn apply_spoofing(
        &mut self,
        customers: &[BankingCustomer],
        transactions: &mut [BankTransaction],
    ) {
        for txn in transactions.iter_mut() {
            if txn.is_suspicious {
                // Find the customer
                let customer = customers
                    .iter()
                    .find(|c| c.account_ids.contains(&txn.account_id));

                if let Some(customer) = customer {
                    self.spoofing_engine.apply(txn, customer);
                }
            }
        }
    }

    /// Generate next scenario ID.
    fn next_scenario_id(&mut self) -> String {
        self.scenario_counter += 1;
        format!("SC-{:06}", self.scenario_counter)
    }

    /// Get all generated scenarios.
    pub fn get_scenarios(&self) -> &[AmlScenario] {
        &self.scenarios
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_typology_injection() {
        let config = BankingConfig::small();
        let mut injector = TypologyInjector::new(config.clone(), 12345);

        // Generate test data
        let mut customer_gen = crate::generators::CustomerGenerator::new(config.clone(), 12345);
        let mut customers = customer_gen.generate_all();

        let mut account_gen = crate::generators::AccountGenerator::new(config.clone(), 12345);
        let mut accounts = account_gen.generate_for_customers(&mut customers);

        let mut txn_gen = crate::generators::TransactionGenerator::new(config, 12345);
        let mut transactions = txn_gen.generate_all(&customers, &mut accounts);

        let initial_count = transactions.len();

        injector.inject(&mut customers, &mut accounts, &mut transactions);

        // Should have added some suspicious transactions
        let suspicious_count = transactions.iter().filter(|t| t.is_suspicious).count();
        assert!(suspicious_count > 0 || transactions.len() > initial_count);
    }
}
