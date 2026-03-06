//! Employee generator with org hierarchy and approval limits.

use chrono::NaiveDate;
use datasynth_core::models::{
    Employee, EmployeePool, EmployeeStatus, JobLevel, SystemRole, TransactionCodeAuth,
};
use datasynth_core::templates::{MultiCultureNameGenerator, NameCulture};
use datasynth_core::utils::seeded_rng;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use tracing::debug;

/// Configuration for employee generation.
#[derive(Debug, Clone)]
pub struct EmployeeGeneratorConfig {
    /// Distribution of job levels (level, probability)
    pub job_level_distribution: Vec<(JobLevel, f64)>,
    /// Approval limits by job level (level, limit)
    pub approval_limits: Vec<(JobLevel, Decimal)>,
    /// Name culture distribution
    pub culture_distribution: Vec<(NameCulture, f64)>,
    /// Email domain
    pub email_domain: String,
    /// Probability of employee being on leave
    pub leave_rate: f64,
    /// Probability of employee being terminated
    pub termination_rate: f64,
    /// Manager span of control (min, max direct reports)
    pub span_of_control: (usize, usize),
}

impl Default for EmployeeGeneratorConfig {
    fn default() -> Self {
        Self {
            job_level_distribution: vec![
                (JobLevel::Staff, 0.50),
                (JobLevel::Senior, 0.25),
                (JobLevel::Manager, 0.12),
                (JobLevel::Director, 0.08),
                (JobLevel::VicePresident, 0.04),
                (JobLevel::Executive, 0.01),
            ],
            approval_limits: vec![
                (JobLevel::Staff, Decimal::from(1_000)),
                (JobLevel::Senior, Decimal::from(5_000)),
                (JobLevel::Manager, Decimal::from(25_000)),
                (JobLevel::Director, Decimal::from(100_000)),
                (JobLevel::VicePresident, Decimal::from(500_000)),
                (JobLevel::Executive, Decimal::from(10_000_000)),
            ],
            culture_distribution: vec![
                (NameCulture::WesternUs, 0.40),
                (NameCulture::Hispanic, 0.20),
                (NameCulture::German, 0.10),
                (NameCulture::French, 0.05),
                (NameCulture::Chinese, 0.10),
                (NameCulture::Japanese, 0.05),
                (NameCulture::Indian, 0.10),
            ],
            email_domain: "company.com".to_string(),
            leave_rate: 0.02,
            termination_rate: 0.01,
            span_of_control: (3, 8),
        }
    }
}

/// Department definitions for employee assignment.
#[derive(Debug, Clone)]
pub struct DepartmentDefinition {
    /// Department code
    pub code: String,
    /// Department name
    pub name: String,
    /// Cost center
    pub cost_center: String,
    /// Target headcount
    pub headcount: usize,
    /// System roles for this department
    pub system_roles: Vec<SystemRole>,
    /// Transaction codes for this department
    pub transaction_codes: Vec<String>,
}

impl DepartmentDefinition {
    /// Finance department.
    pub fn finance(company_code: &str) -> Self {
        Self {
            code: format!("{company_code}-FIN"),
            name: "Finance".to_string(),
            cost_center: format!("CC-{company_code}-FIN"),
            headcount: 15,
            system_roles: vec![
                SystemRole::ApAccountant,
                SystemRole::ArAccountant,
                SystemRole::GeneralAccountant,
                SystemRole::FinancialAnalyst,
            ],
            transaction_codes: vec![
                "FB01".to_string(),
                "FB02".to_string(),
                "FB03".to_string(),
                "F-28".to_string(),
                "F-53".to_string(),
                "FBL1N".to_string(),
            ],
        }
    }

    /// Procurement department.
    pub fn procurement(company_code: &str) -> Self {
        Self {
            code: format!("{company_code}-PROC"),
            name: "Procurement".to_string(),
            cost_center: format!("CC-{company_code}-PROC"),
            headcount: 10,
            system_roles: vec![SystemRole::Buyer, SystemRole::Approver],
            transaction_codes: vec![
                "ME21N".to_string(),
                "ME22N".to_string(),
                "ME23N".to_string(),
                "MIGO".to_string(),
                "ME2M".to_string(),
            ],
        }
    }

