//! Subsequent event generator per ISA 560 and IAS 10.
//!
//! Generates 0–5 subsequent events per period-end.  Events fall within the
//! window from the period-end date to period-end + 60–90 days.  Approximately
//! 40% of events are adjusting (IAS 10.8); 60% are non-adjusting (IAS 10.21).

use chrono::{Duration, NaiveDate};
use datasynth_core::models::audit::subsequent_events::{
    EventClassification, SubsequentEvent, SubsequentEventType,
};
use datasynth_core::utils::seeded_rng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use tracing::info;

/// Configuration for subsequent event generation.
#[derive(Debug, Clone)]
pub struct SubsequentEventGeneratorConfig {
    /// Maximum number of events per period-end (actual count is 0..=max)
    pub max_events_per_period: u32,
    /// Window in days after period-end during which events are discovered (min, max)
    pub discovery_window_days: (i64, i64),
    /// Probability that an event is adjusting (vs non-adjusting)
    pub adjusting_probability: f64,
    /// Range for financial impact (min, max) in reporting currency units
    pub financial_impact_range: (f64, f64),
}

impl Default for SubsequentEventGeneratorConfig {
    fn default() -> Self {
        Self {
            max_events_per_period: 5,
            discovery_window_days: (60, 90),
            adjusting_probability: 0.40,
            financial_impact_range: (10_000.0, 5_000_000.0),
        }
    }
}

/// Input context for coherent subsequent event generation.
///
/// Provides real financial metrics and risk profile data so that generated
/// events scale proportionally to the entity and reflect its risk landscape.
#[derive(Debug, Clone)]
pub struct SubsequentEventInput {
    /// Total revenue for the period.
    pub total_revenue: Decimal,
    /// Total assets at period-end.
    pub total_assets: Decimal,
    /// Pre-tax income for the period (may be negative for loss-making entities).
    pub pretax_income: Decimal,
    /// Account areas assessed as high or moderate risk by the CRA.
    pub high_risk_areas: Vec<String>,
    /// Whether the going-concern assessment identified material uncertainty.
    pub going_concern_doubt: bool,
}

/// Generator for ISA 560 / IAS 10 subsequent events.
pub struct SubsequentEventGenerator {
    rng: ChaCha8Rng,
    config: SubsequentEventGeneratorConfig,
}

