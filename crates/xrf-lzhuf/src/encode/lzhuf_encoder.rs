use xrf_error::{XrfError, XrfResult};

use crate::bit_writer::BitWriter;
use crate::encode::match_finder::MatchFinder;
use crate::encode::source_match::SourceMatch;
use crate::lzhuf_constants::{MATCH_CODE_BASE, MATCH_LENGTH_THRESHOLD};
use crate::match_distance::{LEADING_BIT_COUNT, join_match_distance};
use crate::tree::dynamic_huffman_tree::DynamicHuffmanTree;

/// Rough starting size for the output, since coded data is usually smaller than its source.
const OUTPUT_CAPACITY_DIVISOR: usize = 2;

/// Encodes one headerless X-Ray LZHUF bit stream.
///
/// Greedy, like `Encode` in `LzHuf.cpp`: the longest match at each position wins and the encoder advances
/// past it, with no lookahead for whether a shorter match here would pay off later.
pub(crate) struct LzhufEncoder<'a> {
  source: &'a [u8],
  writer: BitWriter,
  /// Boxed to keep several kilobytes of coder state off the stack of every caller.
  tree: Box<DynamicHuffmanTree>,
  finder: MatchFinder<'a>,
}

impl<'a> LzhufEncoder<'a> {
  pub(crate) fn new(source: &'a [u8]) -> Self {
    Self {
      source,
      writer: BitWriter::with_capacity(source.len() / OUTPUT_CAPACITY_DIVISOR + 64),
      tree: Box::new(DynamicHuffmanTree::new()),
      finder: MatchFinder::new(source),
    }
  }

  /// Code the whole source and return the stream.
  pub(crate) fn encode(mut self) -> XrfResult<Vec<u8>> {
    let mut position: usize = 0;

    while position < self.source.len() {
      match self.finder.find(position) {
        Some(found) => {
          self.write_match(&found)?;

          // Positions covered by the match still have to be recorded, or their prefixes stop being
          // matchable and every later reference into this stretch is lost.
          for offset in 0..found.length {
            self.finder.insert(position + offset);
          }

          position += found.length;
        }
        None => {
          self
            .tree
            .write_code(u16::from(self.source[position]), &mut self.writer)?;
          self.finder.insert(position);

          position += 1;
        }
      }
    }

    Ok(self.writer.finish())
  }

  /// Write one match: a length carried by the command code, then the distance.
  fn write_match(&mut self, found: &SourceMatch) -> XrfResult<()> {
    let Some(coded_length) = found
      .length
      .checked_sub(MATCH_LENGTH_THRESHOLD + 1)
      .and_then(|length| u16::try_from(length).ok())
      .map(|length| MATCH_CODE_BASE + length)
    else {
      return Err(XrfError::new_encoding_error(format!(
        "LZHUF match of {} byte(s) has no command code",
        found.length
      )));
    };

    let Ok(distance) = u16::try_from(found.distance) else {
      return Err(XrfError::new_encoding_error(format!(
        "LZHUF match distance {} does not fit the window",
        found.distance
      )));
    };

    self.tree.write_code(coded_length, &mut self.writer)?;

    let (leading, remaining_bits, remaining) = join_match_distance(distance);

    self.writer.write_bits(u32::from(leading), LEADING_BIT_COUNT)?;
    self.writer.write_bits(u32::from(remaining), remaining_bits)
  }
}
