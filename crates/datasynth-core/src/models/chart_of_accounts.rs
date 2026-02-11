//! Chart of Accounts structures for GL account management.
//!
//! Defines the hierarchical structure of financial accounts used in
//! the general ledger, including account classifications aligned with
//! standard financial reporting requirements.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Primary account type classification following standard financial statement structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    /// Assets - resources owned by the entity
    Asset,
    /// Liabilities - obligations owed to others
    Liability,
    /// Equity - residual interest in assets after deducting liabilities
    Equity,
    /// Revenue - income from operations
    Revenue,
    /// Expense - costs incurred in operations
    Expense,
    /// Statistical - non-financial tracking accounts
    Statistical,
}

impl AccountType {
    /// Returns true if this is a balance sheet account type.
    pub fn is_balance_sheet(&self) -> bool {
        matches!(self, Self::Asset | Self::Liability | Self::Equity)
    }

    /// Returns true if this is an income statement account type.
    pub fn is_income_statement(&self) -> bool {
        matches!(self, Self::Revenue | Self::Expense)
    }

    /// Returns the normal balance side (true = debit, false = credit).
    pub fn normal_debit_balance(&self) -> bool {
        matches!(self, Self::Asset | Self::Expense)
    }
}

/// Detailed sub-classification for accounts within each type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountSubType {
    // Assets
    /// Cash and cash equivalents
    Cash,
    /// Trade receivables
    AccountsReceivable,
    /// Other receivables
    OtherReceivables,
    /// Raw materials, WIP, finished goods
    Inventory,
    /// Prepaid expenses and deferred charges
    PrepaidExpenses,
    /// Property, plant and equipment
    FixedAssets,
    /// Contra-asset for depreciation
    AccumulatedDepreciation,
    /// Long-term investments
    Investments,
    /// Patents, trademarks, goodwill
    IntangibleAssets,
    /// Miscellaneous assets
    OtherAssets,

    // Liabilities
    /// Trade payables
    AccountsPayable,
    /// Accrued expenses
    AccruedLiabilities,
    /// Short-term borrowings
    ShortTermDebt,
    /// Long-term borrowings
    LongTermDebt,
    /// Unearned revenue
    DeferredRevenue,
    /// Current and deferred taxes payable
    TaxLiabilities,
    /// Pension and other post-employment benefits
    PensionLiabilities,
    /// Miscellaneous liabilities
    OtherLiabilities,

    // Equity
    /// Par value of shares issued
    CommonStock,
    /// Accumulated profits
    RetainedEarnings,
    /// Premium on share issuance
    AdditionalPaidInCapital,
    /// Repurchased shares
    TreasuryStock,
    /// Unrealized gains/losses
    OtherComprehensiveIncome,
    /// Current period profit/loss
    NetIncome,

    // Revenue
    /// Sales of products
    ProductRevenue,
    /// Sales of services
    ServiceRevenue,
    /// Interest earned
    InterestIncome,
    /// Dividends received
    DividendIncome,
    /// Gains on asset sales
    GainOnSale,
    /// Miscellaneous income
    OtherIncome,

    // Expense
    /// Direct costs of goods sold
    CostOfGoodsSold,
    /// General operating expenses
    OperatingExpenses,
    /// Sales and marketing costs
    SellingExpenses,
    /// G&A costs
    AdministrativeExpenses,
    /// Depreciation of fixed assets
    DepreciationExpense,
    /// Amortization of intangibles
    AmortizationExpense,
    /// Interest on borrowings
    InterestExpense,
    /// Income tax expense
    TaxExpense,
    /// Foreign exchange losses
    ForeignExchangeLoss,
    /// Losses on asset sales
    LossOnSale,
    /// Miscellaneous expenses
    OtherExpenses,

    // Suspense/Clearing
    /// Clearing accounts for temporary postings
    SuspenseClearing,
    /// GR/IR clearing
    GoodsReceivedClearing,
    /// Bank clearing accounts
    BankClearing,
    /// Intercompany clearing
    IntercompanyClearing,
}

