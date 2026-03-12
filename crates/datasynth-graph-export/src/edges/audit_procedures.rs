//! Audit procedure edge synthesizer.
//!
//! Produces edges linking the 9 audit procedure entity types (ISA 505/330/530/520/610/550)
//! to each other and to existing audit nodes (workpapers, engagements, evidence, accounts).
//!
//! ## Edge Types Produced
//!
//! | Code | Name                    | Direction                                  |
//! |------|-------------------------|--------------------------------------------|
//! | 138  | CONFIRMATION_FOR_ACCOUNT| confirmation -> gl_account                 |
//! | 139  | CONFIRMATION_RESPONSE   | response -> confirmation                   |
//! | 140  | CONFIRMATION_IN_WORKPAPER| confirmation -> workpaper                 |
//! | 141  | STEP_IN_WORKPAPER       | step -> workpaper                          |
//! | 142  | STEP_USES_SAMPLE        | step -> sample (if step.sample_id is Some) |
//! | 143  | STEP_EVIDENCE           | step -> evidence (for each evidence_id)    |
//! | 144  | SAMPLE_FROM_WORKPAPER   | sample -> workpaper                        |
//! | 145  | AP_FOR_ACCOUNT          | analytical -> account (if account_id Some) |
//! | 146  | AP_IN_WORKPAPER         | analytical -> workpaper (if workpaper_id)  |
//! | 147  | IAF_FOR_ENGAGEMENT      | ia_function -> engagement                  |
//! | 148  | REPORT_FROM_IAF         | ia_report -> ia_function                   |
//! | 149  | IA_REPORT_FOR_ENGAGEMENT| ia_report -> engagement                    |
//! | 150  | RP_FOR_ENGAGEMENT       | related_party -> engagement                |
//! | 151  | RPT_WITH_PARTY          | rp_transaction -> related_party            |
//! | 152  | RPT_JOURNAL_ENTRY       | rp_transaction -> journal_entry (if linked)|

use std::collections::HashMap;

use tracing::debug;

use crate::error::ExportError;
use crate::traits::EdgeSynthesisContext;
use crate::types::ExportEdge;

/// Edge type codes produced by this synthesizer.
const CONFIRMATION_FOR_ACCOUNT: u32 = 138;
const CONFIRMATION_RESPONSE: u32 = 139;
const CONFIRMATION_IN_WORKPAPER: u32 = 140;
const STEP_IN_WORKPAPER: u32 = 141;
const STEP_USES_SAMPLE: u32 = 142;
const STEP_EVIDENCE: u32 = 143;
const SAMPLE_FROM_WORKPAPER: u32 = 144;
const AP_FOR_ACCOUNT: u32 = 145;
const AP_IN_WORKPAPER: u32 = 146;
const IAF_FOR_ENGAGEMENT: u32 = 147;
const REPORT_FROM_IAF: u32 = 148;
const IA_REPORT_FOR_ENGAGEMENT: u32 = 149;
const RP_FOR_ENGAGEMENT: u32 = 150;
const RPT_WITH_PARTY: u32 = 151;
#[allow(dead_code)] // Reserved; RPT has no JE FK yet
const RPT_JOURNAL_ENTRY: u32 = 152;

/// Synthesizes edges for all audit procedure entity types (ISA 505/330/530/520/610/550).
pub struct AuditProcedureEdgeSynthesizer;

