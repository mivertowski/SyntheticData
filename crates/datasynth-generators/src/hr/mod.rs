//! Hire-to-Retire (H2R) generators for the HR process chain.
//!
//! Generation pipeline:
//! employees (master data) -> payroll_run + time_entries + expense_reports + benefit_enrollments
//!                         -> pension plans (IAS 19 / ASC 715)

mod benefit_enrollment_generator;
mod expense_report_generator;
mod payroll_generator;
pub mod pension_generator;
mod time_entry_generator;

pub use benefit_enrollment_generator::*;
pub use expense_report_generator::*;
pub use payroll_generator::*;
pub use pension_generator::*;
pub use time_entry_generator::*;
