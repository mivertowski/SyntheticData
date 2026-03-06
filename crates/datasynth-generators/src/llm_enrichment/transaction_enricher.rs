//! LLM-powered transaction description and memo enrichment.
//!
//! Generates realistic transaction descriptions and memo fields using an LLM
//! provider, with deterministic template-based fallbacks.

use std::sync::Arc;

use datasynth_core::error::SynthError;
use datasynth_core::llm::{LlmProvider, LlmRequest};

/// Enriches transaction metadata using an LLM provider.
///
/// Wraps a `dyn LlmProvider` to generate realistic transaction descriptions and
/// memo fields. Falls back to template-based text when the LLM call fails.
pub struct TransactionLlmEnricher {
    provider: Arc<dyn LlmProvider>,
}

impl TransactionLlmEnricher {
    /// Create a new enricher with the given LLM provider.
    pub fn new(provider: Arc<dyn LlmProvider>) -> Self {
        Self { provider }
    }

    /// Generate a realistic transaction description.
    ///
    /// Uses the GL account name, amount range, industry, and fiscal period to
    /// produce a contextually appropriate description.
    pub fn enrich_description(
        &self,
        account_name: &str,
        amount_range: &str,
        industry: &str,
        fiscal_period: u32,
    ) -> Result<String, SynthError> {
        let prompt = format!(
            "Generate a single realistic transaction description for a journal entry. \
             Context: GL account '{account_name}', amount range {amount_range}, {industry} industry, fiscal period {fiscal_period}. \
             Return ONLY the description text, nothing else."
        );

        let request = LlmRequest::new(prompt)
            .with_system(
                "You are an accounting data generator. Return only a single transaction \
                 description line with no extra text."
                    .to_string(),
            )
            .with_max_tokens(128)
            .with_temperature(0.7);

        match self.provider.complete(&request) {
            Ok(response) => {
                let desc = response.content.trim().to_string();
                if desc.is_empty() {
                    Ok(Self::fallback_description(
                        account_name,
                        amount_range,
                        industry,
                        fiscal_period,
                    ))
                } else {
                    Ok(desc)
                }
            }
            Err(_) => Ok(Self::fallback_description(
                account_name,
                amount_range,
                industry,
                fiscal_period,
            )),
        }
    }

    /// Generate a realistic memo field for a document.
    ///
    /// Uses the document type, vendor name, and amount to produce a plausible
    /// memo or notes field.
    pub fn enrich_memo(
        &self,
        doc_type: &str,
        vendor_name: &str,
        amount: &str,
    ) -> Result<String, SynthError> {
        let prompt = format!(
            "Generate a short memo/note for a {doc_type} document from vendor '{vendor_name}' \
             for amount {amount}. Return ONLY the memo text, nothing else."
        );

        let request = LlmRequest::new(prompt)
            .with_system(
                "You are an accounting data generator. Return only a single memo line \
                 with no extra text."
                    .to_string(),
            )
            .with_max_tokens(64)
            .with_temperature(0.6);

        match self.provider.complete(&request) {
            Ok(response) => {
                let memo = response.content.trim().to_string();
                if memo.is_empty() {
                    Ok(Self::fallback_memo(doc_type, vendor_name, amount))
                } else {
                    Ok(memo)
                }
            }
            Err(_) => Ok(Self::fallback_memo(doc_type, vendor_name, amount)),
        }
    }

    /// Deterministic fallback description based on context.
    fn fallback_description(
        account_name: &str,
        amount_range: &str,
        industry: &str,
        fiscal_period: u32,
    ) -> String {
        let period_label = match fiscal_period {
            1..=3 => "Q1",
            4..=6 => "Q2",
            7..=9 => "Q3",
            10..=12 => "Q4",
            _ => "period",
        };
        format!(
            "{industry} {account_name} posting - {period_label} {amount_range} operations ({fiscal_period})"
        )
    }

    /// Deterministic fallback memo based on document context.
    fn fallback_memo(doc_type: &str, vendor_name: &str, amount: &str) -> String {
        format!("{doc_type} from {vendor_name} for {amount} - processed per standard policy")
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::llm::MockLlmProvider;

    #[test]
    fn test_enrich_description_returns_nonempty() {
        let provider = Arc::new(MockLlmProvider::new(42));
        let enricher = TransactionLlmEnricher::new(provider);
        let desc = enricher
            .enrich_description("Accounts Payable", "1000-5000", "manufacturing", 3)
            .expect("should succeed");
        assert!(!desc.is_empty(), "description should not be empty");
    }

    #[test]
    fn test_enrich_description_deterministic() {
        let provider = Arc::new(MockLlmProvider::new(42));
        let enricher = TransactionLlmEnricher::new(provider);
        let desc1 = enricher
            .enrich_description("Revenue", "10000-50000", "retail", 6)
            .expect("should succeed");
        let provider2 = Arc::new(MockLlmProvider::new(42));
        let enricher2 = TransactionLlmEnricher::new(provider2);
        let desc2 = enricher2
            .enrich_description("Revenue", "10000-50000", "retail", 6)
            .expect("should succeed");
        assert_eq!(desc1, desc2, "same seed should yield same description");
    }

    #[test]
    fn test_enrich_memo_returns_nonempty() {
        let provider = Arc::new(MockLlmProvider::new(42));
        let enricher = TransactionLlmEnricher::new(provider);
        let memo = enricher
            .enrich_memo("Purchase Order", "Acme Corp", "15000.00")
            .expect("should succeed");
        assert!(!memo.is_empty(), "memo should not be empty");
    }

    #[test]
    fn test_enrich_memo_deterministic() {
        let provider = Arc::new(MockLlmProvider::new(42));
        let enricher = TransactionLlmEnricher::new(provider);
        let memo1 = enricher
            .enrich_memo("Invoice", "Global Trading", "5000.00")
            .expect("should succeed");
        let provider2 = Arc::new(MockLlmProvider::new(42));
        let enricher2 = TransactionLlmEnricher::new(provider2);
        let memo2 = enricher2
            .enrich_memo("Invoice", "Global Trading", "5000.00")
            .expect("should succeed");
        assert_eq!(memo1, memo2, "same seed should yield same memo");
    }

    #[test]
    fn test_fallback_description() {
        let desc = TransactionLlmEnricher::fallback_description(
            "Cost of Goods Sold",
            "5000-10000",
            "manufacturing",
            11,
        );
        assert!(desc.contains("manufacturing"));
        assert!(desc.contains("Cost of Goods Sold"));
        assert!(desc.contains("Q4"));
    }

    #[test]
    fn test_fallback_memo() {
        let memo = TransactionLlmEnricher::fallback_memo("Payment", "Atlas Solutions", "25000.00");
        assert!(memo.contains("Payment"));
        assert!(memo.contains("Atlas Solutions"));
        assert!(memo.contains("25000.00"));
    }
}
