use xrf_renderer_core::ShaderPermutation;

/// How the static scene's surfaces are drawn: into a shadow's map, which leaves out the surfaces casting none, and with
/// the inspections' code compiled in.
#[derive(Clone, Copy, Debug, ShaderPermutation)]
pub(crate) struct StaticDrawPermutation {
  pub is_shadow_draw: bool,
  pub is_inspected: bool,
}

impl StaticDrawPermutation {
  /// The module whose pipelines it makes.
  pub(crate) const MODULE: &'static str = "static/gbuffer";
}
