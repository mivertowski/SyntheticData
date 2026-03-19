//! Integration tests for ISA 700 audit opinion generator and SOX 302/404.
//!
//! Verifies:
//! - Opinion type determination from findings
//! - Emphasis of Matter for going concern
//! - Key Audit Matters generation
//! - SOX 302 certifications (CEO + CFO)
//! - SOX 404 effectiveness determination
//! - Batch generation across multiple entities

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod audit_opinion_integration_tests {
    use chrono::NaiveDate;
    use datasynth_core::models::audit::{AuditFinding, FindingStatus, FindingType};
    use datasynth_generators::audit::audit_opinion_generator::{
        AuditOpinionGenerator, AuditOpinionInput,
    };
    use datasynth_standards::audit::opinion::{EomMatter, OpinionType};
    use uuid::Uuid;

    fn period_end() -> NaiveDate {
        NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()
    }

    fn make_finding(eng_id: Uuid, ftype: FindingType) -> AuditFinding {
        AuditFinding::new(eng_id, ftype, &format!("{ftype:?} finding"))
    }

    fn base_input(entity_code: &str) -> AuditOpinionInput {
        AuditOpinionInput {
            entity_code: entity_code.to_string(),
            entity_name: format!("{entity_code} Ltd"),
            engagement_id: Uuid::new_v4(),
            period_end: period_end(),
            findings: Vec::new(),
            going_concern: None,
            component_reports: Vec::new(),
            is_us_listed: false,
            auditor_name: "Test Audit LLP".into(),
            engagement_partner: "Jane Senior".into(),
        }
    }

    // -----------------------------------------------------------------------
    // Opinion type tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_unmodified_opinion_with_no_findings() {
        let mut gen = AuditOpinionGenerator::new(1);
        let result = gen.generate(&base_input("E001"));
        assert_eq!(result.opinion.opinion_type, OpinionType::Unmodified);
        assert!(result.opinion.modification.is_none());
    }

    #[test]
    fn test_unmodified_opinion_with_only_closed_material_weakness() {
        let mut gen = AuditOpinionGenerator::new(2);
        let eng_id = Uuid::new_v4();

        let mut finding = make_finding(eng_id, FindingType::MaterialWeakness);
        finding.status = FindingStatus::Closed; // remediated — should not trigger modification

        let mut input = base_input("E002");
        input.engagement_id = eng_id;
        input.findings = vec![finding];

        let result = gen.generate(&input);
        assert_eq!(result.opinion.opinion_type, OpinionType::Unmodified);
    }

    #[test]
    fn test_qualified_opinion_with_one_open_material_weakness() {
        let mut gen = AuditOpinionGenerator::new(3);
        let eng_id = Uuid::new_v4();

        let finding = make_finding(eng_id, FindingType::MaterialWeakness);
        let mut input = base_input("E003");
        input.engagement_id = eng_id;
        input.findings = vec![finding]; // default status is Draft = open

        let result = gen.generate(&input);
        assert_eq!(result.opinion.opinion_type, OpinionType::Qualified);
        assert!(result.opinion.modification.is_some());
    }

    #[test]
    fn test_qualified_opinion_with_two_open_material_misstatements() {
        let mut gen = AuditOpinionGenerator::new(4);
        let eng_id = Uuid::new_v4();

        let findings: Vec<AuditFinding> = (0..2)
            .map(|_| make_finding(eng_id, FindingType::MaterialMisstatement))
            .collect();

        let mut input = base_input("E004");
        input.engagement_id = eng_id;
        input.findings = findings;

        let result = gen.generate(&input);
        assert_eq!(result.opinion.opinion_type, OpinionType::Qualified);
    }

    #[test]
    fn test_adverse_opinion_with_three_or_more_material_weaknesses() {
        let mut gen = AuditOpinionGenerator::new(5);
        let eng_id = Uuid::new_v4();

        let findings: Vec<AuditFinding> = (0..4)
            .map(|_| make_finding(eng_id, FindingType::MaterialWeakness))
            .collect();

        let mut input = base_input("E005");
        input.engagement_id = eng_id;
        input.findings = findings;

        let result = gen.generate(&input);
        assert_eq!(result.opinion.opinion_type, OpinionType::Adverse);
        let modification = result.opinion.modification.as_ref().unwrap();
        assert!(
            modification.is_pervasive,
            "Adverse opinion must be pervasive"
        );
    }

    #[test]
    fn test_disclaimer_opinion_when_component_report_has_scope_limitation() {
        use datasynth_core::models::audit::ComponentAuditorReport;

        let mut gen = AuditOpinionGenerator::new(6);
        let eng_id = Uuid::new_v4();

        let report = ComponentAuditorReport {
            id: "RPT-001".into(),
            instruction_id: "INST-001".into(),
            component_auditor_id: "CA-001".into(),
            entity_code: "E006".into(),
            misstatements_identified: Vec::new(),
            scope_limitations: vec![
                "We were unable to attend the inventory count due to travel restrictions.".into(),
            ],
            significant_findings: Vec::new(),
            conclusion: "Scope limitation identified.".into(),
        };

        let mut input = base_input("E006");
        input.engagement_id = eng_id;
        input.component_reports = vec![report];

        let result = gen.generate(&input);
        assert_eq!(result.opinion.opinion_type, OpinionType::Disclaimer);
    }

    // -----------------------------------------------------------------------
    // Going concern tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_no_emphasis_of_matter_when_no_going_concern_issues() {
        use datasynth_core::models::audit::going_concern::{
            GoingConcernAssessment, GoingConcernConclusion,
        };

        let mut gen = AuditOpinionGenerator::new(7);
        let gc = GoingConcernAssessment {
            entity_code: "E007".into(),
            assessment_date: period_end(),
            assessment_period: "FY2024".into(),
            indicators: Vec::new(),
            management_plans: Vec::new(),
            auditor_conclusion: GoingConcernConclusion::NoMaterialUncertainty,
            material_uncertainty_exists: false,
        };

        let mut input = base_input("E007");
        input.going_concern = Some(gc);

        let result = gen.generate(&input);
        assert!(!result.opinion.material_uncertainty_going_concern);
        // No going concern EOM should appear.
        let gc_eoms: Vec<_> = result
            .opinion
            .emphasis_of_matter
            .iter()
            .filter(|e| matches!(e.matter, EomMatter::GoingConcern))
            .collect();
        assert!(gc_eoms.is_empty());
    }

    #[test]
    fn test_emphasis_of_matter_added_for_material_uncertainty() {
        use datasynth_core::models::audit::going_concern::{
            GoingConcernAssessment, GoingConcernConclusion,
        };

        let mut gen = AuditOpinionGenerator::new(8);
        let gc = GoingConcernAssessment {
            entity_code: "E008".into(),
            assessment_date: period_end(),
            assessment_period: "FY2024".into(),
            indicators: Vec::new(),
            management_plans: Vec::new(),
            auditor_conclusion: GoingConcernConclusion::MaterialUncertaintyExists,
            material_uncertainty_exists: true,
        };

        let mut input = base_input("E008");
        input.going_concern = Some(gc);

        let result = gen.generate(&input);
        assert!(result.opinion.material_uncertainty_going_concern);
        assert!(!result.opinion.emphasis_of_matter.is_empty());
        assert!(result
            .opinion
            .emphasis_of_matter
            .iter()
            .any(|e| matches!(e.matter, EomMatter::GoingConcern)));
    }

    // -----------------------------------------------------------------------
    // Key Audit Matters tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_at_least_one_kam_generated() {
        let mut gen = AuditOpinionGenerator::new(9);
        let result = gen.generate(&base_input("E009"));
        assert!(!result.key_audit_matters.is_empty());
    }

    #[test]
    fn test_kams_embedded_in_opinion() {
        let mut gen = AuditOpinionGenerator::new(10);
        let result = gen.generate(&base_input("E010"));
        assert_eq!(
            result.opinion.key_audit_matters.len(),
            result.key_audit_matters.len()
        );
    }

    #[test]
    fn test_listed_entity_gets_up_to_three_kams() {
        let mut gen = AuditOpinionGenerator::new(11);
        let mut input = base_input("E011");
        input.is_us_listed = true;

        let result = gen.generate(&input);
        assert!(result.key_audit_matters.len() <= 3);
        assert!(!result.key_audit_matters.is_empty());
    }

    // -----------------------------------------------------------------------
    // PCAOB / US-listed tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_pcaob_elements_present_for_listed_entity() {
        let mut gen = AuditOpinionGenerator::new(12);
        let mut input = base_input("E012");
        input.is_us_listed = true;

        let result = gen.generate(&input);
        assert!(result.opinion.pcaob_compliance.is_some());
        let pcaob = result.opinion.pcaob_compliance.as_ref().unwrap();
        assert!(pcaob.is_integrated_audit);
        assert!(pcaob.icfr_opinion.is_some());
    }

    #[test]
    fn test_pcaob_absent_for_non_listed_entity() {
        let mut gen = AuditOpinionGenerator::new(13);
        let input = base_input("E013"); // is_us_listed = false

        let result = gen.generate(&input);
        assert!(result.opinion.pcaob_compliance.is_none());
    }

    #[test]
    fn test_eqcr_performed_for_modified_opinion() {
        let mut gen = AuditOpinionGenerator::new(14);
        let eng_id = Uuid::new_v4();

        let finding = make_finding(eng_id, FindingType::MaterialWeakness);
        let mut input = base_input("E014");
        input.engagement_id = eng_id;
        input.findings = vec![finding];

        let result = gen.generate(&input);
        assert!(result.opinion.eqcr_performed);
    }

    // -----------------------------------------------------------------------
    // Batch generation
    // -----------------------------------------------------------------------

    #[test]
    fn test_batch_generates_one_opinion_per_input() {
        let mut gen = AuditOpinionGenerator::new(99);
        let inputs: Vec<AuditOpinionInput> = (0..8)
            .map(|i| base_input(&format!("BATCH{:03}", i)))
            .collect();

        let results = gen.generate_batch(&inputs);
        assert_eq!(results.len(), inputs.len());

        for result in &results {
            assert!(!result.key_audit_matters.is_empty());
        }
    }

    #[test]
    fn test_deterministic_with_same_seed() {
        let input = base_input("DETERM001");

        let result_a = {
            let mut gen = AuditOpinionGenerator::new(777);
            gen.generate(&input)
        };
        let result_b = {
            let mut gen = AuditOpinionGenerator::new(777);
            gen.generate(&input)
        };

        assert_eq!(result_a.opinion.opinion_type, result_b.opinion.opinion_type);
        assert_eq!(
            result_a.key_audit_matters.len(),
            result_b.key_audit_matters.len()
        );
    }
}

// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod sox_integration_tests {
    use chrono::NaiveDate;
    use datasynth_core::models::audit::{AuditFinding, FindingStatus, FindingType};
    use datasynth_generators::audit::sox_generator::{SoxGenerator, SoxGeneratorInput};
    use datasynth_standards::regulatory::sox::{CertifierRole, IcfrFramework};
    use uuid::Uuid;

    fn period_end() -> NaiveDate {
        NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()
    }

    fn base_input() -> SoxGeneratorInput {
        SoxGeneratorInput {
            company_code: "C001".into(),
            company_name: "Test Corp Inc.".into(),
            fiscal_year: 2024,
            period_end: period_end(),
            ..Default::default()
        }
    }

    fn open_mw(eng_id: Uuid) -> AuditFinding {
        AuditFinding::new(eng_id, FindingType::MaterialWeakness, "AP SoD gap")
    }

    // -----------------------------------------------------------------------
    // SOX 302 tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_two_certifications_produced_ceo_and_cfo() {
        let mut gen = SoxGenerator::new(42);
        let (certs, _) = gen.generate(&base_input());
        assert_eq!(certs.len(), 2);
        assert!(certs.iter().any(|c| c.certifier_role == CertifierRole::Ceo));
        assert!(certs.iter().any(|c| c.certifier_role == CertifierRole::Cfo));
    }

    #[test]
    fn test_certifications_have_non_empty_text() {
        let mut gen = SoxGenerator::new(42);
        let (certs, _) = gen.generate(&base_input());
        for cert in &certs {
            assert!(
                !cert.certification_text.is_empty(),
                "Certification text should not be empty"
            );
        }
    }

    #[test]
    fn test_certifications_effective_when_no_material_weaknesses() {
        let mut gen = SoxGenerator::new(42);
        let (certs, _) = gen.generate(&base_input());
        for cert in &certs {
            assert!(cert.disclosure_controls_effective);
            assert!(cert.material_weaknesses.is_empty());
        }
    }

    #[test]
    fn test_certifications_reflect_material_weakness() {
        let mut gen = SoxGenerator::new(42);
        let eng_id = Uuid::new_v4();

        let mut input = base_input();
        input.findings = vec![open_mw(eng_id)];

        let (certs, _) = gen.generate(&input);
        for cert in &certs {
            assert!(!cert.disclosure_controls_effective);
            assert!(!cert.material_weaknesses.is_empty());
        }
    }

    #[test]
    fn test_certification_date_after_period_end() {
        let mut gen = SoxGenerator::new(42);
        let (certs, _) = gen.generate(&base_input());
        for cert in &certs {
            assert!(cert.certification_date > period_end());
        }
    }

    // -----------------------------------------------------------------------
    // SOX 404 tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_icfr_effective_when_no_material_weaknesses() {
        let mut gen = SoxGenerator::new(42);
        let (_, assessment) = gen.generate(&base_input());
        assert!(assessment.icfr_effective);
        assert!(assessment.material_weaknesses.is_empty());
        assert!(assessment.management_conclusion.contains("effective"));
    }

    #[test]
    fn test_icfr_ineffective_when_open_material_weakness_present() {
        let mut gen = SoxGenerator::new(42);
        let eng_id = Uuid::new_v4();

        let mut input = base_input();
        input.findings = vec![open_mw(eng_id)];

        let (_, assessment) = gen.generate(&input);
        assert!(!assessment.icfr_effective);
        assert!(!assessment.material_weaknesses.is_empty());
        assert!(assessment
            .management_conclusion
            .contains("material weakness"));
    }

    #[test]
    fn test_significant_deficiency_alone_does_not_make_icfr_ineffective() {
        let mut gen = SoxGenerator::new(42);
        let eng_id = Uuid::new_v4();

        let mut input = base_input();
        input.findings = vec![AuditFinding::new(
            eng_id,
            FindingType::SignificantDeficiency,
            "Reconciliation gap",
        )];

        let (_, assessment) = gen.generate(&input);
        assert!(assessment.icfr_effective);
        assert!(!assessment.significant_deficiencies.is_empty());
    }

    #[test]
    fn test_closed_material_weakness_does_not_make_icfr_ineffective() {
        let mut gen = SoxGenerator::new(42);
        let eng_id = Uuid::new_v4();

        let mut finding = open_mw(eng_id);
        finding.status = FindingStatus::Closed; // remediated

        let mut input = base_input();
        input.findings = vec![finding];

        let (_, assessment) = gen.generate(&input);
        assert!(assessment.icfr_effective);
    }

    #[test]
    fn test_coso_2013_framework_is_default() {
        let mut gen = SoxGenerator::new(42);
        let (_, assessment) = gen.generate(&base_input());
        assert_eq!(assessment.framework, IcfrFramework::Coso2013);
    }

    #[test]
    fn test_scoped_entity_included() {
        let mut gen = SoxGenerator::new(42);
        let (_, assessment) = gen.generate(&base_input());
        assert!(!assessment.scope.is_empty());
        assert_eq!(assessment.scope[0].entity_code, "C001");
    }

    #[test]
    fn test_remediation_action_generated_for_open_material_weakness() {
        let mut gen = SoxGenerator::new(42);
        let eng_id = Uuid::new_v4();

        let mut input = base_input();
        input.findings = vec![open_mw(eng_id)]; // open, no remediation yet

        let (_, assessment) = gen.generate(&input);
        assert!(!assessment.remediation_actions.is_empty());
    }

    #[test]
    fn test_no_remediation_action_when_mw_already_closed() {
        let mut gen = SoxGenerator::new(42);
        let eng_id = Uuid::new_v4();

        let mut finding = open_mw(eng_id);
        finding.status = FindingStatus::Closed;

        let mut input = base_input();
        input.findings = vec![finding];

        let (_, assessment) = gen.generate(&input);
        // Closed MW: no remediation action needed.
        assert!(
            assessment.remediation_actions.is_empty()
                || assessment
                    .remediation_actions
                    .iter()
                    .all(|a| a.completion_date.is_some())
        );
    }

    #[test]
    fn test_assessment_date_after_period_end() {
        let mut gen = SoxGenerator::new(42);
        let (_, assessment) = gen.generate(&base_input());
        assert!(assessment.assessment_date > period_end());
    }

    #[test]
    fn test_batch_generation() {
        let mut gen = SoxGenerator::new(42);
        let inputs: Vec<SoxGeneratorInput> = (0..5)
            .map(|i| SoxGeneratorInput {
                company_code: format!("C{:03}", i + 1),
                company_name: format!("Company {}", i + 1),
                ..Default::default()
            })
            .collect();
        let results = gen.generate_batch(&inputs);
        assert_eq!(results.len(), 5);
        for (certs, assessment) in &results {
            assert_eq!(certs.len(), 2); // CEO + CFO
            assert!(assessment.icfr_effective); // No findings → effective
        }
    }
}
