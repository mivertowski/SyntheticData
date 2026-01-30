//! Realistic corporate user ID generation.
//!
//! Generates user IDs in various corporate patterns including standard
//! employee IDs, system accounts, and service accounts.

use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// User ID pattern types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum UserIdPattern {
    /// First initial + last name + disambiguator (e.g., JSMITH001)
    #[default]
    InitialLastName,
    /// First name + last name with dot (e.g., john.smith)
    DotSeparated,
    /// First name + underscore + last name (e.g., john_smith)
    UnderscoreSeparated,
    /// Last name + first initial (e.g., smithj)
    LastNameInitial,
    /// Employee number format (e.g., E00012345)
    EmployeeNumber,
    /// System account format (e.g., SVC_BATCH)
    SystemAccount,
    /// Admin account format (e.g., admin_gl)
    AdminAccount,
    /// Interface account format (e.g., INT_SAP)
    InterfaceAccount,
}

/// User ID generator with multiple pattern support.
#[derive(Debug, Clone)]
pub struct UserIdGenerator {
    default_pattern: UserIdPattern,
    system_prefixes: Vec<&'static str>,
    admin_prefixes: Vec<&'static str>,
    interface_prefixes: Vec<&'static str>,
    system_suffixes: Vec<&'static str>,
}

impl Default for UserIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl UserIdGenerator {
    /// Create a new user ID generator.
    pub fn new() -> Self {
        Self {
            default_pattern: UserIdPattern::InitialLastName,
            system_prefixes: vec!["SVC_", "SYS_", "BATCH_", "AUTO_", "SCHED_"],
            admin_prefixes: vec!["admin_", "ADMIN_", "adm_", "root_"],
            interface_prefixes: vec!["INT_", "IF_", "INTF_", "API_", "EDI_"],
            system_suffixes: vec![
                "BATCH",
                "PROCESS",
                "RECON",
                "IMPORT",
                "EXPORT",
                "SYNC",
                "SCHEDULER",
                "MONITOR",
                "BACKUP",
                "ARCHIVE",
                "CLEANUP",
                "POSTING",
                "INTERFACE",
            ],
        }
    }

    /// Generate a user ID using the default pattern.
    pub fn generate(
        &self,
        first_name: &str,
        last_name: &str,
        index: usize,
        rng: &mut impl Rng,
    ) -> String {
        self.generate_with_pattern(first_name, last_name, index, self.default_pattern, rng)
    }

    /// Generate a user ID with a specific pattern.
    pub fn generate_with_pattern(
        &self,
        first_name: &str,
        last_name: &str,
        index: usize,
        pattern: UserIdPattern,
        rng: &mut impl Rng,
    ) -> String {
        match pattern {
            UserIdPattern::InitialLastName => self.initial_last_name(first_name, last_name, index),
            UserIdPattern::DotSeparated => self.dot_separated(first_name, last_name, index),
            UserIdPattern::UnderscoreSeparated => {
                self.underscore_separated(first_name, last_name, index)
            }
            UserIdPattern::LastNameInitial => self.last_name_initial(first_name, last_name, index),
            UserIdPattern::EmployeeNumber => self.employee_number(index),
            UserIdPattern::SystemAccount => self.system_account(rng),
            UserIdPattern::AdminAccount => self.admin_account(rng),
            UserIdPattern::InterfaceAccount => self.interface_account(rng),
        }
    }

    /// Generate a random pattern user ID.
    pub fn generate_random_pattern(
        &self,
        first_name: &str,
        last_name: &str,
        index: usize,
        rng: &mut impl Rng,
    ) -> String {
        let pattern = self.select_pattern(rng);
        self.generate_with_pattern(first_name, last_name, index, pattern, rng)
    }

    /// Generate a system account ID.
    pub fn generate_system_account(&self, rng: &mut impl Rng) -> String {
        self.system_account(rng)
    }

    /// Generate an admin account ID.
    pub fn generate_admin_account(&self, rng: &mut impl Rng) -> String {
        self.admin_account(rng)
    }

    /// Generate an interface account ID.
    pub fn generate_interface_account(&self, system_name: &str) -> String {
        format!("INT_{}", system_name.to_uppercase())
    }

    fn select_pattern(&self, rng: &mut impl Rng) -> UserIdPattern {
        let roll: f64 = rng.gen();
        if roll < 0.40 {
            UserIdPattern::InitialLastName
        } else if roll < 0.65 {
            UserIdPattern::DotSeparated
        } else if roll < 0.80 {
            UserIdPattern::LastNameInitial
        } else if roll < 0.90 {
            UserIdPattern::UnderscoreSeparated
        } else {
            UserIdPattern::EmployeeNumber
        }
    }

