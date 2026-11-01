use crate::camera::camera_view::CameraView;
use crate::feature::scene::drawable::drawable_kinds::DrawableKinds;
use crate::scene::level::lights::atlas::shadow_tile::ShadowTile;

/// One face of a light's shadow: its square of the atlas, the camera it is drawn through, and what the scene held
/// when it was last drawn, none before it is.
#[derive(Clone, Copy, Debug)]
pub struct LightShadowFace {
  pub tile: ShadowTile,
  pub view: CameraView,
  pub drawn: Option<usize>,
  /// What the scene held when its static tile was last drawn, none before it was or since it went stale.
  pub drawn_static: Option<usize>,
  /// The sway's time it was last drawn at.
  pub drawn_at: f32,
  /// Frames it has waited for a draw it wants.
  pub waited: u32,
  /// The drawable kinds whose casters were drawn into it, which its light casting another set draws it again.
  pub casting: DrawableKinds,
}
