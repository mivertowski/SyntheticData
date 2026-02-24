//! Master data models for vendors and customers.
//!
//! Provides realistic vendor and customer entities for transaction
//! attribution and header/line text generation. Includes payment terms,
//! behavioral patterns, and intercompany support for enterprise simulation.

use rand::seq::SliceRandom;
use rand::Rng;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Payment terms for vendor/customer relationships.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PaymentTerms {
    /// Due immediately
    Immediate,
    /// Net 10 days
    Net10,
    /// Net 15 days
    Net15,
    /// Net 30 days (most common)
    #[default]
    Net30,
    /// Net 45 days
    Net45,
    /// Net 60 days
    Net60,
    /// Net 90 days
    Net90,
    /// 2% discount if paid within 10 days, otherwise net 30
    TwoTenNet30,
    /// 1% discount if paid within 10 days, otherwise net 30
    OneTenNet30,
    /// 2% discount if paid within 15 days, otherwise net 45
    TwoFifteenNet45,
    /// End of month
    EndOfMonth,
    /// End of month plus 30 days
    EndOfMonthPlus30,
    /// Cash on delivery
    CashOnDelivery,
    /// Prepayment required
    Prepayment,
}

impl PaymentTerms {
    /// Get the due date offset in days from invoice date.
    pub fn due_days(&self) -> u16 {
        match self {
            Self::Immediate | Self::CashOnDelivery => 0,
            Self::Prepayment => 0,
            Self::Net10 | Self::TwoTenNet30 | Self::OneTenNet30 => 30, // Final due date
            Self::Net15 | Self::TwoFifteenNet45 => 45,
            Self::Net30 => 30,
            Self::Net45 => 45,
            Self::Net60 => 60,
            Self::Net90 => 90,
            Self::EndOfMonth => 30,       // Approximate
            Self::EndOfMonthPlus30 => 60, // Approximate
        }
    }

    /// Get discount percentage if paid early.
    pub fn early_payment_discount(&self) -> Option<(u16, Decimal)> {
        match self {
            Self::TwoTenNet30 => Some((10, Decimal::from(2))),
            Self::OneTenNet30 => Some((10, Decimal::from(1))),
            Self::TwoFifteenNet45 => Some((15, Decimal::from(2))),
            _ => None,
        }
    }

    /// Check if this requires prepayment.
    pub fn requires_prepayment(&self) -> bool {
        matches!(self, Self::Prepayment | Self::CashOnDelivery)
    }

    /// Get the payment terms code (for display/export).
    pub fn code(&self) -> &'static str {
        match self {
            Self::Immediate => "IMM",
            Self::Net10 => "N10",
            Self::Net15 => "N15",
            Self::Net30 => "N30",
            Self::Net45 => "N45",
            Self::Net60 => "N60",
            Self::Net90 => "N90",
            Self::TwoTenNet30 => "2/10N30",
            Self::OneTenNet30 => "1/10N30",
            Self::TwoFifteenNet45 => "2/15N45",
            Self::EndOfMonth => "EOM",
            Self::EndOfMonthPlus30 => "EOM30",
            Self::CashOnDelivery => "COD",
            Self::Prepayment => "PREP",
        }
    }

    /// Get the net payment days.
    pub fn net_days(&self) -> u16 {
        self.due_days()
    }

    /// Get the discount days (days within which discount applies).
    pub fn discount_days(&self) -> Option<u16> {
        self.early_payment_discount().map(|(days, _)| days)
    }

    /// Get the discount percent.
    pub fn discount_percent(&self) -> Option<Decimal> {
        self.early_payment_discount().map(|(_, percent)| percent)
    }
}

/// Vendor payment behavior for simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VendorBehavior {
    /// Strict - expects payment exactly on due date
    Strict,
    /// Flexible - accepts some late payments
    #[default]
    Flexible,
    /// Very flexible - rarely follows up on late payments
    VeryFlexible,
    /// Aggressive - immediate follow-up on overdue
    Aggressive,
}

impl VendorBehavior {
    /// Get typical grace period in days beyond due date.
    pub fn grace_period_days(&self) -> u16 {
        match self {
            Self::Strict => 0,
            Self::Flexible => 7,
            Self::VeryFlexible => 30,
            Self::Aggressive => 0,
        }
    }
}

/// Customer payment behavior for simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CustomerPaymentBehavior {
    /// Excellent - always pays early or on time
    Excellent,
    /// Early payer (alias for Excellent)
    EarlyPayer,
    /// Good - usually pays on time
    #[default]
    Good,
    /// On time payer (alias for Good)
    OnTime,
    /// Fair - sometimes late
    Fair,
    /// Slightly late (alias for Fair)
    SlightlyLate,
    /// Poor - frequently late
    Poor,
    /// Often late (alias for Poor)
    OftenLate,
    /// Very Poor - chronically delinquent
    VeryPoor,
    /// High risk (alias for VeryPoor)
    HighRisk,
}

