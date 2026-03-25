//! Organizational profile generator.
//!
//! Generates per-entity IT landscape, regulatory environment, prior auditor,
//! and organizational structure descriptions for ISA 315 risk assessment.

use datasynth_core::models::{ItSystem, OrganizationalProfile};
use datasynth_core::utils::seeded_rng;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

// ---------------------------------------------------------------------------
// IT system pools by industry
// ---------------------------------------------------------------------------

/// An IT system template: (name, vendor, module, category).
type SystemDef = (&'static str, &'static str, &'static str, &'static str);

const RETAIL_SYSTEMS: &[SystemDef] = &[
    ("SAP S/4HANA Retail", "SAP", "ERP", "core_financial"),
    ("Oracle Retail", "Oracle", "ERP", "core_financial"),
    (
        "Salesforce Commerce Cloud",
        "Salesforce",
        "CRM",
        "operational",
    ),
    ("Shopify Plus", "Shopify", "E-Commerce", "operational"),
    (
        "Manhattan WMS",
        "Manhattan Associates",
        "WMS",
        "operational",
    ),
    ("SAP SuccessFactors", "SAP", "HCM", "operational"),
    ("Anaplan", "Anaplan", "Planning", "reporting"),
    ("Board International", "Board", "BI", "reporting"),
    ("NCR POS", "NCR", "POS", "operational"),
    (
        "Workday Financials",
        "Workday",
        "Financials",
        "core_financial",
    ),
];

const MANUFACTURING_SYSTEMS: &[SystemDef] = &[
    ("SAP S/4HANA", "SAP", "ERP", "core_financial"),
    ("Oracle EBS", "Oracle", "ERP", "core_financial"),
    ("Siemens Opcenter", "Siemens", "MES", "operational"),
    ("PTC Windchill", "PTC", "PLM", "operational"),
    ("Veeva Vault QMS", "Veeva", "QMS", "operational"),
    (
        "Infor CloudSuite Industrial",
        "Infor",
        "ERP",
        "core_financial",
    ),
    (
        "SAP Integrated Business Planning",
        "SAP",
        "Planning",
        "reporting",
    ),
    (
        "Microsoft Dynamics 365 SCM",
        "Microsoft",
        "SCM",
        "operational",
    ),
    ("Kronos Workforce Central", "Kronos", "HCM", "operational"),
    ("Power BI", "Microsoft", "BI", "reporting"),
];

const FINANCIAL_SERVICES_SYSTEMS: &[SystemDef] = &[
    ("Temenos T24", "Temenos", "Core Banking", "core_financial"),
    (
        "FIS Modern Banking Platform",
        "FIS",
        "Core Banking",
        "core_financial",
    ),
    ("Murex MX.3", "Murex", "Trading", "operational"),
    (
        "Moody's RiskCalc",
        "Moody's",
        "Risk Management",
        "operational",
    ),
    (
        "Wolters Kluwer OneSumX",
        "Wolters Kluwer",
        "Regulatory Reporting",
        "reporting",
    ),
    (
        "Oracle FLEXCUBE",
        "Oracle",
        "Core Banking",
        "core_financial",
    ),
    ("Calypso", "Adenza", "Treasury", "core_financial"),
    ("SAS Anti-Money Laundering", "SAS", "AML", "operational"),
    ("Workday HCM", "Workday", "HCM", "operational"),
    ("Tableau", "Salesforce", "BI", "reporting"),
];

const GENERIC_SYSTEMS: &[SystemDef] = &[
    ("SAP S/4HANA", "SAP", "ERP", "core_financial"),
    (
        "Microsoft Dynamics 365",
        "Microsoft",
        "ERP",
        "core_financial",
    ),
    ("Salesforce CRM", "Salesforce", "CRM", "operational"),
    ("Workday HCM", "Workday", "HCM", "operational"),
    ("ServiceNow", "ServiceNow", "ITSM", "operational"),
    ("Oracle EPM Cloud", "Oracle", "Planning", "reporting"),
    ("Power BI", "Microsoft", "BI", "reporting"),
];

// ---------------------------------------------------------------------------
// Regulatory environment pools
// ---------------------------------------------------------------------------

const RETAIL_REGULATIONS: &[&str] = &[
    "Consumer Protection (FTC Act)",
    "PCI DSS",
    "GDPR / Data Privacy",
    "CCPA",
    "SOX (if publicly listed)",
    "OSHA Workplace Safety",
    "Fair Labor Standards Act",
    "Environmental Compliance (EPA)",
];

const MANUFACTURING_REGULATIONS: &[&str] = &[
    "SOX (if publicly listed)",
    "ISO 9001 Quality Management",
    "ISO 14001 Environmental Management",
    "OSHA Workplace Safety",
    "EPA Environmental Regulations",
    "REACH Chemical Regulations",
    "Export Control (EAR/ITAR)",
    "Customs and Trade Compliance",
    "GDPR / Data Privacy",
];

