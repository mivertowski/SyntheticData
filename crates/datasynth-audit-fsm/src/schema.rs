//! Blueprint and overlay schema types.
//!
//! Provides Rust types for deserializing audit methodology YAML blueprints
//! and generation overlay YAML files.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
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
    /// Informational — context only, no compliance obligation. GAM-specific.
    Informational,
    /// Example — illustrative guidance. GAM-specific.
    Example,
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
    /// Top-level discriminator dimensions (e.g. categories, risk_ratings).
    #[serde(default)]
    pub discriminators: HashMap<String, Vec<String>>,
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

    // ----- GAM-specific metadata (preserved as opaque JSON for export) -----
    /// GAM forms catalog (e.g. EY form templates).
    #[serde(default)]
    pub forms_catalog: Vec<JsonValue>,

    /// ISA cross-reference dependency graph.
    #[serde(default)]
    pub standard_dependencies: Vec<JsonValue>,

    /// ISA coverage statistics.
    #[serde(default)]
    pub coverage: Option<JsonValue>,

    /// Form enrichment data.
    #[serde(default)]
    pub form_enrichment: Option<JsonValue>,
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

    // ----- GAM-specific standard fields (preserved for round-trip) -----
    /// ISA grouping category (e.g. `General Principles and Responsibilities`).
    #[serde(default)]
    pub isa_group: Option<String>,
    /// Number of requirement paragraphs in this standard.
    #[serde(default)]
    pub requirement_count: Option<u32>,
    /// Number of application paragraphs in this standard.
    #[serde(default)]
    pub application_count: Option<u32>,
    /// Detailed paragraph entries (opaque JSON for now).
    #[serde(default)]
    pub paragraphs: Vec<JsonValue>,
    /// Dependency references to other standards.
    #[serde(default)]
    pub dependencies: Vec<JsonValue>,
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

    // ----- GAM-specific evidence fields (preserved for round-trip) -----
    /// EY form references that source this evidence.
    #[serde(default)]
    pub source_forms: Vec<String>,
    /// Whether sign-off is required for this evidence item.
    #[serde(default)]
    pub signoff_required: Vec<String>,
    /// Expected data fields within this evidence artifact.
    #[serde(default)]
    pub required_fields: Vec<String>,
    /// Whether this evidence entry is noise / non-essential.
    #[serde(default)]
    pub is_noise: Option<bool>,
    /// Source type classification (e.g. `ey_form`, `system`, `manual`).
    #[serde(default)]
    pub source_type: Option<String>,
    /// Actor responsible for producing this evidence.
    #[serde(default)]
    pub responsible_actor: Option<String>,
    /// Free-text description of expected content.
    #[serde(default)]
    pub expected_content: Option<String>,
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
    /// Ordering key. Negative values indicate continuous/background phases.
    #[serde(default)]
    pub order: Option<i32>,
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
    /// Discriminator dimensions applicable to this procedure.
    #[serde(default)]
    pub discriminators: HashMap<String, Vec<String>>,
    /// FSM aggregate: initial state, valid states, and transitions.
    #[serde(default)]
    pub aggregate: ProcedureAggregate,
    /// Ordered steps within this procedure.
    #[serde(default)]
    pub steps: Vec<BlueprintStep>,
    /// Estimated base hours for the procedure (used in cost modelling).
    #[serde(default)]
    pub base_hours: Option<f64>,
    /// Roles required to execute this procedure (e.g. `["audit_senior"]`).
    #[serde(default)]
    pub required_roles: Vec<String>,
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

    // ----- GAM-specific step fields -----
    /// Whether this step is ISA-mandated (vs advisory). GAM-specific.
    #[serde(default)]
    pub isa_mandate: Option<String>,

    /// References to EY form templates. GAM-specific.
    #[serde(default)]
    pub form_refs: Vec<JsonValue>,

    /// Expected deliverable fields for this step. GAM-specific.
    #[serde(default)]
    pub deliverable_fields: Vec<String>,

    /// ISA paragraph tracing for standard fields. GAM-specific.
    #[serde(default)]
    pub standard_field_trace: Vec<String>,
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
// Iteration limits
// ---------------------------------------------------------------------------

