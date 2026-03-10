//! Built-in standards catalog.
//!
//! Registers the core set of compliance standards, cross-references,
//! and jurisdiction profiles that ship with DataSynth.

use chrono::NaiveDate;

use datasynth_core::models::compliance::{
    AuditFramework, ChangeImpact, ComplianceDomain, ComplianceStandard, CrossReference,
    CrossReferenceType, IssuingBody, JurisdictionAccountingFramework, JurisdictionProfile,
    JurisdictionStandard, StandardCategory, StandardId, SupranationalBody, TemporalVersion,
};

use super::StandardRegistry;

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("built-in date must be valid")
}

/// Registers all built-in standards, cross-references, and jurisdiction profiles.
pub fn register_built_in_standards(registry: &mut StandardRegistry) {
    register_ifrs(registry);
    register_us_gaap(registry);
    register_isa(registry);
    register_pcaob(registry);
    register_sox(registry);
    register_eu_regulations(registry);
    register_basel(registry);
    register_local_gaap(registry);
    register_cross_references(registry);
    register_jurisdictions(registry);
}

// ─── IFRS ────────────────────────────────────────────────────────────────────

fn register_ifrs(registry: &mut StandardRegistry) {
    // IAS 17 (superseded by IFRS 16)
    registry.register_standard(
        ComplianceStandard::new(
            StandardId::new("IAS", "17"),
            "Leases",
            IssuingBody::Iasb,
            StandardCategory::AccountingStandard,
            ComplianceDomain::FinancialReporting,
        )
        .with_version(
            TemporalVersion::new("1997", date(1997, 1, 1), ChangeImpact::High)
                .superseded_at(date(2019, 1, 1)),
        )
        .superseded_by_standard(StandardId::new("IFRS", "16")),
    );

    // IAS 39 (superseded by IFRS 9)
    registry.register_standard(
        ComplianceStandard::new(
            StandardId::new("IAS", "39"),
            "Financial Instruments: Recognition and Measurement",
            IssuingBody::Iasb,
            StandardCategory::AccountingStandard,
            ComplianceDomain::FinancialReporting,
        )
        .with_version(
            TemporalVersion::new("1998", date(1998, 1, 1), ChangeImpact::High)
                .superseded_at(date(2018, 1, 1)),
        )
        .superseded_by_standard(StandardId::new("IFRS", "9")),
    );

    // IAS 36 - Impairment
    registry.register_standard(
        ComplianceStandard::new(
            StandardId::new("IAS", "36"),
            "Impairment of Assets",
            IssuingBody::Iasb,
            StandardCategory::AccountingStandard,
            ComplianceDomain::FinancialReporting,
        )
        .with_version(TemporalVersion::new(
            "2004",
            date(2004, 3, 31),
            ChangeImpact::High,
        ))
        .with_account_types(&["PP&E", "Intangibles", "Goodwill", "ROUAsset"])
        .with_processes(&["R2R", "A2R"]),
    );

    // IFRS 9 - Financial Instruments
    registry.register_standard(
        ComplianceStandard::new(
            StandardId::new("IFRS", "9"),
            "Financial Instruments",
            IssuingBody::Iasb,
            StandardCategory::AccountingStandard,
            ComplianceDomain::FinancialReporting,
        )
        .with_version(
            TemporalVersion::new("2018", date(2018, 1, 1), ChangeImpact::High)
                .with_change("Expected credit loss model replaces incurred loss")
                .with_change("New classification: amortized cost, FVOCI, FVTPL"),
        )
        .supersedes_standard(StandardId::new("IAS", "39"))
        .with_account_types(&[
            "FinancialAssets",
            "FinancialLiabilities",
            "Derivatives",
            "AccountsReceivable",
            "Investments",
        ])
        .with_processes(&["R2R"]),
    );

    // IFRS 13 - Fair Value
    registry.register_standard(
        ComplianceStandard::new(
            StandardId::new("IFRS", "13"),
            "Fair Value Measurement",
            IssuingBody::Iasb,
            StandardCategory::AccountingStandard,
            ComplianceDomain::FinancialReporting,
        )
        .with_version(TemporalVersion::new(
            "2013",
            date(2013, 1, 1),
            ChangeImpact::High,
        ))
        .with_account_types(&[
            "Investments",
            "Derivatives",
            "InvestmentProperty",
            "BiologicalAssets",
        ])
        .with_processes(&["R2R"]),
    );

    // IFRS 15 - Revenue
    registry.register_standard(
        ComplianceStandard::new(
            StandardId::new("IFRS", "15"),
            "Revenue from Contracts with Customers",
            IssuingBody::Iasb,
            StandardCategory::AccountingStandard,
            ComplianceDomain::FinancialReporting,
        )
        .with_version(
            TemporalVersion::new("2018", date(2018, 1, 1), ChangeImpact::High)
                .with_change("5-step revenue recognition model"),
        )
        .with_account_types(&[
            "Revenue",
            "DeferredRevenue",
            "ContractAsset",
            "AccountsReceivable",
        ])
        .with_processes(&["O2C"]),
    );

    // IFRS 16 - Leases
    registry.register_standard(
        ComplianceStandard::new(
            StandardId::new("IFRS", "16"),
            "Leases",
            IssuingBody::Iasb,
            StandardCategory::AccountingStandard,
            ComplianceDomain::FinancialReporting,
        )
        .with_version(
            TemporalVersion::new("2019", date(2019, 1, 1), ChangeImpact::High)
                .with_change("All leases on balance sheet for lessees")
                .with_change("ROU asset and lease liability recognition"),
        )
        .supersedes_standard(StandardId::new("IAS", "17"))
        .with_account_types(&[
            "Leases",
            "ROUAsset",
            "LeaseLiability",
            "Depreciation",
            "InterestExpense",
        ])
        .with_processes(&["R2R", "P2P"]),
    );

    // IFRS 17 - Insurance
    registry.register_standard(
        ComplianceStandard::new(
            StandardId::new("IFRS", "17"),
            "Insurance Contracts",
            IssuingBody::Iasb,
            StandardCategory::AccountingStandard,
            ComplianceDomain::FinancialReporting,
        )
        .with_version(
            TemporalVersion::new("2023", date(2023, 1, 1), ChangeImpact::Replacement)
                .with_early_adoption(date(2021, 1, 1)),
        )
        .with_account_types(&[
            "InsuranceLiabilities",
            "InsuranceRevenue",
            "DeferredAcquisitionCosts",
        ])
        .with_processes(&["R2R"]),
    );

    // IFRS 18 - Presentation (upcoming)
    registry.register_standard(
        ComplianceStandard::new(
            StandardId::new("IFRS", "18"),
            "Presentation and Disclosure in Financial Statements",
            IssuingBody::Iasb,
            StandardCategory::AccountingStandard,
            ComplianceDomain::FinancialReporting,
        )
        .with_version(
            TemporalVersion::new("2027", date(2027, 1, 1), ChangeImpact::Replacement)
                .with_early_adoption(date(2025, 1, 1))
                .with_change("New P&L categories: operating, investing, financing")
                .with_change("Management-defined performance measures"),
        )
        .with_account_types(&[
            "Revenue",
            "OperatingExpenses",
            "FinancingCosts",
            "InvestmentIncome",
        ])
        .with_processes(&["R2R"]),
    );
}

