//! Risk-control edge synthesizer.
//!
//! Produces governance-layer edges linking risks, controls, employees,
//! findings, workpapers, and accounts.
//!
//! ## Edge Types Produced
//!
//! | Code | Name                    | Direction               |
//! |------|-------------------------|-------------------------|
//! |  75  | RISK_MITIGATED_BY       | risk -> control         |
//! | 120  | CONTROL_COVERS_ACCOUNT  | control -> account      |
//! | 127  | CONTROL_OWNED_BY        | control -> employee     |
//! | 128  | CONTROL_HAS_FINDING     | finding -> control      |
//! | 129  | WORKPAPER_TESTS_CONTROL | workpaper -> control    |
//! |  45  | ControlCoversProcess    | control -> PO/SO        |

use std::collections::HashMap;

use tracing::debug;

use crate::error::{ExportError, WarningSeverity};
use crate::traits::EdgeSynthesisContext;
use crate::types::ExportEdge;

/// Edge type codes produced by this synthesizer.
const RISK_MITIGATED_BY: u32 = 75;
const CONTROL_COVERS_ACCOUNT: u32 = 120;
const CONTROL_OWNED_BY: u32 = 127;
const CONTROL_HAS_FINDING: u32 = 128;
const WORKPAPER_TESTS_CONTROL: u32 = 129;
const CONTROL_COVERS_PROCESS: u32 = 45;

/// Synthesizes risk-control governance edges.
///
/// This is one of the most complex edge synthesizers because it performs
/// domain-aware matching between risks and controls when foreign-key
/// references are missing.
pub struct RiskControlEdgeSynthesizer;

impl crate::traits::EdgeSynthesizer for RiskControlEdgeSynthesizer {
    fn name(&self) -> &'static str {
        "risk_control"
    }

    fn synthesize(
        &self,
        ctx: &mut EdgeSynthesisContext<'_>,
    ) -> Result<Vec<ExportEdge>, ExportError> {
        let mut edges = Vec::new();

        // 1. RISK_MITIGATED_BY: risk -> control (code 75)
        edges.extend(self.synthesize_risk_mitigated_by(ctx));

        // 2. CONTROL_COVERS_ACCOUNT: control -> account (code 120)
        edges.extend(self.synthesize_control_covers_account(ctx));

        // 3. CONTROL_OWNED_BY: control -> employee (code 127)
        edges.extend(self.synthesize_control_owned_by(ctx));

        // 4. CONTROL_HAS_FINDING: finding -> control (code 128)
        edges.extend(self.synthesize_control_has_finding(ctx));

        // 5. WORKPAPER_TESTS_CONTROL: workpaper -> control (code 129)
        edges.extend(self.synthesize_workpaper_tests_control(ctx));

        // 6. ControlCoversProcess: control -> PO/SO (code 45)
        edges.extend(self.synthesize_control_covers_process(ctx));

        debug!(
            "RiskControlEdgeSynthesizer produced {} total edges",
            edges.len()
        );
        Ok(edges)
    }
}

