//! Retail customer persona profiles.

use datasynth_core::models::banking::RetailPersona;

use super::{
    IncomeFrequency, IncomeProfile, IncomeSource, PersonaProfile, SpendingProfile,
    TransactionBehavior,
};

/// Get profile for a retail persona.
pub fn get_profile(persona: RetailPersona) -> PersonaProfile {
    match persona {
        RetailPersona::Student => student_profile(),
        RetailPersona::EarlyCareer => early_career_profile(),
        RetailPersona::MidCareer => mid_career_profile(),
        RetailPersona::Retiree => retiree_profile(),
        RetailPersona::HighNetWorth => hnw_profile(),
        RetailPersona::GigWorker => gig_worker_profile(),
        RetailPersona::SeasonalWorker => seasonal_worker_profile(),
        RetailPersona::LowActivity => low_activity_profile(),
    }
}

fn student_profile() -> PersonaProfile {
    PersonaProfile {
        transaction_behavior: TransactionBehavior {
            monthly_tx_count: 35,
            monthly_tx_std: 15.0,
            avg_amount: 25.0,
            amount_std: 30.0,
            min_amount: 2.0,
            max_amount: 500.0,
            cash_percentage: 0.05,
            international_percentage: 0.02,
            active_hours: (10, 2), // Late nights
            weekend_multiplier: 1.5,
        },
        spending_profile: SpendingProfile {
            groceries: 0.15,
            dining: 0.25,
            entertainment: 0.20,
            shopping: 0.15,
            transportation: 0.10,
            utilities: 0.05,
            healthcare: 0.02,
            travel: 0.03,
            other: 0.05,
        },
        income_profile: Some(IncomeProfile {
            source: IncomeSource::ParentalSupport,
            monthly_amount: 1200.0,
            frequency: IncomeFrequency::Monthly,
            income_day: Some(1),
            has_secondary: true, // Part-time job
        }),
        risk_appetite: 0.3,
        saving_rate: 0.05,
        credit_usage: 0.4,
    }
}

fn early_career_profile() -> PersonaProfile {
    PersonaProfile {
        transaction_behavior: TransactionBehavior {
            monthly_tx_count: 45,
            monthly_tx_std: 12.0,
            avg_amount: 65.0,
            amount_std: 80.0,
            min_amount: 3.0,
            max_amount: 2000.0,
            cash_percentage: 0.08,
            international_percentage: 0.03,
            active_hours: (7, 23),
            weekend_multiplier: 1.3,
        },
        spending_profile: SpendingProfile {
            groceries: 0.18,
            dining: 0.18,
            entertainment: 0.15,
            shopping: 0.18,
            transportation: 0.12,
            utilities: 0.10,
            healthcare: 0.04,
            travel: 0.03,
            other: 0.02,
        },
        income_profile: Some(IncomeProfile {
            source: IncomeSource::Salary,
            monthly_amount: 4500.0,
            frequency: IncomeFrequency::BiWeekly,
            income_day: None,
            has_secondary: false,
        }),
        risk_appetite: 0.5,
        saving_rate: 0.10,
        credit_usage: 0.5,
    }
}

fn mid_career_profile() -> PersonaProfile {
    PersonaProfile {
        transaction_behavior: TransactionBehavior {
            monthly_tx_count: 55,
            monthly_tx_std: 15.0,
            avg_amount: 120.0,
            amount_std: 150.0,
            min_amount: 5.0,
            max_amount: 5000.0,
            cash_percentage: 0.05,
            international_percentage: 0.02,
            active_hours: (6, 22),
            weekend_multiplier: 1.0,
        },
        spending_profile: SpendingProfile {
            groceries: 0.22,
            dining: 0.12,
            entertainment: 0.08,
            shopping: 0.15,
            transportation: 0.12,
            utilities: 0.15,
            healthcare: 0.06,
            travel: 0.05,
            other: 0.05,
        },
        income_profile: Some(IncomeProfile {
            source: IncomeSource::Salary,
            monthly_amount: 8500.0,
            frequency: IncomeFrequency::BiWeekly,
            income_day: None,
            has_secondary: true, // Spouse income or bonus
        }),
        risk_appetite: 0.6,
        saving_rate: 0.18,
        credit_usage: 0.4,
    }
}

fn retiree_profile() -> PersonaProfile {
    PersonaProfile {
        transaction_behavior: TransactionBehavior {
            monthly_tx_count: 25,
            monthly_tx_std: 8.0,
            avg_amount: 85.0,
            amount_std: 100.0,
            min_amount: 5.0,
            max_amount: 2000.0,
            cash_percentage: 0.15,
            international_percentage: 0.01,
            active_hours: (8, 18),
            weekend_multiplier: 0.8,
        },
        spending_profile: SpendingProfile {
            groceries: 0.25,
            dining: 0.08,
            entertainment: 0.05,
            shopping: 0.10,
            transportation: 0.08,
            utilities: 0.20,
            healthcare: 0.15,
            travel: 0.05,
            other: 0.04,
        },
        income_profile: Some(IncomeProfile {
            source: IncomeSource::Pension,
            monthly_amount: 3500.0,
            frequency: IncomeFrequency::Monthly,
            income_day: Some(3),
            has_secondary: true, // Social Security
        }),
        risk_appetite: 0.2,
        saving_rate: 0.08,
        credit_usage: 0.2,
    }
}

