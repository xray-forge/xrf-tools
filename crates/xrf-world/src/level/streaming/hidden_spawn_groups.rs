/// The spawn groups a scene leaves out, a bit a group from the lowest. Group zero, the objects no group names, always
/// stands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HiddenSpawnGroups {
  bits: u32,
}

impl HiddenSpawnGroups {
  /// These and `group` besides.
  #[must_use]
  pub const fn with(self, group: u32) -> Self {
    Self {
      bits: self.bits | Self::get_bit(group),
    }
  }

  /// Whether a group's objects stand in the scene.
  pub const fn is_shown(self, group: u32) -> bool {
    self.bits & Self::get_bit(group) == 0
  }

  /// The bit a group takes, none for group zero.
  const fn get_bit(group: u32) -> u32 {
    match group {
      0 => 0,
      group => 1 << (group - 1),
    }
  }
}
