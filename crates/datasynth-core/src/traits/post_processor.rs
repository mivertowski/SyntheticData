//! Post-processor trait for data quality variations and other post-generation transformations.
//!
//! Post-processors modify records after generation to inject data quality issues,
//! format variations, typos, and other realistic flakiness. They produce labels
//! that can be used for ML training.

use crate::error::SynthResult;
use std::collections::HashMap;

/// Context passed to post-processors during processing.
#[derive(Debug, Clone, Default)]
pub struct ProcessContext {
    /// Current record index in the batch
    pub record_index: usize,
    /// Total records in the batch
    pub batch_size: usize,
    /// Current output format (csv, json, parquet)
    pub output_format: Option<String>,
    /// Additional context data
    pub metadata: HashMap<String, String>,
}

impl ProcessContext {
    /// Create a new processing context.
    pub fn new(record_index: usize, batch_size: usize) -> Self {
        Self {
            record_index,
            batch_size,
            output_format: None,
            metadata: HashMap::new(),
        }
    }

    /// Set the output format.
    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.output_format = Some(format.into());
        self
    }

    /// Add metadata.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Check if processing first record.
    pub fn is_first(&self) -> bool {
        self.record_index == 0
    }

    /// Check if processing last record.
    pub fn is_last(&self) -> bool {
        self.record_index == self.batch_size.saturating_sub(1)
    }
}

/// Statistics from a post-processor run.
#[derive(Debug, Clone, Default)]
pub struct ProcessorStats {
    /// Number of records processed
    pub records_processed: u64,
    /// Number of records modified
    pub records_modified: u64,
    /// Number of labels generated
    pub labels_generated: u64,
    /// Number of errors encountered
    pub errors_encountered: u64,
    /// Processing time in microseconds
    pub processing_time_us: u64,
}

impl ProcessorStats {
    /// Calculate modification rate.
    pub fn modification_rate(&self) -> f64 {
        if self.records_processed == 0 {
            0.0
        } else {
            self.records_modified as f64 / self.records_processed as f64
        }
    }

    /// Merge stats from another processor.
    pub fn merge(&mut self, other: &ProcessorStats) {
        self.records_processed += other.records_processed;
        self.records_modified += other.records_modified;
        self.labels_generated += other.labels_generated;
        self.errors_encountered += other.errors_encountered;
        self.processing_time_us += other.processing_time_us;
    }
}

/// Core trait for post-processors that modify records and generate labels.
///
/// Post-processors are applied after generation to inject realistic data quality
/// issues. Each processor can modify records in place and generate labels
/// describing the modifications for ML training.
pub trait PostProcessor: Send + Sync {
    /// The type of records this processor modifies.
    type Record;
    /// The type of labels this processor produces.
    type Label;

    /// Process a single record, potentially modifying it and generating labels.
    ///
    /// Returns a vector of labels describing any modifications made.
    fn process(
        &mut self,
        record: &mut Self::Record,
        context: &ProcessContext,
    ) -> SynthResult<Vec<Self::Label>>;

    /// Process a batch of records.
    ///
    /// Default implementation calls process for each record.
    fn process_batch(
        &mut self,
        records: &mut [Self::Record],
        base_context: &ProcessContext,
    ) -> SynthResult<Vec<Self::Label>> {
        let mut all_labels = Vec::new();
        let batch_size = records.len();

        for (i, record) in records.iter_mut().enumerate() {
            let context = ProcessContext {
                record_index: i,
                batch_size,
                output_format: base_context.output_format.clone(),
                metadata: base_context.metadata.clone(),
            };
            let labels = self.process(record, &context)?;
            all_labels.extend(labels);
        }

        Ok(all_labels)
    }

    /// Get the name of this processor.
    fn name(&self) -> &'static str;

    /// Check if this processor is enabled.
    fn is_enabled(&self) -> bool;

    /// Get processing statistics.
    fn stats(&self) -> ProcessorStats;

    /// Reset statistics (for testing or between batches).
    fn reset_stats(&mut self);
}

/// A pipeline of post-processors applied in sequence.
pub struct PostProcessorPipeline<R, L> {
    processors: Vec<Box<dyn PostProcessor<Record = R, Label = L>>>,
    stats: ProcessorStats,
}