// ─── US GAAP ─────────────────────────────────────────────────────────────────

#[allow(clippy::type_complexity)]
fn register_us_gaap(registry: &mut StandardRegistry) {
    // (body, number, title, effective_date, account_types, processes)
    let us_stds: Vec<(&str, &str, &str, NaiveDate, &[&str], &[&str])> = vec![
        (
            "ASC",
            "606",
            "Revenue from Contracts with Customers",
            date(2018, 1, 1),
            &[
                "Revenue",
                "DeferredRevenue",
                "ContractAsset",
                "AccountsReceivable",
            ],
            &["O2C"],
        ),
        (
            "ASC",
            "842",
            "Leases",
            date(2019, 1, 1),
            &[
                "Leases",
                "ROUAsset",
                "LeaseLiability",
                "Depreciation",
                "InterestExpense",
            ],
            &["R2R", "P2P"],
        ),
        (
            "ASC",
            "820",
            "Fair Value Measurement",
            date(2008, 1, 1),
            &["Investments", "Derivatives", "InvestmentProperty"],
            &["R2R"],
        ),
        (
            "ASC",
            "326",
            "Financial Instruments — Credit Losses (CECL)",
            date(2020, 1, 1),
            &[
                "AccountsReceivable",
                "FinancialAssets",
                "AllowanceForCreditLosses",
            ],
            &["O2C", "R2R"],
        ),
        (
            "ASC",
            "360",
            "Property, Plant, and Equipment",
            date(2005, 1, 1),
            &["PP&E", "Depreciation", "Intangibles", "Goodwill"],
            &["R2R", "A2R"],
        ),
        (
            "ASC",
            "740",
            "Income Taxes",
            date(1992, 1, 1),
            &[
                "IncomeTax",
                "DeferredTaxAsset",
                "DeferredTaxLiability",
                "TaxProvision",
            ],
            &["R2R"],
        ),
        (
            "ASC",
            "805",
            "Business Combinations",
            date(2009, 1, 1),
            &["Goodwill", "Intangibles", "Investments"],
            &["R2R"],
        ),
        (
            "ASC",
            "810",
            "Consolidation",
            date(2010, 1, 1),
            &["Investments", "NonControllingInterest", "Goodwill"],
            &["R2R", "Intercompany"],
        ),
        (
            "ASC",
            "718",
            "Compensation — Stock Compensation",
            date(2006, 1, 1),
            &["StockCompensation", "EquityAwards", "CompensationExpense"],
            &["H2R", "R2R"],
        ),
    ];

    for (body, num, title, eff, accounts, processes) in us_stds {
        registry.register_standard(
            ComplianceStandard::new(
                StandardId::new(body, num),
                title,
                IssuingBody::Fasb,
                StandardCategory::AccountingStandard,
                ComplianceDomain::FinancialReporting,
            )
            .with_version(TemporalVersion::new(
                eff.format("%Y").to_string(),
                eff,
                ChangeImpact::High,
            ))
            .mandatory_in("US")
            .with_account_types(accounts)
            .with_processes(processes),
        );
    }
}

