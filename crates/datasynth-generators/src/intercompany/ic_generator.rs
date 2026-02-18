//! Intercompany transaction generator.
//!
//! Generates matched pairs of intercompany journal entries that offset
//! between related entities.

use chrono::{Datelike, NaiveDate};
use datasynth_core::utils::weighted_select;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use tracing::debug;

use datasynth_core::models::intercompany::{
    ICLoan, ICMatchedPair, ICTransactionType, OwnershipStructure, RecurringFrequency,
    TransferPricingMethod, TransferPricingPolicy,
};
use datasynth_core::models::{JournalEntry, JournalEntryLine};

/// Configuration for IC transaction generation.
#[derive(Debug, Clone)]
pub struct ICGeneratorConfig {
    /// Probability of generating an IC transaction (0.0 to 1.0).
    pub ic_transaction_rate: f64,
    /// Transfer pricing method to use.
    pub transfer_pricing_method: TransferPricingMethod,
    /// Markup percentage for cost-plus method.
    pub markup_percent: Decimal,
    /// Generate matched pairs (both sides of IC transaction).
    pub generate_matched_pairs: bool,
    /// Transaction type distribution.
    pub transaction_type_weights: HashMap<ICTransactionType, f64>,
    /// Generate netting settlements.
    pub generate_netting: bool,
    /// Netting frequency (if enabled).
    pub netting_frequency: RecurringFrequency,
    /// Generate IC loans.
    pub generate_loans: bool,
    /// Typical loan amount range.
    pub loan_amount_range: (Decimal, Decimal),
    /// Loan interest rate range.
    pub loan_interest_rate_range: (Decimal, Decimal),
}

impl Default for ICGeneratorConfig {
    fn default() -> Self {
        let mut weights = HashMap::new();
        weights.insert(ICTransactionType::GoodsSale, 0.35);
        weights.insert(ICTransactionType::ServiceProvided, 0.20);
        weights.insert(ICTransactionType::ManagementFee, 0.15);
        weights.insert(ICTransactionType::Royalty, 0.10);
        weights.insert(ICTransactionType::CostSharing, 0.10);
        weights.insert(ICTransactionType::LoanInterest, 0.05);
        weights.insert(ICTransactionType::ExpenseRecharge, 0.05);

        Self {
            ic_transaction_rate: 0.15,
            transfer_pricing_method: TransferPricingMethod::CostPlus,
            markup_percent: dec!(5),
            generate_matched_pairs: true,
            transaction_type_weights: weights,
            generate_netting: true,
            netting_frequency: RecurringFrequency::Monthly,
            generate_loans: true,
            loan_amount_range: (dec!(100000), dec!(10000000)),
            loan_interest_rate_range: (dec!(2), dec!(8)),
        }
    }
}

/// Generator for intercompany transactions.
pub struct ICGenerator {
    /// Configuration.
    config: ICGeneratorConfig,
    /// Random number generator.
    rng: ChaCha8Rng,
    /// Ownership structure.
    ownership_structure: OwnershipStructure,
    /// Transfer pricing policies by relationship.
    transfer_pricing_policies: HashMap<String, TransferPricingPolicy>,
    /// Active IC loans.
    active_loans: Vec<ICLoan>,
    /// Generated IC matched pairs.
    matched_pairs: Vec<ICMatchedPair>,
    /// IC reference counter.
    ic_counter: u64,
    /// Document counter.
    doc_counter: u64,
}

impl ICGenerator {
    /// Create a new IC generator.
    pub fn new(
        config: ICGeneratorConfig,
        ownership_structure: OwnershipStructure,
        seed: u64,
    ) -> Self {
        Self {
            config,
            rng: ChaCha8Rng::seed_from_u64(seed),
            ownership_structure,
            transfer_pricing_policies: HashMap::new(),
            active_loans: Vec::new(),
            matched_pairs: Vec::new(),
            ic_counter: 0,
            doc_counter: 0,
        }
    }

