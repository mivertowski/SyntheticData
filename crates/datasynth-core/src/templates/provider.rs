//! Template provider trait and implementations.
//!
//! This module defines the `TemplateProvider` trait for accessing template data,
//! along with implementations that combine embedded and file-based templates.

use rand::seq::IndexedRandom;
use rand::RngCore;
use std::sync::Arc;

use super::loader::{MergeStrategy, TemplateData, TemplateLoader};
use super::names::NameCulture;
use crate::models::BusinessProcess;

/// Trait for providing template data to generators.
///
/// This trait abstracts the source of template data, allowing generators
/// to work with either embedded templates, file-based templates, or a
/// combination of both.
///
/// Methods use `&mut dyn RngCore` to allow the trait to be dyn-compatible.
pub trait TemplateProvider: Send + Sync {
    /// Get a random person first name for the given culture and gender.
    fn get_person_first_name(
        &self,
        culture: NameCulture,
        is_male: bool,
        rng: &mut dyn RngCore,
    ) -> String;

    /// Get a random person last name for the given culture.
    fn get_person_last_name(&self, culture: NameCulture, rng: &mut dyn RngCore) -> String;

    /// Get a random vendor name for the given category.
    fn get_vendor_name(&self, category: &str, rng: &mut dyn RngCore) -> String;

    /// Get a random customer name for the given industry.
    fn get_customer_name(&self, industry: &str, rng: &mut dyn RngCore) -> String;

    /// Get a random material description for the given type.
    fn get_material_description(&self, material_type: &str, rng: &mut dyn RngCore) -> String;

    /// Get a random asset description for the given category.
    fn get_asset_description(&self, category: &str, rng: &mut dyn RngCore) -> String;

    /// Get a random line text for the given process and account type.
    fn get_line_text(
        &self,
        process: BusinessProcess,
        account_type: &str,
        rng: &mut dyn RngCore,
    ) -> String;

    /// Get a random header text template for the given process.
    fn get_header_template(&self, process: BusinessProcess, rng: &mut dyn RngCore) -> String;
}

/// Default template provider using embedded templates with optional file overrides.
pub struct DefaultTemplateProvider {
    /// Loaded template data (file-based)
    template_data: Option<TemplateData>,
    /// Merge strategy for combining embedded and file templates
    merge_strategy: MergeStrategy,
}

impl DefaultTemplateProvider {
    /// Create a new provider with embedded templates only.
    pub fn new() -> Self {
        Self {
            template_data: None,
            merge_strategy: MergeStrategy::Extend,
        }
    }

    /// Create a provider with file-based templates.
    pub fn with_templates(template_data: TemplateData, strategy: MergeStrategy) -> Self {
        Self {
            template_data: Some(template_data),
            merge_strategy: strategy,
        }
    }

    /// Load templates from a file path.
    pub fn from_file(path: &std::path::Path) -> Result<Self, super::loader::TemplateError> {
        let data = TemplateLoader::load_from_file(path)?;
        Ok(Self::with_templates(data, MergeStrategy::Extend))
    }

    /// Load templates from a directory.
    pub fn from_directory(path: &std::path::Path) -> Result<Self, super::loader::TemplateError> {
        let data = TemplateLoader::load_from_directory(path)?;
        Ok(Self::with_templates(data, MergeStrategy::Extend))
    }

    /// Set the merge strategy.
    pub fn with_merge_strategy(mut self, strategy: MergeStrategy) -> Self {
        self.merge_strategy = strategy;
        self
    }

