/// Bytes in one binary kilobyte.
pub const BYTES_PER_KILOBYTE: u64 = 1024;

/// Bytes in one binary megabyte.
pub const BYTES_PER_MEGABYTE: u64 = 1024 * BYTES_PER_KILOBYTE;

/// Bytes in one binary gigabyte.
pub const BYTES_PER_GIGABYTE: u64 = 1024 * BYTES_PER_MEGABYTE;

/// Unit labels by binary magnitude, matching `BYTE_UNITS` in `@xrf/format` (`packages/xrf-format/src/bytes.ts`).
const BYTE_UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

/// Convert a byte count to megabytes, unrounded.
///
/// Rounding is left to the caller because the callers differ: a summary shows a fraction, a form field
/// wants a whole number. Mirrors `bytesToMegabytes` in `@xrf/format` (`packages/xrf-format/src/byte-size.ts`).
#[inline]
pub fn bytes_to_megabytes(bytes: u64) -> f64 {
  bytes as f64 / BYTES_PER_MEGABYTE as f64
}

/// Convert a megabyte count to bytes, for the sizes that are typed in megabytes but stored in bytes.
///
/// Saturates rather than wrapping: a size limit past `u64::MAX` bytes is already every byte there is.
#[inline]
pub fn megabytes_to_bytes(megabytes: u64) -> u64 {
  megabytes.saturating_mul(BYTES_PER_MEGABYTE)
}

/// Format a byte count with the largest binary unit it fills: `512 B`, `1.5 KB`, `1.17 GB`.
///
/// Mirrors `@xrf/format`'s `formatBytes` (`packages/xrf-format/src/bytes.ts`) rule for rule — magnitude-scaled precision
/// with trailing zeros dropped — so the CLI and the desktop app render the same size identically. For two
/// sizes a reader is meant to compare, use [`format_bytes_pair`] so both carry the same unit.
pub fn format_bytes(bytes: u64) -> String {
  format_bytes_as(bytes, byte_unit_index(bytes))
}

/// Format two byte counts that are meant to be compared, both in the larger value's unit.
///
/// A pack summary reads `1.17 GB source, 0.78 GB written`: with each size picking its own unit the
/// ratio between them stops being visible at a glance.
pub fn format_bytes_pair(first: u64, second: u64) -> (String, String) {
  let unit_index: usize = byte_unit_index(first.max(second));

  (format_bytes_as(first, unit_index), format_bytes_as(second, unit_index))
}

/// Index into [`BYTE_UNITS`] of the largest binary unit a byte count fills.
fn byte_unit_index(bytes: u64) -> usize {
  let mut index: usize = 0;
  let mut remaining: u64 = bytes;

  while remaining >= BYTES_PER_KILOBYTE && index < BYTE_UNITS.len() - 1 {
    remaining /= BYTES_PER_KILOBYTE;
    index += 1;
  }

  index
}

fn format_bytes_as(bytes: u64, unit_index: usize) -> String {
  if unit_index == 0 {
    return format!("{bytes} B");
  }

  let value: f64 = bytes as f64 / (BYTES_PER_KILOBYTE as f64).powi(unit_index as i32);
  // The precision the UI uses: whole numbers past 100, one decimal past 10, two below that.
  let precision: usize = if value >= 100.0 {
    0
  } else if value >= 10.0 {
    1
  } else {
    2
  };
  let mut formatted: String = format!("{value:.precision$}");

  if formatted.contains('.') {
    formatted.truncate(formatted.trim_end_matches('0').trim_end_matches('.').len());
  }

  format!("{formatted} {}", BYTE_UNITS[unit_index])
}

#[cfg(test)]
mod tests {
  use crate::{
    BYTES_PER_GIGABYTE, BYTES_PER_KILOBYTE, BYTES_PER_MEGABYTE, bytes_to_megabytes, format_bytes, format_bytes_pair,
    megabytes_to_bytes,
  };

  #[test]
  fn test_constants_are_binary() {
    assert_eq!(BYTES_PER_KILOBYTE, 1024);
    assert_eq!(BYTES_PER_MEGABYTE, 1024 * 1024);
    assert_eq!(BYTES_PER_GIGABYTE, 1024 * 1024 * 1024);
  }

  #[test]
  fn test_bytes_to_megabytes() {
    assert_eq!(bytes_to_megabytes(0), 0.0);
    assert_eq!(bytes_to_megabytes(BYTES_PER_MEGABYTE), 1.0);
    assert_eq!(bytes_to_megabytes(BYTES_PER_MEGABYTE + BYTES_PER_MEGABYTE / 2), 1.5);
  }

  #[test]
  fn test_megabytes_to_bytes() {
    assert_eq!(megabytes_to_bytes(0), 0);
    assert_eq!(megabytes_to_bytes(1), BYTES_PER_MEGABYTE);
    assert_eq!(megabytes_to_bytes(1900), 1900 * BYTES_PER_MEGABYTE);
    assert_eq!(megabytes_to_bytes(u64::MAX), u64::MAX);
  }

  #[test]
  fn test_format_bytes_matches_the_ui_formatter() {
    // The expectations `@xrf/format`'s own `formatBytes` test pins.
    assert_eq!(format_bytes(0), "0 B");
    assert_eq!(format_bytes(1536), "1.5 KB");
    assert_eq!(format_bytes(10 * BYTES_PER_MEGABYTE), "10 MB");
    assert_eq!(format_bytes(1536 * BYTES_PER_MEGABYTE), "1.5 GB");
  }

  #[test]
  fn test_format_bytes_scales_precision_and_drops_trailing_zeros() {
    assert_eq!(format_bytes(512), "512 B");
    assert_eq!(format_bytes(BYTES_PER_KILOBYTE), "1 KB");
    assert_eq!(
      format_bytes(800 * BYTES_PER_MEGABYTE + 100 * BYTES_PER_KILOBYTE),
      "800 MB"
    );
    assert_eq!(format_bytes(BYTES_PER_GIGABYTE + 174 * BYTES_PER_MEGABYTE), "1.17 GB");
    assert_eq!(format_bytes(1536 * BYTES_PER_GIGABYTE), "1.5 TB");
  }

  #[test]
  fn test_format_bytes_pair_shares_the_larger_unit() {
    let (source, written) = format_bytes_pair(BYTES_PER_GIGABYTE + 174 * BYTES_PER_MEGABYTE, 800 * BYTES_PER_MEGABYTE);

    assert_eq!(source, "1.17 GB");
    assert_eq!(written, "0.78 GB");

    let (first, second) = format_bytes_pair(512, 512);

    assert_eq!(first, "512 B");
    assert_eq!(second, "512 B");
  }
}
