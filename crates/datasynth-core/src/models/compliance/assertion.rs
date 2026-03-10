//! Audit assertions per ISA 315.

use serde::{Deserialize, Serialize};

/// Audit assertion types per ISA 315 (Revised 2019).
///
/// These cover transaction-level, balance-level, and disclosure assertions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAssertion {
    // ── Transaction-level assertions ──
    /// Transactions and events that have been recorded have occurred and pertain to the entity
    Occurrence,
    /// All transactions and events that should have been recorded have been recorded
    Completeness,
    /// Amounts and other data relating to recorded transactions have been recorded appropriately
    Accuracy,
    /// Transactions and events have been recorded in the correct accounting period
    Cutoff,
    /// Transactions and events have been recorded in the proper accounts
    Classification,

    // ── Balance-level assertions ──
    /// Assets, liabilities, and equity interests exist
    Existence,
    /// The entity holds or controls the rights to assets; liabilities are obligations of the entity
    RightsAndObligations,
    /// All assets, liabilities, and equity interests that should have been recorded have been recorded
    CompletenessBalance,
    /// Assets, liabilities, and equity interests are included at appropriate amounts
    ValuationAndAllocation,

    // ── Disclosure assertions ──
    /// Disclosed events, transactions, and other matters have occurred and pertain to the entity
    OccurrenceAndRightsDisclosure,
    /// All disclosures that should have been included have been included
    CompletenessDisclosure,
    /// Financial and other information are disclosed fairly and at appropriate amounts
    AccuracyAndValuation,
    /// Financial information is appropriately presented and described, and disclosures are clearly expressed
    ClassificationAndUnderstandability,

    // ── Additional (timeliness / presentation) ──
    /// Information is recorded and reported in a timely manner
    Timeliness,
    /// Information is presented in an appropriate manner
    Presentation,
}

impl ComplianceAssertion {
    /// Returns the assertion category.
    pub fn category(&self) -> AssertionCategory {
        match self {
            Self::Occurrence
            | Self::Completeness
            | Self::Accuracy
            | Self::Cutoff
            | Self::Classification => AssertionCategory::Transaction,

            Self::Existence
            | Self::RightsAndObligations
            | Self::CompletenessBalance
            | Self::ValuationAndAllocation => AssertionCategory::Balance,

            Self::OccurrenceAndRightsDisclosure
            | Self::CompletenessDisclosure
            | Self::AccuracyAndValuation
            | Self::ClassificationAndUnderstandability => AssertionCategory::Disclosure,

            Self::Timeliness | Self::Presentation => AssertionCategory::Presentation,
        }
    }

    /// Returns a numeric encoding for ML features.
    pub fn feature_code(&self) -> f64 {
        match self {
            Self::Occurrence => 0.0,
            Self::Completeness => 1.0,
            Self::Accuracy => 2.0,
            Self::Cutoff => 3.0,
            Self::Classification => 4.0,
            Self::Existence => 5.0,
            Self::RightsAndObligations => 6.0,
            Self::CompletenessBalance => 7.0,
            Self::ValuationAndAllocation => 8.0,
            Self::OccurrenceAndRightsDisclosure => 9.0,
            Self::CompletenessDisclosure => 10.0,
            Self::AccuracyAndValuation => 11.0,
            Self::ClassificationAndUnderstandability => 12.0,
            Self::Timeliness => 13.0,
            Self::Presentation => 14.0,
        }
    }
}

impl std::fmt::Display for ComplianceAssertion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Occurrence => write!(f, "Occurrence"),
            Self::Completeness => write!(f, "Completeness"),
            Self::Accuracy => write!(f, "Accuracy"),
            Self::Cutoff => write!(f, "Cutoff"),
            Self::Classification => write!(f, "Classification"),
            Self::Existence => write!(f, "Existence"),
            Self::RightsAndObligations => write!(f, "Rights and Obligations"),
            Self::CompletenessBalance => write!(f, "Completeness (Balance)"),
            Self::ValuationAndAllocation => write!(f, "Valuation and Allocation"),
            Self::OccurrenceAndRightsDisclosure => write!(f, "Occurrence and Rights (Disclosure)"),
            Self::CompletenessDisclosure => write!(f, "Completeness (Disclosure)"),
            Self::AccuracyAndValuation => write!(f, "Accuracy and Valuation"),
            Self::ClassificationAndUnderstandability => {
                write!(f, "Classification and Understandability")
            }
            Self::Timeliness => write!(f, "Timeliness"),
            Self::Presentation => write!(f, "Presentation"),
        }
    }
}

/// Category of audit assertion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssertionCategory {
    /// Assertions about classes of transactions and events
    Transaction,
    /// Assertions about account balances at period end
    Balance,
    /// Assertions about presentation and disclosure
    Disclosure,
    /// Assertions about timeliness and presentation
    Presentation,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assertion_categories() {
        assert_eq!(
            ComplianceAssertion::Occurrence.category(),
            AssertionCategory::Transaction
        );
        assert_eq!(
            ComplianceAssertion::Existence.category(),
            AssertionCategory::Balance
        );
        assert_eq!(
            ComplianceAssertion::CompletenessDisclosure.category(),
            AssertionCategory::Disclosure
        );
        assert_eq!(
            ComplianceAssertion::Timeliness.category(),
            AssertionCategory::Presentation
        );
    }

    #[test]
    fn test_feature_codes_unique() {
        let assertions = [
            ComplianceAssertion::Occurrence,
            ComplianceAssertion::Completeness,
            ComplianceAssertion::Accuracy,
            ComplianceAssertion::Cutoff,
            ComplianceAssertion::Classification,
            ComplianceAssertion::Existence,
            ComplianceAssertion::RightsAndObligations,
            ComplianceAssertion::CompletenessBalance,
            ComplianceAssertion::ValuationAndAllocation,
            ComplianceAssertion::OccurrenceAndRightsDisclosure,
            ComplianceAssertion::CompletenessDisclosure,
            ComplianceAssertion::AccuracyAndValuation,
            ComplianceAssertion::ClassificationAndUnderstandability,
            ComplianceAssertion::Timeliness,
            ComplianceAssertion::Presentation,
        ];
        let codes: Vec<f64> = assertions.iter().map(|a| a.feature_code()).collect();
        // Each code should be unique
        for (i, c1) in codes.iter().enumerate() {
            for (j, c2) in codes.iter().enumerate() {
                if i != j {
                    assert!(
                        (c1 - c2).abs() > f64::EPSILON,
                        "Feature codes {i} and {j} are equal"
                    );
                }
            }
        }
    }

    #[test]
    fn test_assertion_display() {
        assert_eq!(format!("{}", ComplianceAssertion::Occurrence), "Occurrence");
        assert_eq!(
            format!("{}", ComplianceAssertion::ValuationAndAllocation),
            "Valuation and Allocation"
        );
    }
}