impl AccountSubType {
    /// Get the parent account type for this sub-type.
    pub fn account_type(&self) -> AccountType {
        match self {
            Self::Cash
            | Self::AccountsReceivable
            | Self::OtherReceivables
            | Self::Inventory
            | Self::PrepaidExpenses
            | Self::FixedAssets
            | Self::AccumulatedDepreciation
            | Self::Investments
            | Self::IntangibleAssets
            | Self::OtherAssets => AccountType::Asset,

            Self::AccountsPayable
            | Self::AccruedLiabilities
            | Self::ShortTermDebt
            | Self::LongTermDebt
            | Self::DeferredRevenue
            | Self::TaxLiabilities
            | Self::PensionLiabilities
            | Self::OtherLiabilities => AccountType::Liability,

            Self::CommonStock
            | Self::RetainedEarnings
            | Self::AdditionalPaidInCapital
            | Self::TreasuryStock
            | Self::OtherComprehensiveIncome
            | Self::NetIncome => AccountType::Equity,

            Self::ProductRevenue
            | Self::ServiceRevenue
            | Self::InterestIncome
            | Self::DividendIncome
            | Self::GainOnSale
            | Self::OtherIncome => AccountType::Revenue,

            Self::CostOfGoodsSold
            | Self::OperatingExpenses
            | Self::SellingExpenses
            | Self::AdministrativeExpenses
            | Self::DepreciationExpense
            | Self::AmortizationExpense
            | Self::InterestExpense
            | Self::TaxExpense
            | Self::ForeignExchangeLoss
            | Self::LossOnSale
            | Self::OtherExpenses => AccountType::Expense,

            Self::SuspenseClearing
            | Self::GoodsReceivedClearing
            | Self::BankClearing
            | Self::IntercompanyClearing => AccountType::Asset, // Clearing accounts typically treated as assets
        }
    }

    /// Check if this is a suspense/clearing account type.
    pub fn is_suspense(&self) -> bool {
        matches!(
            self,
            Self::SuspenseClearing
                | Self::GoodsReceivedClearing
                | Self::BankClearing
                | Self::IntercompanyClearing
        )
    }
}

/// Industry sector for account relevance weighting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndustrySector {
    Manufacturing,
    Retail,
    FinancialServices,
    Healthcare,
    Technology,
    ProfessionalServices,
    Energy,
    Transportation,
    RealEstate,
    Telecommunications,
}

/// Industry relevance weights for account selection during generation.
///
/// Weights from 0.0 to 1.0 indicating how relevant an account is for each industry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IndustryWeights {
    pub manufacturing: f64,
    pub retail: f64,
    pub financial_services: f64,
    pub healthcare: f64,
    pub technology: f64,
    pub professional_services: f64,
    pub energy: f64,
    pub transportation: f64,
    pub real_estate: f64,
    pub telecommunications: f64,
}

impl IndustryWeights {
    /// Create weights where all industries have equal relevance.
    pub fn all_equal(weight: f64) -> Self {
        Self {
            manufacturing: weight,
            retail: weight,
            financial_services: weight,
            healthcare: weight,
            technology: weight,
            professional_services: weight,
            energy: weight,
            transportation: weight,
            real_estate: weight,
            telecommunications: weight,
        }
    }

    /// Get weight for a specific industry.
    pub fn get(&self, industry: IndustrySector) -> f64 {
        match industry {
            IndustrySector::Manufacturing => self.manufacturing,
            IndustrySector::Retail => self.retail,
            IndustrySector::FinancialServices => self.financial_services,
            IndustrySector::Healthcare => self.healthcare,
            IndustrySector::Technology => self.technology,
            IndustrySector::ProfessionalServices => self.professional_services,
            IndustrySector::Energy => self.energy,
            IndustrySector::Transportation => self.transportation,
            IndustrySector::RealEstate => self.real_estate,
            IndustrySector::Telecommunications => self.telecommunications,
        }
    }
}

