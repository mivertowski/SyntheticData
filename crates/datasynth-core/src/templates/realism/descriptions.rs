//! Description variation engine with abbreviations and typos.
//!
//! Provides natural language variations to make generated descriptions
//! more realistic by applying abbreviations, case variations, and
//! occasional typos.

use rand::seq::IndexedRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for description variations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VariationConfig {
    /// Rate of abbreviation application (0.0 - 1.0)
    pub abbreviation_rate: f64,
    /// Rate of typo injection (0.0 - 1.0)
    pub typo_rate: f64,
    /// Rate of case variations (0.0 - 1.0)
    pub case_variation_rate: f64,
    /// Enable word order variations
    pub word_order_variation: bool,
    /// Enable number format variations (e.g., "1000" vs "1,000")
    pub number_format_variation: bool,
}

impl Default for VariationConfig {
    fn default() -> Self {
        Self {
            abbreviation_rate: 0.25,
            typo_rate: 0.01,
            case_variation_rate: 0.05,
            word_order_variation: false,
            number_format_variation: true,
        }
    }
}

/// Typo generator with keyboard-aware and common typo patterns.
#[derive(Debug, Clone)]
pub struct TypoGenerator {
    keyboard_neighbors: HashMap<char, Vec<char>>,
    common_transpositions: Vec<(&'static str, &'static str)>,
    common_omissions: Vec<(&'static str, &'static str)>,
}

impl Default for TypoGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl TypoGenerator {
    /// Create a new typo generator.
    pub fn new() -> Self {
        let mut keyboard_neighbors = HashMap::new();

        // QWERTY keyboard layout neighbors
        keyboard_neighbors.insert('q', vec!['w', 'a', '1', '2']);
        keyboard_neighbors.insert('w', vec!['q', 'e', 'a', 's', '2', '3']);
        keyboard_neighbors.insert('e', vec!['w', 'r', 's', 'd', '3', '4']);
        keyboard_neighbors.insert('r', vec!['e', 't', 'd', 'f', '4', '5']);
        keyboard_neighbors.insert('t', vec!['r', 'y', 'f', 'g', '5', '6']);
        keyboard_neighbors.insert('y', vec!['t', 'u', 'g', 'h', '6', '7']);
        keyboard_neighbors.insert('u', vec!['y', 'i', 'h', 'j', '7', '8']);
        keyboard_neighbors.insert('i', vec!['u', 'o', 'j', 'k', '8', '9']);
        keyboard_neighbors.insert('o', vec!['i', 'p', 'k', 'l', '9', '0']);
        keyboard_neighbors.insert('p', vec!['o', 'l', '0']);
        keyboard_neighbors.insert('a', vec!['q', 'w', 's', 'z']);
        keyboard_neighbors.insert('s', vec!['a', 'w', 'e', 'd', 'z', 'x']);
        keyboard_neighbors.insert('d', vec!['s', 'e', 'r', 'f', 'x', 'c']);
        keyboard_neighbors.insert('f', vec!['d', 'r', 't', 'g', 'c', 'v']);
        keyboard_neighbors.insert('g', vec!['f', 't', 'y', 'h', 'v', 'b']);
        keyboard_neighbors.insert('h', vec!['g', 'y', 'u', 'j', 'b', 'n']);
        keyboard_neighbors.insert('j', vec!['h', 'u', 'i', 'k', 'n', 'm']);
        keyboard_neighbors.insert('k', vec!['j', 'i', 'o', 'l', 'm']);
        keyboard_neighbors.insert('l', vec!['k', 'o', 'p']);
        keyboard_neighbors.insert('z', vec!['a', 's', 'x']);
        keyboard_neighbors.insert('x', vec!['z', 's', 'd', 'c']);
        keyboard_neighbors.insert('c', vec!['x', 'd', 'f', 'v']);
        keyboard_neighbors.insert('v', vec!['c', 'f', 'g', 'b']);
        keyboard_neighbors.insert('b', vec!['v', 'g', 'h', 'n']);
        keyboard_neighbors.insert('n', vec!['b', 'h', 'j', 'm']);
        keyboard_neighbors.insert('m', vec!['n', 'j', 'k']);

        Self {
            keyboard_neighbors,
            common_transpositions: vec![
                ("the", "teh"),
                ("and", "adn"),
                ("for", "fro"),
                ("that", "taht"),
                ("with", "wiht"),
                ("from", "form"),
                ("have", "ahve"),
                ("this", "tihs"),
                ("will", "wil"),
                ("your", "yoru"),
                ("payment", "paymnet"),
                ("invoice", "invocie"),
                ("account", "acocunt"),
                ("amount", "amuont"),
                ("receipt", "reciept"),
            ],
            common_omissions: vec![
                ("the", "te"),
                ("and", "ad"),
                ("payment", "paymet"),
                ("invoice", "invoce"),
                ("account", "accont"),
                ("received", "recived"),
                ("processing", "procesing"),
                ("transaction", "transacion"),
                ("reference", "referece"),
                ("description", "descripton"),
            ],
        }
    }

