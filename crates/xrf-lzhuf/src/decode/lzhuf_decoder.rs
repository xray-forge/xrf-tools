use xrf_error::{XrfError, XrfResult};

use crate::bits::BitReader;
use crate::lzhuf_constants::{
  DECOMPRESSED_SIZE_HEADER_LENGTH, MATCH_CODE_BASE, MATCH_LENGTH_MAX, MATCH_LENGTH_THRESHOLD,
};
use crate::match_distance::MatchDistance;
use crate::ring_buffer::RingBuffer;
use crate::tree::DynamicHuffmanTree;

/// Decodes X-Ray LZHUF, the exact byte layout `_decompressLZ` accepts.
pub struct LzhufDecoder<'a> {
  reader: BitReader<'a>,
  /// Boxed to keep several kilobytes of coder state off the stack of every caller.
  tree: Box<DynamicHuffmanTree>,
  history: Box<RingBuffer>,
}

impl<'a> LzhufDecoder<'a> {
  /// Most bytes one input byte may legitimately yield.
  const MAX_EXPANSION_RATIO: usize = (MATCH_LENGTH_MAX * 8).div_ceil(10);

  /// Decompress an X-Ray LZHUF blob: a little-endian `u32` decompressed size followed by the bit stream.
  ///
  /// # Errors
  ///
  /// Fails when the blob is too short to hold a header, declares a zero or implausible size, or ends before the
  /// declared size has been produced.
  pub fn decompress(source: &[u8]) -> XrfResult<Vec<u8>> {
    let Some((header, stream)) = source.split_at_checked(DECOMPRESSED_SIZE_HEADER_LENGTH) else {
      return Err(XrfError::new_parsing_error(format!(
        "LZHUF blob of {} byte(s) is too short to declare a decompressed size",
        source.len()
      )));
    };

    let declared_size: usize = match header.try_into() {
      Ok(bytes) => u32::from_le_bytes(bytes) as usize,
      Err(error) => {
        return Err(XrfError::new_parsing_error(format!(
          "LZHUF blob has no readable decompressed size: {error}"
        )));
      }
    };

    // `Decode` refuses a zero size rather than yielding nothing, because X-Ray never writes such a blob.
    if declared_size == 0 {
      return Err(XrfError::new_parsing_error(
        "LZHUF blob declares an empty decompressed size".to_string(),
      ));
    }

    let plausible_size: usize = stream.len().saturating_mul(Self::MAX_EXPANSION_RATIO);

    if declared_size > plausible_size {
      return Err(XrfError::new_parsing_error(format!(
        "LZHUF blob declares {declared_size} byte(s) of output, more than {} byte(s) of stream can yield",
        stream.len()
      )));
    }

    let mut target: Vec<u8> = vec![0; declared_size];

    Self::decompress_into(stream, &mut target)?;

    Ok(target)
  }

  /// Decompress a headerless X-Ray LZHUF stream into a buffer of known size, filling `target` completely.
  ///
  /// # Errors
  ///
  /// Fails when the stream ends before `target` is full, or when it decodes to a malformed coder state.
  pub fn decompress_into(stream: &[u8], target: &mut [u8]) -> XrfResult<()> {
    LzhufDecoder::new(stream).decode_into(target)
  }

  fn new(stream: &'a [u8]) -> Self {
    Self {
      reader: BitReader::new(stream),
      tree: Box::new(DynamicHuffmanTree::new()),
      history: Box::default(),
    }
  }

  /// Decode exactly `target.len()` bytes.
  fn decode_into(&mut self, target: &mut [u8]) -> XrfResult<()> {
    let length: usize = target.len();
    let mut position: usize = 0;

    while position < length {
      let code: u16 = self.tree.read_code(&mut self.reader)?;

      if code < MATCH_CODE_BASE {
        let byte: u8 = code as u8;

        self.history.push(byte);
        target[position] = byte;
        position += 1;

        continue;
      }

      // Codes above the literal range carry a match length; the shortest match encoded is one longer
      // than the threshold, since anything at or below it is cheaper to emit as literals.
      let match_length: usize = usize::from(code - MATCH_CODE_BASE) + MATCH_LENGTH_THRESHOLD + 1;
      let distance: usize = usize::from(self.read_match_distance()?);

      for _ in 0..match_length {
        if position == length {
          break;
        }

        // Reading and pushing one byte at a time is what makes an overlapping match - a run that
        // reaches into bytes this same match produced - decode the way the encoder intended.
        let byte: u8 = self.history.byte_at_distance(distance);

        self.history.push(byte);
        target[position] = byte;
        position += 1;
      }
    }

    Ok(())
  }

  /// Read a match distance: nine leading bits select a bucket, which names the remaining bits to read.
  fn read_match_distance(&mut self) -> XrfResult<u16> {
    let leading: u16 = self.reader.read_bits(MatchDistance::LEADING_BIT_COUNT)?;
    let (distance, remaining_bits) = MatchDistance::split(leading);

    Ok(distance | self.reader.read_bits(remaining_bits)?)
  }
}

#[cfg(test)]
mod tests {
  use crate::decode::LzhufDecoder;

  #[test]
  fn rejects_a_blob_without_a_size_header() {
    assert!(LzhufDecoder::decompress(&[]).is_err());
    assert!(LzhufDecoder::decompress(&[0x01, 0x02, 0x03]).is_err());
  }

  #[test]
  fn rejects_a_blob_declaring_nothing() {
    assert!(LzhufDecoder::decompress(&[0, 0, 0, 0, 0xff]).is_err());
  }

  #[test]
  fn rejects_a_size_no_stream_that_short_could_produce() {
    // Four bytes of stream cannot yield 16 MB, however the bits decode.
    let mut blob: Vec<u8> = 0x0100_0000u32.to_le_bytes().to_vec();

    blob.extend_from_slice(&[0xff; 4]);

    assert!(LzhufDecoder::decompress(&blob).is_err());
  }

  #[test]
  fn bounds_expansion_above_the_best_possible_match() {
    // Sixty bytes for ten bits is the theoretical best; the guard must not sit below it.
    const { assert!(LzhufDecoder::MAX_EXPANSION_RATIO >= 48) };
  }
}
