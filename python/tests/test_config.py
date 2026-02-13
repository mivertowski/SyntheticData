"""Tests for DataSynth configuration models."""

import json
import unittest

from datasynth_py.config import (
    ChartOfAccountsSettings,
    CompanyConfig,
    Config,
    FraudSettings,
    GlobalSettings,
    GraphExportSettings,
    HypergraphSettings,
    OcpmOutputSettings,
    OcpmProcessSettings,
    OcpmSettings,
    ProcessLayerSettings,
)
from datasynth_py.config.models import (
    AccountingStandardsConfig,
    AdvancedDistributionSettings,
    AuditSettings,
    AuditStandardsConfig,
    BankingSettings,
    CrossProcessLinksConfig,
    CustomerSegmentationConfig,
    DataQualitySettings,
    ExpenseSchemaConfig,
    FairValueConfig,
    FinancialReportingConfig,
    HrConfig,
    ImpairmentConfig,
    IsaComplianceConfig,
    LeaseAccountingConfig,
    ManufacturingProcessConfig,
    MixtureComponentConfig,
    MixtureDistributionConfig,
    OutputSettings,
    PayrollSchemaConfig,
    ProductionOrderSchemaConfig,
    RelationshipStrengthConfig,
    RevenueRecognitionConfig,
    SalesQuoteSchemaConfig,
    ScenarioSettings,
    SourceToPayConfig,
    SoxComplianceConfig,
    TemporalDriftSettings,
    VendorNetworkConfig,
)
from datasynth_py.config.validation import ConfigValidationError


def _make_base_config(**kwargs):
    """Helper to create a minimal valid Config."""
    defaults = dict(
        global_settings=GlobalSettings(
            industry="retail", start_date="2024-01-01", period_months=12,
        ),
        companies=[CompanyConfig(code="C001", name="Test Co")],
        chart_of_accounts=ChartOfAccountsSettings(complexity="small"),
    )
    defaults.update(kwargs)
    return Config(**defaults)


class TestConfigBasics(unittest.TestCase):
    """Test basic Config construction, serialization, and validation."""

    def test_to_dict_matches_cli_schema(self):
        config = _make_base_config(
            global_settings=GlobalSettings(
                industry="manufacturing", start_date="2024-01-01",
                period_months=12, seed=42,
            ),
            companies=[
                CompanyConfig(
                    code="M001", name="Manufacturing Co",
                    currency="USD", country="US",
                    annual_transaction_volume="ten_k",
                ),
            ],
        )
        payload = config.to_dict()

        self.assertEqual(payload["global"]["industry"], "manufacturing")
        self.assertEqual(payload["global"]["seed"], 42)
        self.assertIsInstance(payload["companies"], list)
        self.assertEqual(payload["companies"][0]["code"], "M001")
        self.assertEqual(payload["chart_of_accounts"]["complexity"], "small")

    def test_valid_config_passes_validation(self):
        config = _make_base_config()
        config.validate()  # Should not raise

    def test_validate_reports_schema_errors(self):
        config = Config(
            global_settings=GlobalSettings(period_months=0, industry="invalid"),
            companies=[],
            fraud=FraudSettings(rate=2.0),
        )
        with self.assertRaises(ConfigValidationError) as ctx:
            config.validate()
        messages = [e.message for e in ctx.exception.errors]
        self.assertTrue(any("period_months" in m or "1 and 120" in m for m in messages))
        self.assertTrue(any("company" in m.lower() for m in messages))
        self.assertTrue(any("0 and 1" in m for m in messages))

    def test_override_merges_nested_sections(self):
        base = _make_base_config(extra={"fraud": {"enabled": False}})
        updated = base.override(fraud={"enabled": True, "rate": 0.05})
        payload = updated.to_dict()
        self.assertTrue(payload["fraud"]["enabled"])
        self.assertEqual(payload["fraud"]["rate"], 0.05)

    def test_override_preserves_existing_fields(self):
        base = _make_base_config(
            fraud=FraudSettings(enabled=True, rate=0.01),
        )
        updated = base.override(
            data_quality={"enabled": True, "missing_rate": 0.1},
        )
        payload = updated.to_dict()
        self.assertTrue(payload["fraud"]["enabled"])
        self.assertTrue(payload["data_quality"]["enabled"])

    def test_to_json(self):
        config = _make_base_config()
        result = config.to_json(indent=2)
        parsed = json.loads(result)
        self.assertEqual(parsed["global"]["industry"], "retail")

    def test_to_yaml(self):
        config = _make_base_config()
        result = config.to_yaml()
        self.assertIn("industry: retail", result)
        self.assertIn("start_date:", result)

    def test_extra_fields_passthrough(self):
        data = {
            "global": {"industry": "retail", "start_date": "2024-01-01", "period_months": 6},
            "companies": [{"code": "C001", "name": "Test"}],
            "custom_plugin": {"foo": "bar", "count": 42},
        }
        config = Config.from_dict(data)
        self.assertEqual(config.extra["custom_plugin"]["foo"], "bar")
        payload = config.to_dict()
        self.assertEqual(payload["custom_plugin"]["count"], 42)