const FINANCIAL_SERVICES_REGULATIONS: &[&str] = &[
    "SOX (Sarbanes-Oxley Act)",
    "Basel III / Basel IV Capital Requirements",
    "Dodd-Frank Act",
    "GDPR / Data Privacy",
    "AML / KYC (Bank Secrecy Act)",
    "MiFID II",
    "IFRS 9 Expected Credit Losses",
    "PSD2 Payment Services",
    "FATCA / CRS Tax Reporting",
    "Consumer Financial Protection (CFPB)",
];

const GENERIC_REGULATIONS: &[&str] = &[
    "SOX (if publicly listed)",
    "GDPR / Data Privacy",
    "OSHA Workplace Safety",
    "Environmental Compliance",
    "Tax Compliance (local jurisdiction)",
    "Anti-Corruption (FCPA / UK Bribery Act)",
];

// ---------------------------------------------------------------------------
// Prior auditor pool
// ---------------------------------------------------------------------------

const AUDIT_FIRMS: &[&str] = &[
    "Deloitte",
    "PwC",
    "KPMG",
    "EY",
    "BDO",
    "Grant Thornton",
    "RSM",
];

// ---------------------------------------------------------------------------
// Organizational structure templates
// ---------------------------------------------------------------------------

const ORG_STRUCTURES: &[&str] = &[
    "Functional organizational structure with centralized finance, HR, and IT shared services.",
    "Divisional structure organized by geographic region with local P&L responsibility.",
    "Matrix organization combining functional departments with product-line teams.",
    "Holding company with semi-autonomous subsidiaries and a central treasury function.",
    "Flat organizational structure with cross-functional teams and decentralized decision-making.",
];

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Generates [`OrganizationalProfile`] records per entity, selecting
/// industry-appropriate IT systems, regulations, and structure descriptions.
pub struct OrganizationalProfileGenerator {
    rng: ChaCha8Rng,
}

