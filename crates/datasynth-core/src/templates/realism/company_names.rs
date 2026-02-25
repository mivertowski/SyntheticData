//! Industry-specific company name generation.
//!
//! Generates realistic company names based on industry sector with appropriate
//! naming patterns and legal suffixes.

use rand::seq::IndexedRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Industry sector for company naming.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Industry {
    #[default]
    Manufacturing,
    Retail,
    Technology,
    FinancialServices,
    Healthcare,
    ProfessionalServices,
    Energy,
    Transportation,
    RealEstate,
    Telecommunications,
    Construction,
    Hospitality,
}

impl Industry {
    /// Get all available industries.
    pub fn all() -> &'static [Industry] {
        &[
            Industry::Manufacturing,
            Industry::Retail,
            Industry::Technology,
            Industry::FinancialServices,
            Industry::Healthcare,
            Industry::ProfessionalServices,
            Industry::Energy,
            Industry::Transportation,
            Industry::RealEstate,
            Industry::Telecommunications,
            Industry::Construction,
            Industry::Hospitality,
        ]
    }
}

/// Legal suffix for company names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LegalSuffix {
    Inc,
    Corp,
    Corporation,
    LLC,
    Ltd,
    Co,
    Company,
    Group,
    Partners,
    LLP,
    // International
    GmbH, // German
    AG,   // German/Swiss
    SA,   // French/Spanish
    Pty,  // Australian
    BV,   // Dutch
    PLC,  // UK
    Srl,  // Italian
    AB,   // Swedish
}

impl LegalSuffix {
    /// Get all US suffixes.
    pub fn us_suffixes() -> &'static [LegalSuffix] {
        &[
            LegalSuffix::Inc,
            LegalSuffix::Corp,
            LegalSuffix::Corporation,
            LegalSuffix::LLC,
            LegalSuffix::Co,
            LegalSuffix::Company,
            LegalSuffix::Group,
        ]
    }

    /// Get all international suffixes.
    pub fn international_suffixes() -> &'static [LegalSuffix] {
        &[
            LegalSuffix::Ltd,
            LegalSuffix::GmbH,
            LegalSuffix::AG,
            LegalSuffix::SA,
            LegalSuffix::Pty,
            LegalSuffix::BV,
            LegalSuffix::PLC,
            LegalSuffix::Srl,
            LegalSuffix::AB,
        ]
    }

    /// Convert to display string.
    pub fn as_str(&self) -> &'static str {
        match self {
            LegalSuffix::Inc => "Inc.",
            LegalSuffix::Corp => "Corp.",
            LegalSuffix::Corporation => "Corporation",
            LegalSuffix::LLC => "LLC",
            LegalSuffix::Ltd => "Ltd.",
            LegalSuffix::Co => "Co.",
            LegalSuffix::Company => "Company",
            LegalSuffix::Group => "Group",
            LegalSuffix::Partners => "Partners",
            LegalSuffix::LLP => "LLP",
            LegalSuffix::GmbH => "GmbH",
            LegalSuffix::AG => "AG",
            LegalSuffix::SA => "S.A.",
            LegalSuffix::Pty => "Pty Ltd",
            LegalSuffix::BV => "B.V.",
            LegalSuffix::PLC => "PLC",
            LegalSuffix::Srl => "S.r.l.",
            LegalSuffix::AB => "AB",
        }
    }
}

/// Company name generation style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CompanyNameStyle {
    /// Founder name based (e.g., "Johnson Industries")
    #[default]
    FounderBased,
    /// Descriptive (e.g., "Advanced Manufacturing Solutions")
    Descriptive,
    /// Location based (e.g., "Pacific Coast Manufacturing")
    LocationBased,
    /// Acronym style (e.g., "AMC Industries")
    Acronym,
    /// Abstract/Creative (e.g., "Nexus Technologies")
    Abstract,
}

/// Company name generator with industry-specific patterns.
#[derive(Debug, Clone)]
pub struct CompanyNameGenerator {
    founder_names: Vec<&'static str>,
    location_prefixes: Vec<&'static str>,
    abstract_names: Vec<&'static str>,
    industry_descriptors: IndustryDescriptors,
}

