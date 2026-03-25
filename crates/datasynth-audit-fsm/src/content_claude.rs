//! Claude CLI adapter for content generation.
//!
//! Requires the `claude` CLI tool (Anthropic's official CLI) to be installed
//! and authenticated.  Enable with `--features claude-content`.
//!
//! The adapter shells out to `claude -p <prompt>` for each narrative request.
//! If the CLI is unavailable or returns an error, a fallback message is used
//! so that generation never panics.

use crate::content::{AnalyticalContext, ContentGenerator, FindingContext, ResponseContext, WorkpaperContext};
use std::process::Command;

/// Content generator that calls the Claude CLI for high-quality narratives.
pub struct ClaudeContentGenerator {
    /// Model identifier passed to `claude --model`.
    pub(crate) model: String,
    /// Maximum tokens for each response.
    pub(crate) max_tokens: usize,
}

impl ClaudeContentGenerator {
    /// Create a new adapter with the default model (`claude-sonnet-4-20250514`).
    pub fn new() -> Self {
        Self {
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: 500,
        }
    }

    /// Create an adapter targeting a specific model.
    pub fn with_model(model: &str) -> Self {
        Self {
            model: model.to_string(),
            max_tokens: 500,
        }
    }

    /// Call the Claude CLI with the given prompt.  Returns the trimmed stdout
    /// on success, or a bracketed fallback message on any failure.
    fn call_claude(&self, prompt: &str) -> String {
        let result = Command::new("claude")
            .args([
                "-p",
                prompt,
                "--model",
                &self.model,
                "--max-tokens",
                &self.max_tokens.to_string(),
            ])
            .output();

        match result {
            Ok(output) if output.status.success() => {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::warn!("Claude CLI returned error: {}", stderr);
                let preview = &prompt[..prompt.len().min(100)];
                format!("[Claude unavailable] {preview}")
            }
            Err(e) => {
                tracing::warn!("Failed to invoke Claude CLI: {e}");
                let preview = &prompt[..prompt.len().min(100)];
                format!("[Claude unavailable] {preview}")
            }
        }
    }
}

impl Default for ClaudeContentGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ContentGenerator for ClaudeContentGenerator {
    fn generate_finding_narrative(&self, ctx: &FindingContext) -> String {
        let prompt = format!(
            "You are an audit professional writing a finding narrative for an \
             ISA-compliant audit workpaper. Write a concise (2-3 sentences) \
             finding description.\n\n\
             Procedure: {}\nStep: {}\nStandards: {}\nFinding type: {}\n\
             Condition observed: {}\nApplicable criteria: {}\n\n\
             Write the finding narrative in professional audit language.",
            ctx.procedure_id,
            ctx.step_id,
            ctx.standards_refs.join(", "),
            ctx.finding_type,
            ctx.condition,
            ctx.criteria,
        );
        self.call_claude(&prompt)
    }

    fn generate_workpaper_narrative(&self, ctx: &WorkpaperContext) -> String {
        let prompt = format!(
            "You are an audit professional writing a workpaper objective for \
             an ISA-compliant audit. Write a concise (1-2 sentences) workpaper \
             objective.\n\n\
             Procedure: {}\nSection: {}\nPrepared by: {}\nStandards: {}\n\n\
             Write the workpaper objective in professional audit language.",
            ctx.procedure_id,
            ctx.section,
            ctx.actor,
            ctx.standards_refs.join(", "),
        );
        self.call_claude(&prompt)
    }

    fn generate_management_response(&self, ctx: &ResponseContext) -> String {
        let prompt = format!(
            "You are a company's management responding to an audit finding. \
             Write a concise (2-3 sentences) management response.\n\n\
             Finding type: {}\nCondition: {}\nAuditor recommendation: {}\n\n\
             Write management's response acknowledging the finding and \
             outlining remediation.",
            ctx.finding_type, ctx.condition, ctx.recommendation,
        );
        self.call_claude(&prompt)
    }

    fn generate_analytical_narrative(&self, ctx: &AnalyticalContext) -> String {
        let prompt = format!(
            "You are an audit data analyst writing a narrative for an analytical \
             procedure performed during an ISA-compliant audit. Write a concise \
             (2-3 sentences) description of the procedure and its results.\n\n\
             Procedure: {} ({})\nStep: {}\nType: {}\nDescription: {}\n\
             Data features: {}\nExpected output: {}\nThreshold: {}\n\n\
             Write the analytical procedure narrative in professional audit language.",
            ctx.procedure_id,
            ctx.name,
            ctx.step_id,
            ctx.procedure_type,
            ctx.description,
            ctx.data_features.join(", "),
            ctx.expected_output,
            if ctx.threshold.is_empty() { "N/A" } else { &ctx.threshold },
        );
        self.call_claude(&prompt)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_generator_construction() {
        let gen = ClaudeContentGenerator::new();
        assert_eq!(gen.model, "claude-sonnet-4-20250514");
        assert_eq!(gen.max_tokens, 500);
    }

    #[test]
    fn test_claude_generator_with_model() {
        let gen = ClaudeContentGenerator::with_model("claude-haiku-3");
        assert_eq!(gen.model, "claude-haiku-3");
    }

    #[test]
    fn test_claude_generator_default() {
        let gen = ClaudeContentGenerator::default();
        assert_eq!(gen.model, "claude-sonnet-4-20250514");
    }

    #[test]
    fn test_claude_generator_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ClaudeContentGenerator>();
    }

    /// Verify the fallback works when claude CLI is not installed.
    /// This test should pass in CI where the CLI is absent.
    #[test]
    fn test_claude_fallback_when_cli_absent() {
        let gen = ClaudeContentGenerator::new();
        let ctx = FindingContext {
            procedure_id: "test_procedure".into(),
            step_id: "step_1".into(),
            standards_refs: vec!["ISA-315".into()],
            finding_type: "control_deficiency".into(),
            condition: "test condition".into(),
            criteria: "test criteria".into(),
        };

        let narrative = gen.generate_finding_narrative(&ctx);
        // Should either succeed (if claude is available) or return the fallback.
        assert!(!narrative.is_empty(), "narrative must not be empty");
    }
}
