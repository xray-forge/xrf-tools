use xrf_renderer_core::ShaderPermutation;

/// Whether the lamps' visibility and light read the contact shadows marched for them.
#[derive(Clone, Copy, Debug, ShaderPermutation)]
pub struct ContactMarchPermutation {
  pub contact_marched: bool,
}

impl ContactMarchPermutation {
  /// The modules whose pipelines it makes: the lamps' visibility, and their light.
  pub const VISIBILITY_MODULE: &'static str = "frame/light_visibility";
  pub const LIGHTS_MODULE: &'static str = "frame/lights";
}