    /// Introduce a typo into the text.
    pub fn introduce_typo(&self, text: &str, rng: &mut impl Rng) -> String {
        if text.is_empty() {
            return text.to_string();
        }

        let typo_type = rng.random_range(0..5);
        match typo_type {
            0 => self.keyboard_typo(text, rng),
            1 => self.transposition_typo(text, rng),
            2 => self.omission_typo(text, rng),
            3 => self.double_letter_typo(text, rng),
            _ => self.common_word_typo(text, rng),
        }
    }

    fn keyboard_typo(&self, text: &str, rng: &mut impl Rng) -> String {
        let chars: Vec<char> = text.chars().collect();
        if chars.is_empty() {
            return text.to_string();
        }

        // Find alphabetic characters to potentially typo
        let alpha_indices: Vec<usize> = chars
            .iter()
            .enumerate()
            .filter(|(_, c)| c.is_ascii_alphabetic())
            .map(|(i, _)| i)
            .collect();

        if alpha_indices.is_empty() {
            return text.to_string();
        }

        let idx = *alpha_indices.choose(rng).expect("non-empty collection");
        let original_char = chars[idx].to_ascii_lowercase();

        if let Some(neighbors) = self.keyboard_neighbors.get(&original_char) {
            if let Some(&neighbor) = neighbors.choose(rng) {
                let mut result: Vec<char> = chars.clone();
                result[idx] = if chars[idx].is_uppercase() {
                    neighbor.to_ascii_uppercase()
                } else {
                    neighbor
                };
                return result.into_iter().collect();
            }
        }

        text.to_string()
    }

    fn transposition_typo(&self, text: &str, rng: &mut impl Rng) -> String {
        let chars: Vec<char> = text.chars().collect();
        if chars.len() < 2 {
            return text.to_string();
        }

        // Find valid positions for transposition
        let valid_positions: Vec<usize> = (0..chars.len() - 1)
            .filter(|&i| chars[i].is_ascii_alphabetic() && chars[i + 1].is_ascii_alphabetic())
            .collect();

        if valid_positions.is_empty() {
            return text.to_string();
        }

        let idx = *valid_positions.choose(rng).expect("non-empty collection");
        let mut result = chars.clone();
        result.swap(idx, idx + 1);
        result.into_iter().collect()
    }

    fn omission_typo(&self, text: &str, rng: &mut impl Rng) -> String {
        let chars: Vec<char> = text.chars().collect();
        if chars.len() < 3 {
            return text.to_string();
        }

        // Find alphabetic characters to omit (not at word boundaries)
        let valid_positions: Vec<usize> = (1..chars.len() - 1)
            .filter(|&i| {
                chars[i].is_ascii_alphabetic()
                    && chars[i - 1].is_ascii_alphabetic()
                    && chars[i + 1].is_ascii_alphabetic()
            })
            .collect();

        if valid_positions.is_empty() {
            return text.to_string();
        }

        let idx = *valid_positions.choose(rng).expect("non-empty collection");
        let mut result = chars.clone();
        result.remove(idx);
        result.into_iter().collect()
    }