/// Individual GL account definition.
///
/// Represents a single account in the chart of accounts with all necessary
/// metadata for realistic transaction generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GLAccount {
    /// Account number (e.g., "100000", "400100")
    pub account_number: String,

    /// Short description
    pub short_description: String,

    /// Long description
    pub long_description: String,

    /// Primary account type
    pub account_type: AccountType,

    /// Detailed sub-type classification
    pub sub_type: AccountSubType,

    /// Account class code (first digit typically)
    pub account_class: String,

    /// Account group for reporting
    pub account_group: String,

    /// Is this a control account (subledger summary)
    pub is_control_account: bool,

    /// Is this a suspense/clearing account
    pub is_suspense_account: bool,

    /// Parent account number (for hierarchies)
    pub parent_account: Option<String>,

    /// Account hierarchy level (1 = top level)
    pub hierarchy_level: u8,

    /// Normal balance side (true = debit, false = credit)
    pub normal_debit_balance: bool,

    /// Is posting allowed directly to this account
    pub is_postable: bool,

    /// Is this account blocked for posting
    pub is_blocked: bool,

    /// Allowed document types for this account
    pub allowed_doc_types: Vec<String>,

    /// Required cost center assignment
    pub requires_cost_center: bool,

    /// Required profit center assignment
    pub requires_profit_center: bool,

    /// Industry sector relevance scores (0.0-1.0)
    pub industry_weights: IndustryWeights,

    /// Typical transaction frequency (transactions per month)
    pub typical_frequency: f64,

    /// Typical transaction amount range (min, max)
    pub typical_amount_range: (f64, f64),
}

impl GLAccount {
    /// Create a new GL account with minimal required fields.
    pub fn new(
        account_number: String,
        description: String,
        account_type: AccountType,
        sub_type: AccountSubType,
    ) -> Self {
        Self {
            account_number: account_number.clone(),
            short_description: description.clone(),
            long_description: description,
            account_type,
            sub_type,
            account_class: account_number.chars().next().unwrap_or('0').to_string(),
            account_group: "DEFAULT".to_string(),
            is_control_account: false,
            is_suspense_account: sub_type.is_suspense(),
            parent_account: None,
            hierarchy_level: 1,
            normal_debit_balance: account_type.normal_debit_balance(),
            is_postable: true,
            is_blocked: false,
            allowed_doc_types: vec!["SA".to_string()],
            requires_cost_center: matches!(account_type, AccountType::Expense),
            requires_profit_center: false,
            industry_weights: IndustryWeights::all_equal(1.0),
            typical_frequency: 100.0,
            typical_amount_range: (100.0, 100000.0),
        }
    }

    /// Get account code (alias for account_number).
    pub fn account_code(&self) -> &str {
        &self.account_number
    }

    /// Get description (alias for short_description).
    pub fn description(&self) -> &str {
        &self.short_description
    }
}

/// Chart of Accounts complexity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoAComplexity {
    /// ~100 accounts - small business
    Small,
    /// ~400 accounts - mid-market
    Medium,
    /// ~2500 accounts - enterprise (based on paper's max observation)
    Large,
}

impl CoAComplexity {
    /// Get the target account count for this complexity level.
    pub fn target_count(&self) -> usize {
        match self {
            Self::Small => 100,
            Self::Medium => 400,
            Self::Large => 2500,
        }
    }
}

/// Complete Chart of Accounts structure.
///
/// Contains all GL accounts for an entity along with metadata about
/// the overall structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartOfAccounts {
    /// Unique identifier for this CoA
    pub coa_id: String,

    /// Name/description
    pub name: String,

    /// Country/region code
    pub country: String,

    /// Industry sector this CoA is designed for
    pub industry: IndustrySector,

    /// All accounts in this CoA
    pub accounts: Vec<GLAccount>,

    /// Complexity level
    pub complexity: CoAComplexity,

    /// Account number format (e.g., "######" for 6 digits)
    pub account_format: String,

    /// Index by account number for fast lookup
    #[serde(skip)]
    account_index: HashMap<String, usize>,
}

