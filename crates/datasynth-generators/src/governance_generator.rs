//! Governance generator — board minutes and audit committee meetings.
//!
//! Generates realistic board and audit committee meeting minutes
//! supporting ISA 260 (Communication with Those Charged with Governance)
//! and ISA 315 (risk assessment through governance understanding).

use chrono::NaiveDate;
use datasynth_core::models::BoardMinutes;
use datasynth_core::utils::seeded_rng;
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

// ---------------------------------------------------------------------------
// Template pools
// ---------------------------------------------------------------------------

const KEY_DECISIONS: &[&str] = &[
    "Approved annual operating budget",
    "Reviewed quarterly financial results",
    "Authorized dividend payment",
    "Approved capital expenditure program",
    "Reviewed risk management framework",
    "Appointed external auditor",
    "Reviewed IT security posture",
    "Approved strategic plan update",
    "Authorized share repurchase program",
    "Approved executive compensation structure",
    "Ratified related-party transaction policy",
    "Adopted updated code of conduct",
    "Approved new credit facility",
    "Reviewed succession planning for key executives",
    "Authorized new market expansion",
];

const RISK_DISCUSSIONS: &[&str] = &[
    "Market risk exposure and hedging strategy",
    "Regulatory compliance update",
    "Cybersecurity threat assessment",
    "Going concern considerations",
    "Revenue recognition policy changes",
    "Credit risk and provisioning adequacy",
    "Supply chain disruption risks",
    "Interest rate environment outlook",
    "Foreign exchange exposure",
    "Litigation and contingent liabilities",
    "Climate-related financial risks",
    "Data privacy and GDPR compliance",
];

const AUDIT_COMMITTEE_MATTERS: &[&str] = &[
    "Reviewed external audit plan and scope",
    "Discussed internal audit findings",
    "Evaluated effectiveness of internal controls",
    "Assessed auditor independence",
    "Reviewed whistleblower reports",
    "Discussed accounting policy changes",
    "Evaluated IT general controls",
    "Reviewed fraud risk assessment",
    "Discussed management letter points",
    "Assessed going concern assumptions",
    "Reviewed related-party disclosures",
    "Discussed materiality thresholds",
];

/// Generates [`BoardMinutes`] records for board and audit committee meetings.
pub struct BoardMinutesGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
}