impl CustomerPaymentBehavior {
    /// Get average days past due for this behavior.
    pub fn average_days_past_due(&self) -> i16 {
        match self {
            Self::Excellent | Self::EarlyPayer => -5, // Pays early
            Self::Good | Self::OnTime => 0,
            Self::Fair | Self::SlightlyLate => 10,
            Self::Poor | Self::OftenLate => 30,
            Self::VeryPoor | Self::HighRisk => 60,
        }
    }

    /// Get probability of payment on time.
    pub fn on_time_probability(&self) -> f64 {
        match self {
            Self::Excellent | Self::EarlyPayer => 0.98,
            Self::Good | Self::OnTime => 0.90,
            Self::Fair | Self::SlightlyLate => 0.70,
            Self::Poor | Self::OftenLate => 0.40,
            Self::VeryPoor | Self::HighRisk => 0.20,
        }
    }

    /// Get probability of taking early payment discount.
    pub fn discount_probability(&self) -> f64 {
        match self {
            Self::Excellent | Self::EarlyPayer => 0.80,
            Self::Good | Self::OnTime => 0.50,
            Self::Fair | Self::SlightlyLate => 0.20,
            Self::Poor | Self::OftenLate => 0.05,
            Self::VeryPoor | Self::HighRisk => 0.01,
        }
    }
}

/// Customer credit rating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CreditRating {
    /// Excellent credit
    AAA,
    /// Very good credit
    AA,
    /// Good credit
    #[default]
    A,
    /// Satisfactory credit
    BBB,
    /// Fair credit
    BB,
    /// Marginal credit
    B,
    /// Poor credit
    CCC,
    /// Very poor credit
    CC,
    /// Extremely poor credit
    C,
    /// Default/no credit
    D,
}

impl CreditRating {
    /// Get credit limit multiplier for this rating.
    pub fn credit_limit_multiplier(&self) -> Decimal {
        match self {
            Self::AAA => Decimal::from(5),
            Self::AA => Decimal::from(4),
            Self::A => Decimal::from(3),
            Self::BBB => Decimal::from(2),
            Self::BB => Decimal::from_str_exact("1.5").unwrap_or(Decimal::from(1)),
            Self::B => Decimal::from(1),
            Self::CCC => Decimal::from_str_exact("0.5").unwrap_or(Decimal::from(1)),
            Self::CC => Decimal::from_str_exact("0.25").unwrap_or(Decimal::from(0)),
            Self::C => Decimal::from_str_exact("0.1").unwrap_or(Decimal::from(0)),
            Self::D => Decimal::ZERO,
        }
    }

    /// Check if credit should be blocked.
    pub fn is_credit_blocked(&self) -> bool {
        matches!(self, Self::D)
    }
}

/// Bank account information for payments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankAccount {
    /// Bank name
    pub bank_name: String,
    /// Bank country
    pub bank_country: String,
    /// Account number or IBAN
    pub account_number: String,
    /// Routing number / BIC / SWIFT
    pub routing_code: String,
    /// Account holder name
    pub holder_name: String,
    /// Is this the primary account?
    pub is_primary: bool,
}

impl BankAccount {
    /// Create a new bank account.
    pub fn new(
        bank_name: impl Into<String>,
        account_number: impl Into<String>,
        routing_code: impl Into<String>,
        holder_name: impl Into<String>,
    ) -> Self {
        Self {
            bank_name: bank_name.into(),
            bank_country: "US".to_string(),
            account_number: account_number.into(),
            routing_code: routing_code.into(),
            holder_name: holder_name.into(),
            is_primary: true,
        }
    }
}

/// Type of vendor relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VendorType {
    /// General supplier of goods
    #[default]
    Supplier,
    /// Service provider
    ServiceProvider,
    /// Utility company
    Utility,
    /// Professional services (legal, accounting, consulting)
    ProfessionalServices,
    /// Technology/software vendor
    Technology,
    /// Logistics/shipping
    Logistics,
    /// Contractor/freelancer
    Contractor,
    /// Landlord/property management
    RealEstate,
    /// Financial services
    Financial,
    /// Employee expense reimbursement
    EmployeeReimbursement,
}

impl VendorType {
    /// Get typical expense categories for this vendor type.
    pub fn typical_expense_categories(&self) -> &'static [&'static str] {
        match self {
            Self::Supplier => &["Materials", "Inventory", "Office Supplies", "Equipment"],
            Self::ServiceProvider => &["Services", "Maintenance", "Support"],
            Self::Utility => &["Electricity", "Gas", "Water", "Telecommunications"],
            Self::ProfessionalServices => &["Legal", "Audit", "Consulting", "Tax Services"],
            Self::Technology => &["Software", "Licenses", "Cloud Services", "IT Support"],
            Self::Logistics => &["Freight", "Shipping", "Warehousing", "Customs"],
            Self::Contractor => &["Contract Labor", "Professional Fees", "Consulting"],
            Self::RealEstate => &["Rent", "Property Management", "Facilities"],
            Self::Financial => &["Bank Fees", "Interest", "Insurance", "Financing Costs"],
            Self::EmployeeReimbursement => &["Travel", "Meals", "Entertainment", "Expenses"],
        }
    }
}

