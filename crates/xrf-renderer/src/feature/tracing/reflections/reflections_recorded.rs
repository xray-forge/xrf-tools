use xrf_renderer_core::GraphTexture;

/// What the traced reflections hand on once recorded: the light reflected that combine reads (the history written, or
/// the trace itself where DLSS ray reconstruction takes it raw), why each ray ended, which the present's ends view
/// shows, and the trace, what each traced pixel met with the ray's length in alpha, which the guides read.
#[derive(Clone, Copy)]
pub struct ReflectionsRecorded {
  pub radiance: GraphTexture,
  pub ends: GraphTexture,
  pub trace: GraphTexture,
}