    fn initial_last_name(&self, first_name: &str, last_name: &str, index: usize) -> String {
        let first_initial = first_name
            .chars()
            .next()
            .unwrap_or('X')
            .to_ascii_uppercase();
        let last_part: String = last_name
            .chars()
            .filter(|c| c.is_ascii_alphabetic())
            .take(7)
            .collect::<String>()
            .to_uppercase();

        if index == 0 {
            format!("{}{}", first_initial, last_part)
        } else {
            format!("{}{}{}", first_initial, last_part, index)
        }
    }

    fn dot_separated(&self, first_name: &str, last_name: &str, index: usize) -> String {
        let first: String = first_name
            .chars()
            .filter(|c| c.is_ascii_alphabetic())
            .collect::<String>()
            .to_lowercase();
        let last: String = last_name
            .chars()
            .filter(|c| c.is_ascii_alphabetic())
            .collect::<String>()
            .to_lowercase();

        if index == 0 {
            format!("{}.{}", first, last)
        } else {
            format!("{}.{}{}", first, last, index)
        }
    }

    fn underscore_separated(&self, first_name: &str, last_name: &str, index: usize) -> String {
        let first: String = first_name
            .chars()
            .filter(|c| c.is_ascii_alphabetic())
            .collect::<String>()
            .to_lowercase();
        let last: String = last_name
            .chars()
            .filter(|c| c.is_ascii_alphabetic())
            .collect::<String>()
            .to_lowercase();

        if index == 0 {
            format!("{}_{}", first, last)
        } else {
            format!("{}_{}{}", first, last, index)
        }
    }

    fn last_name_initial(&self, first_name: &str, last_name: &str, index: usize) -> String {
        let last: String = last_name
            .chars()
            .filter(|c| c.is_ascii_alphabetic())
            .take(8)
            .collect::<String>()
            .to_lowercase();
        let first_initial = first_name
            .chars()
            .next()
            .unwrap_or('x')
            .to_ascii_lowercase();

        if index == 0 {
            format!("{}{}", last, first_initial)
        } else {
            format!("{}{}{}", last, first_initial, index)
        }
    }

    fn employee_number(&self, index: usize) -> String {
        format!("E{:08}", index)
    }

    fn system_account(&self, rng: &mut impl Rng) -> String {
        let prefix = self.system_prefixes.choose(rng).unwrap_or(&"SVC_");
        let suffix = self.system_suffixes.choose(rng).unwrap_or(&"BATCH");
        format!("{}{}", prefix, suffix)
    }

    fn admin_account(&self, rng: &mut impl Rng) -> String {
        let prefix = self.admin_prefixes.choose(rng).unwrap_or(&"admin_");
        let systems = ["gl", "ap", "ar", "fa", "mm", "sd", "fi", "co", "hr", "pm"];
        let system = systems.choose(rng).unwrap_or(&"gl");
        format!("{}{}", prefix, system)
    }

    fn interface_account(&self, rng: &mut impl Rng) -> String {
        let prefix = self.interface_prefixes.choose(rng).unwrap_or(&"INT_");
        let systems = [
            "SAP",
            "ORACLE",
            "SALESFORCE",
            "WORKDAY",
            "NETSUITE",
            "DYNAMICS",
            "SAGE",
            "QUICKBOOKS",
            "CONCUR",
            "COUPA",
            "ARIBA",
            "BLACKLINE",
            "HYPERION",
            "ANAPLAN",
        ];
        let system = systems.choose(rng).unwrap_or(&"SAP");
        format!("{}{}", prefix, system)
    }
}

/// Email generator with corporate patterns.
#[derive(Debug, Clone)]
pub struct EmailGenerator {
    domain: String,
    patterns: Vec<EmailPattern>,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum EmailPattern {
    FirstDotLast,
    FirstInitialLast,
    FirstUnderscoreLast,
    LastDotFirst,
    FirstOnly,
}

impl Default for EmailGenerator {
    fn default() -> Self {
        Self::new("company.com")
    }
}

impl EmailGenerator {
    /// Create a new email generator with the specified domain.
    pub fn new(domain: &str) -> Self {
        Self {
            domain: domain.to_string(),
            patterns: vec![
                EmailPattern::FirstDotLast,
                EmailPattern::FirstDotLast,
                EmailPattern::FirstDotLast, // Weight toward common pattern
                EmailPattern::FirstInitialLast,
                EmailPattern::FirstUnderscoreLast,
            ],
        }
    }