/// Vendor master data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vendor {
    /// Vendor ID (e.g., "V-001234")
    pub vendor_id: String,

    /// Vendor name
    pub name: String,

    /// Type of vendor
    pub vendor_type: VendorType,

    /// Country code (ISO 3166-1 alpha-2)
    pub country: String,

    /// Payment terms (structured)
    pub payment_terms: PaymentTerms,

    /// Payment terms in days (legacy, computed from payment_terms)
    pub payment_terms_days: u8,

    /// Typical invoice amount range (min, max)
    pub typical_amount_range: (Decimal, Decimal),

    /// Is this vendor active
    pub is_active: bool,

    /// Vendor account number in sub-ledger
    pub account_number: Option<String>,

    /// Tax ID / VAT number
    pub tax_id: Option<String>,

    /// Bank accounts for payment
    pub bank_accounts: Vec<BankAccount>,

    /// Is this an intercompany vendor?
    pub is_intercompany: bool,

    /// Related company code (if intercompany)
    pub intercompany_code: Option<String>,

    /// Vendor behavior for payment follow-up
    pub behavior: VendorBehavior,

    /// Currency for transactions
    pub currency: String,

    /// Reconciliation account in GL
    pub reconciliation_account: Option<String>,

    /// Withholding tax applicable
    pub withholding_tax_applicable: bool,

    /// Withholding tax rate
    pub withholding_tax_rate: Option<Decimal>,

    /// One-time vendor (no master data)
    pub is_one_time: bool,

    /// Purchasing organization
    pub purchasing_org: Option<String>,

    /// French GAAP: compte auxiliaire (tier-specific) e.g. 4010001, 4010002.
    #[serde(default)]
    pub auxiliary_gl_account: Option<String>,
}

impl Vendor {
    /// Create a new vendor.
    pub fn new(vendor_id: &str, name: &str, vendor_type: VendorType) -> Self {
        Self {
            vendor_id: vendor_id.to_string(),
            name: name.to_string(),
            vendor_type,
            country: "US".to_string(),
            payment_terms: PaymentTerms::Net30,
            payment_terms_days: 30,
            typical_amount_range: (Decimal::from(100), Decimal::from(10000)),
            is_active: true,
            account_number: None,
            tax_id: None,
            bank_accounts: Vec::new(),
            is_intercompany: false,
            intercompany_code: None,
            behavior: VendorBehavior::default(),
            currency: "USD".to_string(),
            reconciliation_account: None,
            withholding_tax_applicable: false,
            withholding_tax_rate: None,
            is_one_time: false,
            purchasing_org: None,
            auxiliary_gl_account: None,
        }
    }

    /// Create an intercompany vendor.
    pub fn new_intercompany(vendor_id: &str, name: &str, related_company_code: &str) -> Self {
        Self::new(vendor_id, name, VendorType::Supplier).with_intercompany(related_company_code)
    }

    /// Set country.
    pub fn with_country(mut self, country: &str) -> Self {
        self.country = country.to_string();
        self
    }

    /// Set structured payment terms.
    pub fn with_payment_terms_structured(mut self, terms: PaymentTerms) -> Self {
        self.payment_terms = terms;
        self.payment_terms_days = terms.due_days() as u8;
        self
    }

    /// Set payment terms (legacy, by days).
    pub fn with_payment_terms(mut self, days: u8) -> Self {
        self.payment_terms_days = days;
        // Map to closest structured terms
        self.payment_terms = match days {
            0 => PaymentTerms::Immediate,
            1..=15 => PaymentTerms::Net15,
            16..=35 => PaymentTerms::Net30,
            36..=50 => PaymentTerms::Net45,
            51..=70 => PaymentTerms::Net60,
            _ => PaymentTerms::Net90,
        };
        self
    }

    /// Set amount range.
    pub fn with_amount_range(mut self, min: Decimal, max: Decimal) -> Self {
        self.typical_amount_range = (min, max);
        self
    }

    /// Set as intercompany vendor.
    pub fn with_intercompany(mut self, related_company_code: &str) -> Self {
        self.is_intercompany = true;
        self.intercompany_code = Some(related_company_code.to_string());
        self
    }

    /// Add a bank account.
    pub fn with_bank_account(mut self, account: BankAccount) -> Self {
        self.bank_accounts.push(account);
        self
    }

    /// Set vendor behavior.
    pub fn with_behavior(mut self, behavior: VendorBehavior) -> Self {
        self.behavior = behavior;
        self
    }

    /// Set currency.
    pub fn with_currency(mut self, currency: &str) -> Self {
        self.currency = currency.to_string();
        self
    }

