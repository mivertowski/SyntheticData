//! Integration tests for newly wired generators (Phases 24-29).
//!
//! These tests verify that the following generators produce correct output when
//! wired into the `EnhancedOrchestrator`:
//!
//! - Phase 24: Process evolution events + organizational events
//! - Phase 24b: Disruption events
//! - Phase 25: Counterfactual JE pairs
//! - Phase 26: Fraud red-flag indicators
//! - Phase 26b: Collusion rings
//! - Phase 27: Bi-temporal vendor version chains
//! - Phase 28: Entity relationship graph + cross-process links
//! - Phase 29: Industry-specific GL accounts

#[allow(clippy::unwrap_used)]
mod tests {
    use datasynth_runtime::{EnhancedOrchestrator, PhaseConfig};
    use datasynth_test_utils::fixtures::{fraud_enabled_config, minimal_config};

    // ========================================================================
    // Helper: create a PhaseConfig with everything off except what we need.
    // ========================================================================

    fn base_phase_config() -> PhaseConfig {
        PhaseConfig {
            generate_master_data: false,
            generate_document_flows: false,
            generate_ocpm_events: false,
            generate_journal_entries: false,
            inject_anomalies: false,
            inject_data_quality: false,
            validate_balances: false,
            show_progress: false,
            generate_audit: false,
            generate_banking: false,
            generate_graph_export: false,
            generate_sourcing: false,
            generate_bank_reconciliation: false,
            generate_financial_statements: false,
            generate_accounting_standards: false,
            generate_manufacturing: false,
            generate_sales_kpi_budgets: false,
            generate_tax: false,
            generate_esg: false,
            generate_intercompany: false,
            generate_evolution_events: false,
            generate_counterfactuals: false,
            ..Default::default()
        }
    }

    // ========================================================================
    // 1. Evolution events generated when enabled
    // ========================================================================

    #[test]
    fn test_evolution_events_generated_when_enabled() {
        let mut config = minimal_config();
        config.global.period_months = 3;

        let phase_config = PhaseConfig {
            generate_journal_entries: true,
            generate_evolution_events: true,
            show_progress: false,
            ..base_phase_config()
        };

        let mut orchestrator =
            EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");
        let result = orchestrator.generate().expect("Generation failed");

        assert!(
            !result.process_evolution.is_empty(),
            "process_evolution should not be empty when evolution events are enabled"
        );
        assert!(
            !result.organizational_events.is_empty(),
            "organizational_events should not be empty when evolution events are enabled"
        );
        assert!(
            result.statistics.process_evolution_event_count > 0,
            "process_evolution_event_count should be > 0"
        );
        assert!(
            result.statistics.organizational_event_count > 0,
            "organizational_event_count should be > 0"
        );
    }

    // ========================================================================
    // 2. Evolution events empty when disabled
    // ========================================================================

    #[test]
    fn test_evolution_events_empty_when_disabled() {
        let mut config = minimal_config();
        config.global.period_months = 3;

        let phase_config = PhaseConfig {
            generate_journal_entries: true,
            generate_evolution_events: false,
            show_progress: false,
            ..base_phase_config()
        };

        let mut orchestrator =
            EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");
        let result = orchestrator.generate().expect("Generation failed");

        assert!(
            result.process_evolution.is_empty(),
            "process_evolution should be empty when evolution events are disabled"
        );
        assert!(
            result.organizational_events.is_empty(),
            "organizational_events should be empty when evolution events are disabled"
        );
        // Note: disruption_events are gated by config.organizational_events.enabled,
        // not by phase_config.generate_evolution_events. See test_disruption_events_generated.
    }

    // ========================================================================
    // 3. Counterfactuals generated when enabled
    // ========================================================================

    #[test]
    fn test_counterfactuals_generated_when_enabled() {
        let config = minimal_config();

        let phase_config = PhaseConfig {
            generate_journal_entries: true,
            generate_counterfactuals: true,
            show_progress: false,
            ..base_phase_config()
        };

        let mut orchestrator =
            EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");
        let result = orchestrator.generate().expect("Generation failed");

        assert!(
            !result.counterfactual_pairs.is_empty(),
            "counterfactual_pairs should not be empty when generation is enabled and JEs exist"
        );
        assert!(
            result.statistics.counterfactual_pair_count > 0,
            "counterfactual_pair_count should be > 0"
        );

        // Verify each pair has both original and mutated JEs with valid structure
        for pair in &result.counterfactual_pairs {
            assert!(
                !pair.pair_id.is_empty(),
                "Each pair should have a non-empty pair_id"
            );
            assert!(
                !pair.original.lines.is_empty(),
                "Original JE should have line items"
            );
            assert!(
                !pair.modified.lines.is_empty(),
                "Modified JE should have line items"
            );
        }
    }