impl crate::traits::EdgeSynthesizer for AuditProcedureEdgeSynthesizer {
	fn name(&self) -> &'static str {
		"audit_procedures"
	}

	fn synthesize(
		&self,
		ctx: &mut EdgeSynthesisContext<'_>,
	) -> Result<Vec<ExportEdge>, ExportError> {
		let mut edges = Vec::new();

		edges.extend(self.synthesize_confirmation_for_account(ctx));
		edges.extend(self.synthesize_confirmation_response(ctx));
		edges.extend(self.synthesize_confirmation_in_workpaper(ctx));
		edges.extend(self.synthesize_step_in_workpaper(ctx));
		edges.extend(self.synthesize_step_uses_sample(ctx));
		edges.extend(self.synthesize_step_evidence(ctx));
		edges.extend(self.synthesize_sample_from_workpaper(ctx));
		edges.extend(self.synthesize_ap_for_account(ctx));
		edges.extend(self.synthesize_ap_in_workpaper(ctx));
		edges.extend(self.synthesize_iaf_for_engagement(ctx));
		edges.extend(self.synthesize_report_from_iaf(ctx));
		edges.extend(self.synthesize_ia_report_for_engagement(ctx));
		edges.extend(self.synthesize_rp_for_engagement(ctx));
		edges.extend(self.synthesize_rpt_with_party(ctx));
		// RPT_JOURNAL_ENTRY (152): RelatedPartyTransaction has no journal_entry FK,
		// so this produces 0 edges until a FK is added to the model.
		edges.extend(self.synthesize_rpt_journal_entry(ctx));

		debug!(
			"AuditProcedureEdgeSynthesizer produced {} total edges",
			edges.len()
		);
		Ok(edges)
	}
}

impl AuditProcedureEdgeSynthesizer {
	/// CONFIRMATION_FOR_ACCOUNT (138): confirmation -> gl_account.
	///
	/// Uses `confirmation.account_id` (Option<String>) which holds the GL account
	/// number directly (same key format as accounting edges).
	fn synthesize_confirmation_for_account(
		&self,
		ctx: &mut EdgeSynthesisContext<'_>,
	) -> Vec<ExportEdge> {
		let confirmations = &ctx.ds_result.audit.confirmations;
		let mut edges = Vec::new();

		for conf in confirmations {
			let Some(ref acct_id) = conf.account_id else {
				continue;
			};

			let conf_ext_id = format!("CONF-{}", conf.confirmation_id);
			let Some(conf_node_id) = ctx.id_map.get(&conf_ext_id) else {
				continue;
			};
			let Some(acct_node_id) = ctx.id_map.get(acct_id) else {
				continue;
			};

			edges.push(ExportEdge {
				source: conf_node_id,
				target: acct_node_id,
				edge_type: CONFIRMATION_FOR_ACCOUNT,
				weight: 1.0,
				properties: HashMap::new(),
			});
		}

		debug!("CONFIRMATION_FOR_ACCOUNT: {} edges", edges.len());
		edges
	}

	/// CONFIRMATION_RESPONSE (139): response -> confirmation.
	fn synthesize_confirmation_response(
		&self,
		ctx: &mut EdgeSynthesisContext<'_>,
	) -> Vec<ExportEdge> {
		let responses = &ctx.ds_result.audit.confirmation_responses;
		let mut edges = Vec::new();

		for resp in responses {
			let resp_ext_id = format!("RESP-{}", resp.response_id);
			let conf_ext_id = format!("CONF-{}", resp.confirmation_id);

			let Some(resp_node_id) = ctx.id_map.get(&resp_ext_id) else {
				continue;
			};
			let Some(conf_node_id) = ctx.id_map.get(&conf_ext_id) else {
				continue;
			};

			edges.push(ExportEdge {
				source: resp_node_id,
				target: conf_node_id,
				edge_type: CONFIRMATION_RESPONSE,
				weight: 1.0,
				properties: HashMap::new(),
			});
		}

		debug!("CONFIRMATION_RESPONSE: {} edges", edges.len());
		edges
	}

