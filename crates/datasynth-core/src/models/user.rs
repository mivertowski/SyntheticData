//! User persona and behavior models.
//!
//! Defines user personas and behavioral patterns for realistic
//! transaction generation, including working hours, error rates,
//! and transaction volumes. Also includes Employee model with
//! manager hierarchy for organizational structure simulation.

use std::fmt;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// User persona classification for behavioral modeling.
///
/// Different personas exhibit different transaction patterns, timing,
/// error rates, and access to accounts/functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum UserPersona {
    /// Entry-level accountant with limited access
    JuniorAccountant,
    /// Experienced accountant with broader access
    SeniorAccountant,
    /// Financial controller with approval authority
    Controller,
    /// Management with override capabilities
    Manager,
    /// CFO/Finance Director with full access
    Executive,
    /// Automated batch job or interface
    #[default]
    AutomatedSystem,
    /// External auditor with read access
    ExternalAuditor,
    /// Fraud actor for simulation scenarios
    FraudActor,
}

impl fmt::Display for UserPersona {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::JuniorAccountant => write!(f, "junior_accountant"),
            Self::SeniorAccountant => write!(f, "senior_accountant"),
            Self::Controller => write!(f, "controller"),
            Self::Manager => write!(f, "manager"),
            Self::Executive => write!(f, "executive"),
            Self::AutomatedSystem => write!(f, "automated_system"),
            Self::ExternalAuditor => write!(f, "external_auditor"),
            Self::FraudActor => write!(f, "fraud_actor"),
        }
    }
}

impl UserPersona {
    /// Check if this persona represents a human user.
    pub fn is_human(&self) -> bool {
        !matches!(self, Self::AutomatedSystem)
    }

    /// Check if this persona has approval authority.
    pub fn has_approval_authority(&self) -> bool {
        matches!(self, Self::Controller | Self::Manager | Self::Executive)
    }

    /// Get typical error rate for this persona (0.0-1.0).
    pub fn error_rate(&self) -> f64 {
        match self {
            Self::JuniorAccountant => 0.02,
            Self::SeniorAccountant => 0.005,
            Self::Controller => 0.002,
            Self::Manager => 0.003,
            Self::Executive => 0.001,
            Self::AutomatedSystem => 0.0001,
            Self::ExternalAuditor => 0.0,
            Self::FraudActor => 0.01,
        }
    }

    /// Get typical transaction volume per day.
    pub fn typical_daily_volume(&self) -> (u32, u32) {
        match self {
            Self::JuniorAccountant => (20, 100),
            Self::SeniorAccountant => (10, 50),
            Self::Controller => (5, 20),
            Self::Manager => (1, 10),
            Self::Executive => (0, 5),
            Self::AutomatedSystem => (100, 10000),
            Self::ExternalAuditor => (0, 0),
            Self::FraudActor => (1, 5),
        }
    }

    /// Get approval threshold amount.
    pub fn approval_threshold(&self) -> Option<f64> {
        match self {
            Self::JuniorAccountant => Some(1000.0),
            Self::SeniorAccountant => Some(10000.0),
            Self::Controller => Some(100000.0),
            Self::Manager => Some(500000.0),
            Self::Executive => None, // Unlimited
            Self::AutomatedSystem => Some(1000000.0),
            Self::ExternalAuditor => Some(0.0), // Read-only
            Self::FraudActor => Some(10000.0),
        }
    }
}

/// Working hours pattern for human users.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingHoursPattern {
    /// Start hour (0-23)
    pub start_hour: u8,
    /// End hour (0-23)
    pub end_hour: u8,
    /// Peak hours (typically mid-morning and mid-afternoon)
    pub peak_hours: Vec<u8>,
    /// Probability of weekend work
    pub weekend_probability: f64,
    /// Probability of after-hours work
    pub after_hours_probability: f64,
}

impl Default for WorkingHoursPattern {
    fn default() -> Self {
        Self {
            start_hour: 8,
            end_hour: 18,
            peak_hours: vec![10, 11, 14, 15],
            weekend_probability: 0.05,
            after_hours_probability: 0.10,
        }
    }
}

impl WorkingHoursPattern {
    /// Pattern for European office hours.
    pub fn european() -> Self {
        Self {
            start_hour: 9,
            end_hour: 17,
            peak_hours: vec![10, 11, 14, 15],
            weekend_probability: 0.02,
            after_hours_probability: 0.05,
        }
    }

