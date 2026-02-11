//! Typo and character error injection for data quality simulation.
//!
//! Simulates realistic typing errors:
//! - Character substitution (nearby keys)
//! - Character transposition (adjacent swaps)
//! - Character insertion (double-typing)
//! - Character deletion (missed keys)
//! - Encoding issues (character corruption)

use rand::Rng;
use std::collections::HashMap;

/// Type of typo/error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypoType {
    /// Substitution with nearby key.
    Substitution,
    /// Adjacent character transposition.
    Transposition,
    /// Extra character insertion.
    Insertion,
    /// Missing character deletion.
    Deletion,
    /// Double character (repetition).
    DoubleChar,
    /// Case error (wrong case).
    CaseError,
    /// Homophone substitution.
    Homophone,
    /// OCR-style error (similar looking characters).
    OCRError,
}

impl TypoType {
    /// Returns all typo types.
    pub fn all() -> Vec<Self> {
        vec![
            TypoType::Substitution,
            TypoType::Transposition,
            TypoType::Insertion,
            TypoType::Deletion,
            TypoType::DoubleChar,
            TypoType::CaseError,
            TypoType::Homophone,
            TypoType::OCRError,
        ]
    }
}

/// Configuration for typo generation.
#[derive(Debug, Clone)]
pub struct TypoConfig {
    /// Overall typo rate (per character).
    pub char_error_rate: f64,
    /// Weights for each typo type.
    pub type_weights: HashMap<TypoType, f64>,
    /// Whether to preserve word boundaries.
    pub preserve_word_boundaries: bool,
    /// Maximum typos per word.
    pub max_typos_per_word: usize,
    /// Fields that should not have typos (identifiers, codes).
    pub protected_fields: Vec<String>,
}

impl Default for TypoConfig {
    fn default() -> Self {
        let mut type_weights = HashMap::new();
        type_weights.insert(TypoType::Substitution, 0.25);
        type_weights.insert(TypoType::Transposition, 0.20);
        type_weights.insert(TypoType::Insertion, 0.10);
        type_weights.insert(TypoType::Deletion, 0.20);
        type_weights.insert(TypoType::DoubleChar, 0.10);
        type_weights.insert(TypoType::CaseError, 0.10);
        type_weights.insert(TypoType::OCRError, 0.05);

        Self {
            char_error_rate: 0.005, // 0.5% per character
            type_weights,
            preserve_word_boundaries: true,
            max_typos_per_word: 2,
            protected_fields: vec![
                "document_number".to_string(),
                "account_code".to_string(),
                "company_code".to_string(),
                "vendor_id".to_string(),
                "customer_id".to_string(),
            ],
        }
    }
}

/// QWERTY keyboard layout for nearby key substitution.
pub struct KeyboardLayout {
    /// Map from character to nearby characters.
    nearby_keys: HashMap<char, Vec<char>>,
}

impl Default for KeyboardLayout {
    fn default() -> Self {
        Self::qwerty()
    }
}