impl SubsequentEventGenerator {
    /// Create a new generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0x560),
            config: SubsequentEventGeneratorConfig::default(),
        }
    }

    /// Create a new generator with custom configuration.
    pub fn with_config(seed: u64, config: SubsequentEventGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0x560),
            config,
        }
    }

    /// Generate subsequent events for a single entity.
    ///
    /// # Arguments
    /// * `entity_code` — Entity code for which events are generated
    /// * `period_end_date` — Balance sheet date; events occur after this date
    pub fn generate_for_entity(
        &mut self,
        entity_code: &str,
        period_end_date: NaiveDate,
    ) -> Vec<SubsequentEvent> {
        info!(
            "Generating subsequent events for entity {} period-end {}",
            entity_code, period_end_date
        );
        let count = self.rng.random_range(0..=self.config.max_events_per_period);
        let window_end_days = self.rng.random_range(
            self.config.discovery_window_days.0..=self.config.discovery_window_days.1,
        );
        let window_end = period_end_date + Duration::days(window_end_days);

        let mut events = Vec::with_capacity(count as usize);

        for _ in 0..count {
            // Event date: 1 day after period-end up to the window end
            let event_offset_days = self.rng.random_range(1..=window_end_days);
            let event_date = period_end_date + Duration::days(event_offset_days);

            // Discovery date: event date up to window end
            let discovery_offset = self
                .rng
                .random_range(0..=(window_end - event_date).num_days());
            let discovery_date = event_date + Duration::days(discovery_offset);
            let discovery_date = discovery_date.min(window_end);

            let event_type = self.random_event_type();
            let classification = if self.rng.random::<f64>() < self.config.adjusting_probability {
                EventClassification::Adjusting
            } else {
                EventClassification::NonAdjusting
            };

            let description = self.describe_event(event_type, &classification, entity_code);

            let mut event = SubsequentEvent::new(
                entity_code,
                event_date,
                discovery_date,
                event_type,
                classification,
                description,
            );

            // Adjusting events always have a financial impact; non-adjusting sometimes do.
            let has_impact = matches!(classification, EventClassification::Adjusting)
                || self.rng.random::<f64>() < 0.50;

            if has_impact {
                let impact_raw = self.rng.random_range(
                    self.config.financial_impact_range.0..=self.config.financial_impact_range.1,
                );
                let impact = Decimal::try_from(impact_raw).unwrap_or(Decimal::new(100_000, 0));
                event = event.with_financial_impact(impact);
            }

            events.push(event);
        }

        info!(
            "Generated {} subsequent events for entity {}",
            events.len(),
            entity_code
        );
        events
    }

    /// Generate subsequent events for multiple entities.
    pub fn generate_for_entities(
        &mut self,
        entity_codes: &[String],
        period_end_date: NaiveDate,
    ) -> Vec<SubsequentEvent> {
        entity_codes
            .iter()
            .flat_map(|code| self.generate_for_entity(code, period_end_date))
            .collect()
    }

    /// Generate subsequent events with real financial context.
    ///
    /// Unlike [`generate_for_entity`], this method:
    /// - Scales financial impact as 0.5–5% of the larger of `total_revenue` and
    ///   `total_assets`, producing amounts proportional to entity size.
    /// - Biases event type selection toward risk areas present in the CRA
    ///   (e.g. more `AssetImpairment` when inventory/fixed-asset risk is high).
    /// - Increases event count and adjusting probability when going-concern
    ///   doubt exists.
    /// - Favours `LitigationSettlement` / `RestructuringAnnouncement` when the
    ///   entity is loss-making.
    pub fn generate_for_entity_with_context(
        &mut self,
        entity_code: &str,
        period_end_date: NaiveDate,
        input: &SubsequentEventInput,
    ) -> Vec<SubsequentEvent> {
        info!(
            "Generating context-aware subsequent events for entity {} period-end {}",
            entity_code, period_end_date
        );

        // --- Event count ---
        let mut count = self.rng.random_range(0..=self.config.max_events_per_period);
        if input.going_concern_doubt {
            count += self.rng.random_range(1..=2);
        }

        // --- Adjusting probability ---
        let adjusting_prob = if input.going_concern_doubt {
            0.60
        } else {
            self.config.adjusting_probability
        };

        // --- Financial impact range (0.5–5% of larger of revenue / assets) ---
        let base = std::cmp::max(input.total_revenue, input.total_assets);
        let base_f64 = base
            .to_string()
            .parse::<f64>()
            .unwrap_or(1_000_000.0)
            .abs()
            .max(100_000.0); // floor to avoid degenerate tiny impacts
        let impact_lo = base_f64 * 0.005;
        let impact_hi = base_f64 * 0.05;

        // --- Discovery window ---
        let window_end_days = self.rng.random_range(
            self.config.discovery_window_days.0..=self.config.discovery_window_days.1,
        );
        let window_end = period_end_date + Duration::days(window_end_days);

        let mut events = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let event_offset_days = self.rng.random_range(1..=window_end_days);
            let event_date = period_end_date + Duration::days(event_offset_days);

            let discovery_offset = self
                .rng
                .random_range(0..=(window_end - event_date).num_days());
            let discovery_date = (event_date + Duration::days(discovery_offset)).min(window_end);

            let event_type = self.weighted_event_type(
                &input.high_risk_areas,
                input.pretax_income.is_sign_negative(),
            );
            let classification = if self.rng.random::<f64>() < adjusting_prob {
                EventClassification::Adjusting
            } else {
                EventClassification::NonAdjusting
            };

            let description = self.describe_event(event_type, &classification, entity_code);

            let mut event = SubsequentEvent::new(
                entity_code,
                event_date,
                discovery_date,
                event_type,
                classification,
                description,
            );

            let has_impact = matches!(classification, EventClassification::Adjusting)
                || self.rng.random::<f64>() < 0.50;

            if has_impact {
                let impact_raw = self.rng.random_range(impact_lo..=impact_hi);
                let impact = Decimal::try_from(impact_raw).unwrap_or(Decimal::new(100_000, 0));
                event = event.with_financial_impact(impact);
            }

            events.push(event);
        }

        info!(
            "Generated {} context-aware subsequent events for entity {}",
            events.len(),
            entity_code
        );
        events
    }

    fn random_event_type(&mut self) -> SubsequentEventType {
        match self.rng.random_range(0u8..8) {
            0 => SubsequentEventType::LitigationSettlement,
            1 => SubsequentEventType::CustomerBankruptcy,
            2 => SubsequentEventType::AssetImpairment,
            3 => SubsequentEventType::RestructuringAnnouncement,
            4 => SubsequentEventType::NaturalDisaster,
            5 => SubsequentEventType::RegulatoryChange,
            6 => SubsequentEventType::MergerAnnouncement,
            _ => SubsequentEventType::DividendDeclaration,
        }
    }

    /// Select event type with weights influenced by high-risk areas and loss
    /// status.  Each event type starts with a base weight of 1.0; matches on
    /// risk area or loss-making bump the weight upward.
    fn weighted_event_type(
        &mut self,
        high_risk_areas: &[String],
        is_loss_making: bool,
    ) -> SubsequentEventType {
        let has_risk = |keywords: &[&str]| -> bool {
            high_risk_areas.iter().any(|area| {
                let lower = area.to_lowercase();
                keywords.iter().any(|kw| lower.contains(kw))
            })
        };

        let mut weights: Vec<(SubsequentEventType, f64)> = vec![
            (SubsequentEventType::LitigationSettlement, 1.0),
            (SubsequentEventType::CustomerBankruptcy, 1.0),
            (SubsequentEventType::AssetImpairment, 1.0),
            (SubsequentEventType::RestructuringAnnouncement, 1.0),
            (SubsequentEventType::NaturalDisaster, 1.0),
            (SubsequentEventType::RegulatoryChange, 1.0),
            (SubsequentEventType::MergerAnnouncement, 1.0),
            (SubsequentEventType::DividendDeclaration, 1.0),
        ];

        // Boost asset impairment when inventory or fixed-asset risk is present.
        if has_risk(&["inventory", "fixed asset", "ppe", "property"]) {
            weights[2].1 += 3.0;
        }
        // Boost customer bankruptcy when receivable risk is present.
        if has_risk(&["receivable", "trade receivable", "revenue"]) {
            weights[1].1 += 3.0;
        }
        // Favour litigation / restructuring for loss-making entities.
        if is_loss_making {
            weights[0].1 += 2.0; // LitigationSettlement
            weights[3].1 += 2.0; // RestructuringAnnouncement
        }

        let total: f64 = weights.iter().map(|(_, w)| w).sum();
        let r: f64 = self.rng.random::<f64>() * total;
        let mut cumulative = 0.0;
        for (et, w) in &weights {
            cumulative += w;
            if r < cumulative {
                return *et;
            }
        }
        SubsequentEventType::DividendDeclaration
    }

    fn describe_event(
        &self,
        event_type: SubsequentEventType,
        classification: &EventClassification,
        entity_code: &str,
    ) -> String {
        let class_str = match classification {
            EventClassification::Adjusting => "Adjusting event (IAS 10.8)",
            EventClassification::NonAdjusting => "Non-adjusting event (IAS 10.21)",
        };

        let event_desc = match event_type {
            SubsequentEventType::LitigationSettlement => {
                format!(
                    "Litigation settlement reached for proceedings against {} that were pending \
                     at the balance sheet date.",
                    entity_code
                )
            }
            SubsequentEventType::CustomerBankruptcy => {
                format!(
                    "A significant customer of {} filed for bankruptcy after the period-end, \
                     indicating a recoverability issue at the balance sheet date.",
                    entity_code
                )
            }
            SubsequentEventType::AssetImpairment => {
                format!(
                    "Indicator of impairment identified for assets held by {} that existed \
                     at the balance sheet date.",
                    entity_code
                )
            }
            SubsequentEventType::RestructuringAnnouncement => {
                format!(
                    "{} announced a restructuring programme after the balance sheet date \
                     that was not planned at that date.",
                    entity_code
                )
            }
            SubsequentEventType::NaturalDisaster => {
                format!(
                    "A natural disaster occurred after the period-end, causing damage to \
                     assets operated by {}.",
                    entity_code
                )
            }
            SubsequentEventType::RegulatoryChange => {
                format!(
                    "A significant regulatory change was enacted after the period-end that \
                     affects operations of {}.",
                    entity_code
                )
            }
            SubsequentEventType::MergerAnnouncement => {
                format!(
                    "{} announced a merger or acquisition after the balance sheet date.",
                    entity_code
                )
            }
            SubsequentEventType::DividendDeclaration => {
                format!(
                    "The board of {} declared a dividend after the balance sheet date.",
                    entity_code
                )
            }
        };

        format!("{} — {}", class_str, event_desc)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn period_end() -> NaiveDate {
        NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()
    }

    #[test]
    fn test_event_count_within_bounds() {
        let mut gen = SubsequentEventGenerator::new(42);
        let events = gen.generate_for_entity("C001", period_end());
        assert!(
            events.len() <= 5,
            "count should be 0..=5, got {}",
            events.len()
        );
    }

    #[test]
    fn test_event_dates_after_period_end() {
        let _gen = SubsequentEventGenerator::new(99);
        let pe = period_end();
        // Run several times to get events
        for seed in [1u64, 2, 3, 4, 5] {
            let mut g = SubsequentEventGenerator::new(seed);
            let events = g.generate_for_entity("C001", pe);
            for event in &events {
                assert!(
                    event.event_date > pe,
                    "event_date {} should be after period_end {}",
                    event.event_date,
                    pe
                );
            }
        }
    }

    #[test]
    fn test_approximately_40_percent_adjusting() {
        let _gen = SubsequentEventGenerator::new(42);
        let pe = period_end();
        let mut total = 0usize;
        let mut adjusting = 0usize;

        // Generate many events to get a stable ratio
        for i in 0..200u64 {
            let mut g = SubsequentEventGenerator::new(i);
            let events = g.generate_for_entity("C001", pe);
            total += events.len();
            adjusting += events
                .iter()
                .filter(|e| matches!(e.classification, EventClassification::Adjusting))
                .count();
        }

        if total > 0 {
            let ratio = adjusting as f64 / total as f64;
            // Allow wide tolerance: 25%–60%
            assert!(
                ratio >= 0.25 && ratio <= 0.60,
                "adjusting ratio = {:.2}, expected ~0.40",
                ratio
            );
        }
    }

    fn default_input() -> SubsequentEventInput {
        SubsequentEventInput {
            total_revenue: Decimal::new(200_000_000, 0),
            total_assets: Decimal::new(350_000_000, 0),
            pretax_income: Decimal::new(15_000_000, 0),
            high_risk_areas: vec![],
            going_concern_doubt: false,
        }
    }

    #[test]
    fn test_context_aware_scales_impact() {
        let _gen = SubsequentEventGenerator::new(42);
        let input = default_input();
        // Generate many to get at least one with impact
        let mut impacts = Vec::new();
        for seed in 0..50u64 {
            let mut g = SubsequentEventGenerator::new(seed);
            let events = g.generate_for_entity_with_context("C001", period_end(), &input);
            for e in &events {
                if let Some(impact) = e.financial_impact {
                    impacts.push(impact);
                }
            }
        }
        // Impacts should be scaled to 0.5–5% of $350M (the larger base)
        // i.e. $1.75M–$17.5M
        let lower = Decimal::new(1_750_000, 0);
        let upper = Decimal::new(17_500_000, 0);
        for impact in &impacts {
            assert!(
                *impact >= lower * Decimal::new(95, 2) && *impact <= upper * Decimal::new(105, 2),
                "impact {} should be roughly between {} and {}",
                impact,
                lower,
                upper
            );
        }
    }

    #[test]
    fn test_going_concern_increases_events() {
        let mut counts_no_gc = Vec::new();
        let mut counts_gc = Vec::new();
        for seed in 0..100u64 {
            let mut g1 = SubsequentEventGenerator::new(seed);
            let input_no_gc = default_input();
            let events = g1.generate_for_entity_with_context("C001", period_end(), &input_no_gc);
            counts_no_gc.push(events.len());

            let mut g2 = SubsequentEventGenerator::new(seed);
            let input_gc = SubsequentEventInput {
                going_concern_doubt: true,
                ..default_input()
            };
            let events = g2.generate_for_entity_with_context("C001", period_end(), &input_gc);
            counts_gc.push(events.len());
        }
        let avg_no_gc: f64 = counts_no_gc.iter().sum::<usize>() as f64 / counts_no_gc.len() as f64;
        let avg_gc: f64 = counts_gc.iter().sum::<usize>() as f64 / counts_gc.len() as f64;
        assert!(
            avg_gc > avg_no_gc,
            "going concern should produce more events on average ({} vs {})",
            avg_gc,
            avg_no_gc
        );
    }

    #[test]
    fn test_adjusting_events_have_financial_impact() {
        let _gen = SubsequentEventGenerator::new(42);
        let pe = period_end();
        for seed in 0..50u64 {
            let mut g = SubsequentEventGenerator::new(seed);
            let events = g.generate_for_entity("C001", pe);
            for event in events
                .iter()
                .filter(|e| matches!(e.classification, EventClassification::Adjusting))
            {
                assert!(
                    event.financial_impact.is_some(),
                    "adjusting event should have a financial impact"
                );
            }
        }
    }
}
