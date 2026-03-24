//! Blueprint and overlay schema types.
//!
//! Provides Rust types for deserializing audit methodology YAML blueprints
//! and generation overlay YAML files.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Depth of coverage requested for this audit methodology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DepthLevel {
    /// Condensed procedure set — minimum viable audit evidence.
    Simplified,
    /// Balanced procedure set — typical engagement.
    #[default]
    Standard,
    /// Exhaustive procedure set — full ISA/PCAOB compliance.
    Full,
}

/// Whether a blueprint element is a mandatory requirement or optional guidance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BindingLevel {
    /// Mandatory — non-compliance constitutes a deficiency.
    #[default]
    Requirement,
    /// Advisory — may be skipped based on actor profile or risk.
    Guidance,
}

// ---------------------------------------------------------------------------
// Blueprint top-level
// ---------------------------------------------------------------------------

/// Root document for an audit methodology YAML blueprint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditBlueprint {
    /// Unique identifier for this blueprint (e.g. `isa-standard-2024`).
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Semver version string.
    pub version: String,
    /// Methodology metadata.
    pub methodology: BlueprintMethodology,
    /// Actor roles defined in this blueprint.
    #[serde(default)]
    pub actors: Vec<BlueprintActor>,
    /// Standards referenced throughout the blueprint.
    #[serde(default)]
    pub standards: Vec<BlueprintStandard>,
    /// Shared evidence templates that procedures may reference.
    #[serde(default)]
    pub evidence_templates: Vec<BlueprintEvidence>,
    /// Ordered list of audit phases.
    #[serde(default)]
    pub phases: Vec<BlueprintPhase>,
}

/// High-level metadata about the audit methodology.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintMethodology {
    /// Short code (e.g. `ISA`, `IIA-GIAS`, `PCAOB`).
    pub framework: String,
    /// Default depth applied when no overlay overrides it.
    #[serde(default)]
    pub default_depth: DepthLevel,
    /// Optional free-text description.
    #[serde(default)]
    pub description: Option<String>,
}

/// An actor role that participates in the audit engagement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintActor {
    /// Machine-readable role identifier (e.g. `senior_auditor`).
    pub id: String,
    /// Human-readable label.
    pub label: String,
    /// Optional description of responsibilities.
    #[serde(default)]
    pub description: Option<String>,
}

/// A standards reference used within the blueprint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintStandard {
    /// Identifier (e.g. `ISA-315`).
    pub id: String,
    /// Full title.
    pub title: String,
    /// Whether compliance is mandatory or advisory.
    #[serde(default)]
    pub binding: BindingLevel,
}

/// A reusable evidence template that steps may reference by id.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintEvidence {
    /// Template identifier.
    pub id: String,
    /// Evidence type label (e.g. `workpaper`, `confirmation`, `analytical`).
    #[serde(rename = "type")]
    pub evidence_type: String,
    /// Human-readable description.
    #[serde(default)]
    pub description: Option<String>,
    /// Whether this evidence item is mandatory.
    #[serde(default)]
    pub required: bool,
}

// ---------------------------------------------------------------------------
// Phase
// ---------------------------------------------------------------------------

/// A high-level audit phase (e.g. Planning, Fieldwork, Reporting).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintPhase {
    /// Machine-readable phase identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Optional description.
    #[serde(default)]
    pub description: Option<String>,
    /// Gate that must be satisfied before entering this phase.
    #[serde(default)]
    pub entry_gate: Option<PhaseGate>,
    /// Gate that must be satisfied before exiting this phase.
    #[serde(default)]
    pub exit_gate: Option<PhaseGate>,
    /// Ordered list of procedures within this phase.
    #[serde(default)]
    pub procedures: Vec<BlueprintProcedure>,
}

/// A gate condition set that controls phase entry/exit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseGate {
    /// All of these conditions must be true (AND semantics).
    #[serde(default)]
    pub all_of: Vec<GateCondition>,
    /// At least one of these conditions must be true (OR semantics).
    #[serde(default)]
    pub any_of: Vec<GateCondition>,
}

/// A single gate predicate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateCondition {
    /// Predicate expression (e.g. `procedure.risk_assessment.complete`).
    pub predicate: String,
    /// Optional human-readable description of why this gate exists.
    #[serde(default)]
    pub rationale: Option<String>,
}

// ---------------------------------------------------------------------------
// Procedure
// ---------------------------------------------------------------------------