    /// Get embedded German first names (sample).
    fn embedded_german_first_names_male() -> Vec<&'static str> {
        vec![
            "Hans", "Klaus", "Wolfgang", "Dieter", "Michael", "Stefan", "Thomas", "Andreas",
            "Peter", "Jürgen", "Matthias", "Frank", "Martin", "Bernd",
        ]
    }

    fn embedded_german_first_names_female() -> Vec<&'static str> {
        vec![
            "Anna",
            "Maria",
            "Elisabeth",
            "Ursula",
            "Monika",
            "Petra",
            "Karin",
            "Sabine",
            "Andrea",
            "Christine",
            "Gabriele",
            "Heike",
            "Birgit",
        ]
    }

    fn embedded_german_last_names() -> Vec<&'static str> {
        vec![
            "Müller",
            "Schmidt",
            "Schneider",
            "Fischer",
            "Weber",
            "Meyer",
            "Wagner",
            "Becker",
            "Schulz",
            "Hoffmann",
            "Schäfer",
            "Koch",
            "Bauer",
            "Richter",
        ]
    }

    fn embedded_us_first_names_male() -> Vec<&'static str> {
        vec![
            "James",
            "John",
            "Robert",
            "Michael",
            "William",
            "David",
            "Richard",
            "Joseph",
            "Thomas",
            "Charles",
            "Christopher",
            "Daniel",
            "Matthew",
        ]
    }

    fn embedded_us_first_names_female() -> Vec<&'static str> {
        vec![
            "Mary",
            "Patricia",
            "Jennifer",
            "Linda",
            "Barbara",
            "Elizabeth",
            "Susan",
            "Jessica",
            "Sarah",
            "Karen",
            "Lisa",
            "Nancy",
            "Betty",
            "Margaret",
        ]
    }

    fn embedded_us_last_names() -> Vec<&'static str> {
        vec![
            "Smith",
            "Johnson",
            "Williams",
            "Brown",
            "Jones",
            "Garcia",
            "Miller",
            "Davis",
            "Rodriguez",
            "Martinez",
            "Hernandez",
            "Lopez",
            "Gonzalez",
        ]
    }

    fn embedded_vendor_names_manufacturing() -> Vec<&'static str> {
        vec![
            "Precision Parts Inc.",
            "Industrial Components LLC",
            "Advanced Materials Corp.",
            "Steel Solutions GmbH",
            "Quality Fasteners Ltd.",
            "Machining Excellence Inc.",
        ]
    }

    fn embedded_vendor_names_services() -> Vec<&'static str> {
        vec![
            "Consulting Partners LLP",
            "Technical Services Inc.",
            "Professional Solutions LLC",
            "Business Advisory Group",
            "Strategic Consulting Co.",
            "Expert Services Ltd.",
        ]
    }

    fn embedded_customer_names_automotive() -> Vec<&'static str> {
        vec![
            "AutoWerke Industries",
            "Vehicle Tech Solutions",
            "Motor Parts Direct",
            "Automotive Excellence Corp.",
            "Drive Systems Inc.",
            "Engine Components Ltd.",
        ]
    }

    fn embedded_customer_names_retail() -> Vec<&'static str> {
        vec![
            "Retail Solutions Corp.",
            "Consumer Goods Direct",
            "Shop Smart Inc.",
            "Merchandise Holdings LLC",
            "Retail Distribution Co.",
            "Store Systems Ltd.",
        ]
    }

    fn culture_to_key(culture: NameCulture) -> &'static str {
        match culture {
            NameCulture::WesternUs => "us",
            NameCulture::German => "german",
            NameCulture::Hispanic => "hispanic",
            NameCulture::French => "french",
            NameCulture::Chinese => "chinese",
            NameCulture::Japanese => "japanese",
            NameCulture::Indian => "indian",
        }
    }

    fn process_to_key(process: BusinessProcess) -> &'static str {
        match process {
            BusinessProcess::P2P => "p2p",
            BusinessProcess::O2C => "o2c",
            BusinessProcess::H2R => "h2r",
            BusinessProcess::R2R => "r2r",
            _ => "other",
        }
    }
}

impl Default for DefaultTemplateProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateProvider for DefaultTemplateProvider {
    fn get_person_first_name(
        &self,
        culture: NameCulture,
        is_male: bool,
        rng: &mut dyn RngCore,
    ) -> String {
        let key = Self::culture_to_key(culture);

        // Try file templates first
        if let Some(ref data) = self.template_data {
            if let Some(culture_names) = data.person_names.cultures.get(key) {
                let names = if is_male {
                    &culture_names.male_first_names
                } else {
                    &culture_names.female_first_names
                };
                if !names.is_empty() {
                    if let Some(name) = names.choose(rng) {
                        return name.clone();
                    }
                }
            }
        }

        // Fall back to embedded templates
        let embedded = match culture {
            NameCulture::German => {
                if is_male {
                    Self::embedded_german_first_names_male()
                } else {
                    Self::embedded_german_first_names_female()
                }
            }
            _ => {
                if is_male {
                    Self::embedded_us_first_names_male()
                } else {
                    Self::embedded_us_first_names_female()
                }
            }
        };

        embedded.choose(rng).unwrap_or(&"Unknown").to_string()
    }

    fn get_person_last_name(&self, culture: NameCulture, rng: &mut dyn RngCore) -> String {
        let key = Self::culture_to_key(culture);

        // Try file templates first
        if let Some(ref data) = self.template_data {
            if let Some(culture_names) = data.person_names.cultures.get(key) {
                if !culture_names.last_names.is_empty() {
                    if let Some(name) = culture_names.last_names.choose(rng) {
                        return name.clone();
                    }
                }
            }
        }

        // Fall back to embedded templates
        let embedded = match culture {
            NameCulture::German => Self::embedded_german_last_names(),
            _ => Self::embedded_us_last_names(),
        };

        embedded.choose(rng).unwrap_or(&"Unknown").to_string()
    }

    fn get_vendor_name(&self, category: &str, rng: &mut dyn RngCore) -> String {
        // Try file templates first
        if let Some(ref data) = self.template_data {
            if let Some(names) = data.vendor_names.categories.get(category) {
                if !names.is_empty() {
                    if let Some(name) = names.choose(rng) {
                        return name.clone();
                    }
                }
            }
        }

        // Fall back to embedded templates
        let embedded = match category {
            "manufacturing" => Self::embedded_vendor_names_manufacturing(),
            "services" => Self::embedded_vendor_names_services(),
            _ => Self::embedded_vendor_names_manufacturing(),
        };

        embedded
            .choose(rng)
            .unwrap_or(&"Unknown Vendor")
            .to_string()
    }

