/// How a change moved its path.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum XrayChangeKind {
  /// The mount held the path and holds other bytes for it now.
  Modified,
  /// The mount holds a path it did not hold.
  Created,
  /// The mount no longer holds a path it held.
  Deleted,
}
