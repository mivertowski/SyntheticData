//! Business customer persona profiles.

use datasynth_core::models::banking::BusinessPersona;

use super::{PersonaProfile, SpendingProfile, TransactionBehavior};

/// Get profile for a business persona.
pub fn get_profile(persona: BusinessPersona) -> PersonaProfile {
    match persona {
        BusinessPersona::SmallBusiness => small_business_profile(),
        BusinessPersona::MidMarket => mid_market_profile(),
        BusinessPersona::Enterprise => enterprise_profile(),
        BusinessPersona::CashIntensive => cash_intensive_profile(),
        BusinessPersona::ImportExport => import_export_profile(),
        BusinessPersona::Startup => startup_profile(),
        BusinessPersona::MoneyServices => money_services_profile(),
        BusinessPersona::ProfessionalServices => professional_services_profile(),
    }
}

fn small_business_profile() -> PersonaProfile {
    PersonaProfile {
        transaction_behavior: TransactionBehavior {
            monthly_tx_count: 150,
            monthly_tx_std: 50.0,
            avg_amount: 450.0,
            amount_std: 800.0,
            min_amount: 10.0,
            max_amount: 25000.0,
            cash_percentage: 0.10,
            international_percentage: 0.02,
            active_hours: (7, 19),
            weekend_multiplier: 0.3,
        },
        spending_profile: business_spending_profile(0.15, 0.05),
        income_profile: None,
        risk_appetite: 0.5,
        saving_rate: 0.10,
        credit_usage: 0.4,
    }
}

fn mid_market_profile() -> PersonaProfile {
    PersonaProfile {
        transaction_behavior: TransactionBehavior {
            monthly_tx_count: 500,
            monthly_tx_std: 150.0,
            avg_amount: 2500.0,
            amount_std: 5000.0,
            min_amount: 50.0,
            max_amount: 250000.0,
            cash_percentage: 0.03,
            international_percentage: 0.08,
            active_hours: (6, 20),
            weekend_multiplier: 0.2,
        },
        spending_profile: business_spending_profile(0.08, 0.10),
        income_profile: None,
        risk_appetite: 0.6,
        saving_rate: 0.15,
        credit_usage: 0.5,
    }
}

fn enterprise_profile() -> PersonaProfile {
    PersonaProfile {
        transaction_behavior: TransactionBehavior {
            monthly_tx_count: 2000,
            monthly_tx_std: 500.0,
            avg_amount: 15000.0,
            amount_std: 50000.0,
            min_amount: 100.0,
            max_amount: 5000000.0,
            cash_percentage: 0.01,
            international_percentage: 0.20,
            active_hours: (0, 24), // 24/7 operations
            weekend_multiplier: 0.5,
        },
        spending_profile: business_spending_profile(0.05, 0.25),
        income_profile: None,
        risk_appetite: 0.7,
        saving_rate: 0.20,
        credit_usage: 0.6,
    }
}

fn cash_intensive_profile() -> PersonaProfile {
    PersonaProfile {
        transaction_behavior: TransactionBehavior {
            monthly_tx_count: 400,
            monthly_tx_std: 120.0,
            avg_amount: 350.0,
            amount_std: 500.0,
            min_amount: 5.0,
            max_amount: 15000.0,
            cash_percentage: 0.45, // High cash intensity
            international_percentage: 0.02,
            active_hours: (8, 22),
            weekend_multiplier: 1.2, // Active on weekends
        },
        spending_profile: SpendingProfile {
            groceries: 0.30, // Inventory for retail/restaurant
            dining: 0.02,
            entertainment: 0.02,
            shopping: 0.05,
            transportation: 0.08,
            utilities: 0.25,
            healthcare: 0.03,
            travel: 0.02,
            other: 0.23, // Supplies, equipment
        },
        income_profile: None,
        risk_appetite: 0.4,
        saving_rate: 0.08,
        credit_usage: 0.3,
    }
}

fn import_export_profile() -> PersonaProfile {
    PersonaProfile {
        transaction_behavior: TransactionBehavior {
            monthly_tx_count: 300,
            monthly_tx_std: 100.0,
            avg_amount: 25000.0,
            amount_std: 75000.0,
            min_amount: 500.0,
            max_amount: 2000000.0,
            cash_percentage: 0.02,
            international_percentage: 0.60, // Very high international
            active_hours: (0, 24),          // Global operations
            weekend_multiplier: 0.4,
        },
        spending_profile: SpendingProfile {
            groceries: 0.00,
            dining: 0.03,
            entertainment: 0.02,
            shopping: 0.05,
            transportation: 0.15, // Shipping, freight
            utilities: 0.08,
            healthcare: 0.02,
            travel: 0.15, // Business travel
            other: 0.50,  // Inventory, customs, logistics
        },
        income_profile: None,
        risk_appetite: 0.7,
        saving_rate: 0.12,
        credit_usage: 0.7, // High credit usage for trade finance
    }
}

