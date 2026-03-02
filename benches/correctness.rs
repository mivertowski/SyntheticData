//! Correctness benchmarks for statistical validation.
//!
//! Tests Benford's Law compliance, balance coherence, and debit/credit balance.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;
use std::sync::Arc;

use datasynth_core::distributions::{
    get_first_digit, AmountDistributionConfig, AmountSampler, BenfordSampler, BENFORD_PROBABILITIES,
};
use datasynth_core::models::{AccountType, JournalEntry};
use rust_decimal::Decimal;

mod common;
use common::*;

/// Chi-square test for Benford's Law compliance.
fn chi_square_benford(digit_counts: &[u32; 9], total: u32) -> f64 {
    let mut chi_sq = 0.0;
    for (i, &observed) in digit_counts.iter().enumerate() {
        let expected = BENFORD_PROBABILITIES[i] * total as f64;
        let diff = observed as f64 - expected;
        chi_sq += (diff * diff) / expected;
    }
    chi_sq
}

/// Benchmark Benford's Law compliance verification.
fn bench_benford_compliance(c: &mut Criterion) {
    let mut group = c.benchmark_group("benford_compliance");

    for sample_size in [1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*sample_size as u64));

        // Benford-compliant sampling + verification
        group.bench_with_input(
            BenchmarkId::new("verify", sample_size),
            sample_size,
            |b, &size| {
                b.iter(|| {
                    let mut sampler = AmountSampler::with_benford(
                        BENCHMARK_SEED,
                        AmountDistributionConfig::default(),
                    );
                    let mut digit_counts = [0u32; 9];

                    for _ in 0..size {
                        let amount = sampler.sample();
                        if let Some(digit) = get_first_digit(amount) {
                            if (1..=9).contains(&digit) {
                                digit_counts[(digit - 1) as usize] += 1;
                            }
                        }
                    }

                    black_box(chi_square_benford(&digit_counts, size as u32))
                });
            },
        );

        // Just sampling (no verification)
        group.bench_with_input(
            BenchmarkId::new("sample_only", sample_size),
            sample_size,
            |b, &size| {
                b.iter(|| {
                    let mut sampler =
                        BenfordSampler::new(BENCHMARK_SEED, AmountDistributionConfig::default());
                    for _ in 0..size {
                        black_box(sampler.sample());
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark digit extraction from amounts.
fn bench_digit_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("digit_extraction");

    // Pre-generate amounts
    let mut sampler = AmountSampler::new(BENCHMARK_SEED);
    let amounts: Vec<Decimal> = (0..10_000).map(|_| sampler.sample()).collect();

    group.throughput(Throughput::Elements(amounts.len() as u64));

    group.bench_function("get_first_digit", |b| {
        b.iter(|| {
            for amount in &amounts {
                black_box(get_first_digit(*amount));
            }
        });
    });

    group.finish();
}

/// Benchmark balance coherence verification (Assets = Liabilities + Equity).
fn bench_balance_coherence(c: &mut Criterion) {
    let mut group = c.benchmark_group("balance_coherence");

    let coa = small_coa();
    let mut gen = create_je_generator(Arc::clone(&coa));

    // Generate entries for balance tracking
    let entries: Vec<JournalEntry> = (0..10_000).map(|_| gen.generate()).collect();

    group.throughput(Throughput::Elements(entries.len() as u64));

    // Simulate balance tracking
    group.bench_function("track_balances", |b| {
        b.iter(|| {
            let mut asset_balance = Decimal::ZERO;
            let mut liability_balance = Decimal::ZERO;
            let mut equity_balance = Decimal::ZERO;

            for entry in &entries {
                for line in &entry.lines {
                    // Get account type from CoA
                    if let Some(account) = coa.get_account(&line.gl_account) {
                        let net = line.debit_amount - line.credit_amount;
                        match account.account_type {
                            AccountType::Asset => asset_balance += net,
                            AccountType::Liability => liability_balance -= net,
                            AccountType::Equity => equity_balance -= net,
                            AccountType::Revenue => equity_balance -= net,
                            AccountType::Expense => equity_balance += net,
                            AccountType::Statistical => {} // Statistical accounts don't affect balance
                        }
                    }
                }
            }

            // Verify: Assets = Liabilities + Equity
            black_box(asset_balance - liability_balance - equity_balance)
        });
    });

    group.finish();
}

/// Benchmark debit/credit balance validation.
fn bench_debit_credit_balance(c: &mut Criterion) {
    let mut group = c.benchmark_group("debit_credit_balance");

    let coa = small_coa();
    let mut gen = create_je_generator(Arc::clone(&coa));

    let entries: Vec<JournalEntry> = (0..10_000).map(|_| gen.generate()).collect();

    group.throughput(Throughput::Elements(entries.len() as u64));

    // Check all entries are balanced
    group.bench_function("verify_all", |b| {
        b.iter(|| {
            let mut balanced_count = 0;
            for entry in &entries {
                if entry.is_balanced() {
                    balanced_count += 1;
                }
            }
            black_box(balanced_count)
        });
    });

    // Calculate totals
    group.bench_function("calculate_totals", |b| {
        b.iter(|| {
            let mut total_debit = Decimal::ZERO;
            let mut total_credit = Decimal::ZERO;

            for entry in &entries {
                total_debit += entry.total_debit();
                total_credit += entry.total_credit();
            }

            black_box((total_debit, total_credit))
        });
    });

    group.finish();
}

/// Benchmark entry validation.
fn bench_entry_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("entry_validation");

    let coa = small_coa();
    let mut gen = create_je_generator(Arc::clone(&coa));

    let entries: Vec<JournalEntry> = (0..10_000).map(|_| gen.generate()).collect();

    group.throughput(Throughput::Elements(entries.len() as u64));

    // Full validation
    group.bench_function("full_validation", |b| {
        b.iter(|| {
            let mut valid_count = 0;
            for entry in &entries {
                // Check balanced
                if !entry.is_balanced() {
                    continue;
                }
                // Check minimum lines
                if entry.line_count() < 2 {
                    continue;
                }
                // Check amounts are positive
                let amounts_ok = entry
                    .lines
                    .iter()
                    .all(|l| l.debit_amount >= Decimal::ZERO && l.credit_amount >= Decimal::ZERO);
                if !amounts_ok {
                    continue;
                }
                valid_count += 1;
            }
            black_box(valid_count)
        });
    });

    group.finish();
}

/// Benchmark line item distribution validation.
fn bench_line_distribution(c: &mut Criterion) {
    let mut group = c.benchmark_group("line_distribution");

    let coa = small_coa();
    let mut gen = create_je_generator(Arc::clone(&coa));

    let entries: Vec<JournalEntry> = (0..100_000).map(|_| gen.generate()).collect();

    group.throughput(Throughput::Elements(entries.len() as u64));

    // Count line item distribution
    group.bench_function("count_distribution", |b| {
        b.iter(|| {
            let mut line_counts = std::collections::HashMap::new();
            for entry in &entries {
                *line_counts.entry(entry.line_count()).or_insert(0u32) += 1;
            }
            black_box(line_counts)
        });
    });

    // Verify two-line dominance (should be ~60%)
    group.bench_function("verify_two_line", |b| {
        b.iter(|| {
            let two_line_count = entries.iter().filter(|e| e.line_count() == 2).count();
            let pct = two_line_count as f64 / entries.len() as f64;
            black_box(pct)
        });
    });

    group.finish();
}

/// Benchmark account usage distribution.
fn bench_account_distribution(c: &mut Criterion) {
    let mut group = c.benchmark_group("account_distribution");

    let coa = small_coa();
    let mut gen = create_je_generator(Arc::clone(&coa));

    let entries: Vec<JournalEntry> = (0..10_000).map(|_| gen.generate()).collect();

    group.throughput(Throughput::Elements(entries.len() as u64));

    // Count account usage
    group.bench_function("count_usage", |b| {
        b.iter(|| {
            let mut account_counts: std::collections::HashMap<String, u32> =
                std::collections::HashMap::new();
            for entry in &entries {
                for line in &entry.lines {
                    *account_counts.entry(line.gl_account.clone()).or_insert(0) += 1;
                }
            }
            black_box(account_counts.len())
        });
    });

    // Count by account type
    group.bench_function("count_by_type", |b| {
        b.iter(|| {
            let mut type_counts: std::collections::HashMap<AccountType, u32> =
                std::collections::HashMap::new();
            for entry in &entries {
                for line in &entry.lines {
                    if let Some(account) = coa.get_account(&line.gl_account) {
                        *type_counts.entry(account.account_type).or_insert(0) += 1;
                    }
                }
            }
            black_box(type_counts)
        });
    });

    group.finish();
}

/// Benchmark amount range validation.
fn bench_amount_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("amount_validation");

    let coa = small_coa();
    let mut gen = create_je_generator(Arc::clone(&coa));

    let entries: Vec<JournalEntry> = (0..10_000).map(|_| gen.generate()).collect();

    let all_amounts: Vec<Decimal> = entries
        .iter()
        .flat_map(|e| e.lines.iter())
        .filter_map(|l| {
            if l.debit_amount > Decimal::ZERO {
                Some(l.debit_amount)
            } else if l.credit_amount > Decimal::ZERO {
                Some(l.credit_amount)
            } else {
                None
            }
        })
        .collect();

    group.throughput(Throughput::Elements(all_amounts.len() as u64));

    // Calculate min/max
    group.bench_function("min_max", |b| {
        b.iter(|| {
            let min = all_amounts.iter().min().copied();
            let max = all_amounts.iter().max().copied();
            black_box((min, max))
        });
    });

    // Calculate mean
    group.bench_function("mean", |b| {
        b.iter(|| {
            let sum: Decimal = all_amounts.iter().copied().sum();
            let mean = sum / Decimal::from(all_amounts.len());
            black_box(mean)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_benford_compliance,
    bench_digit_extraction,
    bench_balance_coherence,
    bench_debit_credit_balance,
    bench_entry_validation,
    bench_line_distribution,
    bench_account_distribution,
    bench_amount_validation,
);

criterion_main!(benches);
