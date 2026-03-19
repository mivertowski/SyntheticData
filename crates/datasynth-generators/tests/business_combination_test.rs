//! Integration tests for the Business Combination Generator (IFRS 3 / ASC 805).
//!
//! Verifies:
//! - Goodwill = consideration - net identifiable assets
//! - All Day 1 JEs are balanced (debits = credits)
//! - Amortization JEs are balanced
//! - PPA fair values are positive for all assets
//! - At least 4 identifiable assets per acquisition
//! - Consideration components sum to total
//! - Deterministic output with same seed

#[allow(clippy::unwrap_used)]
mod tests {
    use chrono::NaiveDate;
    use datasynth_generators::BusinessCombinationGenerator;
    use rust_decimal::Decimal;

    fn make_generator(seed: u64) -> BusinessCombinationGenerator {
        BusinessCombinationGenerator::new(seed)
    }

    fn period() -> (NaiveDate, NaiveDate) {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        (start, end)
    }

    // =========================================================================
    // Core invariants
    // =========================================================================

    #[test]
    fn test_goodwill_equals_consideration_minus_net_identifiable_assets() {
        let mut gen = make_generator(42);
        let (start, end) = period();
        let snap = gen.generate("C001", "USD", start, end, 3, "IFRS");

        assert_eq!(snap.combinations.len(), 3);

        for bc in &snap.combinations {
            let raw =
                bc.consideration.total - bc.purchase_price_allocation.net_identifiable_assets_fv;

            if raw >= Decimal::ZERO {
                assert_eq!(bc.goodwill, raw, "Goodwill mismatch for {}", bc.id);
            } else {
                // Bargain purchase: goodwill must be zero
                assert_eq!(
                    bc.goodwill,
                    Decimal::ZERO,
                    "Bargain-purchase goodwill must be zero for {}",
                    bc.id
                );
            }
        }
    }

    #[test]
    fn test_day1_journal_entries_balanced() {
        let mut gen = make_generator(42);
        let (start, end) = period();
        let snap = gen.generate("C001", "USD", start, end, 3, "US_GAAP");

        let day1_jes: Vec<_> = snap
            .journal_entries
            .iter()
            .filter(|je| je.header.document_type == "BC")
            .collect();

        assert!(
            !day1_jes.is_empty(),
            "Should have at least one Day 1 JE (document_type 'BC')"
        );

        for je in &day1_jes {
            let dr: Decimal = je.lines.iter().map(|l| l.debit_amount).sum();
            let cr: Decimal = je.lines.iter().map(|l| l.credit_amount).sum();
            assert_eq!(
                dr, cr,
                "Day 1 JE {} is unbalanced: debits={}, credits={}",
                je.header.document_id, dr, cr
            );
        }
    }

    #[test]
    fn test_amortization_journal_entries_balanced() {
        let mut gen = make_generator(42);
        let (start, end) = period();
        let snap = gen.generate("C001", "USD", start, end, 2, "IFRS");

        let amort_jes: Vec<_> = snap
            .journal_entries
            .iter()
            .filter(|je| je.header.document_type == "AM")
            .collect();

        assert!(
            !amort_jes.is_empty(),
            "Should have at least one amortization JE (document_type 'AM')"
        );

        for je in &amort_jes {
            let dr: Decimal = je.lines.iter().map(|l| l.debit_amount).sum();
            let cr: Decimal = je.lines.iter().map(|l| l.credit_amount).sum();
            assert_eq!(
                dr, cr,
                "Amortization JE {} is unbalanced: debits={}, credits={}",
                je.header.document_id, dr, cr
            );
        }
    }

    #[test]
    fn test_ppa_fair_values_positive_for_all_assets() {
        let mut gen = make_generator(77);
        let (start, end) = period();
        let snap = gen.generate("C001", "EUR", start, end, 3, "IFRS");

        for bc in &snap.combinations {
            for adj in &bc.purchase_price_allocation.identifiable_assets {
                assert!(
                    adj.fair_value > Decimal::ZERO,
                    "Asset '{}' should have positive FV in combination '{}'",
                    adj.asset_or_liability,
                    bc.id
                );
            }
        }
    }

    #[test]
    fn test_at_least_4_identifiable_assets_per_acquisition() {
        let mut gen = make_generator(123);
        let (start, end) = period();
        let snap = gen.generate("C001", "USD", start, end, 3, "US_GAAP");

        for bc in &snap.combinations {
            assert!(
                bc.purchase_price_allocation.identifiable_assets.len() >= 4,
                "PPA should have >= 4 assets, got {} for acquisition '{}'",
                bc.purchase_price_allocation.identifiable_assets.len(),
                bc.id
            );
        }
    }

    // =========================================================================
    // Consideration validation
    // =========================================================================

    #[test]
    fn test_consideration_components_sum_to_total() {
        let mut gen = make_generator(200);
        let (start, end) = period();
        let snap = gen.generate("C001", "USD", start, end, 5, "IFRS");

        for bc in &snap.combinations {
            let c = &bc.consideration;
            let computed = c.cash
                + c.shares_issued_value.unwrap_or(Decimal::ZERO)
                + c.contingent_consideration.unwrap_or(Decimal::ZERO);
            assert_eq!(
                computed, c.total,
                "Consideration components don't sum to total for '{}'",
                bc.id
            );
        }
    }