class TestConfigRoundTrip(unittest.TestCase):
    """Test from_dict -> to_dict round-trip for all config sections."""

    def _assert_round_trip(self, data):
        """Assert that from_dict(data).to_dict() preserves key fields."""
        config = Config.from_dict(data)
        result = config.to_dict()
        # Check that all non-extra keys from input appear in output
        for key in data:
            if key in ("global",):
                # 'global' key is always present if global_settings is set
                self.assertIn(key, result, f"Missing key: {key}")
            elif key in ("companies", "chart_of_accounts"):
                self.assertIn(key, result, f"Missing key: {key}")
        return config, result

    def test_round_trip_minimal(self):
        data = {
            "global": {"industry": "retail", "start_date": "2024-01-01", "period_months": 12},
            "companies": [{"code": "C001", "name": "Test"}],
            "chart_of_accounts": {"complexity": "small"},
        }
        _, result = self._assert_round_trip(data)
        self.assertEqual(result["global"]["industry"], "retail")

    def test_round_trip_fraud(self):
        data = {
            "global": {"industry": "retail"},
            "companies": [{"code": "C001", "name": "T"}],
            "fraud": {"enabled": True, "rate": 0.05},
        }
        config, result = self._assert_round_trip(data)
        self.assertTrue(result["fraud"]["enabled"])
        self.assertEqual(result["fraud"]["rate"], 0.05)

    def test_round_trip_banking(self):
        data = {
            "global": {"industry": "financial_services"},
            "companies": [{"code": "C001", "name": "T"}],
            "banking": {
                "enabled": True, "retail_customers": 500,
                "business_customers": 100, "typologies_enabled": True,
            },
        }
        config, result = self._assert_round_trip(data)
        self.assertTrue(result["banking"]["enabled"])
        self.assertEqual(result["banking"]["retail_customers"], 500)

    def test_round_trip_data_quality(self):
        data = {
            "global": {"industry": "retail"},
            "companies": [{"code": "C001", "name": "T"}],
            "data_quality": {
                "enabled": True, "missing_rate": 0.1,
                "typo_rate": 0.03, "duplicate_rate": 0.02,
            },
        }
        config, result = self._assert_round_trip(data)
        self.assertTrue(result["data_quality"]["enabled"])
        self.assertAlmostEqual(result["data_quality"]["missing_rate"], 0.1)

    def test_round_trip_source_to_pay(self):
        data = {
            "global": {"industry": "manufacturing"},
            "companies": [{"code": "C001", "name": "T"}],
            "source_to_pay": {
                "enabled": True,
                "sourcing": {"projects_per_year": 10},
                "contracts": {"max_duration_months": 24},
            },
        }
        config, result = self._assert_round_trip(data)
        self.assertTrue(result["source_to_pay"]["enabled"])
        self.assertEqual(result["source_to_pay"]["sourcing"]["projects_per_year"], 10)

    def test_round_trip_hr(self):
        data = {
            "global": {"industry": "retail"},
            "companies": [{"code": "C001", "name": "T"}],
            "hr": {
                "enabled": True,
                "payroll": {"enabled": True, "pay_frequency": "biweekly"},
                "time_attendance": {"enabled": True, "overtime_rate": 0.15},
                "expenses": {"enabled": True, "submission_rate": 0.40},
            },
        }
        config, result = self._assert_round_trip(data)
        self.assertTrue(result["hr"]["enabled"])
        self.assertEqual(result["hr"]["payroll"]["pay_frequency"], "biweekly")
        self.assertAlmostEqual(result["hr"]["time_attendance"]["overtime_rate"], 0.15)

    def test_round_trip_manufacturing(self):
        data = {
            "global": {"industry": "manufacturing"},
            "companies": [{"code": "C001", "name": "T"}],
            "manufacturing": {
                "enabled": True,
                "production_orders": {"orders_per_month": 100, "yield_rate": 0.95},
            },
        }
        config, result = self._assert_round_trip(data)
        self.assertTrue(result["manufacturing"]["enabled"])
        self.assertEqual(
            result["manufacturing"]["production_orders"]["orders_per_month"], 100,
        )

    def test_round_trip_financial_reporting(self):
        data = {
            "global": {"industry": "retail"},
            "companies": [{"code": "C001", "name": "T"}],
            "financial_reporting": {
                "enabled": True,
                "generate_balance_sheet": True,
                "management_kpis": {"enabled": True, "frequency": "quarterly"},
                "budgets": {"enabled": True, "revenue_growth_rate": 0.08},
            },
        }
        config, result = self._assert_round_trip(data)
        self.assertTrue(result["financial_reporting"]["enabled"])
        self.assertEqual(
            result["financial_reporting"]["management_kpis"]["frequency"], "quarterly",
        )

    def test_round_trip_vendor_network(self):
        data = {
            "global": {"industry": "manufacturing"},
            "companies": [{"code": "C001", "name": "T"}],
            "vendor_network": {
                "enabled": True, "depth": 3,
                "tiers": {"tier1": {"count_min": 50}},
            },
        }
        config, result = self._assert_round_trip(data)
        self.assertTrue(result["vendor_network"]["enabled"])
        self.assertEqual(result["vendor_network"]["depth"], 3)

    def test_round_trip_cross_process_links(self):
        data = {
            "global": {"industry": "retail"},
            "companies": [{"code": "C001", "name": "T"}],
            "cross_process_links": {
                "enabled": True,
                "inventory_p2p_o2c": True,
                "payment_bank_reconciliation": True,
            },
        }
        config, result = self._assert_round_trip(data)
        self.assertTrue(result["cross_process_links"]["enabled"])
        self.assertTrue(result["cross_process_links"]["inventory_p2p_o2c"])

    def test_round_trip_legacy_companies_format(self):
        """Legacy format uses a dict with count instead of a list."""
        data = {
            "companies": {"count": 3, "industry": "retail", "complexity": "medium"},
        }
        config = Config.from_dict(data)
        self.assertIsNotNone(config.companies)
        self.assertEqual(len(config.companies), 3)
        self.assertEqual(config.companies[0].code, "C001")
        self.assertIsNotNone(config.chart_of_accounts)
        self.assertEqual(config.chart_of_accounts.complexity, "medium")
        self.assertEqual(config.global_settings.industry, "retail")


