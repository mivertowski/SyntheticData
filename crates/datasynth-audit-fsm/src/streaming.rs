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
/// This function does not panic. Send errors from a dropped receiver are
/// silently discarded.
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
    let ctx = context.clone();

    let handle = thread::spawn(move || {
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
        let ctx = EngagementContext::demo();
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
        let ctx = EngagementContext::demo();
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
        let ctx = EngagementContext::demo();
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