    /// Set reconciliation account.
    pub fn with_reconciliation_account(mut self, account: &str) -> Self {
        self.reconciliation_account = Some(account.to_string());
        self
    }

    /// Set withholding tax.
    pub fn with_withholding_tax(mut self, rate: Decimal) -> Self {
        self.withholding_tax_applicable = true;
        self.withholding_tax_rate = Some(rate);
        self
    }

    /// Get the primary bank account.
    pub fn primary_bank_account(&self) -> Option<&BankAccount> {
        self.bank_accounts
            .iter()
            .find(|a| a.is_primary)
            .or_else(|| self.bank_accounts.first())
    }

    /// Generate a random amount within the typical range.
    pub fn generate_amount(&self, rng: &mut impl Rng) -> Decimal {
        let (min, max) = self.typical_amount_range;
        let range = max - min;
        let random_fraction = Decimal::from_f64_retain(rng.gen::<f64>()).unwrap_or(Decimal::ZERO);
        min + range * random_fraction
    }

    /// Calculate due date for an invoice.
    pub fn calculate_due_date(&self, invoice_date: chrono::NaiveDate) -> chrono::NaiveDate {
        invoice_date + chrono::Duration::days(self.payment_terms.due_days() as i64)
    }
}

/// Type of customer relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CustomerType {
    /// Business-to-business customer
    #[default]
    Corporate,
    /// Small/medium business
    SmallBusiness,
    /// Individual consumer
    Consumer,
    /// Government entity
    Government,
    /// Non-profit organization
    NonProfit,
    /// Intercompany (related party)
    Intercompany,
    /// Distributor/reseller
    Distributor,
}

/// Customer master data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    /// Customer ID (e.g., "C-001234")
    pub customer_id: String,

    /// Customer name
    pub name: String,

    /// Type of customer
    pub customer_type: CustomerType,

    /// Country code (ISO 3166-1 alpha-2)
    pub country: String,

    /// Credit rating
    pub credit_rating: CreditRating,

    /// Credit limit
    #[serde(with = "rust_decimal::serde::str")]
    pub credit_limit: Decimal,

    /// Current credit exposure (outstanding AR)
    #[serde(with = "rust_decimal::serde::str")]
    pub credit_exposure: Decimal,

    /// Payment terms (structured)
    pub payment_terms: PaymentTerms,

    /// Payment terms in days (legacy)
    pub payment_terms_days: u8,

    /// Payment behavior pattern
    pub payment_behavior: CustomerPaymentBehavior,

    /// Is this customer active
    pub is_active: bool,

    /// Customer account number in sub-ledger
    pub account_number: Option<String>,

    /// Typical order amount range (min, max)
    pub typical_order_range: (Decimal, Decimal),

    /// Is this an intercompany customer?
    pub is_intercompany: bool,

    /// Related company code (if intercompany)
    pub intercompany_code: Option<String>,

    /// Currency for transactions
    pub currency: String,

    /// Reconciliation account in GL
    pub reconciliation_account: Option<String>,

    /// Sales organization
    pub sales_org: Option<String>,

    /// Distribution channel
    pub distribution_channel: Option<String>,

    /// Tax ID / VAT number
    pub tax_id: Option<String>,

    /// Is credit blocked?
    pub credit_blocked: bool,

    /// Credit block reason
    pub credit_block_reason: Option<String>,

    /// Dunning procedure
    pub dunning_procedure: Option<String>,

    /// Last dunning date
    pub last_dunning_date: Option<chrono::NaiveDate>,

    /// Dunning level (0-4)
    pub dunning_level: u8,

    /// French GAAP: compte auxiliaire (tier-specific) e.g. 4110001, 4110002.
    #[serde(default)]
    pub auxiliary_gl_account: Option<String>,
}

impl Customer {
    /// Create a new customer.
    pub fn new(customer_id: &str, name: &str, customer_type: CustomerType) -> Self {
        Self {
            customer_id: customer_id.to_string(),
            name: name.to_string(),
            customer_type,
            country: "US".to_string(),
            credit_rating: CreditRating::default(),
            credit_limit: Decimal::from(100000),
            credit_exposure: Decimal::ZERO,
            payment_terms: PaymentTerms::Net30,
            payment_terms_days: 30,
            payment_behavior: CustomerPaymentBehavior::default(),
            is_active: true,
            account_number: None,
            typical_order_range: (Decimal::from(500), Decimal::from(50000)),
            is_intercompany: false,
            intercompany_code: None,
            currency: "USD".to_string(),
            reconciliation_account: None,
            sales_org: None,
            distribution_channel: None,
            tax_id: None,
            credit_blocked: false,
            credit_block_reason: None,
            dunning_procedure: None,
            last_dunning_date: None,
            dunning_level: 0,
            auxiliary_gl_account: None,
        }
    }

    /// Create an intercompany customer.
    pub fn new_intercompany(customer_id: &str, name: &str, related_company_code: &str) -> Self {
        Self::new(customer_id, name, CustomerType::Intercompany)
            .with_intercompany(related_company_code)
    }