#[derive(Debug, Clone)]
struct IndustryDescriptors {
    manufacturing: Vec<&'static str>,
    retail: Vec<&'static str>,
    technology: Vec<&'static str>,
    financial: Vec<&'static str>,
    healthcare: Vec<&'static str>,
    professional: Vec<&'static str>,
    energy: Vec<&'static str>,
    transportation: Vec<&'static str>,
    real_estate: Vec<&'static str>,
    telecom: Vec<&'static str>,
    construction: Vec<&'static str>,
    hospitality: Vec<&'static str>,
}

impl Default for CompanyNameGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CompanyNameGenerator {
    /// Create a new company name generator with default data.
    pub fn new() -> Self {
        Self {
            founder_names: vec![
                "Anderson",
                "Baker",
                "Bennett",
                "Brooks",
                "Campbell",
                "Carter",
                "Chen",
                "Clark",
                "Collins",
                "Cooper",
                "Davis",
                "Edwards",
                "Evans",
                "Fisher",
                "Foster",
                "Garcia",
                "Graham",
                "Green",
                "Griffin",
                "Hall",
                "Hamilton",
                "Harris",
                "Harrison",
                "Hayes",
                "Henderson",
                "Hill",
                "Howard",
                "Hughes",
                "Jackson",
                "James",
                "Jenkins",
                "Johnson",
                "Jones",
                "Kelly",
                "Kennedy",
                "Kim",
                "King",
                "Kumar",
                "Lee",
                "Lewis",
                "Li",
                "Liu",
                "Lopez",
                "Martin",
                "Martinez",
                "Mason",
                "Meyer",
                "Miller",
                "Mitchell",
                "Moore",
                "Morgan",
                "Morris",
                "Murphy",
                "Nelson",
                "Nguyen",
                "O'Brien",
                "Palmer",
                "Parker",
                "Patel",
                "Patterson",
                "Perry",
                "Peterson",
                "Phillips",
                "Powell",
                "Price",
                "Reed",
                "Reynolds",
                "Richardson",
                "Roberts",
                "Robinson",
                "Rogers",
                "Ross",
                "Russell",
                "Sanders",
                "Schmidt",
                "Scott",
                "Shaw",
                "Singh",
                "Smith",
                "Spencer",
                "Stevens",
                "Stewart",
                "Sullivan",
                "Taylor",
                "Thomas",
                "Thompson",
                "Turner",
                "Walker",
                "Walsh",
                "Wang",
                "Ward",
                "Watson",
                "Weber",
                "Wells",
                "White",
                "Williams",
                "Wilson",
                "Wood",
                "Wright",
                "Yang",
                "Young",
                "Zhang",
            ],
            location_prefixes: vec![
                "Pacific",
                "Atlantic",
                "Midwest",
                "Northeast",
                "Southeast",
                "Southwest",
                "Western",
                "Eastern",
                "Northern",
                "Southern",
                "Central",
                "Coastal",
                "Mountain",
                "Valley",
                "Bay",
                "Harbor",
                "Lakeside",
                "Riverside",
                "Highland",
                "Lowland",
                "Metropolitan",
                "Regional",
                "National",
                "Global",
                "Continental",
                "Transatlantic",
                "American",
                "United",
                "Allied",
                "Premier",
            ],
            abstract_names: vec![
                "Apex",
                "Arcadia",
                "Ascend",
                "Atlas",
                "Aurora",
                "Beacon",
                "Catalyst",
                "Centrix",
                "Clearview",
                "Cogent",
                "Coretech",
                "Dynamix",
                "Eclipse",
                "Elevate",
                "Ember",
                "Encompass",
                "Endeavor",
                "Envision",
                "Equinox",
                "Evergreen",
                "Evolve",
                "Excel",
                "Forge",
                "Frontier",
                "Genesis",
                "Granite",
                "Horizon",
                "Ignite",
                "Illuminate",
                "Infinity",
                "Innovate",
                "Inspire",
                "Integrate",
                "Keystone",
                "Kinetic",
                "Latitude",
                "Lumina",
                "Magnitude",
                "Matrix",
                "Meridian",
                "Momentum",
                "Navigate",
                "Nexus",
                "Nova",
                "Nucleus",
                "Oasis",
                "Omega",
                "Optimize",
                "Orbit",
                "Paradigm",
                "Paramount",
                "Peak",
                "Pinnacle",
                "Pioneer",
                "Pivot",
                "Precision",
                "Prism",
                "Propel",
                "Pulse",
                "Quantum",
                "Quest",
                "Radiant",
                "Radius",
                "Reach",
                "Relay",
                "Resolve",
                "Sage",
                "Sentinel",
                "Skyline",
                "Solstice",
                "Spark",
                "Spectrum",
                "Sphere",
                "Summit",
                "Synapse",
                "Synergy",
                "Synthesis",
                "Terra",
                "Thrive",
                "Titan",
                "Transcend",
                "Trident",
                "Trinity",
                "Triumph",
                "Unified",
                "Vanguard",
                "Vector",
                "Velocity",
                "Venture",
                "Vertex",
                "Vista",
                "Voyager",
                "Zenith",
            ],
            industry_descriptors: IndustryDescriptors {
                manufacturing: vec![
                    "Manufacturing",
                    "Industries",
                    "Industrial",
                    "Fabrication",
                    "Production",
                    "Metalworks",
                    "Components",
                    "Assembly",
                    "Machining",
                    "Precision",
                    "Materials",
                    "Systems",
                    "Engineering",
                    "Solutions",
                    "Technologies",
                    "Products",
                    "Works",
                    "Equipment",
                    "Tools",
                    "Dynamics",
                ],
                retail: vec![
                    "Retail",
                    "Stores",
                    "Markets",
                    "Outlets",
                    "Shopping",
                    "Mart",
                    "Goods",
                    "Merchants",
                    "Trading",
                    "Distribution",
                    "Supply",
                    "Wholesale",
                    "Direct",
                    "Consumer",
                    "Brands",
                    "Products",
                    "Marketplace",
                    "Commerce",
                    "Sales",
                ],
                technology: vec![
                    "Technologies",
                    "Tech",
                    "Systems",
                    "Software",
                    "Solutions",
                    "Digital",
                    "Computing",
                    "Networks",
                    "Data",
                    "Cloud",
                    "Cyber",
                    "AI",
                    "Analytics",
                    "Platforms",
                    "Labs",
                    "Innovations",
                    "Interactive",
                    "Intelligence",
                ],
                financial: vec![
                    "Financial",
                    "Capital",
                    "Investments",
                    "Banking",
                    "Trust",
                    "Securities",
                    "Wealth",
                    "Asset",
                    "Advisory",
                    "Holdings",
                    "Ventures",
                    "Partners",
                    "Credit",
                    "Funding",
                    "Finance",
                    "Insurance",
                    "Risk",
                    "Portfolio",
                ],
                healthcare: vec![
                    "Healthcare",
                    "Medical",
                    "Health",
                    "Therapeutics",
                    "Pharma",
                    "Biotech",
                    "Clinical",
                    "Diagnostics",
                    "Life Sciences",
                    "Wellness",
                    "Care",
                    "Surgical",
                    "Devices",
                    "Laboratories",
                    "Medicine",
                    "Biosciences",
                ],
                professional: vec![
                    "Consulting",
                    "Advisory",
                    "Associates",
                    "Partners",
                    "Services",
                    "Group",
                    "Solutions",
                    "Advisors",
                    "Professionals",
                    "Management",
                    "Strategies",
                    "Resources",
                    "Experts",
                    "Specialists",
                    "Counsel",
                    "Practice",
                ],
                energy: vec![
                    "Energy",
                    "Power",
                    "Utilities",
                    "Resources",
                    "Renewables",
                    "Solar",
                    "Wind",
                    "Electric",
                    "Petroleum",
                    "Oil",
                    "Gas",
                    "Fuel",
                    "Generation",
                    "Grid",
                    "Sustainability",
                    "Environmental",
                    "Clean",
                    "Green",
                ],
                transportation: vec![
                    "Logistics",
                    "Transport",
                    "Freight",
                    "Shipping",
                    "Carriers",
                    "Express",
                    "Fleet",
                    "Moving",
                    "Hauling",
                    "Transit",
                    "Airways",
                    "Lines",
                    "Delivery",
                    "Distribution",
                    "Supply Chain",
                    "Trucking",
                    "Rail",
                    "Maritime",
                ],
                real_estate: vec![
                    "Properties",
                    "Realty",
                    "Real Estate",
                    "Development",
                    "Investments",
                    "Holdings",
                    "Land",
                    "Estates",
                    "Commercial",
                    "Residential",
                    "Builders",
                    "Construction",
                    "Management",
                    "Capital",
                    "Ventures",
                    "Trust",
                ],
                telecom: vec![
                    "Communications",
                    "Telecom",
                    "Wireless",
                    "Networks",
                    "Connect",
                    "Broadband",
                    "Cable",
                    "Cellular",
                    "Mobile",
                    "Satellite",
                    "Fiber",
                    "Media",
                    "Digital",
                    "Interactive",
                    "Broadcasting",
                    "Signals",
                ],
                construction: vec![
                    "Construction",
                    "Builders",
                    "Contractors",
                    "Building",
                    "Development",
                    "Infrastructure",
                    "Engineering",
                    "Projects",
                    "Structural",
                    "Civil",
                    "Architectural",
                    "Design-Build",
                    "General Contractors",
                    "Fabricators",
                ],
                hospitality: vec![
                    "Hospitality",
                    "Hotels",
                    "Resorts",
                    "Lodging",
                    "Entertainment",
                    "Leisure",
                    "Dining",
                    "Restaurant",
                    "Food Service",
                    "Catering",
                    "Events",
                    "Travel",
                    "Tourism",
                    "Recreation",
                    "Gaming",
                    "Venues",
                ],
            },
        }
    }

