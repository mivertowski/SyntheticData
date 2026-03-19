//! Notes to financial statements generator.
//!
//! Assembles 8–13 standard notes from available generation outputs.
//! Each note is template-driven: if the underlying data is not present,
//! that note is simply omitted.  The notes produced follow the ordering
//! and content requirements of IAS 1 Presentation of Financial Statements
//! and, where applicable, ASC 235 / the relevant topic-specific US GAAP
//! guidance.
//!
//! # Notes generated (when data is available)
//!
//! 1. Significant Accounting Policies
//! 2. Revenue Recognition (ASC 606 / IFRS 15)
//! 3. Property, Plant & Equipment
//! 4. Income Taxes (IAS 12 / ASC 740)
//! 5. Provisions & Contingencies (IAS 37 / ASC 450)
//! 6. Related Party Transactions (IAS 24 / ASC 850)
//! 7. Subsequent Events (IAS 10 / ASC 855)
//! 8. Employee Benefits / Pensions (IAS 19 / ASC 715)

use chrono::NaiveDate;
use datasynth_core::models::{
    FinancialStatementNote, NoteCategory, NoteSection, NoteTable, NoteTableValue,
};
use datasynth_core::utils::seeded_rng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;

// ---------------------------------------------------------------------------
// Input context
// ---------------------------------------------------------------------------

/// Summary data drawn from the overall generation result, passed into the
/// notes generator so it can decide which notes to include and what numbers
/// to populate them with.
#[derive(Debug, Clone, Default)]
pub struct NotesGeneratorContext {
    /// Entity / company code for which notes are prepared.
    pub entity_code: String,
    /// Reporting framework name (e.g. "IFRS", "US GAAP").
    pub framework: String,
    /// Fiscal period descriptor (e.g. "FY2024").
    pub period: String,
    /// Period end date.
    pub period_end: NaiveDate,
    /// Reporting currency code (e.g. "USD").
    pub currency: String,

    // ---- Revenue ----
    /// Number of customer contracts recognised during the period.
    pub revenue_contract_count: usize,
    /// Total revenue amount.
    pub revenue_amount: Option<Decimal>,
    /// Average number of performance obligations per contract.
    pub avg_obligations_per_contract: Option<Decimal>,

    // ---- PP&E ----
    /// Total gross fixed asset carrying amount.
    pub total_ppe_gross: Option<Decimal>,
    /// Accumulated depreciation on fixed assets.
    pub accumulated_depreciation: Option<Decimal>,

    // ---- Taxes ----
    /// Statutory tax rate (e.g. 0.21 for 21 %).
    pub statutory_tax_rate: Option<Decimal>,
    /// Effective tax rate actually incurred.
    pub effective_tax_rate: Option<Decimal>,
    /// Deferred tax asset balance.
    pub deferred_tax_asset: Option<Decimal>,
    /// Deferred tax liability balance.
    pub deferred_tax_liability: Option<Decimal>,

    // ---- Provisions ----
    /// Number of provisions recognised.
    pub provision_count: usize,
    /// Total carrying value of all provisions.
    pub total_provisions: Option<Decimal>,

    // ---- Related parties ----
    /// Number of related party transactions identified.
    pub related_party_transaction_count: usize,
    /// Total value of related party transactions.
    pub related_party_total_value: Option<Decimal>,

    // ---- Subsequent events ----
    /// Number of subsequent events identified.
    pub subsequent_event_count: usize,
    /// Number of adjusting subsequent events.
    pub adjusting_event_count: usize,