impl RiskControlEdgeSynthesizer {
    /// Produce RISK_MITIGATED_BY edges (code 75): risk -> control.
    ///
    /// Uses a hybrid strategy:
    /// 1. **Foreign key**: If `control.mitigates_risk_ids` contains risk refs, use them.
    /// 2. **Name matching fallback**: Domain-aware matching based on `risk.account_or_process`
    ///    and control naming/assertion patterns.
    fn synthesize_risk_mitigated_by(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        let controls = &ctx.ds_result.internal_controls;
        let risks = &ctx.ds_result.audit.risk_assessments;
        let mut edges = Vec::new();

        if controls.is_empty() || risks.is_empty() {
            return edges;
        }

        // Build risk_ref -> risk index for FK lookup
        let risk_by_ref: HashMap<&str, usize> = risks
            .iter()
            .enumerate()
            .map(|(i, r)| (r.risk_ref.as_str(), i))
            .collect();

        // Track which risks have been matched (for fallback pass)
        let mut matched_risk_indices: std::collections::HashSet<usize> =
            std::collections::HashSet::new();

        // Pass 1: FK-based matching from control.mitigates_risk_ids
        for control in controls {
            let Some(control_id) = ctx.id_map.get(&control.control_id) else {
                continue;
            };
            for risk_ref in &control.mitigates_risk_ids {
                if let Some(&risk_idx) = risk_by_ref.get(risk_ref.as_str()) {
                    let risk = &risks[risk_idx];
                    if let Some(risk_id) = ctx.id_map.get(&risk.risk_ref) {
                        // Direction: risk -> control
                        edges.push(ExportEdge {
                            source: risk_id,
                            target: control_id,
                            edge_type: RISK_MITIGATED_BY,
                            weight: 1.0,
                            properties: HashMap::new(),
                        });
                        matched_risk_indices.insert(risk_idx);
                    }
                }
            }
        }

        let fk_count = edges.len();

        // Pass 2: Domain-aware name matching for unmatched risks
        let unmatched_risks: Vec<usize> = (0..risks.len())
            .filter(|i| !matched_risk_indices.contains(i))
            .collect();

        if !unmatched_risks.is_empty() {
            // Build domain classification for controls
            let control_domains: Vec<(&str, ControlDomain)> = controls
                .iter()
                .map(|c| (c.control_id.as_str(), classify_control_domain(c)))
                .collect();

            for risk_idx in unmatched_risks {
                let risk = &risks[risk_idx];
                let Some(risk_id) = ctx.id_map.get(&risk.risk_ref) else {
                    continue;
                };

                let risk_domain = classify_risk_domain(risk);
                let mut found_match = false;

                for (control_id_str, control_domain) in &control_domains {
                    if domains_compatible(&risk_domain, control_domain) {
                        if let Some(control_id) = ctx.id_map.get(control_id_str) {
                            edges.push(ExportEdge {
                                source: risk_id,
                                target: control_id,
                                edge_type: RISK_MITIGATED_BY,
                                weight: 0.8, // Lower weight for name-matched edges
                                properties: HashMap::new(),
                            });
                            found_match = true;
                        }
                    }
                }

                if !found_match {
                    ctx.warnings.add(
                        "risk_control",
                        WarningSeverity::Low,
                        format!(
                            "No matching control found for risk '{}' (domain: {:?})",
                            risk.risk_ref, risk_domain
                        ),
                    );
                }
            }
        }

        let name_count = edges.len() - fk_count;
        debug!(
            "RISK_MITIGATED_BY: {} FK edges + {} name-matched edges = {} total",
            fk_count,
            name_count,
            edges.len()
        );

        edges
    }

    /// Produce CONTROL_COVERS_ACCOUNT edges (code 120): control -> account.
    ///
    /// Uses `control.covers_account_classes` to link controls to accounts
    /// of the matching type. The `covers_account_classes` field contains
    /// human-readable names like "Assets", "Revenue", "Liabilities", etc.
    /// which map to the `AccountType` enum on `GLAccount`.
    fn synthesize_control_covers_account(
        &self,
        ctx: &mut EdgeSynthesisContext<'_>,
    ) -> Vec<ExportEdge> {
        let controls = &ctx.ds_result.internal_controls;
        let accounts = &ctx.ds_result.chart_of_accounts.accounts;
        let mut edges = Vec::new();

        if controls.is_empty() || accounts.is_empty() {
            return edges;
        }

        // Build account_type_label -> list of account numbers
        // Control's covers_account_classes uses: "Assets", "Revenue", "Liabilities", "Equity", "Expenses"
        // AccountType enum uses: Asset, Liability, Equity, Revenue, Expense
        let mut accounts_by_class: HashMap<&str, Vec<&str>> = HashMap::new();
        for account in accounts {
            let class_label = match account.account_type {
                datasynth_core::AccountType::Asset => "Assets",
                datasynth_core::AccountType::Liability => "Liabilities",
                datasynth_core::AccountType::Equity => "Equity",
                datasynth_core::AccountType::Revenue => "Revenue",
                datasynth_core::AccountType::Expense => "Expenses",
                datasynth_core::AccountType::Statistical => continue,
            };
            accounts_by_class
                .entry(class_label)
                .or_default()
                .push(&account.account_number);
        }

        for control in controls {
            let Some(control_id) = ctx.id_map.get(&control.control_id) else {
                continue;
            };
            for class_name in &control.covers_account_classes {
                if let Some(acct_numbers) = accounts_by_class.get(class_name.as_str()) {
                    for acct_ext_id in acct_numbers {
                        if let Some(acct_id) = ctx.id_map.get(acct_ext_id) {
                            edges.push(ExportEdge {
                                source: control_id,
                                target: acct_id,
                                edge_type: CONTROL_COVERS_ACCOUNT,
                                weight: 1.0,
                                properties: HashMap::new(),
                            });
                        }
                    }
                }
            }
        }

        debug!("CONTROL_COVERS_ACCOUNT: {} edges", edges.len());
        edges
    }

