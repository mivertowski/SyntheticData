//! Pluggable content generation for audit narratives.
//!
//! Defines a [`ContentGenerator`] trait with three methods for generating
//! finding narratives, workpaper narratives, and management responses.
//! The default [`TemplateContentGenerator`] uses simple string interpolation.
//! Alternative implementations (e.g. LLM-backed) can implement the same trait.

// ---------------------------------------------------------------------------
// Context types
// ---------------------------------------------------------------------------

/// Context for generating finding narratives.
pub struct FindingContext {
    /// The procedure within which the finding was identified.
    pub procedure_id: String,
    /// The step within the procedure where the finding was identified.
    pub step_id: String,
    /// Standards references applicable to this finding (e.g. `["ISA-315"]`).
    pub standards_refs: Vec<String>,
    /// Category of finding (e.g. `"control_deficiency"`, `"misstatement"`).
    pub finding_type: String,
    /// Factual condition observed.
    pub condition: String,
    /// Criteria that the condition was evaluated against.
    pub criteria: String,
}

/// Context for generating workpaper narratives.
pub struct WorkpaperContext {
    /// The procedure this workpaper documents.
    pub procedure_id: String,
    /// Section of the workpaper (e.g. `"risk_assessment"`, `"substantive_testing"`).
    pub section: String,
    /// Actor who prepared the workpaper (e.g. actor id or display name).
    pub actor: String,
    /// Standards references applicable to this workpaper.
    pub standards_refs: Vec<String>,
}

/// Context for generating management responses to audit findings.
pub struct ResponseContext {
    /// Category of the finding being responded to.
    pub finding_type: String,
    /// Factual condition observed (mirrors [`FindingContext::condition`]).
    pub condition: String,
    /// Auditor's recommended remediation action.
    pub recommendation: String,
}

/// Context for generating analytical procedure narratives, populated from
/// the analytics inventory.
pub struct AnalyticalContext {
    /// Parent procedure identifier.
    pub procedure_id: String,
    /// Step identifier within the procedure.
    pub step_id: String,
    /// Type of analytical procedure (e.g. `"ratio_analysis"`, `"trend_analysis"`).
    pub procedure_type: String,
    /// Human-readable procedure name.
    pub name: String,
    /// Description of the analytical procedure.
    pub description: String,
    /// Data features analysed by this procedure.
    pub data_features: Vec<String>,
    /// Threshold or tolerance applied.
    pub threshold: String,
    /// Expected output of the procedure.
    pub expected_output: String,
}

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// Trait for pluggable content generation.
///
/// The default implementation ([`TemplateContentGenerator`]) uses simple
/// string templates. LLM backends or richer template engines can implement
/// this trait and be swapped in at call sites without changing the rest of
/// the codebase.
///
/// All methods take a shared reference to `self` so that generators can be
/// used concurrently, and the trait requires `Send + Sync` to support
/// multi-threaded execution contexts.
pub trait ContentGenerator: Send + Sync {
    /// Generate a narrative describing an audit finding.
    fn generate_finding_narrative(&self, context: &FindingContext) -> String;

    /// Generate a narrative for an audit workpaper section.
    fn generate_workpaper_narrative(&self, context: &WorkpaperContext) -> String;

    /// Generate a management response to an audit finding.
    fn generate_management_response(&self, context: &ResponseContext) -> String;

    /// Generate a narrative for an analytical procedure derived from the
    /// analytics inventory.
    fn generate_analytical_narrative(&self, context: &AnalyticalContext) -> String;
}

// ---------------------------------------------------------------------------
// Default implementation
// ---------------------------------------------------------------------------

/// Template-based content generator — no LLM required.
///
/// Each method returns a deterministic string assembled from the context
/// fields. The output is suitable for synthetic audit data and unit testing.
pub struct TemplateContentGenerator;

impl ContentGenerator for TemplateContentGenerator {
    fn generate_finding_narrative(&self, ctx: &FindingContext) -> String {
        format!(
            "During the {} procedure (step {}), a {} was identified. \
             Condition: {}. The applicable criteria per {} require that {}.",
            ctx.procedure_id,
            ctx.step_id,
            ctx.finding_type,
            ctx.condition,
            ctx.standards_refs.join(", "),
            ctx.criteria,
        )
    }

    fn generate_workpaper_narrative(&self, ctx: &WorkpaperContext) -> String {
        format!(
            "Workpaper for {} procedure, {} section. \
             Prepared by {} in accordance with {}.",
            ctx.procedure_id,
            ctx.section,
            ctx.actor,
            ctx.standards_refs.join(", "),
        )
    }

    fn generate_management_response(&self, ctx: &ResponseContext) -> String {
        format!(
            "Management acknowledges the {} finding regarding {}. \
             In response to the recommendation to {}, management will \
             implement corrective action within 90 days.",
            ctx.finding_type, ctx.condition, ctx.recommendation,
        )
    }