    /// Pattern for US office hours.
    pub fn us_standard() -> Self {
        Self {
            start_hour: 8,
            end_hour: 17,
            peak_hours: vec![9, 10, 14, 15],
            weekend_probability: 0.05,
            after_hours_probability: 0.10,
        }
    }

    /// Pattern for Asian office hours.
    pub fn asian() -> Self {
        Self {
            start_hour: 9,
            end_hour: 18,
            peak_hours: vec![10, 11, 15, 16],
            weekend_probability: 0.10,
            after_hours_probability: 0.15,
        }
    }

    /// Pattern for 24/7 batch processing.
    pub fn batch_processing() -> Self {
        Self {
            start_hour: 0,
            end_hour: 24,
            peak_hours: vec![2, 3, 4, 22, 23], // Off-peak hours for systems
            weekend_probability: 1.0,
            after_hours_probability: 1.0,
        }
    }

    /// Check if an hour is within working hours.
    pub fn is_working_hour(&self, hour: u8) -> bool {
        hour >= self.start_hour && hour < self.end_hour
    }

    /// Check if an hour is a peak hour.
    pub fn is_peak_hour(&self, hour: u8) -> bool {
        self.peak_hours.contains(&hour)
    }
}

/// Individual user account for transaction attribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// User ID (login name)
    pub user_id: String,

    /// Display name
    pub display_name: String,

    /// Email address
    pub email: Option<String>,

    /// Persona classification
    pub persona: UserPersona,

    /// Department
    pub department: Option<String>,

    /// Working hours pattern
    pub working_hours: WorkingHoursPattern,

    /// Assigned company codes
    pub company_codes: Vec<String>,

    /// Assigned cost centers (can post to)
    pub cost_centers: Vec<String>,

    /// Is this user currently active
    pub is_active: bool,

    /// Start date of employment
    pub start_date: Option<chrono::NaiveDate>,

    /// End date of employment (if terminated)
    pub end_date: Option<chrono::NaiveDate>,
}

impl User {
    /// Create a new user with minimal required fields.
    pub fn new(user_id: String, display_name: String, persona: UserPersona) -> Self {
        let working_hours = if persona.is_human() {
            WorkingHoursPattern::default()
        } else {
            WorkingHoursPattern::batch_processing()
        };

        Self {
            user_id,
            display_name,
            email: None,
            persona,
            department: None,
            working_hours,
            company_codes: Vec::new(),
            cost_centers: Vec::new(),
            is_active: true,
            start_date: None,
            end_date: None,
        }
    }

    /// Create a system/batch user.
    pub fn system(user_id: &str) -> Self {
        Self::new(
            user_id.to_string(),
            format!("System User {user_id}"),
            UserPersona::AutomatedSystem,
        )
    }

    /// Check if user can post to a company code.
    pub fn can_post_to_company(&self, company_code: &str) -> bool {
        self.company_codes.is_empty() || self.company_codes.iter().any(|c| c == company_code)
    }

    /// Generate a typical username for a persona.
    pub fn generate_username(persona: UserPersona, index: usize) -> String {
        match persona {
            UserPersona::JuniorAccountant => format!("JACC{index:04}"),
            UserPersona::SeniorAccountant => format!("SACC{index:04}"),
            UserPersona::Controller => format!("CTRL{index:04}"),
            UserPersona::Manager => format!("MGR{index:04}"),
            UserPersona::Executive => format!("EXEC{index:04}"),
            UserPersona::AutomatedSystem => format!("BATCH{index:04}"),
            UserPersona::ExternalAuditor => format!("AUDIT{index:04}"),
            UserPersona::FraudActor => format!("USER{index:04}"), // Appears normal
        }
    }
}

/// Pool of users for transaction attribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPool {
    /// All users in the pool
    pub users: Vec<User>,
    /// Index by persona for quick lookup
    #[serde(skip)]
    persona_index: std::collections::HashMap<UserPersona, Vec<usize>>,
}

impl UserPool {
    /// Create a new empty user pool.
    pub fn new() -> Self {
        Self {
            users: Vec::new(),
            persona_index: std::collections::HashMap::new(),
        }
    }

    /// Add a user to the pool.
    pub fn add_user(&mut self, user: User) {
        let idx = self.users.len();
        let persona = user.persona;
        self.users.push(user);
        self.persona_index.entry(persona).or_default().push(idx);
    }

    /// Get all users of a specific persona.
    pub fn get_users_by_persona(&self, persona: UserPersona) -> Vec<&User> {
        self.persona_index
            .get(&persona)
            .map(|indices| indices.iter().map(|&i| &self.users[i]).collect())
            .unwrap_or_default()
    }