    /// Produce CONTROL_OWNED_BY edges (code 127): control -> employee.
    fn synthesize_control_owned_by(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        let controls = &ctx.ds_result.internal_controls;
        let mut edges = Vec::new();

        for control in controls {
            let Some(control_id) = ctx.id_map.get(&control.control_id) else {
                continue;
            };

            if let Some(ref emp_id) = control.owner_employee_id {
                if let Some(employee_id) = ctx.id_map.get(emp_id) {
                    edges.push(ExportEdge {
                        source: control_id,
                        target: employee_id,
                        edge_type: CONTROL_OWNED_BY,
                        weight: 1.0,
                        properties: HashMap::new(),
                    });
                }
            }
        }

        debug!("CONTROL_OWNED_BY: {} edges", edges.len());
        edges
    }

    /// Produce CONTROL_HAS_FINDING edges (code 128): finding -> control.
    fn synthesize_control_has_finding(
        &self,
        ctx: &mut EdgeSynthesisContext<'_>,
    ) -> Vec<ExportEdge> {
        let findings = &ctx.ds_result.audit.findings;
        let mut edges = Vec::new();

        for finding in findings {
            let Some(finding_id) = ctx.id_map.get(&finding.finding_ref) else {
                continue;
            };

            for control_ref in &finding.related_control_ids {
                if let Some(control_id) = ctx.id_map.get(control_ref) {
                    edges.push(ExportEdge {
                        source: finding_id,
                        target: control_id,
                        edge_type: CONTROL_HAS_FINDING,
                        weight: 1.0,
                        properties: HashMap::new(),
                    });
                }
            }
        }

        debug!("CONTROL_HAS_FINDING: {} edges", edges.len());
        edges
    }

    /// Produce WORKPAPER_TESTS_CONTROL edges (code 129): workpaper -> control.
    ///
    /// Workpapers don't have a direct control_id FK. Instead, we match
    /// workpapers to controls by looking at findings that link both:
    /// if a finding references a control AND is documented in a workpaper,
    /// we create a workpaper -> control edge.
    fn synthesize_workpaper_tests_control(
        &self,
        ctx: &mut EdgeSynthesisContext<'_>,
    ) -> Vec<ExportEdge> {
        let findings = &ctx.ds_result.audit.findings;
        let mut edges = Vec::new();
        // Deduplicate: (workpaper_ref, control_id) pairs
        let mut seen: std::collections::HashSet<(String, String)> =
            std::collections::HashSet::new();

        for finding in findings {
            let wp_id_str = match &finding.workpaper_id {
                Some(id) => id.clone(),
                None => continue,
            };

            let Some(workpaper_id) = ctx.id_map.get(&wp_id_str) else {
                continue;
            };

            for control_ref in &finding.related_control_ids {
                let pair = (wp_id_str.clone(), control_ref.clone());
                if seen.contains(&pair) {
                    continue;
                }
                seen.insert(pair);

                if let Some(control_id) = ctx.id_map.get(control_ref) {
                    edges.push(ExportEdge {
                        source: workpaper_id,
                        target: control_id,
                        edge_type: WORKPAPER_TESTS_CONTROL,
                        weight: 1.0,
                        properties: HashMap::new(),
                    });
                }
            }
        }

        debug!("WORKPAPER_TESTS_CONTROL: {} edges", edges.len());
        edges
    }

