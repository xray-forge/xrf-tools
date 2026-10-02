use crate::camera::camera_view::CameraView;
use crate::scene::level::shadow_tile::ShadowTile;

/// One face of a light's shadow: its square of the atlas, the camera it is drawn through, and the sectors resident
/// when it was last drawn, none before it is.
#[derive(Clone, Copy, Debug)]
pub struct LightShadowFace {
  pub tile: ShadowTile,
  pub view: CameraView,
  pub drawn: Option<usize>,
}