	/// CONFIRMATION_IN_WORKPAPER (140): confirmation -> workpaper.
	///
	/// Uses `confirmation.workpaper_id` (Option<Uuid>) to look up the workpaper
	/// by its UUID. Workpapers are keyed in the id_map by their `workpaper_ref`
	/// string, so we must first resolve `workpaper_id` -> `workpaper_ref`.
	fn synthesize_confirmation_in_workpaper(
		&self,
		ctx: &mut EdgeSynthesisContext<'_>,
	) -> Vec<ExportEdge> {
		let audit = &ctx.ds_result.audit;
		let mut edges = Vec::new();

		// Build workpaper_id (Uuid as String) -> workpaper_ref map
		let wp_id_to_ref: HashMap<String, &str> = audit
			.workpapers
			.iter()
			.map(|wp| (wp.workpaper_id.to_string(), wp.workpaper_ref.as_str()))
			.collect();

		for conf in &audit.confirmations {
			let Some(wp_uuid) = conf.workpaper_id else {
				continue;
			};

			let conf_ext_id = format!("CONF-{}", conf.confirmation_id);
			let Some(conf_node_id) = ctx.id_map.get(&conf_ext_id) else {
				continue;
			};

			let wp_uuid_str = wp_uuid.to_string();
			let Some(&wp_ref) = wp_id_to_ref.get(&wp_uuid_str) else {
				continue;
			};
			let Some(wp_node_id) = ctx.id_map.get(wp_ref) else {
				continue;
			};

			edges.push(ExportEdge {
				source: conf_node_id,
				target: wp_node_id,
				edge_type: CONFIRMATION_IN_WORKPAPER,
				weight: 1.0,
				properties: HashMap::new(),
			});
		}

		debug!("CONFIRMATION_IN_WORKPAPER: {} edges", edges.len());
		edges
	}

	/// STEP_IN_WORKPAPER (141): step -> workpaper.
	///
	/// `AuditProcedureStep.workpaper_id` is a Uuid; resolve via workpaper_ref.
	fn synthesize_step_in_workpaper(
		&self,
		ctx: &mut EdgeSynthesisContext<'_>,
	) -> Vec<ExportEdge> {
		let audit = &ctx.ds_result.audit;
		let mut edges = Vec::new();

		// Build workpaper_id (UUID string) -> workpaper_ref map
		let wp_id_to_ref: HashMap<String, &str> = audit
			.workpapers
			.iter()
			.map(|wp| (wp.workpaper_id.to_string(), wp.workpaper_ref.as_str()))
			.collect();

		for step in &audit.procedure_steps {
			let step_ext_id = format!("STEP-{}", step.step_id);
			let Some(step_node_id) = ctx.id_map.get(&step_ext_id) else {
				continue;
			};

			let wp_uuid_str = step.workpaper_id.to_string();
			let Some(&wp_ref) = wp_id_to_ref.get(&wp_uuid_str) else {
				continue;
			};
			let Some(wp_node_id) = ctx.id_map.get(wp_ref) else {
				continue;
			};

			edges.push(ExportEdge {
				source: step_node_id,
				target: wp_node_id,
				edge_type: STEP_IN_WORKPAPER,
				weight: 1.0,
				properties: HashMap::new(),
			});
		}

		debug!("STEP_IN_WORKPAPER: {} edges", edges.len());
		edges
	}

	/// STEP_USES_SAMPLE (142): step -> sample.
	///
	/// `step.sample_id` is `Option<Uuid>`; samples are keyed as `"SAMP-{uuid}"`.
	fn synthesize_step_uses_sample(
		&self,
		ctx: &mut EdgeSynthesisContext<'_>,
	) -> Vec<ExportEdge> {
		let steps = &ctx.ds_result.audit.procedure_steps;
		let mut edges = Vec::new();

		for step in steps {
			let Some(sample_uuid) = step.sample_id else {
				continue;
			};

			let step_ext_id = format!("STEP-{}", step.step_id);
			let Some(step_node_id) = ctx.id_map.get(&step_ext_id) else {
				continue;
			};

			let samp_ext_id = format!("SAMP-{sample_uuid}");
			let Some(samp_node_id) = ctx.id_map.get(&samp_ext_id) else {
				continue;
			};

			edges.push(ExportEdge {
				source: step_node_id,
				target: samp_node_id,
				edge_type: STEP_USES_SAMPLE,
				weight: 1.0,
				properties: HashMap::new(),
			});
		}

		debug!("STEP_USES_SAMPLE: {} edges", edges.len());
		edges
	}

