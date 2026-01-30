//! Statistical distribution samplers for realistic data generation.
//!
//! Based on empirical findings from the accounting network generation paper,
//! these samplers produce data that matches real-world distributions.
//!
//! # Modules
//!
//! - **amount**: Log-normal amount sampling with round-number bias
//! - **benford**: Benford's Law compliant sampling and fraud patterns
//! - **mixture**: Gaussian and Log-Normal mixture models
//! - **pareto**: Heavy-tailed Pareto distribution
//! - **weibull**: Time-to-event Weibull distribution
//! - **beta**: Beta distribution for proportions
//! - **zero_inflated**: Zero-inflated distributions
//! - **correlation**: Cross-field correlation engine
//! - **copula**: Copula-based dependency modeling
//! - **conditional**: Conditional distributions with breakpoints
//! - **drift**: Temporal drift and regime changes
//! - **industry_profiles**: Industry-specific distribution profiles
//! - **holidays**: Holiday calendar handling
//! - **seasonality**: Seasonal patterns
//! - **temporal**: Temporal sampling
//! - **business_day**: Business day calculations and settlement dates
//! - **period_end**: Period-end decay curves and dynamics
//! - **processing_lag**: Event-to-posting lag modeling
//! - **timezone**: Multi-region timezone handling

mod amount;
mod benford;
mod beta;
mod business_day;
mod conditional;
mod copula;
mod correlation;
mod drift;
mod holidays;
mod industry_profiles;
mod line_item;
mod mixture;
mod pareto;
mod period_end;
mod processing_lag;
mod seasonality;
mod temporal;
mod timezone;
mod weibull;
mod zero_inflated;

pub use amount::*;
pub use benford::*;
pub use beta::*;
pub use business_day::*;
pub use conditional::*;
pub use copula::*;
pub use correlation::*;
pub use drift::*;
pub use holidays::*;
pub use industry_profiles::*;
pub use line_item::*;
pub use mixture::*;
pub use pareto::*;
pub use period_end::*;
pub use processing_lag::*;
pub use seasonality::*;
pub use temporal::*;
pub use timezone::*;
pub use weibull::*;
pub use zero_inflated::*;
