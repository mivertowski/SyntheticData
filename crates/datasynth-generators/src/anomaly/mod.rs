//! Anomaly injection framework for synthetic data generation.
//!
//! This module provides comprehensive anomaly injection capabilities:
//! - Configurable anomaly rates per category
//! - Temporal patterns (year-end spikes, clustering)
//! - Labeled output for supervised learning
//! - Multiple injection strategies
//! - Document flow anomalies (3-way match fraud)
//! - Dynamic confidence calculation (FR-003)
//! - Contextual severity scoring (FR-003)
//! - Multi-dimensional labeling with severity and detection difficulty
//! - Multi-stage fraud schemes (embezzlement, revenue manipulation, kickback)
//! - Correlated anomaly injection patterns
//! - Near-miss generation for false positive reduction
//! - Context-aware injection based on entity behaviors

pub mod confidence;
pub mod context;
pub mod correlation;
mod difficulty;
mod document_flow_anomalies;
mod injector;
mod near_miss;
mod patterns;
mod scheme_advancer;
pub mod schemes;
pub mod severity;
mod strategies;
mod types;

pub use confidence::{ConfidenceCalculator, ConfidenceConfig, ConfidenceContext};
pub use context::{
    AccountAnomalyRules, AccountContext, BehavioralBaseline, BehavioralBaselineConfig,
    BehavioralDeviation, DeviationType, EmployeeAnomalyRules, EmployeeContext, EntityAwareConfig,
    EntityAwareInjector, EntityBaseline, Observation, VendorAnomalyRules, VendorContext,
};
pub use correlation::{
    AnomalyCoOccurrence, CascadeConfig, CascadeGenerator, CascadeStep, CoOccurrencePattern,
    CorrelatedAnomaly, TemporalAnomalyCluster, TemporalClusterGenerator, TemporalWindow,
};
pub use difficulty::{
    AmountFactors, BlendingFactors, CollusionFactors, ConcealmentFactors, DifficultyAssessment,
    DifficultyCalculator, DifficultyFactors, TemporalFactors,
};
pub use document_flow_anomalies::*;
pub use injector::*;
pub use near_miss::{NearMissConfig, NearMissGenerator, NearMissStatistics};
pub use patterns::*;
pub use scheme_advancer::{
    MultiStageAnomalyLabel, SchemeAdvancer, SchemeAdvancerConfig, SchemeStatistics,
};
pub use schemes::{
    FraudScheme, GradualEmbezzlementScheme, RevenueManipulationScheme, SchemeAction,
    SchemeActionType, SchemeContext, SchemeStage, SchemeStatus, VendorKickbackScheme,
};
pub use severity::{
    AnomalyScoreCalculator, AnomalyScores, SeverityCalculator, SeverityConfig, SeverityContext,
};
pub use strategies::*;
pub use types::*;
