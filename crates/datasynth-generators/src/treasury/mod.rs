//! Treasury and cash management generators.
//!
//! This module provides generators for:
//! - Daily cash positions aggregated from payment flows
//! - Cash forecasts with probability-weighted items
//! - Cash pool sweeps (zero-balancing, physical, notional)
//! - Hedging instruments and hedge relationship designations
//! - Debt instruments with amortization schedules and covenants
//! - Bank guarantees and letters of credit
//! - Intercompany netting runs

mod cash_forecast_generator;
mod cash_pool_generator;
mod cash_position_generator;
mod debt_generator;
mod hedging_generator;
mod treasury_anomaly;

pub use cash_forecast_generator::*;
pub use cash_pool_generator::*;
pub use cash_position_generator::*;
pub use debt_generator::*;
pub use hedging_generator::*;
pub use treasury_anomaly::*;
