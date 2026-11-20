use xrf_animation_envelope::AnimationEnvelope;

/// One bone's channels in a skeleton motion, `st_BoneMotion`: its position and rotation keyed over time.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableBoneMotion {
  /// The bone it moves, by name as stored; the SDK matches it to the skeleton ignoring case. Version 4 stored none,
  /// so its bones are named by their index.
  pub name: String,
  pub flags: u8,
  /// Position `x`, `y`, `z`, then rotation heading, pitch and bank, `EChannelType`'s order.
  pub channels: [AnimationEnvelope; 6],
}

impl EditableBoneMotion {
  /// The bone keeps its own orientation in the world rather than turning with its parent, `flWorldOrient`.
  pub const WORLD_ORIENT_FLAG: u8 = 1 << 0;

  pub const POSITION_X: usize = 0;
  pub const POSITION_Y: usize = 1;
  pub const POSITION_Z: usize = 2;
  pub const ROTATION_HEADING: usize = 3;
  pub const ROTATION_PITCH: usize = 4;
  pub const ROTATION_BANK: usize = 5;

  /// Whether the bone keeps its world orientation, [`Self::WORLD_ORIENT_FLAG`].
  pub fn is_world_oriented(&self) -> bool {
    self.flags & Self::WORLD_ORIENT_FLAG != 0
  }
}
