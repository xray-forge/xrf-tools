use xrf_renderer_core::{GraphTexture, PassParameters};

/// What reducing one level of a bake's three maps into the next reads and writes: each map's level, and the next.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct ImpostorReduceParameters {
  #[binding(7)]
  #[texture(d2, unfilterable)]
  pub colour_source: GraphTexture,
  #[texture(d2, unfilterable)]
  pub normal_source: GraphTexture,
  #[texture(d2, unfilterable)]
  pub material_source: GraphTexture,
  #[binding(4)]
  #[storage_texture(d2, rgba8unorm, write)]
  pub colour_level: GraphTexture,
  #[storage_texture(d2, rgba8unorm, write)]
  pub normal_level: GraphTexture,
  #[storage_texture(d2, rgba8unorm, write)]
  pub material_level: GraphTexture,
}