    /// Produce ControlCoversProcess edges (code 45): control -> PO or SO.
    ///
    /// Maps controls to process documents based on their SOX assertion:
    /// - P2P controls (C010, C011) -> purchase orders
    /// - O2C controls (C020, C021) -> sales orders
    fn synthesize_control_covers_process(
        &self,
        ctx: &mut EdgeSynthesisContext<'_>,
    ) -> Vec<ExportEdge> {
        if !ctx.config.edge_synthesis.cross_layer_edges {
            return Vec::new();
        }

        let controls = &ctx.ds_result.internal_controls;
        let purchase_orders = &ctx.ds_result.document_flows.purchase_orders;
        let sales_orders = &ctx.ds_result.document_flows.sales_orders;
        let mut edges = Vec::new();

        for control in controls {
            let Some(control_id) = ctx.id_map.get(&control.control_id) else {
                continue;
            };

            let domain = classify_control_domain(control);
            match domain {
                ControlDomain::Expenditure => {
                    // Link to purchase orders (L2 process nodes)
                    for po in purchase_orders {
                        if let Some(po_id) = ctx.id_map.get(&po.header.document_id) {
                            edges.push(ExportEdge {
                                source: control_id,
                                target: po_id,
                                edge_type: CONTROL_COVERS_PROCESS,
                                weight: 0.5,
                                properties: HashMap::new(),
                            });
                        }
                    }
                }
                ControlDomain::Revenue => {
                    // Link to sales orders (L2 process nodes)
                    for so in sales_orders {
                        if let Some(so_id) = ctx.id_map.get(&so.header.document_id) {
                            edges.push(ExportEdge {
                                source: control_id,
                                target: so_id,
                                edge_type: CONTROL_COVERS_PROCESS,
                                weight: 0.5,
                                properties: HashMap::new(),
                            });
                        }
                    }
                }
                _ => {
                    // Other domains don't get process-level coverage edges
                }
            }
        }

        debug!("CONTROL_COVERS_PROCESS: {} edges", edges.len());
        edges
    }
}

// ──────────────────────────── Domain Classification ────────────────────────

/// Broad domain categories for matching risks to controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ControlDomain {
    Revenue,
    Expenditure,
    Asset,
    Treasury,
    Reporting,
    General,
}

/// Classify a control into a broad domain based on its name and assertion.
fn classify_control_domain(
    control: &datasynth_core::models::internal_control::InternalControl,
) -> ControlDomain {
    let name_lower = control.control_name.to_lowercase();
    let desc_lower = control.description.to_lowercase();

    // Name-based classification (strongest signal)
    if name_lower.contains("revenue")
        || name_lower.contains("credit limit")
        || desc_lower.contains("asc 606")
        || desc_lower.contains("sales order")
    {
        return ControlDomain::Revenue;
    }
    if name_lower.contains("three-way match")
        || name_lower.contains("vendor")
        || name_lower.contains("purchase")
        || desc_lower.contains("purchase order")
    {
        return ControlDomain::Expenditure;
    }
    if name_lower.contains("fixed asset")
        || name_lower.contains("depreciation")
        || name_lower.contains("inventory")
    {
        return ControlDomain::Asset;
    }
    if name_lower.contains("cash") || name_lower.contains("bank") || name_lower.contains("treasury")
    {
        return ControlDomain::Treasury;
    }
    if name_lower.contains("journal entry")
        || name_lower.contains("reconciliation")
        || name_lower.contains("period close")
        || name_lower.contains("intercompany")
        || name_lower.contains("financial information")
    {
        return ControlDomain::Reporting;
    }

    // Assertion-based fallback
    use datasynth_core::models::internal_control::SoxAssertion;
    match control.sox_assertion {
        SoxAssertion::Existence => ControlDomain::Asset,
        SoxAssertion::Completeness => ControlDomain::Reporting,
        SoxAssertion::Valuation => ControlDomain::Reporting,
        SoxAssertion::RightsAndObligations => ControlDomain::Asset,
        SoxAssertion::PresentationAndDisclosure => ControlDomain::General,
    }
}