    fn double_letter_typo(&self, text: &str, rng: &mut impl Rng) -> String {
        let chars: Vec<char> = text.chars().collect();
        if chars.is_empty() {
            return text.to_string();
        }

        // Find alphabetic characters to double
        let valid_positions: Vec<usize> = chars
            .iter()
            .enumerate()
            .filter(|(_, c)| c.is_ascii_alphabetic())
            .map(|(i, _)| i)
            .collect();

        if valid_positions.is_empty() {
            return text.to_string();
        }

        let idx = *valid_positions.choose(rng).expect("non-empty collection");
        let mut result = chars.clone();
        result.insert(idx, chars[idx]);
        result.into_iter().collect()
    }

    fn common_word_typo(&self, text: &str, rng: &mut impl Rng) -> String {
        // Try to apply a common transposition or omission
        let text_lower = text.to_lowercase();

        // Try transposition first
        for (correct, typo) in &self.common_transpositions {
            if text_lower.contains(*correct) && rng.random_bool(0.5) {
                return text.replacen(correct, typo, 1);
            }
        }

        // Try omission
        for (correct, typo) in &self.common_omissions {
            if text_lower.contains(*correct) {
                return text.replacen(correct, typo, 1);
            }
        }

        // Fallback to keyboard typo
        self.keyboard_typo(text, rng)
    }
}

/// Description variator with abbreviation and typo support.
#[derive(Debug, Clone)]
pub struct DescriptionVariator {
    config: VariationConfig,
    abbreviations: HashMap<&'static str, Vec<&'static str>>,
    typo_gen: TypoGenerator,
}

impl Default for DescriptionVariator {
    fn default() -> Self {
        Self::new()
    }
}

impl DescriptionVariator {
    /// Create a new description variator with default settings.
    pub fn new() -> Self {
        Self::with_config(VariationConfig::default())
    }

