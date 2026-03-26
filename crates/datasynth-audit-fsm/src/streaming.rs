//! Streaming engagement execution.
//!
//! Enables the FSM engine to emit events via a callback during execution
//! rather than collecting them all in memory. This is the foundation for
//! WebSocket/dashboard integration.

use std::sync::mpsc;
use std::thread;

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::context::EngagementContext;
use crate::engine::{AuditFsmEngine, EngagementResult};
use crate::error::AuditFsmError;
use crate::event::AuditEvent;
use crate::loader::BlueprintWithPreconditions;
use crate::schema::GenerationOverlay;

/// Callback invoked on each FSM event during streaming execution.
pub type EventCallback<'a> = Box<dyn FnMut(&AuditEvent) + Send + 'a>;

/// Run an engagement with streaming event emission.
///
/// This wraps the existing [`AuditFsmEngine::run_engagement`] and calls
/// `on_event` for each event produced. The current implementation runs
/// the engagement to completion and then iterates the event log, calling
/// the callback for each event. This enables the streaming API pattern
/// without refactoring the engine internals.
///
/// # Errors
///
/// Returns an error if the underlying engine encounters a problem during
/// execution (e.g. DAG cycle, guard failure).
pub fn run_engagement_streaming(
    bwp: &BlueprintWithPreconditions,
    overlay: &GenerationOverlay,
    context: &EngagementContext,
    seed: u64,
    mut on_event: EventCallback<'_>,
) -> Result<EngagementResult, AuditFsmError> {
    let rng = ChaCha8Rng::seed_from_u64(seed);
    let mut engine = AuditFsmEngine::new(bwp.clone(), overlay.clone(), rng);
    let result = engine.run_engagement(context)?;

    // Stream events to the callback.
    for event in &result.event_log {
        on_event(event);
    }

    Ok(result)
}