    /// Add a transfer pricing policy.
    pub fn add_transfer_pricing_policy(
        &mut self,
        relationship_id: String,
        policy: TransferPricingPolicy,
    ) {
        self.transfer_pricing_policies
            .insert(relationship_id, policy);
    }

    /// Generate IC reference number.
    fn generate_ic_reference(&mut self, date: NaiveDate) -> String {
        self.ic_counter += 1;
        format!("IC{}{:06}", date.format("%Y%m"), self.ic_counter)
    }

    /// Generate document number.
    fn generate_doc_number(&mut self, prefix: &str) -> String {
        self.doc_counter += 1;
        format!("{}{:08}", prefix, self.doc_counter)
    }

    /// Select a random IC transaction type based on weights.
    fn select_transaction_type(&mut self) -> ICTransactionType {
        let options: Vec<(ICTransactionType, f64)> = self
            .config
            .transaction_type_weights
            .iter()
            .map(|(&tx_type, &weight)| (tx_type, weight))
            .collect();

        if options.is_empty() {
            return ICTransactionType::GoodsSale;
        }

        *weighted_select(&mut self.rng, &options)
    }

    /// Select a random pair of related companies.
    fn select_company_pair(&mut self) -> Option<(String, String)> {
        let relationships = self.ownership_structure.relationships.clone();
        if relationships.is_empty() {
            return None;
        }

        let rel = relationships.choose(&mut self.rng)?;

        // Randomly decide direction (parent sells to sub, or sub sells to parent)
        if self.rng.gen_bool(0.5) {
            Some((rel.parent_company.clone(), rel.subsidiary_company.clone()))
        } else {
            Some((rel.subsidiary_company.clone(), rel.parent_company.clone()))
        }
    }

    /// Generate a base amount for IC transaction.
    fn generate_base_amount(&mut self, tx_type: ICTransactionType) -> Decimal {
        let (min, max) = match tx_type {
            ICTransactionType::GoodsSale => (dec!(1000), dec!(500000)),
            ICTransactionType::ServiceProvided => (dec!(5000), dec!(200000)),
            ICTransactionType::ManagementFee => (dec!(10000), dec!(100000)),
            ICTransactionType::Royalty => (dec!(5000), dec!(150000)),
            ICTransactionType::CostSharing => (dec!(2000), dec!(50000)),
            ICTransactionType::LoanInterest => (dec!(1000), dec!(50000)),
            ICTransactionType::ExpenseRecharge => (dec!(500), dec!(20000)),
            ICTransactionType::Dividend => (dec!(50000), dec!(1000000)),
            _ => (dec!(1000), dec!(100000)),
        };

        let range = max - min;
        let random_factor = Decimal::from_f64_retain(self.rng.gen::<f64>()).unwrap_or(dec!(0.5));
        (min + range * random_factor).round_dp(2)
    }

    /// Apply transfer pricing markup to base amount.
    fn apply_transfer_pricing(&self, base_amount: Decimal, relationship_id: &str) -> Decimal {
        if let Some(policy) = self.transfer_pricing_policies.get(relationship_id) {
            policy.calculate_transfer_price(base_amount)
        } else {
            // Use default config markup
            base_amount * (Decimal::ONE + self.config.markup_percent / dec!(100))
        }
    }

