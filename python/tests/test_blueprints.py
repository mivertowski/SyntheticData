"""Tests for DataSynth blueprint functions."""

import unittest

from datasynth_py.config import blueprints


# Blueprints that can be called with no arguments (standalone configs)
_STANDALONE_BLUEPRINTS = {
    "retail_small", "banking_medium", "manufacturing_large",
    "banking_aml", "ml_training", "audit_engagement",
    "statistical_validation",
}

# Blueprints that require a base_config argument (composable modifiers)
_COMPOSABLE_BLUEPRINTS = {
    "with_graph_export", "with_distributions", "with_regime_changes",
    "with_templates", "with_temporal_patterns", "with_sourcing",
    "with_financial_reporting", "with_hr", "with_manufacturing",
    "with_sales_quotes", "with_process_mining",
    "with_llm_enrichment", "with_diffusion", "with_causal",
    "with_fraud_packs", "with_scenarios", "with_streaming",
}


class TestBlueprintRegistry(unittest.TestCase):
    """Test the blueprint registry."""

    def test_registry_contains_standalone_blueprints(self):
        available = set(blueprints.list())
        for name in _STANDALONE_BLUEPRINTS:
            self.assertIn(name, available, f"Missing standalone blueprint: {name}")

    def test_registry_contains_composable_blueprints(self):
        available = set(blueprints.list())
        for name in _COMPOSABLE_BLUEPRINTS:
            if name in ("with_graph_export", "with_distributions", "with_regime_changes",
                        "with_templates", "with_temporal_patterns"):
                # These are not in the registry (called directly), skip
                continue
            self.assertIn(name, available, f"Missing composable blueprint: {name}")

    def test_get_returns_callable(self):
        for name in blueprints.list():
            factory = blueprints.get(name)
            self.assertTrue(callable(factory))

    def test_get_unknown_raises(self):
        with self.assertRaises(KeyError):
            blueprints.get("nonexistent_blueprint")


class TestStandaloneBlueprints(unittest.TestCase):
    """Test standalone blueprints that produce complete configs."""

    def test_standalone_blueprints_validate(self):
        for name in _STANDALONE_BLUEPRINTS:
            with self.subTest(blueprint=name):
                factory = blueprints.get(name)
                config = factory()
                config.validate()  # Should not raise

    def test_retail_small(self):
        config = blueprints.retail_small(companies=4, transactions=12000)
        payload = config.to_dict()
        self.assertEqual(payload["global"]["industry"], "retail")
        self.assertEqual(payload["global"]["start_date"], "2024-01-01")
        self.assertEqual(payload["global"]["period_months"], 12)
        self.assertEqual(len(payload["companies"]), 4)
        self.assertEqual(payload["companies"][0]["code"], "R001")
        self.assertEqual(payload["chart_of_accounts"]["complexity"], "small")

    def test_retail_small_with_templates(self):
        config = blueprints.retail_small(realistic_names=True)
        payload = config.to_dict()
        self.assertIn("templates", payload)
        self.assertTrue(payload["templates"]["names"]["generate_realistic_names"])

    def test_banking_medium(self):
        config = blueprints.banking_medium(companies=3)
        payload = config.to_dict()
        self.assertEqual(payload["global"]["industry"], "financial_services")
        self.assertEqual(len(payload["companies"]), 3)
        self.assertEqual(payload["chart_of_accounts"]["complexity"], "medium")
        self.assertTrue(payload["fraud"]["enabled"])
        self.assertEqual(payload["fraud"]["rate"], 0.01)

    def test_manufacturing_large(self):
        config = blueprints.manufacturing_large(companies=2)
        payload = config.to_dict()
        self.assertEqual(payload["global"]["industry"], "manufacturing")
        self.assertEqual(payload["chart_of_accounts"]["complexity"], "large")
        self.assertTrue(payload["manufacturing"]["enabled"])

    def test_banking_aml(self):
        config = blueprints.banking_aml(customers=500, typologies=True)
        payload = config.to_dict()
        self.assertTrue(payload["banking"]["enabled"])
        self.assertTrue(payload["banking"]["typologies_enabled"])
        self.assertEqual(payload["banking"]["retail_customers"], 350)  # 70% of 500
        self.assertIn("aml", payload["scenario"]["tags"])
        self.assertTrue(payload["scenario"]["ml_training"])

    def test_ml_training(self):
        config = blueprints.ml_training(
            industry="retail", anomaly_ratio=0.08, with_data_quality=True,
        )
        payload = config.to_dict()
        self.assertTrue(payload["fraud"]["enabled"])
        self.assertEqual(payload["fraud"]["rate"], 0.08)
        self.assertTrue(payload["data_quality"]["enabled"])
        self.assertTrue(payload["graph_export"]["enabled"])
        self.assertTrue(payload["distributions"]["enabled"])
        self.assertTrue(payload["scenario"]["ml_training"])

    def test_ml_training_without_distributions(self):
        config = blueprints.ml_training(with_distributions=False)
        payload = config.to_dict()
        self.assertNotIn("distributions", payload)

    def test_audit_engagement(self):
        config = blueprints.audit_engagement(engagements=10, with_evidence=True)
        payload = config.to_dict()
        self.assertTrue(payload["audit"]["enabled"])
        self.assertEqual(payload["audit"]["engagements"], 10)
        self.assertEqual(payload["audit"]["evidence_per_workpaper"], 5)

    def test_audit_engagement_without_evidence(self):
        config = blueprints.audit_engagement(with_evidence=False)
        payload = config.to_dict()
        self.assertEqual(payload["audit"]["evidence_per_workpaper"], 0)

    def test_statistical_validation(self):
        config = blueprints.statistical_validation(industry="retail", transactions=50000)
        payload = config.to_dict()
        self.assertTrue(payload["distributions"]["enabled"])
        self.assertTrue(payload["distributions"]["correlations"]["enabled"])
        self.assertTrue(len(payload["distributions"]["validation"]["tests"]) >= 3)