    /// Get a random user of a specific persona.
    pub fn get_random_user(&self, persona: UserPersona, rng: &mut impl rand::Rng) -> Option<&User> {
        use rand::seq::IndexedRandom;
        self.get_users_by_persona(persona).choose(rng).copied()
    }

    /// Rebuild the persona index (call after deserialization).
    pub fn rebuild_index(&mut self) {
        self.persona_index.clear();
        for (idx, user) in self.users.iter().enumerate() {
            self.persona_index
                .entry(user.persona)
                .or_default()
                .push(idx);
        }
    }

    /// Generate a standard user pool with typical distribution.
    pub fn generate_standard(company_codes: &[String]) -> Self {
        let mut pool = Self::new();

        // Junior accountants (many)
        for i in 0..10 {
            let mut user = User::new(
                User::generate_username(UserPersona::JuniorAccountant, i),
                format!("Junior Accountant {}", i + 1),
                UserPersona::JuniorAccountant,
            );
            user.company_codes = company_codes.to_vec();
            pool.add_user(user);
        }

        // Senior accountants
        for i in 0..5 {
            let mut user = User::new(
                User::generate_username(UserPersona::SeniorAccountant, i),
                format!("Senior Accountant {}", i + 1),
                UserPersona::SeniorAccountant,
            );
            user.company_codes = company_codes.to_vec();
            pool.add_user(user);
        }

        // Controllers
        for i in 0..2 {
            let mut user = User::new(
                User::generate_username(UserPersona::Controller, i),
                format!("Controller {}", i + 1),
                UserPersona::Controller,
            );
            user.company_codes = company_codes.to_vec();
            pool.add_user(user);
        }

        // Managers
        for i in 0..3 {
            let mut user = User::new(
                User::generate_username(UserPersona::Manager, i),
                format!("Finance Manager {}", i + 1),
                UserPersona::Manager,
            );
            user.company_codes = company_codes.to_vec();
            pool.add_user(user);
        }

        // Automated systems (many)
        for i in 0..20 {
            let mut user = User::new(
                User::generate_username(UserPersona::AutomatedSystem, i),
                format!("Batch Job {}", i + 1),
                UserPersona::AutomatedSystem,
            );
            user.company_codes = company_codes.to_vec();
            pool.add_user(user);
        }

        pool
    }
}

impl Default for UserPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Employee job level in the organization hierarchy.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum JobLevel {
    /// Individual contributor
    #[default]
    Staff,
    /// Senior individual contributor
    Senior,
    /// Lead/principal
    Lead,
    /// First-line manager
    Supervisor,
    /// Middle management
    Manager,
    /// Senior manager / director
    Director,
    /// VP level
    VicePresident,
    /// C-level executive
    Executive,
}

impl JobLevel {
    /// Get the management level (0 = IC, higher = more senior).
    pub fn management_level(&self) -> u8 {
        match self {
            Self::Staff => 0,
            Self::Senior => 0,
            Self::Lead => 1,
            Self::Supervisor => 2,
            Self::Manager => 3,
            Self::Director => 4,
            Self::VicePresident => 5,
            Self::Executive => 6,
        }
    }

    /// Check if this is a management position.
    pub fn is_manager(&self) -> bool {
        self.management_level() >= 2
    }

    /// Get typical direct reports range.
    pub fn typical_direct_reports(&self) -> (u16, u16) {
        match self {
            Self::Staff | Self::Senior => (0, 0),
            Self::Lead => (0, 3),
            Self::Supervisor => (3, 10),
            Self::Manager => (5, 15),
            Self::Director => (3, 8),
            Self::VicePresident => (3, 6),
            Self::Executive => (5, 12),
        }
    }
}

/// Employee status in HR system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EmployeeStatus {
    /// Active employee
    #[default]
    Active,
    /// On leave (sabbatical, parental, etc.)
    OnLeave,
    /// Suspended
    Suspended,
    /// Notice period
    NoticePeriod,
    /// Terminated
    Terminated,
    /// Retired
    Retired,
    /// Contractor (not full employee)
    Contractor,
}

impl EmployeeStatus {
    /// Check if employee can perform transactions.
    pub fn can_transact(&self) -> bool {
        matches!(self, Self::Active | Self::Contractor)
    }

    /// Check if employee is active in some capacity.
    pub fn is_active(&self) -> bool {
        !matches!(self, Self::Terminated | Self::Retired)
    }
}

