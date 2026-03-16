//! Intercompany transaction generators.
//!
//! This module provides generators for:
//! - Matched intercompany journal entry pairs
//! - IC matching and reconciliation
//! - Consolidation elimination entries

mod elimination_generator;
mod elimination_to_je;
mod ic_generator;
mod matching_engine;

pub use elimination_generator::*;
pub use elimination_to_je::elimination_to_journal_entries;
pub use ic_generator::*;
pub use matching_engine::*;