    #[test]
    fn test_consideration_cash_is_positive() {
        let mut gen = make_generator(42);
        let (start, end) = period();
        let snap = gen.generate("C001", "USD", start, end, 3, "IFRS");

        for bc in &snap.combinations {
            assert!(
                bc.consideration.cash > Decimal::ZERO,
                "Cash consideration must be positive for '{}'",
                bc.id
            );
        }
    }

    // =========================================================================
    // PPA arithmetic
    // =========================================================================

    #[test]
    fn test_net_identifiable_assets_fv_is_sum_of_assets_minus_liabilities() {
        let mut gen = make_generator(77);
        let (start, end) = period();
        let snap = gen.generate("C001", "USD", start, end, 3, "US_GAAP");

        for bc in &snap.combinations {
            let ppa = &bc.purchase_price_allocation;
            let total_asset_fv: Decimal =
                ppa.identifiable_assets.iter().map(|a| a.fair_value).sum();
            let total_liab_fv: Decimal = ppa
                .identifiable_liabilities
                .iter()
                .map(|l| l.fair_value)
                .sum();
            let expected_nia = total_asset_fv - total_liab_fv;
            assert_eq!(
                ppa.net_identifiable_assets_fv, expected_nia,
                "NIA FV mismatch for '{}'",
                bc.id
            );
        }
    }

    #[test]
    fn test_step_up_equals_fv_minus_book() {
        let mut gen = make_generator(55);
        let (start, end) = period();
        let snap = gen.generate("C001", "USD", start, end, 3, "IFRS");

        for bc in &snap.combinations {
            for adj in bc
                .purchase_price_allocation
                .identifiable_assets
                .iter()
                .chain(bc.purchase_price_allocation.identifiable_liabilities.iter())
            {
                assert_eq!(
                    adj.step_up,
                    adj.fair_value - adj.book_value,
                    "Step-up mismatch for '{}' in '{}'",
                    adj.asset_or_liability,
                    bc.id
                );
            }
        }
    }

    // =========================================================================
    // Intangibles
    // =========================================================================

    #[test]
    fn test_intangibles_have_useful_lives() {
        let mut gen = make_generator(42);
        let (start, end) = period();
        let snap = gen.generate("C001", "USD", start, end, 3, "IFRS");

        for bc in &snap.combinations {
            let intangible_names = [
                "Customer Relationships",
                "Trade Name",
                "Developed Technology",
            ];
            for adj in &bc.purchase_price_allocation.identifiable_assets {
                if intangible_names.contains(&adj.asset_or_liability.as_str()) {
                    assert!(
                        adj.useful_life_years.is_some(),
                        "Intangible '{}' should have a useful life in '{}'",
                        adj.asset_or_liability,
                        bc.id
                    );
                    let life = adj.useful_life_years.unwrap();
                    assert!(
                        life >= 5 && life <= 20,
                        "Useful life of {} years is outside expected range for '{}' in '{}'",
                        life,
                        adj.asset_or_liability,
                        bc.id
                    );
                }
            }
        }
    }

    // =========================================================================
    // Determinism
    // =========================================================================

    #[test]
    fn test_deterministic_output() {
        let (start, end) = period();

        let mut gen1 = make_generator(99);
        let mut gen2 = make_generator(99);

        let snap1 = gen1.generate("C001", "USD", start, end, 2, "IFRS");
        let snap2 = gen2.generate("C001", "USD", start, end, 2, "IFRS");

        assert_eq!(snap1.combinations.len(), snap2.combinations.len());
        for (a, b) in snap1.combinations.iter().zip(snap2.combinations.iter()) {
            assert_eq!(a.id, b.id);
            assert_eq!(a.goodwill, b.goodwill);
            assert_eq!(a.consideration.total, b.consideration.total);
            assert_eq!(
                a.purchase_price_allocation.net_identifiable_assets_fv,
                b.purchase_price_allocation.net_identifiable_assets_fv
            );
        }
        assert_eq!(snap1.journal_entries.len(), snap2.journal_entries.len());
    }

    // =========================================================================
    // Edge cases
    // =========================================================================

    #[test]
    fn test_zero_count_returns_empty() {
        let mut gen = make_generator(42);
        let (start, end) = period();
        let snap = gen.generate("C001", "USD", start, end, 0, "IFRS");
        assert!(snap.combinations.is_empty());
        assert!(snap.journal_entries.is_empty());
    }

    #[test]
    fn test_max_count_capped_at_five() {
        let mut gen = make_generator(42);
        let (start, end) = period();
        let snap = gen.generate("C001", "USD", start, end, 10, "IFRS");
        assert_eq!(snap.combinations.len(), 5, "Should be capped at 5");
    }

    #[test]
    fn test_acquirer_entity_matches_company_code() {
        let mut gen = make_generator(42);
        let (start, end) = period();
        let snap = gen.generate("TEST_CO", "USD", start, end, 2, "US_GAAP");

        for bc in &snap.combinations {
            assert_eq!(
                bc.acquirer_entity, "TEST_CO",
                "Acquirer entity should match company code"
            );
        }
    }

    #[test]
    fn test_framework_is_recorded() {
        let mut gen = make_generator(42);
        let (start, end) = period();

        let snap_ifrs = gen.generate("C001", "USD", start, end, 1, "IFRS");
        for bc in &snap_ifrs.combinations {
            assert_eq!(bc.framework, "IFRS");
        }

        let mut gen2 = make_generator(42);
        let snap_gaap = gen2.generate("C001", "USD", start, end, 1, "US_GAAP");
        for bc in &snap_gaap.combinations {
            assert_eq!(bc.framework, "US_GAAP");
        }
    }
}
