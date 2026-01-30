//! Multi-regional address generation.
//!
//! Generates realistic addresses for multiple regions with appropriate
//! formatting and regional conventions.

use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Geographic region for address generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AddressRegion {
    #[default]
    NorthAmerica,
    Europe,
    AsiaPacific,
    LatinAmerica,
}

/// Address formatting style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AddressStyle {
    /// Full format with all components
    #[default]
    Full,
    /// Abbreviated format
    Short,
    /// Single line format
    SingleLine,
}

/// Generated address with components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    /// Street number
    pub street_number: String,
    /// Street name
    pub street_name: String,
    /// Street type/suffix (St, Ave, Blvd, etc.)
    pub street_type: String,
    /// Unit/Suite (optional)
    pub unit: Option<String>,
    /// City
    pub city: String,
    /// State/Province/Region
    pub state: String,
    /// Postal/ZIP code
    pub postal_code: String,
    /// Country
    pub country: String,
    /// Region this address is from
    pub region: AddressRegion,
}

impl Address {
    /// Format the address as a single string.
    pub fn format(&self, style: AddressStyle) -> String {
        match style {
            AddressStyle::Full => self.format_full(),
            AddressStyle::Short => self.format_short(),
            AddressStyle::SingleLine => self.format_single_line(),
        }
    }

    fn format_full(&self) -> String {
        let street = if let Some(ref unit) = self.unit {
            format!(
                "{} {} {}, {}",
                self.street_number, self.street_name, self.street_type, unit
            )
        } else {
            format!(
                "{} {} {}",
                self.street_number, self.street_name, self.street_type
            )
        };

        match self.region {
            AddressRegion::NorthAmerica => {
                format!(
                    "{}\n{}, {} {}\n{}",
                    street, self.city, self.state, self.postal_code, self.country
                )
            }
            AddressRegion::Europe => {
                // European format: postal code before city
                format!(
                    "{}\n{} {}\n{}",
                    street, self.postal_code, self.city, self.country
                )
            }
            AddressRegion::AsiaPacific => {
                format!(
                    "{}\n{} {}\n{}",
                    street, self.city, self.postal_code, self.country
                )
            }
            AddressRegion::LatinAmerica => {
                format!(
                    "{}\n{}, {} {}\n{}",
                    street, self.city, self.state, self.postal_code, self.country
                )
            }
        }
    }

    fn format_short(&self) -> String {
        format!(
            "{} {} {}, {}, {}",
            self.street_number, self.street_name, self.street_type, self.city, self.state
        )
    }

    fn format_single_line(&self) -> String {
        let unit_part = self
            .unit
            .as_ref()
            .map(|u| format!(", {}", u))
            .unwrap_or_default();
        format!(
            "{} {} {}{}, {}, {} {}, {}",
            self.street_number,
            self.street_name,
            self.street_type,
            unit_part,
            self.city,
            self.state,
            self.postal_code,
            self.country
        )
    }
}

/// Address generator with regional support.
#[derive(Debug, Clone)]
pub struct AddressGenerator {
    region: AddressRegion,
    data: RegionalData,
}