    /// Generate a single IC matched pair.
    pub fn generate_ic_transaction(
        &mut self,
        date: NaiveDate,
        _fiscal_period: &str,
    ) -> Option<ICMatchedPair> {
        // Check if we should generate an IC transaction
        if !self.rng.gen_bool(self.config.ic_transaction_rate) {
            return None;
        }

        let (seller, buyer) = self.select_company_pair()?;
        let tx_type = self.select_transaction_type();
        let base_amount = self.generate_base_amount(tx_type);

        // Find relationship for transfer pricing
        let relationship_id = format!("{}-{}", seller, buyer);
        let transfer_price = self.apply_transfer_pricing(base_amount, &relationship_id);

        let ic_reference = self.generate_ic_reference(date);
        let seller_doc = self.generate_doc_number("ICS");
        let buyer_doc = self.generate_doc_number("ICB");

        let mut pair = ICMatchedPair::new(
            ic_reference,
            tx_type,
            seller.clone(),
            buyer.clone(),
            transfer_price,
            "USD".to_string(), // Could be parameterized
            date,
        );

        // Assign document numbers
        pair.seller_document = seller_doc;
        pair.buyer_document = buyer_doc;

        // Calculate withholding tax if applicable
        if tx_type.has_withholding_tax() {
            pair.calculate_withholding_tax();
        }

        self.matched_pairs.push(pair.clone());
        Some(pair)
    }

    /// Generate IC journal entries from a matched pair.
    pub fn generate_journal_entries(
        &mut self,
        pair: &ICMatchedPair,
        fiscal_year: i32,
        fiscal_period: u32,
    ) -> (JournalEntry, JournalEntry) {
        let (seller_dr_desc, seller_cr_desc) = pair.transaction_type.seller_accounts();
        let (buyer_dr_desc, buyer_cr_desc) = pair.transaction_type.buyer_accounts();

        // Seller entry: DR IC Receivable, CR Revenue/Income
        let seller_entry = self.create_seller_entry(
            pair,
            fiscal_year,
            fiscal_period,
            seller_dr_desc,
            seller_cr_desc,
        );

        // Buyer entry: DR Expense/Asset, CR IC Payable
        let buyer_entry = self.create_buyer_entry(
            pair,
            fiscal_year,
            fiscal_period,
            buyer_dr_desc,
            buyer_cr_desc,
        );

        (seller_entry, buyer_entry)
    }

    /// Create seller-side journal entry.
    fn create_seller_entry(
        &mut self,
        pair: &ICMatchedPair,
        _fiscal_year: i32,
        _fiscal_period: u32,
        dr_desc: &str,
        cr_desc: &str,
    ) -> JournalEntry {
        let mut je = JournalEntry::new_simple(
            pair.seller_document.clone(),
            pair.seller_company.clone(),
            pair.posting_date,
            format!(
                "IC {} to {}",
                pair.transaction_type.seller_accounts().1,
                pair.buyer_company
            ),
        );

        je.header.reference = Some(pair.ic_reference.clone());
        je.header.document_type = "IC".to_string();
        je.header.currency = pair.currency.clone();
        je.header.exchange_rate = Decimal::ONE;
        je.header.created_by = "IC_GENERATOR".to_string();

        // Debit line: IC Receivable
        let mut debit_amount = pair.amount;
        if pair.withholding_tax.is_some() {
            debit_amount = pair.net_amount();
        }

        je.add_line(JournalEntryLine {
            line_number: 1,
            gl_account: self.get_seller_receivable_account(&pair.buyer_company),
            debit_amount,
            text: Some(format!("{} - {}", dr_desc, pair.description)),
            assignment: Some(pair.ic_reference.clone()),
            reference: Some(pair.buyer_document.clone()),
            ..Default::default()
        });

        // Credit line: Revenue/Income
        je.add_line(JournalEntryLine {
            line_number: 2,
            gl_account: self.get_seller_revenue_account(pair.transaction_type),
            credit_amount: pair.amount,
            text: Some(format!("{} - {}", cr_desc, pair.description)),
            assignment: Some(pair.ic_reference.clone()),
            ..Default::default()
        });

        // Add withholding tax line if applicable
        if let Some(wht) = pair.withholding_tax {
            je.add_line(JournalEntryLine {
                line_number: 3,
                gl_account: "2180".to_string(), // WHT payable
                credit_amount: wht,
                text: Some("Withholding tax on IC transaction".to_string()),
                assignment: Some(pair.ic_reference.clone()),
                ..Default::default()
            });
        }

        je
    }

