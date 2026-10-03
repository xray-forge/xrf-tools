/// What rain wets surfaces with, as `CBlender_rain` binds it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderWetSurfaces {
  /// `s_water`, a volume of rippling normals, a slice a moment.
  pub splash: String,
  /// `s_waterFall`, the normals of water running down.
  pub flow: String,
}