/// Per-procedure iteration limits for the FSM engine.
///
/// Replaces the former global `MAX_ITERATIONS` constant with a configurable
/// default and optional per-procedure overrides.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationLimits {
    /// Default maximum iterations for any procedure.
    #[serde(default = "default_iteration_limit")]
    pub default: usize,
    /// Per-procedure overrides keyed by procedure id.
    #[serde(default)]
    pub per_procedure: HashMap<String, usize>,
}

fn default_iteration_limit() -> usize {
    30
}

impl Default for IterationLimits {
    fn default() -> Self {
        Self {
            default: 30,
            per_procedure: HashMap::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Resource costs
// ---------------------------------------------------------------------------

/// Cost model configuration applied via overlays to compute resource estimates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCosts {
    /// Global multiplier applied to all base hours (e.g. 1.5 for thorough).
    #[serde(default = "default_cost_multiplier")]
    pub cost_multiplier: f64,
    /// Hourly billing rates keyed by role id (e.g. `"engagement_partner": 500`).
    #[serde(default)]
    pub role_hourly_rates: HashMap<String, f64>,
    /// Per-procedure multipliers keyed by procedure id.
    #[serde(default)]
    pub per_procedure_multipliers: HashMap<String, f64>,
}

fn default_cost_multiplier() -> f64 {
    1.0
}

impl Default for ResourceCosts {
    fn default() -> Self {
        Self {
            cost_multiplier: 1.0,
            role_hourly_rates: HashMap::new(),
            per_procedure_multipliers: HashMap::new(),
        }
    }
}

impl ResourceCosts {
    /// Compute the effective hours for a procedure after applying multipliers.
    ///
    /// Falls back to 8.0 if the procedure has no `base_hours` set.
    pub fn effective_hours(&self, proc: &BlueprintProcedure) -> f64 {
        let base = proc.base_hours.unwrap_or(8.0);
        let proc_mult = self
            .per_procedure_multipliers
            .get(&proc.id)
            .copied()
            .unwrap_or(1.0);
        base * self.cost_multiplier * proc_mult
    }

    /// Compute the monetary cost for a procedure.
    ///
    /// Uses the first entry in `required_roles` to look up the hourly rate;
    /// falls back to `"audit_staff"` and then to a default rate of 200.0.
    pub fn procedure_cost(&self, proc: &BlueprintProcedure) -> f64 {
        let hours = self.effective_hours(proc);
        let role = proc
            .required_roles
            .first()
            .map(|r| r.as_str())
            .unwrap_or("audit_staff");
        let rate = self.role_hourly_rates.get(role).copied().unwrap_or(200.0);
        hours * rate
    }
}

// ---------------------------------------------------------------------------
// Overlay types
// ---------------------------------------------------------------------------

/// Generation overlay that customises how a blueprint is instantiated.
///
/// Overlays are applied on top of a blueprint to tune probabilities, volumes,
/// timing distributions, and anomaly injection rates without modifying the
/// canonical blueprint YAML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationOverlay {
    /// Id of the blueprint this overlay targets.
    #[serde(default)]
    pub blueprint_id: Option<String>,
    /// Depth override (overrides `methodology.default_depth`).
    #[serde(default)]
    pub depth: Option<DepthLevel>,
    /// Maximum number of self-loop iterations (e.g. review/revise cycles).
    #[serde(default = "default_max_self_loop_iterations")]
    pub max_self_loop_iterations: usize,
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
    /// Optional discriminator filter: only procedures matching at least one
    /// value in each specified category will execute. `None` means no filter.
    #[serde(default)]
    pub discriminators: Option<HashMap<String, Vec<String>>>,
    /// Per-procedure iteration limits (replaces global MAX_ITERATIONS).
    #[serde(default)]
    pub iteration_limits: IterationLimits,
    /// Resource cost configuration for budget estimation.
    #[serde(default)]
    pub resource_costs: ResourceCosts,
}

fn default_max_self_loop_iterations() -> usize {
    5
}

impl Default for GenerationOverlay {
    fn default() -> Self {
        Self {
            blueprint_id: None,
            depth: None,
            max_self_loop_iterations: default_max_self_loop_iterations(),
            transitions: TransitionConfig::default(),
            artifacts: ArtifactConfig::default(),
            anomalies: AnomalyConfig::default(),
            actor_profiles: HashMap::new(),
            discriminators: None,
            iteration_limits: IterationLimits::default(),
            resource_costs: ResourceCosts::default(),
        }
    }
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
///
/// When `max < min`, sampling methods clamp `max` to `min` to avoid panics.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VolumeRange {
    /// Minimum value (inclusive).
    pub min: u32,
    /// Maximum value (inclusive). Must be >= min; clamped if not.
    pub max: u32,
}

impl VolumeRange {
    /// Sample a random value in `[min, max]` (inclusive).
    ///
    /// If `max < min`, uses `min` as the result.
    pub fn sample(&self, rng: &mut impl rand::Rng) -> u32 {
        let effective_max = self.max.max(self.min);
        if effective_max == self.min {
            return self.min;
        }
        rng.random_range(self.min..=effective_max)
    }
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

