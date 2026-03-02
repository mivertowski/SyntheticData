//! Benchmarks for statistical distribution sampling.
//!
//! Tests the performance of amount, temporal, and line item samplers.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;

use chrono::NaiveDate;
use datasynth_core::distributions::{
    AmountDistributionConfig, AmountSampler, LineItemSampler, SeasonalityConfig, TemporalSampler,
    WorkingHoursConfig,
};
use rust_decimal::Decimal;

mod common;
use common::BENCHMARK_SEED;

/// Benchmark amount sampling with different configurations.
fn bench_amount_sampling(c: &mut Criterion) {
    let mut group = c.benchmark_group("amount_sampling");

    for sample_size in [1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*sample_size as u64));

        // Default log-normal sampling
        group.bench_with_input(
            BenchmarkId::new("lognormal", sample_size),
            sample_size,
            |b, &size| {
                b.iter_with_setup(
                    || AmountSampler::new(BENCHMARK_SEED),
                    |mut sampler| {
                        for _ in 0..size {
                            black_box(sampler.sample());
                        }
                    },
                );
            },
        );

        // Benford-compliant sampling
        group.bench_with_input(
            BenchmarkId::new("benford", sample_size),
            sample_size,
            |b, &size| {
                b.iter_with_setup(
                    || {
                        AmountSampler::with_benford(
                            BENCHMARK_SEED,
                            AmountDistributionConfig::default(),
                        )
                    },
                    |mut sampler| {
                        for _ in 0..size {
                            black_box(sampler.sample());
                        }
                    },
                );
            },
        );
    }

    group.finish();
}

/// Benchmark amount summing to total (for balanced entries).
fn bench_amount_summing(c: &mut Criterion) {
    let mut group = c.benchmark_group("amount_summing");

    let total = Decimal::from(10000);

    for line_count in [2, 4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(line_count),
            line_count,
            |b, &count| {
                b.iter_with_setup(
                    || AmountSampler::new(BENCHMARK_SEED),
                    |mut sampler| {
                        for _ in 0..1000 {
                            black_box(sampler.sample_summing_to(count, total));
                        }
                    },
                );
            },
        );
    }

    group.finish();
}

/// Benchmark amount sampling with different configurations.
fn bench_amount_configs(c: &mut Criterion) {
    let mut group = c.benchmark_group("amount_configs");
    let sample_size = 10_000;

    group.throughput(Throughput::Elements(sample_size as u64));

    // Small transactions (retail)
    group.bench_function("small_transactions", |b| {
        b.iter_with_setup(
            || {
                AmountSampler::with_config(
                    BENCHMARK_SEED,
                    AmountDistributionConfig::small_transactions(),
                )
            },
            |mut sampler| {
                for _ in 0..sample_size {
                    black_box(sampler.sample());
                }
            },
        );
    });

    // Medium transactions (B2B)
    group.bench_function("medium_transactions", |b| {
        b.iter_with_setup(
            || {
                AmountSampler::with_config(
                    BENCHMARK_SEED,
                    AmountDistributionConfig::medium_transactions(),
                )
            },
            |mut sampler| {
                for _ in 0..sample_size {
                    black_box(sampler.sample());
                }
            },
        );
    });

    // Large transactions (enterprise)
    group.bench_function("large_transactions", |b| {
        b.iter_with_setup(
            || {
                AmountSampler::with_config(
                    BENCHMARK_SEED,
                    AmountDistributionConfig::large_transactions(),
                )
            },
            |mut sampler| {
                for _ in 0..sample_size {
                    black_box(sampler.sample());
                }
            },
        );
    });

    group.finish();
}

/// Benchmark temporal sampling.
fn bench_temporal_sampling(c: &mut Criterion) {
    let mut group = c.benchmark_group("temporal_sampling");

    let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

    for sample_size in [1_000, 10_000].iter() {
        group.throughput(Throughput::Elements(*sample_size as u64));

        // Date sampling
        group.bench_with_input(
            BenchmarkId::new("date", sample_size),
            sample_size,
            |b, &size| {
                b.iter_with_setup(
                    || TemporalSampler::new(BENCHMARK_SEED),
                    |mut sampler| {
                        for _ in 0..size {
                            black_box(sampler.sample_date(start, end));
                        }
                    },
                );
            },
        );

        // Time sampling (human)
        group.bench_with_input(
            BenchmarkId::new("time_human", sample_size),
            sample_size,
            |b, &size| {
                b.iter_with_setup(
                    || TemporalSampler::new(BENCHMARK_SEED),
                    |mut sampler| {
                        for _ in 0..size {
                            black_box(sampler.sample_time(true));
                        }
                    },
                );
            },
        );

        // Time sampling (automated)
        group.bench_with_input(
            BenchmarkId::new("time_automated", sample_size),
            sample_size,
            |b, &size| {
                b.iter_with_setup(
                    || TemporalSampler::new(BENCHMARK_SEED),
                    |mut sampler| {
                        for _ in 0..size {
                            black_box(sampler.sample_time(false));
                        }
                    },
                );
            },
        );
    }

    group.finish();
}

