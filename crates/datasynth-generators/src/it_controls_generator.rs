//! IT Controls generator — access logs and change management records.
//!
//! Generates realistic IT access logs and change management records for
//! ITGC (IT General Controls) testing, supporting ISA 315, ISA 330,
//! and SOX 404 audit procedures.

use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime};
use datasynth_core::models::{AccessLog, ChangeManagementRecord};
use datasynth_core::utils::seeded_rng;
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Actions and their approximate cumulative weights for selection.
const ACCESS_ACTIONS: &[(&str, f64)] = &[
    ("login", 0.60),
    ("logout", 0.85),
    ("failed_login", 0.90),
    ("privilege_change", 0.95),
    ("data_export", 1.00),
];

/// Change types and their approximate cumulative weights.
const CHANGE_TYPES: &[(&str, f64)] = &[
    ("config_change", 0.30),
    ("code_deployment", 0.55),
    ("patch", 0.75),
    ("access_change", 0.90),
    ("emergency_fix", 1.00),
];

/// Description templates per change type.
const CONFIG_CHANGE_DESCRIPTIONS: &[&str] = &[
    "Updated firewall rules for DMZ",
    "Modified database connection pool settings",
    "Changed application timeout parameters",
    "Updated email relay configuration",
    "Modified backup retention policy",
    "Adjusted logging verbosity levels",
    "Changed SSL/TLS certificate configuration",
    "Updated LDAP authentication settings",
];

const CODE_DEPLOYMENT_DESCRIPTIONS: &[&str] = &[
    "Deployed financial reporting module v2.3",
    "Released hotfix for invoice processing",
    "Deployed updated reconciliation engine",
    "Released new user interface components",
    "Deployed API gateway update",
    "Released batch processing optimization",
    "Deployed security patch for web application",
    "Released data migration scripts",
];

const PATCH_DESCRIPTIONS: &[&str] = &[
    "Applied OS security patch KB-2024-001",
    "Updated database server to latest patch level",
    "Applied middleware security update",
    "Patched web server vulnerability CVE-2024-1234",
    "Applied ERP kernel update",
    "Updated antivirus definitions",
    "Applied network firmware update",
    "Patched authentication module vulnerability",
];

const ACCESS_CHANGE_DESCRIPTIONS: &[&str] = &[
    "Granted read access to financial reports",
    "Revoked terminated employee access",
    "Modified role assignment for department transfer",
    "Added privileged access for system maintenance",
    "Updated service account permissions",
    "Removed legacy admin access rights",
    "Granted vendor portal access",
    "Modified segregation of duties profile",
];

const EMERGENCY_FIX_DESCRIPTIONS: &[&str] = &[
    "Emergency fix for production outage",
    "Critical security vulnerability remediation",
    "Emergency database recovery procedure",
    "Urgent fix for data corruption issue",
    "Emergency patch for authentication bypass",
    "Critical fix for payment processing failure",
    "Emergency rollback of failed deployment",
    "Urgent fix for regulatory reporting deadline",
];

const TEST_EVIDENCE_TEMPLATES: &[&str] = &[
    "UAT sign-off document ref: UAT-2024-{:04}",
    "Regression test suite passed: TS-{:04}",
    "Integration test report: ITR-{:04}",
    "Performance test results: PTR-{:04}",
    "Security scan report: SEC-{:04}",
    "User acceptance testing completed: UAT-{:04}",
];

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Generates [`AccessLog`] and [`ChangeManagementRecord`] entries for ITGC testing.
pub struct ItControlsGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
}