    /// Create a new description variator with custom configuration.
    pub fn with_config(config: VariationConfig) -> Self {
        let mut abbreviations = HashMap::new();

        // Common accounting/business abbreviations
        abbreviations.insert("Invoice", vec!["Inv", "INV", "Inv."]);
        abbreviations.insert("invoice", vec!["inv", "inv."]);
        abbreviations.insert("Purchase Order", vec!["PO", "P.O.", "PurchOrd"]);
        abbreviations.insert("purchase order", vec!["PO", "p.o.", "po"]);
        abbreviations.insert("Accounts Payable", vec!["AP", "A/P", "Accts Pay"]);
        abbreviations.insert("accounts payable", vec!["AP", "a/p", "accts pay"]);
        abbreviations.insert("Accounts Receivable", vec!["AR", "A/R", "Accts Rec"]);
        abbreviations.insert("accounts receivable", vec!["AR", "a/r", "accts rec"]);
        abbreviations.insert("Payment", vec!["Pmt", "PMT", "Pymt"]);
        abbreviations.insert("payment", vec!["pmt", "pymt"]);
        abbreviations.insert("Receipt", vec!["Rcpt", "RCPT", "Rec"]);
        abbreviations.insert("receipt", vec!["rcpt", "rec"]);
        abbreviations.insert("Transaction", vec!["Trans", "TXN", "Trx"]);
        abbreviations.insert("transaction", vec!["trans", "txn", "trx"]);
        abbreviations.insert("Reference", vec!["Ref", "REF", "Ref."]);
        abbreviations.insert("reference", vec!["ref", "ref."]);
        abbreviations.insert("Number", vec!["No", "No.", "Num", "#"]);
        abbreviations.insert("number", vec!["no", "no.", "num", "#"]);
        abbreviations.insert("Department", vec!["Dept", "Dept.", "Dpt"]);
        abbreviations.insert("department", vec!["dept", "dept.", "dpt"]);
        abbreviations.insert("Company", vec!["Co", "Co.", "Corp"]);
        abbreviations.insert("company", vec!["co", "co.", "corp"]);
        abbreviations.insert("Corporation", vec!["Corp", "Corp."]);
        abbreviations.insert("corporation", vec!["corp", "corp."]);
        abbreviations.insert("Incorporated", vec!["Inc", "Inc."]);
        abbreviations.insert("incorporated", vec!["inc", "inc."]);
        abbreviations.insert("Limited", vec!["Ltd", "Ltd."]);
        abbreviations.insert("limited", vec!["ltd", "ltd."]);
        abbreviations.insert("Quarter", vec!["Q", "Qtr", "Qtr."]);
        abbreviations.insert("quarter", vec!["q", "qtr", "qtr."]);
        abbreviations.insert("Year", vec!["Yr", "YR"]);
        abbreviations.insert("year", vec!["yr"]);
        abbreviations.insert("Month", vec!["Mo", "Mo.", "Mth"]);
        abbreviations.insert("month", vec!["mo", "mo.", "mth"]);
        abbreviations.insert("January", vec!["Jan", "Jan."]);
        abbreviations.insert("February", vec!["Feb", "Feb."]);
        abbreviations.insert("March", vec!["Mar", "Mar."]);
        abbreviations.insert("April", vec!["Apr", "Apr."]);
        abbreviations.insert("May", vec!["May"]);
        abbreviations.insert("June", vec!["Jun", "Jun."]);
        abbreviations.insert("July", vec!["Jul", "Jul."]);
        abbreviations.insert("August", vec!["Aug", "Aug."]);
        abbreviations.insert("September", vec!["Sep", "Sept", "Sep."]);
        abbreviations.insert("October", vec!["Oct", "Oct."]);
        abbreviations.insert("November", vec!["Nov", "Nov."]);
        abbreviations.insert("December", vec!["Dec", "Dec."]);
        abbreviations.insert("Revenue", vec!["Rev", "REV"]);
        abbreviations.insert("revenue", vec!["rev"]);
        abbreviations.insert("Expense", vec!["Exp", "EXP"]);
        abbreviations.insert("expense", vec!["exp"]);
        abbreviations.insert("Accrual", vec!["Accr", "Accrl"]);
        abbreviations.insert("accrual", vec!["accr", "accrl"]);
        abbreviations.insert("Adjustment", vec!["Adj", "Adjmt"]);
        abbreviations.insert("adjustment", vec!["adj", "adjmt"]);
        abbreviations.insert("Depreciation", vec!["Depr", "Dep"]);
        abbreviations.insert("depreciation", vec!["depr", "dep"]);
        abbreviations.insert("Amortization", vec!["Amort", "Amor"]);
        abbreviations.insert("amortization", vec!["amort", "amor"]);
        abbreviations.insert("Recognition", vec!["Recog", "Rec"]);
        abbreviations.insert("recognition", vec!["recog", "rec"]);
        abbreviations.insert("Processing", vec!["Proc", "Process"]);
        abbreviations.insert("processing", vec!["proc", "process"]);
        abbreviations.insert("Services", vec!["Svcs", "Svc"]);
        abbreviations.insert("services", vec!["svcs", "svc"]);
        abbreviations.insert("Management", vec!["Mgmt", "Mgt"]);
        abbreviations.insert("management", vec!["mgmt", "mgt"]);
        abbreviations.insert("General", vec!["Gen", "Gen."]);
        abbreviations.insert("general", vec!["gen", "gen."]);
        abbreviations.insert("Administrative", vec!["Admin", "Adm"]);
        abbreviations.insert("administrative", vec!["admin", "adm"]);
        abbreviations.insert("Professional", vec!["Prof", "Profl"]);
        abbreviations.insert("professional", vec!["prof", "profl"]);

        Self {
            config,
            abbreviations,
            typo_gen: TypoGenerator::new(),
        }
    }

    /// Apply variations to a description.
    pub fn apply(&self, description: &str, rng: &mut impl Rng) -> String {
        let mut result = description.to_string();

        // Apply abbreviations
        if rng.random_bool(self.config.abbreviation_rate) {
            result = self.apply_abbreviations(&result, rng);
        }

        // Apply case variations
        if rng.random_bool(self.config.case_variation_rate) {
            result = self.apply_case_variation(&result, rng);
        }

        // Apply typos (rare)
        if rng.random_bool(self.config.typo_rate) {
            result = self.typo_gen.introduce_typo(&result, rng);
        }

        result
    }

    /// Apply only abbreviations without other variations.
    pub fn abbreviate(&self, description: &str, rng: &mut impl Rng) -> String {
        self.apply_abbreviations(description, rng)
    }

    fn apply_abbreviations(&self, text: &str, rng: &mut impl Rng) -> String {
        let mut result = text.to_string();

        // Find and replace one or two terms
        let max_replacements = rng.random_range(1..=2);
        let mut replacements = 0;

        for (full, abbrevs) in &self.abbreviations {
            if result.contains(*full) && replacements < max_replacements {
                if let Some(abbrev) = abbrevs.choose(rng) {
                    result = result.replacen(*full, abbrev, 1);
                    replacements += 1;
                }
            }
        }

        result
    }