/// An audit procedure within a phase (maps to one FSM node cluster).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintProcedure {
    /// Machine-readable procedure identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Optional description.
    #[serde(default)]
    pub description: Option<String>,
    /// FSM aggregate: initial state, valid states, and transitions.
    #[serde(default)]
    pub aggregate: ProcedureAggregate,
    /// Ordered steps within this procedure.
    #[serde(default)]
    pub steps: Vec<BlueprintStep>,
}

/// Per-procedure FSM aggregate: defines the states and transitions of the
/// procedure's internal state machine as declared in the YAML blueprint.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProcedureAggregate {
    /// The state the procedure starts in (e.g. `not_started`).
    #[serde(default)]
    pub initial_state: String,
    /// All valid states for this procedure's FSM.
    #[serde(default)]
    pub states: Vec<String>,
    /// Directed transitions between states.
    #[serde(default)]
    pub transitions: Vec<ProcedureTransition>,
}

/// A directed FSM transition within a procedure aggregate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureTransition {
    /// Source state.
    pub from_state: String,
    /// Destination state.
    pub to_state: String,
    /// Command that triggers this transition.
    #[serde(default)]
    pub command: Option<String>,
    /// Event name emitted when the transition fires.
    #[serde(default)]
    pub emits: Option<String>,
    /// Guard predicates that must pass before the transition can fire.
    #[serde(default)]
    pub guards: Vec<String>,
}

// ---------------------------------------------------------------------------
// Step
// ---------------------------------------------------------------------------

/// A single atomic step within a procedure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintStep {
    /// Machine-readable step identifier (unique within its procedure).
    pub id: String,
    /// Display name.
    pub name: String,
    /// Optional description.
    #[serde(default)]
    pub description: Option<String>,
    /// Actor role responsible for performing this step.
    #[serde(default)]
    pub actor: Option<String>,
    /// Command that this step executes.
    #[serde(default)]
    pub command: Option<String>,
    /// Event name emitted when this step completes.
    #[serde(default)]
    pub emits: Option<String>,
    /// Whether this step is mandatory or advisory.
    #[serde(default)]
    pub binding: BindingLevel,
    /// Guards that must pass before this step can execute.
    #[serde(default)]
    pub guards: Vec<StepGuard>,
    /// Evidence items produced or consumed by this step.
    #[serde(default)]
    pub evidence: Vec<StepEvidence>,
    /// Standards citations relevant to this step.
    #[serde(default)]
    pub standards: Vec<StepStandard>,
    /// Optional decision branch emitted after the step completes.
    #[serde(default)]
    pub decision: Option<StepDecision>,
}

/// A guard predicate that blocks step execution until satisfied.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepGuard {
    /// Guard type keyword (e.g. `state`, `evidence_present`, `actor_available`).
    #[serde(rename = "type")]
    pub guard_type: String,
    /// Expression evaluated against the current context.
    pub expression: String,
    /// Human-readable reason shown when the guard fails.
    #[serde(default)]
    pub failure_message: Option<String>,
}

/// Evidence produced or consumed by a step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepEvidence {
    /// Reference to a shared evidence template.
    #[serde(default)]
    pub template_ref: Option<EvidenceRef>,
    /// Inline evidence type when no shared template is used.
    #[serde(rename = "type", default)]
    pub evidence_type: Option<String>,
    /// Whether this evidence item must be present for the step to be complete.
    #[serde(default)]
    pub required: bool,
    /// Direction: `produces` or `consumes`.
    #[serde(default = "default_produces")]
    pub direction: String,
}

fn default_produces() -> String {
    "produces".to_string()
}

/// A reference to a shared evidence template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRef {
    /// The `id` of the referenced `BlueprintEvidence` entry.
    #[serde(rename = "ref")]
    pub ref_id: String,
}

/// A standards citation attached to a step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepStandard {
    /// The `id` of the referenced `BlueprintStandard`.
    #[serde(rename = "ref")]
    pub ref_id: String,
    /// Specific paragraph or requirement within the standard.
    #[serde(default)]
    pub paragraph: Option<String>,
}

/// A decision point emitted after a step, routing to different next steps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepDecision {
    /// Human-readable question posed by this decision.
    pub question: String,
    /// Possible outcomes of the decision.
    #[serde(default)]
    pub branches: Vec<DecisionBranch>,
}

