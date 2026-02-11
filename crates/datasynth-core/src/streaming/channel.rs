//! Channel utilities for streaming generation.
//!
//! Provides bounded channels with backpressure support for
//! producer-consumer streaming patterns.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use crate::error::{SynthError, SynthResult};
use crate::traits::{BackpressureStrategy, StreamEvent};

/// Statistics for a streaming channel.
#[derive(Debug, Clone, Default)]
pub struct ChannelStats {
    /// Total items sent through the channel.
    pub items_sent: u64,
    /// Total items received from the channel.
    pub items_received: u64,
    /// Items dropped due to backpressure.
    pub items_dropped: u64,
    /// Current buffer size.
    pub buffer_size: usize,
    /// Maximum buffer size reached.
    pub max_buffer_size: usize,
    /// Times sender blocked waiting for space.
    pub send_blocks: u64,
    /// Times receiver blocked waiting for items.
    pub receive_blocks: u64,
}

/// A bounded channel with configurable backpressure handling.
pub struct BoundedChannel<T> {
    /// Internal state protected by mutex.
    inner: Arc<ChannelInner<T>>,
    /// Channel capacity.
    capacity: usize,
    /// Backpressure strategy.
    strategy: BackpressureStrategy,
}

struct ChannelInner<T> {
    /// The buffer of items.
    buffer: Mutex<VecDeque<T>>,
    /// Condition variable for waiting senders.
    not_full: Condvar,
    /// Condition variable for waiting receivers.
    not_empty: Condvar,
    /// Whether the channel is closed.
    closed: AtomicBool,
    /// Statistics.
    items_sent: AtomicU64,
    items_received: AtomicU64,
    items_dropped: AtomicU64,
    send_blocks: AtomicU64,
    receive_blocks: AtomicU64,
    max_buffer_size: AtomicU64,
}

impl<T> BoundedChannel<T> {
    /// Creates a new bounded channel with the given capacity and backpressure strategy.
    pub fn new(capacity: usize, strategy: BackpressureStrategy) -> Self {
        Self {
            inner: Arc::new(ChannelInner {
                buffer: Mutex::new(VecDeque::with_capacity(capacity)),
                not_full: Condvar::new(),
                not_empty: Condvar::new(),
                closed: AtomicBool::new(false),
                items_sent: AtomicU64::new(0),
                items_received: AtomicU64::new(0),
                items_dropped: AtomicU64::new(0),
                send_blocks: AtomicU64::new(0),
                receive_blocks: AtomicU64::new(0),
                max_buffer_size: AtomicU64::new(0),
            }),
            capacity,
            strategy,
        }
    }

    /// Sends an item through the channel.
    ///
    /// Returns `Ok(true)` if the item was sent, `Ok(false)` if it was dropped,
    /// or `Err` if the channel is closed.
    pub fn send(&self, item: T) -> SynthResult<bool> {
        if self.inner.closed.load(Ordering::SeqCst) {
            return Err(SynthError::ChannelClosed);
        }

        let mut buffer = self.inner.buffer.lock().expect("mutex poisoned");

        // Check if buffer is full
        if buffer.len() >= self.capacity {
            match self.strategy {
                BackpressureStrategy::Block => {
                    self.inner.send_blocks.fetch_add(1, Ordering::Relaxed);
                    // Wait until space is available
                    buffer = self
                        .inner
                        .not_full
                        .wait_while(buffer, |b| {
                            b.len() >= self.capacity && !self.inner.closed.load(Ordering::SeqCst)
                        })
                        .expect("condvar wait");

                    if self.inner.closed.load(Ordering::SeqCst) {
                        return Err(SynthError::ChannelClosed);
                    }
                }
                BackpressureStrategy::DropOldest => {
                    // Drop oldest item to make room
                    buffer.pop_front();
                    self.inner.items_dropped.fetch_add(1, Ordering::Relaxed);
                }
                BackpressureStrategy::DropNewest => {
                    // Don't add the new item
                    self.inner.items_dropped.fetch_add(1, Ordering::Relaxed);
                    return Ok(false);
                }
                BackpressureStrategy::Buffer { max_overflow } => {
                    // Allow overflow up to max_overflow
                    if buffer.len() >= self.capacity + max_overflow {
                        self.inner.send_blocks.fetch_add(1, Ordering::Relaxed);
                        buffer = self
                            .inner
                            .not_full
                            .wait_while(buffer, |b| {
                                b.len() >= self.capacity + max_overflow
                                    && !self.inner.closed.load(Ordering::SeqCst)
                            })
                            .expect("condvar wait");

                        if self.inner.closed.load(Ordering::SeqCst) {
                            return Err(SynthError::ChannelClosed);
                        }
                    }
                }
            }
        }

        buffer.push_back(item);
        let current_size = buffer.len() as u64;
        self.inner.items_sent.fetch_add(1, Ordering::Relaxed);

        // Update max buffer size
        let mut max_size = self.inner.max_buffer_size.load(Ordering::Relaxed);
        while current_size > max_size {
            match self.inner.max_buffer_size.compare_exchange_weak(
                max_size,
                current_size,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => max_size = x,
            }
        }

        drop(buffer);
        self.inner.not_empty.notify_one();

        Ok(true)
    }