// ─── ISA ─────────────────────────────────────────────────────────────────────

fn register_isa(registry: &mut StandardRegistry) {
    let isa_stds = vec![
        (
            "200",
            "Overall Objectives of the Independent Auditor",
            date(2009, 12, 15),
        ),
        (
            "210",
            "Agreeing the Terms of Audit Engagements",
            date(2009, 12, 15),
        ),
        ("220", "Quality Management for an Audit", date(2022, 12, 15)),
        ("230", "Audit Documentation", date(2009, 12, 15)),
        (
            "240",
            "The Auditor's Responsibilities Relating to Fraud",
            date(2009, 12, 15),
        ),
        (
            "250",
            "Consideration of Laws and Regulations",
            date(2009, 12, 15),
        ),
        (
            "260",
            "Communication with Those Charged with Governance",
            date(2009, 12, 15),
        ),
        (
            "265",
            "Communicating Deficiencies in Internal Control",
            date(2009, 12, 15),
        ),
        (
            "300",
            "Planning an Audit of Financial Statements",
            date(2009, 12, 15),
        ),
        (
            "315",
            "Identifying and Assessing Risks of Material Misstatement",
            date(2021, 12, 15),
        ),
        (
            "320",
            "Materiality in Planning and Performing an Audit",
            date(2009, 12, 15),
        ),
        (
            "330",
            "The Auditor's Responses to Assessed Risks",
            date(2009, 12, 15),
        ),
        (
            "450",
            "Evaluation of Misstatements Identified during the Audit",
            date(2009, 12, 15),
        ),
        ("500", "Audit Evidence", date(2009, 12, 15)),
        ("505", "External Confirmations", date(2009, 12, 15)),
        ("520", "Analytical Procedures", date(2009, 12, 15)),
        ("530", "Audit Sampling", date(2009, 12, 15)),
        (
            "540",
            "Auditing Accounting Estimates and Related Disclosures",
            date(2019, 12, 15),
        ),
        ("550", "Related Parties", date(2009, 12, 15)),
        ("560", "Subsequent Events", date(2009, 12, 15)),
        ("570", "Going Concern", date(2015, 12, 15)),
        ("580", "Written Representations", date(2009, 12, 15)),
        (
            "600",
            "Special Considerations — Audits of Group Financial Statements",
            date(2023, 12, 15),
        ),
        (
            "700",
            "Forming an Opinion and Reporting on Financial Statements",
            date(2016, 12, 15),
        ),
        ("701", "Communicating Key Audit Matters", date(2016, 12, 15)),
        (
            "705",
            "Modifications to the Opinion in the Independent Auditor's Report",
            date(2016, 12, 15),
        ),
        (
            "706",
            "Emphasis of Matter and Other Matter Paragraphs",
            date(2016, 12, 15),
        ),
        (
            "720",
            "The Auditor's Responsibilities Relating to Other Information",
            date(2016, 12, 15),
        ),
    ];

    for (num, title, eff) in isa_stds {
        // Map ISA standards to relevant account types and processes
        let (accounts, processes): (&[&str], &[&str]) = match num {
            "240" => (
                &["Revenue", "Cash", "AccountsReceivable"],
                &["O2C", "R2R", "P2P"],
            ),
            "315" | "330" => (&[], &["O2C", "P2P", "R2R", "H2R", "A2R", "Intercompany"]),
            "320" | "450" => (&[], &["R2R"]),
            "500" | "530" => (&[], &["O2C", "P2P", "R2R"]),
            "505" => (
                &[
                    "AccountsReceivable",
                    "AccountsPayable",
                    "Cash",
                    "Investments",
                ],
                &["O2C", "P2P", "R2R"],
            ),
            "520" => (&["Revenue", "OperatingExpenses", "COGS"], &["R2R"]),
            "540" => (
                &[
                    "AllowanceForCreditLosses",
                    "Goodwill",
                    "Provisions",
                    "FairValue",
                ],
                &["R2R"],
            ),
            "550" => (&[], &["Intercompany"]),
            "570" => (&[], &["R2R"]),
            _ => (&[], &[]),
        };

        let mut std = ComplianceStandard::new(
            StandardId::new("ISA", num),
            title,
            IssuingBody::Iaasb,
            StandardCategory::AuditingStandard,
            ComplianceDomain::ExternalAudit,
        )
        .with_version(TemporalVersion::new(
            eff.format("%Y").to_string(),
            eff,
            ChangeImpact::High,
        ));

        if !accounts.is_empty() {
            std = std.with_account_types(accounts);
        }
        if !processes.is_empty() {
            std = std.with_processes(processes);
        }

        registry.register_standard(std);
    }
}