    /// Set country.
    pub fn with_country(mut self, country: &str) -> Self {
        self.country = country.to_string();
        self
    }

    /// Set credit rating.
    pub fn with_credit_rating(mut self, rating: CreditRating) -> Self {
        self.credit_rating = rating;
        // Adjust credit limit based on rating
        self.credit_limit *= rating.credit_limit_multiplier();
        if rating.is_credit_blocked() {
            self.credit_blocked = true;
            self.credit_block_reason = Some("Credit rating D".to_string());
        }
        self
    }

    /// Set credit limit.
    pub fn with_credit_limit(mut self, limit: Decimal) -> Self {
        self.credit_limit = limit;
        self
    }

    /// Set structured payment terms.
    pub fn with_payment_terms_structured(mut self, terms: PaymentTerms) -> Self {
        self.payment_terms = terms;
        self.payment_terms_days = terms.due_days() as u8;
        self
    }

    /// Set payment terms (legacy, by days).
    pub fn with_payment_terms(mut self, days: u8) -> Self {
        self.payment_terms_days = days;
        self.payment_terms = match days {
            0 => PaymentTerms::Immediate,
            1..=15 => PaymentTerms::Net15,
            16..=35 => PaymentTerms::Net30,
            36..=50 => PaymentTerms::Net45,
            51..=70 => PaymentTerms::Net60,
            _ => PaymentTerms::Net90,
        };
        self
    }

    /// Set payment behavior.
    pub fn with_payment_behavior(mut self, behavior: CustomerPaymentBehavior) -> Self {
        self.payment_behavior = behavior;
        self
    }

    /// Set as intercompany customer.
    pub fn with_intercompany(mut self, related_company_code: &str) -> Self {
        self.is_intercompany = true;
        self.intercompany_code = Some(related_company_code.to_string());
        self.customer_type = CustomerType::Intercompany;
        // Intercompany customers typically have excellent credit
        self.credit_rating = CreditRating::AAA;
        self.payment_behavior = CustomerPaymentBehavior::Excellent;
        self
    }

    /// Set currency.
    pub fn with_currency(mut self, currency: &str) -> Self {
        self.currency = currency.to_string();
        self
    }

    /// Set sales organization.
    pub fn with_sales_org(mut self, org: &str) -> Self {
        self.sales_org = Some(org.to_string());
        self
    }

    /// Block credit.
    pub fn block_credit(&mut self, reason: &str) {
        self.credit_blocked = true;
        self.credit_block_reason = Some(reason.to_string());
    }

    /// Unblock credit.
    pub fn unblock_credit(&mut self) {
        self.credit_blocked = false;
        self.credit_block_reason = None;
    }

    /// Check if order can be placed (credit check).
    pub fn can_place_order(&self, order_amount: Decimal) -> bool {
        if self.credit_blocked {
            return false;
        }
        if !self.is_active {
            return false;
        }
        // Check credit limit
        self.credit_exposure + order_amount <= self.credit_limit
    }

    /// Available credit.
    pub fn available_credit(&self) -> Decimal {
        if self.credit_blocked {
            Decimal::ZERO
        } else {
            (self.credit_limit - self.credit_exposure).max(Decimal::ZERO)
        }
    }

    /// Update credit exposure.
    pub fn add_credit_exposure(&mut self, amount: Decimal) {
        self.credit_exposure += amount;
    }

    /// Reduce credit exposure (payment received).
    pub fn reduce_credit_exposure(&mut self, amount: Decimal) {
        self.credit_exposure = (self.credit_exposure - amount).max(Decimal::ZERO);
    }

    /// Generate a random order amount within typical range.
    pub fn generate_order_amount(&self, rng: &mut impl Rng) -> Decimal {
        let (min, max) = self.typical_order_range;
        let range = max - min;
        let random_fraction = Decimal::from_f64_retain(rng.gen::<f64>()).unwrap_or(Decimal::ZERO);
        min + range * random_fraction
    }

    /// Calculate due date for an invoice.
    pub fn calculate_due_date(&self, invoice_date: chrono::NaiveDate) -> chrono::NaiveDate {
        invoice_date + chrono::Duration::days(self.payment_terms.due_days() as i64)
    }

    /// Simulate payment date based on payment behavior.
    pub fn simulate_payment_date(
        &self,
        due_date: chrono::NaiveDate,
        rng: &mut impl Rng,
    ) -> chrono::NaiveDate {
        let days_offset = self.payment_behavior.average_days_past_due();
        // Add some random variation
        let variation: i16 = rng.gen_range(-5..=10);
        let total_offset = days_offset + variation;
        due_date + chrono::Duration::days(total_offset as i64)
    }
}

/// Pool of vendors for transaction generation.
#[derive(Debug, Clone, Default)]
pub struct VendorPool {
    /// All vendors
    pub vendors: Vec<Vendor>,
    /// Index by vendor type
    type_index: HashMap<VendorType, Vec<usize>>,
}