    /// Sends an item with a timeout.
    pub fn send_timeout(&self, item: T, timeout: Duration) -> SynthResult<bool> {
        if self.inner.closed.load(Ordering::SeqCst) {
            return Err(SynthError::ChannelClosed);
        }

        let deadline = Instant::now() + timeout;
        let mut buffer = self.inner.buffer.lock().expect("mutex poisoned");

        // Check if buffer is full
        while buffer.len() >= self.capacity {
            if self.inner.closed.load(Ordering::SeqCst) {
                return Err(SynthError::ChannelClosed);
            }

            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                // Timeout - apply strategy
                match self.strategy {
                    BackpressureStrategy::DropNewest => {
                        self.inner.items_dropped.fetch_add(1, Ordering::Relaxed);
                        return Ok(false);
                    }
                    BackpressureStrategy::DropOldest => {
                        buffer.pop_front();
                        self.inner.items_dropped.fetch_add(1, Ordering::Relaxed);
                        break;
                    }
                    _ => {
                        return Err(SynthError::GenerationError("send timeout".to_string()));
                    }
                }
            }

            let (new_buffer, wait_result) = self
                .inner
                .not_full
                .wait_timeout(buffer, remaining)
                .expect("condvar wait");
            buffer = new_buffer;

            if wait_result.timed_out() && buffer.len() >= self.capacity {
                match self.strategy {
                    BackpressureStrategy::DropNewest => {
                        self.inner.items_dropped.fetch_add(1, Ordering::Relaxed);
                        return Ok(false);
                    }
                    BackpressureStrategy::DropOldest => {
                        buffer.pop_front();
                        self.inner.items_dropped.fetch_add(1, Ordering::Relaxed);
                        break;
                    }
                    _ => {
                        return Err(SynthError::GenerationError("send timeout".to_string()));
                    }
                }
            }
        }

        buffer.push_back(item);
        self.inner.items_sent.fetch_add(1, Ordering::Relaxed);
        drop(buffer);
        self.inner.not_empty.notify_one();