    /// Sales department.
    pub fn sales(company_code: &str) -> Self {
        Self {
            code: format!("{company_code}-SALES"),
            name: "Sales".to_string(),
            cost_center: format!("CC-{company_code}-SALES"),
            headcount: 20,
            system_roles: vec![SystemRole::Creator, SystemRole::Approver],
            transaction_codes: vec![
                "VA01".to_string(),
                "VA02".to_string(),
                "VA03".to_string(),
                "VL01N".to_string(),
                "VF01".to_string(),
            ],
        }
    }

    /// Warehouse/Logistics department.
    pub fn warehouse(company_code: &str) -> Self {
        Self {
            code: format!("{company_code}-WH"),
            name: "Warehouse".to_string(),
            cost_center: format!("CC-{company_code}-WH"),
            headcount: 12,
            system_roles: vec![SystemRole::Creator, SystemRole::Viewer],
            transaction_codes: vec![
                "MIGO".to_string(),
                "MB51".to_string(),
                "MMBE".to_string(),
                "LT01".to_string(),
            ],
        }
    }

    /// IT department.
    pub fn it(company_code: &str) -> Self {
        Self {
            code: format!("{company_code}-IT"),
            name: "Information Technology".to_string(),
            cost_center: format!("CC-{company_code}-IT"),
            headcount: 8,
            system_roles: vec![SystemRole::Admin],
            transaction_codes: vec!["SU01".to_string(), "PFCG".to_string(), "SM21".to_string()],
        }
    }

    /// Standard departments for a company.
    pub fn standard_departments(company_code: &str) -> Vec<Self> {
        vec![
            Self::finance(company_code),
            Self::procurement(company_code),
            Self::sales(company_code),
            Self::warehouse(company_code),
            Self::it(company_code),
        ]
    }
}

/// Generator for employee master data with org hierarchy.
pub struct EmployeeGenerator {
    rng: ChaCha8Rng,
    seed: u64,
    config: EmployeeGeneratorConfig,
    name_generator: MultiCultureNameGenerator,
    employee_counter: usize,
    /// Optional country pack for locale-aware generation.
    country_pack: Option<datasynth_core::CountryPack>,
}

impl EmployeeGenerator {
    /// Create a new employee generator.
    pub fn new(seed: u64) -> Self {
        Self::with_config(seed, EmployeeGeneratorConfig::default())
    }

    /// Create a new employee generator with custom configuration.
    pub fn with_config(seed: u64, config: EmployeeGeneratorConfig) -> Self {
        let mut name_gen =
            MultiCultureNameGenerator::with_distribution(config.culture_distribution.clone());
        name_gen.set_email_domain(&config.email_domain);

        Self {
            rng: seeded_rng(seed, 0),
            seed,
            name_generator: name_gen,
            config,
            employee_counter: 0,
            country_pack: None,
        }
    }

    /// Set the country pack for locale-aware generation.
    ///
    /// When set, the generator can use locale-specific names and
    /// business rules from the country pack.  Currently the pack is
    /// stored for future expansion; existing behaviour is unchanged
    /// when no pack is provided.
    pub fn set_country_pack(&mut self, pack: datasynth_core::CountryPack) {
        self.country_pack = Some(pack);
    }

