//! Core Generator trait for data generation.
//!
//! Defines the interface that all data generators must implement,
//! supporting both batch and streaming generation patterns.

// Error types are available via crate::error if needed

/// Core trait for all data generators.
///
/// Generators produce synthetic data items based on configuration and
/// statistical distributions. They support deterministic generation
/// via seeding for reproducibility.
pub trait Generator {
    /// The type of items this generator produces.
    type Item: Clone + Send;

    /// The configuration type for this generator.
    type Config: Clone + Send + Sync;

    /// Initialize the generator with configuration and seed.
    ///
    /// The seed ensures deterministic, reproducible generation.
    fn new(config: Self::Config, seed: u64) -> Self
    where
        Self: Sized;

    /// Generate a single item.
    fn generate_one(&mut self) -> Self::Item;

    /// Generate a batch of items.
    ///
    /// Default implementation calls generate_one repeatedly.
    fn generate_batch(&mut self, count: usize) -> Vec<Self::Item> {
        (0..count).map(|_| self.generate_one()).collect()
    }

    /// Generate items into an iterator.
    ///
    /// Useful for lazy evaluation and streaming.
    fn generate_iter(&mut self, count: usize) -> GeneratorIterator<'_, Self>
    where
        Self: Sized,
    {
        GeneratorIterator {
            generator: self,
            remaining: count,
        }
    }

    /// Reset the generator to initial state (same seed).
    ///
    /// After reset, the generator will produce the same sequence of items.
    fn reset(&mut self);

    /// Get the current generation count.
    fn count(&self) -> u64;

    /// Get the seed used by this generator.
    fn seed(&self) -> u64;
}

/// Iterator adapter for generators.
pub struct GeneratorIterator<'a, G: Generator> {
    generator: &'a mut G,
    remaining: usize,
}

impl<'a, G: Generator> Iterator for GeneratorIterator<'a, G> {
    type Item = G::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining > 0 {
            self.remaining -= 1;
            Some(self.generator.generate_one())
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<'a, G: Generator> ExactSizeIterator for GeneratorIterator<'a, G> {}

/// Trait for generators that can be parallelized.
///
/// Allows splitting a generator into multiple independent generators
/// for parallel execution.
pub trait ParallelGenerator: Generator + Sized {
    /// Split the generator into multiple independent generators.
    ///
    /// Each split generator will produce a portion of the total items.
    /// The splits should be deterministic based on the original seed.
    fn split(self, parts: usize) -> Vec<Self>;

    /// Merge results from parallel execution.
    ///
    /// Combines results from multiple generators into a single sequence.
    fn merge_results(results: Vec<Vec<Self::Item>>) -> Vec<Self::Item> {
        results.into_iter().flatten().collect()
    }
}

/// Progress information for long-running generation.
#[derive(Debug, Clone)]
pub struct GenerationProgress {
    /// Total items to generate.
    pub total: u64,
    /// Items generated so far.
    pub completed: u64,
    /// Items per second throughput.
    pub items_per_second: f64,
    /// Estimated seconds remaining.
    pub eta_seconds: Option<u64>,
    /// Current phase/stage description.
    pub phase: String,
}

impl GenerationProgress {
    /// Create a new progress tracker.
    pub fn new(total: u64) -> Self {
        Self {
            total,
            completed: 0,
            items_per_second: 0.0,
            eta_seconds: None,
            phase: String::new(),
        }
    }

    /// Get progress as a percentage (0.0 to 1.0).
    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            1.0
        } else {
            self.completed as f64 / self.total as f64
        }
    }

    /// Check if generation is complete.
    pub fn is_complete(&self) -> bool {
        self.completed >= self.total
    }
}

/// Trait for components that can report progress.
pub trait ProgressReporter {
    /// Report current progress.
    fn report_progress(&self, progress: &GenerationProgress);
}

/// No-op progress reporter for when progress tracking is not needed.
pub struct NoopProgressReporter;

impl ProgressReporter for NoopProgressReporter {
    fn report_progress(&self, _progress: &GenerationProgress) {}
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    struct SimpleGenerator {
        seed: u64,
        count: u64,
        value: u64,
    }

    impl Generator for SimpleGenerator {
        type Item = u64;
        type Config = ();

        fn new(_config: Self::Config, seed: u64) -> Self {
            Self {
                seed,
                count: 0,
                value: seed,
            }
        }

        fn generate_one(&mut self) -> Self::Item {
            self.count += 1;
            self.value = self.value.wrapping_mul(6364136223846793005).wrapping_add(1);
            self.value
        }

        fn reset(&mut self) {
            self.count = 0;
            self.value = self.seed;
        }

        fn count(&self) -> u64 {
            self.count
        }

        fn seed(&self) -> u64 {
            self.seed
        }
    }

    #[test]
    fn test_generator_batch() {
        let mut gen = SimpleGenerator::new((), 42);
        let batch = gen.generate_batch(10);
        assert_eq!(batch.len(), 10);
        assert_eq!(gen.count(), 10);
    }

    #[test]
    fn test_generator_determinism() {
        let mut gen1 = SimpleGenerator::new((), 42);
        let mut gen2 = SimpleGenerator::new((), 42);

        for _ in 0..100 {
            assert_eq!(gen1.generate_one(), gen2.generate_one());
        }
    }

    #[test]
    fn test_generator_reset() {
        let mut gen = SimpleGenerator::new((), 42);
        let first_run: Vec<_> = gen.generate_iter(10).collect();

        gen.reset();
        let second_run: Vec<_> = gen.generate_iter(10).collect();

        assert_eq!(first_run, second_run);
    }
}