    /// Set the email domain.
    pub fn with_domain(mut self, domain: &str) -> Self {
        self.domain = domain.to_string();
        self
    }

    /// Generate an email address.
    pub fn generate(&self, first_name: &str, last_name: &str, rng: &mut impl Rng) -> String {
        let pattern = self
            .patterns
            .choose(rng)
            .unwrap_or(&EmailPattern::FirstDotLast);
        self.generate_with_pattern(first_name, last_name, *pattern)
    }

    /// Generate an email with a specific pattern.
    fn generate_with_pattern(
        &self,
        first_name: &str,
        last_name: &str,
        pattern: EmailPattern,
    ) -> String {
        let first = self.sanitize_for_email(first_name);
        let last = self.sanitize_for_email(last_name);

        let local_part = match pattern {
            EmailPattern::FirstDotLast => format!("{}.{}", first, last),
            EmailPattern::FirstInitialLast => {
                let initial = first.chars().next().unwrap_or('x');
                format!("{}{}", initial, last)
            }
            EmailPattern::FirstUnderscoreLast => format!("{}_{}", first, last),
            EmailPattern::LastDotFirst => format!("{}.{}", last, first),
            EmailPattern::FirstOnly => first,
        };

        format!("{}@{}", local_part, self.domain)
    }

    /// Generate a generic/functional email address.
    pub fn generate_functional(&self, function: &str) -> String {
        format!("{}@{}", function.to_lowercase(), self.domain)
    }

    fn sanitize_for_email(&self, name: &str) -> String {
        name.chars()
            .filter(|c| c.is_ascii_alphabetic())
            .collect::<String>()
            .to_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_initial_last_name_pattern() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let gen = UserIdGenerator::new();

        let id =
            gen.generate_with_pattern("John", "Smith", 0, UserIdPattern::InitialLastName, &mut rng);
        assert_eq!(id, "JSMITH");

        let id2 =
            gen.generate_with_pattern("John", "Smith", 5, UserIdPattern::InitialLastName, &mut rng);
        assert_eq!(id2, "JSMITH5");
    }

    #[test]
    fn test_dot_separated_pattern() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let gen = UserIdGenerator::new();

        let id =
            gen.generate_with_pattern("John", "Smith", 0, UserIdPattern::DotSeparated, &mut rng);
        assert_eq!(id, "john.smith");

        let id2 =
            gen.generate_with_pattern("John", "Smith", 3, UserIdPattern::DotSeparated, &mut rng);
        assert_eq!(id2, "john.smith3");
    }

    #[test]
    fn test_employee_number_pattern() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let gen = UserIdGenerator::new();

        let id = gen.generate_with_pattern(
            "John",
            "Smith",
            12345,
            UserIdPattern::EmployeeNumber,
            &mut rng,
        );
        assert_eq!(id, "E00012345");
    }

    #[test]
    fn test_system_account() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let gen = UserIdGenerator::new();

        let id = gen.generate_system_account(&mut rng);
        assert!(
            id.starts_with("SVC_")
                || id.starts_with("SYS_")
                || id.starts_with("BATCH_")
                || id.starts_with("AUTO_")
                || id.starts_with("SCHED_")
        );
    }

    #[test]
    fn test_interface_account() {
        let gen = UserIdGenerator::new();
        let id = gen.generate_interface_account("SAP");
        assert_eq!(id, "INT_SAP");
    }

    #[test]
    fn test_email_generation() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let gen = EmailGenerator::new("acme.com");

        let email = gen.generate("John", "Smith", &mut rng);
        assert!(email.ends_with("@acme.com"));
        assert!(email.contains("john") || email.contains("smith") || email.contains("j"));
    }

    #[test]
    fn test_email_with_non_ascii() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let gen = EmailGenerator::new("company.de");

        let email = gen.generate("Jürgen", "Müller", &mut rng);
        assert!(email.ends_with("@company.de"));
        // Non-ASCII should be filtered out
        assert!(!email.contains('ü'));
    }

    #[test]
    fn test_functional_email() {
        let gen = EmailGenerator::new("company.com");
        let email = gen.generate_functional("accounts.payable");
        assert_eq!(email, "accounts.payable@company.com");
    }

    #[test]
    fn test_random_pattern_variety() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let gen = UserIdGenerator::new();

        let mut patterns = std::collections::HashSet::new();
        for i in 0..100 {
            let id = gen.generate_random_pattern("John", "Smith", i, &mut rng);
            patterns.insert(id);
        }

        // Should generate diverse IDs
        assert!(patterns.len() > 10);
    }
}