    /// Generate a single employee.
    pub fn generate_employee(
        &mut self,
        company_code: &str,
        department: &DepartmentDefinition,
        hire_date: NaiveDate,
    ) -> Employee {
        self.employee_counter += 1;

        let name = self.name_generator.generate_name(&mut self.rng);
        let employee_id = format!("EMP-{}-{:06}", company_code, self.employee_counter);
        let user_id = format!("u{:06}", self.employee_counter);
        let email = self.name_generator.generate_email(&name);

        let job_level = self.select_job_level();
        let approval_limit = self.get_approval_limit(&job_level);

        let mut employee = Employee::new(
            employee_id,
            user_id,
            name.first_name.clone(),
            name.last_name.clone(),
            company_code.to_string(),
        );

        // Set additional fields
        employee.email = email;
        employee.job_level = job_level;

        // Set department info
        employee.department_id = Some(department.name.clone());
        employee.cost_center = Some(department.cost_center.clone());

        // Set dates
        employee.hire_date = Some(hire_date);

        // Set approval limits based on job level
        employee.approval_limit = approval_limit;
        employee.can_approve_pr = matches!(
            job_level,
            JobLevel::Manager | JobLevel::Director | JobLevel::VicePresident | JobLevel::Executive
        );
        employee.can_approve_po = matches!(
            job_level,
            JobLevel::Senior
                | JobLevel::Manager
                | JobLevel::Director
                | JobLevel::VicePresident
                | JobLevel::Executive
        );
        employee.can_approve_je = matches!(
            job_level,
            JobLevel::Manager | JobLevel::Director | JobLevel::VicePresident | JobLevel::Executive
        );

        // Assign system roles
        if !department.system_roles.is_empty() {
            let role_idx = self.rng.random_range(0..department.system_roles.len());
            employee
                .system_roles
                .push(department.system_roles[role_idx].clone());
        }

        // Assign transaction codes
        for tcode in &department.transaction_codes {
            employee.transaction_codes.push(TransactionCodeAuth {
                tcode: tcode.clone(),
                activity: datasynth_core::models::ActivityType::Create,
                active: true,
            });
        }

        // Set status
        employee.status = self.select_status();
        if employee.status == EmployeeStatus::Terminated {
            employee.termination_date =
                Some(hire_date + chrono::Duration::days(self.rng.random_range(365..1825) as i64));
        }

        employee
    }

    /// Generate an employee with specific job level.
    pub fn generate_employee_with_level(
        &mut self,
        company_code: &str,
        department: &DepartmentDefinition,
        job_level: JobLevel,
        hire_date: NaiveDate,
    ) -> Employee {
        let mut employee = self.generate_employee(company_code, department, hire_date);
        employee.job_level = job_level;
        employee.approval_limit = self.get_approval_limit(&job_level);
        employee.can_approve_pr = matches!(
            job_level,
            JobLevel::Manager | JobLevel::Director | JobLevel::VicePresident | JobLevel::Executive
        );
        employee.can_approve_po = matches!(
            job_level,
            JobLevel::Senior
                | JobLevel::Manager
                | JobLevel::Director
                | JobLevel::VicePresident
                | JobLevel::Executive
        );
        employee.can_approve_je = matches!(
            job_level,
            JobLevel::Manager | JobLevel::Director | JobLevel::VicePresident | JobLevel::Executive
        );
        employee
    }

    /// Generate an employee pool for a department.
    pub fn generate_department_pool(
        &mut self,
        company_code: &str,
        department: &DepartmentDefinition,
        hire_date_range: (NaiveDate, NaiveDate),
    ) -> EmployeePool {
        let mut pool = EmployeePool::new();

        let (start_date, end_date) = hire_date_range;
        let days_range = (end_date - start_date).num_days() as u64;

        // Generate department head (Director or Manager)
        let head_level = if department.headcount >= 15 {
            JobLevel::Director
        } else {
            JobLevel::Manager
        };
        let hire_date =
            start_date + chrono::Duration::days(self.rng.random_range(0..=days_range / 2) as i64);
        let dept_head =
            self.generate_employee_with_level(company_code, department, head_level, hire_date);
        let dept_head_id = dept_head.employee_id.clone();
        pool.add_employee(dept_head);

        // Generate remaining employees
        for _ in 1..department.headcount {
            let hire_date =
                start_date + chrono::Duration::days(self.rng.random_range(0..=days_range) as i64);
            let mut employee = self.generate_employee(company_code, department, hire_date);

            // Assign manager (department head)
            employee.manager_id = Some(dept_head_id.clone());

            pool.add_employee(employee);
        }

        // Collect direct reports first to avoid borrow conflict
        let direct_reports: Vec<String> = pool
            .employees
            .iter()
            .filter(|e| e.manager_id.as_ref() == Some(&dept_head_id))
            .map(|e| e.employee_id.clone())
            .collect();

        // Update direct reports for department head
        if let Some(head) = pool
            .employees
            .iter_mut()
            .find(|e| e.employee_id == dept_head_id)
        {
            head.direct_reports = direct_reports;
        }

        pool
    }

