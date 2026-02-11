//! Diffusion model training pipeline: fit from column statistics, persist, and evaluate.
//!
//! The [`DiffusionTrainer`] fits a [`TrainedDiffusionModel`] from per-column statistics
//! (mean, std, min, max, type) and an optional correlation matrix. The trained model
//! can be serialized to JSON for persistence and later reloaded for generation.
//!
//! Generation uses the same statistical diffusion approach as
//! [`StatisticalDiffusionBackend`](super::StatisticalDiffusionBackend): start from
//! Gaussian noise, iteratively denoise toward the target distribution, then apply
//! correlation structure via Cholesky decomposition.

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::backend::DiffusionConfig;
use super::statistical::StatisticalDiffusionBackend;
use super::DiffusionBackend;
use crate::error::SynthError;

// ---------------------------------------------------------------------------
// Column types and parameters
// ---------------------------------------------------------------------------

/// The type of a column in the dataset.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ColumnType {
    /// A continuous (floating-point) column.
    Continuous,
    /// A categorical column with a fixed set of string categories.
    Categorical { categories: Vec<String> },
    /// An integer-valued column.
    Integer,
}

/// Statistical parameters for a single column in the trained model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDiffusionParams {
    /// Column name.
    pub name: String,
    /// Target mean.
    pub mean: f64,
    /// Target standard deviation.
    pub std: f64,
    /// Minimum observed value.
    pub min: f64,
    /// Maximum observed value.
    pub max: f64,
    /// Column type (continuous, categorical, integer).
    pub col_type: ColumnType,
}

// ---------------------------------------------------------------------------
// Metadata
// ---------------------------------------------------------------------------

/// Metadata about a trained diffusion model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    /// ISO-8601 timestamp of when the model was trained.
    pub training_timestamp: String,
    /// Number of diffusion steps used.
    pub n_steps: usize,
    /// Noise schedule type (e.g. "linear", "cosine", "sigmoid").
    pub schedule_type: String,
    /// Number of columns in the model.
    pub n_columns: usize,
    /// Model format version.
    pub version: String,
}

// ---------------------------------------------------------------------------
// Trained model
// ---------------------------------------------------------------------------

/// A trained diffusion model that can generate samples and be persisted to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainedDiffusionModel {
    /// Per-column statistical parameters.
    pub column_params: Vec<ColumnDiffusionParams>,
    /// Correlation matrix (n_columns x n_columns).
    pub correlation_matrix: Vec<Vec<f64>>,
    /// Diffusion configuration used during training.
    pub config: DiffusionConfig,
    /// Model metadata.
    pub metadata: ModelMetadata,
}

impl TrainedDiffusionModel {
    /// Generate `n_samples` rows of synthetic data using the trained model parameters.
    ///
    /// Each row contains one value per column. Column types are respected:
    /// - **Continuous**: clipped to `[min, max]`
    /// - **Integer**: rounded and clipped to `[min, max]`
    /// - **Categorical**: mapped to category indices, rounded and clipped to `[0, n_categories - 1]`
    pub fn generate(&self, n_samples: usize, seed: u64) -> Vec<Vec<f64>> {
        let n_features = self.column_params.len();
        if n_samples == 0 || n_features == 0 {
            return vec![];
        }

        // Use the StatisticalDiffusionBackend for the core generation
        let means: Vec<f64> = self.column_params.iter().map(|c| c.mean).collect();
        let stds: Vec<f64> = self.column_params.iter().map(|c| c.std.max(1e-8)).collect();

        let backend = StatisticalDiffusionBackend::new(means, stds, self.config.clone())
            .with_correlations(self.correlation_matrix.clone());

        let mut samples = backend.generate(n_samples, n_features, seed);

        // Post-process according to column types
        for row in samples.iter_mut() {
            for (j, val) in row.iter_mut().enumerate() {
                if j >= self.column_params.len() {
                    continue;
                }
                let col = &self.column_params[j];
                match &col.col_type {
                    ColumnType::Continuous => {
                        *val = val.clamp(col.min, col.max);
                    }
                    ColumnType::Integer => {
                        *val = val.round().clamp(col.min, col.max);
                    }
                    ColumnType::Categorical { categories } => {
                        let n_cats = categories.len().max(1) as f64;
                        *val = val.round().clamp(0.0, n_cats - 1.0);
                    }
                }
            }
        }

        samples
    }

