/// What one path cost across a session's reads.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct XrayReadTraceEntry {
  /// Times the bytes were read from a source.
  pub reads: u64,
  /// Bytes read in total, which is the redundant figure when `reads` is above one.
  pub bytes: u64,
  /// Bytes the most recent read returned, so unique volume is a sum rather than a division.
  ///
  /// Tracked separately because a write between two reads can change a path's length, and dividing `bytes` by `reads`
  /// would then be quietly wrong rather than obviously so.
  pub size: u64,
}
