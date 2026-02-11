//! Natural language to YAML configuration generator.
//!
//! Takes a free-text description of desired synthetic data (e.g., "Generate 1 year of
//! retail data for a medium US company with fraud detection") and produces a valid
//! `GeneratorConfig` YAML string.

use super::provider::{LlmProvider, LlmRequest};
use crate::error::SynthError;

/// Structured representation of user intent extracted from natural language.
#[derive(Debug, Clone, Default)]
pub struct ConfigIntent {
    /// Target industry (e.g., "retail", "manufacturing", "financial_services").
    pub industry: Option<String>,
    /// Country code (e.g., "US", "DE", "GB").
    pub country: Option<String>,
    /// Company size: "small", "medium", or "large".
    pub company_size: Option<String>,
    /// Duration in months.
    pub period_months: Option<u32>,
    /// Requested feature flags (e.g., "fraud", "audit", "banking", "controls").
    pub features: Vec<String>,
}

/// Generates YAML configuration from natural language descriptions.
///
/// The generator uses a two-phase approach:
/// 1. Parse the natural language description into a structured [`ConfigIntent`].
/// 2. Map the intent to a YAML configuration string using preset templates.
pub struct NlConfigGenerator;

impl NlConfigGenerator {
    /// Generate a YAML configuration from a natural language description.
    ///
    /// Uses the provided LLM provider to help parse the description, with
    /// keyword-based fallback parsing for reliability.
    ///
    /// # Errors
    ///
    /// Returns `SynthError::GenerationError` if the description cannot be parsed
    /// or the resulting configuration is invalid.
    pub fn generate(description: &str, provider: &dyn LlmProvider) -> Result<String, SynthError> {
        if description.trim().is_empty() {
            return Err(SynthError::generation(
                "Natural language description cannot be empty",
            ));
        }

        let intent = Self::parse_intent(description, provider)?;
        Self::intent_to_yaml(&intent)
    }

    /// Parse a natural language description into a structured [`ConfigIntent`].
    ///
    /// Attempts to use the LLM provider first, then falls back to keyword-based
    /// extraction for reliability.
    pub fn parse_intent(
        description: &str,
        provider: &dyn LlmProvider,
    ) -> Result<ConfigIntent, SynthError> {
        // Try LLM-based parsing first
        let llm_intent = Self::parse_with_llm(description, provider);

        // Always run keyword-based parsing as fallback/supplement
        let keyword_intent = Self::parse_with_keywords(description);

        // Merge: prefer LLM results where available, fall back to keywords
        match llm_intent {
            Ok(llm) => Ok(Self::merge_intents(llm, keyword_intent)),
            Err(_) => Ok(keyword_intent),
        }
    }