    /// Create buyer-side journal entry.
    fn create_buyer_entry(
        &mut self,
        pair: &ICMatchedPair,
        _fiscal_year: i32,
        _fiscal_period: u32,
        dr_desc: &str,
        cr_desc: &str,
    ) -> JournalEntry {
        let mut je = JournalEntry::new_simple(
            pair.buyer_document.clone(),
            pair.buyer_company.clone(),
            pair.posting_date,
            format!(
                "IC {} from {}",
                pair.transaction_type.buyer_accounts().0,
                pair.seller_company
            ),
        );

        je.header.reference = Some(pair.ic_reference.clone());
        je.header.document_type = "IC".to_string();
        je.header.currency = pair.currency.clone();
        je.header.exchange_rate = Decimal::ONE;
        je.header.created_by = "IC_GENERATOR".to_string();

        // Debit line: Expense/Asset
        je.add_line(JournalEntryLine {
            line_number: 1,
            gl_account: self.get_buyer_expense_account(pair.transaction_type),
            debit_amount: pair.amount,
            cost_center: Some("CC100".to_string()),
            text: Some(format!("{} - {}", dr_desc, pair.description)),
            assignment: Some(pair.ic_reference.clone()),
            reference: Some(pair.seller_document.clone()),
            ..Default::default()
        });

        // Credit line: IC Payable
        je.add_line(JournalEntryLine {
            line_number: 2,
            gl_account: self.get_buyer_payable_account(&pair.seller_company),
            credit_amount: pair.amount,
            text: Some(format!("{} - {}", cr_desc, pair.description)),
            assignment: Some(pair.ic_reference.clone()),
            ..Default::default()
        });

        je
    }

    /// Get IC receivable account for seller.
    fn get_seller_receivable_account(&self, buyer_company: &str) -> String {
        format!("1310{}", &buyer_company[..buyer_company.len().min(2)])
    }

    /// Get IC revenue account for seller.
    fn get_seller_revenue_account(&self, tx_type: ICTransactionType) -> String {
        match tx_type {
            ICTransactionType::GoodsSale => "4100".to_string(),
            ICTransactionType::ServiceProvided => "4200".to_string(),
            ICTransactionType::ManagementFee => "4300".to_string(),
            ICTransactionType::Royalty => "4400".to_string(),
            ICTransactionType::LoanInterest => "4500".to_string(),
            ICTransactionType::Dividend => "4600".to_string(),
            _ => "4900".to_string(),
        }
    }

    /// Get IC expense account for buyer.
    fn get_buyer_expense_account(&self, tx_type: ICTransactionType) -> String {
        match tx_type {
            ICTransactionType::GoodsSale => "5100".to_string(),
            ICTransactionType::ServiceProvided => "5200".to_string(),
            ICTransactionType::ManagementFee => "5300".to_string(),
            ICTransactionType::Royalty => "5400".to_string(),
            ICTransactionType::LoanInterest => "5500".to_string(),
            ICTransactionType::Dividend => "3100".to_string(), // Retained earnings
            _ => "5900".to_string(),
        }
    }

    /// Get IC payable account for buyer.
    fn get_buyer_payable_account(&self, seller_company: &str) -> String {
        format!("2110{}", &seller_company[..seller_company.len().min(2)])
    }

