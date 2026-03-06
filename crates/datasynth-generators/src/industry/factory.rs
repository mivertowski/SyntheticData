//! Industry-specific generator factory.
//!
//! Dispatches to the appropriate industry transaction generator based on the
//! configured industry sector and returns the GL accounts specific to that
//! industry vertical.

use super::common::IndustryGlAccount;
use datasynth_core::models::IndustrySector;
use serde::{Deserialize, Serialize};

/// Industry generation output.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IndustryOutput {
    /// Industry-specific GL accounts.
    pub gl_accounts: Vec<IndustryGlAccount>,
    /// The industry sector that was used.
    pub industry: String,
}

/// Generate industry-specific data based on the configured industry sector.
///
/// Currently returns GL accounts for retail, manufacturing, and healthcare.
/// Other industry sectors return an empty account list.
pub fn generate_industry_output(sector: IndustrySector) -> IndustryOutput {
    let gl_accounts = match sector {
        IndustrySector::Retail => super::RetailTransactionGenerator::gl_accounts(),
        IndustrySector::Manufacturing => super::ManufacturingTransactionGenerator::gl_accounts(),
        IndustrySector::Healthcare => super::HealthcareTransactionGenerator::gl_accounts(),
        _ => Vec::new(),
    };

    IndustryOutput {
        gl_accounts,
        industry: format!("{sector:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retail_factory() {
        let output = generate_industry_output(IndustrySector::Retail);
        assert_eq!(output.industry, "Retail");
        assert!(!output.gl_accounts.is_empty());
        assert!(output.gl_accounts.len() >= 10);
        // Verify a known retail account exists
        assert!(output
            .gl_accounts
            .iter()
            .any(|a| a.account_number == "1400"));
    }

    #[test]
    fn test_manufacturing_factory() {
        let output = generate_industry_output(IndustrySector::Manufacturing);
        assert_eq!(output.industry, "Manufacturing");
        assert!(!output.gl_accounts.is_empty());
        assert!(output.gl_accounts.len() >= 10);
        // Verify WIP account exists
        assert!(output
            .gl_accounts
            .iter()
            .any(|a| a.account_number == "1400" && a.name.contains("Work in Process")));
    }

    #[test]
    fn test_healthcare_factory() {
        let output = generate_industry_output(IndustrySector::Healthcare);
        assert_eq!(output.industry, "Healthcare");
        assert!(!output.gl_accounts.is_empty());
        assert!(output.gl_accounts.len() >= 10);
        // Verify patient AR account exists
        assert!(output
            .gl_accounts
            .iter()
            .any(|a| a.account_number == "1200"));
    }

    #[test]
    fn test_unsupported_industry_returns_empty() {
        let output = generate_industry_output(IndustrySector::Technology);
        assert_eq!(output.industry, "Technology");
        assert!(output.gl_accounts.is_empty());
    }

    #[test]
    fn test_output_serializable() {
        let output = generate_industry_output(IndustrySector::Retail);
        let json = serde_json::to_string(&output).expect("should serialize");
        assert!(json.contains("Retail"));
        assert!(json.contains("gl_accounts"));
    }
}