    // ========================================================================
    // 4. Counterfactuals empty when disabled
    // ========================================================================

    #[test]
    fn test_counterfactuals_empty_when_disabled() {
        let config = minimal_config();

        // Default PhaseConfig has generate_counterfactuals: false
        let phase_config = PhaseConfig {
            generate_journal_entries: true,
            generate_counterfactuals: false,
            show_progress: false,
            ..base_phase_config()
        };

        let mut orchestrator =
            EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");
        let result = orchestrator.generate().expect("Generation failed");

        assert!(
            result.counterfactual_pairs.is_empty(),
            "counterfactual_pairs should be empty when generation is disabled"
        );
    }

    // ========================================================================
    // 5. Industry output generated for manufacturing
    // ========================================================================

    #[test]
    fn test_industry_output_generated() {
        let mut config = minimal_config();
        // Enable the industry-specific config gate
        config.industry_specific.enabled = true;

        let phase_config = PhaseConfig {
            generate_journal_entries: true,
            show_progress: false,
            ..base_phase_config()
        };

        let mut orchestrator =
            EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");
        let result = orchestrator.generate().expect("Generation failed");

        assert!(
            result.industry_output.is_some(),
            "industry_output should be Some for Manufacturing industry"
        );
        let industry_output = result.industry_output.unwrap();
        assert_eq!(
            industry_output.industry, "Manufacturing",
            "industry should be 'Manufacturing'"
        );
        assert!(
            result.statistics.industry_gl_account_count > 0,
            "industry_gl_account_count should be > 0"
        );
    }

    // ========================================================================
    // 6. Temporal vendor chains with master data
    // ========================================================================

    #[test]
    fn test_temporal_vendor_chains_with_master_data() {
        let mut config = minimal_config();
        // Enable the temporal attributes config gate
        config.temporal_attributes.enabled = true;

        let phase_config = PhaseConfig {
            generate_master_data: true,
            generate_journal_entries: true,
            vendors_per_company: 5,
            show_progress: false,
            ..base_phase_config()
        };

        let mut orchestrator =
            EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");
        let result = orchestrator.generate().expect("Generation failed");

        assert!(
            !result.temporal_vendor_chains.is_empty(),
            "temporal_vendor_chains should not be empty when temporal attributes are enabled and vendors exist"
        );
        assert!(
            result.statistics.temporal_version_chain_count > 0,
            "temporal_version_chain_count should be > 0"
        );
    }

    // ========================================================================
    // 7. Entity graph with master data
    // ========================================================================

    #[test]
    fn test_entity_graph_with_master_data() {
        let mut config = minimal_config();
        // Enable the relationship strength config gate
        config.relationship_strength.enabled = true;

        let phase_config = PhaseConfig {
            generate_master_data: true,
            generate_journal_entries: true,
            vendors_per_company: 5,
            customers_per_company: 5,
            show_progress: false,
            ..base_phase_config()
        };

        let mut orchestrator =
            EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");
        let result = orchestrator.generate().expect("Generation failed");

        assert!(
            result.entity_relationship_graph.is_some(),
            "entity_relationship_graph should be Some when relationship_strength is enabled"
        );
        assert!(
            result.statistics.entity_relationship_node_count > 0,
            "entity_relationship_node_count should be > 0"
        );
    }

    // ========================================================================
    // 8. Red flags with fraud enabled
    // ========================================================================

    #[test]
    fn test_red_flags_with_fraud_enabled() {
        let config = fraud_enabled_config();
        // fraud_enabled_config() already sets fraud.enabled = true and fraud_rate = 0.1

        let phase_config = PhaseConfig {
            generate_master_data: true,
            generate_document_flows: true,
            generate_journal_entries: true,
            inject_anomalies: true,
            p2p_chains: 10,
            o2c_chains: 10,
            vendors_per_company: 5,
            customers_per_company: 5,
            materials_per_company: 5,
            show_progress: false,
            ..base_phase_config()
        };

        let mut orchestrator =
            EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");
        let result = orchestrator.generate().expect("Generation failed");

        assert!(
            !result.red_flags.is_empty(),
            "red_flags should not be empty when fraud is enabled with P2P/O2C chains"
        );
        assert!(
            result.statistics.red_flag_count > 0,
            "red_flag_count should be > 0"
        );
    }