    /// Generate an IC loan.
    pub fn generate_ic_loan(
        &mut self,
        lender: String,
        borrower: String,
        start_date: NaiveDate,
        term_months: u32,
    ) -> ICLoan {
        let (min_amount, max_amount) = self.config.loan_amount_range;
        let range = max_amount - min_amount;
        let random_factor = Decimal::from_f64_retain(self.rng.gen::<f64>()).unwrap_or(dec!(0.5));
        let principal = (min_amount + range * random_factor).round_dp(0);

        let (min_rate, max_rate) = self.config.loan_interest_rate_range;
        let rate_range = max_rate - min_rate;
        let rate_factor = Decimal::from_f64_retain(self.rng.gen::<f64>()).unwrap_or(dec!(0.5));
        let interest_rate = (min_rate + rate_range * rate_factor).round_dp(2);

        let maturity_date = start_date
            .checked_add_months(chrono::Months::new(term_months))
            .unwrap_or(start_date);

        let loan_id = format!(
            "LOAN{}{:04}",
            start_date.format("%Y"),
            self.active_loans.len() + 1
        );

        let loan = ICLoan::new(
            loan_id,
            lender,
            borrower,
            principal,
            "USD".to_string(),
            interest_rate,
            start_date,
            maturity_date,
        );

        self.active_loans.push(loan.clone());
        loan
    }

    /// Generate interest entries for active loans.
    pub fn generate_loan_interest_entries(
        &mut self,
        as_of_date: NaiveDate,
        fiscal_year: i32,
        fiscal_period: u32,
    ) -> Vec<(JournalEntry, JournalEntry)> {
        // Collect loan data to avoid borrow issues
        let loans_data: Vec<_> = self
            .active_loans
            .iter()
            .filter(|loan| !loan.is_repaid())
            .map(|loan| {
                let period_start = NaiveDate::from_ymd_opt(
                    if fiscal_period == 1 {
                        fiscal_year - 1
                    } else {
                        fiscal_year
                    },
                    if fiscal_period == 1 {
                        12
                    } else {
                        fiscal_period - 1
                    },
                    1,
                )
                .unwrap_or(as_of_date);

                let interest = loan.calculate_interest(period_start, as_of_date);
                (
                    loan.loan_id.clone(),
                    loan.lender_company.clone(),
                    loan.borrower_company.clone(),
                    loan.currency.clone(),
                    interest,
                )
            })
            .filter(|(_, _, _, _, interest)| *interest > Decimal::ZERO)
            .collect();

        let mut entries = Vec::new();

        for (loan_id, lender, borrower, currency, interest) in loans_data {
            let ic_ref = self.generate_ic_reference(as_of_date);
            let seller_doc = self.generate_doc_number("INT");
            let buyer_doc = self.generate_doc_number("INT");

            let mut pair = ICMatchedPair::new(
                ic_ref,
                ICTransactionType::LoanInterest,
                lender,
                borrower,
                interest,
                currency,
                as_of_date,
            );
            pair.seller_document = seller_doc;
            pair.buyer_document = buyer_doc;
            pair.description = format!("Interest on loan {}", loan_id);

            let (seller_je, buyer_je) =
                self.generate_journal_entries(&pair, fiscal_year, fiscal_period);
            entries.push((seller_je, buyer_je));
        }

        entries
    }

    /// Get all generated matched pairs.
    pub fn get_matched_pairs(&self) -> &[ICMatchedPair] {
        &self.matched_pairs
    }

    /// Get open (unsettled) matched pairs.
    pub fn get_open_pairs(&self) -> Vec<&ICMatchedPair> {
        self.matched_pairs.iter().filter(|p| p.is_open()).collect()
    }

    /// Get active loans.
    pub fn get_active_loans(&self) -> &[ICLoan] {
        &self.active_loans
    }

    /// Generate multiple IC transactions for a date range.
    pub fn generate_transactions_for_period(
        &mut self,
        start_date: NaiveDate,
        end_date: NaiveDate,
        transactions_per_day: usize,
    ) -> Vec<ICMatchedPair> {
        debug!(%start_date, %end_date, transactions_per_day, "Generating intercompany transactions");
        let mut pairs = Vec::new();
        let mut current_date = start_date;

        while current_date <= end_date {
            let fiscal_period = format!("{}{:02}", current_date.year(), current_date.month());

            for _ in 0..transactions_per_day {
                if let Some(pair) = self.generate_ic_transaction(current_date, &fiscal_period) {
                    pairs.push(pair);
                }
            }

            current_date = current_date.succ_opt().unwrap_or(current_date);
        }

        pairs
    }