fn hnw_profile() -> PersonaProfile {
    PersonaProfile {
        transaction_behavior: TransactionBehavior {
            monthly_tx_count: 40,
            monthly_tx_std: 20.0,
            avg_amount: 2500.0,
            amount_std: 5000.0,
            min_amount: 20.0,
            max_amount: 100000.0,
            cash_percentage: 0.02,
            international_percentage: 0.15,
            active_hours: (7, 23),
            weekend_multiplier: 1.2,
        },
        spending_profile: SpendingProfile {
            groceries: 0.08,
            dining: 0.15,
            entertainment: 0.10,
            shopping: 0.20,
            transportation: 0.05,
            utilities: 0.05,
            healthcare: 0.05,
            travel: 0.25,
            other: 0.07,
        },
        income_profile: Some(IncomeProfile {
            source: IncomeSource::Investment,
            monthly_amount: 35000.0,
            frequency: IncomeFrequency::Irregular,
            income_day: None,
            has_secondary: true,
        }),
        risk_appetite: 0.7,
        saving_rate: 0.35,
        credit_usage: 0.6,
    }
}

fn gig_worker_profile() -> PersonaProfile {
    PersonaProfile {
        transaction_behavior: TransactionBehavior {
            monthly_tx_count: 60,
            monthly_tx_std: 25.0,
            avg_amount: 45.0,
            amount_std: 60.0,
            min_amount: 3.0,
            max_amount: 1500.0,
            cash_percentage: 0.12,
            international_percentage: 0.02,
            active_hours: (6, 24),
            weekend_multiplier: 1.4,
        },
        spending_profile: SpendingProfile {
            groceries: 0.18,
            dining: 0.15,
            entertainment: 0.10,
            shopping: 0.12,
            transportation: 0.20, // High due to work
            utilities: 0.12,
            healthcare: 0.05,
            travel: 0.03,
            other: 0.05,
        },
        income_profile: Some(IncomeProfile {
            source: IncomeSource::Gig,
            monthly_amount: 3200.0,
            frequency: IncomeFrequency::Irregular,
            income_day: None,
            has_secondary: false,
        }),
        risk_appetite: 0.4,
        saving_rate: 0.05,
        credit_usage: 0.5,
    }
}

fn seasonal_worker_profile() -> PersonaProfile {
    PersonaProfile {
        transaction_behavior: TransactionBehavior {
            monthly_tx_count: 30,
            monthly_tx_std: 20.0,
            avg_amount: 55.0,
            amount_std: 70.0,
            min_amount: 3.0,
            max_amount: 1000.0,
            cash_percentage: 0.20,
            international_percentage: 0.01,
            active_hours: (6, 22),
            weekend_multiplier: 1.0,
        },
        spending_profile: SpendingProfile {
            groceries: 0.25,
            dining: 0.10,
            entertainment: 0.08,
            shopping: 0.12,
            transportation: 0.15,
            utilities: 0.18,
            healthcare: 0.04,
            travel: 0.03,
            other: 0.05,
        },
        income_profile: Some(IncomeProfile {
            source: IncomeSource::HourlyWage,
            monthly_amount: 2800.0,
            frequency: IncomeFrequency::Irregular,
            income_day: None,
            has_secondary: false,
        }),
        risk_appetite: 0.3,
        saving_rate: 0.05,
        credit_usage: 0.4,
    }
}

fn low_activity_profile() -> PersonaProfile {
    PersonaProfile {
        transaction_behavior: TransactionBehavior {
            monthly_tx_count: 12,
            monthly_tx_std: 5.0,
            avg_amount: 35.0,
            amount_std: 40.0,
            min_amount: 2.0,
            max_amount: 400.0,
            cash_percentage: 0.10,
            international_percentage: 0.01,
            active_hours: (9, 20),
            weekend_multiplier: 0.8,
        },
        spending_profile: SpendingProfile {
            groceries: 0.35,
            dining: 0.05,
            entertainment: 0.05,
            shopping: 0.10,
            transportation: 0.08,
            utilities: 0.25,
            healthcare: 0.05,
            travel: 0.02,
            other: 0.05,
        },
        income_profile: Some(IncomeProfile {
            source: IncomeSource::Other,
            monthly_amount: 1200.0,
            frequency: IncomeFrequency::Monthly,
            income_day: Some(1),
            has_secondary: false,
        }),
        risk_appetite: 0.1,
        saving_rate: 0.0,
        credit_usage: 0.2,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_retail_profiles() {
        let student = get_profile(RetailPersona::Student);
        let hnw = get_profile(RetailPersona::HighNetWorth);

        // Student should have lower income
        assert!(
            student.income_profile.as_ref().unwrap().monthly_amount
                < hnw.income_profile.as_ref().unwrap().monthly_amount
        );

        // HNW should have higher travel spending
        assert!(student.spending_profile.travel < hnw.spending_profile.travel);
    }
}