#[derive(Debug, Clone)]
struct RegionalData {
    street_names: Vec<&'static str>,
    street_types: Vec<(&'static str, f64)>, // (type, weight)
    cities: Vec<(&'static str, &'static str, &'static str)>, // (city, state, country)
    unit_probability: f64,
}

impl Default for AddressGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl AddressGenerator {
    /// Create a new address generator for North America (default).
    pub fn new() -> Self {
        Self::for_region(AddressRegion::NorthAmerica)
    }

    /// Create an address generator for a specific region.
    pub fn for_region(region: AddressRegion) -> Self {
        let data = match region {
            AddressRegion::NorthAmerica => Self::north_america_data(),
            AddressRegion::Europe => Self::europe_data(),
            AddressRegion::AsiaPacific => Self::asia_pacific_data(),
            AddressRegion::LatinAmerica => Self::latin_america_data(),
        };

        Self { region, data }
    }

    /// Generate an address.
    pub fn generate(&self, rng: &mut impl Rng) -> Address {
        let street_number = self.generate_street_number(rng);
        let street_name = self.data.street_names.choose(rng).unwrap_or(&"Main");
        let street_type = self.select_street_type(rng);
        let unit = if rng.gen_bool(self.data.unit_probability) {
            Some(self.generate_unit(rng))
        } else {
            None
        };
        let (city, state, country) = self
            .data
            .cities
            .choose(rng)
            .unwrap_or(&("City", "ST", "US"));
        let postal_code = self.generate_postal_code(state, rng);

        Address {
            street_number,
            street_name: (*street_name).to_string(),
            street_type: street_type.to_string(),
            unit,
            city: (*city).to_string(),
            state: (*state).to_string(),
            postal_code,
            country: (*country).to_string(),
            region: self.region,
        }
    }

    /// Generate a commercial/business address.
    pub fn generate_commercial(&self, rng: &mut impl Rng) -> Address {
        let mut addr = self.generate(rng);
        // Commercial addresses more likely to have suite numbers
        if rng.gen_bool(0.7) && addr.unit.is_none() {
            addr.unit = Some(self.generate_commercial_unit(rng));
        }
        // Use lower street numbers for commercial areas
        addr.street_number = format!("{}", rng.gen_range(1..500));
        addr
    }

    fn generate_street_number(&self, rng: &mut impl Rng) -> String {
        // Log-normal-ish distribution for street numbers
        let base = rng.gen_range(1..100);
        let multiplier = if rng.gen_bool(0.7) { 1 } else { 10 };
        format!("{}", base * multiplier)
    }

    fn select_street_type(&self, rng: &mut impl Rng) -> &'static str {
        let total_weight: f64 = self.data.street_types.iter().map(|(_, w)| w).sum();
        let mut roll = rng.gen::<f64>() * total_weight;

        for (street_type, weight) in &self.data.street_types {
            roll -= weight;
            if roll <= 0.0 {
                return street_type;
            }
        }

        self.data.street_types[0].0
    }

    fn generate_unit(&self, rng: &mut impl Rng) -> String {
        let style = rng.gen_range(0..5);
        match style {
            0 => format!("Apt {}", rng.gen_range(1..500)),
            1 => format!("Suite {}", rng.gen_range(100..999)),
            2 => format!("Unit {}", rng.gen_range(1..200)),
            3 => format!("#{}", rng.gen_range(1..300)),
            _ => format!("Floor {}", rng.gen_range(1..30)),
        }
    }

    fn generate_commercial_unit(&self, rng: &mut impl Rng) -> String {
        let style = rng.gen_range(0..4);
        match style {
            0 => format!("Suite {}", rng.gen_range(100..999)),
            1 => format!("Floor {}", rng.gen_range(1..50)),
            2 => format!("Ste. {}", rng.gen_range(100..999)),
            _ => format!("Unit {}", (b'A' + rng.gen_range(0..26)) as char),
        }
    }

    fn generate_postal_code(&self, state: &str, rng: &mut impl Rng) -> String {
        match self.region {
            AddressRegion::NorthAmerica => {
                // US ZIP code or Canadian postal code
                if state.len() == 2 && state.chars().all(|c| c.is_ascii_uppercase()) {
                    // US state - ZIP code
                    format!("{:05}", rng.gen_range(10000..99999))
                } else {
                    // Canadian postal code
                    let letter1 = (b'A' + rng.gen_range(0..26)) as char;
                    let num1 = rng.gen_range(1..9);
                    let letter2 = (b'A' + rng.gen_range(0..26)) as char;
                    let num2 = rng.gen_range(1..9);
                    let letter3 = (b'A' + rng.gen_range(0..26)) as char;
                    let num3 = rng.gen_range(1..9);
                    format!("{}{}{} {}{}{}", letter1, num1, letter2, num2, letter3, num3)
                }
            }
            AddressRegion::Europe => {
                // Various European formats
                format!("{:05}", rng.gen_range(10000..99999))
            }
            AddressRegion::AsiaPacific => {
                // Various APAC formats
                format!("{:06}", rng.gen_range(100000..999999))
            }
            AddressRegion::LatinAmerica => {
                // Latin American formats
                format!("{:05}", rng.gen_range(10000..99999))
            }
        }
    }

    fn north_america_data() -> RegionalData {
        RegionalData {
            street_names: vec![
                "Main",
                "Oak",
                "Maple",
                "Cedar",
                "Pine",
                "Elm",
                "Washington",
                "Lincoln",
                "Jefferson",
                "Madison",
                "Franklin",
                "Adams",
                "Jackson",
                "Park",
                "Lake",
                "River",
                "Hill",
                "Valley",
                "Forest",
                "Meadow",
                "Spring",
                "Sunset",
                "Highland",
                "Fairview",
                "Central",
                "Broadway",
                "Market",
                "Church",
                "School",
                "Mill",
                "Industrial",
                "Commerce",
                "Corporate",
                "Executive",
                "Business",
                "Technology",
                "Innovation",
                "Enterprise",
                "Professional",
                "Financial",
            ],
            street_types: vec![
                ("Street", 0.25),
                ("Avenue", 0.15),
                ("Road", 0.12),
                ("Drive", 0.12),
                ("Boulevard", 0.08),
                ("Lane", 0.08),
                ("Way", 0.06),
                ("Court", 0.05),
                ("Place", 0.04),
                ("Circle", 0.03),
                ("Parkway", 0.02),
            ],
            cities: vec![
                ("New York", "NY", "USA"),
                ("Los Angeles", "CA", "USA"),
                ("Chicago", "IL", "USA"),
                ("Houston", "TX", "USA"),
                ("Phoenix", "AZ", "USA"),
                ("Philadelphia", "PA", "USA"),
                ("San Antonio", "TX", "USA"),
                ("San Diego", "CA", "USA"),
                ("Dallas", "TX", "USA"),
                ("San Jose", "CA", "USA"),
                ("Austin", "TX", "USA"),
                ("Jacksonville", "FL", "USA"),
                ("San Francisco", "CA", "USA"),
                ("Columbus", "OH", "USA"),
                ("Indianapolis", "IN", "USA"),
                ("Seattle", "WA", "USA"),
                ("Denver", "CO", "USA"),
                ("Boston", "MA", "USA"),
                ("Nashville", "TN", "USA"),
                ("Portland", "OR", "USA"),
                ("Toronto", "ON", "Canada"),
                ("Vancouver", "BC", "Canada"),
                ("Montreal", "QC", "Canada"),
                ("Calgary", "AB", "Canada"),
            ],
            unit_probability: 0.25,
        }
    }

    fn europe_data() -> RegionalData {
        RegionalData {
            street_names: vec![
                "High",
                "King",
                "Queen",
                "Market",
                "Church",
                "Station",
                "Park",
                "Victoria",
                "George",
                "Oxford",
                "Regent",
                "Bond",
                "Baker",
                "Fleet",
                "Lombard",
                "Strand",
                "Hauptstraße",
                "Bahnhofstraße",
                "Schillerstraße",
                "Goethestraße",
                "Lindenstraße",
                "Rue de la Paix",
                "Avenue des Champs",
                "Boulevard Saint",
                "Rue du Commerce",
            ],
            street_types: vec![
                ("Street", 0.30),
                ("Road", 0.20),
                ("Avenue", 0.15),
                ("Lane", 0.10),
                ("Way", 0.08),
                ("Place", 0.07),
                ("Close", 0.05),
                ("Crescent", 0.05),
            ],
            cities: vec![
                ("London", "England", "United Kingdom"),
                ("Manchester", "England", "United Kingdom"),
                ("Birmingham", "England", "United Kingdom"),
                ("Edinburgh", "Scotland", "United Kingdom"),
                ("Paris", "Île-de-France", "France"),
                ("Lyon", "Auvergne-Rhône-Alpes", "France"),
                ("Marseille", "Provence", "France"),
                ("Berlin", "Berlin", "Germany"),
                ("Munich", "Bavaria", "Germany"),
                ("Frankfurt", "Hessen", "Germany"),
                ("Hamburg", "Hamburg", "Germany"),
                ("Amsterdam", "North Holland", "Netherlands"),
                ("Rotterdam", "South Holland", "Netherlands"),
                ("Brussels", "Brussels", "Belgium"),
                ("Madrid", "Madrid", "Spain"),
                ("Barcelona", "Catalonia", "Spain"),
                ("Milan", "Lombardy", "Italy"),
                ("Rome", "Lazio", "Italy"),
                ("Zurich", "Zürich", "Switzerland"),
                ("Vienna", "Vienna", "Austria"),
            ],
            unit_probability: 0.15,
        }
    }

    fn asia_pacific_data() -> RegionalData {
        RegionalData {
            street_names: vec![
                "Orchard",
                "Marina",
                "Raffles",
                "Shenton",
                "Robinson",
                "Cecil",
                "Collyer",
                "Victoria",
                "Queen",
                "King",
                "George",
                "Elizabeth",
                "Bourke",
                "Collins",
                "Flinders",
                "Swanston",
                "Spring",
                "Exhibition",
                "Lonsdale",
            ],
            street_types: vec![
                ("Road", 0.30),
                ("Street", 0.25),
                ("Avenue", 0.15),
                ("Boulevard", 0.10),
                ("Way", 0.10),
                ("Drive", 0.10),
            ],
            cities: vec![
                ("Singapore", "Singapore", "Singapore"),
                ("Hong Kong", "Hong Kong", "Hong Kong"),
                ("Tokyo", "Tokyo", "Japan"),
                ("Osaka", "Osaka", "Japan"),
                ("Seoul", "Seoul", "South Korea"),
                ("Shanghai", "Shanghai", "China"),
                ("Beijing", "Beijing", "China"),
                ("Sydney", "NSW", "Australia"),
                ("Melbourne", "VIC", "Australia"),
                ("Brisbane", "QLD", "Australia"),
                ("Auckland", "Auckland", "New Zealand"),
                ("Mumbai", "Maharashtra", "India"),
                ("Delhi", "Delhi", "India"),
                ("Bangalore", "Karnataka", "India"),
            ],
            unit_probability: 0.30,
        }
    }

    fn latin_america_data() -> RegionalData {
        RegionalData {
            street_names: vec![
                "Reforma",
                "Insurgentes",
                "Juárez",
                "Madero",
                "Hidalgo",
                "Morelos",
                "Independencia",
                "Revolución",
                "Constitución",
                "Victoria",
                "Libertad",
                "Paulista",
                "Faria Lima",
                "Berrini",
                "Brigadeiro",
                "Augusta",
            ],
            street_types: vec![
                ("Avenida", 0.35),
                ("Calle", 0.30),
                ("Boulevard", 0.15),
                ("Paseo", 0.10),
                ("Calzada", 0.10),
            ],
            cities: vec![
                ("Mexico City", "CDMX", "Mexico"),
                ("Guadalajara", "Jalisco", "Mexico"),
                ("Monterrey", "Nuevo León", "Mexico"),
                ("São Paulo", "SP", "Brazil"),
                ("Rio de Janeiro", "RJ", "Brazil"),
                ("Brasília", "DF", "Brazil"),
                ("Buenos Aires", "Buenos Aires", "Argentina"),
                ("Santiago", "Santiago", "Chile"),
                ("Lima", "Lima", "Peru"),
                ("Bogotá", "Cundinamarca", "Colombia"),
            ],
            unit_probability: 0.20,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_address_generation() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let gen = AddressGenerator::new();

        let addr = gen.generate(&mut rng);
        assert!(!addr.street_number.is_empty());
        assert!(!addr.street_name.is_empty());
        assert!(!addr.city.is_empty());
        assert_eq!(addr.region, AddressRegion::NorthAmerica);
    }

    #[test]
    fn test_address_formatting() {
        let addr = Address {
            street_number: "123".to_string(),
            street_name: "Main".to_string(),
            street_type: "Street".to_string(),
            unit: Some("Suite 100".to_string()),
            city: "New York".to_string(),
            state: "NY".to_string(),
            postal_code: "10001".to_string(),
            country: "USA".to_string(),
            region: AddressRegion::NorthAmerica,
        };

        let full = addr.format(AddressStyle::Full);
        assert!(full.contains("123 Main Street"));
        assert!(full.contains("Suite 100"));
        assert!(full.contains("New York"));

        let short = addr.format(AddressStyle::Short);
        assert!(short.contains("123 Main Street"));

        let single = addr.format(AddressStyle::SingleLine);
        assert!(single.contains("10001"));
    }

    #[test]
    fn test_european_format() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let gen = AddressGenerator::for_region(AddressRegion::Europe);

        let addr = gen.generate(&mut rng);
        assert_eq!(addr.region, AddressRegion::Europe);

        let formatted = addr.format(AddressStyle::Full);
        // European format has postal code before city
        assert!(formatted.contains(&addr.postal_code));
    }

    #[test]
    fn test_commercial_address() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let gen = AddressGenerator::new();

        let mut has_unit = false;
        for _ in 0..20 {
            let addr = gen.generate_commercial(&mut rng);
            if addr.unit.is_some() {
                has_unit = true;
                break;
            }
        }
        // Commercial addresses should often have units
        assert!(has_unit);
    }

