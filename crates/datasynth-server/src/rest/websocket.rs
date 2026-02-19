//! WebSocket handlers for real-time data streaming.

use std::time::Duration;

use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use serde::Serialize;
use tracing::{error, info, warn};

use super::routes::AppState;
use datasynth_runtime::{EnhancedOrchestrator, PhaseConfig};

/// Metrics update sent via WebSocket.
#[derive(Debug, Serialize)]
pub struct MetricsUpdate {
    pub timestamp: String,
    pub total_entries: u64,
    pub total_anomalies: u64,
    pub entries_per_second: f64,
    pub active_streams: u32,
    pub uptime_seconds: u64,
}

/// Event sent via WebSocket.
#[derive(Debug, Serialize)]
pub struct EventUpdate {
    pub sequence: u64,
    pub timestamp: String,
    pub event_type: String,
    pub document_id: String,
    pub company_code: String,
    pub amount: String,
    pub is_anomaly: bool,
}

/// Marker type for metrics stream.
pub struct MetricsStream;

/// Handle WebSocket connection for metrics streaming.
pub async fn handle_metrics_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    info!("Metrics WebSocket connected");

    // Spawn a task to send metrics updates
    let state_clone = state.clone();
    let mut interval = tokio::time::interval(Duration::from_secs(1));

    loop {
        tokio::select! {
            // Send metrics every second
            _ = interval.tick() => {
                let uptime = state_clone.server_state.uptime_seconds();
                let total_entries = state_clone.server_state.total_entries.load(std::sync::atomic::Ordering::Relaxed);

                let entries_per_second = if uptime > 0 {
                    total_entries as f64 / uptime as f64
                } else {
                    0.0
                };

                let update = MetricsUpdate {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    total_entries,
                    total_anomalies: state_clone.server_state.total_anomalies.load(std::sync::atomic::Ordering::Relaxed),
                    entries_per_second,
                    active_streams: state_clone.server_state.active_streams.load(std::sync::atomic::Ordering::Relaxed) as u32,
                    uptime_seconds: uptime,
                };

                match serde_json::to_string(&update) {
                    Ok(json) => {
                        if sender.send(Message::Text(json.into())).await.is_err() {
                            info!("Metrics WebSocket client disconnected");
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Failed to serialize metrics: {}", e);
                    }
                }
            }
            // Handle incoming messages (for ping/pong or close)
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => {
                        info!("Metrics WebSocket closed by client");
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        if sender.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        warn!("Metrics WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Handle WebSocket connection for event streaming.
pub async fn handle_events_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    info!("Events WebSocket connected");

    // Increment active streams
    state
        .server_state
        .active_streams
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let config = state.server_state.config.read().await.clone();

    let phase_config = PhaseConfig {
        generate_master_data: false,
        generate_document_flows: false,
        generate_journal_entries: true,
        inject_anomalies: false,
        show_progress: false,
        ..Default::default()
    };

    let mut sequence = 0u64;
    let delay = Duration::from_millis(100); // 10 events per second

    loop {
        // Check if we should stop
        if state
            .server_state
            .stream_stopped
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            info!("Events stream stopped by control command");
            break;
        }

        // Check if we should pause
        while state
            .server_state
            .stream_paused
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            tokio::time::sleep(Duration::from_millis(100)).await;
            if state
                .server_state
                .stream_stopped
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                break;
            }
        }

        // Check for incoming messages
        tokio::select! {
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => {
                        info!("Events WebSocket closed by client");
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        if sender.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        warn!("Events WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
            _ = tokio::time::sleep(delay) => {
                // Generate and send an event
                let mut orchestrator = match EnhancedOrchestrator::new(config.clone(), phase_config.clone()) {
                    Ok(o) => o,
                    Err(e) => {
                        error!("Failed to create orchestrator: {}", e);
                        break;
                    }
                };

                let result = match orchestrator.generate() {
                    Ok(r) => r,
                    Err(e) => {
                        error!("Generation failed: {}", e);
                        break;
                    }
                };

                // Send each entry
                for entry in result.journal_entries.iter().take(1) {
                    sequence += 1;
                    state.server_state.total_stream_events.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    state.server_state.total_entries.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                    let total_amount: rust_decimal::Decimal = entry.lines.iter()
                        .map(|l| l.debit_amount)
                        .sum();

                    let event = EventUpdate {
                        sequence,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        event_type: "JournalEntry".to_string(),
                        document_id: entry.header.document_id.to_string(),
                        company_code: entry.header.company_code.clone(),
                        amount: total_amount.to_string(),
                        is_anomaly: entry.header.is_fraud,
                    };

                    match serde_json::to_string(&event) {
                        Ok(json) => {
                            if sender.send(Message::Text(json.into())).await.is_err() {
                                info!("Events WebSocket client disconnected");
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Failed to serialize event: {}", e);
                        }
                    }
                }
            }
        }
    }

    // Decrement active streams
    state
        .server_state
        .active_streams
        .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_update_serialization() {
        let update = MetricsUpdate {
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            total_entries: 1000,
            total_anomalies: 10,
            entries_per_second: 16.67,
            active_streams: 1,
            uptime_seconds: 60,
        };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("total_entries"));
        assert!(json.contains("1000"));
    }

    #[test]
    fn test_event_update_serialization() {
        let event = EventUpdate {
            sequence: 1,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            event_type: "JournalEntry".to_string(),
            document_id: "12345".to_string(),
            company_code: "1000".to_string(),
            amount: "1000.00".to_string(),
            is_anomaly: false,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("JournalEntry"));
        assert!(json.contains("12345"));
    }
}