    #[test]
    fn test_phase_with_order_deserialize() {
        let yaml = r#"
id: continuous_phase
name: Continuous Monitoring
order: -2
"#;
        let phase: BlueprintPhase = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(phase.id, "continuous_phase");
        assert_eq!(phase.order, Some(-2));
    }

    #[test]
    fn test_phase_without_order_defaults_none() {
        let yaml = r#"
id: regular_phase
name: Planning
"#;
        let phase: BlueprintPhase = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(phase.id, "regular_phase");
        assert_eq!(phase.order, None);
    }

    #[test]
    fn test_overlay_max_self_loop_iterations_default() {
        let overlay = GenerationOverlay::default();
        assert_eq!(overlay.max_self_loop_iterations, 5);
    }

    #[test]
    fn test_overlay_max_self_loop_iterations_from_yaml() {
        let yaml = r#"
max_self_loop_iterations: 10
"#;
        let overlay: GenerationOverlay = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(overlay.max_self_loop_iterations, 10);
    }

    #[test]
    fn test_blueprint_discriminators_deserialize() {
        let yaml = r#"
id: test-bp
name: Test
version: "1.0"
methodology:
  framework: TEST
discriminators:
  categories: [financial, operational]
  risk_ratings: [high, medium, low]
phases: []
"#;
        let bp: AuditBlueprint = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(bp.discriminators.len(), 2);
        assert_eq!(
            bp.discriminators.get("categories").unwrap(),
            &vec!["financial".to_string(), "operational".to_string()]
        );
        assert_eq!(bp.discriminators.get("risk_ratings").unwrap().len(), 3);
    }

    #[test]
    fn test_procedure_discriminators_deserialize() {
        let yaml = r#"
id: test_proc
name: Test Procedure
discriminators:
  engagement_types: [assurance, advisory]
"#;
        let proc: BlueprintProcedure = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(proc.discriminators.len(), 1);
        assert_eq!(
            proc.discriminators.get("engagement_types").unwrap(),
            &vec!["assurance".to_string(), "advisory".to_string()]
        );
    }

    // -----------------------------------------------------------------------
    // Cost model tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_resource_costs_default() {
        let costs = ResourceCosts::default();
        assert!((costs.cost_multiplier - 1.0).abs() < f64::EPSILON);
        assert!(costs.role_hourly_rates.is_empty());
        assert!(costs.per_procedure_multipliers.is_empty());
    }

    #[test]
    fn test_effective_hours_default() {
        let costs = ResourceCosts::default();
        let proc = BlueprintProcedure {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: None,
            discriminators: HashMap::new(),
            aggregate: ProcedureAggregate::default(),
            steps: vec![],
            base_hours: None,
            required_roles: vec![],
        };
        // base=8.0 (default), multiplier=1.0, no proc override -> 8.0
        assert!((costs.effective_hours(&proc) - 8.0).abs() < 0.01);
    }

    #[test]
    fn test_effective_hours_with_base() {
        let costs = ResourceCosts::default();
        let proc = BlueprintProcedure {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: None,
            discriminators: HashMap::new(),
            aggregate: ProcedureAggregate::default(),
            steps: vec![],
            base_hours: Some(16.0),
            required_roles: vec![],
        };
        assert!((costs.effective_hours(&proc) - 16.0).abs() < 0.01);
    }

