/// The longest line the engine's XML reader takes, which a string table has to keep every line within.
///
/// The reader goes line by line into a fixed buffer before parsing, to expand `#include`s, and a longer line is fatal.
/// xray-16's `XMLDocument` (`ParseFile`, `IReader::try_r_string`) takes up to 4095 bytes; Anomaly's `xrXMLParser`
/// (`IReader::r_string`) asserts below 4095, so 4094. The limit here is the stricter, so a table loads in either.
pub(crate) struct TranslationLineLimit;

impl TranslationLineLimit {
  /// Bytes a line may hold, its line break aside.
  pub(crate) const MAX_BYTES: usize = 4094;

  /// Every line of encoded `bytes` longer than [`Self::MAX_BYTES`], as its one-based number and its bytes.
  ///
  /// Lines end where the engine's reader ends them, at a line feed or a carriage return; they are numbered by line
  /// feeds, as an editor shows them.
  pub(crate) fn find_overlong(bytes: &[u8]) -> Vec<(usize, &[u8])> {
    bytes
      .split(|byte| *byte == b'\n')
      .enumerate()
      .filter_map(|(index, line)| {
        line
          .split(|byte| *byte == b'\r')
          .max_by_key(|segment| segment.len())
          .filter(|segment| segment.len() > Self::MAX_BYTES)
          .map(|segment| (index + 1, segment))
      })
      .collect()
  }

  /// How a refusal says what is wrong with one line.
  pub(crate) fn describe(number: usize, length: usize) -> String {
    format!(
      "line {number} is {length} bytes, past the {} the game's XML reader takes before refusing the file",
      Self::MAX_BYTES
    )
  }
}