class TestOcpmConfig(unittest.TestCase):
    """Test OCPM configuration models."""

    def test_ocpm_defaults(self):
        ocpm = OcpmSettings()
        self.assertFalse(ocpm.enabled)
        self.assertTrue(ocpm.generate_lifecycle_events)
        self.assertTrue(ocpm.include_object_relationships)
        self.assertTrue(ocpm.compute_variants)
        self.assertEqual(ocpm.max_variants, 0)
        self.assertIsNone(ocpm.p2p_process)
        self.assertIsNone(ocpm.o2c_process)
        self.assertIsNone(ocpm.output)

    def test_ocpm_process_settings(self):
        proc = OcpmProcessSettings(rework_probability=0.1, skip_step_probability=0.05)
        self.assertAlmostEqual(proc.rework_probability, 0.1)
        self.assertAlmostEqual(proc.skip_step_probability, 0.05)
        self.assertAlmostEqual(proc.out_of_order_probability, 0.03)

    def test_ocpm_output_settings(self):
        out = OcpmOutputSettings(ocel_json=True, xes=True)
        self.assertTrue(out.ocel_json)
        self.assertFalse(out.ocel_xml)
        self.assertTrue(out.xes)

    def test_ocpm_to_dict(self):
        config = _make_base_config(
            ocpm=OcpmSettings(
                enabled=True,
                p2p_process=OcpmProcessSettings(rework_probability=0.1),
                output=OcpmOutputSettings(ocel_json=True, xes=True),
            ),
        )
        payload = config.to_dict()
        self.assertIn("ocpm", payload)
        self.assertTrue(payload["ocpm"]["enabled"])
        self.assertAlmostEqual(payload["ocpm"]["p2p_process"]["rework_probability"], 0.1)
        self.assertTrue(payload["ocpm"]["output"]["xes"])

    def test_ocpm_round_trip(self):
        data = {
            "global": {"industry": "retail"},
            "companies": [{"code": "C001", "name": "T"}],
            "ocpm": {
                "enabled": True,
                "generate_lifecycle_events": True,
                "include_object_relationships": False,
                "compute_variants": True,
                "max_variants": 50,
                "p2p_process": {
                    "rework_probability": 0.08,
                    "skip_step_probability": 0.03,
                    "out_of_order_probability": 0.04,
                },
                "o2c_process": {"rework_probability": 0.06},
                "output": {"ocel_json": True, "ocel_xml": True, "xes": False},
            },
        }
        config = Config.from_dict(data)
        self.assertTrue(config.ocpm.enabled)
        self.assertFalse(config.ocpm.include_object_relationships)
        self.assertEqual(config.ocpm.max_variants, 50)
        self.assertAlmostEqual(config.ocpm.p2p_process.rework_probability, 0.08)
        self.assertAlmostEqual(config.ocpm.o2c_process.rework_probability, 0.06)
        self.assertTrue(config.ocpm.output.ocel_xml)

        # Round-trip
        result = config.to_dict()
        self.assertEqual(result["ocpm"]["max_variants"], 50)
        self.assertAlmostEqual(result["ocpm"]["p2p_process"]["rework_probability"], 0.08)

    def test_ocpm_none_by_default(self):
        config = _make_base_config()
        payload = config.to_dict()
        self.assertNotIn("ocpm", payload)


