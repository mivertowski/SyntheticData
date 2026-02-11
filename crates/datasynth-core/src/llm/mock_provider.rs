use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use sha2::{Digest, Sha256};

use super::provider::{LlmProvider, LlmRequest, LlmResponse, TokenUsage};
use crate::error::SynthError;

/// Deterministic mock LLM provider for testing.
///
/// Generates responses based on prompt hash + seed for reproducibility.
/// No network calls are made.
pub struct MockLlmProvider {
    seed: u64,
}

impl MockLlmProvider {
    /// Create a new mock provider with the given seed.
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    fn make_rng(&self, request: &LlmRequest) -> ChaCha8Rng {
        let mut hasher = Sha256::new();
        hasher.update(request.prompt.as_bytes());
        if let Some(ref system) = request.system {
            hasher.update(system.as_bytes());
        }
        hasher.update(request.seed.unwrap_or(self.seed).to_le_bytes());
        let hash = hasher.finalize();
        let seed_bytes: [u8; 32] = hash.into();
        ChaCha8Rng::from_seed(seed_bytes)
    }

    fn generate_vendor_name(rng: &mut ChaCha8Rng) -> String {
        const PREFIXES: &[&str] = &[
            "Global", "Pacific", "Atlas", "Vertex", "Nordic", "Prime", "Apex", "Metro", "Summit",
            "Coastal",
        ];
        const TYPES: &[&str] = &[
            "Industries",
            "Solutions",
            "Corp",
            "Systems",
            "Trading",
            "Logistics",
            "Services",
            "Group",
            "Partners",
            "Supply",
        ];
        let prefix = PREFIXES[rng.gen_range(0..PREFIXES.len())];
        let suffix = TYPES[rng.gen_range(0..TYPES.len())];
        format!("{} {}", prefix, suffix)
    }

    fn generate_description(rng: &mut ChaCha8Rng) -> String {
        const DESCRIPTIONS: &[&str] = &[
            "Regular procurement of office supplies and materials",
            "Quarterly maintenance service agreement payment",
            "Professional consulting services for Q4 audit preparation",
            "Equipment lease payment for manufacturing facility",
            "Raw materials procurement for production line alpha",
            "IT infrastructure hosting and cloud services",
            "Employee training and development program costs",
            "Marketing campaign expenses for product launch",
            "Warehouse logistics and distribution services",
            "Annual software license renewal and support",
        ];
        DESCRIPTIONS[rng.gen_range(0..DESCRIPTIONS.len())].to_string()
    }

    fn generate_memo(rng: &mut ChaCha8Rng) -> String {
        const MEMOS: &[&str] = &[
            "Per PO agreement dated last quarter",
            "Approved by department head per policy",
            "Three-way match verified",
            "Recurring monthly charge",
            "Special approval obtained for threshold exception",
            "Year-end accrual adjustment",
            "Intercompany settlement",
            "Variance analysis completed",
        ];
        MEMOS[rng.gen_range(0..MEMOS.len())].to_string()
    }
}

impl LlmProvider for MockLlmProvider {
    fn name(&self) -> &str {
        "mock"
    }

    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, SynthError> {
        let mut rng = self.make_rng(request);
        let prompt_lower = request.prompt.to_lowercase();

        let content = if prompt_lower.contains("vendor") || prompt_lower.contains("supplier") {
            Self::generate_vendor_name(&mut rng)
        } else if prompt_lower.contains("description") || prompt_lower.contains("transaction") {
            Self::generate_description(&mut rng)
        } else if prompt_lower.contains("memo") || prompt_lower.contains("note") {
            Self::generate_memo(&mut rng)
        } else if prompt_lower.contains("anomaly") || prompt_lower.contains("explain") {
            "Unusual transaction pattern detected: amount significantly exceeds historical \
             average for this vendor and period combination."
                .to_string()
        } else {
            // Generic response
            format!(
                "Generated response for: {}",
                &request.prompt[..request.prompt.len().min(50)]
            )
        };

        let input_tokens = (request.prompt.len() / 4) as u32;
        let output_tokens = (content.len() / 4) as u32;

        Ok(LlmResponse {
            content,
            usage: TokenUsage {
                input_tokens,
                output_tokens,
            },
            cached: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_deterministic_same_seed() {
        let provider = MockLlmProvider::new(42);
        let req = LlmRequest::new("Generate a vendor name");
        let r1 = provider.complete(&req).unwrap();
        let r2 = provider.complete(&req).unwrap();
        assert_eq!(r1.content, r2.content);
    }

    #[test]
    fn test_mock_different_seeds_differ() {
        let p1 = MockLlmProvider::new(42);
        let p2 = MockLlmProvider::new(99);
        let req = LlmRequest::new("Generate a vendor name");
        let r1 = p1.complete(&req).unwrap();
        let r2 = p2.complete(&req).unwrap();
        // Different seeds should generally produce different results
        // (not guaranteed but highly likely with these seeds)
        assert_ne!(r1.content, r2.content);
    }

    #[test]
    fn test_mock_vendor_prompt() {
        let provider = MockLlmProvider::new(42);
        let req = LlmRequest::new("Generate a vendor name for manufacturing");
        let resp = provider.complete(&req).unwrap();
        assert!(!resp.content.is_empty());
        assert!(resp.usage.output_tokens > 0);
    }

    #[test]
    fn test_mock_description_prompt() {
        let provider = MockLlmProvider::new(42);
        let req = LlmRequest::new("Generate a transaction description");
        let resp = provider.complete(&req).unwrap();
        assert!(!resp.content.is_empty());
    }

    #[test]
    fn test_mock_batch() {
        let provider = MockLlmProvider::new(42);
        let requests = vec![
            LlmRequest::new("vendor name"),
            LlmRequest::new("transaction description"),
            LlmRequest::new("memo note"),
        ];
        let responses = provider.complete_batch(&requests).unwrap();
        assert_eq!(responses.len(), 3);
    }

    #[test]
    fn test_mock_with_request_seed() {
        let provider = MockLlmProvider::new(42);
        let req1 = LlmRequest::new("vendor name").with_seed(100);
        let req2 = LlmRequest::new("vendor name").with_seed(200);
        let r1 = provider.complete(&req1).unwrap();
        let r2 = provider.complete(&req2).unwrap();
        // Request-level seeds should affect output
        assert_ne!(r1.content, r2.content);
    }
}
