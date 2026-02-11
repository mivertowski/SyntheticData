//! Correlation fingerprint models.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Correlation fingerprint containing relationship information between columns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationFingerprint {
    /// Correlation matrices by table.
    pub matrices: HashMap<String, CorrelationMatrix>,

    /// Cross-table correlations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cross_table_correlations: Vec<CrossTableCorrelation>,

    /// Gaussian copulas for multivariate generation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub copulas: Vec<GaussianCopula>,
}

impl CorrelationFingerprint {
    /// Create a new empty correlation fingerprint.
    pub fn new() -> Self {
        Self {
            matrices: HashMap::new(),
            cross_table_correlations: Vec::new(),
            copulas: Vec::new(),
        }
    }

    /// Add a correlation matrix for a table.
    pub fn add_matrix(&mut self, table: impl Into<String>, matrix: CorrelationMatrix) {
        self.matrices.insert(table.into(), matrix);
    }

    /// Add a copula.
    pub fn add_copula(&mut self, copula: GaussianCopula) {
        self.copulas.push(copula);
    }
}

impl Default for CorrelationFingerprint {
    fn default() -> Self {
        Self::new()
    }
}

/// Correlation matrix for numeric columns within a table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationMatrix {
    /// Column names in the matrix (determines order).
    pub columns: Vec<String>,

    /// Correlation values (Pearson) as flattened upper triangular matrix.
    /// Order: (0,1), (0,2), ..., (0,n-1), (1,2), (1,3), ..., (n-2, n-1)
    pub correlations: Vec<f64>,

    /// Correlation type used.
    pub correlation_type: CorrelationType,

    /// Sample size used for computation.
    pub sample_size: u64,
}

impl CorrelationMatrix {
    /// Create a new correlation matrix.
    pub fn new(columns: Vec<String>, correlation_type: CorrelationType) -> Self {
        let n = columns.len();
        let size = n * (n - 1) / 2; // Upper triangular without diagonal
        Self {
            columns,
            correlations: vec![0.0; size],
            correlation_type,
            sample_size: 0,
        }
    }

    /// Get the correlation between two columns by index.
    pub fn get(&self, i: usize, j: usize) -> Option<f64> {
        if i == j {
            return Some(1.0); // Diagonal
        }

        let (low, high) = if i < j { (i, j) } else { (j, i) };
        let n = self.columns.len();
        if high >= n {
            return None;
        }

        // Calculate index in flattened upper triangular
        // Index = sum(n-1 + n-2 + ... + n-low) + (high - low - 1)
        let idx = (0..low).map(|k| n - k - 1).sum::<usize>() + (high - low - 1);
        self.correlations.get(idx).copied()
    }

    /// Set the correlation between two columns by index.
    pub fn set(&mut self, i: usize, j: usize, value: f64) {
        if i == j {
            return; // Cannot set diagonal
        }

        let (low, high) = if i < j { (i, j) } else { (j, i) };
        let n = self.columns.len();
        if high >= n {
            return;
        }

        let idx = (0..low).map(|k| n - k - 1).sum::<usize>() + (high - low - 1);
        if idx < self.correlations.len() {
            self.correlations[idx] = value;
        }
    }

    /// Get the correlation between two columns by name.
    pub fn get_by_name(&self, col1: &str, col2: &str) -> Option<f64> {
        let i = self.columns.iter().position(|c| c == col1)?;
        let j = self.columns.iter().position(|c| c == col2)?;
        self.get(i, j)
    }

    /// Convert to a full square matrix.
    pub fn to_full_matrix(&self) -> Vec<Vec<f64>> {
        let n = self.columns.len();
        let mut matrix = vec![vec![0.0; n]; n];

        for i in 0..n {
            for j in 0..n {
                matrix[i][j] = self.get(i, j).unwrap_or(0.0);
            }
        }

        matrix
    }

    /// Create from a full square matrix.
    pub fn from_full_matrix(
        columns: Vec<String>,
        matrix: &[Vec<f64>],
        correlation_type: CorrelationType,
    ) -> Self {
        let n = columns.len();
        let size = n * (n - 1) / 2;
        let mut correlations = Vec::with_capacity(size);

        for i in 0..n {
            for j in (i + 1)..n {
                correlations.push(matrix[i][j]);
            }
        }

        Self {
            columns,
            correlations,
            correlation_type,
            sample_size: 0,
        }
    }
}