    /// Map a [`ConfigIntent`] to a YAML configuration string.
    pub fn intent_to_yaml(intent: &ConfigIntent) -> Result<String, SynthError> {
        let industry = intent.industry.as_deref().unwrap_or("manufacturing");
        let country = intent.country.as_deref().unwrap_or("US");
        let complexity = intent.company_size.as_deref().unwrap_or("medium");
        let period_months = intent.period_months.unwrap_or(12);

        // Validate inputs
        if !(1..=120).contains(&period_months) {
            return Err(SynthError::generation(format!(
                "Period months must be between 1 and 120, got {}",
                period_months
            )));
        }

        let valid_complexities = ["small", "medium", "large"];
        if !valid_complexities.contains(&complexity) {
            return Err(SynthError::generation(format!(
                "Invalid company size '{}', must be one of: small, medium, large",
                complexity
            )));
        }

        let currency = Self::country_to_currency(country);
        let company_name = Self::industry_company_name(industry);

        let mut yaml = String::with_capacity(2048);

        // Global settings
        yaml.push_str(&format!(
            "global:\n  industry: {}\n  start_date: \"2024-01-01\"\n  period_months: {}\n  seed: 42\n\n",
            industry, period_months
        ));

        // Companies
        yaml.push_str(&format!(
            "companies:\n  - code: \"C001\"\n    name: \"{}\"\n    currency: \"{}\"\n    country: \"{}\"\n\n",
            company_name, currency, country
        ));

        // Chart of accounts
        yaml.push_str(&format!(
            "chart_of_accounts:\n  complexity: {}\n\n",
            complexity
        ));

        // Transactions
        let tx_count = Self::complexity_to_tx_count(complexity);
        yaml.push_str(&format!(
            "transactions:\n  count: {}\n  anomaly_rate: 0.02\n\n",
            tx_count
        ));

        // Output
        yaml.push_str("output:\n  format: csv\n  compression: false\n\n");

        // Feature-specific sections
        for feature in &intent.features {
            match feature.as_str() {
                "fraud" => {
                    yaml.push_str(
                        "fraud:\n  enabled: true\n  types:\n    - fictitious_transaction\n    - duplicate_payment\n    - split_transaction\n  injection_rate: 0.03\n\n",
                    );
                }
                "audit" => {
                    yaml.push_str(
                        "audit_standards:\n  enabled: true\n  isa_compliance:\n    enabled: true\n    compliance_level: standard\n    framework: isa\n  analytical_procedures:\n    enabled: true\n    procedures_per_account: 3\n  confirmations:\n    enabled: true\n    positive_response_rate: 0.85\n  sox:\n    enabled: true\n    materiality_threshold: 10000.0\n\n",
                    );
                }
                "banking" => {
                    yaml.push_str(
                        "banking:\n  enabled: true\n  customer_count: 100\n  account_types:\n    - checking\n    - savings\n    - loan\n  kyc_enabled: true\n  aml_enabled: true\n\n",
                    );
                }
                "controls" => {
                    yaml.push_str(
                        "internal_controls:\n  enabled: true\n  coso_enabled: true\n  include_entity_level_controls: true\n  target_maturity_level: \"managed\"\n  exception_rate: 0.02\n  sod_violation_rate: 0.01\n\n",
                    );
                }
                "process_mining" => {
                    yaml.push_str(
                        "business_processes:\n  enabled: true\n  ocel_export: true\n  p2p:\n    enabled: true\n  o2c:\n    enabled: true\n\n",
                    );
                }
                "intercompany" => {
                    yaml.push_str(
                        "intercompany:\n  enabled: true\n  matching_tolerance: 0.01\n  elimination_enabled: true\n\n",
                    );
                }
                "distributions" => {
                    yaml.push_str(&format!(
                        "distributions:\n  enabled: true\n  industry_profile: {}\n  amounts:\n    enabled: true\n    distribution_type: lognormal\n    benford_compliance: true\n\n",
                        industry
                    ));
                }
                _ => {} // Unknown features are silently ignored
            }
        }

        Ok(yaml)
    }

    /// Attempt LLM-based parsing of the description.
    fn parse_with_llm(
        description: &str,
        provider: &dyn LlmProvider,
    ) -> Result<ConfigIntent, SynthError> {
        let system_prompt = "You are a configuration parser. Extract structured fields from a natural language description of desired synthetic data generation. Return ONLY a JSON object with these fields: industry (string or null), country (string or null), company_size (string or null), period_months (number or null), features (array of strings). Valid industries: retail, manufacturing, financial_services, healthcare, technology. Valid sizes: small, medium, large. Valid features: fraud, audit, banking, controls, process_mining, intercompany, distributions.";

        let request = LlmRequest::new(description)
            .with_system(system_prompt.to_string())
            .with_temperature(0.1)
            .with_max_tokens(512);

        let response = provider.complete(&request)?;
        Self::parse_llm_response(&response.content)
    }

    /// Parse the LLM response JSON into a ConfigIntent.
    fn parse_llm_response(content: &str) -> Result<ConfigIntent, SynthError> {
        // Try to find JSON in the response
        let json_str = Self::extract_json(content)
            .ok_or_else(|| SynthError::generation("No JSON found in LLM response"))?;

        let value: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| SynthError::generation(format!("Failed to parse LLM JSON: {}", e)))?;