impl OrganizationalProfileGenerator {
    /// Create a new generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
        }
    }

    /// Generate an organizational profile for the given entity and industry.
    pub fn generate(&mut self, entity_code: &str, industry: &str) -> OrganizationalProfile {
        let systems_pool = match industry.to_lowercase().as_str() {
            "retail" => RETAIL_SYSTEMS,
            "manufacturing" => MANUFACTURING_SYSTEMS,
            "financial_services" | "financial services" => FINANCIAL_SERVICES_SYSTEMS,
            _ => GENERIC_SYSTEMS,
        };

        let regs_pool = match industry.to_lowercase().as_str() {
            "retail" => RETAIL_REGULATIONS,
            "manufacturing" => MANUFACTURING_REGULATIONS,
            "financial_services" | "financial services" => FINANCIAL_SERVICES_REGULATIONS,
            _ => GENERIC_REGULATIONS,
        };

        // Pick 3-6 IT systems
        let sys_count = self.rng.random_range(3..=6).min(systems_pool.len());
        let it_systems = self.pick_systems(systems_pool, sys_count);

        // Pick 3-6 regulatory items
        let reg_count = self.rng.random_range(3..=6).min(regs_pool.len());
        let regulatory_environment = self.pick_strings(regs_pool, reg_count);

        // Prior auditor: ~75% chance of having one
        let prior_auditor = if self.rng.random_bool(0.75) {
            let idx = self.rng.random_range(0..AUDIT_FIRMS.len());
            Some(AUDIT_FIRMS[idx].to_string())
        } else {
            None
        };

        // Org structure description
        let struct_idx = self.rng.random_range(0..ORG_STRUCTURES.len());
        let org_structure_description = ORG_STRUCTURES[struct_idx].to_string();

        OrganizationalProfile {
            entity_code: entity_code.to_string(),
            it_systems,
            regulatory_environment,
            prior_auditor,
            org_structure_description,
        }
    }

    /// Pick `count` systems from the pool without replacement.
    fn pick_systems(&mut self, pool: &[SystemDef], count: usize) -> Vec<ItSystem> {
        let mut indices: Vec<usize> = (0..pool.len()).collect();
        indices.shuffle(&mut self.rng);
        indices.truncate(count);
        indices.sort_unstable();
        indices
            .iter()
            .map(|&i| {
                let (name, vendor, module, category) = pool[i];
                ItSystem {
                    name: name.to_string(),
                    vendor: vendor.to_string(),
                    module: module.to_string(),
                    category: category.to_string(),
                }
            })
            .collect()
    }

    /// Pick `count` strings from the pool without replacement.
    fn pick_strings(&mut self, pool: &[&str], count: usize) -> Vec<String> {
        let mut indices: Vec<usize> = (0..pool.len()).collect();
        indices.shuffle(&mut self.rng);
        indices.truncate(count);
        indices.sort_unstable();
        indices.iter().map(|&i| pool[i].to_string()).collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_generates_non_empty_output() {
        let mut gen = OrganizationalProfileGenerator::new(42);
        let profile = gen.generate("C001", "retail");
        assert!(!profile.it_systems.is_empty(), "should have IT systems");
        assert!(
            !profile.regulatory_environment.is_empty(),
            "should have regulations"
        );
        assert!(
            !profile.org_structure_description.is_empty(),
            "should have org description"
        );
    }

    #[test]
    fn test_industry_specific_it_systems() {
        let mut gen = OrganizationalProfileGenerator::new(42);
        let retail = gen.generate("C001", "retail");

        let mut gen2 = OrganizationalProfileGenerator::new(42);
        let fin = gen2.generate("C002", "financial_services");

        let retail_names: std::collections::HashSet<_> =
            retail.it_systems.iter().map(|s| s.name.as_str()).collect();
        let fin_names: std::collections::HashSet<_> =
            fin.it_systems.iter().map(|s| s.name.as_str()).collect();

        // They should not be identical (different system pools)
        assert_ne!(
            retail_names, fin_names,
            "retail and financial services IT systems should differ"
        );
    }

    #[test]
    fn test_industry_specific_regulations() {
        let mut gen = OrganizationalProfileGenerator::new(42);
        let mfg = gen.generate("C001", "manufacturing");

        let mut gen2 = OrganizationalProfileGenerator::new(42);
        let fin = gen2.generate("C002", "financial_services");

        let mfg_regs: std::collections::HashSet<_> =
            mfg.regulatory_environment.iter().cloned().collect();
        let fin_regs: std::collections::HashSet<_> =
            fin.regulatory_environment.iter().cloned().collect();

        assert_ne!(
            mfg_regs, fin_regs,
            "manufacturing and financial services regulations should differ"
        );
    }

    #[test]
    fn test_entity_code_propagated() {
        let mut gen = OrganizationalProfileGenerator::new(42);
        let profile = gen.generate("ENTITY_XYZ", "retail");
        assert_eq!(profile.entity_code, "ENTITY_XYZ");
    }

    #[test]
    fn test_prior_auditor_from_known_firms() {
        // Run multiple times to get a non-None auditor
        let mut gen = OrganizationalProfileGenerator::new(42);
        let known: std::collections::HashSet<&str> = AUDIT_FIRMS.iter().copied().collect();
        let mut found_some = false;
        for i in 0..20 {
            let profile = gen.generate(&format!("C{:03}", i), "retail");
            if let Some(ref auditor) = profile.prior_auditor {
                assert!(
                    known.contains(auditor.as_str()),
                    "prior auditor {} should be from known firm list",
                    auditor
                );
                found_some = true;
            }
        }
        assert!(
            found_some,
            "at least one entity should have a prior auditor"
        );
    }

    #[test]
    fn test_it_system_fields_populated() {
        let mut gen = OrganizationalProfileGenerator::new(42);
        let profile = gen.generate("C001", "manufacturing");
        for sys in &profile.it_systems {
            assert!(!sys.name.is_empty(), "system name should not be empty");
            assert!(!sys.vendor.is_empty(), "vendor should not be empty");
            assert!(!sys.module.is_empty(), "module should not be empty");
            assert!(
                ["core_financial", "operational", "reporting"].contains(&sys.category.as_str()),
                "category should be valid: {}",
                sys.category
            );
        }
    }

    #[test]
    fn test_deterministic_with_same_seed() {
        let mut gen1 = OrganizationalProfileGenerator::new(777);
        let p1 = gen1.generate("C001", "retail");

        let mut gen2 = OrganizationalProfileGenerator::new(777);
        let p2 = gen2.generate("C001", "retail");

        assert_eq!(p1.entity_code, p2.entity_code);
        assert_eq!(p1.it_systems.len(), p2.it_systems.len());
        assert_eq!(p1.regulatory_environment, p2.regulatory_environment);
        assert_eq!(p1.prior_auditor, p2.prior_auditor);
        assert_eq!(p1.org_structure_description, p2.org_structure_description);
        for (a, b) in p1.it_systems.iter().zip(p2.it_systems.iter()) {
            assert_eq!(a.name, b.name);
        }
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut gen = OrganizationalProfileGenerator::new(42);
        let profile = gen.generate("C001", "financial_services");
        let json = serde_json::to_string(&profile).expect("serialize");
        let parsed: OrganizationalProfile = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(profile.entity_code, parsed.entity_code);
        assert_eq!(profile.it_systems.len(), parsed.it_systems.len());
        assert_eq!(profile.prior_auditor, parsed.prior_auditor);
    }

    #[test]
    fn test_unknown_industry_uses_generic() {
        let mut gen = OrganizationalProfileGenerator::new(42);
        let profile = gen.generate("C001", "aerospace");
        assert!(
            !profile.it_systems.is_empty(),
            "generic fallback should produce systems"
        );
        assert!(
            !profile.regulatory_environment.is_empty(),
            "generic fallback should produce regulations"
        );
    }
}