    /// Serialize and save the model to a JSON file at `path`.
    pub fn save(&self, path: &Path) -> Result<(), SynthError> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| SynthError::generation(format!("Failed to serialize model: {e}")))?;
        std::fs::write(path, json).map_err(|e| {
            SynthError::generation(format!("Failed to write model to {}: {e}", path.display()))
        })?;
        Ok(())
    }

    /// Load a model from a JSON file at `path`.
    pub fn load(path: &Path) -> Result<Self, SynthError> {
        let data = std::fs::read_to_string(path).map_err(|e| {
            SynthError::generation(format!("Failed to read model from {}: {e}", path.display()))
        })?;
        let model: Self = serde_json::from_str(&data)
            .map_err(|e| SynthError::generation(format!("Failed to deserialize model: {e}")))?;
        Ok(model)
    }
}

// ---------------------------------------------------------------------------
// Trainer
// ---------------------------------------------------------------------------

/// Trainer that fits diffusion model parameters from column statistics.
///
/// This is a stateless builder: call [`DiffusionTrainer::fit`] with the desired
/// parameters, then use the returned [`TrainedDiffusionModel`] for generation or
/// persistence.
pub struct DiffusionTrainer;

impl DiffusionTrainer {
    /// Fit a diffusion model from per-column statistics and a correlation matrix.
    ///
    /// The resulting [`TrainedDiffusionModel`] captures the target distribution
    /// and can generate new samples via its `generate` method.
    pub fn fit(
        column_params: Vec<ColumnDiffusionParams>,
        correlation_matrix: Vec<Vec<f64>>,
        config: DiffusionConfig,
    ) -> TrainedDiffusionModel {
        let schedule_type = match config.schedule {
            super::backend::NoiseScheduleType::Linear => "linear".to_string(),
            super::backend::NoiseScheduleType::Cosine => "cosine".to_string(),
            super::backend::NoiseScheduleType::Sigmoid => "sigmoid".to_string(),
        };

        let metadata = ModelMetadata {
            training_timestamp: chrono::Utc::now().to_rfc3339(),
            n_steps: config.n_steps,
            schedule_type,
            n_columns: column_params.len(),
            version: "1.0.0".to_string(),
        };

        TrainedDiffusionModel {
            column_params,
            correlation_matrix,
            config,
            metadata,
        }
    }

