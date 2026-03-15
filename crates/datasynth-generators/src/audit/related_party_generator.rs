//! Related party generator per ISA 550.
//!
//! Generates related parties and their transactions for an audit engagement.
//! Related party transactions carry inherent risk and may indicate management
//! override; the generator models arm's length assessments and disclosure
//! adequacy to support realistic audit risk scenarios.

use datasynth_core::utils::seeded_rng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use uuid::Uuid;

/// Generate a UUID from the seeded RNG so output is fully deterministic.
fn rng_uuid(rng: &mut ChaCha8Rng) -> Uuid {
    let mut bytes = [0u8; 16];
    rng.fill(&mut bytes);
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

use datasynth_core::models::audit::{
    AuditEngagement, IdentificationSource, RelatedParty, RelatedPartyTransaction, RelatedPartyType,
    RelationshipBasis, RptTransactionType,
};

/// Configuration for related party and transaction generation (ISA 550).
#[derive(Debug, Clone)]
pub struct RelatedPartyGeneratorConfig {
    /// Number of related parties per engagement (min, max).
    pub parties_per_engagement: (u32, u32),
    /// Number of transactions per related party (min, max).
    pub transactions_per_party: (u32, u32),
    /// Fraction of parties that are NOT disclosed in the financial statements.
    pub undisclosed_party_ratio: f64,
    /// Fraction of transactions that are on arm's length terms.
    pub arms_length_ratio: f64,
    /// Fraction of transactions that pose a management override risk.
    pub management_override_risk_ratio: f64,
}

impl Default for RelatedPartyGeneratorConfig {
    fn default() -> Self {
        Self {
            parties_per_engagement: (3, 8),
            transactions_per_party: (1, 4),
            undisclosed_party_ratio: 0.10,
            arms_length_ratio: 0.70,
            management_override_risk_ratio: 0.15,
        }
    }
}

/// Generator for related parties and transactions per ISA 550.
pub struct RelatedPartyGenerator {
    /// Seeded random number generator.
    rng: ChaCha8Rng,
    /// Configuration.
    config: RelatedPartyGeneratorConfig,
}

impl RelatedPartyGenerator {
    /// Create a new generator with the given seed and default configuration.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config: RelatedPartyGeneratorConfig::default(),
        }
    }

    /// Create a new generator with custom configuration.
    pub fn with_config(seed: u64, config: RelatedPartyGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config,
        }
    }

    /// Generate related parties and their transactions.
    ///
    /// # Arguments
    /// * `engagement`  — The audit engagement these related parties belong to.
    /// * `vendor_names` — Vendor names available in the master data (used for
    ///   realistic party names). Pass an empty slice if none are available.
    /// * `customer_names` — Customer names available in the master data.
    ///
    /// Returns `(parties, transactions)`.
    pub fn generate(
        &mut self,
        engagement: &AuditEngagement,
        vendor_names: &[String],
        customer_names: &[String],
    ) -> (Vec<RelatedParty>, Vec<RelatedPartyTransaction>) {
        let party_count = self.rng.random_range(
            self.config.parties_per_engagement.0..=self.config.parties_per_engagement.1,
        ) as usize;

        let mut parties = Vec::with_capacity(party_count);
        let mut transactions = Vec::new();

        for i in 0..party_count {
            let party = self.generate_party(engagement, i, vendor_names, customer_names);
            let txns = self.generate_transactions(engagement, &party);

            transactions.extend(txns);
            parties.push(party);
        }

        (parties, transactions)
    }

    // -------------------------------------------------------------------------
    // Private helpers
    // -------------------------------------------------------------------------

    fn generate_party(
        &mut self,
        engagement: &AuditEngagement,
        index: usize,
        vendor_names: &[String],
        customer_names: &[String],
    ) -> RelatedParty {
        let party_type = self.pick_party_type(index);
        let relationship_basis = self.basis_for_type(party_type);

        // Pick a realistic name from the provided pools, or fall back to a
        // generated placeholder.
        let party_name = self.pick_party_name(party_type, vendor_names, customer_names, index);

        let mut party = RelatedParty::new(
            engagement.engagement_id,
            &party_name,
            party_type,
            relationship_basis,
        );
        // Override UUID for determinism.
        let party_id = rng_uuid(&mut self.rng);
        party.party_id = party_id;
        party.party_ref = format!("RP-{}", &party_id.simple().to_string()[..8]);

        // Ownership percentage for subsidiaries and associates.
        if matches!(
            party_type,
            RelatedPartyType::Subsidiary | RelatedPartyType::Associate
        ) {
            let pct = match party_type {
                RelatedPartyType::Subsidiary => self.rng.random_range(51.0_f64..=100.0_f64),
                RelatedPartyType::Associate => self.rng.random_range(20.0_f64..=49.9_f64),
                _ => unreachable!(),
            };
            party.ownership_percentage = Some((pct * 10.0).round() / 10.0);
        }

        // Board representation for subsidiaries, joint ventures, associates.
        party.board_representation = matches!(
            party_type,
            RelatedPartyType::Subsidiary
                | RelatedPartyType::JointVenture
                | RelatedPartyType::Associate
        ) && self.rng.random::<f64>() < 0.60;

        // Key management flag.
        party.key_management = matches!(
            party_type,
            RelatedPartyType::KeyManagement | RelatedPartyType::CloseFamily
        );

        // Disclosure.
        let undisclosed: bool = self.rng.random::<f64>() < self.config.undisclosed_party_ratio;
        party.disclosed_in_financials = !undisclosed;
        party.disclosure_adequate = if undisclosed {
            Some(false)
        } else if self.rng.random::<f64>() < 0.90 {
            Some(true)
        } else {
            None // under review
        };

        // Identification source.
        party.identified_by = self.pick_identification_source(undisclosed);

        party
    }

    fn generate_transactions(
        &mut self,
        engagement: &AuditEngagement,
        party: &RelatedParty,
    ) -> Vec<RelatedPartyTransaction> {
        let count = self.rng.random_range(
            self.config.transactions_per_party.0..=self.config.transactions_per_party.1,
        ) as usize;

        let mut txns = Vec::with_capacity(count);

        let fieldwork_days = (engagement.fieldwork_end - engagement.fieldwork_start)
            .num_days()
            .max(1);

        for _ in 0..count {
            let txn_type = self.pick_txn_type(party.party_type);
            let description = self.txn_description(txn_type, &party.party_name);

            // Amount: $50k–$5M.
            let amount_units: i64 = self.rng.random_range(50_000_i64..=5_000_000_i64);
            let amount = Decimal::new(amount_units * 100, 2); // cents → dollars

            let currency = self.pick_currency();

            // Transaction date within fieldwork window as proxy for period.
            let offset = self.rng.random_range(0_i64..fieldwork_days);
            let txn_date = engagement.fieldwork_start + chrono::Duration::days(offset);

            let mut txn = RelatedPartyTransaction::new(
                engagement.engagement_id,
                party.party_id,
                txn_type,
                &description,
                amount,
                currency,
                txn_date,
            );
            // Override UUID for determinism.
            let txn_id = rng_uuid(&mut self.rng);
            txn.transaction_id = txn_id;
            txn.transaction_ref = format!("RPT-{}", &txn_id.simple().to_string()[..8]);

            // Terms description.
            txn.terms_description = self.terms_description(txn_type);

            // Arm's length assessment.
            let is_arms_length: bool = self.rng.random::<f64>() < self.config.arms_length_ratio;
            txn.arms_length = Some(is_arms_length);
            if is_arms_length {
                txn.arms_length_evidence = Some(
					"Comparable uncontrolled price analysis performed; terms consistent with market."
						.to_string(),
				);
            }

            // Business rationale.
            txn.business_rationale = Some(self.business_rationale(txn_type));

            // Approval.
            txn.approved_by = Some(self.approver());

            // Management override risk.
            let override_risk: bool =
                self.rng.random::<f64>() < self.config.management_override_risk_ratio;
            txn.management_override_risk = override_risk;

            // Disclosure mirrors party-level disclosure.
            txn.disclosed_in_financials = party.disclosed_in_financials;
            txn.disclosure_adequate = party.disclosure_adequate;

            txns.push(txn);
        }

        txns
    }

    fn pick_party_type(&mut self, index: usize) -> RelatedPartyType {
        // Spread across types using index + random jitter to avoid clustering.
        let total = 8;
        let fraction = (index as f64 + self.rng.random::<f64>()) / total.max(1) as f64;
        if fraction < 0.25 {
            RelatedPartyType::Subsidiary
        } else if fraction < 0.35 {
            RelatedPartyType::Associate
        } else if fraction < 0.45 {
            RelatedPartyType::JointVenture
        } else if fraction < 0.60 {
            RelatedPartyType::KeyManagement
        } else if fraction < 0.70 {
            RelatedPartyType::CloseFamily
        } else if fraction < 0.80 {
            RelatedPartyType::ShareholderSignificant
        } else if fraction < 0.90 {
            RelatedPartyType::CommonDirector
        } else {
            RelatedPartyType::Other
        }
    }

    fn basis_for_type(&self, party_type: RelatedPartyType) -> RelationshipBasis {
        match party_type {
            RelatedPartyType::Subsidiary | RelatedPartyType::JointVenture => {
                RelationshipBasis::Ownership
            }
            RelatedPartyType::Associate => RelationshipBasis::SignificantInfluence,
            RelatedPartyType::KeyManagement => RelationshipBasis::KeyManagementPersonnel,
            RelatedPartyType::CloseFamily => RelationshipBasis::CloseFamily,
            RelatedPartyType::ShareholderSignificant => RelationshipBasis::SignificantInfluence,
            RelatedPartyType::CommonDirector => RelationshipBasis::Control,
            RelatedPartyType::Other => RelationshipBasis::Other,
        }
    }

    fn pick_party_name(
        &mut self,
        party_type: RelatedPartyType,
        vendor_names: &[String],
        customer_names: &[String],
        index: usize,
    ) -> String {
        match party_type {
            RelatedPartyType::KeyManagement | RelatedPartyType::CloseFamily => {
                // People names for key management / family.
                let names = [
                    "James Whitfield",
                    "Catherine Moore",
                    "Robert Park",
                    "Elena Vasquez",
                    "Andrew Campbell",
                    "Diane Fletcher",
                    "Marcus Osei",
                    "Natasha Brennan",
                ];
                let idx = self.rng.random_range(0..names.len());
                names[idx].to_string()
            }
            RelatedPartyType::Subsidiary | RelatedPartyType::Associate => {
                // Prefer customer names (subsidiaries are often end-market entities).
                if !customer_names.is_empty() {
                    let idx = self.rng.random_range(0..customer_names.len());
                    customer_names[idx].clone()
                } else {
                    format!("Subsidiary-{:03}", index + 1)
                }
            }
            _ => {
                // Prefer vendor names for other corporate relationships.
                if !vendor_names.is_empty() {
                    let idx = self.rng.random_range(0..vendor_names.len());
                    vendor_names[idx].clone()
                } else if !customer_names.is_empty() {
                    let idx = self.rng.random_range(0..customer_names.len());
                    customer_names[idx].clone()
                } else {
                    format!("Entity-{:03}", index + 1)
                }
            }
        }
    }

    fn pick_txn_type(&mut self, party_type: RelatedPartyType) -> RptTransactionType {
        match party_type {
            RelatedPartyType::Subsidiary | RelatedPartyType::Associate => {
                let roll: f64 = self.rng.random();
                if roll < 0.30 {
                    RptTransactionType::ManagementFee
                } else if roll < 0.55 {
                    RptTransactionType::Sale
                } else if roll < 0.70 {
                    RptTransactionType::Dividend
                } else if roll < 0.85 {
                    RptTransactionType::Loan
                } else {
                    RptTransactionType::ServiceAgreement
                }
            }
            RelatedPartyType::KeyManagement => {
                let roll: f64 = self.rng.random();
                if roll < 0.50 {
                    RptTransactionType::Loan
                } else if roll < 0.75 {
                    RptTransactionType::Lease
                } else {
                    RptTransactionType::Other
                }
            }
            RelatedPartyType::JointVenture => {
                let roll: f64 = self.rng.random();
                if roll < 0.40 {
                    RptTransactionType::CapitalContribution
                } else if roll < 0.70 {
                    RptTransactionType::ServiceAgreement
                } else {
                    RptTransactionType::Purchase
                }
            }
            _ => {
                // Generic distribution for other types.
                let txn_types = [
                    RptTransactionType::Sale,
                    RptTransactionType::Purchase,
                    RptTransactionType::Lease,
                    RptTransactionType::ManagementFee,
                    RptTransactionType::LicenseRoyalty,
                    RptTransactionType::ServiceAgreement,
                    RptTransactionType::Transfer,
                    RptTransactionType::Guarantee,
                ];
                let idx = self.rng.random_range(0..txn_types.len());
                txn_types[idx]
            }
        }
    }

    fn txn_description(&self, txn_type: RptTransactionType, party_name: &str) -> String {
        let verb = match txn_type {
            RptTransactionType::Sale => "Sale of goods/services to",
            RptTransactionType::Purchase => "Purchase of goods/services from",
            RptTransactionType::Lease => "Lease of property to/from",
            RptTransactionType::Loan => "Intercompany loan to/from",
            RptTransactionType::Guarantee => "Guarantee provided for",
            RptTransactionType::ManagementFee => "Management fee charged to/from",
            RptTransactionType::Dividend => "Dividend paid/received from",
            RptTransactionType::Transfer => "Asset transfer to/from",
            RptTransactionType::ServiceAgreement => "Shared services agreement with",
            RptTransactionType::LicenseRoyalty => "License/royalty arrangement with",
            RptTransactionType::CapitalContribution => "Capital contribution to",
            RptTransactionType::Other => "Transaction with",
        };
        format!("{} {}", verb, party_name)
    }

    fn terms_description(&self, txn_type: RptTransactionType) -> String {
        match txn_type {
            RptTransactionType::Loan => {
                "Fixed interest rate loan; repayable on demand or within 12 months.".to_string()
            }
            RptTransactionType::ManagementFee => {
                "Annual management fee based on cost-plus 5% mark-up.".to_string()
            }
            RptTransactionType::Lease => {
                "Operating lease at market rental rate reviewed annually.".to_string()
            }
            RptTransactionType::Dividend => {
                "Declared and paid in accordance with the shareholder agreement.".to_string()
            }
            _ => "Terms agreed between parties; documented in a formal agreement.".to_string(),
        }
    }

    fn pick_currency(&mut self) -> &'static str {
        let currencies = ["USD", "GBP", "EUR", "CAD", "AUD", "JPY", "CHF"];
        let idx = self.rng.random_range(0..currencies.len());
        currencies[idx]
    }

    fn business_rationale(&self, txn_type: RptTransactionType) -> String {
        match txn_type {
			RptTransactionType::ManagementFee => {
				"Centralised group services to achieve economies of scale.".to_string()
			}
			RptTransactionType::Loan => {
				"Intercompany financing to fund subsidiary working capital requirements.".to_string()
			}
			RptTransactionType::Sale | RptTransactionType::Purchase => {
				"Preferential group pricing for goods/services; arms length market comparison performed.".to_string()
			}
			RptTransactionType::Dividend => {
				"Routine return of capital from subsidiary in accordance with group dividend policy.".to_string()
			}
			RptTransactionType::Lease => {
				"Group property rationalisation strategy; lease terms consistent with market.".to_string()
			}
			_ => "Transaction supports group business objectives and is documented in the group policy.".to_string(),
		}
    }

    fn approver(&mut self) -> String {
        let approvers = [
            "Audit Committee",
            "Board of Directors",
            "Chief Financial Officer",
            "Risk Committee",
            "Remuneration Committee",
        ];
        let idx = self.rng.random_range(0..approvers.len());
        approvers[idx].to_string()
    }

    fn pick_identification_source(&mut self, undisclosed: bool) -> IdentificationSource {
        if undisclosed {
            // Undisclosed parties are more likely found via auditor inquiry or public records.
            let roll: f64 = self.rng.random();
            if roll < 0.40 {
                IdentificationSource::AuditorInquiry
            } else if roll < 0.65 {
                IdentificationSource::PublicRecords
            } else if roll < 0.80 {
                IdentificationSource::BankConfirmation
            } else if roll < 0.92 {
                IdentificationSource::LegalReview
            } else {
                IdentificationSource::WhistleblowerTip
            }
        } else {
            // Most disclosed parties are identified by management disclosure.
            if self.rng.random::<f64>() < 0.85 {
                IdentificationSource::ManagementDisclosure
            } else {
                IdentificationSource::AuditorInquiry
            }
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::audit::test_helpers::create_test_engagement;

    fn make_gen(seed: u64) -> RelatedPartyGenerator {
        RelatedPartyGenerator::new(seed)
    }

    fn empty_names() -> Vec<String> {
        Vec::new()
    }

    fn sample_vendor_names() -> Vec<String> {
        vec![
            "Allied Components GmbH".to_string(),
            "BestSource Procurement".to_string(),
        ]
    }

    fn sample_customer_names() -> Vec<String> {
        vec![
            "Acme Industries Ltd".to_string(),
            "Beacon Holdings PLC".to_string(),
        ]
    }

    // -------------------------------------------------------------------------

    /// Party count is within the configured range.
    #[test]
    fn test_generates_parties() {
        let engagement = create_test_engagement();
        let mut gen = make_gen(42);
        let (parties, _) = gen.generate(&engagement, &empty_names(), &empty_names());

        let min = RelatedPartyGeneratorConfig::default()
            .parties_per_engagement
            .0 as usize;
        let max = RelatedPartyGeneratorConfig::default()
            .parties_per_engagement
            .1 as usize;
        assert!(
            parties.len() >= min && parties.len() <= max,
            "expected {min}..={max} parties, got {}",
            parties.len()
        );
    }

    /// Every party has at least one transaction.
    #[test]
    fn test_generates_transactions() {
        let engagement = create_test_engagement();
        let mut gen = make_gen(7);
        let (parties, transactions) = gen.generate(
            &engagement,
            &sample_vendor_names(),
            &sample_customer_names(),
        );

        assert!(
            !transactions.is_empty(),
            "should generate at least one transaction"
        );
        // Every transaction's related_party_id should match a known party.
        let party_ids: std::collections::HashSet<_> = parties.iter().map(|p| p.party_id).collect();
        for txn in &transactions {
            assert!(
                party_ids.contains(&txn.related_party_id),
                "transaction {} references unknown party {}",
                txn.transaction_ref,
                txn.related_party_id
            );
        }
    }

    /// With undisclosed ratio = 1.0, all parties must be undisclosed.
    #[test]
    fn test_undisclosed_ratio() {
        let engagement = create_test_engagement();
        let config = RelatedPartyGeneratorConfig {
            undisclosed_party_ratio: 1.0,
            ..Default::default()
        };
        let mut gen = RelatedPartyGenerator::with_config(11, config);
        let (parties, _) = gen.generate(&engagement, &empty_names(), &empty_names());

        for party in &parties {
            assert!(
                !party.disclosed_in_financials,
                "party '{}' should be undisclosed",
                party.party_name
            );
        }
    }

    /// With arms_length_ratio = 1.0, all transactions must be flagged arms length.
    #[test]
    fn test_arms_length_ratio() {
        let engagement = create_test_engagement();
        let config = RelatedPartyGeneratorConfig {
            arms_length_ratio: 1.0,
            ..Default::default()
        };
        let mut gen = RelatedPartyGenerator::with_config(22, config);
        let (_, transactions) = gen.generate(&engagement, &empty_names(), &empty_names());

        for txn in &transactions {
            assert_eq!(
                txn.arms_length,
                Some(true),
                "transaction '{}' should be arms length",
                txn.transaction_ref
            );
        }
    }

    /// With management_override_risk_ratio = 1.0, all transactions carry the risk.
    #[test]
    fn test_management_override() {
        let engagement = create_test_engagement();
        let config = RelatedPartyGeneratorConfig {
            management_override_risk_ratio: 1.0,
            ..Default::default()
        };
        let mut gen = RelatedPartyGenerator::with_config(33, config);
        let (_, transactions) = gen.generate(&engagement, &empty_names(), &empty_names());

        for txn in &transactions {
            assert!(
                txn.management_override_risk,
                "transaction '{}' should carry management override risk",
                txn.transaction_ref
            );
        }
    }

    /// Same seed produces identical output.
    #[test]
    fn test_deterministic() {
        let engagement = create_test_engagement();
        let vendors = sample_vendor_names();
        let customers = sample_customer_names();

        let (parties_a, txns_a) = {
            let mut gen = make_gen(777);
            gen.generate(&engagement, &vendors, &customers)
        };
        let (parties_b, txns_b) = {
            let mut gen = make_gen(777);
            gen.generate(&engagement, &vendors, &customers)
        };

        assert_eq!(parties_a.len(), parties_b.len());
        assert_eq!(txns_a.len(), txns_b.len());
        for (a, b) in parties_a.iter().zip(parties_b.iter()) {
            assert_eq!(a.party_ref, b.party_ref);
            assert_eq!(a.party_name, b.party_name);
            assert_eq!(a.party_type, b.party_type);
            assert_eq!(a.disclosed_in_financials, b.disclosed_in_financials);
        }
        for (a, b) in txns_a.iter().zip(txns_b.iter()) {
            assert_eq!(a.transaction_ref, b.transaction_ref);
            assert_eq!(a.amount, b.amount);
            assert_eq!(a.transaction_type, b.transaction_type);
        }
    }
}
