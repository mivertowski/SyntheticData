//! Company code and organizational structures.
//!
//! Defines the company code entity which represents a legal entity
//! or organizational unit within an enterprise group.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::models::intercompany::ConsolidationMethod;

/// Fiscal year variant defining the fiscal calendar.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FiscalYearVariant {
    /// Variant code (e.g., "K4" for calendar year)
    pub code: String,
    /// Description
    pub description: String,
    /// Number of posting periods (typically 12)
    pub periods: u8,
    /// Number of special periods (typically 4)
    pub special_periods: u8,
    /// First month of fiscal year (1-12)
    pub first_month: u8,
}

impl FiscalYearVariant {
    /// Calendar year fiscal variant (Jan-Dec).
    pub fn calendar_year() -> Self {
        Self {
            code: "K4".to_string(),
            description: "Calendar Year".to_string(),
            periods: 12,
            special_periods: 4,
            first_month: 1,
        }
    }

    /// US federal fiscal year (Oct-Sep).
    pub fn us_federal() -> Self {
        Self {
            code: "V3".to_string(),
            description: "US Federal Fiscal Year".to_string(),
            periods: 12,
            special_periods: 4,
            first_month: 10,
        }
    }

    /// April fiscal year (Apr-Mar).
    pub fn april_year() -> Self {
        Self {
            code: "K1".to_string(),
            description: "April Fiscal Year".to_string(),
            periods: 12,
            special_periods: 4,
            first_month: 4,
        }
    }
}

impl Default for FiscalYearVariant {
    fn default() -> Self {
        Self::calendar_year()
    }
}

/// Company code representing a legal entity or organizational unit.
///
/// In SAP terminology, a company code is the smallest organizational unit
/// for which a complete self-contained set of accounts can be drawn up
/// for external reporting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyCode {
    /// Company code identifier (typically 4 characters)
    pub code: String,

    /// Company name
    pub name: String,

    /// Legal name (for official documents)
    pub legal_name: String,

    /// Local/functional currency (ISO 4217)
    pub currency: String,

    /// Country code (ISO 3166-1 alpha-2)
    pub country: String,

    /// City
    pub city: Option<String>,

    /// Fiscal year variant
    pub fiscal_year_variant: FiscalYearVariant,

    /// Chart of accounts ID used by this company
    pub coa_id: String,

    /// Parent company code (for group structure)
    pub parent_company: Option<String>,

    /// Is this the group parent/consolidation entity
    pub is_group_parent: bool,

    /// Controlling area for cost accounting
    pub controlling_area: Option<String>,

    /// Credit control area
    pub credit_control_area: Option<String>,

    /// Time zone
    pub time_zone: String,

    /// VAT registration number
    pub vat_number: Option<String>,

    /// Tax jurisdiction
    pub tax_jurisdiction: Option<String>,
}

impl CompanyCode {
    /// Create a new company code with required fields.
    pub fn new(code: String, name: String, currency: String, country: String) -> Self {
        Self {
            code: code.clone(),
            name: name.clone(),
            legal_name: name,
            currency,
            country,
            city: None,
            fiscal_year_variant: FiscalYearVariant::default(),
            coa_id: "OPER".to_string(),
            parent_company: None,
            is_group_parent: false,
            controlling_area: Some(code.clone()),
            credit_control_area: Some(code),
            time_zone: "UTC".to_string(),
            vat_number: None,
            tax_jurisdiction: None,
        }
    }

    /// Create a US company code.
    pub fn us(code: &str, name: &str) -> Self {
        Self::new(
            code.to_string(),
            name.to_string(),
            "USD".to_string(),
            "US".to_string(),
        )
    }

    /// Create a German company code.
    pub fn de(code: &str, name: &str) -> Self {
        Self::new(
            code.to_string(),
            name.to_string(),
            "EUR".to_string(),
            "DE".to_string(),
        )
    }

    /// Create a UK company code.
    pub fn gb(code: &str, name: &str) -> Self {
        Self::new(
            code.to_string(),
            name.to_string(),
            "GBP".to_string(),
            "GB".to_string(),
        )
    }

    /// Create a Swiss company code.
    pub fn ch(code: &str, name: &str) -> Self {
        Self::new(
            code.to_string(),
            name.to_string(),
            "CHF".to_string(),
            "CH".to_string(),
        )
    }
}

/// Enterprise group structure containing multiple company codes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseGroup {
    /// Group identifier
    pub group_id: String,

    /// Group name
    pub name: String,

    /// Group consolidation currency
    pub group_currency: String,

    /// Company codes in the group
    pub companies: Vec<CompanyCode>,

    /// Intercompany relationships (from_code, to_code)
    pub intercompany_links: Vec<(String, String)>,
}