class TestHypergraphConfig(unittest.TestCase):
    """Test hypergraph and process layer configuration models."""

    def test_process_layer_defaults(self):
        pl = ProcessLayerSettings()
        self.assertTrue(pl.include_p2p)
        self.assertTrue(pl.include_o2c)
        self.assertTrue(pl.include_s2c)
        self.assertTrue(pl.include_h2r)
        self.assertTrue(pl.include_mfg)
        self.assertTrue(pl.include_bank)
        self.assertTrue(pl.include_audit)
        self.assertTrue(pl.include_r2r)
        self.assertTrue(pl.events_as_hyperedges)
        self.assertEqual(pl.docs_per_counterparty_threshold, 20)

    def test_process_layer_selective(self):
        pl = ProcessLayerSettings(
            include_mfg=False, include_bank=False, events_as_hyperedges=False,
        )
        self.assertTrue(pl.include_p2p)
        self.assertFalse(pl.include_mfg)
        self.assertFalse(pl.include_bank)
        self.assertFalse(pl.events_as_hyperedges)

    def test_hypergraph_settings(self):
        hg = HypergraphSettings(
            enabled=True,
            process_layer=ProcessLayerSettings(include_audit=False),
        )
        self.assertTrue(hg.enabled)
        self.assertFalse(hg.process_layer.include_audit)
        self.assertTrue(hg.process_layer.include_p2p)

    def test_graph_export_with_hypergraph_to_dict(self):
        config = _make_base_config(
            graph_export=GraphExportSettings(
                enabled=True,
                formats=["pytorch_geometric", "neo4j"],
                hypergraph=HypergraphSettings(
                    enabled=True,
                    process_layer=ProcessLayerSettings(
                        include_s2c=False,
                        events_as_hyperedges=True,
                        docs_per_counterparty_threshold=50,
                    ),
                ),
            ),
        )
        payload = config.to_dict()
        ge = payload["graph_export"]
        self.assertTrue(ge["enabled"])
        self.assertIn("pytorch_geometric", ge["formats"])
        self.assertTrue(ge["hypergraph"]["enabled"])
        self.assertFalse(ge["hypergraph"]["process_layer"]["include_s2c"])
        self.assertTrue(ge["hypergraph"]["process_layer"]["events_as_hyperedges"])
        self.assertEqual(ge["hypergraph"]["process_layer"]["docs_per_counterparty_threshold"], 50)

    def test_graph_export_with_hypergraph_round_trip(self):
        data = {
            "global": {"industry": "retail"},
            "companies": [{"code": "C001", "name": "T"}],
            "graph_export": {
                "enabled": True,
                "formats": ["dgl"],
                "hypergraph": {
                    "enabled": True,
                    "process_layer": {
                        "include_p2p": True,
                        "include_o2c": True,
                        "include_mfg": False,
                        "events_as_hyperedges": False,
                    },
                },
            },
        }
        config = Config.from_dict(data)
        self.assertTrue(config.graph_export.hypergraph.enabled)
        self.assertFalse(config.graph_export.hypergraph.process_layer.include_mfg)
        self.assertFalse(config.graph_export.hypergraph.process_layer.events_as_hyperedges)
        # Defaults for unspecified fields
        self.assertTrue(config.graph_export.hypergraph.process_layer.include_s2c)

        result = config.to_dict()
        self.assertFalse(result["graph_export"]["hypergraph"]["process_layer"]["include_mfg"])

    def test_graph_export_without_hypergraph(self):
        config = _make_base_config(
            graph_export=GraphExportSettings(enabled=True, formats=["neo4j"]),
        )
        payload = config.to_dict()
        self.assertNotIn("hypergraph", payload["graph_export"])

    def test_graph_export_without_hypergraph_round_trip(self):
        data = {
            "global": {"industry": "retail"},
            "companies": [{"code": "C001", "name": "T"}],
            "graph_export": {"enabled": True, "formats": ["neo4j"]},
        }
        config = Config.from_dict(data)
        self.assertTrue(config.graph_export.enabled)
        self.assertIsNone(config.graph_export.hypergraph)