// ─── PCAOB ───────────────────────────────────────────────────────────────────

fn register_pcaob(registry: &mut StandardRegistry) {
    let pcaob_stds = vec![
        (
            "PCAOB-AS",
            "2201",
            "An Audit of ICFR Integrated with an Audit of Financial Statements",
            date(2007, 12, 20),
        ),
        (
            "PCAOB-AS",
            "2110",
            "Identifying and Assessing Risks of Material Misstatement",
            date(2010, 12, 15),
        ),
        (
            "PCAOB-AS",
            "2301",
            "The Auditor's Responses to the Risks of Material Misstatement",
            date(2010, 12, 15),
        ),
        (
            "PCAOB-AS",
            "3101",
            "The Auditor's Report on an Audit of Financial Statements",
            date(2017, 6, 1),
        ),
    ];

    for (body, num, title, eff) in pcaob_stds {
        let processes: &[&str] = match num {
            "2201" => &["O2C", "P2P", "R2R", "H2R", "A2R", "Intercompany"],
            "2110" | "2301" => &["O2C", "P2P", "R2R"],
            _ => &["R2R"],
        };
        registry.register_standard(
            ComplianceStandard::new(
                StandardId::new(body, num),
                title,
                IssuingBody::Pcaob,
                StandardCategory::AuditingStandard,
                ComplianceDomain::ExternalAudit,
            )
            .with_version(TemporalVersion::new(
                eff.format("%Y").to_string(),
                eff,
                ChangeImpact::High,
            ))
            .mandatory_in("US")
            .with_processes(processes),
        );
    }
}

