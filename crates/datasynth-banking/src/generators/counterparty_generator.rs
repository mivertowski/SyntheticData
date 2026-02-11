//! Counterparty generator for banking data.

use datasynth_core::models::banking::MerchantCategoryCode;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use uuid::Uuid;

use crate::config::BankingConfig;
use crate::models::{
    CounterpartyPool, Employer, GovernmentAgency, GovernmentAgencyType, Merchant, PayFrequency,
    UtilityCompany, UtilityType,
};

/// Generator for counterparty data.
pub struct CounterpartyGenerator {
    _rng: ChaCha8Rng,
}

impl CounterpartyGenerator {
    /// Create a new counterparty generator.
    pub fn new(seed: u64) -> Self {
        Self {
            _rng: ChaCha8Rng::seed_from_u64(seed.wrapping_add(3000)),
        }
    }

    /// Generate a comprehensive counterparty pool.
    pub fn generate_pool(&mut self, config: &BankingConfig) -> CounterpartyPool {
        let mut pool = CounterpartyPool::new();

        // Generate merchants
        self.generate_merchants(&mut pool, config);

        // Generate employers
        self.generate_employers(&mut pool, config);

        // Generate utilities
        self.generate_utilities(&mut pool);

        // Generate government agencies
        self.generate_government_agencies(&mut pool);

        pool
    }

    /// Generate merchants.
    fn generate_merchants(&mut self, pool: &mut CounterpartyPool, _config: &BankingConfig) {
        // Grocery stores
        let grocery_names = [
            "Walmart",
            "Kroger",
            "Costco",
            "Safeway",
            "Publix",
            "Whole Foods",
            "Trader Joe's",
            "Aldi",
            "Target Grocery",
            "Stop & Shop",
        ];
        for name in grocery_names {
            pool.add_merchant(Merchant::new(
                Uuid::new_v4(),
                name,
                MerchantCategoryCode::GROCERY_STORES,
                "US",
            ));
        }

        // Restaurants
        let restaurant_names = [
            "McDonald's",
            "Starbucks",
            "Chipotle",
            "Chick-fil-A",
            "Panera Bread",
            "Olive Garden",
            "Applebee's",
            "Taco Bell",
            "Wendy's",
            "Subway",
        ];
        for name in restaurant_names {
            pool.add_merchant(Merchant::new(
                Uuid::new_v4(),
                name,
                MerchantCategoryCode::RESTAURANTS,
                "US",
            ));
        }

        // Gas stations
        let gas_names = [
            "Shell",
            "ExxonMobil",
            "Chevron",
            "BP",
            "Circle K",
            "7-Eleven",
        ];
        for name in gas_names {
            pool.add_merchant(Merchant::new(
                Uuid::new_v4(),
                name,
                MerchantCategoryCode::GAS_STATIONS,
                "US",
            ));
        }

        // Department stores
        let dept_names = ["Target", "Macy's", "Nordstrom", "Kohl's", "JCPenney"];
        for name in dept_names {
            pool.add_merchant(Merchant::new(
                Uuid::new_v4(),
                name,
                MerchantCategoryCode::DEPARTMENT_STORES,
                "US",
            ));
        }

        // Online merchants
        let online_names = ["Amazon", "eBay", "Etsy", "Wayfair"];
        for name in online_names {
            pool.add_merchant(Merchant::online(
                Uuid::new_v4(),
                name,
                MerchantCategoryCode(5999),
            ));
        }
    }

    /// Generate employers.
    fn generate_employers(&mut self, pool: &mut CounterpartyPool, _config: &BankingConfig) {
        let employer_names = [
            ("Tech Corp Inc", "US", PayFrequency::BiWeekly),
            ("Finance Solutions LLC", "US", PayFrequency::SemiMonthly),
            ("Healthcare Partners", "US", PayFrequency::BiWeekly),
            ("Retail Holdings Co", "US", PayFrequency::Weekly),
            ("Manufacturing Industries", "US", PayFrequency::BiWeekly),
            ("Government Services", "US", PayFrequency::Monthly),
            ("Education Institute", "US", PayFrequency::Monthly),
            ("Consulting Group", "US", PayFrequency::SemiMonthly),
        ];

        for (name, country, pay_freq) in employer_names {
            let mut employer = Employer::new(Uuid::new_v4(), name, country);
            employer.pay_frequency = pay_freq;
            pool.add_employer(employer);
        }
    }

    /// Generate utilities.
    fn generate_utilities(&mut self, pool: &mut CounterpartyPool) {
        let utilities = [
            ("Electric Company", UtilityType::Electric),
            ("Gas Utility", UtilityType::Gas),
            ("Water Department", UtilityType::Water),
            ("Comcast", UtilityType::Internet),
            ("AT&T", UtilityType::Phone),
            ("Verizon", UtilityType::Phone),
            ("Netflix", UtilityType::Streaming),
            ("Disney+", UtilityType::Streaming),
            ("State Farm", UtilityType::Insurance),
            ("Progressive", UtilityType::Insurance),
        ];

        for (name, utype) in utilities {
            pool.add_utility(UtilityCompany::new(Uuid::new_v4(), name, utype, "US"));
        }
    }

    /// Generate government agencies.
    fn generate_government_agencies(&mut self, pool: &mut CounterpartyPool) {
        let agencies = [
            ("IRS", GovernmentAgencyType::TaxAuthority),
            (
                "Social Security Administration",
                GovernmentAgencyType::SocialSecurity,
            ),
            ("State Tax Authority", GovernmentAgencyType::State),
            ("City Services", GovernmentAgencyType::Local),
        ];

        for (name, atype) in agencies {
            pool.add_government(GovernmentAgency::new(Uuid::new_v4(), name, atype, "US"));
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_counterparty_generation() {
        let config = BankingConfig::default();
        let mut gen = CounterpartyGenerator::new(12345);
        let pool = gen.generate_pool(&config);

        assert!(!pool.merchants.is_empty());
        assert!(!pool.employers.is_empty());
        assert!(!pool.utilities.is_empty());
    }
}