fn startup_profile() -> PersonaProfile {
    PersonaProfile {
        transaction_behavior: TransactionBehavior {
            monthly_tx_count: 100,
            monthly_tx_std: 60.0,
            avg_amount: 800.0,
            amount_std: 2500.0,
            min_amount: 10.0,
            max_amount: 100000.0,
            cash_percentage: 0.03,
            international_percentage: 0.10,
            active_hours: (6, 24), // Long hours
            weekend_multiplier: 0.6,
        },
        spending_profile: SpendingProfile {
            groceries: 0.02,
            dining: 0.08,
            entertainment: 0.03,
            shopping: 0.10,
            transportation: 0.05,
            utilities: 0.15,
            healthcare: 0.03,
            travel: 0.08,
            other: 0.46, // Software, services, equipment
        },
        income_profile: None,
        risk_appetite: 0.8, // High risk tolerance
        saving_rate: 0.05,  // Burning cash
        credit_usage: 0.5,
    }
}

fn money_services_profile() -> PersonaProfile {
    PersonaProfile {
        transaction_behavior: TransactionBehavior {
            monthly_tx_count: 5000,
            monthly_tx_std: 2000.0,
            avg_amount: 1500.0,
            amount_std: 5000.0,
            min_amount: 10.0,
            max_amount: 500000.0,
            cash_percentage: 0.40,          // High cash handling
            international_percentage: 0.35, // Remittances
            active_hours: (8, 20),
            weekend_multiplier: 0.8,
        },
        spending_profile: SpendingProfile {
            groceries: 0.00,
            dining: 0.02,
            entertainment: 0.01,
            shopping: 0.03,
            transportation: 0.05,
            utilities: 0.15,
            healthcare: 0.02,
            travel: 0.05,
            other: 0.67, // Fees, compliance, correspondent banking
        },
        income_profile: None,
        risk_appetite: 0.5,
        saving_rate: 0.15,
        credit_usage: 0.3,
    }
}

fn professional_services_profile() -> PersonaProfile {
    PersonaProfile {
        transaction_behavior: TransactionBehavior {
            monthly_tx_count: 80,
            monthly_tx_std: 30.0,
            avg_amount: 3500.0,
            amount_std: 8000.0,
            min_amount: 50.0,
            max_amount: 200000.0,
            cash_percentage: 0.02,
            international_percentage: 0.05,
            active_hours: (8, 18),
            weekend_multiplier: 0.2,
        },
        spending_profile: SpendingProfile {
            groceries: 0.00,
            dining: 0.08,
            entertainment: 0.04,
            shopping: 0.05,
            transportation: 0.08,
            utilities: 0.12,
            healthcare: 0.03,
            travel: 0.10,
            other: 0.50, // Professional fees, subscriptions, office
        },
        income_profile: None,
        risk_appetite: 0.4,
        saving_rate: 0.20,
        credit_usage: 0.3,
    }
}

/// Create standard business spending profile.
fn business_spending_profile(_cash_intensity: f64, international_rate: f64) -> SpendingProfile {
    SpendingProfile {
        groceries: 0.00,
        dining: 0.05,
        entertainment: 0.03,
        shopping: 0.08,
        transportation: 0.10,
        utilities: 0.20,
        healthcare: 0.03,
        travel: 0.08 + international_rate * 0.2,
        other: 0.43 - international_rate * 0.2, // Payroll, supplies, services
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_business_profiles() {
        let small = get_profile(BusinessPersona::SmallBusiness);
        let enterprise = get_profile(BusinessPersona::Enterprise);

        // Enterprise should have higher transaction volume
        assert!(
            small.transaction_behavior.monthly_tx_count
                < enterprise.transaction_behavior.monthly_tx_count
        );

        // Enterprise should have higher international percentage
        assert!(
            small.transaction_behavior.international_percentage
                < enterprise.transaction_behavior.international_percentage
        );
    }

    #[test]
    fn test_cash_intensive() {
        let cash_biz = get_profile(BusinessPersona::CashIntensive);
        let enterprise = get_profile(BusinessPersona::Enterprise);

        // Cash intensive should have much higher cash percentage
        assert!(cash_biz.transaction_behavior.cash_percentage > 0.3);
        assert!(enterprise.transaction_behavior.cash_percentage < 0.05);
    }

    #[test]
    fn test_import_export() {
        let ie = get_profile(BusinessPersona::ImportExport);

        // Should have very high international percentage
        assert!(ie.transaction_behavior.international_percentage > 0.5);
    }
}