/// System role for access control.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemRole {
    /// View-only access
    Viewer,
    /// Can create documents
    Creator,
    /// Can approve documents
    Approver,
    /// Can release payments
    PaymentReleaser,
    /// Can perform bank transactions
    BankProcessor,
    /// Can post journal entries
    JournalPoster,
    /// Can perform period close activities
    PeriodClose,
    /// System administrator
    Admin,
    /// AP Accountant
    ApAccountant,
    /// AR Accountant
    ArAccountant,
    /// Buyer / Procurement
    Buyer,
    /// Executive / Management
    Executive,
    /// Financial Analyst
    FinancialAnalyst,
    /// General Accountant
    GeneralAccountant,
    /// Custom role with name
    Custom(String),
}

/// Transaction code authorization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionCodeAuth {
    /// Transaction code (e.g., "FB01", "ME21N")
    pub tcode: String,
    /// Activity type (create, change, display, delete)
    pub activity: ActivityType,
    /// Is authorization active?
    pub active: bool,
}

/// Activity types for authorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    /// Display only
    #[default]
    Display,
    /// Create new
    Create,
    /// Change existing
    Change,
    /// Delete
    Delete,
    /// Execute (for reports)
    Execute,
}

/// Employee master data with organizational hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Employee {
    /// Employee ID (e.g., "E-001234")
    pub employee_id: String,

    /// User ID (login name, links to User)
    pub user_id: String,

    /// Display name
    pub display_name: String,

    /// First name
    pub first_name: String,

    /// Last name
    pub last_name: String,

    /// Email address
    pub email: String,

    /// Persona classification
    pub persona: UserPersona,

    /// Job level
    pub job_level: JobLevel,

    /// Job title
    pub job_title: String,

    /// Department ID
    pub department_id: Option<String>,

    /// Cost center
    pub cost_center: Option<String>,

    /// Manager's employee ID (for hierarchy)
    pub manager_id: Option<String>,

    /// Direct reports (employee IDs)
    pub direct_reports: Vec<String>,

    /// Employment status
    pub status: EmployeeStatus,

    /// Company code
    pub company_code: String,

    /// Working hours pattern
    pub working_hours: WorkingHoursPattern,

    /// Authorized company codes
    pub authorized_company_codes: Vec<String>,

    /// Authorized cost centers
    pub authorized_cost_centers: Vec<String>,

    /// Approval limit (monetary threshold)
    pub approval_limit: Decimal,

    /// Can approve purchase requisitions
    pub can_approve_pr: bool,

    /// Can approve purchase orders
    pub can_approve_po: bool,

    /// Can approve invoices
    pub can_approve_invoice: bool,

    /// Can approve journal entries
    pub can_approve_je: bool,

    /// Can release payments
    pub can_release_payment: bool,

    /// System roles
    pub system_roles: Vec<SystemRole>,

    /// Authorized transaction codes
    pub transaction_codes: Vec<TransactionCodeAuth>,

    /// Hire date
    pub hire_date: Option<chrono::NaiveDate>,

    /// Termination date (if applicable)
    pub termination_date: Option<chrono::NaiveDate>,

    /// Location / plant
    pub location: Option<String>,

    /// Is this an intercompany employee (works for multiple entities)?
    pub is_shared_services: bool,

    /// Phone number
    pub phone: Option<String>,

    /// Annual base salary in the company's local currency.
    ///
    /// Used by the payroll generator to compute monthly gross pay
    /// (`base_salary / 12`) instead of a hardcoded default.
    #[serde(with = "rust_decimal::serde::str")]
    pub base_salary: rust_decimal::Decimal,
}

impl Employee {
    /// Create a new employee.
    pub fn new(
        employee_id: impl Into<String>,
        user_id: impl Into<String>,
        first_name: impl Into<String>,
        last_name: impl Into<String>,
        company_code: impl Into<String>,
    ) -> Self {
        let first = first_name.into();
        let last = last_name.into();
        let uid = user_id.into();
        let display_name = format!("{first} {last}");
        let email = format!(
            "{}.{}@company.com",
            first.to_lowercase(),
            last.to_lowercase()
        );

        Self {
            employee_id: employee_id.into(),
            user_id: uid,
            display_name,
            first_name: first,
            last_name: last,
            email,
            persona: UserPersona::JuniorAccountant,
            job_level: JobLevel::Staff,
            job_title: "Staff Accountant".to_string(),
            department_id: None,
            cost_center: None,
            manager_id: None,
            direct_reports: Vec::new(),
            status: EmployeeStatus::Active,
            company_code: company_code.into(),
            working_hours: WorkingHoursPattern::default(),
            authorized_company_codes: Vec::new(),
            authorized_cost_centers: Vec::new(),
            approval_limit: Decimal::ZERO,
            can_approve_pr: false,
            can_approve_po: false,
            can_approve_invoice: false,
            can_approve_je: false,
            can_release_payment: false,
            system_roles: Vec::new(),
            transaction_codes: Vec::new(),
            hire_date: None,
            termination_date: None,
            location: None,
            is_shared_services: false,
            phone: None,
            base_salary: Decimal::ZERO,
        }
    }

