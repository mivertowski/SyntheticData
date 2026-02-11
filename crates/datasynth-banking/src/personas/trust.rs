//! Trust and foundation customer persona profiles.

use datasynth_core::models::banking::TrustPersona;

use super::{PersonaProfile, SpendingProfile, TransactionBehavior};

/// Get profile for a trust persona.
pub fn get_profile(persona: TrustPersona) -> PersonaProfile {
    match persona {
        TrustPersona::FamilyTrust => family_trust_profile(),
        TrustPersona::PrivateFoundation => private_foundation_profile(),
        TrustPersona::CharitableTrust => charitable_trust_profile(),
        TrustPersona::InvestmentHolding => investment_holding_profile(),
        TrustPersona::SpecialPurposeVehicle => spv_profile(),
    }
}

fn family_trust_profile() -> PersonaProfile {
    PersonaProfile {
        transaction_behavior: TransactionBehavior {
            monthly_tx_count: 15,
            monthly_tx_std: 8.0,
            avg_amount: 25000.0,
            amount_std: 50000.0,
            min_amount: 500.0,
            max_amount: 500000.0,
            cash_percentage: 0.01,
            international_percentage: 0.08,
            active_hours: (9, 17),
            weekend_multiplier: 0.1,
        },
        spending_profile: SpendingProfile {
            groceries: 0.00,
            dining: 0.00,
            entertainment: 0.00,
            shopping: 0.00,
            transportation: 0.00,
            utilities: 0.02,
            healthcare: 0.05,
            travel: 0.03,
            other: 0.90, // Distributions, investments, legal fees
        },
        income_profile: None,
        risk_appetite: 0.5,
        saving_rate: 0.60, // High preservation focus
        credit_usage: 0.1,
    }
}

fn private_foundation_profile() -> PersonaProfile {
    PersonaProfile {
        transaction_behavior: TransactionBehavior {
            monthly_tx_count: 50,
            monthly_tx_std: 20.0,
            avg_amount: 15000.0,
            amount_std: 40000.0,
            min_amount: 100.0,
            max_amount: 1000000.0,
            cash_percentage: 0.02,
            international_percentage: 0.15, // International grants
            active_hours: (9, 17),
            weekend_multiplier: 0.2,
        },
        spending_profile: SpendingProfile {
            groceries: 0.00,
            dining: 0.02,
            entertainment: 0.01,
            shopping: 0.02,
            transportation: 0.03,
            utilities: 0.05,
            healthcare: 0.00,
            travel: 0.05,
            other: 0.82, // Grants, programs, admin
        },
        income_profile: None,
        risk_appetite: 0.4,
        saving_rate: 0.40, // Endowment preservation
        credit_usage: 0.05,
    }
}

fn charitable_trust_profile() -> PersonaProfile {
    PersonaProfile {
        transaction_behavior: TransactionBehavior {
            monthly_tx_count: 30,
            monthly_tx_std: 15.0,
            avg_amount: 8000.0,
            amount_std: 20000.0,
            min_amount: 100.0,
            max_amount: 500000.0,
            cash_percentage: 0.01,
            international_percentage: 0.10,
            active_hours: (9, 17),
            weekend_multiplier: 0.1,
        },
        spending_profile: SpendingProfile {
            groceries: 0.00,
            dining: 0.01,
            entertainment: 0.01,
            shopping: 0.01,
            transportation: 0.02,
            utilities: 0.03,
            healthcare: 0.00,
            travel: 0.03,
            other: 0.89, // Charitable distributions
        },
        income_profile: None,
        risk_appetite: 0.3,
        saving_rate: 0.50,
        credit_usage: 0.02,
    }
}

fn investment_holding_profile() -> PersonaProfile {
    PersonaProfile {
        transaction_behavior: TransactionBehavior {
            monthly_tx_count: 100,
            monthly_tx_std: 50.0,
            avg_amount: 75000.0,
            amount_std: 200000.0,
            min_amount: 1000.0,
            max_amount: 10000000.0,
            cash_percentage: 0.00,
            international_percentage: 0.30, // Global investments
            active_hours: (6, 20),          // Market hours
            weekend_multiplier: 0.1,
        },
        spending_profile: SpendingProfile {
            groceries: 0.00,
            dining: 0.00,
            entertainment: 0.00,
            shopping: 0.00,
            transportation: 0.00,
            utilities: 0.01,
            healthcare: 0.00,
            travel: 0.01,
            other: 0.98, // Investment activity, fees
        },
        income_profile: None,
        risk_appetite: 0.7, // Higher risk for returns
        saving_rate: 0.80,  // Investment focus
        credit_usage: 0.3,  // Margin, leverage
    }
}

fn spv_profile() -> PersonaProfile {
    PersonaProfile {
        transaction_behavior: TransactionBehavior {
            monthly_tx_count: 20,
            monthly_tx_std: 15.0,
            avg_amount: 100000.0,
            amount_std: 300000.0,
            min_amount: 1000.0,
            max_amount: 50000000.0,
            cash_percentage: 0.00,
            international_percentage: 0.25,
            active_hours: (9, 17),
            weekend_multiplier: 0.05,
        },
        spending_profile: SpendingProfile {
            groceries: 0.00,
            dining: 0.00,
            entertainment: 0.00,
            shopping: 0.00,
            transportation: 0.00,
            utilities: 0.01,
            healthcare: 0.00,
            travel: 0.01,
            other: 0.98, // Single purpose transactions
        },
        income_profile: None,
        risk_appetite: 0.6,
        saving_rate: 0.90,
        credit_usage: 0.4,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_profiles() {
        let family = get_profile(TrustPersona::FamilyTrust);
        let investment = get_profile(TrustPersona::InvestmentHolding);

        // Investment holding should have higher transaction volume
        assert!(
            family.transaction_behavior.monthly_tx_count
                < investment.transaction_behavior.monthly_tx_count
        );
    }

    #[test]
    fn test_investment_holding() {
        let investment = get_profile(TrustPersona::InvestmentHolding);

        // Should have very high "other" spending (investment activity)
        assert!(investment.spending_profile.other > 0.95);

        // Should have higher risk appetite
        assert!(investment.risk_appetite > 0.6);
    }

    #[test]
    fn test_private_foundation() {
        let foundation = get_profile(TrustPersona::PrivateFoundation);

        // Should have international exposure for grants
        assert!(foundation.transaction_behavior.international_percentage > 0.10);

        // Should have low credit usage
        assert!(foundation.credit_usage < 0.10);
    }
}