impl KeyboardLayout {
    /// Creates a QWERTY keyboard layout.
    pub fn qwerty() -> Self {
        let mut nearby_keys = HashMap::new();

        // Row 1: qwertyuiop
        nearby_keys.insert('q', vec!['w', 'a', '1', '2']);
        nearby_keys.insert('w', vec!['q', 'e', 'a', 's', '2', '3']);
        nearby_keys.insert('e', vec!['w', 'r', 's', 'd', '3', '4']);
        nearby_keys.insert('r', vec!['e', 't', 'd', 'f', '4', '5']);
        nearby_keys.insert('t', vec!['r', 'y', 'f', 'g', '5', '6']);
        nearby_keys.insert('y', vec!['t', 'u', 'g', 'h', '6', '7']);
        nearby_keys.insert('u', vec!['y', 'i', 'h', 'j', '7', '8']);
        nearby_keys.insert('i', vec!['u', 'o', 'j', 'k', '8', '9']);
        nearby_keys.insert('o', vec!['i', 'p', 'k', 'l', '9', '0']);
        nearby_keys.insert('p', vec!['o', 'l', '0']);

        // Row 2: asdfghjkl
        nearby_keys.insert('a', vec!['q', 'w', 's', 'z']);
        nearby_keys.insert('s', vec!['a', 'w', 'e', 'd', 'z', 'x']);
        nearby_keys.insert('d', vec!['s', 'e', 'r', 'f', 'x', 'c']);
        nearby_keys.insert('f', vec!['d', 'r', 't', 'g', 'c', 'v']);
        nearby_keys.insert('g', vec!['f', 't', 'y', 'h', 'v', 'b']);
        nearby_keys.insert('h', vec!['g', 'y', 'u', 'j', 'b', 'n']);
        nearby_keys.insert('j', vec!['h', 'u', 'i', 'k', 'n', 'm']);
        nearby_keys.insert('k', vec!['j', 'i', 'o', 'l', 'm']);
        nearby_keys.insert('l', vec!['k', 'o', 'p']);

        // Row 3: zxcvbnm
        nearby_keys.insert('z', vec!['a', 's', 'x']);
        nearby_keys.insert('x', vec!['z', 's', 'd', 'c']);
        nearby_keys.insert('c', vec!['x', 'd', 'f', 'v']);
        nearby_keys.insert('v', vec!['c', 'f', 'g', 'b']);
        nearby_keys.insert('b', vec!['v', 'g', 'h', 'n']);
        nearby_keys.insert('n', vec!['b', 'h', 'j', 'm']);
        nearby_keys.insert('m', vec!['n', 'j', 'k']);

        // Numbers
        nearby_keys.insert('1', vec!['2', 'q']);
        nearby_keys.insert('2', vec!['1', '3', 'q', 'w']);
        nearby_keys.insert('3', vec!['2', '4', 'w', 'e']);
        nearby_keys.insert('4', vec!['3', '5', 'e', 'r']);
        nearby_keys.insert('5', vec!['4', '6', 'r', 't']);
        nearby_keys.insert('6', vec!['5', '7', 't', 'y']);
        nearby_keys.insert('7', vec!['6', '8', 'y', 'u']);
        nearby_keys.insert('8', vec!['7', '9', 'u', 'i']);
        nearby_keys.insert('9', vec!['8', '0', 'i', 'o']);
        nearby_keys.insert('0', vec!['9', 'o', 'p']);

        Self { nearby_keys }
    }

    /// Gets nearby keys for a character.
    pub fn get_nearby(&self, c: char) -> Vec<char> {
        self.nearby_keys
            .get(&c.to_ascii_lowercase())
            .cloned()
            .unwrap_or_else(|| vec![c])
    }
}

/// OCR-similar characters (often confused in OCR).
pub struct OCRConfusions {
    /// Map from character to similar-looking characters.
    confusions: HashMap<char, Vec<char>>,
}

impl Default for OCRConfusions {
    fn default() -> Self {
        Self::new()
    }
}

impl OCRConfusions {
    /// Creates OCR confusion mappings.
    pub fn new() -> Self {
        let mut confusions = HashMap::new();

        // Commonly confused characters
        confusions.insert('0', vec!['O', 'o', 'Q', 'D']);
        confusions.insert('O', vec!['0', 'Q', 'D', 'o']);
        confusions.insert('o', vec!['0', 'O', 'a', 'e']);
        confusions.insert('1', vec!['l', 'I', 'i', '|', '7']);
        confusions.insert('l', vec!['1', 'I', 'i', '|']);
        confusions.insert('I', vec!['1', 'l', 'i', '|']);
        confusions.insert('i', vec!['1', 'l', 'I', 'j']);
        confusions.insert('5', vec!['S', 's']);
        confusions.insert('S', vec!['5', 's', '8']);
        confusions.insert('s', vec!['5', 'S', 'z']);
        confusions.insert('8', vec!['B', '&', 'S']);
        confusions.insert('B', vec!['8', 'R', 'D']);
        confusions.insert('6', vec!['G', 'b']);
        confusions.insert('G', vec!['6', 'C', 'O']);
        confusions.insert('2', vec!['Z', 'z']);
        confusions.insert('Z', vec!['2', 'z', '7']);
        confusions.insert('z', vec!['2', 'Z', 's']);
        confusions.insert('n', vec!['m', 'h', 'r']);
        confusions.insert('m', vec!['n', 'r']);
        confusions.insert('h', vec!['n', 'b', 'k']);
        confusions.insert('c', vec!['e', 'o', '(']);
        confusions.insert('e', vec!['c', 'a', 'o']);
        confusions.insert('a', vec!['e', 'o', 'd']);
        confusions.insert('d', vec!['a', 'o', 'c']);
        confusions.insert('g', vec!['q', '9', 'a']);
        confusions.insert('q', vec!['g', '9', 'p']);
        confusions.insert('9', vec!['g', 'q']);
        confusions.insert('v', vec!['u', 'w', 'y']);
        confusions.insert('u', vec!['v', 'n', 'w']);
        confusions.insert('w', vec!['v', 'u', 'x']);
        confusions.insert('y', vec!['v', 'u', 'j']);
        confusions.insert('f', vec!['t', 'r']);
        confusions.insert('t', vec!['f', 'l', '+']);
        confusions.insert('r', vec!['n', 'f']);

        Self { confusions }
    }