    /// Set persona and adjust defaults accordingly.
    pub fn with_persona(mut self, persona: UserPersona) -> Self {
        self.persona = persona;

        // Adjust job level and approval limit based on persona
        match persona {
            UserPersona::JuniorAccountant => {
                self.job_level = JobLevel::Staff;
                self.job_title = "Junior Accountant".to_string();
                self.approval_limit = Decimal::from(1000);
            }
            UserPersona::SeniorAccountant => {
                self.job_level = JobLevel::Senior;
                self.job_title = "Senior Accountant".to_string();
                self.approval_limit = Decimal::from(10000);
                self.can_approve_je = true;
            }
            UserPersona::Controller => {
                self.job_level = JobLevel::Manager;
                self.job_title = "Controller".to_string();
                self.approval_limit = Decimal::from(100000);
                self.can_approve_pr = true;
                self.can_approve_po = true;
                self.can_approve_invoice = true;
                self.can_approve_je = true;
            }
            UserPersona::Manager => {
                self.job_level = JobLevel::Director;
                self.job_title = "Finance Director".to_string();
                self.approval_limit = Decimal::from(500000);
                self.can_approve_pr = true;
                self.can_approve_po = true;
                self.can_approve_invoice = true;
                self.can_approve_je = true;
                self.can_release_payment = true;
            }
            UserPersona::Executive => {
                self.job_level = JobLevel::Executive;
                self.job_title = "CFO".to_string();
                self.approval_limit = Decimal::from(999_999_999); // Unlimited
                self.can_approve_pr = true;
                self.can_approve_po = true;
                self.can_approve_invoice = true;
                self.can_approve_je = true;
                self.can_release_payment = true;
            }
            UserPersona::AutomatedSystem => {
                self.job_level = JobLevel::Staff;
                self.job_title = "Batch Process".to_string();
                self.working_hours = WorkingHoursPattern::batch_processing();
            }
            UserPersona::ExternalAuditor => {
                self.job_level = JobLevel::Senior;
                self.job_title = "External Auditor".to_string();
                self.approval_limit = Decimal::ZERO;
            }
            UserPersona::FraudActor => {
                self.job_level = JobLevel::Staff;
                self.job_title = "Staff Accountant".to_string();
                self.approval_limit = Decimal::from(10000);
            }
        }
        self
    }

    /// Set job level.
    pub fn with_job_level(mut self, level: JobLevel) -> Self {
        self.job_level = level;
        self
    }

    /// Set job title.
    pub fn with_job_title(mut self, title: impl Into<String>) -> Self {
        self.job_title = title.into();
        self
    }

    /// Set manager.
    pub fn with_manager(mut self, manager_id: impl Into<String>) -> Self {
        self.manager_id = Some(manager_id.into());
        self
    }

    /// Set department.
    pub fn with_department(mut self, department_id: impl Into<String>) -> Self {
        self.department_id = Some(department_id.into());
        self
    }

    /// Set cost center.
    pub fn with_cost_center(mut self, cost_center: impl Into<String>) -> Self {
        self.cost_center = Some(cost_center.into());
        self
    }

    /// Set approval limit.
    pub fn with_approval_limit(mut self, limit: Decimal) -> Self {
        self.approval_limit = limit;
        self
    }

    /// Add authorized company code.
    pub fn with_authorized_company(mut self, company_code: impl Into<String>) -> Self {
        self.authorized_company_codes.push(company_code.into());
        self
    }

    /// Add system role.
    pub fn with_role(mut self, role: SystemRole) -> Self {
        self.system_roles.push(role);
        self
    }

    /// Set hire date.
    pub fn with_hire_date(mut self, date: chrono::NaiveDate) -> Self {
        self.hire_date = Some(date);
        self
    }

    /// Add a direct report.
    pub fn add_direct_report(&mut self, employee_id: String) {
        if !self.direct_reports.contains(&employee_id) {
            self.direct_reports.push(employee_id);
        }
    }

    /// Check if employee can approve an amount.
    pub fn can_approve_amount(&self, amount: Decimal) -> bool {
        if self.status != EmployeeStatus::Active {
            return false;
        }
        amount <= self.approval_limit
    }