    fn get_customer_name(&self, industry: &str, rng: &mut dyn RngCore) -> String {
        // Try file templates first
        if let Some(ref data) = self.template_data {
            if let Some(names) = data.customer_names.industries.get(industry) {
                if !names.is_empty() {
                    if let Some(name) = names.choose(rng) {
                        return name.clone();
                    }
                }
            }
        }

        // Fall back to embedded templates
        let embedded = match industry {
            "automotive" => Self::embedded_customer_names_automotive(),
            "retail" => Self::embedded_customer_names_retail(),
            _ => Self::embedded_customer_names_retail(),
        };

        embedded
            .choose(rng)
            .unwrap_or(&"Unknown Customer")
            .to_string()
    }

    fn get_material_description(&self, material_type: &str, rng: &mut dyn RngCore) -> String {
        // Try file templates first
        if let Some(ref data) = self.template_data {
            if let Some(descs) = data.material_descriptions.by_type.get(material_type) {
                if !descs.is_empty() {
                    if let Some(desc) = descs.choose(rng) {
                        return desc.clone();
                    }
                }
            }
        }

        // Fall back to generic
        format!("{} material", material_type)
    }

    fn get_asset_description(&self, category: &str, rng: &mut dyn RngCore) -> String {
        // Try file templates first
        if let Some(ref data) = self.template_data {
            if let Some(descs) = data.asset_descriptions.by_category.get(category) {
                if !descs.is_empty() {
                    if let Some(desc) = descs.choose(rng) {
                        return desc.clone();
                    }
                }
            }
        }

        // Fall back to generic
        format!("{} asset", category)
    }

    fn get_line_text(
        &self,
        process: BusinessProcess,
        account_type: &str,
        rng: &mut dyn RngCore,
    ) -> String {
        let key = Self::process_to_key(process);

        // Try file templates first
        if let Some(ref data) = self.template_data {
            let descs_map = match process {
                BusinessProcess::P2P => &data.line_item_descriptions.p2p,
                BusinessProcess::O2C => &data.line_item_descriptions.o2c,
                BusinessProcess::H2R => &data.line_item_descriptions.h2r,
                BusinessProcess::R2R => &data.line_item_descriptions.r2r,
                _ => &data.line_item_descriptions.p2p,
            };

            if let Some(descs) = descs_map.get(account_type) {
                if !descs.is_empty() {
                    if let Some(desc) = descs.choose(rng) {
                        return desc.clone();
                    }
                }
            }
        }

        // Fall back to generic
        format!("{} posting", key.to_uppercase())
    }

    fn get_header_template(&self, process: BusinessProcess, rng: &mut dyn RngCore) -> String {
        let key = Self::process_to_key(process);

        // Try file templates first
        if let Some(ref data) = self.template_data {
            if let Some(templates) = data.header_text_templates.by_process.get(key) {
                if !templates.is_empty() {
                    if let Some(template) = templates.choose(rng) {
                        return template.clone();
                    }
                }
            }
        }

        // Fall back to generic
        format!("{} Transaction", key.to_uppercase())
    }
}

/// A thread-safe wrapper around a template provider.
pub type SharedTemplateProvider = Arc<dyn TemplateProvider>;

/// Create a default shared template provider.
pub fn default_provider() -> SharedTemplateProvider {
    Arc::new(DefaultTemplateProvider::new())
}

/// Create a shared template provider from a file.
pub fn provider_from_file(
    path: &std::path::Path,
) -> Result<SharedTemplateProvider, super::loader::TemplateError> {
    Ok(Arc::new(DefaultTemplateProvider::from_file(path)?))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_default_provider() {
        let provider = DefaultTemplateProvider::new();
        let mut rng = ChaCha8Rng::seed_from_u64(12345);

        let name = provider.get_person_first_name(NameCulture::German, true, &mut rng);
        assert!(!name.is_empty());

        let last_name = provider.get_person_last_name(NameCulture::German, &mut rng);
        assert!(!last_name.is_empty());
    }

    #[test]
    fn test_vendor_names() {
        let provider = DefaultTemplateProvider::new();
        let mut rng = ChaCha8Rng::seed_from_u64(12345);

        let name = provider.get_vendor_name("manufacturing", &mut rng);
        assert!(!name.is_empty());
        assert!(!name.contains("Unknown"));
    }

    #[test]
    fn test_shared_provider() {
        let provider = default_provider();
        let mut rng = ChaCha8Rng::seed_from_u64(12345);

        let name = provider.get_customer_name("retail", &mut rng);
        assert!(!name.is_empty());
    }
}
