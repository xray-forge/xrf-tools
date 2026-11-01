use glam::Vec4;

/// Where the level's surface stands: its centre in `x` and `z`, its half width and the height it is seen from; then its
/// texels across, the metres it reaches down, and the metres a cell of puddle sites is across.
#[derive(Clone, Copy, Debug)]
pub struct SurfaceWindow {
  pub window: Vec4,
  pub shape: Vec4,
}
