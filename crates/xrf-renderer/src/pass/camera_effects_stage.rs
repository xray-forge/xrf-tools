/// The camera effects' stages, by the pipeline each draws with.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CameraEffectsStage {
  /// The distance and the motion under each shown pixel, read from the drawn frame.
  Inputs,
  /// The longest motion over each block of eight pixels.
  MotionReduce,
  /// The longest motion over each tile, from the blocks.
  MotionTiles,
  /// The longest motion over each tile and its neighbours.
  MotionNeighbours,
  /// The frame blurred along its motion.
  MotionBlur,
  /// The frame at half its size, with each texel's circle of confusion.
  FocusReduce,
  /// The nearest and the farthest circle over each tile.
  FocusTiles,
  /// The tiles' circles spread over the tiles they reach.
  FocusDilate,
  /// The half-size frame gathered over its circles, near and far.
  FocusGather,
  /// The gathered frame laid over the sharp one by each pixel's circle.
  FocusCombine,
  /// The lens's colour split and falloff.
  Lens,
}

impl CameraEffectsStage {
  pub const ALL: [Self; 11] = [
    Self::Inputs,
    Self::MotionReduce,
    Self::MotionTiles,
    Self::MotionNeighbours,
    Self::MotionBlur,
    Self::FocusReduce,
    Self::FocusTiles,
    Self::FocusDilate,
    Self::FocusGather,
    Self::FocusCombine,
    Self::Lens,
  ];
  pub const COUNT: usize = Self::ALL.len();

  /// Its place among the stages, which its pipeline is kept at.
  pub const fn get_index(self) -> usize {
    self as usize
  }

  /// The pass it is added as, every one starting with `camera` (`JitterGuard::ALLOWED`).
  pub const fn get_pass(self) -> &'static str {
    match self {
      Self::Inputs => "camera inputs",
      Self::MotionReduce => "camera motion reduce",
      Self::MotionTiles => "camera motion tiles",
      Self::MotionNeighbours => "camera motion neighbours",
      Self::MotionBlur => "camera motion blur",
      Self::FocusReduce => "camera focus reduce",
      Self::FocusTiles => "camera focus tiles",
      Self::FocusDilate => "camera focus dilate",
      Self::FocusGather => "camera focus gather",
      Self::FocusCombine => "camera focus combine",
      Self::Lens => "camera lens",
    }
  }

  /// Its fragment entry in `frame/camera_effects`.
  pub const fn get_fragment(self) -> &'static str {
    match self {
      Self::Inputs => "fs_inputs",
      Self::MotionReduce => "fs_motion_reduce",
      Self::MotionTiles => "fs_motion_tiles",
      Self::MotionNeighbours => "fs_motion_neighbours",
      Self::MotionBlur => "fs_motion_blur",
      Self::FocusReduce => "fs_focus_reduce",
      Self::FocusTiles => "fs_focus_tiles",
      Self::FocusDilate => "fs_focus_dilate",
      Self::FocusGather => "fs_focus_gather",
      Self::FocusCombine => "fs_focus_combine",
      Self::Lens => "fs_lens",
    }
  }
}