    /// Gets OCR-confusable characters.
    pub fn get_confusions(&self, c: char) -> Vec<char> {
        self.confusions.get(&c).cloned().unwrap_or_else(|| vec![c])
    }
}

/// Common homophones (words that sound alike).
pub struct Homophones {
    /// Map from word to homophones.
    homophones: HashMap<String, Vec<String>>,
}

impl Default for Homophones {
    fn default() -> Self {
        Self::new()
    }
}

impl Homophones {
    /// Creates homophone mappings.
    pub fn new() -> Self {
        let mut homophones = HashMap::new();

        // Common business/accounting homophones
        homophones.insert("to".to_string(), vec!["two".to_string(), "too".to_string()]);
        homophones.insert("two".to_string(), vec!["to".to_string(), "too".to_string()]);
        homophones.insert(
            "their".to_string(),
            vec!["there".to_string(), "they're".to_string()],
        );
        homophones.insert(
            "there".to_string(),
            vec!["their".to_string(), "they're".to_string()],
        );
        homophones.insert("its".to_string(), vec!["it's".to_string()]);
        homophones.insert("your".to_string(), vec!["you're".to_string()]);
        homophones.insert("than".to_string(), vec!["then".to_string()]);
        homophones.insert("then".to_string(), vec!["than".to_string()]);
        homophones.insert("accept".to_string(), vec!["except".to_string()]);
        homophones.insert("affect".to_string(), vec!["effect".to_string()]);
        homophones.insert("effect".to_string(), vec!["affect".to_string()]);
        homophones.insert("capital".to_string(), vec!["capitol".to_string()]);
        homophones.insert("principal".to_string(), vec!["principle".to_string()]);
        homophones.insert("compliment".to_string(), vec!["complement".to_string()]);
        homophones.insert("stationary".to_string(), vec!["stationery".to_string()]);
        homophones.insert("advice".to_string(), vec!["advise".to_string()]);
        homophones.insert(
            "loss".to_string(),
            vec!["lost".to_string(), "lose".to_string()],
        );

        Self { homophones }
    }

    /// Gets homophones for a word.
    pub fn get_homophones(&self, word: &str) -> Option<&Vec<String>> {
        self.homophones.get(&word.to_lowercase())
    }
}

/// Typo generator.
pub struct TypoGenerator {
    config: TypoConfig,
    keyboard: KeyboardLayout,
    ocr: OCRConfusions,
    homophones: Homophones,
    stats: TypoStats,
}

/// Statistics for typo generation.
#[derive(Debug, Clone, Default)]
pub struct TypoStats {
    pub total_characters: usize,
    pub total_typos: usize,
    pub by_type: HashMap<TypoType, usize>,
    pub total_words: usize,
    pub words_with_typos: usize,
}

impl TypoGenerator {
    /// Creates a new typo generator.
    pub fn new(config: TypoConfig) -> Self {
        Self {
            config,
            keyboard: KeyboardLayout::default(),
            ocr: OCRConfusions::default(),
            homophones: Homophones::default(),
            stats: TypoStats::default(),
        }
    }

    /// Introduces typos into text.
    pub fn introduce_typos<R: Rng>(&mut self, text: &str, rng: &mut R) -> String {
        if self.config.preserve_word_boundaries {
            self.introduce_typos_by_word(text, rng)
        } else {
            self.introduce_typos_by_char(text, rng)
        }
    }

