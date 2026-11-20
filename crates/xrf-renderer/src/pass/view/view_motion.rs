use glam::Mat4;

/// The view projections a frame measures motion and reprojects by: this frame's unjittered one and the last frame's,
/// which every surface's motion is measured between, and the last frame's jittered one, which a read of its targets
/// reprojects through. The last frame's are this frame's own on a first frame and on a cut.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ViewMotion {
  pub current: Mat4,
  pub previous: Mat4,
  pub jittered_previous: Mat4,
}

impl ViewMotion {
  /// No motion: a camera standing where `current` and `jittered` put it on its last frame too.
  pub(crate) fn still(current: Mat4, jittered: Mat4) -> Self {
    Self {
      current,
      previous: current,
      jittered_previous: jittered,
    }
  }
}
