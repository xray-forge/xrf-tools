/// How the round trip of one omf file came out.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OmfRepackOutcome {
  Identical,
  /// Read and parsed, but serialized into different bytes.
  Mismatched,
  /// Read, but not an omf file the reader accepts: judged content, like a mismatch.
  Malformed,
  /// Not read at all, which is the run's own failure rather than a verdict on the file.
  Failed,
}
