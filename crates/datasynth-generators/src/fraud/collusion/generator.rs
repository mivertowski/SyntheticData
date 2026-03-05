//! Bulk collusion ring generator.
//!
//! Creates realistic collusion rings from employee and vendor pools,
//! then simulates their lifecycle across a configurable number of months.

use chrono::NaiveDate;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use datasynth_core::AcfeFraudCategory;

use super::network::{CollusionRing, CollusionRingType, Conspirator, ConspiratorRole, EntityType};

/// Generates collusion rings from available employee and vendor pools.
pub struct CollusionRingGenerator {
    rng: ChaCha8Rng,
}

impl CollusionRingGenerator {
    /// Create a new generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    /// Generate collusion rings and advance them over the simulation period.
    ///
    /// Creates 1-3 rings (resources permitting) from the supplied employee and
    /// vendor ID pools, picks ring types appropriate to the available entities,
    /// populates each ring with `Conspirator` members, and then advances every
    /// ring month-by-month.
    pub fn generate(
        &mut self,
        employee_ids: &[String],
        vendor_ids: &[String],
        start_date: NaiveDate,
        months: u32,
    ) -> Vec<CollusionRing> {
        // Need at least 2 employees to form any ring.
        if employee_ids.len() < 2 {
            return Vec::new();
        }

        // Decide how many rings to create (1-3).
        let max_rings = 3.min(employee_ids.len() / 2);
        let ring_count = self.rng.random_range(1..=max_rings);

        let mut rings = Vec::with_capacity(ring_count);
        let mut employee_cursor = 0usize;
        let mut vendor_cursor = 0usize;

        for _ in 0..ring_count {
            // Pick a ring type that we can actually populate.
            let ring_type = self.pick_ring_type(
                employee_ids.len() - employee_cursor,
                vendor_ids.len() - vendor_cursor,
            );

            let (min_size, max_size) = ring_type.typical_size_range();

            // Pick a fraud category weighted by ACFE frequencies.
            let fraud_category = self.pick_fraud_category();

            let mut ring = CollusionRing::new(ring_type, fraud_category, start_date);

            // Determine ring size within bounds.
            let available_employees = employee_ids.len() - employee_cursor;
            let _available_vendors = vendor_ids.len() - vendor_cursor;

            let size = if max_size <= min_size {
                min_size
            } else {
                self.rng.random_range(min_size..=max_size)
            };

            // Populate the ring with conspirators.
            let mut added = 0usize;

            // The first member is always the Initiator (an employee).
            if employee_cursor < employee_ids.len() {
                let c = self.make_conspirator(
                    &employee_ids[employee_cursor],
                    EntityType::Employee,
                    ConspiratorRole::Initiator,
                    start_date,
                );
                ring.add_member(c);
                employee_cursor += 1;
                added += 1;
            }

            // For external ring types, add at least one external member.
            if ring_type.involves_external() && vendor_cursor < vendor_ids.len() && added < size {
                let c = self.make_conspirator(
                    &vendor_ids[vendor_cursor],
                    EntityType::Vendor,
                    ConspiratorRole::Beneficiary,
                    start_date,
                );
                ring.add_member(c);
                vendor_cursor += 1;
                added += 1;
            }

            // Fill remaining slots from employee pool with varied roles.
            let remaining_roles = [
                ConspiratorRole::Executor,
                ConspiratorRole::Approver,
                ConspiratorRole::Concealer,
                ConspiratorRole::Lookout,
            ];
            let mut role_idx = 0;

            while added < size && employee_cursor < employee_ids.len() && available_employees > 0 {
                let role = remaining_roles[role_idx % remaining_roles.len()];
                let c = self.make_conspirator(
                    &employee_ids[employee_cursor],
                    EntityType::Employee,
                    role,
                    start_date,
                );
                ring.add_member(c);
                employee_cursor += 1;
                added += 1;
                role_idx += 1;
            }

            // Advance the ring month-by-month.
            for _ in 0..months {
                ring.advance_month(&mut self.rng);
            }

            rings.push(ring);

            // If we've exhausted all employees, stop creating rings.
            if employee_cursor >= employee_ids.len() {
                break;
            }
            // Need at least 2 employees remaining for the next ring.
            if employee_ids.len() - employee_cursor < 2 {
                break;
            }
        }

        rings
    }