class TestComposableBlueprints(unittest.TestCase):
    """Test composable blueprints that modify a base config."""

    def _base(self):
        return blueprints.retail_small(companies=1)

    def test_with_process_mining(self):
        config = blueprints.with_process_mining(self._base())
        payload = config.to_dict()
        self.assertIn("ocpm", payload)
        self.assertTrue(payload["ocpm"]["enabled"])
        self.assertTrue(payload["ocpm"]["generate_lifecycle_events"])
        self.assertTrue(payload["ocpm"]["compute_variants"])
        self.assertIn("graph_export", payload)
        self.assertTrue(payload["graph_export"]["enabled"])
        self.assertTrue(payload["graph_export"]["hypergraph"]["enabled"])
        pl = payload["graph_export"]["hypergraph"]["process_layer"]
        self.assertTrue(pl["include_p2p"])
        self.assertTrue(pl["include_s2c"])
        self.assertTrue(pl["include_h2r"])
        self.assertTrue(pl["events_as_hyperedges"])

    def test_with_process_mining_no_hyperedges(self):
        config = blueprints.with_process_mining(self._base(), events_as_hyperedges=False)
        payload = config.to_dict()
        self.assertFalse(
            payload["graph_export"]["hypergraph"]["process_layer"]["events_as_hyperedges"],
        )

    def test_with_process_mining_preserves_graph_formats(self):
        from datasynth_py.config.models import GraphExportSettings
        base = blueprints.retail_small()
        base = base.override(graph_export={"enabled": True, "formats": ["neo4j", "dgl"]})
        config = blueprints.with_process_mining(base)
        payload = config.to_dict()
        self.assertIn("neo4j", payload["graph_export"]["formats"])
        self.assertIn("dgl", payload["graph_export"]["formats"])

    def test_with_graph_export(self):
        config = blueprints.with_graph_export(self._base(), formats=["neo4j", "dgl"])
        payload = config.to_dict()
        self.assertTrue(payload["graph_export"]["enabled"])
        self.assertIn("neo4j", payload["graph_export"]["formats"])

    def test_with_sourcing(self):
        config = blueprints.with_sourcing(self._base(), projects_per_year=20)
        payload = config.to_dict()
        self.assertTrue(payload["source_to_pay"]["enabled"])
        self.assertEqual(payload["source_to_pay"]["sourcing"]["projects_per_year"], 20)

    def test_with_financial_reporting(self):
        config = blueprints.with_financial_reporting(
            self._base(), with_kpis=True, with_budgets=True,
        )
        payload = config.to_dict()
        self.assertTrue(payload["financial_reporting"]["enabled"])
        self.assertTrue(payload["financial_reporting"]["management_kpis"]["enabled"])
        self.assertTrue(payload["financial_reporting"]["budgets"]["enabled"])

    def test_with_hr(self):
        config = blueprints.with_hr(self._base())
        payload = config.to_dict()
        self.assertTrue(payload["hr"]["enabled"])
        self.assertTrue(payload["hr"]["payroll"]["enabled"])
        self.assertTrue(payload["hr"]["time_attendance"]["enabled"])
        self.assertTrue(payload["hr"]["expenses"]["enabled"])

    def test_with_hr_selective(self):
        config = blueprints.with_hr(
            self._base(), with_payroll=True, with_time_tracking=False, with_expenses=False,
        )
        payload = config.to_dict()
        self.assertTrue(payload["hr"]["enabled"])
        self.assertIn("payroll", payload["hr"])
        self.assertNotIn("time_attendance", payload["hr"])
        self.assertNotIn("expenses", payload["hr"])

    def test_with_manufacturing(self):
        config = blueprints.with_manufacturing(self._base(), orders_per_month=100)
        payload = config.to_dict()
        self.assertTrue(payload["manufacturing"]["enabled"])
        self.assertEqual(
            payload["manufacturing"]["production_orders"]["orders_per_month"], 100,
        )

    def test_with_sales_quotes(self):
        config = blueprints.with_sales_quotes(
            self._base(), quotes_per_month=50, win_rate=0.45,
        )
        payload = config.to_dict()
        self.assertTrue(payload["sales_quotes"]["enabled"])
        self.assertEqual(payload["sales_quotes"]["quotes_per_month"], 50)
        self.assertAlmostEqual(payload["sales_quotes"]["win_rate"], 0.45)

    def test_with_distributions(self):
        config = blueprints.with_distributions(
            self._base(), industry_profile="retail", with_correlations=True,
        )
        payload = config.to_dict()
        self.assertTrue(payload["distributions"]["enabled"])
        self.assertEqual(payload["distributions"]["industry_profile"], "retail")
        self.assertTrue(payload["distributions"]["correlations"]["enabled"])

    def test_with_temporal_patterns(self):
        config = blueprints.with_temporal_patterns(
            self._base(),
            regions=["US", "DE"],
            with_business_days=True,
            with_period_end_curves=True,
            with_processing_lags=True,
            with_intraday_patterns=True,
            with_timezones=True,
        )
        payload = config.to_dict()
        tp = payload["temporal_patterns"]
        self.assertTrue(tp["enabled"])
        self.assertIn("US", tp["calendars"]["regions"])
        self.assertTrue(tp["business_days"]["enabled"])
        self.assertTrue(tp["period_end"]["enabled"])
        self.assertTrue(tp["processing_lags"]["enabled"])
        self.assertTrue(tp["intraday"]["enabled"])
        self.assertTrue(tp["timezones"]["enabled"])

    def test_with_templates(self):
        config = blueprints.with_templates(
            self._base(), email_domain="test.com", invoice_prefix="TEST",
        )
        payload = config.to_dict()
        self.assertIn("templates", payload)
        self.assertEqual(payload["templates"]["names"]["email_domain"], "test.com")
        self.assertEqual(payload["templates"]["references"]["invoice_prefix"], "TEST")

    def test_with_regime_changes(self):
        config = blueprints.with_regime_changes(self._base(), with_economic_cycle=True)
        payload = config.to_dict()
        self.assertTrue(payload["distributions"]["enabled"])
        self.assertTrue(payload["distributions"]["regime_changes"]["enabled"])
        self.assertTrue(payload["distributions"]["regime_changes"]["economic_cycle"]["enabled"])

    def test_with_llm_enrichment(self):
        config = blueprints.with_llm_enrichment(provider="mock")
        payload = config.to_dict()
        self.assertTrue(payload["llm"]["enabled"])
        self.assertEqual(payload["llm"]["provider"], "mock")

    def test_with_diffusion(self):
        config = blueprints.with_diffusion(n_steps=500, schedule="linear")
        payload = config.to_dict()
        self.assertTrue(payload["diffusion"]["enabled"])
        self.assertEqual(payload["diffusion"]["n_steps"], 500)
        self.assertEqual(payload["diffusion"]["noise_schedule"], "linear")

    def test_with_causal(self):
        config = blueprints.with_causal(template="supply_chain")
        payload = config.to_dict()
        self.assertTrue(payload["causal"]["enabled"])
        self.assertEqual(payload["causal"]["template"], "supply_chain")


