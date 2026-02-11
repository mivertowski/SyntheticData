# Diffusion Models

> **New in v0.5.0**

DataSynth integrates a statistical diffusion model backend for learned distribution capture, offering an alternative and complement to rule-based generation.

## Overview

Diffusion models generate data through a learned denoising process: starting from pure noise and iteratively removing it to produce realistic samples. DataSynth's implementation uses a statistical backend that captures column-level distributions and inter-column correlations from fingerprint data, then generates new samples through a configurable noise schedule.

```
Forward Process (Training):     x₀ → x₁ → x₂ → ... → xₜ (pure noise)
Reverse Process (Generation):   xₜ → xₜ₋₁ → ... → x₁ → x₀ (data)
```

## Architecture

### DiffusionBackend Trait

All diffusion backends implement a common interface:

```rust
pub trait DiffusionBackend: Send + Sync {
    fn name(&self) -> &str;
    fn forward(&self, x: &[Vec<f64>], t: usize) -> Vec<Vec<f64>>;
    fn reverse(&self, x_t: &[Vec<f64>], t: usize) -> Vec<Vec<f64>>;
    fn generate(&self, n_samples: usize, n_features: usize, seed: u64) -> Vec<Vec<f64>>;
}
```

### Statistical Diffusion Backend

The `StatisticalDiffusionBackend` uses per-column means and standard deviations (extracted from fingerprint data) to guide the denoising process:

```rust
use synth_core::diffusion::{StatisticalDiffusionBackend, DiffusionConfig, NoiseScheduleType};

let config = DiffusionConfig {
    n_steps: 1000,
    schedule: NoiseScheduleType::Cosine,
    seed: 42,
};

let backend = StatisticalDiffusionBackend::new(
    vec![5000.0, 3.5, 2.0],    // column means
    vec![2000.0, 1.5, 0.8],    // column standard deviations
    config,
);

// Optionally add correlation structure
let backend = backend.with_correlations(vec![
    vec![1.0, 0.65, 0.72],
    vec![0.65, 1.0, 0.55],
    vec![0.72, 0.55, 1.0],
]);

let samples = backend.generate(1000, 3, 42);
```

## Noise Schedules

The noise schedule controls how noise is added during the forward process and removed during the reverse process.

| Schedule | Formula | Characteristics |
|----------|---------|-----------------|
| **Linear** | β_t = β_min + t/T × (β_max - β_min) | Uniform noise addition; simple and robust |
| **Cosine** | β_t = 1 - ᾱ_t/ᾱ_{t-1}, ᾱ_t = cos²(π/2 × t/T) | Slower noise addition; better for preserving fine details |
| **Sigmoid** | β_t = sigmoid(a + (b-a) × t/T) | Smooth transition; balanced between linear and cosine |

```rust
use synth_core::diffusion::{NoiseSchedule, NoiseScheduleType};

let schedule = NoiseSchedule::new(&NoiseScheduleType::Cosine, 1000);

// Access schedule components
println!("Steps: {}", schedule.n_steps());
println!("First beta: {}", schedule.betas[0]);
println!("Last alpha_bar: {}", schedule.alpha_bars[999]);
```

### Schedule Properties

The `NoiseSchedule` precomputes all values needed for efficient forward/reverse steps:

| Property | Description |
|----------|-------------|
| `betas` | Noise variance at each step |
| `alphas` | 1 - beta at each step |
| `alpha_bars` | Cumulative product of alphas |
| `sqrt_alpha_bars` | √(ᾱ_t) for forward process |
| `sqrt_one_minus_alpha_bars` | √(1 - ᾱ_t) for noise scaling |

## Hybrid Generation

The `HybridGenerator` blends rule-based and diffusion-generated data to combine the structural guarantees of rule-based generation with the distributional fidelity of diffusion models.

### Blend Strategies

| Strategy | Description | Best For |
|----------|-------------|----------|
| **Interpolate** | Weighted average: `w × diffusion + (1-w) × rule_based` | Smooth blending of continuous values |
| **Select** | Per-record random selection from either source | Maintaining distinct record characteristics |
| **Ensemble** | Column-level: diffusion for amounts, rule-based for categoricals | Mixed-type data with different generation needs |