    /// Evaluate a trained model by comparing generated samples against the
    /// target statistics captured in the model.
    ///
    /// Returns a [`FitReport`] with per-column errors, correlation error, and
    /// an overall quality score.
    pub fn evaluate(model: &TrainedDiffusionModel, n_eval_samples: usize, seed: u64) -> FitReport {
        let samples = model.generate(n_eval_samples, seed);
        let n_cols = model.column_params.len();

        if samples.is_empty() || n_cols == 0 {
            return FitReport {
                mean_errors: vec![],
                std_errors: vec![],
                correlation_error: 0.0,
                overall_score: 0.0,
            };
        }

        let n = samples.len() as f64;

        // Per-column mean and std errors (normalized by target std)
        let mut mean_errors = Vec::with_capacity(n_cols);
        let mut std_errors = Vec::with_capacity(n_cols);

        for j in 0..n_cols {
            let col = &model.column_params[j];
            let target_std = col.std.max(1e-8);

            let sample_mean: f64 = samples.iter().map(|r| r[j]).sum::<f64>() / n;
            let sample_var: f64 = samples
                .iter()
                .map(|r| (r[j] - sample_mean).powi(2))
                .sum::<f64>()
                / n;
            let sample_std = sample_var.sqrt();

            let me = (sample_mean - col.mean).abs() / target_std;
            let se = (sample_std - col.std).abs() / target_std;

            mean_errors.push(me);
            std_errors.push(se);
        }

        // Correlation matrix error: Frobenius norm of difference
        let correlation_error = Self::compute_correlation_error(&samples, model);

        // Overall score: 1.0 - mean(all individual errors), clamped to [0, 1]
        let all_errors: Vec<f64> = mean_errors
            .iter()
            .chain(std_errors.iter())
            .copied()
            .collect();
        let total_error_count = all_errors.len() as f64;
        let avg_error = if total_error_count > 0.0 {
            all_errors.iter().sum::<f64>() / total_error_count
        } else {
            0.0
        };
        // Include correlation error in the overall score (weighted equally with
        // the average of the per-column errors)
        let combined = (avg_error + correlation_error) / 2.0;
        let overall_score = (1.0 - combined).clamp(0.0, 1.0);

        FitReport {
            mean_errors,
            std_errors,
            correlation_error,
            overall_score,
        }
    }

    /// Compute the Frobenius norm of (sample_corr - target_corr) normalized by
    /// the number of elements.
    fn compute_correlation_error(samples: &[Vec<f64>], model: &TrainedDiffusionModel) -> f64 {
        let n_cols = model.column_params.len();
        if n_cols < 2 || samples.is_empty() {
            return 0.0;
        }

        let n = samples.len() as f64;

        // Compute sample means
        let mut means = vec![0.0; n_cols];
        for row in samples {
            for (j, &v) in row.iter().enumerate().take(n_cols) {
                means[j] += v;
            }
        }
        for m in &mut means {
            *m /= n;
        }

        // Compute sample stds
        let mut stds = vec![0.0; n_cols];
        for row in samples {
            for (j, &v) in row.iter().enumerate().take(n_cols) {
                stds[j] += (v - means[j]).powi(2);
            }
        }
        for s in &mut stds {
            *s = (*s / n).sqrt().max(1e-8);
        }

        // Compute sample correlation matrix
        let mut sample_corr = vec![vec![0.0; n_cols]; n_cols];
        for row in samples {
            for i in 0..n_cols {
                for j in 0..n_cols {
                    sample_corr[i][j] += (row[i] - means[i]) * (row[j] - means[j]);
                }
            }
        }
        for (i, corr_row) in sample_corr.iter_mut().enumerate().take(n_cols) {
            for (j, corr_val) in corr_row.iter_mut().enumerate().take(n_cols) {
                *corr_val /= n * stds[i] * stds[j];
            }
        }

        // Frobenius norm of difference, normalized by matrix size
        let target_corr = &model.correlation_matrix;
        let mut frobenius_sq = 0.0;
        for (i, corr_row) in sample_corr.iter().enumerate().take(n_cols) {
            for (j, &corr_val) in corr_row.iter().enumerate().take(n_cols) {
                let target_val = target_corr
                    .get(i)
                    .and_then(|row| row.get(j))
                    .copied()
                    .unwrap_or(if i == j { 1.0 } else { 0.0 });
                let diff = corr_val - target_val;
                frobenius_sq += diff * diff;
            }
        }

        (frobenius_sq / (n_cols * n_cols) as f64).sqrt()
    }
}

// ---------------------------------------------------------------------------
// Fit report
// ---------------------------------------------------------------------------

