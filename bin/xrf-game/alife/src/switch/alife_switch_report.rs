/// What one sweep of the switch did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AlifeSwitchReport {
  /// Objects of the level it weighed.
  pub swept: usize,
  /// Objects it brought online, and took offline, children aside.
  pub went_online: usize,
  pub went_offline: usize,
}