impl VendorPool {
    /// Create a new empty vendor pool.
    pub fn new() -> Self {
        Self {
            vendors: Vec::new(),
            type_index: HashMap::new(),
        }
    }

    /// Create a vendor pool from a vector of vendors.
    ///
    /// This is the preferred way to create a pool from generated master data,
    /// ensuring JEs reference real entities.
    pub fn from_vendors(vendors: Vec<Vendor>) -> Self {
        let mut pool = Self::new();
        for vendor in vendors {
            pool.add_vendor(vendor);
        }
        pool
    }

    /// Add a vendor to the pool.
    pub fn add_vendor(&mut self, vendor: Vendor) {
        let idx = self.vendors.len();
        let vendor_type = vendor.vendor_type;
        self.vendors.push(vendor);
        self.type_index.entry(vendor_type).or_default().push(idx);
    }

    /// Get a random vendor.
    pub fn random_vendor(&self, rng: &mut impl Rng) -> Option<&Vendor> {
        self.vendors.choose(rng)
    }

    /// Get a random vendor of a specific type.
    pub fn random_vendor_of_type(
        &self,
        vendor_type: VendorType,
        rng: &mut impl Rng,
    ) -> Option<&Vendor> {
        self.type_index
            .get(&vendor_type)
            .and_then(|indices| indices.choose(rng))
            .map(|&idx| &self.vendors[idx])
    }

    /// Rebuild the type index (call after deserialization).
    pub fn rebuild_index(&mut self) {
        self.type_index.clear();
        for (idx, vendor) in self.vendors.iter().enumerate() {
            self.type_index
                .entry(vendor.vendor_type)
                .or_default()
                .push(idx);
        }
    }

    /// Generate a standard vendor pool with realistic vendors.
    pub fn standard() -> Self {
        let mut pool = Self::new();

        // Suppliers
        let suppliers = [
            ("V-000001", "Acme Supplies Inc", VendorType::Supplier),
            ("V-000002", "Global Materials Corp", VendorType::Supplier),
            ("V-000003", "Office Depot Business", VendorType::Supplier),
            ("V-000004", "Industrial Parts Co", VendorType::Supplier),
            ("V-000005", "Premium Components Ltd", VendorType::Supplier),
        ];

        // Service providers
        let services = [
            ("V-000010", "CleanCo Services", VendorType::ServiceProvider),
            (
                "V-000011",
                "Building Maintenance Inc",
                VendorType::ServiceProvider,
            ),
            (
                "V-000012",
                "Security Solutions LLC",
                VendorType::ServiceProvider,
            ),
        ];

        // Utilities
        let utilities = [
            ("V-000020", "City Electric Utility", VendorType::Utility),
            ("V-000021", "Natural Gas Co", VendorType::Utility),
            ("V-000022", "Metro Water Authority", VendorType::Utility),
            ("V-000023", "Telecom Network Inc", VendorType::Utility),
        ];

        // Professional services
        let professional = [
            (
                "V-000030",
                "Baker & Associates LLP",
                VendorType::ProfessionalServices,
            ),
            (
                "V-000031",
                "PricewaterhouseCoopers",
                VendorType::ProfessionalServices,
            ),
            (
                "V-000032",
                "McKinsey & Company",
                VendorType::ProfessionalServices,
            ),
            (
                "V-000033",
                "Deloitte Consulting",
                VendorType::ProfessionalServices,
            ),
        ];

        // Technology
        let technology = [
            ("V-000040", "Microsoft Corporation", VendorType::Technology),
            ("V-000041", "Amazon Web Services", VendorType::Technology),
            ("V-000042", "Salesforce Inc", VendorType::Technology),
            ("V-000043", "SAP America Inc", VendorType::Technology),
            ("V-000044", "Oracle Corporation", VendorType::Technology),
            ("V-000045", "Adobe Systems", VendorType::Technology),
        ];

        // Logistics
        let logistics = [
            ("V-000050", "FedEx Corporation", VendorType::Logistics),
            ("V-000051", "UPS Shipping", VendorType::Logistics),
            ("V-000052", "DHL Express", VendorType::Logistics),
        ];

        // Real estate
        let real_estate = [
            (
                "V-000060",
                "Commercial Properties LLC",
                VendorType::RealEstate,
            ),
            ("V-000061", "CBRE Group", VendorType::RealEstate),
        ];

        // Add all vendors
        for (id, name, vtype) in suppliers {
            pool.add_vendor(
                Vendor::new(id, name, vtype)
                    .with_amount_range(Decimal::from(500), Decimal::from(50000)),
            );
        }

        for (id, name, vtype) in services {
            pool.add_vendor(
                Vendor::new(id, name, vtype)
                    .with_amount_range(Decimal::from(200), Decimal::from(5000)),
            );
        }

        for (id, name, vtype) in utilities {
            pool.add_vendor(
                Vendor::new(id, name, vtype)
                    .with_amount_range(Decimal::from(500), Decimal::from(20000)),
            );
        }

        for (id, name, vtype) in professional {
            pool.add_vendor(
                Vendor::new(id, name, vtype)
                    .with_amount_range(Decimal::from(5000), Decimal::from(500000)),
            );
        }

        for (id, name, vtype) in technology {
            pool.add_vendor(
                Vendor::new(id, name, vtype)
                    .with_amount_range(Decimal::from(100), Decimal::from(100000)),
            );
        }

        for (id, name, vtype) in logistics {
            pool.add_vendor(
                Vendor::new(id, name, vtype)
                    .with_amount_range(Decimal::from(50), Decimal::from(10000)),
            );
        }

        for (id, name, vtype) in real_estate {
            pool.add_vendor(
                Vendor::new(id, name, vtype)
                    .with_amount_range(Decimal::from(5000), Decimal::from(100000)),
            );
        }

        pool
    }
}