    /// Introduces typos word by word.
    fn introduce_typos_by_word<R: Rng>(&mut self, text: &str, rng: &mut R) -> String {
        let mut result = String::new();
        let chars = text.chars().peekable();
        let mut current_word = String::new();

        for c in chars {
            if c.is_alphanumeric() {
                current_word.push(c);
            } else {
                // Process the word
                if !current_word.is_empty() {
                    self.stats.total_words += 1;
                    let processed = self.process_word(&current_word, rng);
                    if processed != current_word {
                        self.stats.words_with_typos += 1;
                    }
                    result.push_str(&processed);
                    current_word.clear();
                }
                result.push(c);
            }
        }

        // Process remaining word
        if !current_word.is_empty() {
            self.stats.total_words += 1;
            let processed = self.process_word(&current_word, rng);
            if processed != current_word {
                self.stats.words_with_typos += 1;
            }
            result.push_str(&processed);
        }

        result
    }

    /// Processes a single word for typos.
    fn process_word<R: Rng>(&mut self, word: &str, rng: &mut R) -> String {
        // Check for homophone substitution first
        if let Some(homophones) = self.homophones.get_homophones(word) {
            if rng.gen::<f64>() < self.config.char_error_rate * 10.0 {
                // Higher probability for whole-word substitution
                self.stats.total_typos += 1;
                *self.stats.by_type.entry(TypoType::Homophone).or_insert(0) += 1;
                return homophones[rng.gen_range(0..homophones.len())].clone();
            }
        }

        let mut result: Vec<char> = word.chars().collect();
        let mut typos_in_word = 0;
        let mut i = 0;

        while i < result.len() {
            if typos_in_word >= self.config.max_typos_per_word {
                break;
            }

            self.stats.total_characters += 1;

            if rng.gen::<f64>() < self.config.char_error_rate {
                let typo_type = self.select_typo_type(rng);
                let c = result[i];

                match typo_type {
                    TypoType::Substitution => {
                        let nearby = self.keyboard.get_nearby(c);
                        if !nearby.is_empty() {
                            result[i] = nearby[rng.gen_range(0..nearby.len())];
                        }
                    }
                    TypoType::Transposition => {
                        if i + 1 < result.len() {
                            result.swap(i, i + 1);
                        }
                    }
                    TypoType::Deletion => {
                        if result.len() > 1 {
                            result.remove(i);
                            // Don't increment i since we removed the current element
                            // Stats are tracked below, just continue to avoid index issues
                            self.stats.total_typos += 1;
                            *self.stats.by_type.entry(typo_type).or_insert(0) += 1;
                            typos_in_word += 1;
                            continue;
                        }
                    }
                    TypoType::Insertion => {
                        let nearby = self.keyboard.get_nearby(c);
                        if !nearby.is_empty() {
                            result.insert(i, nearby[rng.gen_range(0..nearby.len())]);
                            // Skip the inserted character
                            i += 1;
                        }
                    }
                    TypoType::DoubleChar => {
                        result.insert(i, c);
                        // Skip the duplicated character
                        i += 1;
                    }
                    TypoType::CaseError => {
                        if c.is_uppercase() {
                            result[i] = c.to_ascii_lowercase();
                        } else {
                            result[i] = c.to_ascii_uppercase();
                        }
                    }
                    TypoType::OCRError => {
                        let confusions = self.ocr.get_confusions(c);
                        if !confusions.is_empty() {
                            result[i] = confusions[rng.gen_range(0..confusions.len())];
                        }
                    }
                    TypoType::Homophone => {
                        // Already handled above
                    }
                }

                self.stats.total_typos += 1;
                *self.stats.by_type.entry(typo_type).or_insert(0) += 1;
                typos_in_word += 1;
            }
            i += 1;
        }

        result.into_iter().collect()
    }

    /// Introduces typos character by character.
    fn introduce_typos_by_char<R: Rng>(&mut self, text: &str, rng: &mut R) -> String {
        let mut result = String::new();

        for c in text.chars() {
            self.stats.total_characters += 1;

            if c.is_alphanumeric() && rng.gen::<f64>() < self.config.char_error_rate {
                let typo_type = self.select_typo_type(rng);

                match typo_type {
                    TypoType::Substitution => {
                        let nearby = self.keyboard.get_nearby(c);
                        if !nearby.is_empty() {
                            result.push(nearby[rng.gen_range(0..nearby.len())]);
                        } else {
                            result.push(c);
                        }
                    }
                    TypoType::Deletion => {
                        // Skip character (deletion)
                    }
                    TypoType::Insertion => {
                        result.push(c);
                        let nearby = self.keyboard.get_nearby(c);
                        if !nearby.is_empty() {
                            result.push(nearby[rng.gen_range(0..nearby.len())]);
                        }
                    }
                    TypoType::DoubleChar => {
                        result.push(c);
                        result.push(c);
                    }
                    TypoType::CaseError => {
                        if c.is_uppercase() {
                            result.push(c.to_ascii_lowercase());
                        } else {
                            result.push(c.to_ascii_uppercase());
                        }
                    }
                    _ => {
                        result.push(c);
                    }
                }

                self.stats.total_typos += 1;
                *self.stats.by_type.entry(typo_type).or_insert(0) += 1;
            } else {
                result.push(c);
            }
        }

        result
    }