    /// Generate a company name for the specified industry.
    pub fn generate(&self, industry: Industry, rng: &mut impl Rng) -> String {
        let style = self.select_style(rng);
        let name = self.generate_base_name(industry, style, rng);
        let suffix = self.select_suffix(rng);
        format!("{} {}", name, suffix.as_str())
    }

    /// Generate a company name with a specific style.
    pub fn generate_with_style(
        &self,
        industry: Industry,
        style: CompanyNameStyle,
        rng: &mut impl Rng,
    ) -> String {
        let name = self.generate_base_name(industry, style, rng);
        let suffix = self.select_suffix(rng);
        format!("{} {}", name, suffix.as_str())
    }

    /// Generate just the base name without suffix.
    pub fn generate_base_name(
        &self,
        industry: Industry,
        style: CompanyNameStyle,
        rng: &mut impl Rng,
    ) -> String {
        let descriptors = self.get_descriptors(industry);

        match style {
            CompanyNameStyle::FounderBased => {
                let founder = self.founder_names.choose(rng).expect("non-empty name pool");
                let descriptor = descriptors.choose(rng).expect("non-empty name pool");
                format!("{} {}", founder, descriptor)
            }
            CompanyNameStyle::Descriptive => {
                let adjective = self.select_adjective(rng);
                let descriptor = descriptors.choose(rng).expect("non-empty name pool");
                format!("{} {}", adjective, descriptor)
            }
            CompanyNameStyle::LocationBased => {
                let location = self
                    .location_prefixes
                    .choose(rng)
                    .expect("non-empty name pool");
                let descriptor = descriptors.choose(rng).expect("non-empty name pool");
                format!("{} {}", location, descriptor)
            }
            CompanyNameStyle::Acronym => {
                let letters: String = (0..3)
                    .map(|_| (b'A' + rng.random_range(0..26)) as char)
                    .collect();
                let descriptor = descriptors.choose(rng).expect("non-empty name pool");
                format!("{} {}", letters, descriptor)
            }
            CompanyNameStyle::Abstract => {
                let abstract_name = self
                    .abstract_names
                    .choose(rng)
                    .expect("non-empty name pool");
                let descriptor = descriptors.choose(rng).expect("non-empty name pool");
                format!("{} {}", abstract_name, descriptor)
            }
        }
    }

