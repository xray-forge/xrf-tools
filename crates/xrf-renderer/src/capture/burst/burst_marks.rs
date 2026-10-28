/// A burst frame's G-buffer marks: each texel's byte of the marks the material target holds in its alpha
/// (`shaders/common/marks.wgsl`), rows top to bottom at the size the scene was drawn at, which a temporal upscale draws
/// smaller than the frame.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BurstMarks {
  pub width: u32,
  pub height: u32,
  pub marks: Vec<u8>,
}

impl BurstMarks {
  /// What is a plant: grass and leaves, however lit.
  pub const PLANT: u8 = 8;
  /// What moved by itself since the last frame, swayed by the wind.
  pub const MOVED: u8 = 2;
  /// What is grass.
  pub const GRASS: u8 = 1;
}
