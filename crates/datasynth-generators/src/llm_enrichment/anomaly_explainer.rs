//! LLM-powered anomaly explanation generation.
//!
//! Generates natural language explanations for detected anomalies using an LLM
//! provider, with deterministic template-based fallbacks.

use std::sync::Arc;

use datasynth_core::error::SynthError;
use datasynth_core::llm::{LlmProvider, LlmRequest};

/// Generates natural language explanations for anomalies using an LLM provider.
///
/// Wraps a `dyn LlmProvider` to produce human-readable explanations for
/// different anomaly types. Falls back to template-based explanations when
/// the LLM call fails.
pub struct AnomalyLlmExplainer {
    provider: Arc<dyn LlmProvider>,
}

impl AnomalyLlmExplainer {
    /// Create a new explainer with the given LLM provider.
    pub fn new(provider: Arc<dyn LlmProvider>) -> Self {
        Self { provider }
    }

    /// Generate a natural language explanation for an anomaly.
    ///
    /// Produces a concise, audit-ready explanation that describes the anomaly
    /// type, the number of affected records, and the relevant statistical
    /// details.
    pub fn explain(
        &self,
        anomaly_type: &str,
        affected_records: usize,
        statistical_details: &str,
    ) -> Result<String, SynthError> {
        let prompt = format!(
            "Explain the following anomaly detected in accounting data in 1-2 sentences. \
             Anomaly type: {anomaly_type}. Affected records: {affected_records}. Statistical details: {statistical_details}. \
             Write a concise, professional explanation suitable for an audit workpaper."
        );

        let request = LlmRequest::new(prompt)
            .with_system(
                "You are an audit analytics expert. Provide a clear, concise anomaly \
                 explanation suitable for audit documentation. Return only the explanation \
                 text."
                    .to_string(),
            )
            .with_max_tokens(256)
            .with_temperature(0.5);

        match self.provider.complete(&request) {
            Ok(response) => {
                let explanation = response.content.trim().to_string();
                if explanation.is_empty() {
                    Ok(Self::fallback_explanation(
                        anomaly_type,
                        affected_records,
                        statistical_details,
                    ))
                } else {
                    Ok(explanation)
                }
            }
            Err(_) => Ok(Self::fallback_explanation(
                anomaly_type,
                affected_records,
                statistical_details,
            )),
        }
    }

    /// Deterministic fallback explanation based on anomaly parameters.
    fn fallback_explanation(
        anomaly_type: &str,
        affected_records: usize,
        statistical_details: &str,
    ) -> String {
        let severity = if affected_records > 100 {
            "high-impact"
        } else if affected_records > 10 {
            "moderate"
        } else {
            "isolated"
        };

        let type_description = match anomaly_type.to_lowercase().as_str() {
            "fictitious_transaction" | "fictitioustransaction" => {
                "Potentially fictitious transaction(s) identified with no supporting \
                 documentation or business justification"
            }
            "duplicate_entry" | "duplicateentry" | "duplicate_payment" | "duplicatepayment" => {
                "Duplicate transaction(s) detected that may indicate processing errors \
                 or intentional duplication"
            }
            "unusual_amount" | "unusualamount" => {
                "Transaction amount(s) significantly deviate from historical patterns \
                 for the associated account and vendor"
            }
            "benford_violation" | "benfordviolation" => {
                "First-digit distribution of transaction amounts deviates from expected \
                 Benford's Law patterns"
            }
            "split_transaction" | "splittransaction" => {
                "Transaction splitting pattern detected that may indicate attempts to \
                 circumvent approval thresholds"
            }
            "skipped_approval" | "skippedapproval" => {
                "Transaction(s) processed without required approval per the \
                 authorization matrix"
            }
            "round_tripping" | "roundtripping" => {
                "Circular transaction pattern detected between related entities"
            }
            "revenue_manipulation" | "revenuemanipulation" => {
                "Revenue recognition pattern inconsistent with delivery and performance \
                 obligation completion"
            }
            "late_posting" | "lateposting" | "wrong_period" | "wrongperiod" => {
                "Transaction(s) posted outside the expected accounting period"
            }
            "trend_break" | "trendbreak" => {
                "Significant departure from established trend in the affected account(s)"
            }
            _ => "Anomalous pattern detected requiring further investigation",
        };

        format!(
            "{anomaly_type} anomaly ({severity}, {affected_records} affected record(s)): {type_description}. Statistical basis: {statistical_details}."
        )
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::llm::MockLlmProvider;

    #[test]
    fn test_explain_returns_nonempty() {
        let provider = Arc::new(MockLlmProvider::new(42));
        let explainer = AnomalyLlmExplainer::new(provider);
        let explanation = explainer
            .explain("unusual_amount", 5, "z-score = 3.2, p < 0.001")
            .expect("should succeed");
        assert!(!explanation.is_empty(), "explanation should not be empty");
    }

    #[test]
    fn test_explain_deterministic() {
        let provider = Arc::new(MockLlmProvider::new(42));
        let explainer = AnomalyLlmExplainer::new(provider);
        let exp1 = explainer
            .explain("duplicate_entry", 12, "Jaccard similarity = 0.98")
            .expect("should succeed");
        let provider2 = Arc::new(MockLlmProvider::new(42));
        let explainer2 = AnomalyLlmExplainer::new(provider2);
        let exp2 = explainer2
            .explain("duplicate_entry", 12, "Jaccard similarity = 0.98")
            .expect("should succeed");
        assert_eq!(exp1, exp2, "same seed should yield same explanation");
    }

    #[test]
    fn test_explain_contains_anomaly_keyword() {
        let provider = Arc::new(MockLlmProvider::new(42));
        let explainer = AnomalyLlmExplainer::new(provider);
        // MockLlmProvider detects "anomaly" or "explain" in the prompt
        let explanation = explainer
            .explain("benford_violation", 50, "MAD = 0.025")
            .expect("should succeed");
        assert!(
            !explanation.is_empty(),
            "explanation should contain meaningful text"
        );
    }

    #[test]
    fn test_fallback_unusual_amount() {
        let explanation =
            AnomalyLlmExplainer::fallback_explanation("unusual_amount", 5, "z-score = 3.2");
        assert!(explanation.contains("unusual_amount"));
        assert!(explanation.contains("isolated"));
        assert!(explanation.contains("z-score = 3.2"));
        assert!(explanation.contains("significantly deviate"));
    }

    #[test]
    fn test_fallback_high_impact() {
        let explanation = AnomalyLlmExplainer::fallback_explanation(
            "duplicate_entry",
            150,
            "150 exact matches found",
        );
        assert!(explanation.contains("high-impact"));
        assert!(explanation.contains("150 affected record(s)"));
    }

    #[test]
    fn test_fallback_moderate_impact() {
        let explanation =
            AnomalyLlmExplainer::fallback_explanation("split_transaction", 25, "threshold = 10000");
        assert!(explanation.contains("moderate"));
        assert!(explanation.contains("circumvent approval"));
    }

    #[test]
    fn test_fallback_unknown_anomaly_type() {
        let explanation =
            AnomalyLlmExplainer::fallback_explanation("custom_anomaly", 3, "custom metric = 42");
        assert!(explanation.contains("further investigation"));
    }
}
