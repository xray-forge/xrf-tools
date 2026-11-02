use xrf_error::{XrfError, XrfResult};

/// Widest single write the codec performs.
const WRITE_BITS_MAX: u32 = 32;

/// Writes big-endian bit runs into a growing byte buffer.
pub(crate) struct BitWriter {
  target: Vec<u8>,
  /// Bits awaiting a full byte, right-aligned, with `count` of them valid.
  buffer: u64,
  count: u32,
}

impl BitWriter {
  pub(crate) fn with_capacity(capacity: usize) -> Self {
    Self {
      target: Vec::with_capacity(capacity),
      buffer: 0,
      count: 0,
    }
  }

  /// Append the low `count` bits of `value`, most significant first.
  pub(crate) fn write_bits(&mut self, value: u32, count: u32) -> XrfResult<()> {
    if count == 0 {
      return Ok(());
    }

    if count > WRITE_BITS_MAX {
      return Err(XrfError::new_encoding_error(format!(
        "LZHUF stream write of {count} bits exceeds the {WRITE_BITS_MAX} bit maximum"
      )));
    }

    let mask: u64 = if count == u64::BITS {
      u64::MAX
    } else {
      (1u64 << count) - 1
    };

    // At most seven bits are pending, so a full-width write still fits.
    self.buffer = (self.buffer << count) | (u64::from(value) & mask);
    self.count += count;

    while self.count >= 8 {
      self.count -= 8;
      self.target.push((self.buffer >> self.count) as u8);
    }

    self.buffer &= (1u64 << self.count) - 1;

    Ok(())
  }

  /// Flush any partial byte and return the stream.
  ///
  /// The trailing byte is padded with zero bits, which `PutFlush` in `LzHuf.cpp` also does. A decoder
  /// stops on its own output size and never interprets the padding.
  pub(crate) fn finish(mut self) -> Vec<u8> {
    if self.count > 0 {
      self.target.push((self.buffer << (8 - self.count)) as u8);
    }

    self.target
  }
}

#[cfg(test)]
mod tests {
  use super::{BitWriter, WRITE_BITS_MAX};
  use crate::bits::BitReader;

  #[test]
  fn writes_bits_most_significant_first() {
    let mut writer: BitWriter = BitWriter::with_capacity(0);

    writer.write_bits(0b1, 1).expect("bit");
    writer.write_bits(0b011, 3).expect("bits");
    writer.write_bits(0b0010, 4).expect("bits");

    assert_eq!(writer.finish(), vec![0b1011_0010]);
  }

  #[test]
  fn pads_the_final_byte_with_zeroes() {
    let mut writer: BitWriter = BitWriter::with_capacity(0);

    writer.write_bits(0b11, 2).expect("bits");

    assert_eq!(writer.finish(), vec![0b1100_0000]);
  }

  #[test]
  fn round_trips_wide_runs_through_the_reader() {
    let runs: [(u32, u32); 6] = [(0x1234_5678, 32), (0b101, 3), (0, 9), (0xffff, 16), (1, 1), (0x3f, 6)];
    let mut writer: BitWriter = BitWriter::with_capacity(0);

    for (value, count) in runs {
      writer.write_bits(value, count).expect("run");
    }

    let written: Vec<u8> = writer.finish();
    let mut reader: BitReader = BitReader::new(&written);

    for (value, count) in runs {
      // The reader tops out at sixteen bits, so a wide run reads back in halves.
      let read: u32 = if count > 16 {
        let high: u32 = u32::from(reader.read_bits(count - 16).expect("high bits"));

        (high << 16) | u32::from(reader.read_bits(16).expect("low bits"))
      } else {
        u32::from(reader.read_bits(count).expect("bits"))
      };

      assert_eq!(read, value, "run of {count} bits");
    }
  }

  #[test]
  fn rejects_a_run_wider_than_it_can_stage() {
    let mut writer: BitWriter = BitWriter::with_capacity(0);

    assert!(writer.write_bits(0, WRITE_BITS_MAX + 1).is_err());
  }
}
