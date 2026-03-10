//! Canonical standard identifier.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Canonical identifier for a compliance standard.
///
/// Format: `"{BODY}-{NUMBER}"` (e.g., `"IFRS-16"`, `"ISA-315"`, `"SOX-404"`, `"ASC-606"`)
///
/// # Examples
///
/// ```
/// use datasynth_core::models::compliance::StandardId;
///
/// let id = StandardId::new("IFRS", "16");
/// assert_eq!(id.body(), "IFRS");
/// assert_eq!(id.number(), "16");
/// assert_eq!(id.as_str(), "IFRS-16");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StandardId(pub String);

impl StandardId {
    /// Creates a new standard ID from body and number.
    pub fn new(body: &str, number: &str) -> Self {
        Self(format!("{body}-{number}"))
    }

    /// Creates a standard ID from a full string (e.g., "IFRS-16").
    pub fn parse(s: &str) -> Self {
        Self(s.to_string())
    }

    /// Returns the full identifier string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the issuing body prefix (e.g., "IFRS", "ISA", "SOX", "ASC").
    pub fn body(&self) -> &str {
        self.0.split('-').next().unwrap_or("")
    }

    /// Returns the standard number/section (e.g., "16", "315", "404", "606").
    pub fn number(&self) -> &str {
        // Handle multi-part identifiers like "BASEL-III-CAP" or "PCAOB-AS-2201"
        let parts: Vec<&str> = self.0.splitn(2, '-').collect();
        if parts.len() > 1 {
            parts[1]
        } else {
            ""
        }
    }
}

impl fmt::Display for StandardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for StandardId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for StandardId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_id_creation() {
        let id = StandardId::new("IFRS", "16");
        assert_eq!(id.as_str(), "IFRS-16");
        assert_eq!(id.body(), "IFRS");
        assert_eq!(id.number(), "16");
    }

    #[test]
    fn test_standard_id_display() {
        let id = StandardId::new("SOX", "404");
        assert_eq!(format!("{id}"), "SOX-404");
    }

    #[test]
    fn test_standard_id_equality() {
        let a = StandardId::new("ISA", "315");
        let b = StandardId::from("ISA-315");
        assert_eq!(a, b);
    }

    #[test]
    fn test_standard_id_ordering() {
        let mut ids = [
            StandardId::from("SOX-404"),
            StandardId::from("ASC-606"),
            StandardId::from("IFRS-16"),
        ];
        ids.sort();
        assert_eq!(ids[0].as_str(), "ASC-606");
        assert_eq!(ids[1].as_str(), "IFRS-16");
        assert_eq!(ids[2].as_str(), "SOX-404");
    }
}
