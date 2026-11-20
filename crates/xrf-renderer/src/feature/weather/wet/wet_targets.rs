use xrf_renderer_core::GraphTexture;

/// What the wetting writes beside the G-buffer for the reflections and combine to read, as the frame's graph holds it:
/// the wet surface the enhanced patch writes, and the gloss the reflections' wet reach adds; a texel of nothing for
/// each while dry.
#[derive(Clone, Copy)]
pub(crate) struct WetTargets {
  pub surface: GraphTexture,
  pub reach: GraphTexture,
}