        Ok(true)
    }

    /// Receives an item from the channel.
    ///
    /// Returns `None` if the channel is closed and empty.
    pub fn recv(&self) -> Option<T> {
        let mut buffer = self.inner.buffer.lock().expect("mutex poisoned");

        while buffer.is_empty() {
            if self.inner.closed.load(Ordering::SeqCst) {
                return None;
            }
            self.inner.receive_blocks.fetch_add(1, Ordering::Relaxed);
            buffer = self.inner.not_empty.wait(buffer).expect("condvar wait");
        }

        let item = buffer.pop_front();
        if item.is_some() {
            self.inner.items_received.fetch_add(1, Ordering::Relaxed);
        }
        drop(buffer);
        self.inner.not_full.notify_one();

        item
    }

    /// Receives an item with a timeout.
    pub fn recv_timeout(&self, timeout: Duration) -> Option<T> {
        let deadline = Instant::now() + timeout;
        let mut buffer = self.inner.buffer.lock().expect("mutex poisoned");

        while buffer.is_empty() {
            if self.inner.closed.load(Ordering::SeqCst) {
                return None;
            }

            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return None;
            }

            let (new_buffer, wait_result) = self
                .inner
                .not_empty
                .wait_timeout(buffer, remaining)
                .expect("condvar wait");
            buffer = new_buffer;

            if wait_result.timed_out() && buffer.is_empty() {
                return None;
            }
        }

        let item = buffer.pop_front();
        if item.is_some() {
            self.inner.items_received.fetch_add(1, Ordering::Relaxed);
        }
        drop(buffer);
        self.inner.not_full.notify_one();

        item
    }

    /// Tries to receive an item without blocking.
    pub fn try_recv(&self) -> Option<T> {
        let mut buffer = self.inner.buffer.lock().expect("mutex poisoned");
        let item = buffer.pop_front();
        if item.is_some() {
            self.inner.items_received.fetch_add(1, Ordering::Relaxed);
            drop(buffer);
            self.inner.not_full.notify_one();
        }
        item
    }

    /// Closes the channel.
    pub fn close(&self) {
        self.inner.closed.store(true, Ordering::SeqCst);
        self.inner.not_full.notify_all();
        self.inner.not_empty.notify_all();
    }

    /// Returns whether the channel is closed.
    pub fn is_closed(&self) -> bool {
        self.inner.closed.load(Ordering::SeqCst)
    }

    /// Returns the current number of items in the buffer.
    pub fn len(&self) -> usize {
        self.inner.buffer.lock().expect("mutex poisoned").len()
    }

    /// Returns whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the channel capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns channel statistics.
    pub fn stats(&self) -> ChannelStats {
        ChannelStats {
            items_sent: self.inner.items_sent.load(Ordering::Relaxed),
            items_received: self.inner.items_received.load(Ordering::Relaxed),
            items_dropped: self.inner.items_dropped.load(Ordering::Relaxed),
            buffer_size: self.len(),
            max_buffer_size: self.inner.max_buffer_size.load(Ordering::Relaxed) as usize,
            send_blocks: self.inner.send_blocks.load(Ordering::Relaxed),
            receive_blocks: self.inner.receive_blocks.load(Ordering::Relaxed),
        }
    }
}

impl<T> Clone for BoundedChannel<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            capacity: self.capacity,
            strategy: self.strategy,
        }
    }
}

/// Creates a stream event channel pair.
pub fn stream_channel<T>(
    capacity: usize,
    strategy: BackpressureStrategy,
) -> (StreamSender<T>, StreamReceiver<T>) {
    let channel = BoundedChannel::new(capacity, strategy);
    (
        StreamSender {
            channel: channel.clone(),
        },
        StreamReceiver { channel },
    )
}

/// Sender side of a stream event channel.
pub struct StreamSender<T> {
    channel: BoundedChannel<StreamEvent<T>>,
}

impl<T> StreamSender<T> {
    /// Sends a stream event.
    pub fn send(&self, event: StreamEvent<T>) -> SynthResult<bool> {
        self.channel.send(event)
    }

    /// Sends a data item.
    pub fn send_data(&self, item: T) -> SynthResult<bool> {
        self.channel.send(StreamEvent::Data(item))
    }

    /// Closes the sender.
    pub fn close(&self) {
        self.channel.close();
    }

    /// Returns channel statistics.
    pub fn stats(&self) -> ChannelStats {
        self.channel.stats()
    }
}

impl<T> Clone for StreamSender<T> {
    fn clone(&self) -> Self {
        Self {
            channel: self.channel.clone(),
        }
    }
}

/// Receiver side of a stream event channel.
pub struct StreamReceiver<T> {
    channel: BoundedChannel<StreamEvent<T>>,
}

