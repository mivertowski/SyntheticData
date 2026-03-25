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
}