    #[test]
    fn test_all_regions() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        for region in &[
            AddressRegion::NorthAmerica,
            AddressRegion::Europe,
            AddressRegion::AsiaPacific,
            AddressRegion::LatinAmerica,
        ] {
            let gen = AddressGenerator::for_region(*region);
            let addr = gen.generate(&mut rng);
            assert_eq!(addr.region, *region);
            assert!(!addr.city.is_empty());
        }
    }

    #[test]
    fn test_deterministic_generation() {
        let gen = AddressGenerator::new();

        let mut rng1 = ChaCha8Rng::seed_from_u64(12345);
        let mut rng2 = ChaCha8Rng::seed_from_u64(12345);

        let addr1 = gen.generate(&mut rng1);
        let addr2 = gen.generate(&mut rng2);

        assert_eq!(addr1.street_number, addr2.street_number);
        assert_eq!(addr1.street_name, addr2.street_name);
        assert_eq!(addr1.city, addr2.city);
    }

    #[test]
    fn test_postal_code_format() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let gen = AddressGenerator::new();

        for _ in 0..10 {
            let addr = gen.generate(&mut rng);
            // US ZIP codes should be 5 digits or Canadian format
            assert!(
                addr.postal_code.len() == 5 || addr.postal_code.len() == 7, // Canadian with space
                "Unexpected postal code format: {}",
                addr.postal_code
            );
        }
    }
}