    /// Check if employee can approve in a company code.
    pub fn can_approve_in_company(&self, company_code: &str) -> bool {
        if self.status != EmployeeStatus::Active {
            return false;
        }
        self.authorized_company_codes.is_empty()
            || self
                .authorized_company_codes
                .iter()
                .any(|c| c == company_code)
    }

    /// Check if employee has a specific role.
    pub fn has_role(&self, role: &SystemRole) -> bool {
        self.system_roles.contains(role)
    }

    /// Get the depth in the org hierarchy (0 = top).
    pub fn hierarchy_depth(&self) -> u8 {
        // This would need the full employee registry to compute properly
        // For now, estimate based on job level
        match self.job_level {
            JobLevel::Executive => 0,
            JobLevel::VicePresident => 1,
            JobLevel::Director => 2,
            JobLevel::Manager => 3,
            JobLevel::Supervisor => 4,
            JobLevel::Lead => 5,
            JobLevel::Senior => 6,
            JobLevel::Staff => 7,
        }
    }

    /// Convert to a User (for backward compatibility).
    pub fn to_user(&self) -> User {
        let mut user = User::new(
            self.user_id.clone(),
            self.display_name.clone(),
            self.persona,
        );
        user.email = Some(self.email.clone());
        user.department = self.department_id.clone();
        user.working_hours = self.working_hours.clone();
        user.company_codes = self.authorized_company_codes.clone();
        user.cost_centers = self.authorized_cost_centers.clone();
        user.is_active = self.status.can_transact();
        user.start_date = self.hire_date;
        user.end_date = self.termination_date;
        user
    }
}

/// Pool of employees with organizational hierarchy support.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmployeePool {
    /// All employees
    pub employees: Vec<Employee>,
    /// Index by employee ID
    #[serde(skip)]
    id_index: std::collections::HashMap<String, usize>,
    /// Index by manager ID (for finding direct reports)
    #[serde(skip)]
    manager_index: std::collections::HashMap<String, Vec<usize>>,
    /// Index by persona
    #[serde(skip)]
    persona_index: std::collections::HashMap<UserPersona, Vec<usize>>,
    /// Index by department
    #[serde(skip)]
    department_index: std::collections::HashMap<String, Vec<usize>>,
}

impl EmployeePool {
    /// Create a new empty employee pool.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an employee to the pool.
    pub fn add_employee(&mut self, employee: Employee) {
        let idx = self.employees.len();

        self.id_index.insert(employee.employee_id.clone(), idx);

        if let Some(ref mgr_id) = employee.manager_id {
            self.manager_index
                .entry(mgr_id.clone())
                .or_default()
                .push(idx);
        }

        self.persona_index
            .entry(employee.persona)
            .or_default()
            .push(idx);

        if let Some(ref dept_id) = employee.department_id {
            self.department_index
                .entry(dept_id.clone())
                .or_default()
                .push(idx);
        }

        self.employees.push(employee);
    }

    /// Get employee by ID.
    pub fn get_by_id(&self, employee_id: &str) -> Option<&Employee> {
        self.id_index
            .get(employee_id)
            .map(|&idx| &self.employees[idx])
    }

    /// Get mutable employee by ID.
    pub fn get_by_id_mut(&mut self, employee_id: &str) -> Option<&mut Employee> {
        self.id_index
            .get(employee_id)
            .copied()
            .map(|idx| &mut self.employees[idx])
    }

    /// Get direct reports of a manager.
    pub fn get_direct_reports(&self, manager_id: &str) -> Vec<&Employee> {
        self.manager_index
            .get(manager_id)
            .map(|indices| indices.iter().map(|&i| &self.employees[i]).collect())
            .unwrap_or_default()
    }

    /// Get employees by persona.
    pub fn get_by_persona(&self, persona: UserPersona) -> Vec<&Employee> {
        self.persona_index
            .get(&persona)
            .map(|indices| indices.iter().map(|&i| &self.employees[i]).collect())
            .unwrap_or_default()
    }

    /// Get employees by department.
    pub fn get_by_department(&self, department_id: &str) -> Vec<&Employee> {
        self.department_index
            .get(department_id)
            .map(|indices| indices.iter().map(|&i| &self.employees[i]).collect())
            .unwrap_or_default()
    }

    /// Get a random employee with approval authority.
    pub fn get_random_approver(&self, rng: &mut impl rand::Rng) -> Option<&Employee> {
        use rand::seq::IndexedRandom;

        let approvers: Vec<_> = self
            .employees
            .iter()
            .filter(|e| e.persona.has_approval_authority() && e.status == EmployeeStatus::Active)
            .collect();

        approvers.choose(rng).copied()
    }