impl<R, L> PostProcessorPipeline<R, L> {
    /// Create a new empty pipeline.
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
            stats: ProcessorStats::default(),
        }
    }

    /// Add a processor to the pipeline.
    pub fn add<P>(&mut self, processor: P)
    where
        P: PostProcessor<Record = R, Label = L> + 'static,
    {
        self.processors.push(Box::new(processor));
    }

    /// Add a processor and return self for chaining.
    pub fn with<P>(mut self, processor: P) -> Self
    where
        P: PostProcessor<Record = R, Label = L> + 'static,
    {
        self.add(processor);
        self
    }

    /// Process a single record through all processors.
    pub fn process(&mut self, record: &mut R, context: &ProcessContext) -> SynthResult<Vec<L>> {
        let mut all_labels = Vec::new();

        for processor in &mut self.processors {
            if processor.is_enabled() {
                let labels = processor.process(record, context)?;
                all_labels.extend(labels);
            }
        }

        self.stats.records_processed += 1;
        if !all_labels.is_empty() {
            self.stats.records_modified += 1;
        }
        self.stats.labels_generated += all_labels.len() as u64;

        Ok(all_labels)
    }

    /// Process a batch of records through all processors.
    pub fn process_batch(
        &mut self,
        records: &mut [R],
        base_context: &ProcessContext,
    ) -> SynthResult<Vec<L>> {
        let mut all_labels = Vec::new();
        let batch_size = records.len();

        for (i, record) in records.iter_mut().enumerate() {
            let context = ProcessContext {
                record_index: i,
                batch_size,
                output_format: base_context.output_format.clone(),
                metadata: base_context.metadata.clone(),
            };
            let labels = self.process(record, &context)?;
            all_labels.extend(labels);
        }

        Ok(all_labels)
    }

    /// Get aggregate statistics for the pipeline.
    ///
    /// Returns the pipeline's own stats tracking records processed through
    /// the entire pipeline. Use `processor_stats()` to get individual
    /// processor statistics.
    pub fn stats(&self) -> ProcessorStats {
        self.stats.clone()
    }

    /// Get individual processor statistics.
    pub fn processor_stats(&self) -> Vec<(&'static str, ProcessorStats)> {
        self.processors
            .iter()
            .map(|p| (p.name(), p.stats()))
            .collect()
    }

    /// Check if pipeline has any enabled processors.
    pub fn has_enabled_processors(&self) -> bool {
        self.processors.iter().any(|p| p.is_enabled())
    }

    /// Get number of processors in the pipeline.
    pub fn len(&self) -> usize {
        self.processors.len()
    }

    /// Check if pipeline is empty.
    pub fn is_empty(&self) -> bool {
        self.processors.is_empty()
    }

    /// Reset all statistics.
    pub fn reset_stats(&mut self) {
        self.stats = ProcessorStats::default();
        for processor in &mut self.processors {
            processor.reset_stats();
        }
    }
}

impl<R, L> Default for PostProcessorPipeline<R, L> {
    fn default() -> Self {
        Self::new()
    }
}

/// A no-op processor that passes records through unchanged.
pub struct PassthroughProcessor<R, L> {
    enabled: bool,
    stats: ProcessorStats,
    _phantom: std::marker::PhantomData<(R, L)>,
}

