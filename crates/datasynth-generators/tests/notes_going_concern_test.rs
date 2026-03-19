//! Integration tests for Notes to Financial Statements (Task 3.4) and
//! Going Concern Indicators (Task 3.5).

use chrono::NaiveDate;
use datasynth_core::models::audit::going_concern::GoingConcernConclusion;
use datasynth_generators::audit::going_concern_generator::GoingConcernGenerator;
use datasynth_generators::period_close::notes_generator::{NotesGenerator, NotesGeneratorContext};
use rust_decimal::Decimal;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn period_end() -> NaiveDate {
    NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()
}

fn assessment_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2025, 3, 15).unwrap()
}

fn full_context() -> NotesGeneratorContext {
    NotesGeneratorContext {
        entity_code: "C001".to_string(),
        framework: "IFRS".to_string(),
        period: "FY2024".to_string(),
        period_end: period_end(),
        currency: "USD".to_string(),
        revenue_contract_count: 75,
        revenue_amount: Some(Decimal::new(25_000_000, 0)),
        avg_obligations_per_contract: Some(Decimal::new(2, 0)),
        total_ppe_gross: Some(Decimal::new(8_000_000, 0)),
        accumulated_depreciation: Some(Decimal::new(2_400_000, 0)),
        statutory_tax_rate: Some(Decimal::new(21, 2)),
        effective_tax_rate: Some(Decimal::new(23, 2)),
        deferred_tax_asset: Some(Decimal::new(350_000, 0)),
        deferred_tax_liability: Some(Decimal::new(120_000, 0)),
        provision_count: 5,
        total_provisions: Some(Decimal::new(1_200_000, 0)),
        related_party_transaction_count: 8,
        related_party_total_value: Some(Decimal::new(4_500_000, 0)),
        subsequent_event_count: 2,
        adjusting_event_count: 1,
        pension_plan_count: 3,
        total_dbo: Some(Decimal::new(20_000_000, 0)),
        total_plan_assets: Some(Decimal::new(17_000_000, 0)),
    }
}

// ===========================================================================
// Notes to Financial Statements — Task 3.4
// ===========================================================================

#[test]
fn notes_at_least_three_generated_with_full_context() {
    let mut gen = NotesGenerator::new(42);
    let notes = gen.generate(&full_context());
    assert!(
        notes.len() >= 3,
        "Expected at least 3 notes, got {}",
        notes.len()
    );
}

#[test]
fn notes_sequential_numbers_starting_at_one() {
    let mut gen = NotesGenerator::new(42);
    let notes = gen.generate(&full_context());
    for (i, note) in notes.iter().enumerate() {
        assert_eq!(
            note.note_number,
            (i + 1) as u32,
            "Note at index {} has note_number {}, expected {}",
            i,
            note.note_number,
            i + 1
        );
    }
}

#[test]
fn notes_every_note_has_title() {
    let mut gen = NotesGenerator::new(42);
    let notes = gen.generate(&full_context());
    for note in &notes {
        assert!(
            !note.title.is_empty(),
            "Note {} has an empty title",
            note.note_number
        );
    }
}

#[test]
fn notes_every_note_has_at_least_one_content_section() {
    let mut gen = NotesGenerator::new(42);
    let notes = gen.generate(&full_context());
    for note in &notes {
        assert!(
            !note.content_sections.is_empty(),
            "Note '{}' (#{}) has no content sections",
            note.title,
            note.note_number
        );
    }
}

#[test]
fn notes_content_sections_have_non_empty_narratives() {
    let mut gen = NotesGenerator::new(42);
    let notes = gen.generate(&full_context());
    for note in &notes {
        for section in &note.content_sections {
            assert!(
                !section.narrative.is_empty(),
                "Note '{}' section '{}' has empty narrative",
                note.title,
                section.heading
            );
        }
    }
}

#[test]
fn notes_first_note_is_accounting_policies() {
    let mut gen = NotesGenerator::new(42);
    let notes = gen.generate(&full_context());
    assert!(!notes.is_empty(), "Should generate at least one note");
    assert!(
        notes[0].title.contains("Accounting Policies"),
        "First note should be Accounting Policies, got '{}'",
        notes[0].title
    );
}

