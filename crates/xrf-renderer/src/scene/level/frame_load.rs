/// What a frame already draws of the work drawn once in so many frames, so work that may wait lands on a frame light on
/// the rest rather than on one already heavy: a frame drawing the sun's third or fourth cascade on its staggered turn is
/// heavy, and every other frame takes one job that waited, so two such jobs due together draw a frame apart.
pub struct FrameLoad {
  /// Whether a job that waited may still draw this frame: a light one no job took yet.
  is_free: bool,
  /// Whether a job that waited as long as it may drew this frame.
  is_forced: bool,
}

impl FrameLoad {
  /// Frames a job waits for a light frame at most, one turn of the farthest cascade's stagger; past it a job draws
  /// on a heavy frame too, one a frame.
  pub const WAIT_MOST: u32 = 8;

  pub fn new(is_heavy: bool) -> Self {
    Self {
      is_free: !is_heavy,
      is_forced: false,
    }
  }

  /// Whether a job that waited `waited` frames draws this frame, taking the frame's place where it does.
  pub fn take(&mut self, waited: u32) -> bool {
    if self.is_free {
      self.is_free = false;

      true
    } else if waited >= Self::WAIT_MOST && !self.is_forced {
      self.is_forced = true;

      true
    } else {
      false
    }
  }
}
