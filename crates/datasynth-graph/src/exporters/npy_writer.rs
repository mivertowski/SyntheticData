//! Shared NPY (NumPy binary format) writer utilities.
//!
//! Provides functions for writing arrays in NPY format, shared between the
//! PyTorch Geometric and DGL exporters. The NPY format uses little-endian
//! encoding with a padded header aligned to 64 bytes.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Writes an NPY header to the writer.
///
/// The header includes the magic number, format version 1.0, data type descriptor,
/// Fortran order flag, and shape. The header is padded to a 64-byte boundary.
pub fn write_npy_header<W: Write>(writer: &mut W, dtype: &str, shape: &str) -> std::io::Result<()> {
    // Magic number and version
    writer.write_all(&[0x93])?; // \x93
    writer.write_all(b"NUMPY")?;
    writer.write_all(&[0x01, 0x00])?; // Version 1.0

    // Header dict
    let header = format!("{{'descr': '{dtype}', 'fortran_order': False, 'shape': {shape} }}");

    // Pad header to multiple of 64 bytes (including magic, version, header_len)
    let header_len = header.len();
    let total_len = 10 + header_len + 1; // magic(6) + version(2) + header_len(2) + header + newline
    let padding = (64 - (total_len % 64)) % 64;
    let padded_len = header_len + 1 + padding;

    writer.write_all(&(padded_len as u16).to_le_bytes())?;
    writer.write_all(header.as_bytes())?;
    for _ in 0..padding {
        writer.write_all(b" ")?;
    }
    writer.write_all(b"\n")?;

    Ok(())
}

/// Writes a 1D array of i64 values in NPY format.
pub fn write_npy_1d_i64(path: &Path, data: &[i64]) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::with_capacity(256 * 1024, file);

    let shape = format!("({},)", data.len());
    write_npy_header(&mut writer, "<i8", &shape)?;

    for &val in data {
        writer.write_all(&val.to_le_bytes())?;
    }

    Ok(())
}

/// Writes a 1D array of bool values in NPY format.
pub fn write_npy_1d_bool(path: &Path, data: &[bool]) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::with_capacity(256 * 1024, file);

    let shape = format!("({},)", data.len());
    write_npy_header(&mut writer, "|b1", &shape)?;

    for &val in data {
        writer.write_all(&[if val { 1u8 } else { 0u8 }])?;
    }

    Ok(())
}

/// Writes a 2D array of i64 values in NPY format (row-major order).
///
/// Short rows are padded with zeros to match the column count of the first row.
pub fn write_npy_2d_i64(path: &Path, data: &[Vec<i64>]) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::with_capacity(256 * 1024, file);

    let rows = data.len();
    let cols = data.first().map(|r| r.len()).unwrap_or(0);

    let shape = format!("({rows}, {cols})");
    write_npy_header(&mut writer, "<i8", &shape)?;

    for row in data {
        for &val in row {
            writer.write_all(&val.to_le_bytes())?;
        }
        // Pad short rows if needed
        for _ in row.len()..cols {
            writer.write_all(&0_i64.to_le_bytes())?;
        }
    }

    Ok(())
}

/// Writes a 2D array of f64 values in NPY format (row-major order).
///
/// Short rows are padded with zeros to match the column count of the first row.
pub fn write_npy_2d_f64(path: &Path, data: &[Vec<f64>]) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::with_capacity(256 * 1024, file);

    let rows = data.len();
    let cols = data.first().map(|r| r.len()).unwrap_or(0);

    let shape = format!("({rows}, {cols})");
    write_npy_header(&mut writer, "<f8", &shape)?;

    for row in data {
        for &val in row {
            writer.write_all(&val.to_le_bytes())?;
        }
        // Pad short rows with zeros
        for _ in row.len()..cols {
            writer.write_all(&0.0_f64.to_le_bytes())?;
        }
    }

    Ok(())
}

/// Generates train/val/test boolean masks and writes them as NPY files.
///
/// Uses a simple xorshift64 PRNG seeded with `seed` to shuffle node indices,
/// then assigns them to train, validation, and test splits based on the given ratios.
pub fn export_masks(
    output_dir: &Path,
    n: usize,
    seed: u64,
    train_ratio: f64,
    val_ratio: f64,
) -> std::io::Result<()> {
    let mut rng = SimpleRng::new(seed);

    let train_size = (n as f64 * train_ratio) as usize;
    let val_size = (n as f64 * val_ratio) as usize;

    // Create shuffled indices
    let mut indices: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        let j = (rng.next_u64() % (i as u64 + 1)) as usize;
        indices.swap(i, j);
    }

    // Create masks
    let mut train_mask = vec![false; n];
    let mut val_mask = vec![false; n];
    let mut test_mask = vec![false; n];

    for (i, &idx) in indices.iter().enumerate() {
        if i < train_size {
            train_mask[idx] = true;
        } else if i < train_size + val_size {
            val_mask[idx] = true;
        } else {
            test_mask[idx] = true;
        }
    }

    write_npy_1d_bool(&output_dir.join("train_mask.npy"), &train_mask)?;
    write_npy_1d_bool(&output_dir.join("val_mask.npy"), &val_mask)?;
    write_npy_1d_bool(&output_dir.join("test_mask.npy"), &test_mask)?;

    Ok(())
}

/// Simple random number generator (xorshift64) for deterministic mask generation.
pub struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    /// Create a new RNG with the given seed (seed of 0 is mapped to 1).
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    /// Generate the next random u64 value.
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}