```rust
use synth_core::diffusion::{HybridGenerator, BlendStrategy};

let hybrid = HybridGenerator::new(0.3);  // 30% diffusion weight
println!("Weight: {}", hybrid.weight());

// Interpolation blend
let blended = hybrid.blend(
    &rule_based_data,
    &diffusion_data,
    BlendStrategy::Interpolate,
    42,
);

// Ensemble blend (specify which columns use diffusion)
let ensemble = hybrid.blend_ensemble(
    &rule_based_data,
    &diffusion_data,
    &[0, 2],  // columns 0 and 2 from diffusion
);
```

## Training Pipeline

The `DiffusionTrainer` fits a model from column-level parameters and correlation matrices (typically extracted from a fingerprint):

### Training

```rust
use synth_core::diffusion::{DiffusionTrainer, ColumnDiffusionParams, ColumnType, DiffusionConfig};

let params = vec![
    ColumnDiffusionParams {
        name: "amount".into(),
        mean: 5000.0,
        std: 2000.0,
        min: 0.0,
        max: 100000.0,
        col_type: ColumnType::Continuous,
    },
    ColumnDiffusionParams {
        name: "line_items".into(),
        mean: 3.5,
        std: 1.5,
        min: 1.0,
        max: 20.0,
        col_type: ColumnType::Integer,
    },
];

let corr_matrix = vec![
    vec![1.0, 0.65],
    vec![0.65, 1.0],
];

let config = DiffusionConfig { n_steps: 1000, schedule: NoiseScheduleType::Cosine, seed: 42 };
let model = DiffusionTrainer::fit(params, corr_matrix, config);
```

### Generation from Trained Model

```rust
let samples = model.generate(5000, 42);

// Save/load model
model.save(Path::new("./model.json"))?;
let loaded = TrainedDiffusionModel::load(Path::new("./model.json"))?;
```

### Evaluation

```rust
let report = DiffusionTrainer::evaluate(&model, 5000, 42);

println!("Overall score: {:.3}", report.overall_score);
println!("Correlation error: {:.4}", report.correlation_error);
for (i, (mean_err, std_err)) in report.mean_errors.iter().zip(&report.std_errors).enumerate() {
    println!("Column {}: mean_err={:.4}, std_err={:.4}", i, mean_err, std_err);
}
```

The `FitReport` contains:

| Metric | Description |
|--------|-------------|
| `mean_errors` | Per-column mean absolute error |
| `std_errors` | Per-column standard deviation error |
| `correlation_error` | RMSE of correlation matrix |
| `overall_score` | Weighted composite score (0-1, higher is better) |

## CLI Usage

### Train a Model

```bash
datasynth-data diffusion train \
    --fingerprint ./fingerprint.dsf \
    --output ./model.json \
    --n-steps 1000 \
    --schedule cosine
```

### Evaluate a Model

```bash
datasynth-data diffusion evaluate \
    --model ./model.json \
    --samples 5000
```

## Configuration

```yaml
diffusion:
  enabled: true
  n_steps: 1000           # Number of diffusion steps
  schedule: "cosine"       # Noise schedule: linear, cosine, sigmoid
  sample_size: 1000        # Samples to generate
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `false` | Enable diffusion generation |
| `n_steps` | integer | `1000` | Forward/reverse diffusion steps |
| `schedule` | string | `"linear"` | Noise schedule type |
| `sample_size` | integer | `1000` | Number of samples |

## Utility Functions

DataSynth provides helper functions for working with diffusion data:

```rust
use synth_core::diffusion::{
    add_gaussian_noise, normalize_features, denormalize_features,
    clip_values, generate_noise,
};

// Normalize data to zero mean, unit variance
let (normalized, means, stds) = normalize_features(&data);

// Add calibrated noise
let noisy = add_gaussian_noise(&normalized[0], 0.1, &mut rng);

// Denormalize back to original scale
let original_scale = denormalize_features(&generated, &means, &stds);

// Clip to valid ranges
clip_values(&mut samples, 0.0, 100000.0);
```

## See Also

- [AI & ML Configuration](../configuration/ai-ml-features.md)
- [datasynth-core Diffusion Module](../crates/datasynth-core.md)
- [Generation Pipeline](../architecture/generation-pipeline.md)
