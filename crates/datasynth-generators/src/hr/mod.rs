//! Hire-to-Retire (H2R) generators for the HR process chain.
//!
//! Generation pipeline:
//! employees (master data) -> payroll_run + time_entries + expense_reports

mod expense_report_generator;
mod payroll_generator;
mod time_entry_generator;

pub use expense_report_generator::*;
pub use payroll_generator::*;
pub use time_entry_generator::*;