impl<R, L> PassthroughProcessor<R, L> {
    /// Create a new passthrough processor.
    pub fn new() -> Self {
        Self {
            enabled: true,
            stats: ProcessorStats::default(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Create a disabled passthrough processor.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            stats: ProcessorStats::default(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<R, L> Default for PassthroughProcessor<R, L> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R: Send + Sync, L: Send + Sync> PostProcessor for PassthroughProcessor<R, L> {
    type Record = R;
    type Label = L;

    fn process(
        &mut self,
        _record: &mut Self::Record,
        _context: &ProcessContext,
    ) -> SynthResult<Vec<Self::Label>> {
        self.stats.records_processed += 1;
        Ok(Vec::new())
    }

    fn name(&self) -> &'static str {
        "passthrough"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn stats(&self) -> ProcessorStats {
        self.stats.clone()
    }

    fn reset_stats(&mut self) {
        self.stats = ProcessorStats::default();
    }
}

/// Builder for creating post-processor pipelines.
pub struct PipelineBuilder<R, L> {
    pipeline: PostProcessorPipeline<R, L>,
}

impl<R, L> PipelineBuilder<R, L> {
    /// Create a new pipeline builder.
    pub fn new() -> Self {
        Self {
            pipeline: PostProcessorPipeline::new(),
        }
    }

    /// Add a processor to the pipeline.
    #[allow(clippy::should_implement_trait)]
    pub fn add<P>(mut self, processor: P) -> Self
    where
        P: PostProcessor<Record = R, Label = L> + 'static,
    {
        self.pipeline.add(processor);
        self
    }

    /// Conditionally add a processor.
    pub fn add_if<P>(mut self, condition: bool, processor: P) -> Self
    where
        P: PostProcessor<Record = R, Label = L> + 'static,
    {
        if condition {
            self.pipeline.add(processor);
        }
        self
    }

    /// Build the pipeline.
    pub fn build(self) -> PostProcessorPipeline<R, L> {
        self.pipeline
    }
}

impl<R, L> Default for PipelineBuilder<R, L> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // Simple test record type
    #[derive(Debug, Clone)]
    struct TestRecord {
        value: String,
    }

    // Simple test label type
    #[allow(dead_code)]
    #[derive(Debug, Clone)]
    struct TestLabel {
        field: String,
        change: String,
    }

    // Test processor that uppercases strings
    struct UppercaseProcessor {
        enabled: bool,
        stats: ProcessorStats,
    }

    impl UppercaseProcessor {
        fn new() -> Self {
            Self {
                enabled: true,
                stats: ProcessorStats::default(),
            }
        }
    }

    impl PostProcessor for UppercaseProcessor {
        type Record = TestRecord;
        type Label = TestLabel;

        fn process(
            &mut self,
            record: &mut Self::Record,
            _context: &ProcessContext,
        ) -> SynthResult<Vec<Self::Label>> {
            self.stats.records_processed += 1;
            let original = record.value.clone();
            record.value = record.value.to_uppercase();
            if record.value != original {
                self.stats.records_modified += 1;
                self.stats.labels_generated += 1;
                Ok(vec![TestLabel {
                    field: "value".to_string(),
                    change: format!("{} -> {}", original, record.value),
                }])
            } else {
                Ok(vec![])
            }
        }

        fn name(&self) -> &'static str {
            "uppercase"
        }

        fn is_enabled(&self) -> bool {
            self.enabled
        }

        fn stats(&self) -> ProcessorStats {
            self.stats.clone()
        }

        fn reset_stats(&mut self) {
            self.stats = ProcessorStats::default();
        }
    }

    #[test]
    fn test_pipeline_basic() {
        let mut pipeline = PostProcessorPipeline::new();
        pipeline.add(UppercaseProcessor::new());

        let mut record = TestRecord {
            value: "hello".to_string(),
        };
        let context = ProcessContext::new(0, 1);

        let labels = pipeline.process(&mut record, &context).unwrap();

        assert_eq!(record.value, "HELLO");
        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0].field, "value");
    }

    #[test]
    fn test_pipeline_batch() {
        let mut pipeline = PostProcessorPipeline::new();
        pipeline.add(UppercaseProcessor::new());

        let mut records = vec![
            TestRecord {
                value: "a".to_string(),
            },
            TestRecord {
                value: "b".to_string(),
            },
            TestRecord {
                value: "c".to_string(),
            },
        ];
        let context = ProcessContext::new(0, 3);

        let labels = pipeline.process_batch(&mut records, &context).unwrap();

        assert_eq!(records[0].value, "A");
        assert_eq!(records[1].value, "B");
        assert_eq!(records[2].value, "C");
        assert_eq!(labels.len(), 3);
    }

    #[test]
    fn test_pipeline_stats() {
        let mut pipeline = PostProcessorPipeline::new();
        pipeline.add(UppercaseProcessor::new());

        let context = ProcessContext::new(0, 1);

        for _ in 0..5 {
            let mut record = TestRecord {
                value: "test".to_string(),
            };
            let _ = pipeline.process(&mut record, &context);
        }

        let stats = pipeline.stats();
        assert_eq!(stats.records_processed, 5);
        assert_eq!(stats.records_modified, 5);
    }

    #[test]
    fn test_passthrough_processor() {
        let mut processor = PassthroughProcessor::<TestRecord, TestLabel>::new();
        let mut record = TestRecord {
            value: "unchanged".to_string(),
        };
        let context = ProcessContext::new(0, 1);

        let labels = processor.process(&mut record, &context).unwrap();

        assert_eq!(record.value, "unchanged");
        assert!(labels.is_empty());
    }

    #[test]
    fn test_pipeline_builder() {
        let pipeline: PostProcessorPipeline<TestRecord, TestLabel> = PipelineBuilder::new()
            .add(UppercaseProcessor::new())
            .add_if(false, PassthroughProcessor::new())
            .build();

        assert_eq!(pipeline.len(), 1);
    }
}
