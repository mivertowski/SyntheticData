# Dataset Generation Performance Improvements

> A deep analysis of throughput bottlenecks and concrete strategies for scaling
> synthetic data generation from 100K entries/sec to 1M+ entries/sec.

## Table of Contents

1. [Current Performance Baseline](#1-current-performance-baseline)
2. [CPU-Level Optimizations](#2-cpu-level-optimizations)
3. [Parallelism & Concurrency](#3-parallelism--concurrency)
4. [Memory & Allocation Optimizations](#4-memory--allocation-optimizations)
5. [I/O Pipeline Improvements](#5-io-pipeline-improvements)
6. [GPU-Accelerated Generation](#6-gpu-accelerated-generation)
7. [Algorithmic Improvements](#7-algorithmic-improvements)
8. [Architecture-Level Changes](#8-architecture-level-changes)
9. [Benchmarking & Measurement](#9-benchmarking--measurement)
10. [Implementation Roadmap](#10-implementation-roadmap)

---

## 1. Current Performance Baseline

### Measured Throughput

| Component | Throughput | Bottleneck |
|-----------|-----------|------------|
| Journal Entry generation | ~100K entries/sec | Single-threaded RNG + decimal math |
| JE with approval workflows | ~60-70K entries/sec | Additional branch logic |
| Master data (vendors) | ~500/sec | String/UUID generation |
| Master data (customers) | ~400/sec | String/UUID generation |
| Master data (materials) | ~300/sec | String/UUID generation |
| CSV output | ~500K line items/sec | BufWriter 8KB + disk I/O |
| JSON output | ~400K line items/sec | serde serialization overhead |
| Multi-company JE | ~80K entries/sec | Sequential per-company |

### Current Architecture Constraints

The system currently operates under several design constraints that limit throughput:

- **Single-threaded generators**: Each generator type runs on one OS thread. The `ParallelGenerator` trait exists in `datasynth-core/src/traits/generator.rs` but has no implementations.
- **Sequential orchestration**: The `EnhancedOrchestrator` runs 15+ generation phases sequentially (CoA → Master Data → Document Flows → JEs → Subledger → ...).
- **Rayon imported but unused**: The `rayon` crate is a workspace dependency but `par_iter()` is never called.
- **8KB I/O buffers**: Output sinks use `BufWriter<File>` with the default 8KB buffer.
- **Per-item resource checks**: Memory/disk guards check every 500 operations, adding conditional branch overhead in hot loops.
- **`rust_decimal` for all amounts**: Precise but ~10x slower than `f64` arithmetic. Not SIMD-vectorizable.
- **Deterministic RNG requirement**: ChaCha8 is sequential by nature — each sample depends on the prior state.

### Where Time Is Spent (Estimated Profile)

```
┌────────────────────────────────────────────────────────────┐
│ JournalEntry::generate()  100%                             │
│ ├── Amount sampling (log-normal + Benford)      ~25%       │
│ ├── Temporal sampling (business day + lags)      ~15%       │
│ ├── Line item construction + Decimal ops        ~20%       │
│ ├── UUID generation (AtomicU64 + hashing)       ~10%       │
│ ├── Account selection (HashMap lookup)           ~8%       │
│ ├── Company/user/vendor pool selection           ~7%       │
│ ├── Approval workflow logic                      ~5%       │
│ ├── Serialization (serde → CSV/JSON)            ~5%       │
│ └── Resource guard checks + overhead             ~5%       │
└────────────────────────────────────────────────────────────┘
```

---

## 2. CPU-Level Optimizations

### 2.1 SIMD-Accelerated Distribution Sampling

The amount sampling pipeline (log-normal → Benford compliance → rounding) is the single
hottest code path at ~25% of generation time. Modern CPUs can process 4-8 `f64` values
simultaneously via AVX2/AVX-512.

**Strategy**: Batch-sample raw `f64` values using SIMD, then convert to `rust_decimal`
only at the serialization boundary.

```rust
// Before: one-at-a-time sampling
fn sample_amount(&mut self) -> Decimal {
    let raw = self.log_normal.sample(&mut self.rng);
    Decimal::from_f64_retain(raw).unwrap()
}

// After: SIMD batch sampling with deferred Decimal conversion
fn sample_amounts_batch(&mut self, count: usize) -> Vec<Decimal> {
    // Sample raw f64 values in SIMD-width chunks (4x AVX2, 8x AVX-512)
    let raw_values: Vec<f64> = (0..count)
        .map(|_| self.log_normal.sample(&mut self.rng))
        .collect();

    // Benford compliance check in batch (vectorizable comparison)
    // Convert to Decimal only for the final output
    raw_values.iter()
        .map(|&v| Decimal::from_f64_retain(v).unwrap())
        .collect()
}
```

**Expected gain**: 2-4x for amount sampling (~25% of total → saves 12-18% overall).

### 2.2 Deferred Decimal Conversion

`rust_decimal` is necessary for output correctness (no IEEE 754 rounding in financial
data), but most intermediate calculations don't need exact decimal precision.

**Strategy**: Use `f64` for all internal computation (sampling, thresholds, comparisons).
Convert to `Decimal` only when constructing the final `JournalEntryLine`.

```
Internal pipeline:  f64 → f64 → f64 → ... → Decimal (only at output)
Current pipeline:   Decimal → Decimal → Decimal → ... → Decimal (everywhere)
```

This eliminates the ~20% overhead from intermediate Decimal arithmetic in line item
construction. The only constraint is ensuring the final conversion uses
`Decimal::from_f64_retain()` to preserve the sampled value exactly.

**Expected gain**: 15-20% overall throughput improvement.

### 2.3 Branch Prediction Optimization

The generation hot loop contains many conditional checks (fraud injection, approval
workflows, period-end dynamics, processing lags) that are rarely taken. Reorganize
to put the common path first and use `#[cold]` annotations:

```rust
// Hot path: normal generation (>95% of iterations)
let entry = self.generate_normal_entry();

// Cold paths: rare conditions
if unlikely(self.should_inject_fraud()) {
    self.apply_fraud_pattern(&mut entry);
}
if unlikely(self.is_period_end()) {
    self.apply_period_end_dynamics(&mut entry);
}
```

Use `#[inline(always)]` on small, frequently-called methods like `sample_amount()`,
`next_uuid()`, and `select_account()`.

**Expected gain**: 3-5% from reduced branch misprediction.

### 2.4 Compile-Time Feature Gating

Not all generation runs need every feature. Use Cargo features to compile out
entire code paths:

```toml
[features]
default = ["full"]
full = ["fraud", "temporal", "copulas", "audit", "banking"]
minimal = []  # Just JE generation, no extras
fraud = []
temporal = []
copulas = []
```

When generating simple datasets without fraud patterns, copulas, or audit trails, the
compiler can eliminate entire branches and reduce instruction cache pressure.

**Expected gain**: 5-10% for simple generation profiles.

---

## 3. Parallelism & Concurrency

### 3.1 Implement the ParallelGenerator Trait

The `ParallelGenerator` trait already exists but has no implementations:

```rust
pub trait ParallelGenerator: Generator {
    fn split(self, parts: usize) -> Vec<Self>;
}
```

**Strategy**: Split a generator into N independent sub-generators, each with a
deterministic seed derived from the parent:

```rust
impl ParallelGenerator for JournalEntryGenerator {
    fn split(self, parts: usize) -> Vec<Self> {
        (0..parts).map(|i| {
            let sub_seed = self.seed.wrapping_add(i as u64 * 0x9E3779B97F4A7C15);
            let count = self.total_entries / parts
                + if i < self.total_entries % parts { 1 } else { 0 };
            JournalEntryGenerator::new_with_seed(sub_seed, count, self.config.clone())
        }).collect()
    }
}
```

Then use Rayon to run them in parallel:

```rust
let generators = generator.split(num_cpus::get());
let entries: Vec<JournalEntry> = generators
    .into_par_iter()
    .flat_map(|mut gen| gen.generate_batch(gen.count))
    .collect();
```

**Determinism guarantee**: Each sub-generator gets a unique seed derived from the
parent seed + partition index. The results are deterministic for a given partition
count. Document that changing the thread count changes output (but each count is
independently reproducible).

**Expected gain**: Near-linear scaling up to ~8-12 cores (limited by memory bandwidth
for large datasets). On a 16-core machine: **8-12x throughput**.

### 3.2 Phase-Level Parallelism in the Orchestrator

The `EnhancedOrchestrator` runs 15+ phases sequentially, but many are independent:

```
                    ┌─── Vendor Generation ───┐
                    ├─── Customer Generation ──┤
CoA Generation ─────├─── Material Generation ──├──── Document Flows ──── JEs
                    ├─── Employee Generation ──┤
                    └─── Asset Generation ─────┘

Independent:                                    Dependent:
- All master data types                         - Doc flows need master data
- Banking + OCPM (no JE dependency)             - JEs need CoA + master data
- Audit (needs JEs)                             - Subledger needs JEs
```

**Strategy**: Build a dependency DAG and execute independent phases concurrently:

```rust
// Phase 1: CoA (must be first)
let coa = generate_coa(&config);

// Phase 2: All master data in parallel (all depend only on CoA)
let (vendors, customers, materials, employees, assets) = rayon::join5(
    || generate_vendors(&config, &coa),
    || generate_customers(&config, &coa),
    || generate_materials(&config, &coa),
    || generate_employees(&config, &coa),
    || generate_assets(&config, &coa),
);

// Phase 3: Document flows + JEs (depend on master data)
// Phase 4: Subledger + Period close (depend on JEs)
// Phase 5: Anomaly injection + Labels (depend on everything)
```

**Expected gain**: Master data generation drops from sequential (sum of all times)
to parallel (max of all times). For 5 master data types at ~400/sec each generating
1000 records: from ~12.5s to ~2.5s — a **5x speedup for master data phases**.

### 3.3 Pipeline Parallelism (Generate While Writing)

Currently generation and output are sequential: generate all → write all. Use a
bounded channel to overlap generation and I/O:

```
Thread 1 (Generator):  [G1][G2][G3][G4][G5][G6]...
                          ↓   ↓   ↓   ↓   ↓
Channel (bounded, 1024 items):
                          ↓   ↓   ↓   ↓   ↓
Thread 2 (Writer):        [W1][W2][W3][W4][W5]...
```

```rust
let (tx, rx) = crossbeam_channel::bounded(1024);

// Producer thread
let producer = std::thread::spawn(move || {
    for _ in 0..total {
        let entry = generator.generate_one();
        tx.send(entry).unwrap(); // blocks if channel full (backpressure)
    }
});

// Consumer thread (can also be parallel with multiple sinks)
let consumer = std::thread::spawn(move || {
    for entry in rx {
        sink.write(entry).unwrap();
    }
});
```

This hides I/O latency behind generation compute and vice versa. The crossbeam
channel infrastructure already exists in `datasynth-core/src/streaming/`.

**Expected gain**: 20-40% throughput improvement by overlapping CPU and I/O.

### 3.4 Multi-Company Parallelism

Currently, multi-company generation is sequential. Each company's data is independent
(different seed, different CoA subset), making this embarrassingly parallel:

```rust
let results: Vec<CompanyResult> = config.companies
    .par_iter()
    .map(|company| generate_company_data(company, &shared_coa))
    .collect();
```

**Expected gain**: Linear scaling with number of companies. 10 companies on 10 cores:
**~10x faster** than sequential.

### 3.5 Lock-Free UUID Generation

The current `UuidFactory` uses `AtomicU64::fetch_add()` which is already lock-free,
but creates cache-line contention when multiple threads increment the same counter.

**Strategy**: Per-thread UUID counters with partitioned ranges:

```rust
// Thread 0: counter range [0,         1_000_000)
// Thread 1: counter range [1_000_000, 2_000_000)
// Thread N: counter range [N*1M,      (N+1)*1M)
```

This eliminates all atomic contention. Each thread owns its counter exclusively.

**Expected gain**: Eliminates atomic CAS contention — measurable at >4 threads
(~5-10% at 16 threads).

---

## 4. Memory & Allocation Optimizations

### 4.1 Arena Allocation for Batch Generation

Each `JournalEntry` allocates multiple heap objects: `Vec<JournalEntryLine>`,
`String` fields (description, reference, created_by), and `Uuid` values. For batch
generation, an arena allocator amortizes allocation overhead:

```rust
use bumpalo::Bump;

fn generate_batch_arena(generator: &mut JeGenerator, count: usize) -> Vec<JournalEntry> {
    let arena = Bump::with_capacity(count * 2048); // ~2KB per entry
    let mut entries = Vec::with_capacity(count);

    for _ in 0..count {
        // All intermediate strings allocated in arena
        // Final entry moved to output vec
        entries.push(generator.generate_in_arena(&arena));
    }
    // Arena freed in bulk when dropped — no per-object dealloc
    entries
}
```

**Expected gain**: 10-15% from reduced allocator pressure. Allocation-heavy paths
(master data generation at ~300-500/sec) benefit most.

### 4.2 String Interning for Repeated Values

Many generated values are drawn from small pools and repeat frequently:

| Field | Pool Size | Repetition Rate |
|-------|-----------|-----------------|
| Account codes | 100-2,500 | Very high |
| Company codes | 1-50 | Very high |
| Currency codes | ~10 | Very high |
| User IDs | 10-100 | High |
| Cost center codes | 10-50 | High |
| Vendor/customer names | 100-10,000 | Moderate |

**Strategy**: Use a string interner to deduplicate these values in memory:

```rust
use lasso::{Spur, ThreadedRodeo};

struct InternedGenerator {
    interner: ThreadedRodeo,
    // ...
}

impl InternedGenerator {
    fn intern_account(&self, code: &str) -> Spur {
        self.interner.get_or_intern(code)
    }
}
```

For 1M journal entries with ~4 line items each, this reduces string allocations
from ~4M to ~2,500 (the number of unique accounts). Each subsequent reference is
a 32-bit key instead of a heap-allocated `String`.

**Expected gain**: 15-25% memory reduction, 5-10% speed improvement from reduced
allocation and improved cache locality.

### 4.3 SmallVec for Line Items

Most journal entries have 2-8 line items. Using `SmallVec<[JournalEntryLine; 4]>`
keeps small entries entirely on the stack:

```rust
use smallvec::SmallVec;

struct JournalEntry {
    // Before: lines: Vec<JournalEntryLine>  — always heap-allocated
    // After:  lines stored inline for ≤4 items, spills to heap for >4
    lines: SmallVec<[JournalEntryLine; 4]>,
}
```

For a typical distribution where ~60% of entries have ≤4 lines, this eliminates
60% of `Vec` heap allocations in the hottest struct.

**Expected gain**: 5-8% throughput improvement from reduced heap allocation.

### 4.4 Pre-Computed Lookup Tables

Several hot-path computations can be replaced with table lookups:

**Benford CDF**: Already pre-computed as arrays — good. But the rejection sampling
loop can be replaced with a pre-computed inverse CDF table (256 entries) for O(1)
sampling instead of O(9) linear scan.

**Holiday calendar**: Pre-compute a `HashSet<NaiveDate>` for the entire generation
period at startup instead of checking rules per-date.

**Account selection weights**: Pre-compute a cumulative distribution array for
`O(log n)` binary search instead of linear scan through account weights.

```rust
// Before: O(n) linear scan each time
fn select_account(&mut self) -> &Account {
    let r: f64 = self.rng.gen();
    let mut cumulative = 0.0;
    for account in &self.accounts {
        cumulative += account.weight;
        if r < cumulative { return account; }
    }
    self.accounts.last().unwrap()
}

// After: O(log n) with pre-computed CDF
fn select_account(&mut self) -> &Account {
    let r: f64 = self.rng.gen();
    let idx = self.account_cdf.partition_point(|&w| w < r);
    &self.accounts[idx]
}
```

**Expected gain**: 3-5% from reduced per-sample computation.

---

## 5. I/O Pipeline Improvements

### 5.1 Larger Write Buffers

The current `BufWriter` uses the default 8KB buffer. For high-throughput generation,
this causes excessive `write()` syscalls:

```rust
// Current: 8KB default
let writer = BufWriter::new(file);

// Improved: 256KB buffer (one syscall per ~1000 entries instead of ~30)
let writer = BufWriter::with_capacity(256 * 1024, file);
```

At ~300 bytes per CSV line, an 8KB buffer flushes every ~27 lines. A 256KB buffer
flushes every ~870 lines — **32x fewer syscalls**.

**Expected gain**: 10-20% I/O throughput improvement, especially on networked
filesystems.

### 5.2 Parallel Output Sinks

When generating multiple output files simultaneously (journal_entries.csv,
vendors.csv, customers.csv, etc.), each file can be written by a dedicated thread:

```
Generator Thread Pool
    ├── JE Generator → Channel → JE Writer Thread → journal_entries.csv
    ├── Vendor Gen   → Channel → Vendor Writer     → vendors.csv
    ├── Customer Gen → Channel → Customer Writer    → customers.csv
    └── ...
```

This is especially impactful when writing to multiple disks or SSDs.

**Expected gain**: Eliminates I/O serialization between file types. On NVMe SSDs:
2-4x write throughput improvement.

### 5.3 Vectorized CSV Serialization

Instead of formatting one field at a time with `serde`, batch-format entire rows:

```rust
// Before: field-by-field via serde
csv_writer.serialize(&entry)?;  // serde processes each field individually

// After: pre-format the row as a single string with a reusable buffer
fn format_je_row(entry: &JournalEntry, buf: &mut String) {
    buf.clear();
    buf.push_str(&entry.id.to_string());
    buf.push(',');
    buf.push_str(&entry.date.to_string());
    buf.push(',');
    // ... direct string formatting, no serde overhead
    buf.push('\n');
}

// Write the pre-formatted buffer
writer.write_all(buf.as_bytes())?;
```

This avoids serde's reflection machinery and trait object dispatch for known schemas.

For even higher throughput, use `itoa` and `ryu` crates for fast integer/float
formatting:

```rust
use itoa;
use ryu;

// 2-3x faster than Display trait formatting for numbers
itoa::fmt(&mut buf, entry.line_number)?;
ryu::Buffer::new().format(entry.amount.to_f64().unwrap());
```

**Expected gain**: 30-50% improvement in serialization throughput.

### 5.4 io_uring for Linux (Async I/O)

On Linux 5.1+, `io_uring` provides kernel-level async I/O that eliminates syscall
overhead entirely for batched writes:

```rust
use tokio_uring::fs::File;

async fn write_batch(entries: &[JournalEntry], path: &str) {
    let file = File::create(path).await.unwrap();
    let mut offset = 0u64;

    for chunk in entries.chunks(1024) {
        let buf = serialize_chunk(chunk);
        let (res, buf) = file.write_at(buf, offset).await;
        offset += res.unwrap() as u64;
    }
}
```

This submits multiple write operations in a single syscall and processes completions
in batch. Particularly effective for Parquet output where large column chunks are
written.

**Expected gain**: 15-30% I/O improvement on Linux with NVMe storage. Less impactful
on SATA SSDs or HDDs (which are seek-bound, not syscall-bound).

### 5.5 Compressed Output Pipeline

For large datasets, compression can actually *improve* throughput by reducing disk
I/O at the cost of CPU:

```rust
use zstd::stream::write::Encoder;

let file = File::create("journal_entries.csv.zst")?;
let encoder = Encoder::new(file, 3)?;  // Level 3: fast compression
let writer = BufWriter::with_capacity(256 * 1024, encoder);
```

Zstandard at level 3 typically achieves 3-4x compression on CSV data at ~500 MB/s
compression speed (single thread). This means:

- **Without compression**: 1 GB data → 1 GB written → limited by disk write speed
- **With compression**: 1 GB data → ~300 MB written → limited by CPU (which is faster)

For parallel compression, use `pigz` (parallel gzip) or zstd's multi-threaded mode:

```rust
let mut encoder = Encoder::new(file, 3)?;
encoder.multithread(4)?;  // Use 4 threads for compression
```

**Expected gain**: 2-3x effective I/O throughput when disk-bound.

---

## 6. GPU-Accelerated Generation

### 6.1 When GPU Acceleration Makes Sense

GPUs excel at **data-parallel** workloads where the same operation is applied to
millions of independent data points. In synthetic data generation, the best
candidates are:

| Task | GPU Suitability | Reason |
|------|----------------|--------|
| Distribution sampling (log-normal, Gaussian) | Excellent | Millions of independent samples |
| Copula sampling | Excellent | Matrix operations (Cholesky, CDF) |
| Benford compliance checking | Good | Parallel digit extraction |
| UUID generation | Good | Hash computation is parallel |
| Correlation matrix operations | Excellent | Dense linear algebra |
| CSV/JSON serialization | Poor | String operations, branching |
| Document chain linking | Poor | Sequential dependencies |
| Approval workflow logic | Poor | Complex branching |

### 6.2 CUDA-Based Distribution Sampling

Use cuRAND for massively parallel random number generation:

```rust
// Pseudocode for GPU-accelerated amount sampling
// Using rust-cuda or cudarc crate

fn gpu_sample_amounts(count: usize, seed: u64) -> Vec<f64> {
    // 1. Initialize cuRAND generator on GPU
    let generator = CurandGenerator::new(CurandRngType::XORWOW, seed);

    // 2. Allocate device memory
    let mut d_output = DeviceBuffer::zeros(count);

    // 3. Generate log-normal samples (millions in parallel)
    generator.generate_log_normal(&mut d_output, mu, sigma);

    // 4. Copy back to host
    d_output.copy_to_host()
}
```

For 10M samples, a mid-range GPU (RTX 3070) generates ~10 billion random numbers/sec
versus ~500M/sec on CPU. This is a **20x speedup** for the sampling step.

### 6.3 GPU-Accelerated Copula Computation

The Gaussian copula requires Cholesky decomposition and matrix multiplication — prime
GPU territory:

```
CPU Cholesky (100×100 matrix): ~1ms
GPU Cholesky (100×100 matrix): ~0.01ms (with kernel launch overhead)
GPU Cholesky (1000×1000 matrix): ~0.1ms
```

For large correlation matrices or when sampling millions of correlated vectors, the
GPU advantage is significant. Use cuBLAS via `cudarc`:

```rust
use cudarc::cublas::CudaBlas;

fn gpu_correlated_sample(
    correlation_matrix: &[f64],  // N×N
    n_samples: usize,
) -> Vec<Vec<f64>> {
    let blas = CudaBlas::new(device)?;

    // 1. Cholesky decomposition: L = cholesky(Σ)
    let l_matrix = gpu_cholesky(correlation_matrix);

    // 2. Generate N×n_samples independent standard normals
    let z = gpu_standard_normal(n_dims, n_samples);

    // 3. Matrix multiply: X = L × Z (correlated samples)
    blas.gemm(/* L, Z, X */);

    // 4. Apply marginal CDFs (transform to target distributions)
    gpu_apply_marginals(x)
}
```

### 6.4 Hybrid CPU-GPU Pipeline

The most effective architecture uses the GPU for bulk sampling and the CPU for
sequential logic:

```
┌─────────────────────────────────────────────────────────────────┐
│ GPU (Bulk Sampling)                                             │
│ ┌─────────┐  ┌──────────────┐  ┌────────────────┐              │
│ │ cuRAND   │→│ Log-Normal   │→│ Benford Filter │→ Buffer      │
│ │ 10B/sec  │  │ Sampling     │  │ (parallel)     │              │
│ └─────────┘  └──────────────┘  └────────────────┘              │
└──────────────────────────────┬──────────────────────────────────┘
                               │ DMA Transfer (PCIe)
                               ↓
┌──────────────────────────────┴──────────────────────────────────┐
│ CPU (Sequential Assembly)                                       │
│ ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│ │ Assign   │→│ Build    │→│ Apply    │→│ Serialize│→ Output  │
│ │ Accounts │  │ Entries  │  │ Workflow │  │ CSV/JSON │         │
│ └──────────┘  └──────────┘  └──────────┘  └──────────┘        │
└─────────────────────────────────────────────────────────────────┘
```

**Key insight**: Pre-generate all random values on the GPU in large batches (1M+),
transfer to host memory, then consume them on the CPU during entry construction.
The GPU stays busy generating the next batch while the CPU processes the current one.

### 6.5 GPU Feasibility Assessment

| Factor | Assessment |
|--------|-----------|
| **Benefit** | 10-20x for distribution sampling (~25% of total time) |
| **Net speedup** | ~3-5x overall (Amdahl's law: only 25% is GPU-suitable) |
| **Complexity** | High — CUDA dependency, device management, error handling |
| **Portability** | Limited — NVIDIA only (unless using Vulkan compute or wgpu) |
| **When worth it** | Datasets >10M entries with complex copula correlations |
| **Alternative** | CPU SIMD (AVX-512) gets ~4x for much less complexity |

**Recommendation**: GPU acceleration is most valuable for:
1. Copula-heavy workloads (correlated multi-variate generation)
2. Datasets exceeding 100M entries
3. Environments where GPUs are already available (cloud instances)

For most use cases, CPU parallelism with Rayon (Section 3) provides better
cost-effectiveness.

---

## 7. Algorithmic Improvements

### 7.1 Document Chain Linking (O(n²) → O(n))

The current document flow linking (PO → GR → Invoice → Payment) can degrade to
O(n²) when matching documents by tolerance windows.

**Strategy**: Use sorted indices with binary search:

```rust
// Before: linear scan for matching documents
fn find_matching_gr(po: &PurchaseOrder, grs: &[GoodsReceipt]) -> Option<&GoodsReceipt> {
    grs.iter().find(|gr| gr.po_number == po.number && within_tolerance(gr, po))
}

// After: pre-sort by PO number, binary search + range scan
fn find_matching_gr(po: &PurchaseOrder, grs: &SortedIndex) -> Option<&GoodsReceipt> {
    let range = grs.range_by_po(po.number);  // O(log n)
    range.iter().find(|gr| within_tolerance(gr, po))  // O(k) where k << n
}
```

For 100K POs with 100K GRs, this reduces matching from ~10B comparisons to ~100K×log(100K) ≈ 1.7M.

**Expected gain**: Document flow generation from O(n²) to O(n log n) — critical for
datasets >50K documents.

### 7.2 Alias Method for Weighted Selection

Account selection, company selection, and user selection all use weighted random
choice. The current linear-scan approach is O(n). The Vose alias method provides
O(1) weighted selection after O(n) preprocessing:

```rust
struct AliasTable {
    probability: Vec<f64>,
    alias: Vec<usize>,
}

impl AliasTable {
    fn new(weights: &[f64]) -> Self { /* O(n) setup */ }

    fn sample(&self, rng: &mut impl Rng) -> usize {
        let i = rng.gen_range(0..self.probability.len());
        let r: f64 = rng.gen();
        if r < self.probability[i] { i } else { self.alias[i] }
        // Always O(1), exactly 2 random numbers
    }
}
```

For a CoA with 2,500 accounts, this reduces per-sample selection from ~1,250 comparisons
(average) to exactly 2 operations.

**Expected gain**: 5-8% overall (account selection is ~8% of generation time).

### 7.3 Batch RNG Advancement

ChaCha8 can generate blocks of 64 bytes (8 `u64` values) at once internally.
Instead of calling `rng.gen()` once per value, request a block:

```rust
use rand::RngCore;

// Before: one value at a time (RNG state update per call)
let v1: u64 = rng.next_u64();
let v2: u64 = rng.next_u64();

// After: fill a buffer (RNG advances in larger steps)
let mut buf = [0u8; 64];
rng.fill_bytes(&mut buf);
// Extract values from buffer
```

This reduces RNG state management overhead by processing more random bytes per
ChaCha round.

**Expected gain**: 5-10% improvement in RNG-heavy generation.

### 7.4 Lazy Anomaly Injection

Currently, anomaly injection logic runs for every entry even when the injection
rate is low (typically 1-5%):

```rust
// Before: check every entry
for entry in &mut entries {
    if rng.gen::<f64>() < anomaly_rate {
        inject_anomaly(entry);
    }
}

// After: pre-compute anomaly positions, skip non-anomaly entries
let anomaly_positions: Vec<usize> = (0..total)
    .filter(|_| rng.gen::<f64>() < anomaly_rate)
    .collect();

// Generate normal entries in bulk, then patch anomalies
let mut entries = generate_normal_batch(total);
for pos in anomaly_positions {
    inject_anomaly(&mut entries[pos]);
}
```

Better yet, use geometric distribution to jump directly to the next anomaly position:

```rust
// O(anomalies) instead of O(entries)
let geometric = Geometric::new(anomaly_rate).unwrap();
let mut pos = geometric.sample(&mut rng) as usize;
while pos < total {
    inject_anomaly(&mut entries[pos]);
    pos += geometric.sample(&mut rng) as usize;
}
```

**Expected gain**: 2-3% by eliminating per-entry branching for rare events.

---

## 8. Architecture-Level Changes

### 8.1 Columnar Generation (Struct-of-Arrays)

The current architecture generates data in row-oriented format (Array-of-Structs):

```rust
// Current: Array of Structs (AoS)
struct JournalEntry {
    id: Uuid,
    date: NaiveDate,
    amount: Decimal,
    // ...
}
let entries: Vec<JournalEntry> = generate(1_000_000);
```

For high-throughput generation, a columnar (Struct-of-Arrays) layout is more
cache-friendly and SIMD-compatible:

```rust
// Proposed: Struct of Arrays (SoA)
struct JournalEntryBatch {
    ids: Vec<Uuid>,          // All IDs contiguous in memory
    dates: Vec<NaiveDate>,   // All dates contiguous
    amounts: Vec<Decimal>,   // All amounts contiguous
    // ...
}
```

**Benefits**:
- Generating all amounts at once allows SIMD vectorization
- Generating all dates at once allows batch temporal sampling
- Parquet output is inherently columnar — zero-copy possible
- Cache lines contain useful data (no wasted bytes from unrelated fields)

**Compatibility**: Provide an iterator adapter that reconstructs row-oriented
entries from the columnar batch for APIs that need `JournalEntry`:

```rust
impl JournalEntryBatch {
    fn iter_rows(&self) -> impl Iterator<Item = JournalEntryRef<'_>> {
        (0..self.len()).map(|i| JournalEntryRef {
            id: &self.ids[i],
            date: &self.dates[i],
            amount: &self.amounts[i],
        })
    }
}
```

**Expected gain**: 20-40% for large batches due to improved cache utilization and
vectorization opportunities. Particularly impactful for Parquet output (eliminates
row→column transpose).

### 8.2 Memory-Mapped Output

For very large datasets (>10GB output), use memory-mapped files to let the OS
manage page writeback:

```rust
use memmap2::MmapMut;

let file = OpenOptions::new().read(true).write(true).create(true).open(path)?;
file.set_len(estimated_size as u64)?;
let mut mmap = unsafe { MmapMut::map_mut(&file)? };

// Write directly to mapped memory — OS handles disk writeback
let mut offset = 0;
for entry in entries {
    let bytes = format_csv_row(&entry);
    mmap[offset..offset + bytes.len()].copy_from_slice(bytes.as_bytes());
    offset += bytes.len();
}

mmap.flush()?;
```

**Benefits**: The OS kernel manages when to flush pages to disk, using all available
RAM as a write cache. No explicit `BufWriter` needed. Async writeback means the
generation thread never blocks on I/O.

**Caveat**: Requires pre-estimating output file size (or using a generous upper bound
and truncating).

**Expected gain**: 15-25% for large files on systems with ample RAM.

### 8.3 Streaming Aggregation

Instead of collecting all entries in memory before writing, process and aggregate
on-the-fly:

```rust
// Current: collect everything, then compute statistics
let entries: Vec<JournalEntry> = generate_all();
let stats = compute_statistics(&entries);
write_all(&entries);

// Proposed: streaming aggregation
let mut stats = StreamingStats::new();
let (tx, rx) = bounded(4096);

// Generator thread
thread::spawn(move || {
    for _ in 0..total {
        let entry = generator.generate_one();
        stats.update(&entry);  // O(1) incremental statistics
        tx.send(entry).unwrap();
    }
});

// Writer thread
for entry in rx {
    sink.write(entry)?;
}
```

This keeps memory usage constant regardless of dataset size — critical for
100M+ entry generation where holding all entries in memory is infeasible.

**Expected gain**: Reduces peak memory from O(n) to O(1). Enables generation of
arbitrarily large datasets on machines with limited RAM.

### 8.4 Tiered Storage Pipeline

For datasets that exceed available RAM, use a tiered approach:

```
Level 1: CPU L1/L2 Cache  (~256KB-1MB)  → Current batch (256 entries)
Level 2: RAM               (~16-64GB)    → Write buffer + statistics
Level 3: NVMe SSD          (~1-8TB)      → Output files
Level 4: Network/Cloud     (unlimited)   → S3, GCS, Azure Blob
```

Each tier has a flush policy:
- L1→RAM: Every 256 entries (batch generation)
- RAM→SSD: Every 256KB (BufWriter flush)
- SSD→Cloud: After generation completes (async upload)

### 8.5 Distributed Generation

For truly massive datasets (1B+ entries), distribute generation across multiple
machines:

```
                    ┌─── Worker 1: Companies C001-C010 ──→ Partition 1
Coordinator ────────├─── Worker 2: Companies C011-C020 ──→ Partition 2
(assigns ranges)    ├─── Worker 3: Companies C021-C030 ──→ Partition 3
                    └─── Worker N: Companies C031-C040 ──→ Partition N
                                                           ↓
                                                    Merge/Consolidate
```

Each worker gets a seed derived from the master seed + partition index, ensuring
deterministic output. Workers are fully independent (no inter-worker communication
during generation).

The existing `datasynth-server` gRPC infrastructure can be extended with a
coordinator service:

```protobuf
service GenerationCoordinator {
    rpc SubmitJob(JobSpec) returns (JobId);
    rpc GetPartitions(JobId) returns (stream Partition);
    rpc WorkerReport(WorkerStatus) returns (Ack);
    rpc MergeResults(JobId) returns (MergeStatus);
}
```

**Expected gain**: Linear horizontal scaling. 10 workers ≈ 10x throughput.

---

## 9. Benchmarking & Measurement

### 9.1 Benchmarking Framework

The existing benchmark suite in `benches/` covers:
- `generation_throughput.rs` — entries/sec baseline
- `distribution_sampling.rs` — sampler performance
- `output_sinks.rs` — CSV/JSON/Parquet write speed
- `scalability.rs` — multi-core scaling
- `correctness.rs` — statistical compliance

**Additions needed**:

```rust
// benches/parallel_scaling.rs
fn bench_parallel_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_scaling");

    for threads in [1, 2, 4, 8, 16] {
        group.bench_with_input(
            BenchmarkId::new("je_generation", threads),
            &threads,
            |b, &t| {
                b.iter(|| {
                    let pool = rayon::ThreadPoolBuilder::new()
                        .num_threads(t)
                        .build().unwrap();
                    pool.install(|| generate_parallel(100_000, t))
                })
            },
        );
    }
    group.finish();
}
```

### 9.2 Key Metrics to Track

| Metric | Target | Current |
|--------|--------|---------|
| Entries/sec (single-threaded) | 200K | ~100K |
| Entries/sec (8 cores) | 1M+ | ~100K (no parallelism) |
| Peak memory (1M entries) | <500MB | ~650MB |
| Memory (streaming, any size) | <100MB | N/A (batch only) |
| CSV write throughput | 1M lines/sec | ~500K lines/sec |
| Parquet write throughput | 2M lines/sec | Not measured |
| Time to 10M entries (8 cores) | <15 sec | ~100 sec |
| Time to 100M entries (16 cores) | <120 sec | ~1000 sec |

### 9.3 Profiling Tools

```bash
# CPU profiling with perf (Linux)
perf record -g target/release/datasynth-data generate --demo -o /tmp/out
perf report

# Flamegraph
cargo install flamegraph
cargo flamegraph --bin datasynth-data -- generate --demo -o /tmp/out

# Memory profiling with DHAT
cargo install dhat
# Add dhat instrumentation, rebuild, run

# Cache analysis with cachegrind
valgrind --tool=cachegrind target/release/datasynth-data generate --demo -o /tmp/out

# Allocation tracking with heaptrack
heaptrack target/release/datasynth-data generate --demo -o /tmp/out
heaptrack_gui heaptrack.datasynth-data.*.gz
```

---

## 10. Implementation Roadmap

### Phase 1: Low-Hanging Fruit (Est. Impact: 2-3x)

These changes require minimal architectural changes and can be implemented
independently:

| # | Change | Impact | Effort | Risk |
|---|--------|--------|--------|------|
| 1 | Increase BufWriter to 256KB | 10-20% I/O | Trivial | None |
| 2 | Deferred Decimal conversion (use f64 internally) | 15-20% | Low | Low |
| 3 | Pre-computed CDF + alias tables for weighted selection | 5-8% | Low | None |
| 4 | SmallVec for line items | 5-8% | Low | None |
| 5 | `#[inline(always)]` on hot-path methods | 3-5% | Trivial | None |
| 6 | Batch RNG buffer | 5-10% | Low | None |

### Phase 2: Parallelism (Est. Impact: 4-10x)

| # | Change | Impact | Effort | Risk |
|---|--------|--------|--------|------|
| 7 | Implement `ParallelGenerator` for JE generator | 4-8x | Medium | Medium |
| 8 | Pipeline parallelism (generate ∥ write) | 20-40% | Medium | Low |
| 9 | Phase-level parallelism in orchestrator | 3-5x phases | Medium | Medium |
| 10 | Multi-company parallel generation | Linear w/ companies | Low | Low |
| 11 | Per-thread UUID counters | 5-10% at >4 threads | Low | None |

### Phase 3: I/O & Serialization (Est. Impact: 2-3x I/O)

| # | Change | Impact | Effort | Risk |
|---|--------|--------|--------|------|
| 12 | Vectorized CSV serialization (itoa/ryu) | 30-50% I/O | Medium | Low |
| 13 | Parallel compression (zstd multithreaded) | 2-3x effective I/O | Low | None |
| 14 | Streaming aggregation (constant memory) | Unbounded scale | High | Medium |
| 15 | Columnar generation (SoA layout) | 20-40% | High | High |

### Phase 4: Advanced (Est. Impact: 10x+ for specific workloads)

| # | Change | Impact | Effort | Risk |
|---|--------|--------|--------|------|
| 16 | GPU distribution sampling | 3-5x overall | High | High |
| 17 | Distributed generation | Linear horizontal | Very High | High |
| 18 | Memory-mapped output | 15-25% large files | Medium | Medium |
| 19 | io_uring async I/O (Linux) | 15-30% I/O | High | Medium |

### Combined Impact Estimate

```
Phase 1 alone:           ~2-3x   (100K → 200-300K entries/sec)
Phase 1 + 2:             ~10-20x (100K → 1-2M entries/sec, 8 cores)
Phase 1 + 2 + 3:         ~15-30x (with I/O no longer bottleneck)
Phase 1 + 2 + 3 + 4:     ~30-50x (GPU + distributed for extreme scale)
```

### Critical Path

The highest-impact changes with lowest risk are:

1. **Deferred Decimal** (Phase 1, #2) — easiest 15-20% win
2. **ParallelGenerator** (Phase 2, #7) — biggest single improvement (4-8x)
3. **Pipeline parallelism** (Phase 2, #8) — overlaps CPU and I/O
4. **Larger buffers** (Phase 1, #1) — trivial change, meaningful I/O gain

These four changes alone could take throughput from ~100K/sec to ~800K-1.2M/sec
on an 8-core machine.
