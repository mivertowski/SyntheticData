//! User pool generator with realistic names and department assignments.

use datasynth_core::models::{
    Department, OrganizationStructure, User, UserPersona, UserPool, WorkingHoursPattern,
};
use datasynth_core::templates::{MultiCultureNameGenerator, NameCulture, PersonName};
use datasynth_core::utils::seeded_rng;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

/// Configuration for user generation.
#[derive(Debug, Clone)]
pub struct UserGeneratorConfig {
    /// Name culture distribution
    pub culture_distribution: Vec<(NameCulture, f64)>,
    /// Email domain for user emails
    pub email_domain: String,
    /// Generate realistic names vs generic IDs
    pub generate_realistic_names: bool,
}

impl Default for UserGeneratorConfig {
    fn default() -> Self {
        Self {
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
            generate_realistic_names: true,
        }
    }
}

/// Generator for user pools with realistic names.
pub struct UserGenerator {
    rng: ChaCha8Rng,
    seed: u64,
    name_generator: MultiCultureNameGenerator,
    config: UserGeneratorConfig,
    user_counter: usize,
}

impl UserGenerator {
    /// Create a new user generator.
    pub fn new(seed: u64) -> Self {
        Self::with_config(seed, UserGeneratorConfig::default())
    }

    /// Create a new user generator with custom configuration.
    pub fn with_config(seed: u64, config: UserGeneratorConfig) -> Self {
        let mut name_gen =
            MultiCultureNameGenerator::with_distribution(config.culture_distribution.clone());
        name_gen.set_email_domain(&config.email_domain);

        Self {
            rng: seeded_rng(seed, 0),
            seed,
            name_generator: name_gen,
            config,
            user_counter: 0,
        }
    }