/// One branch of a step-level decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionBranch {
    /// Branch label (e.g. `yes`, `no`, `escalate`).
    pub label: String,
    /// Target step id or terminal keyword to transition to.
    pub target: String,
    /// Optional condition expression for this branch.
    #[serde(default)]
    pub condition: Option<String>,
}

// ---------------------------------------------------------------------------
// Overlay types
// ---------------------------------------------------------------------------

/// Generation overlay that customises how a blueprint is instantiated.
///
/// Overlays are applied on top of a blueprint to tune probabilities, volumes,
/// timing distributions, and anomaly injection rates without modifying the
/// canonical blueprint YAML.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GenerationOverlay {
    /// Id of the blueprint this overlay targets.
    #[serde(default)]
    pub blueprint_id: Option<String>,
    /// Depth override (overrides `methodology.default_depth`).
    #[serde(default)]
    pub depth: Option<DepthLevel>,
    /// Transition probability and timing configuration.
    #[serde(default)]
    pub transitions: TransitionConfig,
    /// Artifact volume configuration.
    #[serde(default)]
    pub artifacts: ArtifactConfig,
    /// Anomaly injection configuration.
    #[serde(default)]
    pub anomalies: AnomalyConfig,
    /// Per-actor behavioural profiles keyed by actor id.
    #[serde(default)]
    pub actor_profiles: HashMap<String, ActorProfile>,
}

/// Transition probability and timing overrides.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransitionConfig {
    /// Global defaults applied to all transitions not explicitly overridden.
    #[serde(default)]
    pub defaults: TransitionDefaults,
    /// Per-transition overrides keyed by `"<from_procedure_id>-><target>"`.
    #[serde(default)]
    pub overrides: HashMap<String, TransitionOverride>,
}

/// Default timing and probability values for all transitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionDefaults {
    /// Probability [0, 1] that a completed step is revised before the transition fires.
    pub revision_probability: f64,
    /// Timing distribution for the transition delay.
    pub timing: TimingDistribution,
}

impl Default for TransitionDefaults {
    fn default() -> Self {
        Self {
            revision_probability: 0.15,
            timing: TimingDistribution {
                mu_hours: 24.0,
                sigma_hours: 8.0,
            },
        }
    }
}

/// Override for a specific transition edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionOverride {
    /// Override the revision probability for this specific transition.
    #[serde(default)]
    pub revision_probability: Option<f64>,
    /// Override the timing distribution for this specific transition.
    #[serde(default)]
    pub timing: Option<TimingDistribution>,
}

/// Log-normal timing distribution parameters (hours).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingDistribution {
    /// Mean delay in hours.
    pub mu_hours: f64,
    /// Standard deviation in hours.
    pub sigma_hours: f64,
}

/// Artifact volume configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactConfig {
    /// Number of workpapers generated per step.
    pub workpapers_per_step: VolumeRange,
    /// Number of evidence items attached to each workpaper.
    pub evidence_items_per_workpaper: VolumeRange,
}

impl Default for ArtifactConfig {
    fn default() -> Self {
        Self {
            workpapers_per_step: VolumeRange { min: 1, max: 3 },
            evidence_items_per_workpaper: VolumeRange { min: 2, max: 5 },
        }
    }
}

/// An inclusive integer range [min, max].
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VolumeRange {
    pub min: u32,
    pub max: u32,
}

/// Anomaly injection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyConfig {
    /// Probability that an approval step is skipped.
    pub skipped_approval: f64,
    /// Probability that a posting arrives late.
    pub late_posting: f64,
    /// Maximum late-posting delay in hours.
    pub max_delay_hours: f64,
    /// Probability that required evidence is missing.
    pub missing_evidence: f64,
    /// Probability that a step occurs out of the defined sequence.
    pub out_of_sequence: f64,
    /// Additional fine-grained anomaly rules.
    #[serde(default)]
    pub rules: Vec<AnomalyRule>,
}

impl Default for AnomalyConfig {
    fn default() -> Self {
        Self {
            skipped_approval: 0.02,
            late_posting: 0.05,
            max_delay_hours: 72.0,
            missing_evidence: 0.03,
            out_of_sequence: 0.01,
            rules: Vec::new(),
        }
    }
}

/// A fine-grained anomaly rule targeting a specific procedure or step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyRule {
    /// Anomaly type keyword (e.g. `skipped_approval`, `duplicate_evidence`).
    pub anomaly_type: String,
    /// Optional procedure id to scope the rule.
    #[serde(default)]
    pub procedure_id: Option<String>,
    /// Optional step id to scope the rule.
    #[serde(default)]
    pub step_id: Option<String>,
    /// Injection probability [0, 1].
    pub probability: f64,
}