// ─── SOX ─────────────────────────────────────────────────────────────────────

fn register_sox(registry: &mut StandardRegistry) {
    let sox_stds = vec![
        ("302", "Corporate Responsibility for Financial Reports"),
        ("404", "Management Assessment of Internal Controls"),
        (
            "906",
            "Corporate Responsibility for Financial Reports (Criminal)",
        ),
    ];

    for (num, title) in sox_stds {
        registry.register_standard(
            ComplianceStandard::new(
                StandardId::new("SOX", num),
                title,
                IssuingBody::Sec,
                StandardCategory::RegulatoryRequirement,
                ComplianceDomain::InternalControl,
            )
            .with_version(TemporalVersion::new(
                "2002",
                date(2002, 7, 30),
                ChangeImpact::Replacement,
            ))
            .mandatory_in("US")
            .with_processes(&["O2C", "P2P", "R2R", "H2R", "A2R", "Intercompany"]),
        );
    }
}

// ─── EU Regulations ──────────────────────────────────────────────────────────

fn register_eu_regulations(registry: &mut StandardRegistry) {
    registry.register_standard(
        ComplianceStandard::new(
            StandardId::new("EU-AR", "537"),
            "EU Audit Regulation (No 537/2014)",
            IssuingBody::EuropeanUnion,
            StandardCategory::RegulatoryRequirement,
            ComplianceDomain::ExternalAudit,
        )
        .with_version(TemporalVersion::new(
            "2016",
            date(2016, 6, 17),
            ChangeImpact::High,
        )),
    );

    registry.register_standard(
        ComplianceStandard::new(
            StandardId::from("EU-CSRD"),
            "Corporate Sustainability Reporting Directive",
            IssuingBody::EuropeanUnion,
            StandardCategory::SustainabilityStandard,
            ComplianceDomain::Sustainability,
        )
        .with_version(
            TemporalVersion::new("2024-phase1", date(2024, 1, 1), ChangeImpact::High)
                .with_change("Large PIEs begin sustainability reporting"),
        )
        .with_version(
            TemporalVersion::new("2025-phase2", date(2025, 1, 1), ChangeImpact::Medium)
                .with_change("Large non-PIE entities"),
        ),
    );

    registry.register_standard(
        ComplianceStandard::new(
            StandardId::from("EU-AMLD-6"),
            "6th Anti-Money Laundering Directive",
            IssuingBody::EuropeanUnion,
            StandardCategory::RegulatoryRequirement,
            ComplianceDomain::AntiMoneyLaundering,
        )
        .with_version(TemporalVersion::new(
            "2021",
            date(2021, 12, 3),
            ChangeImpact::High,
        )),
    );
}

// ─── Basel ───────────────────────────────────────────────────────────────────

