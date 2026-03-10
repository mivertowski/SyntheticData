//! Cross-reference links between compliance standards.

use serde::{Deserialize, Serialize};

use super::standard_id::StandardId;

/// Type of relationship between two standards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CrossReferenceType {
    /// Standards are substantially converged (e.g., IFRS 15 ↔ ASC 606)
    Converged,
    /// Related but with significant differences (e.g., IFRS 16 ↔ ASC 842)
    Related,
    /// One standard complements the other
    Complementary,
    /// One standard is derived from or based on the other
    DerivedFrom,
    /// One standard has been incorporated into the other
    IncorporatedInto,
    /// Standards address the same domain from different angles
    Parallel,
    /// ISA ↔ PCAOB mapping
    AuditMapping,
    /// Control framework mapping (COSO ↔ SOX ↔ ISA)
    ControlFrameworkMapping,
}

impl std::fmt::Display for CrossReferenceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Converged => write!(f, "Converged"),
            Self::Related => write!(f, "Related"),
            Self::Complementary => write!(f, "Complementary"),
            Self::DerivedFrom => write!(f, "Derived From"),
            Self::IncorporatedInto => write!(f, "Incorporated Into"),
            Self::Parallel => write!(f, "Parallel"),
            Self::AuditMapping => write!(f, "Audit Mapping"),
            Self::ControlFrameworkMapping => write!(f, "Control Framework Mapping"),
        }
    }
}

/// A cross-reference link between two standards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossReference {
    /// Source standard
    pub from_standard: StandardId,
    /// Target standard
    pub to_standard: StandardId,
    /// Type of relationship
    pub relationship: CrossReferenceType,
    /// Convergence level (0.0 = no convergence, 1.0 = fully converged)
    pub convergence_level: f64,
    /// Description of the relationship
    pub description: Option<String>,
}

impl CrossReference {
    /// Creates a new cross-reference.
    pub fn new(from: StandardId, to: StandardId, relationship: CrossReferenceType) -> Self {
        let convergence = match relationship {
            CrossReferenceType::Converged => 0.9,
            CrossReferenceType::IncorporatedInto => 0.85,
            CrossReferenceType::Related => 0.6,
            CrossReferenceType::Complementary => 0.5,
            CrossReferenceType::DerivedFrom => 0.7,
            CrossReferenceType::Parallel => 0.4,
            CrossReferenceType::AuditMapping => 0.7,
            CrossReferenceType::ControlFrameworkMapping => 0.6,
        };

        Self {
            from_standard: from,
            to_standard: to,
            relationship,
            convergence_level: convergence,
            description: None,
        }
    }

    /// Sets the convergence level explicitly.
    pub fn with_convergence(mut self, level: f64) -> Self {
        self.convergence_level = level.clamp(0.0, 1.0);
        self
    }

    /// Adds a description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cross_reference() {
        let xref = CrossReference::new(
            StandardId::new("IFRS", "15"),
            StandardId::new("ASC", "606"),
            CrossReferenceType::Converged,
        )
        .with_description("Joint standard — substantially converged");

        assert_eq!(xref.from_standard.as_str(), "IFRS-15");
        assert_eq!(xref.to_standard.as_str(), "ASC-606");
        assert!(xref.convergence_level > 0.8);
    }
}