#[test]
fn notes_omit_absent_data_sections() {
    // Minimal context — only accounting policies note expected
    let ctx = NotesGeneratorContext {
        entity_code: "C002".to_string(),
        framework: "US GAAP".to_string(),
        period: "FY2024".to_string(),
        period_end: period_end(),
        currency: "EUR".to_string(),
        ..NotesGeneratorContext::default()
    };
    let mut gen = NotesGenerator::new(42);
    let notes = gen.generate(&ctx);
    // Should have accounting policy note but no revenue/PPE/tax/provisions etc.
    assert_eq!(
        notes.len(),
        1,
        "Expected 1 note for minimal context, got {}",
        notes.len()
    );
    assert!(notes[0].title.contains("Accounting Policies"));
}

#[test]
fn notes_deterministic_with_same_seed() {
    let ctx = full_context();
    let notes_a = NotesGenerator::new(99).generate(&ctx);
    let notes_b = NotesGenerator::new(99).generate(&ctx);
    assert_eq!(notes_a.len(), notes_b.len());
    for (a, b) in notes_a.iter().zip(notes_b.iter()) {
        assert_eq!(a.note_number, b.note_number);
        assert_eq!(a.title, b.title);
        assert_eq!(a.category, b.category);
        assert_eq!(a.content_sections.len(), b.content_sections.len());
    }
}

#[test]
fn notes_can_serialize_to_json() {
    let mut gen = NotesGenerator::new(42);
    let notes = gen.generate(&full_context());
    let json = serde_json::to_string_pretty(&notes).expect("Should serialize to JSON");
    assert!(json.contains("note_number"));
    assert!(json.contains("title"));
}

#[test]
fn notes_tables_have_matching_row_widths() {
    let mut gen = NotesGenerator::new(42);
    let notes = gen.generate(&full_context());
    for note in &notes {
        for section in &note.content_sections {
            for table in &section.tables {
                for (ri, row) in table.rows.iter().enumerate() {
                    assert_eq!(
                        row.len(),
                        table.headers.len(),
                        "Note '{}' table '{}' row {} has {} cells but {} headers",
                        note.title,
                        table.caption,
                        ri,
                        row.len(),
                        table.headers.len()
                    );
                }
            }
        }
    }
}

// ===========================================================================
// Going Concern Indicators — Task 3.5
// ===========================================================================

#[test]
fn gc_one_assessment_per_entity() {
    let entities = vec!["C001".to_string(), "C002".to_string(), "C003".to_string()];
    let mut gen = GoingConcernGenerator::new(42);
    let assessments = gen.generate_for_entities(&entities, assessment_date(), "FY2024");
    assert_eq!(
        assessments.len(),
        3,
        "Expected one assessment per entity, got {}",
        assessments.len()
    );
    // Verify entity codes match
    let codes: Vec<&str> = assessments.iter().map(|a| a.entity_code.as_str()).collect();
    for entity in &entities {
        assert!(
            codes.contains(&entity.as_str()),
            "Missing assessment for entity {}",
            entity
        );
    }
}

#[test]
fn gc_approximately_90_percent_no_material_uncertainty() {
    let mut clean = 0usize;
    let total = 200usize;
    for seed in 0..total as u64 {
        let mut gen = GoingConcernGenerator::new(seed);
        let a = gen.generate_for_entity("C001", assessment_date(), "FY2024");
        if matches!(
            a.auditor_conclusion,
            GoingConcernConclusion::NoMaterialUncertainty
        ) {
            clean += 1;
        }
    }
    let ratio = clean as f64 / total as f64;
    assert!(
        ratio >= 0.80 && ratio <= 0.98,
        "Clean ratio {:.2} is outside expected range [0.80, 0.98]",
        ratio
    );
}