    #[test]
    fn test_effective_hours_with_multipliers() {
        let mut costs = ResourceCosts {
            cost_multiplier: 1.5,
            ..Default::default()
        };
        costs
            .per_procedure_multipliers
            .insert("test_proc".into(), 1.3);
        let proc = BlueprintProcedure {
            id: "test_proc".to_string(),
            name: "Test".to_string(),
            description: None,
            discriminators: HashMap::new(),
            aggregate: ProcedureAggregate::default(),
            steps: vec![],
            base_hours: Some(10.0),
            required_roles: vec![],
        };
        // 10.0 * 1.5 * 1.3 = 19.5
        assert!((costs.effective_hours(&proc) - 19.5).abs() < 0.01);
    }

    #[test]
    fn test_procedure_cost_with_role() {
        let mut costs = ResourceCosts::default();
        costs
            .role_hourly_rates
            .insert("engagement_partner".into(), 500.0);
        let proc = BlueprintProcedure {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: None,
            discriminators: HashMap::new(),
            aggregate: ProcedureAggregate::default(),
            steps: vec![],
            base_hours: Some(4.0),
            required_roles: vec!["engagement_partner".to_string()],
        };
        // 4.0 * 1.0 * 500.0 = 2000.0
        assert!((costs.procedure_cost(&proc) - 2000.0).abs() < 0.01);
    }

    #[test]
    fn test_procedure_cost_fallback_role() {
        let costs = ResourceCosts::default();
        let proc = BlueprintProcedure {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: None,
            discriminators: HashMap::new(),
            aggregate: ProcedureAggregate::default(),
            steps: vec![],
            base_hours: Some(10.0),
            required_roles: vec![],
        };
        // No role -> fallback to audit_staff -> no rate -> fallback 200.0
        // 10.0 * 1.0 * 200.0 = 2000.0
        assert!((costs.procedure_cost(&proc) - 2000.0).abs() < 0.01);
    }

    #[test]
    fn test_resource_costs_yaml_roundtrip() {
        let yaml = r#"
cost_multiplier: 1.5
role_hourly_rates:
  engagement_partner: 500.0
  audit_staff: 120.0
per_procedure_multipliers:
  risk_identification: 1.2
"#;
        let costs: ResourceCosts = serde_yaml::from_str(yaml).unwrap();
        assert!((costs.cost_multiplier - 1.5).abs() < f64::EPSILON);
        assert_eq!(costs.role_hourly_rates.len(), 2);
        assert!(
            (costs.role_hourly_rates.get("engagement_partner").unwrap() - 500.0).abs()
                < f64::EPSILON
        );
        assert_eq!(costs.per_procedure_multipliers.len(), 1);
    }

    #[test]
    fn test_overlay_resource_costs_default() {
        let overlay = GenerationOverlay::default();
        assert!((overlay.resource_costs.cost_multiplier - 1.0).abs() < f64::EPSILON);
        assert!(overlay.resource_costs.role_hourly_rates.is_empty());
    }

    #[test]
    fn test_overlay_resource_costs_from_yaml() {
        let yaml = r#"
resource_costs:
  cost_multiplier: 1.5
  role_hourly_rates:
    audit_manager: 300.0
"#;
        let overlay: GenerationOverlay = serde_yaml::from_str(yaml).unwrap();
        assert!((overlay.resource_costs.cost_multiplier - 1.5).abs() < f64::EPSILON);
        assert_eq!(overlay.resource_costs.role_hourly_rates.len(), 1);
    }

    #[test]
    fn test_procedure_base_hours_deserialize() {
        let yaml = r#"
id: test_proc
name: Test Procedure
base_hours: 12.5
required_roles: [audit_senior, audit_staff]
"#;
        let proc: BlueprintProcedure = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(proc.base_hours, Some(12.5));
        assert_eq!(proc.required_roles, vec!["audit_senior", "audit_staff"]);
    }

    #[test]
    fn test_procedure_base_hours_defaults_none() {
        let yaml = r#"
id: test_proc
name: Test Procedure
"#;
        let proc: BlueprintProcedure = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(proc.base_hours, None);
        assert!(proc.required_roles.is_empty());
    }
}
