use glam::Vec3;
use xrf_renderer_core::ProxyHandle;
use xrf_visual::LightDescription;

use crate::lighting::light_basis::LightBasis;
use crate::scene::level::shadow::shadow_sway::ShadowSway;
use crate::scene::static_scene::static_scene::StaticScene;

/// A shadowed light asking for its faces this frame: the light, how it stands, where the camera stands and looks,
/// whether a view sees it or it is only kept ready, and whether the asking view is its scene's first; and what its
/// faces are drawn of: the static scene, the sway, and how far the grass's cards cast, none where they cast none.
#[derive(Clone, Copy)]
pub struct LightShadowAsk<'a> {
  pub handle: ProxyHandle<LightDescription>,
  pub light: &'a LightDescription,
  pub basis: &'a LightBasis,
  pub eye: Vec3,
  pub forward: Vec3,
  pub is_in_view: bool,
  pub is_scene_first: bool,
  pub statics: &'a StaticScene,
  pub sway: &'a ShadowSway<'a>,
  pub grass_reach: Option<f32>,
}
