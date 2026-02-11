//! Privacy audit trail models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Privacy audit trail documenting all privacy decisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyAudit {
    /// Total epsilon budget allowed.
    pub epsilon_budget: f64,

    /// Total epsilon spent.
    pub total_epsilon_spent: f64,

    /// K-anonymity threshold used.
    pub k_anonymity: u32,

    /// All privacy actions taken.
    pub actions: Vec<PrivacyAction>,

    /// Summary of privacy measures by category.
    pub summary: PrivacySummary,

    /// Timestamp of audit creation.
    pub created_at: DateTime<Utc>,

    /// Warnings generated during privacy processing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<PrivacyWarning>,

    /// The composition method used for budget accounting (e.g., "naive", "renyi_dp", "zcdp").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub composition_method: Option<String>,

    /// The optimal Renyi DP alpha order (only set when using RDP composition).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rdp_alpha_effective: Option<f64>,
}

impl PrivacyAudit {
    /// Create a new privacy audit.
    pub fn new(epsilon_budget: f64, k_anonymity: u32) -> Self {
        Self {
            epsilon_budget,
            total_epsilon_spent: 0.0,
            k_anonymity,
            actions: Vec::new(),
            summary: PrivacySummary::default(),
            created_at: Utc::now(),
            warnings: Vec::new(),
            composition_method: None,
            rdp_alpha_effective: None,
        }
    }

    /// Record a privacy action.
    pub fn record_action(&mut self, action: PrivacyAction) {
        if let Some(epsilon) = action.epsilon_spent {
            self.total_epsilon_spent += epsilon;
        }

        // Update summary
        match action.action_type {
            PrivacyActionType::LaplaceNoise => self.summary.noise_additions += 1,
            PrivacyActionType::Suppression => self.summary.suppressions += 1,
            PrivacyActionType::Generalization => self.summary.generalizations += 1,
            PrivacyActionType::Winsorization => self.summary.winsorizations += 1,
            PrivacyActionType::Binning => self.summary.binnings += 1,
            PrivacyActionType::Rounding => self.summary.roundings += 1,
        }

        self.actions.push(action);
    }

    /// Check if privacy budget is exhausted.
    pub fn is_budget_exhausted(&self) -> bool {
        self.total_epsilon_spent >= self.epsilon_budget
    }

    /// Get remaining epsilon budget.
    pub fn remaining_budget(&self) -> f64 {
        (self.epsilon_budget - self.total_epsilon_spent).max(0.0)
    }

    /// Add a warning.
    pub fn add_warning(&mut self, warning: PrivacyWarning) {
        self.warnings.push(warning);
    }
}

/// A single privacy action taken during extraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyAction {
    /// Type of privacy action.
    pub action_type: PrivacyActionType,

    /// Target of the action (table.column or statistic name).
    pub target: String,

    /// Description of what was done.
    pub description: String,

    /// Epsilon spent for this action (if DP).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epsilon_spent: Option<f64>,

    /// Original value (for auditing, may be redacted).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_value: Option<String>,

    /// Resulting value after action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resulting_value: Option<String>,

    /// Reason for the action.
    pub reason: String,

    /// Timestamp of the action.
    pub timestamp: DateTime<Utc>,
}

impl PrivacyAction {
    /// Create a new privacy action.
    pub fn new(
        action_type: PrivacyActionType,
        target: impl Into<String>,
        description: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            action_type,
            target: target.into(),
            description: description.into(),
            epsilon_spent: None,
            original_value: None,
            resulting_value: None,
            reason: reason.into(),
            timestamp: Utc::now(),
        }
    }

    /// Add epsilon spent.
    pub fn with_epsilon(mut self, epsilon: f64) -> Self {
        self.epsilon_spent = Some(epsilon);
        self
    }

    /// Add original and resulting values.
    pub fn with_values(
        mut self,
        original: impl Into<String>,
        resulting: impl Into<String>,
    ) -> Self {
        self.original_value = Some(original.into());
        self.resulting_value = Some(resulting.into());
        self
    }
}

/// Types of privacy actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyActionType {
    /// Added Laplace noise (differential privacy).
    LaplaceNoise,
    /// Suppressed value (k-anonymity).
    Suppression,
    /// Generalized value (e.g., exact age -> age range).
    Generalization,
    /// Winsorized outliers.
    Winsorization,
    /// Binned continuous values.
    Binning,
    /// Rounded value.
    Rounding,
}

impl std::fmt::Display for PrivacyActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LaplaceNoise => write!(f, "Laplace Noise"),
            Self::Suppression => write!(f, "Suppression"),
            Self::Generalization => write!(f, "Generalization"),
            Self::Winsorization => write!(f, "Winsorization"),
            Self::Binning => write!(f, "Binning"),
            Self::Rounding => write!(f, "Rounding"),
        }
    }
}

/// Summary of privacy measures applied.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrivacySummary {
    /// Number of noise additions.
    pub noise_additions: u64,

    /// Number of value suppressions.
    pub suppressions: u64,

    /// Number of generalizations.
    pub generalizations: u64,

    /// Number of winsorizations.
    pub winsorizations: u64,

    /// Number of binnings.
    pub binnings: u64,

    /// Number of roundings.
    pub roundings: u64,

    /// Total columns processed.
    pub columns_processed: u64,

    /// Columns with privacy modifications.
    pub columns_modified: u64,

    /// Categorical values suppressed.
    pub categorical_values_suppressed: u64,

    /// Rows potentially affected.
    pub rows_affected: u64,
}

impl PrivacySummary {
    /// Get total privacy actions.
    pub fn total_actions(&self) -> u64 {
        self.noise_additions
            + self.suppressions
            + self.generalizations
            + self.winsorizations
            + self.binnings
            + self.roundings
    }
}

/// Privacy warning generated during processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyWarning {
    /// Warning level.
    pub level: WarningLevel,

    /// Warning message.
    pub message: String,

    /// Target of the warning.
    pub target: Option<String>,

    /// Recommendation.
    pub recommendation: Option<String>,

    /// Timestamp.
    pub timestamp: DateTime<Utc>,
}

impl PrivacyWarning {
    /// Create a new warning.
    pub fn new(level: WarningLevel, message: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
            target: None,
            recommendation: None,
            timestamp: Utc::now(),
        }
    }

    /// Add target.
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    /// Add recommendation.
    pub fn with_recommendation(mut self, recommendation: impl Into<String>) -> Self {
        self.recommendation = Some(recommendation.into());
        self
    }
}

/// Warning level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarningLevel {
    /// Informational warning.
    Info,
    /// Warning that may affect utility.
    Warning,
    /// Serious warning that may affect privacy or utility significantly.
    Serious,
    /// Critical warning - action may be needed.
    Critical,
}
