use xrf_renderer_core::{GraphTexture, PassParameters};

/// Where the reflections' trace writes under ray budgets, as a compute pass: what each pixel reflected and the ray's
/// length, and how its ray ended.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 4)]
pub struct ReflectionTraceOutParameters {
  #[storage_texture(d2, rgba16float, write)]
  pub traced_out: GraphTexture,
  #[storage_texture(d2, r32float, write)]
  pub ends_out: GraphTexture,
}
