//! Benchmarks for output sinks (CSV, JSON, Compressed).
//!
//! Tests the throughput of writing journal entries to different formats.
//! Includes Phase 3 benchmarks for fast_csv utilities and compressed output.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;
use std::io::Write;
use tempfile::NamedTempFile;

use datasynth_core::traits::Sink;
use datasynth_output::fast_csv;
use datasynth_output::{CompressedWriter, CompressionConfig, CsvSink, JsonLinesSink};

mod common;
use common::generate_entries;

/// Benchmark CSV sink writing at different batch sizes.
fn bench_csv_sink(c: &mut Criterion) {
    let mut group = c.benchmark_group("csv_sink");

    for batch_size in [100, 1_000, 10_000].iter() {
        let entries = generate_entries(*batch_size);

        group.throughput(Throughput::Elements(*batch_size as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &entries,
            |b, entries| {
                b.iter_with_setup(
                    || {
                        let temp_file = NamedTempFile::new().unwrap();
                        CsvSink::new(temp_file.path().to_path_buf()).unwrap()
                    },
                    |mut sink| {
                        for entry in entries.iter().cloned() {
                            sink.write(entry).unwrap();
                        }
                        sink.flush().unwrap();
                    },
                );
            },
        );
    }

    group.finish();
}

/// Benchmark JSON Lines sink writing at different batch sizes.
fn bench_json_sink(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_lines_sink");

    for batch_size in [100, 1_000, 10_000].iter() {
        let entries = generate_entries(*batch_size);

        group.throughput(Throughput::Elements(*batch_size as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &entries,
            |b, entries| {
                b.iter_with_setup(
                    || {
                        let temp_file = NamedTempFile::new().unwrap();
                        JsonLinesSink::new(temp_file.path().to_path_buf()).unwrap()
                    },
                    |mut sink| {
                        for entry in entries.iter().cloned() {
                            sink.write(entry).unwrap();
                        }
                        sink.flush().unwrap();
                    },
                );
            },
        );
    }

    group.finish();
}

/// Benchmark JSON serialization only (no I/O).
fn bench_json_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_serialization");

    for batch_size in [100, 1_000, 10_000].iter() {
        let entries = generate_entries(*batch_size);

        group.throughput(Throughput::Elements(*batch_size as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &entries,
            |b, entries| {
                b.iter(|| {
                    for entry in entries {
                        black_box(serde_json::to_string(entry).unwrap());
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark JSON to_writer vs to_string serialization.
///
/// Compares the old approach (to_string -> write_all) with the new approach
/// (to_writer directly to buffer).
fn bench_json_to_writer_vs_to_string(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_to_writer_vs_to_string");
    let entries = generate_entries(5_000);

    group.throughput(Throughput::Elements(5_000));

    // Old approach: to_string + write_all
    group.bench_function("to_string", |b| {
        b.iter(|| {
            let mut buffer = Vec::with_capacity(4 * 1024 * 1024);
            for entry in &entries {
                let json = serde_json::to_string(entry).unwrap();
                buffer.write_all(json.as_bytes()).unwrap();
                buffer.write_all(b"\n").unwrap();
            }
            black_box(buffer)
        });
    });

    // New approach: to_writer directly
    group.bench_function("to_writer", |b| {
        b.iter(|| {
            let mut buffer = Vec::with_capacity(4 * 1024 * 1024);
            for entry in &entries {
                serde_json::to_writer(&mut buffer, entry).unwrap();
                buffer.write_all(b"\n").unwrap();
            }
            black_box(buffer)
        });
    });

    group.finish();
}

/// Benchmark CSV formatting only (no I/O).
fn bench_csv_formatting(c: &mut Criterion) {
    let mut group = c.benchmark_group("csv_formatting");

    for batch_size in [100, 1_000, 10_000].iter() {
        let entries = generate_entries(*batch_size);

        group.throughput(Throughput::Elements(*batch_size as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &entries,
            |b, entries| {
                b.iter(|| {
                    let mut buffer = Vec::with_capacity(1024 * 1024);
                    for entry in entries {
                        for line in &entry.lines {
                            writeln!(
                                buffer,
                                "{},{},{},{},{},{},{},{:?},{},{},{},{}",
                                entry.header.document_id,
                                entry.header.company_code,
                                entry.header.fiscal_year,
                                entry.header.fiscal_period,
                                entry.header.posting_date,
                                entry.header.document_type,
                                entry.header.currency,
                                entry.header.source,
                                line.line_number,
                                line.gl_account,
                                line.debit_amount,
                                line.credit_amount,
                            )
                            .unwrap();
                        }
                    }
                    black_box(buffer)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark fast_csv write-through formatting vs format!() allocation.
///
/// This is the core Phase 3 micro-benchmark showing the per-field speedup
/// from itoa/ryu/write-through formatting.
fn bench_fast_csv_vs_format(c: &mut Criterion) {
    let mut group = c.benchmark_group("fast_csv_vs_format");
    let entries = generate_entries(5_000);

    let total_lines: u64 = entries.iter().map(|e| e.line_count() as u64).sum();
    group.throughput(Throughput::Elements(total_lines));

    // Old: format!() per row (allocates String per row)
    group.bench_function("format_alloc", |b| {
        b.iter(|| {
            let mut buffer = Vec::with_capacity(2 * 1024 * 1024);
            for entry in &entries {
                for line in &entry.lines {
                    let row = format!(
                        "{},{},{},{},{},{},{},{:?},{},{},{},{}\n",
                        entry.header.document_id,
                        entry.header.company_code,
                        entry.header.fiscal_year,
                        entry.header.fiscal_period,
                        entry.header.posting_date,
                        entry.header.document_type,
                        entry.header.currency,
                        entry.header.source,
                        line.line_number,
                        line.gl_account,
                        line.debit_amount,
                        line.credit_amount,
                    );
                    buffer.write_all(row.as_bytes()).unwrap();
                }
            }
            black_box(buffer)
        });
    });

    // New: write!() directly to buffer (no intermediate String)
    group.bench_function("write_through", |b| {
        b.iter(|| {
            let mut buffer = Vec::with_capacity(2 * 1024 * 1024);
            for entry in &entries {
                for line in &entry.lines {
                    writeln!(
                        buffer,
                        "{},{},{},{},{},{},{},{:?},{},{},{},{}",
                        entry.header.document_id,
                        entry.header.company_code,
                        entry.header.fiscal_year,
                        entry.header.fiscal_period,
                        entry.header.posting_date,
                        entry.header.document_type,
                        entry.header.currency,
                        entry.header.source,
                        line.line_number,
                        line.gl_account,
                        line.debit_amount,
                        line.credit_amount,
                    )
                    .unwrap();
                }
            }
            black_box(buffer)
        });
    });

    // Newest: fast_csv utilities with itoa for integers
    group.bench_function("fast_csv_itoa", |b| {
        b.iter(|| {
            let mut buffer = Vec::with_capacity(2 * 1024 * 1024);
            for entry in &entries {
                for line in &entry.lines {
                    // Use fast_csv utilities for type-specific formatting
                    fast_csv::write_csv_field(&mut buffer, &entry.header.document_id.to_string())
                        .unwrap();
                    fast_csv::write_sep(&mut buffer).unwrap();
                    fast_csv::write_csv_field(&mut buffer, &entry.header.company_code).unwrap();
                    fast_csv::write_sep(&mut buffer).unwrap();
                    fast_csv::write_csv_int(&mut buffer, entry.header.fiscal_year).unwrap();
                    fast_csv::write_sep(&mut buffer).unwrap();
                    fast_csv::write_csv_int(&mut buffer, entry.header.fiscal_period as i32)
                        .unwrap();
                    fast_csv::write_sep(&mut buffer).unwrap();
                    write!(buffer, "{}", entry.header.posting_date).unwrap();
                    fast_csv::write_sep(&mut buffer).unwrap();
                    fast_csv::write_csv_field(&mut buffer, &entry.header.document_type).unwrap();
                    fast_csv::write_sep(&mut buffer).unwrap();
                    fast_csv::write_csv_field(&mut buffer, &entry.header.currency).unwrap();
                    fast_csv::write_sep(&mut buffer).unwrap();
                    write!(buffer, "{:?}", entry.header.source).unwrap();
                    fast_csv::write_sep(&mut buffer).unwrap();
                    fast_csv::write_csv_int(&mut buffer, line.line_number as i32).unwrap();
                    fast_csv::write_sep(&mut buffer).unwrap();
                    fast_csv::write_csv_field(&mut buffer, &line.gl_account).unwrap();
                    fast_csv::write_sep(&mut buffer).unwrap();
                    fast_csv::write_csv_decimal(&mut buffer, &line.debit_amount).unwrap();
                    fast_csv::write_sep(&mut buffer).unwrap();
                    fast_csv::write_csv_decimal(&mut buffer, &line.credit_amount).unwrap();
                    fast_csv::write_newline(&mut buffer).unwrap();
                }
            }
            black_box(buffer)
        });
    });

    group.finish();
}

/// Benchmark compressed CSV output vs uncompressed.
fn bench_compressed_output(c: &mut Criterion) {
    let mut group = c.benchmark_group("compressed_output");
    let entries = generate_entries(5_000);

    group.throughput(Throughput::Elements(5_000));

    // Uncompressed CSV
    group.bench_function("csv_uncompressed", |b| {
        b.iter_with_setup(
            || {
                let temp_file = NamedTempFile::new().unwrap();
                CsvSink::new(temp_file.path().to_path_buf()).unwrap()
            },
            |mut sink| {
                for entry in entries.iter().cloned() {
                    sink.write(entry).unwrap();
                }
                sink.close().unwrap();
            },
        );
    });

    // Compressed CSV via CompressedWriter
    group.bench_function("csv_zstd_level3", |b| {
        b.iter_with_setup(
            || {
                let temp_file = NamedTempFile::new().unwrap();
                let config = CompressionConfig::default();
                let writer =
                    CompressedWriter::new(temp_file.path(), &config).unwrap();
                writer
            },
            |mut writer| {
                // Write CSV header
                writer
                    .write_all(
                        b"document_id,company_code,fiscal_year,fiscal_period,posting_date,\
                    document_type,currency,source,line_number,gl_account,debit_amount,credit_amount\n",
                    )
                    .unwrap();
                // Write entries
                for entry in &entries {
                    for line in &entry.lines {
                        writeln!(
                            writer,
                            "{},{},{},{},{},{},{},{:?},{},{},{},{}",
                            entry.header.document_id,
                            entry.header.company_code,
                            entry.header.fiscal_year,
                            entry.header.fiscal_period,
                            entry.header.posting_date,
                            entry.header.document_type,
                            entry.header.currency,
                            entry.header.source,
                            line.line_number,
                            line.gl_account,
                            line.debit_amount,
                            line.credit_amount,
                        )
                        .unwrap();
                    }
                }
                writer.finish().unwrap();
            },
        );
    });

    group.finish();
}

/// Benchmark line item throughput (lines per second).
fn bench_line_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("line_throughput");

    let entries = generate_entries(10_000);
    let total_lines: u64 = entries.iter().map(|e| e.line_count() as u64).sum();

    group.throughput(Throughput::Elements(total_lines));

    // CSV line throughput
    group.bench_function("csv", |b| {
        b.iter_with_setup(
            || {
                let temp_file = NamedTempFile::new().unwrap();
                CsvSink::new(temp_file.path().to_path_buf()).unwrap()
            },
            |mut sink| {
                for entry in entries.iter().cloned() {
                    sink.write(entry).unwrap();
                }
                sink.flush().unwrap();
            },
        );
    });

    // JSON line throughput
    group.bench_function("json", |b| {
        b.iter_with_setup(
            || {
                let temp_file = NamedTempFile::new().unwrap();
                JsonLinesSink::new(temp_file.path().to_path_buf()).unwrap()
            },
            |mut sink| {
                for entry in entries.iter().cloned() {
                    sink.write(entry).unwrap();
                }
                sink.flush().unwrap();
            },
        );
    });

    group.finish();
}

/// Compare CSV vs JSON sink performance.
fn bench_sink_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("sink_comparison");
    let entries = generate_entries(5_000);

    group.throughput(Throughput::Elements(5_000));

    group.bench_function("csv_sink", |b| {
        b.iter_with_setup(
            || {
                let temp_file = NamedTempFile::new().unwrap();
                CsvSink::new(temp_file.path().to_path_buf()).unwrap()
            },
            |mut sink| {
                for entry in entries.iter().cloned() {
                    sink.write(entry).unwrap();
                }
                sink.close().unwrap();
            },
        );
    });

    group.bench_function("json_sink", |b| {
        b.iter_with_setup(
            || {
                let temp_file = NamedTempFile::new().unwrap();
                JsonLinesSink::new(temp_file.path().to_path_buf()).unwrap()
            },
            |mut sink| {
                for entry in entries.iter().cloned() {
                    sink.write(entry).unwrap();
                }
                sink.close().unwrap();
            },
        );
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_csv_sink,
    bench_json_sink,
    bench_json_serialization,
    bench_json_to_writer_vs_to_string,
    bench_csv_formatting,
    bench_fast_csv_vs_format,
    bench_compressed_output,
    bench_line_throughput,
    bench_sink_comparison,
);

criterion_main!(benches);