	/// STEP_EVIDENCE (143): step -> evidence (one edge per evidence_id in step.evidence_ids).
	///
	/// `step.evidence_ids` is `Vec<Uuid>` where each UUID is the `evidence_id` of an
	/// Evidence record. Evidence is keyed in the id_map by `evidence_ref`, so we
	/// build an `evidence_id -> evidence_ref` lookup first.
	fn synthesize_step_evidence(
		&self,
		ctx: &mut EdgeSynthesisContext<'_>,
	) -> Vec<ExportEdge> {
		let audit = &ctx.ds_result.audit;
		let mut edges = Vec::new();

		// Build evidence_id (UUID string) -> evidence_ref map
		let ev_id_to_ref: HashMap<String, &str> = audit
			.evidence
			.iter()
			.map(|ev| (ev.evidence_id.to_string(), ev.evidence_ref.as_str()))
			.collect();

		for step in &audit.procedure_steps {
			if step.evidence_ids.is_empty() {
				continue;
			}

			let step_ext_id = format!("STEP-{}", step.step_id);
			let Some(step_node_id) = ctx.id_map.get(&step_ext_id) else {
				continue;
			};

			for ev_uuid in &step.evidence_ids {
				let ev_uuid_str = ev_uuid.to_string();
				let Some(&ev_ref) = ev_id_to_ref.get(&ev_uuid_str) else {
					continue;
				};
				let Some(ev_node_id) = ctx.id_map.get(ev_ref) else {
					continue;
				};

				edges.push(ExportEdge {
					source: step_node_id,
					target: ev_node_id,
					edge_type: STEP_EVIDENCE,
					weight: 1.0,
					properties: HashMap::new(),
				});
			}
		}

		debug!("STEP_EVIDENCE: {} edges", edges.len());
		edges
	}

	/// SAMPLE_FROM_WORKPAPER (144): sample -> workpaper.
	///
	/// `AuditSample.workpaper_id` is a Uuid; resolve via workpaper_ref.
	fn synthesize_sample_from_workpaper(
		&self,
		ctx: &mut EdgeSynthesisContext<'_>,
	) -> Vec<ExportEdge> {
		let audit = &ctx.ds_result.audit;
		let mut edges = Vec::new();

		// Build workpaper_id (UUID string) -> workpaper_ref map
		let wp_id_to_ref: HashMap<String, &str> = audit
			.workpapers
			.iter()
			.map(|wp| (wp.workpaper_id.to_string(), wp.workpaper_ref.as_str()))
			.collect();

		for sample in &audit.samples {
			let samp_ext_id = format!("SAMP-{}", sample.sample_id);
			let Some(samp_node_id) = ctx.id_map.get(&samp_ext_id) else {
				continue;
			};

			let wp_uuid_str = sample.workpaper_id.to_string();
			let Some(&wp_ref) = wp_id_to_ref.get(&wp_uuid_str) else {
				continue;
			};
			let Some(wp_node_id) = ctx.id_map.get(wp_ref) else {
				continue;
			};

			edges.push(ExportEdge {
				source: samp_node_id,
				target: wp_node_id,
				edge_type: SAMPLE_FROM_WORKPAPER,
				weight: 1.0,
				properties: HashMap::new(),
			});
		}

		debug!("SAMPLE_FROM_WORKPAPER: {} edges", edges.len());
		edges
	}

	/// AP_FOR_ACCOUNT (145): analytical_procedure_result -> gl_account.
	///
	/// Uses `ap.account_id` (Option<String>) which is the GL account number.
	fn synthesize_ap_for_account(
		&self,
		ctx: &mut EdgeSynthesisContext<'_>,
	) -> Vec<ExportEdge> {
		let results = &ctx.ds_result.audit.analytical_results;
		let mut edges = Vec::new();

		for ap in results {
			let Some(ref acct_id) = ap.account_id else {
				continue;
			};

			let ap_ext_id = format!("AP-{}", ap.result_id);
			let Some(ap_node_id) = ctx.id_map.get(&ap_ext_id) else {
				continue;
			};
			let Some(acct_node_id) = ctx.id_map.get(acct_id) else {
				continue;
			};

			edges.push(ExportEdge {
				source: ap_node_id,
				target: acct_node_id,
				edge_type: AP_FOR_ACCOUNT,
				weight: 1.0,
				properties: HashMap::new(),
			});
		}

		debug!("AP_FOR_ACCOUNT: {} edges", edges.len());
		edges
	}

