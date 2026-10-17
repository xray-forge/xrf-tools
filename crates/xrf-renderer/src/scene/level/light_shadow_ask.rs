use glam::Vec3;
use xrf_renderer_core::ProxyHandle;
use xrf_visual::LightDescription;

use crate::lighting::light_basis::LightBasis;

/// A shadowed light asking for its faces this frame: the light, how it stands, where the camera stands and looks,
/// whether a view sees it or it is only kept ready, and whether the asking view is its scene's first.
#[derive(Clone, Copy)]
pub struct LightShadowAsk<'a> {
  pub handle: ProxyHandle<LightDescription>,
  pub light: &'a LightDescription,
  pub basis: &'a LightBasis,
  pub eye: Vec3,
  pub forward: Vec3,
  pub is_in_view: bool,
  pub is_scene_first: bool,
}
