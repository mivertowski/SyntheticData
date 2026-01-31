//! Multi-stage fraud scheme framework.
//!
//! This module provides realistic multi-stage fraud schemes that evolve over time,
//! including embezzlement, revenue manipulation, and kickback schemes.

mod embezzlement;
mod kickback;
mod revenue_manipulation;
mod scheme;

pub use embezzlement::GradualEmbezzlementScheme;
pub use kickback::VendorKickbackScheme;
pub use revenue_manipulation::RevenueManipulationScheme;
pub use scheme::{
    FraudScheme, SchemeAction, SchemeActionType, SchemeContext, SchemeStage, SchemeStatus,
    SchemeTransactionRef,
};