    /// Get approver for a specific amount.
    pub fn get_approver_for_amount(
        &self,
        amount: Decimal,
        rng: &mut impl rand::Rng,
    ) -> Option<&Employee> {
        use rand::seq::IndexedRandom;

        let approvers: Vec<_> = self
            .employees
            .iter()
            .filter(|e| e.can_approve_amount(amount))
            .collect();

        approvers.choose(rng).copied()
    }

    /// Get all managers (employees with direct reports).
    pub fn get_managers(&self) -> Vec<&Employee> {
        self.employees
            .iter()
            .filter(|e| !e.direct_reports.is_empty() || e.job_level.is_manager())
            .collect()
    }

    /// Get org chart path from employee to top.
    pub fn get_reporting_chain(&self, employee_id: &str) -> Vec<&Employee> {
        let mut chain = Vec::new();
        let mut current_id = employee_id.to_string();

        while let Some(emp) = self.get_by_id(&current_id) {
            chain.push(emp);
            if let Some(ref mgr_id) = emp.manager_id {
                current_id = mgr_id.clone();
            } else {
                break;
            }
        }

        chain
    }

    /// Rebuild indices after deserialization.
    pub fn rebuild_indices(&mut self) {
        self.id_index.clear();
        self.manager_index.clear();
        self.persona_index.clear();
        self.department_index.clear();

        for (idx, employee) in self.employees.iter().enumerate() {
            self.id_index.insert(employee.employee_id.clone(), idx);

            if let Some(ref mgr_id) = employee.manager_id {
                self.manager_index
                    .entry(mgr_id.clone())
                    .or_default()
                    .push(idx);
            }

            self.persona_index
                .entry(employee.persona)
                .or_default()
                .push(idx);

            if let Some(ref dept_id) = employee.department_id {
                self.department_index
                    .entry(dept_id.clone())
                    .or_default()
                    .push(idx);
            }
        }
    }

    /// Get count.
    pub fn len(&self) -> usize {
        self.employees.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.employees.is_empty()
    }
}

/// Type of employee lifecycle event recorded in the change history.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EmployeeEventType {
    /// Employee was hired (always the first event).
    #[default]
    Hired,
    /// Employee received a promotion to a higher job level.
    Promoted,
    /// Employee received a salary adjustment (increase or decrease).
    SalaryAdjustment,
    /// Employee was transferred to a different department or cost center.
    Transfer,
    /// Employee's employment was terminated.
    Terminated,
}

impl std::fmt::Display for EmployeeEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hired => write!(f, "hired"),
            Self::Promoted => write!(f, "promoted"),
            Self::SalaryAdjustment => write!(f, "salary_adjustment"),
            Self::Transfer => write!(f, "transfer"),
            Self::Terminated => write!(f, "terminated"),
        }
    }
}

/// A single entry in an employee's change history.
///
/// Captures point-in-time changes to an employee record (hire, promotion,
/// salary adjustment, transfer, or termination).  The `old_value` /
/// `new_value` fields carry a string representation of the changed attribute
/// (e.g., job level code, salary amount, department name) to keep the schema
/// generic and easy to consume from downstream analytics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeChangeEvent {
    /// Employee this event belongs to.
    pub employee_id: String,

    /// Calendar date on which the event was recorded in the HR system.
    pub event_date: chrono::NaiveDate,

    /// Type of HR event.
    pub event_type: EmployeeEventType,

    /// Previous value of the changed attribute (`None` for the Hired event).
    pub old_value: Option<String>,

    /// New value of the changed attribute.
    pub new_value: Option<String>,

    /// Date from which the change is effective (may differ from `event_date`
    /// for retroactive or future-dated transactions).
    pub effective_date: chrono::NaiveDate,
}