    /// Reset counters (for testing).
    pub fn reset_counters(&mut self) {
        self.ic_counter = 0;
        self.doc_counter = 0;
        self.matched_pairs.clear();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use datasynth_core::models::intercompany::IntercompanyRelationship;
    use rust_decimal_macros::dec;

    fn create_test_ownership_structure() -> OwnershipStructure {
        let mut structure = OwnershipStructure::new("1000".to_string());
        structure.add_relationship(IntercompanyRelationship::new(
            "REL001".to_string(),
            "1000".to_string(),
            "1100".to_string(),
            dec!(100),
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        ));
        structure.add_relationship(IntercompanyRelationship::new(
            "REL002".to_string(),
            "1000".to_string(),
            "1200".to_string(),
            dec!(100),
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        ));
        structure
    }

    #[test]
    fn test_ic_generator_creation() {
        let config = ICGeneratorConfig::default();
        let structure = create_test_ownership_structure();
        let generator = ICGenerator::new(config, structure, 12345);

        assert!(generator.matched_pairs.is_empty());
        assert!(generator.active_loans.is_empty());
    }

    #[test]
    fn test_generate_ic_transaction() {
        let config = ICGeneratorConfig {
            ic_transaction_rate: 1.0, // Always generate
            ..Default::default()
        };

        let structure = create_test_ownership_structure();
        let mut generator = ICGenerator::new(config, structure, 12345);

        let date = NaiveDate::from_ymd_opt(2022, 6, 15).unwrap();
        let pair = generator.generate_ic_transaction(date, "202206");

        assert!(pair.is_some());
        let pair = pair.unwrap();
        assert!(!pair.ic_reference.is_empty());
        assert!(pair.amount > Decimal::ZERO);
    }

    #[test]
    fn test_generate_journal_entries() {
        let config = ICGeneratorConfig {
            ic_transaction_rate: 1.0,
            ..Default::default()
        };

        let structure = create_test_ownership_structure();
        let mut generator = ICGenerator::new(config, structure, 12345);

        let date = NaiveDate::from_ymd_opt(2022, 6, 15).unwrap();
        let pair = generator.generate_ic_transaction(date, "202206").unwrap();

        let (seller_je, buyer_je) = generator.generate_journal_entries(&pair, 2022, 6);

        assert_eq!(seller_je.company_code(), pair.seller_company);
        assert_eq!(buyer_je.company_code(), pair.buyer_company);
        assert_eq!(seller_je.header.reference, Some(pair.ic_reference.clone()));
        assert_eq!(buyer_je.header.reference, Some(pair.ic_reference));
    }

    #[test]
    fn test_generate_ic_loan() {
        let config = ICGeneratorConfig::default();
        let structure = create_test_ownership_structure();
        let mut generator = ICGenerator::new(config, structure, 12345);

        let loan = generator.generate_ic_loan(
            "1000".to_string(),
            "1100".to_string(),
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
            24,
        );

        assert!(!loan.loan_id.is_empty());
        assert!(loan.principal > Decimal::ZERO);
        assert!(loan.interest_rate > Decimal::ZERO);
        assert_eq!(generator.active_loans.len(), 1);
    }

    #[test]
    fn test_generate_transactions_for_period() {
        let config = ICGeneratorConfig {
            ic_transaction_rate: 1.0,
            ..Default::default()
        };

        let structure = create_test_ownership_structure();
        let mut generator = ICGenerator::new(config, structure, 12345);

        let start = NaiveDate::from_ymd_opt(2022, 6, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2022, 6, 5).unwrap();

        let pairs = generator.generate_transactions_for_period(start, end, 2);

        // 5 days * 2 transactions per day = 10 transactions
        assert_eq!(pairs.len(), 10);
    }
}