/// Report from evaluating a trained diffusion model against its target statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitReport {
    /// Per-column normalized mean error: `|sample_mean - target_mean| / target_std`.
    pub mean_errors: Vec<f64>,
    /// Per-column normalized std error: `|sample_std - target_std| / target_std`.
    pub std_errors: Vec<f64>,
    /// Root-mean-square element difference between sample and target correlation matrices.
    pub correlation_error: f64,
    /// Overall quality score in `[0.0, 1.0]` (higher is better).
    pub overall_score: f64,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::super::backend::NoiseScheduleType;
    use super::*;

    fn make_config(n_steps: usize, seed: u64) -> DiffusionConfig {
        DiffusionConfig {
            n_steps,
            schedule: NoiseScheduleType::Linear,
            seed,
        }
    }

    fn sample_column_params() -> Vec<ColumnDiffusionParams> {
        vec![
            ColumnDiffusionParams {
                name: "amount".to_string(),
                mean: 100.0,
                std: 15.0,
                min: 0.0,
                max: 500.0,
                col_type: ColumnType::Continuous,
            },
            ColumnDiffusionParams {
                name: "quantity".to_string(),
                mean: 10.0,
                std: 3.0,
                min: 1.0,
                max: 50.0,
                col_type: ColumnType::Integer,
            },
            ColumnDiffusionParams {
                name: "category".to_string(),
                mean: 1.5,
                std: 0.8,
                min: 0.0,
                max: 3.0,
                col_type: ColumnType::Categorical {
                    categories: vec![
                        "A".to_string(),
                        "B".to_string(),
                        "C".to_string(),
                        "D".to_string(),
                    ],
                },
            },
        ]
    }

    fn sample_correlation_matrix() -> Vec<Vec<f64>> {
        vec![
            vec![1.0, 0.6, 0.2],
            vec![0.6, 1.0, 0.3],
            vec![0.2, 0.3, 1.0],
        ]
    }

    // 1. Fit from column params produces valid model
    #[test]
    fn test_fit_produces_valid_model() {
        let params = sample_column_params();
        let corr = sample_correlation_matrix();
        let config = make_config(100, 42);

        let model = DiffusionTrainer::fit(params.clone(), corr.clone(), config);

        assert_eq!(model.column_params.len(), 3);
        assert_eq!(model.correlation_matrix.len(), 3);
        assert_eq!(model.metadata.n_columns, 3);
        assert_eq!(model.metadata.n_steps, 100);
        assert_eq!(model.metadata.schedule_type, "linear");
        assert_eq!(model.metadata.version, "1.0.0");
        assert!(!model.metadata.training_timestamp.is_empty());

        // Column params are preserved
        assert_eq!(model.column_params[0].name, "amount");
        assert!((model.column_params[0].mean - 100.0).abs() < 1e-10);
        assert_eq!(model.column_params[1].col_type, ColumnType::Integer);
    }

    // 2. Save/load roundtrip produces identical model
    #[test]
    fn test_save_load_roundtrip() {
        let model = DiffusionTrainer::fit(
            sample_column_params(),
            sample_correlation_matrix(),
            make_config(50, 42),
        );

        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let path = dir.path().join("model.json");

        model.save(&path).expect("Failed to save model");
        let loaded = TrainedDiffusionModel::load(&path).expect("Failed to load model");

        // Verify all fields match
        assert_eq!(model.column_params.len(), loaded.column_params.len());
        for (orig, load) in model.column_params.iter().zip(loaded.column_params.iter()) {
            assert_eq!(orig.name, load.name);
            assert!((orig.mean - load.mean).abs() < 1e-10);
            assert!((orig.std - load.std).abs() < 1e-10);
            assert!((orig.min - load.min).abs() < 1e-10);
            assert!((orig.max - load.max).abs() < 1e-10);
            assert_eq!(orig.col_type, load.col_type);
        }
        assert_eq!(model.correlation_matrix, loaded.correlation_matrix);
        assert_eq!(model.config.n_steps, loaded.config.n_steps);
        assert_eq!(model.config.seed, loaded.config.seed);
        assert_eq!(model.metadata.version, loaded.metadata.version);
        assert_eq!(
            model.metadata.training_timestamp,
            loaded.metadata.training_timestamp
        );
    }

    // 3. Generated samples have correct dimensions
    #[test]
    fn test_generate_correct_dimensions() {
        let model = DiffusionTrainer::fit(
            sample_column_params(),
            sample_correlation_matrix(),
            make_config(50, 42),
        );

        let samples = model.generate(200, 99);
        assert_eq!(samples.len(), 200);
        for row in &samples {
            assert_eq!(row.len(), 3);
        }
    }

    // 4. Generated means within tolerance of target
    #[test]
    fn test_generated_means_within_tolerance() {
        let params = sample_column_params();
        let model = DiffusionTrainer::fit(
            params.clone(),
            sample_correlation_matrix(),
            make_config(100, 42),
        );

        let samples = model.generate(5000, 42);
        let n = samples.len() as f64;

        for (j, col) in params.iter().enumerate() {
            let sample_mean: f64 = samples.iter().map(|r| r[j]).sum::<f64>() / n;
            // Allow tolerance of 2 * target_std (generous but verifies convergence)
            let tolerance = 2.0 * col.std;
            assert!(
                (sample_mean - col.mean).abs() < tolerance,
                "Column {} ('{}') mean {} is more than {} from target {}",
                j,
                col.name,
                sample_mean,
                tolerance,
                col.mean,
            );
        }
    }

    // 5. Same seed produces same output
    #[test]
    fn test_same_seed_deterministic() {
        let model = DiffusionTrainer::fit(
            sample_column_params(),
            sample_correlation_matrix(),
            make_config(50, 42),
        );

        let samples1 = model.generate(100, 123);
        let samples2 = model.generate(100, 123);

        for (row1, row2) in samples1.iter().zip(samples2.iter()) {
            for (&v1, &v2) in row1.iter().zip(row2.iter()) {
                assert!(
                    (v1 - v2).abs() < 1e-12,
                    "Determinism failed: {} vs {}",
                    v1,
                    v2,
                );
            }
        }
    }

    // 6. Different seeds produce different output
    #[test]
    fn test_different_seeds_differ() {
        let model = DiffusionTrainer::fit(
            sample_column_params(),
            sample_correlation_matrix(),
            make_config(50, 42),
        );

        let samples1 = model.generate(100, 1);
        let samples2 = model.generate(100, 2);

        let mut any_diff = false;
        for (row1, row2) in samples1.iter().zip(samples2.iter()) {
            for (&v1, &v2) in row1.iter().zip(row2.iter()) {
                if (v1 - v2).abs() > 1e-8 {
                    any_diff = true;
                    break;
                }
            }
            if any_diff {
                break;
            }
        }
        assert!(any_diff, "Different seeds should produce different samples");
    }

    // 7. Evaluation report has reasonable scores for well-fitted model
    #[test]
    fn test_evaluation_reasonable_scores() {
        let model = DiffusionTrainer::fit(
            sample_column_params(),
            sample_correlation_matrix(),
            make_config(100, 42),
        );

        let report = DiffusionTrainer::evaluate(&model, 5000, 42);

        // Mean errors should be small (less than 1.0 normalized by std)
        for (j, &me) in report.mean_errors.iter().enumerate() {
            assert!(
                me < 1.0,
                "Column {} mean error {} is too large (should be < 1.0)",
                j,
                me,
            );
        }

        // Std errors should be reasonable
        for (j, &se) in report.std_errors.iter().enumerate() {
            assert!(
                se < 1.5,
                "Column {} std error {} is too large (should be < 1.5)",
                j,
                se,
            );
        }

        // Correlation error should be bounded
        assert!(
            report.correlation_error < 1.0,
            "Correlation error {} is too large",
            report.correlation_error,
        );

        // Overall score should be positive for a reasonable model
        assert!(
            report.overall_score > 0.0,
            "Overall score {} should be positive",
            report.overall_score,
        );
    }

    // 8. Correlation structure is preserved
    #[test]
    fn test_correlation_structure_preserved() {
        // Use only 2 continuous columns with strong correlation for cleaner test
        let params = vec![
            ColumnDiffusionParams {
                name: "x".to_string(),
                mean: 0.0,
                std: 1.0,
                min: -10.0,
                max: 10.0,
                col_type: ColumnType::Continuous,
            },
            ColumnDiffusionParams {
                name: "y".to_string(),
                mean: 0.0,
                std: 1.0,
                min: -10.0,
                max: 10.0,
                col_type: ColumnType::Continuous,
            },
        ];
        let corr = vec![vec![1.0, 0.8], vec![0.8, 1.0]];

        let model = DiffusionTrainer::fit(params, corr, make_config(100, 42));
        let samples = model.generate(5000, 42);

        // Compute sample correlation
        let n = samples.len() as f64;
        let mean_x: f64 = samples.iter().map(|r| r[0]).sum::<f64>() / n;
        let mean_y: f64 = samples.iter().map(|r| r[1]).sum::<f64>() / n;
        let std_x: f64 = (samples.iter().map(|r| (r[0] - mean_x).powi(2)).sum::<f64>() / n).sqrt();
        let std_y: f64 = (samples.iter().map(|r| (r[1] - mean_y).powi(2)).sum::<f64>() / n).sqrt();
        let cov_xy: f64 = samples
            .iter()
            .map(|r| (r[0] - mean_x) * (r[1] - mean_y))
            .sum::<f64>()
            / n;

        let sample_corr = if std_x > 1e-8 && std_y > 1e-8 {
            cov_xy / (std_x * std_y)
        } else {
            0.0
        };

        // Correlation should be positive and reasonably close to 0.8
        assert!(
            sample_corr > 0.3,
            "Expected positive correlation near 0.8, got {}",
            sample_corr,
        );
    }

    // 9. Integer column type produces integer values
    #[test]
    fn test_integer_column_produces_integers() {
        let params = vec![ColumnDiffusionParams {
            name: "count".to_string(),
            mean: 10.0,
            std: 3.0,
            min: 1.0,
            max: 50.0,
            col_type: ColumnType::Integer,
        }];
        let corr = vec![vec![1.0]];

        let model = DiffusionTrainer::fit(params, corr, make_config(50, 42));
        let samples = model.generate(500, 42);

        for row in &samples {
            let val = row[0];
            assert!(
                (val - val.round()).abs() < 1e-10,
                "Integer column produced non-integer value: {}",
                val,
            );
            assert!(
                val >= 1.0 && val <= 50.0,
                "Integer column value {} out of range [1, 50]",
                val,
            );
        }
    }

    // 10. Categorical column produces valid indices
    #[test]
    fn test_categorical_column_produces_valid_indices() {
        let params = vec![ColumnDiffusionParams {
            name: "category".to_string(),
            mean: 1.0,
            std: 0.8,
            min: 0.0,
            max: 2.0,
            col_type: ColumnType::Categorical {
                categories: vec!["A".to_string(), "B".to_string(), "C".to_string()],
            },
        }];
        let corr = vec![vec![1.0]];

        let model = DiffusionTrainer::fit(params, corr, make_config(50, 42));
        let samples = model.generate(500, 42);

        for row in &samples {
            let val = row[0];
            assert!(
                (val - val.round()).abs() < 1e-10,
                "Categorical column produced non-integer index: {}",
                val,
            );
            assert!(
                val >= 0.0 && val <= 2.0,
                "Categorical index {} out of range [0, 2]",
                val,
            );
        }
    }

    // 11. Empty model generates empty samples
    #[test]
    fn test_empty_model_generates_empty() {
        let model = DiffusionTrainer::fit(vec![], vec![], make_config(50, 0));
        let samples = model.generate(100, 0);
        assert!(samples.is_empty());
    }

    // 12. Load from non-existent file returns error
    #[test]
    fn test_load_nonexistent_returns_error() {
        let result = TrainedDiffusionModel::load(Path::new("/tmp/nonexistent_model_12345.json"));
        assert!(result.is_err());
    }
}