        let industry = value
            .get("industry")
            .and_then(|v| v.as_str())
            .map(String::from);
        let country = value
            .get("country")
            .and_then(|v| v.as_str())
            .map(String::from);
        let company_size = value
            .get("company_size")
            .and_then(|v| v.as_str())
            .map(String::from);
        let period_months = value
            .get("period_months")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);
        let features = value
            .get("features")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(ConfigIntent {
            industry,
            country,
            company_size,
            period_months,
            features,
        })
    }

    /// Extract a JSON object substring from potentially noisy LLM output.
    fn extract_json(content: &str) -> Option<&str> {
        // Find the first '{' and matching '}'
        let start = content.find('{')?;
        let mut depth = 0i32;
        for (i, ch) in content[start..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(&content[start..start + i + 1]);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Keyword-based parsing as a reliable fallback.
    fn parse_with_keywords(description: &str) -> ConfigIntent {
        let lower = description.to_lowercase();

        let industry = Self::extract_industry(&lower);
        let country = Self::extract_country(&lower);
        let company_size = Self::extract_size(&lower);
        let period_months = Self::extract_period(&lower);
        let features = Self::extract_features(&lower);

        ConfigIntent {
            industry,
            country,
            company_size,
            period_months,
            features,
        }
    }

    /// Extract industry from lowercased text.
    ///
    /// Uses a scoring approach: each industry gets points for keyword matches,
    /// and the highest-scoring industry wins. This avoids order-dependent
    /// issues where "banking" in a feature context incorrectly triggers
    /// "financial_services" over "technology".
    fn extract_industry(text: &str) -> Option<String> {
        let patterns: &[(&[&str], &str)] = &[
            (
                &["retail", "store", "shop", "e-commerce", "ecommerce"],
                "retail",
            ),
            (
                &["manufactur", "factory", "production", "assembly"],
                "manufacturing",
            ),
            (
                &[
                    "financial",
                    "finance",
                    "insurance",
                    "fintech",
                    "investment firm",
                ],
                "financial_services",
            ),
            (
                &["health", "hospital", "medical", "pharma", "clinic"],
                "healthcare",
            ),
            (
                &["tech", "software", "saas", "startup", "digital"],
                "technology",
            ),
        ];

        let mut best: Option<(&str, usize)> = None;
        for (keywords, industry) in patterns {
            let count = keywords.iter().filter(|kw| text.contains(*kw)).count();
            if count > 0 && (best.is_none() || count > best.expect("checked is_some").1) {
                best = Some((industry, count));
            }
        }
        best.map(|(industry, _)| industry.to_string())
    }

    /// Extract country from lowercased text.
    fn extract_country(text: &str) -> Option<String> {
        // Check full country names first (most reliable), then short codes.
        // Short codes like "in", "de", "us" can clash with English words,
        // so we only use unambiguous short codes.
        let name_patterns = [
            (&["united states", "u.s.", "america"][..], "US"),
            (&["germany", "german"][..], "DE"),
            (&["united kingdom", "british", "england"][..], "GB"),
            (&["china", "chinese"][..], "CN"),
            (&["japan", "japanese"][..], "JP"),
            (&["india", "indian"][..], "IN"),
            (&["brazil", "brazilian"][..], "BR"),
            (&["mexico", "mexican"][..], "MX"),
            (&["australia", "australian"][..], "AU"),
            (&["singapore", "singaporean"][..], "SG"),
            (&["korea", "korean"][..], "KR"),
            (&["france", "french"][..], "FR"),
            (&["canada", "canadian"][..], "CA"),
        ];

        for (keywords, code) in &name_patterns {
            if keywords.iter().any(|kw| text.contains(kw)) {
                return Some(code.to_string());
            }
        }

        // Fall back to short codes (padded with spaces).
        // Excluded: "in" (India - clashes with preposition "in"),
        //           "de" (Germany - clashes with various uses).
        let padded = format!(" {} ", text);
        let safe_codes = [
            (" us ", "US"),
            (" uk ", "GB"),
            (" gb ", "GB"),
            (" cn ", "CN"),
            (" jp ", "JP"),
            (" br ", "BR"),
            (" mx ", "MX"),
            (" au ", "AU"),
            (" sg ", "SG"),
            (" kr ", "KR"),
            (" fr ", "FR"),
            (" ca ", "CA"),
        ];

        for (code_pattern, code) in &safe_codes {
            if padded.contains(code_pattern) {
                return Some(code.to_string());
            }
        }

        None
    }

    /// Extract company size from lowercased text.
    fn extract_size(text: &str) -> Option<String> {
        if text.contains("small") || text.contains("startup") || text.contains("tiny") {
            Some("small".to_string())
        } else if text.contains("large")
            || text.contains("enterprise")
            || text.contains("big")
            || text.contains("multinational")
            || text.contains("fortune 500")
        {
            Some("large".to_string())
        } else if text.contains("medium")
            || text.contains("mid-size")
            || text.contains("midsize")
            || text.contains("mid size")
        {
            Some("medium".to_string())
        } else {
            None
        }
    }

    /// Extract period in months from lowercased text.
    fn extract_period(text: &str) -> Option<u32> {
        // Match patterns like "1 year", "2 years", "6 months", "18 months"
        // Also handle "one year", "two years", etc.
        let word_numbers = [
            ("one", 1u32),
            ("two", 2),
            ("three", 3),
            ("four", 4),
            ("five", 5),
            ("six", 6),
            ("twelve", 12),
            ("eighteen", 18),
            ("twenty-four", 24),
        ];

        // Try "N year(s)" pattern
        for (word, num) in &word_numbers {
            if text.contains(&format!("{} year", word)) {
                return Some(num * 12);
            }
            if text.contains(&format!("{} month", word)) {
                return Some(*num);
            }
        }

        // Try numeric patterns: "N year(s)", "N month(s)"
        let tokens: Vec<&str> = text.split_whitespace().collect();
        for window in tokens.windows(2) {
            if let Ok(num) = window[0].parse::<u32>() {
                if window[1].starts_with("year") {
                    return Some(num * 12);
                }
                if window[1].starts_with("month") {
                    return Some(num);
                }
            }
        }

        None
    }

    /// Extract feature flags from lowercased text.
    fn extract_features(text: &str) -> Vec<String> {
        let mut features = Vec::new();

        let feature_patterns = [
            (&["fraud", "fraudulent", "suspicious"][..], "fraud"),
            (&["audit", "auditing", "assurance"][..], "audit"),
            (&["banking", "bank account", "kyc", "aml"][..], "banking"),
            (
                &["control", "sox", "sod", "segregation of duties", "coso"][..],
                "controls",
            ),
            (
                &["process mining", "ocel", "event log"][..],
                "process_mining",
            ),
            (
                &["intercompany", "inter-company", "consolidation"][..],
                "intercompany",
            ),
            (
                &["distribution", "benford", "statistical"][..],
                "distributions",
            ),
        ];

        for (keywords, feature) in &feature_patterns {
            if keywords.iter().any(|kw| text.contains(kw)) {
                features.push(feature.to_string());
            }
        }

        features
    }

    /// Merge two ConfigIntents, preferring the primary where available.
    fn merge_intents(primary: ConfigIntent, fallback: ConfigIntent) -> ConfigIntent {
        ConfigIntent {
            industry: primary.industry.or(fallback.industry),
            country: primary.country.or(fallback.country),
            company_size: primary.company_size.or(fallback.company_size),
            period_months: primary.period_months.or(fallback.period_months),
            features: if primary.features.is_empty() {
                fallback.features
            } else {
                primary.features
            },
        }
    }

    /// Map country code to default currency.
    fn country_to_currency(country: &str) -> &'static str {
        match country {
            "US" | "CA" => "USD",
            "DE" | "FR" => "EUR",
            "GB" => "GBP",
            "CN" => "CNY",
            "JP" => "JPY",
            "IN" => "INR",
            "BR" => "BRL",
            "MX" => "MXN",
            "AU" => "AUD",
            "SG" => "SGD",
            "KR" => "KRW",
            _ => "USD",
        }
    }

    /// Generate a company name based on industry.
    fn industry_company_name(industry: &str) -> &'static str {
        match industry {
            "retail" => "Retail Corp",
            "manufacturing" => "Manufacturing Industries Inc",
            "financial_services" => "Financial Services Group",
            "healthcare" => "HealthCare Solutions",
            "technology" => "TechCorp Solutions",
            _ => "DataSynth Corp",
        }
    }

    /// Map complexity to an appropriate transaction count.
    fn complexity_to_tx_count(complexity: &str) -> u32 {
        match complexity {
            "small" => 1000,
            "medium" => 5000,
            "large" => 25000,
            _ => 5000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::mock_provider::MockLlmProvider;

    #[test]
    fn test_parse_retail_description() {
        let provider = MockLlmProvider::new(42);
        let intent = NlConfigGenerator::parse_intent(
            "Generate 1 year of retail data for a medium US company",
            &provider,
        )
        .expect("should parse successfully");

        assert_eq!(intent.industry, Some("retail".to_string()));
        assert_eq!(intent.country, Some("US".to_string()));
        assert_eq!(intent.company_size, Some("medium".to_string()));
        assert_eq!(intent.period_months, Some(12));
    }

    #[test]
    fn test_parse_manufacturing_with_fraud() {
        let provider = MockLlmProvider::new(42);
        let intent = NlConfigGenerator::parse_intent(
            "Create 6 months of manufacturing data for a large German company with fraud detection",
            &provider,
        )
        .expect("should parse successfully");

        assert_eq!(intent.industry, Some("manufacturing".to_string()));
        assert_eq!(intent.country, Some("DE".to_string()));
        assert_eq!(intent.company_size, Some("large".to_string()));
        assert_eq!(intent.period_months, Some(6));
        assert!(intent.features.contains(&"fraud".to_string()));
    }

    #[test]
    fn test_parse_financial_services_with_audit() {
        let provider = MockLlmProvider::new(42);
        let intent = NlConfigGenerator::parse_intent(
            "I need 2 years of financial services data for audit testing with SOX controls",
            &provider,
        )
        .expect("should parse successfully");

        assert_eq!(intent.industry, Some("financial_services".to_string()));
        assert_eq!(intent.period_months, Some(24));
        assert!(intent.features.contains(&"audit".to_string()));
        assert!(intent.features.contains(&"controls".to_string()));
    }

    #[test]
    fn test_parse_healthcare_small() {
        let provider = MockLlmProvider::new(42);
        let intent = NlConfigGenerator::parse_intent(
            "Small healthcare company in Japan, 3 months of data",
            &provider,
        )
        .expect("should parse successfully");

        assert_eq!(intent.industry, Some("healthcare".to_string()));
        assert_eq!(intent.country, Some("JP".to_string()));
        assert_eq!(intent.company_size, Some("small".to_string()));
        assert_eq!(intent.period_months, Some(3));
    }

    #[test]
    fn test_parse_technology_with_banking() {
        let provider = MockLlmProvider::new(42);
        let intent = NlConfigGenerator::parse_intent(
            "Generate data for a technology startup in Singapore with banking and KYC",
            &provider,
        )
        .expect("should parse successfully");

        assert_eq!(intent.industry, Some("technology".to_string()));
        assert_eq!(intent.country, Some("SG".to_string()));
        assert_eq!(intent.company_size, Some("small".to_string()));
        assert!(intent.features.contains(&"banking".to_string()));
    }

    #[test]
    fn test_parse_word_numbers() {
        let provider = MockLlmProvider::new(42);
        let intent =
            NlConfigGenerator::parse_intent("Generate two years of retail data", &provider)
                .expect("should parse successfully");

        assert_eq!(intent.period_months, Some(24));
    }

    #[test]
    fn test_parse_multiple_features() {
        let provider = MockLlmProvider::new(42);
        let intent = NlConfigGenerator::parse_intent(
            "Manufacturing data with fraud detection, audit trail, process mining, and intercompany consolidation",
            &provider,
        )
        .expect("should parse successfully");

        assert_eq!(intent.industry, Some("manufacturing".to_string()));
        assert!(intent.features.contains(&"fraud".to_string()));
        assert!(intent.features.contains(&"audit".to_string()));
        assert!(intent.features.contains(&"process_mining".to_string()));
        assert!(intent.features.contains(&"intercompany".to_string()));
    }

    #[test]
    fn test_intent_to_yaml_basic() {
        let intent = ConfigIntent {
            industry: Some("retail".to_string()),
            country: Some("US".to_string()),
            company_size: Some("medium".to_string()),
            period_months: Some(12),
            features: vec![],
        };

        let yaml = NlConfigGenerator::intent_to_yaml(&intent).expect("should generate YAML");

        assert!(yaml.contains("industry: retail"));
        assert!(yaml.contains("period_months: 12"));
        assert!(yaml.contains("currency: \"USD\""));
        assert!(yaml.contains("country: \"US\""));
        assert!(yaml.contains("complexity: medium"));
        assert!(yaml.contains("count: 5000"));
    }

    #[test]
    fn test_intent_to_yaml_with_features() {
        let intent = ConfigIntent {
            industry: Some("manufacturing".to_string()),
            country: Some("DE".to_string()),
            company_size: Some("large".to_string()),
            period_months: Some(24),
            features: vec![
                "fraud".to_string(),
                "audit".to_string(),
                "controls".to_string(),
            ],
        };

        let yaml = NlConfigGenerator::intent_to_yaml(&intent).expect("should generate YAML");

        assert!(yaml.contains("industry: manufacturing"));
        assert!(yaml.contains("currency: \"EUR\""));
        assert!(yaml.contains("complexity: large"));
        assert!(yaml.contains("count: 25000"));
        assert!(yaml.contains("fraud:"));
        assert!(yaml.contains("audit_standards:"));
        assert!(yaml.contains("internal_controls:"));
    }

    #[test]
    fn test_intent_to_yaml_defaults() {
        let intent = ConfigIntent::default();

        let yaml = NlConfigGenerator::intent_to_yaml(&intent).expect("should generate YAML");

        // Should use defaults
        assert!(yaml.contains("industry: manufacturing"));
        assert!(yaml.contains("period_months: 12"));
        assert!(yaml.contains("complexity: medium"));
    }

    #[test]
    fn test_intent_to_yaml_invalid_period() {
        let intent = ConfigIntent {
            period_months: Some(0),
            ..ConfigIntent::default()
        };

        let result = NlConfigGenerator::intent_to_yaml(&intent);
        assert!(result.is_err());

        let intent = ConfigIntent {
            period_months: Some(121),
            ..ConfigIntent::default()
        };

        let result = NlConfigGenerator::intent_to_yaml(&intent);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_end_to_end() {
        let provider = MockLlmProvider::new(42);
        let yaml = NlConfigGenerator::generate(
            "Generate 1 year of retail data for a medium US company with fraud detection",
            &provider,
        )
        .expect("should generate YAML");

        assert!(yaml.contains("industry: retail"));
        assert!(yaml.contains("period_months: 12"));
        assert!(yaml.contains("currency: \"USD\""));
        assert!(yaml.contains("fraud:"));
        assert!(yaml.contains("complexity: medium"));
    }

    #[test]
    fn test_generate_empty_description() {
        let provider = MockLlmProvider::new(42);
        let result = NlConfigGenerator::generate("", &provider);
        assert!(result.is_err());

        let result = NlConfigGenerator::generate("   ", &provider);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_json_from_response() {
        let content = r#"Here is the parsed output: {"industry": "retail", "country": "US"} done"#;
        let json = NlConfigGenerator::extract_json(content);
        assert!(json.is_some());
        assert_eq!(
            json.expect("json should be present"),
            r#"{"industry": "retail", "country": "US"}"#
        );
    }

    #[test]
    fn test_extract_json_nested() {
        let content = r#"{"industry": "retail", "features": ["fraud", "audit"]}"#;
        let json = NlConfigGenerator::extract_json(content);
        assert!(json.is_some());
    }

    #[test]
    fn test_extract_json_missing() {
        let content = "No JSON here at all";
        let json = NlConfigGenerator::extract_json(content);
        assert!(json.is_none());
    }

    #[test]
    fn test_parse_llm_response_valid() {
        let content = r#"{"industry": "retail", "country": "US", "company_size": "medium", "period_months": 12, "features": ["fraud"]}"#;
        let intent =
            NlConfigGenerator::parse_llm_response(content).expect("should parse valid JSON");

        assert_eq!(intent.industry, Some("retail".to_string()));
        assert_eq!(intent.country, Some("US".to_string()));
        assert_eq!(intent.company_size, Some("medium".to_string()));
        assert_eq!(intent.period_months, Some(12));
        assert_eq!(intent.features, vec!["fraud".to_string()]);
    }

    #[test]
    fn test_parse_llm_response_partial() {
        let content = r#"{"industry": "retail"}"#;
        let intent =
            NlConfigGenerator::parse_llm_response(content).expect("should parse partial JSON");

        assert_eq!(intent.industry, Some("retail".to_string()));
        assert_eq!(intent.country, None);
        assert!(intent.features.is_empty());
    }

    #[test]
    fn test_country_to_currency_mapping() {
        assert_eq!(NlConfigGenerator::country_to_currency("US"), "USD");
        assert_eq!(NlConfigGenerator::country_to_currency("DE"), "EUR");
        assert_eq!(NlConfigGenerator::country_to_currency("GB"), "GBP");
        assert_eq!(NlConfigGenerator::country_to_currency("JP"), "JPY");
        assert_eq!(NlConfigGenerator::country_to_currency("CN"), "CNY");
        assert_eq!(NlConfigGenerator::country_to_currency("BR"), "BRL");
        assert_eq!(NlConfigGenerator::country_to_currency("XX"), "USD"); // Unknown defaults to USD
    }

    #[test]
    fn test_merge_intents() {
        let primary = ConfigIntent {
            industry: Some("retail".to_string()),
            country: None,
            company_size: None,
            period_months: Some(12),
            features: vec![],
        };
        let fallback = ConfigIntent {
            industry: Some("manufacturing".to_string()),
            country: Some("DE".to_string()),
            company_size: Some("large".to_string()),
            period_months: Some(6),
            features: vec!["fraud".to_string()],
        };

        let merged = NlConfigGenerator::merge_intents(primary, fallback);
        assert_eq!(merged.industry, Some("retail".to_string())); // primary wins
        assert_eq!(merged.country, Some("DE".to_string())); // fallback fills gap
        assert_eq!(merged.company_size, Some("large".to_string())); // fallback fills gap
        assert_eq!(merged.period_months, Some(12)); // primary wins
        assert_eq!(merged.features, vec!["fraud".to_string()]); // fallback since primary empty
    }

    #[test]
    fn test_parse_uk_country() {
        let provider = MockLlmProvider::new(42);
        let intent = NlConfigGenerator::parse_intent(
            "Generate data for a UK manufacturing company",
            &provider,
        )
        .expect("should parse successfully");

        assert_eq!(intent.country, Some("GB".to_string()));
    }

    #[test]
    fn test_intent_to_yaml_banking_feature() {
        let intent = ConfigIntent {
            industry: Some("financial_services".to_string()),
            country: Some("US".to_string()),
            company_size: Some("large".to_string()),
            period_months: Some(12),
            features: vec!["banking".to_string()],
        };

        let yaml = NlConfigGenerator::intent_to_yaml(&intent).expect("should generate YAML");

        assert!(yaml.contains("banking:"));
        assert!(yaml.contains("kyc_enabled: true"));
        assert!(yaml.contains("aml_enabled: true"));
    }

    #[test]
    fn test_intent_to_yaml_process_mining_feature() {
        let intent = ConfigIntent {
            features: vec!["process_mining".to_string()],
            ..ConfigIntent::default()
        };

        let yaml = NlConfigGenerator::intent_to_yaml(&intent).expect("should generate YAML");

        assert!(yaml.contains("business_processes:"));
        assert!(yaml.contains("ocel_export: true"));
    }

    #[test]
    fn test_intent_to_yaml_distributions_feature() {
        let intent = ConfigIntent {
            industry: Some("retail".to_string()),
            features: vec!["distributions".to_string()],
            ..ConfigIntent::default()
        };

        let yaml = NlConfigGenerator::intent_to_yaml(&intent).expect("should generate YAML");

        assert!(yaml.contains("distributions:"));
        assert!(yaml.contains("industry_profile: retail"));
        assert!(yaml.contains("benford_compliance: true"));
    }
}
