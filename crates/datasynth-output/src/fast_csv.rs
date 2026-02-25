//! Fast CSV writing utilities using itoa/ryu for zero-allocation number formatting.
//!
//! The standard `format!()` macro allocates a new String per row. This module provides
//! write-through helpers that format numbers directly into the output buffer using
//! `itoa` (integers) and `ryu` (floats), avoiding intermediate allocations entirely.

use std::io::Write;

/// Write a CSV-escaped string field directly to a writer.
///
/// Only quotes the string if it contains special characters (comma, quote, newline).
/// This avoids the allocation that `format!("\"{}\"", s.replace('"', "\"\""))` incurs.
#[inline]
pub fn write_csv_field<W: Write>(w: &mut W, s: &str) -> std::io::Result<()> {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        w.write_all(b"\"")?;
        for byte in s.as_bytes() {
            if *byte == b'"' {
                w.write_all(b"\"\"")?;
            } else {
                w.write_all(std::slice::from_ref(byte))?;
            }
        }
        w.write_all(b"\"")?;
    } else {
        w.write_all(s.as_bytes())?;
    }
    Ok(())
}

/// Write an Option<String> field, writing empty string for None.
#[inline]
pub fn write_csv_opt_field<W: Write>(w: &mut W, opt: &Option<String>) -> std::io::Result<()> {
    match opt {
        Some(s) => write_csv_field(w, s),
        None => Ok(()),
    }
}

/// Write an integer field using itoa (no allocation).
#[inline]
pub fn write_csv_int<W: Write, I: itoa::Integer>(w: &mut W, val: I) -> std::io::Result<()> {
    let mut buf = itoa::Buffer::new();
    w.write_all(buf.format(val).as_bytes())
}

/// Write a float field using ryu (no allocation).
#[inline]
pub fn write_csv_float<W: Write, F: ryu::Float>(w: &mut W, val: F) -> std::io::Result<()> {
    let mut buf = ryu::Buffer::new();
    w.write_all(buf.format(val).as_bytes())
}

/// Write a rust_decimal::Decimal field directly (avoids to_string() allocation).
///
/// Uses a stack-allocated buffer to format the decimal, then writes it directly.
#[inline]
pub fn write_csv_decimal<W: Write>(w: &mut W, val: &rust_decimal::Decimal) -> std::io::Result<()> {
    // rust_decimal's Display impl writes to a formatter; we use a small stack buffer
    use std::fmt::Write as FmtWrite;
    let mut buf = DecimalBuffer::new();
    // This cannot fail since DecimalBuffer always has capacity
    let _ = write!(buf, "{}", val);
    w.write_all(buf.as_bytes())
}

/// Write a CSV comma separator.
#[inline]
pub fn write_sep<W: Write>(w: &mut W) -> std::io::Result<()> {
    w.write_all(b",")
}

/// Write a newline.
#[inline]
pub fn write_newline<W: Write>(w: &mut W) -> std::io::Result<()> {
    w.write_all(b"\n")
}

/// Write a boolean as "true" or "false".
#[inline]
pub fn write_csv_bool<W: Write>(w: &mut W, val: bool) -> std::io::Result<()> {
    w.write_all(if val { b"true" } else { b"false" })
}

/// Small stack-allocated buffer for formatting Decimals without heap allocation.
///
/// rust_decimal values are at most ~30 characters, so 48 bytes is plenty.
struct DecimalBuffer {
    buf: [u8; 48],
    len: usize,
}

impl DecimalBuffer {
    #[inline]
    fn new() -> Self {
        Self {
            buf: [0u8; 48],
            len: 0,
        }
    }

    #[inline]
    fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

impl std::fmt::Write for DecimalBuffer {
    #[inline]
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        let bytes = s.as_bytes();
        let remaining = self.buf.len() - self.len;
        if bytes.len() > remaining {
            return Err(std::fmt::Error);
        }
        self.buf[self.len..self.len + bytes.len()].copy_from_slice(bytes);
        self.len += bytes.len();
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_write_csv_field_simple() {
        let mut buf = Vec::new();
        write_csv_field(&mut buf, "hello").unwrap();
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "hello");
    }

    #[test]
    fn test_write_csv_field_with_comma() {
        let mut buf = Vec::new();
        write_csv_field(&mut buf, "hello,world").unwrap();
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "\"hello,world\"");
    }

    #[test]
    fn test_write_csv_field_with_quote() {
        let mut buf = Vec::new();
        write_csv_field(&mut buf, "say \"hi\"").unwrap();
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "\"say \"\"hi\"\"\"");
    }

    #[test]
    fn test_write_csv_int() {
        let mut buf = Vec::new();
        write_csv_int(&mut buf, 42i32).unwrap();
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "42");
    }

    #[test]
    fn test_write_csv_int_negative() {
        let mut buf = Vec::new();
        write_csv_int(&mut buf, -123i64).unwrap();
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "-123");
    }

    #[test]
    fn test_write_csv_decimal() {
        let mut buf = Vec::new();
        write_csv_decimal(&mut buf, &dec!(1234.56)).unwrap();
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "1234.56");
    }

    #[test]
    fn test_write_csv_decimal_zero() {
        let mut buf = Vec::new();
        write_csv_decimal(&mut buf, &dec!(0.00)).unwrap();
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "0.00");
    }

    #[test]
    fn test_write_csv_opt_field_some() {
        let mut buf = Vec::new();
        let val = Some("test".to_string());
        write_csv_opt_field(&mut buf, &val).unwrap();
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "test");
    }

    #[test]
    fn test_write_csv_opt_field_none() {
        let mut buf = Vec::new();
        write_csv_opt_field(&mut buf, &None).unwrap();
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "");
    }

    #[test]
    fn test_write_csv_bool() {
        let mut buf = Vec::new();
        write_csv_bool(&mut buf, true).unwrap();
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "true");

        let mut buf = Vec::new();
        write_csv_bool(&mut buf, false).unwrap();
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "false");
    }

    #[test]
    fn test_combined_row() {
        let mut buf = Vec::new();
        write_csv_field(&mut buf, "DOC001").unwrap();
        write_sep(&mut buf).unwrap();
        write_csv_int(&mut buf, 2024i32).unwrap();
        write_sep(&mut buf).unwrap();
        write_csv_decimal(&mut buf, &dec!(1500.00)).unwrap();
        write_sep(&mut buf).unwrap();
        write_csv_bool(&mut buf, false).unwrap();
        write_newline(&mut buf).unwrap();

        assert_eq!(
            std::str::from_utf8(&buf).unwrap(),
            "DOC001,2024,1500.00,false\n"
        );
    }
}