    /// Generate a full company employee pool with hierarchy.
    pub fn generate_company_pool(
        &mut self,
        company_code: &str,
        hire_date_range: (NaiveDate, NaiveDate),
    ) -> EmployeePool {
        debug!(company_code, "Generating employee company pool");
        let mut pool = EmployeePool::new();

        let (start_date, end_date) = hire_date_range;
        let _days_range = (end_date - start_date).num_days() as u64;

        // First, generate C-level executives
        let ceo = self.generate_executive(company_code, "CEO", start_date);
        let ceo_id = ceo.employee_id.clone();
        pool.add_employee(ceo);

        let cfo = self.generate_executive(company_code, "CFO", start_date);
        let cfo_id = cfo.employee_id.clone();
        pool.employees
            .last_mut()
            .expect("just added CEO")
            .manager_id = Some(ceo_id.clone());
        pool.add_employee(cfo);

        let coo = self.generate_executive(company_code, "COO", start_date);
        let coo_id = coo.employee_id.clone();
        pool.employees
            .last_mut()
            .expect("just added CFO")
            .manager_id = Some(ceo_id.clone());
        pool.add_employee(coo);

        // Generate department pools
        let departments = DepartmentDefinition::standard_departments(company_code);

        for dept in &departments {
            let dept_pool = self.generate_department_pool(company_code, dept, hire_date_range);

            // Assign department heads to executives
            for mut employee in dept_pool.employees {
                if employee.manager_id.is_none() {
                    // Department head reports to CFO (finance) or COO (others)
                    employee.manager_id = if dept.name == "Finance" {
                        Some(cfo_id.clone())
                    } else {
                        Some(coo_id.clone())
                    };
                }
                pool.add_employee(employee);
            }
        }

        // Update executive direct reports
        self.update_direct_reports(&mut pool);

        pool
    }

    /// Generate an executive employee.
    fn generate_executive(
        &mut self,
        company_code: &str,
        title: &str,
        hire_date: NaiveDate,
    ) -> Employee {
        self.employee_counter += 1;

        let name = self.name_generator.generate_name(&mut self.rng);
        let employee_id = format!("EMP-{}-{:06}", company_code, self.employee_counter);
        let user_id = format!("exec{:04}", self.employee_counter);
        let email = self.name_generator.generate_email(&name);

        let mut employee = Employee::new(
            employee_id,
            user_id,
            name.first_name.clone(),
            name.last_name.clone(),
            company_code.to_string(),
        );

        employee.email = email;
        employee.job_level = JobLevel::Executive;
        employee.job_title = title.to_string();
        employee.department_id = Some("Executive".to_string());
        employee.cost_center = Some(format!("CC-{company_code}-EXEC"));
        employee.hire_date = Some(hire_date);
        employee.approval_limit = Decimal::from(100_000_000);
        employee.can_approve_pr = true;
        employee.can_approve_po = true;
        employee.can_approve_je = true;
        employee.system_roles.push(SystemRole::Executive);

        employee
    }

    /// Update direct reports for all managers.
    fn update_direct_reports(&self, pool: &mut EmployeePool) {
        // Collect manager -> direct reports mapping
        let mut direct_reports_map: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();

        for employee in &pool.employees {
            if let Some(manager_id) = &employee.manager_id {
                direct_reports_map
                    .entry(manager_id.clone())
                    .or_default()
                    .push(employee.employee_id.clone());
            }
        }

        // Update each manager's direct reports
        for employee in &mut pool.employees {
            if let Some(reports) = direct_reports_map.get(&employee.employee_id) {
                employee.direct_reports = reports.clone();
            }
        }
    }

    /// Select job level based on distribution.
    fn select_job_level(&mut self) -> JobLevel {
        let roll: f64 = self.rng.random();
        let mut cumulative = 0.0;

        for (level, prob) in &self.config.job_level_distribution {
            cumulative += prob;
            if roll < cumulative {
                return *level;
            }
        }

        JobLevel::Staff
    }

    /// Get approval limit for job level.
    fn get_approval_limit(&self, job_level: &JobLevel) -> Decimal {
        for (level, limit) in &self.config.approval_limits {
            if level == job_level {
                return *limit;
            }
        }
        Decimal::from(1_000)
    }