/// Run an engagement on a background thread, sending events through an mpsc channel.
///
/// Returns a `Receiver` that yields each [`AuditEvent`] and a `JoinHandle`
/// whose value is the full [`EngagementResult`].
///
/// # Panics
///
/// The spawned thread will panic if the receiver is dropped before all
/// events are sent, but the join handle will capture the panic.
pub fn run_engagement_to_channel(
    bwp: &BlueprintWithPreconditions,
    overlay: &GenerationOverlay,
    context: &EngagementContext,
    seed: u64,
) -> (
    mpsc::Receiver<AuditEvent>,
    thread::JoinHandle<Result<EngagementResult, AuditFsmError>>,
) {
    let (tx, rx) = mpsc::channel();

    // Clone owned data for the spawned thread.
    let bwp = bwp.clone();
    let overlay = overlay.clone();
    let context_company_code = context.company_code.clone();
    let context_company_name = context.company_name.clone();
    let context_fiscal_year = context.fiscal_year;
    let context_currency = context.currency.clone();
    let context_total_revenue = context.total_revenue;
    let context_total_assets = context.total_assets;
    let context_engagement_start = context.engagement_start;
    let context_report_date = context.report_date;
    let context_pretax_income = context.pretax_income;
    let context_equity = context.equity;
    let context_gross_profit = context.gross_profit;
    let context_working_capital = context.working_capital;
    let context_operating_cash_flow = context.operating_cash_flow;
    let context_total_debt = context.total_debt;
    let context_team_member_ids = context.team_member_ids.clone();
    let context_team_member_pairs = context.team_member_pairs.clone();
    let context_accounts = context.accounts.clone();
    let context_vendor_names = context.vendor_names.clone();
    let context_customer_names = context.customer_names.clone();
    let context_journal_entry_ids = context.journal_entry_ids.clone();
    let context_account_balances = context.account_balances.clone();
    let context_control_ids = context.control_ids.clone();
    let context_anomaly_refs = context.anomaly_refs.clone();
    let context_is_us_listed = context.is_us_listed;
    let context_entity_codes = context.entity_codes.clone();

    let handle = thread::spawn(move || {
        let ctx = EngagementContext {
            company_code: context_company_code,
            company_name: context_company_name,
            fiscal_year: context_fiscal_year,
            currency: context_currency,
            total_revenue: context_total_revenue,
            total_assets: context_total_assets,
            engagement_start: context_engagement_start,
            report_date: context_report_date,
            pretax_income: context_pretax_income,
            equity: context_equity,
            gross_profit: context_gross_profit,
            working_capital: context_working_capital,
            operating_cash_flow: context_operating_cash_flow,
            total_debt: context_total_debt,
            team_member_ids: context_team_member_ids,
            team_member_pairs: context_team_member_pairs,
            accounts: context_accounts,
            vendor_names: context_vendor_names,
            customer_names: context_customer_names,
            journal_entry_ids: context_journal_entry_ids,
            account_balances: context_account_balances,
            control_ids: context_control_ids,
            anomaly_refs: context_anomaly_refs,
            is_us_listed: context_is_us_listed,
            entity_codes: context_entity_codes,
        };

        let rng = ChaCha8Rng::seed_from_u64(seed);
        let mut engine = AuditFsmEngine::new(bwp, overlay, rng);
        let result = engine.run_engagement(&ctx)?;

        // Send each event through the channel.
        for event in &result.event_log {
            // Ignore send errors — the receiver may have been dropped.
            let _ = tx.send(event.clone());
        }

        Ok(result)
    });

    (rx, handle)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::{default_overlay, BlueprintWithPreconditions};

    fn load_fsa() -> (BlueprintWithPreconditions, GenerationOverlay) {
        let bwp = BlueprintWithPreconditions::load_builtin_fsa().unwrap();
        bwp.validate().unwrap();
        let overlay = default_overlay();
        (bwp, overlay)
    }

    #[test]
    fn test_streaming_callback_receives_all_events() {
        let (bwp, overlay) = load_fsa();
        let ctx = EngagementContext::test_default();
        let seed = 42u64;

        let mut received = Vec::new();
        let result = run_engagement_streaming(
            &bwp,
            &overlay,
            &ctx,
            seed,
            Box::new(|evt| {
                received.push(evt.clone());
            }),
        )
        .unwrap();

        assert!(!received.is_empty(), "callback should receive events");
        assert_eq!(
            received.len(),
            result.event_log.len(),
            "callback count must match event_log length"
        );
    }

    #[test]
    fn test_channel_receives_all_events() {
        let (bwp, overlay) = load_fsa();
        let ctx = EngagementContext::test_default();
        let seed = 42u64;

        let (rx, handle) = run_engagement_to_channel(&bwp, &overlay, &ctx, seed);

        // Collect all events from the channel.
        let mut channel_events = Vec::new();
        while let Ok(evt) = rx.recv() {
            channel_events.push(evt);
        }

        let result = handle.join().expect("thread should not panic").unwrap();
        assert!(!channel_events.is_empty(), "channel should receive events");
        assert_eq!(
            channel_events.len(),
            result.event_log.len(),
            "channel count must match event_log length"
        );
    }

    #[test]
    fn test_streaming_results_match_non_streaming() {
        let (bwp, overlay) = load_fsa();
        let ctx = EngagementContext::test_default();
        let seed = 77u64;

        // Run non-streaming.
        let rng = ChaCha8Rng::seed_from_u64(seed);
        let mut engine = AuditFsmEngine::new(bwp.clone(), overlay.clone(), rng);
        let baseline = engine.run_engagement(&ctx).unwrap();

        // Run streaming.
        let streaming =
            run_engagement_streaming(&bwp, &overlay, &ctx, seed, Box::new(|_| {})).unwrap();

        assert_eq!(
            baseline.event_log.len(),
            streaming.event_log.len(),
            "event counts must match"
        );
        assert_eq!(
            baseline.procedure_states, streaming.procedure_states,
            "procedure states must match"
        );
        assert!(
            (baseline.total_duration_hours - streaming.total_duration_hours).abs() < 0.001,
            "durations must match"
        );
    }
}