/// Classify a risk into a broad domain based on `account_or_process`.
fn classify_risk_domain(
    risk: &datasynth_core::models::audit::risk::RiskAssessment,
) -> ControlDomain {
    let aop = risk.account_or_process.to_lowercase();

    if aop.contains("revenue")
        || aop.contains("sales")
        || aop.contains("receivable")
        || aop.contains("customer")
    {
        return ControlDomain::Revenue;
    }
    if aop.contains("expenditure")
        || aop.contains("payable")
        || aop.contains("vendor")
        || aop.contains("purchase")
        || aop.contains("procurement")
    {
        return ControlDomain::Expenditure;
    }
    if aop.contains("asset")
        || aop.contains("inventory")
        || aop.contains("depreciation")
        || aop.contains("property")
    {
        return ControlDomain::Asset;
    }
    if aop.contains("cash")
        || aop.contains("bank")
        || aop.contains("treasury")
        || aop.contains("liquidity")
    {
        return ControlDomain::Treasury;
    }
    if aop.contains("financial statement")
        || aop.contains("reporting")
        || aop.contains("journal")
        || aop.contains("closing")
        || aop.contains("reconciliation")
    {
        return ControlDomain::Reporting;
    }

    ControlDomain::General
}

/// Check if a risk domain is compatible with a control domain.
fn domains_compatible(risk: &ControlDomain, control: &ControlDomain) -> bool {
    // Exact match is always compatible
    if risk == control {
        return true;
    }
    // General controls cover any domain
    if *control == ControlDomain::General {
        return true;
    }
    // General risks can be covered by any domain
    if *risk == ControlDomain::General {
        return true;
    }
    false
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::internal_control::{ControlType, InternalControl, SoxAssertion};

    #[test]
    fn classify_revenue_control() {
        let control = InternalControl::new(
            "C020",
            "Revenue Recognition Review",
            ControlType::Detective,
            "Review revenue",
        )
        .with_assertion(SoxAssertion::Valuation);
        assert_eq!(classify_control_domain(&control), ControlDomain::Revenue);
    }

    #[test]
    fn classify_expenditure_control() {
        let control = InternalControl::new(
            "C010",
            "Three-Way Match",
            ControlType::Preventive,
            "Match PO/GR/Invoice",
        )
        .with_assertion(SoxAssertion::Completeness);
        assert_eq!(
            classify_control_domain(&control),
            ControlDomain::Expenditure
        );
    }

    #[test]
    fn classify_treasury_control() {
        let control = InternalControl::new(
            "C001",
            "Cash Account Daily Review",
            ControlType::Detective,
            "Review cash",
        )
        .with_assertion(SoxAssertion::Existence);
        assert_eq!(classify_control_domain(&control), ControlDomain::Treasury);
    }

    #[test]
    fn classify_reporting_control() {
        let control = InternalControl::new(
            "C030",
            "GL Account Reconciliation",
            ControlType::Detective,
            "Reconcile GL",
        )
        .with_assertion(SoxAssertion::Completeness);
        assert_eq!(classify_control_domain(&control), ControlDomain::Reporting);
    }

    #[test]
    fn classify_asset_control() {
        let control = InternalControl::new(
            "C050",
            "Fixed Asset Addition Approval",
            ControlType::Preventive,
            "Approve assets",
        )
        .with_assertion(SoxAssertion::Existence);
        assert_eq!(classify_control_domain(&control), ControlDomain::Asset);
    }

    #[test]
    fn classify_general_fallback() {
        let control = InternalControl::new(
            "C070",
            "Code of Conduct and Ethics",
            ControlType::Preventive,
            "Ethics",
        )
        .with_assertion(SoxAssertion::PresentationAndDisclosure);
        assert_eq!(classify_control_domain(&control), ControlDomain::General);
    }

    #[test]
    fn domain_compatibility() {
        assert!(domains_compatible(
            &ControlDomain::Revenue,
            &ControlDomain::Revenue
        ));
        assert!(!domains_compatible(
            &ControlDomain::Revenue,
            &ControlDomain::Expenditure
        ));
        assert!(domains_compatible(
            &ControlDomain::Revenue,
            &ControlDomain::General
        ));
        assert!(domains_compatible(
            &ControlDomain::General,
            &ControlDomain::Revenue
        ));
        assert!(domains_compatible(
            &ControlDomain::General,
            &ControlDomain::General
        ));
    }
}