    /// Select employee status.
    fn select_status(&mut self) -> EmployeeStatus {
        let roll: f64 = self.rng.random();

        if roll < self.config.termination_rate {
            EmployeeStatus::Terminated
        } else if roll < self.config.termination_rate + self.config.leave_rate {
            EmployeeStatus::OnLeave
        } else {
            EmployeeStatus::Active
        }
    }

    /// Reset the generator.
    pub fn reset(&mut self) {
        self.rng = seeded_rng(self.seed, 0);
        self.employee_counter = 0;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_employee_generation() {
        let mut gen = EmployeeGenerator::new(42);
        let dept = DepartmentDefinition::finance("1000");
        let employee =
            gen.generate_employee("1000", &dept, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

        assert!(!employee.employee_id.is_empty());
        assert!(!employee.display_name.is_empty());
        assert!(!employee.email.is_empty());
        assert!(employee.approval_limit > Decimal::ZERO);
    }

    #[test]
    fn test_department_pool() {
        let mut gen = EmployeeGenerator::new(42);
        let dept = DepartmentDefinition::finance("1000");
        let pool = gen.generate_department_pool(
            "1000",
            &dept,
            (
                NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            ),
        );

        assert_eq!(pool.employees.len(), dept.headcount);

        // Should have at least one manager
        let managers: Vec<_> = pool
            .employees
            .iter()
            .filter(|e| matches!(e.job_level, JobLevel::Manager | JobLevel::Director))
            .collect();
        assert!(!managers.is_empty());

        // Department head should have direct reports
        let dept_head = managers.first().unwrap();
        assert!(!dept_head.direct_reports.is_empty());
    }

    #[test]
    fn test_company_pool() {
        let mut gen = EmployeeGenerator::new(42);
        let pool = gen.generate_company_pool(
            "1000",
            (
                NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            ),
        );

        // Should have executives
        let executives: Vec<_> = pool
            .employees
            .iter()
            .filter(|e| e.job_level == JobLevel::Executive)
            .collect();
        assert!(executives.len() >= 3); // CEO, CFO, COO

        // Executives should have direct reports
        let cfo = pool.employees.iter().find(|e| e.job_title == "CFO");
        assert!(cfo.is_some());
    }

    #[test]
    fn test_hierarchy() {
        let mut gen = EmployeeGenerator::new(42);
        let pool = gen.generate_company_pool(
            "1000",
            (
                NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            ),
        );

        // Every non-CEO employee should have a manager
        let non_ceo_without_manager: Vec<_> = pool
            .employees
            .iter()
            .filter(|e| e.job_title != "CEO")
            .filter(|e| e.manager_id.is_none())
            .collect();

        // Most employees should have managers (some edge cases may exist)
        assert!(non_ceo_without_manager.len() <= 1);
    }

    #[test]
    fn test_deterministic_generation() {
        let mut gen1 = EmployeeGenerator::new(42);
        let mut gen2 = EmployeeGenerator::new(42);

        let dept = DepartmentDefinition::finance("1000");
        let employee1 =
            gen1.generate_employee("1000", &dept, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        let employee2 =
            gen2.generate_employee("1000", &dept, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

        assert_eq!(employee1.employee_id, employee2.employee_id);
        assert_eq!(employee1.display_name, employee2.display_name);
    }

    #[test]
    fn test_approval_limits() {
        let mut gen = EmployeeGenerator::new(42);
        let dept = DepartmentDefinition::finance("1000");

        let staff = gen.generate_employee_with_level(
            "1000",
            &dept,
            JobLevel::Staff,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );
        let manager = gen.generate_employee_with_level(
            "1000",
            &dept,
            JobLevel::Manager,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        assert!(manager.approval_limit > staff.approval_limit);
        assert!(!staff.can_approve_pr);
        assert!(manager.can_approve_pr);
    }

    #[test]
    fn test_country_pack_does_not_break_generation() {
        let mut gen = EmployeeGenerator::new(42);
        // Setting a default country pack should not alter basic generation behaviour.
        gen.set_country_pack(datasynth_core::CountryPack::default());

        let dept = DepartmentDefinition::finance("1000");
        let employee =
            gen.generate_employee("1000", &dept, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

        assert!(!employee.employee_id.is_empty());
        assert!(!employee.display_name.is_empty());
        assert!(!employee.email.is_empty());
        assert!(employee.approval_limit > Decimal::ZERO);
    }
}
