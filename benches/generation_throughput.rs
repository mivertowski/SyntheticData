//! Throughput benchmarks for journal entry and master data generation.
//!
//! Target: 100K+ entries/second, 300K+ line items/second.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;
use std::sync::Arc;

use datasynth_config::schema::TransactionConfig;
use datasynth_core::models::{CoAComplexity, IndustrySector};
use datasynth_generators::{
    AssetGenerator, ChartOfAccountsGenerator, CustomerGenerator, JournalEntryGenerator,
    MaterialGenerator, VendorGenerator,
};

mod common;
use common::*;

/// Benchmark journal entry generation at different batch sizes.
fn bench_je_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("je_generation");

    let coa = small_coa();

    for batch_size in [100, 1_000, 10_000].iter() {
        group.throughput(Throughput::Elements(*batch_size as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            batch_size,
            |b, &size| {
                b.iter_with_setup(
                    || create_je_generator(Arc::clone(&coa)),
                    |mut gen| {
                        for _ in 0..size {
                            black_box(gen.generate());
                        }
                    },
                );
            },
        );
    }

    group.finish();
}

/// Benchmark journal entry generation with different CoA complexities.
fn bench_je_by_coa_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("je_by_coa_complexity");
    let batch_size = 1_000;

    group.throughput(Throughput::Elements(batch_size as u64));

    // Small CoA (~100 accounts)
    let small = small_coa();
    group.bench_function("small_coa", |b| {
        b.iter_with_setup(
            || create_je_generator(Arc::clone(&small)),
            |mut gen| {
                for _ in 0..batch_size {
                    black_box(gen.generate());
                }
            },
        );
    });

    // Medium CoA (~400 accounts)
    let medium = medium_coa();
    group.bench_function("medium_coa", |b| {
        b.iter_with_setup(
            || create_je_generator(Arc::clone(&medium)),
            |mut gen| {
                for _ in 0..batch_size {
                    black_box(gen.generate());
                }
            },
        );
    });

    // Large CoA (~2500 accounts)
    let large = large_coa();
    group.bench_function("large_coa", |b| {
        b.iter_with_setup(
            || create_je_generator(Arc::clone(&large)),
            |mut gen| {
                for _ in 0..batch_size {
                    black_box(gen.generate());
                }
            },
        );
    });

    group.finish();
}

/// Benchmark journal entry generation with approval workflow.
fn bench_je_with_approval(c: &mut Criterion) {
    let mut group = c.benchmark_group("je_with_approval");
    let batch_size = 1_000;

    group.throughput(Throughput::Elements(batch_size as u64));

    let coa = small_coa();

    // Without approval
    group.bench_function("without_approval", |b| {
        b.iter_with_setup(
            || create_je_generator(Arc::clone(&coa)),
            |mut gen| {
                for _ in 0..batch_size {
                    black_box(gen.generate());
                }
            },
        );
    });

    // With approval workflow
    group.bench_function("with_approval", |b| {
        b.iter_with_setup(
            || create_je_generator_with_approval(Arc::clone(&coa)),
            |mut gen| {
                for _ in 0..batch_size {
                    black_box(gen.generate());
                }
            },
        );
    });

    group.finish();
}

/// Benchmark line item generation rate.
fn bench_line_item_rate(c: &mut Criterion) {
    let mut group = c.benchmark_group("line_item_rate");
    let batch_size = 1_000;

    let coa = small_coa();
    let mut gen = create_je_generator(Arc::clone(&coa));

    // Pre-generate entries to count line items
    let entries: Vec<_> = (0..batch_size).map(|_| gen.generate()).collect();
    let total_lines: u64 = entries.iter().map(|e| e.line_count() as u64).sum();

    group.throughput(Throughput::Elements(total_lines));

    group.bench_function("line_items", |b| {
        b.iter_with_setup(
            || create_je_generator(Arc::clone(&coa)),
            |mut gen| {
                let mut total = 0u64;
                for _ in 0..batch_size {
                    let entry = gen.generate();
                    total += entry.line_count() as u64;
                }
                black_box(total)
            },
        );
    });

    group.finish();
}