impl<T> StreamReceiver<T> {
    /// Receives the next stream event.
    pub fn recv(&self) -> Option<StreamEvent<T>> {
        self.channel.recv()
    }

    /// Receives with timeout.
    pub fn recv_timeout(&self, timeout: Duration) -> Option<StreamEvent<T>> {
        self.channel.recv_timeout(timeout)
    }

    /// Tries to receive without blocking.
    pub fn try_recv(&self) -> Option<StreamEvent<T>> {
        self.channel.try_recv()
    }

    /// Returns whether the channel is closed.
    pub fn is_closed(&self) -> bool {
        self.channel.is_closed()
    }

    /// Returns channel statistics.
    pub fn stats(&self) -> ChannelStats {
        self.channel.stats()
    }
}

impl<T> Iterator for StreamReceiver<T> {
    type Item = StreamEvent<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.recv()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_bounded_channel_basic() {
        let channel: BoundedChannel<i32> = BoundedChannel::new(10, BackpressureStrategy::Block);

        channel.send(1).unwrap();
        channel.send(2).unwrap();
        channel.send(3).unwrap();

        assert_eq!(channel.recv(), Some(1));
        assert_eq!(channel.recv(), Some(2));
        assert_eq!(channel.recv(), Some(3));
    }

    #[test]
    fn test_bounded_channel_drop_oldest() {
        let channel: BoundedChannel<i32> = BoundedChannel::new(2, BackpressureStrategy::DropOldest);

        channel.send(1).unwrap();
        channel.send(2).unwrap();
        channel.send(3).unwrap(); // Should drop 1

        let stats = channel.stats();
        assert_eq!(stats.items_dropped, 1);
        assert_eq!(channel.recv(), Some(2));
        assert_eq!(channel.recv(), Some(3));
    }

    #[test]
    fn test_bounded_channel_drop_newest() {
        let channel: BoundedChannel<i32> = BoundedChannel::new(2, BackpressureStrategy::DropNewest);

        channel.send(1).unwrap();
        channel.send(2).unwrap();
        let sent = channel.send(3).unwrap(); // Should be dropped

        assert!(!sent);
        let stats = channel.stats();
        assert_eq!(stats.items_dropped, 1);
    }

    #[test]
    fn test_bounded_channel_close() {
        let channel: BoundedChannel<i32> = BoundedChannel::new(10, BackpressureStrategy::Block);

        channel.send(1).unwrap();
        channel.close();

        assert_eq!(channel.recv(), Some(1));
        assert_eq!(channel.recv(), None);
        assert!(channel.send(2).is_err());
    }

    #[test]
    fn test_bounded_channel_threaded() {
        let channel: BoundedChannel<i32> = BoundedChannel::new(10, BackpressureStrategy::Block);
        let sender = channel.clone();

        let handle = thread::spawn(move || {
            for i in 0..100 {
                sender.send(i).unwrap();
            }
            sender.close();
        });

        let mut received = Vec::new();
        while let Some(item) = channel.recv() {
            received.push(item);
        }

        handle.join().unwrap();

        assert_eq!(received, (0..100).collect::<Vec<_>>());
    }

    #[test]
    fn test_stream_channel() {
        let (sender, receiver) = stream_channel::<i32>(10, BackpressureStrategy::Block);

        sender.send_data(1).unwrap();
        sender.send_data(2).unwrap();
        sender.close();

        let events: Vec<_> = receiver.collect();
        assert_eq!(events.len(), 2);

        assert!(matches!(events[0], StreamEvent::Data(1)));
        assert!(matches!(events[1], StreamEvent::Data(2)));
    }

    #[test]
    fn test_channel_stats() {
        let channel: BoundedChannel<i32> = BoundedChannel::new(10, BackpressureStrategy::Block);

        channel.send(1).unwrap();
        channel.send(2).unwrap();
        channel.recv();

        let stats = channel.stats();
        assert_eq!(stats.items_sent, 2);
        assert_eq!(stats.items_received, 1);
        assert_eq!(stats.buffer_size, 1);
    }
}