impl EnterpriseGroup {
    /// Create a new enterprise group.
    pub fn new(group_id: String, name: String, group_currency: String) -> Self {
        Self {
            group_id,
            name,
            group_currency,
            companies: Vec::new(),
            intercompany_links: Vec::new(),
        }
    }

    /// Add a company to the group.
    pub fn add_company(&mut self, company: CompanyCode) {
        self.companies.push(company);
    }

    /// Add an intercompany link.
    pub fn add_intercompany_link(&mut self, from_code: String, to_code: String) {
        self.intercompany_links.push((from_code, to_code));
    }

    /// Get company by code.
    pub fn get_company(&self, code: &str) -> Option<&CompanyCode> {
        self.companies.iter().find(|c| c.code == code)
    }

    /// Get all company codes.
    pub fn company_codes(&self) -> Vec<&str> {
        self.companies.iter().map(|c| c.code.as_str()).collect()
    }

    /// Get intercompany partners for a company.
    pub fn get_intercompany_partners(&self, code: &str) -> Vec<&str> {
        self.intercompany_links
            .iter()
            .filter_map(|(from, to)| {
                if from == code {
                    Some(to.as_str())
                } else if to == code {
                    Some(from.as_str())
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Company entity for graph building and consolidation.
///
/// This is a simplified view of a company entity with ownership and
/// consolidation attributes for building entity relationship graphs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    /// Company code identifier
    pub company_code: String,

    /// Company name
    pub company_name: String,

    /// Country code (ISO 3166-1 alpha-2)
    pub country: String,

    /// Local/functional currency (ISO 4217)
    pub local_currency: String,

    /// Functional currency for translation
    pub functional_currency: String,

    /// Is this the parent/holding company
    pub is_parent: bool,

    /// Parent company code (if subsidiary)
    pub parent_company: Option<String>,

    /// Ownership percentage (0-100)
    pub ownership_percentage: Option<Decimal>,

    /// Consolidation method
    pub consolidation_method: ConsolidationMethod,
}

impl Company {
    /// Create a new company entity.
    pub fn new(
        company_code: impl Into<String>,
        company_name: impl Into<String>,
        country: impl Into<String>,
        local_currency: impl Into<String>,
    ) -> Self {
        let currency = local_currency.into();
        Self {
            company_code: company_code.into(),
            company_name: company_name.into(),
            country: country.into(),
            local_currency: currency.clone(),
            functional_currency: currency,
            is_parent: false,
            parent_company: None,
            ownership_percentage: None,
            consolidation_method: ConsolidationMethod::Full,
        }
    }

    /// Create a parent company.
    pub fn parent(
        company_code: impl Into<String>,
        company_name: impl Into<String>,
        country: impl Into<String>,
        currency: impl Into<String>,
    ) -> Self {
        let mut company = Self::new(company_code, company_name, country, currency);
        company.is_parent = true;
        company
    }

    /// Create a subsidiary company.
    pub fn subsidiary(
        company_code: impl Into<String>,
        company_name: impl Into<String>,
        country: impl Into<String>,
        currency: impl Into<String>,
        parent_code: impl Into<String>,
        ownership_pct: Decimal,
    ) -> Self {
        let mut company = Self::new(company_code, company_name, country, currency);
        company.parent_company = Some(parent_code.into());
        company.ownership_percentage = Some(ownership_pct);
        company
    }
}

impl From<&CompanyCode> for Company {
    fn from(cc: &CompanyCode) -> Self {
        Self {
            company_code: cc.code.clone(),
            company_name: cc.name.clone(),
            country: cc.country.clone(),
            local_currency: cc.currency.clone(),
            functional_currency: cc.currency.clone(),
            is_parent: cc.is_group_parent,
            parent_company: cc.parent_company.clone(),
            ownership_percentage: None,
            consolidation_method: ConsolidationMethod::Full,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_company_creation() {
        let company = CompanyCode::us("1000", "US Operations");
        assert_eq!(company.code, "1000");
        assert_eq!(company.currency, "USD");
        assert_eq!(company.country, "US");
    }

    #[test]
    fn test_enterprise_group() {
        let mut group = EnterpriseGroup::new(
            "CORP".to_string(),
            "Global Corp".to_string(),
            "USD".to_string(),
        );

        group.add_company(CompanyCode::us("1000", "US HQ"));
        group.add_company(CompanyCode::de("2000", "EU Operations"));
        group.add_intercompany_link("1000".to_string(), "2000".to_string());

        assert_eq!(group.companies.len(), 2);
        assert_eq!(group.get_intercompany_partners("1000"), vec!["2000"]);
    }
}