class TestAccountingStandards(unittest.TestCase):
    """Test accounting and audit standards config round-trip."""

    def test_accounting_standards_round_trip(self):
        data = {
            "global": {"industry": "retail"},
            "companies": [{"code": "C001", "name": "T"}],
            "accounting_standards": {
                "enabled": True,
                "framework": "dual_reporting",
                "revenue_recognition": {"enabled": True, "contract_count": 200},
                "leases": {"enabled": True, "lease_count": 75},
                "fair_value": {"enabled": True, "level1_percent": 0.50},
                "impairment": {"enabled": True, "impairment_rate": 0.15},
                "generate_differences": True,
            },
        }
        config = Config.from_dict(data)
        self.assertTrue(config.accounting_standards.enabled)
        self.assertEqual(config.accounting_standards.framework, "dual_reporting")
        self.assertEqual(config.accounting_standards.revenue_recognition.contract_count, 200)
        self.assertEqual(config.accounting_standards.leases.lease_count, 75)
        self.assertAlmostEqual(config.accounting_standards.fair_value.level1_percent, 0.50)

        result = config.to_dict()
        self.assertTrue(result["accounting_standards"]["generate_differences"])

    def test_audit_standards_round_trip(self):
        data = {
            "global": {"industry": "retail"},
            "companies": [{"code": "C001", "name": "T"}],
            "audit_standards": {
                "enabled": True,
                "isa_compliance": {
                    "enabled": True,
                    "compliance_level": "comprehensive",
                    "framework": "dual",
                },
                "sox": {"enabled": True, "materiality_threshold": 25000.0},
                "generate_audit_trail": True,
            },
        }
        config = Config.from_dict(data)
        self.assertTrue(config.audit_standards.enabled)
        self.assertEqual(config.audit_standards.isa_compliance.framework, "dual")
        self.assertAlmostEqual(config.audit_standards.sox.materiality_threshold, 25000.0)

        result = config.to_dict()
        self.assertTrue(result["audit_standards"]["generate_audit_trail"])