impl ItControlsGenerator {
    /// Create a new generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::ItControls),
        }
    }

    /// Generate IT access logs for the given employees and systems.
    ///
    /// Produces 10-30 log entries per employee per month with realistic
    /// distributions:
    /// - Actions: login (60%), logout (25%), failed_login (5%),
    ///   privilege_change (5%), data_export (5%)
    /// - 80% of events during business hours (8am-6pm)
    /// - Failed logins clustered in brute-force patterns (3-5 consecutive)
    /// - IP addresses from internal 10.0.0.0/8 range
    pub fn generate_access_logs(
        &mut self,
        employee_ids: &[(String, String)], // (id, name) pairs
        systems: &[String],
        start_date: NaiveDate,
        period_months: u32,
    ) -> Vec<AccessLog> {
        if employee_ids.is_empty() || systems.is_empty() {
            return Vec::new();
        }

        let mut logs = Vec::new();

        for month_offset in 0..period_months {
            let year = start_date.year() + (start_date.month0() + month_offset) as i32 / 12;
            let month = (start_date.month0() + month_offset) % 12 + 1;
            let days_in_month = days_in_month(year, month);

            for (user_id, user_name) in employee_ids {
                let log_count = self.rng.random_range(10u32..=30);
                // Assign a consistent primary system and IP for this employee
                let primary_system = &systems[self.rng.random_range(0..systems.len())];
                let ip_address = self.generate_ip();

                // Decide whether this employee gets a failed login cluster this month
                let has_failed_cluster = self.rng.random_bool(0.08);
                let cluster_day = if has_failed_cluster {
                    self.rng.random_range(1..=days_in_month)
                } else {
                    1 // unused
                };

                for i in 0..log_count {
                    let day = self.rng.random_range(1..=days_in_month);
                    let (hour, minute, second) = self.generate_time();

                    let Some(date) = NaiveDate::from_ymd_opt(year, month, day) else {
                        continue;
                    };
                    let Some(time) = NaiveTime::from_hms_opt(hour, minute, second) else {
                        continue;
                    };
                    let timestamp = NaiveDateTime::new(date, time);

                    let (action, success) = self.pick_action();
                    let system = if self.rng.random_bool(0.7) {
                        primary_system.clone()
                    } else {
                        systems[self.rng.random_range(0..systems.len())].clone()
                    };

                    let session_duration = if action == "logout" {
                        Some(self.rng.random_range(5u32..=480))
                    } else {
                        None
                    };

                    logs.push(AccessLog {
                        log_id: self.uuid_factory.next(),
                        timestamp,
                        user_id: user_id.clone(),
                        user_name: user_name.clone(),
                        system,
                        action,
                        success,
                        ip_address: ip_address.clone(),
                        session_duration_minutes: session_duration,
                    });

                    // Insert failed login cluster if applicable
                    if has_failed_cluster && i == 0 {
                        let cluster_size = self.rng.random_range(3u32..=5);
                        let Some(cluster_date) =
                            NaiveDate::from_ymd_opt(year, month, cluster_day)
                        else {
                            continue;
                        };

                        for j in 0..cluster_size {
                            let cluster_minute = self.rng.random_range(0u32..=2);
                            let cluster_second = self.rng.random_range(0u32..=59);
                            let cluster_hour = self.rng.random_range(1u32..=5); // off-hours
                            let Some(ct) = NaiveTime::from_hms_opt(
                                cluster_hour,
                                cluster_minute + j,
                                cluster_second,
                            ) else {
                                continue;
                            };

                            logs.push(AccessLog {
                                log_id: self.uuid_factory.next(),
                                timestamp: NaiveDateTime::new(cluster_date, ct),
                                user_id: user_id.clone(),
                                user_name: user_name.clone(),
                                system: primary_system.clone(),
                                action: "failed_login".to_string(),
                                success: false,
                                ip_address: self.generate_ip(), // different IP = external attacker
                                session_duration_minutes: None,
                            });
                        }
                    }
                }
            }
        }

        // Sort chronologically
        logs.sort_by_key(|l| l.timestamp);
        logs
    }

    /// Generate change management records for the given systems and period.
    ///
    /// Produces 5-15 changes per month with realistic distributions:
    /// - Types: config_change (30%), code_deployment (25%), patch (20%),
    ///   access_change (15%), emergency_fix (10%)
    /// - 90% have approval (10% gap = ITGC finding)
    /// - 85% tested before deployment
    /// - 95% have rollback plans
    /// - Emergency fixes: lower approval/testing rates (realistic weakness)
    pub fn generate_change_records(
        &mut self,
        employee_ids: &[(String, String)],
        systems: &[String],
        start_date: NaiveDate,
        period_months: u32,
    ) -> Vec<ChangeManagementRecord> {
        if employee_ids.is_empty() || systems.is_empty() {
            return Vec::new();
        }

        let mut records = Vec::new();

        for month_offset in 0..period_months {
            let year = start_date.year() + (start_date.month0() + month_offset) as i32 / 12;
            let month = (start_date.month0() + month_offset) % 12 + 1;
            let days_in_month = days_in_month(year, month);

            let changes_this_month = self.rng.random_range(5u32..=15);

            for _ in 0..changes_this_month {
                let change_type = self.pick_change_type();
                let system = &systems[self.rng.random_range(0..systems.len())];
                let description = self.pick_description(&change_type);

                let requester_idx = self.rng.random_range(0..employee_ids.len());
                let requested_by = employee_ids[requester_idx].1.clone();

                // Pick implementer (different from requester when possible)
                let implementer_idx = if employee_ids.len() > 1 {
                    let mut idx = self.rng.random_range(0..employee_ids.len());
                    if idx == requester_idx {
                        idx = (idx + 1) % employee_ids.len();
                    }
                    idx
                } else {
                    0
                };
                let implemented_by = employee_ids[implementer_idx].1.clone();

                // Approval: emergency fixes have ~30% approval, others ~95%
                let is_emergency = change_type == "emergency_fix";
                let has_approval = if is_emergency {
                    self.rng.random_bool(0.30)
                } else {
                    self.rng.random_bool(0.95)
                };

                let approved_by = if has_approval {
                    // Pick approver (different from requester and implementer when possible)
                    let mut approver_idx = self.rng.random_range(0..employee_ids.len());
                    if employee_ids.len() > 2 {
                        while approver_idx == requester_idx || approver_idx == implementer_idx {
                            approver_idx = self.rng.random_range(0..employee_ids.len());
                        }
                    }
                    Some(employee_ids[approver_idx].1.clone())
                } else {
                    None
                };

                // Testing: emergency fixes have ~20% testing, others ~90%
                let tested = if is_emergency {
                    self.rng.random_bool(0.20)
                } else {
                    self.rng.random_bool(0.90)
                };

                let test_evidence = if tested {
                    let evidence_num = self.rng.random_range(1u32..=9999);
                    let template =
                        TEST_EVIDENCE_TEMPLATES[self.rng.random_range(0..TEST_EVIDENCE_TEMPLATES.len())];
                    Some(template.replace("{:04}", &format!("{:04}", evidence_num)))
                } else {
                    None
                };

                // Rollback plan: emergency fixes have ~50%, others ~98%
                let rollback_plan = if is_emergency {
                    self.rng.random_bool(0.50)
                } else {
                    self.rng.random_bool(0.98)
                };

                // Request date: random day in the month
                let request_day = self.rng.random_range(1..=days_in_month);
                let request_hour = self.rng.random_range(8u32..=17);
                let request_minute = self.rng.random_range(0u32..=59);
                let Some(request_date_d) = NaiveDate::from_ymd_opt(year, month, request_day) else {
                    continue;
                };
                let Some(request_time) = NaiveTime::from_hms_opt(request_hour, request_minute, 0)
                else {
                    continue;
                };
                let request_date = NaiveDateTime::new(request_date_d, request_time);

                // Implementation date: 0-14 days after request
                // Emergency fixes: 0-1 days; others: 1-14 days
                let impl_lag_days = if is_emergency {
                    self.rng.random_range(0i64..=1)
                } else {
                    self.rng.random_range(1i64..=14)
                };
                let impl_date_d = request_date_d + chrono::Duration::days(impl_lag_days);
                let impl_hour = self.rng.random_range(8u32..=22);
                let impl_minute = self.rng.random_range(0u32..=59);
                let Some(impl_time) = NaiveTime::from_hms_opt(impl_hour, impl_minute, 0) else {
                    continue;
                };
                let implementation_date = NaiveDateTime::new(impl_date_d, impl_time);

                records.push(ChangeManagementRecord {
                    change_id: self.uuid_factory.next(),
                    system: system.clone(),
                    change_type,
                    description,
                    requested_by,
                    approved_by,
                    implemented_by,
                    request_date,
                    implementation_date,
                    tested,
                    test_evidence,
                    rollback_plan,
                });
            }
        }

        // Sort by request date
        records.sort_by_key(|r| r.request_date);
        records
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Pick an action based on weighted distribution.
    fn pick_action(&mut self) -> (String, bool) {
        let r: f64 = self.rng.random_range(0.0..1.0);
        for &(action, threshold) in ACCESS_ACTIONS {
            if r < threshold {
                let success = action != "failed_login";
                return (action.to_string(), success);
            }
        }
        ("login".to_string(), true)
    }

    /// Pick a change type based on weighted distribution.
    fn pick_change_type(&mut self) -> String {
        let r: f64 = self.rng.random_range(0.0..1.0);
        for &(ct, threshold) in CHANGE_TYPES {
            if r < threshold {
                return ct.to_string();
            }
        }
        "config_change".to_string()
    }

    /// Pick a description template for a given change type.
    fn pick_description(&mut self, change_type: &str) -> String {
        let pool = match change_type {
            "config_change" => CONFIG_CHANGE_DESCRIPTIONS,
            "code_deployment" => CODE_DEPLOYMENT_DESCRIPTIONS,
            "patch" => PATCH_DESCRIPTIONS,
            "access_change" => ACCESS_CHANGE_DESCRIPTIONS,
            "emergency_fix" => EMERGENCY_FIX_DESCRIPTIONS,
            _ => CONFIG_CHANGE_DESCRIPTIONS,
        };
        pool.choose(&mut self.rng)
            .map(|s| s.to_string())
            .unwrap_or_else(|| "System change".to_string())
    }

    /// Generate a timestamp hour/minute/second with 80% business hours bias.
    fn generate_time(&mut self) -> (u32, u32, u32) {
        let is_business_hours = self.rng.random_bool(0.80);
        let hour = if is_business_hours {
            self.rng.random_range(8u32..=17)
        } else {
            // Off-hours: 0-7 or 18-23
            if self.rng.random_bool(0.5) {
                self.rng.random_range(0u32..=7)
            } else {
                self.rng.random_range(18u32..=23)
            }
        };
        let minute = self.rng.random_range(0u32..=59);
        let second = self.rng.random_range(0u32..=59);
        (hour, minute, second)
    }

    /// Generate an IP address in the 10.0.0.0/8 range.
    fn generate_ip(&mut self) -> String {
        format!(
            "10.{}.{}.{}",
            self.rng.random_range(0u8..=255),
            self.rng.random_range(0u8..=255),
            self.rng.random_range(1u8..=254),
        )
    }
}

