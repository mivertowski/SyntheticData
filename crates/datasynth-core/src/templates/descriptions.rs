//! Description templates for journal entry header and line text.
//!
//! Provides realistic, business-process-specific text templates
//! for populating header_text and line_text fields.

use crate::models::BusinessProcess;
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Pattern for header text generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderTextPattern {
    /// The pattern template with placeholders
    pub template: String,
    /// Business process this pattern applies to
    pub business_process: BusinessProcess,
}

/// Pattern for line text generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineTextPattern {
    /// The pattern template
    pub template: String,
    /// Account type prefix this applies to (e.g., "1" for assets, "5" for expenses)
    pub account_prefix: Option<String>,
}

/// Context for text generation with available placeholders.
#[derive(Debug, Clone, Default)]
pub struct DescriptionContext {
    /// Vendor name for P2P transactions
    pub vendor_name: Option<String>,
    /// Customer name for O2C transactions
    pub customer_name: Option<String>,
    /// Invoice number reference
    pub invoice_number: Option<String>,
    /// PO number reference
    pub po_number: Option<String>,
    /// Month name (e.g., "January")
    pub month_name: Option<String>,
    /// Year (e.g., "2024")
    pub year: Option<String>,
    /// Quarter (e.g., "Q1")
    pub quarter: Option<String>,
    /// Asset description
    pub asset_description: Option<String>,
    /// Project name
    pub project_name: Option<String>,
    /// Department name
    pub department_name: Option<String>,
    /// Employee name for H2R
    pub employee_name: Option<String>,
    /// Amount for reference
    pub amount: Option<String>,
}

impl DescriptionContext {
    /// Create a context with month and year.
    pub fn with_period(month: u32, year: i32) -> Self {
        let month_name = match month {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "Unknown",
        };

        let quarter = match month {
            1..=3 => "Q1",
            4..=6 => "Q2",
            7..=9 => "Q3",
            10..=12 => "Q4",
            _ => "Q1",
        };

        Self {
            month_name: Some(month_name.to_string()),
            year: Some(year.to_string()),
            quarter: Some(quarter.to_string()),
            ..Default::default()
        }
    }
}

/// Generator for journal entry descriptions.
#[derive(Debug, Clone)]
pub struct DescriptionGenerator {
    /// Header text patterns by business process
    header_patterns: Vec<HeaderTextPattern>,
    /// Line text patterns (reserved for future use)
    #[allow(dead_code)]
    line_patterns: Vec<LineTextPattern>,
    /// Expense descriptions
    expense_descriptions: Vec<&'static str>,
    /// Revenue descriptions
    revenue_descriptions: Vec<&'static str>,
    /// Asset descriptions
    asset_descriptions: Vec<&'static str>,
    /// Liability descriptions
    liability_descriptions: Vec<&'static str>,
    /// Bank/cash descriptions
    bank_descriptions: Vec<&'static str>,
    /// Process-specific line descriptions for P2P
    p2p_line_descriptions: Vec<&'static str>,
    /// Process-specific line descriptions for O2C
    o2c_line_descriptions: Vec<&'static str>,
    /// Process-specific line descriptions for H2R (Hire to Retire)
    h2r_line_descriptions: Vec<&'static str>,
    /// Process-specific line descriptions for R2R (Record to Report)
    r2r_line_descriptions: Vec<&'static str>,
}

impl Default for DescriptionGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl DescriptionGenerator {
    /// Create a new description generator with default templates.
    pub fn new() -> Self {
        Self {
            header_patterns: Self::default_header_patterns(),
            line_patterns: Self::default_line_patterns(),
            expense_descriptions: Self::default_expense_descriptions(),
            revenue_descriptions: Self::default_revenue_descriptions(),
            asset_descriptions: Self::default_asset_descriptions(),
            liability_descriptions: Self::default_liability_descriptions(),
            bank_descriptions: Self::default_bank_descriptions(),
            p2p_line_descriptions: Self::default_p2p_line_descriptions(),
            o2c_line_descriptions: Self::default_o2c_line_descriptions(),
            h2r_line_descriptions: Self::default_h2r_line_descriptions(),
            r2r_line_descriptions: Self::default_r2r_line_descriptions(),
        }
    }