class TestAdvancedDistributions(unittest.TestCase):
    """Test advanced distribution config round-trip."""

    def test_distributions_round_trip(self):
        data = {
            "global": {"industry": "retail"},
            "companies": [{"code": "C001", "name": "T"}],
            "distributions": {
                "enabled": True,
                "industry_profile": "manufacturing",
                "amounts": {
                    "enabled": True,
                    "distribution_type": "lognormal",
                    "components": [
                        {"weight": 0.6, "mu": 6.0, "sigma": 1.5, "label": "routine"},
                        {"weight": 0.4, "mu": 9.0, "sigma": 1.0, "label": "major"},
                    ],
                    "benford_compliance": True,
                },
                "correlations": {
                    "enabled": True,
                    "copula_type": "gaussian",
                    "fields": [
                        {"name": "amount", "distribution_type": "lognormal"},
                        {"name": "line_items", "distribution_type": "normal", "min_value": 1},
                    ],
                    "matrix": [[1.0, 0.65], [0.65, 1.0]],
                },
                "validation": {
                    "enabled": True,
                    "tests": [
                        {"test_type": "benford_first_digit", "threshold_mad": 0.015},
                    ],
                    "fail_on_violation": False,
                },
            },
        }
        config = Config.from_dict(data)
        self.assertTrue(config.distributions.enabled)
        self.assertEqual(config.distributions.industry_profile, "manufacturing")
        self.assertEqual(len(config.distributions.amounts.components), 2)
        self.assertAlmostEqual(config.distributions.amounts.components[0].weight, 0.6)
        self.assertEqual(config.distributions.correlations.copula_type, "gaussian")
        self.assertEqual(len(config.distributions.validation.tests), 1)

        result = config.to_dict()
        self.assertEqual(len(result["distributions"]["amounts"]["components"]), 2)
        self.assertEqual(result["distributions"]["correlations"]["matrix"][0][1], 0.65)


