/// What of a static scene a selection marks: the place it stands in, or any, and the runs of clusters drawing it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StaticSelection {
  /// The place every marked cluster is drawn in, or none for any.
  pub place: Option<u32>,
  /// Runs of clusters, each its first and the one past its last.
  pub runs: Vec<[u32; 2]>,
}

impl StaticSelection {
  /// Every cluster: what a selection standing in one place alone marks.
  pub const ANY_CLUSTER: [u32; 2] = [0, u32::MAX];
}
