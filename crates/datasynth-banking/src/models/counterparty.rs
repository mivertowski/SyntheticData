//! Counterparty models for banking transactions.

use datasynth_core::models::banking::MerchantCategoryCode;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A merchant counterparty.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Merchant {
    /// Unique merchant identifier
    pub merchant_id: Uuid,
    /// Merchant name
    pub name: String,
    /// Merchant category code
    pub mcc: MerchantCategoryCode,
    /// Country (ISO 3166-1 alpha-2)
    pub country: String,
    /// City
    pub city: Option<String>,
    /// Is online-only merchant
    pub is_online: bool,
    /// Typical transaction amount range
    pub typical_amount_range: (Decimal, Decimal),
    /// Whether merchant is high-risk
    pub is_high_risk: bool,
}

impl Merchant {
    /// Create a new merchant.
    pub fn new(merchant_id: Uuid, name: &str, mcc: MerchantCategoryCode, country: &str) -> Self {
        Self {
            merchant_id,
            name: name.to_string(),
            mcc,
            country: country.to_string(),
            city: None,
            is_online: false,
            typical_amount_range: (Decimal::from(10), Decimal::from(500)),
            is_high_risk: mcc.is_high_risk(),
        }
    }

    /// Create an online merchant.
    pub fn online(merchant_id: Uuid, name: &str, mcc: MerchantCategoryCode) -> Self {
        Self {
            merchant_id,
            name: name.to_string(),
            mcc,
            country: "US".to_string(),
            city: None,
            is_online: true,
            typical_amount_range: (Decimal::from(10), Decimal::from(1000)),
            is_high_risk: mcc.is_high_risk(),
        }
    }
}

/// An employer counterparty.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Employer {
    /// Unique employer identifier
    pub employer_id: Uuid,
    /// Employer name
    pub name: String,
    /// Country (ISO 3166-1 alpha-2)
    pub country: String,
    /// Industry code (NAICS)
    pub industry_code: Option<String>,
    /// Typical salary range
    pub salary_range: (Decimal, Decimal),
    /// Pay frequency
    pub pay_frequency: PayFrequency,
}

impl Employer {
    /// Create a new employer.
    pub fn new(employer_id: Uuid, name: &str, country: &str) -> Self {
        Self {
            employer_id,
            name: name.to_string(),
            country: country.to_string(),
            industry_code: None,
            salary_range: (Decimal::from(3000), Decimal::from(10000)),
            pay_frequency: PayFrequency::Monthly,
        }
    }
}

/// Pay frequency for salary deposits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PayFrequency {
    /// Weekly pay
    Weekly,
    /// Bi-weekly pay
    BiWeekly,
    /// Semi-monthly (twice per month)
    SemiMonthly,
    /// Monthly pay
    #[default]
    Monthly,
}

impl PayFrequency {
    /// Days between payments.
    pub fn interval_days(&self) -> u32 {
        match self {
            Self::Weekly => 7,
            Self::BiWeekly => 14,
            Self::SemiMonthly => 15,
            Self::Monthly => 30,
        }
    }
}

/// A utility company counterparty.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilityCompany {
    /// Unique utility identifier
    pub utility_id: Uuid,
    /// Utility name
    pub name: String,
    /// Utility type
    pub utility_type: UtilityType,
    /// Country (ISO 3166-1 alpha-2)
    pub country: String,
    /// Typical bill range
    pub typical_bill_range: (Decimal, Decimal),
}

impl UtilityCompany {
    /// Create a new utility company.
    pub fn new(utility_id: Uuid, name: &str, utility_type: UtilityType, country: &str) -> Self {
        let bill_range = match utility_type {
            UtilityType::Electric => (Decimal::from(50), Decimal::from(300)),
            UtilityType::Gas => (Decimal::from(30), Decimal::from(200)),
            UtilityType::Water => (Decimal::from(20), Decimal::from(100)),
            UtilityType::Internet => (Decimal::from(40), Decimal::from(150)),
            UtilityType::Phone => (Decimal::from(30), Decimal::from(200)),
            UtilityType::Cable => (Decimal::from(50), Decimal::from(200)),
            UtilityType::Streaming => (Decimal::from(10), Decimal::from(50)),
            UtilityType::Insurance => (Decimal::from(100), Decimal::from(500)),
        };

        Self {
            utility_id,
            name: name.to_string(),
            utility_type,
            country: country.to_string(),
            typical_bill_range: bill_range,
        }
    }
}