class TestTemporalPatterns(unittest.TestCase):
    """Test temporal patterns config round-trip."""

    def test_temporal_patterns_round_trip(self):
        data = {
            "global": {"industry": "retail"},
            "companies": [{"code": "C001", "name": "T"}],
            "temporal_patterns": {
                "enabled": True,
                "business_days": {
                    "enabled": True,
                    "half_day_policy": "half_day",
                    "settlement_rules": {"equity_days": 2, "fx_spot_days": 2},
                },
                "calendars": {"regions": ["US", "DE"]},
                "period_end": {
                    "enabled": True,
                    "model": "exponential",
                    "month_end": {"start_day": -10, "peak_multiplier": 3.5},
                },
                "intraday": {
                    "enabled": True,
                    "segments": [
                        {"name": "morning", "start": "08:30", "end": "10:00", "multiplier": 1.8},
                    ],
                },
                "timezones": {
                    "enabled": True,
                    "default_timezone": "America/New_York",
                    "entity_mappings": [
                        {"pattern": "EU_*", "timezone": "Europe/London"},
                    ],
                },
            },
        }
        config = Config.from_dict(data)
        self.assertTrue(config.temporal_patterns.enabled)
        self.assertEqual(config.temporal_patterns.business_days.half_day_policy, "half_day")
        self.assertIn("US", config.temporal_patterns.calendars.regions)
        self.assertEqual(config.temporal_patterns.period_end.model, "exponential")
        self.assertEqual(len(config.temporal_patterns.intraday.segments), 1)
        self.assertEqual(
            config.temporal_patterns.timezones.entity_mappings[0].timezone, "Europe/London",
        )

        result = config.to_dict()
        self.assertIn("DE", result["temporal_patterns"]["calendars"]["regions"])
        self.assertAlmostEqual(
            result["temporal_patterns"]["period_end"]["month_end"]["peak_multiplier"], 3.5,
        )


class TestFullConfig(unittest.TestCase):
    """Test a full enterprise config with all major sections."""

    def test_full_enterprise_config_round_trip(self):
        config = Config(
            global_settings=GlobalSettings(
                industry="manufacturing", start_date="2024-01-01",
                period_months=12, seed=42,
            ),
            companies=[
                CompanyConfig(code="M001", name="Mfg Corp", currency="USD", country="US"),
                CompanyConfig(code="M002", name="Mfg EU", currency="EUR", country="DE"),
            ],
            chart_of_accounts=ChartOfAccountsSettings(complexity="medium"),
            fraud=FraudSettings(enabled=True, rate=0.02),
            data_quality=DataQualitySettings(enabled=True, missing_rate=0.05),
            graph_export=GraphExportSettings(
                enabled=True,
                formats=["pytorch_geometric"],
                hypergraph=HypergraphSettings(
                    enabled=True,
                    process_layer=ProcessLayerSettings(events_as_hyperedges=True),
                ),
            ),
            ocpm=OcpmSettings(
                enabled=True,
                p2p_process=OcpmProcessSettings(rework_probability=0.08),
            ),
            audit=AuditSettings(enabled=True, engagements=3),
            source_to_pay=SourceToPayConfig(enabled=True),
            hr=HrConfig(
                enabled=True,
                payroll=PayrollSchemaConfig(enabled=True),
                expenses=ExpenseSchemaConfig(enabled=True),
            ),
            manufacturing=ManufacturingProcessConfig(
                enabled=True,
                production_orders=ProductionOrderSchemaConfig(orders_per_month=80),
            ),
            financial_reporting=FinancialReportingConfig(enabled=True),
            cross_process_links=CrossProcessLinksConfig(enabled=True),
        )

        payload = config.to_dict()

        # Verify all sections present
        for key in [
            "global", "companies", "chart_of_accounts", "fraud", "data_quality",
            "graph_export", "ocpm", "audit", "source_to_pay", "hr",
            "manufacturing", "financial_reporting", "cross_process_links",
        ]:
            self.assertIn(key, payload, f"Missing section: {key}")

        # Round-trip
        restored = Config.from_dict(payload)
        self.assertEqual(restored.global_settings.seed, 42)
        self.assertEqual(len(restored.companies), 2)
        self.assertTrue(restored.ocpm.enabled)
        self.assertTrue(restored.graph_export.hypergraph.enabled)
        self.assertTrue(restored.hr.enabled)
        self.assertTrue(restored.manufacturing.enabled)

        # Double round-trip should be stable
        payload2 = restored.to_dict()
        self.assertEqual(payload, payload2)


if __name__ == "__main__":
    unittest.main()