    // ========================================================================
    // 9. Collusion rings with employees and vendors
    // ========================================================================

    #[test]
    fn test_collusion_rings_with_employees_vendors() {
        let mut config = minimal_config();
        config.fraud.enabled = true;
        config.fraud.clustering_enabled = true;

        let phase_config = PhaseConfig {
            generate_master_data: true,
            generate_journal_entries: true,
            employees_per_company: 10,
            vendors_per_company: 10,
            show_progress: false,
            ..base_phase_config()
        };

        let mut orchestrator =
            EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");
        let result = orchestrator.generate().expect("Generation failed");

        assert!(
            !result.collusion_rings.is_empty(),
            "collusion_rings should not be empty when fraud clustering is enabled with enough employees/vendors"
        );
        assert!(
            result.statistics.collusion_ring_count > 0,
            "collusion_ring_count should be > 0"
        );
    }

    // ========================================================================
    // 10. Disruption events generated
    // ========================================================================

    #[test]
    fn test_disruption_events_generated() {
        let mut config = minimal_config();
        config.global.period_months = 6;
        // Disruption events are gated on organizational_events.enabled
        config.organizational_events.enabled = true;

        let phase_config = PhaseConfig {
            generate_journal_entries: true,
            generate_evolution_events: true,
            show_progress: false,
            ..base_phase_config()
        };

        let mut orchestrator =
            EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");
        let result = orchestrator.generate().expect("Generation failed");

        assert!(
            !result.disruption_events.is_empty(),
            "disruption_events should not be empty when organizational events are enabled"
        );
        assert!(
            result.statistics.disruption_event_count > 0,
            "disruption_event_count should be > 0"
        );
    }

    // ========================================================================
    // 11. Deterministic output across repeated runs
    // ========================================================================

    #[test]
    fn test_deterministic_output() {
        let make_config = || {
            let mut config = minimal_config();
            config.global.seed = Some(42);
            config.global.period_months = 3;
            config.organizational_events.enabled = true;
            config
        };

        let phase_config = || PhaseConfig {
            generate_journal_entries: true,
            generate_evolution_events: true,
            generate_counterfactuals: true,
            show_progress: false,
            ..base_phase_config()
        };

        // Run 1
        let mut orch1 =
            EnhancedOrchestrator::new(make_config(), phase_config()).expect("orchestrator 1");
        let r1 = orch1.generate().expect("generation 1");

        // Run 2 (same seed, same config)
        let mut orch2 =
            EnhancedOrchestrator::new(make_config(), phase_config()).expect("orchestrator 2");
        let r2 = orch2.generate().expect("generation 2");

        // Process evolution events should be identical
        assert_eq!(
            r1.process_evolution.len(),
            r2.process_evolution.len(),
            "process_evolution count must be deterministic"
        );
        for (a, b) in r1.process_evolution.iter().zip(r2.process_evolution.iter()) {
            assert_eq!(
                a.effective_date, b.effective_date,
                "process evolution effective dates must match"
            );
        }

        // Organizational events should be identical
        assert_eq!(
            r1.organizational_events.len(),
            r2.organizational_events.len(),
            "organizational_events count must be deterministic"
        );
        for (a, b) in r1
            .organizational_events
            .iter()
            .zip(r2.organizational_events.iter())
        {
            assert_eq!(
                a.effective_date, b.effective_date,
                "organizational event effective dates must match"
            );
        }

        // Disruption events should be identical
        assert_eq!(
            r1.disruption_events.len(),
            r2.disruption_events.len(),
            "disruption_events count must be deterministic"
        );

        // Counterfactual pairs should be identical
        assert_eq!(
            r1.counterfactual_pairs.len(),
            r2.counterfactual_pairs.len(),
            "counterfactual_pairs count must be deterministic"
        );
        for (a, b) in r1
            .counterfactual_pairs
            .iter()
            .zip(r2.counterfactual_pairs.iter())
        {
            assert_eq!(
                a.pair_id, b.pair_id,
                "counterfactual pair_id must be deterministic"
            );
        }
    }
}
