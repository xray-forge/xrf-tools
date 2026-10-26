use xrf_error::{XrfError, XrfResult};

/// Bits the reader may invent past the end of the stream before it calls the stream truncated.
const PHANTOM_BIT_BUDGET: u32 = 32;

/// Widest single read the codec performs, which bounds how much the buffer can hold at once.
const READ_BITS_MAX: u32 = 16;

/// Reads big-endian bit runs out of an in-memory LZHUF stream.
pub(crate) struct BitReader<'a> {
  source: &'a [u8],
  /// Index of the next byte to pull into `buffer`.
  position: usize,
  /// Bits pending consumption, right-aligned, with `count` of them valid.
  buffer: u32,
  count: u32,
  /// Bits invented past the end of `source`, budgeted by [`PHANTOM_BIT_BUDGET`].
  phantom_bits: u32,
}

impl<'a> BitReader<'a> {
  pub(crate) fn new(source: &'a [u8]) -> Self {
    Self {
      source,
      position: 0,
      buffer: 0,
      count: 0,
      phantom_bits: 0,
    }
  }

  /// Read the next `count` bits, most significant first.
  ///
  /// Returns a parsing error once the stream runs out by more than [`PHANTOM_BIT_BUDGET`].
  pub(crate) fn read_bits(&mut self, count: u32) -> XrfResult<u16> {
    debug_assert!(
      count <= READ_BITS_MAX,
      "codec never reads more than {READ_BITS_MAX} bits"
    );

    if count == 0 {
      return Ok(0);
    }

    if count > READ_BITS_MAX {
      return Err(XrfError::new_parsing_error(format!(
        "LZHUF stream read of {count} bits exceeds the {READ_BITS_MAX} bit maximum"
      )));
    }

    while self.count < count {
      self.buffer = (self.buffer << 8) | u32::from(self.next_byte()?);
      self.count += 8;
    }

    let remaining: u32 = self.count - count;
    let value: u32 = (self.buffer >> remaining) & ((1u32 << count) - 1);

    self.count = remaining;
    self.buffer &= (1u32 << remaining) - 1;

    Ok(value as u16)
  }

  fn next_byte(&mut self) -> XrfResult<u8> {
    match self.source.get(self.position) {
      Some(&byte) => {
        self.position += 1;

        Ok(byte)
      }
      None => {
        self.phantom_bits += 8;

        if self.phantom_bits > PHANTOM_BIT_BUDGET {
          return Err(XrfError::new_parsing_error(format!(
            "LZHUF stream of {} byte(s) ended while more data was still expected",
            self.source.len()
          )));
        }

        Ok(0)
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::{BitReader, PHANTOM_BIT_BUDGET};

  #[test]
  fn reads_bits_most_significant_first() {
    let mut reader: BitReader = BitReader::new(&[0b1011_0010, 0b0100_0001]);

    assert_eq!(reader.read_bits(1).expect("bit"), 0b1);
    assert_eq!(reader.read_bits(3).expect("bits"), 0b011);
    assert_eq!(reader.read_bits(4).expect("bits"), 0b0010);
    assert_eq!(reader.read_bits(8).expect("bits"), 0b0100_0001);
  }

  #[test]
  fn reads_runs_that_span_byte_boundaries() {
    let mut reader: BitReader = BitReader::new(&[0b1111_0000, 0b1010_1010]);

    assert_eq!(reader.read_bits(6).expect("bits"), 0b111100);
    // Two bits left in the first byte, then the whole second byte, taken most significant first.
    assert_eq!(reader.read_bits(9).expect("bits"), 0b0_0101_0101);
  }

  #[test]
  fn zero_bit_read_yields_zero_without_consuming() {
    let mut reader: BitReader = BitReader::new(&[0b1000_0000]);

    assert_eq!(reader.read_bits(0).expect("bits"), 0);
    assert_eq!(reader.read_bits(1).expect("bit"), 1);
  }

  #[test]
  fn pads_a_short_tail_the_way_the_engine_does() {
    let mut reader: BitReader = BitReader::new(&[0b1100_0000]);

    assert_eq!(reader.read_bits(2).expect("bits"), 0b11);
    assert_eq!(reader.read_bits(16).expect("padded bits"), 0);
  }

  #[test]
  fn rejects_a_stream_that_runs_out_for_good() {
    let mut reader: BitReader = BitReader::new(&[]);

    for _ in 0..(PHANTOM_BIT_BUDGET / 8) {
      assert_eq!(reader.read_bits(8).expect("padded bits"), 0);
    }

    assert!(reader.read_bits(8).is_err(), "budget exhausted");
  }
}
