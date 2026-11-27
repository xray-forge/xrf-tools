/// What one update of the schedule did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AlifeScheduleReport {
  /// Offline objects it updated.
  pub updated: usize,
}