/// Return the number of days in the given month.
fn days_in_month(year: i32, month: u32) -> u32 {
    // Get the first day of the next month, then subtract one day
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .and_then(|d| d.pred_opt())
        .map(|d| d.day())
        .unwrap_or(28)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::Timelike;

    fn sample_employees() -> Vec<(String, String)> {
        (1..=10)
            .map(|i| (format!("EMP-{:04}", i), format!("Employee {}", i)))
            .collect()
    }

    fn sample_systems() -> Vec<String> {
        vec![
            "SAP-FI".to_string(),
            "Active Directory".to_string(),
            "Oracle-HR".to_string(),
            "ServiceNow".to_string(),
        ]
    }

    #[test]
    fn test_access_logs_generated() {
        let mut gen = ItControlsGenerator::new(42);
        let logs = gen.generate_access_logs(
            &sample_employees(),
            &sample_systems(),
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            3,
        );
        assert!(!logs.is_empty(), "should produce access logs");
        for log in &logs {
            assert!(!log.user_id.is_empty());
            assert!(!log.user_name.is_empty());
            assert!(!log.system.is_empty());
            assert!(!log.action.is_empty());
            assert!(!log.ip_address.is_empty());
            assert!(log.ip_address.starts_with("10."));
        }
    }

    #[test]
    fn test_access_log_business_hours() {
        let mut gen = ItControlsGenerator::new(42);
        let logs = gen.generate_access_logs(
            &sample_employees(),
            &sample_systems(),
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            6,
        );
        let total = logs.len() as f64;
        let business_hours_count = logs
            .iter()
            .filter(|l| {
                let hour = l.timestamp.time().hour();
                (8..=17).contains(&hour)
            })
            .count() as f64;
        let ratio = business_hours_count / total;
        assert!(
            ratio > 0.70,
            "expected >70% business hours, got {:.1}%",
            ratio * 100.0
        );
    }

    #[test]
    fn test_failed_login_rate() {
        let mut gen = ItControlsGenerator::new(42);
        let logs = gen.generate_access_logs(
            &sample_employees(),
            &sample_systems(),
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            6,
        );
        let total = logs.len() as f64;
        let failed = logs.iter().filter(|l| l.action == "failed_login").count() as f64;
        let rate = failed / total;
        assert!(
            rate >= 0.02 && rate <= 0.15,
            "expected 2-15% failed login rate, got {:.1}%",
            rate * 100.0
        );
    }

    #[test]
    fn test_access_log_references_employees() {
        let employees = sample_employees();
        let employee_ids: std::collections::HashSet<&str> =
            employees.iter().map(|(id, _)| id.as_str()).collect();

        let mut gen = ItControlsGenerator::new(42);
        let logs = gen.generate_access_logs(
            &employees,
            &sample_systems(),
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            3,
        );

        for log in &logs {
            assert!(
                employee_ids.contains(log.user_id.as_str()),
                "user_id {} should come from employee input",
                log.user_id
            );
        }
    }

    #[test]
    fn test_change_records_generated() {
        let mut gen = ItControlsGenerator::new(42);
        let records = gen.generate_change_records(
            &sample_employees(),
            &sample_systems(),
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            3,
        );
        assert!(!records.is_empty(), "should produce change records");
        for r in &records {
            assert!(!r.system.is_empty());
            assert!(!r.change_type.is_empty());
            assert!(!r.description.is_empty());
            assert!(!r.requested_by.is_empty());
            assert!(!r.implemented_by.is_empty());
        }
    }

    #[test]
    fn test_change_approval_rate() {
        let mut gen = ItControlsGenerator::new(42);
        let records = gen.generate_change_records(
            &sample_employees(),
            &sample_systems(),
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            12,
        );
        let total = records.len() as f64;
        let approved = records.iter().filter(|r| r.approved_by.is_some()).count() as f64;
        let rate = approved / total;
        // Overall rate should be ~85-95% (mix of emergency and normal)
        assert!(
            rate > 0.75 && rate < 0.99,
            "expected ~85-95% approval rate, got {:.1}%",
            rate * 100.0
        );
    }

    #[test]
    fn test_emergency_fixes_unapproved() {
        let mut gen = ItControlsGenerator::new(42);
        let records = gen.generate_change_records(
            &sample_employees(),
            &sample_systems(),
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            24,
        );

        let emergency: Vec<_> = records
            .iter()
            .filter(|r| r.change_type == "emergency_fix")
            .collect();
        let non_emergency: Vec<_> = records
            .iter()
            .filter(|r| r.change_type != "emergency_fix")
            .collect();

        if !emergency.is_empty() && !non_emergency.is_empty() {
            let emergency_approval_rate =
                emergency.iter().filter(|r| r.approved_by.is_some()).count() as f64
                    / emergency.len() as f64;
            let non_emergency_approval_rate =
                non_emergency
                    .iter()
                    .filter(|r| r.approved_by.is_some())
                    .count() as f64
                    / non_emergency.len() as f64;

            assert!(
                emergency_approval_rate < non_emergency_approval_rate,
                "emergency fixes ({:.0}%) should have lower approval rate than normal changes ({:.0}%)",
                emergency_approval_rate * 100.0,
                non_emergency_approval_rate * 100.0
            );
        }
    }

    #[test]
    fn test_change_dates_ordered() {
        let mut gen = ItControlsGenerator::new(42);
        let records = gen.generate_change_records(
            &sample_employees(),
            &sample_systems(),
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            6,
        );

        for r in &records {
            // implementation_date should be on or after request_date (comparing date portion)
            assert!(
                r.implementation_date.date() >= r.request_date.date(),
                "implementation date {} should be >= request date {} for change {}",
                r.implementation_date,
                r.request_date,
                r.change_id
            );
        }
    }
}