/// Benchmark Chart of Accounts generation.
fn bench_coa_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("coa_generation");

    group.bench_function("small", |b| {
        b.iter(|| {
            let mut gen = ChartOfAccountsGenerator::new(
                CoAComplexity::Small,
                IndustrySector::Manufacturing,
                black_box(42),
            );
            black_box(gen.generate())
        });
    });

    group.bench_function("medium", |b| {
        b.iter(|| {
            let mut gen = ChartOfAccountsGenerator::new(
                CoAComplexity::Medium,
                IndustrySector::Manufacturing,
                black_box(42),
            );
            black_box(gen.generate())
        });
    });

    group.bench_function("large", |b| {
        b.iter(|| {
            let mut gen = ChartOfAccountsGenerator::new(
                CoAComplexity::Large,
                IndustrySector::Manufacturing,
                black_box(42),
            );
            black_box(gen.generate())
        });
    });

    group.finish();
}

/// Benchmark vendor generation.
fn bench_vendor_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("vendor_generation");
    let effective_date = start_date();

    for count in [100, 500, 1000].iter() {
        group.throughput(Throughput::Elements(*count as u64));

        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &size| {
            b.iter(|| {
                let mut gen = VendorGenerator::new(BENCHMARK_SEED);
                let vendors: Vec<_> = (0..size)
                    .map(|_| gen.generate_vendor("1000", effective_date))
                    .collect();
                black_box(vendors)
            });
        });
    }

    group.finish();
}

/// Benchmark customer generation.
fn bench_customer_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("customer_generation");
    let effective_date = start_date();

    for count in [100, 500, 1000].iter() {
        group.throughput(Throughput::Elements(*count as u64));

        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &size| {
            b.iter(|| {
                let mut gen = CustomerGenerator::new(BENCHMARK_SEED);
                let customers: Vec<_> = (0..size)
                    .map(|_| gen.generate_customer("1000", effective_date))
                    .collect();
                black_box(customers)
            });
        });
    }

    group.finish();
}

/// Benchmark material generation.
fn bench_material_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("material_generation");
    let effective_date = start_date();

    for count in [100, 500, 1000].iter() {
        group.throughput(Throughput::Elements(*count as u64));

        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &size| {
            b.iter(|| {
                let mut gen = MaterialGenerator::new(BENCHMARK_SEED);
                let materials: Vec<_> = (0..size)
                    .map(|_| gen.generate_material("1000", effective_date))
                    .collect();
                black_box(materials)
            });
        });
    }

    group.finish();
}

/// Benchmark fixed asset generation.
fn bench_asset_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("asset_generation");
    let acquisition_date = start_date();

    for count in [50, 100, 500].iter() {
        group.throughput(Throughput::Elements(*count as u64));

        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &size| {
            b.iter(|| {
                let mut gen = AssetGenerator::new(BENCHMARK_SEED);
                let assets: Vec<_> = (0..size)
                    .map(|_| gen.generate_asset("1000", acquisition_date))
                    .collect();
                black_box(assets)
            });
        });
    }

    group.finish();
}

/// Benchmark multi-company journal entry generation.
fn bench_multi_company_je(c: &mut Criterion) {
    let mut group = c.benchmark_group("multi_company_je");
    let batch_size = 1_000;

    group.throughput(Throughput::Elements(batch_size as u64));

    let coa = small_coa();
    let companies = multi_company_codes();

    group.bench_function("3_companies", |b| {
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
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_je_generation,
    bench_je_by_coa_complexity,
    bench_je_with_approval,
    bench_line_item_rate,
    bench_coa_generation,
    bench_vendor_generation,
    bench_customer_generation,
    bench_material_generation,
    bench_asset_generation,
    bench_multi_company_je,
);

criterion_main!(benches);
