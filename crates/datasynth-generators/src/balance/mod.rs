//! Balance coherence generators.
//!
//! This module provides generators for:
//! - Opening balances with industry-specific compositions
//! - Running balance tracking with real-time validation
//! - Trial balance generation at period end
//!
//! The generators ensure that all account balances maintain:
//! - Balance sheet equation: Assets = Liabilities + Equity
//! - Financial ratio coherence (DSO, DPO, margins)
//! - Subledger-to-GL reconciliation

mod balance_tracker;
mod opening_balance_converter;
mod opening_balance_generator;
mod trial_balance_generator;

pub use balance_tracker::*;
pub use opening_balance_converter::*;
pub use opening_balance_generator::*;
pub use trial_balance_generator::*;