/// Behavioural profile for a specific actor role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorProfile {
    /// Multiplier applied to the base revision probability for this actor.
    pub revision_multiplier: f64,
    /// Multiplier applied to the base evidence volume for this actor.
    pub evidence_multiplier: f64,
    /// When true, `Guidance`-level steps may be skipped by this actor.
    pub skip_guidance_steps: bool,
}

impl Default for ActorProfile {
    fn default() -> Self {
        Self {
            revision_multiplier: 1.0,
            evidence_multiplier: 1.0,
            skip_guidance_steps: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_depth_level_deserialize() {
        let yaml = "simplified";
        let val: DepthLevel = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(val, DepthLevel::Simplified);

        let yaml = "standard";
        let val: DepthLevel = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(val, DepthLevel::Standard);

        let yaml = "full";
        let val: DepthLevel = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(val, DepthLevel::Full);

        // Default
        let val = DepthLevel::default();
        assert_eq!(val, DepthLevel::Standard);
    }

    #[test]
    fn test_binding_level_deserialize() {
        let yaml = "requirement";
        let val: BindingLevel = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(val, BindingLevel::Requirement);

        let yaml = "guidance";
        let val: BindingLevel = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(val, BindingLevel::Guidance);

        // Default
        let val = BindingLevel::default();
        assert_eq!(val, BindingLevel::Requirement);
    }

    #[test]
    fn test_procedure_aggregate_roundtrip() {
        let agg = ProcedureAggregate {
            initial_state: "not_started".to_string(),
            states: vec![
                "not_started".to_string(),
                "in_progress".to_string(),
                "completed".to_string(),
            ],
            transitions: vec![ProcedureTransition {
                from_state: "not_started".to_string(),
                to_state: "in_progress".to_string(),
                command: Some("start".to_string()),
                emits: Some("Started".to_string()),
                guards: vec![],
            }],
        };
        let serialized = serde_yaml::to_string(&agg).unwrap();
        let deserialized: ProcedureAggregate = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.initial_state, "not_started");
        assert_eq!(deserialized.states.len(), 3);
        assert_eq!(deserialized.transitions.len(), 1);
        assert_eq!(deserialized.transitions[0].from_state, "not_started");
        assert_eq!(deserialized.transitions[0].to_state, "in_progress");

        // Default
        let default_agg = ProcedureAggregate::default();
        assert!(default_agg.initial_state.is_empty());
        assert!(default_agg.states.is_empty());
        assert!(default_agg.transitions.is_empty());
    }

    #[test]
    fn test_step_with_evidence_deserialize() {
        let yaml = r#"
id: step_review
name: Review documentation
binding: requirement
guards:
  - type: state
    expression: "procedure.planning.complete"
    failure_message: "Planning must be complete before review"
evidence:
  - type: workpaper
    required: true
    direction: produces
  - template_ref:
      ref: ev_tpl_001
    required: false
standards:
  - ref: ISA-315
    paragraph: "A21"
decision:
  question: "Is the risk assessment adequate?"
  branches:
    - label: "yes"
      target: step_conclude
    - label: "no"
      target: step_expand_scope
      condition: "risk_score > 0.7"
"#;
        let step: BlueprintStep = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(step.id, "step_review");
        assert_eq!(step.binding, BindingLevel::Requirement);
        assert_eq!(step.guards.len(), 1);
        assert_eq!(step.guards[0].guard_type, "state");
        assert_eq!(step.evidence.len(), 2);
        assert!(step.evidence[0].required);
        assert_eq!(step.evidence[0].direction, "produces");
        assert!(step.evidence[1].template_ref.is_some());
        assert_eq!(
            step.evidence[1].template_ref.as_ref().unwrap().ref_id,
            "ev_tpl_001"
        );
        assert_eq!(step.standards.len(), 1);
        assert_eq!(step.standards[0].ref_id, "ISA-315");
        assert_eq!(step.standards[0].paragraph.as_deref(), Some("A21"));
        let decision = step.decision.unwrap();
        assert_eq!(decision.branches.len(), 2);
        assert_eq!(decision.branches[1].label, "no");
        assert_eq!(decision.branches[1].target, "step_expand_scope");
    }
}