/// Pool of customers for transaction generation.
#[derive(Debug, Clone, Default)]
pub struct CustomerPool {
    /// All customers
    pub customers: Vec<Customer>,
    /// Index by customer type
    type_index: HashMap<CustomerType, Vec<usize>>,
}

impl CustomerPool {
    /// Create a new empty customer pool.
    pub fn new() -> Self {
        Self {
            customers: Vec::new(),
            type_index: HashMap::new(),
        }
    }

    /// Create a customer pool from a vector of customers.
    ///
    /// This is the preferred way to create a pool from generated master data,
    /// ensuring JEs reference real entities.
    pub fn from_customers(customers: Vec<Customer>) -> Self {
        let mut pool = Self::new();
        for customer in customers {
            pool.add_customer(customer);
        }
        pool
    }

    /// Add a customer to the pool.
    pub fn add_customer(&mut self, customer: Customer) {
        let idx = self.customers.len();
        let customer_type = customer.customer_type;
        self.customers.push(customer);
        self.type_index.entry(customer_type).or_default().push(idx);
    }

    /// Get a random customer.
    pub fn random_customer(&self, rng: &mut impl Rng) -> Option<&Customer> {
        self.customers.choose(rng)
    }

    /// Get a random customer of a specific type.
    pub fn random_customer_of_type(
        &self,
        customer_type: CustomerType,
        rng: &mut impl Rng,
    ) -> Option<&Customer> {
        self.type_index
            .get(&customer_type)
            .and_then(|indices| indices.choose(rng))
            .map(|&idx| &self.customers[idx])
    }

    /// Rebuild the type index.
    pub fn rebuild_index(&mut self) {
        self.type_index.clear();
        for (idx, customer) in self.customers.iter().enumerate() {
            self.type_index
                .entry(customer.customer_type)
                .or_default()
                .push(idx);
        }
    }

    /// Generate a standard customer pool.
    pub fn standard() -> Self {
        let mut pool = Self::new();

        // Corporate customers
        let corporate = [
            ("C-000001", "Northwind Traders", CustomerType::Corporate),
            ("C-000002", "Contoso Corporation", CustomerType::Corporate),
            ("C-000003", "Adventure Works", CustomerType::Corporate),
            ("C-000004", "Fabrikam Industries", CustomerType::Corporate),
            ("C-000005", "Wide World Importers", CustomerType::Corporate),
            ("C-000006", "Tailspin Toys", CustomerType::Corporate),
            ("C-000007", "Proseware Inc", CustomerType::Corporate),
            ("C-000008", "Coho Vineyard", CustomerType::Corporate),
            ("C-000009", "Alpine Ski House", CustomerType::Corporate),
            ("C-000010", "VanArsdel Ltd", CustomerType::Corporate),
        ];

        // Small business
        let small_business = [
            ("C-000020", "Smith & Co LLC", CustomerType::SmallBusiness),
            (
                "C-000021",
                "Johnson Enterprises",
                CustomerType::SmallBusiness,
            ),
            (
                "C-000022",
                "Williams Consulting",
                CustomerType::SmallBusiness,
            ),
            (
                "C-000023",
                "Brown Brothers Shop",
                CustomerType::SmallBusiness,
            ),
            (
                "C-000024",
                "Davis Family Business",
                CustomerType::SmallBusiness,
            ),
        ];

        // Government
        let government = [
            (
                "C-000030",
                "US Federal Government",
                CustomerType::Government,
            ),
            ("C-000031", "State of California", CustomerType::Government),
            ("C-000032", "City of New York", CustomerType::Government),
        ];

        // Distributors
        let distributors = [
            (
                "C-000040",
                "National Distribution Co",
                CustomerType::Distributor,
            ),
            (
                "C-000041",
                "Regional Wholesale Inc",
                CustomerType::Distributor,
            ),
            (
                "C-000042",
                "Pacific Distributors",
                CustomerType::Distributor,
            ),
        ];

        for (id, name, ctype) in corporate {
            pool.add_customer(
                Customer::new(id, name, ctype).with_credit_limit(Decimal::from(500000)),
            );
        }

        for (id, name, ctype) in small_business {
            pool.add_customer(
                Customer::new(id, name, ctype).with_credit_limit(Decimal::from(50000)),
            );
        }

        for (id, name, ctype) in government {
            pool.add_customer(
                Customer::new(id, name, ctype)
                    .with_credit_limit(Decimal::from(1000000))
                    .with_payment_terms(45),
            );
        }

        for (id, name, ctype) in distributors {
            pool.add_customer(
                Customer::new(id, name, ctype).with_credit_limit(Decimal::from(250000)),
            );
        }

        pool
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_vendor_creation() {
        let vendor = Vendor::new("V-001", "Test Vendor", VendorType::Supplier)
            .with_country("DE")
            .with_payment_terms(45);

        assert_eq!(vendor.vendor_id, "V-001");
        assert_eq!(vendor.country, "DE");
        assert_eq!(vendor.payment_terms_days, 45);
    }

    #[test]
    fn test_vendor_pool() {
        let pool = VendorPool::standard();

        assert!(!pool.vendors.is_empty());

        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let vendor = pool.random_vendor(&mut rng);
        assert!(vendor.is_some());

        let tech_vendor = pool.random_vendor_of_type(VendorType::Technology, &mut rng);
        assert!(tech_vendor.is_some());
    }

    #[test]
    fn test_customer_pool() {
        let pool = CustomerPool::standard();

        assert!(!pool.customers.is_empty());

        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let customer = pool.random_customer(&mut rng);
        assert!(customer.is_some());
    }

    #[test]
    fn test_amount_generation() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let vendor = Vendor::new("V-001", "Test", VendorType::Supplier)
            .with_amount_range(Decimal::from(100), Decimal::from(1000));

        let amount = vendor.generate_amount(&mut rng);
        assert!(amount >= Decimal::from(100));
        assert!(amount <= Decimal::from(1000));
    }