    /// Selects a typo type based on weights.
    fn select_typo_type<R: Rng>(&self, rng: &mut R) -> TypoType {
        let total_weight: f64 = self.config.type_weights.values().sum();
        let mut random_weight = rng.gen::<f64>() * total_weight;

        for (typo_type, weight) in &self.config.type_weights {
            random_weight -= weight;
            if random_weight <= 0.0 {
                return *typo_type;
            }
        }

        TypoType::Substitution // Default fallback
    }

    /// Checks if a field is protected.
    pub fn is_protected(&self, field: &str) -> bool {
        self.config.protected_fields.contains(&field.to_string())
    }

    /// Returns statistics.
    pub fn stats(&self) -> &TypoStats {
        &self.stats
    }

    /// Resets statistics.
    pub fn reset_stats(&mut self) {
        self.stats = TypoStats::default();
    }
}

/// Encoding issue types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EncodingIssue {
    /// Mojibake (wrong encoding interpretation).
    Mojibake,
    /// Missing characters (replaced with ?).
    MissingChars,
    /// UTF-8 BOM inserted.
    BOM,
    /// Control characters inserted.
    ControlChars,
    /// HTML entities.
    HTMLEntities,
}

/// Introduces encoding issues.
pub fn introduce_encoding_issue<R: Rng>(text: &str, issue: EncodingIssue, rng: &mut R) -> String {
    match issue {
        EncodingIssue::Mojibake => {
            // Simulate common Mojibake patterns
            text.replace('é', "Ã©")
                .replace('ñ', "Ã±")
                .replace('ü', "Ã¼")
                .replace('ö', "Ã¶")
                .replace('ä', "Ã¤")
                .replace('€', "â‚¬")
        }
        EncodingIssue::MissingChars => text
            .chars()
            .map(|c| {
                if !c.is_ascii() && rng.gen::<f64>() < 0.5 {
                    '?'
                } else {
                    c
                }
            })
            .collect(),
        EncodingIssue::BOM => {
            format!("\u{FEFF}{}", text)
        }
        EncodingIssue::ControlChars => {
            let mut result = String::new();
            for c in text.chars() {
                result.push(c);
                if rng.gen::<f64>() < 0.01 {
                    // Insert random control character
                    result.push('\u{0000}');
                }
            }
            result
        }
        EncodingIssue::HTMLEntities => text
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace(' ', "&nbsp;"),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_keyboard_nearby_keys() {
        let keyboard = KeyboardLayout::qwerty();
        let nearby = keyboard.get_nearby('e');
        assert!(nearby.contains(&'w'));
        assert!(nearby.contains(&'r'));
        assert!(nearby.contains(&'s'));
        assert!(nearby.contains(&'d'));
    }

    #[test]
    fn test_typo_generation() {
        let config = TypoConfig {
            char_error_rate: 0.5, // High rate for testing
            ..Default::default()
        };

        let mut generator = TypoGenerator::new(config);
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let text = "Hello World";
        let _with_typos = generator.introduce_typos(text, &mut rng);

        // With high error rate, should have some typos
        assert!(generator.stats().total_typos > 0);
    }

    #[test]
    fn test_encoding_issues() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let text = "Héllo & Wörld";
        let mojibake = introduce_encoding_issue(text, EncodingIssue::Mojibake, &mut rng);
        assert!(mojibake.contains("Ã©"));

        let html = introduce_encoding_issue("A & B", EncodingIssue::HTMLEntities, &mut rng);
        assert!(html.contains("&amp;"));
    }

    #[test]
    fn test_homophones() {
        let homophones = Homophones::new();
        let alternatives = homophones.get_homophones("their");
        assert!(alternatives.is_some());
        assert!(alternatives.unwrap().contains(&"there".to_string()));
    }
}
