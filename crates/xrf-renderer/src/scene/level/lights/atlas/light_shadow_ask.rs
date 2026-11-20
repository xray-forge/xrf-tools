use glam::Vec3;
use xrf_renderer_core::ProxyHandle;
use xrf_visual::LightDescription;

use crate::feature::scene::drawable::casters::light_casters::LightCasters;
use crate::lighting::lamp::light_basis::LightBasis;
use crate::scene::level::shadow::shadow_sway::ShadowSway;
use crate::scene::static_scene::static_scene::StaticScene;

/// A shadowed light asking for its faces this frame: the light, how it stands, where the camera stands and looks,
/// whether a view sees it or it is only kept ready, and whether the asking view drives its scene; and what its faces are
/// drawn of: the static scene, the sway, and what the drawable kinds cast into them.
#[derive(Clone, Copy)]
pub(crate) struct LightShadowAsk<'a> {
  pub handle: ProxyHandle<LightDescription>,
  pub light: &'a LightDescription,
  pub basis: &'a LightBasis,
  pub eye: Vec3,
  pub forward: Vec3,
  pub is_in_view: bool,
  /// Whether the asking view is its scene's driving one, whose casters the faces are drawn with.
  pub is_driving: bool,
  pub statics: &'a StaticScene,
  pub sway: &'a ShadowSway<'a>,
  pub casters: &'a [LightCasters],
}
