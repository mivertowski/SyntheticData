//! Scalability benchmarks for memory and large volume generation.
//!
//! Tests memory efficiency and performance at scale.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;
use std::sync::Arc;

use datasynth_config::schema::TransactionConfig;
use datasynth_core::models::{CoAComplexity, IndustrySector, JournalEntry};
use datasynth_generators::{ChartOfAccountsGenerator, JournalEntryGenerator};

mod common;
use common::*;

/// Benchmark memory efficiency at different scales.
///
/// Note: This doesn't directly measure memory, but exercises the generation
/// at different scales to allow profiling with external tools.
fn bench_scale_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("scale_generation");
    group.sample_size(10); // Fewer samples for large tests

    let coa = small_coa();

    for count in [1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*count as u64));

        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &size| {
            b.iter_with_setup(
                || create_je_generator(Arc::clone(&coa)),
                |mut gen| {
                    let mut entries = Vec::with_capacity(size);
                    for _ in 0..size {
                        entries.push(gen.generate());
                    }
                    black_box(entries)
                },
            );
        });
    }

    group.finish();
}

/// Benchmark streaming generation (without storing all entries).
fn bench_streaming_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming_generation");
    group.sample_size(10);

    let coa = small_coa();

    for count in [10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*count as u64));

        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &size| {
            b.iter_with_setup(
                || create_je_generator(Arc::clone(&coa)),
                |mut gen| {
                    let mut total_lines = 0u64;
                    for _ in 0..size {
                        let entry = gen.generate();
                        total_lines += entry.line_count() as u64;
                        // Don't store - just count
                    }
                    black_box(total_lines)
                },
            );
        });
    }

    group.finish();
}

/// Benchmark with different CoA sizes to test scaling.
fn bench_coa_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("coa_scaling");
    let batch_size = 10_000;

    group.throughput(Throughput::Elements(batch_size as u64));
    group.sample_size(10);

    // Benchmark account lookup time by using larger CoAs
    let configs = [
        ("small_100", CoAComplexity::Small),
        ("medium_400", CoAComplexity::Medium),
        ("large_2500", CoAComplexity::Large),
    ];

    for (name, complexity) in configs {
        let mut coa_gen = ChartOfAccountsGenerator::new(
            complexity,
            IndustrySector::Manufacturing,
            BENCHMARK_SEED,
        );
        let coa = Arc::new(coa_gen.generate());

        group.bench_function(name, |b| {
            b.iter_with_setup(
                || create_je_generator(Arc::clone(&coa)),
                |mut gen| {
                    for _ in 0..batch_size {
                        black_box(gen.generate());
                    }
                },
            );
        });
    }

    group.finish();
}

/// Benchmark multi-company scaling.
fn bench_company_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("company_scaling");
    let batch_size = 10_000;

    group.throughput(Throughput::Elements(batch_size as u64));

    let coa = small_coa();

    for num_companies in [1, 3, 10].iter() {
        let companies: Vec<String> = (1..=*num_companies)
            .map(|i| format!("{:04}", i * 1000))
            .collect();

        group.bench_with_input(
            BenchmarkId::from_parameter(num_companies),
            &companies,
            |b, companies| {
                b.iter_with_setup(
                    || {
                        JournalEntryGenerator::new_with_params(
                            TransactionConfig::default(),
                            Arc::clone(&coa),
                            companies.clone(),
                            start_date(),
                            end_date(),
                            BENCHMARK_SEED,
                        )
                        .with_persona_errors(false)
                        .with_approval(false)
                    },
                    |mut gen| {
                        for _ in 0..batch_size {
                            black_box(gen.generate());
                        }
                    },
                );
            },
        );
    }

    group.finish();
}

/// Benchmark entry size distribution.
fn bench_entry_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("entry_sizes");
    let batch_size = 10_000;

    let coa = small_coa();
    let mut gen = create_je_generator(Arc::clone(&coa));

    // Generate entries and categorize by size
    let entries: Vec<JournalEntry> = (0..batch_size).map(|_| gen.generate()).collect();

    let small_entries: Vec<_> = entries.iter().filter(|e| e.line_count() == 2).collect();
    let medium_entries: Vec<_> = entries
        .iter()
        .filter(|e| e.line_count() > 2 && e.line_count() <= 8)
        .collect();
    let large_entries: Vec<_> = entries.iter().filter(|e| e.line_count() > 8).collect();

    // Measure processing time for different entry sizes
    if !small_entries.is_empty() {
        group.throughput(Throughput::Elements(small_entries.len() as u64));
        group.bench_function("small_2_lines", |b| {
            b.iter(|| {
                for entry in &small_entries {
                    black_box(entry.is_balanced());
                    black_box(entry.total_debit());
                }
            });
        });
    }

    if !medium_entries.is_empty() {
        group.throughput(Throughput::Elements(medium_entries.len() as u64));
        group.bench_function("medium_3_8_lines", |b| {
            b.iter(|| {
                for entry in &medium_entries {
                    black_box(entry.is_balanced());
                    black_box(entry.total_debit());
                }
            });
        });
    }

    if !large_entries.is_empty() {
        group.throughput(Throughput::Elements(large_entries.len() as u64));
        group.bench_function("large_9plus_lines", |b| {
            b.iter(|| {
                for entry in &large_entries {
                    black_box(entry.is_balanced());
                    black_box(entry.total_debit());
                }
            });
        });
    }

    group.finish();
}

/// Benchmark memory usage by measuring allocation patterns.
fn bench_allocation_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation_patterns");
    group.sample_size(10);

    let coa = small_coa();

    // Pre-allocated vector
    group.bench_function("preallocated", |b| {
        b.iter_with_setup(
            || create_je_generator(Arc::clone(&coa)),
            |mut gen| {
                let mut entries = Vec::with_capacity(10_000);
                for _ in 0..10_000 {
                    entries.push(gen.generate());
                }
                black_box(entries)
            },
        );
    });

    // Growing vector (simulates unknown size)
    group.bench_function("growing", |b| {
        b.iter_with_setup(
            || create_je_generator(Arc::clone(&coa)),
            |mut gen| {
                let mut entries = Vec::new();
                for _ in 0..10_000 {
                    entries.push(gen.generate());
                }
                black_box(entries)
            },
        );
    });

    group.finish();
}

/// Benchmark sustained generation (simulates long-running generation).
fn bench_sustained_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("sustained_generation");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(10));

    let coa = small_coa();

    group.throughput(Throughput::Elements(50_000));

    group.bench_function("50k_entries", |b| {
        b.iter_with_setup(
            || create_je_generator(Arc::clone(&coa)),
            |mut gen| {
                let mut total_amount = rust_decimal::Decimal::ZERO;
                for _ in 0..50_000 {
                    let entry = gen.generate();
                    total_amount += entry.total_debit();
                }
                black_box(total_amount)
            },
        );
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_scale_generation,
    bench_streaming_generation,
    bench_coa_scaling,
    bench_company_scaling,
    bench_entry_sizes,
    bench_allocation_patterns,
    bench_sustained_generation,
);

criterion_main!(benches);
