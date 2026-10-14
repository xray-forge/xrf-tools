use xrf_renderer_core::ShaderPermutation;

/// How the static scene's surfaces are drawn (`static/gbuffer`): into a shadow's map, which leaves out the surfaces
/// casting none, and with the inspections' code compiled in.
#[derive(Clone, Copy, Debug, ShaderPermutation)]
pub struct StaticDrawPermutation {
  pub is_shadow_draw: bool,
  pub is_inspected: bool,
}