    fn get_descriptors(&self, industry: Industry) -> &[&'static str] {
        match industry {
            Industry::Manufacturing => &self.industry_descriptors.manufacturing,
            Industry::Retail => &self.industry_descriptors.retail,
            Industry::Technology => &self.industry_descriptors.technology,
            Industry::FinancialServices => &self.industry_descriptors.financial,
            Industry::Healthcare => &self.industry_descriptors.healthcare,
            Industry::ProfessionalServices => &self.industry_descriptors.professional,
            Industry::Energy => &self.industry_descriptors.energy,
            Industry::Transportation => &self.industry_descriptors.transportation,
            Industry::RealEstate => &self.industry_descriptors.real_estate,
            Industry::Telecommunications => &self.industry_descriptors.telecom,
            Industry::Construction => &self.industry_descriptors.construction,
            Industry::Hospitality => &self.industry_descriptors.hospitality,
        }
    }

    fn select_style(&self, rng: &mut impl Rng) -> CompanyNameStyle {
        let roll: f64 = rng.random();
        if roll < 0.35 {
            CompanyNameStyle::FounderBased
        } else if roll < 0.55 {
            CompanyNameStyle::Descriptive
        } else if roll < 0.70 {
            CompanyNameStyle::LocationBased
        } else if roll < 0.85 {
            CompanyNameStyle::Abstract
        } else {
            CompanyNameStyle::Acronym
        }
    }

