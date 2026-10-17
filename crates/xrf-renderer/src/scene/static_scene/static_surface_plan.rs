use xrf_material::XraySurfaceTerrain;

use crate::scene::static_scene::static_class::StaticClass;
use crate::scene::texture::texture_role::TextureRole;

/// What a shader table entry decides of a static surface before any of its textures is asked for: its class, alpha
/// reference and flags, and the textures it names.
pub struct StaticSurfacePlan<'a> {
  pub class: StaticClass,
  pub alpha_reference: u8,
  pub flags: u32,
  /// The surface's own textures and the role each is asked for as, at `StaticSurface`'s slots from the base to the
  /// hemisphere; none where it names none.
  pub slots: [Option<(&'a str, TextureRole)>; 7],
  /// A terrain's layers and mask, where its base is named.
  pub terrain: Option<&'a XraySurfaceTerrain>,
  /// An environment-mapped model's cube.
  pub environment: Option<&'a str>,
  /// A blended level surface's own lightmap, bound in the detail slot.
  pub lightmap: Option<&'a str>,
  /// What a distorting composited surface draws into the distortion target.
  pub distortion: Option<&'a str>,
  pub detail_scale: f32,
  /// The lighting model its texture's descriptor gives it.
  pub material: f32,
}
