//! LLM-powered vendor name enrichment.
//!
//! Generates realistic vendor names using an LLM provider, with deterministic
//! template-based fallbacks when the provider is unavailable or returns errors.

use std::sync::Arc;

use datasynth_core::error::SynthError;
use datasynth_core::llm::{LlmProvider, LlmRequest};

/// Enriches vendor metadata using an LLM provider.
///
/// Wraps a `dyn LlmProvider` to generate realistic vendor names based on
/// industry, spend category, and country context. Falls back to template-based
/// names when the LLM call fails.
pub struct VendorLlmEnricher {
    provider: Arc<dyn LlmProvider>,
}

impl VendorLlmEnricher {
    /// Create a new enricher with the given LLM provider.
    pub fn new(provider: Arc<dyn LlmProvider>) -> Self {
        Self { provider }
    }

    /// Generate a realistic vendor name for the given context.
    ///
    /// The LLM is prompted with the industry, spend category, and country to
    /// produce a plausible vendor name. If the LLM call fails, a template-based
    /// fallback name is returned.
    pub fn enrich_vendor_name(
        &self,
        industry: &str,
        spend_category: &str,
        country: &str,
    ) -> Result<String, SynthError> {
        let prompt = format!(
            "Generate a single realistic vendor/supplier company name for a {} company \
             in {} that provides {}. Return ONLY the company name, nothing else.",
            industry, country, spend_category
        );

        let request = LlmRequest::new(prompt)
            .with_system(
                "You are a business data generator. Return only a single company name \
                 with no explanation or extra text."
                    .to_string(),
            )
            .with_max_tokens(64)
            .with_temperature(0.8);

        match self.provider.complete(&request) {
            Ok(response) => {
                let name = response.content.trim().to_string();
                if name.is_empty() {
                    Ok(Self::fallback_vendor_name(
                        industry,
                        spend_category,
                        country,
                    ))
                } else {
                    Ok(name)
                }
            }
            Err(_) => Ok(Self::fallback_vendor_name(
                industry,
                spend_category,
                country,
            )),
        }
    }

    /// Generate vendor names in batch.
    ///
    /// Each tuple in `requests` contains `(industry, spend_category, country)`.
    /// A deterministic seed is applied to each request for reproducibility.
    pub fn enrich_batch(
        &self,
        requests: &[(String, String, String)],
        seed: u64,
    ) -> Result<Vec<String>, SynthError> {
        let llm_requests: Vec<LlmRequest> = requests
            .iter()
            .enumerate()
            .map(|(i, (industry, spend_category, country))| {
                let prompt = format!(
                    "Generate a single realistic vendor/supplier company name for a {} company \
                     in {} that provides {}. Return ONLY the company name, nothing else.",
                    industry, country, spend_category
                );
                LlmRequest::new(prompt)
                    .with_system(
                        "You are a business data generator. Return only a single company name \
                         with no explanation or extra text."
                            .to_string(),
                    )
                    .with_max_tokens(64)
                    .with_temperature(0.8)
                    .with_seed(seed.wrapping_add(i as u64))
            })
            .collect();

        match self.provider.complete_batch(&llm_requests) {
            Ok(responses) => {
                let names: Vec<String> = responses
                    .iter()
                    .enumerate()
                    .map(|(i, resp)| {
                        let name = resp.content.trim().to_string();
                        if name.is_empty() {
                            let (ref ind, ref cat, ref cty) = requests[i];
                            Self::fallback_vendor_name(ind, cat, cty)
                        } else {
                            name
                        }
                    })
                    .collect();
                Ok(names)
            }
            Err(_) => {
                // Fall back to template names for the entire batch
                let names = requests
                    .iter()
                    .map(|(ind, cat, cty)| Self::fallback_vendor_name(ind, cat, cty))
                    .collect();
                Ok(names)
            }
        }
    }

    /// Deterministic fallback vendor name based on context parameters.
    fn fallback_vendor_name(industry: &str, spend_category: &str, country: &str) -> String {
        let industry_prefix = match industry.to_lowercase().as_str() {
            "manufacturing" => "Industrial",
            "retail" => "Retail",
            "financial_services" | "finance" => "Financial",
            "healthcare" => "Medical",
            "technology" => "Tech",
            _ => "Global",
        };

        let category_suffix = match spend_category.to_lowercase().as_str() {
            "office_supplies" | "office supplies" => "Office Supply Co",
            "raw_materials" | "raw materials" => "Materials Corp",
            "it_services" | "it services" | "technology" => "Systems Inc",
            "consulting" => "Consulting Group",
            "logistics" | "transportation" => "Logistics Ltd",
            "maintenance" => "Maintenance Services",
            "marketing" => "Marketing Partners",
            _ => "Solutions LLC",
        };

        let country_tag = match country.to_uppercase().as_str() {
            "US" | "USA" => "",
            "DE" | "GERMANY" => " GmbH",
            "GB" | "UK" => " PLC",
            "JP" | "JAPAN" => " KK",
            "CN" | "CHINA" => " Ltd (CN)",
            _ => " Intl",
        };

        format!("{} {}{}", industry_prefix, category_suffix, country_tag)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::llm::MockLlmProvider;

    #[test]
    fn test_enrich_vendor_name_returns_nonempty() {
        let provider = Arc::new(MockLlmProvider::new(42));
        let enricher = VendorLlmEnricher::new(provider);
        let name = enricher
            .enrich_vendor_name("manufacturing", "raw_materials", "US")
            .expect("should succeed");
        assert!(!name.is_empty(), "vendor name should not be empty");
    }

    #[test]
    fn test_enrich_vendor_name_deterministic() {
        let provider = Arc::new(MockLlmProvider::new(42));
        let enricher = VendorLlmEnricher::new(provider);
        let name1 = enricher
            .enrich_vendor_name("retail", "office_supplies", "DE")
            .expect("should succeed");
        let provider2 = Arc::new(MockLlmProvider::new(42));
        let enricher2 = VendorLlmEnricher::new(provider2);
        let name2 = enricher2
            .enrich_vendor_name("retail", "office_supplies", "DE")
            .expect("should succeed");
        assert_eq!(name1, name2, "same seed should yield same name");
    }

    #[test]
    fn test_enrich_batch() {
        let provider = Arc::new(MockLlmProvider::new(42));
        let enricher = VendorLlmEnricher::new(provider);
        let requests = vec![
            ("manufacturing".into(), "raw_materials".into(), "US".into()),
            ("retail".into(), "office_supplies".into(), "DE".into()),
            ("technology".into(), "it_services".into(), "GB".into()),
        ];
        let names = enricher
            .enrich_batch(&requests, 100)
            .expect("batch should succeed");
        assert_eq!(names.len(), 3);
        for name in &names {
            assert!(!name.is_empty(), "each name should be non-empty");
        }
    }

    #[test]
    fn test_fallback_vendor_name_manufacturing() {
        let name = VendorLlmEnricher::fallback_vendor_name("manufacturing", "raw_materials", "US");
        assert_eq!(name, "Industrial Materials Corp");
    }

    #[test]
    fn test_fallback_vendor_name_with_country_suffix() {
        let name = VendorLlmEnricher::fallback_vendor_name("technology", "consulting", "DE");
        assert_eq!(name, "Tech Consulting Group GmbH");
    }

    #[test]
    fn test_fallback_vendor_name_unknown_category() {
        let name = VendorLlmEnricher::fallback_vendor_name("healthcare", "misc", "JP");
        assert_eq!(name, "Medical Solutions LLC KK");
    }
}