impl ChartOfAccounts {
    /// Create a new empty Chart of Accounts.
    pub fn new(
        coa_id: String,
        name: String,
        country: String,
        industry: IndustrySector,
        complexity: CoAComplexity,
    ) -> Self {
        Self {
            coa_id,
            name,
            country,
            industry,
            accounts: Vec::new(),
            complexity,
            account_format: "######".to_string(),
            account_index: HashMap::new(),
        }
    }

    /// Add an account to the CoA.
    pub fn add_account(&mut self, account: GLAccount) {
        let idx = self.accounts.len();
        self.account_index
            .insert(account.account_number.clone(), idx);
        self.accounts.push(account);
    }

    /// Rebuild the account index (call after deserialization).
    pub fn rebuild_index(&mut self) {
        self.account_index.clear();
        for (idx, account) in self.accounts.iter().enumerate() {
            self.account_index
                .insert(account.account_number.clone(), idx);
        }
    }

    /// Get an account by number.
    pub fn get_account(&self, account_number: &str) -> Option<&GLAccount> {
        self.account_index
            .get(account_number)
            .map(|&idx| &self.accounts[idx])
    }

    /// Get all postable accounts.
    pub fn get_postable_accounts(&self) -> Vec<&GLAccount> {
        self.accounts
            .iter()
            .filter(|a| a.is_postable && !a.is_blocked)
            .collect()
    }

    /// Get all accounts of a specific type.
    pub fn get_accounts_by_type(&self, account_type: AccountType) -> Vec<&GLAccount> {
        self.accounts
            .iter()
            .filter(|a| a.account_type == account_type && a.is_postable && !a.is_blocked)
            .collect()
    }

    /// Get all accounts of a specific sub-type.
    pub fn get_accounts_by_sub_type(&self, sub_type: AccountSubType) -> Vec<&GLAccount> {
        self.accounts
            .iter()
            .filter(|a| a.sub_type == sub_type && a.is_postable && !a.is_blocked)
            .collect()
    }

    /// Get suspense/clearing accounts.
    pub fn get_suspense_accounts(&self) -> Vec<&GLAccount> {
        self.accounts
            .iter()
            .filter(|a| a.is_suspense_account && a.is_postable)
            .collect()
    }

    /// Get accounts weighted by industry relevance.
    pub fn get_industry_weighted_accounts(
        &self,
        account_type: AccountType,
    ) -> Vec<(&GLAccount, f64)> {
        self.get_accounts_by_type(account_type)
            .into_iter()
            .map(|a| {
                let weight = a.industry_weights.get(self.industry);
                (a, weight)
            })
            .filter(|(_, w)| *w > 0.0)
            .collect()
    }

    /// Get total account count.
    pub fn account_count(&self) -> usize {
        self.accounts.len()
    }

    /// Get count of postable accounts.
    pub fn postable_count(&self) -> usize {
        self.accounts.iter().filter(|a| a.is_postable).count()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_account_type_balance() {
        assert!(AccountType::Asset.normal_debit_balance());
        assert!(AccountType::Expense.normal_debit_balance());
        assert!(!AccountType::Liability.normal_debit_balance());
        assert!(!AccountType::Revenue.normal_debit_balance());
        assert!(!AccountType::Equity.normal_debit_balance());
    }

    #[test]
    fn test_coa_complexity_count() {
        assert_eq!(CoAComplexity::Small.target_count(), 100);
        assert_eq!(CoAComplexity::Medium.target_count(), 400);
        assert_eq!(CoAComplexity::Large.target_count(), 2500);
    }

    #[test]
    fn test_coa_account_lookup() {
        let mut coa = ChartOfAccounts::new(
            "TEST".to_string(),
            "Test CoA".to_string(),
            "US".to_string(),
            IndustrySector::Manufacturing,
            CoAComplexity::Small,
        );

        coa.add_account(GLAccount::new(
            "100000".to_string(),
            "Cash".to_string(),
            AccountType::Asset,
            AccountSubType::Cash,
        ));

        assert!(coa.get_account("100000").is_some());
        assert!(coa.get_account("999999").is_none());
    }
}