/// Benchmark date multiplier calculations (used for weighted date selection).
fn bench_date_multiplier(c: &mut Criterion) {
    let mut group = c.benchmark_group("date_multiplier");

    let sampler = TemporalSampler::new(BENCHMARK_SEED);

    // Regular weekday
    let regular_day = NaiveDate::from_ymd_opt(2024, 6, 12).unwrap();
    group.bench_function("regular_weekday", |b| {
        b.iter(|| black_box(sampler.get_date_multiplier(regular_day)));
    });

    // Weekend
    let weekend = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
    group.bench_function("weekend", |b| {
        b.iter(|| black_box(sampler.get_date_multiplier(weekend)));
    });

    // Month-end
    let month_end = NaiveDate::from_ymd_opt(2024, 6, 28).unwrap();
    group.bench_function("month_end", |b| {
        b.iter(|| black_box(sampler.get_date_multiplier(month_end)));
    });

    // Year-end
    let year_end = NaiveDate::from_ymd_opt(2024, 12, 30).unwrap();
    group.bench_function("year_end", |b| {
        b.iter(|| black_box(sampler.get_date_multiplier(year_end)));
    });

    group.finish();
}

/// Benchmark temporal sampler with different seasonality configs.
fn bench_temporal_configs(c: &mut Criterion) {
    let mut group = c.benchmark_group("temporal_configs");

    let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
    let sample_size = 1_000;

    group.throughput(Throughput::Elements(sample_size as u64));

    // Default config
    group.bench_function("default_seasonality", |b| {
        b.iter_with_setup(
            || TemporalSampler::new(BENCHMARK_SEED),
            |mut sampler| {
                for _ in 0..sample_size {
                    black_box(sampler.sample_date(start, end));
                }
            },
        );
    });

    // No seasonality
    group.bench_function("no_seasonality", |b| {
        let config = SeasonalityConfig {
            month_end_spike: false,
            quarter_end_spike: false,
            year_end_spike: false,
            day_of_week_patterns: false,
            ..Default::default()
        };
        b.iter_with_setup(
            || {
                TemporalSampler::with_config(
                    BENCHMARK_SEED,
                    config.clone(),
                    WorkingHoursConfig::default(),
                    Vec::new(),
                )
            },
            |mut sampler| {
                for _ in 0..sample_size {
                    black_box(sampler.sample_date(start, end));
                }
            },
        );
    });

    group.finish();
}

/// Benchmark line item count sampling.
fn bench_line_item_sampling(c: &mut Criterion) {
    let mut group = c.benchmark_group("line_item_sampling");

    for sample_size in [1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*sample_size as u64));

        // Raw count sampling
        group.bench_with_input(
            BenchmarkId::new("count_only", sample_size),
            sample_size,
            |b, &size| {
                b.iter_with_setup(
                    || LineItemSampler::new(BENCHMARK_SEED),
                    |mut sampler| {
                        for _ in 0..size {
                            black_box(sampler.sample_count());
                        }
                    },
                );
            },
        );

        // Full spec sampling (includes parity and split)
        group.bench_with_input(
            BenchmarkId::new("full_spec", sample_size),
            sample_size,
            |b, &size| {
                b.iter_with_setup(
                    || LineItemSampler::new(BENCHMARK_SEED),
                    |mut sampler| {
                        for _ in 0..size {
                            black_box(sampler.sample());
                        }
                    },
                );
            },
        );
    }

    group.finish();
}

/// Benchmark line item parity sampling.
fn bench_line_item_parity(c: &mut Criterion) {
    let mut group = c.benchmark_group("line_item_parity");
    let sample_size = 10_000;

    group.throughput(Throughput::Elements(sample_size as u64));

    // Without parity constraint
    group.bench_function("without_parity", |b| {
        b.iter_with_setup(
            || LineItemSampler::new(BENCHMARK_SEED),
            |mut sampler| {
                for _ in 0..sample_size {
                    black_box(sampler.sample_count());
                }
            },
        );
    });

    // With parity constraint
    group.bench_function("with_parity", |b| {
        b.iter_with_setup(
            || LineItemSampler::new(BENCHMARK_SEED),
            |mut sampler| {
                for _ in 0..sample_size {
                    black_box(sampler.sample_count_with_parity());
                }
            },
        );
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_amount_sampling,
    bench_amount_summing,
    bench_amount_configs,
    bench_temporal_sampling,
    bench_date_multiplier,
    bench_temporal_configs,
    bench_line_item_sampling,
    bench_line_item_parity,
);

criterion_main!(benches);