	/// AP_IN_WORKPAPER (146): analytical_procedure_result -> workpaper.
	///
	/// Uses `ap.workpaper_id` (Option<Uuid>); resolve via workpaper_ref.
	fn synthesize_ap_in_workpaper(
		&self,
		ctx: &mut EdgeSynthesisContext<'_>,
	) -> Vec<ExportEdge> {
		let audit = &ctx.ds_result.audit;
		let mut edges = Vec::new();

		// Build workpaper_id (UUID string) -> workpaper_ref map
		let wp_id_to_ref: HashMap<String, &str> = audit
			.workpapers
			.iter()
			.map(|wp| (wp.workpaper_id.to_string(), wp.workpaper_ref.as_str()))
			.collect();

		for ap in &audit.analytical_results {
			let Some(wp_uuid) = ap.workpaper_id else {
				continue;
			};

			let ap_ext_id = format!("AP-{}", ap.result_id);
			let Some(ap_node_id) = ctx.id_map.get(&ap_ext_id) else {
				continue;
			};

			let wp_uuid_str = wp_uuid.to_string();
			let Some(&wp_ref) = wp_id_to_ref.get(&wp_uuid_str) else {
				continue;
			};
			let Some(wp_node_id) = ctx.id_map.get(wp_ref) else {
				continue;
			};

			edges.push(ExportEdge {
				source: ap_node_id,
				target: wp_node_id,
				edge_type: AP_IN_WORKPAPER,
				weight: 1.0,
				properties: HashMap::new(),
			});
		}

		debug!("AP_IN_WORKPAPER: {} edges", edges.len());
		edges
	}

	/// IAF_FOR_ENGAGEMENT (147): ia_function -> engagement.
	///
	/// Uses `iaf.engagement_id` (Uuid); engagements are keyed by `engagement_ref`.
	fn synthesize_iaf_for_engagement(
		&self,
		ctx: &mut EdgeSynthesisContext<'_>,
	) -> Vec<ExportEdge> {
		let audit = &ctx.ds_result.audit;
		let mut edges = Vec::new();

		// Build engagement_id (UUID string) -> engagement_ref map
		let eng_id_to_ref: HashMap<String, &str> = audit
			.engagements
			.iter()
			.map(|eng| (eng.engagement_id.to_string(), eng.engagement_ref.as_str()))
			.collect();

		for iaf in &audit.ia_functions {
			let iaf_ext_id = format!("IAF-{}", iaf.function_id);
			let Some(iaf_node_id) = ctx.id_map.get(&iaf_ext_id) else {
				continue;
			};

			let eng_uuid_str = iaf.engagement_id.to_string();
			let Some(&eng_ref) = eng_id_to_ref.get(&eng_uuid_str) else {
				continue;
			};
			let Some(eng_node_id) = ctx.id_map.get(eng_ref) else {
				continue;
			};

			edges.push(ExportEdge {
				source: iaf_node_id,
				target: eng_node_id,
				edge_type: IAF_FOR_ENGAGEMENT,
				weight: 1.0,
				properties: HashMap::new(),
			});
		}

		debug!("IAF_FOR_ENGAGEMENT: {} edges", edges.len());
		edges
	}

