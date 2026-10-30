use xrf_renderer_core::{GraphTexture, PassParameters};

/// What packing a tree type's baked frames reads and writes: the G-buffer the frames were drawn into, and the first
/// level of the bake's three maps (`ImpostorAtlas`'s layout, before compression).
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct ImpostorBakeParameters {
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub albedo_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub normal_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub material_target: GraphTexture,
  #[storage_texture(d2, rgba8unorm, write)]
  pub colour_level: GraphTexture,
  #[storage_texture(d2, rgba8unorm, write)]
  pub normal_level: GraphTexture,
  #[storage_texture(d2, rgba8unorm, write)]
  pub material_level: GraphTexture,
}