    // ----------------------------------------------------------------
    // Helpers
    // ----------------------------------------------------------------

    /// Pick a ring type that can be populated with the remaining entity pools.
    fn pick_ring_type(
        &mut self,
        remaining_employees: usize,
        remaining_vendors: usize,
    ) -> CollusionRingType {
        let mut candidates: Vec<CollusionRingType> = Vec::new();

        if remaining_employees >= 2 {
            candidates.push(CollusionRingType::EmployeePair);
        }
        if remaining_employees >= 3 {
            candidates.push(CollusionRingType::DepartmentRing);
            candidates.push(CollusionRingType::CrossDepartment);
        }
        if remaining_employees >= 2 {
            candidates.push(CollusionRingType::ManagementSubordinate);
        }
        if remaining_employees >= 1 && remaining_vendors >= 1 {
            candidates.push(CollusionRingType::EmployeeVendor);
        }

        if candidates.is_empty() {
            // Fallback: should not happen because the caller checks min 2 employees.
            return CollusionRingType::EmployeePair;
        }

        let idx = self.rng.random_range(0..candidates.len());
        candidates[idx]
    }

    /// Pick a fraud category using ACFE-weighted probabilities.
    fn pick_fraud_category(&mut self) -> AcfeFraudCategory {
        let roll: f64 = self.rng.random();
        if roll < 0.50 {
            AcfeFraudCategory::AssetMisappropriation
        } else if roll < 0.80 {
            AcfeFraudCategory::Corruption
        } else {
            AcfeFraudCategory::FinancialStatementFraud
        }
    }

    /// Create a single `Conspirator` with randomised loyalty / risk tolerance.
    fn make_conspirator(
        &mut self,
        entity_id: &str,
        entity_type: EntityType,
        role: ConspiratorRole,
        join_date: NaiveDate,
    ) -> Conspirator {
        let loyalty = 0.5 + self.rng.random::<f64>() * 0.4; // 0.50 – 0.90
        let risk_tolerance = 0.3 + self.rng.random::<f64>() * 0.5; // 0.30 – 0.80
        let proceeds_share = 0.1 + self.rng.random::<f64>() * 0.4; // 0.10 – 0.50

        Conspirator::new(entity_id, entity_type, role, join_date)
            .with_loyalty(loyalty)
            .with_risk_tolerance(risk_tolerance)
            .with_proceeds_share(proceeds_share)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_empty_when_insufficient_employees() {
        let mut gen = CollusionRingGenerator::new(42);
        let employees = vec!["EMP001".to_string()];
        let vendors = vec!["V001".to_string()];
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        let rings = gen.generate(&employees, &vendors, start, 6);
        assert!(rings.is_empty(), "Need at least 2 employees");
    }

    #[test]
    fn test_generate_creates_rings() {
        let mut gen = CollusionRingGenerator::new(42);
        let employees: Vec<String> = (1..=10).map(|i| format!("EMP{:03}", i)).collect();
        let vendors: Vec<String> = (1..=5).map(|i| format!("V{:03}", i)).collect();
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        let rings = gen.generate(&employees, &vendors, start, 12);

        assert!(!rings.is_empty(), "Should generate at least one ring");
        assert!(rings.len() <= 3, "Should generate at most 3 rings");

        for ring in &rings {
            assert!(ring.size() >= 2, "Each ring should have at least 2 members");
            assert!(
                ring.active_months > 0 || ring.status.is_terminated(),
                "Ring should have been advanced or terminated"
            );
        }
    }

    #[test]
    fn test_generate_deterministic() {
        let employees: Vec<String> = (1..=6).map(|i| format!("EMP{:03}", i)).collect();
        let vendors: Vec<String> = (1..=3).map(|i| format!("V{:03}", i)).collect();
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        let rings_a = CollusionRingGenerator::new(99).generate(&employees, &vendors, start, 6);
        let rings_b = CollusionRingGenerator::new(99).generate(&employees, &vendors, start, 6);

        assert_eq!(rings_a.len(), rings_b.len());
        for (a, b) in rings_a.iter().zip(rings_b.iter()) {
            assert_eq!(a.ring_type, b.ring_type);
            assert_eq!(a.size(), b.size());
            assert_eq!(a.active_months, b.active_months);
            assert_eq!(a.status, b.status);
        }
    }
}
