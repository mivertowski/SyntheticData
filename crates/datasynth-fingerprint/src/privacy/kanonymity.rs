//! K-anonymity mechanisms.

/// K-anonymity filter for categorical values.
pub struct KAnonymity {
    /// Minimum group size (k).
    k: u32,
    /// Minimum occurrence threshold.
    min_occurrence: u32,
}

impl KAnonymity {
    /// Create a new k-anonymity filter.
    pub fn new(k: u32, min_occurrence: u32) -> Self {
        Self { k, min_occurrence }
    }

    /// Filter frequencies to ensure k-anonymity.
    ///
    /// Returns (kept values with frequencies, count of suppressed values).
    pub fn filter_frequencies(
        &self,
        frequencies: Vec<(String, u64)>,
        total: u64,
    ) -> (Vec<(String, f64)>, usize) {
        if total == 0 {
            tracing::warn!("K-anonymity filter called with total=0, returning empty frequencies");
            return (Vec::new(), frequencies.len());
        }
        let threshold = self.k.max(self.min_occurrence) as u64;

        let mut kept = Vec::new();
        let mut suppressed_count = 0;
        let mut suppressed_total = 0u64;

        for (value, count) in frequencies {
            if count >= threshold {
                kept.push((value, count as f64 / total as f64));
            } else {
                suppressed_count += 1;
                suppressed_total += count;
            }
        }

        // If significant suppression, add an "Other" category
        if suppressed_total > 0 && total > 0 {
            kept.push((
                "__OTHER__".to_string(),
                suppressed_total as f64 / total as f64,
            ));
        }

        (kept, suppressed_count)
    }

    /// Check if a value meets the k-anonymity threshold.
    pub fn meets_threshold(&self, count: u64) -> bool {
        count >= self.k as u64
    }

    /// Get the k value.
    pub fn k(&self) -> u32 {
        self.k
    }
}

/// Generalization strategies for quasi-identifiers.
pub mod generalization {
    /// Generalize a numeric value to a range.
    pub fn numeric_to_range(value: f64, bin_size: f64) -> (f64, f64) {
        let bin = (value / bin_size).floor();
        (bin * bin_size, (bin + 1.0) * bin_size)
    }

    /// Generalize an age to an age group.
    pub fn age_to_group(age: u32) -> String {
        match age {
            0..=17 => "0-17".to_string(),
            18..=24 => "18-24".to_string(),
            25..=34 => "25-34".to_string(),
            35..=44 => "35-44".to_string(),
            45..=54 => "45-54".to_string(),
            55..=64 => "55-64".to_string(),
            _ => "65+".to_string(),
        }
    }

    /// Generalize a date to month.
    pub fn date_to_month(date: &str) -> Option<String> {
        // Assumes YYYY-MM-DD format
        date.get(0..7).map(|s| s.to_string())
    }

    /// Generalize a date to year.
    pub fn date_to_year(date: &str) -> Option<String> {
        date.get(0..4).map(|s| s.to_string())
    }

    /// Generalize a zip code to prefix.
    pub fn zip_to_prefix(zip: &str, digits: usize) -> String {
        let prefix = zip.chars().take(digits).collect::<String>();
        format!("{}*", prefix)
    }
}

/// L-diversity check for sensitive attributes.
pub struct LDiversity {
    /// Minimum distinct sensitive values per equivalence class.
    l: usize,
}

impl LDiversity {
    /// Create a new l-diversity checker.
    pub fn new(l: usize) -> Self {
        Self { l }
    }

    /// Check if a group satisfies l-diversity.
    pub fn satisfies(&self, sensitive_values: &[String]) -> bool {
        let unique: std::collections::HashSet<_> = sensitive_values.iter().collect();
        unique.len() >= self.l
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_k_anonymity_filtering() {
        let kanon = KAnonymity::new(5, 5);

        let frequencies = vec![
            ("A".to_string(), 100),
            ("B".to_string(), 50),
            ("C".to_string(), 3), // Below threshold
            ("D".to_string(), 2), // Below threshold
        ];

        let (kept, suppressed) = kanon.filter_frequencies(frequencies, 155);

        assert_eq!(suppressed, 2);
        assert!(kept.iter().any(|(v, _)| v == "A"));
        assert!(kept.iter().any(|(v, _)| v == "B"));
        assert!(!kept.iter().any(|(v, _)| v == "C"));
        assert!(kept.iter().any(|(v, _)| v == "__OTHER__"));
    }

    #[test]
    fn test_generalization() {
        assert_eq!(generalization::age_to_group(25), "25-34");
        assert_eq!(generalization::age_to_group(65), "65+");
        assert_eq!(
            generalization::date_to_month("2024-03-15"),
            Some("2024-03".to_string())
        );
        assert_eq!(generalization::zip_to_prefix("12345", 3), "123*");
    }
}
