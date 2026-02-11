//! Streaming HTTP client for pushing unified JSONL to a RustGraph ingest endpoint.
//!
//! Implements `std::io::Write` so it can be passed directly to
//! `RustGraphUnifiedExporter::export_to_writer`. Buffers JSONL lines in memory
//! and auto-flushes when batch size is reached.

use std::io::{self, Write};
use std::time::Duration;

use reqwest::blocking::Client;
use tracing::{debug, warn};

/// Configuration for the streaming client.
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Target URL for the RustGraph ingest endpoint.
    pub target_url: String,
    /// Number of JSONL lines per HTTP POST batch.
    pub batch_size: usize,
    /// HTTP request timeout in seconds.
    pub timeout_secs: u64,
    /// Optional API key for authentication.
    pub api_key: Option<String>,
    /// Maximum number of retries per batch.
    pub max_retries: u32,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            target_url: String::new(),
            batch_size: 1000,
            timeout_secs: 30,
            api_key: None,
            max_retries: 3,
        }
    }
}

/// HTTP streaming client that implements `Write` for JSONL output.
///
/// Each complete line (terminated by `\n`) is buffered. When the buffer
/// reaches `batch_size` lines, the batch is POSTed to the target URL.
pub struct StreamClient {
    config: StreamConfig,
    client: Client,
    /// Buffer of complete JSONL lines ready to send.
    lines: Vec<String>,
    /// Partial line buffer (data received without trailing newline).
    partial: String,
    /// Total lines sent.
    total_sent: usize,
}

impl StreamClient {
    /// Create a new streaming client with the given configuration.
    pub fn new(config: StreamConfig) -> io::Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(io::Error::other)?;

        Ok(Self {
            config,
            client,
            lines: Vec::new(),
            partial: String::new(),
            total_sent: 0,
        })
    }

    /// Returns the total number of lines sent so far.
    pub fn total_sent(&self) -> usize {
        self.total_sent
    }

    /// Send a batch of lines to the target endpoint.
    fn send_batch(&mut self) -> io::Result<()> {
        if self.lines.is_empty() {
            return Ok(());
        }

        let payload = self.lines.join("");
        let batch_len = self.lines.len();

        for attempt in 0..=self.config.max_retries {
            let mut request = self
                .client
                .post(&self.config.target_url)
                .header("Content-Type", "application/x-ndjson")
                .body(payload.clone());

            if let Some(ref key) = self.config.api_key {
                request = request.header("X-API-Key", key);
            }

            match request.send() {
                Ok(response) if response.status().is_success() => {
                    debug!(
                        "Streamed batch of {} lines (total: {})",
                        batch_len,
                        self.total_sent + batch_len
                    );
                    self.total_sent += batch_len;
                    self.lines.clear();
                    return Ok(());
                }
                Ok(response) => {
                    let status = response.status();
                    if attempt < self.config.max_retries {
                        warn!(
                            "Stream batch failed (HTTP {}), retry {}/{}",
                            status,
                            attempt + 1,
                            self.config.max_retries
                        );
                        std::thread::sleep(Duration::from_millis(500 * (attempt as u64 + 1)));
                    } else {
                        return Err(io::Error::other(format!(
                            "Stream batch failed after {} retries (HTTP {})",
                            self.config.max_retries, status
                        )));
                    }
                }
                Err(e) => {
                    if attempt < self.config.max_retries {
                        warn!(
                            "Stream batch error: {}, retry {}/{}",
                            e,
                            attempt + 1,
                            self.config.max_retries
                        );
                        std::thread::sleep(Duration::from_millis(500 * (attempt as u64 + 1)));
                    } else {
                        return Err(io::Error::other(format!(
                            "Stream batch failed after {} retries: {}",
                            self.config.max_retries, e
                        )));
                    }
                }
            }
        }

        Ok(())
    }
}

impl Write for StreamClient {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s =
            std::str::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        self.partial.push_str(s);

        // Split on newlines and buffer complete lines
        while let Some(pos) = self.partial.find('\n') {
            let line = self.partial[..=pos].to_string();
            self.partial = self.partial[pos + 1..].to_string();
            self.lines.push(line);

            // Auto-flush when batch size reached
            if self.lines.len() >= self.config.batch_size {
                self.send_batch()?;
            }
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        // If there's a partial line remaining, treat it as a complete line
        if !self.partial.is_empty() {
            let mut line = std::mem::take(&mut self.partial);
            if !line.ends_with('\n') {
                line.push('\n');
            }
            self.lines.push(line);
        }

        // Send any remaining buffered lines
        self.send_batch()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_config_default() {
        let config = StreamConfig::default();
        assert_eq!(config.batch_size, 1000);
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.max_retries, 3);
        assert!(config.api_key.is_none());
    }

    #[test]
    fn test_line_buffering() {
        // We can't easily test HTTP posting without a server, but we can
        // test the line buffering logic by using a large batch_size so
        // no actual sends happen.
        let config = StreamConfig {
            target_url: "http://localhost:9999/ingest".to_string(),
            batch_size: 10000, // Large so no auto-flush
            ..Default::default()
        };
        let mut client = StreamClient::new(config).unwrap();

        // Write some JSONL
        client
            .write_all(b"{\"_type\":\"node\",\"id\":\"1\"}\n")
            .unwrap();
        client
            .write_all(b"{\"_type\":\"node\",\"id\":\"2\"}\n")
            .unwrap();

        assert_eq!(client.lines.len(), 2);
        assert!(client.partial.is_empty());
        assert_eq!(client.total_sent, 0);
    }

    #[test]
    fn test_partial_line_handling() {
        let config = StreamConfig {
            target_url: "http://localhost:9999/ingest".to_string(),
            batch_size: 10000,
            ..Default::default()
        };
        let mut client = StreamClient::new(config).unwrap();

        // Write partial line
        client.write_all(b"{\"_type\":\"node\"").unwrap();
        assert_eq!(client.lines.len(), 0);
        assert_eq!(client.partial, "{\"_type\":\"node\"");

        // Complete the line
        client.write_all(b",\"id\":\"1\"}\n").unwrap();
        assert_eq!(client.lines.len(), 1);
        assert!(client.partial.is_empty());
    }
}