fn register_basel(registry: &mut StandardRegistry) {
    registry.register_standard(
        ComplianceStandard::new(
            StandardId::from("BASEL-III-CAP"),
            "Basel III Capital Requirements",
            IssuingBody::Bcbs,
            StandardCategory::PrudentialRegulation,
            ComplianceDomain::PrudentialCapital,
        )
        .with_version(TemporalVersion::new(
            "2013",
            date(2013, 1, 1),
            ChangeImpact::High,
        )),
    );

    registry.register_standard(
        ComplianceStandard::new(
            StandardId::from("BASEL-III-LCR"),
            "Liquidity Coverage Ratio",
            IssuingBody::Bcbs,
            StandardCategory::PrudentialRegulation,
            ComplianceDomain::PrudentialCapital,
        )
        .with_version(TemporalVersion::new(
            "2015",
            date(2015, 1, 1),
            ChangeImpact::High,
        )),
    );

    registry.register_standard(
        ComplianceStandard::new(
            StandardId::from("BASEL-III-NSFR"),
            "Net Stable Funding Ratio",
            IssuingBody::Bcbs,
            StandardCategory::PrudentialRegulation,
            ComplianceDomain::PrudentialCapital,
        )
        .with_version(TemporalVersion::new(
            "2018",
            date(2018, 1, 1),
            ChangeImpact::High,
        )),
    );

    registry.register_standard(
        ComplianceStandard::new(
            StandardId::from("BASEL-IV-SA"),
            "Basel IV Standardized Approach (Revised)",
            IssuingBody::Bcbs,
            StandardCategory::PrudentialRegulation,
            ComplianceDomain::PrudentialCapital,
        )
        .with_version(TemporalVersion::new(
            "2025",
            date(2025, 1, 1),
            ChangeImpact::High,
        )),
    );
}

// ─── Local GAAP ──────────────────────────────────────────────────────────────

fn register_local_gaap(registry: &mut StandardRegistry) {
    // German HGB
    registry.register_standard(
        ComplianceStandard::new(
            StandardId::from("HGB-253"),
            "Depreciation and Write-downs (§253 HGB)",
            IssuingBody::Custom("German Legislature".to_string()),
            StandardCategory::AccountingStandard,
            ComplianceDomain::FinancialReporting,
        )
        .with_version(TemporalVersion::new(
            "1985",
            date(1985, 1, 1),
            ChangeImpact::High,
        ))
        .mandatory_in("DE"),
    );

    // French PCG
    registry.register_standard(
        ComplianceStandard::new(
            StandardId::from("PCG-99"),
            "Plan Comptable Général",
            IssuingBody::Anc,
            StandardCategory::AccountingStandard,
            ComplianceDomain::FinancialReporting,
        )
        .with_version(TemporalVersion::new(
            "1999",
            date(1999, 1, 1),
            ChangeImpact::High,
        ))
        .mandatory_in("FR"),
    );

    // UK FRS 102
    registry.register_standard(
        ComplianceStandard::new(
            StandardId::from("FRS-102"),
            "The Financial Reporting Standard applicable in the UK and Republic of Ireland",
            IssuingBody::Frc,
            StandardCategory::AccountingStandard,
            ComplianceDomain::FinancialReporting,
        )
        .with_version(TemporalVersion::new(
            "2015",
            date(2015, 1, 1),
            ChangeImpact::High,
        ))
        .with_version(
            TemporalVersion::new("2026-amended", date(2026, 1, 1), ChangeImpact::High)
                .with_change("IFRS 15-aligned revenue recognition")
                .with_change("IFRS 16-aligned lease accounting"),
        )
        .mandatory_in("GB"),
    );
}

// ─── Cross-References ────────────────────────────────────────────────────────

