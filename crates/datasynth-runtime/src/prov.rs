//! W3C PROV-JSON export for data lineage interoperability.
//!
//! Exports generation lineage in the W3C PROV-JSON format as defined by
//! <https://www.w3.org/Submission/2013/SUBM-prov-json-20130424/>.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::run_manifest::RunManifest;

/// A W3C PROV-JSON document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvDocument {
    /// Namespace prefixes.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub prefix: HashMap<String, String>,
    /// Entities (data artifacts).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub entity: HashMap<String, ProvEntity>,
    /// Activities (processes).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub activity: HashMap<String, ProvActivity>,
    /// Agents (software/people).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub agent: HashMap<String, ProvAgent>,
    /// Generation relationships.
    #[serde(
        default,
        rename = "wasGeneratedBy",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub was_generated_by: HashMap<String, ProvGeneration>,
    /// Usage relationships.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub used: HashMap<String, ProvUsage>,
    /// Attribution relationships.
    #[serde(
        default,
        rename = "wasAttributedTo",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub was_attributed_to: HashMap<String, ProvAttribution>,
    /// Derivation relationships.
    #[serde(
        default,
        rename = "wasDerivedFrom",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub was_derived_from: HashMap<String, ProvDerivation>,
}

/// A PROV entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvEntity {
    /// Entity type.
    #[serde(rename = "prov:type", skip_serializing_if = "Option::is_none")]
    pub prov_type: Option<String>,
    /// Entity label.
    #[serde(rename = "prov:label", skip_serializing_if = "Option::is_none")]
    pub prov_label: Option<String>,
    /// Additional attributes.
    #[serde(flatten)]
    pub attributes: HashMap<String, serde_json::Value>,
}

/// A PROV activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvActivity {
    /// Activity type.
    #[serde(rename = "prov:type", skip_serializing_if = "Option::is_none")]
    pub prov_type: Option<String>,
    /// Activity label.
    #[serde(rename = "prov:label", skip_serializing_if = "Option::is_none")]
    pub prov_label: Option<String>,
    /// Start time.
    #[serde(rename = "prov:startTime", skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    /// End time.
    #[serde(rename = "prov:endTime", skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
    /// Additional attributes.
    #[serde(flatten)]
    pub attributes: HashMap<String, serde_json::Value>,
}

/// A PROV agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvAgent {
    /// Agent type.
    #[serde(rename = "prov:type", skip_serializing_if = "Option::is_none")]
    pub prov_type: Option<String>,
    /// Agent label.
    #[serde(rename = "prov:label", skip_serializing_if = "Option::is_none")]
    pub prov_label: Option<String>,
    /// Additional attributes.
    #[serde(flatten)]
    pub attributes: HashMap<String, serde_json::Value>,
}

/// A PROV generation relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvGeneration {
    /// The generated entity.
    #[serde(rename = "prov:entity")]
    pub entity: String,
    /// The activity that generated it.
    #[serde(rename = "prov:activity")]
    pub activity: String,
}

/// A PROV usage relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvUsage {
    /// The activity that used the entity.
    #[serde(rename = "prov:activity")]
    pub activity: String,
    /// The entity that was used.
    #[serde(rename = "prov:entity")]
    pub entity: String,
}

/// A PROV attribution relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvAttribution {
    /// The entity that is attributed.
    #[serde(rename = "prov:entity")]
    pub entity: String,
    /// The agent it is attributed to.
    #[serde(rename = "prov:agent")]
    pub agent: String,
}

/// A PROV derivation relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvDerivation {
    /// The derived entity.
    #[serde(rename = "prov:generatedEntity")]
    pub generated_entity: String,
    /// The source entity.
    #[serde(rename = "prov:usedEntity")]
    pub used_entity: String,
}