#[test]
fn gc_conclusion_consistent_with_indicator_count() {
    for seed in 0..150u64 {
        let mut gen = GoingConcernGenerator::new(seed);
        let a = gen.generate_for_entity("C001", assessment_date(), "FY2024");
        let n = a.indicators.len();
        match a.auditor_conclusion {
            GoingConcernConclusion::NoMaterialUncertainty => {
                assert_eq!(
                    n, 0,
                    "seed={}: NoMaterialUncertainty but {} indicators present",
                    seed, n
                );
            }
            GoingConcernConclusion::MaterialUncertaintyExists => {
                assert!(
                    n >= 1 && n <= 2,
                    "seed={}: MaterialUncertaintyExists but {} indicators (expected 1–2)",
                    seed,
                    n
                );
            }
            GoingConcernConclusion::GoingConcernDoubt => {
                assert!(
                    n >= 3,
                    "seed={}: GoingConcernDoubt but only {} indicators (expected 3+)",
                    seed,
                    n
                );
            }
        }
    }
}

#[test]
fn gc_indicators_have_severity() {
    for seed in 0..50u64 {
        let mut gen = GoingConcernGenerator::new(seed);
        let a = gen.generate_for_entity("C001", assessment_date(), "FY2024");
        for indicator in &a.indicators {
            // Confirm severity serialises cleanly (enum exhaustively covered)
            let _s = format!("{:?}", indicator.severity);
            assert!(
                !indicator.description.is_empty(),
                "Indicator description must not be empty"
            );
        }
    }
}

#[test]
fn gc_material_uncertainty_flag_matches_conclusion() {
    for seed in 0..150u64 {
        let mut gen = GoingConcernGenerator::new(seed);
        let a = gen.generate_for_entity("C001", assessment_date(), "FY2024");
        match a.auditor_conclusion {
            GoingConcernConclusion::NoMaterialUncertainty => {
                assert!(
                    !a.material_uncertainty_exists,
                    "seed={}: Clean conclusion but material_uncertainty_exists=true",
                    seed
                );
            }
            _ => {
                assert!(
                    a.material_uncertainty_exists,
                    "seed={}: Uncertain/doubt conclusion but material_uncertainty_exists=false",
                    seed
                );
            }
        }
    }
}

#[test]
fn gc_management_plans_present_when_indicators_exist() {
    for seed in 0..200u64 {
        let mut gen = GoingConcernGenerator::new(seed);
        let a = gen.generate_for_entity("C001", assessment_date(), "FY2024");
        if !a.indicators.is_empty() {
            assert!(
                !a.management_plans.is_empty(),
                "seed={}: Indicators present but no management plans",
                seed
            );
        } else {
            assert!(
                a.management_plans.is_empty(),
                "seed={}: No indicators but management plans present",
                seed
            );
        }
    }
}

#[test]
fn gc_can_serialize_to_json() {
    let mut gen = GoingConcernGenerator::new(42);
    let a = gen.generate_for_entity("C001", assessment_date(), "FY2024");
    let json = serde_json::to_string_pretty(&a).expect("Should serialize to JSON");
    assert!(json.contains("entity_code"));
    assert!(json.contains("auditor_conclusion"));
}

#[test]
fn gc_deterministic_with_same_seed() {
    let mut gen1 = GoingConcernGenerator::new(42);
    let mut gen2 = GoingConcernGenerator::new(42);
    let a1 = gen1.generate_for_entity("C001", assessment_date(), "FY2024");
    let a2 = gen2.generate_for_entity("C001", assessment_date(), "FY2024");
    assert_eq!(a1.indicators.len(), a2.indicators.len());
    assert_eq!(
        format!("{:?}", a1.auditor_conclusion),
        format!("{:?}", a2.auditor_conclusion)
    );
}

#[test]
fn gc_quantitative_measures_on_indicators() {
    // Run many seeds looking for an entity with indicators; verify measures present.
    let mut found_with_indicators = false;
    for seed in 0..200u64 {
        let mut gen = GoingConcernGenerator::new(seed);
        let a = gen.generate_for_entity("C001", assessment_date(), "FY2024");
        if !a.indicators.is_empty() {
            found_with_indicators = true;
            for ind in &a.indicators {
                assert!(
                    ind.quantitative_measure.is_some(),
                    "seed={}: indicator {:?} missing quantitative_measure",
                    seed,
                    ind.indicator_type
                );
                assert!(
                    ind.threshold.is_some(),
                    "seed={}: indicator {:?} missing threshold",
                    seed,
                    ind.indicator_type
                );
            }
        }
    }
    assert!(
        found_with_indicators,
        "Did not encounter any entity with going concern indicators across 200 seeds"
    );
}