fn register_cross_references(registry: &mut StandardRegistry) {
    let xrefs = vec![
        // Revenue: IFRS 15 ↔ ASC 606 (joint standard)
        (
            ("IFRS", "15"),
            ("ASC", "606"),
            CrossReferenceType::Converged,
        ),
        // Leases: IFRS 16 ↔ ASC 842 (related but different classification)
        (("IFRS", "16"), ("ASC", "842"), CrossReferenceType::Related),
        // Fair Value: IFRS 13 ↔ ASC 820 (substantially converged)
        (
            ("IFRS", "13"),
            ("ASC", "820"),
            CrossReferenceType::Converged,
        ),
        // Financial Instruments: IFRS 9 ↔ ASC 326 (different impairment models)
        (("IFRS", "9"), ("ASC", "326"), CrossReferenceType::Related),
        // Business Combinations: IFRS 3 and ASC 805 are joint
        // Audit: ISA 315 ↔ PCAOB AS 2110
        (
            ("ISA", "315"),
            ("PCAOB-AS", "2110"),
            CrossReferenceType::AuditMapping,
        ),
        // Audit: ISA 330 ↔ PCAOB AS 2301
        (
            ("ISA", "330"),
            ("PCAOB-AS", "2301"),
            CrossReferenceType::AuditMapping,
        ),
        // SOX 404 ↔ PCAOB AS 2201 (ICFR)
        (
            ("SOX", "404"),
            ("PCAOB-AS", "2201"),
            CrossReferenceType::ControlFrameworkMapping,
        ),
        // Impairment: IAS 36 ↔ ASC 360
        (("IAS", "36"), ("ASC", "360"), CrossReferenceType::Related),
    ];

    for ((fb, fn_), (tb, tn), rel) in xrefs {
        registry.add_cross_reference(CrossReference::new(
            StandardId::new(fb, fn_),
            StandardId::new(tb, tn),
            rel,
        ));
    }
}

// ─── Jurisdictions ───────────────────────────────────────────────────────────