/// Converts a RunManifest into a W3C PROV-JSON document.
pub fn manifest_to_prov(manifest: &RunManifest) -> ProvDocument {
    let mut doc = ProvDocument {
        prefix: HashMap::new(),
        entity: HashMap::new(),
        activity: HashMap::new(),
        agent: HashMap::new(),
        was_generated_by: HashMap::new(),
        used: HashMap::new(),
        was_attributed_to: HashMap::new(),
        was_derived_from: HashMap::new(),
    };

    // Prefixes
    doc.prefix
        .insert("dsf".to_string(), "https://datasynth.io/ns/".to_string());
    doc.prefix
        .insert("prov".to_string(), "http://www.w3.org/ns/prov#".to_string());

    let run_id = &manifest.run_id;

    // The generation run as an Activity
    let activity_id = format!("dsf:run/{}", run_id);
    doc.activity.insert(
        activity_id.clone(),
        ProvActivity {
            prov_type: Some("dsf:GenerationRun".to_string()),
            prov_label: Some(format!("DataSynth generation run {}", run_id)),
            start_time: Some(manifest.started_at.to_rfc3339()),
            end_time: manifest.completed_at.map(|t| t.to_rfc3339()),
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert(
                    "dsf:seed".to_string(),
                    serde_json::Value::Number(manifest.seed.into()),
                );
                attrs.insert(
                    "dsf:generatorVersion".to_string(),
                    serde_json::Value::String(manifest.generator_version.clone()),
                );
                attrs
            },
        },
    );

    // DataSynth as the Agent
    let agent_id = format!("dsf:agent/datasynth-{}", manifest.generator_version);
    doc.agent.insert(
        agent_id.clone(),
        ProvAgent {
            prov_type: Some("prov:SoftwareAgent".to_string()),
            prov_label: Some(format!("DataSynth v{}", manifest.generator_version)),
            attributes: HashMap::new(),
        },
    );

    // Config as an input Entity
    let config_entity_id = format!("dsf:config/{}", manifest.config_hash);
    doc.entity.insert(
        config_entity_id.clone(),
        ProvEntity {
            prov_type: Some("dsf:GeneratorConfig".to_string()),
            prov_label: Some("Generation configuration".to_string()),
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert(
                    "dsf:configHash".to_string(),
                    serde_json::Value::String(manifest.config_hash.clone()),
                );
                attrs
            },
        },
    );

    // Activity used the config
    doc.used.insert(
        format!("dsf:usage/{}/config", run_id),
        ProvUsage {
            activity: activity_id.clone(),
            entity: config_entity_id,
        },
    );

    // Each output file as an Entity
    for (i, file_info) in manifest.output_files.iter().enumerate() {
        let entity_id = format!("dsf:output/{}/{}", run_id, file_info.path.replace('/', "_"));
        let mut attrs = HashMap::new();
        attrs.insert(
            "dsf:format".to_string(),
            serde_json::Value::String(file_info.format.clone()),
        );
        if let Some(count) = file_info.record_count {
            attrs.insert("dsf:recordCount".to_string(), serde_json::json!(count));
        }
        if let Some(size) = file_info.size_bytes {
            attrs.insert("dsf:sizeBytes".to_string(), serde_json::json!(size));
        }
        if let Some(ref checksum) = file_info.sha256_checksum {
            attrs.insert(
                "dsf:sha256".to_string(),
                serde_json::Value::String(checksum.clone()),
            );
        }

        doc.entity.insert(
            entity_id.clone(),
            ProvEntity {
                prov_type: Some("dsf:OutputFile".to_string()),
                prov_label: Some(file_info.path.clone()),
                attributes: attrs,
            },
        );

        // wasGeneratedBy
        doc.was_generated_by.insert(
            format!("dsf:gen/{}/{}", run_id, i),
            ProvGeneration {
                entity: entity_id.clone(),
                activity: activity_id.clone(),
            },
        );

        // wasAttributedTo
        doc.was_attributed_to.insert(
            format!("dsf:attr/{}/{}", run_id, i),
            ProvAttribution {
                entity: entity_id,
                agent: agent_id.clone(),
            },
        );
    }

    doc
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::run_manifest::{OutputFileInfo, RunManifest};
    use datasynth_config::schema::*;

    fn create_test_manifest() -> RunManifest {
        let config = GeneratorConfig {
            global: GlobalConfig {
                industry: datasynth_core::models::IndustrySector::Manufacturing,
                start_date: "2024-01-01".to_string(),
                period_months: 1,
                seed: Some(42),
                parallel: false,
                group_currency: "USD".to_string(),
                worker_threads: 1,
                memory_limit_mb: 512,
                fiscal_year_months: None,
            },
            companies: vec![CompanyConfig {
                code: "TEST".to_string(),
                name: "Test Company".to_string(),
                currency: "USD".to_string(),
                country: "US".to_string(),
                annual_transaction_volume: TransactionVolume::TenK,
                volume_weight: 1.0,
                fiscal_year_variant: "K4".to_string(),
            }],
            chart_of_accounts: ChartOfAccountsConfig::default(),
            transactions: TransactionConfig::default(),
            output: OutputConfig::default(),
            fraud: FraudConfig::default(),
            internal_controls: InternalControlsConfig::default(),
            business_processes: BusinessProcessConfig::default(),
            user_personas: UserPersonaConfig::default(),
            templates: TemplateConfig::default(),
            approval: ApprovalConfig::default(),
            departments: DepartmentConfig::default(),
            master_data: MasterDataConfig::default(),
            document_flows: DocumentFlowConfig::default(),
            intercompany: IntercompanyConfig::default(),
            balance: BalanceConfig::default(),
            ocpm: OcpmConfig::default(),
            audit: AuditGenerationConfig::default(),
            banking: datasynth_banking::BankingConfig::default(),
            data_quality: DataQualitySchemaConfig::default(),
            scenario: ScenarioConfig::default(),
            temporal: TemporalDriftConfig::default(),
            graph_export: GraphExportConfig::default(),
            streaming: StreamingSchemaConfig::default(),
            rate_limit: RateLimitSchemaConfig::default(),
            temporal_attributes: TemporalAttributeSchemaConfig::default(),
            relationships: RelationshipSchemaConfig::default(),
            accounting_standards: AccountingStandardsConfig::default(),
            audit_standards: AuditStandardsConfig::default(),
            distributions: Default::default(),
            temporal_patterns: Default::default(),
            vendor_network: VendorNetworkSchemaConfig::default(),
            customer_segmentation: CustomerSegmentationSchemaConfig::default(),
            relationship_strength: RelationshipStrengthSchemaConfig::default(),
            cross_process_links: CrossProcessLinksSchemaConfig::default(),
            organizational_events: OrganizationalEventsSchemaConfig::default(),
            behavioral_drift: BehavioralDriftSchemaConfig::default(),
            market_drift: MarketDriftSchemaConfig::default(),
            drift_labeling: DriftLabelingSchemaConfig::default(),
            anomaly_injection: Default::default(),
            industry_specific: Default::default(),
            fingerprint_privacy: Default::default(),
            quality_gates: Default::default(),
            compliance: Default::default(),
            webhooks: Default::default(),
            llm: Default::default(),
            diffusion: Default::default(),
            causal: Default::default(),
            source_to_pay: Default::default(),
            financial_reporting: Default::default(),
            hr: Default::default(),
            manufacturing: Default::default(),
            sales_quotes: Default::default(),
            tax: Default::default(),
            treasury: Default::default(),
            project_accounting: Default::default(),
            esg: Default::default(),
            country_packs: None,
            scenarios: Default::default(),
            session: Default::default(),
        };

        let mut manifest = RunManifest::new(&config, 42);
        manifest.add_output_file(OutputFileInfo {
            path: "journal_entries.csv".to_string(),
            format: "csv".to_string(),
            record_count: Some(1000),
            size_bytes: Some(102400),
            sha256_checksum: Some("abc123".to_string()),
            first_record_index: Some(0),
            last_record_index: Some(999),
        });
        manifest.add_output_file(OutputFileInfo {
            path: "vendors.csv".to_string(),
            format: "csv".to_string(),
            record_count: Some(50),
            size_bytes: None,
            sha256_checksum: None,
            first_record_index: None,
            last_record_index: None,
        });
        manifest
    }

    #[test]
    fn test_manifest_to_prov_structure() {
        let manifest = create_test_manifest();
        let prov = manifest_to_prov(&manifest);

        // Should have prefixes
        assert!(prov.prefix.contains_key("dsf"));
        assert!(prov.prefix.contains_key("prov"));

        // Should have 1 activity (the run)
        assert_eq!(prov.activity.len(), 1);

        // Should have 1 agent (DataSynth)
        assert_eq!(prov.agent.len(), 1);

        // Should have 3 entities: config + 2 output files
        assert_eq!(prov.entity.len(), 3);

        // Each output file should have wasGeneratedBy
        assert_eq!(prov.was_generated_by.len(), 2);

        // Each output file should have wasAttributedTo
        assert_eq!(prov.was_attributed_to.len(), 2);

        // Config should be used
        assert_eq!(prov.used.len(), 1);
    }

    #[test]
    fn test_prov_json_roundtrip() {
        let manifest = create_test_manifest();
        let prov = manifest_to_prov(&manifest);

        let json = serde_json::to_string_pretty(&prov).expect("serialize");
        let deserialized: ProvDocument = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.entity.len(), prov.entity.len());
        assert_eq!(deserialized.activity.len(), prov.activity.len());
        assert_eq!(
            deserialized.was_generated_by.len(),
            prov.was_generated_by.len()
        );
    }

    #[test]
    fn test_all_output_files_have_was_generated_by() {
        let manifest = create_test_manifest();
        let prov = manifest_to_prov(&manifest);

        // Every output file entity should have a corresponding wasGeneratedBy
        let generated_entities: Vec<_> = prov
            .was_generated_by
            .values()
            .map(|g| g.entity.clone())
            .collect();

        for (id, entity) in &prov.entity {
            if entity.prov_type.as_deref() == Some("dsf:OutputFile") {
                assert!(
                    generated_entities.contains(id),
                    "Output file {} has no wasGeneratedBy",
                    id
                );
            }
        }
    }

    #[test]
    fn test_prov_checksum_included() {
        let manifest = create_test_manifest();
        let prov = manifest_to_prov(&manifest);

        // Find the journal_entries entity
        let je_entity = prov
            .entity
            .values()
            .find(|e| e.prov_label.as_deref() == Some("journal_entries.csv"))
            .expect("should find journal_entries entity");

        assert!(je_entity.attributes.contains_key("dsf:sha256"));
    }
}