    fn generate_analytical_narrative(&self, ctx: &AnalyticalContext) -> String {
        format!(
            "{} — {}: {}. Data features analyzed: {}. {}{}",
            ctx.name,
            ctx.procedure_type,
            ctx.description,
            ctx.data_features.join(", "),
            ctx.expected_output,
            if ctx.threshold.is_empty() {
                String::new()
            } else {
                format!(" Threshold: {}.", ctx.threshold)
            },
        )
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_finding_ctx() -> FindingContext {
        FindingContext {
            procedure_id: "risk_assessment".into(),
            step_id: "step_identify_risks".into(),
            standards_refs: vec!["ISA-315".into(), "ISA-330".into()],
            finding_type: "control_deficiency".into(),
            condition: "segregation of duties not enforced in AP module".into(),
            criteria: "adequate controls exist over financial reporting".into(),
        }
    }

    fn make_workpaper_ctx() -> WorkpaperContext {
        WorkpaperContext {
            procedure_id: "substantive_testing".into(),
            section: "revenue_recognition".into(),
            actor: "audit_senior".into(),
            standards_refs: vec!["ISA-500".into()],
        }
    }

    fn make_response_ctx() -> ResponseContext {
        ResponseContext {
            finding_type: "material_weakness".into(),
            condition: "lack of review controls over journal entries".into(),
            recommendation: "implement a daily review of manual journal entries".into(),
        }
    }

    #[test]
    fn test_template_finding_narrative() {
        let gen = TemplateContentGenerator;
        let ctx = make_finding_ctx();
        let narrative = gen.generate_finding_narrative(&ctx);
        assert!(!narrative.is_empty(), "narrative must not be empty");
        assert!(
            narrative.contains(&ctx.procedure_id),
            "narrative must contain procedure_id"
        );
        assert!(
            narrative.contains(&ctx.finding_type),
            "narrative must contain finding_type"
        );
        assert!(
            narrative.contains(&ctx.condition),
            "narrative must contain condition"
        );
    }

    #[test]
    fn test_template_workpaper_narrative() {
        let gen = TemplateContentGenerator;
        let ctx = make_workpaper_ctx();
        let narrative = gen.generate_workpaper_narrative(&ctx);
        assert!(!narrative.is_empty(), "narrative must not be empty");
        assert!(
            narrative.contains(&ctx.section),
            "narrative must contain section"
        );
        assert!(
            narrative.contains(&ctx.actor),
            "narrative must contain actor"
        );
    }

    #[test]
    fn test_template_management_response() {
        let gen = TemplateContentGenerator;
        let ctx = make_response_ctx();
        let response = gen.generate_management_response(&ctx);
        assert!(!response.is_empty(), "response must not be empty");
        assert!(
            response.contains(&ctx.finding_type),
            "response must contain finding_type"
        );
        assert!(
            response.contains(&ctx.condition),
            "response must contain condition"
        );
    }

    #[test]
    fn test_template_generator_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<TemplateContentGenerator>();
    }

    fn make_analytical_ctx() -> AnalyticalContext {
        AnalyticalContext {
            procedure_id: "risk_assessment".into(),
            step_id: "risk_step_1".into(),
            procedure_type: "trend_analysis".into(),
            name: "Revenue trend analysis".into(),
            description: "Analyze revenue trends over periods".into(),
            data_features: vec!["revenue".into(), "period".into(), "growth_rate".into()],
            threshold: "5% deviation".into(),
            expected_output: "Trend line with variance flags".into(),
        }
    }

    #[test]
    fn test_analytical_narrative_template() {
        let gen = TemplateContentGenerator;
        let ctx = make_analytical_ctx();
        let narrative = gen.generate_analytical_narrative(&ctx);
        assert!(
            !narrative.is_empty(),
            "analytical narrative must not be empty"
        );
        assert!(
            narrative.contains(&ctx.name),
            "narrative must contain procedure name"
        );
        assert!(
            narrative.contains(&ctx.procedure_type),
            "narrative must contain procedure type"
        );
        assert!(
            narrative.contains("revenue"),
            "narrative must contain data features"
        );
        assert!(
            narrative.contains(&ctx.expected_output),
            "narrative must contain expected output"
        );
        assert!(
            narrative.contains("Threshold: 5% deviation"),
            "narrative must contain threshold"
        );
    }

    #[test]
    fn test_analytical_narrative_empty_threshold() {
        let gen = TemplateContentGenerator;
        let ctx = AnalyticalContext {
            procedure_id: "p1".into(),
            step_id: "s1".into(),
            procedure_type: "ratio_analysis".into(),
            name: "Test ratio".into(),
            description: "Test desc".into(),
            data_features: vec!["f1".into()],
            threshold: String::new(),
            expected_output: "Expected output".into(),
        };
        let narrative = gen.generate_analytical_narrative(&ctx);
        assert!(
            !narrative.contains("Threshold"),
            "empty threshold should not appear in narrative"
        );
    }
}