impl BoardMinutesGenerator {
    /// Create a new generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::Governance),
        }
    }

    /// Generate board and audit committee minutes for a given entity and period.
    ///
    /// Produces:
    /// - One regular board meeting per quarter (4 per full year)
    /// - One audit committee meeting per month (up to 12 per full year)
    ///
    /// Attendees are drawn from the supplied `employee_names` pool.
    pub fn generate_board_minutes(
        &mut self,
        _entity_code: &str,
        fiscal_year: i32,
        period_months: u32,
        employee_names: &[String],
    ) -> Vec<BoardMinutes> {
        let mut minutes = Vec::new();
        let months = period_months.min(12);

        // --- Quarterly board meetings ---
        for q in 0..months.div_ceil(3) {
            let month = (q * 3 + 2).clamp(1, 12);
            if let Some(date) = NaiveDate::from_ymd_opt(fiscal_year, month, 15) {
                let attendees = self.pick_attendees(employee_names, 7, 12);
                let decisions = self.pick_items(KEY_DECISIONS, 3, 6);
                let risks = self.pick_items(RISK_DISCUSSIONS, 2, 4);
                let acm = self.pick_items(AUDIT_COMMITTEE_MATTERS, 1, 3);

                minutes.push(BoardMinutes {
                    meeting_id: self.uuid_factory.next(),
                    meeting_date: date,
                    meeting_type: "regular".to_string(),
                    attendees,
                    key_decisions: decisions,
                    risk_discussions: risks,
                    audit_committee_matters: acm,
                });
            }
        }

        // --- Monthly audit committee meetings ---
        for m in 1..=months {
            if let Some(date) = NaiveDate::from_ymd_opt(fiscal_year, m, 20) {
                let attendees = self.pick_attendees(employee_names, 4, 7);
                let decisions = self.pick_items(KEY_DECISIONS, 1, 2);
                let risks = self.pick_items(RISK_DISCUSSIONS, 1, 3);
                let acm = self.pick_items(AUDIT_COMMITTEE_MATTERS, 2, 5);

                minutes.push(BoardMinutes {
                    meeting_id: self.uuid_factory.next(),
                    meeting_date: date,
                    meeting_type: "audit_committee".to_string(),
                    attendees,
                    key_decisions: decisions,
                    risk_discussions: risks,
                    audit_committee_matters: acm,
                });
            }
        }

        // Sort chronologically
        minutes.sort_by_key(|m| m.meeting_date);
        minutes
    }

    /// Randomly pick `min..=max` attendees from the pool.
    fn pick_attendees(&mut self, pool: &[String], min: usize, max: usize) -> Vec<String> {
        if pool.is_empty() {
            return vec!["Board Member".to_string()];
        }
        let count = self.rng.random_range(min..=max).min(pool.len());
        let mut indices: Vec<usize> = (0..pool.len()).collect();
        indices.shuffle(&mut self.rng);
        indices.truncate(count);
        indices.sort_unstable();
        indices.iter().map(|&i| pool[i].clone()).collect()
    }

    /// Randomly pick `min..=max` items from a template pool.
    fn pick_items(&mut self, pool: &[&str], min: usize, max: usize) -> Vec<String> {
        let count = self.rng.random_range(min..=max).min(pool.len());
        let mut indices: Vec<usize> = (0..pool.len()).collect();
        indices.shuffle(&mut self.rng);
        indices.truncate(count);
        indices.sort_unstable();
        indices.iter().map(|&i| pool[i].to_string()).collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::Datelike;

    fn sample_employees() -> Vec<String> {
        (1..=20).map(|i| format!("Employee_{:03}", i)).collect()
    }

    #[test]
    fn test_generates_non_empty_output() {
        let mut gen = BoardMinutesGenerator::new(42);
        let minutes = gen.generate_board_minutes("C001", 2025, 12, &sample_employees());
        assert!(!minutes.is_empty(), "should produce meeting minutes");
    }

    #[test]
    fn test_full_year_meeting_count() {
        let mut gen = BoardMinutesGenerator::new(42);
        let minutes = gen.generate_board_minutes("C001", 2025, 12, &sample_employees());

        let board_count = minutes
            .iter()
            .filter(|m| m.meeting_type == "regular")
            .count();
        let ac_count = minutes
            .iter()
            .filter(|m| m.meeting_type == "audit_committee")
            .count();

        assert_eq!(board_count, 4, "should have 4 quarterly board meetings");
        assert_eq!(
            ac_count, 12,
            "should have 12 monthly audit committee meetings"
        );
    }

    #[test]
    fn test_partial_year() {
        let mut gen = BoardMinutesGenerator::new(42);
        let minutes = gen.generate_board_minutes("C001", 2025, 6, &sample_employees());

        let board_count = minutes
            .iter()
            .filter(|m| m.meeting_type == "regular")
            .count();
        let ac_count = minutes
            .iter()
            .filter(|m| m.meeting_type == "audit_committee")
            .count();

        assert_eq!(
            board_count, 2,
            "6-month period should have 2 board meetings"
        );
        assert_eq!(ac_count, 6, "6-month period should have 6 AC meetings");
    }

    #[test]
    fn test_meeting_ids_unique() {
        let mut gen = BoardMinutesGenerator::new(42);
        let minutes = gen.generate_board_minutes("C001", 2025, 12, &sample_employees());
        let ids: std::collections::HashSet<_> = minutes.iter().map(|m| m.meeting_id).collect();
        assert_eq!(ids.len(), minutes.len(), "all meeting IDs should be unique");
    }

    #[test]
    fn test_dates_are_valid_and_sorted() {
        let mut gen = BoardMinutesGenerator::new(42);
        let minutes = gen.generate_board_minutes("C001", 2025, 12, &sample_employees());
        for m in &minutes {
            assert_eq!(m.meeting_date.year(), 2025);
        }
        for w in minutes.windows(2) {
            assert!(
                w[0].meeting_date <= w[1].meeting_date,
                "minutes should be sorted chronologically"
            );
        }
    }

    #[test]
    fn test_attendees_from_pool() {
        let employees = sample_employees();
        let mut gen = BoardMinutesGenerator::new(42);
        let minutes = gen.generate_board_minutes("C001", 2025, 12, &employees);
        for m in &minutes {
            assert!(!m.attendees.is_empty(), "should have attendees");
            for a in &m.attendees {
                assert!(
                    employees.contains(a),
                    "attendee {} should come from the employee pool",
                    a
                );
            }
        }
    }

    #[test]
    fn test_empty_employee_pool_fallback() {
        let mut gen = BoardMinutesGenerator::new(42);
        let minutes = gen.generate_board_minutes("C001", 2025, 3, &[]);
        assert!(!minutes.is_empty());
        for m in &minutes {
            assert!(!m.attendees.is_empty(), "should have fallback attendee");
        }
    }

    #[test]
    fn test_has_decisions_and_risks() {
        let mut gen = BoardMinutesGenerator::new(42);
        let minutes = gen.generate_board_minutes("C001", 2025, 12, &sample_employees());
        for m in &minutes {
            assert!(!m.key_decisions.is_empty(), "should have key decisions");
            assert!(
                !m.risk_discussions.is_empty(),
                "should have risk discussions"
            );
        }
    }

    #[test]
    fn test_deterministic_with_same_seed() {
        let employees = sample_employees();

        let mut gen1 = BoardMinutesGenerator::new(999);
        let m1 = gen1.generate_board_minutes("C001", 2025, 12, &employees);

        let mut gen2 = BoardMinutesGenerator::new(999);
        let m2 = gen2.generate_board_minutes("C001", 2025, 12, &employees);

        assert_eq!(m1.len(), m2.len());
        for (a, b) in m1.iter().zip(m2.iter()) {
            assert_eq!(a.meeting_id, b.meeting_id);
            assert_eq!(a.meeting_date, b.meeting_date);
            assert_eq!(a.meeting_type, b.meeting_type);
            assert_eq!(a.key_decisions, b.key_decisions);
        }
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut gen = BoardMinutesGenerator::new(42);
        let minutes = gen.generate_board_minutes("C001", 2025, 12, &sample_employees());
        let json = serde_json::to_string(&minutes).expect("serialize");
        let parsed: Vec<BoardMinutes> = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(minutes.len(), parsed.len());
        for (orig, rt) in minutes.iter().zip(parsed.iter()) {
            assert_eq!(orig.meeting_id, rt.meeting_id);
            assert_eq!(orig.meeting_date, rt.meeting_date);
        }
    }
}