class TestBlueprintComposition(unittest.TestCase):
    """Test chaining multiple composable blueprints."""

    def test_chain_multiple_modifiers(self):
        config = blueprints.retail_small(companies=2)
        config = blueprints.with_sourcing(config)
        config = blueprints.with_hr(config)
        config = blueprints.with_manufacturing(config)
        config = blueprints.with_process_mining(config)

        payload = config.to_dict()
        self.assertTrue(payload["source_to_pay"]["enabled"])
        self.assertTrue(payload["hr"]["enabled"])
        self.assertTrue(payload["manufacturing"]["enabled"])
        self.assertTrue(payload["ocpm"]["enabled"])
        self.assertTrue(payload["graph_export"]["hypergraph"]["enabled"])

    def test_chain_preserves_base_settings(self):
        config = blueprints.manufacturing_large(companies=3)
        config = blueprints.with_process_mining(config)
        config = blueprints.with_financial_reporting(config)

        payload = config.to_dict()
        self.assertEqual(payload["global"]["industry"], "manufacturing")
        self.assertEqual(len(payload["companies"]), 3)
        self.assertEqual(payload["chart_of_accounts"]["complexity"], "large")
        self.assertTrue(payload["ocpm"]["enabled"])
        self.assertTrue(payload["financial_reporting"]["enabled"])

    def test_full_enterprise_blueprint_chain(self):
        """Build a full enterprise config through blueprint composition."""
        config = blueprints.manufacturing_large(companies=5)
        config = blueprints.with_sourcing(config, projects_per_year=15)
        config = blueprints.with_hr(config)
        config = blueprints.with_manufacturing(config, orders_per_month=200)
        config = blueprints.with_financial_reporting(config)
        config = blueprints.with_process_mining(config)
        config = blueprints.with_distributions(config, industry_profile="manufacturing")
        config = blueprints.with_temporal_patterns(config, regions=["US", "DE"])

        payload = config.to_dict()

        # All sections should be present
        for key in [
            "global", "companies", "source_to_pay", "hr", "manufacturing",
            "financial_reporting", "ocpm", "graph_export", "distributions",
            "temporal_patterns",
        ]:
            self.assertIn(key, payload, f"Missing section: {key}")

        # Validate the composed config
        from datasynth_py.config.models import Config
        restored = Config.from_dict(payload)
        restored.validate()