    // ---- Pensions ----
    /// Number of defined benefit plans.
    pub pension_plan_count: usize,
    /// Total DBO at period end.
    pub total_dbo: Option<Decimal>,
    /// Total plan assets at fair value.
    pub total_plan_assets: Option<Decimal>,
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Generator for notes to the financial statements.
pub struct NotesGenerator {
    rng: ChaCha8Rng,
}

impl NotesGenerator {
    /// Create a new generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0x4E07), // discriminator for "NOTE"
        }
    }

    /// Generate all applicable notes for a single entity/period.
    ///
    /// Returns between 0 and 8 notes depending on which data is present in
    /// the provided context.  Note numbers are assigned sequentially starting
    /// from 1.
    pub fn generate(&mut self, ctx: &NotesGeneratorContext) -> Vec<FinancialStatementNote> {
        let mut notes: Vec<FinancialStatementNote> = Vec::new();

        // Note 1 — Significant Accounting Policies (always generated)
        notes.push(self.note_accounting_policies(ctx));

        // Note 2 — Revenue Recognition
        if ctx.revenue_contract_count > 0 || ctx.revenue_amount.is_some() {
            notes.push(self.note_revenue_recognition(ctx));
        }

        // Note 3 — Property, Plant & Equipment
        if ctx.total_ppe_gross.is_some() {
            notes.push(self.note_property_plant_equipment(ctx));
        }

        // Note 4 — Income Taxes
        if ctx.statutory_tax_rate.is_some() || ctx.deferred_tax_asset.is_some() {
            notes.push(self.note_income_taxes(ctx));
        }

        // Note 5 — Provisions & Contingencies
        if ctx.provision_count > 0 || ctx.total_provisions.is_some() {
            notes.push(self.note_provisions(ctx));
        }

        // Note 6 — Related Party Transactions
        if ctx.related_party_transaction_count > 0 {
            notes.push(self.note_related_parties(ctx));
        }

        // Note 7 — Subsequent Events
        if ctx.subsequent_event_count > 0 {
            notes.push(self.note_subsequent_events(ctx));
        }

        // Note 8 — Employee Benefits (Pensions)
        if ctx.pension_plan_count > 0 || ctx.total_dbo.is_some() {
            notes.push(self.note_employee_benefits(ctx));
        }

        // Assign sequential note numbers
        for (i, note) in notes.iter_mut().enumerate() {
            note.note_number = (i + 1) as u32;
        }

        notes
    }

    // -----------------------------------------------------------------------
    // Note builders
    // -----------------------------------------------------------------------

    fn note_accounting_policies(&mut self, ctx: &NotesGeneratorContext) -> FinancialStatementNote {
        let framework = &ctx.framework;
        let narrative = format!(
            "The financial statements of {} have been prepared in accordance with {} \
             on a going concern basis, using the historical cost convention except where \
             otherwise stated.  The financial statements are presented in {} and all \
             values are rounded to the nearest unit unless otherwise indicated.  \
             Critical accounting estimates and judgements are described in the relevant \
             notes below.",
            ctx.entity_code, framework, ctx.currency
        );

        let key_policies = [
            ("Revenue Recognition", format!("Revenue is recognised in accordance with {} 15 (Revenue from Contracts with Customers). The five-step model is applied to identify contracts, performance obligations, and transaction prices.", if framework.to_lowercase().contains("ifrs") { "IFRS" } else { "ASC 606" })),
            ("Property, Plant & Equipment", "PP&E is stated at cost less accumulated depreciation and impairment losses. Depreciation is computed on a straight-line basis over the estimated useful lives of the assets.".to_string()),
            ("Income Taxes", "Income tax expense comprises current and deferred tax. Deferred tax is recognised using the balance sheet liability method.".to_string()),
            ("Provisions", "A provision is recognised when the entity has a present obligation as a result of a past event, and it is probable that an outflow of resources will be required to settle the obligation.".to_string()),
        ];

        let table = NoteTable {
            caption: "Summary of Key Accounting Policies".to_string(),
            headers: vec![
                "Policy Area".to_string(),
                "Accounting Treatment".to_string(),
            ],
            rows: key_policies
                .iter()
                .map(|(area, treatment)| {
                    vec![
                        NoteTableValue::Text(area.to_string()),
                        NoteTableValue::Text(treatment.clone()),
                    ]
                })
                .collect(),
        };

        FinancialStatementNote {
            note_number: 0, // renumbered later
            title: "Significant Accounting Policies".to_string(),
            category: NoteCategory::AccountingPolicy,
            content_sections: vec![NoteSection {
                heading: "Basis of Preparation".to_string(),
                narrative,
                tables: vec![table],
            }],
            cross_references: Vec::new(),
        }
    }

    fn note_revenue_recognition(&mut self, ctx: &NotesGeneratorContext) -> FinancialStatementNote {
        let contract_count = ctx.revenue_contract_count;
        let revenue_str = ctx
            .revenue_amount
            .map(|a| format!("{} {:.0}", ctx.currency, a))
            .unwrap_or_else(|| "N/A".to_string());
        let avg_oblig = ctx
            .avg_obligations_per_contract
            .map(|v| format!("{:.1}", v))
            .unwrap_or_else(|| "N/A".to_string());

        let narrative = format!(
            "Revenue is recognised when (or as) performance obligations are satisfied by \
             transferring control of a promised good or service to the customer.  During \
             {} the entity entered into {} revenue contracts with an average of {} \
             performance obligation(s) per contract.  Total revenue recognised was {}.",
            ctx.period, contract_count, avg_oblig, revenue_str
        );

        let rows = vec![
            vec![
                NoteTableValue::Text("Number of contracts".to_string()),
                NoteTableValue::Text(contract_count.to_string()),
            ],
            vec![
                NoteTableValue::Text("Revenue recognised".to_string()),
                NoteTableValue::Text(revenue_str),
            ],
            vec![
                NoteTableValue::Text("Avg. performance obligations per contract".to_string()),
                NoteTableValue::Text(avg_oblig),
            ],
        ];

        FinancialStatementNote {
            note_number: 0,
            title: "Revenue Recognition".to_string(),
            category: NoteCategory::StandardSpecific,
            content_sections: vec![NoteSection {
                heading: "Revenue from Contracts with Customers".to_string(),
                narrative,
                tables: vec![NoteTable {
                    caption: "Revenue Disaggregation Summary".to_string(),
                    headers: vec!["Metric".to_string(), "Value".to_string()],
                    rows,
                }],
            }],
            cross_references: vec!["Note 1 — Accounting Policies".to_string()],
        }
    }

    fn note_property_plant_equipment(
        &mut self,
        ctx: &NotesGeneratorContext,
    ) -> FinancialStatementNote {
        let gross = ctx.total_ppe_gross.unwrap_or(Decimal::ZERO);
        let acc_dep = ctx.accumulated_depreciation.unwrap_or(Decimal::ZERO).abs();
        let net = gross - acc_dep;

        // Generate 2–4 asset category rows
        let num_categories = self.rng.random_range(2usize..=4);
        let category_names = [
            "Land & Buildings",
            "Machinery & Equipment",
            "Motor Vehicles",
            "IT Equipment & Fixtures",
        ];
        let mut rows = Vec::new();
        for name in category_names.iter().take(num_categories) {
            let share = Decimal::new(self.rng.random_range(10i64..=40), 2); // 0.10–0.40
            rows.push(vec![
                NoteTableValue::Text(name.to_string()),
                NoteTableValue::Amount(gross * share),
                NoteTableValue::Amount(acc_dep * share),
                NoteTableValue::Amount((gross - acc_dep) * share),
            ]);
        }
        // Totals row
        rows.push(vec![
            NoteTableValue::Text("Total".to_string()),
            NoteTableValue::Amount(gross),
            NoteTableValue::Amount(acc_dep),
            NoteTableValue::Amount(net),
        ]);

        let narrative = format!(
            "Property, plant and equipment is stated at cost less accumulated depreciation \
             and any recognised impairment loss.  At {} the gross carrying amount was \
             {currency} {gross:.0} with accumulated depreciation of {currency} {acc_dep:.0}, \
             resulting in a net book value of {currency} {net:.0}.",
            ctx.period_end,
            currency = ctx.currency,
        );

        FinancialStatementNote {
            note_number: 0,
            title: "Property, Plant & Equipment".to_string(),
            category: NoteCategory::DetailDisclosure,
            content_sections: vec![NoteSection {
                heading: "PP&E Roll-Forward".to_string(),
                narrative,
                tables: vec![NoteTable {
                    caption: format!("PP&E Carrying Amounts at {}", ctx.period_end),
                    headers: vec![
                        "Category".to_string(),
                        format!("Gross ({currency})", currency = ctx.currency),
                        format!("Acc. Dep. ({currency})", currency = ctx.currency),
                        format!("Net ({currency})", currency = ctx.currency),
                    ],
                    rows,
                }],
            }],
            cross_references: Vec::new(),
        }
    }

    fn note_income_taxes(&mut self, ctx: &NotesGeneratorContext) -> FinancialStatementNote {
        let statutory = ctx
            .statutory_tax_rate
            .unwrap_or_else(|| Decimal::new(21, 2)); // default 21%
        let effective = ctx.effective_tax_rate.unwrap_or_else(|| {
            let adj = Decimal::new(self.rng.random_range(-5i64..=5), 2);
            statutory + adj
        });
        let dta = ctx.deferred_tax_asset.unwrap_or(Decimal::ZERO);
        let dtl = ctx.deferred_tax_liability.unwrap_or(Decimal::ZERO);

        let narrative = format!(
            "The entity is subject to income taxes in multiple jurisdictions.  The statutory \
             tax rate applicable to the primary jurisdiction is {statutory:.1}%.  \
             The effective tax rate for {period} was {effective:.1}%, reflecting permanent \
             differences and the utilisation of deferred tax balances.  At period end a \
             deferred tax asset of {currency} {dta:.0} and a deferred tax liability of \
             {currency} {dtl:.0} were recognised.",
            statutory = statutory * Decimal::new(100, 0),
            period = ctx.period,
            effective = effective * Decimal::new(100, 0),
            currency = ctx.currency,
        );

        let rows = vec![
            vec![
                NoteTableValue::Text("Statutory tax rate".to_string()),
                NoteTableValue::Percentage(statutory),
            ],
            vec![
                NoteTableValue::Text("Effective tax rate".to_string()),
                NoteTableValue::Percentage(effective),
            ],
            vec![
                NoteTableValue::Text("Deferred tax asset".to_string()),
                NoteTableValue::Amount(dta),
            ],
            vec![
                NoteTableValue::Text("Deferred tax liability".to_string()),
                NoteTableValue::Amount(dtl),
            ],
            vec![
                NoteTableValue::Text("Net deferred tax position".to_string()),
                NoteTableValue::Amount(dta - dtl),
            ],
        ];

        FinancialStatementNote {
            note_number: 0,
            title: "Income Taxes".to_string(),
            category: NoteCategory::StandardSpecific,
            content_sections: vec![NoteSection {
                heading: "Tax Charge and Deferred Tax Balances".to_string(),
                narrative,
                tables: vec![NoteTable {
                    caption: "Income Tax Summary".to_string(),
                    headers: vec!["Item".to_string(), "Value".to_string()],
                    rows,
                }],
            }],
            cross_references: Vec::new(),
        }
    }

    fn note_provisions(&mut self, ctx: &NotesGeneratorContext) -> FinancialStatementNote {
        let count = ctx.provision_count;
        let total = ctx
            .total_provisions
            .unwrap_or_else(|| Decimal::new(self.rng.random_range(50_000i64..=5_000_000), 0));

        let narrative = format!(
            "Provisions are recognised when the entity has a present obligation \
             (legal or constructive) as a result of a past event, it is probable that \
             an outflow of resources embodying economic benefits will be required to settle \
             the obligation, and a reliable estimate can be made of the amount.  At {} a \
             total of {} provision(s) were recognised with a combined carrying value of \
             {} {:.0}.",
            ctx.period_end, count, ctx.currency, total
        );

        let provision_types = [
            ("Warranty",),
            ("Legal Claims",),
            ("Restructuring",),
            ("Environmental",),
        ];
        let num_rows = count.min(provision_types.len()).max(2);
        let per_provision = if num_rows > 0 {
            total / Decimal::new(num_rows as i64, 0)
        } else {
            total
        };
        let mut rows: Vec<Vec<NoteTableValue>> = provision_types[..num_rows]
            .iter()
            .map(|(name,)| {
                vec![
                    NoteTableValue::Text(name.to_string()),
                    NoteTableValue::Amount(per_provision),
                ]
            })
            .collect();
        rows.push(vec![
            NoteTableValue::Text("Total".to_string()),
            NoteTableValue::Amount(total),
        ]);

        FinancialStatementNote {
            note_number: 0,
            title: "Provisions & Contingencies".to_string(),
            category: NoteCategory::Contingency,
            content_sections: vec![NoteSection {
                heading: "Movement in Provisions".to_string(),
                narrative,
                tables: vec![NoteTable {
                    caption: format!("Provisions at {} ({})", ctx.period_end, ctx.currency),
                    headers: vec![
                        "Provision Type".to_string(),
                        format!("Carrying Amount ({})", ctx.currency),
                    ],
                    rows,
                }],
            }],
            cross_references: vec!["Note 1 — Accounting Policies".to_string()],
        }
    }

    fn note_related_parties(&mut self, ctx: &NotesGeneratorContext) -> FinancialStatementNote {
        let count = ctx.related_party_transaction_count;
        let total = ctx
            .related_party_total_value
            .unwrap_or_else(|| Decimal::new(self.rng.random_range(100_000i64..=10_000_000), 0));

        let narrative = format!(
            "During {} the entity engaged in {} related party transaction(s) with a \
             combined value of {} {:.0}.  All transactions were conducted on an arm's-length \
             basis and have been approved by the board of directors.",
            ctx.period, count, ctx.currency, total
        );

        let rows = vec![
            vec![
                NoteTableValue::Text("Number of transactions".to_string()),
                NoteTableValue::Text(count.to_string()),
            ],
            vec![
                NoteTableValue::Text("Total transaction value".to_string()),
                NoteTableValue::Amount(total),
            ],
        ];

        FinancialStatementNote {
            note_number: 0,
            title: "Related Party Transactions".to_string(),
            category: NoteCategory::RelatedParty,
            content_sections: vec![NoteSection {
                heading: "Transactions with Related Parties".to_string(),
                narrative,
                tables: vec![NoteTable {
                    caption: "Related Party Summary".to_string(),
                    headers: vec!["Item".to_string(), "Value".to_string()],
                    rows,
                }],
            }],
            cross_references: Vec::new(),
        }
    }

    fn note_subsequent_events(&mut self, ctx: &NotesGeneratorContext) -> FinancialStatementNote {
        let count = ctx.subsequent_event_count;
        let adj = ctx.adjusting_event_count;
        let non_adj = count.saturating_sub(adj);

        let narrative = format!(
            "Management has evaluated events and transactions that occurred after the \
             balance sheet date of {} through the financial statement issuance date.  \
             {} event(s) were identified: {} adjusting event(s) and {} non-adjusting \
             event(s).  Non-adjusting events are disclosed but do not result in \
             adjustments to the financial statements.",
            ctx.period_end, count, adj, non_adj
        );

        let rows = vec![
            vec![
                NoteTableValue::Text("Total subsequent events".to_string()),
                NoteTableValue::Text(count.to_string()),
            ],
            vec![
                NoteTableValue::Text("Adjusting (IAS 10.8 / ASC 855)".to_string()),
                NoteTableValue::Text(adj.to_string()),
            ],
            vec![
                NoteTableValue::Text("Non-adjusting — disclosed only".to_string()),
                NoteTableValue::Text(non_adj.to_string()),
            ],
        ];

        FinancialStatementNote {
            note_number: 0,
            title: "Subsequent Events".to_string(),
            category: NoteCategory::SubsequentEvent,
            content_sections: vec![NoteSection {
                heading: "Events after the Reporting Period".to_string(),
                narrative,
                tables: vec![NoteTable {
                    caption: "Subsequent Events Summary".to_string(),
                    headers: vec!["Category".to_string(), "Count".to_string()],
                    rows,
                }],
            }],
            cross_references: Vec::new(),
        }
    }

    fn note_employee_benefits(&mut self, ctx: &NotesGeneratorContext) -> FinancialStatementNote {
        let plan_count = ctx.pension_plan_count;
        let dbo = ctx
            .total_dbo
            .unwrap_or_else(|| Decimal::new(self.rng.random_range(500_000i64..=50_000_000), 0));
        let assets = ctx
            .total_plan_assets
            .unwrap_or_else(|| dbo * Decimal::new(85, 2)); // funded at ~85%
        let funded_status = assets - dbo;

        let narrative = format!(
            "The entity operates {} defined benefit pension plan(s) for qualifying employees.  \
             The defined benefit obligation (DBO) is measured using the Projected Unit Credit \
             method.  At {} the DBO totalled {} {:.0}, while plan assets at fair value \
             amounted to {} {:.0}, resulting in a net funded status of {} {:.0}.",
            plan_count,
            ctx.period_end,
            ctx.currency,
            dbo,
            ctx.currency,
            assets,
            ctx.currency,
            funded_status
        );

        let rows = vec![
            vec![
                NoteTableValue::Text("Number of defined benefit plans".to_string()),
                NoteTableValue::Text(plan_count.to_string()),
            ],
            vec![
                NoteTableValue::Text("Defined Benefit Obligation (DBO)".to_string()),
                NoteTableValue::Amount(dbo),
            ],
            vec![
                NoteTableValue::Text("Plan assets at fair value".to_string()),
                NoteTableValue::Amount(assets),
            ],
            vec![
                NoteTableValue::Text("Net funded status".to_string()),
                NoteTableValue::Amount(funded_status),
            ],
        ];

        FinancialStatementNote {
            note_number: 0,
            title: "Employee Benefits".to_string(),
            category: NoteCategory::StandardSpecific,
            content_sections: vec![NoteSection {
                heading: "Defined Benefit Pension Plans".to_string(),
                narrative,
                tables: vec![NoteTable {
                    caption: format!(
                        "Pension Plan Summary at {} ({})",
                        ctx.period_end, ctx.currency
                    ),
                    headers: vec!["Item".to_string(), "Value".to_string()],
                    rows,
                }],
            }],
            cross_references: vec!["Note 1 — Accounting Policies".to_string()],
        }
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn default_context() -> NotesGeneratorContext {
        NotesGeneratorContext {
            entity_code: "C001".to_string(),
            framework: "IFRS".to_string(),
            period: "FY2024".to_string(),
            period_end: NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            currency: "USD".to_string(),
            revenue_contract_count: 50,
            revenue_amount: Some(Decimal::new(10_000_000, 0)),
            avg_obligations_per_contract: Some(Decimal::new(2, 0)),
            total_ppe_gross: Some(Decimal::new(5_000_000, 0)),
            accumulated_depreciation: Some(Decimal::new(1_500_000, 0)),
            statutory_tax_rate: Some(Decimal::new(21, 2)),
            effective_tax_rate: Some(Decimal::new(24, 2)),
            deferred_tax_asset: Some(Decimal::new(200_000, 0)),
            deferred_tax_liability: Some(Decimal::new(50_000, 0)),
            provision_count: 4,
            total_provisions: Some(Decimal::new(800_000, 0)),
            related_party_transaction_count: 12,
            related_party_total_value: Some(Decimal::new(2_500_000, 0)),
            subsequent_event_count: 3,
            adjusting_event_count: 1,
            pension_plan_count: 2,
            total_dbo: Some(Decimal::new(15_000_000, 0)),
            total_plan_assets: Some(Decimal::new(13_000_000, 0)),
        }
    }

    #[test]
    fn test_at_least_three_notes_generated() {
        let mut gen = NotesGenerator::new(42);
        let ctx = default_context();
        let notes = gen.generate(&ctx);
        assert!(
            notes.len() >= 3,
            "Expected at least 3 notes, got {}",
            notes.len()
        );
    }

    #[test]
    fn test_note_numbers_are_sequential() {
        let mut gen = NotesGenerator::new(42);
        let ctx = default_context();
        let notes = gen.generate(&ctx);
        for (i, note) in notes.iter().enumerate() {
            assert_eq!(
                note.note_number,
                (i + 1) as u32,
                "Note at index {} has number {}, expected {}",
                i,
                note.note_number,
                i + 1
            );
        }
    }

    #[test]
    fn test_every_note_has_title_and_content() {
        let mut gen = NotesGenerator::new(42);
        let ctx = default_context();
        let notes = gen.generate(&ctx);
        for note in &notes {
            assert!(
                !note.title.is_empty(),
                "Note {} has an empty title",
                note.note_number
            );
            assert!(
                !note.content_sections.is_empty(),
                "Note '{}' has no content sections",
                note.title
            );
        }
    }

    #[test]
    fn test_accounting_policy_note_always_first() {
        let mut gen = NotesGenerator::new(42);
        let ctx = default_context();
        let notes = gen.generate(&ctx);
        assert!(!notes.is_empty());
        assert_eq!(notes[0].note_number, 1);
        assert!(
            notes[0].title.contains("Accounting Policies"),
            "First note should be Accounting Policies, got '{}'",
            notes[0].title
        );
    }

    #[test]
    fn test_no_revenue_note_when_no_revenue_data() {
        let mut gen = NotesGenerator::new(42);
        let ctx = NotesGeneratorContext {
            entity_code: "C001".to_string(),
            framework: "US GAAP".to_string(),
            period: "FY2024".to_string(),
            period_end: NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            currency: "USD".to_string(),
            ..NotesGeneratorContext::default()
        };
        let notes = gen.generate(&ctx);
        // Should still have at least the accounting policies note
        assert!(!notes.is_empty());
        let has_revenue_note = notes.iter().any(|n| n.title.contains("Revenue"));
        assert!(
            !has_revenue_note,
            "Should not generate revenue note when no data"
        );
    }

    #[test]
    fn test_deterministic_output() {
        let ctx = default_context();
        let notes1 = NotesGenerator::new(42).generate(&ctx);
        let notes2 = NotesGenerator::new(42).generate(&ctx);
        assert_eq!(notes1.len(), notes2.len());
        for (a, b) in notes1.iter().zip(notes2.iter()) {
            assert_eq!(a.note_number, b.note_number);
            assert_eq!(a.title, b.title);
        }
    }
}
