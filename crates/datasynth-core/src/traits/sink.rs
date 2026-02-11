//! Output Sink trait for writing generated data.
//!
//! Defines the interface for output destinations including files,
//! streams, and databases.

use crate::error::SynthError;

/// Core trait for output sinks.
///
/// Sinks receive generated data and write it to a destination.
/// They handle batching, buffering, and format conversion.
pub trait Sink {
    /// The type of items this sink accepts.
    type Item;

    /// Write a single item to the sink.
    fn write(&mut self, item: Self::Item) -> Result<(), SynthError>;

    /// Write a batch of items to the sink.
    ///
    /// Default implementation calls write repeatedly.
    fn write_batch(&mut self, items: Vec<Self::Item>) -> Result<(), SynthError> {
        for item in items {
            self.write(item)?;
        }
        Ok(())
    }

    /// Flush any buffered data to the destination.
    fn flush(&mut self) -> Result<(), SynthError>;

    /// Close the sink and release resources.
    ///
    /// After calling close, the sink should not be used.
    fn close(self) -> Result<(), SynthError>
    where
        Self: Sized;

    /// Get the number of items written.
    fn items_written(&self) -> u64;

    /// Get the number of bytes written (if applicable).
    fn bytes_written(&self) -> Option<u64> {
        None
    }
}

/// A sink that discards all data (useful for benchmarking).
pub struct NullSink {
    count: u64,
}

impl NullSink {
    pub fn new() -> Self {
        Self { count: 0 }
    }
}

impl Default for NullSink {
    fn default() -> Self {
        Self::new()
    }
}

/// Implement Sink for any type that can be counted.
/// Usage: let mut sink: NullSink = NullSink::new();
///        sink.write_any(item);
impl NullSink {
    /// Write any item (type-erased counting).
    pub fn write_any<T>(&mut self, _item: T) {
        self.count += 1;
    }

    /// Get the number of items written.
    pub fn items_written(&self) -> u64 {
        self.count
    }
}

/// A sink that collects items into a vector.
pub struct VecSink<T> {
    items: Vec<T>,
}

impl<T> VecSink<T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
        }
    }

    /// Consume the sink and return collected items.
    pub fn into_items(self) -> Vec<T> {
        self.items
    }

    /// Get a reference to collected items.
    pub fn items(&self) -> &[T] {
        &self.items
    }
}

impl<T> Default for VecSink<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Sink for VecSink<T> {
    type Item = T;

    fn write(&mut self, item: Self::Item) -> Result<(), SynthError> {
        self.items.push(item);
        Ok(())
    }

    fn write_batch(&mut self, items: Vec<Self::Item>) -> Result<(), SynthError> {
        self.items.extend(items);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), SynthError> {
        Ok(())
    }

    fn close(self) -> Result<(), SynthError> {
        Ok(())
    }

    fn items_written(&self) -> u64 {
        self.items.len() as u64
    }
}

/// Trait for sinks that support partitioned output.
pub trait PartitionedSink: Sink {
    /// The partition key type.
    type PartitionKey;

    /// Write an item to a specific partition.
    fn write_to_partition(
        &mut self,
        partition: Self::PartitionKey,
        item: Self::Item,
    ) -> Result<(), SynthError>;

    /// Flush a specific partition.
    fn flush_partition(&mut self, partition: Self::PartitionKey) -> Result<(), SynthError>;
}

/// Configuration for buffered sinks.
#[derive(Debug, Clone)]
pub struct SinkBufferConfig {
    /// Maximum number of items to buffer before flushing.
    pub max_items: usize,
    /// Maximum bytes to buffer before flushing (if applicable).
    pub max_bytes: Option<usize>,
    /// Flush on every write (for debugging).
    pub flush_on_write: bool,
}

impl Default for SinkBufferConfig {
    fn default() -> Self {
        Self {
            max_items: 10_000,
            max_bytes: Some(64 * 1024 * 1024), // 64MB
            flush_on_write: false,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_null_sink() {
        let mut sink = NullSink::new();
        sink.write_any(42);
        sink.write_any(43);
        assert_eq!(sink.items_written(), 2);
    }

    #[test]
    fn test_vec_sink() {
        let mut sink = VecSink::new();
        sink.write(1).unwrap();
        sink.write(2).unwrap();
        sink.write(3).unwrap();

        assert_eq!(sink.items_written(), 3);
        assert_eq!(sink.into_items(), vec![1, 2, 3]);
    }

    #[test]
    fn test_vec_sink_batch() {
        let mut sink = VecSink::new();
        sink.write_batch(vec![1, 2, 3]).unwrap();
        sink.write_batch(vec![4, 5]).unwrap();

        assert_eq!(sink.items_written(), 5);
        assert_eq!(sink.into_items(), vec![1, 2, 3, 4, 5]);
    }
}