class TestVolumeMapping(unittest.TestCase):
    """Test the transaction count to volume preset mapping."""

    def test_volume_presets(self):
        self.assertEqual(
            blueprints.retail_small(transactions=5000).to_dict()
            ["companies"][0]["annual_transaction_volume"], "ten_k",
        )
        self.assertEqual(
            blueprints.retail_small(transactions=50000).to_dict()
            ["companies"][0]["annual_transaction_volume"], "hundred_k",
        )
        self.assertEqual(
            blueprints.retail_small(transactions=500000).to_dict()
            ["companies"][0]["annual_transaction_volume"], "one_m",
        )
        self.assertEqual(
            blueprints.retail_small(transactions=5000000).to_dict()
            ["companies"][0]["annual_transaction_volume"], "ten_m",
        )
        self.assertEqual(
            blueprints.retail_small(transactions=50000000).to_dict()
            ["companies"][0]["annual_transaction_volume"], "hundred_m",
        )


class TestFraudPacksBlueprint(unittest.TestCase):
    """Test with_fraud_packs composable blueprint."""

    def _base(self):
        return blueprints.retail_small(companies=1)

    def test_with_fraud_packs_default(self):
        config = blueprints.with_fraud_packs(self._base())
        payload = config.to_dict()
        self.assertTrue(payload["fraud"]["enabled"])
        self.assertEqual(payload["fraud"]["fraud_packs"], ["comprehensive"])

    def test_with_fraud_packs_specific(self):
        config = blueprints.with_fraud_packs(
            self._base(), packs=["revenue_fraud", "payroll_ghost"],
        )
        payload = config.to_dict()
        self.assertTrue(payload["fraud"]["enabled"])
        self.assertEqual(
            payload["fraud"]["fraud_packs"],
            ["revenue_fraud", "payroll_ghost"],
        )

    def test_with_fraud_packs_unknown_raises(self):
        with self.assertRaises(ValueError) as ctx:
            blueprints.with_fraud_packs(self._base(), packs=["nonexistent_pack"])
        self.assertIn("nonexistent_pack", str(ctx.exception))
        self.assertIn("Valid packs", str(ctx.exception))

    def test_with_fraud_packs_with_rate(self):
        config = blueprints.with_fraud_packs(self._base(), fraud_rate=0.15)
        payload = config.to_dict()
        self.assertTrue(payload["fraud"]["enabled"])
        self.assertAlmostEqual(payload["fraud"]["rate"], 0.15)
        self.assertEqual(payload["fraud"]["fraud_packs"], ["comprehensive"])

    def test_fraud_packs_constant(self):
        self.assertEqual(len(blueprints.FRAUD_PACKS), 5)
        self.assertIn("revenue_fraud", blueprints.FRAUD_PACKS)
        self.assertIn("payroll_ghost", blueprints.FRAUD_PACKS)
        self.assertIn("vendor_kickback", blueprints.FRAUD_PACKS)
        self.assertIn("management_override", blueprints.FRAUD_PACKS)
        self.assertIn("comprehensive", blueprints.FRAUD_PACKS)


