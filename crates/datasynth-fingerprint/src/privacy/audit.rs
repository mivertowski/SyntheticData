//! Privacy audit utilities.

use crate::models::{PrivacyAction, PrivacyAudit, PrivacyWarning, WarningLevel};

/// Builder for creating privacy audits.
pub struct PrivacyAuditBuilder {
    epsilon_budget: f64,
    k_anonymity: u32,
    actions: Vec<PrivacyAction>,
    warnings: Vec<PrivacyWarning>,
}

impl PrivacyAuditBuilder {
    /// Create a new audit builder.
    pub fn new(epsilon_budget: f64, k_anonymity: u32) -> Self {
        Self {
            epsilon_budget,
            k_anonymity,
            actions: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Add an action.
    pub fn add_action(&mut self, action: PrivacyAction) {
        self.actions.push(action);
    }

    /// Add a warning.
    pub fn add_warning(&mut self, warning: PrivacyWarning) {
        self.warnings.push(warning);
    }

    /// Build the audit.
    pub fn build(self) -> PrivacyAudit {
        let mut audit = PrivacyAudit::new(self.epsilon_budget, self.k_anonymity);

        for action in self.actions {
            audit.record_action(action);
        }

        for warning in self.warnings {
            audit.add_warning(warning);
        }

        audit
    }
}

/// Generate a summary report of privacy actions.
pub fn generate_privacy_report(audit: &PrivacyAudit) -> String {
    let mut report = String::new();

    report.push_str("=== Privacy Audit Report ===\n\n");

    report.push_str(&format!("Epsilon Budget: {:.3}\n", audit.epsilon_budget));
    report.push_str(&format!(
        "Epsilon Spent:  {:.3} ({:.1}%)\n",
        audit.total_epsilon_spent,
        audit.total_epsilon_spent / audit.epsilon_budget * 100.0
    ));
    report.push_str(&format!("K-Anonymity:    {}\n", audit.k_anonymity));
    report.push_str(&format!("Total Actions:  {}\n\n", audit.actions.len()));

    report.push_str("Summary:\n");
    report.push_str(&format!(
        "  - Noise additions:   {}\n",
        audit.summary.noise_additions
    ));
    report.push_str(&format!(
        "  - Suppressions:      {}\n",
        audit.summary.suppressions
    ));
    report.push_str(&format!(
        "  - Generalizations:   {}\n",
        audit.summary.generalizations
    ));
    report.push_str(&format!(
        "  - Winsorizations:    {}\n",
        audit.summary.winsorizations
    ));
    report.push_str(&format!(
        "  - Binnings:          {}\n",
        audit.summary.binnings
    ));
    report.push_str(&format!(
        "  - Roundings:         {}\n",
        audit.summary.roundings
    ));

    if !audit.warnings.is_empty() {
        report.push_str(&format!("\nWarnings ({}):\n", audit.warnings.len()));
        for warning in &audit.warnings {
            let level_str = match warning.level {
                WarningLevel::Info => "INFO",
                WarningLevel::Warning => "WARN",
                WarningLevel::Serious => "SERIOUS",
                WarningLevel::Critical => "CRITICAL",
            };
            report.push_str(&format!("  [{}] {}\n", level_str, warning.message));
        }
    }

    report
}

/// Check audit for potential issues.
pub fn check_audit_issues(audit: &PrivacyAudit) -> Vec<String> {
    let mut issues = Vec::new();

    // Check epsilon usage
    if audit.total_epsilon_spent > audit.epsilon_budget {
        issues.push(format!(
            "Epsilon budget exceeded: spent {:.3}, budget {:.3}",
            audit.total_epsilon_spent, audit.epsilon_budget
        ));
    }

    // Check for high suppression rate
    let total_actions = audit.summary.total_actions();
    if total_actions > 0 {
        let suppression_rate = audit.summary.suppressions as f64 / total_actions as f64;
        if suppression_rate > 0.5 {
            issues.push(format!(
                "High suppression rate: {:.1}% of actions were suppressions",
                suppression_rate * 100.0
            ));
        }
    }

    // Check for critical warnings
    let critical_count = audit
        .warnings
        .iter()
        .filter(|w| w.level == WarningLevel::Critical)
        .count();
    if critical_count > 0 {
        issues.push(format!("{critical_count} critical warnings present"));
    }

    issues
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::models::PrivacyActionType;

    #[test]
    fn test_audit_builder() {
        let mut builder = PrivacyAuditBuilder::new(1.0, 5);

        builder.add_action(
            PrivacyAction::new(
                PrivacyActionType::LaplaceNoise,
                "test.column",
                "Added noise",
                "DP protection",
            )
            .with_epsilon(0.1),
        );

        let audit = builder.build();

        assert_eq!(audit.epsilon_budget, 1.0);
        assert_eq!(audit.k_anonymity, 5);
        assert_eq!(audit.actions.len(), 1);
        assert_eq!(audit.total_epsilon_spent, 0.1);
    }
}