    #[test]
    fn test_payment_terms() {
        assert_eq!(PaymentTerms::Net30.due_days(), 30);
        assert_eq!(PaymentTerms::Net60.due_days(), 60);
        assert!(PaymentTerms::Prepayment.requires_prepayment());

        let discount = PaymentTerms::TwoTenNet30.early_payment_discount();
        assert!(discount.is_some());
        let (days, percent) = discount.unwrap();
        assert_eq!(days, 10);
        assert_eq!(percent, Decimal::from(2));
    }

    #[test]
    fn test_credit_rating() {
        assert!(
            CreditRating::AAA.credit_limit_multiplier() > CreditRating::B.credit_limit_multiplier()
        );
        assert!(CreditRating::D.is_credit_blocked());
        assert!(!CreditRating::A.is_credit_blocked());
    }

    #[test]
    fn test_customer_credit_check() {
        let mut customer = Customer::new("C-001", "Test", CustomerType::Corporate)
            .with_credit_limit(Decimal::from(10000));

        // Should be able to place order within limit
        assert!(customer.can_place_order(Decimal::from(5000)));

        // Add some exposure
        customer.add_credit_exposure(Decimal::from(8000));

        // Now should fail for large order
        assert!(!customer.can_place_order(Decimal::from(5000)));

        // But small order should work
        assert!(customer.can_place_order(Decimal::from(2000)));

        // Block credit
        customer.block_credit("Testing");
        assert!(!customer.can_place_order(Decimal::from(100)));
    }

    #[test]
    fn test_intercompany_vendor() {
        let vendor = Vendor::new_intercompany("V-IC-001", "Subsidiary Co", "2000");

        assert!(vendor.is_intercompany);
        assert_eq!(vendor.intercompany_code, Some("2000".to_string()));
    }

    #[test]
    fn test_intercompany_customer() {
        let customer = Customer::new_intercompany("C-IC-001", "Parent Co", "1000");

        assert!(customer.is_intercompany);
        assert_eq!(customer.customer_type, CustomerType::Intercompany);
        assert_eq!(customer.credit_rating, CreditRating::AAA);
    }

    #[test]
    fn test_payment_behavior() {
        assert!(CustomerPaymentBehavior::Excellent.on_time_probability() > 0.95);
        assert!(CustomerPaymentBehavior::VeryPoor.on_time_probability() < 0.25);
        assert!(CustomerPaymentBehavior::Excellent.average_days_past_due() < 0);
        // Pays early
    }

    #[test]
    fn test_vendor_due_date() {
        let vendor = Vendor::new("V-001", "Test", VendorType::Supplier)
            .with_payment_terms_structured(PaymentTerms::Net30);

        let invoice_date = chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let due_date = vendor.calculate_due_date(invoice_date);

        assert_eq!(
            due_date,
            chrono::NaiveDate::from_ymd_opt(2024, 2, 14).unwrap()
        );
    }
}