class TestScenariosBlueprint(unittest.TestCase):
    """Test with_scenarios composable blueprint."""

    def _base(self):
        return blueprints.retail_small(companies=1)

    def test_with_scenarios_default(self):
        config = blueprints.with_scenarios(self._base())
        payload = config.to_dict()
        self.assertTrue(payload["causal"]["enabled"])
        self.assertEqual(payload["causal"]["template"], "fraud_detection")
        self.assertTrue(payload["causal"]["interventions"]["enabled"])
        self.assertTrue(payload["causal"]["counterfactuals"]["enabled"])
        self.assertEqual(payload["causal"]["counterfactuals"]["samples_per_record"], 5)

    def test_with_scenarios_custom(self):
        config = blueprints.with_scenarios(
            self._base(), template="supply_chain", with_interventions=False,
        )
        payload = config.to_dict()
        self.assertTrue(payload["causal"]["enabled"])
        self.assertEqual(payload["causal"]["template"], "supply_chain")
        self.assertFalse(payload["causal"]["interventions"]["enabled"])
        self.assertTrue(payload["causal"]["counterfactuals"]["enabled"])


class TestStreamingBlueprint(unittest.TestCase):
    """Test with_streaming composable blueprint."""

    def _base(self):
        return blueprints.retail_small(companies=1)

    def test_with_streaming_default(self):
        config = blueprints.with_streaming(self._base())
        payload = config.to_dict()
        self.assertTrue(payload["streaming"]["enabled"])
        self.assertEqual(payload["streaming"]["buffer_size"], 1000)
        self.assertEqual(payload["streaming"]["backpressure"], "block")

    def test_with_streaming_custom(self):
        config = blueprints.with_streaming(self._base(), buffer_size=5000)
        payload = config.to_dict()
        self.assertTrue(payload["streaming"]["enabled"])
        self.assertEqual(payload["streaming"]["buffer_size"], 5000)
        self.assertEqual(payload["streaming"]["backpressure"], "block")

    def test_with_streaming_invalid_backpressure(self):
        with self.assertRaises(ValueError) as ctx:
            blueprints.with_streaming(self._base(), backpressure="explode")
        self.assertIn("explode", str(ctx.exception))
        self.assertIn("Valid strategies", str(ctx.exception))

    def test_with_streaming_all_valid_backpressure_strategies(self):
        for strategy in ["block", "drop_oldest", "drop_newest", "buffer"]:
            with self.subTest(strategy=strategy):
                config = blueprints.with_streaming(self._base(), backpressure=strategy)
                payload = config.to_dict()
                self.assertEqual(payload["streaming"]["backpressure"], strategy)


class TestNewBlueprintComposition(unittest.TestCase):
    """Test composing the new blueprints with each other and existing ones."""

    def test_composition_fraud_streaming(self):
        config = blueprints.retail_small(companies=2)
        config = blueprints.with_fraud_packs(
            config, packs=["vendor_kickback", "revenue_fraud"], fraud_rate=0.08,
        )
        config = blueprints.with_streaming(config, buffer_size=2000)

        payload = config.to_dict()
        # Fraud settings should be present
        self.assertTrue(payload["fraud"]["enabled"])
        self.assertEqual(
            payload["fraud"]["fraud_packs"],
            ["vendor_kickback", "revenue_fraud"],
        )
        self.assertAlmostEqual(payload["fraud"]["rate"], 0.08)
        # Streaming settings should be present
        self.assertTrue(payload["streaming"]["enabled"])
        self.assertEqual(payload["streaming"]["buffer_size"], 2000)
        # Base config should be preserved
        self.assertEqual(payload["global"]["industry"], "retail")
        self.assertEqual(len(payload["companies"]), 2)

    def test_composition_scenarios_fraud_streaming(self):
        config = blueprints.manufacturing_large(companies=3)
        config = blueprints.with_fraud_packs(config, packs=["comprehensive"])
        config = blueprints.with_scenarios(config, template="revenue_impact")
        config = blueprints.with_streaming(config, buffer_size=500, backpressure="drop_oldest")

        payload = config.to_dict()
        self.assertTrue(payload["fraud"]["enabled"])
        self.assertTrue(payload["causal"]["enabled"])
        self.assertEqual(payload["causal"]["template"], "revenue_impact")
        self.assertTrue(payload["streaming"]["enabled"])
        self.assertEqual(payload["streaming"]["backpressure"], "drop_oldest")
        # Base preserved
        self.assertEqual(payload["global"]["industry"], "manufacturing")
        self.assertEqual(len(payload["companies"]), 3)


if __name__ == "__main__":
    unittest.main()
