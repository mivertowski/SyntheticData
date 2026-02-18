//! Time entry generator for the Hire-to-Retire (H2R) process.
//!
//! Generates daily time entries for employees across business days in a period,
//! including regular hours, overtime, PTO, and sick leave with approval statuses.

use chrono::{Datelike, NaiveDate};
use datasynth_config::schema::TimeAttendanceConfig;
use datasynth_core::models::{TimeApprovalStatus, TimeEntry};
use datasynth_core::utils::seeded_rng;
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use tracing::debug;

/// Default PTO rate (probability that an employee takes PTO on a given business day).
const DEFAULT_PTO_RATE: f64 = 0.03;

/// Default sick leave rate.
const DEFAULT_SICK_RATE: f64 = 0.01;

/// Generates [`TimeEntry`] records for employees across business days in a period.
pub struct TimeEntryGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
}

impl TimeEntryGenerator {
    /// Create a new time entry generator.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::TimeEntry),
        }
    }

    /// Generate time entries for a set of employees over a date range.
    ///
    /// # Arguments
    ///
    /// * `employee_ids` - Slice of employee identifiers
    /// * `period_start` - Start of the period (inclusive)
    /// * `period_end` - End of the period (inclusive)
    /// * `config` - Time and attendance configuration
    pub fn generate(
        &mut self,
        employee_ids: &[String],
        period_start: NaiveDate,
        period_end: NaiveDate,
        config: &TimeAttendanceConfig,
    ) -> Vec<TimeEntry> {
        debug!(employee_count = employee_ids.len(), %period_start, %period_end, "Generating time entries");
        let mut entries = Vec::new();
        let business_days = self.collect_business_days(period_start, period_end);

        let overtime_rate = config.overtime_rate;

        for employee_id in employee_ids {
            for &day in &business_days {
                let entry = self.generate_entry(employee_id, day, overtime_rate);
                entries.push(entry);
            }
        }

        entries
    }

    /// Collect all business days (Mon-Fri) within the given date range.
    fn collect_business_days(&self, start: NaiveDate, end: NaiveDate) -> Vec<NaiveDate> {
        let mut days = Vec::new();
        let mut current = start;
        while current <= end {
            let weekday = current.weekday();
            if weekday != chrono::Weekday::Sat && weekday != chrono::Weekday::Sun {
                days.push(current);
            }
            current += chrono::Duration::days(1);
        }
        days
    }

    /// Generate a single time entry for an employee on a given day.
    fn generate_entry(
        &mut self,
        employee_id: &str,
        date: NaiveDate,
        overtime_rate: f64,
    ) -> TimeEntry {
        let entry_id = self.uuid_factory.next().to_string();

        // Determine entry type: PTO, sick, or regular working day
        let pto_roll: f64 = self.rng.gen();
        let sick_roll: f64 = self.rng.gen();

        let (hours_regular, hours_overtime, hours_pto, hours_sick) = if pto_roll < DEFAULT_PTO_RATE
        {
            // PTO day: 8 hours PTO, no work
            (0.0, 0.0, 8.0, 0.0)
        } else if sick_roll < DEFAULT_SICK_RATE {
            // Sick day: 8 hours sick leave, no work
            (0.0, 0.0, 0.0, 8.0)
        } else {
            // Regular working day
            let regular = 8.0;
            let overtime = if self.rng.gen_bool(overtime_rate) {
                self.rng.gen_range(1.0..=4.0)
            } else {
                0.0
            };
            (regular, overtime, 0.0, 0.0)
        };

        // Project assignment: ~60% of entries have a project
        let project_id = if self.rng.gen_bool(0.60) {
            Some(format!("PROJ-{:04}", self.rng.gen_range(1..=50)))
        } else {
            None
        };

        // Cost center: ~70% of entries have a cost center
        let cost_center = if self.rng.gen_bool(0.70) {
            Some(format!("CC-{:03}", self.rng.gen_range(100..=500)))
        } else {
            None
        };

        // Description based on entry type
        let description = if hours_pto > 0.0 {
            Some("Paid time off".to_string())
        } else if hours_sick > 0.0 {
            Some("Sick leave".to_string())
        } else if hours_overtime > 0.0 {
            Some("Regular work + overtime".to_string())
        } else {
            None
        };

        // Approval status: 90% approved, 5% pending, 5% rejected
        let status_roll: f64 = self.rng.gen();
        let approval_status = if status_roll < 0.90 {
            TimeApprovalStatus::Approved
        } else if status_roll < 0.95 {
            TimeApprovalStatus::Pending
        } else {
            TimeApprovalStatus::Rejected
        };

        let approved_by = if approval_status == TimeApprovalStatus::Approved {
            Some(format!("MGR-{:04}", self.rng.gen_range(1..=100)))
        } else {
            None
        };

        let submitted_at =
            if approval_status != TimeApprovalStatus::Pending || self.rng.gen_bool(0.5) {
                // Most entries are submitted on the day or the next day
                let lag = self.rng.gen_range(0..=2);
                Some(date + chrono::Duration::days(lag))
            } else {
                None
            };

        TimeEntry {
            entry_id,
            employee_id: employee_id.to_string(),
            date,
            hours_regular,
            hours_overtime,
            hours_pto,
            hours_sick,
            project_id,
            cost_center,
            description,
            approval_status,
            approved_by,
            submitted_at,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn test_employee_ids() -> Vec<String> {
        vec![
            "EMP-001".to_string(),
            "EMP-002".to_string(),
            "EMP-003".to_string(),
        ]
    }

    #[test]
    fn test_basic_time_entry_generation() {
        let mut gen = TimeEntryGenerator::new(42);
        let employees = test_employee_ids();
        let period_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let period_end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let config = TimeAttendanceConfig::default();

        let entries = gen.generate(&employees, period_start, period_end, &config);

        // January 2024 has 23 business days, 3 employees => 69 entries
        assert!(!entries.is_empty());
        assert_eq!(entries.len(), 23 * 3);

        for entry in &entries {
            assert!(!entry.entry_id.is_empty());
            assert!(!entry.employee_id.is_empty());
            // Each day should have some hours
            let total =
                entry.hours_regular + entry.hours_overtime + entry.hours_pto + entry.hours_sick;
            assert!(total > 0.0, "Entry should have some hours recorded");
            // No weekend entries
            let weekday = entry.date.weekday();
            assert!(
                weekday != chrono::Weekday::Sat && weekday != chrono::Weekday::Sun,
                "Should not generate weekend entries"
            );
        }
    }

    #[test]
    fn test_deterministic_time_entries() {
        let employees = test_employee_ids();
        let period_start = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let period_end = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();
        let config = TimeAttendanceConfig::default();

        let mut gen1 = TimeEntryGenerator::new(42);
        let entries1 = gen1.generate(&employees, period_start, period_end, &config);

        let mut gen2 = TimeEntryGenerator::new(42);
        let entries2 = gen2.generate(&employees, period_start, period_end, &config);

        assert_eq!(entries1.len(), entries2.len());
        for (a, b) in entries1.iter().zip(entries2.iter()) {
            assert_eq!(a.entry_id, b.entry_id);
            assert_eq!(a.employee_id, b.employee_id);
            assert_eq!(a.date, b.date);
            assert_eq!(a.hours_regular, b.hours_regular);
            assert_eq!(a.hours_overtime, b.hours_overtime);
            assert_eq!(a.approval_status, b.approval_status);
        }
    }

    #[test]
    fn test_approval_status_distribution() {
        let mut gen = TimeEntryGenerator::new(99);
        // Use more employees for a larger sample
        let employees: Vec<String> = (1..=20).map(|i| format!("EMP-{:04}", i)).collect();
        let period_start = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let period_end = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();
        let config = TimeAttendanceConfig::default();

        let entries = gen.generate(&employees, period_start, period_end, &config);

        let approved_count = entries
            .iter()
            .filter(|e| e.approval_status == TimeApprovalStatus::Approved)
            .count();
        let pending_count = entries
            .iter()
            .filter(|e| e.approval_status == TimeApprovalStatus::Pending)
            .count();
        let rejected_count = entries
            .iter()
            .filter(|e| e.approval_status == TimeApprovalStatus::Rejected)
            .count();

        let total = entries.len() as f64;
        // Approved should be dominant (~90%)
        assert!(
            (approved_count as f64 / total) > 0.80,
            "Expected >80% approved, got {:.1}%",
            approved_count as f64 / total * 100.0
        );
        // Pending and rejected should exist
        assert!(pending_count > 0, "Expected at least some pending entries");
        assert!(
            rejected_count > 0,
            "Expected at least some rejected entries"
        );
    }
}