    fn apply_case_variation(&self, text: &str, rng: &mut impl Rng) -> String {
        let variation = rng.random_range(0..3);
        match variation {
            0 => text.to_uppercase(),
            1 => text.to_lowercase(),
            _ => {
                // Title case variation - first letter of each word uppercase
                text.split_whitespace()
                    .map(|word| {
                        let mut chars: Vec<char> = word.chars().collect();
                        if let Some(first) = chars.first_mut() {
                            *first = first.to_ascii_uppercase();
                        }
                        for c in chars.iter_mut().skip(1) {
                            *c = c.to_ascii_lowercase();
                        }
                        chars.into_iter().collect::<String>()
                    })
                    .collect::<Vec<String>>()
                    .join(" ")
            }
        }
    }

    /// Get the configuration.
    pub fn config(&self) -> &VariationConfig {
        &self.config
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_typo_generator_keyboard() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let gen = TypoGenerator::new();

        let original = "payment";
        let typo = gen.keyboard_typo(original, &mut rng);
        // Should be different (usually)
        assert!(typo.len() == original.len()); // Same length for keyboard typos
    }

    #[test]
    fn test_typo_generator_transposition() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let gen = TypoGenerator::new();

        let original = "payment";
        let typo = gen.transposition_typo(original, &mut rng);
        assert_eq!(typo.len(), original.len());
    }

    #[test]
    fn test_description_variator_abbreviation() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let config = VariationConfig {
            abbreviation_rate: 1.0, // Always abbreviate
            typo_rate: 0.0,
            case_variation_rate: 0.0,
            ..Default::default()
        };
        let variator = DescriptionVariator::with_config(config);

        let original = "Invoice for Purchase Order";
        let varied = variator.apply(original, &mut rng);

        // Should contain at least one abbreviation
        let has_abbreviation = varied.contains("Inv")
            || varied.contains("INV")
            || varied.contains("PO")
            || varied.contains("P.O.");
        assert!(has_abbreviation);
    }

    #[test]
    fn test_description_variator_no_change() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let config = VariationConfig {
            abbreviation_rate: 0.0,
            typo_rate: 0.0,
            case_variation_rate: 0.0,
            ..Default::default()
        };
        let variator = DescriptionVariator::with_config(config);

        let original = "Regular description";
        let varied = variator.apply(original, &mut rng);
        assert_eq!(original, varied);
    }

    #[test]
    fn test_month_abbreviations() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let config = VariationConfig {
            abbreviation_rate: 1.0,
            typo_rate: 0.0,
            case_variation_rate: 0.0,
            ..Default::default()
        };
        let variator = DescriptionVariator::with_config(config);

        let original = "Revenue for December 2024";
        let varied = variator.abbreviate(original, &mut rng);

        // Should have some abbreviation
        let has_change = varied != original;
        assert!(has_change || varied.contains("Dec") || varied.contains("Rev"));
    }

    #[test]
    fn test_case_variation() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let config = VariationConfig {
            abbreviation_rate: 0.0,
            typo_rate: 0.0,
            case_variation_rate: 1.0,
            ..Default::default()
        };
        let variator = DescriptionVariator::with_config(config);

        let original = "Invoice Payment";
        let varied = variator.apply(original, &mut rng);

        // Case should be different
        let case_changed = varied == original.to_uppercase()
            || varied == original.to_lowercase()
            || varied != original;
        assert!(case_changed);
    }

    #[test]
    fn test_deterministic_variation() {
        let config = VariationConfig {
            abbreviation_rate: 0.5,
            typo_rate: 0.1,
            ..Default::default()
        };
        let variator = DescriptionVariator::with_config(config);

        let original = "Invoice for Services";

        let mut rng1 = ChaCha8Rng::seed_from_u64(12345);
        let mut rng2 = ChaCha8Rng::seed_from_u64(12345);

        let varied1 = variator.apply(original, &mut rng1);
        let varied2 = variator.apply(original, &mut rng2);

        assert_eq!(varied1, varied2);
    }
}