/// Types of correlation coefficients.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorrelationType {
    /// Pearson correlation for numeric-numeric.
    Pearson,
    /// Spearman rank correlation (more robust to outliers).
    Spearman,
    /// Kendall's tau (for ordinal data).
    Kendall,
    /// Cramer's V for categorical-categorical.
    CramersV,
    /// Eta coefficient for numeric-categorical.
    Eta,
    /// Point-biserial for numeric-binary.
    PointBiserial,
}

/// Cross-table correlation between columns in different tables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossTableCorrelation {
    /// First table and column.
    pub table1: String,
    pub column1: String,

    /// Second table and column.
    pub table2: String,
    pub column2: String,

    /// Correlation value.
    pub correlation: f64,

    /// Type of correlation.
    pub correlation_type: CorrelationType,

    /// Sample size.
    pub sample_size: u64,

    /// Join key used to compute correlation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub join_key: Option<JoinKey>,
}

/// Join key for cross-table correlations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinKey {
    /// Column in table1.
    pub column1: String,
    /// Column in table2.
    pub column2: String,
}

/// Gaussian copula for preserving multivariate dependencies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaussianCopula {
    /// Name identifier for this copula.
    pub name: String,

    /// Table this copula applies to.
    pub table: String,

    /// Columns included in the copula.
    pub columns: Vec<String>,

    /// Correlation matrix (Pearson correlations after marginal transformation).
    pub correlation_matrix: Vec<f64>,

    /// Marginal CDFs for each column (as empirical CDFs).
    pub marginal_cdfs: Vec<EmpiricalCdf>,
}

impl GaussianCopula {
    /// Create a new Gaussian copula.
    pub fn new(name: impl Into<String>, table: impl Into<String>, columns: Vec<String>) -> Self {
        let n = columns.len();
        Self {
            name: name.into(),
            table: table.into(),
            columns,
            correlation_matrix: vec![1.0; n * n], // Identity initially
            marginal_cdfs: Vec::new(),
        }
    }

    /// Get the number of dimensions.
    pub fn dimensions(&self) -> usize {
        self.columns.len()
    }
}

/// Empirical CDF representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmpiricalCdf {
    /// Column name.
    pub column: String,

    /// Sorted values (after privacy processing).
    pub values: Vec<f64>,

    /// Cumulative probabilities.
    pub probabilities: Vec<f64>,
}

impl EmpiricalCdf {
    /// Create an empirical CDF from sorted values.
    pub fn from_sorted_values(column: impl Into<String>, values: Vec<f64>) -> Self {
        let n = values.len();
        let probabilities: Vec<f64> = (1..=n).map(|i| i as f64 / n as f64).collect();

        Self {
            column: column.into(),
            values,
            probabilities,
        }
    }

    /// Evaluate CDF at a value.
    pub fn cdf(&self, x: f64) -> f64 {
        match self.values.binary_search_by(|v| v.total_cmp(&x)) {
            Ok(i) => self.probabilities[i],
            Err(i) => {
                if i == 0 {
                    0.0
                } else if i >= self.values.len() {
                    1.0
                } else {
                    // Linear interpolation
                    let (x0, x1) = (self.values[i - 1], self.values[i]);
                    let (p0, p1) = (self.probabilities[i - 1], self.probabilities[i]);
                    p0 + (p1 - p0) * (x - x0) / (x1 - x0)
                }
            }
        }
    }

    /// Evaluate inverse CDF (quantile function) at a probability.
    pub fn quantile(&self, p: f64) -> f64 {
        if p <= 0.0 {
            return *self.values.first().unwrap_or(&0.0);
        }
        if p >= 1.0 {
            return *self.values.last().unwrap_or(&0.0);
        }

        match self.probabilities.binary_search_by(|v| v.total_cmp(&p)) {
            Ok(i) => self.values[i],
            Err(i) => {
                if i == 0 {
                    self.values[0]
                } else if i >= self.probabilities.len() {
                    *self.values.last().unwrap_or(&0.0)
                } else {
                    // Linear interpolation
                    let (p0, p1) = (self.probabilities[i - 1], self.probabilities[i]);
                    let (x0, x1) = (self.values[i - 1], self.values[i]);
                    x0 + (x1 - x0) * (p - p0) / (p1 - p0)
                }
            }
        }
    }
}
