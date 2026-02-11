//! Causal and counterfactual data generation.
//!
//! Implements Structural Causal Models (SCMs) for:
//! - Defining causal graphs with typed variables and mechanisms
//! - Generating data respecting causal structure
//! - do-calculus interventions (do(X=x))
//! - Counterfactual "what-if" scenarios

pub mod counterfactual;
pub mod graph;
pub mod intervention;
pub mod scm;
pub mod validation;

pub use counterfactual::*;
pub use graph::*;
pub use intervention::*;
pub use scm::*;
pub use validation::*;