impl EmployeeChangeEvent {
    /// Create a Hired event for a new employee.
    pub fn hired(employee_id: impl Into<String>, hire_date: chrono::NaiveDate) -> Self {
        Self {
            employee_id: employee_id.into(),
            event_date: hire_date,
            event_type: EmployeeEventType::Hired,
            old_value: None,
            new_value: Some("active".to_string()),
            effective_date: hire_date,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_persona_properties() {
        assert!(UserPersona::JuniorAccountant.is_human());
        assert!(!UserPersona::AutomatedSystem.is_human());
        assert!(UserPersona::Controller.has_approval_authority());
        assert!(!UserPersona::JuniorAccountant.has_approval_authority());
    }

    #[test]
    fn test_persona_display_snake_case() {
        assert_eq!(
            UserPersona::JuniorAccountant.to_string(),
            "junior_accountant"
        );
        assert_eq!(
            UserPersona::SeniorAccountant.to_string(),
            "senior_accountant"
        );
        assert_eq!(UserPersona::Controller.to_string(), "controller");
        assert_eq!(UserPersona::Manager.to_string(), "manager");
        assert_eq!(UserPersona::Executive.to_string(), "executive");
        assert_eq!(UserPersona::AutomatedSystem.to_string(), "automated_system");
        assert_eq!(UserPersona::ExternalAuditor.to_string(), "external_auditor");
        assert_eq!(UserPersona::FraudActor.to_string(), "fraud_actor");

        // Verify no persona produces concatenated words (the bug this fixes)
        for persona in [
            UserPersona::JuniorAccountant,
            UserPersona::SeniorAccountant,
            UserPersona::Controller,
            UserPersona::Manager,
            UserPersona::Executive,
            UserPersona::AutomatedSystem,
            UserPersona::ExternalAuditor,
            UserPersona::FraudActor,
        ] {
            let s = persona.to_string();
            assert!(
                !s.contains(char::is_uppercase),
                "Display output '{}' should be all lowercase snake_case",
                s
            );
        }
    }

    #[test]
    fn test_user_pool() {
        let pool = UserPool::generate_standard(&["1000".to_string()]);
        assert!(!pool.users.is_empty());
        assert!(!pool
            .get_users_by_persona(UserPersona::JuniorAccountant)
            .is_empty());
    }

    #[test]
    fn test_job_level_hierarchy() {
        assert!(JobLevel::Executive.management_level() > JobLevel::Manager.management_level());
        assert!(JobLevel::Manager.is_manager());
        assert!(!JobLevel::Staff.is_manager());
    }

    #[test]
    fn test_employee_creation() {
        let employee = Employee::new("E-001", "jsmith", "John", "Smith", "1000")
            .with_persona(UserPersona::Controller);

        assert_eq!(employee.employee_id, "E-001");
        assert_eq!(employee.display_name, "John Smith");
        assert!(employee.can_approve_je);
        assert_eq!(employee.job_level, JobLevel::Manager);
    }

    #[test]
    fn test_employee_approval_limits() {
        let employee = Employee::new("E-001", "test", "Test", "User", "1000")
            .with_approval_limit(Decimal::from(10000));

        assert!(employee.can_approve_amount(Decimal::from(5000)));
        assert!(!employee.can_approve_amount(Decimal::from(15000)));
    }

    #[test]
    fn test_employee_pool_hierarchy() {
        let mut pool = EmployeePool::new();

        let cfo = Employee::new("E-001", "cfo", "Jane", "CEO", "1000")
            .with_persona(UserPersona::Executive);

        let controller = Employee::new("E-002", "ctrl", "Bob", "Controller", "1000")
            .with_persona(UserPersona::Controller)
            .with_manager("E-001");

        let accountant = Employee::new("E-003", "acc", "Alice", "Accountant", "1000")
            .with_persona(UserPersona::JuniorAccountant)
            .with_manager("E-002");

        pool.add_employee(cfo);
        pool.add_employee(controller);
        pool.add_employee(accountant);

        // Test getting direct reports
        let direct_reports = pool.get_direct_reports("E-001");
        assert_eq!(direct_reports.len(), 1);
        assert_eq!(direct_reports[0].employee_id, "E-002");

        // Test reporting chain
        let chain = pool.get_reporting_chain("E-003");
        assert_eq!(chain.len(), 3);
        assert_eq!(chain[0].employee_id, "E-003");
        assert_eq!(chain[1].employee_id, "E-002");
        assert_eq!(chain[2].employee_id, "E-001");
    }

    #[test]
    fn test_employee_to_user() {
        let employee = Employee::new("E-001", "jdoe", "John", "Doe", "1000")
            .with_persona(UserPersona::SeniorAccountant);

        let user = employee.to_user();

        assert_eq!(user.user_id, "jdoe");
        assert_eq!(user.persona, UserPersona::SeniorAccountant);
        assert!(user.is_active);
    }

    #[test]
    fn test_employee_status() {
        assert!(EmployeeStatus::Active.can_transact());
        assert!(EmployeeStatus::Contractor.can_transact());
        assert!(!EmployeeStatus::Terminated.can_transact());
        assert!(!EmployeeStatus::OnLeave.can_transact());
    }
}