    fn default_p2p_line_descriptions() -> Vec<&'static str> {
        vec![
            "Inventory purchase",
            "Raw materials receipt",
            "Goods received - standard",
            "Vendor invoice posting",
            "AP invoice match",
            "Purchase goods receipt",
            "Material receipt",
            "Components inventory",
            "Supplies procurement",
            "Service receipt",
            "Purchase payment",
            "Vendor payment",
            "AP settlement",
            "GR/IR clearing",
            "Price variance adjustment",
            "Quantity variance",
            "Procurement expense",
            "Freight charges",
            "Customs duties",
            "Import taxes",
        ]
    }

    fn default_o2c_line_descriptions() -> Vec<&'static str> {
        vec![
            "Product sales",
            "Service delivery",
            "Customer invoice",
            "Revenue recognition",
            "Sales order fulfillment",
            "Goods shipped",
            "Delivery completion",
            "Customer receipt",
            "AR receipt",
            "Cash application",
            "Sales discount given",
            "Trade discount",
            "Early payment discount",
            "Finished goods sale",
            "Merchandise sale",
            "Contract revenue",
            "Subscription revenue",
            "License fee revenue",
            "Commission earned",
            "COGS recognition",
        ]
    }

    fn default_h2r_line_descriptions() -> Vec<&'static str> {
        vec![
            "Salary expense",
            "Wages allocation",
            "Benefits expense",
            "Payroll taxes",
            "Commission payment",
            "Bonus accrual",
            "Vacation accrual",
            "Health insurance",
            "Retirement contribution",
            "Training expense",
            "Recruitment costs",
            "Relocation expense",
            "Employee reimbursement",
            "Contractor payment",
            "Temporary staff",
            "Overtime payment",
            "Shift differential",
            "On-call allowance",
            "Travel reimbursement",
            "Expense report",
        ]
    }

    fn default_r2r_line_descriptions() -> Vec<&'static str> {
        vec![
            "Period close adjustment",
            "Depreciation expense",
            "Amortization expense",
            "Accrual entry",
            "Accrual reversal",
            "Reclassification entry",
            "Intercompany elimination",
            "Currency translation",
            "FX revaluation",
            "Reserve adjustment",
            "Provision update",
            "Impairment charge",
            "Bad debt provision",
            "Inventory adjustment",
            "Valuation adjustment",
            "Consolidation entry",
            "Manual adjustment",
            "Year-end closing",
            "Opening balance",
            "Trial balance adjustment",
        ]
    }

    fn default_header_patterns() -> Vec<HeaderTextPattern> {
        vec![
            // O2C - Order to Cash
            HeaderTextPattern {
                template: "Customer Invoice - {CustomerName}".to_string(),
                business_process: BusinessProcess::O2C,
            },
            HeaderTextPattern {
                template: "Sales Order Fulfillment - {CustomerName}".to_string(),
                business_process: BusinessProcess::O2C,
            },
            HeaderTextPattern {
                template: "Revenue Recognition - {Month} {Year}".to_string(),
                business_process: BusinessProcess::O2C,
            },
            HeaderTextPattern {
                template: "Customer Payment Receipt - {CustomerName}".to_string(),
                business_process: BusinessProcess::O2C,
            },
            HeaderTextPattern {
                template: "AR Collection - {InvoiceNumber}".to_string(),
                business_process: BusinessProcess::O2C,
            },
            HeaderTextPattern {
                template: "Credit Memo - {CustomerName}".to_string(),
                business_process: BusinessProcess::O2C,
            },
            HeaderTextPattern {
                template: "Deferred Revenue Release - {Month}".to_string(),
                business_process: BusinessProcess::O2C,
            },
            HeaderTextPattern {
                template: "Sales Commission Accrual - {Quarter}".to_string(),
                business_process: BusinessProcess::O2C,
            },
            // P2P - Procure to Pay
            HeaderTextPattern {
                template: "Vendor Invoice - {VendorName}".to_string(),
                business_process: BusinessProcess::P2P,
            },
            HeaderTextPattern {
                template: "Purchase Order - {PONumber}".to_string(),
                business_process: BusinessProcess::P2P,
            },
            HeaderTextPattern {
                template: "AP Payment Run - {Month} {Year}".to_string(),
                business_process: BusinessProcess::P2P,
            },
            HeaderTextPattern {
                template: "Vendor Payment - {VendorName}".to_string(),
                business_process: BusinessProcess::P2P,
            },
            HeaderTextPattern {
                template: "Expense Accrual - {Month} {Year}".to_string(),
                business_process: BusinessProcess::P2P,
            },
            HeaderTextPattern {
                template: "Travel Expense Report - {EmployeeName}".to_string(),
                business_process: BusinessProcess::P2P,
            },
            HeaderTextPattern {
                template: "Utility Bill - {VendorName}".to_string(),
                business_process: BusinessProcess::P2P,
            },
            HeaderTextPattern {
                template: "Goods Receipt - {PONumber}".to_string(),
                business_process: BusinessProcess::P2P,
            },
            // R2R - Record to Report
            HeaderTextPattern {
                template: "Month End Close - {Month} {Year}".to_string(),
                business_process: BusinessProcess::R2R,
            },
            HeaderTextPattern {
                template: "Depreciation - {Month} {Year}".to_string(),
                business_process: BusinessProcess::R2R,
            },
            HeaderTextPattern {
                template: "Amortization - {Month} {Year}".to_string(),
                business_process: BusinessProcess::R2R,
            },
            HeaderTextPattern {
                template: "Accrual Reversal - {Month}".to_string(),
                business_process: BusinessProcess::R2R,
            },
            HeaderTextPattern {
                template: "Prepaid Expense Release - {Month}".to_string(),
                business_process: BusinessProcess::R2R,
            },
            HeaderTextPattern {
                template: "FX Revaluation - {Month} {Year}".to_string(),
                business_process: BusinessProcess::R2R,
            },
            HeaderTextPattern {
                template: "Bank Reconciliation Adjustment".to_string(),
                business_process: BusinessProcess::R2R,
            },
            HeaderTextPattern {
                template: "Manual Journal Entry - {Department}".to_string(),
                business_process: BusinessProcess::R2R,
            },
            HeaderTextPattern {
                template: "Intercompany Allocation - {Month}".to_string(),
                business_process: BusinessProcess::R2R,
            },
            HeaderTextPattern {
                template: "Cost Allocation - {Quarter} {Year}".to_string(),
                business_process: BusinessProcess::R2R,
            },
            // H2R - Hire to Retire
            HeaderTextPattern {
                template: "Payroll - {Month} {Year}".to_string(),
                business_process: BusinessProcess::H2R,
            },
            HeaderTextPattern {
                template: "Benefits Accrual - {Month}".to_string(),
                business_process: BusinessProcess::H2R,
            },
            HeaderTextPattern {
                template: "Bonus Accrual - {Quarter} {Year}".to_string(),
                business_process: BusinessProcess::H2R,
            },
            HeaderTextPattern {
                template: "Pension Contribution - {Month}".to_string(),
                business_process: BusinessProcess::H2R,
            },
            HeaderTextPattern {
                template: "Stock Compensation - {Month} {Year}".to_string(),
                business_process: BusinessProcess::H2R,
            },
            HeaderTextPattern {
                template: "Payroll Tax Remittance - {Month}".to_string(),
                business_process: BusinessProcess::H2R,
            },
            HeaderTextPattern {
                template: "401k Contribution - {Month} {Year}".to_string(),
                business_process: BusinessProcess::H2R,
            },
            // A2R - Acquire to Retire
            HeaderTextPattern {
                template: "Asset Acquisition - {AssetDescription}".to_string(),
                business_process: BusinessProcess::A2R,
            },
            HeaderTextPattern {
                template: "Capital Project - {ProjectName}".to_string(),
                business_process: BusinessProcess::A2R,
            },
            HeaderTextPattern {
                template: "Asset Disposal - {AssetDescription}".to_string(),
                business_process: BusinessProcess::A2R,
            },
            HeaderTextPattern {
                template: "Asset Transfer - {AssetDescription}".to_string(),
                business_process: BusinessProcess::A2R,
            },
            HeaderTextPattern {
                template: "CIP Settlement - {ProjectName}".to_string(),
                business_process: BusinessProcess::A2R,
            },
            HeaderTextPattern {
                template: "Impairment Write-down - {Quarter} {Year}".to_string(),
                business_process: BusinessProcess::A2R,
            },
            // Treasury
            HeaderTextPattern {
                template: "Bank Transfer - {Month} {Year}".to_string(),
                business_process: BusinessProcess::Treasury,
            },
            HeaderTextPattern {
                template: "Cash Pooling - {Month}".to_string(),
                business_process: BusinessProcess::Treasury,
            },
            HeaderTextPattern {
                template: "Investment Transaction".to_string(),
                business_process: BusinessProcess::Treasury,
            },
            HeaderTextPattern {
                template: "Loan Interest Payment - {Month}".to_string(),
                business_process: BusinessProcess::Treasury,
            },
            // Tax
            HeaderTextPattern {
                template: "Tax Provision - {Quarter} {Year}".to_string(),
                business_process: BusinessProcess::Tax,
            },
            HeaderTextPattern {
                template: "VAT/GST Remittance - {Month}".to_string(),
                business_process: BusinessProcess::Tax,
            },
            HeaderTextPattern {
                template: "Withholding Tax - {Month} {Year}".to_string(),
                business_process: BusinessProcess::Tax,
            },
            // Intercompany
            HeaderTextPattern {
                template: "IC Service Charge - {Month} {Year}".to_string(),
                business_process: BusinessProcess::Intercompany,
            },
            HeaderTextPattern {
                template: "IC Management Fee - {Quarter}".to_string(),
                business_process: BusinessProcess::Intercompany,
            },
            HeaderTextPattern {
                template: "IC Goods Transfer".to_string(),
                business_process: BusinessProcess::Intercompany,
            },
        ]
    }

    fn default_line_patterns() -> Vec<LineTextPattern> {
        vec![
            // Generic patterns
            LineTextPattern {
                template: "See header".to_string(),
                account_prefix: None,
            },
            LineTextPattern {
                template: "Per attached documentation".to_string(),
                account_prefix: None,
            },
        ]
    }

    fn default_expense_descriptions() -> Vec<&'static str> {
        vec![
            "Office supplies and materials",
            "Software subscription - monthly",
            "Professional services fee",
            "Travel expense - airfare",
            "Travel expense - hotel",
            "Travel expense - meals",
            "Conference registration fee",
            "Equipment maintenance",
            "Telecommunication services",
            "Internet and data services",
            "Insurance premium",
            "Legal services",
            "Consulting services",
            "Marketing materials",
            "Advertising expense",
            "Training and development",
            "Membership and subscriptions",
            "Postage and shipping",
            "Utilities expense",
            "Rent expense - monthly",
            "Cleaning services",
            "Security services",
            "Repair and maintenance",
            "Vehicle expense",
            "Fuel expense",
            "Bank charges",
            "Credit card processing fees",
            "Recruitment expense",
            "Employee benefits",
            "Medical insurance contribution",
            "Office refreshments",
            "Team building event",
            "Client entertainment",
            "Research materials",
            "Cloud computing services",
            "Data storage services",
            "Audit fees",
            "Tax preparation services",
            "License and permits",
            "Bad debt expense",
        ]
    }

    fn default_revenue_descriptions() -> Vec<&'static str> {
        vec![
            "Product sales revenue",
            "Service revenue",
            "Consulting revenue",
            "Subscription revenue - monthly",
            "License fee revenue",
            "Maintenance contract revenue",
            "Support services revenue",
            "Training revenue",
            "Commission income",
            "Referral fee income",
            "Rental income",
            "Interest income",
            "Dividend income",
            "Royalty income",
            "Grant revenue",
            "Milestone payment",
            "Setup fee revenue",
            "Implementation revenue",
            "Project completion payment",
            "Retainer fee",
        ]
    }

    fn default_asset_descriptions() -> Vec<&'static str> {
        vec![
            "Cash receipt",
            "Bank deposit",
            "AR collection",
            "Prepaid expense",
            "Security deposit",
            "Inventory receipt",
            "Fixed asset addition",
            "Computer equipment",
            "Office furniture",
            "Leasehold improvement",
            "Software license",
            "Patent acquisition",
            "Investment purchase",
            "Loan receivable",
            "Intercompany receivable",
            "Other current asset",
            "Deferred tax asset",
            "Work in progress",
            "Raw materials",
            "Finished goods",
        ]
    }

    fn default_liability_descriptions() -> Vec<&'static str> {
        vec![
            "AP - vendor invoice",
            "Accrued expense",
            "Accrued payroll",
            "Accrued bonus",
            "Deferred revenue",
            "Customer deposit",
            "Sales tax payable",
            "VAT payable",
            "Income tax payable",
            "Withholding tax",
            "Pension liability",
            "Lease liability",
            "Loan payable - current",
            "Loan payable - long term",
            "Intercompany payable",
            "Accrued interest",
            "Warranty reserve",
            "Legal reserve",
            "Other accrued liability",
            "Gift card liability",
        ]
    }

    fn default_bank_descriptions() -> Vec<&'static str> {
        vec![
            "Wire transfer",
            "ACH payment",
            "Check deposit",
            "Cash withdrawal",
            "Bank fee",
            "Interest earned",
            "Transfer between accounts",
            "Direct deposit",
            "ATM withdrawal",
            "Credit card payment",
        ]
    }

    /// Generate header text for a business process.
    pub fn generate_header_text(
        &self,
        process: BusinessProcess,
        context: &DescriptionContext,
        rng: &mut impl Rng,
    ) -> String {
        // Filter patterns for this business process
        let matching: Vec<_> = self
            .header_patterns
            .iter()
            .filter(|p| p.business_process == process)
            .collect();

        if matching.is_empty() {
            return format!("{:?} Transaction", process);
        }

        let pattern = matching.choose(rng).expect("non-empty collection");
        self.substitute_placeholders(&pattern.template, context, rng)
    }

    /// Generate line text based on account type.
    pub fn generate_line_text(
        &self,
        gl_account: &str,
        context: &DescriptionContext,
        rng: &mut impl Rng,
    ) -> String {
        // Determine account type from first digit
        let first_char = gl_account.chars().next().unwrap_or('0');

        match first_char {
            '1' => {
                // Assets
                self.asset_descriptions
                    .choose(rng)
                    .unwrap_or(&"Asset posting")
                    .to_string()
            }
            '2' => {
                // Liabilities
                self.liability_descriptions
                    .choose(rng)
                    .unwrap_or(&"Liability posting")
                    .to_string()
            }
            '3' => {
                // Equity - use generic
                "Equity adjustment".to_string()
            }
            '4' => {
                // Revenue
                self.revenue_descriptions
                    .choose(rng)
                    .unwrap_or(&"Revenue posting")
                    .to_string()
            }
            '5' | '6' | '7' => {
                // Expenses
                self.expense_descriptions
                    .choose(rng)
                    .unwrap_or(&"Expense posting")
                    .to_string()
            }
            '8' | '9' => {
                // Statistical / Other
                "Statistical posting".to_string()
            }
            '0' => {
                // Cash/Bank
                self.bank_descriptions
                    .choose(rng)
                    .unwrap_or(&"Bank transaction")
                    .to_string()
            }
            _ => self.substitute_placeholders("Transaction posting", context, rng),
        }
    }

    /// Generate line text based on business process and account type.
    ///
    /// This method provides semantically appropriate line descriptions that match
    /// the business process context. If a business process is provided, it uses
    /// process-specific descriptions; otherwise falls back to account-type-based
    /// descriptions.
    pub fn generate_line_text_for_process(
        &self,
        gl_account: &str,
        business_process: Option<BusinessProcess>,
        _context: &DescriptionContext,
        rng: &mut impl Rng,
    ) -> String {
        // If business process is specified, use process-specific descriptions
        if let Some(process) = business_process {
            let pool = match process {
                BusinessProcess::P2P => &self.p2p_line_descriptions,
                BusinessProcess::O2C => &self.o2c_line_descriptions,
                BusinessProcess::H2R => &self.h2r_line_descriptions,
                BusinessProcess::R2R => &self.r2r_line_descriptions,
                _ => {
                    // For other processes, fall back to account-type-based
                    return self.generate_line_text_by_account(gl_account, rng);
                }
            };

            if let Some(desc) = pool.choose(rng) {
                return (*desc).to_string();
            }
        }

        // Fall back to account-type-based descriptions
        self.generate_line_text_by_account(gl_account, rng)
    }

    /// Generate line text based solely on account type.
    fn generate_line_text_by_account(&self, gl_account: &str, rng: &mut impl Rng) -> String {
        let first_char = gl_account.chars().next().unwrap_or('0');

        match first_char {
            '1' => self
                .asset_descriptions
                .choose(rng)
                .unwrap_or(&"Asset posting")
                .to_string(),
            '2' => self
                .liability_descriptions
                .choose(rng)
                .unwrap_or(&"Liability posting")
                .to_string(),
            '3' => "Equity adjustment".to_string(),
            '4' => self
                .revenue_descriptions
                .choose(rng)
                .unwrap_or(&"Revenue posting")
                .to_string(),
            '5' | '6' | '7' => self
                .expense_descriptions
                .choose(rng)
                .unwrap_or(&"Expense posting")
                .to_string(),
            '8' | '9' => "Statistical posting".to_string(),
            '0' => self
                .bank_descriptions
                .choose(rng)
                .unwrap_or(&"Bank transaction")
                .to_string(),
            _ => "Transaction posting".to_string(),
        }
    }

    /// Substitute placeholders in a template string.
    fn substitute_placeholders(
        &self,
        template: &str,
        context: &DescriptionContext,
        rng: &mut impl Rng,
    ) -> String {
        let mut result = template.to_string();

        // Substitute all placeholders with context values or defaults
        if let Some(ref val) = context.vendor_name {
            result = result.replace("{VendorName}", val);
        } else {
            result = result.replace("{VendorName}", &self.generate_vendor_name(rng));
        }

        if let Some(ref val) = context.customer_name {
            result = result.replace("{CustomerName}", val);
        } else {
            result = result.replace("{CustomerName}", &self.generate_customer_name(rng));
        }

        if let Some(ref val) = context.invoice_number {
            result = result.replace("{InvoiceNumber}", val);
        } else {
            result = result.replace(
                "{InvoiceNumber}",
                &format!("INV-{:06}", rng.gen_range(1..999999)),
            );
        }

        if let Some(ref val) = context.po_number {
            result = result.replace("{PONumber}", val);
        } else {
            result = result.replace("{PONumber}", &format!("PO-{:06}", rng.gen_range(1..999999)));
        }

        if let Some(ref val) = context.month_name {
            result = result.replace("{Month}", val);
        } else {
            result = result.replace("{Month}", "January");
        }

        if let Some(ref val) = context.year {
            result = result.replace("{Year}", val);
        } else {
            result = result.replace("{Year}", "2024");
        }

        if let Some(ref val) = context.quarter {
            result = result.replace("{Quarter}", val);
        } else {
            result = result.replace("{Quarter}", "Q1");
        }

        if let Some(ref val) = context.asset_description {
            result = result.replace("{AssetDescription}", val);
        } else {
            result = result.replace("{AssetDescription}", &self.generate_asset_description(rng));
        }

        if let Some(ref val) = context.project_name {
            result = result.replace("{ProjectName}", val);
        } else {
            result = result.replace("{ProjectName}", &self.generate_project_name(rng));
        }

        if let Some(ref val) = context.department_name {
            result = result.replace("{Department}", val);
        } else {
            result = result.replace("{Department}", "Finance");
        }

        if let Some(ref val) = context.employee_name {
            result = result.replace("{EmployeeName}", val);
        } else {
            result = result.replace("{EmployeeName}", "Employee");
        }

        result
    }

    fn generate_vendor_name(&self, rng: &mut impl Rng) -> String {
        let vendors = [
            "Acme Supplies Inc",
            "Global Tech Solutions",
            "Office Depot",
            "Amazon Business",
            "Dell Technologies",
            "Microsoft Corporation",
            "Adobe Systems",
            "Salesforce Inc",
            "Oracle Corporation",
            "ServiceNow Inc",
            "Workday Inc",
            "SAP America",
            "IBM Corporation",
            "Cisco Systems",
            "HP Inc",
            "Lenovo Group",
            "Apple Inc",
            "Google Cloud",
            "AWS Inc",
            "Zoom Communications",
        ];
        vendors.choose(rng).unwrap_or(&"Vendor").to_string()
    }

    fn generate_customer_name(&self, rng: &mut impl Rng) -> String {
        let customers = [
            "Northwind Traders",
            "Contoso Ltd",
            "Adventure Works",
            "Fabrikam Inc",
            "Tailspin Toys",
            "Wide World Importers",
            "Proseware Inc",
            "Coho Vineyard",
            "Alpine Ski House",
            "Bellows College",
            "Datum Corporation",
            "Litware Inc",
            "Lucerne Publishing",
            "Margie Travel",
            "Trey Research",
            "Fourth Coffee",
            "Graphic Design Institute",
            "School of Fine Art",
            "VanArsdel Ltd",
            "Wingtip Toys",
        ];
        customers.choose(rng).unwrap_or(&"Customer").to_string()
    }

    fn generate_asset_description(&self, rng: &mut impl Rng) -> String {
        let assets = [
            "Server Equipment",
            "Network Infrastructure",
            "Office Renovation",
            "Manufacturing Equipment",
            "Delivery Vehicle",
            "Computer Hardware",
            "Software License",
            "Building Improvement",
            "Lab Equipment",
            "Security System",
        ];
        assets.choose(rng).unwrap_or(&"Asset").to_string()
    }

    fn generate_project_name(&self, rng: &mut impl Rng) -> String {
        let projects = [
            "Digital Transformation",
            "ERP Implementation",
            "Data Center Upgrade",
            "Office Expansion",
            "Process Automation",
            "Cloud Migration",
            "Security Enhancement",
            "Customer Portal",
            "Mobile App Development",
            "Infrastructure Modernization",
        ];
        projects.choose(rng).unwrap_or(&"Project").to_string()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_header_text_generation() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let generator = DescriptionGenerator::new();
        let context = DescriptionContext::with_period(3, 2024);

        let text = generator.generate_header_text(BusinessProcess::P2P, &context, &mut rng);
        assert!(!text.is_empty());
    }

    #[test]
    fn test_line_text_generation() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let generator = DescriptionGenerator::new();
        let context = DescriptionContext::default();

        // Test expense account
        let expense_text = generator.generate_line_text("500100", &context, &mut rng);
        assert!(!expense_text.is_empty());

        // Test revenue account
        let revenue_text = generator.generate_line_text("400100", &context, &mut rng);
        assert!(!revenue_text.is_empty());

        // Test asset account
        let asset_text = generator.generate_line_text("100000", &context, &mut rng);
        assert!(!asset_text.is_empty());
    }

    #[test]
    fn test_placeholder_substitution() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let generator = DescriptionGenerator::new();
        let mut context = DescriptionContext::with_period(6, 2024);
        context.vendor_name = Some("Test Vendor Inc".to_string());

        let text = generator.generate_header_text(BusinessProcess::P2P, &context, &mut rng);

        // Should not contain raw placeholders
        assert!(!text.contains('{'));
        assert!(!text.contains('}'));
    }

    #[test]
    fn test_all_business_processes() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let generator = DescriptionGenerator::new();
        let context = DescriptionContext::with_period(1, 2024);

        let processes = [
            BusinessProcess::O2C,
            BusinessProcess::P2P,
            BusinessProcess::R2R,
            BusinessProcess::H2R,
            BusinessProcess::A2R,
            BusinessProcess::Treasury,
            BusinessProcess::Tax,
            BusinessProcess::Intercompany,
        ];

        for process in processes {
            let text = generator.generate_header_text(process, &context, &mut rng);
            assert!(!text.is_empty(), "Empty text for {:?}", process);
        }
    }
}