fn register_jurisdictions(registry: &mut StandardRegistry) {
    // United States
    registry.register_jurisdiction({
        let mut p = JurisdictionProfile::new(
            "US",
            "United States of America",
            JurisdictionAccountingFramework::UsGaap,
            AuditFramework::Pcaob,
            "USD",
        );
        p.accounting_standards_body = "FASB".to_string();
        p.audit_oversight_body = "PCAOB".to_string();
        p.securities_regulator = Some("SEC".to_string());
        p.stock_exchanges = vec!["NYSE".to_string(), "NASDAQ".to_string()];
        p.corporate_tax_rate = Some(0.21);
        p.mandatory_standards = vec![
            JurisdictionStandard {
                standard_id: StandardId::new("SOX", "302"),
                local_effective_date: None,
                local_designation: None,
                applicability: vec![],
            },
            JurisdictionStandard {
                standard_id: StandardId::new("SOX", "404"),
                local_effective_date: None,
                local_designation: None,
                applicability: vec![],
            },
        ];
        p
    });

    // Germany
    registry.register_jurisdiction({
        let mut p = JurisdictionProfile::new(
            "DE",
            "Federal Republic of Germany",
            JurisdictionAccountingFramework::LocalGaapWithIfrs,
            AuditFramework::IsaLocal,
            "EUR",
        );
        p.memberships = vec![
            SupranationalBody::Eu,
            SupranationalBody::Eea,
            SupranationalBody::Eurozone,
        ];
        p.accounting_standards_body = "DRSC".to_string();
        p.audit_oversight_body = "IDW".to_string();
        p.securities_regulator = Some("BaFin".to_string());
        p.stock_exchanges = vec!["XETRA".to_string()];
        p.corporate_tax_rate = Some(0.15);
        p.ifrs_required_for_listed = true;
        p.audit_export_format = Some("gobd".to_string());
        p
    });

    // United Kingdom
    registry.register_jurisdiction({
        let mut p = JurisdictionProfile::new(
            "GB",
            "United Kingdom",
            JurisdictionAccountingFramework::LocalGaapWithIfrs,
            AuditFramework::IsaLocal,
            "GBP",
        );
        p.accounting_standards_body = "FRC".to_string();
        p.audit_oversight_body = "FRC".to_string();
        p.securities_regulator = Some("FCA".to_string());
        p.stock_exchanges = vec!["LSE".to_string()];
        p.corporate_tax_rate = Some(0.25);
        p.ifrs_required_for_listed = true;
        p
    });

    // France
    registry.register_jurisdiction({
        let mut p = JurisdictionProfile::new(
            "FR",
            "French Republic",
            JurisdictionAccountingFramework::LocalGaapWithIfrs,
            AuditFramework::Isa,
            "EUR",
        );
        p.memberships = vec![
            SupranationalBody::Eu,
            SupranationalBody::Eea,
            SupranationalBody::Eurozone,
        ];
        p.accounting_standards_body = "ANC".to_string();
        p.audit_oversight_body = "H3C".to_string();
        p.securities_regulator = Some("AMF".to_string());
        p.corporate_tax_rate = Some(0.25);
        p.ifrs_required_for_listed = true;
        p.audit_export_format = Some("fec".to_string());
        p
    });

    // Japan
    registry.register_jurisdiction({
        let mut p = JurisdictionProfile::new(
            "JP",
            "Japan",
            JurisdictionAccountingFramework::LocalGaap,
            AuditFramework::LocalIsaBased,
            "JPY",
        );
        p.accounting_standards_body = "ASBJ".to_string();
        p.audit_oversight_body = "JICPA".to_string();
        p.securities_regulator = Some("JFSA".to_string());
        p.stock_exchanges = vec!["TSE".to_string()];
        p.corporate_tax_rate = Some(0.2315);
        p
    });

    // India
    registry.register_jurisdiction({
        let mut p = JurisdictionProfile::new(
            "IN",
            "Republic of India",
            JurisdictionAccountingFramework::IfrsConverged,
            AuditFramework::LocalIsaBased,
            "INR",
        );
        p.accounting_standards_body = "ICAI".to_string();
        p.audit_oversight_body = "ICAI".to_string();
        p.securities_regulator = Some("SEBI".to_string());
        p.stock_exchanges = vec!["BSE".to_string(), "NSE".to_string()];
        p.corporate_tax_rate = Some(0.2517);
        p
    });

    // Singapore
    registry.register_jurisdiction({
        let mut p = JurisdictionProfile::new(
            "SG",
            "Republic of Singapore",
            JurisdictionAccountingFramework::Ifrs,
            AuditFramework::Isa,
            "SGD",
        );
        p.accounting_standards_body = "ASC".to_string();
        p.audit_oversight_body = "ISCA".to_string();
        p.securities_regulator = Some("MAS".to_string());
        p.stock_exchanges = vec!["SGX".to_string()];
        p.corporate_tax_rate = Some(0.17);
        p
    });

    // Australia
    registry.register_jurisdiction({
        let mut p = JurisdictionProfile::new(
            "AU",
            "Commonwealth of Australia",
            JurisdictionAccountingFramework::Ifrs,
            AuditFramework::LocalIsaBased,
            "AUD",
        );
        p.accounting_standards_body = "AASB".to_string();
        p.audit_oversight_body = "AUASB".to_string();
        p.securities_regulator = Some("ASIC".to_string());
        p.stock_exchanges = vec!["ASX".to_string()];
        p.corporate_tax_rate = Some(0.30);
        p
    });

    // Brazil
    registry.register_jurisdiction({
        let mut p = JurisdictionProfile::new(
            "BR",
            "Federative Republic of Brazil",
            JurisdictionAccountingFramework::IfrsConverged,
            AuditFramework::LocalIsaBased,
            "BRL",
        );
        p.accounting_standards_body = "CPC".to_string();
        p.audit_oversight_body = "CFC".to_string();
        p.securities_regulator = Some("CVM".to_string());
        p.stock_exchanges = vec!["B3".to_string()];
        p.corporate_tax_rate = Some(0.34);
        p
    });

    // South Korea
    registry.register_jurisdiction({
        let mut p = JurisdictionProfile::new(
            "KR",
            "Republic of Korea",
            JurisdictionAccountingFramework::Ifrs,
            AuditFramework::LocalIsaBased,
            "KRW",
        );
        p.accounting_standards_body = "KASB".to_string();
        p.audit_oversight_body = "KICPA".to_string();
        p.securities_regulator = Some("FSC".to_string());
        p.stock_exchanges = vec!["KRX".to_string()];
        p.corporate_tax_rate = Some(0.24);
        p
    });
}
