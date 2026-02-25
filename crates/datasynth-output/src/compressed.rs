//! Compressed output writers using zstd for CSV/JSON files.
//!
//! Provides transparent compression wrappers that can wrap any `Write` sink.
//! Uses zstd multithreaded encoding for parallel compression on multi-core systems.

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

/// Compression configuration for output files.
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// Zstd compression level (1-22, default 3).
    pub level: i32,
    /// Number of worker threads for parallel compression (0 = auto-detect).
    pub threads: u32,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            level: 3,
            threads: 0,
        }
    }
}

impl CompressionConfig {
    /// Create a config with the given compression level.
    pub fn with_level(mut self, level: i32) -> Self {
        self.level = level.clamp(1, 22);
        self
    }

    /// Create a config with the given number of threads.
    pub fn with_threads(mut self, threads: u32) -> Self {
        self.threads = threads;
        self
    }
}

/// A writer that transparently compresses output using zstd.
///
/// Wraps a `BufWriter<File>` with zstd compression. The compressed data
/// is written to a file with a `.zst` extension appended to the original path.
pub struct CompressedWriter<'a> {
    encoder: zstd::Encoder<'a, BufWriter<File>>,
    bytes_written: u64,
}

impl<'a> CompressedWriter<'a> {
    /// Create a new compressed writer for the given path.
    pub fn new(path: &Path, config: &CompressionConfig) -> io::Result<Self> {
        let file = File::create(path)?;
        let buf_writer = BufWriter::with_capacity(256 * 1024, file);
        let mut encoder = zstd::Encoder::new(buf_writer, config.level)?;

        // Enable multithreaded compression if requested
        if config.threads > 0 {
            encoder
                .set_parameter(zstd::zstd_safe::CParameter::NbWorkers(config.threads))
                .map_err(|_| io::Error::other("Failed to set zstd worker threads"))?;
        }

        Ok(Self {
            encoder,
            bytes_written: 0,
        })
    }

    /// Get total uncompressed bytes written.
    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }

    /// Finish compression and flush all remaining data.
    pub fn finish(self) -> io::Result<()> {
        self.encoder.finish()?;
        Ok(())
    }
}

impl Write for CompressedWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.encoder.write(buf)?;
        self.bytes_written += n as u64;
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.encoder.flush()
    }
}

/// Determine the compressed output path (adds .zst extension).
pub fn compressed_path(path: &Path) -> PathBuf {
    let mut p = path.as_os_str().to_owned();
    p.push(".zst");
    PathBuf::from(p)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::tempdir;

    #[test]
    fn test_compressed_writer_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.csv.zst");

        let config = CompressionConfig::default();
        let mut writer = CompressedWriter::new(&path, &config).unwrap();

        let data = "id,name,value\n1,hello,42.5\n2,world,99.9\n";
        writer.write_all(data.as_bytes()).unwrap();
        writer.finish().unwrap();

        // Decompress and verify
        let compressed = std::fs::read(&path).unwrap();
        let mut decoder = zstd::Decoder::new(&compressed[..]).unwrap();
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed).unwrap();

        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_compressed_writer_large_data() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("large.csv.zst");

        let config = CompressionConfig::default().with_level(3);
        let mut writer = CompressedWriter::new(&path, &config).unwrap();

        // Write 10K rows
        writer.write_all(b"id,name,value\n").unwrap();
        for i in 0..10_000u32 {
            let row = format!("{},item_{},{}.{:02}\n", i, i, i * 100, i % 100);
            writer.write_all(row.as_bytes()).unwrap();
        }
        let bytes_written = writer.bytes_written();
        writer.finish().unwrap();

        // Verify compressed file is smaller
        let file_size = std::fs::metadata(&path).unwrap().len();
        assert!(
            file_size < bytes_written,
            "Compressed size {} should be less than uncompressed {}",
            file_size,
            bytes_written
        );

        // Verify decompression roundtrip
        let compressed = std::fs::read(&path).unwrap();
        let mut decoder = zstd::Decoder::new(&compressed[..]).unwrap();
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed).unwrap();
        assert!(decompressed.starts_with("id,name,value\n"));
        let line_count = decompressed.lines().count();
        assert_eq!(line_count, 10_001); // header + 10K rows
    }

    #[test]
    fn test_compressed_path() {
        let path = Path::new("/tmp/output/data.csv");
        let cp = compressed_path(path);
        assert_eq!(cp, PathBuf::from("/tmp/output/data.csv.zst"));
    }

    #[test]
    fn test_compression_config() {
        let config = CompressionConfig::default().with_level(6).with_threads(4);
        assert_eq!(config.level, 6);
        assert_eq!(config.threads, 4);
    }

    #[test]
    fn test_compression_level_clamp() {
        let config = CompressionConfig::default().with_level(50);
        assert_eq!(config.level, 22);

        let config = CompressionConfig::default().with_level(-5);
        assert_eq!(config.level, 1);
    }
}
