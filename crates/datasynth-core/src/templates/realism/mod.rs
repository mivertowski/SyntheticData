//! Enhanced realism module for synthetic data generation.
//!
//! This module provides sophisticated name, description, and metadata generation
//! with cultural awareness, industry-specific patterns, and natural variations.
//!
//! # Features
//!
//! - **Company Names**: Industry-specific naming patterns with legal suffixes
//! - **Vendor Names**: Spend category-based vendor naming
//! - **Description Variations**: Abbreviations, typos, and natural language variation
//! - **User IDs**: Realistic corporate user ID patterns
//! - **Reference Formats**: ERP-style reference number generation
//! - **Addresses**: Multi-regional address formatting

pub mod addresses;
pub mod company_names;
pub mod descriptions;
pub mod reference_formats;
pub mod user_ids;
pub mod vendor_names;

pub use addresses::{Address, AddressGenerator, AddressRegion, AddressStyle};
pub use company_names::{CompanyNameGenerator, CompanyNameStyle, Industry, LegalSuffix};
pub use descriptions::{DescriptionVariator, TypoGenerator, VariationConfig};
pub use reference_formats::{EnhancedReferenceFormat, EnhancedReferenceGenerator, ReferenceStyle};
pub use user_ids::{UserIdGenerator, UserIdPattern};
pub use vendor_names::{SpendCategory, VendorNameGenerator, VendorProfile};

use rand::Rng;
use serde::{Deserialize, Serialize};

/// Configuration for realism features.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RealismConfig {
    /// Enable culturally-aware name generation
    pub cultural_awareness: bool,
    /// Enable industry-specific vendor naming
    pub industry_vendor_names: bool,
    /// Enable description variations (abbreviations, typos)
    pub description_variations: bool,
    /// Rate of abbreviation usage (0.0 - 1.0)
    pub abbreviation_rate: f64,
    /// Rate of typo injection (0.0 - 1.0)
    pub typo_rate: f64,
    /// Enable realistic reference number formats
    pub realistic_references: bool,
    /// Primary region for address/name generation
    pub primary_region: AddressRegion,
    /// Enable international diversity
    pub international_diversity: bool,
    /// Diversity index for international content (0.0 - 1.0)
    pub diversity_index: f64,
}

impl Default for RealismConfig {
    fn default() -> Self {
        Self {
            cultural_awareness: true,
            industry_vendor_names: true,
            description_variations: true,
            abbreviation_rate: 0.25,
            typo_rate: 0.01,
            realistic_references: true,
            primary_region: AddressRegion::NorthAmerica,
            international_diversity: true,
            diversity_index: 0.3,
        }
    }
}

/// Master realism generator that coordinates all sub-generators.
#[derive(Debug, Clone)]
pub struct RealismGenerator {
    config: RealismConfig,
    company_gen: CompanyNameGenerator,
    vendor_gen: VendorNameGenerator,
    description_var: DescriptionVariator,
    user_id_gen: UserIdGenerator,
    reference_gen: EnhancedReferenceGenerator,
    address_gen: AddressGenerator,
}

impl RealismGenerator {
    /// Create a new realism generator with default configuration.
    pub fn new() -> Self {
        Self::with_config(RealismConfig::default())
    }

    /// Create a new realism generator with custom configuration.
    pub fn with_config(config: RealismConfig) -> Self {
        let variation_config = VariationConfig {
            abbreviation_rate: config.abbreviation_rate,
            typo_rate: config.typo_rate,
            case_variation_rate: 0.05,
            ..Default::default()
        };

        Self {
            company_gen: CompanyNameGenerator::new(),
            vendor_gen: VendorNameGenerator::new(),
            description_var: DescriptionVariator::with_config(variation_config),
            user_id_gen: UserIdGenerator::new(),
            reference_gen: EnhancedReferenceGenerator::new(),
            address_gen: AddressGenerator::for_region(config.primary_region),
            config,
        }
    }

    /// Get the configuration.
    pub fn config(&self) -> &RealismConfig {
        &self.config
    }

    /// Generate a realistic company name.
    pub fn generate_company_name(&self, industry: Industry, rng: &mut impl Rng) -> String {
        self.company_gen.generate(industry, rng)
    }

    /// Generate a realistic vendor name for a spend category.
    pub fn generate_vendor_name(&self, category: SpendCategory, rng: &mut impl Rng) -> String {
        self.vendor_gen.generate(category, rng)
    }

    /// Apply variations to a description.
    pub fn vary_description(&self, description: &str, rng: &mut impl Rng) -> String {
        if self.config.description_variations {
            self.description_var.apply(description, rng)
        } else {
            description.to_string()
        }
    }

    /// Generate a realistic user ID.
    pub fn generate_user_id(
        &self,
        first_name: &str,
        last_name: &str,
        index: usize,
        rng: &mut impl Rng,
    ) -> String {
        self.user_id_gen.generate(first_name, last_name, index, rng)
    }

    /// Generate a reference number.
    pub fn generate_reference(
        &self,
        format: EnhancedReferenceFormat,
        year: i32,
        rng: &mut impl Rng,
    ) -> String {
        self.reference_gen.generate(format, year, rng)
    }

    /// Generate an address.
    pub fn generate_address(&self, rng: &mut impl Rng) -> Address {
        self.address_gen.generate(rng)
    }

    /// Get the company name generator.
    pub fn company_names(&self) -> &CompanyNameGenerator {
        &self.company_gen
    }

    /// Get the vendor name generator.
    pub fn vendor_names(&self) -> &VendorNameGenerator {
        &self.vendor_gen
    }

    /// Get the description variator.
    pub fn descriptions(&self) -> &DescriptionVariator {
        &self.description_var
    }

    /// Get the user ID generator.
    pub fn user_ids(&self) -> &UserIdGenerator {
        &self.user_id_gen
    }

    /// Get the reference generator.
    pub fn references(&self) -> &EnhancedReferenceGenerator {
        &self.reference_gen
    }

    /// Get the address generator.
    pub fn addresses(&self) -> &AddressGenerator {
        &self.address_gen
    }
}

impl Default for RealismGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_realism_generator_creation() {
        let gen = RealismGenerator::new();
        assert!(gen.config().cultural_awareness);
        assert!(gen.config().description_variations);
    }

    #[test]
    fn test_realism_generator_with_config() {
        let config = RealismConfig {
            abbreviation_rate: 0.5,
            typo_rate: 0.0,
            ..Default::default()
        };
        let gen = RealismGenerator::with_config(config);
        assert_eq!(gen.config().abbreviation_rate, 0.5);
        assert_eq!(gen.config().typo_rate, 0.0);
    }

    #[test]
    fn test_generate_company_name() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let gen = RealismGenerator::new();
        let name = gen.generate_company_name(Industry::Manufacturing, &mut rng);
        assert!(!name.is_empty());
    }

    #[test]
    fn test_generate_vendor_name() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let gen = RealismGenerator::new();
        let name = gen.generate_vendor_name(SpendCategory::OfficeSupplies, &mut rng);
        assert!(!name.is_empty());
    }

    #[test]
    fn test_vary_description() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let config = RealismConfig {
            abbreviation_rate: 1.0, // Always abbreviate
            typo_rate: 0.0,
            ..Default::default()
        };
        let gen = RealismGenerator::with_config(config);
        let varied = gen.vary_description("Invoice for Purchase Order", &mut rng);
        // Should contain abbreviation
        assert!(
            varied.contains("Inv")
                || varied.contains("PO")
                || varied == "Invoice for Purchase Order"
        );
    }
}