	/// REPORT_FROM_IAF (148): ia_report -> ia_function.
	///
	/// Uses `iar.ia_function_id` (Uuid); ia_functions are keyed as `"IAF-{uuid}"`.
	fn synthesize_report_from_iaf(
		&self,
		ctx: &mut EdgeSynthesisContext<'_>,
	) -> Vec<ExportEdge> {
		let reports = &ctx.ds_result.audit.ia_reports;
		let mut edges = Vec::new();

		for iar in reports {
			let iar_ext_id = format!("IAR-{}", iar.report_id);
			let Some(iar_node_id) = ctx.id_map.get(&iar_ext_id) else {
				continue;
			};

			let iaf_ext_id = format!("IAF-{}", iar.ia_function_id);
			let Some(iaf_node_id) = ctx.id_map.get(&iaf_ext_id) else {
				continue;
			};

			edges.push(ExportEdge {
				source: iar_node_id,
				target: iaf_node_id,
				edge_type: REPORT_FROM_IAF,
				weight: 1.0,
				properties: HashMap::new(),
			});
		}

		debug!("REPORT_FROM_IAF: {} edges", edges.len());
		edges
	}

	/// IA_REPORT_FOR_ENGAGEMENT (149): ia_report -> engagement.
	///
	/// Uses `iar.engagement_id` (Uuid); engagements are keyed by `engagement_ref`.
	fn synthesize_ia_report_for_engagement(
		&self,
		ctx: &mut EdgeSynthesisContext<'_>,
	) -> Vec<ExportEdge> {
		let audit = &ctx.ds_result.audit;
		let mut edges = Vec::new();

		// Build engagement_id (UUID string) -> engagement_ref map
		let eng_id_to_ref: HashMap<String, &str> = audit
			.engagements
			.iter()
			.map(|eng| (eng.engagement_id.to_string(), eng.engagement_ref.as_str()))
			.collect();

		for iar in &audit.ia_reports {
			let iar_ext_id = format!("IAR-{}", iar.report_id);
			let Some(iar_node_id) = ctx.id_map.get(&iar_ext_id) else {
				continue;
			};

			let eng_uuid_str = iar.engagement_id.to_string();
			let Some(&eng_ref) = eng_id_to_ref.get(&eng_uuid_str) else {
				continue;
			};
			let Some(eng_node_id) = ctx.id_map.get(eng_ref) else {
				continue;
			};

			edges.push(ExportEdge {
				source: iar_node_id,
				target: eng_node_id,
				edge_type: IA_REPORT_FOR_ENGAGEMENT,
				weight: 1.0,
				properties: HashMap::new(),
			});
		}

		debug!("IA_REPORT_FOR_ENGAGEMENT: {} edges", edges.len());
		edges
	}

	/// RP_FOR_ENGAGEMENT (150): related_party -> engagement.
	///
	/// Uses `rp.engagement_id` (Uuid); engagements are keyed by `engagement_ref`.
	fn synthesize_rp_for_engagement(
		&self,
		ctx: &mut EdgeSynthesisContext<'_>,
	) -> Vec<ExportEdge> {
		let audit = &ctx.ds_result.audit;
		let mut edges = Vec::new();

		// Build engagement_id (UUID string) -> engagement_ref map
		let eng_id_to_ref: HashMap<String, &str> = audit
			.engagements
			.iter()
			.map(|eng| (eng.engagement_id.to_string(), eng.engagement_ref.as_str()))
			.collect();

		for rp in &audit.related_parties {
			let rp_ext_id = format!("RP-{}", rp.party_id);
			let Some(rp_node_id) = ctx.id_map.get(&rp_ext_id) else {
				continue;
			};

			let eng_uuid_str = rp.engagement_id.to_string();
			let Some(&eng_ref) = eng_id_to_ref.get(&eng_uuid_str) else {
				continue;
			};
			let Some(eng_node_id) = ctx.id_map.get(eng_ref) else {
				continue;
			};

			edges.push(ExportEdge {
				source: rp_node_id,
				target: eng_node_id,
				edge_type: RP_FOR_ENGAGEMENT,
				weight: 1.0,
				properties: HashMap::new(),
			});
		}

		debug!("RP_FOR_ENGAGEMENT: {} edges", edges.len());
		edges
	}