    /// Create a new user generator with a pre-built name generator.
    ///
    /// This is useful when the name generator has been constructed from a
    /// [`CountryPack`] and the caller wants full control over its pools
    /// and distribution weights.
    pub fn with_name_generator(
        seed: u64,
        config: UserGeneratorConfig,
        name_gen: MultiCultureNameGenerator,
    ) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            seed,
            name_generator: name_gen,
            config,
            user_counter: 0,
        }
    }

    /// Generate a single user for a specific persona.
    pub fn generate_user(&mut self, persona: UserPersona, department: Option<&Department>) -> User {
        self.user_counter += 1;

        let (user_id, display_name, email) = if self.config.generate_realistic_names {
            let name = self.name_generator.generate_name(&mut self.rng);
            let user_id = name.to_user_id(self.user_counter);
            let email = self.name_generator.generate_email(&name);
            (user_id, name.display_name, Some(email))
        } else {
            let user_id = User::generate_username(persona, self.user_counter);
            let display_name = format!("{} {}", persona, self.user_counter);
            (user_id, display_name, None)
        };

        let working_hours = if persona.is_human() {
            self.select_working_hours_pattern()
        } else {
            WorkingHoursPattern::batch_processing()
        };

        let mut user = User::new(user_id, display_name, persona);
        user.email = email;
        user.working_hours = working_hours;

        if let Some(dept) = department {
            user.department = Some(dept.name.clone());
            user.cost_centers = vec![dept.cost_center.clone()];
        }

        user
    }

    /// Generate a user with a specific name (for deterministic generation).
    pub fn generate_user_with_name(
        &mut self,
        name: PersonName,
        persona: UserPersona,
        department: Option<&Department>,
    ) -> User {
        self.user_counter += 1;

        let user_id = name.to_user_id(self.user_counter);
        let email = self.name_generator.generate_email(&name);

        let working_hours = if persona.is_human() {
            self.select_working_hours_pattern()
        } else {
            WorkingHoursPattern::batch_processing()
        };

        let mut user = User::new(user_id, name.display_name, persona);
        user.email = Some(email);
        user.working_hours = working_hours;

        if let Some(dept) = department {
            user.department = Some(dept.name.clone());
            user.cost_centers = vec![dept.cost_center.clone()];
        }

        user
    }

    /// Select a working hours pattern based on random distribution.
    fn select_working_hours_pattern(&mut self) -> WorkingHoursPattern {
        let roll: f64 = self.rng.random();
        if roll < 0.5 {
            WorkingHoursPattern::us_standard()
        } else if roll < 0.75 {
            WorkingHoursPattern::european()
        } else {
            WorkingHoursPattern::asian()
        }
    }

    /// Generate a user pool from an organization structure.
    pub fn generate_from_organization(
        &mut self,
        org: &OrganizationStructure,
        company_codes: &[String],
    ) -> UserPool {
        let mut pool = UserPool::new();

        for dept in &org.departments {
            self.generate_department_users(&mut pool, dept, company_codes);
        }

        pool
    }

    /// Generate users for a specific department.
    fn generate_department_users(
        &mut self,
        pool: &mut UserPool,
        dept: &Department,
        company_codes: &[String],
    ) {
        let headcount = &dept.standard_headcount;

        // Generate junior accountants
        for _ in 0..headcount.junior_accountant {
            let mut user = self.generate_user(UserPersona::JuniorAccountant, Some(dept));
            user.company_codes = company_codes.to_vec();
            pool.add_user(user);
        }

        // Generate senior accountants
        for _ in 0..headcount.senior_accountant {
            let mut user = self.generate_user(UserPersona::SeniorAccountant, Some(dept));
            user.company_codes = company_codes.to_vec();
            pool.add_user(user);
        }

        // Generate controllers
        for _ in 0..headcount.controller {
            let mut user = self.generate_user(UserPersona::Controller, Some(dept));
            user.company_codes = company_codes.to_vec();
            pool.add_user(user);
        }

        // Generate managers
        for _ in 0..headcount.manager {
            let mut user = self.generate_user(UserPersona::Manager, Some(dept));
            user.company_codes = company_codes.to_vec();
            pool.add_user(user);
        }

        // Generate executives
        for _ in 0..headcount.executive {
            let mut user = self.generate_user(UserPersona::Executive, Some(dept));
            user.company_codes = company_codes.to_vec();
            pool.add_user(user);
        }

        // Generate automated systems
        for _ in 0..headcount.automated_system {
            let mut user = self.generate_user(UserPersona::AutomatedSystem, Some(dept));
            user.company_codes = company_codes.to_vec();
            pool.add_user(user);
        }
    }

    /// Generate a simple user pool with specified counts per persona.
    pub fn generate_simple_pool(
        &mut self,
        junior_count: usize,
        senior_count: usize,
        controller_count: usize,
        manager_count: usize,
        automated_count: usize,
        company_codes: &[String],
    ) -> UserPool {
        let mut pool = UserPool::new();

        for _ in 0..junior_count {
            let mut user = self.generate_user(UserPersona::JuniorAccountant, None);
            user.company_codes = company_codes.to_vec();
            pool.add_user(user);
        }

        for _ in 0..senior_count {
            let mut user = self.generate_user(UserPersona::SeniorAccountant, None);
            user.company_codes = company_codes.to_vec();
            pool.add_user(user);
        }

        for _ in 0..controller_count {
            let mut user = self.generate_user(UserPersona::Controller, None);
            user.company_codes = company_codes.to_vec();
            pool.add_user(user);
        }

        for _ in 0..manager_count {
            let mut user = self.generate_user(UserPersona::Manager, None);
            user.company_codes = company_codes.to_vec();
            pool.add_user(user);
        }

        for _ in 0..automated_count {
            let mut user = self.generate_user(UserPersona::AutomatedSystem, None);
            user.company_codes = company_codes.to_vec();
            pool.add_user(user);
        }

        pool
    }

    /// Generate a standard user pool (equivalent to UserPool::generate_standard but with realistic names).
    pub fn generate_standard(&mut self, company_codes: &[String]) -> UserPool {
        self.generate_simple_pool(10, 5, 2, 3, 20, company_codes)
    }

    /// Reset the generator.
    pub fn reset(&mut self) {
        self.rng = seeded_rng(self.seed, 0);
        self.user_counter = 0;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_user_generation() {
        let mut gen = UserGenerator::new(42);

        let user = gen.generate_user(UserPersona::SeniorAccountant, None);

        assert!(!user.user_id.is_empty());
        assert!(!user.display_name.is_empty());
        assert!(user.email.is_some());
        assert_eq!(user.persona, UserPersona::SeniorAccountant);
    }

    #[test]
    fn test_generate_standard_pool() {
        let mut gen = UserGenerator::new(42);
        let pool = gen.generate_standard(&["1000".to_string()]);

        assert_eq!(pool.users.len(), 40); // 10 + 5 + 2 + 3 + 20

        // Check we have users of each type
        assert!(!pool
            .get_users_by_persona(UserPersona::JuniorAccountant)
            .is_empty());
        assert!(!pool
            .get_users_by_persona(UserPersona::AutomatedSystem)
            .is_empty());
    }

    #[test]
    fn test_generate_from_organization() {
        let mut gen = UserGenerator::new(42);
        let org = OrganizationStructure::standard("1000");
        let pool = gen.generate_from_organization(&org, &["1000".to_string()]);

        // Should have users from all departments
        assert!(pool.users.len() > 20);

        // Users should have departments assigned
        let has_dept_users = pool.users.iter().any(|u| u.department.is_some());
        assert!(has_dept_users);
    }

    #[test]
    fn test_deterministic_generation() {
        let mut gen1 = UserGenerator::new(42);
        let mut gen2 = UserGenerator::new(42);

        let user1 = gen1.generate_user(UserPersona::SeniorAccountant, None);
        let user2 = gen2.generate_user(UserPersona::SeniorAccountant, None);

        assert_eq!(user1.user_id, user2.user_id);
        assert_eq!(user1.display_name, user2.display_name);
    }

    #[test]
    fn test_generic_names() {
        let config = UserGeneratorConfig {
            generate_realistic_names: false,
            ..Default::default()
        };
        let mut gen = UserGenerator::with_config(42, config);

        let user = gen.generate_user(UserPersona::JuniorAccountant, None);

        // Should use the generic format
        assert!(user.user_id.starts_with("JACC"));
    }
}