    fn select_adjective(&self, rng: &mut impl Rng) -> &'static str {
        const ADJECTIVES: &[&str] = &[
            "Advanced",
            "Premier",
            "Elite",
            "Quality",
            "Superior",
            "Professional",
            "Innovative",
            "Modern",
            "Strategic",
            "Dynamic",
            "Progressive",
            "Integrated",
            "Comprehensive",
            "Specialized",
            "Technical",
            "Precision",
            "Custom",
            "Global",
            "National",
            "American",
            "United",
            "Allied",
            "Universal",
            "General",
        ];
        ADJECTIVES.choose(rng).expect("non-empty name pool")
    }

    fn select_suffix(&self, rng: &mut impl Rng) -> LegalSuffix {
        let suffixes = LegalSuffix::us_suffixes();
        let weights = [30, 20, 10, 25, 5, 5, 5]; // Inc, Corp, Corporation, LLC, Co, Company, Group
        let total: i32 = weights.iter().sum();
        let roll = rng.random_range(0..total);

        let mut cumulative = 0;
        for (i, &weight) in weights.iter().enumerate() {
            cumulative += weight;
            if roll < cumulative {
                return suffixes[i];
            }
        }
        LegalSuffix::Inc
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_company_name_generation() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let gen = CompanyNameGenerator::new();

        for industry in Industry::all() {
            let name = gen.generate(*industry, &mut rng);
            assert!(!name.is_empty());
            // Should contain a legal suffix
            assert!(
                name.contains("Inc.")
                    || name.contains("Corp.")
                    || name.contains("LLC")
                    || name.contains("Ltd.")
                    || name.contains("Co.")
                    || name.contains("Company")
                    || name.contains("Group")
                    || name.contains("Corporation")
            );
        }
    }

    #[test]
    fn test_founder_based_style() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let gen = CompanyNameGenerator::new();

        let name = gen.generate_with_style(
            Industry::Manufacturing,
            CompanyNameStyle::FounderBased,
            &mut rng,
        );
        assert!(!name.is_empty());
    }

    #[test]
    fn test_acronym_style() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let gen = CompanyNameGenerator::new();

        let name =
            gen.generate_with_style(Industry::Technology, CompanyNameStyle::Acronym, &mut rng);
        // Should contain 3 uppercase letters
        let has_acronym = name
            .split_whitespace()
            .next()
            .map(|s| s.len() == 3 && s.chars().all(|c| c.is_ascii_uppercase()))
            .unwrap_or(false);
        assert!(has_acronym);
    }

    #[test]
    fn test_legal_suffix_display() {
        assert_eq!(LegalSuffix::Inc.as_str(), "Inc.");
        assert_eq!(LegalSuffix::LLC.as_str(), "LLC");
        assert_eq!(LegalSuffix::GmbH.as_str(), "GmbH");
        assert_eq!(LegalSuffix::SA.as_str(), "S.A.");
    }

    #[test]
    fn test_deterministic_generation() {
        let gen = CompanyNameGenerator::new();

        let mut rng1 = ChaCha8Rng::seed_from_u64(12345);
        let mut rng2 = ChaCha8Rng::seed_from_u64(12345);

        let name1 = gen.generate(Industry::Manufacturing, &mut rng1);
        let name2 = gen.generate(Industry::Manufacturing, &mut rng2);

        assert_eq!(name1, name2);
    }

    #[test]
    fn test_variety_in_generation() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let gen = CompanyNameGenerator::new();

        let mut names = std::collections::HashSet::new();
        for _ in 0..100 {
            names.insert(gen.generate(Industry::Technology, &mut rng));
        }

        // Should generate diverse names
        assert!(names.len() > 50);
    }
}