/// Type of utility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UtilityType {
    /// Electric utility
    Electric,
    /// Gas utility
    Gas,
    /// Water utility
    Water,
    /// Internet service
    Internet,
    /// Phone service
    Phone,
    /// Cable/satellite TV
    Cable,
    /// Streaming service
    Streaming,
    /// Insurance
    Insurance,
}

/// A government agency counterparty.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernmentAgency {
    /// Unique agency identifier
    pub agency_id: Uuid,
    /// Agency name
    pub name: String,
    /// Agency type
    pub agency_type: GovernmentAgencyType,
    /// Country (ISO 3166-1 alpha-2)
    pub country: String,
}

impl GovernmentAgency {
    /// Create a new government agency.
    pub fn new(
        agency_id: Uuid,
        name: &str,
        agency_type: GovernmentAgencyType,
        country: &str,
    ) -> Self {
        Self {
            agency_id,
            name: name.to_string(),
            agency_type,
            country: country.to_string(),
        }
    }
}

/// Type of government agency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GovernmentAgencyType {
    /// Tax authority
    TaxAuthority,
    /// Social security / benefits
    SocialSecurity,
    /// Unemployment benefits
    Unemployment,
    /// Veterans benefits
    Veterans,
    /// Local government
    Local,
    /// Federal government
    Federal,
    /// State/provincial government
    State,
}

/// Pool of counterparties for transaction generation.
#[derive(Debug, Clone, Default)]
pub struct CounterpartyPool {
    /// Merchants
    pub merchants: Vec<Merchant>,
    /// Employers
    pub employers: Vec<Employer>,
    /// Utilities
    pub utilities: Vec<UtilityCompany>,
    /// Government agencies
    pub government_agencies: Vec<GovernmentAgency>,
}

impl CounterpartyPool {
    /// Create a new empty pool.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a merchant.
    pub fn add_merchant(&mut self, merchant: Merchant) {
        self.merchants.push(merchant);
    }

    /// Add an employer.
    pub fn add_employer(&mut self, employer: Employer) {
        self.employers.push(employer);
    }

    /// Add a utility.
    pub fn add_utility(&mut self, utility: UtilityCompany) {
        self.utilities.push(utility);
    }

    /// Add a government agency.
    pub fn add_government(&mut self, agency: GovernmentAgency) {
        self.government_agencies.push(agency);
    }

    /// Create a standard counterparty pool.
    pub fn standard() -> Self {
        let mut pool = Self::new();

        // Add common merchants
        let merchants = [
            ("Walmart", MerchantCategoryCode::GROCERY_STORES),
            ("Amazon", MerchantCategoryCode(5999)),
            ("Target", MerchantCategoryCode::DEPARTMENT_STORES),
            ("Starbucks", MerchantCategoryCode::RESTAURANTS),
            ("Shell Gas", MerchantCategoryCode::GAS_STATIONS),
            ("CVS Pharmacy", MerchantCategoryCode::DRUG_STORES),
            ("Netflix", MerchantCategoryCode(4899)),
            ("Uber", MerchantCategoryCode(4121)),
        ];

        for (name, mcc) in merchants {
            pool.add_merchant(Merchant::new(Uuid::new_v4(), name, mcc, "US"));
        }

        // Add common utilities
        let utilities = [
            ("Electric Company", UtilityType::Electric),
            ("Gas Company", UtilityType::Gas),
            ("Water Utility", UtilityType::Water),
            ("Comcast", UtilityType::Internet),
            ("AT&T", UtilityType::Phone),
            ("State Farm Insurance", UtilityType::Insurance),
        ];

        for (name, utype) in utilities {
            pool.add_utility(UtilityCompany::new(Uuid::new_v4(), name, utype, "US"));
        }

        // Add government agencies
        pool.add_government(GovernmentAgency::new(
            Uuid::new_v4(),
            "IRS",
            GovernmentAgencyType::TaxAuthority,
            "US",
        ));
        pool.add_government(GovernmentAgency::new(
            Uuid::new_v4(),
            "Social Security Administration",
            GovernmentAgencyType::SocialSecurity,
            "US",
        ));

        pool
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_merchant_creation() {
        let merchant = Merchant::new(
            Uuid::new_v4(),
            "Test Store",
            MerchantCategoryCode::GROCERY_STORES,
            "US",
        );
        assert!(!merchant.is_high_risk);
    }

    #[test]
    fn test_counterparty_pool() {
        let pool = CounterpartyPool::standard();
        assert!(!pool.merchants.is_empty());
        assert!(!pool.utilities.is_empty());
    }

    #[test]
    fn test_pay_frequency() {
        assert_eq!(PayFrequency::Weekly.interval_days(), 7);
        assert_eq!(PayFrequency::Monthly.interval_days(), 30);
    }
}