	/// RPT_WITH_PARTY (151): rp_transaction -> related_party.
	///
	/// Uses `rpt.related_party_id` (Uuid); related_parties are keyed as `"RP-{uuid}"`.
	fn synthesize_rpt_with_party(
		&self,
		ctx: &mut EdgeSynthesisContext<'_>,
	) -> Vec<ExportEdge> {
		let transactions = &ctx.ds_result.audit.related_party_transactions;
		let mut edges = Vec::new();

		for rpt in transactions {
			let rpt_ext_id = format!("RPT-{}", rpt.transaction_id);
			let Some(rpt_node_id) = ctx.id_map.get(&rpt_ext_id) else {
				continue;
			};

			let rp_ext_id = format!("RP-{}", rpt.related_party_id);
			let Some(rp_node_id) = ctx.id_map.get(&rp_ext_id) else {
				continue;
			};

			edges.push(ExportEdge {
				source: rpt_node_id,
				target: rp_node_id,
				edge_type: RPT_WITH_PARTY,
				weight: 1.0,
				properties: HashMap::new(),
			});
		}

		debug!("RPT_WITH_PARTY: {} edges", edges.len());
		edges
	}

	/// RPT_JOURNAL_ENTRY (152): rp_transaction -> journal_entry.
	///
	/// `RelatedPartyTransaction` has no `journal_entry_id` FK in the current model,
	/// so this produces 0 edges until a FK is added.
	fn synthesize_rpt_journal_entry(
		&self,
		_ctx: &mut EdgeSynthesisContext<'_>,
	) -> Vec<ExportEdge> {
		// No journal_entry_id FK on RelatedPartyTransaction; placeholder for future FK.
		debug!("RPT_JOURNAL_ENTRY: 0 edges (no JE FK on RelatedPartyTransaction)");
		Vec::new()
	}
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
	use super::*;
	use crate::traits::EdgeSynthesizer;

	#[test]
	fn synthesizer_metadata() {
		let s = AuditProcedureEdgeSynthesizer;
		assert_eq!(s.name(), "audit_procedures");
	}

	#[test]
	fn edge_type_codes_are_distinct() {
		let codes = [
			CONFIRMATION_FOR_ACCOUNT,
			CONFIRMATION_RESPONSE,
			CONFIRMATION_IN_WORKPAPER,
			STEP_IN_WORKPAPER,
			STEP_USES_SAMPLE,
			STEP_EVIDENCE,
			SAMPLE_FROM_WORKPAPER,
			AP_FOR_ACCOUNT,
			AP_IN_WORKPAPER,
			IAF_FOR_ENGAGEMENT,
			REPORT_FROM_IAF,
			IA_REPORT_FOR_ENGAGEMENT,
			RP_FOR_ENGAGEMENT,
			RPT_WITH_PARTY,
			RPT_JOURNAL_ENTRY,
		];
		let mut seen = std::collections::HashSet::new();
		for &code in &codes {
			assert!(seen.insert(code), "Duplicate edge type code: {code}");
		}
		assert_eq!(seen.len(), 15);
	}

	#[test]
	fn edge_type_codes_in_range() {
		let codes = [
			CONFIRMATION_FOR_ACCOUNT,
			CONFIRMATION_RESPONSE,
			CONFIRMATION_IN_WORKPAPER,
			STEP_IN_WORKPAPER,
			STEP_USES_SAMPLE,
			STEP_EVIDENCE,
			SAMPLE_FROM_WORKPAPER,
			AP_FOR_ACCOUNT,
			AP_IN_WORKPAPER,
			IAF_FOR_ENGAGEMENT,
			REPORT_FROM_IAF,
			IA_REPORT_FOR_ENGAGEMENT,
			RP_FOR_ENGAGEMENT,
			RPT_WITH_PARTY,
			RPT_JOURNAL_ENTRY,
		];
		for &code in &codes {
			assert!(
				(138..=152).contains(&code),
				"Edge type code {code} outside expected range 138-152"
			);
		}
	}
}
