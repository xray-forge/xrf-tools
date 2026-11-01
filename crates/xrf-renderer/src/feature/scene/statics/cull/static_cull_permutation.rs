use xrf_renderer_core::ShaderPermutation;

/// Which view the static cull fills: the camera's by default, else a shadow's, which is a light's face kept while
/// nothing it casts from changes (`is_finest`), the overhead map of the level's water (`is_water`), or a sun cascade
/// keeping its trees only where their shadows can fall in the camera's view (`is_received`).
#[derive(Clone, Copy, Debug, Default, ShaderPermutation)]
pub struct StaticCullPermutation {
  pub is_shadow: bool,
  pub is_finest: bool,
  pub is_water: bool,
  pub is_received: bool,
}

impl StaticCullPermutation {
  /// The module whose pipelines it makes.
  pub const MODULE: &'static str = "static/cull";
}
